use crossterm::style::Color as CColor;
use talos_conversation::MessageSource;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::highlight::HighlightEngine;
use crate::inline_terminal::{HistoryAttrs, HistorySegment};
use crate::stream_markdown::{
    BlockDecision, FallbackReason, HoldStatus, MarkdownBlockKind, StreamBlockClassifier,
};

pub(crate) const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Clone, Debug, Eq)]
pub(crate) struct ScrollbackLine {
    pub(crate) text: String,
    pub(crate) segments: Vec<HistorySegment>,
    pub(crate) bg: Option<CColor>,
    pub(crate) fill: Option<HistorySegment>,
}

impl PartialEq for ScrollbackLine {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.bg == other.bg && self.fill == other.fill
    }
}

impl ScrollbackLine {
    pub(crate) fn plain(text: impl Into<String>, bg: Option<CColor>) -> Self {
        let text = text.into();
        Self {
            segments: vec![HistorySegment::raw(text.clone())],
            text,
            bg,
            fill: None,
        }
    }

    pub(crate) fn styled(segments: Vec<HistorySegment>, bg: Option<CColor>) -> Self {
        Self::styled_with_fill(segments, bg, None)
    }

    pub(crate) fn styled_with_fill(
        segments: Vec<HistorySegment>,
        bg: Option<CColor>,
        fill: Option<HistorySegment>,
    ) -> Self {
        let text = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>();
        Self {
            text,
            segments,
            bg,
            fill,
        }
    }

    pub(crate) fn has_plain_segments_only(&self) -> bool {
        if self.fill.is_some() {
            return false;
        }
        self.segments
            .iter()
            .all(|segment| segment.fg.is_none() && segment.attrs == HistoryAttrs::default())
    }
}

/// Wraps one finalized history line to the terminal width.
///
/// Finalized scrollback is inserted one physical row at a time. Relying on the
/// terminal's implicit wrap would scroll the viewport by only one row even when
/// a logical line occupied several rows. This helper makes every occupied row
/// explicit, preserves segment styling/backgrounds, and aligns continuations
/// beneath the standard three-column history prefix.
pub(crate) fn wrap_scrollback_line(line: ScrollbackLine, width: u16) -> Vec<ScrollbackLine> {
    if width == 0 {
        return vec![];
    }
    if line.fill.is_some() || UnicodeWidthStr::width(line.text.as_str()) <= width as usize {
        return vec![line];
    }

    let continuation = line
        .segments
        .first()
        .filter(|_| line.segments.len() > 1)
        .and_then(|segment| {
            let prefix_width = UnicodeWidthStr::width(segment.text.as_str());
            (prefix_width > 0 && prefix_width <= 3 && prefix_width < width as usize).then(|| {
                HistorySegment::styled(
                    " ".repeat(prefix_width),
                    segment.fg,
                    HistoryAttrs::default(),
                )
            })
        });

    let w_cap = width as usize;
    let mut rows = Vec::new();
    let mut current = Vec::new();
    let mut used = 0usize;

    for segment in line.segments {
        for ch in segment.text.chars() {
            let raw_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            // A character whose display width exceeds the viewport (e.g. CJK
            // width 2 at viewport width 1) cannot fit without splitting a
            // Unicode scalar. Substitute with a 1-cell marker so the row
            // count and width constraint remain exact.
            let (effective_ch, effective_width) = if raw_width > w_cap {
                ('.', 1usize)
            } else {
                (ch, raw_width)
            };

            if used > 0 && used.saturating_add(effective_width) > w_cap {
                rows.push(ScrollbackLine::styled(
                    std::mem::take(&mut current),
                    line.bg,
                ));
                used = 0;
                if let Some(prefix) = continuation.as_ref() {
                    let prefix_w = UnicodeWidthStr::width(prefix.text.as_str());
                    // Only add continuation indent if it leaves room for the
                    // current character; prevents prefix-only rows.
                    if prefix_w.saturating_add(effective_width) <= w_cap {
                        used = prefix_w;
                        current.push(prefix.clone());
                    }
                }
            }

            append_styled_char(&mut current, effective_ch, &segment);
            used = used.saturating_add(effective_width);
        }
    }

    if !current.is_empty() {
        rows.push(ScrollbackLine::styled(current, line.bg));
    }
    rows
}

