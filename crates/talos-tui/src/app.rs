use std::io;
use std::pin::Pin;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, EventStream, KeyCode, KeyEventKind, MouseEventKind};
use futures::{Stream, StreamExt};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use talos_conversation::{
    ContentOutput, CopyScope, TipKind, TodoPanelData, TurnPhase, UiOutput, UserInput,
};
use talos_core::ApprovalChoice;
use talos_core::message::Message;
use talos_core::tool_filter::ToolSyntaxFilter;
use tokio::{sync::mpsc, time::MissedTickBehavior};

use crate::app_layout::{ComponentMetrics, compute_app_layout};
use crate::evolution::{self, EvolutionPanel};
use crate::history_projection::{HistoryProjection, HistoryScrollState, project_history};
use crate::inline_terminal::{HistoryAttrs, HistorySegment, TerminalSession, ViewportComponent};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, PanelAction, Tip, TuiState};
use crate::theme::{semantic, to_crossterm_color};
use crate::transcript::{TranscriptBlock, TranscriptStore};

pub(crate) use crate::app_stream::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState};

fn history_line(row: &crate::history_projection::RenderedHistoryRow) -> Line<'static> {
    let mut style = Style::default();
    if let Some(bg) = row.line.bg {
        style = style.bg(ratatui_color(bg));
    }
    let spans = row
        .line
        .segments
        .iter()
        .map(|segment| {
            let mut segment_style = style;
            if let Some(fg) = segment.fg {
                segment_style = segment_style.fg(ratatui_color(fg));
            }
            let mut modifiers = Modifier::empty();
            if segment.attrs.bold {
                modifiers |= Modifier::BOLD;
            }
            if segment.attrs.italic {
                modifiers |= Modifier::ITALIC;
            }
            if segment.attrs.underlined {
                modifiers |= Modifier::UNDERLINED;
            }
            if segment.attrs.dim {
                modifiers |= Modifier::DIM;
            }
            Span::styled(segment.text.clone(), segment_style.add_modifier(modifiers))
        })
        .collect::<Vec<_>>();
    Line::from(spans).style(style)
}

fn frame_history_lines(
    history: &HistoryProjection,
    splash: &[Line<'static>],
    frame_start: usize,
    startup_spacer_rows: usize,
) -> Vec<Line<'static>> {
    let splash_len = splash.len();
    let prefix_len = splash_len + startup_spacer_rows;
    let mut lines = Vec::new();

    if frame_start < splash_len {
        lines.extend(splash.iter().skip(frame_start).cloned());
        for _ in 0..startup_spacer_rows {
            lines.push(Line::default());
        }
    } else if frame_start < prefix_len {
        let remaining_spacers = prefix_len - frame_start;
        for _ in 0..remaining_spacers {
            lines.push(Line::default());
        }
    }

    lines.extend(history.rows.iter().map(history_line));
    lines
}

fn ratatui_color(color: crossterm::style::Color) -> ratatui::style::Color {
    use crossterm::style::Color as C;
    use ratatui::style::Color as R;
    match color {
        C::Reset => R::Reset,
        C::Black => R::Black,
        C::DarkGrey => R::DarkGray,
        C::Grey => R::Gray,
        C::White => R::White,
        C::Red => R::Red,
        C::DarkRed => R::Indexed(1),
        C::Green => R::Green,
        C::DarkGreen => R::Indexed(2),
        C::Yellow => R::Yellow,
        C::DarkYellow => R::Indexed(3),
        C::Blue => R::Blue,
        C::DarkBlue => R::Indexed(4),
        C::Magenta => R::Magenta,
        C::DarkMagenta => R::Indexed(5),
        C::Cyan => R::Cyan,
        C::DarkCyan => R::Indexed(6),
        C::Rgb { r, g, b } => R::Rgb(r, g, b),
        C::AnsiValue(value) => R::Indexed(value),
    }
}

const PROCESSING_FRAME_INTERVAL: Duration = Duration::from_millis(150);
const IME_ENTER_WINDOW: Duration = Duration::from_millis(50);
const MOUSE_HISTORY_SCROLL_ROWS: usize = 3;

pub struct Tui {
    state: TuiState,
    terminal: TerminalSession,
    skill_sidebar: SkillSidebar,
    evolution_panel: EvolutionPanel,
    ui_output_rx: Option<mpsc::UnboundedReceiver<UiOutput>>,
    user_input_tx: Option<mpsc::UnboundedSender<UserInput>>,
    /// Logical output awaiting the next application-owned transcript commit.
    pending_transcript: Vec<TranscriptBlock>,
    transcript: TranscriptStore,
    history_scroll: HistoryScrollState,
    last_history_projection: HistoryProjection,
    last_history_viewport_height: u16,
    history_prefix_start: Option<usize>,
    last_frame_history_start: usize,
    last_splash_row_count: usize,
    queued_outputs: Vec<UiOutput>,
    active_stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    ordered_content_open: bool,
    stream_render: StreamRenderState,
    stream_opening_pending: bool,
    pending_stream_opening: Vec<ScrollbackLine>,
    text_filter: ToolSyntaxFilter,
    processing_frame: usize,
    stream_count: usize,
    session_id: Option<String>,
    last_char_time: Option<Instant>,
    first_message_dispatched: bool,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let _ = crossterm::terminal::disable_raw_mode();

