use std::io;
use std::pin::Pin;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    style::Color as CColor,
    terminal::enable_raw_mode,
};
use futures::{Stream, StreamExt};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph},
};
use talos_conversation::{
    MessageSource, StatusSnapshot, TipKind, ToolCallDisplay, UiOutput, UserInput,
};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use talos_core::message::Message;
use talos_core::tool::ToolProvenance;
use talos_core::tool_filter::ToolSyntaxFilter;
use tokio::sync::mpsc;

use crate::evolution::{self, EvolutionPanel};
use crate::highlight::HighlightEngine;
use crate::inline_terminal::{
    ComponentStack, HistoryAttrs, HistorySegment, InlineFrame, InlineTerminal, ViewportComponent,
};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, Tip, TuiState};
use crate::stream_markdown::{BlockDecision, HoldStatus, MarkdownBlockKind, StreamBlockClassifier};
use crate::theme::{semantic, to_crossterm_color};

const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Clone, Debug, Eq)]
pub(crate) struct ScrollbackLine {
    pub(crate) text: String,
    segments: Vec<HistorySegment>,
    bg: Option<CColor>,
}

impl PartialEq for ScrollbackLine {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.bg == other.bg
    }
}

impl ScrollbackLine {
    fn plain(text: impl Into<String>, bg: Option<CColor>) -> Self {
        let text = text.into();
        Self {
            segments: vec![HistorySegment::raw(text.clone())],
            text,
            bg,
        }
    }

    fn styled(segments: Vec<HistorySegment>, bg: Option<CColor>) -> Self {
        let text = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>();
        Self { text, segments, bg }
    }

    fn has_plain_segments_only(&self) -> bool {
        self.segments
            .iter()
            .all(|segment| segment.fg.is_none() && segment.attrs == HistoryAttrs::default())
    }
}

#[derive(Default)]
struct StreamRenderState {
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
    fn start(&mut self, source: MessageSource) -> Vec<ScrollbackLine> {
        self.start_with_hold(source, false)
    }

    fn start_with_hold(
        &mut self,
        source: MessageSource,
        hold_complete_lines: bool,
    ) -> Vec<ScrollbackLine> {
        let bg = stream_bg_for(Some(&source));
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

    fn source(&self) -> Option<&MessageSource> {
        self.source.as_ref()
    }

    fn preview(&self) -> &str {
        &self.preview
    }

    fn hold_status(&self) -> Option<&HoldStatus> {
        self.hold_status.as_ref()
    }

    fn push_chunk(&mut self, chunk: &str) -> Vec<ScrollbackLine> {
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

    fn finish(&mut self) -> Vec<ScrollbackLine> {
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
        let padding = stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        segments.extend(render_markdown_segments(line, block));
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_segments_line(
        &self,
        line_index: usize,
        content_segments: Vec<HistorySegment>,
    ) -> ScrollbackLine {
        let padding = stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        segments.extend(content_segments);
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_plain_line(&self, line_index: usize, line: &str) -> ScrollbackLine {
        let padding = stream_padding_for(self.source(), line_index);
        let segments = vec![
            HistorySegment::styled(
                padding,
                prefix_color_for(self.source(), line_index),
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
            let bg = stream_bg_for(bg_source.as_ref());
            if let Some(rendered) =
                Self::try_highlight_code_block(&mut self.highlight_engine, &block_lines, bg_source)
            {
                return rendered;
            }
            return render_code_block(&block_lines, bg);
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

        Some(build_code_block(
            &highlighted_lines,
            lang,
            stream_bg_for(source.as_ref()),
        ))
    }

    fn render_table_lines(&mut self, block_lines: Vec<String>) -> Vec<ScrollbackLine> {
        let table_lines = render_table_block(&block_lines).unwrap_or_else(|| {
            block_lines
                .into_iter()
                .enumerate()
                .map(|(row_index, line)| render_table_history_line(&line, row_index))
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
        stream_bg_for(self.source())
    }

    fn reset(&mut self) {
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

struct PreviewComponent<'a> {
    padding: &'a str,
    text: &'a str,
    spinner_color: Option<Color>,
    text_color: Option<Color>,
}

impl ViewportComponent for PreviewComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let line = self.text.split('\n').next_back().unwrap_or("");
        let text_color = self.text_color.unwrap_or(semantic::PREVIEW_FG);
        if let Some(color) = self.spinner_color {
            let full = format!("{}{}", self.padding, line);
            let display = truncate_end_to_width(&full, area.width);
            let padding_len = self.padding.chars().count();
            let (pad_part, text_part) = display.split_at(
                display
                    .char_indices()
                    .nth(padding_len)
                    .map(|(i, _)| i)
                    .unwrap_or(display.len()),
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(pad_part.to_string(), Style::default().fg(color)),
                    Span::styled(text_part.to_string(), Style::default().fg(text_color)),
                ])),
                area,
            );
        } else {
            let full = format!("{}{}", self.padding, line);
            let display = truncate_end_to_width(&full, area.width);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    display,
                    Style::default().fg(text_color),
                ))),
                area,
            );
        }
    }
}

fn animated_hold_preview_text(status: &HoldStatus, frame: usize) -> String {
    let base = status.preview_text().trim_end_matches('.');
    let dots = match (frame / 2) % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    };
    format!("{base}{dots}")
}

fn hold_preview_color(frame: usize) -> Color {
    semantic::HOLD_PREVIEW[(frame / 2) % semantic::HOLD_PREVIEW.len()]
}

fn preview_spinner_padding(processing_frame: usize, _processing_tick: usize) -> (String, usize) {
    let n = SPINNER_FRAMES.len();
    let lead_idx = (processing_frame + n / 2) % n;
    let chase_idx = processing_frame % n;
    (
        format!(" {}{}", SPINNER_FRAMES[lead_idx], SPINNER_FRAMES[chase_idx]),
        lead_idx,
    )
}

struct QueuePreviewComponent {
    count: usize,
    steering: usize,
    followup: usize,
}

