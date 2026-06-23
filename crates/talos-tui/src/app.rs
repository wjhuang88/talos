use std::io;
use std::pin::Pin;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    style::Color as CColor,
    terminal::enable_raw_mode,
};
use futures::{Stream, StreamExt};
use talos_conversation::{CopyScope, MessageSource, TipKind, UiOutput, UserInput};
use talos_core::ApprovalChoice;
use talos_core::message::Message;
use talos_core::tool_filter::ToolSyntaxFilter;
use tokio::sync::mpsc;

use crate::evolution::{self, EvolutionPanel};
use crate::highlight::HighlightEngine;
use crate::inline_terminal::{
    ComponentStack, HistoryAttrs, HistorySegment, InlineTerminal, ViewportComponent,
};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, PanelAction, Tip, TuiState};
use crate::stream_markdown::{BlockDecision, HoldStatus, MarkdownBlockKind, StreamBlockClassifier};
use crate::theme::{semantic, to_crossterm_color};

pub(crate) const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Clone, Debug, Eq)]
pub(crate) struct ScrollbackLine {
    pub(crate) text: String,
    segments: Vec<HistorySegment>,
    bg: Option<CColor>,
    fill: Option<HistorySegment>,
}

impl PartialEq for ScrollbackLine {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.bg == other.bg && self.fill == other.fill
    }
}

impl ScrollbackLine {
    pub(crate) fn plain(text: impl Into<String>, bg: Option<CColor>) -> Self {
        let text = text.into();
        Self {
            segments: vec![HistorySegment::raw(text.clone())],
            text,
            bg,
            fill: None,
        }
    }

    pub(crate) fn styled(segments: Vec<HistorySegment>, bg: Option<CColor>) -> Self {
        Self::styled_with_fill(segments, bg, None)
    }

    pub(crate) fn styled_with_fill(
        segments: Vec<HistorySegment>,
        bg: Option<CColor>,
        fill: Option<HistorySegment>,
    ) -> Self {
        let text = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>();
        Self {
            text,
            segments,
            bg,
            fill,
        }
    }

    fn has_plain_segments_only(&self) -> bool {
        if self.fill.is_some() {
            return false;
        }
        self.segments
            .iter()
            .all(|segment| segment.fg.is_none() && segment.attrs == HistoryAttrs::default())
    }
}

#[derive(Default)]
pub(crate) struct StreamRenderState {
    source: Option<MessageSource>,
    line_count: usize,
    buffer: String,
    preview: String,
    hold_complete_lines: bool,
    held_lines: Vec<(usize, String)>,
    block_classifier: StreamBlockClassifier,
    hold_status: Option<HoldStatus>,
    highlight_engine: HighlightEngine,
}

impl StreamRenderState {
    pub(crate) fn start(&mut self, source: MessageSource) -> Vec<ScrollbackLine> {
        self.start_with_hold(source, false)
    }

