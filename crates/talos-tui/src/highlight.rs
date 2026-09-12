//! Syntax highlighting engine using arborium (tree-sitter grammar bundle).

use crossterm::style::Color as CColor;

use crate::theme::to_crossterm_color;
use talos_text::HighlightProvider;

type LineSegments = Vec<(String, Option<CColor>)>;

pub(crate) struct HighlightEngine {
    highlighter: Box<dyn HighlightProvider>,
}

impl Default for HighlightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightEngine {
    pub(crate) fn new() -> Self {
        Self {
            highlighter: Box::new(talos_text::BuiltinHighlighter::default()),
        }
    }

    /// Construct an engine with an explicitly selected highlighting provider.
    pub(crate) fn with_provider(provider: Box<dyn HighlightProvider>) -> Self {
        Self {
            highlighter: provider,
        }
    }

    /// Highlight code and return per-line segments with crossterm colors.
    ///
    /// Each inner `Vec` represents one line; each element is `(text, color)`
    /// where color is `None` for the default text color.
    /// Returns `None` if parsing fails or exceeds 500ms.
    pub(crate) fn highlight(&mut self, language: &str, code: &str) -> Option<Vec<LineSegments>> {
        // Preserve the previous Arborium boundary: only canonical grammar keys are
        // highlighted. Alias normalization remains available to neutral consumers and tools,
        // but must not silently change TUI fallback behavior.
        if !self.supports(language) {
            return None;
        }
        let language = talos_text::LanguageId::parse(language)?;
        let result = self.highlighter.highlight(&language, code);
        match result {
            talos_text::HighlightResult::PlainText => None,
            _ => Some(segments_from_result(code, &result)),
        }
    }

    pub(crate) fn supports(&self, language: &str) -> bool {
        talos_text::LanguageId::parse(language)
            .is_some_and(|id| id.as_str() == language && self.highlighter.supports(&id))
    }
}

/// Convert raw tree-sitter spans into per-line colored text segments.
fn segments_from_result(code: &str, result: &talos_text::HighlightResult) -> Vec<LineSegments> {
    let spans = result.validated_spans(code).unwrap_or(&[]);
    let line_offsets: Vec<usize> = code
        .match_indices('\n')
        .map(|(i, _)| i + 1)
        .chain(std::iter::once(code.len()))
        .collect();

    let mut lines: Vec<LineSegments> = vec![Vec::new(); line_offsets.len()];
    let mut line_idx: usize = 0;
    let mut cursor: usize = 0;

    for span in spans {
        let s = span.start;
        let e = span.end;

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
        n if n.starts_with("comment") => crate::theme::nord::NORD4,
        n if n.starts_with("operator") || n.starts_with("punctuation") => crate::theme::nord::NORD4,
        n if n.starts_with("variable") => crate::theme::nord::NORD5,
        n if n.starts_with("property") || n.starts_with("field") => crate::theme::nord::NORD7,
        n if n.starts_with("attribute") || n.starts_with("tag") => crate::theme::nord::NORD9,
        n if n.starts_with("boolean") => crate::theme::nord::NORD15,
        _ => return None,
    };
    to_crossterm_color(nord_color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_text::{HighlightResult, HighlightSpan};

    struct RecordingProvider {
        calls: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
        result: HighlightResult,
    }

    impl HighlightProvider for RecordingProvider {
        fn supports(&self, language: &talos_text::LanguageId) -> bool {
            self.calls
                .borrow_mut()
                .push(format!("supports:{}", language.as_str()));
            true
        }

        fn highlight(
            &mut self,
            language: &talos_text::LanguageId,
            source: &str,
        ) -> HighlightResult {
            self.calls
                .borrow_mut()
                .push(format!("highlight:{}:{source}", language.as_str()));
            std::mem::replace(&mut self.result, HighlightResult::PlainText)
        }
    }

    #[test]
    fn injected_provider_drives_rendering_and_plain_fallback() {
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut engine = HighlightEngine::with_provider(Box::new(RecordingProvider {
            calls: calls.clone(),
            result: HighlightResult::Spans(vec![HighlightSpan {
                start: 0,
                end: 1,
                capture: "keyword".into(),
            }]),
        }));
        assert_eq!(
            engine.highlight("rust", "x"),
            Some(vec![vec![("x".into(), capture_color("keyword"))]])
        );
        assert!(engine.highlight("rust", "fallback").is_none());
        assert_eq!(
            *calls.borrow(),
            [
                "supports:rust",
                "highlight:rust:x",
                "supports:rust",
                "highlight:rust:fallback"
            ]
        );
    }

    #[test]
    fn unavailable_grammar_keeps_existing_markdown_fallback() {
        let mut engine = HighlightEngine::new();
        assert!(!engine.supports("not-a-supported-language"));
        assert!(
            engine
                .highlight("not-a-supported-language", "一\nsecond\n")
                .is_none()
        );
        assert!(engine.highlight("rust", "fn main() {}\n").is_some());
    }

    #[test]
    fn aliases_keep_the_previous_plain_fallback_boundary() {
        let mut engine = HighlightEngine::new();
        for language in ["rs", "py", "TSX", "tsx", ".rs", "RUST", " rust "] {
            assert!(!engine.supports(language), "{language}");
            assert!(engine.highlight(language, "fn main() {}\n").is_none());
        }
        for language in ["rust", "python", "typescript"] {
            assert!(engine.supports(language), "{language}");
        }
    }

    #[test]
    fn plain_fallback_preserves_line_structure() {
        assert_eq!(
            segments_from_result("一\nsecond\n", &HighlightResult::PlainText),
            vec![
                vec![("一".into(), None)],
                vec![("second".into(), None)],
                vec![]
            ]
        );
    }

    #[test]
    fn malformed_neutral_span_cannot_split_utf8() {
        let result = HighlightResult::Spans(vec![HighlightSpan {
            start: 1,
            end: 2,
            capture: "keyword".into(),
        }]);
        assert_eq!(
            segments_from_result("一", &result),
            vec![vec![("一".into(), None)]]
        );
    }

    #[test]
    fn later_bad_span_discards_earlier_coloring() {
        for bad in [(2, 3), (0, 1), (4, 99)] {
            let result = HighlightResult::Spans(vec![
                HighlightSpan {
                    start: 0,
                    end: 1,
                    capture: "keyword".into(),
                },
                HighlightSpan {
                    start: bad.0,
                    end: bad.1,
                    capture: "string".into(),
                },
            ]);
            assert_eq!(
                segments_from_result("a界z", &result),
                vec![vec![("a界z".into(), None)]]
            );
        }
    }
}
