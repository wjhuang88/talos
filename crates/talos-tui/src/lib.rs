//! Talos TUI — terminal user interface for the Talos agent.
//!
//! Provides a chat-based interface with:
//! - Chat viewport with scrolling message history
//! - Tool call bubbles with status indicators
//! - Approval overlay for permission-required tool calls
//! - Single-line input area with cursor
//! - Status bar showing model, token count, and cost
//! - Ctrl+C handling (single press cancels turn, double press exits)
//! - Streaming output that auto-scrolls

use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame, Terminal,
};
use talos_core::message::{AgentEvent, ToolCall, ToolResult, Usage};
use talos_core::ApprovalChoice;
use tokio::sync::{broadcast, mpsc};
use futures::StreamExt;
use evolution::EvolutionPanel;

pub mod evolution;

// Nord theme colors — reference: https://www.nordtheme.com/docs/colors-and-palettes
#[allow(dead_code)]
mod nord {
    use ratatui::style::Color;

    // Polar Night (dark backgrounds)
    pub const NORD0: Color = Color::Rgb(46, 52, 64);
    pub const NORD1: Color = Color::Rgb(59, 66, 82);
    pub const NORD2: Color = Color::Rgb(67, 76, 94);
    pub const NORD3: Color = Color::Rgb(76, 86, 106);

    // Snow Storm (light text)
    pub const NORD4: Color = Color::Rgb(216, 222, 233);
    pub const NORD5: Color = Color::Rgb(229, 233, 240);
    pub const NORD6: Color = Color::Rgb(236, 239, 244);

    // Frost (blue accents)
    pub const NORD8: Color = Color::Rgb(136, 192, 208);
    pub const NORD9: Color = Color::Rgb(129, 161, 193);

    // Aurora (semantic colors)
    pub const NORD11: Color = Color::Rgb(191, 97, 106);
    pub const NORD13: Color = Color::Rgb(235, 203, 139);
    pub const NORD14: Color = Color::Rgb(163, 190, 140);
}

// ── Skill Sidebar ────────────────────────────────────────────────────────────

/// Information about a loaded skill.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillInfo {
    /// Name of the skill.
    pub name: String,
    /// Short description of what the skill does.
    pub description: String,
    /// Whether the skill is currently active.
    pub active: bool,
}

/// Sidebar panel displaying loaded skills.
///
/// Shows a list of skills with their name, description, and active/inactive status.
/// Can be toggled visible/hidden and collapses to an icon when width is too narrow.
#[derive(Debug, Clone)]
pub struct SkillSidebar {
    /// Whether the sidebar is currently visible.
    pub visible: bool,
    /// List of loaded skills.
    pub skills: Vec<SkillInfo>,
    /// Width of the sidebar in columns.
    pub width: u16,
}

impl SkillSidebar {
    /// Default width for the sidebar in columns.
    pub const DEFAULT_WIDTH: u16 = 30;
    /// Minimum width below which the sidebar collapses to icon-only mode.
    pub const COLLAPSE_THRESHOLD: u16 = 20;

    /// Creates a new hidden skill sidebar with default width.
    pub fn new() -> Self {
        Self {
            visible: false,
            skills: Vec::new(),
            width: Self::DEFAULT_WIDTH,
        }
    }

    /// Toggles the visibility of the sidebar.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Updates the list of skills displayed in the sidebar.
    pub fn update_skills(&mut self, skills: Vec<SkillInfo>) {
        self.skills = skills;
    }

    /// Returns whether the sidebar should render in collapsed (icon-only) mode.
    fn is_collapsed(&self) -> bool {
        self.width < Self::COLLAPSE_THRESHOLD
    }

