use crossterm::style::Color as CColor;

use crate::app::ScrollbackLine;
use crate::inline_terminal::{HistoryAttrs, HistorySegment};
use crate::stream_markdown::MarkdownBlockKind;
use crate::theme::{semantic, to_crossterm_color};

pub(crate) fn render_markdown_segments(
    line: &str,
    block: Option<(&MarkdownBlockKind, usize)>,
) -> Vec<HistorySegment> {
    match block {
        Some((MarkdownBlockKind::CodeFence, _)) => render_code_block_line(line),
        Some((MarkdownBlockKind::Table, row_index)) => render_table_history_line(line, row_index),
        Some((MarkdownBlockKind::List, _)) => render_list_line(line),
        Some((MarkdownBlockKind::Quote, _)) => render_quote_line(line),
        None => render_inline_markdown(line),
    }
}

fn render_code_block_line(line: &str) -> Vec<HistorySegment> {
    let trimmed = line.trim_start();
    let attrs = if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        HistoryAttrs {
            dim: true,
            ..HistoryAttrs::default()
        }
    } else {
        HistoryAttrs::default()
    };
    vec![HistorySegment::styled(
        line,
        to_crossterm_color(semantic::MARKDOWN_CODE),
        attrs,
    )]
}

pub(crate) fn render_mermaid_block(mermaid_src: &str, bg: Option<CColor>) -> Vec<ScrollbackLine> {
    let dim_color = to_crossterm_color(semantic::DIM_TEXT);
    let text_color = to_crossterm_color(semantic::MARKDOWN_CODE);

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mermaid_text::render(mermaid_src)
    })) {
        Ok(Ok(rendered)) => {
            let header = ScrollbackLine::styled(
                vec![HistorySegment::styled(
                    format!("   [mermaid] {}", "─".repeat(40)),
                    dim_color,
                    HistoryAttrs::default(),
                )],
                bg,
            );
            let mut lines = vec![header];
            for text_line in rendered.lines() {
                lines.push(ScrollbackLine::styled(
                    vec![HistorySegment::styled(
                        format!("   {text_line}"),
                        text_color,
                        HistoryAttrs::default(),
                    )],
                    bg,
                ));
            }
            lines
        }
        Ok(Err(_)) | Err(_) => {
            let plain_lines: Vec<Vec<(String, Option<CColor>)>> = mermaid_src
                .lines()
                .map(|l| vec![(l.to_string(), None)])
                .collect();
            build_code_block(&plain_lines, "mermaid", bg)
        }
    }
}

pub(crate) fn render_code_block(block_lines: &[String], bg: Option<CColor>) -> Vec<ScrollbackLine> {
    if block_lines.len() < 3 {
        return block_lines
            .iter()
            .map(|l| ScrollbackLine::styled(render_code_block_line(l), bg))
            .collect();
    }

    let opening = &block_lines[0];
    let lang = opening.trim_start().trim_start_matches(['`', '~']).trim();
    let code_lines = &block_lines[1..block_lines.len() - 1];

    let plain_lines: Vec<Vec<(String, Option<CColor>)>> = code_lines
        .iter()
        .map(|line| vec![(line.clone(), None)])
        .collect();

    build_code_block(&plain_lines, lang, bg)
}

pub(crate) fn build_code_block(
    lines: &[Vec<(String, Option<CColor>)>],
    lang: &str,
    bg: Option<CColor>,
) -> Vec<ScrollbackLine> {
    let dim_color = to_crossterm_color(semantic::DIM_TEXT);
    let line_num_color = to_crossterm_color(semantic::DIM_TEXT);

    let content_max = lines
        .iter()
        .map(|segs| segs.iter().map(|(t, _)| t.len()).sum::<usize>())
        .max()
        .unwrap_or(0);

    let max_width = content_max.max(lang.len() + 2);
    let indent = "   ";
    let mut rendered = Vec::with_capacity(lines.len() + 1);

    let top_line = if lang.is_empty() {
        format!("{}{}", indent, "─".repeat(max_width))
    } else {
        let label = format!("[{}]", lang);
        let remaining = max_width.saturating_sub(label.len());
        format!("{}{}{}", indent, label, "─".repeat(remaining))
    };
    rendered.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            top_line,
            dim_color,
            HistoryAttrs::default(),
        )],
        bg,
    ));

    let num_width = if lines.is_empty() {
        1
    } else {
        (lines.len() as f64).log10().floor() as usize + 1
    };

    for (i, line_segments) in lines.iter().enumerate() {
        let mut segments = Vec::new();

        let num_text = format!("{:>width$}", i + 1, width = num_width);
        segments.push(HistorySegment::styled(
            format!("{}{}", indent, num_text),
            line_num_color,
            HistoryAttrs::default(),
        ));
        segments.push(HistorySegment::styled(
            " │ ".to_string(),
            dim_color,
            HistoryAttrs::default(),
        ));

        if line_segments.is_empty() {
            segments.push(HistorySegment::styled(
                String::new(),
                None,
                HistoryAttrs::default(),
            ));
        } else {
            for (text, color) in line_segments {
                segments.push(HistorySegment::styled(
                    text.clone(),
                    *color,
                    HistoryAttrs::default(),
                ));
            }
        }

        rendered.push(ScrollbackLine::styled(segments, bg));
    }

    let bottom_line = format!("{}{}", indent, "─".repeat(max_width));
    rendered.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            bottom_line,
            dim_color,
            HistoryAttrs::default(),
        )],
        bg,
    ));

    rendered
}

