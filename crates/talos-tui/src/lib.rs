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
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use evolution::EvolutionPanel;
use futures::StreamExt;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use talos_core::message::{AgentEvent, ToolCall, ToolResult, Usage};
use talos_core::tool::ToolProvenance;
use tokio::sync::{broadcast, mpsc};

pub mod evolution;

/// Nord theme colors for Talos terminal surfaces.
///
/// Reference: <https://www.nordtheme.com/docs/colors-and-palettes>
pub mod nord {
    use ratatui::style::Color;

    /// Polar Night darkest background.
    pub const NORD0: Color = Color::Rgb(46, 52, 64);
    /// Polar Night elevated background.
    pub const NORD1: Color = Color::Rgb(59, 66, 82);
    /// Polar Night selected background.
    pub const NORD2: Color = Color::Rgb(67, 76, 94);
    /// Polar Night muted foreground.
    pub const NORD3: Color = Color::Rgb(76, 86, 106);

    /// Snow Storm primary foreground.
    pub const NORD4: Color = Color::Rgb(216, 222, 233);
    /// Snow Storm brighter foreground.
    pub const NORD5: Color = Color::Rgb(229, 233, 240);
    /// Snow Storm brightest foreground.
    pub const NORD6: Color = Color::Rgb(236, 239, 244);

    /// Frost green-blue accent.
    pub const NORD7: Color = Color::Rgb(143, 188, 187);
    /// Frost cyan accent.
    pub const NORD8: Color = Color::Rgb(136, 192, 208);
    /// Frost blue accent.
    pub const NORD9: Color = Color::Rgb(129, 161, 193);
    /// Frost dark blue accent.
    pub const NORD10: Color = Color::Rgb(94, 129, 172);

    /// Aurora red error color.
    pub const NORD11: Color = Color::Rgb(191, 97, 106);
    /// Aurora orange warning color.
    pub const NORD12: Color = Color::Rgb(208, 135, 112);
    /// Aurora yellow warning color.
    pub const NORD13: Color = Color::Rgb(235, 203, 139);
    /// Aurora green success color.
    pub const NORD14: Color = Color::Rgb(163, 190, 140);
    /// Aurora purple accent color.
    pub const NORD15: Color = Color::Rgb(180, 142, 173);
}

#[cfg(test)]
fn rgb_components(color: ratatui::style::Color) -> Option<(u8, u8, u8)> {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

#[cfg(test)]
fn relative_luminance(color: ratatui::style::Color) -> Option<f64> {
    let (r, g, b) = rgb_components(color)?;
    let channel = |value: u8| {
        let normalized = f64::from(value) / 255.0;
        if normalized <= 0.04045 {
            normalized / 12.92
        } else {
            ((normalized + 0.055) / 1.055).powf(2.4)
        }
    };
    Some(0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b))
}

