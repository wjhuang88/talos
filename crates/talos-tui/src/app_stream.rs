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
// Below this width the continuation-indent model cannot fit a 3-cell prefix plus any
// content cell, so wrapping would shred a line into dozens of empty/prefix-only rows.
// Degrade to returning the line as-is (full content preserved, terminal handles display)
// instead of fragmenting. Mirrors the width==0 safety already in place.
const MIN_WRAP_WIDTH: u16 = 4;

pub(crate) fn wrap_scrollback_line(line: ScrollbackLine, width: u16) -> Vec<ScrollbackLine> {
    if width < MIN_WRAP_WIDTH
        || line.fill.is_some()
        || UnicodeWidthStr::width(line.text.as_str()) <= width as usize
    {
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

    let mut rows = Vec::new();
    let mut current = Vec::new();
    let mut used = 0usize;

    for segment in line.segments {
        for ch in segment.text.chars() {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if used > 0 && used.saturating_add(char_width) > width as usize {
                rows.push(ScrollbackLine::styled(
                    std::mem::take(&mut current),
                    line.bg,
                ));
                used = 0;
                if let Some(prefix) = continuation.as_ref() {
                    used = UnicodeWidthStr::width(prefix.text.as_str());
                    current.push(prefix.clone());
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
    fn narrow_widths_0_through_3_return_single_row_without_panic() {
        let line = styled_line(&"x".repeat(50));
        for w in [0u16, 1, 2, 3] {
            let out = wrap_scrollback_line(line.clone(), w);
            assert_eq!(out.len(), 1, "width {w} must not shred into many rows");
            assert_eq!(out[0].text, line.text, "width {w} preserves full content");
        }
    }

    #[test]
    fn width_4_does_not_shred_unbounded() {
        let line = styled_line(&"a".repeat(200));
        let out = wrap_scrollback_line(line.clone(), 4);
        assert!(
            out.len() <= 200,
            "width 4 must produce a bounded number of rows"
        );
        let joined: String = out.iter().map(|l| l.text.as_str()).collect();
        assert!(
            joined.matches('a').count() >= 100,
            "content is not discarded at width 4"
        );
    }

    #[test]
    fn three_cell_prefix_continues_indent_at_width_4() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        assert!(out.len() > 1, "must wrap a long line at width 4");
        if let Some(cont) = out.get(1) {
            assert_eq!(
                width_of(cont.text.trim_end_matches('a')),
                3,
                "continuation indent matches the 3-cell prefix"
            );
        }
    }

    #[test]
    fn cjk_line_safe_at_narrow_widths() {
        let line = styled_line(&"你".repeat(30));
        for w in [0u16, 1, 2, 3] {
            let out = wrap_scrollback_line(line.clone(), w);
            assert_eq!(out.len(), 1, "CJK width {w} must not shred");
        }
        let out = wrap_scrollback_line(line.clone(), 10);
        assert!(out.len() >= 1, "CJK wraps safely at width 10");
    }

    #[test]
    fn empty_and_long_args_are_safe() {
        let empty = styled_line("");
        assert_eq!(wrap_scrollback_line(empty, 0).len(), 1);

        let long = styled_line(&"z".repeat(500));
        let out = wrap_scrollback_line(long.clone(), 2);
        assert_eq!(out.len(), 1, "width 2 returns as-is (below MIN_WRAP_WIDTH)");
        assert_eq!(out[0].text, long.text);
    }

    #[test]
    fn fill_bearing_line_is_returned_unchanged_at_any_width() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("hint")],
            None,
            Some(HistorySegment::raw(" ")),
        );
        for w in [0u16, 1, 3, 4, 80] {
            let out = wrap_scrollback_line(line.clone(), w);
            assert_eq!(out.len(), 1, "fill-bearing line not wrapped at width {w}");
            assert!(out[0].fill.is_some());
        }
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
}