pub(crate) fn render_table_history_line(line: &str, row_index: usize) -> Vec<HistorySegment> {
    let cells = split_table_cells(line);
    if cells.is_empty() {
        return render_inline_markdown(line);
    }

    if row_index == 1 {
        let sep_color = to_crossterm_color(semantic::STATUS_VALUE);
        return vec![HistorySegment::styled(
            cells
                .iter()
                .map(|cell| "─".repeat(cell.len().max(3)))
                .collect::<Vec<_>>()
                .join("\t"),
            sep_color,
            HistoryAttrs::default(),
        )];
    }

    let mut segments = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            segments.push(HistorySegment::raw("\t"));
        }
        let mut cell_segments = render_inline_markdown(cell);
        if row_index == 0 {
            for segment in &mut cell_segments {
                segment.attrs.bold = true;
                if segment.fg.is_none() {
                    segment.fg = to_crossterm_color(semantic::MARKDOWN_TABLE_HEADER);
                }
            }
        }
        segments.extend(cell_segments);
    }
    segments
}

pub(crate) fn render_table_block(lines: &[String]) -> Option<Vec<Vec<HistorySegment>>> {
    if lines.len() < 2 || !is_table_separator_line(&lines[1]) {
        return None;
    }

    let body_rows: Vec<Vec<Vec<HistorySegment>>> = lines
        .iter()
        .enumerate()
        .filter(|(row_index, _)| *row_index != 1)
        .map(|(row_index, line)| {
            split_table_cells(line)
                .into_iter()
                .map(|cell| {
                    let mut segments = render_inline_markdown(&cell);
                    if row_index == 0 {
                        emphasize_table_header(&mut segments);
                    }
                    segments
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let column_count = body_rows.iter().map(Vec::len).max().unwrap_or(0);
    if column_count < 2 {
        return None;
    }

    let mut widths = vec![3usize; column_count];
    for row in &body_rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(history_segments_width(cell));
        }
    }

    let mut rendered = Vec::new();
    rendered.push(table_border_line('╭', '┬', '╮', &widths));

    for (row_index, row) in body_rows.iter().enumerate() {
        if row_index == 1 {
            rendered.push(table_border_line('├', '┼', '┤', &widths));
        }
        rendered.push(table_content_line(row, &widths));
    }
    rendered.push(table_border_line('╰', '┴', '╯', &widths));
    Some(rendered)
}

fn table_border_line(
    left: char,
    separator: char,
    right: char,
    widths: &[usize],
) -> Vec<HistorySegment> {
    let sep_color = to_crossterm_color(semantic::STATUS_VALUE);
    let mut text = String::new();
    text.push(left);
    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            text.push(separator);
        }
        text.push_str(&"─".repeat(width + 2));
    }
    text.push(right);
    vec![HistorySegment::styled(
        text,
        sep_color,
        HistoryAttrs::default(),
    )]
}

fn table_content_line(row: &[Vec<HistorySegment>], widths: &[usize]) -> Vec<HistorySegment> {
    let mut segments = vec![table_border_segment("│")];
    for (i, width) in widths.iter().enumerate() {
        if i > 0 {
            segments.push(table_border_segment("│"));
        }
        segments.push(HistorySegment::raw(" "));
        let cell_segments = row.get(i).cloned().unwrap_or_default();
        let cell_width = history_segments_width(&cell_segments);
        segments.extend(cell_segments);
        if *width > cell_width {
            segments.push(HistorySegment::raw(" ".repeat(width - cell_width)));
        }
        segments.push(HistorySegment::raw(" "));
    }
    segments.push(table_border_segment("│"));
    segments
}