pub(crate) fn prepare_history_rows(
    lines: Vec<ScrollbackLine>,
    viewport_width: u16,
) -> Vec<ScrollbackLine> {
    lines
        .into_iter()
        .flat_map(|line| wrap_scrollback_line(line, viewport_width))
        .collect()
}

fn append_styled_char(segments: &mut Vec<HistorySegment>, ch: char, source: &HistorySegment) {
    if let Some(last) = segments.last_mut()
        && last.fg == source.fg
        && last.attrs == source.attrs
    {
        last.text.push(ch);
        return;
    }
    segments.push(HistorySegment::styled(
        ch.to_string(),
        source.fg,
        source.attrs,
    ));
}

#[derive(Default)]
pub(crate) struct StreamRenderState {
    source: Option<MessageSource>,
    line_count: usize,
    buffer: String,
    preview: String,
    hold_complete_lines: bool,
    held_lines: Vec<(usize, String)>,
    block_classifier: StreamBlockClassifier,
    hold_status: Option<HoldStatus>,
    highlight_engine: HighlightEngine,
}

impl StreamRenderState {
    pub(crate) fn start(&mut self, source: MessageSource) -> Vec<ScrollbackLine> {
        self.start_with_hold(source, false)
    }

    pub(crate) fn start_with_hold(
        &mut self,
        source: MessageSource,
        hold_complete_lines: bool,
    ) -> Vec<ScrollbackLine> {
        let bg = crate::scrollback::stream_bg_for(Some(&source));
        self.source = Some(source);
        self.line_count = 0;
        self.buffer.clear();
        self.preview.clear();
        self.hold_complete_lines = hold_complete_lines;
        self.held_lines.clear();
        self.block_classifier.reset();
        self.hold_status = None;

        if bg.is_some() {
            vec![ScrollbackLine::plain(String::new(), bg)]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn source(&self) -> Option<&MessageSource> {
        self.source.as_ref()
    }

    pub(crate) fn preview(&self) -> &str {
        &self.preview
    }

    pub(crate) fn hold_status(&self) -> Option<&HoldStatus> {
        self.hold_status.as_ref()
    }

    pub(crate) fn push_chunk(&mut self, chunk: &str) -> Vec<ScrollbackLine> {
        self.buffer.push_str(chunk);
        let mut lines = Vec::new();

        while let Some(pos) = self.buffer.find('\n') {
            let line = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 1..].to_string();
            if self.hold_complete_lines {
                self.held_lines.push((self.line_count, line));
            } else {
                lines.extend(self.push_complete_line(line));
            }
        }

        if self.hold_status.is_none() {
            self.preview = self.buffer.clone();
        }
        lines
    }

    pub(crate) fn finish(&mut self) -> Vec<ScrollbackLine> {
        let mut lines = Vec::new();

        let held_lines = std::mem::take(&mut self.held_lines);
        for (_, line) in held_lines {
            lines.push(self.render_next_line(&line));
        }

        let decisions = self.block_classifier.finish();
        lines.extend(self.apply_block_decisions(decisions));

        if !self.preview.is_empty() {
            let preview = std::mem::take(&mut self.preview);
            lines.push(self.render_next_line(&preview));
        }

        if self.bg().is_some() {
            lines.push(ScrollbackLine::plain(String::new(), self.bg()));
        }

        self.reset();
        lines
    }

    fn render_line(
        &self,
        line_index: usize,
        line: &str,
        block: Option<(&MarkdownBlockKind, usize)>,
    ) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            crate::scrollback::prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        if block.is_none() && crate::scrollback::is_horizontal_rule(line) {
            let fill = crate::scrollback::horizontal_rule_segment("─");
            segments.push(fill.clone());
            return ScrollbackLine::styled_with_fill(segments, self.bg(), Some(fill));
        }
        segments.extend(crate::scrollback::render_markdown_segments(line, block));
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_segments_line(
        &self,
        line_index: usize,
        content_segments: Vec<HistorySegment>,
    ) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let mut segments = vec![HistorySegment::styled(
            padding,
            crate::scrollback::prefix_color_for(self.source(), line_index),
            HistoryAttrs {
                bold: line_index == 0 && self.source().is_some(),
                ..HistoryAttrs::default()
            },
        )];
        segments.extend(content_segments);
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_plain_line(&self, line_index: usize, line: &str) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        let segments = vec![
            HistorySegment::styled(
                padding,
                crate::scrollback::prefix_color_for(self.source(), line_index),
                HistoryAttrs {
                    bold: line_index == 0 && self.source().is_some(),
                    ..HistoryAttrs::default()
                },
            ),
            HistorySegment::raw(line),
        ];
        ScrollbackLine::styled(segments, self.bg())
    }

