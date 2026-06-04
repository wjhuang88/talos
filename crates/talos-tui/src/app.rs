//! TUI application loop and layout rendering.

use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use talos_core::message::{AgentEvent, Usage};
use tokio::sync::{broadcast, mpsc};

use crate::evolution::{self, EvolutionPanel};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, ChatLine, CtrlCState, TuiState};
use crate::widgets::{ApprovalOverlay, ToolCallBubble};

/// Main TUI application for the Talos agent.
///
/// Provides a terminal-based chat interface with streaming output,
/// input handling, and status display.
pub struct Tui {
    /// Internal state of the TUI.
    state: TuiState,
    /// Terminal backend.
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Skill sidebar panel.
    skill_sidebar: SkillSidebar,
    /// Evolution insights panel.
    evolution_panel: EvolutionPanel,
    /// Channel to send user messages to the agent loop.
    message_tx: Option<mpsc::UnboundedSender<String>>,
}

impl Tui {
    /// Creates a new TUI instance, setting up the terminal in raw mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized or
    /// raw mode cannot be enabled.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            state: TuiState::new(),
            terminal,
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: evolution::EvolutionPanel::new(),
            message_tx: None,
        })
    }

    /// Runs the main TUI event loop, receiving agent events from the broadcast channel.
    ///
    /// This method blocks until the user exits via double Ctrl+C.
    ///
    /// # Arguments
    ///
    /// * `event_rx` — A receiver for [`AgentEvent`] from the agent's broadcast channel.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal I/O fails and the event channel closes unexpectedly.
    pub async fn run(&mut self, mut event_rx: broadcast::Receiver<AgentEvent>) -> Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            let state = &self.state;
            let sidebar = &self.skill_sidebar;
            let evo = &self.evolution_panel;
            self.terminal
                .draw(|frame| render(frame, state, sidebar, evo))?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                }
                event = event_rx.recv() => {
                    match event {
                        Ok(agent_event) => {
                            let is_turn_end = matches!(agent_event, AgentEvent::TurnEnd { .. });
                            self.state.handle_event(&agent_event);
                            if is_turn_end {
                                if let Some(msg) = self.state.drain_steering_queue() {
                                    if let Some(ref tx) = self.message_tx {
                                        let _ = tx.send(msg);
                                    }
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            self.state.append_system(&format!("Warning: dropped {n} event(s)"));
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        Ok(())
    }

    /// Runs the TUI with approval channel support.
    ///
    /// Like [`Tui::run`], but also listens for [`TuiApprovalRequest`] on
    /// `approval_rx`. When a request arrives, shows the approval overlay
    /// and sends the user's choice back via the request's oneshot channel.
    pub async fn run_with_approval(
        &mut self,
        mut event_rx: broadcast::Receiver<AgentEvent>,
        mut approval_rx: mpsc::UnboundedReceiver<TuiApprovalRequest>,
    ) -> Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            let state = &self.state;
            let sidebar = &self.skill_sidebar;
            let evo = &self.evolution_panel;
            self.terminal
                .draw(|frame| render(frame, state, sidebar, evo))?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                }
                event = event_rx.recv() => {
                    match event {
                        Ok(agent_event) => {
                            let is_turn_end = matches!(agent_event, AgentEvent::TurnEnd { .. });
                            self.state.handle_event(&agent_event);
                            if is_turn_end {
                                if let Some(msg) = self.state.drain_steering_queue() {
                                    if let Some(ref tx) = self.message_tx {
                                        let _ = tx.send(msg);
                                    }
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            self.state.append_system(&format!("Warning: dropped {n} event(s)"));
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                Some(request) = approval_rx.recv() => {
                    self.state.pending_approval_response = Some(request.response);
                    self.show_approval(&request.tool_name, &request.arguments);
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        Ok(())
    }

    /// Shows the approval overlay with the given tool info.
    pub fn show_approval(&mut self, tool_name: &str, arguments: &str) {
        self.state.approval_state = ApprovalState::Visible {
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            selected: ApprovalChoice::ApproveOnce,
        };
    }

    /// Hides the approval overlay.
    pub fn hide_approval(&mut self) {
        self.state.approval_state = ApprovalState::Hidden;
    }

    /// Sets the channel to send user messages to the agent loop.
    pub fn set_message_tx(&mut self, tx: mpsc::UnboundedSender<String>) {
        self.message_tx = Some(tx);
    }

    /// Sets the model name displayed in the status bar.
    pub fn set_model_name(&mut self, name: String) {
        self.state.model_name = name;
    }

    /// Toggles the visibility of the skill sidebar.
    pub fn toggle_skill_sidebar(&mut self) {
        self.skill_sidebar.toggle();
    }

    /// Toggles the visibility of the evolution insights panel.
    pub fn toggle_evolution_panel(&mut self) {
        self.evolution_panel.toggle();
    }

    /// Updates the patterns displayed in the evolution panel.
    pub fn update_evolution_patterns(&mut self, patterns: Vec<evolution::PatternInfo>) {
        self.evolution_panel.update_patterns(patterns);
    }

    /// Updates the skills displayed in the sidebar.
    pub fn update_skills(&mut self, skills: Vec<SkillInfo>) {
        self.skill_sidebar.update_skills(skills);
    }

    /// Returns the current approval choice, if any.
    pub fn approval_choice(&self) -> Option<&ApprovalChoice> {
        match &self.state.approval_state {
            ApprovalState::Visible { selected, .. } => Some(selected),
            ApprovalState::Hidden => None,
        }
    }

    /// Handles a key press while the approval overlay is visible.
    /// Returns the chosen action if a valid key was pressed.
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

    /// Handles a terminal input event.
    ///
    /// Returns `true` if the TUI should exit.
    fn handle_input_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return self.state.handle_ctrl_c();
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
                            self.state.append_system(&format!(
                                "Tool call {}",
                                match choice {
                                    ApprovalChoice::ApproveOnce => "approved once",
                                    ApprovalChoice::AlwaysApprove => "always approved",
                                    ApprovalChoice::Deny => "denied",
                                }
                            ));
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
                        if !input.is_empty() {
                            if input.starts_with('/') {
                                self.state.handle_slash_command(&input);
                            } else if self.state.is_processing {
                                self.state.steering_queue.push(input.clone());
                                self.state.append_system(
                                    "Message queued (steering). Press Esc to cancel.",
                                );
                            } else {
                                self.state.append_user_message(&input);
                                self.state.is_processing = true;
                                if let Some(ref tx) = self.message_tx {
                                    let _ = tx.send(input);
                                }
                            }
                        }
                    }
                    KeyCode::Tab => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.input_buffer.starts_with('/') {
                            self.state.complete_slash_command();
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        if self.state.restore_last_queued() {
                            self.state
                                .append_system("Queued message restored to input.");
                        } else {
                            self.state.input_clear();
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }
}

pub(crate) fn render(
    frame: &mut Frame,
    state: &TuiState,
    skill_sidebar: &SkillSidebar,
    evolution_panel: &evolution::EvolutionPanel,
) {
    let full_area = frame.area();

    // If the evolution panel is visible, carve a right column for it first.
    let main_area = if evolution_panel.visible {
        let evo_width = evolution_panel
            .width
            .min(full_area.width.saturating_sub(40));
        let cols = Layout::horizontal([Constraint::Min(40), Constraint::Length(evo_width)])
            .split(full_area);

        evolution_panel.render(frame, cols[1]);
        cols[0]
    } else {
        full_area
    };

    // If sidebar is visible, split off the right portion
    let (chat_area, input_area, status_area) = if skill_sidebar.visible {
        let sidebar_width = skill_sidebar.width.min(main_area.width.saturating_sub(40));
        let chunks = Layout::horizontal([Constraint::Min(40), Constraint::Length(sidebar_width)])
            .split(main_area);

        skill_sidebar.render(frame, chunks[1]);

        let main_chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(chunks[0]);

        (main_chunks[0], main_chunks[1], main_chunks[2])
    } else {
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(main_area);

        (chunks[0], chunks[1], chunks[2])
    };

    let chat_text = build_chat_text(state);
    let chat_paragraph = Paragraph::new(chat_text)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .wrap(Wrap { trim: false })
        .scroll((chat_scroll_offset(state, chat_area.height as usize), 0));
    frame.render_widget(chat_paragraph, chat_area);

    let input_text = build_input_text(state);
    let input_paragraph = Paragraph::new(input_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Input (Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution) "),
    );
    frame.render_widget(input_paragraph, input_area);

    let status_text = build_status_text(state);
    let status_paragraph =
        Paragraph::new(status_text).style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_widget(status_paragraph, status_area);

    if let ApprovalState::Visible {
        tool_name,
        arguments,
        selected,
    } = &state.approval_state
    {
        let overlay = ApprovalOverlay::new(tool_name, arguments, selected);
        frame.render_widget(overlay, chat_area);
    }
}

pub(crate) fn build_chat_text(state: &TuiState) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in &state.chat_lines {
        match line {
            ChatLine::Text(text) => {
                lines.push(Line::from(text.clone()));
            }
            ChatLine::Assistant(text) => {
                let rendered = tui_markdown::from_str(text.as_str());
                for line in rendered.lines {
                    let owned_spans: Vec<Span<'static>> = line
                        .spans
                        .into_iter()
                        .map(|s| Span::styled(s.content.into_owned(), s.style))
                        .collect();
                    lines.push(Line::from(owned_spans));
                }
            }
            ChatLine::ToolCall {
                tool_name,
                arguments,
                provenance,
                result,
            } => {
                let mut bubble = ToolCallBubble::new(tool_name.as_str(), arguments.as_str())
                    .with_provenance(provenance.clone());
                if let Some(result) = result {
                    bubble = bubble.with_result(result.is_error, &result.content);
                }
                let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, 40, 6));
                bubble.render(buf.area, &mut buf);
                for y in 0..buf.area.height {
                    let mut spans = Vec::new();
                    for x in 0..buf.area.width {
                        let cell = buf.cell((x, y)).expect("cell within buffer bounds");
                        spans.push(Span::styled(cell.symbol().to_string(), cell.style()));
                    }
                    lines.push(Line::from(spans));
                }
            }
        }
    }

    if !state.current_turn_text.is_empty() {
        let rendered = tui_markdown::from_str(&state.current_turn_text);
        for line in rendered.lines {
            let owned_spans: Vec<Span<'static>> = line
                .spans
                .into_iter()
                .map(|s| Span::styled(s.content.into_owned(), s.style))
                .collect();
            lines.push(Line::from(owned_spans));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(
            "Welcome to Talos. Type a message and press Enter.",
        ));
    }

    Text::from(lines)
}

pub(crate) fn chat_scroll_offset(state: &TuiState, viewport_height: usize) -> u16 {
    let total_lines = state.chat_lines.len()
        + if state.current_turn_text.is_empty() {
            0
        } else {
            1
        };
    let total_lines = if total_lines == 0 { 1 } else { total_lines };

    if total_lines <= viewport_height {
        0
    } else {
        (total_lines - viewport_height) as u16
    }
}

pub(crate) fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let char_count = buffer.chars().count();
    let cursor_pos = state.cursor_pos.min(char_count);

    let before: String = buffer.chars().take(cursor_pos).collect();
    let after: String = buffer.chars().skip(cursor_pos).collect();

    let mut spans = Vec::new();
    spans.push(Span::raw(before));
    if after.is_empty() {
        spans.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::REVERSED),
        ));
    } else {
        let mut chars = after.chars();
        if let Some(first) = chars.next() {
            let rest: String = chars.collect();
            spans.push(Span::styled(
                first.to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            ));
            spans.push(Span::raw(rest));
        } else {
            spans.push(Span::styled(
                " ",
                Style::default().add_modifier(Modifier::REVERSED),
            ));
        }
    }

    Text::from(Line::from(spans))
}