fn table_border_segment(text: impl Into<String>) -> HistorySegment {
    HistorySegment::styled(
        text,
        to_crossterm_color(semantic::STATUS_VALUE),
        HistoryAttrs::default(),
    )
}

fn emphasize_table_header(segments: &mut [HistorySegment]) {
    for segment in segments {
        segment.attrs.bold = true;
        if segment.fg.is_none() {
            segment.fg = to_crossterm_color(semantic::MARKDOWN_TABLE_HEADER);
        }
    }
}

pub(crate) fn history_segments_width(segments: &[HistorySegment]) -> usize {
    segments
        .iter()
        .map(|segment| unicode_width::UnicodeWidthStr::width(segment.text.as_str()))
        .sum()
}

pub(crate) fn append_fill_segment(
    segments: &mut Vec<HistorySegment>,
    fill: HistorySegment,
    target_width: u16,
    trailing_padding: usize,
) {
    let target = target_width as usize;
    let width = history_segments_width(segments);
    let avail = target
        .saturating_sub(width)
        .saturating_sub(trailing_padding);
    if avail == 0 {
        return;
    }

    let fill_width = unicode_width::UnicodeWidthStr::width(fill.text.as_str()).max(1);
    let repeat = avail.div_ceil(fill_width);
    let mut fill_segment = fill;
    fill_segment.text = fill_segment.text.repeat(repeat);
    segments.push(fill_segment);

    if trailing_padding > 0 {
        segments.push(HistorySegment::raw(" ".repeat(trailing_padding)));
    }
}

fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|').trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.split('|').all(|cell| {
        let cell = cell.trim();
        let cell = cell.trim_start_matches(':').trim_end_matches(':');
        cell.len() >= 3 && cell.chars().all(|ch| ch == '-')
    })
}

fn render_list_line(line: &str) -> Vec<HistorySegment> {
    let Some((prefix, body)) = split_list_marker(line) else {
        return render_inline_markdown(line);
    };
    let mut segments = vec![HistorySegment::styled(
        prefix,
        to_crossterm_color(semantic::MARKDOWN_LIST_MARKER),
        HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        },
    )];
    segments.extend(render_inline_markdown(body));
    segments
}

fn render_quote_line(line: &str) -> Vec<HistorySegment> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let rest = rest.strip_prefix('>').unwrap_or(rest);
    let rest = rest.strip_prefix(' ').unwrap_or(rest);
    let mut segments = Vec::new();
    if !indent.is_empty() {
        segments.push(HistorySegment::raw(indent));
    }
    segments.push(HistorySegment::styled(
        "> ",
        to_crossterm_color(semantic::MARKDOWN_QUOTE_MARKER),
        HistoryAttrs {
            bold: true,
            ..HistoryAttrs::default()
        },
    ));
    for mut segment in render_inline_markdown(rest) {
        segment.attrs.dim = true;
        if segment.fg.is_none() {
            segment.fg = to_crossterm_color(semantic::MARKDOWN_QUOTE_TEXT);
        }
        segments.push(segment);
    }
    segments
}

fn render_inline_markdown(line: &str) -> Vec<HistorySegment> {
    if is_horizontal_rule(line) {
        let hr_width = 40usize;
        return vec![horizontal_rule_segment("─".repeat(hr_width))];
    }

    if let Some((indent, marker, heading)) = split_heading(line) {
        let mut segments = Vec::new();
        if !indent.is_empty() {
            segments.push(HistorySegment::raw(indent));
        }
        if heading.is_empty() {
            segments.push(HistorySegment::raw(marker));
        } else {
            for mut segment in parse_inline_delimiters(heading) {
                segment.attrs.bold = true;
                if segment.fg.is_none() {
                    segment.fg = to_crossterm_color(semantic::MARKDOWN_HEADING);
                }
                segments.push(segment);
            }
        }
        return segments;
    }

    parse_inline_delimiters(line)
}

pub(crate) fn horizontal_rule_segment(text: impl Into<String>) -> HistorySegment {
    HistorySegment::styled(
        text,
        to_crossterm_color(semantic::STATUS_VALUE),
        HistoryAttrs::default(),
    )
}

pub(crate) fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let Some(marker) = trimmed.chars().next() else {
        return false;
    };
    matches!(marker, '-' | '*' | '_') && trimmed.chars().all(|ch| ch == marker)
}

