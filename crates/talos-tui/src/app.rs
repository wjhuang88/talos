use std::io;
use std::net::SocketAddr;
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
use crate::history_projection::{
    HistoryProjection, HistoryProjectionCache, HistoryScrollMode, HistoryScrollState,
    HistorySelectionPoint,
};
use crate::inline_terminal::{HistoryAttrs, HistorySegment, TerminalSession, ViewportComponent};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, PanelAction, Tip, TuiState};
use crate::theme::{semantic, to_crossterm_color};
use crate::transcript::{TranscriptBlock, TranscriptStore};

pub(crate) use crate::app_stream::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState};

mod frame;
mod input;
mod output;

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
    history_projection_cache: HistoryProjectionCache,
    last_history_projection: HistoryProjection,
    last_history_viewport_height: u16,
    last_history_area: Option<ratatui::layout::Rect>,
    history_prefix_start: Option<usize>,
    last_frame_history_start: usize,
    last_splash_row_count: usize,
    last_history_prefix_row_count: usize,
    queued_outputs: Vec<UiOutput>,
    active_stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    ordered_content_open: bool,
    stream_render: StreamRenderState,
    stream_opening_pending: bool,
    pending_stream_opening: Vec<ScrollbackLine>,
    tool_placeholder_gate: crate::app::output::ToolPlaceholderGate,
    text_filter: ToolSyntaxFilter,
    processing_frame: usize,
    stream_count: usize,
    session_id: Option<String>,
    last_char_time: Option<Instant>,
    first_message_dispatched: bool,
    selection: Option<SelectionState>,
    dashboard_availability: Option<crate::splash::DashboardAvailability>,
    approval_viewport_snapshot: Option<ApprovalViewportSnapshot>,
    approval_preview_fully_visible: bool,
}

#[derive(Clone, Debug)]
struct ApprovalViewportSnapshot {
    history_scroll: HistoryScrollState,
    history_prefix_start: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectionState {
    anchor: (u16, u16),
    focus: (u16, u16),
    dragging: bool,
    edge: i8,
    history_anchor: Option<HistorySelectionPoint>,
    history_focus: Option<HistorySelectionPoint>,
}

impl SelectionState {
    fn points(self) -> ((u16, u16), (u16, u16)) {
        let anchor = (self.anchor.1, self.anchor.0);
        let focus = (self.focus.1, self.focus.0);
        if anchor <= focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }

    fn update_history_focus(
        &mut self,
        history_focus: Option<HistorySelectionPoint>,
        keep_on_missing: bool,
    ) {
        if self.history_anchor.is_some() && (history_focus.is_some() || !keep_on_missing) {
            self.history_focus = history_focus;
        }
    }
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
            history_projection_cache: HistoryProjectionCache::default(),
            last_history_projection: HistoryProjection::default(),
            last_history_viewport_height: 0,
            last_history_area: None,
            history_prefix_start: None,
            last_frame_history_start: 0,
            last_splash_row_count: 0,
            last_history_prefix_row_count: 0,
            queued_outputs: Vec::new(),
            active_stream: None,
            ordered_content_open: false,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            tool_placeholder_gate: crate::app::output::ToolPlaceholderGate::default(),
            text_filter: ToolSyntaxFilter::new(),
            processing_frame: 0,
            stream_count: 0,
            session_id: None,
            last_char_time: None,
            first_message_dispatched: false,
            selection: None,
            dashboard_availability: None,
            approval_viewport_snapshot: None,
            approval_preview_fully_visible: true,
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
            history_projection_cache: HistoryProjectionCache::default(),
            last_history_projection: HistoryProjection::default(),
            last_history_viewport_height: 0,
            last_history_area: None,
            history_prefix_start: None,
            last_frame_history_start: 0,
            last_splash_row_count: 0,
            last_history_prefix_row_count: 0,
            queued_outputs: Vec::new(),
            active_stream: None,
            ordered_content_open: false,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            tool_placeholder_gate: crate::app::output::ToolPlaceholderGate::default(),
            text_filter: ToolSyntaxFilter::default(),
            processing_frame: 0,
            stream_count: 0,
            session_id: None,
            last_char_time: None,
            first_message_dispatched: false,
            selection: None,
            dashboard_availability: None,
            approval_viewport_snapshot: None,
            approval_preview_fully_visible: true,
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

