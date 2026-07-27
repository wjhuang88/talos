//! Width-dependent projection of application-owned transcript facts.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app_stream::ScrollbackLine;
use crate::inline_terminal::HistorySegment;
use crate::transcript::{TranscriptBlock, TranscriptEntryId, TranscriptStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LogicalContentAnchor {
    pub(crate) entry_id: TranscriptEntryId,
    pub(crate) logical_line: usize,
    pub(crate) scalar_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HistoryScrollMode {
    FollowTail,
    Anchored {
        anchor: LogicalContentAnchor,
        screen_row: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HistoryScrollState {
    pub(crate) mode: HistoryScrollMode,
}

impl HistoryScrollState {
    pub(crate) const fn follow_tail() -> Self {
        Self {
            mode: HistoryScrollMode::FollowTail,
        }
    }

    pub(crate) fn anchor(&mut self, anchor: LogicalContentAnchor, screen_row: u16) {
        self.mode = HistoryScrollMode::Anchored { anchor, screen_row };
    }

    pub(crate) fn jump_to_end(&mut self) {
        *self = Self::follow_tail();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderedHistoryRow {
    pub(crate) start_anchor: LogicalContentAnchor,
    pub(crate) end_anchor: LogicalContentAnchor,
    pub(crate) is_last_row_of_logical_line: bool,
    pub(crate) is_empty_logical_line: bool,
    pub(crate) line: ScrollbackLine,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoryProjection {
    pub(crate) rows: Vec<RenderedHistoryRow>,
    pub(crate) total_rows: usize,
    pub(crate) visible_start: usize,
    all_rows: Vec<RenderedHistoryRow>,
}

impl HistoryProjection {
    pub(crate) fn first_anchor(&self) -> Option<LogicalContentAnchor> {
        self.all_rows.first().map(|row| row.start_anchor)
    }

    pub(crate) fn page_up(&self, height: u16) -> Option<LogicalContentAnchor> {
        let page = usize::from(height.saturating_sub(1).max(1));
        self.all_rows
            .get(self.visible_start.saturating_sub(page))
            .map(|row| row.start_anchor)
    }

    pub(crate) fn page_down(&self, height: u16) -> Option<LogicalContentAnchor> {
        let page = usize::from(height.saturating_sub(1).max(1));
        let max_start = self.total_rows.saturating_sub(usize::from(height));
        self.all_rows
            .get((self.visible_start + page).min(max_start))
            .map(|row| row.start_anchor)
    }

    pub(crate) fn page_down_reaches_tail(&self, height: u16) -> bool {
        let page = usize::from(height.saturating_sub(1).max(1));
        self.visible_start.saturating_add(page)
            >= self.total_rows.saturating_sub(usize::from(height))
    }

    pub(crate) fn scroll_up(&self, rows: usize) -> Option<LogicalContentAnchor> {
        self.all_rows
            .get(self.visible_start.saturating_sub(rows))
            .map(|row| row.start_anchor)
    }

    pub(crate) fn scroll_down(&self, rows: usize, height: u16) -> Option<LogicalContentAnchor> {
        let max_start = self.total_rows.saturating_sub(usize::from(height));
        self.all_rows
            .get(self.visible_start.saturating_add(rows).min(max_start))
            .map(|row| row.start_anchor)
    }

    pub(crate) fn scroll_down_reaches_tail(&self, rows: usize, height: u16) -> bool {
        self.visible_start.saturating_add(rows)
            >= self.total_rows.saturating_sub(usize::from(height))
    }
}

/// Projects logical transcript lines without changing their stored content.
pub(crate) fn project_history(
    transcript: &TranscriptStore,
    width: u16,
    height: u16,
    scroll: &HistoryScrollState,
) -> HistoryProjection {
    if width == 0 || height == 0 {
        return HistoryProjection::default();
    }

    let mut all = Vec::new();
    for entry in transcript.entries() {
        for (logical_line, line) in logical_lines(&entry.block).into_iter().enumerate() {
            let chunks = project_line_chunks(&line, width);
            let last = chunks.len().saturating_sub(1);
            all.extend(
                chunks
                    .into_iter()
                    .enumerate()
                    .map(|(index, chunk)| RenderedHistoryRow {
                        start_anchor: LogicalContentAnchor {
                            entry_id: entry.id,
                            logical_line,
                            scalar_offset: chunk.logical_start,
                        },
                        end_anchor: LogicalContentAnchor {
                            entry_id: entry.id,
                            logical_line,
                            scalar_offset: chunk.logical_end,
                        },
                        is_last_row_of_logical_line: index == last,
                        is_empty_logical_line: line.text.is_empty(),
                        line: chunk.line,
                    }),
            );
        }
    }
    let total_rows = all.len();
    let start = match scroll.mode {
        HistoryScrollMode::FollowTail => total_rows.saturating_sub(height as usize),
        HistoryScrollMode::Anchored { anchor, screen_row } => {
            // A removed/changed row falls forward to the first surviving entry;
            // if none survives, clamp at the end. This keeps the anchor logical.
            let index = all
                .iter()
                .position(|row| row_contains_anchor(row, anchor))
                .unwrap_or_else(|| {
                    all.iter()
                        .position(|row| row.start_anchor.entry_id >= anchor.entry_id)
                        .unwrap_or(total_rows)
                });
            index.saturating_sub(usize::from(screen_row))
        }
    };
    let end = (start + usize::from(height)).min(total_rows);
    HistoryProjection {
        rows: all[start..end].to_vec(),
        total_rows,
        visible_start: start,
        all_rows: all,
    }
}

fn row_contains_anchor(row: &RenderedHistoryRow, anchor: LogicalContentAnchor) -> bool {
    if row.start_anchor.entry_id != anchor.entry_id
        || row.start_anchor.logical_line != anchor.logical_line
    {
        return false;
    }
    let start = row.start_anchor.scalar_offset;
    let end = row.end_anchor.scalar_offset;
    if start == end {
        return row.is_empty_logical_line && anchor.scalar_offset == start;
    }
    start <= anchor.scalar_offset
        && (anchor.scalar_offset < end
            || (row.is_last_row_of_logical_line && anchor.scalar_offset == end))
}

/// Stable semantic lines. Tool display is formatted once at an effectively
/// unbounded width; only `project_line` performs current-viewport wrapping.
fn logical_lines(block: &TranscriptBlock) -> Vec<ScrollbackLine> {
    match block {
        TranscriptBlock::StyledLine(line) => vec![line.clone()],
        TranscriptBlock::ToolCall(display) => {
            crate::tool_display::build_tool_call_scrollback_lines(display, u16::MAX)
        }
        TranscriptBlock::ToolResult(display) => {
            let icon = if display.is_error { "✗" } else { "" };
            let color = if display.is_error {
                crate::theme::to_crossterm_color(crate::theme::semantic::TEXT_ERROR)
            } else {
                crate::theme::to_crossterm_color(crate::theme::semantic::TEXT_SUCCESS)
            };
            crate::tool_display::build_tool_result_scrollback_lines(display, icon, color, u16::MAX)
        }
    }
}

struct ProjectedLineChunk {
    line: ScrollbackLine,
    logical_start: usize,
    logical_end: usize,
}

fn project_line_chunks(line: &ScrollbackLine, width: u16) -> Vec<ProjectedLineChunk> {
    let capacity = usize::from(width);
    let mut rows = Vec::new();
    let mut current = Vec::<HistorySegment>::new();
    let mut used = 0usize;
    let mut logical_offset = 0usize;
    let mut row_start = 0usize;

    for segment in &line.segments {
        for ch in segment.text.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            // A double-width scalar cannot be represented at width one. This is
            // projection-only; the original scalar remains in TranscriptStore.
            if char_width > capacity {
                if !current.is_empty() {
                    rows.push(ProjectedLineChunk {
                        line: ScrollbackLine::styled(std::mem::take(&mut current), line.bg),
                        logical_start: row_start,
                        logical_end: logical_offset,
                    });
                    used = 0;
                }
                rows.push(ProjectedLineChunk {
                    line: ScrollbackLine::plain("…", line.bg),
                    logical_start: logical_offset,
                    logical_end: logical_offset + 1,
                });
                logical_offset += 1;
                row_start = logical_offset;
                continue;
            }
            if used > 0 && used.saturating_add(char_width) > capacity {
                rows.push(ProjectedLineChunk {
                    line: ScrollbackLine::styled(std::mem::take(&mut current), line.bg),
                    logical_start: row_start,
                    logical_end: logical_offset,
                });
                used = 0;
                row_start = logical_offset;
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
            logical_offset += 1;
        }
    }
    if !current.is_empty() {
        rows.push(ProjectedLineChunk {
            line: ScrollbackLine::styled(current, line.bg),
            logical_start: row_start,
            logical_end: logical_offset,
        });
    }
    if rows.is_empty() && line.fill.is_none() {
        return vec![ProjectedLineChunk {
            line: line.clone(),
            logical_start: 0,
            logical_end: logical_offset,
        }];
    }
    if rows.is_empty() {
        rows.push(ProjectedLineChunk {
            line: ScrollbackLine::styled(Vec::new(), line.bg),
            logical_start: 0,
            logical_end: 0,
        });
    }
    if let Some(fill) = &line.fill
        && let Some(last) = rows.last_mut()
    {
        let used = UnicodeWidthStr::width(last.line.text.as_str());
        if let Some(fill) = project_fill_segment(fill, capacity.saturating_sub(used)) {
            last.line.segments.push(fill);
            last.line.text = last
                .line
                .segments
                .iter()
                .map(|segment| segment.text.as_str())
                .collect();
        }
    }
    rows
}

#[cfg(test)]
fn project_line(line: &ScrollbackLine, width: u16) -> Vec<ScrollbackLine> {
    project_line_chunks(line, width)
        .into_iter()
        .map(|chunk| chunk.line)
        .collect()
}

/// Repeats a fill token only up to the available display cells. The loop is by
/// Unicode scalar, so no scalar is split even when a multi-cell token cannot
/// fill the final cell exactly.
fn project_fill_segment(fill: &HistorySegment, available_cells: usize) -> Option<HistorySegment> {
    if available_cells == 0 || fill.text.is_empty() {
        return None;
    }
    let mut text = String::new();
    let mut used = 0usize;
    let scalars: Vec<char> = fill.text.chars().collect();
    if scalars.is_empty() {
        return None;
    }
    while used < available_cells {
        let mut progressed = false;
        for ch in &scalars {
            let width = UnicodeWidthChar::width(*ch).unwrap_or(0);
            if width == 0 {
                continue;
            }
            if used.saturating_add(width) > available_cells {
                continue;
            }
            text.push(*ch);
            used = used.saturating_add(width);
            progressed = true;
            if used == available_cells {
                break;
            }
        }
        if !progressed {
            break;
        }
    }
    (!text.is_empty()).then(|| HistorySegment::styled(text, fill.fg, fill.attrs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline_terminal::HistoryAttrs;

    #[test]
    fn repeated_resize_projection_is_reversible() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "abc你好xyz",
            None,
        )));
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain("", None)));
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
        assert_eq!(project_history(&transcript, 160, 100, &scroll), initial);
    }

    #[test]
    fn width_one_degrades_without_mutating_cjk() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "你", None,
        )));
        let scroll = HistoryScrollState::follow_tail();
        assert_eq!(
            project_history(&transcript, 1, 1, &scroll).rows[0]
                .line
                .text,
            "…"
        );
        assert!(
            matches!(&transcript.entries()[0].block, TranscriptBlock::StyledLine(line) if line.text == "你")
        );
        assert_eq!(
            project_history(&transcript, 2, 1, &scroll).rows[0]
                .line
                .text,
            "你"
        );
    }

    #[test]
    fn new_content_does_not_move_anchored_history() {
        let mut transcript = TranscriptStore::default();
        for index in 0..100 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                index.to_string(),
                None,
            )));
        }
        let tail = project_history(&transcript, 80, 10, &HistoryScrollState::follow_tail());
        let anchor = tail.rows[3].start_anchor;
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 3);
        let before = project_history(&transcript, 80, 10, &scroll);
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "new", None,
        )));
        let after = project_history(&transcript, 80, 10, &scroll);
        assert_eq!(before.rows[3].start_anchor, anchor);
        assert_eq!(after.rows[3].start_anchor, anchor);
    }

    #[test]
    fn resize_preserves_logical_scroll_anchor() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "anchor content with enough words to wrap on narrow width",
            None,
        )));
        let wide = project_history(&transcript, 80, 8, &HistoryScrollState::follow_tail());
        let anchor = wide.rows[0].start_anchor;
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        assert_eq!(
            project_history(&transcript, 40, 8, &scroll).rows[0]
                .start_anchor
                .entry_id,
            anchor.entry_id
        );
        assert_eq!(
            project_history(&transcript, 120, 8, &scroll).rows[0]
                .start_anchor
                .entry_id,
            anchor.entry_id
        );
    }

    #[test]
    fn resize_preserves_anchor_inside_multiline_entry() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "0123456789abcdefghijklmnopqrstuvwxyz你好世界",
            None,
        )));
        let wide = project_history(&transcript, 10, 20, &HistoryScrollState::follow_tail());
        let anchor = wide.rows[2].start_anchor;
        assert!(anchor.scalar_offset > 0);
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        for width in [40, 1, 80] {
            let projection = project_history(&transcript, width, 20, &scroll);
            let row = &projection.rows[0];
            assert!(row_contains_anchor(row, anchor));
        }
    }

    #[test]
    fn anchor_at_wrapped_row_start_resolves_to_that_row() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "0123456789abcdefghijABCDEFGHIJ",
            None,
        )));
        let projection = project_history(&transcript, 10, 10, &HistoryScrollState::follow_tail());
        let second_row_anchor = projection.rows[1].start_anchor;
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(second_row_anchor, 0);
        let anchored = project_history(&transcript, 10, 10, &scroll);
        assert_eq!(anchored.visible_start, 1);
        assert_eq!(anchored.rows[0].start_anchor, second_row_anchor);
    }

    #[test]
    fn anchor_at_logical_line_end_resolves_last_row() {
        let mut transcript = TranscriptStore::default();
        let id = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "0123456789abcdefghij",
            None,
        )));
        let eof = LogicalContentAnchor {
            entry_id: id,
            logical_line: 0,
            scalar_offset: 20,
        };
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(eof, 0);
        let projection = project_history(&transcript, 10, 10, &scroll);
        assert_eq!(projection.visible_start, 1);
        assert_eq!(projection.rows[0].end_anchor, eof);
    }

    #[test]
    fn empty_line_anchor_resolves_deterministically() {
        let mut transcript = TranscriptStore::default();
        let first = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain("", None)));
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain("", None)));
        let anchor = LogicalContentAnchor {
            entry_id: first,
            logical_line: 0,
            scalar_offset: 0,
        };
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        let projection = project_history(&transcript, 10, 2, &scroll);
        assert_eq!(projection.rows[0].start_anchor, anchor);
    }

    #[test]
    fn fill_only_line_anchor_is_independent_of_projected_fill_length() {
        for width in [1, 3, 40] {
            let mut transcript = TranscriptStore::default();
            let id = transcript.append(TranscriptBlock::StyledLine(
                ScrollbackLine::styled_with_fill(Vec::new(), None, Some(HistorySegment::raw("─"))),
            ));
            let projection =
                project_history(&transcript, width, 2, &HistoryScrollState::follow_tail());
            assert_eq!(
                projection.rows[0].start_anchor,
                LogicalContentAnchor {
                    entry_id: id,
                    logical_line: 0,
                    scalar_offset: 0,
                }
            );
            assert_eq!(projection.rows[0].end_anchor.scalar_offset, 0);
        }
    }

    #[test]
    fn fill_only_line_projects_to_current_width() {
        let line = ScrollbackLine::styled_with_fill(
            Vec::new(),
            Some(crossterm::style::Color::Blue),
            Some(HistorySegment::styled(
                "─",
                Some(crossterm::style::Color::Cyan),
                HistoryAttrs::default(),
            )),
        );
        for width in [1, 2, 3, 40] {
            let rows = project_line(&line, width);
            assert_eq!(rows.len(), 1);
            assert!(!rows[0].text.is_empty());
            assert!(UnicodeWidthStr::width(rows[0].text.as_str()) <= usize::from(width));
            assert_eq!(rows[0].bg, line.bg);
            assert_eq!(rows[0].segments[0].fg, Some(crossterm::style::Color::Cyan));
        }
    }

    #[test]
    fn multi_cell_fill_never_exceeds_viewport() {
        let fill = HistorySegment::raw("你好");
        for width in [1_u16, 2, 3, 4, 5, 40] {
            let projected = project_fill_segment(&fill, usize::from(width));
            assert!(
                projected
                    .as_ref()
                    .is_none_or(|segment| UnicodeWidthStr::width(segment.text.as_str())
                        <= usize::from(width))
            );
        }
    }

    #[test]
    fn fill_reprojects_reversibly_without_splitting_scalars() {
        let line =
            ScrollbackLine::styled_with_fill(Vec::new(), None, Some(HistorySegment::raw("你好")));
        let initial = project_line(&line, 40);
        for width in [1, 2, 3, 40] {
            for row in project_line(&line, width) {
                assert!(UnicodeWidthStr::width(row.text.as_str()) <= usize::from(width));
                assert!(!row.text.contains('\u{fffd}'));
            }
        }
        assert_eq!(project_line(&line, 40), initial);
    }

    #[test]
    fn missing_anchor_degrades_to_nearest_valid_position() {
        let mut transcript = TranscriptStore::default();
        let _first = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "a", None,
        )));
        let second = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "b", None,
        )));
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(
            LogicalContentAnchor {
                entry_id: second,
                logical_line: 0,
                scalar_offset: 99,
            },
            0,
        );
        let projection = project_history(&transcript, 80, 2, &scroll);
        assert_eq!(projection.rows[0].start_anchor.entry_id, second);
    }

    #[test]
    fn tool_call_reprojects_reversibly_across_40_1_160() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::ToolCall(
            talos_conversation::ToolCallDisplay {
                tool_name: "bash".into(),
                arguments: serde_json::json!({"command": "echo 你好 -- a long argument"}),
                provenance: talos_core::tool::ToolProvenance::Native,
                summary_fields: vec!["command".into()],
            },
        ));
        let stored = match &transcript.entries()[0].block {
            TranscriptBlock::ToolCall(display) => (
                display.tool_name.clone(),
                display.arguments.clone(),
                display.provenance.clone(),
                display.summary_fields.clone(),
            ),
            _ => panic!("tool call stored logically"),
        };
        let scroll = HistoryScrollState::follow_tail();
        let initial = project_history(&transcript, 160, 100, &scroll);
        for width in [40, 1, 160] {
            let _ = project_history(&transcript, width, 100, &scroll);
        }
        let after = match &transcript.entries()[0].block {
            TranscriptBlock::ToolCall(display) => (
                display.tool_name.clone(),
                display.arguments.clone(),
                display.provenance.clone(),
                display.summary_fields.clone(),
            ),
            _ => panic!("tool call stored logically"),
        };
        assert_eq!(after, stored);
        assert_eq!(project_history(&transcript, 160, 100, &scroll), initial);
    }

    #[test]
    fn anchor_inside_tool_call_arguments_survives_resize() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::ToolCall(
            talos_conversation::ToolCallDisplay {
                tool_name: "bash".into(),
                arguments: serde_json::json!({
                    "command": "printf 你好 && echo a-very-long-command-argument",
                    "cwd": "/tmp/project",
                    "timeout": 30
                }),
                provenance: talos_core::tool::ToolProvenance::Native,
                summary_fields: vec!["command".into(), "cwd".into()],
            },
        ));
        let initial = project_history(&transcript, 10, 100, &HistoryScrollState::follow_tail());
        let anchor = initial.rows[2].start_anchor;
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        for width in [160, 40, 3, 2, 1, 80, 160] {
            let projection = project_history(&transcript, width, 100, &scroll);
            assert!(row_contains_anchor(&projection.rows[0], anchor));
        }
    }

    #[test]
    fn anchor_inside_multiline_tool_result_survives_resize() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::ToolResult(
            talos_conversation::ToolResultDisplay {
                tool_name: Some("bash".into()),
                is_error: false,
                content: "first\nsecond line with 你好 and long content\n\nfourth\nfifth".into(),
            },
        ));
        let initial = project_history(&transcript, 8, 100, &HistoryScrollState::follow_tail());
        let anchor = initial
            .rows
            .iter()
            .find(|row| row.start_anchor.logical_line >= 1 && row.start_anchor.scalar_offset > 0)
            .expect("wrapped result row")
            .start_anchor;
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        for width in [160, 40, 3, 2, 1, 80, 160] {
            let projection = project_history(&transcript, width, 100, &scroll);
            assert!(row_contains_anchor(&projection.rows[0], anchor));
        }
    }
}