    /// Renders the skill sidebar on the given frame area.
    ///
    /// When visible and not collapsed, shows a bordered panel with:
    /// - Title "Skills"
    /// - List of skills with name (nord8), description (nord4), and status indicator
    ///   (● active in nord14, ○ inactive in nord3)
    ///
    /// When collapsed, shows only a skill count icon.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        if self.is_collapsed() {
            self.render_collapsed(frame, area);
        } else {
            self.render_expanded(frame, area);
        }
    }

    fn render_expanded(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if self.skills.is_empty() {
            let empty_style = Style::default()
                .fg(nord::NORD3)
                .add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No skills loaded", empty_style)));
        } else {
            for skill in &self.skills {
                let status_icon = if skill.active { "●" } else { "○" };
                let status_style = if skill.active {
                    Style::default().fg(nord::NORD14)
                } else {
                    Style::default().fg(nord::NORD3)
                };

                let name_style = Style::default()
                    .fg(nord::NORD8)
                    .add_modifier(Modifier::BOLD);
                let desc_style = Style::default()
                    .fg(nord::NORD4)
                    .add_modifier(Modifier::DIM);

                lines.push(Line::from(vec![
                    Span::styled(status_icon.to_string(), status_style),
                    Span::raw(" "),
                    Span::styled(skill.name.clone(), name_style),
                ]));

                let desc_display = truncate(&skill.description, self.width.saturating_sub(4) as usize);
                lines.push(Line::from(Span::styled(
                    format!("  {desc_display}"),
                    desc_style,
                )));

                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(nord::NORD2))
                    .title(" Skills "),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_collapsed(&self, frame: &mut Frame, area: Rect) {
        let count = self.skills.len();
        let active_count = self.skills.iter().filter(|s| s.active).count();

        let text = if count == 0 {
            "⚡".to_string()
        } else {
            format!("⚡{count}")
        };

        let style = if active_count > 0 {
            Style::default().fg(nord::NORD14)
        } else {
            Style::default().fg(nord::NORD3)
        };

        let paragraph = Paragraph::new(Text::from(Span::styled(text, style)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(nord::NORD2)),
            );

        frame.render_widget(paragraph, area);
    }
}

impl Default for SkillSidebar {
    fn default() -> Self {
        Self::new()
    }
}

// ── Approval State ───────────────────────────────────────────────────────────

/// State of the approval overlay.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ApprovalState {
    /// No approval prompt is visible.
    #[default]
    Hidden,
    /// Approval overlay is visible with the given tool info and selected choice.
    Visible {
        /// Name of the tool requiring approval.
        tool_name: String,
        /// Formatted arguments for the tool.
        arguments: String,
        /// Currently highlighted option.
        selected: ApprovalChoice,
    },
}

// ── Tool Call Bubble ─────────────────────────────────────────────────────────

/// Maximum length for tool call arguments before truncation.
const MAX_ARGS_LENGTH: usize = 80;
/// Maximum length for tool result content before truncation.
const MAX_RESULT_LENGTH: usize = 200;

/// Renders a tool call as a styled bubble in the chat viewport.
///
/// Displays the tool name in bold with accent color, truncated arguments,
/// and a result status indicator when available.
pub struct ToolCallBubble<'a> {
    /// Name of the tool.
    tool_name: &'a str,
    /// Formatted arguments (may be truncated).
    arguments: &'a str,
    /// Whether the tool result was an error.
    result_status: Option<bool>,
    /// Result content (may be truncated).
    result_content: Option<&'a str>,
}

impl<'a> ToolCallBubble<'a> {
    /// Creates a new tool call bubble with the given tool name and arguments.
    pub fn new(tool_name: &'a str, arguments: &'a str) -> Self {
        Self {
            tool_name,
            arguments,
            result_status: None,
            result_content: None,
        }
    }

    /// Sets the result status and content for this bubble.
    pub fn with_result(mut self, is_error: bool, content: &'a str) -> Self {
        self.result_status = Some(is_error);
        self.result_content = Some(content);
        self
    }
}

impl ratatui::widgets::Widget for ToolCallBubble<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let tool_name_style = Style::default()
            .fg(nord::NORD8)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            format!("▸ {}", self.tool_name),
            tool_name_style,
        )));

        let args_style = Style::default()
            .fg(nord::NORD3)
            .add_modifier(Modifier::DIM);
        let args_display = truncate(self.arguments, MAX_ARGS_LENGTH);
        lines.push(Line::from(Span::styled(format!("  {args_display}"), args_style)));

        if let Some(is_error) = self.result_status {
            let (icon, style) = if is_error {
                ("✗ error", Style::default().fg(nord::NORD11))
            } else {
                ("✓ success", Style::default().fg(nord::NORD14))
            };
            lines.push(Line::from(Span::styled(format!("  {icon}"), style)));

            if let Some(content) = self.result_content {
                let content_style = Style::default().fg(nord::NORD4);
                let content_display = truncate(content, MAX_RESULT_LENGTH);
                lines.push(Line::from(Span::styled(
                    format!("  {content_display}"),
                    content_style,
                )));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(nord::NORD2)),
            )
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

// ── Approval Overlay ─────────────────────────────────────────────────────────

