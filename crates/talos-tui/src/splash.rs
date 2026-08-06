//! Splash content for the first alternate-screen application frame.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::theme::semantic;

/// Left margin applied to every splash row for a consistent left-aligned layout.
const INDENT: &str = "  ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogoRenderMode {
    /// Full-width ANSI Shadow block wordmark (>= 80 cols).
    Canvas,
    /// Compact block wordmark for narrow terminals (< 80 cols).
    UnicodeBlock,
}

fn select_render_mode(width: u16) -> LogoRenderMode {
    if width >= 80 {
        LogoRenderMode::Canvas
    } else {
        LogoRenderMode::UnicodeBlock
    }
}

/// `TALOS` wordmark (ANSI Shadow figlet, 6 rows).
///
/// All rows are 42 columns wide.
fn talos_wordmark() -> &'static [&'static str] {
    &[
        "████████╗ █████╗ ██╗      ██████╗ ███████╗",
        "╚══██╔══╝██╔══██╗██║     ██╔═══██╗██╔════╝",
        "   ██║   ███████║██║     ██║   ██║███████╗",
        "   ██║   ██╔══██╗██║     ██║   ██║╚════██║",
        "   ██║   ██║  ██║███████╗╚██████╔╝███████║",
        "   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚══════╝",
    ]
}

/// Compact `TALOS` wordmark for narrow terminals (4 rows).
///
/// All rows are ~26 columns wide.
fn talos_wordmark_compact() -> &'static [&'static str] {
    &[
        " _____ _    _    ___  ___",
        "|_   _/ \\  | |  / _ \\/ __|",
        "  | || _ \\ | |_| (_) \\__ \\",
        "  |_||_/ \\_|___|\\___/|___/",
    ]
}

/// Vertical Frost gradient applied row-by-row to the wordmark.
fn wordmark_gradient(rows: usize) -> Vec<Color> {
    let ramp = &semantic::LOGO_GRADIENT;
    (0..rows)
        .map(|i| {
            if rows <= 1 {
                ramp[1]
            } else {
                let idx = i * (ramp.len() - 1) / (rows - 1);
                ramp[idx]
            }
        })
        .collect()
}

const SUBTITLE: &str = "⬡ The watchman never sleeps";