impl ViewportComponent for QueuePreviewComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        if self.count == 0 {
            0
        } else {
            1 + (self.count as u16).min(2)
        }
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let dim = Style::default().fg(semantic::DIM_TEXT);
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(" ", dim),
            Span::styled(
                format!(
                    "{} queued input{}",
                    self.count,
                    if self.count == 1 { "" } else { "s" }
                ),
                dim,
            ),
            Span::styled(" (will send after current turn)", dim),
        ]));
        let max_width = (area.width as usize).saturating_sub(4);
        let show_steering = self.steering.min(2);
        for i in 0..show_steering {
            let label = if i == 0 { "steering" } else { "…" };
            let text = if label.len() > max_width {
                format!("{}…", &label[..max_width - 1])
            } else {
                label.to_string()
            };
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(text, dim),
            ]));
        }
        if self.followup > 0 && lines.len() < 3 {
            let label = if self.followup == 1 {
                "followup".to_string()
            } else {
                format!("followup x{}", self.followup)
            };
            let text = if label.len() > max_width {
                format!("{}…", &label[..max_width - 1])
            } else {
                label
            };
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(text, dim),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), area);
    }
}

struct TipsComponent<'a> {
    tip: Option<&'a Tip>,
}

impl ViewportComponent for TipsComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let text = if let Some(tip) = self.tip {
            let color = match tip.kind {
                TipKind::ExitHint | TipKind::QueueHint => semantic::TIP_SUCCESS,
                TipKind::ApprovalResult => semantic::TIP_RESULT,
                TipKind::LagWarning | TipKind::Error => semantic::TIP_ERROR,
                TipKind::Info => semantic::TIP_INFO,
            };
            Text::from(Line::from(Span::styled(
                format!(" {}", tip.text),
                Style::default().fg(color),
            )))
        } else {
            Text::from(Line::from(Span::styled(
                " Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution",
                Style::default().fg(semantic::DIM_TEXT),
            )))
        };
        frame.render_widget(Paragraph::new(text), area);
    }
}

struct InputPadComponent;

impl ViewportComponent for InputPadComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(semantic::INPUT_BG)),
            area,
        );
    }
}

struct InputComponent<'a> {
    state: &'a TuiState,
}

impl ViewportComponent for InputComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        input_line_count(&self.state.input_buffer)
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let input_text = build_input_text(self.state);
        let input_block = Block::default()
            .style(Style::default().bg(semantic::INPUT_BG))
            .padding(Padding::new(0, 1, 0, 0));
        frame.render_widget(Paragraph::new(input_text).block(input_block), area);
    }
}

struct StatusComponent<'a> {
    status: &'a StatusSnapshot,
}

