use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use unicode_width::UnicodeWidthChar;

use crate::theme::semantic;

/// The terminal columns reserved by the composer's `▸ ` prefix.
pub(crate) const COMPOSER_LEFT_PAD: u16 = 3;
/// The terminal column reserved by the composer's right-side block padding.
pub(crate) const COMPOSER_RIGHT_PAD: u16 = 1;

/// Returns the width available after the composer prefix and block padding.
pub(crate) fn composer_content_width(terminal_width: u16) -> u16 {
    terminal_width
        .saturating_sub(COMPOSER_LEFT_PAD)
        .saturating_sub(COMPOSER_RIGHT_PAD)
        .max(1)
}

#[cfg_attr(not(test), expect(dead_code, reason = "used by legacy composer tests"))]
pub(crate) fn input_line_count(buffer: &str) -> u16 {
    buffer.split('\n').count().max(1) as u16
}

/// Counts the visual content rows occupied by `buffer` at the given cell width.
pub(crate) fn input_line_count_with_width(buffer: &str, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }

    buffer.split('\n').fold(0u16, |rows, segment| {
        let (_, segment_rows) = segment.chars().fold((0u16, 1u16), |(used, rows), ch| {
            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if used.saturating_add(char_width) > width && used > 0 {
                (char_width, rows.saturating_add(1))
            } else {
                (used.saturating_add(char_width), rows)
            }
        });
        rows.saturating_add(segment_rows)
    })
}

#[cfg_attr(not(test), expect(dead_code, reason = "used by legacy composer tests"))]
pub(crate) fn cursor_line_col(buffer_before_cursor: &str) -> (u16, u16) {
    let row = buffer_before_cursor.matches('\n').count() as u16;
    let col = buffer_before_cursor
        .rsplit('\n')
        .next()
        .map(unicode_width::UnicodeWidthStr::width)
        .unwrap_or(0) as u16;
    (row, col)
}

/// Returns the visual cursor row and column at the given content width.
pub(crate) fn cursor_line_col_with_width(buffer_before_cursor: &str, width: u16) -> (u16, u16) {
    if width == 0 {
        return (0, 0);
    }

    let (row, col) = buffer_before_cursor
        .chars()
        .fold((0u16, 0u16), |(row, used), ch| {
            if ch == '\n' {
                return (row.saturating_add(1), 0);
            }

            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if used.saturating_add(char_width) > width && used > 0 {
                (row.saturating_add(1), char_width)
            } else {
                (row, used.saturating_add(char_width))
            }
        });

    if col == width {
        (row.saturating_add(1), 0)
    } else {
        (row, col)
    }
}

/// Returns the first visual row shown by a capped composer while keeping its cursor visible.
pub(crate) fn composer_scroll_offset(
    buffer_before_cursor: &str,
    buffer: &str,
    width: u16,
    max_lines: u16,
) -> u16 {
    let content_rows = input_line_count_with_width(buffer, width);
    let cursor_row = cursor_line_col_with_width(buffer_before_cursor, width).0;
    let visible_rows = content_rows.max(cursor_row.saturating_add(1));

    if visible_rows <= max_lines {
        return 0;
    }

    let mut offset = visible_rows.saturating_sub(max_lines);
    if cursor_row < offset {
        offset = cursor_row;
    } else if cursor_row >= offset.saturating_add(max_lines) {
        offset = cursor_row.saturating_sub(max_lines).saturating_add(1);
    }

    offset
}

pub(crate) fn credential_display_text(buffer: &str) -> std::borrow::Cow<'_, str> {
    if buffer.is_empty() {
        "Enter API key…".into()
    } else {
        "•".repeat(buffer.chars().count()).into()
    }
}

pub(crate) fn credential_cursor_col(buffer: &str) -> u16 {
    3u16.saturating_add(buffer.chars().count() as u16)
}

#[cfg(test)]
pub(crate) fn build_input_text(state: &crate::state::TuiState, width: u16) -> Text<'static> {
    build_input_text_with_max_height(state, width, crate::scrollback::MAX_COMPOSER_LINES)
}

/// Build the visible composer window for an explicitly allocated height.
pub(crate) fn build_input_text_with_max_height(
    state: &crate::state::TuiState,
    width: u16,
    max_height: u16,
) -> Text<'static> {
    let buffer = &state.input_buffer;
    let prompt_style = Style::default().fg(semantic::APPROVAL_PROMPT);
    let content_rows = input_line_count_with_width(buffer, width);
    let cursor_byte_pos = state.cursor_byte_pos();
    let (cursor_row, _) = cursor_line_col_with_width(&buffer[..cursor_byte_pos], width);
    let scroll_offset =
        composer_scroll_offset(&buffer[..cursor_byte_pos], buffer, width, max_height);
    let total_rows = content_rows.max(cursor_row.saturating_add(1));
    let visible_rows = total_rows.saturating_sub(scroll_offset).min(max_height) as usize;
    let mut visual_lines = Vec::with_capacity(total_rows as usize);
    let mut current_line = String::new();
    let mut used = 0u16;

    for ch in buffer.chars() {
        if ch == '\n' {
            visual_lines.push(std::mem::take(&mut current_line));
            used = 0;
            continue;
        }

        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if used.saturating_add(char_width) > width && used > 0 {
            visual_lines.push(std::mem::take(&mut current_line));
            used = 0;
        }
        current_line.push(ch);
        used = used.saturating_add(char_width);
    }
    visual_lines.push(current_line);
    visual_lines.resize(total_rows as usize, String::new());

    let mut lines = Vec::with_capacity(visible_rows);
    for (line_index, line) in visual_lines
        .into_iter()
        .skip(scroll_offset as usize)
        .take(visible_rows)
        .enumerate()
    {
        let absolute_line_index = scroll_offset as usize + line_index;
        let prefix = input_prefix_for_line(absolute_line_index);
        let prefix_span = if absolute_line_index == 0 {
            Span::styled(prefix, prompt_style)
        } else {
            Span::raw(prefix)
        };
        lines.push(Line::from(vec![prefix_span, Span::raw(line)]));
    }

    Text::from(lines)
}

pub(crate) fn input_prefix_for_line(line_index: usize) -> &'static str {
    if line_index == 0 { " > " } else { "   " }
}
