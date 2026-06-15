//! Skill sidebar widget.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::theme::semantic;
use crate::widgets::truncate;

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
    pub(crate) fn is_collapsed(&self) -> bool {
        self.width < Self::COLLAPSE_THRESHOLD
    }

    /// Renders the skill sidebar on the given frame area.
    ///
    /// When visible and not collapsed, shows a bordered panel with:
    /// - Title "Skills"
    /// - List of skills with name (nord8), description (nord4), and status indicator
    ///   (◆ active in nord14, ◇ inactive in nord3)
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
                .fg(semantic::DIM_TEXT)
                .add_modifier(Modifier::DIM);
            lines.push(Line::from(Span::styled("No skills loaded", empty_style)));
        } else {
            for skill in &self.skills {
                let status_icon = if skill.active { "◆" } else { "◇" };
                let status_style = if skill.active {
                    Style::default().fg(semantic::TEXT_SUCCESS)
                } else {
                    Style::default().fg(semantic::DIM_TEXT)
                };

                let name_style = Style::default()
                    .fg(semantic::TEXT_ACCENT)
                    .add_modifier(Modifier::BOLD);
                let desc_style = Style::default()
                    .fg(semantic::TEXT_PRIMARY)
                    .add_modifier(Modifier::DIM);

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
                    .border_style(Style::default().fg(semantic::BORDER_DEFAULT))
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
            Style::default().fg(semantic::TEXT_SUCCESS)
        } else {
            Style::default().fg(semantic::DIM_TEXT)
        };

        let paragraph = Paragraph::new(Text::from(Span::styled(text, style))).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(semantic::BORDER_DEFAULT)),
        );

        frame.render_widget(paragraph, area);
    }
}

impl Default for SkillSidebar {
    fn default() -> Self {
        Self::new()
    }
}
