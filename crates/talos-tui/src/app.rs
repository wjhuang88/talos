//! TUI application loop and layout rendering.
//!
//! Inline-by-default rendering model (I022):
//! - No alternate screen; content is appended to the terminal's native scrollback.
//! - Finalized turns are pushed above the viewport via `insert_history`.
//! - The viewport only contains the active streaming area + input + status bar.
//! - Exit leaves all content in the terminal for native scrollback review.

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    terminal::enable_raw_mode,
};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph},
};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use talos_core::message::{AgentEvent, Usage};
use tokio::sync::{broadcast, mpsc};

use crate::evolution::{self, EvolutionPanel};
use crate::inline_terminal::InlineTerminal;
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, ChatMessage, CtrlCState, MessageRole, MessageStatus, TuiState, Tip, TipKind};
use crate::widgets::ApprovalOverlay;

struct ViewportLayout {
    preview: Rect,
    queue_preview: Option<Rect>,
    gap: Rect,
    tips: Rect,
    input_pad_top: Rect,
    input: Rect,
    input_pad_bot: Rect,
    status: Rect,
}

impl ViewportLayout {
    const BASE_HEIGHT: u16 = 7;

    fn height(queue_line_count: u16) -> u16 {
        Self::BASE_HEIGHT + queue_line_count
    }

    fn split(area: Rect, queue_line_count: u16) -> Self {
        let mut constraints = vec![];
        constraints.push(Constraint::Length(1));
        if queue_line_count > 0 {
            constraints.push(Constraint::Length(queue_line_count));
        }
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(1));
        constraints.extend_from_slice(&[
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let chunks = Layout::vertical(&constraints).split(area);

        let mut idx = 0;
        let preview = chunks[idx];
        idx += 1;

        let queue_preview = if queue_line_count > 0 {
            let r = chunks[idx];
            idx += 1;
            Some(r)
        } else {
            None
        };

        let gap = chunks[idx];
        idx += 1;
        let tips = chunks[idx];
        idx += 1;
        let input_pad_top = chunks[idx];
        idx += 1;
        let input = chunks[idx];
        idx += 1;
        let input_pad_bot = chunks[idx];
        idx += 1;
        let status = chunks[idx];

        Self {
            preview,
            queue_preview,
            gap,
            tips,
            input_pad_top,
            input,
            input_pad_bot,
            status,
        }
    }
}

/// Main TUI application for the Talos agent.
///
/// Provides a terminal-based chat interface with streaming output,
/// input handling, and status display. Uses inline-by-default rendering:
/// content appends to the terminal, not to an alternate screen buffer.
pub struct Tui {
    /// Internal state of the TUI.
    state: TuiState,
    /// Terminal backend.
    terminal: InlineTerminal,
    /// Skill sidebar panel.
    skill_sidebar: SkillSidebar,
    /// Evolution insights panel.
    evolution_panel: EvolutionPanel,
    /// Channel to send user messages to the agent loop.
    message_tx: Option<mpsc::UnboundedSender<String>>,
    /// Index into messages up to which content has been pushed to scrollback.
    last_pushed_history: usize,
}

impl Tui {
    /// Creates a new TUI instance with an inline viewport.
    ///
    /// The viewport is anchored at the current cursor position with height = 0,
    /// so no blank lines are produced on initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized or
    /// raw mode cannot be enabled.
    pub fn new() -> Result<Self> {
        print_banner();

        enable_raw_mode()?;

        let terminal = InlineTerminal::new()?;

        Ok(Self {
            state: TuiState::new(),
            terminal,
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: evolution::EvolutionPanel::new(),
            message_tx: None,
            last_pushed_history: 0,
        })
    }

    /// Runs the main TUI event loop, receiving agent events from the broadcast channel.
    ///
    /// This method blocks until the user exits via double Ctrl+C.
    pub async fn run(&mut self, mut event_rx: broadcast::Receiver<AgentEvent>) -> Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            self.state.expire_tip();
            if let Err(e) = self.flush_scrollback() {
                eprintln!("warning: flush scrollback failed: {e}");
            }
            self.draw_frame()?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                }
                event = event_rx.recv() => {
                    if self.handle_agent_event(event) {
                        break;
                    }
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        self.restore();
        Ok(())
    }