    pub(crate) fn start_with_hold(
        &mut self,
        source: MessageSource,
        hold_complete_lines: bool,
    ) -> Vec<ScrollbackLine> {
        let bg = crate::scrollback::stream_bg_for(Some(&source));
        self.source = Some(source);
        self.line_count = 0;
        self.buffer.clear();
        self.preview.clear();
        self.hold_complete_lines = hold_complete_lines;
        self.held_lines.clear();
        self.block_classifier.reset();
        self.hold_status = None;

        if bg.is_some() {
            vec![ScrollbackLine::plain(String::new(), bg)]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn source(&self) -> Option<&MessageSource> {
        self.source.as_ref()
    }

    pub(crate) fn preview(&self) -> &str {
        &self.preview
    }

    pub(crate) fn hold_status(&self) -> Option<&HoldStatus> {
        self.hold_status.as_ref()
    }

    pub(crate) fn push_chunk(&mut self, chunk: &str) -> Vec<ScrollbackLine> {
        self.buffer.push_str(chunk);
        let mut lines = Vec::new();

        while let Some(pos) = self.buffer.find('\n') {
            let line = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 1..].to_string();
            if self.hold_complete_lines {
                self.held_lines.push((self.line_count, line));
            } else {
                lines.extend(self.push_complete_line(line));
            }
        }

        if self.hold_status.is_none() {
            self.preview = self.buffer.clone();
        }
        lines
    }

    pub(crate) fn finish(&mut self) -> Vec<ScrollbackLine> {
        let mut lines = Vec::new();

        let held_lines = std::mem::take(&mut self.held_lines);
        for (_, line) in held_lines {
            lines.push(self.render_next_line(&line));
        }

        let decisions = self.block_classifier.finish();
        lines.extend(self.apply_block_decisions(decisions));

        if !self.preview.is_empty() {
            let preview = std::mem::take(&mut self.preview);
            lines.push(self.render_next_line(&preview));
        }

        if self.bg().is_some() {
            lines.push(ScrollbackLine::plain(String::new(), self.bg()));
        }

        self.reset();
        lines
    }

    fn render_line(
        &self,
        line_index: usize,
        line: &str,
        block: Option<(&MarkdownBlockKind, usize)>,
    ) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            crate::scrollback::prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        if block.is_none() && crate::scrollback::is_horizontal_rule(line) {
            let fill = crate::scrollback::horizontal_rule_segment("─");
            segments.push(fill.clone());
            return ScrollbackLine::styled_with_fill(segments, self.bg(), Some(fill));
        }
        segments.extend(crate::scrollback::render_markdown_segments(line, block));
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_segments_line(
        &self,
        line_index: usize,
        content_segments: Vec<HistorySegment>,
    ) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            crate::scrollback::prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        segments.extend(content_segments);
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_plain_line(&self, line_index: usize, line: &str) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let segments = vec![
            HistorySegment::styled(
                padding,
                crate::scrollback::prefix_color_for(self.source(), line_index),
                HistoryAttrs {
                    bold: line_index == 0 && self.source().is_some(),
                    ..HistoryAttrs::default()
                },
            ),
            HistorySegment::raw(line),
        ];
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_block_line(
        &mut self,
        line: &str,
        kind: &MarkdownBlockKind,
        block_line_index: usize,
    ) -> ScrollbackLine {
        let rendered = self.render_line(self.line_count, line, Some((kind, block_line_index)));
        self.line_count += 1;
        rendered
    }

    fn render_next_line(&mut self, line: &str) -> ScrollbackLine {
        let rendered = if self.markdown_enabled() {
            self.render_line(self.line_count, line, None)
        } else {
            self.render_plain_line(self.line_count, line)
        };
        self.line_count += 1;
        rendered
    }

    fn render_block_lines(
        &mut self,
        kind: &MarkdownBlockKind,
        block_lines: Vec<String>,
    ) -> Vec<ScrollbackLine> {
        if kind == &MarkdownBlockKind::Table {
            return self.render_table_lines(block_lines);
        }
        if kind == &MarkdownBlockKind::CodeFence {
            let bg_source = self.source().cloned();
            let bg = crate::scrollback::stream_bg_for(bg_source.as_ref());

            if block_lines.len() >= 3 {
                let opening = &block_lines[0];
                let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();
                if lang == "mermaid" {
                    let code_lines = &block_lines[1..block_lines.len() - 1];
                    let mermaid_src = code_lines.join("\n");
                    return crate::scrollback::render_mermaid_block(&mermaid_src, bg);
                }
            }

            if let Some(rendered) =
                Self::try_highlight_code_block(&mut self.highlight_engine, &block_lines, bg_source)
            {
                return rendered;
            }
            return crate::scrollback::render_code_block(&block_lines, bg);
        }

        let mut rendered = Vec::with_capacity(block_lines.len());
        for (block_line_index, line) in block_lines.into_iter().enumerate() {
            rendered.push(self.render_block_line(&line, kind, block_line_index));
        }
        rendered
    }

    fn try_highlight_code_block(
        engine: &mut HighlightEngine,
        block_lines: &[String],
        source: Option<MessageSource>,
    ) -> Option<Vec<ScrollbackLine>> {
        if block_lines.len() < 3 {
            return None;
        }

        let opening = &block_lines[0];
        let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();

        if lang.is_empty() || !engine.supports(lang) {
            return None;
        }

        let code_lines = &block_lines[1..block_lines.len() - 1];
        let code = code_lines.join("\n");
        let highlighted_lines = engine.highlight(lang, &code)?;

        Some(crate::scrollback::build_code_block(
            &highlighted_lines,
            lang,
            crate::scrollback::stream_bg_for(source.as_ref()),
        ))
    }

    fn render_table_lines(&mut self, block_lines: Vec<String>) -> Vec<ScrollbackLine> {
        let table_lines =
            crate::scrollback::render_table_block(&block_lines).unwrap_or_else(|| {
                block_lines
                    .into_iter()
                    .enumerate()
                    .map(|(row_index, line)| {
                        crate::scrollback::render_table_history_line(&line, row_index)
                    })
                    .collect()
            });
        let mut rendered = Vec::with_capacity(table_lines.len());
        for content_segments in table_lines {
            let line = self.render_segments_line(self.line_count, content_segments);
            self.line_count += 1;
            rendered.push(line);
        }
        rendered
    }

    fn push_complete_line(&mut self, line: String) -> Vec<ScrollbackLine> {
        if !self.markdown_enabled() {
            return vec![self.render_next_line(&line)];
        }
        let decisions = self.block_classifier.push_line(line);
        self.apply_block_decisions(decisions)
    }

    fn markdown_enabled(&self) -> bool {
        !matches!(self.source(), Some(MessageSource::User))
    }

    fn apply_block_decisions(&mut self, decisions: Vec<BlockDecision>) -> Vec<ScrollbackLine> {
        let mut lines = Vec::new();
        for decision in decisions {
            match decision {
                BlockDecision::ImmediateLine(line) => {
                    self.hold_status = None;
                    if self.buffer.is_empty() {
                        self.preview.clear();
                    }
                    lines.push(self.render_next_line(&line));
                }
                BlockDecision::StartHold { status } | BlockDecision::ContinueHold { status } => {
                    self.preview = status.preview_text().to_string();
                    self.hold_status = Some(status);
                }
                BlockDecision::FinishHold {
                    status: _,
                    kind,
                    lines: rendered,
                } => {
                    self.hold_status = None;
                    self.preview = self.buffer.clone();
                    lines.extend(self.render_block_lines(&kind, rendered));
                }
                BlockDecision::FallbackImmediate {
                    status: _,
                    kind,
                    reason: _,
                    lines: rendered,
                } => {
                    self.hold_status = None;
                    self.preview = self.buffer.clone();
                    lines.extend(self.render_block_lines(&kind, rendered));
                }
            }
        }
        lines
    }

    fn bg(&self) -> Option<CColor> {
        crate::scrollback::stream_bg_for(self.source())
    }

    pub(crate) fn reset(&mut self) {
        self.source = None;
        self.line_count = 0;
        self.buffer.clear();
        self.preview.clear();
        self.hold_complete_lines = false;
        self.held_lines.clear();
        self.block_classifier.reset();
        self.hold_status = None;
    }
}

pub struct Tui {
    state: TuiState,
    terminal: InlineTerminal,
    skill_sidebar: SkillSidebar,
    evolution_panel: EvolutionPanel,
    ui_output_rx: Option<mpsc::UnboundedReceiver<UiOutput>>,
    user_input_tx: Option<mpsc::UnboundedSender<UserInput>>,
    pending_scrollback: Vec<ScrollbackLine>,
    queued_outputs: Vec<UiOutput>,
    active_stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    stream_render: StreamRenderState,
    stream_opening_pending: bool,
    pending_stream_opening: Vec<ScrollbackLine>,
    text_filter: ToolSyntaxFilter,
    processing_frame: usize,
    processing_tick: usize,
    stream_count: usize,
    last_total_height: u16,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let _ = crossterm::terminal::disable_raw_mode();

