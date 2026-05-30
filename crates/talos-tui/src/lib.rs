//! Talos TUI — terminal user interface for the Talos agent.
//!
//! Provides a chat-based interface with:
//! - Chat viewport with scrolling message history
//! - Single-line input area with cursor
//! - Status bar showing model, token count, and cost
//! - Ctrl+C handling (single press cancels turn, double press exits)
//! - Streaming output that auto-scrolls

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use talos_core::message::{AgentEvent, Usage};
use tokio::sync::broadcast;

/// Duration window for detecting double Ctrl+C press.
const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

/// State machine for Ctrl+C handling.
#[derive(Debug, Clone, PartialEq, Default)]
enum CtrlCState {
    /// No Ctrl+C pressed yet.
    #[default]
    Idle,
    /// First Ctrl+C pressed, waiting for second press within window.
    Waiting(Instant),
}

/// Tracks the state of the TUI application.
#[derive(Debug, Default)]
struct TuiState {
    /// Chat messages displayed in the viewport.
    chat_lines: Vec<String>,
    /// Current input buffer.
    input_buffer: String,
    /// Cursor position within the input buffer (character index).
    cursor_pos: usize,
    /// Whether the agent is currently processing a turn.
    is_processing: bool,
    /// Current turn's accumulated text (for streaming).
    current_turn_text: String,
    /// Ctrl+C state machine.
    ctrl_c_state: CtrlCState,
    /// Whether the TUI should exit.
    should_exit: bool,
    /// Last usage statistics from the agent.
    usage: Usage,
    /// Model name displayed in the status bar.
    model_name: String,
}

impl TuiState {
    /// Creates a new default TUI state.
    fn new() -> Self {
        Self::default()
    }

    /// Appends a text delta to the current turn's streaming text.
    fn append_delta(&mut self, delta: &str) {
        self.current_turn_text.push_str(delta);
    }

    /// Finalizes the current turn by appending accumulated text to chat history.
    fn finalize_turn(&mut self) {
        if !self.current_turn_text.is_empty() {
            self.chat_lines.push(self.current_turn_text.clone());
            self.current_turn_text.clear();
        }
    }

    /// Appends a user message to the chat history.
    fn append_user_message(&mut self, content: &str) {
        self.chat_lines.push(format!("> {content}"));
    }

    /// Appends an error message to the chat history.
    fn append_error(&mut self, message: &str) {
        self.chat_lines.push(format!("[Error] {message}"));
    }

    /// Appends a system message to the chat history.
    fn append_system(&mut self, message: &str) {
        self.chat_lines.push(format!("[System] {message}"));
    }

    /// Appends a character to the input buffer at the cursor position.
    fn input_append_char(&mut self, ch: char) {
        self.input_buffer.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
    }

