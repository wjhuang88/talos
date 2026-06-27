use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use crate::theme::semantic;

pub(crate) fn input_line_count(buffer: &str) -> u16 {
    buffer.split('\n').count().max(1) as u16
}

pub(crate) fn cursor_line_col(buffer_before_cursor: &str) -> (u16, u16) {
    let row = buffer_before_cursor.matches('\n').count() as u16;
    let col = buffer_before_cursor
        .rsplit('\n')
        .next()
        .map(unicode_width::UnicodeWidthStr::width)
        .unwrap_or(0) as u16;
    (row, col)
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

pub(crate) fn build_input_text(state: &crate::state::TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let prompt_style = Style::default().fg(semantic::APPROVAL_PROMPT);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut line_index = 0usize;
    let mut spans = vec![Span::styled(" > ", prompt_style)];

    for ch in buffer.chars() {
        if ch == '\n' {
            lines.push(Line::from(spans));
            line_index += 1;
            spans = vec![Span::raw(input_prefix_for_line(line_index))];
        } else {
            spans.push(Span::raw(ch.to_string()));
        }
    }
    lines.push(Line::from(spans));

    Text::from(lines)
}

pub(crate) fn input_prefix_for_line(line_index: usize) -> &'static str {
    if line_index == 0 { " > " } else { "   " }
}
