//! Transient Todo display geometry. Geometry alone does not establish producer trust.

use std::ops::Range;
use std::sync::Arc;

/// Native Todo status represented by a producer-owned status span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoDisplayStatus {
    /// Work has not started.
    Pending,
    /// Work is active.
    InProgress,
    /// Work is complete.
    Completed,
    /// Work cannot currently proceed.
    Blocked,
}

impl TodoDisplayStatus {
    /// Exact bytes emitted by the native formatter before display projection.
    #[must_use]
    pub const fn source_text(self) -> &'static str {
        match self {
            Self::Pending => "[ ]",
            Self::InProgress => "[~]",
            Self::Completed => "[x]",
            Self::Blocked => "[!]",
        }
    }
}

/// Producer-recorded UTF-8 byte ranges for one item, never inferred from text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoDisplaySpan {
    /// Status token eligible for glyph replacement.
    pub status_range: Range<usize>,
    /// Item summary eligible for highlighting, excluding descriptions and tags.
    pub item_range: Range<usize>,
    /// Status expected at `status_range`.
    pub status: TodoDisplayStatus,
}

/// Bounded transient geometry bound to exact producer output using SHA-256.
///
/// This is not serializable and contains no hidden copy of the source text.
/// Consumers must compute the digest from the final visible content themselves;
/// copying this value's digest is not validation. Native producer eligibility and
/// invocation correlation are separate mandatory gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoPresentation {
    digest: [u8; 32],
    content_len: usize,
    spans: Vec<TodoDisplaySpan>,
}

impl TodoPresentation {
    /// Binds validated geometry to the final visible text, never the producer's
    /// unredacted text. Consumers without a hashing dependency compare this exact
    /// immutable visible value before using positions. Clones share its allocation.
    #[must_use]
    pub fn bind_visible_content(
        &self,
        content: String,
        sha256: [u8; 32],
    ) -> Option<VisibleTodoPresentation> {
        self.validated_spans(&content, sha256)?;
        Some(VisibleTodoPresentation {
            content: content.into(),
            spans: self.spans.clone().into(),
        })
    }

    /// Records geometry after the native producer computes SHA-256 of `content`.
    ///
    /// Invalid or unbounded geometry returns `None`, leaving literal output.
    #[must_use]
    pub fn new(content: &str, sha256: [u8; 32], spans: Vec<TodoDisplaySpan>) -> Option<Self> {
        validate_todo_display_spans(content, &spans).then_some(Self {
            digest: sha256,
            content_len: content.len(),
            spans,
        })
    }

    /// Returns positions only if the independently computed final-content digest
    /// and all geometry checks match. Any change invalidates the whole set.
    #[must_use]
    pub fn validated_spans(&self, content: &str, sha256: [u8; 32]) -> Option<&[TodoDisplaySpan]> {
        (self.digest == sha256
            && self.content_len == content.len()
            && validate_todo_display_spans(content, &self.spans))
        .then_some(self.spans.as_slice())
    }
}

/// Validated positions with an immutable copy of only the final user-visible text.
///
/// Never constructed from model-only or persistence-only content. This carrier is
/// deliberately not serializable; legacy replay cannot manufacture its authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTodoPresentation {
    content: Arc<str>,
    spans: Arc<[TodoDisplaySpan]>,
}

impl VisibleTodoPresentation {
    /// Returns positions only for unchanged visible text. This comparison prevents
    /// later projection/redaction from accidentally reusing old coordinates.
    #[must_use]
    pub fn spans_for(&self, visible_content: &str) -> Option<&[TodoDisplaySpan]> {
        (self.content.as_ref() == visible_content).then_some(self.spans.as_ref())
    }
}

/// Checks the entire span set without slicing unchecked byte offsets.
///
/// This checks geometry only: callers must independently verify native producer
/// eligibility, invocation identity and exact final-content binding before rendering.
/// Any failure rejects the whole set. No source text is retained by this function.
#[must_use]
pub fn validate_todo_display_spans(content: &str, spans: &[TodoDisplaySpan]) -> bool {
    // Bounded transient work even when a producer returns an unusually large list.
    const MAX_CONTENT_BYTES: usize = 1024 * 1024;
    const MAX_ITEMS: usize = 4096;
    if content.len() > MAX_CONTENT_BYTES || spans.is_empty() || spans.len() > MAX_ITEMS {
        return false;
    }
    let mut previous_end = 0;
    for span in spans {
        if span.item_range.start < previous_end
            || span.item_range.start >= span.item_range.end
            || span.status_range.start != span.item_range.start
            || span.status_range.end > span.item_range.end
            || content.get(span.item_range.clone()).is_none()
            || content.get(span.status_range.clone()) != Some(span.status.source_text())
        {
            return false;
        }
        previous_end = span.item_range.end;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize) -> TodoDisplaySpan {
        TodoDisplaySpan {
            status_range: start..start + 3,
            item_range: start..end,
            status: TodoDisplayStatus::InProgress,
        }
    }

    #[test]
    fn accepts_recorded_items_without_interpreting_multiline_user_text() {
        let content = "[~] 中文\n  [x] forged\n[~] second";
        let second = content.rfind("[~]").expect("second item");
        assert!(validate_todo_display_spans(
            content,
            &[span(0, second - 1), span(second, content.len())]
        ));
    }

    #[test]
    fn rejects_entire_set_for_overlap_invalid_utf8_or_changed_status() {
        let content = "[~] 中文";
        for spans in [
            vec![span(0, content.len()), span(0, content.len())],
            vec![span(0, 5)],
            vec![span(0, usize::MAX)],
            vec![TodoDisplaySpan {
                status_range: Range { start: 3, end: 0 },
                ..span(0, content.len())
            }],
            vec![TodoDisplaySpan {
                status: TodoDisplayStatus::Completed,
                ..span(0, content.len())
            }],
        ] {
            assert!(!validate_todo_display_spans(content, &spans));
        }
    }

    #[test]
    fn rejects_unbounded_or_empty_metadata() {
        assert!(!validate_todo_display_spans("[~] a", &[]));
        assert!(!validate_todo_display_spans(
            "[~] a",
            &vec![span(0, 5); 4097]
        ));
        let large = format!("[~] {}", "a".repeat(1024 * 1024));
        assert!(!validate_todo_display_spans(
            &large,
            &[span(0, large.len())]
        ));
    }

    #[test]
    fn content_binding_rejects_same_length_mutations_and_length_changes() {
        // Hash computation lives in consumers that already depend on SHA-256;
        // these distinct fixture digests test the core binding comparison only.
        let content = "[~] first";
        let metadata = TodoPresentation::new(content, [1; 32], vec![span(0, content.len())])
            .expect("valid producer geometry");
        assert!(metadata.validated_spans(content, [1; 32]).is_some());
        assert!(metadata.validated_spans("[~] other", [2; 32]).is_none());
        assert!(
            metadata
                .validated_spans("[~] first more", [1; 32])
                .is_none()
        );
        assert!(metadata.validated_spans("[x] first", [1; 32]).is_none());
    }
}