    /// Adds a successful local Dashboard endpoint to the display-only Logo prefix.
    ///
    /// The socket address keeps the rendered target free of userinfo, query values,
    /// fragments, and bearer credentials. This state never enters the transcript.
    pub fn set_dashboard_availability(
        &mut self,
        address: SocketAddr,
        authentication_required: bool,
    ) {
        self.dashboard_availability = Some(crate::splash::DashboardAvailability::new(
            address,
            authentication_required,
        ));
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
        self.show_approval_with_preview(tool_name, arguments, None);
    }

    pub fn show_approval_with_preview(
        &mut self,
        tool_name: &str,
        arguments: &str,
        preview: Option<String>,
    ) {
        self.approval_preview_fully_visible = preview
            .as_deref()
            .is_none_or(|value| value.trim().is_empty());
        if self.approval_viewport_snapshot.is_none() {
            self.approval_viewport_snapshot = Some(ApprovalViewportSnapshot {
                history_scroll: self.history_scroll.clone(),
                history_prefix_start: self.history_prefix_start,
            });

            // A FollowTail projection is sized for the pre-prompt natural flow. Once the
            // approval panel is present, that flow loses rows to the modal. Preserve the
            // first visible logical row so the triggering context does not jump to the tail.
            if matches!(self.history_scroll.mode, HistoryScrollMode::FollowTail) {
                if let Some(anchor) = self.last_history_projection.first_anchor() {
                    self.history_scroll.anchor(anchor, 0);
                }
                self.history_prefix_start = None;
            }
        }
        self.state.activate_approval(tool_name, arguments);
        self.state.slash_menu = crate::state::BottomPanelState::open_approval_with_preview(
            tool_name, arguments, preview,
        );
    }

    pub fn hide_approval(&mut self) {
        self.state.approval_state = ApprovalState::Hidden;
        self.approval_preview_fully_visible = true;
        if let Some(snapshot) = self.approval_viewport_snapshot.take() {
            self.history_scroll = snapshot.history_scroll;
            self.history_prefix_start = snapshot.history_prefix_start;
        }
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
    if let Some(TurnPhase::Reconnecting {
        attempt,
        max_attempts,
    }) = phase
        && is_processing
    {
        return format!("Reconnecting... (attempt {attempt}/{max_attempts})");
    }
    if let Some(TurnPhase::RunningTool { name }) = phase
        && is_processing
    {
        return format!("running tool: {name}...");
    }

    if let Some(thinking) = thinking_preview
        && is_processing
    {
        return format!("thinking: {thinking}");
    }

    if matches!(phase, Some(TurnPhase::Connecting)) && is_processing {
        return "Connecting...".to_string();
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

#[cfg(test)]
mod i168_terminal_tests {
    use super::*;

    #[test]
    fn terminal_truncation_tip_is_retained_by_tui() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let text = "Response truncated: provider reached the output token limit. Partial response preserved.";
        let should_exit = tui.handle_ui_output(UiOutput::Tip {
            text: text.into(),
            kind: TipKind::Error,
        });

        assert!(!should_exit);
        let tip = tui.state.tip.as_ref().expect("truncation tip");
        assert_eq!(tip.kind, TipKind::Error);
        assert_eq!(tip.text, text);
        assert!(tip.ttl >= Duration::from_secs(4));
    }

    #[test]
    fn terminal_known_provider_policy_tip_is_retained_by_tui() {
        for text in [
            "provider response filtered by content policy (finish_reason=content_filter)",
            "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
            "provider refused request (stop_reason=refusal)",
            "unsupported provider stop_reason: fixture_unknown_reason",
        ] {
            let mut tui = Tui::for_test(TuiState::new(), None);
            let should_exit = tui.handle_ui_output(UiOutput::Tip {
                text: text.into(),
                kind: TipKind::Error,
            });

            assert!(!should_exit);
            let tip = tui.state.tip.as_ref().expect("provider policy tip");
            assert_eq!(tip.kind, TipKind::Error);
            assert_eq!(tip.text, text);
        }
    }

    #[test]
    fn terminal_processing_clear_status_reaches_tui_state() {
        let mut tui = Tui::for_test(TuiState::new(), None);
        let status = talos_conversation::StatusSnapshot {
            is_processing: false,
            phase: Some(talos_conversation::TurnPhase::Failed),
            ..Default::default()
        };
        tui.handle_ui_output(UiOutput::Status(status.clone()));

        assert_eq!(tui.state.status, status);
        assert!(!tui.state.status.is_processing);
    }
}