        let terminal = TerminalSession::new()?;

        Ok(Self {
            state: TuiState::new(),
            terminal,
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: EvolutionPanel::new(),
            ui_output_rx: None,
            user_input_tx: None,
            pending_transcript: Vec::new(),
            transcript: TranscriptStore::default(),
            history_scroll: HistoryScrollState::follow_tail(),
            last_history_projection: HistoryProjection::default(),
            last_history_viewport_height: 0,
            history_prefix_start: None,
            last_frame_history_start: 0,
            last_splash_row_count: 0,
            queued_outputs: Vec::new(),
            active_stream: None,
            ordered_content_open: false,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            text_filter: ToolSyntaxFilter::new(),
            processing_frame: 0,
            stream_count: 0,
            session_id: None,
            last_char_time: None,
            first_message_dispatched: false,
        })
    }

    /// Creates a minimal Tui for unit testing key dispatch without terminal access.
    #[cfg(test)]
    pub(crate) fn for_test(
        state: TuiState,
        user_input_tx: Option<mpsc::UnboundedSender<UserInput>>,
    ) -> Self {
        Self {
            state,
            terminal: TerminalSession::test_instance(),
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: EvolutionPanel::new(),
            ui_output_rx: None,
            user_input_tx,
            pending_transcript: Vec::new(),
            transcript: TranscriptStore::default(),
            history_scroll: HistoryScrollState::follow_tail(),
            last_history_projection: HistoryProjection::default(),
            last_history_viewport_height: 0,
            history_prefix_start: None,
            last_frame_history_start: 0,
            last_splash_row_count: 0,
            queued_outputs: Vec::new(),
            active_stream: None,
            ordered_content_open: false,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            text_filter: ToolSyntaxFilter::default(),
            processing_frame: 0,
            stream_count: 0,
            session_id: None,
            last_char_time: None,
            first_message_dispatched: false,
        }
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

    pub fn set_provider(&mut self, provider: String) {
        self.state.status.provider = provider;
    }

    pub fn set_workspace_path(&mut self, path: String) {
        self.state.status.workspace_path = path;
    }

    pub fn set_session_id(&mut self, id: String) {
        self.session_id = Some(id);
    }

    fn dispatch_panel_action(&mut self, action: PanelAction) {
        match action {
            PanelAction::SendMessage(msg) => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::Message(msg));
                }
            }
            PanelAction::ProviderSetup(provider) => {
                self.state
                    .open_credential_input(&provider, None, false, None);
                self.state.input_clear();
            }
            PanelAction::SwitchModel {
                provider,
                model_id,
                variant,
            } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::SwitchModel {
                        provider,
                        model_id,
                        variant,
                    });
                }
            }
            PanelAction::ConnectSelect { provider } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::ConnectSelect { provider });
                }
            }
            PanelAction::RegisterCustomProvider {
                name,
                protocol,
                base_url,
                api_key,
            } => {
                if let Some(ref tx) = self.user_input_tx {
                    let _ = tx.send(UserInput::RegisterCustomProvider {
                        name,
                        protocol,
                        base_url,
                        api_key,
                    });
                }
            }
            PanelAction::None => {}
        }
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
                Message::Assistant {
                    content,
                    tool_calls,
                    reasoning,
                    ..
                } => {
                    if let Some(ar) = reasoning
                        && let Some(text) = talos_core::message::project_displayable_reasoning(ar)
                    {
                        let display_text = format!("Thinking: {text}\n");
                        self.handle_ui_output(UiOutput::Content(ContentOutput::Block {
                            source: talos_conversation::MessageSource::Reasoning,
                            text: display_text,
                        }));
                    }

                    let tool_calls_in_text =
                        talos_core::message::extract_tool_calls_from_text(content);
                    let cleaned = talos_core::message::strip_tool_syntax(content);
                    let has_tool_calls = !tool_calls.is_empty() || !tool_calls_in_text.is_empty();

                    pending_tool_names.clear();
                    for tc in tool_calls {
                        pending_tool_names.push(tc.name.clone());
                    }

                    if !has_tool_calls && !cleaned.is_empty() {
                        self.handle_ui_output(UiOutput::Content(ContentOutput::Block {
                            source: talos_conversation::MessageSource::Assistant,
                            text: cleaned,
                        }));
                    }

                    let calls: Vec<ToolCallDisplay> = if !tool_calls.is_empty() {
                        tool_calls
                            .iter()
                            .map(|tc| ToolCallDisplay {
                                tool_name: tc.name.clone(),
                                arguments: tc.input.clone(),
                                provenance: ToolProvenance::Native,
                                summary_fields: crate::scrollback::summary_fields_for(&tc.name),
                            })
                            .collect()
                    } else if !tool_calls_in_text.is_empty() {
                        for tc in &tool_calls_in_text {
                            pending_tool_names.push(tc.name.clone());
                        }
                        tool_calls_in_text
                            .iter()
                            .map(|tc| ToolCallDisplay {
                                tool_name: tc.name.clone(),
                                arguments: tc.input.clone(),
                                provenance: ToolProvenance::Native,
                                summary_fields: crate::scrollback::summary_fields_for(&tc.name),
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    for call in &calls {
                        self.handle_ui_output(UiOutput::ToolCall(call.clone()));
                    }
                }
                Message::User { content } => {
                    self.handle_ui_output(UiOutput::Content(ContentOutput::Block {
                        source: talos_conversation::MessageSource::User,
                        text: content.clone(),
                    }));
                }
                Message::System { content, .. } if !content.is_empty() => {
                    self.handle_ui_output(UiOutput::Content(ContentOutput::Block {
                        source: talos_conversation::MessageSource::System,
                        text: content.clone(),
                    }));
                }
                Message::Context { content } if !content.is_empty() => {
                    self.handle_ui_output(UiOutput::Content(ContentOutput::Block {
                        source: talos_conversation::MessageSource::System,
                        text: content.clone(),
                    }));
                }
                _ => {}
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
            ApprovalChoice::ApproveOnce => (
                "\u{2713}",
                to_crossterm_color(semantic::TEXT_SUCCESS),
                "approved",
            ),
            ApprovalChoice::AlwaysApprove => (
                "\u{2713}",
                to_crossterm_color(semantic::TEXT_SUCCESS),
                "always approved",
            ),
            ApprovalChoice::Deny => (
                "\u{2717}",
                to_crossterm_color(semantic::TEXT_ERROR),
                "denied",
            ),
        };
        self.pending_transcript
            .push(TranscriptBlock::StyledLine(ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("   {icon} {msg}"),
                    color,
                    HistoryAttrs::default(),
                )],
                None,
            )));
        let _ = self.commit_pending_transcript();

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
        let mut render_interval = tokio::time::interval(PROCESSING_FRAME_INTERVAL);
        render_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut ui_output_rx = self.ui_output_rx.take().expect("ui_output_rx not set");

        self.draw_frame()?;

        loop {
            self.state.expire_tip();
            self.commit_pending_transcript()?;
            self.draw_frame()?;

            tokio::select! {
                _ = render_interval.tick() => self.advance_processing_frame(),
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
                                self.commit_pending_transcript()?;
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
                            self.commit_pending_transcript()?;
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
        self.restore()?;
        self.print_exit_summary(elapsed);
        Ok(())
    }

    fn print_exit_summary(&mut self, elapsed: Duration) {
        for line in crate::app_summary::build_exit_summary_lines(
            &self.state.status,
            elapsed,
            self.stream_count,
            self.session_id.as_deref(),
        ) {
            println!("{}", line.text);
        }
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
            self.append_styled_lines(lines);
        }
        self.active_stream = None;
    }

    fn finalize_ordered_content(&mut self) {
        if !self.ordered_content_open {
            return;
        }
        let lines = self.stream_render.finish();
        if self.stream_opening_pending {
            self.stream_opening_pending = false;
            self.pending_stream_opening.clear();
        } else {
            self.append_styled_lines(lines);
        }
        self.ordered_content_open = false;
    }

    fn consume_stream_chunk(&mut self, chunk: &str) {
        let filter_out = self.text_filter.push_chunk(chunk);

        if filter_out.tool_call_started && self.active_stream.is_some() {
            self.finalize_active_stream();
        }

        if !filter_out.text.is_empty() {
            if self.stream_opening_pending {
                let opening = crate::scrollback::stream_opening_lines(
                    self.stream_count,
                    std::mem::take(&mut self.pending_stream_opening),
                );
                self.append_styled_lines(opening);
                self.stream_opening_pending = false;
                self.stream_count += 1;
            }
            let lines = self.stream_render.push_chunk(&filter_out.text);
            self.append_styled_lines(lines);
        }
    }

    fn handle_ui_output(&mut self, output: UiOutput) -> bool {
        match output {
            UiOutput::Content(content) => match content {
                ContentOutput::Start { source } => {
                    if self.active_stream.is_some() {
                        self.finalize_active_stream();
                    }
                    self.finalize_ordered_content();
                    self.pending_stream_opening = self.stream_render.start(source);
                    self.stream_opening_pending = true;
                    self.ordered_content_open = true;
                }
                ContentOutput::Delta { text } => {
                    if self.ordered_content_open {
                        self.consume_stream_chunk(&text);
                    }
                }
                ContentOutput::End => self.finalize_ordered_content(),
                ContentOutput::Block { source, text } => {
                    if self.active_stream.is_some() {
                        self.finalize_active_stream();
                    }
                    self.finalize_ordered_content();
                    let lines = crate::scrollback::render_history_message(
                        &mut self.stream_count,
                        source,
                        &text,
                    );
                    self.append_styled_lines(lines);
                }
            },
            UiOutput::Stream(msg) => {
                self.finalize_ordered_content();
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
                self.pending_stream_opening = self.stream_render.start(msg.source.clone());
                self.stream_opening_pending = true;
                self.active_stream = Some(msg.stream);
            }
            UiOutput::Reasoning(text) => {
                let lines = crate::scrollback::render_history_message(
                    &mut self.stream_count,
                    talos_conversation::MessageSource::Reasoning,
                    &text,
                );
                self.append_styled_lines(lines);
            }
            UiOutput::ToolCallStarted { .. } => {
                self.finalize_ordered_content();
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
            }
            UiOutput::ToolCall(display) => {
                self.pending_transcript
                    .push(TranscriptBlock::ToolCall(display));
            }
            UiOutput::ToolResult(display) => {
                let icon = if display.is_error { "✗" } else { "" };
                let color = if display.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                let _ = (icon, color);
                self.pending_transcript
                    .push(TranscriptBlock::ToolResult(display));
            }
            UiOutput::TodoPanel(data) => {
                self.append_styled_lines(build_todo_panel_lines(&data));
            }
            UiOutput::ThinkingPreview { text } => {
                self.state.thinking_preview = text;
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
                let workspace_path = std::mem::take(&mut self.state.status.workspace_path);
                self.state.status = snapshot;
                if self.state.status.workspace_path.is_empty() {
                    self.state.status.workspace_path = workspace_path;
                }
            }
            UiOutput::SessionIdentity { id } => {
                self.session_id = Some(id);
            }
            UiOutput::Tip { text, kind } => {
                self.state.tip = Some(Tip {
                    ttl: tip_ttl(&kind),
                    kind,
                    text,
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
            UiOutput::SessionNew(_)
            | UiOutput::SessionResume(_)
            | UiOutput::SessionFork(_)
            | UiOutput::SessionDelete(_)
            | UiOutput::TodoCommand(_)
            | UiOutput::ModelSwitchRequest(_)
            | UiOutput::SkillCommand(_)
            | UiOutput::CredentialResponse(_) => {
                // Handled by the bridge → mode runner lifecycle handler.
                // Should not reach the TUI directly.
            }
            UiOutput::SessionPicker(sessions) => {
                self.state.open_session_picker(&sessions);
            }
            UiOutput::ModelPicker(data) => {
                self.state.open_model_picker(&data);
            }
            UiOutput::ConnectPicker(data) => {
                self.state.open_connect_picker(&data);
            }
            UiOutput::ConnectProviderRequest { .. } => {}
            UiOutput::SteeringQueueSnapshot(snapshot) => {
                self.state.steering_queue_snapshot = Some(snapshot);
            }
            UiOutput::CredentialRequest(req) => {
                self.state.open_credential_input(
                    &req.provider,
                    req.model_id.as_deref(),
                    req.connect_mode,
                    req.default_base_url.clone(),
                );
            }
            UiOutput::HydrateHistory(messages) => {
                self.finalize_ordered_content();
                self.finalize_active_stream();
                self.commit_pending_transcript().ok();
                self.hydrate_history(&messages);
                self.commit_pending_transcript().ok();
            }
            UiOutput::AttachImageRequest { .. } => {
                // Handled by the bridge — should not reach the TUI directly.
            }
        }
        false
    }

    fn append_styled_lines(&mut self, lines: impl IntoIterator<Item = ScrollbackLine>) {
        self.pending_transcript
            .extend(lines.into_iter().map(TranscriptBlock::StyledLine));
    }

    fn commit_pending_transcript(&mut self) -> io::Result<()> {
        for block in std::mem::take(&mut self.pending_transcript) {
            self.transcript.append(block);
        }
        Ok(())
    }

    fn advance_processing_frame(&mut self) {
        self.processing_frame =
            next_processing_frame(self.state.status.is_processing, self.processing_frame);
    }

    fn is_startup_mode(&self) -> bool {
        self.transcript.entries().is_empty()
            && !self.first_message_dispatched
            && !self.state.slash_menu.is_open
            && matches!(self.state.approval_state, ApprovalState::Hidden)
    }

    fn draw_frame(&mut self) -> io::Result<()> {
        let state = &self.state;
        let status = &state.status;

        let (preview_padding, spinner_color) = if status.is_processing {
            let (padding, color_idx) =
                crate::scrollback::preview_spinner_padding(self.processing_frame);
            (padding, Some(semantic::PROCESSING_SPINNER[color_idx]))
        } else {
            self.processing_frame = 0;
            ("   ".to_string(), None)
        };
        let hold_status = self.stream_render.hold_status().cloned();
        let preview_text = preview_text_for_state(
            hold_status.as_ref(),
            status.phase.as_ref(),
            self.state.thinking_preview.as_deref(),
            status.is_processing,
            self.stream_render.preview(),
            self.processing_frame,
        );
        let preview_text_color = hold_status
            .as_ref()
            .map(|_| crate::scrollback::hold_preview_color(self.processing_frame));
        let thinking_label_frame = self
            .state
            .thinking_preview
            .as_ref()
            .filter(|_| status.is_processing && hold_status.is_none())
            .map(|_| self.processing_frame);
        let preview = crate::scrollback::PreviewComponent {
            padding: &preview_padding,
            text: &preview_text,
            spinner_color,
            text_color: preview_text_color,
            thinking_label_frame,
        };
        let tips = crate::scrollback::TipsComponent {
            tip: state.tip.as_ref(),
        };
        let input_pad_top = crate::scrollback::InputPadComponent;
        let input_pad_bot = crate::scrollback::InputPadComponent;
        let query_for_panel = state.panel_query();
        let mut bottom_panel = crate::scrollback::BottomPanelComponent {
            menu: &state.slash_menu,
            query: query_for_panel,
            max_height: u16::MAX,
        };

        let screen_size = self.terminal.size()?;
        let width = screen_size.width;
        let is_startup = self.is_startup_mode();
        let splash = crate::splash::viewport_splash_lines(width);
        let splash_rows = splash.len();
        let startup_spacer_rows: usize = if is_startup { 1 } else { 0 };
        let follows_tail = matches!(
            self.history_scroll.mode,
            crate::history_projection::HistoryScrollMode::FollowTail
        );
        // A short FollowTail conversation is a single vertical flow: Logo,
        // transcript, then the input frame.  `history_cap` makes AppLayout
        // allocate only that natural history height.  Once it cannot fit, the
        // normal allocation clamps it to the remaining frame and the composer
        // naturally becomes bottom-fixed.  Anchored history deliberately uses
        // the regular viewport so scrolling never moves the input frame.
        let history_cap = if is_startup {
            Some((splash_rows + startup_spacer_rows) as u16)
        } else if follows_tail
            && !self.state.slash_menu.is_open
            && matches!(self.state.approval_state, ApprovalState::Hidden)
        {
            let natural_rows = project_history(
                &self.transcript,
                screen_size.width,
                screen_size.height,
                &self.history_scroll,
            )
            .total_rows;
            Some(
                splash_rows
                    .saturating_add(natural_rows)
                    .min(u16::MAX as usize) as u16,
            )
        } else {
            None
        };
        let status_comp = crate::scrollback::StatusComponent { status, width };

        let preview_h = if is_startup {
            0
        } else {
            preview.height_hint(width)
        };
        // Tips remain visible during startup so transient service notices (for
        // example the loopback Dashboard address) have a stable, copyable
        // location before the first message is submitted.
        let tips_h = tips.height_hint(width);

        let input_natural = crate::scrollback::InputComponent {
            state,
            max_height: crate::scrollback::MAX_COMPOSER_LINES,
        }
        .height_hint(width);
        let modal_natural = bottom_panel.height_hint(width);
        let fixed_heights = preview_h
            + tips_h
            + input_pad_top.height_hint(width)
            + input_pad_bot.height_hint(width)
            + status_comp.height_hint(width);
        let queue_natural = crate::scrollback::QueuePreviewComponent {
            snapshot: state.steering_queue_snapshot.as_ref(),
            followup_count: status.followup_count,
            max_rows: 6,
        }
        .height_hint(width);

        let content_budget = screen_size.height.saturating_sub(fixed_heights);
        let compressed = crate::scrollback::compress_layout(
            content_budget,
            modal_natural,
            input_natural,
            queue_natural,
        );
        bottom_panel.max_height = compressed.panel_max_height;

        let input = crate::scrollback::InputComponent {
            state,
            max_height: compressed.input_max_height,
        };
        let queue = crate::scrollback::QueuePreviewComponent {
            snapshot: state.steering_queue_snapshot.as_ref(),
            followup_count: status.followup_count,
            max_rows: compressed.queue_max_rows,
        };

        let actual_input_h = input.height_hint(width);
        // `bottom_panel_placement` adds its third argument itself, so this base
        // deliberately excludes the panel height.
        let base_height = fixed_heights + actual_input_h + queue.height_hint(width);
        let menu_placement = crate::scrollback::bottom_panel_placement(
            screen_size.height,
            base_height,
            modal_natural,
        );

        let app_layout = compute_app_layout(
            screen_size,
            ComponentMetrics {
                preview: preview_h,
                queue: queue.height_hint(width),
                tips: tips_h,
                panel_required: if state.slash_menu.is_credential_input()
                    || state.slash_menu.is_provider_wizard()
                    || !matches!(state.approval_state, ApprovalState::Hidden)
                {
                    bottom_panel.height_hint(width).min(4)
                } else {
                    0
                },
                panel_preferred: bottom_panel.height_hint(width),
                composer: actual_input_h,
                history_cap,
            },
            menu_placement,
        );
        let history_height = app_layout.history.map_or(0, |rect| rect.height);
        self.last_history_viewport_height = history_height;
        let history = project_history(
            &self.transcript,
            screen_size.width,
            history_height,
            &self.history_scroll,
        );
        self.last_history_projection = history.clone();
        if let Some(prefix_start) = self.history_prefix_start.as_mut() {
            *prefix_start = (*prefix_start).min(splash_rows.saturating_sub(1));
        }
        let follow_tail = follows_tail;
        let natural_start = if history.total_rows == 0 {
            0
        } else if follow_tail {
            splash_rows
                .saturating_add(history.total_rows)
                .saturating_sub(usize::from(history_height))
        } else {
            splash_rows.saturating_add(history.visible_start)
        };
        let frame_history_start = self.history_prefix_start.unwrap_or(natural_start);
        self.last_frame_history_start = frame_history_start;
        self.last_splash_row_count = splash_rows;

        self.terminal.draw(screen_size, |frame| {
            if let Some(area) = app_layout.history {
                let history_text = frame_history_lines(
                    &history,
                    &splash,
                    frame_history_start,
                    startup_spacer_rows,
                );
                frame.render_widget(Paragraph::new(history_text), area);
            }
            if let Some(area) = app_layout.preview {
                preview.render(frame, area);
            }
            if let Some(area) = app_layout.queue {
                queue.render(frame, area);
            }
            if let Some(area) = app_layout.tips {
                tips.render(frame, area);
            }
            if let Some(area) = app_layout.panel {
                bottom_panel.render(frame, area);
            }
            if let Some(area) = app_layout.composer_top_pad {
                input_pad_top.render(frame, area);
            }
            if let Some(area) = app_layout.composer {
                input.render(frame, area);
            }
            if let Some(area) = app_layout.composer_bottom_pad {
                input_pad_bot.render(frame, area);
            }
            if let Some(area) = app_layout.status {
                status_comp.render(frame, area);
            }
        })?;

        {
            let screen_w = screen_size.width;
            if self.state.slash_menu.is_credential_input() {
                let (Some(panel), Some(local)) = (
                    app_layout.panel,
                    crate::scrollback::credential_cursor_position(&self.state.slash_menu),
                ) else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                self.terminal
                    .set_cursor_if_visible_in_rect(panel, local.col, local.row)?;
            } else if self.state.slash_menu.is_provider_wizard() {
                let (Some(panel), Some(local)) = (
                    app_layout.panel,
                    crate::scrollback::provider_wizard_local_cursor_position(
                        &self.state.slash_menu,
                    ),
                ) else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                self.terminal
                    .set_cursor_if_visible_in_rect(panel, local.col, local.row)?;
            } else {
                let Some(composer_rect) = app_layout.composer else {
                    self.terminal.hide_cursor()?;
                    return Ok(());
                };
                let byte_pos = self.state.cursor_byte_pos();
                let content_w = crate::scrollback::composer_content_width(screen_w);
                let (cursor_row_offset, cursor_col_offset) =
                    crate::scrollback::cursor_line_col_with_width(
                        &self.state.input_buffer[..byte_pos],
                        content_w,
                    );
                let scroll_offset = crate::scrollback::composer_scroll_offset(
                    &self.state.input_buffer[..byte_pos],
                    &self.state.input_buffer,
                    content_w,
                    composer_rect.height,
                );
                let local_row = cursor_row_offset.saturating_sub(scroll_offset);
                let cursor_col = crate::scrollback::COMPOSER_LEFT_PAD + cursor_col_offset;
                self.terminal
                    .set_cursor_in_rect(composer_rect, cursor_col, local_row)?;
            }
        }

        Ok(())
    }

    fn anchor_frame_history_start(&mut self, frame_start: usize) {
        if frame_start < self.last_splash_row_count {
            self.history_prefix_start = Some(frame_start);
            if let Some(anchor) = self.last_history_projection.first_anchor() {
                self.history_scroll.anchor(anchor, 0);
            } else {
                self.history_scroll.jump_to_end();
            }
            return;
        }

        self.history_prefix_start = None;
        let transcript_index = frame_start.saturating_sub(self.last_splash_row_count);
        if let Some(anchor) = self.last_history_projection.anchor_at(transcript_index) {
            self.history_scroll.anchor(anchor, 0);
        }
    }

    fn scroll_frame_history_up(&mut self, rows: usize) {
        self.anchor_frame_history_start(self.last_frame_history_start.saturating_sub(rows));
    }

    fn scroll_frame_history_down(&mut self, rows: usize, height: u16) {
        let total_rows = self
            .last_splash_row_count
            .saturating_add(self.last_history_projection.total_rows);
        let max_start = total_rows.saturating_sub(usize::from(height));
        let target = self
            .last_frame_history_start
            .saturating_add(rows)
            .min(max_start);

        if target >= max_start {
            self.history_prefix_start = None;
            self.history_scroll.jump_to_end();
        } else {
            self.anchor_frame_history_start(target);
        }
    }

    fn handle_input_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                match key.code {
                    KeyCode::PageUp => {
                        self.history_prefix_start = None;
                        let height = self.last_history_viewport_height;
                        if height == 0 {
                            return false;
                        }
                        if let Some(anchor) = self.last_history_projection.page_up(height) {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::PageDown => {
                        self.history_prefix_start = None;
                        let height = self.last_history_viewport_height;
                        if height == 0 {
                            return false;
                        }
                        if self.last_history_projection.page_down_reaches_tail(height) {
                            self.history_scroll.jump_to_end();
                        } else if let Some(anchor) = self.last_history_projection.page_down(height)
                        {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::Home if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.history_prefix_start = None;
                        if let Some(anchor) = self.last_history_projection.first_anchor() {
                            self.history_scroll.anchor(anchor, 0);
                        }
                        return false;
                    }
                    KeyCode::End if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.history_prefix_start = None;
                        self.history_scroll.jump_to_end();
                        return false;
                    }
                    _ => {}
                }
                if !matches!(self.state.approval_state, ApprovalState::Hidden) {
                    self.handle_pending_approval_input(key.code);
                    return false;
                }
                if self.state.slash_menu.is_credential_input() {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(t) = self.last_char_time
                                && t.elapsed() < IME_ENTER_WINDOW
                            {
                                return false;
                            }
                            if let Some(resp) = self.state.credential_submit()
                                && let Some(ref tx) = self.user_input_tx
                            {
                                let _ = tx.send(UserInput::Credential(resp));
                            }
                            self.state.input_clear();
                        }
                        KeyCode::Esc => {
                            self.state.credential_cancel();
                            self.state.input_clear();
                        }
                        KeyCode::Backspace => {
                            self.state.credential_backspace();
                        }
                        KeyCode::Char(c) => {
                            self.last_char_time = Some(Instant::now());
                            self.state.credential_append_char(c);
                        }
                        _ => {}
                    }
                    return false;
                }
                if self.state.slash_menu.is_provider_wizard() {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(t) = self.last_char_time
                                && t.elapsed() < IME_ENTER_WINDOW
                            {
                                return false;
                            }
                            if let Some(action) = self.state.wizard_advance() {
                                self.dispatch_panel_action(action);
                            }
                            self.state.input_clear();
                        }
                        KeyCode::Esc => {
                            self.state.wizard_cancel();
                            self.state.input_clear();
                        }
                        KeyCode::Backspace => {
                            self.state.wizard_backspace();
                        }
                        KeyCode::Up | KeyCode::Down => {
                            self.state.wizard_cycle_protocol();
                        }
                        KeyCode::Char(c) => {
                            self.last_char_time = Some(Instant::now());
                            self.state.wizard_append_char(c);
                        }
                        _ => {}
                    }
                    return false;
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        let was_processing = self.state.status.is_processing;
                        if was_processing {
                            // Active-turn cancellation and the idle double-Ctrl+C exit gesture
                            // are separate state machines. A queued message can start another turn
                            // immediately after cancellation, so never leave that turn armed as
                            // the first press of the idle exit gesture.
                            self.state.ctrl_c_state = CtrlCState::Idle;
                            self.state.tip = Some(Tip {
                                kind: TipKind::ExitHint,
                                text: "Turn cancellation requested.".to_string(),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
                            if let Some(ref tx) = self.user_input_tx {
                                let _ = tx.send(UserInput::Cancel);
                            }
                            return false;
                        }
                        if !self.state.input_buffer.is_empty() {
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
                        return self.state.handle_ctrl_c();
                    }
                    KeyCode::Char('a') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.state.input_cursor_to_line_start();
                    }
                    KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.state.input_cursor_to_line_end();
                    }
                    KeyCode::Char('g') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.slash_menu.close();
                        self.toggle_evolution_panel();
                    }
                    KeyCode::Up if self.state.slash_menu.is_open => {
                        let query = self.state.panel_query().to_string();
                        self.state.slash_menu.select_prev(&query);
                    }
                    KeyCode::Down if self.state.slash_menu.is_open => {
                        let query = self.state.panel_query().to_string();
                        self.state.slash_menu.select_next(&query);
                    }
                    KeyCode::Up if !self.state.slash_menu.is_open => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.history_prev();
                    }
                    KeyCode::Down if !self.state.slash_menu.is_open => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.history_next();
                    }
                    KeyCode::Tab if self.state.slash_menu.is_open => {
                        let action = self.state.complete_selected_panel_item();
                        self.dispatch_panel_action(action);
                    }
                    KeyCode::Enter if self.state.slash_menu.is_open => {
                        if let Some(t) = self.last_char_time
                            && t.elapsed() < IME_ENTER_WINDOW
                        {
                            return false;
                        }
                        let action = self.state.accept_selected_panel_item();
                        self.dispatch_panel_action(action);
                    }
                    KeyCode::Esc if self.state.slash_menu.is_open => {
                        self.state.slash_menu.close();
                    }
                    KeyCode::Char('j')
                        if key.modifiers.contains(event::KeyModifiers::CONTROL)
                            && !self.state.slash_menu.is_open =>
                    {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_append_char('\n');
                    }
                    KeyCode::Char('/') if self.state.input_buffer.is_empty() => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        let registry = talos_conversation::command_registry();
                        self.state.open_slash_menu(registry);
                    }
                    KeyCode::Char(c) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.last_char_time = Some(Instant::now());
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
                        self.state.input_cursor_left();
                    }
                    KeyCode::Right => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_right();
                    }
                    KeyCode::Enter => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                            self.state.input_append_char('\n');
                        } else {
                            let sent = submit_input_message(
                                &mut self.state,
                                &mut self.stream_render,
                                self.user_input_tx.as_ref(),
                            );
                            if sent {
                                self.first_message_dispatched = true;
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                    }
                    _ => {}
                }
            }
            Event::Paste(text) => {
                self.state.input_paste(text);
            }
            Event::Mouse(mouse) => {
                let height = self.last_history_viewport_height;
                if height == 0 {
                    return false;
                }
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
                    }
                    MouseEventKind::ScrollDown => {
                        self.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, height);
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }

    fn restore(&mut self) -> io::Result<()> {
        self.terminal.restore()
    }
}