    /// Deletes the character before the cursor in the input buffer.
    fn input_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.input_buffer.remove(self.cursor_pos);
        }
    }

    /// Moves the cursor left in the input buffer.
    fn input_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Moves the cursor right in the input buffer.
    fn input_cursor_right(&mut self) {
        if self.cursor_pos < self.input_buffer.len() {
            self.cursor_pos += 1;
        }
    }

    /// Clears the input buffer and resets cursor position.
    fn input_clear(&mut self) {
        self.input_buffer.clear();
        self.cursor_pos = 0;
    }

    /// Submits the input buffer, returning its content and clearing it.
    fn input_submit(&mut self) -> String {
        let content = self.input_buffer.clone();
        self.input_clear();
        content
    }

    /// Handles a Ctrl+C press, returning whether to exit.
    ///
    /// First press cancels the current turn (if processing).
    /// Second press within [`DOUBLE_CTRL_C_WINDOW`] exits the TUI.
    fn handle_ctrl_c(&mut self) -> bool {
        let now = Instant::now();
        match &self.ctrl_c_state {
            CtrlCState::Idle => {
                if self.is_processing {
                    self.append_system("Turn cancelled. Press Ctrl+C again to exit.");
                } else {
                    self.append_system("Press Ctrl+C again within 2 seconds to exit.");
                }
                self.ctrl_c_state = CtrlCState::Waiting(now);
                false
            }
            CtrlCState::Waiting(pressed_at) => {
                if now.duration_since(*pressed_at) < DOUBLE_CTRL_C_WINDOW {
                    self.should_exit = true;
                    true
                } else {
                    self.append_system("Press Ctrl+C again within 2 seconds to exit.");
                    self.ctrl_c_state = CtrlCState::Waiting(now);
                    false
                }
            }
        }
    }

    /// Handles an agent event, updating state accordingly.
    fn handle_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::TurnStart => {
                self.is_processing = true;
                self.current_turn_text.clear();
            }
            AgentEvent::TextDelta { delta } => {
                self.append_delta(delta);
            }
            AgentEvent::ToolCall { call } => {
                self.chat_lines.push(format!("[tool: {}]", call.name));
            }
            AgentEvent::ToolResult { result } => {
                let status = if result.is_error { "error" } else { "ok" };
                self.chat_lines.push(format!("[tool result: {status}]"));
            }
            AgentEvent::TurnEnd { usage, .. } => {
                self.is_processing = false;
                self.finalize_turn();
                self.usage = usage.clone();
            }
            AgentEvent::Error { message } => {
                self.is_processing = false;
                self.current_turn_text.clear();
                self.append_error(message);
            }
        }
    }
}