        crate::splash::print_splash_scrollback();

        let (_, cursor_y) = crossterm::cursor::position()?;
        let (_, screen_h) = crossterm::terminal::size()?;
        let viewport_height: u16 = 6;
        if cursor_y.saturating_add(viewport_height) > screen_h {
            for _ in 0..viewport_height.saturating_sub(1) {
                println!();
            }
        }

        enable_raw_mode()?;
        let terminal = InlineTerminal::new()?;

        Ok(Self {
            state: TuiState::new(),
            terminal,
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: EvolutionPanel::new(),
            ui_output_rx: None,
            user_input_tx: None,
            pending_scrollback: Vec::new(),
            queued_outputs: Vec::new(),
            active_stream: None,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            text_filter: ToolSyntaxFilter::new(),
            processing_frame: 0,
            processing_tick: 0,
            stream_count: 0,
            last_total_height: 0,
        })
    }

    pub fn set_ui_output_rx(&mut self, rx: mpsc::UnboundedReceiver<UiOutput>) {
        self.ui_output_rx = Some(rx);
    }

    pub fn set_user_input_tx(&mut self, tx: mpsc::UnboundedSender<UserInput>) {
        self.user_input_tx = Some(tx);
    }

    pub fn set_model_name(&mut self, name: String) {
        self.state.status.model_name = name;
    }

    pub fn hydrate_history(&mut self, history: &[Message]) {
        use talos_conversation::ToolCallDisplay;
        use talos_core::tool::ToolProvenance;

        let mut pending_tool_names: Vec<String> = Vec::new();

        for message in history {
            match message {
                Message::Tool { result } => {
                    let tool_name = if !pending_tool_names.is_empty() {
                        pending_tool_names.remove(0)
                    } else {
                        result.tool_use_id.clone()
                    };
                    let content = crate::scrollback::strip_llm_hints(&result.content);
                    self.handle_ui_output(UiOutput::ToolResult(
                        talos_conversation::ToolResultDisplay {
                            tool_name: Some(tool_name),
                            is_error: result.is_error,
                            content,
                        },
                    ));
                }
                Message::Assistant { content, tool_calls } => {
                    let tool_calls_in_text =
                        talos_core::message::extract_tool_calls_from_text(content);
                    let cleaned = talos_core::message::strip_tool_syntax(content);
                    let has_tool_calls = !tool_calls.is_empty() || !tool_calls_in_text.is_empty();

                    pending_tool_names.clear();
                    for tc in tool_calls {
                        pending_tool_names.push(tc.name.clone());
                    }

                    if !has_tool_calls && !cleaned.is_empty() {
                        let stream = futures::stream::iter(vec![cleaned]);
                        let msg = talos_conversation::StreamMessage {
                            source: talos_conversation::MessageSource::Assistant,
                            stream: Box::pin(stream),
                        };
                        self.handle_ui_output(UiOutput::Stream(msg));
                        self.consume_stream_completely();
                        self.finalize_active_stream();
                    }

                    let calls: Vec<ToolCallDisplay> = if !tool_calls.is_empty() {
                        tool_calls.iter().map(|tc| ToolCallDisplay {
                            tool_name: tc.name.clone(), arguments: tc.input.clone(),
                            provenance: ToolProvenance::Native,
                            summary_fields: crate::scrollback::summary_fields_for(&tc.name),
                        }).collect()
                    } else if !tool_calls_in_text.is_empty() {
                        for tc in &tool_calls_in_text {
                            pending_tool_names.push(tc.name.clone());
                        }
                        tool_calls_in_text.iter().map(|tc| ToolCallDisplay {
                            tool_name: tc.name.clone(), arguments: tc.input.clone(),
                            provenance: ToolProvenance::Native,
                            summary_fields: crate::scrollback::summary_fields_for(&tc.name),
                        }).collect()
                    } else { vec![] };

                    for call in &calls {
                        self.handle_ui_output(UiOutput::ToolCall(call.clone()));
                    }
                }
                Message::User { content } => {
                    let stream = futures::stream::iter(vec![content.clone()]);
                    let msg = talos_conversation::StreamMessage {
                        source: talos_conversation::MessageSource::User,
                        stream: Box::pin(stream),
                    };
                    self.handle_ui_output(UiOutput::Stream(msg));
                    self.consume_stream_completely();
                    self.finalize_active_stream();
                }
                _ => {}
            }
        }
    }

