//! Width-dependent projection of application-owned transcript facts.

use std::sync::Arc;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app_stream::ScrollbackLine;
use crate::inline_terminal::HistorySegment;
use crate::transcript::{TranscriptBlock, TranscriptEntryId, TranscriptStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LogicalContentAnchor {
    pub(crate) entry_id: TranscriptEntryId,
    pub(crate) logical_line: usize,
    pub(crate) scalar_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HistorySelectionPoint {
    anchor: LogicalContentAnchor,
    scalar_end: usize,
}

impl HistorySelectionPoint {
    fn order_key(self) -> (TranscriptEntryId, usize, usize) {
        (
            self.anchor.entry_id,
            self.anchor.logical_line,
            self.anchor.scalar_offset,
        )
    }
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
    pub(crate) expandable_header: bool,
    clipped_logical_length: Option<usize>,
    pub(crate) synthetic_prefix: usize,
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
    all_rows: Arc<Vec<RenderedHistoryRow>>,
    logical_lines: Arc<Vec<(TranscriptEntryId, usize, String)>>,
}

/// Beginning of the final projection unit, including pairing state before it.
/// Appends may extend this whole poll group or complete its pending call.
#[derive(Clone, Copy, Debug, Default)]
struct ProjectionTail {
    entry: usize,
    pending_calls: usize,
    rows: usize,
    logical_lines: usize,
}

#[derive(Debug, Default)]
pub(crate) struct HistoryProjectionCache {
    transcript_revision: Option<u64>,
    edit_revision: u64,
    entry_count: usize,
    tail: ProjectionTail,
    width: u16,
    all_rows: Arc<Vec<RenderedHistoryRow>>,
    logical_lines: Arc<Vec<(TranscriptEntryId, usize, String)>>,
    #[cfg(test)]
    rebuilds: usize,
}

impl HistoryProjectionCache {
    pub(crate) fn project(
        &mut self,
        transcript: &TranscriptStore,
        width: u16,
        height: u16,
        scroll: &HistoryScrollState,
    ) -> HistoryProjection {
        if width == 0 || height == 0 {
            return HistoryProjection::default();
        }

        self.ensure_projection(transcript, width);

        project_viewport(
            self.all_rows.clone(),
            self.logical_lines.clone(),
            height,
            scroll,
        )
    }

    /// Obtain cached row count without constructing a visible-window projection.
    pub(crate) fn total_rows(&mut self, transcript: &TranscriptStore, width: u16) -> usize {
        if width == 0 {
            return 0;
        }
        self.ensure_projection(transcript, width);
        self.all_rows.len()
    }

    pub(crate) fn needs_update(&self, transcript: &TranscriptStore, width: u16) -> bool {
        self.transcript_revision != Some(transcript.revision()) || self.width != width
    }

    fn ensure_projection(&mut self, transcript: &TranscriptStore, width: u16) {
        let revision = transcript.revision();
        if self.needs_update(transcript, width) {
            let _metric =
                crate::render_metrics::measure(crate::render_metrics::Stage::HistoryRebuild);
            let append_only = self.transcript_revision.is_some()
                && self.width == width
                && self.edit_revision == transcript.edit_revision()
                && self.entry_count < transcript.entries().len();
            if !append_only {
                // Replace instead of cloning an obsolete snapshot for a full reflow.
                self.all_rows = Arc::default();
                self.logical_lines = Arc::default();
                self.tail = ProjectionTail::default();
            }
            self.tail = project_history_tail(
                transcript,
                width,
                self.tail,
                Arc::make_mut(&mut self.all_rows),
                Arc::make_mut(&mut self.logical_lines),
            );
            self.transcript_revision = Some(revision);
            self.edit_revision = transcript.edit_revision();
            self.entry_count = transcript.entries().len();
            self.width = width;
            #[cfg(test)]
            {
                self.rebuilds = self.rebuilds.saturating_add(1);
            }
        } else {
            crate::render_metrics::count(crate::render_metrics::Stage::HistoryHit);
        }
    }

    #[cfg(test)]
    pub(crate) const fn rebuilds(&self) -> usize {
        self.rebuilds
    }
}

impl HistoryProjection {
    pub(crate) fn expandable_header_at(&self, row: usize) -> Option<TranscriptEntryId> {
        self.all_rows
            .get(row)
            .filter(|row| row.expandable_header)
            .map(|row| row.start_anchor.entry_id)
    }
    pub(crate) fn selection_point(
        &self,
        row_index: usize,
        column: u16,
    ) -> Option<HistorySelectionPoint> {
        let row = self.all_rows.get(row_index)?;
        if usize::from(column) < row.synthetic_prefix {
            // A padding hit is a logical boundary, not a terminal-only selection.
            // Otherwise a drag starting here copies rendered wrap/padding bytes.
            return Some(HistorySelectionPoint {
                anchor: row.start_anchor,
                scalar_end: row.start_anchor.scalar_offset,
            });
        }
        let rendered = row
            .line
            .segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>();
        let (scalar_start, scalar_end) = scalar_span_at_display_cell(&rendered, column);
        let scalar_start = scalar_start.saturating_sub(row.synthetic_prefix);
        let scalar_end = scalar_end.saturating_sub(row.synthetic_prefix);
        let semantic_len = row
            .end_anchor
            .scalar_offset
            .saturating_sub(row.start_anchor.scalar_offset);
        if scalar_start > semantic_len || scalar_end > semantic_len {
            return None;
        }
        Some(HistorySelectionPoint {
            anchor: LogicalContentAnchor {
                entry_id: row.start_anchor.entry_id,
                logical_line: row.start_anchor.logical_line,
                scalar_offset: row.start_anchor.scalar_offset.saturating_add(scalar_start),
            },
            scalar_end: row.start_anchor.scalar_offset.saturating_add(scalar_end),
        })
    }

    pub(crate) fn selected_text(
        &self,
        start: HistorySelectionPoint,
        end: HistorySelectionPoint,
    ) -> String {
        let (start, end) = if start.order_key() <= end.order_key() {
            (start, end)
        } else {
            (end, start)
        };
        let mut lines = Vec::new();
        for (entry_id, logical_line, text) in self.logical_lines.iter() {
            let key = (*entry_id, *logical_line);
            let start_key = (start.anchor.entry_id, start.anchor.logical_line);
            let end_key = (end.anchor.entry_id, end.anchor.logical_line);
            if key < start_key || key > end_key {
                continue;
            }
            let first = if key == start_key {
                start.anchor.scalar_offset
            } else {
                0
            };
            let last = if key == end_key {
                end.scalar_end
            } else {
                text.chars().count()
            };
            lines.push(select_scalar_range(text, first, last));
        }
        lines.join("\n")
    }

    /// Projects a logical selection onto the currently rendered frame.
    ///
    /// Logical anchors outlive wrapping and scrolling, so the highlight must
    /// be derived from the current row projection rather than stale pointer
    /// coordinates. Rows outside the viewport are clipped while preserving the
    /// full-row span between the logical endpoints.
    pub(crate) fn visible_selection(
        &self,
        start: HistorySelectionPoint,
        end: HistorySelectionPoint,
        frame_start: usize,
        prefix_rows: usize,
        area: ratatui::layout::Rect,
    ) -> Option<((u16, u16), (u16, u16))> {
        if area.width == 0 || area.height == 0 {
            return None;
        }

        let (start, end) = if start.order_key() <= end.order_key() {
            (start, end)
        } else {
            (end, start)
        };
        let start_key = (start.anchor.entry_id, start.anchor.logical_line);
        let end_key = (end.anchor.entry_id, end.anchor.logical_line);
        let mut selected_rows = Vec::new();

        for (row_index, row) in self.all_rows.iter().enumerate() {
            let key = (row.start_anchor.entry_id, row.start_anchor.logical_line);
            if key < start_key || key > end_key {
                continue;
            }

            let row_start = row.start_anchor.scalar_offset;
            let row_end = row.end_anchor.scalar_offset;
            let selected_start = if key == start_key {
                start.anchor.scalar_offset
            } else {
                row_start
            }
            .max(row_start);
            let selected_end = if key == end_key {
                end.scalar_end
            } else {
                row_end
            }
            .min(row_end);
            if selected_start >= selected_end {
                continue;
            }

            let rendered = row
                .line
                .segments
                .iter()
                .map(|segment| segment.text.as_str())
                .collect::<String>();
            let start_col = if key == start_key {
                display_column_for_scalar(
                    &rendered,
                    selected_start.saturating_sub(row_start) + row.synthetic_prefix,
                )
            } else {
                0
            };
            let end_col = if key == end_key {
                display_column_for_scalar(
                    &rendered,
                    selected_end.saturating_sub(row_start) + row.synthetic_prefix,
                )
                .saturating_sub(1)
            } else {
                usize::from(area.width.saturating_sub(1))
            };
            selected_rows.push((prefix_rows.saturating_add(row_index), start_col, end_col));
        }

        let first_selected = selected_rows.first()?.0;
        let last_selected = selected_rows.last()?.0;
        let visible_end = frame_start.saturating_add(usize::from(area.height));
        let visible = selected_rows
            .into_iter()
            .filter(|(row, _, _)| *row >= frame_start && *row < visible_end)
            .collect::<Vec<_>>();
        let first = visible.first()?;
        let last = visible.last()?;
        let start_col = if first.0 == first_selected {
            first.1
        } else {
            0
        };
        let end_col = if last.0 == last_selected {
            last.2
        } else {
            usize::from(area.width.saturating_sub(1))
        };
        let to_screen = |row: usize, col: usize| {
            (
                area.x
                    .saturating_add(col.min(usize::from(area.width.saturating_sub(1))) as u16),
                area.y
                    .saturating_add(row.saturating_sub(frame_start) as u16),
            )
        };
        Some((to_screen(first.0, start_col), to_screen(last.0, end_col)))
    }

    pub(crate) fn first_anchor(&self) -> Option<LogicalContentAnchor> {
        self.all_rows.first().map(|row| row.start_anchor)
    }

    pub(crate) fn anchor_at(&self, index: usize) -> Option<LogicalContentAnchor> {
        self.all_rows.get(index).map(|row| row.start_anchor)
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
}

fn scalar_span_at_display_cell(text: &str, target: u16) -> (usize, usize) {
    let chars = text.chars().collect::<Vec<_>>();
    let mut column = 0u16;
    for (index, ch) in chars.iter().enumerate() {
        let width = UnicodeWidthChar::width(*ch).unwrap_or(0) as u16;
        if width == 0 {
            continue;
        }
        if target < column.saturating_add(width) {
            let mut end = index + 1;
            while end < chars.len() && UnicodeWidthChar::width(chars[end]).unwrap_or(0) == 0 {
                end += 1;
            }
            return (index, end);
        }
        column = column.saturating_add(width);
    }
    (chars.len(), chars.len())
}

fn display_column_for_scalar(text: &str, scalar: usize) -> usize {
    let mut column = 0usize;
    for (scalar_index, ch) in text.chars().enumerate() {
        if scalar_index >= scalar {
            break;
        }
        column = column.saturating_add(UnicodeWidthChar::width(ch).unwrap_or(0));
    }
    column
}

fn select_scalar_range(text: &str, first: usize, end: usize) -> String {
    text.chars()
        .enumerate()
        .filter_map(|(index, ch)| (index >= first && index < end).then_some(ch))
        .collect()
}

/// Projects logical transcript lines without changing their stored content.
#[cfg(test)]
pub(crate) fn project_history(
    transcript: &TranscriptStore,
    width: u16,
    height: u16,
    scroll: &HistoryScrollState,
) -> HistoryProjection {
    HistoryProjectionCache::default().project(transcript, width, height, scroll)
}

fn project_history_tail(
    transcript: &TranscriptStore,
    width: u16,
    mut tail: ProjectionTail,
    all: &mut Vec<RenderedHistoryRow>,
    projected_logical_lines: &mut Vec<(TranscriptEntryId, usize, String)>,
) -> ProjectionTail {
    all.truncate(tail.rows);
    projected_logical_lines.truncate(tail.logical_lines);
    let mut projected = Vec::new();
    let entries = transcript.entries();
    let mut index = tail.entry;
    let mut pending_calls = tail.pending_calls;
    while index < entries.len() {
        tail = ProjectionTail {
            entry: index,
            pending_calls,
            rows: all.len(),
            logical_lines: projected_logical_lines.len(),
        };
        let entry = &entries[index];
        // Display events have no call IDs: never infer pairing inside a parallel batch.
        if pending_calls == 0
            && let Some(group) = crate::process_poll::group_at(&entries[index..])
        {
            let expanded = transcript.poll_expanded(entry.id);
            let color = if group.terminal && group.state != "completed" {
                crate::theme::to_crossterm_color(crate::theme::semantic::TEXT_ERROR)
            } else if group.terminal {
                crate::theme::to_crossterm_color(crate::theme::semantic::TEXT_SUCCESS)
            } else {
                crate::tool_display::secondary_result_color()
            };
            projected.push((
                entry.id,
                0,
                true,
                false,
                ScrollbackLine::plain(group.summary(expanded), color),
            ));
            if expanded {
                for (offset, detail) in entries[index..index + group.entries].iter().enumerate() {
                    let block = match &detail.block {
                        TranscriptBlock::ToolResult(result) => {
                            TranscriptBlock::ToolResult(crate::process_poll::detail_result(result))
                        }
                        block => block.clone(),
                    };
                    for (line_index, line) in logical_lines(&block).into_iter().enumerate() {
                        projected.push((
                            detail.id,
                            line_index + usize::from(offset == 0),
                            false,
                            matches!(block, TranscriptBlock::ToolResult(_)),
                            line,
                        ));
                    }
                }
            }
            index += group.entries;
        } else {
            match &entry.block {
                TranscriptBlock::ToolCall(_) => pending_calls = pending_calls.saturating_add(1),
                TranscriptBlock::ToolResult(_) => pending_calls = pending_calls.saturating_sub(1),
                _ => {}
            }
            for (line_index, line) in logical_lines(&entry.block).into_iter().enumerate() {
                let header =
                    matches!(entry.block, TranscriptBlock::Reasoning { .. }) && line_index == 0;
                projected.push((
                    entry.id,
                    line_index,
                    header,
                    matches!(entry.block, TranscriptBlock::ToolResult(_)),
                    line,
                ));
            }
            index += 1;
        }
        for (entry_id, logical_line, expandable_header, clip, line) in projected.drain(..) {
            projected_logical_lines.push((
                entry_id,
                logical_line,
                line.segments
                    .iter()
                    .map(|segment| segment.text.as_str())
                    .collect(),
            ));
            let mut chunks = if clip {
                project_result_line(&line, width)
            } else {
                project_line_chunks(&line, width)
            };
            // A collapsed entry remains one hit target even in a narrow viewport.
            // Keep the full semantic title above so resizing can restore it.
            if expandable_header {
                chunks.truncate(1);
            }
            let last = chunks.len().saturating_sub(1);
            all.extend(
                chunks
                    .into_iter()
                    .enumerate()
                    .map(|(index, chunk)| RenderedHistoryRow {
                        expandable_header,
                        clipped_logical_length: clip.then(|| line.text.chars().count()),
                        synthetic_prefix: chunk.synthetic_prefix,
                        start_anchor: LogicalContentAnchor {
                            entry_id,
                            logical_line,
                            scalar_offset: chunk.logical_start,
                        },
                        end_anchor: LogicalContentAnchor {
                            entry_id,
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
    tail
}

fn project_viewport(
    all: Arc<Vec<RenderedHistoryRow>>,
    logical_lines: Arc<Vec<(TranscriptEntryId, usize, String)>>,
    height: u16,
    scroll: &HistoryScrollState,
) -> HistoryProjection {
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
        logical_lines,
    }
}

fn row_contains_anchor(row: &RenderedHistoryRow, anchor: LogicalContentAnchor) -> bool {
    if row.start_anchor.entry_id != anchor.entry_id
        || row.start_anchor.logical_line != anchor.logical_line
    {
        return false;
    }
    if let Some(length) = row.clipped_logical_length {
        return anchor.scalar_offset <= length;
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
/// unbounded width; viewport projection wraps chat/calls and clips tool results.
fn logical_lines(block: &TranscriptBlock) -> Vec<ScrollbackLine> {
    match block {
        TranscriptBlock::Reasoning { text, expanded } => {
            let color = crate::tool_display::secondary_result_color();
            let mut lines = vec![ScrollbackLine::styled(
                vec![
                    HistorySegment::styled(
                        crate::scrollback::stream_padding_for(
                            Some(&talos_conversation::MessageSource::Reasoning),
                            0,
                        ),
                        crate::scrollback::prefix_color_for(
                            Some(&talos_conversation::MessageSource::Reasoning),
                            0,
                        ),
                        crate::inline_terminal::HistoryAttrs {
                            bold: true,
                            ..Default::default()
                        },
                    ),
                    HistorySegment::styled(
                        if *expanded {
                            "Thinking ▾"
                        } else {
                            "Thinking ▸"
                        },
                        color,
                        Default::default(),
                    ),
                ],
                None,
            )];
            if *expanded {
                lines.extend(text.lines().map(|text| {
                    let mut line = ScrollbackLine::styled(
                        vec![HistorySegment::styled(
                            format!("   {text}"),
                            color,
                            Default::default(),
                        )],
                        None,
                    );
                    line.continuation_indent = 3;
                    line
                }));
            }
            lines
        }
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
            crate::tool_display::build_tool_result_display_lines(display, icon, color)
        }
    }
}

struct ProjectedLineChunk {
    synthetic_prefix: usize,
    line: ScrollbackLine,
    logical_start: usize,
    logical_end: usize,
}

// Result logical lines remain intact above; only this viewport projection clips.
// The synthetic ellipsis is outside the selectable semantic range.
fn project_result_line(line: &ScrollbackLine, width: u16) -> Vec<ProjectedLineChunk> {
    let text = if width == 0 {
        String::new()
    } else {
        crate::scrollback::truncate_to_display_width(&line.text, usize::from(width))
    };
    let clipped = text != line.text;
    let visible = text
        .chars()
        .count()
        .saturating_sub(usize::from(clipped && width > 0));
    let mut remaining = visible;
    let mut segments = Vec::new();
    for segment in &line.segments {
        let take = remaining.min(segment.text.chars().count());
        if take > 0 {
            let mut part = segment.clone();
            part.text = segment.text.chars().take(take).collect();
            segments.push(part);
            remaining -= take;
        }
    }
    if clipped && width > 0 {
        let mut ellipsis = segments
            .last()
            .or_else(|| line.segments.first())
            .cloned()
            .unwrap_or_else(|| HistorySegment::styled("", None, Default::default()));
        ellipsis.text = "…".into();
        segments.push(ellipsis);
    }
    let mut rendered = ScrollbackLine::styled(segments, line.bg);
    if let Some(fill) = &line.fill {
        let used = UnicodeWidthStr::width(rendered.text.as_str());
        if let Some(fill) = project_fill_segment(fill, usize::from(width).saturating_sub(used)) {
            rendered.text.push_str(&fill.text);
            rendered.segments.push(fill);
        }
    }
    vec![ProjectedLineChunk {
        synthetic_prefix: 0,
        line: rendered,
        logical_start: 0,
        logical_end: visible,
    }]
}

fn project_line_chunks(line: &ScrollbackLine, width: u16) -> Vec<ProjectedLineChunk> {
    let capacity = usize::from(width);
    let mut rows = Vec::new();
    let mut current = Vec::<HistorySegment>::new();
    let mut used = 0usize;
    let mut logical_offset = 0usize;
    let mut row_start = 0usize;
    let mut synthetic_prefix = 0usize;

    for segment in &line.segments {
        for ch in segment.text.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            // A double-width scalar cannot be represented at width one. This is
            // projection-only; the original scalar remains in TranscriptStore.
            if char_width > capacity {
                if !current.is_empty() {
                    rows.push(ProjectedLineChunk {
                        synthetic_prefix,
                        line: ScrollbackLine::styled(std::mem::take(&mut current), line.bg),
                        logical_start: row_start,
                        logical_end: logical_offset,
                    });
                    used = 0;
                    synthetic_prefix = 0;
                }
                rows.push(ProjectedLineChunk {
                    synthetic_prefix: 0,
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
                    synthetic_prefix,
                    line: ScrollbackLine::styled(std::mem::take(&mut current), line.bg),
                    logical_start: row_start,
                    logical_end: logical_offset,
                });
                // Leave enough room for the next scalar even in a tiny viewport.
                synthetic_prefix = line
                    .continuation_indent
                    .min(capacity.saturating_sub(char_width.max(1)));
                used = synthetic_prefix;
                if synthetic_prefix > 0 {
                    current.push(HistorySegment::raw(" ".repeat(synthetic_prefix)));
                }
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
            synthetic_prefix,
            line: ScrollbackLine::styled(current, line.bg),
            logical_start: row_start,
            logical_end: logical_offset,
        });
    }
    if rows.is_empty() && line.fill.is_none() {
        return vec![ProjectedLineChunk {
            synthetic_prefix: 0,
            line: line.clone(),
            logical_start: 0,
            logical_end: logical_offset,
        }];
    }
    if rows.is_empty() {
        rows.push(ProjectedLineChunk {
            synthetic_prefix: 0,
            line: ScrollbackLine::styled(Vec::new(), line.bg),
            logical_start: 0,
            logical_end: 0,
        });
    }
    if let Some(fill) = &line.fill {
        let last_index = rows.len().saturating_sub(1);
        for (index, row) in rows.iter_mut().enumerate() {
            // Background padding covers every visual row, including the spare
            // cell before a wide character wraps. Decorative fills stay last-only.
            if index != last_index && fill.text != " " {
                continue;
            }
            let used = UnicodeWidthStr::width(row.line.text.as_str());
            if let Some(fill) = project_fill_segment(fill, capacity.saturating_sub(used)) {
                row.line.segments.push(fill);
                row.line.text = row
                    .line
                    .segments
                    .iter()
                    .map(|segment| segment.text.as_str())
                    .collect();
            }
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
    fn incremental_tail_matches_full_projection_for_grouping_edits_and_anchors() {
        let mut fixture = TranscriptStore::default();
        for (job, action, state) in [
            ("a", "read", "running"),
            ("a", "status", "running"),
            ("a", "read", "completed"),
            ("b", "read", "running"),
        ] {
            crate::process_poll::tests::append_poll(&mut fixture, job, action, state);
        }
        let blocks: Vec<_> = fixture
            .entries()
            .iter()
            .map(|entry| entry.block.clone())
            .collect();
        // Adjacent polls, pending tails, parallel calls, out-of-order results,
        // terminal observations, ordinary lines and expandable reasoning.
        let order = [0, 1, 2, 3, 0, 1, 4, 5, 6, 7, 0, 2, 3, 1, 7, 0, 1];
        for initial_width in [1, 7, 40, 120] {
            let mut transcript = TranscriptStore::default();
            let mut cache = HistoryProjectionCache::default();
            let mut retained = Vec::new();
            for step in 0..64 {
                let block = if step % 11 == 0 {
                    TranscriptBlock::Reasoning {
                        text: "中文 🦀 e\u{301}\nsecond".into(),
                        expanded: false,
                    }
                } else if step % 7 == 0 {
                    TranscriptBlock::StyledLine(ScrollbackLine::plain("ordinary 中文\nnext", None))
                } else {
                    blocks[order[step % order.len()]].clone()
                };
                transcript.append(block);
                // Toggle an old entry, not just the tail; edits require full reflow.
                if step % 9 == 0 {
                    let id = transcript.entries()[0].id;
                    transcript.toggle_expansion(id);
                }
                if step % 13 == 0 {
                    let ids: Vec<_> = transcript.entries().iter().map(|entry| entry.id).collect();
                    for id in ids {
                        transcript.toggle_expansion(id);
                    }
                }
                let width = if step % 5 == 0 {
                    initial_width + 1
                } else {
                    initial_width
                };
                let scroll = if step % 3 == 0 {
                    HistoryScrollState {
                        mode: HistoryScrollMode::Anchored {
                            anchor: LogicalContentAnchor {
                                entry_id: transcript.entries()[0].id,
                                logical_line: 0,
                                scalar_offset: 0,
                            },
                            screen_row: 2,
                        },
                    }
                } else {
                    HistoryScrollState::follow_tail()
                };
                let full = project_history(&transcript, width, 8, &scroll);
                let actual = cache.project(&transcript, width, 8, &scroll);
                assert_eq!(actual, full, "width {width}, step {step}");
                if let (Some(start), Some(end)) = (
                    actual.selection_point(0, 0),
                    actual.selection_point(actual.total_rows.saturating_sub(1), 0),
                ) {
                    assert_eq!(
                        actual.selected_text(start, end),
                        full.selected_text(start, end)
                    );
                }
                for (previous, expected) in &retained {
                    assert_eq!(previous, expected);
                }
                // Live older views must not change when the cache appends or regroups.
                retained.push((actual, full));
            }
        }
    }

    #[test]
    fn repeated_reflow_releases_obsolete_snapshots_after_view_is_dropped() {
        let mut transcript = TranscriptStore::default();
        for _ in 0..128 {
            transcript.append(TranscriptBlock::Reasoning {
                text: "中文 🦀 e\u{301}\nsecond line".into(),
                expanded: false,
            });
        }
        let mut cache = HistoryProjectionCache::default();
        let scroll = HistoryScrollState::follow_tail();
        for step in 0..128 {
            let width = [7, 80, 120, 1][step % 4];
            let view = cache.project(&transcript, width, 12, &scroll);
            let old_rows = Arc::downgrade(&view.all_rows);
            let old_lines = Arc::downgrade(&view.logical_lines);
            let id = transcript.entries()[step].id;
            assert!(transcript.toggle_expansion(id));
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("new history {step}"),
                None,
            )));
            let next = cache.project(&transcript, width, 12, &scroll);
            assert_eq!(next, project_history(&transcript, width, 12, &scroll));
            // Retained views are intentionally valid until their consumer releases them.
            assert!(old_rows.upgrade().is_some());
            assert!(old_lines.upgrade().is_some());
            drop(view);
            assert!(old_rows.upgrade().is_none(), "obsolete rows at step {step}");
            assert!(
                old_lines.upgrade().is_none(),
                "obsolete text at step {step}"
            );
            drop(next);
            assert_eq!(Arc::strong_count(&cache.all_rows), 1);
            assert_eq!(Arc::strong_count(&cache.logical_lines), 1);
        }
    }

    #[test]
    fn append_reuses_owned_prefix_without_cloning_its_text() {
        let mut transcript = TranscriptStore::default();
        for index in 0..100 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("stable {index}"),
                None,
            )));
        }
        let mut cache = HistoryProjectionCache::default();
        cache.total_rows(&transcript, 80);
        let text = cache.all_rows[0].line.text.as_ptr();
        let logical = cache.logical_lines[0].2.as_ptr();
        for _ in 0..30 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                "new tail", None,
            )));
            cache.total_rows(&transcript, 80);
            assert_eq!(cache.all_rows[0].line.text.as_ptr(), text);
            assert_eq!(cache.logical_lines[0].2.as_ptr(), logical);
            assert_eq!(
                cache.project(&transcript, 80, 8, &HistoryScrollState::follow_tail()),
                project_history(&transcript, 80, 8, &HistoryScrollState::follow_tail())
            );
        }
    }

    #[test]
    fn parallel_calls_are_not_guessed_into_poll_pairs() {
        let mut fixture = TranscriptStore::default();
        crate::process_poll::tests::append_poll(&mut fixture, "job_a", "read", "running");
        let call = fixture.entries()[0].block.clone();
        let result = fixture.entries()[1].block.clone();
        let mut transcript = TranscriptStore::default();
        for block in [call.clone(), call, result.clone(), result] {
            transcript.append(block);
        }
        let projected = project_history(&transcript, 120, 100, &HistoryScrollState::follow_tail());
        assert!(!projected.all_rows.iter().any(|row| row.expandable_header));
    }

    #[test]
    fn process_poll_expansion_resize_selection_and_later_anchor() {
        let mut transcript = TranscriptStore::default();
        let id =
            crate::process_poll::tests::append_poll(&mut transcript, "job_a", "read", "running");
        crate::process_poll::tests::append_poll(&mut transcript, "job_a", "status", "running");
        let later = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "later anchor",
            None,
        )));
        let mut cache = HistoryProjectionCache::default();
        for width in [18, 120, 30, 120] {
            let collapsed =
                cache.project(&transcript, width, 100, &HistoryScrollState::follow_tail());
            assert_eq!(collapsed.total_rows, 2);
            assert_eq!(collapsed.expandable_header_at(0), Some(id));
            let start = collapsed.selection_point(0, 0).expect("start");
            let end = collapsed.selection_point(0, 5).expect("end");
            assert!(collapsed.selected_text(start, end).starts_with("▸ proc"));
            assert!(transcript.toggle_expansion(id));
            let expanded =
                cache.project(&transcript, width, 100, &HistoryScrollState::follow_tail());
            assert!(expanded.total_rows > 5);
            assert!(
                expanded
                    .all_rows
                    .iter()
                    .any(|row| row.start_anchor.entry_id == later)
            );
            let mut scroll = HistoryScrollState::follow_tail();
            let anchor = LogicalContentAnchor {
                entry_id: later,
                logical_line: 0,
                scalar_offset: 0,
            };
            scroll.anchor(anchor, 0);
            assert_eq!(
                cache.project(&transcript, width, 1, &scroll).rows[0].start_anchor,
                anchor
            );
            assert!(transcript.toggle_expansion(id));
            assert_eq!(
                cache.project(&transcript, width, 1, &scroll).rows[0].start_anchor,
                anchor
            );
        }
    }

    #[test]
    fn result_history_clips_at_viewport_and_restores_content_after_resize() {
        let content = format!("+ {}TAIL\n- second", "中文e\u{301}".repeat(30));
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::ToolResult(
            talos_conversation::ToolResultDisplay {
                tool_name: Some("edit".into()),
                content: content.clone(),
                is_error: false,
                todo_presentation: None,
            },
        ));
        let raw = format!("{:?}", transcript.entries());
        let mut cache = HistoryProjectionCache::default();
        let scroll = HistoryScrollState::follow_tail();
        let narrow = cache.project(&transcript, 12, 100, &scroll);
        assert_eq!(narrow.total_rows, 2);
        assert!(narrow.rows[0].line.text.trim_end().ends_with('…'));
        assert!(
            narrow.selection_point(0, 11).is_none(),
            "ellipsis is synthetic"
        );
        let start = narrow.selection_point(0, 3).expect("start");
        let end = narrow.selection_point(0, 6).expect("end");
        let selected = narrow.selected_text(start, end);
        for width in [300, 12, 1] {
            let projection = cache.project(&transcript, width, 100, &scroll);
            assert_eq!(projection.total_rows, 2);
            assert!(
                projection
                    .rows
                    .iter()
                    .all(|row| row.line.text.width() <= usize::from(width))
            );
            assert_eq!(projection.selected_text(start, end), selected);
            assert!(projection.rows.iter().all(|row| row.line.bg.is_some()));
            if width == 300 {
                assert!(projection.rows[0].line.text.contains("TAIL"));
            }
        }
        assert_eq!(format!("{:?}", transcript.entries()), raw);
    }

    #[test]
    fn validation_history_clips_results_but_keeps_chat_wrapping() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::ToolResult(talos_conversation::ToolResultDisplay {
            tool_name: Some("validate_project".into()),
            content: serde_json::json!({"profile":"workspace", "authority":"read only",
                "findings":[], "checks":[{"id":"test", "command":"cargo test --locked --workspace", "required":true}]}).to_string(),
            is_error: false, todo_presentation: None,
        }));
        let scroll = HistoryScrollState::follow_tail();
        let narrow = project_history(&transcript, 12, 100, &scroll);
        assert_eq!(narrow.total_rows, 3);
        assert!(narrow.rows[0].line.text.ends_with('…'));
        let wide = project_history(&transcript, 200, 100, &scroll);
        assert!(wide.rows[0].line.text.contains("NOT EXECUTED"));
        assert!(
            wide.rows[2]
                .line
                .text
                .contains("cargo test --locked --workspace")
        );
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "a".repeat(40),
            None,
        )));
        assert!(project_history(&transcript, 12, 100, &scroll).total_rows > 4);
    }

    #[test]
    fn edit_background_fills_every_projected_row() {
        use talos_conversation::ToolResultDisplay;
        for count in [2, 40] {
            let display = ToolResultDisplay {
                todo_presentation: None,
                tool_name: Some("edit".into()),
                content: (0..count)
                    .map(|i| format!("{} 中文中文中文", if i % 2 == 0 { '+' } else { '-' }))
                    .collect::<Vec<_>>()
                    .join("\n"),
                is_error: false,
            };
            let lines = crate::tool_display::build_tool_result_scrollback_lines(
                &display,
                "",
                None,
                u16::MAX,
            );
            for width in [7, 20, 80] {
                for line in &lines {
                    let original = line.text.clone();
                    for row in project_line(line, width) {
                        if line.bg.is_some() {
                            assert_eq!(
                                UnicodeWidthStr::width(row.text.as_str()),
                                usize::from(width)
                            );
                            assert_eq!(row.bg, line.bg);
                        } else {
                            assert!(line.fill.is_none());
                        }
                    }
                    assert_eq!(line.text, original);
                }
            }
        }
    }

    #[test]
    fn row_count_matches_full_projection_after_width_and_revision_changes() {
        let mut transcript = TranscriptStore::default();
        let mut cache = HistoryProjectionCache::default();
        for text in [
            "中文 🦀 e\u{301}\nexplicit newline",
            "second entry with wrapping words",
        ] {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
            for width in [1, 7, 40, 80] {
                let expected =
                    project_history(&transcript, width, 6, &HistoryScrollState::follow_tail());
                assert_eq!(cache.total_rows(&transcript, width), expected.total_rows);
                let rebuilds = cache.rebuilds();
                let visible =
                    cache.project(&transcript, width, 6, &HistoryScrollState::follow_tail());
                assert_eq!(visible.rows, expected.rows);
                assert_eq!(cache.rebuilds(), rebuilds);
            }
        }
        assert_eq!(cache.total_rows(&transcript, 0), 0);
    }

    #[test]
    fn cache_reuses_full_projection_for_stable_transcript_and_width() {
        let mut transcript = TranscriptStore::default();
        for index in 0..1_000 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("history row {index:04}"),
                None,
            )));
        }
        let mut cache = HistoryProjectionCache::default();
        let scroll = HistoryScrollState::follow_tail();

        let first = cache.project(&transcript, 80, 24, &scroll);
        let second = cache.project(&transcript, 80, 24, &scroll);

        assert_eq!(cache.rebuilds(), 1);
        assert_eq!(second, first);
    }

    #[test]
    fn viewport_changes_do_not_rebuild_full_projection() {
        let mut transcript = TranscriptStore::default();
        for index in 0..100 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("history row {index:03}"),
                None,
            )));
        }
        let mut cache = HistoryProjectionCache::default();
        let tail = cache.project(&transcript, 80, 10, &HistoryScrollState::follow_tail());
        let anchor = tail.rows[2].start_anchor;
        let mut anchored = HistoryScrollState::follow_tail();
        anchored.anchor(anchor, 2);

        let shorter = cache.project(&transcript, 80, 5, &anchored);

        assert_eq!(cache.rebuilds(), 1);
        assert_eq!(shorter.rows.len(), 5);
        assert_eq!(shorter.rows[2].start_anchor, anchor);
    }

    #[test]
    fn transcript_append_and_width_change_invalidate_full_projection() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "first history row",
            None,
        )));
        let mut cache = HistoryProjectionCache::default();
        let scroll = HistoryScrollState::follow_tail();

        cache.project(&transcript, 80, 24, &scroll);
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "second history row",
            None,
        )));
        let appended = cache.project(&transcript, 80, 24, &scroll);
        let resized = cache.project(&transcript, 40, 24, &scroll);

        assert_eq!(cache.rebuilds(), 3);
        assert_eq!(appended.total_rows, 2);
        assert_eq!(resized.total_rows, 2);
    }

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
    fn expandable_header_stays_one_row_across_narrow_resize() {
        let mut transcript = TranscriptStore::default();
        let id = transcript.append(TranscriptBlock::Reasoning {
            text: "中文思考内容".into(),
            expanded: false,
        });
        let scroll = HistoryScrollState::follow_tail();
        for width in [40, 8, 3, 1, 8, 40] {
            let projection = project_history(&transcript, width, 20, &scroll);
            assert_eq!(projection.total_rows, 1, "width={width}");
            assert_eq!(projection.expandable_header_at(0), Some(id));
            assert!(projection.rows[0].line.text.width() <= usize::from(width));
            if width == 40 {
                assert_eq!(projection.rows[0].line.text, " ◇ Thinking ▸");
                let segments = &projection.rows[0].line.segments;
                assert_eq!(segments[0].text, " ◇ ");
                assert_eq!(
                    segments[0].fg,
                    crate::scrollback::prefix_color_for(
                        Some(&talos_conversation::MessageSource::Reasoning),
                        0,
                    )
                );
            }
        }
    }

    #[test]
    fn folding_reasoning_preserves_an_anchor_in_a_later_entry() {
        let mut transcript = TranscriptStore::default();
        let reasoning = transcript.append(TranscriptBlock::Reasoning {
            text: "long reasoning text with 中文 and multiple wrapped rows".into(),
            expanded: true,
        });
        let answer = transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "answer anchor",
            None,
        )));
        for _ in 0..12 {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                "later", None,
            )));
        }
        let anchor = LogicalContentAnchor {
            entry_id: answer,
            logical_line: 0,
            scalar_offset: 0,
        };
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        let mut cache = HistoryProjectionCache::default();
        for width in [12, 30, 8, 30] {
            for _ in 0..2 {
                let projection = cache.project(&transcript, width, 4, &scroll);
                assert_eq!(projection.rows[0].start_anchor, anchor);
                assert!(transcript.toggle_reasoning(reasoning));
            }
        }
        let tail = cache.project(&transcript, 30, 4, &HistoryScrollState::follow_tail());
        assert_eq!(tail.visible_start + tail.rows.len(), tail.total_rows);
        assert!(transcript.toggle_reasoning(reasoning));
        let tail = cache.project(&transcript, 30, 4, &HistoryScrollState::follow_tail());
        assert_eq!(tail.visible_start + tail.rows.len(), tail.total_rows);
    }

    #[test]
    fn reasoning_entries_default_collapsed_and_toggle_independently() {
        let mut transcript = TranscriptStore::default();
        let first = transcript.append(TranscriptBlock::Reasoning {
            text: "first reasoning body".into(),
            expanded: false,
        });
        let second = transcript.append(TranscriptBlock::Reasoning {
            text: "second reasoning body".into(),
            expanded: false,
        });
        let scroll = HistoryScrollState::follow_tail();
        let collapsed = project_history(&transcript, 40, 20, &scroll);
        assert_eq!(collapsed.total_rows, 2);
        assert_eq!(collapsed.expandable_header_at(0), Some(first));
        assert_eq!(collapsed.expandable_header_at(1), Some(second));
        let revision = transcript.revision();
        assert!(transcript.toggle_reasoning(first));
        assert!(transcript.revision() > revision);
        let expanded = project_history(&transcript, 40, 20, &scroll);
        assert_eq!(expanded.total_rows, 3);
        assert_eq!(expanded.rows[1].line.text, "   first reasoning body");
        assert_eq!(expanded.expandable_header_at(1), None);
        assert!(
            !expanded
                .rows
                .iter()
                .any(|row| row.line.text.contains("second reasoning body"))
        );
        assert!(transcript.toggle_reasoning(first));
        assert_eq!(project_history(&transcript, 40, 20, &scroll).total_rows, 2);
    }

    #[test]
    fn ordinary_history_continuations_keep_prefix_without_copying_synthetic_spaces() {
        for source in [
            talos_conversation::MessageSource::User,
            talos_conversation::MessageSource::Assistant,
            talos_conversation::MessageSource::Reasoning,
        ] {
            for text in [
                "abcdefghijklmnopqrstuv",
                "中文历史续行需要保留空白前缀并正确复制",
            ] {
                let mut transcript = TranscriptStore::default();
                let mut stream_count = 0;
                for line in crate::scrollback::render_history_message(
                    &mut stream_count,
                    source.clone(),
                    text,
                ) {
                    transcript.append(TranscriptBlock::StyledLine(line));
                }
                for width in [12, 8, 20, 8] {
                    let projection = project_history(
                        &transcript,
                        width,
                        100,
                        &HistoryScrollState::follow_tail(),
                    );
                    let continuation = projection
                        .all_rows
                        .iter()
                        .enumerate()
                        .find(|(_, row)| row.start_anchor.scalar_offset > 0)
                        .expect("wrapped row");
                    assert!(
                        continuation.1.line.text.starts_with("   "),
                        "{source:?} width {width}: {:?}",
                        continuation.1.line.text
                    );
                    let padding = projection
                        .selection_point(continuation.0, 0)
                        .expect("synthetic prefix maps to a logical boundary");
                    assert_eq!(padding.anchor, continuation.1.start_anchor);
                    assert_eq!(projection.selected_text(padding, padding), "");
                    let point = projection
                        .selection_point(continuation.0, 3)
                        .expect("first content cell");
                    assert_eq!(point.anchor, continuation.1.start_anchor);
                    assert!(!projection.selected_text(point, point).starts_with(' '));
                    assert_eq!(
                        projection.selected_text(padding, point),
                        projection.selected_text(point, point)
                    );
                }
            }
        }
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
    fn tiny_padded_history_preserves_unicode_payload_and_width_bound() {
        for text in ["abcdef", "中文🙂a中文", "e\u{301}中a"] {
            let mut transcript = TranscriptStore::default();
            let mut line = ScrollbackLine::plain(text, None);
            line.continuation_indent = 3;
            transcript.append(TranscriptBlock::StyledLine(line));
            for width in 1..=7 {
                let projection =
                    project_history(&transcript, width, 100, &HistoryScrollState::follow_tail());
                for row in &projection.rows {
                    assert!(
                        row.line.text.width() <= usize::from(width),
                        "width={width}: {:?}",
                        row.line.text
                    );
                }
                let first = projection.selection_point(0, 0).expect("first cell");
                let last_index = projection.total_rows - 1;
                let last = &projection.rows[last_index];
                let last_column = last.line.text.width().saturating_sub(1) as u16;
                let end = projection
                    .selection_point(last_index, last_column)
                    .expect("last cell");
                assert_eq!(projection.selected_text(first, end), text, "width={width}");
            }
        }
    }

    #[test]
    fn wrapped_markdown_preserves_styles_and_synthetic_padding() {
        let mut stream_count = 0;
        let lines = crate::scrollback::render_history_message(
            &mut stream_count,
            talos_conversation::MessageSource::Assistant,
            "**abcdefghijklmnopqrstuv**\n",
        );
        let mut transcript = TranscriptStore::default();
        for line in lines {
            transcript.append(TranscriptBlock::StyledLine(line));
        }
        let projection = project_history(&transcript, 10, 100, &HistoryScrollState::follow_tail());
        assert!(projection.rows.len() > 1);
        for row in &projection.rows {
            if row.synthetic_prefix > 0 {
                assert!(row.line.text.starts_with("   "));
            }
            for segment in &row.line.segments {
                if segment.text.chars().any(|ch| ch.is_ascii_alphabetic()) {
                    assert!(segment.attrs.bold, "Markdown emphasis lost across wrap");
                }
            }
        }
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
                todo_presentation: None,
                tool_name: Some("bash".into()),
                is_error: false,
                content: "first\nsecond line with 你好 and long content\n\nfourth\nfifth".into(),
            },
        ));
        let initial = project_history(&transcript, 8, 100, &HistoryScrollState::follow_tail());
        assert_eq!(
            initial.total_rows, 5,
            "explicit newlines remain logical rows"
        );
        let mut anchor = initial.rows[1].start_anchor;
        anchor.scalar_offset = 20; // Hidden at narrow widths, restored at wide widths.
        let mut scroll = HistoryScrollState::follow_tail();
        scroll.anchor(anchor, 0);
        for width in [160, 40, 3, 2, 1, 80, 160] {
            let projection = project_history(&transcript, width, 100, &scroll);
            assert!(row_contains_anchor(&projection.rows[0], anchor));
        }
    }

    #[test]
    fn selected_text_spans_rows_outside_the_visible_window() {
        let mut transcript = TranscriptStore::default();
        for text in ["alpha", "中文🙂", "omega"] {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
        }
        let projection = project_history(&transcript, 20, 1, &HistoryScrollState::follow_tail());
        let start = projection.selection_point(0, 2).expect("first row point");
        let end = projection.selection_point(2, 2).expect("last row point");

        assert_eq!(projection.selected_text(start, end), "pha\n中文🙂\nome");
    }

    #[test]
    fn logical_selection_payload_survives_reflow() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "abcdefghij",
            None,
        )));
        let narrow = project_history(&transcript, 4, 10, &HistoryScrollState::follow_tail());
        let start = narrow.selection_point(0, 1).expect("b point");
        let end = narrow.selection_point(1, 3).expect("h point");
        assert_eq!(narrow.selected_text(start, end), "bcdefgh");

        let wide = project_history(&transcript, 10, 10, &HistoryScrollState::follow_tail());
        assert_eq!(wide.selected_text(start, end), "bcdefgh");
    }

    #[test]
    fn visible_selection_reprojects_after_scroll() {
        let mut transcript = TranscriptStore::default();
        for text in ["alpha", "bravo", "charlie"] {
            transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
        }
        let projection = project_history(&transcript, 20, 3, &HistoryScrollState::follow_tail());
        let start = projection.selection_point(0, 0).expect("start");
        let end = projection.selection_point(2, 6).expect("end");
        let area = ratatui::layout::Rect::new(0, 0, 20, 3);

        assert_eq!(
            projection.visible_selection(start, end, 1, 1, area),
            Some(((0, 0), (6, 2)))
        );
        assert_eq!(
            projection.visible_selection(start, end, 2, 1, area),
            Some(((0, 0), (6, 1)))
        );
    }

    #[test]
    fn visible_selection_limits_a_click_to_its_wrapped_row() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "abcdefghijklmnopqrstuvwxyz",
            None,
        )));
        let projection = project_history(&transcript, 10, 3, &HistoryScrollState::follow_tail());
        let point = projection.selection_point(1, 3).expect("wrapped point");
        let area = ratatui::layout::Rect::new(0, 0, 10, 3);

        assert_eq!(
            projection.visible_selection(point, point, 0, 0, area),
            Some(((3, 1), (3, 1)))
        );
    }

    #[test]
    fn logical_selection_preserves_selected_trailing_spaces() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "alpha  ", None,
        )));
        let projection = project_history(&transcript, 20, 10, &HistoryScrollState::follow_tail());
        let start = projection.selection_point(0, 0).expect("start");
        let end = projection
            .selection_point(0, 6)
            .expect("second trailing space");

        assert_eq!(projection.selected_text(start, end), "alpha  ");
    }

    #[test]
    fn logical_selection_payload_survives_appended_redraw_content() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "selected text",
            None,
        )));
        let initial = project_history(&transcript, 20, 10, &HistoryScrollState::follow_tail());
        let start = initial.selection_point(0, 0).expect("start");
        let end = initial.selection_point(0, 7).expect("end");

        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "later stream output",
            None,
        )));
        let redrawn = project_history(&transcript, 20, 10, &HistoryScrollState::follow_tail());
        assert_eq!(redrawn.selected_text(start, end), "selected");
    }

    #[test]
    fn display_cell_selection_keeps_combining_marks_with_their_base() {
        let mut transcript = TranscriptStore::default();
        transcript.append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "a\u{301}bc",
            None,
        )));
        let projection = project_history(&transcript, 10, 10, &HistoryScrollState::follow_tail());
        let first = projection.selection_point(0, 0).expect("combined point");
        let second = projection.selection_point(0, 1).expect("second point");

        assert_eq!(projection.selected_text(first, first), "a\u{301}");
        assert_eq!(projection.selected_text(second, second), "b");
    }
}