    fn render_block_line(
        &mut self,
        line: &str,
        kind: &MarkdownBlockKind,
        block_line_index: usize,
    ) -> ScrollbackLine {
        let rendered = self.render_line(self.line_count, line, Some((kind, block_line_index)));
        self.line_count += 1;
        rendered
    }

    fn render_next_line(&mut self, line: &str) -> ScrollbackLine {
        let rendered = if matches!(self.source(), Some(MessageSource::Reasoning)) {
            self.render_reasoning_line(self.line_count, line)
        } else if self.markdown_enabled() {
            self.render_line(self.line_count, line, None)
        } else {
            self.render_plain_line(self.line_count, line)
        };
        self.line_count += 1;
        rendered
    }

    fn render_reasoning_line(&self, line_index: usize, line: &str) -> ScrollbackLine {
        let padding = crate::scrollback::stream_padding_for(self.source(), line_index);
        ScrollbackLine::styled(
            vec![
                HistorySegment::styled(
                    padding,
                    crate::scrollback::prefix_color_for(self.source(), line_index),
                    HistoryAttrs {
                        bold: line_index == 0,
                        ..HistoryAttrs::default()
                    },
                ),
                HistorySegment::styled(
                    line,
                    crate::tool_display::secondary_result_color(),
                    HistoryAttrs::default(),
                ),
            ],
            self.bg(),
        )
    }

    fn render_block_lines(
        &mut self,
        kind: &MarkdownBlockKind,
        block_lines: Vec<String>,
    ) -> Vec<ScrollbackLine> {
        if kind == &MarkdownBlockKind::Table {
            return self.render_table_lines(block_lines);
        }
        if kind == &MarkdownBlockKind::CodeFence {
            let bg_source = self.source().cloned();
            let bg = crate::scrollback::stream_bg_for(bg_source.as_ref());

            if block_lines.len() >= 3 {
                let opening = &block_lines[0];
                let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();
                if lang == "mermaid" {
                    let code_lines = &block_lines[1..block_lines.len() - 1];
                    let mermaid_src = code_lines.join("\n");
                    return crate::scrollback::render_mermaid_block(&mermaid_src, bg);
                }
            }

            if let Some(rendered) =
                Self::try_highlight_code_block(&mut self.highlight_engine, &block_lines, bg_source)
            {
                return rendered;
            }
            return crate::scrollback::render_code_block(&block_lines, bg);
        }

        let mut rendered = Vec::with_capacity(block_lines.len());
        for (block_line_index, line) in block_lines.into_iter().enumerate() {
            rendered.push(self.render_block_line(&line, kind, block_line_index));
        }
        rendered
    }

