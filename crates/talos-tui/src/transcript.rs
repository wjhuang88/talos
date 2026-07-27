//! Application-owned logical transcript facts.
//!
//! These entries intentionally carry no terminal geometry. Width-dependent rows
//! are created by `history_projection` for a single frame only.

use talos_conversation::{ToolCallDisplay, ToolResultDisplay};

use crate::app_stream::ScrollbackLine;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TranscriptEntryId(u64);

/// Geometry-free transcript fact. Wrapping is deliberately deferred to the
/// current-frame history projection.
#[derive(Clone, Debug)]
pub(crate) enum TranscriptBlock {
    StyledLine(ScrollbackLine),
    ToolCall(ToolCallDisplay),
    ToolResult(ToolResultDisplay),
}

#[derive(Clone, Debug)]
pub(crate) struct TranscriptEntry {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) id: TranscriptEntryId,
    pub(crate) block: TranscriptBlock,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TranscriptStore {
    entries: Vec<TranscriptEntry>,
    next_id: u64,
}

impl TranscriptStore {
    pub(crate) fn append(&mut self, block: TranscriptBlock) -> TranscriptEntryId {
        let id = TranscriptEntryId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.entries.push(TranscriptEntry { id, block });
        id
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
    use talos_core::tool::ToolProvenance;

    #[test]
    fn append_plain_line_preserves_order() {
        let mut store = TranscriptStore::default();
        store.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "first", None,
        )));
        store.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "second", None,
        )));
        assert!(
            matches!(&store.entries()[0].block, TranscriptBlock::StyledLine(line) if line.text == "first")
        );
        assert!(
            matches!(&store.entries()[1].block, TranscriptBlock::StyledLine(line) if line.text == "second")
        );
    }

    #[test]
    fn append_styled_line_preserves_segments_and_attrs() {
        let mut store = TranscriptStore::default();
        let attrs = HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        };
        store.append(TranscriptBlock::StyledLine(ScrollbackLine::styled(
            vec![HistorySegment::styled("styled", Some(Color::Blue), attrs)],
            Some(Color::Black),
        )));
        let TranscriptBlock::StyledLine(line) = &store.entries()[0].block else {
            panic!("styled line")
        };
        assert_eq!(line.segments[0].attrs, attrs);
        assert_eq!(line.segments[0].fg, Some(Color::Blue));
        assert_eq!(line.bg, Some(Color::Black));
    }

    #[test]
    fn blank_line_is_a_first_class_transcript_fact() {
        let mut store = TranscriptStore::default();
        store.append(TranscriptBlock::StyledLine(ScrollbackLine::plain("", None)));
        assert_eq!(store.entries().len(), 1);
        assert!(
            matches!(&store.entries()[0].block, TranscriptBlock::StyledLine(line) if line.text.is_empty())
        );
    }

    #[test]
    fn tool_call_transcript_is_independent_of_current_viewport_width() {
        let display = ToolCallDisplay {
            tool_name: "bash".to_string(),
            arguments: serde_json::json!({"command": "echo 你好 -- very long argument"}),
            provenance: ToolProvenance::Native,
            summary_fields: vec!["command".to_string()],
        };
        let mut narrow = TranscriptStore::default();
        let mut wide = TranscriptStore::default();
        narrow.append(TranscriptBlock::ToolCall(display.clone()));
        wide.append(TranscriptBlock::ToolCall(display));
        let TranscriptBlock::ToolCall(narrow) = &narrow.entries()[0].block else {
            panic!("tool call")
        };
        let TranscriptBlock::ToolCall(wide) = &wide.entries()[0].block else {
            panic!("tool call")
        };
        assert_eq!(narrow.tool_name, wide.tool_name);
        assert_eq!(narrow.arguments, wide.arguments);
        assert_eq!(narrow.summary_fields, wide.summary_fields);
    }

    #[test]
    fn tool_result_transcript_is_independent_of_current_viewport_width() {
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            is_error: false,
            content: "你好\nlong result line".to_string(),
        };
        let mut store = TranscriptStore::default();
        store.append(TranscriptBlock::ToolResult(display.clone()));
        let TranscriptBlock::ToolResult(stored) = &store.entries()[0].block else {
            panic!("tool result")
        };
        assert_eq!(stored.content, display.content);
        assert_eq!(stored.tool_name, display.tool_name);
    }
}
