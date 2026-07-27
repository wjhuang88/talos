//! Width-dependent projection of application-owned transcript facts.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app_stream::ScrollbackLine;
use crate::inline_terminal::HistorySegment;
use crate::transcript::{TranscriptEntryId, TranscriptStore};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoryScrollState {
    pub(crate) follow_tail: bool,
    pub(crate) bottom_offset_rows: usize,
}

impl HistoryScrollState {
    pub(crate) const fn follow_tail() -> Self {
        Self {
            follow_tail: true,
            bottom_offset_rows: 0,
        }
    }

    pub(crate) fn page_up(&mut self, rows: usize) {
        self.follow_tail = false;
        self.bottom_offset_rows = self.bottom_offset_rows.saturating_add(rows);
    }

    pub(crate) fn page_down(&mut self, rows: usize) {
        self.bottom_offset_rows = self.bottom_offset_rows.saturating_sub(rows);
        self.follow_tail = self.bottom_offset_rows == 0;
    }

    pub(crate) fn jump_to_start(&mut self, total_rows: usize) {
        self.follow_tail = false;
        self.bottom_offset_rows = total_rows;
    }

    pub(crate) fn jump_to_end(&mut self) {
        *self = Self::follow_tail();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderedHistoryRow {
    pub(crate) entry_id: TranscriptEntryId,
    pub(crate) line: ScrollbackLine,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoryProjection {
    pub(crate) rows: Vec<RenderedHistoryRow>,
    pub(crate) total_rows: usize,
}

/// Projects logical transcript lines without changing their stored content.
pub(crate) fn project_history(
    transcript: &TranscriptStore,
    width: u16,
    height: u16,
    _scroll: &HistoryScrollState,
) -> HistoryProjection {
    if width == 0 || height == 0 {
        return HistoryProjection::default();
    }

    let mut all = Vec::new();
    for entry in transcript.entries() {
        for line in project_line(&entry.line, width) {
            all.push(RenderedHistoryRow {
                entry_id: entry.id,
                line,
            });
        }
    }
    let total_rows = all.len();
    let end = total_rows.saturating_sub(_scroll.bottom_offset_rows);
    let start = end.saturating_sub(height as usize);
    HistoryProjection {
        rows: all[start..end].to_vec(),
        total_rows,
    }
}

fn project_line(line: &ScrollbackLine, width: u16) -> Vec<ScrollbackLine> {
    if line.text.is_empty() {
        return vec![line.clone()];
    }
    let capacity = usize::from(width);
    let mut rows = Vec::new();
    let mut current = Vec::<HistorySegment>::new();
    let mut used = 0usize;

    for segment in &line.segments {
        for ch in segment.text.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            // A double-width scalar cannot be represented at width one. This is
            // projection-only; the original scalar remains in TranscriptStore.
            if char_width > capacity {
                if !current.is_empty() {
                    rows.push(ScrollbackLine::styled(
                        std::mem::take(&mut current),
                        line.bg,
                    ));
                    used = 0;
                }
                rows.push(ScrollbackLine::plain("…", line.bg));
                continue;
            }
            if used > 0 && used.saturating_add(char_width) > capacity {
                rows.push(ScrollbackLine::styled(
                    std::mem::take(&mut current),
                    line.bg,
                ));
                used = 0;
            }
            if let Some(last) = current.last_mut()
                && last.fg == segment.fg
                && last.attrs == segment.attrs
            {
                last.text.push(ch);
            } else {
                current.push(HistorySegment::styled(
                    ch.to_string(),
                    segment.fg,
                    segment.attrs,
                ));
            }
            used = used.saturating_add(char_width);
        }
    }
    if !current.is_empty() {
        rows.push(ScrollbackLine::styled(current, line.bg));
    }
    if rows.is_empty() {
        return vec![line.clone()];
    }
    if let Some(fill) = &line.fill
        && let Some(last) = rows.last_mut()
    {
        let used = UnicodeWidthStr::width(last.text.as_str());
        let fill_width = UnicodeWidthStr::width(fill.text.as_str()).max(1);
        let repeat = capacity.saturating_sub(used).div_ceil(fill_width);
        if repeat > 0 {
            let mut fill = fill.clone();
            fill.text = fill.text.repeat(repeat);
            last.segments.push(fill);
            last.text = last
                .segments
                .iter()
                .map(|segment| segment.text.as_str())
                .collect();
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_resize_projection_is_reversible() {
        let mut transcript = TranscriptStore::default();
        transcript.append(ScrollbackLine::plain("abc你好xyz", None));
        transcript.append(ScrollbackLine::plain("", None));
        let before = transcript.clone();
        let scroll = HistoryScrollState::follow_tail();
        let initial = project_history(&transcript, 160, 100, &scroll);
        for width in [120, 80, 40, 3, 2, 1, 40, 160] {
            let projection = project_history(&transcript, width, 100, &scroll);
            assert!(projection.rows.iter().all(|row| {
                row.line
                    .text
                    .chars()
                    .all(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) <= usize::from(width))
            }));
        }
        assert_eq!(transcript, before);
        assert_eq!(project_history(&transcript, 160, 100, &scroll), initial);
    }

    #[test]
    fn width_one_degrades_without_mutating_cjk() {
        let mut transcript = TranscriptStore::default();
        transcript.append(ScrollbackLine::plain("你", None));
        let scroll = HistoryScrollState::follow_tail();
        assert_eq!(
            project_history(&transcript, 1, 1, &scroll).rows[0]
                .line
                .text,
            "…"
        );
        assert_eq!(transcript.entries()[0].line.text, "你");
        assert_eq!(
            project_history(&transcript, 2, 1, &scroll).rows[0]
                .line
                .text,
            "你"
        );
    }
}