    fn try_highlight_code_block(
        engine: &mut HighlightEngine,
        block_lines: &[String],
        source: Option<MessageSource>,
    ) -> Option<Vec<ScrollbackLine>> {
        if block_lines.len() < 3 {
            return None;
        }

        let opening = &block_lines[0];
        let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();

        if lang.is_empty() || !engine.supports(lang) {
            return None;
        }

        let code_lines = &block_lines[1..block_lines.len() - 1];
        let code = code_lines.join("\n");
        let highlighted_lines = engine.highlight(lang, &code)?;

        Some(crate::scrollback::build_code_block(
            &highlighted_lines,
            lang,
            crate::scrollback::stream_bg_for(source.as_ref()),
        ))
    }

    fn render_table_lines(&mut self, block_lines: Vec<String>) -> Vec<ScrollbackLine> {
        let table_lines =
            crate::scrollback::render_table_block(&block_lines).unwrap_or_else(|| {
                block_lines
                    .into_iter()
                    .enumerate()
                    .map(|(row_index, line)| {
                        crate::scrollback::render_table_history_line(&line, row_index)
                    })
                    .collect()
            });
        let mut rendered = Vec::with_capacity(table_lines.len());
        for content_segments in table_lines {
            let line = self.render_segments_line(self.line_count, content_segments);
            self.line_count += 1;
            rendered.push(line);
        }
        rendered
    }

    fn push_complete_line(&mut self, line: String) -> Vec<ScrollbackLine> {
        if !self.markdown_enabled() {
            return vec![self.render_next_line(&line)];
        }
        let decisions = self.block_classifier.push_line(line);
        self.apply_block_decisions(decisions)
    }

    fn markdown_enabled(&self) -> bool {
        !matches!(self.source(), Some(MessageSource::User))
    }

    fn apply_block_decisions(&mut self, decisions: Vec<BlockDecision>) -> Vec<ScrollbackLine> {
        let mut lines = Vec::new();
        for decision in decisions {
            match decision {
                BlockDecision::ImmediateLine(line) => {
                    self.hold_status = None;
                    if self.buffer.is_empty() {
                        self.preview.clear();
                    }
                    lines.push(self.render_next_line(&line));
                }
                BlockDecision::StartHold { status } | BlockDecision::ContinueHold { status } => {
                    self.preview = status.preview_text().to_string();
                    self.hold_status = Some(status);
                }
                BlockDecision::FinishHold {
                    status: _,
                    kind,
                    lines: rendered,
                } => {
                    self.hold_status = None;
                    self.preview = self.buffer.clone();
                    lines.extend(self.render_block_lines(&kind, rendered));
                }
                BlockDecision::FallbackImmediate {
                    status: _,
                    kind,
                    reason,
                    lines: rendered,
                } => {
                    self.hold_status = None;
                    self.preview = self.buffer.clone();
                    if kind == MarkdownBlockKind::CodeFence
                        && reason == FallbackReason::UnterminatedCodeFence
                    {
                        lines.extend(self.render_inline_fallback_lines(rendered));
                    } else {
                        lines.extend(self.render_block_lines(&kind, rendered));
                    }
                }
            }
        }
        lines
    }

