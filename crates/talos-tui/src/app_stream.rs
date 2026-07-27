#[cfg(test)]
use std::io;

use crossterm::style::Color as CColor;
use talos_conversation::MessageSource;
#[cfg(test)]
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

    #[cfg(test)]
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
#[cfg(test)]
pub(crate) struct PreparedLogicalLine {
    pub original: ScrollbackLine,
    pub physical_rows: Vec<ScrollbackLine>,
}

#[cfg(test)]
pub(crate) struct HistoryPreparation {
    pub ready_prefix: Vec<PreparedLogicalLine>,
    pub deferred_suffix: Vec<ScrollbackLine>,
}

#[cfg(test)]
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

#[cfg(test)]
fn line_overflows_viewport(line: &ScrollbackLine, viewport_width: u16) -> bool {
    UnicodeWidthStr::width(line.text.as_str()) > viewport_width as usize
}

#[cfg(test)]
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

#[cfg(test)]
pub(crate) fn prepare_history_rows(
    lines: Vec<ScrollbackLine>,
    viewport_width: u16,
) -> HistoryPreparation {
    let mut ready_prefix = Vec::new();
    let mut deferred_suffix = Vec::new();
    let mut blocked = false;

    for line in lines {
        if blocked
            || line_has_unrenderable_scalar(&line, viewport_width)
            || (line.fill.is_some() && line_overflows_viewport(&line, viewport_width))
        {
            blocked = true;
            deferred_suffix.push(line);
        } else if line.fill.is_some() {
            ready_prefix.push(PreparedLogicalLine {
                original: line.clone(),
                physical_rows: vec![line],
            });
        } else {
            let physical_rows = wrap_scrollback_line(line.clone(), viewport_width);
            ready_prefix.push(PreparedLogicalLine {
                original: line,
                physical_rows,
            });
        }
    }
    HistoryPreparation {
        ready_prefix,
        deferred_suffix,
    }
}

#[cfg(test)]
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

#[cfg(test)]
pub(crate) trait HistoryWriter {
    fn insert_plain(&mut self, text: &str, bg: Option<CColor>) -> io::Result<()>;
    fn insert_styled(&mut self, segments: &[HistorySegment], bg: Option<CColor>) -> io::Result<()>;
}