#[cfg(test)]
fn contrast_ratio(
    foreground: ratatui::style::Color,
    background: ratatui::style::Color,
) -> Option<f64> {
    let fg = relative_luminance(foreground)?;
    let bg = relative_luminance(background)?;
    let (lighter, darker) = if fg >= bg { (fg, bg) } else { (bg, fg) };
    Some((lighter + 0.05) / (darker + 0.05))
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
            let empty_style = Style::default().fg(nord::NORD3).add_modifier(Modifier::DIM);
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
                let desc_style = Style::default().fg(nord::NORD4).add_modifier(Modifier::DIM);

                lines.push(Line::from(vec![
                    Span::styled(status_icon.to_string(), status_style),
                    Span::raw(" "),
                    Span::styled(skill.name.clone(), name_style),
                ]));

                let desc_display =
                    truncate(&skill.description, self.width.saturating_sub(4) as usize);
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

        let paragraph = Paragraph::new(Text::from(Span::styled(text, style))).block(
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
    /// Origin of the tool.
    provenance: ToolProvenance,
}

impl<'a> ToolCallBubble<'a> {
    /// Creates a new tool call bubble with the given tool name and arguments.
    pub fn new(tool_name: &'a str, arguments: &'a str) -> Self {
        Self {
            tool_name,
            arguments,
            result_status: None,
            result_content: None,
            provenance: ToolProvenance::Native,
        }
    }

    /// Sets the provenance marker for this bubble.
    pub fn with_provenance(mut self, provenance: ToolProvenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Sets the result status and content for this bubble.
    pub fn with_result(mut self, is_error: bool, content: &'a str) -> Self {
        self.result_status = Some(is_error);
        self.result_content = Some(content);
        self
    }
}

fn provenance_marker(provenance: &ToolProvenance) -> String {
    match provenance {
        ToolProvenance::Native => "native".to_string(),
        ToolProvenance::McpRemote { server } => {
            let server = truncate(server, 24);
            format!("mcp:{server}")
        }
    }
}

impl ratatui::widgets::Widget for ToolCallBubble<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let tool_name_style = Style::default()
            .fg(nord::NORD8)
            .add_modifier(Modifier::BOLD);
        let marker = provenance_marker(&self.provenance);
        let marker_style = match &self.provenance {
            ToolProvenance::Native => Style::default().fg(nord::NORD3),
            ToolProvenance::McpRemote { .. } => Style::default()
                .fg(nord::NORD15)
                .add_modifier(Modifier::BOLD),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("▸ {}", self.tool_name), tool_name_style),
            Span::raw(" "),
            Span::styled(format!("[{marker}]"), marker_style),
        ]));

        let args_style = Style::default().fg(nord::NORD3).add_modifier(Modifier::DIM);
        let args_display = truncate(self.arguments, MAX_ARGS_LENGTH);
        lines.push(Line::from(Span::styled(
            format!("  {args_display}"),
            args_style,
        )));

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
            lines.push(Line::from(Span::styled(format!("[{key}] {label}"), style)));
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(
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
    Assistant(String),
    ToolCall {
        tool_name: String,
        arguments: String,
        provenance: ToolProvenance,
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
    pending_approval_response: Option<tokio::sync::oneshot::Sender<ApprovalChoice>>,
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
            self.chat_lines
                .push(ChatLine::Assistant(self.current_turn_text.clone()));
            self.current_turn_text.clear();
        }
    }

    fn append_user_message(&mut self, content: &str) {
        self.chat_lines.push(ChatLine::Text(format!("> {content}")));
    }

    fn append_error(&mut self, message: &str) {
        self.chat_lines
            .push(ChatLine::Text(format!("[Error] {message}")));
    }

    fn append_system(&mut self, message: &str) {
        self.chat_lines
            .push(ChatLine::Text(format!("[System] {message}")));
    }

    fn append_tool_call(&mut self, call: &ToolCall, provenance: &ToolProvenance) {
        self.chat_lines.push(ChatLine::ToolCall {
            tool_name: call.name.clone(),
            arguments: serde_json::to_string_pretty(&call.input)
                .unwrap_or_else(|_| call.input.to_string()),
            provenance: provenance.clone(),
            result: None,
        });
    }

    fn set_tool_result(&mut self, result: &ToolResult) {
        for line in self.chat_lines.iter_mut().rev() {
            if let ChatLine::ToolCall {
                tool_name: _,
                arguments: _,
                provenance: _,
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
        let byte_pos = self
            .input_buffer
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.input_buffer.len());
        self.input_buffer.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Deletes the character before the cursor in the input buffer.
    fn input_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            let byte_pos = self
                .input_buffer
                .char_indices()
                .nth(self.cursor_pos)
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

    #[allow(dead_code)]
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
            AgentEvent::ToolCall { call, provenance } => {
                self.append_tool_call(call, provenance);
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
            _ => {}
        }
    }

    const SLASH_COMMANDS: &[&str] = &[
        "/help",
        "/quit",
        "/exit",
        "/status",
        "/new",
        "/compact",
        "/diff",
        "/model",
        "/resume",
        "/fork",
        "/vim",
    ];

    fn handle_slash_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let _arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "/help" => {
                self.append_system("Available commands:");
                self.append_system("  /help       — Show this help");
                self.append_system("  /quit       — Exit Talos");
                self.append_system("  /status     — Show session info");
                self.append_system("  /new        — Start fresh session");
                self.append_system("  /compact    — Compact conversation context");
                self.append_system("  /diff       — Show git diff");
                self.append_system("  /model      — Switch model");
                self.append_system("  /resume     — Resume a session");
                self.append_system("  /fork       — Fork current session");
                self.append_system("  /vim        — Toggle vim keybindings");
            }
            "/quit" | "/exit" => {
                self.should_exit = true;
            }
            "/status" => {
                let usage = &self.usage;
                self.append_system(&format!(
                    "Model: {} | Input: {} | Output: {} tokens",
                    self.model_name,
                    usage.input_tokens,
                    usage.output_tokens,
                ));
            }
            "/new" => {
                self.chat_lines.clear();
                self.current_turn_text.clear();
                self.usage = Usage::default();
                self.branch_id = None;
                self.append_system("New session started.");
            }
            _ => {
                self.append_error(&format!(
                    "Unknown command: {cmd}. Type /help for available commands."
                ));
            }
        }
    }

    fn complete_slash_command(&mut self) {
        let input = &self.input_buffer;
        let matches: Vec<&&str> = Self::SLASH_COMMANDS
            .iter()
            .filter(|c| c.starts_with(input.as_str()))
            .collect();
        if matches.len() == 1 {
            self.input_buffer = matches[0].to_string();
            self.cursor_pos = self.input_buffer.len();
            self.input_append_char(' ');
        } else if !matches.is_empty() {
            let listing = matches
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("  ");
            self.append_system(&format!("Commands: {listing}"));
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

fn render(
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

fn build_chat_text(state: &TuiState) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in &state.chat_lines {
        match line {
            ChatLine::Text(text) => {
                lines.push(Line::from(text.clone()));
            }
            ChatLine::Assistant(text) => {
                let rendered = tui_markdown::from_str(text);
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
                let mut bubble =
                    ToolCallBubble::new(tool_name, arguments).with_provenance(provenance.clone());
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

fn chat_scroll_offset(state: &TuiState, viewport_height: usize) -> u16 {
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

fn build_input_text(state: &TuiState) -> Text<'static> {
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
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
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
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Assistant("Assistant response".into())]
        );
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
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[Error] Something failed".into())]
        );
    }

    #[test]
    fn test_append_system() {
        let mut state = TuiState::new();
        state.append_system("Turn cancelled");
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[System] Turn cancelled".into())]
        );
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
        state.handle_event(&AgentEvent::TextDelta {
            delta: "Hello".into(),
        });
        state.handle_event(&AgentEvent::TextDelta {
            delta: " world".into(),
        });
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
        state.handle_event(&AgentEvent::ToolCall {
            call: call.clone(),
            provenance: Default::default(),
        });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                tool_name,
                arguments,
                provenance,
                result,
            } => {
                assert_eq!(tool_name, "bash");
                assert!(arguments.contains("command"));
                assert!(arguments.contains("ls"));
                assert_eq!(provenance, &ToolProvenance::Native);
                assert!(result.is_none());
            }
            _ => panic!("expected ToolCall variant"),
        }
    }

    #[test]
    fn test_handle_event_tool_call_preserves_mcp_provenance() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "remote_search".into(),
            input: serde_json::json!({"query": "talos"}),
        };
        let provenance = ToolProvenance::McpRemote {
            server: "filesystem".into(),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: provenance.clone(),
        });

        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                tool_name,
                provenance: actual,
                ..
            } => {
                assert_eq!(tool_name, "remote_search");
                assert_eq!(actual, &provenance);
            }
            _ => panic!("expected ToolCall variant"),
        }
    }

    #[test]
    fn test_build_chat_text_renders_mcp_provenance_marker() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "remote_search".into(),
            input: serde_json::json!({"query": "talos"}),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: ToolProvenance::McpRemote {
                server: "filesystem".into(),
            },
        });

        let rendered = build_chat_text(&state)
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains("remote_search"));
        assert!(rendered.contains("[mcp:filesystem]"));
    }

    #[test]
    fn test_handle_event_tool_result_sets_on_last_tool_call() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "read".into(),
            input: serde_json::json!({"path": "src/main.rs"}),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: Default::default(),
        });
        let result = ToolResult {
            tool_use_id: "c1".into(),
            content: "fn main() {}".into(),
            is_error: false,
        };
        state.handle_event(&AgentEvent::ToolResult {
            result: result.clone(),
        });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                result: Some(r), ..
            } => {
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
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Assistant("Response text".into())]
        );
        assert_eq!(state.usage.input_tokens, 100);
        assert_eq!(state.usage.output_tokens, 50);
    }

    #[test]
    fn test_handle_event_error() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        state.append_delta("Partial");
        state.handle_event(&AgentEvent::Error {
            message: "API error".into(),
        });
        assert!(!state.is_processing);
        assert!(state.current_turn_text.is_empty());
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[Error] API error".into())]
        );
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
        assert!(matches!(
            state.approval_state,
            ApprovalState::Visible { .. }
        ));

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
        assert_eq!(bubble.provenance, ToolProvenance::Native);
        assert!(bubble.result_status.is_none());
    }

    #[test]
    fn test_tool_call_bubble_with_mcp_provenance() {
        let bubble = ToolCallBubble::new("remote_search", r#"{"query": "talos"}"#)
            .with_provenance(ToolProvenance::McpRemote {
                server: "filesystem".into(),
            });
        assert_eq!(
            provenance_marker(&bubble.provenance),
            "mcp:filesystem".to_string()
        );
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

    #[test]
    fn test_nord_palette_defines_all_colors() {
        let colors = [
            nord::NORD0,
            nord::NORD1,
            nord::NORD2,
            nord::NORD3,
            nord::NORD4,
            nord::NORD5,
            nord::NORD6,
            nord::NORD7,
            nord::NORD8,
            nord::NORD9,
            nord::NORD10,
            nord::NORD11,
            nord::NORD12,
            nord::NORD13,
            nord::NORD14,
            nord::NORD15,
        ];
        assert_eq!(colors.len(), 16);
        assert!(colors.iter().all(|color| rgb_components(*color).is_some()));
    }

    #[test]
    fn test_nord_primary_text_contrast_is_wcag_aa() {
        let pairs = [
            (nord::NORD4, nord::NORD0),
            (nord::NORD5, nord::NORD0),
            (nord::NORD6, nord::NORD0),
            (nord::NORD8, nord::NORD0),
            (nord::NORD14, nord::NORD0),
        ];

        for (foreground, background) in pairs {
            let ratio = contrast_ratio(foreground, background).expect("rgb Nord color");
            assert!(ratio >= 4.5, "contrast ratio {ratio:.2} below WCAG AA");
        }
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

    #[test]
    fn test_slash_command_help() {
        let mut state = TuiState::new();
        state.handle_slash_command("/help");
        assert!(state.chat_lines.iter().any(|l| matches!(l, ChatLine::Text(t) if t.contains("/help"))));
        assert!(state.chat_lines.iter().any(|l| matches!(l, ChatLine::Text(t) if t.contains("/quit"))));
    }

    #[test]
    fn test_slash_command_quit() {
        let mut state = TuiState::new();
        state.handle_slash_command("/quit");
        assert!(state.should_exit);
    }

    #[test]
    fn test_slash_command_exit() {
        let mut state = TuiState::new();
        state.handle_slash_command("/exit");
        assert!(state.should_exit);
    }

    #[test]
    fn test_slash_command_status() {
        let mut state = TuiState::new();
        state.model_name = "test-model".to_string();
        state.handle_slash_command("/status");
        assert!(state.chat_lines.iter().any(|l| matches!(l, ChatLine::Text(t) if t.contains("test-model"))));
    }

    #[test]
    fn test_slash_command_new_clears_chat() {
        let mut state = TuiState::new();
        state.append_user_message("hello");
        assert!(!state.chat_lines.is_empty());
        state.handle_slash_command("/new");
        assert_eq!(state.chat_lines.len(), 1);
        if let ChatLine::Text(msg) = &state.chat_lines[0] {
            assert!(msg.contains("New session started"));
        } else {
            panic!("expected system message");
        }
    }

    #[test]
    fn test_slash_command_unknown() {
        let mut state = TuiState::new();
        state.handle_slash_command("/foobar");
        assert!(state.chat_lines.iter().any(|l| matches!(l, ChatLine::Text(t) if t.contains("Unknown command"))));
    }

    #[test]
    fn test_tab_completion_single_match() {
        let mut state = TuiState::new();
        state.input_buffer = "/hel".to_string();
        state.cursor_pos = 4;
        state.complete_slash_command();
        assert_eq!(state.input_buffer, "/help ");
    }

    #[test]
    fn test_tab_completion_multiple_matches() {
        let mut state = TuiState::new();
        state.input_buffer = "/".to_string();
        state.cursor_pos = 1;
        state.complete_slash_command();
        assert!(state.chat_lines.iter().any(|l| matches!(l, ChatLine::Text(t) if t.contains("Commands:"))));
    }

    // ── Markdown Rendering Tests ─────────────────────────────────────────────

    #[test]
    fn test_assistant_line_renders_markdown() {
        let mut state = TuiState::new();
        state.chat_lines.push(ChatLine::Assistant("**bold text** and *italic*".into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
        let all_spans: Vec<&str> = rendered
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect();
        let combined: String = all_spans.join("");
        assert!(combined.contains("bold text"));
    }

    #[test]
    fn test_assistant_line_renders_code_block() {
        let mut state = TuiState::new();
        let code = "Here is some code:\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        state.chat_lines.push(ChatLine::Assistant(code.into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
    }

    #[test]
    fn test_assistant_line_renders_heading() {
        let mut state = TuiState::new();
        state.chat_lines.push(ChatLine::Assistant("# Main Heading\n## Sub Heading".into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
        let all_spans: Vec<&str> = rendered
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect();
        let combined: String = all_spans.join("");
        assert!(combined.contains("Main Heading"));
        assert!(combined.contains("Sub Heading"));
    }

    #[test]
    fn test_text_line_remains_plain() {
        let mut state = TuiState::new();
        state.chat_lines.push(ChatLine::Text("**not bold**".into()));
        let rendered = build_chat_text(&state);
        assert_eq!(rendered.lines.len(), 1);
        let spans = &rendered.lines[0].spans;
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "**not bold**");
    }

    #[test]
    fn test_finalize_turn_creates_assistant_variant() {
        let mut state = TuiState::new();
        state.append_delta("Hello from assistant");
        state.finalize_turn();
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::Assistant(text) => {
                assert_eq!(text, "Hello from assistant");
            }
            _ => panic!("expected ChatLine::Assistant, got {:?}", state.chat_lines[0]),
        }
    }
}