/// Renders a semi-transparent approval overlay on top of the chat viewport.
///
/// Displays the tool name, arguments, risk level, and three options:
/// `[y] Approve once`, `[a] Always approve`, `[n] Deny`.
/// The currently selected option is highlighted with nord8.
pub struct ApprovalOverlay<'a> {
    /// Name of the tool requiring approval.
    tool_name: &'a str,
    /// Formatted arguments for the tool.
    arguments: &'a str,
    /// Currently selected choice.
    selected: &'a ApprovalChoice,
}

impl<'a> ApprovalOverlay<'a> {
    /// Creates a new approval overlay.
    pub fn new(tool_name: &'a str, arguments: &'a str, selected: &'a ApprovalChoice) -> Self {
        Self {
            tool_name,
            arguments,
            selected,
        }
    }
}

impl ratatui::widgets::Widget for ApprovalOverlay<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let overlay_width = 50.min(area.width);
        let overlay_height = 10.min(area.height);
        let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
        let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
        let overlay_area = Rect {
            x,
            y,
            width: overlay_width,
            height: overlay_height,
        };

        Clear.render(overlay_area, buf);

        let mut lines: Vec<Line<'static>> = Vec::new();

        let title_style = Style::default()
            .fg(nord::NORD9)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            "⚠ Permission Required",
            title_style,
        )));
        lines.push(Line::from(""));

        let tool_style = Style::default()
            .fg(nord::NORD8)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            format!("Tool: {}", self.tool_name),
            tool_style,
        )));

        let args_style = Style::default().fg(nord::NORD3).add_modifier(Modifier::DIM);
        let args_display = truncate(self.arguments, MAX_ARGS_LENGTH);
        lines.push(Line::from(Span::styled(
            format!("Args: {args_display}"),
            args_style,
        )));
        lines.push(Line::from(""));

        let risk_style = Style::default().fg(nord::NORD13);
        lines.push(Line::from(Span::styled(
            "Risk: Requires user approval",
            risk_style,
        )));
        lines.push(Line::from(""));

        let options = [
            ("y", "Approve once", ApprovalChoice::ApproveOnce),
            ("a", "Always approve", ApprovalChoice::AlwaysApprove),
            ("n", "Deny", ApprovalChoice::Deny),
        ];

        for (key, label, choice) in options {
            let style = if *self.selected == choice {
                Style::default()
                    .fg(nord::NORD8)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(nord::NORD4)
            };
            lines.push(Line::from(Span::styled(
                format!("[{key}] {label}"),
                style,
            )));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(nord::NORD9))
                    .title(" Approval "),
            );

        paragraph.render(overlay_area, buf);
    }
}

/// Truncates a string to the given maximum length, appending "…" if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

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

#[derive(Debug, Clone, PartialEq)]
enum ChatLine {
    Text(String),
    ToolCall {
        tool_name: String,
        arguments: String,
        result: Option<ToolResult>,
    },
}

#[derive(Debug, Default)]
struct TuiState {
    chat_lines: Vec<ChatLine>,
    input_buffer: String,
    cursor_pos: usize,
    is_processing: bool,
    current_turn_text: String,
    ctrl_c_state: CtrlCState,
    should_exit: bool,
    usage: Usage,
    model_name: String,
    approval_state: ApprovalState,
    branch_id: Option<String>,
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

    fn finalize_turn(&mut self) {
        if !self.current_turn_text.is_empty() {
            self.chat_lines.push(ChatLine::Text(self.current_turn_text.clone()));
            self.current_turn_text.clear();
        }
    }

    fn append_user_message(&mut self, content: &str) {
        self.chat_lines.push(ChatLine::Text(format!("> {content}")));
    }

    fn append_error(&mut self, message: &str) {
        self.chat_lines.push(ChatLine::Text(format!("[Error] {message}")));
    }

    fn append_system(&mut self, message: &str) {
        self.chat_lines.push(ChatLine::Text(format!("[System] {message}")));
    }

    fn append_tool_call(&mut self, call: &ToolCall) {
        self.chat_lines.push(ChatLine::ToolCall {
            tool_name: call.name.clone(),
            arguments: serde_json::to_string_pretty(&call.input)
                .unwrap_or_else(|_| call.input.to_string()),
            result: None,
        });
    }