#[cfg(test)]
pub(crate) fn flush_prepared_with_writer<W: HistoryWriter>(
    prepared: HistoryPreparation,
    writer: &mut W,
) -> (Vec<ScrollbackLine>, io::Result<()>) {
    let mut remaining_ready = prepared.ready_prefix.into_iter();
    let deferred = prepared.deferred_suffix;

    while let Some(logical) = remaining_ready.next() {
        let physical_rows = &logical.physical_rows;
        let mut committed_count = 0usize;

        let result: io::Result<()> = (|| {
            for physical in physical_rows {
                if physical.has_plain_segments_only() {
                    writer.insert_plain(&physical.text, physical.bg)?;
                } else {
                    writer.insert_styled(&physical.segments, physical.bg)?;
                }
                committed_count += 1;
            }
            Ok(())
        })();

        if let Err(err) = result {
            let mut restored = Vec::new();
            if committed_count > 0 {
                restored.extend(physical_rows[committed_count..].iter().cloned());
            } else {
                restored.push(logical.original);
            }
            restored.extend(remaining_ready.map(|item| item.original));
            restored.extend(deferred);
            return (restored, Err(err));
        }
    }

    (Vec::new(), Ok(()))
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

    fn ready_texts(prep: &HistoryPreparation) -> Vec<String> {
        prep.ready_prefix
            .iter()
            .flat_map(|l| l.physical_rows.iter())
            .map(|r| r.text.clone())
            .collect()
    }

    fn deferred_texts(prep: &HistoryPreparation) -> Vec<String> {
        prep.deferred_suffix
            .iter()
            .map(|l| l.text.clone())
            .collect()
    }

    #[test]
    fn width_zero_defers_all_lines() {
        let lines = vec![styled_line(&"x".repeat(50)), styled_line("hello")];
        let prep = prepare_history_rows(lines.clone(), 0);
        assert!(
            prep.ready_prefix.is_empty(),
            "width 0 must defer everything"
        );
        assert_eq!(prep.deferred_suffix.len(), 2);
        assert_eq!(prep.deferred_suffix[0].text, lines[0].text);
        assert_eq!(prep.deferred_suffix[1].text, lines[1].text);
    }

    #[test]
    fn repeated_unrenderable_flush_does_not_duplicate_or_drop() {
        let original = vec![styled_line("hello"), styled_line("你好")];
        for _ in 0..5 {
            let prep = prepare_history_rows(original.clone(), 0);
            assert_eq!(prep.deferred_suffix.len(), 2);
            assert_eq!(prep.ready_prefix.len(), 0);
            assert_eq!(prep.deferred_suffix[0].text, "hello");
            assert_eq!(prep.deferred_suffix[1].text, "你好");
        }
    }

    #[test]
    fn unrenderable_middle_line_defers_entire_suffix() {
        let lines = vec![styled_line("AAA"), styled_line("你好"), styled_line("CCC")];
        let prep = prepare_history_rows(lines, 1);
        let rt = ready_texts(&prep);
        assert!(!rt.is_empty(), "ready prefix exists");
        assert!(
            rt.iter().all(|s| s == "A"),
            "AAA wrapped at width 1 into single-cell rows"
        );
        let dt = deferred_texts(&prep);
        assert_eq!(
            dt,
            vec!["你好", "CCC"],
            "the unrenderable line and everything after it is deferred"
        );
    }

    #[test]
    fn unrenderable_first_line_blocks_later_lines() {
        let lines = vec![styled_line("你好"), styled_line("CCC")];
        let prep = prepare_history_rows(lines, 1);
        assert!(
            prep.ready_prefix.is_empty(),
            "nothing ready when first line blocks"
        );
        let dt = deferred_texts(&prep);
        assert_eq!(dt, vec!["你好", "CCC"]);
    }

    #[test]
    fn repeated_defer_restore_preserves_fifo_exactly_once() {
        let lines = vec![
            styled_line("A"),
            styled_line("你好"),
            styled_line("C"),
            styled_line("世界"),
            styled_line("E"),
        ];
        let mut pending = lines.clone();
        let mut inserted: Vec<String> = Vec::new();

        for w in [1u16, 2, 1, 80] {
            let prep = prepare_history_rows(pending.clone(), w);
            inserted.extend(ready_texts(&prep));
            pending = prep.deferred_suffix;
        }
        let joined = inserted.join("");
        assert!(joined.starts_with("A"), "starts with A");
        assert!(joined.contains("你好"), "CJK pair 1 preserved");
        assert!(joined.contains("C"), "C preserved");
        assert!(joined.contains("世界"), "CJK pair 2 preserved");
        assert!(joined.ends_with("E"), "ends with E");
        assert_eq!(
            joined.chars().filter(|c| *c == 'A').count(),
            1,
            "A exactly once"
        );
        assert_eq!(
            joined.chars().filter(|c| *c == 'C').count(),
            1,
            "C exactly once"
        );
        assert_eq!(
            joined.chars().filter(|c| *c == 'E').count(),
            1,
            "E exactly once"
        );
        assert!(pending.is_empty(), "pending drained after width 80");
    }

    #[test]
    fn narrow_ascii_widths_split_into_rows_within_viewport() {
        let line = styled_line(&"x".repeat(50));
        for w in [1u16, 2, 3] {
            let prep = prepare_history_rows(vec![line.clone()], w);
            assert!(
                prep.deferred_suffix.is_empty(),
                "ASCII width {w} not deferred"
            );
            let rt = ready_texts(&prep);
            assert!(!rt.is_empty(), "width {w} produces rows");
            for (i, t) in rt.iter().enumerate() {
                assert!(
                    width_of(t) <= w as usize,
                    "width {w} row {i} is {} cells",
                    width_of(t)
                );
            }
        }
    }

    #[test]
    fn cjk_at_width_one_is_deferred_without_substitution() {
        let cjk = styled_line("你好");
        let prep = prepare_history_rows(vec![cjk.clone()], 1);
        assert!(prep.ready_prefix.is_empty(), "CJK at width 1 must defer");
        assert_eq!(prep.deferred_suffix.len(), 1);
        assert_eq!(prep.deferred_suffix[0].text, "你好");
        assert!(
            !prep.deferred_suffix[0].text.contains('.')
                && !prep.deferred_suffix[0].text.contains('\u{fffd}'),
            "no substitution markers"
        );
    }

    #[test]
    fn cjk_at_width_two_wraps_losslessly() {
        let cjk = styled_line(&"你好".repeat(20));
        let prep = prepare_history_rows(vec![cjk.clone()], 2);
        assert!(prep.deferred_suffix.is_empty());
        let rt = ready_texts(&prep);
        assert!(!rt.is_empty());
        for t in &rt {
            assert!(width_of(t) <= 2, "width 2 row within viewport");
        }
        let joined: String = rt.join("");
        assert!(joined.contains('你') && joined.contains('好'));
    }

    #[test]
    fn mixed_ascii_cjk_at_width_one_defers_preserving_content() {
        let mixed = styled_line("abc你好xyz");
        let prep = prepare_history_rows(vec![mixed.clone()], 1);
        assert!(prep.ready_prefix.is_empty());
        assert_eq!(prep.deferred_suffix[0].text, "abc你好xyz");
    }

    #[test]
    fn mixed_ascii_cjk_at_width_two_wraps_losslessly() {
        let mixed = styled_line("abc你好xyz");
        let prep = prepare_history_rows(vec![mixed.clone()], 2);
        assert!(prep.deferred_suffix.is_empty());
        let joined: String = ready_texts(&prep).join("");
        assert!(joined.contains("abc") && joined.contains("你好") && joined.contains("xyz"));
    }

    #[test]
    fn width_four_wraps_with_bounded_rows() {
        let line = styled_line(&"a".repeat(200));
        let prep = prepare_history_rows(vec![line], 4);
        assert!(prep.deferred_suffix.is_empty());
        let rt = ready_texts(&prep);
        assert!(rt.len() <= 200);
        for t in &rt {
            assert!(width_of(t) <= 4);
        }
    }

    #[test]
    fn three_cell_prefix_continuation_at_width_4() {
        let line = line_with_prefix(" → ", &"a".repeat(40));
        let out = wrap_scrollback_line(line, 4);
        assert!(out.len() > 1);
        for row in &out {
            assert!(width_of(&row.text) <= 4);
        }
    }

    #[test]
    fn empty_line_safe_at_all_widths() {
        let empty = styled_line("");
        for w in [1u16, 2, 3, 4, 80] {
            let out = wrap_scrollback_line(empty.clone(), w);
            assert_eq!(out.len(), 1);
            assert!(out[0].text.is_empty());
        }
        let prep = prepare_history_rows(vec![empty], 0);
        assert!(prep.ready_prefix.is_empty());
        assert_eq!(prep.deferred_suffix.len(), 1);
    }

    #[test]
    fn long_ascii_at_width_2_splits() {
        let long = styled_line(&"z".repeat(500));
        let prep = prepare_history_rows(vec![long], 2);
        assert!(prep.deferred_suffix.is_empty());
        let rt = ready_texts(&prep);
        assert!(rt.len() > 1);
        for t in &rt {
            assert!(width_of(t) <= 2);
        }
    }

    #[test]
    fn fill_bearing_fits_passes_through() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("hi")],
            None,
            Some(HistorySegment::raw(" ")),
        );
        let prep = prepare_history_rows(vec![line], 80);
        assert!(prep.deferred_suffix.is_empty());
        assert_eq!(prep.ready_prefix.len(), 1);
        assert!(prep.ready_prefix[0].physical_rows[0].fill.is_some());
    }

    #[test]
    fn fill_bearing_overflow_defers() {
        let line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("x".repeat(100))],
            None,
            Some(HistorySegment::raw(" ")),
        );
        let prep = prepare_history_rows(vec![line], 40);
        assert!(prep.ready_prefix.is_empty());
        assert_eq!(prep.deferred_suffix.len(), 1);
        assert!(prep.deferred_suffix[0].fill.is_some());
    }

    #[test]
    fn normal_width_wrapping_still_works() {
        let line = styled_line(&"a".repeat(40));
        let out = wrap_scrollback_line(line, 20);
        assert!(out.len() > 1);
        for row in &out {
            assert!(width_of(row.text.trim()) <= 20);
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
            let rt = ready_texts(&prep);
            assert!(!rt.is_empty(), "width {w} produces ready rows");
            for (i, t) in rt.iter().enumerate() {
                assert!(
                    width_of(t) <= w as usize,
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
            let combined: String = ready_texts(&prep)
                .into_iter()
                .flat_map(|s: String| s.chars().collect::<Vec<char>>())
                .filter(|c| c.is_ascii_alphabetic())
                .collect();
            for ch in args.chars() {
                assert!(
                    combined.matches(ch).count() >= args.matches(ch).count(),
                    "width {w}: char '{ch}' not preserved"
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
            let rt = ready_texts(&prep);
            assert!(!rt.is_empty(), "width {w} produces rows");
            assert!(rt.len() <= 500, "width {w} bounded");
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
        assert_eq!(prep.deferred_suffix.len(), 1);
        let d = &prep.deferred_suffix[0];
        assert_eq!(d.text, line.text);
        assert_eq!(d.segments.len(), line.segments.len());
        assert_eq!(d.bg, line.bg);
    }

    // --- Failure recovery tests using flush_prepared_with_writer ---

    struct MockWriter {
        inserted: Vec<String>,
        fail_on_nth: Option<usize>,
        call_count: usize,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                inserted: Vec::new(),
                fail_on_nth: None,
                call_count: 0,
            }
        }
        fn fail_on(mut self, n: usize) -> Self {
            self.fail_on_nth = Some(n);
            self
        }
    }

    impl HistoryWriter for MockWriter {
        fn insert_plain(&mut self, text: &str, _bg: Option<CColor>) -> io::Result<()> {
            self.call_count += 1;
            if self.fail_on_nth == Some(self.call_count) {
                return Err(io::Error::other("mock failure"));
            }
            self.inserted.push(text.to_string());
            Ok(())
        }
        fn insert_styled(
            &mut self,
            segments: &[HistorySegment],
            _bg: Option<CColor>,
        ) -> io::Result<()> {
            self.call_count += 1;
            if self.fail_on_nth == Some(self.call_count) {
                return Err(io::Error::other("mock failure"));
            }
            let text: String = segments.iter().map(|s| s.text.as_str()).collect();
            self.inserted.push(text);
            Ok(())
        }
    }

    #[test]
    fn plain_insert_failure_restores_uncommitted_suffix() {
        let lines = vec![styled_line("A"), styled_line("B"), styled_line("C")];
        let prep = prepare_history_rows(lines, 80);
        assert!(prep.deferred_suffix.is_empty());

        let mut writer = MockWriter::new().fail_on(2);
        let (restored, result) = flush_prepared_with_writer(prep, &mut writer);

        assert!(result.is_err());
        assert_eq!(writer.inserted, vec!["A"], "A was committed, B failed");
        assert_eq!(
            restored.iter().map(|l| l.text.clone()).collect::<Vec<_>>(),
            vec!["B", "C"],
            "B (failed) and C (uncommitted) restored in FIFO order"
        );
    }

    #[test]
    fn styled_insert_failure_restores_uncommitted_suffix() {
        let styled = ScrollbackLine::styled(
            vec![HistorySegment::styled(
                "cmd",
                Some(CColor::Green),
                HistoryAttrs::default(),
            )],
            None,
        );
        let plain = styled_line("plain_line");
        let lines = vec![styled, plain];
        let prep = prepare_history_rows(lines, 80);

        let mut writer = MockWriter::new().fail_on(1);
        let (restored, result) = flush_prepared_with_writer(prep, &mut writer);

        assert!(result.is_err());
        assert!(writer.inserted.is_empty(), "nothing committed");
        assert_eq!(restored.len(), 2, "both logical lines restored");
        assert_eq!(restored[0].text, "cmd");
        assert_eq!(restored[1].text, "plain_line");
    }

    #[test]
    fn retry_after_failure_is_exactly_once() {
        let lines = vec![styled_line("A"), styled_line("B"), styled_line("C")];
        let prep1 = prepare_history_rows(lines.clone(), 80);

        let mut writer1 = MockWriter::new().fail_on(2);
        let (pending, result1) = flush_prepared_with_writer(prep1, &mut writer1);
        assert!(result1.is_err());
        assert_eq!(writer1.inserted, vec!["A"]);

        let prep2 = prepare_history_rows(pending.clone(), 80);
        let mut writer2 = MockWriter::new();
        let (pending2, result2) = flush_prepared_with_writer(prep2, &mut writer2);
        assert!(result2.is_ok());
        assert_eq!(
            writer2.inserted,
            vec!["B", "C"],
            "B and C inserted on retry"
        );
        assert!(pending2.is_empty(), "nothing left");

        let mut writer3 = MockWriter::new();
        let prep3 = prepare_history_rows(pending2, 80);
        let (_, result3) = flush_prepared_with_writer(prep3, &mut writer3);
        assert!(result3.is_ok());
        assert!(
            writer3.inserted.is_empty(),
            "third flush: nothing to insert"
        );

        let mut all = writer1.inserted.clone();
        all.extend(writer2.inserted.iter().cloned());
        assert_eq!(
            all,
            vec!["A", "B", "C"],
            "total: each line exactly once across retries"
        );
    }

    #[test]
    fn partial_physical_row_failure_does_not_duplicate_committed_rows() {
        let long_text = "a".repeat(200);
        let long = styled_line(&long_text);
        let next = styled_line("after");
        let prep = prepare_history_rows(vec![long.clone(), next.clone()], 4);

        assert!(!prep.ready_prefix.is_empty());
        let physical_count_0 = prep.ready_prefix[0].physical_rows.len();
        assert!(
            physical_count_0 >= 2,
            "long line wraps to many physical rows at width 4"
        );

        let fail_at = 2;
        let mut writer = MockWriter::new().fail_on(fail_at);
        let (restored, result) = flush_prepared_with_writer(
            prepare_history_rows(vec![long.clone(), next.clone()], 4),
            &mut writer,
        );

        assert!(result.is_err());
        assert_eq!(
            writer.inserted.len(),
            1,
            "exactly one physical row committed before failure"
        );

        let committed_text = &writer.inserted[0];
        for r in &restored {
            assert!(
                !r.text.contains(committed_text.as_str())
                    || r.text == committed_text.as_str()
                    || r.text == "after",
                "restored row should not contain committed text as a prefix"
            );
        }
        let restored_texts: Vec<String> = restored.iter().map(|l| l.text.clone()).collect();
        assert!(
            restored_texts.iter().any(|t| t == "after"),
            "uncommitted logical line restored"
        );
        let committed_a_count = writer
            .inserted
            .iter()
            .map(|t| t.matches('a').count())
            .sum::<usize>();
        let restored_a_count = restored_texts
            .iter()
            .filter(|t| *t != "after")
            .map(|t| t.matches('a').count())
            .sum::<usize>();
        assert_eq!(
            committed_a_count + restored_a_count,
            long_text.matches('a').count(),
            "total 'a' count across committed + restored equals original"
        );
    }

    #[test]
    fn empty_physical_row_is_preserved_on_failure_recovery() {
        let logical = PreparedLogicalLine {
            original: styled_line("ABC"),
            physical_rows: vec![styled_line("A"), styled_line(""), styled_line("B")],
        };
        let prepared = HistoryPreparation {
            ready_prefix: vec![logical],
            deferred_suffix: vec![],
        };
        let mut writer = MockWriter::new().fail_on(2);
        let (restored, result) = flush_prepared_with_writer(prepared, &mut writer);
        assert!(result.is_err());
        assert_eq!(writer.inserted, vec!["A"]);
        assert_eq!(
            restored.iter().map(|l| l.text.clone()).collect::<Vec<_>>(),
            vec!["", "B"],
            "empty physical row must be preserved without filtering"
        );
    }

    #[test]
    fn fill_only_row_is_preserved_on_failure_recovery() {
        let fill_line = ScrollbackLine::styled_with_fill(
            vec![HistorySegment::raw("")],
            Some(CColor::Blue),
            Some(HistorySegment::raw("─")),
        );
        let logical = PreparedLogicalLine {
            original: fill_line.clone(),
            physical_rows: vec![styled_line("first"), fill_line.clone()],
        };
        let prepared = HistoryPreparation {
            ready_prefix: vec![logical],
            deferred_suffix: vec![],
        };
        let mut writer = MockWriter::new().fail_on(2);
        let (restored, result) = flush_prepared_with_writer(prepared, &mut writer);
        assert!(result.is_err());
        assert_eq!(writer.inserted.len(), 1);
        assert_eq!(restored.len(), 1, "fill-only row preserved");
        assert_eq!(restored[0].text, "");
        assert_eq!(restored[0].fill.as_ref().unwrap().text, "─");
        assert_eq!(restored[0].bg, Some(CColor::Blue));
    }

    #[test]
    fn styled_empty_row_is_preserved_on_failure_recovery() {
        let styled_empty = ScrollbackLine::styled(
            vec![HistorySegment::styled(
                "",
                Some(CColor::Red),
                HistoryAttrs {
                    bold: true,
                    ..Default::default()
                },
            )],
            Some(CColor::Green),
        );
        let logical = PreparedLogicalLine {
            original: styled_empty.clone(),
            physical_rows: vec![styled_empty],
        };
        let prepared = HistoryPreparation {
            ready_prefix: vec![logical],
            deferred_suffix: vec![],
        };
        let mut writer = MockWriter::new().fail_on(1);
        let (restored, result) = flush_prepared_with_writer(prepared, &mut writer);
        assert!(result.is_err());
        assert_eq!(restored.len(), 1, "styled-empty row not filtered");
        assert_eq!(restored[0].text, "");
        assert_eq!(restored[0].bg, Some(CColor::Green));
        assert_eq!(restored[0].segments.len(), 1);
        assert!(restored[0].segments[0].fg == Some(CColor::Red));
        assert!(restored[0].segments[0].attrs.bold);
    }

    #[test]
    fn retry_with_empty_semantic_row_is_exactly_once() {
        let logical = PreparedLogicalLine {
            original: styled_line("A__B"),
            physical_rows: vec![styled_line("A"), styled_line(""), styled_line("B")],
        };
        let prepared1 = HistoryPreparation {
            ready_prefix: vec![logical],
            deferred_suffix: vec![],
        };
        let mut w1 = MockWriter::new().fail_on(2);
        let (pending, r1) = flush_prepared_with_writer(prepared1, &mut w1);
        assert!(r1.is_err());
        assert_eq!(w1.inserted, vec!["A"]);

        let prepared2 = prepare_history_rows(pending.clone(), 80);
        let mut w2 = MockWriter::new();
        let (pending2, r2) = flush_prepared_with_writer(prepared2, &mut w2);
        assert!(r2.is_ok());

        let prepared3 = prepare_history_rows(pending2, 80);
        let mut w3 = MockWriter::new();
        let (_, r3) = flush_prepared_with_writer(prepared3, &mut w3);
        assert!(r3.is_ok());
        assert!(w3.inserted.is_empty(), "third flush: nothing");

        let mut all = w1.inserted.clone();
        all.extend(w2.inserted.iter().cloned());
        assert_eq!(
            all,
            vec!["A", "", "B"],
            "exact FIFO order, empty row preserved, each once"
        );
    }

    #[test]
    fn recovered_physical_suffix_can_rewrap_at_narrower_width() {
        let long = styled_line(&"a".repeat(40));
        let prep = prepare_history_rows(vec![long], 4);
        let physical_count = prep.ready_prefix[0].physical_rows.len();
        assert!(physical_count >= 2);

        let mut w1 = MockWriter::new().fail_on(2);
        let (restored, r1) = flush_prepared_with_writer(
            prepare_history_rows(vec![styled_line(&"a".repeat(40))], 4),
            &mut w1,
        );
        assert!(r1.is_err());
        assert_eq!(w1.inserted.len(), 1, "one row committed");

        let prep2 = prepare_history_rows(restored, 2);
        let mut w2 = MockWriter::new();
        let (_restored2, r2) = flush_prepared_with_writer(prep2, &mut w2);
        assert!(r2.is_ok());

        let committed = w1.inserted.iter().map(|s| s.chars().count()).sum::<usize>();
        let retry_committed = w2.inserted.iter().map(|s| s.chars().count()).sum::<usize>();
        assert_eq!(
            committed + retry_committed,
            40,
            "total 'a' count across original commit + retry = original"
        );
        for t in &w2.inserted {
            assert!(width_of(t) <= 2, "rewrapped row within width 2");
        }
    }
}