pub(crate) fn build_status_text(state: &TuiState) -> Text<'static> {
    let processing_indicator = if state.is_processing { "● " } else { "" };
    let status = if state.is_processing {
        "Processing..."
    } else {
        "Ready"
    };

    let total_tokens = state.usage.input_tokens + state.usage.output_tokens;
    let cost = calculate_cost(&state.usage);

    let branch_info = state
        .branch_id
        .as_ref()
        .map(|b| {
            let short: String = b.chars().take(8).collect();
            format!(" | Branch: {short}")
        })
        .unwrap_or_default();

    let queue_info = if !state.steering_queue.is_empty() || !state.followup_queue.is_empty() {
        let mut parts = Vec::new();
        if !state.steering_queue.is_empty() {
            parts.push(format!("Steering: {}", state.steering_queue.len()));
        }
        if !state.followup_queue.is_empty() {
            parts.push(format!("Follow-up: {}", state.followup_queue.len()));
        }
        format!(" | {}", parts.join(", "))
    } else {
        String::new()
    };

    let text = format!(
        " {processing_indicator}{status}{queue_info} | Model: {} | Tokens: {}{branch_info} | Cost: {}",
        state.model_name, total_tokens, cost
    );

    Text::from(Line::from(text))
}

pub(crate) fn calculate_cost(usage: &Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

/// Restores the terminal to its original state.
pub(crate) fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