fn split_heading(line: &str) -> Option<(&str, &str, &str)> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let marker_len = rest.chars().take_while(|ch| *ch == '#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }
    let marker_end = marker_len;
    let after_marker = rest.get(marker_end..)?;
    if !after_marker.starts_with(' ') {
        return None;
    }
    Some((indent, &rest[..=marker_end], after_marker.trim_start()))
}

fn parse_inline_delimiters(line: &str) -> Vec<HistorySegment> {
    let mut segments = Vec::new();
    let mut plain = String::new();
    let mut rest = line;

    while !rest.is_empty() {
        if let Some(after_tick) = rest.strip_prefix('`')
            && let Some(end) = after_tick.find('`')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (code, after) = after_tick.split_at(end);
            segments.push(HistorySegment::styled(
                code,
                to_crossterm_color(semantic::MARKDOWN_CODE),
                HistoryAttrs::default(),
            ));
            rest = &after[1..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix("**")
            && let Some(end) = after_marker.find("**")
        {
            push_plain_segment(&mut segments, &mut plain);
            let (strong, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                strong,
                to_crossterm_color(semantic::MARKDOWN_TEXT_STRONG),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[2..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix("__")
            && let Some(end) = after_marker.find("__")
        {
            push_plain_segment(&mut segments, &mut plain);
            let (strong, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                strong,
                to_crossterm_color(semantic::MARKDOWN_TEXT_STRONG),
                HistoryAttrs {
                    bold: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[2..];
            continue;
        }

        if let Some(after_bracket) = rest.strip_prefix('[')
            && let Some(label_end) = after_bracket.find("](")
        {
            let (label, after_label) = after_bracket.split_at(label_end);
            let after_url_start = &after_label[2..];
            if let Some(url_end) = after_url_start.find(')') {
                push_plain_segment(&mut segments, &mut plain);
                let (url, after_url) = after_url_start.split_at(url_end);
                segments.push(HistorySegment::styled(
                    label,
                    to_crossterm_color(semantic::MARKDOWN_LINK),
                    HistoryAttrs {
                        underlined: true,
                        ..HistoryAttrs::default()
                    },
                ));
                if !url.is_empty() {
                    segments.push(HistorySegment::styled(
                        format!(" ({url})"),
                        to_crossterm_color(semantic::MARKDOWN_LINK_URL),
                        HistoryAttrs {
                            dim: true,
                            ..HistoryAttrs::default()
                        },
                    ));
                }
                rest = &after_url[1..];
                continue;
            }
        }

        if let Some(after_marker) = rest.strip_prefix('*')
            && !after_marker.starts_with('*')
            && let Some(end) = after_marker.find('*')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (emphasis, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                emphasis,
                to_crossterm_color(semantic::MARKDOWN_TEXT_EMPHASIS),
                HistoryAttrs {
                    italic: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[1..];
            continue;
        }

        if let Some(after_marker) = rest.strip_prefix('_')
            && !after_marker.starts_with('_')
            && let Some(end) = after_marker.find('_')
        {
            push_plain_segment(&mut segments, &mut plain);
            let (emphasis, after) = after_marker.split_at(end);
            segments.push(HistorySegment::styled(
                emphasis,
                to_crossterm_color(semantic::MARKDOWN_TEXT_EMPHASIS),
                HistoryAttrs {
                    italic: true,
                    ..HistoryAttrs::default()
                },
            ));
            rest = &after[1..];
            continue;
        }

        let ch = rest.chars().next().expect("rest is not empty");
        plain.push(ch);
        rest = &rest[ch.len_utf8()..];
    }

    push_plain_segment(&mut segments, &mut plain);
    if segments.is_empty() {
        vec![HistorySegment::raw("")]
    } else {
        segments
    }
}

fn push_plain_segment(segments: &mut Vec<HistorySegment>, plain: &mut String) {
    if !plain.is_empty() {
        segments.push(HistorySegment::raw(std::mem::take(plain)));
    }
}

fn split_list_marker(line: &str) -> Option<(&str, &str)> {
    let trimmed_start = line.len() - line.trim_start().len();
    let rest = &line[trimmed_start..];
    for marker in ["- ", "* ", "+ "] {
        if let Some(body) = rest.strip_prefix(marker) {
            return Some((&line[..trimmed_start + marker.len()], body));
        }
    }
    let dot = rest.find('.')?;
    if dot == 0 || !rest[..dot].chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    rest.get(dot + 1..)
        .and_then(|after| after.starts_with(' ').then_some(()))?;
    let marker_len = dot + 2;
    Some((&line[..trimmed_start + marker_len], &rest[marker_len..]))
}

fn split_table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}