impl ViewportComponent for StatusComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let text = build_status_text(self.status);
        frame.render_widget(Paragraph::new(text), area);
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
    active_stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    stream_render: StreamRenderState,
    stream_opening_pending: bool,
    pending_stream_opening: Vec<ScrollbackLine>,
    text_filter: ToolSyntaxFilter,
    processing_frame: usize,
    processing_tick: usize,
    stream_count: usize,
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
            active_stream: None,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            text_filter: ToolSyntaxFilter::new(),
            processing_frame: 0,
            processing_tick: 0,
            stream_count: 0,
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
        self.pending_scrollback
            .extend(render_history_messages(&mut self.stream_count, history));
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
        self.state.approval_state = ApprovalState::Visible {
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            selected: ApprovalChoice::ApproveOnce,
        };
        let warn = to_crossterm_color(semantic::TEXT_WARNING);
        let accent = to_crossterm_color(semantic::TEXT_ACCENT);
        self.pending_scrollback.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("⚠ Approval required: {}", tool_name),
                warn,
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            )],
            None,
        ));
        self.pending_scrollback.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                "[y] approve once  [a] always  [n] deny",
                accent,
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    pub fn hide_approval(&mut self) {
        self.state.approval_state = ApprovalState::Hidden;
    }

    pub async fn run_with_approval(
        &mut self,
        mut approval_rx: mpsc::UnboundedReceiver<TuiApprovalRequest>,
    ) -> io::Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));
        let mut ui_output_rx = self.ui_output_rx.take().expect("ui_output_rx not set");

        // Establish the viewport before flushing restored history. Otherwise the
        // first scrollback lines can be written into the future input area and
        // then erased by the first frame draw.
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
                }
                Some(output) = ui_output_rx.recv() => {
                    let is_tool = matches!(&output, UiOutput::ToolCall(_));
                    if self.handle_ui_output(output) {
                        break;
                    }
                    if is_tool {
                        self.flush_pending_scrollback()?;
                        self.draw_frame()?;
                    }
                }
                Some(request) = approval_rx.recv() => {
                    self.state.pending_approval_response = Some(request.response);
                    self.show_approval(&request.tool_name, &request.arguments);
                }
                Some(chunk) = self.next_stream_chunk() => {
                    self.consume_stream_chunk(&chunk);
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        self.restore();
        Ok(())
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
                self.pending_scrollback.extend(stream_opening_lines(
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
                let line = build_tool_call_scrollback_line(&display);
                self.pending_scrollback.push(line);
            }
            UiOutput::ToolResult(display) => {
                let icon = if display.is_error { "✗" } else { "✓" };
                let color = if display.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                let content_trunc = {
                    let first = display
                        .content
                        .lines()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or(&display.content);
                    if first.len() > 120 {
                        format!("{}…", &first[..120])
                    } else {
                        first.to_string()
                    }
                };
                let line_text = format!("   {} {}", icon, content_trunc);
                let segments = vec![HistorySegment::styled(
                    line_text,
                    color,
                    HistoryAttrs::default(),
                )];
                self.pending_scrollback
                    .push(ScrollbackLine::styled(segments, None));
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
            UiOutput::Exit => {
                self.state.should_exit = true;
                return true;
            }
        }
        false
    }

    fn flush_pending_scrollback(&mut self) -> io::Result<()> {
        if self.pending_scrollback.is_empty() {
            return Ok(());
        }
        let lines = std::mem::take(&mut self.pending_scrollback);
        for line in &lines {
            if line.has_plain_segments_only() {
                self.terminal.insert_history(&line.text, line.bg)?;
            } else {
                self.terminal
                    .insert_styled_history(&line.segments, line.bg)?;
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
            let (padding, color_idx) =
                preview_spinner_padding(self.processing_frame, self.processing_tick);
            (padding, Some(semantic::PROCESSING_SPINNER[color_idx]))
        } else {
            self.processing_frame = 0;
            self.processing_tick = 0;
            ("   ".to_string(), None)
        };
        let hold_status = self.stream_render.hold_status().cloned();
        let preview_text = hold_status
            .as_ref()
            .map(|status| animated_hold_preview_text(status, self.processing_frame))
            .unwrap_or_else(|| self.stream_render.preview().to_string());
        let preview_text_color = hold_status
            .as_ref()
            .map(|_| hold_preview_color(self.processing_frame));
        let preview = PreviewComponent {
            padding: &preview_padding,
            text: &preview_text,
            spinner_color,
            text_color: preview_text_color,
        };
        let queue = QueuePreviewComponent {
            count: status.steering_count + status.followup_count,
            steering: status.steering_count,
            followup: status.followup_count,
        };
        let tips = TipsComponent {
            tip: state.tip.as_ref(),
        };
        let input_pad_top = InputPadComponent;
        let input = InputComponent { state };
        let input_pad_bot = InputPadComponent;
        let status_comp = StatusComponent { status };

        let stack = ComponentStack::new(vec![
            &preview,
            &queue,
            &tips,
            &input_pad_top,
            &input,
            &input_pad_bot,
            &status_comp,
        ]);

        let total_height = stack.total_height(self.terminal.screen_size().width);

        self.terminal.draw(total_height, |frame| {
            let layout = stack.layout(frame.area(), frame.area().width);
            for (component, area) in layout {
                component.render(frame, area);
            }
        })?;

        {
            let viewport = self.terminal.viewport_area();
            let screen_w = self.terminal.screen_size().width;
            let input_y_offset: u16 = preview.height_hint(screen_w)
                + queue.height_hint(screen_w)
                + tips.height_hint(screen_w)
                + input_pad_top.height_hint(screen_w);
            let input_top = viewport.bottom().saturating_sub(total_height) + input_y_offset;
            let byte_pos = self.state.cursor_byte_pos();
            let (cursor_row_offset, cursor_col_offset) =
                cursor_line_col(&self.state.input_buffer[..byte_pos]);
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
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        let was_processing = self.state.status.is_processing;
                        let should_exit = self.state.handle_ctrl_c();
                        if was_processing && let Some(ref tx) = self.user_input_tx {
                            let _ = tx.send(UserInput::Cancel);
                        }
                        return should_exit;
                    }
                    KeyCode::Char('k') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.toggle_skill_sidebar();
                    }
                    KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.toggle_evolution_panel();
                    }
                    KeyCode::Char(c)
                        if !matches!(self.state.approval_state, ApprovalState::Hidden) =>
                    {
                        if let Some(choice) = self.handle_approval_key(c) {
                            if let Some(response_tx) = self.state.pending_approval_response.take() {
                                let _ = response_tx.send(choice.clone());
                            }
                            self.hide_approval();
                            self.state.tip = Some(Tip {
                                kind: TipKind::ApprovalResult,
                                text: format!(
                                    "Tool call {}",
                                    match choice {
                                        ApprovalChoice::ApproveOnce => "approved once",
                                        ApprovalChoice::AlwaysApprove => "always approved",
                                        ApprovalChoice::Deny => "denied",
                                    }
                                ),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
                        }
                    }
                    KeyCode::Char(c) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_append_char(c);
                    }
                    KeyCode::Backspace => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_backspace();
                    }
                    KeyCode::Left => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_left();
                    }
                    KeyCode::Right => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
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
                        self.state.input_clear();
                    }
                    _ => {}
                }
            }
            Event::Paste(text) => {
                self.state.ctrl_c_state = CtrlCState::Idle;
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

pub(crate) fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let char_count = buffer.chars().count();
    let cursor_pos = state.cursor_pos.min(char_count);
    let cursor_style = Style::default()
        .fg(semantic::APPROVAL_BUTTON)
        .bg(semantic::APPROVAL_BUTTON_BG);

    let prompt_style = Style::default().fg(semantic::APPROVAL_PROMPT);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut line_index = 0usize;
    let mut spans = vec![Span::styled(" > ", prompt_style)];
    let mut cursor_rendered = false;

    for (idx, ch) in buffer.chars().enumerate() {
        if idx == cursor_pos {
            cursor_rendered = true;
            if ch == '\n' {
                spans.push(Span::styled(" ", cursor_style));
            } else {
                spans.push(Span::styled(ch.to_string(), cursor_style));
                continue;
            }
        }

        if ch == '\n' {
            lines.push(Line::from(spans));
            line_index += 1;
            spans = vec![Span::raw(input_prefix_for_line(line_index))];
        } else {
            spans.push(Span::raw(ch.to_string()));
        }
    }

    if !cursor_rendered {
        spans.push(Span::styled(" ", cursor_style));
    }
    lines.push(Line::from(spans));

    Text::from(lines)
}

pub(crate) fn input_prefix_for_line(line_index: usize) -> &'static str {
    if line_index == 0 { " > " } else { "   " }
}

pub(crate) fn stream_padding_for(
    source: Option<&MessageSource>,
    line_index: usize,
) -> &'static str {
    if line_index > 0 {
        return "   ";
    }

    match source {
        Some(MessageSource::User) => " > ",
        Some(MessageSource::Assistant) => " ● ",
        Some(MessageSource::System) => " # ",
        Some(MessageSource::Error) => " ! ",
        Some(MessageSource::Tool { .. }) => " ● ",
        None => "   ",
    }
}

fn stream_bg_for(source: Option<&MessageSource>) -> Option<CColor> {
    match source {
        Some(MessageSource::User) => to_crossterm_color(semantic::INPUT_BG),
        _ => None,
    }
}

fn prefix_color_for(source: Option<&MessageSource>, line_index: usize) -> Option<CColor> {
    if line_index > 0 {
        return None;
    }

    match source {
        Some(MessageSource::User) => to_crossterm_color(semantic::PREFIX_USER),
        Some(MessageSource::Assistant) | Some(MessageSource::Tool { .. }) => {
            to_crossterm_color(semantic::PREFIX_ASSISTANT)
        }
        Some(MessageSource::System) => to_crossterm_color(semantic::PREFIX_SYSTEM),
        Some(MessageSource::Error) => to_crossterm_color(semantic::PREFIX_ERROR),
        None => None,
    }
}

fn stream_opening_lines(stream_count: usize, opening: Vec<ScrollbackLine>) -> Vec<ScrollbackLine> {
    let mut lines = Vec::new();
    if stream_count > 0 {
        lines.push(ScrollbackLine::plain(String::new(), None));
    }
    lines.extend(opening);
    lines
}