    fn consume_stream_completely(&mut self) {
        while let Some(ref mut stream) = self.active_stream {
            match stream.as_mut().poll_next(&mut std::task::Context::from_waker(
                futures::task::noop_waker_ref(),
            )) {
                std::task::Poll::Ready(Some(chunk)) => {
                    self.consume_stream_chunk(&chunk);
                }
                std::task::Poll::Ready(None) => {
                    self.active_stream = None;
                    break;
                }
                std::task::Poll::Pending => break,
            }
        }
    }

    pub fn toggle_skill_sidebar(&mut self) {
        self.skill_sidebar.toggle();
    }

    pub fn toggle_evolution_panel(&mut self) {
        self.evolution_panel.toggle();
    }

    pub fn update_evolution_patterns(&mut self, patterns: Vec<evolution::PatternInfo>) {
        self.evolution_panel.update_patterns(patterns);
    }

    pub fn update_skills(&mut self, skills: Vec<SkillInfo>) {
        self.skill_sidebar.update_skills(skills);
    }

    pub fn approval_choice(&self) -> Option<&ApprovalChoice> {
        match &self.state.approval_state {
            ApprovalState::Visible { selected, .. } => Some(selected),
            ApprovalState::Hidden => None,
        }
    }

    pub fn handle_approval_key(&mut self, key: char) -> Option<ApprovalChoice> {
        let ApprovalState::Visible { selected, .. } = &mut self.state.approval_state else {
            return None;
        };

        match key {
            'y' => {
                *selected = ApprovalChoice::ApproveOnce;
                Some(ApprovalChoice::ApproveOnce)
            }
            'a' => {
                *selected = ApprovalChoice::AlwaysApprove;
                Some(ApprovalChoice::AlwaysApprove)
            }
            'n' => {
                *selected = ApprovalChoice::Deny;
                Some(ApprovalChoice::Deny)
            }
            _ => None,
        }
    }

    pub fn show_approval(&mut self, tool_name: &str, arguments: &str) {
        self.state.activate_approval(tool_name, arguments);
        self.state.slash_menu = crate::state::BottomPanelState::open_approval(tool_name, arguments);
    }

    pub fn hide_approval(&mut self) {
        self.state.approval_state = ApprovalState::Hidden;
    }