fn version_line() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// Badge labels with their accent colors and the separator glyph between them.
fn badges() -> [(Color, &'static str); 3] {
    [
        (semantic::LOGO_BADGE_1, "Precision"),
        (semantic::LOGO_BADGE_2, "Safety"),
        (semantic::LOGO_BADGE_3, "Reliability"),
    ]
}

/// Builds the one authoritative splash representation used by the first
/// alternate-screen frame. It is display-only and never enters TranscriptStore.
pub(crate) fn viewport_splash_lines(width: u16) -> Vec<Line<'static>> {
    let mode = select_render_mode(width);
    let wordmark = match mode {
        LogoRenderMode::Canvas => talos_wordmark(),
        LogoRenderMode::UnicodeBlock => talos_wordmark_compact(),
    };

    // Keep the wordmark off the terminal's top edge without coupling its
    // placement to transcript/history geometry.
    let mut lines = vec![Line::default()];
    let gradient = wordmark_gradient(wordmark.len());
    for (line, color) in wordmark.iter().zip(gradient.iter()) {
        lines.push(Line::from(vec![
            Span::raw(INDENT),
            Span::styled(
                (*line).to_string(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let wordmark_width = wordmark[0].chars().count();
    let version = version_line();
    let version_width = version.chars().count();
    let version_padding = wordmark_width.saturating_sub(version_width);
    lines.push(Line::from(Span::styled(
        format!("{INDENT}{}{version}", " ".repeat(version_padding)),
        Style::default().fg(semantic::LOGO_VERSION),
    )));
    lines.push(Line::from(vec![
        Span::raw(INDENT),
        Span::styled(
            SUBTITLE,
            Style::default()
                .fg(semantic::LOGO_SUBTITLE)
                .add_modifier(Modifier::ITALIC),
        ),
    ]));

    let mut badge_spans = vec![Span::raw(INDENT)];
    for (i, (color, label)) in badges().iter().enumerate() {
        if i > 0 {
            badge_spans.push(Span::styled(
                "  ·  ",
                Style::default().fg(semantic::LOGO_SEPARATOR),
            ));
        }
        badge_spans.push(Span::styled(
            *label,
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(badge_spans));
    lines.push(Line::default());
    lines
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn select_render_mode_canvas_at_80_cols() {
        assert_eq!(select_render_mode(80), LogoRenderMode::Canvas);
    }

    #[test]
    fn select_render_mode_canvas_at_wide_terminal() {
        assert_eq!(select_render_mode(120), LogoRenderMode::Canvas);
        assert_eq!(select_render_mode(200), LogoRenderMode::Canvas);
    }

    #[test]
    fn select_render_mode_unicode_block_at_79_cols() {
        assert_eq!(select_render_mode(79), LogoRenderMode::UnicodeBlock);
    }

    #[test]
    fn select_render_mode_unicode_block_at_narrow_terminal() {
        assert_eq!(select_render_mode(40), LogoRenderMode::UnicodeBlock);
        assert_eq!(select_render_mode(10), LogoRenderMode::UnicodeBlock);
    }

    #[test]
    fn select_render_mode_boundary_exact() {
        assert_eq!(select_render_mode(79), LogoRenderMode::UnicodeBlock);
        assert_eq!(select_render_mode(80), LogoRenderMode::Canvas);
    }

    #[test]
    fn full_wordmark_rows_are_equal_width() {
        let rows = talos_wordmark();
        let width = rows[0].chars().count();
        assert_eq!(width, 42, "ANSI Shadow TALOS rows should be 42 columns");
        for row in rows {
            assert_eq!(
                row.chars().count(),
                width,
                "wordmark row '{row}' is misaligned"
            );
        }
    }

    #[test]
    fn full_wordmark_fits_in_eighty_columns() {
        let width = talos_wordmark()[0].chars().count() + INDENT.len();
        assert!(width <= 80, "wide wordmark width {width} exceeds 80 cols");
    }

    #[test]
    fn compact_wordmark_fits_narrow_terminal() {
        let max = talos_wordmark_compact()
            .iter()
            .map(|r| r.chars().count())
            .max()
            .unwrap_or(0)
            + INDENT.len();
        assert!(
            max < 80,
            "compact wordmark width {max} should fit < 80 cols"
        );
    }

    #[test]
    fn full_wordmark_has_six_rows() {
        assert_eq!(talos_wordmark().len(), 6);
    }

    #[test]
    fn compact_wordmark_has_four_rows() {
        assert_eq!(talos_wordmark_compact().len(), 4);
    }

    #[test]
    fn wordmark_uses_block_or_box_characters() {
        let joined: String = talos_wordmark().concat();
        assert!(joined.contains('\u{2588}'), "wordmark should use █ blocks");
    }

    #[test]
    fn gradient_runs_dark_to_light_frost() {
        let g = wordmark_gradient(6);
        assert_eq!(g.len(), 6);
        let first = g.first().copied().expect("operation should succeed");
        let last = g.last().copied().expect("operation should succeed");
        let lum = |c: Color| match c {
            Color::Rgb(r, gc, b) => r as u32 + gc as u32 + b as u32,
            _ => 0,
        };
        assert!(
            lum(last) > lum(first),
            "gradient should brighten from dark Frost to light Frost"
        );
    }

    #[test]
    fn gradient_handles_single_row() {
        assert_eq!(wordmark_gradient(1).len(), 1);
    }

    #[test]
    fn gradient_matches_row_count() {
        assert_eq!(wordmark_gradient(talos_wordmark().len()).len(), 6);
        assert_eq!(wordmark_gradient(talos_wordmark_compact().len()).len(), 4);
    }

    #[test]
    fn splash_does_not_use_reserved_todo_symbols() {
        let mut all = String::new();
        all.push_str(&talos_wordmark().concat());
        all.push_str(&talos_wordmark_compact().concat());
        all.push_str(SUBTITLE);
        for (_, label) in badges() {
            all.push_str(label);
        }
        assert!(!all.contains('\u{25cb}'), "splash must not use ○ (todo)");
        assert!(!all.contains('\u{25c9}'), "splash must not use ◉ (todo)");
    }

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn alternate_screen_splash_lines_contain_wordmark_and_subtitle() {
        let rendered = viewport_splash_lines(80)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("████████"));
        assert!(rendered.contains(SUBTITLE));
    }

    #[test]
    fn alternate_screen_splash_leaves_one_blank_row_above_logo() {
        let rendered = viewport_splash_lines(80);

        assert_eq!(rendered.first().map(line_text).as_deref(), Some(""));
        assert!(
            rendered
                .get(1)
                .map(line_text)
                .as_deref()
                .is_some_and(|line| line.contains("████████")),
            "the wide wordmark should start immediately after the top spacer"
        );
    }

    #[test]
    fn alternate_screen_splash_uses_compact_wordmark_when_narrow() {
        let rendered = viewport_splash_lines(40)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("_____"));
        assert!(!rendered.contains("████████"));
    }
}