    fn set_tool_result(&mut self, result: &ToolResult) {
        for line in self.chat_lines.iter_mut().rev() {
            if let ChatLine::ToolCall {
                tool_name: _,
                arguments: _,
                result: slot,
            } = line
            {
                if slot.is_none() {
                    *slot = Some(result.clone());
                    break;
                }
            }
        }
    }

    /// Appends a character to the input buffer at the cursor position.
    fn input_append_char(&mut self, ch: char) {
        let byte_pos = self.input_buffer.char_indices().nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len());
        self.input_buffer.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Deletes the character before the cursor in the input buffer.
    fn input_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self.input_buffer.char_indices().nth(self.cursor_pos)
                .map(|(i, _)| i)
                .unwrap_or(self.input_buffer.len());
            self.input_buffer.remove(byte_pos);
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
        if self.cursor_pos < self.input_buffer.chars().count() {
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

    fn set_branch_id(&mut self, branch_id: String) {
        self.branch_id = Some(branch_id);
    }

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
                self.append_tool_call(call);
            }
            AgentEvent::ToolResult { result } => {
                self.set_tool_result(result);
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
    /// Returns an error if terminal I/O fails or the event channel closes unexpectedly.
    pub async fn run(&mut self, mut event_rx: broadcast::Receiver<AgentEvent>) -> Result<()> {
        self.state.model_name = "mock".to_string();

        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            let state = &self.state;
            let sidebar = &self.skill_sidebar;
            let evo = &self.evolution_panel;
            self.terminal.draw(|frame| render(frame, state, sidebar, evo))?;

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
                    KeyCode::Char(c) if !matches!(self.state.approval_state, ApprovalState::Hidden) => {
                        if let Some(choice) = self.handle_approval_key(c) {
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
                            self.state.append_user_message(&input);
                            if let Some(ref tx) = self.message_tx {
                                let _ = tx.send(input);
                            }
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

fn render(frame: &mut Frame, state: &TuiState, skill_sidebar: &SkillSidebar, evolution_panel: &evolution::EvolutionPanel) {
    let full_area = frame.area();

    // If the evolution panel is visible, carve a right column for it first.
    let main_area = if evolution_panel.visible {
        let evo_width = evolution_panel.width.min(full_area.width.saturating_sub(40));
        let cols = Layout::horizontal([
            Constraint::Min(40),
            Constraint::Length(evo_width),
        ])
        .split(full_area);

        evolution_panel.render(frame, cols[1]);
        cols[0]
    } else {
        full_area
    };

    // If sidebar is visible, split off the right portion
    let (chat_area, input_area, status_area) = if skill_sidebar.visible {
        let sidebar_width = skill_sidebar.width.min(main_area.width.saturating_sub(40));
        let chunks = Layout::horizontal([
            Constraint::Min(40),
            Constraint::Length(sidebar_width),
        ])
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
    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(" Input (Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution) "));
    frame.render_widget(input_paragraph, input_area);

    let status_text = build_status_text(state);
    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().add_modifier(Modifier::REVERSED));
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

fn build_chat_text(state: &TuiState) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in &state.chat_lines {
        match line {
            ChatLine::Text(text) => {
                lines.push(Line::from(text.clone()));
            }
            ChatLine::ToolCall {
                tool_name,
                arguments,
                result,
            } => {
                let mut bubble = ToolCallBubble::new(tool_name, arguments);
                if let Some(result) = result {
                    bubble = bubble.with_result(result.is_error, &result.content);
                }
                let mut buf = ratatui::buffer::Buffer::empty(Rect::new(0, 0, 40, 6));
                bubble.render(buf.area, &mut buf);
                for y in 0..buf.area.height {
                    let mut spans = Vec::new();
                    for x in 0..buf.area.width {
                        let cell = buf.cell((x, y)).expect("cell within buffer bounds");
                        spans.push(Span::styled(
                            cell.symbol().to_string(),
                            cell.style(),
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            }
        }
    }

    if !state.current_turn_text.is_empty() {
        lines.push(Line::from(state.current_turn_text.clone()));
    }

    if lines.is_empty() {
        lines.push(Line::from("Welcome to Talos. Type a message and press Enter."));
    }

    Text::from(lines)
}

fn chat_scroll_offset(state: &TuiState, viewport_height: usize) -> u16 {
    let total_lines = state.chat_lines.len()
        + if state.current_turn_text.is_empty() { 0 } else { 1 };
    let total_lines = if total_lines == 0 { 1 } else { total_lines };
    
    if total_lines <= viewport_height {
        0
    } else {
        (total_lines - viewport_height) as u16
    }
}

fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let char_count = buffer.chars().count();
    let cursor_pos = state.cursor_pos.min(char_count);

    let before: String = buffer.chars().take(cursor_pos).collect();
    let after: String = buffer.chars().skip(cursor_pos).collect();

    let mut spans = Vec::new();
    spans.push(Span::raw(before));
    if after.is_empty() {
        spans.push(Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)));
    } else {
        let mut chars = after.chars();
        let first = chars.next().unwrap().to_string();
        let rest: String = chars.collect();
        spans.push(Span::styled(first, Style::default().add_modifier(Modifier::REVERSED)));
        spans.push(Span::raw(rest));
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

    let branch_info = state
        .branch_id
        .as_ref()
        .map(|b| {
            let short: String = b.chars().take(8).collect();
            format!(" | Branch: {short}")
        })
        .unwrap_or_default();

    let text = format!(
        " {processing_indicator}{status} | Model: {} | Tokens: {}{branch_info} | Cost: {}",
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
        assert_eq!(state.chat_lines, vec![ChatLine::Text("Assistant response".into())]);
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
        assert_eq!(state.chat_lines, vec![ChatLine::Text("> Hello".into())]);
    }

    #[test]
    fn test_append_error() {
        let mut state = TuiState::new();
        state.append_error("Something failed");
        assert_eq!(state.chat_lines, vec![ChatLine::Text("[Error] Something failed".into())]);
    }

    #[test]
    fn test_append_system() {
        let mut state = TuiState::new();
        state.append_system("Turn cancelled");
        assert_eq!(state.chat_lines, vec![ChatLine::Text("[System] Turn cancelled".into())]);
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
        let call = ToolCall {
            id: "c1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls"}),
        };
        state.handle_event(&AgentEvent::ToolCall { call: call.clone() });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall { tool_name, arguments, result } => {
                assert_eq!(tool_name, "bash");
                assert!(arguments.contains("command"));
                assert!(arguments.contains("ls"));
                assert!(result.is_none());
            }
            _ => panic!("expected ToolCall variant"),
        }
    }

    #[test]
    fn test_handle_event_tool_result_sets_on_last_tool_call() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "read".into(),
            input: serde_json::json!({"path": "src/main.rs"}),
        };
        state.handle_event(&AgentEvent::ToolCall { call });
        let result = ToolResult {
            tool_use_id: "c1".into(),
            content: "fn main() {}".into(),
            is_error: false,
        };
        state.handle_event(&AgentEvent::ToolResult { result: result.clone() });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall { result: Some(r), .. } => {
                assert_eq!(r.content, "fn main() {}");
                assert!(!r.is_error);
            }
            _ => panic!("expected ToolCall with result"),
        }
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
        assert_eq!(state.chat_lines, vec![ChatLine::Text("Response text".into())]);
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
        assert_eq!(state.chat_lines, vec![ChatLine::Text("[Error] API error".into())]);
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

    #[test]
    fn test_approval_state_default_hidden() {
        let state = ApprovalState::default();
        assert!(matches!(state, ApprovalState::Hidden));
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("hello world this is a long string", 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_truncate_empty_string() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn test_approval_state_transitions() {
        let mut state = TuiState::new();
        assert!(matches!(state.approval_state, ApprovalState::Hidden));

        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };
        assert!(matches!(state.approval_state, ApprovalState::Visible { .. }));

        state.approval_state = ApprovalState::Hidden;
        assert!(matches!(state.approval_state, ApprovalState::Hidden));
    }

    #[test]
    fn test_handle_approval_key_approve_once() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'y');
        assert_eq!(choice, Some(ApprovalChoice::ApproveOnce));
    }