fn render_history_message(
    stream_count: &mut usize,
    source: MessageSource,
    content: &str,
) -> Vec<ScrollbackLine> {
    let mut renderer = StreamRenderState::default();
    let mut lines = stream_opening_lines(*stream_count, renderer.start(source));
    lines.extend(renderer.push_chunk(content));
    if !content.ends_with('\n') {
        lines.extend(renderer.push_chunk("\n"));
    }
    lines.extend(renderer.finish());
    *stream_count += 1;
    lines
}

fn render_history_messages(stream_count: &mut usize, history: &[Message]) -> Vec<ScrollbackLine> {
    let mut lines = Vec::new();
    for message in history {
        let Some((source, content)) = history_message_parts(message) else {
            continue;
        };
        if content.is_empty() {
            continue;
        }

        lines.extend(render_history_message(stream_count, source, content));
    }
    lines
}

fn history_message_parts(message: &Message) -> Option<(MessageSource, &str)> {
    match message {
        Message::User { content } => Some((MessageSource::User, content.as_str())),
        Message::Assistant { content, .. } => Some((MessageSource::Assistant, content.as_str())),
        Message::Tool { result } => Some((
            MessageSource::Tool {
                name: result.tool_use_id.clone(),
            },
            result.content.as_str(),
        )),
        Message::System { content } => Some((MessageSource::System, content.as_str())),
        Message::Context { content } => Some((MessageSource::System, content.as_str())),
    }
}

fn render_markdown_segments(
    line: &str,
    block: Option<(&MarkdownBlockKind, usize)>,
) -> Vec<HistorySegment> {
    match block {
        Some((MarkdownBlockKind::CodeFence, _)) => render_code_block_line(line),
        Some((MarkdownBlockKind::Table, row_index)) => render_table_history_line(line, row_index),
        Some((MarkdownBlockKind::List, _)) => render_list_line(line),
        Some((MarkdownBlockKind::Quote, _)) => render_quote_line(line),
        None => render_inline_markdown(line),
    }
}

fn render_code_block_line(line: &str) -> Vec<HistorySegment> {
    let trimmed = line.trim_start();
    let attrs = if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        HistoryAttrs {
            dim: true,
            ..HistoryAttrs::default()
        }
    } else {
        HistoryAttrs::default()
    };
    vec![HistorySegment::styled(
        line,
        to_crossterm_color(semantic::MARKDOWN_CODE),
        attrs,
    )]
}

fn render_code_block(block_lines: &[String], bg: Option<CColor>) -> Vec<ScrollbackLine> {
    if block_lines.len() < 3 {
        return block_lines
            .iter()
            .map(|l| ScrollbackLine::styled(render_code_block_line(l), bg))
            .collect();
    }

    let opening = &block_lines[0];
    let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();
    let code_lines = &block_lines[1..block_lines.len() - 1];

    let plain_lines: Vec<Vec<(String, Option<CColor>)>> = code_lines
        .iter()
        .map(|line| vec![(line.clone(), None)])
        .collect();

    build_code_block(&plain_lines, lang, bg)
}

fn build_code_block(
    lines: &[Vec<(String, Option<CColor>)>],
    lang: &str,
    bg: Option<CColor>,
) -> Vec<ScrollbackLine> {
    let dim_color = to_crossterm_color(semantic::DIM_TEXT);
    let line_num_color = to_crossterm_color(semantic::DIM_TEXT);

    let content_max = lines
        .iter()
        .map(|segs| segs.iter().map(|(t, _)| t.len()).sum::<usize>())
        .max()
        .unwrap_or(0);

    let max_width = content_max.max(lang.len() + 2);
    let indent = "   ";
    let mut rendered = Vec::with_capacity(lines.len() + 1);

    let top_line = if lang.is_empty() {
        format!("{}{}", indent, "─".repeat(max_width))
    } else {
        let label = format!("[{}]", lang);
        let remaining = max_width.saturating_sub(label.len());
        format!("{}{}{}", indent, label, "─".repeat(remaining))
    };
    rendered.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            top_line,
            dim_color,
            HistoryAttrs::default(),
        )],
        bg,
    ));

    let num_width = if lines.is_empty() {
        1
    } else {
        (lines.len() as f64).log10().floor() as usize + 1
    };

    for (i, line_segments) in lines.iter().enumerate() {
        let mut segments = Vec::new();

        let num_text = format!("{:>width$}", i + 1, width = num_width);
        segments.push(HistorySegment::styled(
            format!("{}{}", indent, num_text),
            line_num_color,
            HistoryAttrs::default(),
        ));
        segments.push(HistorySegment::styled(
            " │ ".to_string(),
            dim_color,
            HistoryAttrs::default(),
        ));

        if line_segments.is_empty() {
            segments.push(HistorySegment::styled(
                String::new(),
                None,
                HistoryAttrs::default(),
            ));
        } else {
            for (text, color) in line_segments {
                segments.push(HistorySegment::styled(
                    text.clone(),
                    *color,
                    HistoryAttrs::default(),
                ));
            }
        }

        rendered.push(ScrollbackLine::styled(segments, bg));
    }

    let bottom_line = format!("{}{}", indent, "─".repeat(max_width));
    rendered.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            bottom_line,
            dim_color,
            HistoryAttrs::default(),
        )],
        bg,
    ));

    rendered
}

fn render_table_history_line(line: &str, row_index: usize) -> Vec<HistorySegment> {
    let cells = split_table_cells(line);
    if cells.is_empty() {
        return render_inline_markdown(line);
    }

    if row_index == 1 {
        let sep_color = to_crossterm_color(semantic::STATUS_VALUE);
        return vec![HistorySegment::styled(
            cells
                .iter()
                .map(|cell| "─".repeat(cell.len().max(3)))
                .collect::<Vec<_>>()
                .join("\t"),
            sep_color,
            HistoryAttrs::default(),
        )];
    }

    let mut segments = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            segments.push(HistorySegment::raw("\t"));
        }
        let mut cell_segments = render_inline_markdown(cell);
        if row_index == 0 {
            for segment in &mut cell_segments {
                segment.attrs.bold = true;
                if segment.fg.is_none() {
                    segment.fg = to_crossterm_color(semantic::MARKDOWN_TABLE_HEADER);
                }
            }
        }
        segments.extend(cell_segments);
    }
    segments
}