/// Main TUI application for the Talos agent.
///
/// Provides a terminal-based chat interface with streaming output,
/// input handling, and status display.
pub struct Tui {
    /// Internal state of the TUI.
    state: TuiState,
    /// Terminal backend.
    terminal: Terminal<CrosstermBackend<Stdout>>,
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
    /// Returns an error if terminal I/O fails or the event channel closes unexpectedly.
    pub async fn run(&mut self, mut event_rx: broadcast::Receiver<AgentEvent>) -> Result<()> {
        self.state.model_name = "mock".to_string();

        loop {
            let state = &self.state;
            self.terminal.draw(|frame| render(frame, state))?;

            tokio::select! {
                _ = Self::poll_event() => {
                    if let Ok(event) = event::read() {
                        if self.handle_input_event(&event) {
                            break;
                        }
                    }
                }
                event = event_rx.recv() => {
                    match event {
                        Ok(agent_event) => {
                            self.state.handle_event(&agent_event);
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

    /// Polls for a terminal event with a short timeout.
    ///
    /// Returns `Ok(())` if an event is available, `Err` on timeout.
    async fn poll_event() -> Result<()> {
        if event::poll(Duration::from_millis(50))? {
            Ok(())
        } else {
            Err(anyhow::anyhow!("no event"))
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
                            self.state.append_user_message(&input);
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_clear();
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

fn render(frame: &mut Frame, state: &TuiState) {
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let chat_text = build_chat_text(state);
    let chat_paragraph = Paragraph::new(chat_text)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .wrap(Wrap { trim: false })
        .scroll((chat_scroll_offset(state), 0));
    frame.render_widget(chat_paragraph, chunks[0]);

    let input_text = build_input_text(state);
    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(" Input (Enter to send, Esc to clear) "));
    frame.render_widget(input_paragraph, chunks[1]);

    let status_text = build_status_text(state);
    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_widget(status_paragraph, chunks[2]);
}

fn build_chat_text(state: &TuiState) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in &state.chat_lines {
        lines.push(Line::from(line.clone()));
    }

    if !state.current_turn_text.is_empty() {
        lines.push(Line::from(state.current_turn_text.clone()));
    }

    if lines.is_empty() {
        lines.push(Line::from("Welcome to Talos. Type a message and press Enter."));
    }

    Text::from(lines)
}

fn chat_scroll_offset(state: &TuiState) -> u16 {
    let total_lines = state.chat_lines.len()
        + if state.current_turn_text.is_empty() { 0 } else { 1 };
    let total_lines = if total_lines == 0 { 1 } else { total_lines };
    total_lines.saturating_sub(1) as u16
}

fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let cursor_pos = state.cursor_pos.min(buffer.len());

    let before = &buffer[..cursor_pos];
    let after = &buffer[cursor_pos..];

    let mut spans = Vec::new();
    spans.push(Span::raw(before.to_string()));
    if after.is_empty() {
        spans.push(Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)));
    } else {
        let (first, rest) = after.split_at(1);
        spans.push(Span::styled(first.to_string(), Style::default().add_modifier(Modifier::REVERSED)));
        spans.push(Span::raw(rest.to_string()));
    }

    Text::from(Line::from(spans))
}

fn build_status_text(state: &TuiState) -> Text<'static> {
    let processing_indicator = if state.is_processing { "● " } else { "" };
    let status = if state.is_processing {
        "Processing..."
    } else {
        "Ready"
    };

    let total_tokens = state.usage.input_tokens + state.usage.output_tokens;
    let cost = calculate_cost(&state.usage);

    let text = format!(
        " {processing_indicator}{status} | Model: {} | Tokens: {} | Cost: {}",
        state.model_name, total_tokens, cost
    );

    Text::from(Line::from(text))
}

fn calculate_cost(usage: &Usage) -> String {
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
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::StopReason;

    #[test]
    fn test_state_new() {
        let state = TuiState::new();
        assert!(state.chat_lines.is_empty());
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
        assert!(!state.is_processing);
        assert!(state.current_turn_text.is_empty());
        assert!(!state.should_exit);
    }

    #[test]
    fn test_append_delta() {
        let mut state = TuiState::new();
        state.append_delta("Hello");
        state.append_delta(", ");
        state.append_delta("world!");
        assert_eq!(state.current_turn_text, "Hello, world!");
    }

    #[test]
    fn test_finalize_turn_with_text() {
        let mut state = TuiState::new();
        state.append_delta("Assistant response");
        state.finalize_turn();
        assert_eq!(state.chat_lines, vec!["Assistant response"]);
        assert!(state.current_turn_text.is_empty());
    }

    #[test]
    fn test_finalize_turn_empty() {
        let mut state = TuiState::new();
        state.finalize_turn();
        assert!(state.chat_lines.is_empty());
    }

    #[test]
    fn test_append_user_message() {
        let mut state = TuiState::new();
        state.append_user_message("Hello");
        assert_eq!(state.chat_lines, vec!["> Hello"]);
    }

    #[test]
    fn test_append_error() {
        let mut state = TuiState::new();
        state.append_error("Something failed");
        assert_eq!(state.chat_lines, vec!["[Error] Something failed"]);
    }

    #[test]
    fn test_append_system() {
        let mut state = TuiState::new();
        state.append_system("Turn cancelled");
        assert_eq!(state.chat_lines, vec!["[System] Turn cancelled"]);
    }

    #[test]
    fn test_input_append_char() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_append_char('c');
        assert_eq!(state.input_buffer, "abc");
        assert_eq!(state.cursor_pos, 3);
    }

    #[test]
    fn test_input_append_char_at_position() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('c');
        state.input_cursor_left();
        state.input_append_char('b');
        assert_eq!(state.input_buffer, "abc");
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_input_backspace() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_backspace();
        assert_eq!(state.input_buffer, "a");
        assert_eq!(state.cursor_pos, 1);
    }

    #[test]
    fn test_input_backspace_at_start() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_cursor_left();
        state.input_backspace();
        assert_eq!(state.input_buffer, "a");
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_movement() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_append_char('c');

        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 2);

        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 1);

        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_input_cursor_bounds() {
        let mut state = TuiState::new();
        state.input_append_char('a');

        state.input_cursor_left();
        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 0);

        state.input_cursor_right();
        state.input_cursor_right();
        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 1);
    }

    #[test]
    fn test_input_clear() {
        let mut state = TuiState::new();
        state.input_append_char('h');
        state.input_append_char('i');
        state.input_clear();
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_submit() {
        let mut state = TuiState::new();
        state.input_append_char('h');
        state.input_append_char('i');
        let result = state.input_submit();
        assert_eq!(result, "hi");
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_ctrl_c_single_press_idle() {
        let mut state = TuiState::new();
        let should_exit = state.handle_ctrl_c();
        assert!(!should_exit);
        assert!(matches!(state.ctrl_c_state, CtrlCState::Waiting(_)));
    }

    #[test]
    fn test_ctrl_c_double_press_exits() {
        let mut state = TuiState::new();
        let should_exit = state.handle_ctrl_c();
        assert!(!should_exit);

        let should_exit = state.handle_ctrl_c();
        assert!(should_exit);
        assert!(state.should_exit);
    }

    #[test]
    fn test_ctrl_c_reset_on_char() {
        let mut state = TuiState::new();
        state.handle_ctrl_c();
        assert!(matches!(state.ctrl_c_state, CtrlCState::Waiting(_)));

        state.ctrl_c_state = CtrlCState::Idle;
        assert!(matches!(state.ctrl_c_state, CtrlCState::Idle));
    }

    #[test]
    fn test_handle_event_turn_start() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        assert!(state.is_processing);
        assert!(state.current_turn_text.is_empty());
    }

    #[test]
    fn test_handle_event_text_delta() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TextDelta { delta: "Hello".into() });
        state.handle_event(&AgentEvent::TextDelta { delta: " world".into() });
        assert_eq!(state.current_turn_text, "Hello world");
    }

    #[test]
    fn test_handle_event_tool_call() {
        let mut state = TuiState::new();
        let call = talos_core::message::ToolCall {
            id: "c1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls"}),
        };
        state.handle_event(&AgentEvent::ToolCall { call });
        assert_eq!(state.chat_lines, vec!["[tool: bash]"]);
    }

    #[test]
    fn test_handle_event_tool_result_ok() {
        let mut state = TuiState::new();
        let result = talos_core::message::ToolResult {
            tool_use_id: "c1".into(),
            content: "file.rs".into(),
            is_error: false,
        };
        state.handle_event(&AgentEvent::ToolResult { result });
        assert_eq!(state.chat_lines, vec!["[tool result: ok]"]);
    }

    #[test]
    fn test_handle_event_tool_result_error() {
        let mut state = TuiState::new();
        let result = talos_core::message::ToolResult {
            tool_use_id: "c1".into(),
            content: "failed".into(),
            is_error: true,
        };
        state.handle_event(&AgentEvent::ToolResult { result });
        assert_eq!(state.chat_lines, vec!["[tool result: error]"]);
    }

    #[test]
    fn test_handle_event_turn_end() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        state.append_delta("Response text");
        state.handle_event(&AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
        });
        assert!(!state.is_processing);
        assert_eq!(state.chat_lines, vec!["Response text"]);
        assert_eq!(state.usage.input_tokens, 100);
        assert_eq!(state.usage.output_tokens, 50);
    }

    #[test]
    fn test_handle_event_error() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        state.append_delta("Partial");
        state.handle_event(&AgentEvent::Error { message: "API error".into() });
        assert!(!state.is_processing);
        assert!(state.current_turn_text.is_empty());
        assert_eq!(state.chat_lines, vec!["[Error] API error"]);
    }

    #[test]
    fn test_calculate_cost_zero() {
        let usage = Usage::default();
        let cost = calculate_cost(&usage);
        assert_eq!(cost, "$0.0000");
    }

    #[test]
    fn test_calculate_cost_nonzero() {
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let cost = calculate_cost(&usage);
        assert_eq!(cost, "$0.0045");
    }
}