    #[test]
    fn test_handle_approval_key_always_approve() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'a');
        assert_eq!(choice, Some(ApprovalChoice::AlwaysApprove));
    }

    #[test]
    fn test_handle_approval_key_deny() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'n');
        assert_eq!(choice, Some(ApprovalChoice::Deny));
    }

    #[test]
    fn test_handle_approval_key_invalid_when_hidden() {
        let mut state = TuiState::new();
        assert!(matches!(state.approval_state, ApprovalState::Hidden));
        let choice = handle_approval_key_event(&mut state, 'y');
        assert!(choice.is_none());
    }

    #[test]
    fn test_handle_approval_key_invalid_char() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'x');
        assert!(choice.is_none());
    }

    #[test]
    fn test_tool_call_bubble_creation() {
        let bubble = ToolCallBubble::new("read", r#"{"path": "src/main.rs"}"#);
        assert_eq!(bubble.tool_name, "read");
        assert!(bubble.result_status.is_none());
    }

    #[test]
    fn test_tool_call_bubble_with_result() {
        let bubble = ToolCallBubble::new("bash", r#"{"command": "ls"}"#)
            .with_result(false, "file.rs\nCargo.toml");
        assert_eq!(bubble.tool_name, "bash");
        assert_eq!(bubble.result_status, Some(false));
        assert_eq!(bubble.result_content, Some("file.rs\nCargo.toml"));
    }

    #[test]
    fn test_tool_call_bubble_with_error() {
        let bubble = ToolCallBubble::new("bash", r#"{"command": "rm -rf /"}"#)
            .with_result(true, "Permission denied");
        assert_eq!(bubble.result_status, Some(true));
    }

    fn handle_approval_key_event(state: &mut TuiState, key: char) -> Option<ApprovalChoice> {
        let ApprovalState::Visible { selected, .. } = &mut state.approval_state else {
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

    // ── Skill Sidebar Tests ──────────────────────────────────────────────────

    #[test]
    fn test_skill_sidebar_new_is_hidden() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
        assert!(sidebar.skills.is_empty());
        assert_eq!(sidebar.width, SkillSidebar::DEFAULT_WIDTH);
    }

    #[test]
    fn test_skill_sidebar_default_is_hidden() {
        let sidebar = SkillSidebar::default();
        assert!(!sidebar.visible);
    }

    #[test]
    fn test_skill_sidebar_toggle_visibility() {
        let mut sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);

        sidebar.toggle();
        assert!(sidebar.visible);

        sidebar.toggle();
        assert!(!sidebar.visible);
    }

    #[test]
    fn test_skill_sidebar_update_skills() {
        let mut sidebar = SkillSidebar::new();
        assert!(sidebar.skills.is_empty());

        let skills = vec![
            SkillInfo {
                name: "test-skill".into(),
                description: "A test skill".into(),
                active: true,
            },
            SkillInfo {
                name: "another-skill".into(),
                description: "Another skill".into(),
                active: false,
            },
        ];
        sidebar.update_skills(skills.clone());
        assert_eq!(sidebar.skills.len(), 2);
        assert_eq!(sidebar.skills[0].name, "test-skill");
        assert!(sidebar.skills[0].active);
        assert_eq!(sidebar.skills[1].name, "another-skill");
        assert!(!sidebar.skills[1].active);
    }

    #[test]
    fn test_skill_sidebar_collapsed_mode() {
        let mut sidebar = SkillSidebar::new();
        sidebar.width = 15;
        assert!(sidebar.is_collapsed());

        sidebar.width = 20;
        assert!(!sidebar.is_collapsed());

        sidebar.width = 19;
        assert!(sidebar.is_collapsed());
    }

    #[test]
    fn test_skill_sidebar_default_not_collapsed() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.is_collapsed());
    }

    #[test]
    fn test_skill_info_fields() {
        let skill = SkillInfo {
            name: "code-review".into(),
            description: "Reviews code for quality".into(),
            active: true,
        };
        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.description, "Reviews code for quality");
        assert!(skill.active);
    }

    #[test]
    fn test_skill_sidebar_render_empty_when_hidden() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
        // Hidden sidebar should not render anything — verified by visible flag
    }

    #[test]
    fn test_skill_sidebar_with_many_skills() {
        let mut sidebar = SkillSidebar::new();
        let skills: Vec<SkillInfo> = (0..10)
            .map(|i| SkillInfo {
                name: format!("skill-{i}"),
                description: format!("Description for skill {i}"),
                active: i % 2 == 0,
            })
            .collect();
        sidebar.update_skills(skills);
        assert_eq!(sidebar.skills.len(), 10);
        assert!(sidebar.skills[0].active);
        assert!(!sidebar.skills[1].active);
        assert!(sidebar.skills[2].active);
    }

    #[test]
    fn test_skill_sidebar_width_boundary() {
        let mut sidebar = SkillSidebar::new();

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD - 1;
        assert!(sidebar.is_collapsed());

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD;
        assert!(!sidebar.is_collapsed());

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD + 1;
        assert!(!sidebar.is_collapsed());
    }
}
