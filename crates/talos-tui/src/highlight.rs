//! Syntax highlighting engine using arborium (tree-sitter grammar bundle).

use std::time::Instant;

use crossterm::style::Color as CColor;

use crate::theme::to_crossterm_color;

type LineSegments = Vec<(String, Option<CColor>)>;

pub(crate) struct HighlightEngine {
    highlighter: arborium::Highlighter,
}

impl Default for HighlightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightEngine {
    pub(crate) fn new() -> Self {
        Self {
            highlighter: arborium::Highlighter::new(),
        }
    }

    /// Highlight code and return per-line segments with crossterm colors.
    ///
    /// Each inner `Vec` represents one line; each element is `(text, color)`
    /// where color is `None` for the default text color.
    /// Returns `None` if parsing fails or exceeds 500ms.
    pub(crate) fn highlight(&mut self, language: &str, code: &str) -> Option<Vec<LineSegments>> {
        let start = Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.highlighter.highlight_spans(language, code)
        }));

        let spans = match result {
            Ok(Ok(spans)) => spans,
            _ => return None,
        };

        if start.elapsed().as_millis() > 500 {
            return None;
        }

        Some(segments_from_spans(code, &spans))
    }

    pub(crate) fn supports(&self, language: &str) -> bool {
        arborium::get_language(language).is_some()
    }
}

/// Convert raw tree-sitter spans into per-line colored text segments.
fn segments_from_spans(code: &str, spans: &[arborium::advanced::Span]) -> Vec<LineSegments> {
    let line_offsets: Vec<usize> = code
        .match_indices('\n')
        .map(|(i, _)| i + 1)
        .chain(std::iter::once(code.len()))
        .collect();

    let mut lines: Vec<LineSegments> = vec![Vec::new(); line_offsets.len()];
    let mut line_idx: usize = 0;
    let mut cursor: usize = 0;

    for span in spans {
        let s = span.start.min(code.len() as u32) as usize;
        let e = span.end.min(code.len() as u32) as usize;

        if s < cursor || e < s {
            continue;
        }

        if s > cursor {
            emit_plain_segment(code, &mut lines, &mut line_idx, cursor, s);
        }

        if e > s {
            let color = capture_color(&span.capture);
            emit_colored_segment(code, &mut lines, &mut line_idx, s, e, color);
        }

        cursor = e;
    }

    if cursor < code.len() {
        emit_plain_segment(code, &mut lines, &mut line_idx, cursor, code.len());
    }

    lines
}

fn emit_plain_segment(
    code: &str,
    lines: &mut [LineSegments],
    line_idx: &mut usize,
    start: usize,
    end: usize,
) {
    let text = &code[start..end];
    for part in text.split_inclusive('\n') {
        let stripped = part.strip_suffix('\n').unwrap_or(part);
        if !stripped.is_empty() {
            lines[*line_idx].push((stripped.to_string(), None));
        }
        if part.ends_with('\n') {
            *line_idx += 1;
        }
    }
}

fn emit_colored_segment(
    code: &str,
    lines: &mut [LineSegments],
    line_idx: &mut usize,
    start: usize,
    end: usize,
    color: Option<CColor>,
) {
    let text = &code[start..end];
    for part in text.split_inclusive('\n') {
        let stripped = part.strip_suffix('\n').unwrap_or(part);
        if !stripped.is_empty() {
            lines[*line_idx].push((stripped.to_string(), color));
        }
        if part.ends_with('\n') {
            *line_idx += 1;
        }
    }
}

/// Map tree-sitter capture name to Nord theme crossterm color.
fn capture_color(capture: &str) -> Option<CColor> {
    let nord_color = match capture {
        n if n.starts_with("keyword") => crate::theme::nord::NORD9,
        n if n.starts_with("type") || n.starts_with("constructor") => crate::theme::nord::NORD7,
        n if n.starts_with("function") || n.starts_with("method") => crate::theme::nord::NORD8,
        n if n.starts_with("string") => crate::theme::nord::NORD14,
        n if n.starts_with("number") || n.starts_with("constant") => crate::theme::nord::NORD15,
        n if n.starts_with("comment") => crate::theme::nord::NORD3,
        n if n.starts_with("operator") || n.starts_with("punctuation") => crate::theme::nord::NORD4,
        n if n.starts_with("variable") => crate::theme::nord::NORD5,
        n if n.starts_with("property") || n.starts_with("field") => crate::theme::nord::NORD7,
        n if n.starts_with("attribute") || n.starts_with("tag") => crate::theme::nord::NORD9,
        n if n.starts_with("boolean") => crate::theme::nord::NORD15,
        _ => return None,
    };
    to_crossterm_color(nord_color)
}