fn render_table_block(lines: &[String]) -> Option<Vec<Vec<HistorySegment>>> {
    if lines.len() < 2 || !is_table_separator_line(&lines[1]) {
        return None;
    }

    let body_rows: Vec<Vec<Vec<HistorySegment>>> = lines
        .iter()
        .enumerate()
        .filter(|(row_index, _)| *row_index != 1)
        .map(|(row_index, line)| {
            split_table_cells(line)
                .into_iter()
                .map(|cell| {
                    let mut segments = render_inline_markdown(&cell);
                    if row_index == 0 {
                        emphasize_table_header(&mut segments);
                    }
                    segments
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let column_count = body_rows.iter().map(Vec::len).max().unwrap_or(0);
    if column_count < 2 {
        return None;
    }

    let mut widths = vec![3usize; column_count];
    for row in &body_rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(history_segments_width(cell));
        }
    }

    let mut rendered = Vec::new();
    rendered.push(table_border_line('╭', '┬', '╮', &widths));
    for (row_index, row) in body_rows.iter().enumerate() {
        if row_index == 1 {
            rendered.push(table_border_line('├', '┼', '┤', &widths));
        }
        rendered.push(table_content_line(row, &widths));
    }
    rendered.push(table_border_line('╰', '┴', '╯', &widths));
    Some(rendered)
}

fn table_border_line(
    left: char,
    separator: char,
    right: char,
    widths: &[usize],
) -> Vec<HistorySegment> {
    let sep_color = to_crossterm_color(semantic::STATUS_VALUE);
    let mut text = String::new();
    text.push(left);
    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            text.push(separator);
        }
        text.push_str(&"─".repeat(width + 2));
    }
    text.push(right);
    vec![HistorySegment::styled(
        text,
        sep_color,
        HistoryAttrs::default(),
    )]
}

fn table_content_line(row: &[Vec<HistorySegment>], widths: &[usize]) -> Vec<HistorySegment> {
    let mut segments = vec![table_border_segment("│")];
    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            segments.push(table_border_segment("│"));
        }
        segments.push(HistorySegment::raw(" "));
        let cell_segments = row.get(i).cloned().unwrap_or_default();
        let cell_width = history_segments_width(&cell_segments);
        segments.extend(cell_segments);
        if *width > cell_width {
            segments.push(HistorySegment::raw(" ".repeat(width - cell_width)));
        }
        segments.push(HistorySegment::raw(" "));
    }
    segments.push(table_border_segment("│"));
    segments
}

fn table_border_segment(text: impl Into<String>) -> HistorySegment {
    HistorySegment::styled(
        text,
        to_crossterm_color(semantic::STATUS_VALUE),
        HistoryAttrs::default(),
    )
}

fn emphasize_table_header(segments: &mut [HistorySegment]) {
    for segment in segments {
        segment.attrs.bold = true;
        if segment.fg.is_none() {
            segment.fg = to_crossterm_color(semantic::MARKDOWN_TABLE_HEADER);
        }
    }
}

fn history_segments_width(segments: &[HistorySegment]) -> usize {
    segments
        .iter()
        .map(|segment| unicode_width::UnicodeWidthStr::width(segment.text.as_str()))
        .sum()
}

fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|').trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.split('|').all(|cell| {
        let cell = cell.trim();
        let cell = cell.trim_start_matches(':').trim_end_matches(':');
        cell.len() >= 3 && cell.chars().all(|ch| ch == '-')
    })
}

fn render_list_line(line: &str) -> Vec<HistorySegment> {
    let Some((prefix, body)) = split_list_marker(line) else {
        return render_inline_markdown(line);
    };
    let mut segments = vec![HistorySegment::styled(
        prefix,
        to_crossterm_color(semantic::MARKDOWN_LIST_MARKER),
        HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        },
    )];
    segments.extend(render_inline_markdown(body));
    segments
}

fn render_quote_line(line: &str) -> Vec<HistorySegment> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let rest = rest.strip_prefix('>').unwrap_or(rest);
    let rest = rest.strip_prefix(' ').unwrap_or(rest);
    let mut segments = Vec::new();
    if !indent.is_empty() {
        segments.push(HistorySegment::raw(indent));
    }
    segments.push(HistorySegment::styled(
        "> ",
        to_crossterm_color(semantic::MARKDOWN_QUOTE_MARKER),
        HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        },
    ));
    for mut segment in render_inline_markdown(rest) {
        segment.attrs.dim = true;
        if segment.fg.is_none() {
            segment.fg = to_crossterm_color(semantic::MARKDOWN_QUOTE_TEXT);
        }
        segments.push(segment);
    }
    segments
}

fn render_inline_markdown(line: &str) -> Vec<HistorySegment> {
    if is_horizontal_rule(line) {
        let hr_width = 40usize;
        return vec![HistorySegment::styled(
            "─".repeat(hr_width),
            to_crossterm_color(semantic::STATUS_VALUE),
            HistoryAttrs::default(),
        )];
    }

    if let Some((indent, marker, heading)) = split_heading(line) {
        let mut segments = Vec::new();
        if !indent.is_empty() {
            segments.push(HistorySegment::raw(indent));
        }
        if heading.is_empty() {
            segments.push(HistorySegment::raw(marker));
        } else {
            for mut segment in parse_inline_delimiters(heading) {
                segment.attrs.bold = true;
                if segment.fg.is_none() {
                    segment.fg = to_crossterm_color(semantic::MARKDOWN_HEADING);
                }
                segments.push(segment);
            }
        }
        return segments;
    }

    parse_inline_delimiters(line)
}

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let Some(marker) = trimmed.chars().next() else {
        return false;
    };
    matches!(marker, '-' | '*' | '_') && trimmed.chars().all(|ch| ch == marker)
}

fn split_heading(line: &str) -> Option<(&str, &str, &str)> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let marker_len = rest.chars().take_while(|ch| *ch == '#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }
    let marker_end = marker_len;
    let after_marker = rest.get(marker_end..)?;
    if !after_marker.starts_with(' ') {
        return None;
    }
    Some((indent, &rest[..=marker_end], after_marker.trim_start()))
}

