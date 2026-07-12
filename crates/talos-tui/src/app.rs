use std::io;
use std::pin::Pin;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    terminal::enable_raw_mode,
};
use futures::{Stream, StreamExt};
use talos_conversation::{
    ContentOutput, CopyScope, TipKind, TodoPanelData, TurnPhase, UiOutput, UserInput,
};
use talos_core::ApprovalChoice;
use talos_core::message::Message;
use talos_core::tool_filter::ToolSyntaxFilter;
use tokio::{sync::mpsc, time::MissedTickBehavior};

use crate::evolution::{self, EvolutionPanel};
use crate::inline_terminal::{
    ComponentStack, HistoryAttrs, HistorySegment, InlineTerminal, ViewportComponent,
};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, PanelAction, Tip, TuiState};
use crate::theme::{semantic, to_crossterm_color};

pub(crate) use crate::app_stream::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState};

const PROCESSING_FRAME_INTERVAL: Duration = Duration::from_millis(150);
const IME_ENTER_WINDOW: Duration = Duration::from_millis(50);

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
    ordered_content_open: bool,
    stream_render: StreamRenderState,
    stream_opening_pending: bool,
    pending_stream_opening: Vec<ScrollbackLine>,
    text_filter: ToolSyntaxFilter,
    processing_frame: usize,
    stream_count: usize,
    session_id: Option<String>,
    last_total_height: u16,
    last_char_time: Option<Instant>,
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
            ordered_content_open: false,
            stream_render: StreamRenderState::default(),
            stream_opening_pending: false,
            pending_stream_opening: Vec::new(),
            text_filter: ToolSyntaxFilter::new(),
            processing_frame: 0,
            stream_count: 0,
            session_id: None,
            last_total_height: 0,
            last_char_time: None,
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
        let mut render_interval = tokio::time::interval(PROCESSING_FRAME_INTERVAL);
        render_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut ui_output_rx = self.ui_output_rx.take().expect("ui_output_rx not set");

        self.draw_frame()?;

        loop {
            self.state.expire_tip();
            self.flush_pending_scrollback()?;
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
        for line in crate::app_summary::build_exit_summary_lines(
            &self.state.status,
            elapsed,
            self.stream_count,
            self.session_id.as_deref(),
        ) {
            let _ = self.terminal.insert_history(&line.text, line.bg);
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
            self.pending_scrollback.extend(lines);
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
            self.pending_scrollback.extend(lines);
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
                    self.pending_scrollback
                        .extend(crate::scrollback::render_history_message(
                            &mut self.stream_count,
                            source,
                            &text,
                        ));
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
                self.pending_scrollback
                    .extend(crate::scrollback::render_history_message(
                        &mut self.stream_count,
                        talos_conversation::MessageSource::Reasoning,
                        &text,
                    ));
            }
            UiOutput::ToolCallStarted { .. } => {
                self.finalize_ordered_content();
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
            }
            UiOutput::ToolCall(display) => {
                let line = crate::tool_display::build_tool_call_scrollback_line(&display);
                self.pending_scrollback.push(line);
            }
            UiOutput::ToolResult(display) => {
                let icon = if display.is_error { "✗" } else { "" };
                let color = if display.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                self.pending_scrollback.extend(
                    crate::tool_display::build_tool_result_scrollback_lines(&display, icon, color),
                );
            }
            UiOutput::TodoPanel(data) => {
                self.pending_scrollback
                    .extend(build_todo_panel_lines(&data));
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
                self.flush_pending_scrollback().ok();
                self.hydrate_history(&messages);
                self.flush_pending_scrollback().ok();
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

    fn advance_processing_frame(&mut self) {
        self.processing_frame =
            next_processing_frame(self.state.status.is_processing, self.processing_frame);
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
        let query_for_panel = state.panel_query();
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
            let stack_top = viewport.bottom().saturating_sub(total_height);
            if self.state.slash_menu.is_credential_input() {
                let panel_y_offset = match menu_placement {
                    crate::scrollback::BottomPanelPlacement::AboveInput => {
                        preview.height_hint(screen_w)
                            + queue.height_hint(screen_w)
                            + tips.height_hint(screen_w)
                    }
                    crate::scrollback::BottomPanelPlacement::BelowInput => {
                        preview.height_hint(screen_w)
                            + queue.height_hint(screen_w)
                            + tips.height_hint(screen_w)
                            + input_pad_top.height_hint(screen_w)
                            + input.height_hint(screen_w)
                    }
                };
                let field_row_offset = match self.state.slash_menu.credential_field {
                    crate::state::CredentialField::ApiKey => 2,
                    crate::state::CredentialField::BaseUrl => 3,
                };
                let input_row = stack_top
                    .saturating_add(panel_y_offset)
                    .saturating_add(field_row_offset);
                let active_buffer = match self.state.slash_menu.credential_field {
                    crate::state::CredentialField::ApiKey => {
                        &self.state.slash_menu.credential_buffer
                    }
                    crate::state::CredentialField::BaseUrl => {
                        &self.state.slash_menu.base_url_buffer
                    }
                };
                let cursor_col = crate::scrollback::credential_cursor_col(active_buffer);
                self.terminal.set_cursor(cursor_col, input_row)?;
            } else {
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
                let input_top = stack_top + input_y_offset;
                let byte_pos = self.state.cursor_byte_pos();
                let (cursor_row_offset, cursor_col_offset) =
                    crate::scrollback::cursor_line_col(&self.state.input_buffer[..byte_pos]);
                let input_row = input_top.saturating_add(cursor_row_offset);
                let cursor_col = 3u16 + cursor_col_offset;
                self.terminal.set_cursor(cursor_col, input_row)?;
            }
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
                        submit_input_message(
                            &mut self.state,
                            &mut self.stream_render,
                            self.user_input_tx.as_ref(),
                        );
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
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }

    fn restore(&self) {
        self.terminal.restore();
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

    if matches!(phase, Some(TurnPhase::TimedOut)) {
        return "⏱ timed out".to_string();
    }
    if matches!(phase, Some(TurnPhase::Failed)) {
        return "✗ failed".to_string();
    }
    if matches!(phase, Some(TurnPhase::Cancelled)) {
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
        return format!("thinking: {thinking}");
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
        self.restore();
    }
}

#[allow(warnings)]
#[cfg(test)]
mod app_tests;
