//! TUI widgets and render helpers.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};
use talos_core::ApprovalChoice;

use crate::theme::semantic;

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
        let args_budget = area.width.saturating_sub(8) as usize;
        let args_display = truncate(self.arguments, args_budget.max(1));
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