fn parse_inline_delimiters(line: &str) -> Vec<HistorySegment> {
    let mut segments = Vec::new();
    let mut plain = String::new();
    let mut rest = line;

    while !rest.is_empty() {
        if let Some(after_tick) = rest.strip_prefix('`')
            && let Some(end) = after_tick.find('`')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (code, after) = after_tick.split_at(end);
            segments.push(HistorySegment::styled(
                code,
                to_crossterm_color(semantic::MARKDOWN_CODE),
                HistoryAttrs::default(),
            ));
            rest = &after[1..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix("**")
            && let Some(end) = after_marker.find("**")
        {
            push_plain_segment(&mut segments, &mut plain);
            let (strong, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                strong,
                to_crossterm_color(semantic::MARKDOWN_TEXT_STRONG),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[2..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix("__")
            && let Some(end) = after_marker.find("__")
        {
            push_plain_segment(&mut segments, &mut plain);
            let (strong, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                strong,
                to_crossterm_color(semantic::MARKDOWN_TEXT_STRONG),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[2..];
            continue;
        }

        if let Some(after_bracket) = rest.strip_prefix('[')
            && let Some(label_end) = after_bracket.find("](")
        {
            let (label, after_label) = after_bracket.split_at(label_end);
            let after_url_start = &after_label[2..];
            if let Some(url_end) = after_url_start.find(')') {
                push_plain_segment(&mut segments, &mut plain);
                let (url, after_url) = after_url_start.split_at(url_end);
                segments.push(HistorySegment::styled(
                    label,
                    to_crossterm_color(semantic::MARKDOWN_LINK),
                    HistoryAttrs {
                        underlined: true,
                        ..HistoryAttrs::default()
                    },
                ));
                if !url.is_empty() {
                    segments.push(HistorySegment::styled(
                        format!(" ({url})"),
                        to_crossterm_color(semantic::MARKDOWN_LINK_URL),
                        HistoryAttrs {
                            dim: true,
                            ..HistoryAttrs::default()
                        },
                    ));
                }
                rest = &after_url[1..];
                continue;
            }
        }

        if let Some(after_marker) = rest.strip_prefix('*')
            && !after_marker.starts_with('*')
            && let Some(end) = after_marker.find('*')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (emphasis, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                emphasis,
                to_crossterm_color(semantic::MARKDOWN_TEXT_EMPHASIS),
                HistoryAttrs {
                    italic: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[1..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix('_')
            && !after_marker.starts_with('_')
            && let Some(end) = after_marker.find('_')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (emphasis, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                emphasis,
                to_crossterm_color(semantic::MARKDOWN_TEXT_EMPHASIS),
                HistoryAttrs {
                    italic: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[1..];
            continue;
        }

        let ch = rest.chars().next().expect("rest is not empty");
        plain.push(ch);
        rest = &rest[ch.len_utf8()..];
    }

    push_plain_segment(&mut segments, &mut plain);
    if segments.is_empty() {
        vec![HistorySegment::raw("")]
    } else {
        segments
    }
}

fn push_plain_segment(segments: &mut Vec<HistorySegment>, plain: &mut String) {
    if !plain.is_empty() {
        segments.push(HistorySegment::raw(std::mem::take(plain)));
    }
}

fn split_list_marker(line: &str) -> Option<(&str, &str)> {
    let trimmed_start = line.len() - line.trim_start().len();
    let rest = &line[trimmed_start..];
    for marker in ["- ", "* ", "+ "] {
        if let Some(body) = rest.strip_prefix(marker) {
            return Some((&line[..trimmed_start + marker.len()], body));
        }
    }
    let dot = rest.find('.')?;
    if dot == 0 || !rest[..dot].chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    rest.get(dot + 1..)
        .and_then(|after| after.starts_with(' ').then_some(()))?;
    let marker_len = dot + 2;
    Some((&line[..trimmed_start + marker_len], &rest[marker_len..]))
}

fn split_table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

pub(crate) fn input_line_count(buffer: &str) -> u16 {
    buffer.split('\n').count().max(1) as u16
}

pub(crate) fn cursor_line_col(buffer_before_cursor: &str) -> (u16, u16) {
    let row = buffer_before_cursor.matches('\n').count() as u16;
    let col = buffer_before_cursor
        .rsplit('\n')
        .next()
        .map(unicode_width::UnicodeWidthStr::width)
        .unwrap_or(0) as u16;
    (row, col)
}

pub(crate) fn build_status_text(status: &StatusSnapshot) -> Text<'static> {
    let model_name = status.model_name.clone();
    let total_tokens = status.usage.input_tokens + status.usage.output_tokens;
    let cost = calculate_cost(&status.usage);

    let branch_info = status
        .branch_id
        .as_ref()
        .map(|b| {
            let short: String = b.chars().take(8).collect();
            format!(" │ {short}")
        })
        .unwrap_or_default();

    let queue_info = if status.steering_count > 0 || status.followup_count > 0 {
        let mut parts = Vec::new();
        if status.steering_count > 0 {
            parts.push(format!("S:{}", status.steering_count));
        }
        if status.followup_count > 0 {
            parts.push(format!("F:{}", status.followup_count));
        }
        format!(" │ {}", parts.join(", "))
    } else {
        String::new()
    };

    let dim = Style::default().fg(semantic::DIM_TEXT);
    let sep = Span::styled(" │ ", dim);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let spans = vec![
        Span::styled(" ", dim),
        Span::styled(model_name, val),
        sep.clone(),
        Span::styled(format!("{} tokens", total_tokens), val),
        Span::styled(branch_info, val),
        sep.clone(),
        Span::styled(cost, val),
        Span::styled(queue_info, val),
    ];

    Text::from(Line::from(spans))
}

pub(crate) fn calculate_cost(usage: &talos_core::message::Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}

fn truncate_end_to_width(s: &str, max_width: u16) -> String {
    let max = max_width as usize;
    if unicode_width::UnicodeWidthStr::width(s) <= max {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut width = 0usize;
    let mut start = chars.len();
    for (i, ch) in chars.iter().enumerate().rev() {
        let cw = unicode_width::UnicodeWidthChar::width(*ch).unwrap_or(0);
        if width + cw > max {
            break;
        }
        width += cw;
        start = i;
    }
    chars[start..].iter().collect()
}

fn summarize_tool_args(tool_name: &str, args_str: &str) -> String {
    let fallback = || args_str.chars().take(80).collect::<String>();
    let obj = serde_json::from_str::<serde_json::Value>(args_str).ok();

    match tool_name {
        "write" | "edit" => {
            if let Some(ref obj) = obj {
                let path = obj.get("path").and_then(|p| p.as_str()).unwrap_or("?");
                let content_len = obj
                    .get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0);
                format!("path: {path} ({content_len} bytes)")
            } else {
                fallback()
            }
        }
        "read" | "delete" => {
            if let Some(ref obj) = obj {
                let path = obj.get("path").and_then(|p| p.as_str()).unwrap_or("?");
                format!("path: {path}")
            } else {
                fallback()
            }
        }
        "grep" => {
            if let Some(ref obj) = obj {
                let pattern = obj.get("pattern").and_then(|p| p.as_str()).unwrap_or("?");
                format!("pattern: {pattern}")
            } else {
                fallback()
            }
        }
        "glob" => {
            if let Some(ref obj) = obj {
                let pattern = obj.get("pattern").and_then(|p| p.as_str()).unwrap_or("?");
                format!("pattern: {pattern}")
            } else {
                fallback()
            }
        }
        "ls" => {
            if let Some(ref obj) = obj {
                let path = obj
                    .get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or(".");
                format!("path: {path}")
            } else {
                fallback()
            }
        }
        "bash" => {
            if let Some(ref obj) = obj {
                let cmd = obj.get("command").and_then(|c| c.as_str()).unwrap_or("?");
                let display = if cmd.len() > 80 {
                    format!("{}...", &cmd[..80])
                } else {
                    cmd.to_string()
                };
                format!("command: {display}")
            } else {
                fallback()
            }
        }
        "find_symbol" => {
            if let Some(ref obj) = obj {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let path = obj.get("path").and_then(|v| v.as_str());
                match path {
                    Some(p) => format!("name: {name} in {p}"),
                    None => format!("name: {name}"),
                }
            } else {
                fallback()
            }
        }
        "find_references" => {
            if let Some(ref obj) = obj {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let file = obj.get("file").and_then(|v| v.as_str()).unwrap_or("?");
                format!("name: {name} in file: {file}")
            } else {
                fallback()
            }
        }
        "list_symbols" => {
            if let Some(ref obj) = obj {
                let path = obj.get("path").and_then(|v| v.as_str()).unwrap_or("?");
                let kind = obj.get("kind").and_then(|v| v.as_str());
                match kind {
                    Some(k) => format!("path: {path}, kind: {k}"),
                    None => format!("path: {path}"),
                }
            } else {
                fallback()
            }
        }
        "list_imports" => {
            if let Some(ref obj) = obj {
                let file = obj.get("file").and_then(|v| v.as_str()).unwrap_or("?");
                format!("file: {file}")
            } else {
                fallback()
            }
        }
        _ => fallback(),
    }
}

fn build_tool_call_scrollback_line(tool_call: &ToolCallDisplay) -> ScrollbackLine {
    let args_str = serde_json::to_string_pretty(&tool_call.arguments)
        .unwrap_or_else(|_| tool_call.arguments.to_string());
    let args_summary = summarize_tool_args(&tool_call.tool_name, &args_str);
    let provenance_marker = match &tool_call.provenance {
        ToolProvenance::Native => None,
        ToolProvenance::McpRemote { server } => Some(format!("[mcp:{}]", server)),
    };
    let accent = to_crossterm_color(semantic::TEXT_ACCENT);
    let prefix_color = to_crossterm_color(semantic::PREFIX_ASSISTANT);
    let dim = to_crossterm_color(semantic::DIM_TEXT);
    let mut segments = vec![
        HistorySegment::styled(
            " → ",
            prefix_color,
            HistoryAttrs {
                bold: true,
                ..HistoryAttrs::default()
            },
        ),
        HistorySegment::styled(
            tool_call.tool_name.to_string(),
            accent,
            HistoryAttrs {
                bold: true,
                ..HistoryAttrs::default()
            },
        ),
    ];
    if let Some(marker) = provenance_marker {
        segments.push(HistorySegment::styled(marker, dim, HistoryAttrs::default()));
    }
    segments.push(HistorySegment::raw(", "));
    segments.push(HistorySegment::styled(
        args_summary,
        dim,
        HistoryAttrs::default(),
    ));
    ScrollbackLine::styled(segments, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_to_width_ascii() {
        assert_eq!(truncate_end_to_width("hello world", 5), "world");
    }

    #[test]
    fn truncate_to_width_cjk() {
        assert_eq!(truncate_end_to_width("你好世界", 4), "世界");
    }

    #[test]
    fn truncate_to_width_short_enough() {
        assert_eq!(truncate_end_to_width("hi", 10), "hi");
    }

    #[test]
    fn stream_render_state_tracks_lines_and_preview() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        assert_eq!(state.push_chunk("first\nsec"), vec![state_line(" ● first")]);
        assert_eq!(state.preview(), "sec");
        assert_eq!(
            state.push_chunk("ond\nthird"),
            vec![state_line("   second")]
        );
        assert_eq!(state.finish(), vec![state_line("   third")]);
        assert!(state.source().is_none());
        assert_eq!(state.preview(), "");
    }

    #[test]
    fn stream_render_state_wraps_user_blocks_with_background_rows() {
        let mut state = StreamRenderState::default();
        let bg = stream_bg_for(Some(&MessageSource::User));

        assert_eq!(
            state.start(MessageSource::User),
            vec![ScrollbackLine::plain(String::new(), bg)]
        );
        assert_eq!(
            state.finish(),
            vec![ScrollbackLine::plain(String::new(), bg)]
        );

        state.reset();
        assert!(state.source().is_none());
        assert_eq!(state.preview(), "");
    }

    #[test]
    fn stream_render_state_can_hold_complete_lines_until_finish() {
        let mut state = StreamRenderState::default();
        assert!(
            state
                .start_with_hold(MessageSource::Assistant, true)
                .is_empty()
        );

        assert!(state.push_chunk("first\nsecond\nthi").is_empty());
        assert_eq!(state.preview(), "thi");
        assert!(state.push_chunk("rd").is_empty());
        assert_eq!(state.preview(), "third");

        assert_eq!(
            state.finish(),
            vec![
                state_line(" ● first"),
                state_line("   second"),
                state_line("   third")
            ]
        );
        assert!(state.source().is_none());
        assert_eq!(state.preview(), "");
    }

    #[test]
    fn stream_render_state_holds_table_and_flushes_aligned_rows() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        assert!(state.push_chunk("| **A** | Longer `code` |\n").is_empty());
        assert_eq!(state.preview(), "rendering table...");
        assert!(state.push_chunk("| --- | --- |\n").is_empty());
        assert_eq!(state.preview(), "rendering table...");
        assert!(state.push_chunk("| x | yy |\n").is_empty());

        let lines = state.finish();
        assert_eq!(lines.len(), 5, "header + sep + 2 rows + footer");
        assert!(lines[0].text.contains("╭"), "rounded top border");
        assert!(lines[2].text.contains("┼"), "separator");
        assert!(lines[4].text.contains("╰"), "rounded bottom border");
        assert!(
            lines[1]
                .segments
                .iter()
                .any(|segment| segment.text == "A" && segment.attrs.bold)
        );
        assert!(
            lines[1]
                .segments
                .iter()
                .any(|segment| segment.text == "code"
                    && segment.fg == to_crossterm_color(semantic::MARKDOWN_CODE))
        );
        assert_eq!(state.preview(), "");
    }

    #[test]
    fn markdown_hold_preview_animates_text_and_color() {
        let status = HoldStatus {
            kind: MarkdownBlockKind::Table,
            lines: 2,
            bytes: 24,
            boundary_hint: crate::stream_markdown::BoundaryHint::TableEnd,
        };

        assert_eq!(animated_hold_preview_text(&status, 0), "rendering table");
        assert_eq!(animated_hold_preview_text(&status, 2), "rendering table.");
        assert_eq!(animated_hold_preview_text(&status, 4), "rendering table..");
        assert_eq!(animated_hold_preview_text(&status, 6), "rendering table...");
        assert_eq!(hold_preview_color(0), hold_preview_color(1));
        assert_ne!(hold_preview_color(0), hold_preview_color(2));
    }

    #[test]
    fn preview_spinner_uses_canon_rhythm() {
        let n = SPINNER_FRAMES.len();
        let phase = n / 2;

        let (p0, _) = preview_spinner_padding(0, 0);
        let (p1, _) = preview_spinner_padding(1, 0);

        let lead0 = SPINNER_FRAMES[(n - phase) % n];
        let chase0 = SPINNER_FRAMES[0];
        assert_eq!(p0, format!(" {lead0}{chase0}"));

        let lead1 = SPINNER_FRAMES[(1 + n - phase) % n];
        let chase1 = SPINNER_FRAMES[1];
        assert_eq!(p1, format!(" {lead1}{chase1}"));

        assert_ne!(lead0, chase0);
    }

    #[test]
    fn stream_render_state_renders_code_fence_on_finish() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        let mut lines = state.push_chunk("```rust\nfn main() {}\n```\n");
        lines.extend(state.finish());

        assert!(!lines.is_empty(), "code block lines returned");
    }

    #[test]
    fn render_code_block_produces_header_and_line_numbers() {
        let block_lines = vec![
            "```rust".to_string(),
            "fn main() {}".to_string(),
            "```".to_string(),
        ];
        let result = render_code_block(&block_lines, None);
        assert_eq!(result.len(), 3, "header + one code line + footer");
        assert!(result[0].text.contains("rust"), "language label");
        assert!(result[1].text.contains("1"), "line number");
        assert!(result[1].text.contains("fn main() {}"), "code content");
    }

    #[test]
    fn stream_render_state_renders_inline_markdown_segments() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        let lines = state.push_chunk("# Title with **strong** and `code`\n");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, " ● Title with strong and code");
        assert!(lines[0].segments.iter().any(|segment| segment.attrs.bold));
        assert!(
            lines[0]
                .segments
                .iter()
                .any(|segment| segment.text == "code"
                    && segment.fg == to_crossterm_color(semantic::MARKDOWN_CODE))
        );
    }

    #[test]
    fn stream_render_state_renders_horizontal_rule() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        let lines = state.push_chunk("---\n");

        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].text.starts_with(" ● ──"),
            "horizontal rule with prefix and dashes"
        );
        assert!(lines[0].text.len() > 40, "horizontal rule should be long");
    }

    #[test]
    fn stream_render_state_styles_block_markdown_rows() {
        let mut state = StreamRenderState::default();
        assert!(state.start(MessageSource::Assistant).is_empty());

        assert!(state.push_chunk("- **first**\n").is_empty());
        assert!(state.push_chunk("- second\n").is_empty());
        let lines = state.finish();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, " ● - first");
        assert_eq!(lines[1].text, "   - second");
        assert!(
            lines[0]
                .segments
                .iter()
                .any(|segment| segment.text == "- " && segment.attrs.bold)
        );
    }

    #[test]
    fn stream_render_state_keeps_user_markdown_literal() {
        let mut state = StreamRenderState::default();
        let bg = stream_bg_for(Some(&MessageSource::User));
        assert_eq!(
            state.start(MessageSource::User),
            vec![ScrollbackLine::plain(String::new(), bg)]
        );

        let lines = state.push_chunk("# literal **user** `input`\n");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, " > # literal **user** `input`");
        assert!(
            lines[0]
                .segments
                .iter()
                .all(|segment| !segment.attrs.italic)
        );
    }

    #[test]
    fn stream_opening_lines_adds_separator_only_after_first_stream() {
        let bg = stream_bg_for(Some(&MessageSource::User));
        let opening = vec![ScrollbackLine::plain(String::new(), bg)];

        assert_eq!(stream_opening_lines(0, opening.clone()), opening);
        assert_eq!(
            stream_opening_lines(1, opening.clone()),
            vec![
                ScrollbackLine::plain(String::new(), None),
                ScrollbackLine::plain(String::new(), bg)
            ]
        );
    }

    #[test]
    fn render_history_message_reuses_completed_stream_rendering() {
        let mut stream_count = 0;
        let lines = render_history_message(
            &mut stream_count,
            MessageSource::Assistant,
            "hello\n| A | B |\n| --- | --- |\n| x | y |",
        );

        assert_eq!(stream_count, 1);
        assert_eq!(lines[0].text, " ● hello");
        assert!(lines.iter().any(|line| line.text.contains("╭")));
        assert!(
            lines
                .iter()
                .any(|line| line.text.contains("│ x") && line.text.contains("│ y"))
        );
    }

    #[test]
    fn hydrate_history_preserves_prefixes_and_stream_count() {
        let mut stream_count = 0;
        let lines = render_history_messages(
            &mut stream_count,
            &[
                Message::User {
                    content: "first\nsecond".to_string(),
                },
                Message::Assistant {
                    content: "reply".to_string(),
                    tool_calls: vec![],
                },
            ],
        );

        let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
        assert!(texts.contains(&" > first"));
        assert!(texts.contains(&"   second"));
        assert!(texts.contains(&" ● reply"));
        assert_eq!(stream_count, 2);
    }

    fn state_line(text: &str) -> ScrollbackLine {
        ScrollbackLine::plain(text, None)
    }
}