    fn render_inline_fallback_lines(&mut self, block_lines: Vec<String>) -> Vec<ScrollbackLine> {
        block_lines
            .into_iter()
            .map(|line| {
                let trimmed = line.trim_start();
                let rendered = if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                    self.render_plain_line(self.line_count, &line)
                } else {
                    self.render_line(self.line_count, &line, None)
                };
                self.line_count += 1;
                rendered
            })
            .collect()
    }

    fn bg(&self) -> Option<CColor> {
        crate::scrollback::stream_bg_for(self.source())
    }

    pub(crate) fn reset(&mut self) {
        self.source = None;
        self.line_count = 0;
        self.buffer.clear();
        self.preview.clear();
        self.hold_complete_lines = false;
        self.held_lines.clear();
        self.block_classifier.reset();
        self.hold_status = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline_terminal::HistorySegment;

    fn styled_line(text: &str) -> ScrollbackLine {
        ScrollbackLine::styled(vec![HistorySegment::raw(text)], None)
    }

    fn line_with_prefix(prefix: &str, body: &str) -> ScrollbackLine {
        ScrollbackLine::styled(
            vec![HistorySegment::raw(prefix), HistorySegment::raw(body)],
            None,
        )
    }

    fn width_of(s: &str) -> usize {
        unicode_width::UnicodeWidthStr::width(s)
    }

    #[test]
    fn width_0_returns_empty_to_skip_history_insertion() {
        let line = styled_line(&"x".repeat(50));
        let out = wrap_scrollback_line(line, 0);
        assert!(
            out.is_empty(),
            "width 0 must return empty to avoid terminal autowrap"
        );
    }

    #[test]
    fn narrow_widths_split_into_rows_within_viewport() {
        let line = styled_line(&"x".repeat(50));
        for w in [1u16, 2, 3] {
            let out = wrap_scrollback_line(line.clone(), w);
            assert!(!out.is_empty(), "width {w} must produce rows");
            for (i, row) in out.iter().enumerate() {
                let rw = width_of(&row.text);
                assert!(
                    rw <= w as usize,
                    "width {w} row {i} is {rw} cells, exceeds viewport"
                );
            }
            assert!(
                !out[0].text.trim().is_empty(),
                "width {w} first row not blank"
            );
            assert!(
                out.len() <= 50,
                "width {w} row count bounded (got {})",
                out.len()
            );
        }
    }

    #[test]
    fn width_4_wraps_with_bounded_rows_and_preserved_content() {
        let line = styled_line(&"a".repeat(200));
        let out = wrap_scrollback_line(line, 4);
        assert!(out.len() <= 200, "width 4 bounded row count");
        for (i, row) in out.iter().enumerate() {
            assert!(width_of(&row.text) <= 4, "width 4 row {i} within viewport");
        }
        let joined: String = out.iter().map(|l| l.text.as_str()).collect();
        assert!(
            joined.matches('a').count() >= 100,
            "content preserved at width 4"
        );
    }

    #[test]
    fn three_cell_prefix_continues_indent_at_width_4() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        assert!(out.len() > 1, "must wrap at width 4");
        for row in &out {
            assert!(width_of(&row.text) <= 4, "each row within width 4");
        }
        if let Some(cont) = out.get(1) {
            assert_eq!(
                width_of(cont.text.trim_end_matches('a')),
                3,
                "continuation indent matches the 3-cell prefix"
            );
        }
    }

    #[test]
    fn cjk_at_narrow_widths_respects_viewport_constraint() {
        let cjk = styled_line(&"你".repeat(30));
        // width 0: empty
        assert!(wrap_scrollback_line(cjk.clone(), 0).is_empty());
        // width 1: CJK (width 2) exceeds viewport → substitution marker
        let out1 = wrap_scrollback_line(cjk.clone(), 1);
        assert!(!out1.is_empty(), "width 1 produces rows");
        for row in &out1 {
            assert!(width_of(&row.text) <= 1, "width 1 row within viewport");
        }
        // width 2: CJK fits (2 cells = 2 width)
        let out2 = wrap_scrollback_line(cjk.clone(), 2);
        assert!(!out2.is_empty(), "width 2 produces rows");
        for row in &out2 {
            assert!(width_of(&row.text) <= 2, "width 2 row within viewport");
        }
        // width 3-5: safe
        for w in [3u16, 4, 5] {
            let out = wrap_scrollback_line(cjk.clone(), w);
            assert!(!out.is_empty(), "width {w} produces rows");
            for row in &out {
                assert!(
                    width_of(&row.text) <= w as usize,
                    "width {w} row within viewport"
                );
            }
        }
    }

    #[test]
    fn cjk_width_1_uses_substitution_marker_not_half_char() {
        let cjk = styled_line("你好");
        let out = wrap_scrollback_line(cjk, 1);
        assert!(!out.is_empty());
        for row in &out {
            assert!(width_of(&row.text) <= 1, "width 1 row is 1 cell");
            assert!(
                !row.text.contains('你') && !row.text.contains('好'),
                "CJK scalar not split — substitution used instead"
            );
        }
    }

    #[test]
    fn empty_line_safe_at_all_widths() {
        let empty = styled_line("");
        assert!(
            wrap_scrollback_line(empty.clone(), 0).is_empty(),
            "width 0 empty"
        );
        for w in [1u16, 2, 3, 4, 80] {
            let out = wrap_scrollback_line(empty.clone(), w);
            assert_eq!(out.len(), 1, "width {w} empty line → 1 row");
            assert!(out[0].text.is_empty());
        }
    }

    #[test]
    fn long_line_at_width_2_splits_into_2_cell_rows() {
        let long = styled_line(&"z".repeat(500));
        let out = wrap_scrollback_line(long, 2);
        assert!(out.len() > 1, "width 2 must split a 500-char line");
        for (i, row) in out.iter().enumerate() {
            assert!(width_of(&row.text) <= 2, "width 2 row {i} within viewport");
        }
    }

    #[test]
    fn fill_bearing_line_passes_through_at_positive_width() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("hint")],
            None,
            Some(HistorySegment::raw(" ")),
        );
        for w in [1u16, 3, 4, 80] {
            let out = wrap_scrollback_line(line.clone(), w);
            assert_eq!(out.len(), 1, "fill-bearing line not wrapped at width {w}");
            assert!(out[0].fill.is_some());
        }
        assert!(
            wrap_scrollback_line(line, 0).is_empty(),
            "width 0 skips fill line too"
        );
    }

    #[test]
    fn normal_width_wrapping_still_works() {
        let line = styled_line(&"a".repeat(40));
        let out = wrap_scrollback_line(line, 20);
        assert!(out.len() > 1, "width 20 wraps a 40-char line");
        for row in &out {
            assert!(
                width_of(row.text.trim()) <= 20,
                "each row fits within width 20"
            );
        }
    }

    #[test]
    fn no_prefix_only_continuation_rows() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        for (i, row) in out.iter().enumerate() {
            let trimmed = row.text.trim();
            assert!(
                !trimmed.is_empty(),
                "row {i} is prefix-only (all whitespace)"
            );
        }
    }

    #[test]
    fn builder_to_renderer_chain_respects_viewport_width() {
        let display = crate::tool_display::test_display_with_long_args();
        for w in [1u16, 2, 3, 4] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let rows = prepare_history_rows(built, w);
            assert!(!rows.is_empty(), "width {w} chain produces rows");
            for (i, row) in rows.iter().enumerate() {
                let rw = width_of(&row.text);
                assert!(
                    rw <= w as usize,
                    "width {w}: builder→renderer row {i} is {rw} cells, exceeds viewport"
                );
            }
            assert!(
                rows.iter().any(|r| !r.text.trim().is_empty()),
                "width {w}: at least one row has visible content"
            );
        }
    }

    #[test]
    fn ascii_content_preserved_through_extreme_widths() {
        let args = "abcdefghij".repeat(10);
        let display = crate::tool_display::test_display_with_args(&args);
        for w in [1u16, 2, 3] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let rows = prepare_history_rows(built, w);
            let combined: String = rows
                .iter()
                .flat_map(|r| r.text.chars())
                .filter(|c| c.is_ascii_alphabetic())
                .collect();
            for ch in args.chars() {
                let before = combined.matches(ch).count();
                let expected = args.matches(ch).count();
                assert!(
                    before >= expected,
                    "width {w}: char '{ch}' count {before} < expected {expected}"
                );
            }
        }
    }

    #[test]
    fn physical_row_count_is_bounded_and_nonzero() {
        let display = crate::tool_display::test_display_with_long_args();
        for w in [1u16, 2, 3, 4, 40] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let rows = prepare_history_rows(built, w);
            assert!(!rows.is_empty(), "width {w} produces rows");
            assert!(
                rows.len() <= 500,
                "width {w} row count bounded (got {})",
                rows.len()
            );
        }
    }
}
