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
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use talos_core::message::{AgentEvent, Usage};
use tokio::sync::{broadcast, mpsc};

use crate::evolution::{self, EvolutionPanel};
use crate::inline_terminal::InlineTerminal;
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, ChatLine, CtrlCState, TuiState};
use crate::widgets::ApprovalOverlay;

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
    /// Index into chat_lines up to which content has been pushed to scrollback.
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
            self.state.expire_status_message();
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
            self.state.expire_status_message();
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
        let viewport_height = 4u16;

        self.terminal.draw(viewport_height, |frame| {
            let main_area = frame.area();
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(main_area);

            let input_text = build_input_text(state);
            let input_paragraph = Paragraph::new(input_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input (Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution) "),
            );
            frame.render_widget(input_paragraph, chunks[0]);

            let status_text = build_status_text(state);
            let status_paragraph = Paragraph::new(status_text)
                .style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_widget(status_paragraph, chunks[1]);

            if let ApprovalState::Visible {
                tool_name,
                arguments,
                selected,
            } = &state.approval_state
            {
                let overlay = ApprovalOverlay::new(tool_name, arguments, selected);
                frame.render_widget(overlay, chunks[0]);
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

        let start = self.last_pushed_history.min(self.state.chat_lines.len());
        for line in &self.state.chat_lines[start..] {
            new_lines.extend(chat_line_to_text_lines(line));
        }
        self.last_pushed_history = self.state.chat_lines.len();

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
            self.state
                .chat_lines
                .push(ChatLine::Assistant(text));
            self.last_pushed_history = self.state.chat_lines.len();
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
                        if let Some(ref tx) = self.message_tx {
                            let _ = tx.send(msg);
                        }
                    }
                }
                false
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                self.state.status_message = Some((format!("Warning: dropped {n} event(s)"), Instant::now()));
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
                            self.state.status_message = Some((format!(
                                "Tool call {}",
                                match choice {
                                    ApprovalChoice::ApproveOnce => "approved once",
                                    ApprovalChoice::AlwaysApprove => "always approved",
                                    ApprovalChoice::Deny => "denied",
                                }
                            ), Instant::now()));
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
                                self.state.status_message = Some(("Message queued (steering). Press Esc to cancel.".into(), Instant::now()));
                            } else {
                                self.state.append_user_message(&input);
                                self.state.is_processing = true;
                                if let Some(ref tx) = self.message_tx {
                                    let _ = tx.send(input);
                                }
                                if let Err(e) = self.flush_scrollback() {
                                    eprintln!("warning: flush scrollback failed: {e}");
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
                            self.state.status_message = Some(("Queued message restored to input.".into(), Instant::now()));
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

    let status_msg = state
        .status_message
        .as_ref()
        .map(|(m, _)| format!(" | {}", m))
        .unwrap_or_default();

    let text = format!(
        " {processing_indicator}{status}{queue_info} | Model: {} | Tokens: {}{branch_info} | Cost: {}{status_msg}",
        state.model_name, total_tokens, cost
    );

    Text::from(Line::from(text))
}

pub(crate) fn calculate_cost(usage: &Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Converts a [`ChatLine`] to plain-text lines suitable for scrollback.
pub(crate) fn chat_line_to_text_lines(line: &ChatLine) -> Vec<String> {
    let mut buf = String::new();
    TuiState::append_line_plain(&mut buf, line);
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

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ChatLine;
    use talos_core::message::ToolResult;
    use talos_core::tool::ToolProvenance;

    #[test]
    fn chat_line_to_text_lines_user_text() {
        let line = ChatLine::Text("> hello".to_string());
        assert_eq!(chat_line_to_text_lines(&line), vec!["> hello".to_string()]);
    }

    #[test]
    fn chat_line_to_text_lines_user_text_multiline() {
        let line = ChatLine::Text("> line1\n> line2".to_string());
        assert_eq!(
            chat_line_to_text_lines(&line),
            vec!["> line1".to_string(), "> line2".to_string()],
        );
    }

    #[test]
    fn chat_line_to_text_lines_assistant() {
        let line = ChatLine::Assistant("response".to_string());
        assert_eq!(
            chat_line_to_text_lines(&line),
            vec!["response".to_string()],
        );
    }

    #[test]
    fn chat_line_to_text_lines_tool_call_pending() {
        let line = ChatLine::ToolCall {
            tool_name: "echo".to_string(),
            arguments: "{}".to_string(),
            provenance: ToolProvenance::Native,
            result: None,
        };
        assert_eq!(
            chat_line_to_text_lines(&line),
            vec!["▸ echo [native]".to_string(), "  {}".to_string()],
        );
    }

    #[test]
    fn chat_line_to_text_lines_tool_call_with_result() {
        let line = ChatLine::ToolCall {
            tool_name: "echo".to_string(),
            arguments: "{}".to_string(),
            provenance: ToolProvenance::Native,
            result: Some(ToolResult {
                tool_use_id: "x".to_string(),
                content: "ok".to_string(),
                is_error: false,
            }),
        };
        assert_eq!(
            chat_line_to_text_lines(&line),
            vec![
                "▸ echo [native]".to_string(),
                "  {}".to_string(),
                "  ✓ ok".to_string(),
            ],
        );
    }

    #[test]
    fn chat_line_to_text_lines_tool_call_with_error() {
        let line = ChatLine::ToolCall {
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
        };
        assert_eq!(
            chat_line_to_text_lines(&line),
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