    /// Runs the TUI with approval channel support.
    pub async fn run_with_approval(
        &mut self,
        mut event_rx: broadcast::Receiver<AgentEvent>,
        mut approval_rx: mpsc::UnboundedReceiver<TuiApprovalRequest>,
    ) -> Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            self.state.expire_tip();
            if let Err(e) = self.flush_scrollback() {
                eprintln!("warning: flush scrollback failed: {e}");
            }
            self.draw_frame()?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                }
                event = event_rx.recv() => {
                    if self.handle_agent_event(event) {
                        break;
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

        self.restore();
        Ok(())
    }

    // ── Drawing ──────────────────────────────────────────────────────

    fn draw_frame(&mut self) -> Result<()> {
        let state = &self.state;

        let queue_count = state.steering_queue.len() + state.followup_queue.len();
        let queue_line_count = if queue_count == 0 {
            0u16
        } else {
            1 + (queue_count as u16).min(2)
        };

        self.terminal
            .draw_with_overlap(ViewportLayout::height(queue_line_count), 1, |frame| {
                let layout = ViewportLayout::split(frame.area(), queue_line_count);

                if !state.current_turn_text.is_empty() {
                    let line = state.current_turn_text.split('\n').last().unwrap_or("");
                    let display = truncate_end_to_width(line, layout.preview.width);
                    frame.render_widget(
                        Paragraph::new(Line::from(Span::styled(
                            display,
                            Style::default().fg(Color::Rgb(0xE5, 0xE9, 0xF0)),
                        ))),
                        layout.preview,
                    );
                }

                let hint_text = if let Some(tip) = &state.tip {
                    let color = match tip.kind {
                        TipKind::ExitHint | TipKind::QueueHint => {
                            Color::Rgb(0xA3, 0xBE, 0x8C)
                        }
                        TipKind::ApprovalResult => Color::Rgb(0xB4, 0x8E, 0xAD),
                        TipKind::LagWarning => Color::Rgb(0xBF, 0x61, 0x6C),
                        TipKind::Info => Color::Rgb(0x88, 0xC0, 0xD0),
                    };
                    Text::from(Line::from(Span::styled(
                        format!(" {}", tip.text),
                        Style::default().fg(color),
                    )))
                } else if state.is_processing {
                    Text::from(Line::from(Span::styled(
                        " Processing… Esc to clear, Ctrl+C to cancel",
                        Style::default().fg(Color::Rgb(0xD0, 0x87, 0x70)),
                    )))
                } else {
                    Text::from(Line::from(Span::styled(
                        " Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution",
                        Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A)),
                    )))
                };
                frame.render_widget(Paragraph::new(hint_text), layout.tips);

                if let Some(qp_area) = layout.queue_preview {
                    let mut qp_lines = Vec::new();
                    let dim = Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A));

                    let msg_count = queue_count;
                    qp_lines.push(Line::from(vec![
                        Span::styled(" ", dim),
                        Span::styled(
                            format!("{} queued input{}", msg_count, if msg_count == 1 { "" } else { "s" }),
                            dim,
                        ),
                        Span::styled(" (will send after current turn)", dim),
                    ]));

                    let max_width = (qp_area.width as usize).saturating_sub(4);
                    for item in state.steering_queue.iter().chain(state.followup_queue.iter()).take(2) {
                        let text = if item.len() > max_width {
                            format!("{}…", &item[..max_width - 1])
                        } else {
                            item.clone()
                        };
                        qp_lines.push(Line::from(vec![
                            Span::styled("  ", dim),
                            Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                            Span::styled(text, dim),
                        ]));
                    }

                    frame.render_widget(Paragraph::new(qp_lines), qp_area);
                }

                let input_bg = Color::Rgb(0x3B, 0x42, 0x52);

                frame.render_widget(
                    Paragraph::new("").style(Style::default().bg(input_bg)),
                    layout.input_pad_top,
                );

                let input_text = build_input_text(state);
                let input_block = Block::default()
                    .style(Style::default().bg(input_bg))
                    .padding(Padding::new(1, 1, 0, 0));
                let input_paragraph = Paragraph::new(input_text).block(input_block);
                frame.render_widget(input_paragraph, layout.input);

                frame.render_widget(
                    Paragraph::new("").style(Style::default().bg(input_bg)),
                    layout.input_pad_bot,
                );

                let status_text = build_status_text(state);
                frame.render_widget(Paragraph::new(status_text), layout.status);

                if let ApprovalState::Visible {
                    tool_name,
                    arguments,
                    selected,
                } = &state.approval_state
                {
                    let overlay = ApprovalOverlay::new(tool_name, arguments, selected);
                    frame.render_widget(overlay, layout.input);
                }
            })?;

        Ok(())
    }

    fn flush_scrollback(&mut self) -> Result<()> {
        let lines = self.extract_new_scrollback_lines();
        if lines.is_empty() {
            return Ok(());
        }
        self.terminal.insert_history(&lines)?;
        Ok(())
    }

    fn extract_new_scrollback_lines(&mut self) -> Vec<String> {
        let mut new_lines = Vec::new();

        let start = self.last_pushed_history.min(self.state.messages.len());
        for msg in &self.state.messages[start..] {
            new_lines.extend(message_to_text_lines(msg));
        }
        self.last_pushed_history = self.state.messages.len();

        if !self.state.current_turn_text.is_empty() {
            let all_lines: Vec<&str> = self.state.current_turn_text.split('\n').collect();
            let already_scrolled = self.state.scrollback.scrolled_line_count;
            let complete_count = all_lines.len().saturating_sub(1);

            if complete_count > already_scrolled {
                for line in all_lines[already_scrolled..complete_count].iter() {
                    new_lines.push(line.to_string());
                }
                self.state.scrollback.scrolled_line_count = complete_count;
            }
        }

        new_lines
    }

    fn finalize_scrollback(&mut self) -> Result<()> {
        if self.state.current_turn_text.is_empty() {
            return Ok(());
        }

        let text = std::mem::take(&mut self.state.current_turn_text);
        let remaining_start = self.state.scrollback.scrolled_line_count;
        let all_lines: Vec<&str> = text.split('\n').collect();

        if remaining_start < all_lines.len() {
            let tail: Vec<String> = all_lines[remaining_start..]
                .iter()
                .map(|s| s.to_string())
                .collect();
            if !tail.is_empty() {
                self.terminal.insert_history(&tail)?;
            }
        }

        self.state.scrollback.scrolled_line_count = 0;

        if !text.is_empty() {
            self.state.messages.push(ChatMessage {
                role: MessageRole::Assistant,
                status: MessageStatus::Completed,
                content: text,
                tool_call: None,
                created_at: Instant::now(),
            });
            self.last_pushed_history = self.state.messages.len();
        }

        Ok(())
    }

    // ── Agent event handling ─────────────────────────────────────────

    fn handle_agent_event(&mut self, event: std::result::Result<AgentEvent, broadcast::error::RecvError>) -> bool {
        match event {
            Ok(agent_event) => {
                let is_turn_end = matches!(agent_event, AgentEvent::TurnEnd { .. });

                if is_turn_end {
                    if let Err(e) = self.finalize_scrollback() {
                        eprintln!("warning: finalize scrollback failed: {e}");
                    }
                }

                self.state.handle_event(&agent_event);

                if is_turn_end {
                    if let Some(msg) = self.state.drain_steering_queue() {
                        self.submit_message(msg);
                    }
                }
                false
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                self.state.tip = Some(Tip {
                    kind: TipKind::LagWarning,
                    text: format!("Warning: dropped {n} event(s)"),
                    ttl: Duration::from_secs(2),
                    created_at: Instant::now(),
                });
                false
            }
            Err(broadcast::error::RecvError::Closed) => true,
        }
    }

    // ── Public API ───────────────────────────────────────────────────

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

    // ── Message submission ──────────────────────────────────────────

    fn submit_message(&mut self, msg: String) {
        self.state.append_user_message(&msg);
        if let Some(ref tx) = self.message_tx {
            let _ = tx.send(msg);
        }
        if let Err(e) = self.flush_scrollback() {
            eprintln!("warning: flush scrollback failed: {e}");
        }
    }

    // ── Input handling ───────────────────────────────────────────────

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
                        if !input.is_empty() {
                            if input.starts_with('/') {
                                self.state.handle_slash_command(&input);
                            } else if self.state.is_processing {
                                self.state.steering_queue.push(input.clone());
                                self.state.tip = Some(Tip {
                                    kind: TipKind::QueueHint,
                                    text: "Message queued (steering). Press Esc to cancel.".into(),
                                    ttl: Duration::from_secs(2),
                                    created_at: Instant::now(),
                                });
                            } else {
                                self.state.is_processing = true;
                                self.submit_message(input);
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
                            self.state.tip = Some(Tip {
                                kind: TipKind::QueueHint,
                                text: "Queued message restored to input.".into(),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
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

    // ── Teardown ─────────────────────────────────────────────────────

    /// Restores the terminal to its original state.
    ///
    /// Content in the scrollback buffer is preserved so the user can
    /// scroll up to review the conversation after exit.
    fn restore(&self) {
        self.terminal.restore();
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.restore();
    }
}

// ── Banner ───────────────────────────────────────────────────────────

/// Prints the Talos startup banner to stdout *before* raw mode is entered,
/// so it becomes part of the terminal's native scrollback.
fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  \u{1f6e0} Talos v{version}");
    println!("  Safety-first agent runtime");
    println!();
}

// ── Rendering ────────────────────────────────────────────────────────

pub(crate) fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let char_count = buffer.chars().count();
    let cursor_pos = state.cursor_pos.min(char_count);

    let before: String = buffer.chars().take(cursor_pos).collect();
    let after: String = buffer.chars().skip(cursor_pos).collect();

    let cursor_style = Style::default()
        .fg(Color::Rgb(0x2E, 0x34, 0x40))
        .bg(Color::Rgb(0x88, 0xC0, 0xD0));

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " ❯ ",
        Style::default().fg(Color::Rgb(0xA3, 0xBE, 0x8C)),
    ));
    spans.push(Span::raw(before));
    if after.is_empty() {
        spans.push(Span::styled(" ", cursor_style));
    } else {
        let mut chars = after.chars();
        if let Some(first) = chars.next() {
            let rest: String = chars.collect();
            spans.push(Span::styled(first.to_string(), cursor_style));
            spans.push(Span::raw(rest));
        } else {
            spans.push(Span::styled(" ", cursor_style));
        }
    }

    Text::from(Line::from(spans))
}

