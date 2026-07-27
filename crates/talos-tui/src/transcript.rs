//! Application-owned logical transcript facts.
//!
//! These entries intentionally carry no terminal geometry. Width-dependent rows
//! are created by `history_projection` for a single frame only.

use crate::app_stream::ScrollbackLine;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TranscriptEntryId(u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TranscriptEntry {
    pub(crate) id: TranscriptEntryId,
    pub(crate) line: ScrollbackLine,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TranscriptStore {
    entries: Vec<TranscriptEntry>,
    next_id: u64,
}

impl TranscriptStore {
    pub(crate) fn append(&mut self, line: ScrollbackLine) -> TranscriptEntryId {
        let id = TranscriptEntryId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push(TranscriptEntry { id, line });
        id
    }

    pub(crate) fn extend(&mut self, lines: impl IntoIterator<Item = ScrollbackLine>) {
        for line in lines {
            self.append(line);
        }
    }

    pub(crate) fn entries(&self) -> &[TranscriptEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;

    use super::*;
    use crate::inline_terminal::{HistoryAttrs, HistorySegment};

    #[test]
    fn append_plain_line_preserves_order() {
        let mut store = TranscriptStore::default();
        store.append(ScrollbackLine::plain("first", None));
        store.append(ScrollbackLine::plain("second", None));
        assert_eq!(store.entries()[0].line.text, "first");
        assert_eq!(store.entries()[1].line.text, "second");
    }

    #[test]
    fn append_styled_line_preserves_segments_and_attrs() {
        let mut store = TranscriptStore::default();
        let attrs = HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        };
        store.append(ScrollbackLine::styled(
            vec![HistorySegment::styled("styled", Some(Color::Blue), attrs)],
            Some(Color::Black),
        ));
        assert_eq!(store.entries()[0].line.segments[0].attrs, attrs);
        assert_eq!(store.entries()[0].line.segments[0].fg, Some(Color::Blue));
        assert_eq!(store.entries()[0].line.bg, Some(Color::Black));
    }

    #[test]
    fn blank_line_is_a_first_class_transcript_fact() {
        let mut store = TranscriptStore::default();
        store.append(ScrollbackLine::plain("", None));
        assert_eq!(store.entries().len(), 1);
        assert!(store.entries()[0].line.text.is_empty());
    }
}
