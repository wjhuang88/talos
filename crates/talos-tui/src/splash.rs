//! Splash screen module — styled ANSI scrollback output.
//!
//! `print_splash_scrollback()` prints the styled splash to stdout BEFORE raw mode.
//! All rendering respects the inline-by-default model (ADR-018) — no alt-screen.
//! Per ADR-019, the splash is scrollback-only; no viewport overlay is provided.

use std::io::{self, Write};

use crossterm::{
    cursor::MoveToColumn,
    execute,
    style::{Attribute, Color as CColor, Print, SetAttribute, SetForegroundColor},
    terminal,
};
use ratatui::style::Color;

use crate::theme::{semantic, to_crossterm_color};

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

/// Phase 1: print the styled splash to stdout before raw mode is enabled.
///
/// This is a best-effort cosmetic output; failures are intentionally ignored
/// because the splash is purely decorative and should not block TUI startup.
pub fn print_splash_scrollback() {
    let width = terminal::size().map(|(w, _)| w).unwrap_or(80);
    let mut stdout = io::stdout();
    let _ = write_splash_scrollback(&mut stdout, width);
    let _ = stdout.flush();
}

fn write_splash_scrollback<W: Write>(writer: &mut W, width: u16) -> io::Result<()> {
    let mode = select_render_mode(width);

    let wordmark = match mode {
        LogoRenderMode::Canvas => talos_wordmark(),
        LogoRenderMode::UnicodeBlock => talos_wordmark_compact(),
    };

    let gradient = wordmark_gradient(wordmark.len());
    for (line, color) in wordmark.iter().zip(gradient.iter()) {
        print_styled_line(writer, line, *color, &[Attribute::Bold])?;
    }

    let wordmark_width = wordmark[0].chars().count();
    print_right_aligned_version(writer, wordmark_width)?;

    print_styled_line(
        writer,
        SUBTITLE,
        semantic::LOGO_SUBTITLE,
        &[Attribute::Italic],
    )?;

    print_badge_line(writer)?;

    execute!(writer, Print("\r\n\r\n"))
}

fn print_right_aligned_version<W: Write>(writer: &mut W, wordmark_width: usize) -> io::Result<()> {
    let version = version_line();
    let version_width = version.chars().count();
    let version_col = (INDENT.len() + wordmark_width - version_width) as u16;

    execute!(
        writer,
        Print("\r\n"),
        MoveToColumn(version_col),
        SetForegroundColor(to_crossterm_color(semantic::LOGO_VERSION).unwrap_or(CColor::Reset)),
        Print(version),
        SetForegroundColor(CColor::Reset)
    )
}

fn print_styled_line<W: Write>(
    writer: &mut W,
    text: &str,
    color: Color,
    attrs: &[Attribute],
) -> io::Result<()> {
    execute!(
        writer,
        Print("\r\n"),
        MoveToColumn(0),
        Print(INDENT),
        SetForegroundColor(to_crossterm_color(color).unwrap_or(CColor::Reset))
    )?;
    for attr in attrs {
        execute!(writer, SetAttribute(*attr))?;
    }

    execute!(writer, Print(text))?;

    execute!(
        writer,
        SetAttribute(Attribute::NormalIntensity),
        SetAttribute(Attribute::NoItalic),
        SetForegroundColor(CColor::Reset)
    )
}

fn print_badge_line<W: Write>(writer: &mut W) -> io::Result<()> {
    execute!(writer, Print("\r\n"), MoveToColumn(0), Print(INDENT))?;

    for (i, (color, label)) in badges().iter().enumerate() {
        if i > 0 {
            execute!(
                writer,
                SetForegroundColor(
                    to_crossterm_color(semantic::LOGO_SEPARATOR).unwrap_or(CColor::Reset)
                )
            )?;
            execute!(writer, Print("  ·  "))?;
        }
        execute!(
            writer,
            SetForegroundColor(to_crossterm_color(*color).unwrap_or(CColor::Reset))
        )?;
        execute!(writer, SetAttribute(Attribute::Bold))?;
        execute!(writer, Print(*label))?;
        execute!(
            writer,
            SetAttribute(Attribute::NormalIntensity),
            SetForegroundColor(CColor::Reset)
        )?;
    }

    Ok(())
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
        let first = g.first().copied().unwrap();
        let last = g.last().copied().unwrap();
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

    #[test]
    fn rendered_splash_starts_with_crlf() {
        let mut output = Vec::new();
        write_splash_scrollback(&mut output, 80).expect("render splash");
        let output = String::from_utf8(output).expect("utf8 splash");

        assert!(output.starts_with("\r\n"), "splash must start with CRLF");
    }

    #[test]
    fn rendered_splash_contains_wordmark_content() {
        let mut output = Vec::new();
        write_splash_scrollback(&mut output, 80).expect("render splash");
        let output = String::from_utf8(output).expect("utf8 splash");

        assert!(
            output.contains("████████"),
            "splash must contain the wordmark"
        );
        assert!(
            output.contains(SUBTITLE),
            "splash must contain the subtitle"
        );
    }
}