    fn handle_pending_approval_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                self.state.slash_menu.select_prev("");
            }
            KeyCode::Down => {
                self.state.slash_menu.select_next("");
            }
            KeyCode::Enter => {
                let idx = self.state.slash_menu.selected_index;
                let choice = match idx {
                    0 => ApprovalChoice::ApproveOnce,
                    1 => ApprovalChoice::AlwaysApprove,
                    _ => ApprovalChoice::Deny,
                };
                self.resolve_approval(choice);
            }
            KeyCode::Char(c) => {
                if let Some(choice) = self.handle_approval_key(c) {
                    self.resolve_approval(choice);
                }
            }
            _ => {}
        }
    }

    fn resolve_approval(&mut self, choice: ApprovalChoice) {
        let (icon, color, msg) = match &choice {
            ApprovalChoice::ApproveOnce => {
                ("\u{2713}", to_crossterm_color(semantic::TEXT_SUCCESS), "approved")
            }
            ApprovalChoice::AlwaysApprove => (
                "\u{2713}",
                to_crossterm_color(semantic::TEXT_SUCCESS),
                "always approved",
            ),
            ApprovalChoice::Deny => ("\u{2717}", to_crossterm_color(semantic::TEXT_ERROR), "denied"),
        };
        self.pending_scrollback.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("   {icon} {msg}"),
                color,
                HistoryAttrs::default(),
            )],
            None,
        ));
        let _ = self.flush_pending_scrollback();

        if let Some(response_tx) = self.state.pending_approval_response.take() {
            let _ = response_tx.send(choice);
        }
        self.hide_approval();
        self.state.slash_menu.close();
        self.state.tip = Some(Tip {
            kind: TipKind::ApprovalResult,
            text: format!("Tool call {msg}"),
            ttl: Duration::from_secs(2),
            created_at: Instant::now(),
        });
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let session_start = Instant::now();
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));
        let mut ui_output_rx = self.ui_output_rx.take().expect("ui_output_rx not set");

        self.draw_frame()?;

        loop {
            self.state.expire_tip();
            self.flush_pending_scrollback()?;
            self.draw_frame()?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                    if matches!(self.state.approval_state, ApprovalState::Hidden) && !self.queued_outputs.is_empty() {
                        while !self.queued_outputs.is_empty()
                            && matches!(self.state.approval_state, ApprovalState::Hidden)
                        {
                            let output = self.queued_outputs.remove(0);
                            let is_tool = matches!(&output, UiOutput::ToolCall(_) | UiOutput::ToolApprovalRequest { .. });
                            if self.handle_ui_output(output) {
                                self.state.should_exit = true;
                                break;
                            }
                            if is_tool {
                                self.flush_pending_scrollback()?;
                                self.draw_frame()?;
                            }
                        }
                    }
                }
                Some(output) = ui_output_rx.recv() => {
                    if !matches!(self.state.approval_state, ApprovalState::Hidden) {
                        self.queued_outputs.push(output);
                    } else {
                        let is_tool = matches!(&output, UiOutput::ToolCall(_) | UiOutput::ToolApprovalRequest { .. });
                        if self.handle_ui_output(output) {
                            break;
                        }
                        if is_tool {
                            self.flush_pending_scrollback()?;
                            self.draw_frame()?;
                        }
                    }
                }
                Some(chunk) = self.next_stream_chunk() => {
                    self.consume_stream_chunk(&chunk);
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        let elapsed = session_start.elapsed();
        self.print_exit_summary(elapsed);

        self.restore();
        Ok(())
    }

    fn print_exit_summary(&mut self, elapsed: Duration) {
        let status = &self.state.status;
        let elapsed_secs = elapsed.as_secs();
        let usage = &status.usage;
        let total_tokens = (usage.input_tokens + usage.output_tokens) as u64;

        let mut lines = vec![ScrollbackLine::plain(String::new(), None)];

        let header_sep = "─".repeat(32);
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("⬡ Talos session complete {header_sep}"),
                to_crossterm_color(semantic::TEXT_ACCENT),
                HistoryAttrs::default(),
            )],
            None,
        ));

        lines.push(ScrollbackLine::plain(String::new(), None));

        if !status.model_name.is_empty() {
            lines.push(ScrollbackLine::styled(
                vec![
                    HistorySegment::styled(
                        format!("  {}  ", status.model_name),
                        to_crossterm_color(semantic::TEXT_ACCENT),
                        HistoryAttrs::default(),
                    ),
                    HistorySegment::styled(
                        format!(
                            "{}  ",
                            crate::formatting::format_duration(elapsed_secs)
                        ),
                        to_crossterm_color(semantic::STATUS_VALUE),
                        HistoryAttrs::default(),
                    ),
                    HistorySegment::styled(
                        format!("{} turns", self.stream_count),
                        to_crossterm_color(semantic::DIM_TEXT),
                        HistoryAttrs::default(),
                    ),
                ],
                None,
            ));
        } else {
            lines.push(ScrollbackLine::styled(
                vec![
                    HistorySegment::styled(
                        format!(
                            "{}  ",
                            crate::formatting::format_duration(elapsed_secs)
                        ),
                        to_crossterm_color(semantic::STATUS_VALUE),
                        HistoryAttrs::default(),
                    ),
                    HistorySegment::styled(
                        format!("{} turns", self.stream_count),
                        to_crossterm_color(semantic::DIM_TEXT),
                        HistoryAttrs::default(),
                    ),
                ],
                None,
            ));
        }

        if usage.input_tokens > 0 || usage.output_tokens > 0 {
            lines.push(ScrollbackLine::plain(String::new(), None));
            lines.push(ScrollbackLine::styled(
                vec![
                    HistorySegment::styled(
                        format!(
                            "  {} tokens in",
                            crate::formatting::format_tokens(usage.input_tokens as u64)
                        ),
                        to_crossterm_color(semantic::STATUS_VALUE),
                        HistoryAttrs::default(),
                    ),
                    HistorySegment::styled(
                        format!(
                            "      {} tokens out",
                            crate::formatting::format_tokens(usage.output_tokens as u64)
                        ),
                        to_crossterm_color(semantic::STATUS_VALUE),
                        HistoryAttrs::default(),
                    ),
                ],
                None,
            ));
            lines.push(ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("  {} tokens total", crate::formatting::format_tokens(total_tokens)),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                )],
                None,
            ));
        }

        if let Some(cost) = self.estimate_cost(usage) {
            lines.push(ScrollbackLine::plain(String::new(), None));
            lines.push(ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("  Est cost: ${cost:.2}"),
                    to_crossterm_color(semantic::TEXT_ACCENT),
                    HistoryAttrs::default(),
                )],
                None,
            ));
        }

        lines.push(ScrollbackLine::plain(String::new(), None));

        for line in lines {
            let _ = self.terminal.insert_history(&line.text, line.bg);
        }
    }

    fn estimate_cost(&self, usage: &talos_core::message::Usage) -> Option<f64> {
        if usage.input_tokens == 0 && usage.output_tokens == 0 {
            return None;
        }
        let input_cost = usage.input_tokens as f64 * 3.0 / 1_000_000.0;
        let output_cost = usage.output_tokens as f64 * 15.0 / 1_000_000.0;
        Some(input_cost + output_cost)
    }

    async fn next_stream_chunk(&mut self) -> Option<String> {
        match self.active_stream.as_mut() {
            Some(stream) => {
                let chunk = stream.next().await;
                if chunk.is_none() {
                    self.finalize_active_stream();
                }
                chunk
            }
            None => std::future::pending().await,
        }
    }

    fn finalize_active_stream(&mut self) {
        let lines = self.stream_render.finish();
        if self.stream_opening_pending {
            self.stream_opening_pending = false;
            self.pending_stream_opening.clear();
        } else {
            self.pending_scrollback.extend(lines);
        }
        self.active_stream = None;
    }

    fn consume_stream_chunk(&mut self, chunk: &str) {
        let filter_out = self.text_filter.push_chunk(chunk);

        if filter_out.tool_call_started && self.active_stream.is_some() {
            self.finalize_active_stream();
        }

        if !filter_out.text.is_empty() {
            if self.stream_opening_pending {
                self.pending_scrollback
                    .extend(crate::scrollback::stream_opening_lines(
                        self.stream_count,
                        std::mem::take(&mut self.pending_stream_opening),
                    ));
                self.stream_opening_pending = false;
                self.stream_count += 1;
            }
            self.pending_scrollback
                .extend(self.stream_render.push_chunk(&filter_out.text));
        }
    }

    fn handle_ui_output(&mut self, output: UiOutput) -> bool {
        match output {
            UiOutput::Stream(msg) => {
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
                self.pending_stream_opening = self.stream_render.start(msg.source.clone());
                self.stream_opening_pending = true;
                self.active_stream = Some(msg.stream);
            }
            UiOutput::ToolCallStarted { .. } => {
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
            }
            UiOutput::ToolCall(display) => {
                let line = crate::tool_display::build_tool_call_scrollback_line(&display);
                self.pending_scrollback.push(line);
            }
            UiOutput::ToolResult(display) => {
                let icon = if display.is_error { "✗" } else { "✓" };
                let color = if display.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                self.pending_scrollback.extend(
                    crate::tool_display::build_tool_result_scrollback_lines(&display, icon, color),
                );
            }
            UiOutput::ToolApprovalRequest {
                tool_name,
                arguments,
                summary_fields,
                response,
            } => {
                self.state.pending_approval_response = Some(response);
                let args_str = serde_json::to_string_pretty(&arguments)
                    .unwrap_or_else(|_| arguments.to_string());
                let summary = crate::tool_display::summarize_tool_args(
                    &tool_name,
                    &args_str,
                    &summary_fields,
                );
                self.show_approval(&tool_name, &summary);
            }
            UiOutput::Status(snapshot) => {
                self.state.status = snapshot;
            }
            UiOutput::Tip { text, kind } => {
                self.state.tip = Some(Tip {
                    kind,
                    text,
                    ttl: Duration::from_secs(2),
                    created_at: Instant::now(),
                });
            }
            UiOutput::CopyToClipboard { text, scope } => {
                let label = match scope {
                    CopyScope::Last => "last message",
                    CopyScope::All => "transcript",
                };
                match crate::clipboard::copy_text(&text) {
                    Ok(backend) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Info,
                            text: format!("Copied {label} to clipboard (via {backend:?})",),
                            ttl: Duration::from_secs(3),
                            created_at: Instant::now(),
                        });
                    }
                    Err(e) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Failed to copy {label}: {e:?}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                }
            }
            UiOutput::ExportToFile { path, content } => {
                let engine = talos_permission::PermissionEngine::default();
                match crate::export::export_transcript(&engine, &path, &content) {
                    Ok(()) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Info,
                            text: format!("Exported transcript to {}", path.display()),
                            ttl: Duration::from_secs(3),
                            created_at: Instant::now(),
                        });
                    }
                    Err(crate::export::ExportError::PermissionDenied(reason)) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Export denied: {reason}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                    Err(crate::export::ExportError::WriteFailed(reason)) => {
                        self.state.tip = Some(Tip {
                            kind: TipKind::Error,
                            text: format!("Export failed: {reason}"),
                            ttl: Duration::from_secs(4),
                            created_at: Instant::now(),
                        });
                    }
                }
            }
            UiOutput::Exit => {
                self.state.should_exit = true;
                return true;
            }
            UiOutput::SessionNew(_) | UiOutput::SessionResume(_) | UiOutput::SessionFork(_) => {
                // Handled by the bridge → mode runner lifecycle handler.
                // Should not reach the TUI directly.
            }
            UiOutput::SessionPicker(sessions) => {
                self.state.open_session_picker(&sessions);
            }
        }
        false
    }

    fn flush_pending_scrollback(&mut self) -> io::Result<()> {
        if self.pending_scrollback.is_empty() {
            return Ok(());
        }
        let lines = std::mem::take(&mut self.pending_scrollback);
        for line in lines {
            if line.has_plain_segments_only() {
                self.terminal.insert_history(&line.text, line.bg)?;
            } else {
                let mut segments = line.segments;
                if let Some(fill) = line.fill {
                    let trailing = segments
                        .first()
                        .map(|s| unicode_width::UnicodeWidthStr::width(s.text.as_str()))
                        .unwrap_or(0);
                    crate::scrollback::append_fill_segment(
                        &mut segments,
                        fill,
                        self.terminal.screen_size().width,
                        trailing,
                    );
                }
                self.terminal.insert_styled_history(&segments, line.bg)?;
            }
        }
        Ok(())
    }

    fn draw_frame(&mut self) -> io::Result<()> {
        let state = &self.state;
        let status = &state.status;

        let (preview_padding, spinner_color) = if status.is_processing {
            self.processing_tick += 1;
            if self.processing_tick.is_multiple_of(3) {
                self.processing_frame = self.processing_frame.wrapping_add(1);
            }
            let (padding, color_idx) = crate::scrollback::preview_spinner_padding(
                self.processing_frame,
                self.processing_tick,
            );
            (padding, Some(semantic::PROCESSING_SPINNER[color_idx]))
        } else {
            self.processing_frame = 0;
            self.processing_tick = 0;
            ("   ".to_string(), None)
        };
        let hold_status = self.stream_render.hold_status().cloned();
        let preview_text = hold_status
            .as_ref()
            .map(|status| {
                crate::scrollback::animated_hold_preview_text(status, self.processing_frame)
            })
            .unwrap_or_else(|| self.stream_render.preview().to_string());
        let preview_text_color = hold_status
            .as_ref()
            .map(|_| crate::scrollback::hold_preview_color(self.processing_frame));
        let preview = crate::scrollback::PreviewComponent {
            padding: &preview_padding,
            text: &preview_text,
            spinner_color,
            text_color: preview_text_color,
        };
        let queue = crate::scrollback::QueuePreviewComponent {
            count: status.steering_count + status.followup_count,
            steering: status.steering_count,
            followup: status.followup_count,
        };
        let tips = crate::scrollback::TipsComponent {
            tip: state.tip.as_ref(),
        };
        let input_pad_top = crate::scrollback::InputPadComponent;
        let input = crate::scrollback::InputComponent { state };
        let query = state.slash_query();
        let query_for_panel = if state.slash_menu.is_picker() { "" } else { query };
        let mut bottom_panel = crate::scrollback::BottomPanelComponent {
            menu: &state.slash_menu,
            query: query_for_panel,
            max_height: u16::MAX,
        };
        let input_pad_bot = crate::scrollback::InputPadComponent;

        let screen_size = self.terminal.screen_size();
        let width = screen_size.width;
        let status_comp = crate::scrollback::StatusComponent { status, width };

        let base_height = preview.height_hint(width)
            + queue.height_hint(width)
            + tips.height_hint(width)
            + input_pad_top.height_hint(width)
            + input.height_hint(width)
            + input_pad_bot.height_hint(width)
            + status_comp.height_hint(width);
        let natural_menu_height = bottom_panel.height_hint(width);
        let menu_placement = crate::scrollback::bottom_panel_placement(
            screen_size.height,
            base_height,
            natural_menu_height,
        );
        if matches!(
            menu_placement,
            crate::scrollback::BottomPanelPlacement::AboveInput
        ) {
            bottom_panel.max_height = screen_size.height.saturating_sub(base_height);
        }

        let stack = match menu_placement {
            crate::scrollback::BottomPanelPlacement::AboveInput => ComponentStack::new(vec![
                &preview,
                &queue,
                &tips,
                &bottom_panel,
                &input_pad_top,
                &input,
                &input_pad_bot,
                &status_comp,
            ]),
            crate::scrollback::BottomPanelPlacement::BelowInput => ComponentStack::new(vec![
                &preview,
                &queue,
                &tips,
                &input_pad_top,
                &input,
                &bottom_panel,
                &input_pad_bot,
                &status_comp,
            ]),
        };

        let total_height = stack.total_height(self.terminal.screen_size().width);

        if total_height > self.last_total_height && self.last_total_height > 0 {
            let viewport = self.terminal.viewport_area();
            let screen_h = self.terminal.screen_size().height;
            let new_bottom = viewport.y.saturating_add(total_height);
            let overflow = new_bottom.saturating_sub(screen_h);
            self.terminal.push_scrollback_up(overflow);
        }
        self.last_total_height = total_height;

        self.terminal.draw(total_height, |frame| {
            let layout = stack.layout(frame.area(), frame.area().width);
            for (component, area) in layout {
                component.render(frame, area);
            }
        })?;

        {
            let viewport = self.terminal.viewport_area();
            let screen_w = self.terminal.screen_size().width;
            let mut input_y_offset: u16 = preview.height_hint(screen_w)
                + queue.height_hint(screen_w)
                + tips.height_hint(screen_w)
                + input_pad_top.height_hint(screen_w);
            if matches!(
                menu_placement,
                crate::scrollback::BottomPanelPlacement::AboveInput
            ) {
                input_y_offset += bottom_panel.height_hint(screen_w);
            }
            let input_top = viewport.bottom().saturating_sub(total_height) + input_y_offset;
            let byte_pos = self.state.cursor_byte_pos();
            let (cursor_row_offset, cursor_col_offset) =
                crate::scrollback::cursor_line_col(&self.state.input_buffer[..byte_pos]);
            let input_row = input_top.saturating_add(cursor_row_offset);
            let cursor_col = 3u16 + cursor_col_offset;
            self.terminal.set_cursor(cursor_col, input_row)?;
        }

        Ok(())
    }

    fn handle_input_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                if !matches!(self.state.approval_state, ApprovalState::Hidden) {
                    self.handle_pending_approval_input(key.code);
                    return false;
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        let was_processing = self.state.status.is_processing;
                        if !was_processing && !self.state.input_buffer.is_empty() {
                            self.state.input_clear();
                            self.state.slash_menu.close();
                            self.state.ctrl_c_state = CtrlCState::Idle;
                            self.state.tip = Some(Tip {
                                kind: TipKind::ExitHint,
                                text: "Input cleared. Press Ctrl+C twice to exit.".to_string(),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
                            return false;
                        }
                        let should_exit = self.state.handle_ctrl_c();
                        if was_processing && let Some(ref tx) = self.user_input_tx {
                            let _ = tx.send(UserInput::Cancel);
                        }
                        return should_exit;
                    }
                    KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.toggle_evolution_panel();
                    }
                    KeyCode::Up if self.state.slash_menu.is_open => {
                        let query = if self.state.slash_menu.is_picker() {
                            String::new()
                        } else {
                            self.state.slash_query().to_string()
                        };
                        self.state.slash_menu.select_prev(&query);
                    }
                    KeyCode::Down if self.state.slash_menu.is_open => {
                        let query = if self.state.slash_menu.is_picker() {
                            String::new()
                        } else {
                            self.state.slash_query().to_string()
                        };
                        self.state.slash_menu.select_next(&query);
                    }
                    KeyCode::Tab if self.state.slash_menu.is_open => {
                        let action = self.state.accept_selected_panel_item();
                        if let PanelAction::SendMessage(msg) = action
                            && let Some(ref tx) = self.user_input_tx
                        {
                            let _ = tx.send(UserInput::Message(msg));
                        }
                    }
                    KeyCode::Enter if self.state.slash_menu.is_open => {
                        let action = self.state.accept_selected_panel_item();
                        if let PanelAction::SendMessage(msg) = action
                            && let Some(ref tx) = self.user_input_tx
                        {
                            let _ = tx.send(UserInput::Message(msg));
                        }
                    }
                    KeyCode::Esc if self.state.slash_menu.is_open => {
                        self.state.slash_menu.close();
                    }
                    KeyCode::Char('/') if self.state.input_buffer.is_empty() => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        let registry = talos_conversation::command_registry();
                        self.state.open_slash_menu(registry);
                    }
                    KeyCode::Char(c) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.slash_menu.is_open {
                            self.state.append_slash_query_char(c);
                        } else {
                            self.state.input_append_char(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.slash_menu.is_open {
                            self.state.backspace_slash_query();
                        } else {
                            self.state.input_backspace();
                        }
                    }
                    KeyCode::Left => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.slash_menu.is_open {
                            self.state.slash_menu.close();
                        }
                        self.state.input_cursor_left();
                    }
                    KeyCode::Right => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.slash_menu.is_open {
                            self.state.slash_menu.close();
                        }
                        self.state.input_cursor_right();
                    }
                    KeyCode::Enter => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        let input = self.state.input_submit();
                        if !input.is_empty()
                            && let Some(ref tx) = self.user_input_tx
                        {
                            let _ = tx.send(UserInput::Message(input));
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                    }
                    _ => {}
                }
            }
            Event::Paste(text) => {
                self.state.ctrl_c_state = CtrlCState::Idle;
                self.state.slash_menu.close();
                self.state.input_append_str(text);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }

    fn restore(&self) {
        self.terminal.restore();
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.restore();
    }
}

#[allow(warnings)]
#[cfg(test)]
mod app_tests;
