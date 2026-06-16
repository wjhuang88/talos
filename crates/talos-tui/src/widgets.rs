//! TUI widgets and render helpers.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use talos_core::ApprovalChoice;
use talos_core::tool::ToolProvenance;

use crate::theme::semantic;

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
    pub(crate) tool_name: &'a str,
    /// Formatted arguments (may be truncated).
    pub(crate) arguments: &'a str,
    /// Whether the tool result was an error.
    pub(crate) result_status: Option<bool>,
    /// Result content (may be truncated).
    pub(crate) result_content: Option<&'a str>,
    /// Origin of the tool.
    pub(crate) provenance: ToolProvenance,
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

impl ratatui::widgets::Widget for ToolCallBubble<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let prefix_style = Style::default()
            .fg(semantic::PREFIX_ASSISTANT)
            .add_modifier(Modifier::BOLD);
        let tool_name_style = Style::default()
            .fg(semantic::TEXT_ACCENT)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(semantic::DIM_TEXT);
        let args_display = truncate(self.arguments, MAX_ARGS_LENGTH);

        let mut spans = vec![
            Span::styled(" ▸ ", prefix_style),
            Span::styled(self.tool_name.to_string(), tool_name_style),
        ];
        if let ToolProvenance::McpRemote { server } = &self.provenance {
            let server_display = truncate(server, 24);
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!("[mcp:{}]", server_display), dim_style));
        }
        spans.push(Span::raw(", "));
        spans.push(Span::styled(args_display, dim_style));
        lines.push(Line::from(spans));

        if let Some(is_error) = self.result_status {
            let (icon, style) = if is_error {
                ("✗ error", Style::default().fg(semantic::TEXT_ERROR))
            } else {
                ("✓ success", Style::default().fg(semantic::TEXT_SUCCESS))
            };
            lines.push(Line::from(Span::styled(format!("  {icon}"), style)));

            if let Some(content) = self.result_content {
                if let Some(diff_lines) = render_diff(content) {
                    lines.extend(diff_lines);
                } else {
                    let content_style = Style::default().fg(semantic::TEXT_PRIMARY);
                    let content_display = truncate(content, MAX_RESULT_LENGTH);
                    lines.push(Line::from(Span::styled(
                        format!("  {content_display}"),
                        content_style,
                    )));
                }
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(semantic::BORDER_DEFAULT)),
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
            .fg(semantic::TEXT_SECONDARY_ACCENT)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            "⚠ Permission Required",
            title_style,
        )));
        lines.push(Line::from(""));

        let tool_style = Style::default()
            .fg(semantic::TEXT_ACCENT)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            format!("Tool: {}", self.tool_name),
            tool_style,
        )));

        let args_style = Style::default()
            .fg(semantic::DIM_TEXT)
            .add_modifier(Modifier::DIM);
        let args_display = truncate(self.arguments, MAX_ARGS_LENGTH);
        lines.push(Line::from(Span::styled(
            format!("Args: {args_display}"),
            args_style,
        )));
        lines.push(Line::from(""));

        let risk_style = Style::default().fg(semantic::TEXT_WARNING);
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
                    .fg(semantic::TEXT_ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(semantic::TEXT_PRIMARY)
            };
            lines.push(Line::from(Span::styled(format!("[{key}] {label}"), style)));
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(semantic::BORDER_ACCENT))
                .title(" Approval "),
        );

        paragraph.render(overlay_area, buf);
    }
}

/// Truncates a string to the given maximum length, appending "…" if truncated.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

/// Detects unified diff content and renders it with color-coded lines.
///
/// Returns `Some` when the content appears to be a unified diff, `None` otherwise.
/// Detection heuristic: content contains lines starting with `diff --git`, `@@`,
/// or starts with `--- a/` / `+++ b/`.
pub(crate) fn render_diff(content: &str) -> Option<Vec<Line<'static>>> {
    let mut is_diff = false;
    for line in content.lines() {
        if line.starts_with("diff --git")
            || line.starts_with("@@")
            || line.starts_with("--- a/")
            || line.starts_with("+++ b/")
        {
            is_diff = true;
            break;
        }
    }
    if !is_diff {
        return None;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();
    for line in content.lines() {
        let styled_line = if line.starts_with("diff --git")
            || line.starts_with("--- a/")
            || line.starts_with("+++ b/")
        {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_SECONDARY_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ))
        } else if line.starts_with("@@") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(semantic::TEXT_ACCENT),
            ))
        } else if line.starts_with('+') && !line.starts_with("+++") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(semantic::TEXT_SUCCESS),
            ))
        } else if line.starts_with('-') && !line.starts_with("---") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(semantic::TEXT_ERROR),
            ))
        } else {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_PRIMARY)
                    .add_modifier(Modifier::DIM),
            ))
        };
        lines.push(styled_line);
    }

    Some(lines)
}