pub(crate) fn build_status_text(state: &TuiState) -> Text<'static> {
    let model_name = state.model_name.clone();
    let total_tokens = state.usage.input_tokens + state.usage.output_tokens;
    let cost = calculate_cost(&state.usage);

    let branch_info = state
        .branch_id
        .as_ref()
        .map(|b| {
            let short: String = b.chars().take(8).collect();
            format!(" │ {short}")
        })
        .unwrap_or_default();

    let queue_info = if !state.steering_queue.is_empty() || !state.followup_queue.is_empty() {
        let mut parts = Vec::new();
        if !state.steering_queue.is_empty() {
            parts.push(format!("S:{}", state.steering_queue.len()));
        }
        if !state.followup_queue.is_empty() {
            parts.push(format!("F:{}", state.followup_queue.len()));
        }
        format!(" │ {}", parts.join(", "))
    } else {
        String::new()
    };

    let dim = Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A));
    let sep = Span::styled(" │ ", dim);
    let val = Style::default().fg(Color::Rgb(0x81, 0xA1, 0xC1));

    let spans = vec![
        Span::styled(" ", dim),
        Span::styled(model_name, val),
        sep.clone(),
        Span::styled(format!("{} tokens", total_tokens), val),
        Span::styled(branch_info, val),
        sep.clone(),
        Span::styled(format!("${cost:.4}"), val),
        Span::styled(queue_info, val),
    ];

    Text::from(Line::from(spans))
}

