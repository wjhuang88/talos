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
pub(crate) struct HistoryPreparation {
    pub ready: Vec<ScrollbackLine>,
    pub deferred: Vec<ScrollbackLine>,
}

fn line_has_unrenderable_scalar(line: &ScrollbackLine, viewport_width: u16) -> bool {
    if viewport_width == 0 {
        return true;
    }
    let w_cap = viewport_width as usize;
    line.segments.iter().any(|seg| {
        seg.text
            .chars()
            .any(|ch| UnicodeWidthChar::width(ch).unwrap_or(0) > w_cap)
    })
}

pub(crate) fn wrap_scrollback_line(line: ScrollbackLine, width: u16) -> Vec<ScrollbackLine> {
    if UnicodeWidthStr::width(line.text.as_str()) <= width as usize {
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
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);

            if used > 0 && used.saturating_add(char_width) > w_cap {
                rows.push(ScrollbackLine::styled(
                    std::mem::take(&mut current),
                    line.bg,
                ));
                used = 0;
                if let Some(prefix) = continuation.as_ref() {
                    let prefix_w = UnicodeWidthStr::width(prefix.text.as_str());
                    if prefix_w.saturating_add(char_width) <= w_cap {
                        used = prefix_w;
                        current.push(prefix.clone());
                    }
                }
            }

            append_styled_char(&mut current, ch, &segment);
            used = used.saturating_add(char_width);
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
) -> HistoryPreparation {
    let mut ready = Vec::new();
    let mut deferred = Vec::new();
    for line in lines {
        if line_has_unrenderable_scalar(&line, viewport_width) {
            deferred.push(line);
        } else if line.fill.is_some() {
            if UnicodeWidthStr::width(line.text.as_str()) > viewport_width as usize {
                deferred.push(line);
            } else {
                ready.push(line);
            }
        } else {
            ready.extend(wrap_scrollback_line(line, viewport_width));
        }
    }
    HistoryPreparation { ready, deferred }
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
    fn width_zero_defers_all_lines() {
        let lines = vec![styled_line(&"x".repeat(50)), styled_line("hello")];
        let prep = prepare_history_rows(lines.clone(), 0);
        assert!(prep.ready.is_empty(), "width 0 must defer everything");
        assert_eq!(prep.deferred.len(), 2, "width 0 defers all original lines");
        assert_eq!(prep.deferred[0].text, lines[0].text);
        assert_eq!(prep.deferred[1].text, lines[1].text);
    }

    #[test]
    fn repeated_unrenderable_flush_does_not_duplicate_or_drop() {
        let original = vec![styled_line("hello"), styled_line("你好")];
        for _ in 0..5 {
            let prep = prepare_history_rows(original.clone(), 0);
            assert_eq!(prep.deferred.len(), 2, "defer count stable");
            assert_eq!(prep.ready.len(), 0);
            assert_eq!(prep.deferred[0].text, "hello");
            assert_eq!(prep.deferred[1].text, "你好");
        }
    }

    #[test]
    fn narrow_ascii_widths_split_into_rows_within_viewport() {
        let line = styled_line(&"x".repeat(50));
        for w in [1u16, 2, 3] {
            let prep = prepare_history_rows(vec![line.clone()], w);
            assert!(prep.deferred.is_empty(), "ASCII width {w} not deferred");
            assert!(!prep.ready.is_empty(), "width {w} produces rows");
            for (i, row) in prep.ready.iter().enumerate() {
                assert!(
                    width_of(&row.text) <= w as usize,
                    "width {w} row {i} is {} cells",
                    width_of(&row.text)
                );
            }
            assert!(prep.ready.len() <= 50, "width {w} bounded");
        }
    }

    #[test]
    fn cjk_at_width_one_is_deferred_without_substitution() {
        let cjk = styled_line("你好");
        let prep = prepare_history_rows(vec![cjk.clone()], 1);
        assert!(prep.ready.is_empty(), "CJK at width 1 must defer");
        assert_eq!(prep.deferred.len(), 1);
        assert_eq!(prep.deferred[0].text, "你好", "original content preserved");
        assert!(
            !prep.deferred[0].text.contains('.') && !prep.deferred[0].text.contains('�'),
            "no substitution markers"
        );
    }

    #[test]
    fn cjk_at_width_two_wraps_losslessly() {
        let cjk = styled_line(&"你好".repeat(20));
        let prep = prepare_history_rows(vec![cjk.clone()], 2);
        assert!(prep.deferred.is_empty(), "CJK at width 2 is renderable");
        assert!(!prep.ready.is_empty(), "width 2 produces rows");
        for (i, row) in prep.ready.iter().enumerate() {
            assert!(width_of(&row.text) <= 2, "width 2 row {i} within viewport");
        }
        let joined: String = prep.ready.iter().map(|r| r.text.as_str()).collect();
        assert!(joined.contains('你'), "CJK preserved in wrapped output");
        assert!(joined.contains('好'), "CJK preserved in wrapped output");
    }

    #[test]
    fn mixed_ascii_cjk_at_width_one_defers_preserving_content() {
        let mixed = styled_line("abc你好xyz");
        let prep = prepare_history_rows(vec![mixed.clone()], 1);
        assert!(prep.ready.is_empty(), "mixed with CJK at width 1 deferred");
        assert_eq!(prep.deferred[0].text, "abc你好xyz");
    }

    #[test]
    fn mixed_ascii_cjk_at_width_two_wraps_losslessly() {
        let mixed = styled_line("abc你好xyz");
        let prep = prepare_history_rows(vec![mixed.clone()], 2);
        assert!(prep.deferred.is_empty(), "width 2 renderable");
        let joined: String = prep.ready.iter().map(|r| r.text.as_str()).collect();
        assert!(joined.contains("abc"), "ASCII prefix preserved");
        assert!(joined.contains("你好"), "CJK preserved");
        assert!(joined.contains("xyz"), "ASCII suffix preserved");
    }

    #[test]
    fn width_four_wraps_with_bounded_rows() {
        let line = styled_line(&"a".repeat(200));
        let prep = prepare_history_rows(vec![line], 4);
        assert!(prep.deferred.is_empty());
        assert!(prep.ready.len() <= 200, "width 4 bounded");
        for (i, row) in prep.ready.iter().enumerate() {
            assert!(width_of(&row.text) <= 4, "width 4 row {i} within viewport");
        }
    }

    #[test]
    fn three_cell_prefix_continuation_at_width_4() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        assert!(out.len() > 1, "must wrap at width 4");
        for row in &out {
            assert!(width_of(&row.text) <= 4, "each row within width 4");
        }
    }

    #[test]
    fn empty_line_safe_at_all_widths() {
        let empty = styled_line("");
        for w in [1u16, 2, 3, 4, 80] {
            let out = wrap_scrollback_line(empty.clone(), w);
            assert_eq!(out.len(), 1, "width {w} empty line → 1 row");
            assert!(out[0].text.is_empty());
        }
        let prep = prepare_history_rows(vec![empty.clone()], 0);
        assert!(prep.ready.is_empty(), "width 0 defers empty too");
        assert_eq!(prep.deferred.len(), 1);
    }

    #[test]
    fn long_ascii_at_width_2_splits() {
        let long = styled_line(&"z".repeat(500));
        let prep = prepare_history_rows(vec![long], 2);
        assert!(prep.deferred.is_empty());
        assert!(prep.ready.len() > 1, "width 2 splits 500-char line");
        for (i, row) in prep.ready.iter().enumerate() {
            assert!(width_of(&row.text) <= 2, "width 2 row {i}");
        }
    }

    #[test]
    fn fill_bearing_fits_passes_through() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("hi")],
            None,
            Some(HistorySegment::raw(" ")),
        );
        let prep = prepare_history_rows(vec![line.clone()], 80);
        assert!(prep.deferred.is_empty(), "fits at width 80");
        assert_eq!(prep.ready.len(), 1);
        assert!(prep.ready[0].fill.is_some(), "fill preserved");
    }

    #[test]
    fn fill_bearing_overflow_defers() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw(&"x".repeat(100))],
            None,
            Some(HistorySegment::raw(" ")),
        );
        let prep = prepare_history_rows(vec![line.clone()], 40);
        assert_eq!(prep.ready.len(), 0, "overflow fill deferred");
        assert_eq!(prep.deferred.len(), 1);
        assert!(
            prep.deferred[0].fill.is_some(),
            "fill preserved in deferred"
        );
    }

    #[test]
    fn normal_width_wrapping_still_works() {
        let line = styled_line(&"a".repeat(40));
        let out = wrap_scrollback_line(line, 20);
        assert!(out.len() > 1, "width 20 wraps 40-char line");
        for row in &out {
            assert!(width_of(row.text.trim()) <= 20, "fits within width 20");
        }
    }

    #[test]
    fn no_prefix_only_continuation_rows() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        for (i, row) in out.iter().enumerate() {
            assert!(!row.text.trim().is_empty(), "row {i} is prefix-only");
        }
    }

    #[test]
    fn builder_to_renderer_chain_respects_viewport_width() {
        let display = crate::tool_display::test_display_with_long_args();
        for w in [1u16, 2, 3, 4] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let prep = prepare_history_rows(built, w);
            assert!(!prep.ready.is_empty(), "width {w} produces ready rows");
            for (i, row) in prep.ready.iter().enumerate() {
                assert!(
                    width_of(&row.text) <= w as usize,
                    "width {w} row {i} exceeds viewport"
                );
            }
        }
    }

    #[test]
    fn ascii_content_preserved_at_extreme_widths() {
        let args = "abcdefghij".repeat(10);
        let display = crate::tool_display::test_display_with_args(&args);
        for w in [1u16, 2, 3] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let prep = prepare_history_rows(built, w);
            let combined: String = prep
                .ready
                .iter()
                .flat_map(|r| r.text.chars())
                .filter(|c| c.is_ascii_alphabetic())
                .collect();
            for ch in args.chars() {
                assert!(
                    combined.matches(ch).count() >= args.matches(ch).count(),
                    "width {w}: char '{ch}' not fully preserved"
                );
            }
        }
    }

    #[test]
    fn physical_row_count_is_bounded_and_nonzero() {
        let display = crate::tool_display::test_display_with_long_args();
        for w in [1u16, 2, 3, 4, 40] {
            let built = crate::tool_display::build_tool_call_scrollback_lines(&display, w);
            let prep = prepare_history_rows(built, w);
            assert!(!prep.ready.is_empty(), "width {w} produces rows");
            assert!(prep.ready.len() <= 500, "width {w} bounded");
        }
    }

    #[test]
    fn deferred_line_preserves_segments_and_style() {
        let line = ScrollbackLine::styled(
            vec![
                HistorySegment::styled(" → ", None, HistoryAttrs::default()),
                HistorySegment::raw("你好"),
            ],
            None,
        );
        let prep = prepare_history_rows(vec![line.clone()], 1);
        assert_eq!(prep.deferred.len(), 1);
        let d = &prep.deferred[0];
        assert_eq!(d.text, line.text, "text identical");
        assert_eq!(
            d.segments.len(),
            line.segments.len(),
            "segment count identical"
        );
        assert_eq!(d.bg, line.bg, "bg identical");
    }
}