fn submit_input_message(
    state: &mut TuiState,
    stream_render: &mut StreamRenderState,
    user_input_tx: Option<&mpsc::UnboundedSender<UserInput>>,
) -> bool {
    let input = state.input_submit();
    if input.is_empty() {
        return false;
    }

    let Some(tx) = user_input_tx else {
        return false;
    };

    if tx.send(UserInput::Message(input)).is_err() {
        return false;
    }

    // Clear stale preview state from any prior cancellation/resume only after a
    // new user message has actually been accepted for dispatch (TUI-028).
    stream_render.reset();
    state.thinking_preview = None;
    true
}

pub(crate) fn preview_text_for_state(
    hold_status: Option<&crate::stream_markdown::HoldStatus>,
    phase: Option<&TurnPhase>,
    thinking_preview: Option<&str>,
    is_processing: bool,
    stream_preview: &str,
    processing_frame: usize,
) -> String {
    if let Some(status) = hold_status {
        return crate::scrollback::animated_hold_preview_text(status, processing_frame);
    }

    // Terminal phases remain visible in the status bar, but the preview is reserved for an active
    // turn. Otherwise a cancelled/failed/timed-out label persists above the composer until the
    // next turn replaces the engine phase.
    if is_processing && matches!(phase, Some(TurnPhase::TimedOut)) {
        return "⏱ timed out".to_string();
    }
    if is_processing && matches!(phase, Some(TurnPhase::Failed)) {
        return "✗ failed".to_string();
    }
    if is_processing && matches!(phase, Some(TurnPhase::Cancelled)) {
        return "cancelled".to_string();
    }
    if let Some(TurnPhase::Retrying { attempt }) = phase {
        return format!("retrying (attempt {attempt})...");
    }
    if let Some(TurnPhase::RunningTool { name }) = phase
        && is_processing
    {
        return format!("running tool: {name}...");
    }

    if let Some(thinking) = thinking_preview
        && is_processing
    {
        let display = crate::scrollback::extract_thinking_title(thinking).unwrap_or(thinking);
        return format!("thinking: {display}");
    }

    if matches!(phase, Some(TurnPhase::Connecting)) && is_processing {
        return "connecting...".to_string();
    }

    if is_processing && stream_preview.is_empty() {
        return crate::scrollback::idle_processing_preview_text(processing_frame).to_string();
    }

    stream_preview.to_string()
}