pub(crate) fn calculate_cost(usage: &Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Converts a [`ChatMessage`] to plain-text lines suitable for scrollback.
pub(crate) fn message_to_text_lines(msg: &ChatMessage) -> Vec<String> {
    let mut buf = String::new();
    TuiState::append_message_plain(&mut buf, msg);
    buf.lines().map(String::from).collect()
}


/// Truncates a string to fit within `max_width` terminal columns.
#[allow(dead_code)]
fn truncate_to_width(s: &str, max_width: u16) -> String {
    let max = max_width as usize;
    if unicode_width::UnicodeWidthStr::width(s) <= max {
        return s.to_string();
    }
    let mut width = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw > max {
            break;
        }
        result.push(ch);
        width += cw;
    }
    result
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

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ChatMessage, MessageRole, MessageStatus, ToolCallInfo};
    use talos_core::message::ToolResult;
    use talos_core::tool::ToolProvenance;

    #[test]
    fn message_to_text_lines_user_text() {
        let msg = ChatMessage {
            role: MessageRole::User,
            status: MessageStatus::Completed,
            content: "> hello".to_string(),
            tool_call: None,
            created_at: Instant::now(),
        };
        assert_eq!(message_to_text_lines(&msg), vec!["> hello".to_string()]);
    }

    #[test]
    fn message_to_text_lines_user_text_multiline() {
        let msg = ChatMessage {
            role: MessageRole::User,
            status: MessageStatus::Completed,
            content: "> line1\n> line2".to_string(),
            tool_call: None,
            created_at: Instant::now(),
        };
        assert_eq!(
            message_to_text_lines(&msg),
            vec!["> line1".to_string(), "> line2".to_string()],
        );
    }

    #[test]
    fn message_to_text_lines_assistant() {
        let msg = ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: "response".to_string(),
            tool_call: None,
            created_at: Instant::now(),
        };
        assert_eq!(
            message_to_text_lines(&msg),
            vec!["response".to_string()],
        );
    }

    #[test]
    fn message_to_text_lines_tool_call_pending() {
        let msg = ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: String::new(),
            tool_call: Some(ToolCallInfo {
                tool_name: "echo".to_string(),
                arguments: "{}".to_string(),
                provenance: ToolProvenance::Native,
                result: None,
            }),
            created_at: Instant::now(),
        };
        assert_eq!(
            message_to_text_lines(&msg),
            vec!["▸ echo [native]".to_string(), "  {}".to_string()],
        );
    }

    #[test]
    fn message_to_text_lines_tool_call_with_result() {
        let msg = ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: String::new(),
            tool_call: Some(ToolCallInfo {
                tool_name: "echo".to_string(),
                arguments: "{}".to_string(),
                provenance: ToolProvenance::Native,
                result: Some(ToolResult {
                    tool_use_id: "x".to_string(),
                    content: "ok".to_string(),
                    is_error: false,
                }),
            }),
            created_at: Instant::now(),
        };
        assert_eq!(
            message_to_text_lines(&msg),
            vec![
                "▸ echo [native]".to_string(),
                "  {}".to_string(),
                "  ✓ ok".to_string(),
            ],
        );
    }

    #[test]
    fn message_to_text_lines_tool_call_with_error() {
        let msg = ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: String::new(),
            tool_call: Some(ToolCallInfo {
                tool_name: "bad".to_string(),
                arguments: "{}".to_string(),
                provenance: ToolProvenance::McpRemote {
                    server: "srv".to_string(),
                },
                result: Some(ToolResult {
                    tool_use_id: "x".to_string(),
                    content: "boom".to_string(),
                    is_error: true,
                }),
            }),
            created_at: Instant::now(),
        };
        assert_eq!(
            message_to_text_lines(&msg),
            vec![
                "▸ bad [mcp:srv]".to_string(),
                "  {}".to_string(),
                "  ✗ boom".to_string(),
            ],
        );
    }

    #[test]
    fn truncate_to_width_ascii() {
        assert_eq!(truncate_to_width("hello world", 5), "hello");
    }

    #[test]
    fn truncate_to_width_cjk() {
        assert_eq!(truncate_to_width("你好世界", 4), "你好");
    }

    #[test]
    fn truncate_to_width_short_enough() {
        assert_eq!(truncate_to_width("hi", 10), "hi");
    }
}
