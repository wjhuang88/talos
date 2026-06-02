//! Evolution insights panel for TUI.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// A learned pattern for display.
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub description: String,
    pub instruction: String,
    pub confidence: f64,
    pub evidence_count: u32,
    pub category: String,
}

/// Evolution insights panel showing learned patterns.
pub struct EvolutionPanel {
    pub visible: bool,
    pub patterns: Vec<PatternInfo>,
    pub width: u16,
}

impl EvolutionPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            patterns: Vec::new(),
            width: 40,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn update_patterns(&mut self, patterns: Vec<PatternInfo>) {
        self.patterns = patterns;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let mut lines: Vec<Line<'static>> = Vec::new();

        if self.patterns.is_empty() {
            lines.push(Line::from(Span::styled(
                "No patterns learned yet",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, pattern) in self.patterns.iter().enumerate() {
                let confidence_pct = (pattern.confidence * 100.0) as u32;
                let header = format!(
                    "{}. [{}] ({}%, {} evidence)",
                    i + 1,
                    pattern.category,
                    confidence_pct,
                    pattern.evidence_count
                );

                lines.push(Line::from(Span::styled(
                    header,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));

                lines.push(Line::from(Span::raw(format!("   {}", pattern.instruction))));

                if i < self.patterns.len() - 1 {
                    lines.push(Line::from(""));
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Evolution Insights (Ctrl+E to toggle) ")
                    .style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

impl Default for EvolutionPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_panel_new() {
        let panel = EvolutionPanel::new();
        assert!(!panel.visible);
        assert!(panel.patterns.is_empty());
    }

    #[test]
    fn test_toggle() {
        let mut panel = EvolutionPanel::new();
        assert!(!panel.visible);

        panel.toggle();
        assert!(panel.visible);

        panel.toggle();
        assert!(!panel.visible);
    }

    #[test]
    fn test_update_patterns() {
        let mut panel = EvolutionPanel::new();
        let patterns = vec![PatternInfo {
            description: "Test pattern".to_string(),
            instruction: "Use functional style".to_string(),
            confidence: 0.9,
            evidence_count: 5,
            category: "preference".to_string(),
        }];

        panel.update_patterns(patterns);
        assert_eq!(panel.patterns.len(), 1);
    }
}