/// Map a panel row status string to its display form.
/// Known checkbox icons (`[ ]`, `[~]`, `[x]`, `[!]`) pass through as-is.
/// Unknown strings get the bracket fallback `[{status}]`.
fn status_display(status: &str) -> String {
    match status {
        "[ ]" | "[~]" | "[x]" | "[!]" => status.to_string(),
        other => format!("[{other}]"),
    }
}

pub(crate) fn build_todo_panel_lines(data: &TodoPanelData) -> Vec<ScrollbackLine> {
    let header = ScrollbackLine::styled(
        vec![
            HistorySegment::styled(
                "   TODO ",
                to_crossterm_color(semantic::TEXT_ACCENT),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ),
            HistorySegment::styled(
                data.title.clone(),
                to_crossterm_color(semantic::TEXT_PRIMARY),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ),
        ],
        None,
    );
    let mut lines = vec![header];

    if data.rows.is_empty() {
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                "      (no todo rows)",
                to_crossterm_color(semantic::DIM_TEXT),
                HistoryAttrs::default(),
            )],
            None,
        ));
    } else {
        for row in &data.rows {
            let mut segments = vec![
                HistorySegment::styled(
                    format!("   {} ", row.id),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    status_display(&row.status),
                    to_crossterm_color(semantic::TEXT_ACCENT),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    format!("[{}] ", row.priority),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    row.title.clone(),
                    to_crossterm_color(semantic::TEXT_PRIMARY),
                    HistoryAttrs::default(),
                ),
            ];
            if let Some(detail) = &row.detail {
                segments.push(HistorySegment::styled(
                    format!(" — {detail}"),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                ));
            }
            lines.push(ScrollbackLine::styled(segments, None));
        }
    }

    if let Some(footer) = &data.footer {
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("      {footer}"),
                to_crossterm_color(semantic::DIM_TEXT),
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    lines
}

pub(crate) fn next_processing_frame(is_processing: bool, processing_frame: usize) -> usize {
    if is_processing {
        processing_frame.wrapping_add(1)
    } else {
        0
    }
}

pub(crate) fn tip_ttl(kind: &TipKind) -> Duration {
    match kind {
        TipKind::Info => Duration::from_secs(8),
        TipKind::Error => Duration::from_secs(5),
        _ => Duration::from_secs(3),
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[allow(warnings)]
#[cfg(test)]
mod app_tests;
