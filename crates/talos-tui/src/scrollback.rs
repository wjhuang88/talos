use crossterm::style::Color as CColor;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph},
};
use talos_conversation::MessageSource;
use talos_core::message::Message;

use crate::app::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState};
use crate::inline_terminal::{InlineFrame, ViewportComponent};
use crate::stream_markdown::HoldStatus;
use crate::theme::{semantic, to_crossterm_color};

#[cfg(test)]
pub(crate) use crate::scrollback_input::build_input_text;
#[cfg(test)]
pub(crate) use crate::scrollback_input::cursor_line_col;
#[cfg(test)]
pub(crate) use crate::scrollback_input::input_line_count;
pub(crate) use crate::scrollback_input::{
    COMPOSER_LEFT_PAD, COMPOSER_RIGHT_PAD, composer_content_width, composer_scroll_offset,
    credential_cursor_col, credential_display_text, cursor_line_col_with_width,
    input_line_count_with_width,
};
#[cfg(test)]
pub(crate) use crate::scrollback_markdown::history_segments_width;
pub(crate) use crate::scrollback_markdown::{
    append_fill_segment, build_code_block, horizontal_rule_segment, is_horizontal_rule,
    render_code_block, render_markdown_segments, render_mermaid_block, render_table_block,
    render_table_history_line,
};
pub(crate) use crate::scrollback_status::build_status_text;
#[cfg(test)]
pub(crate) use crate::scrollback_status::truncate_str;

pub(crate) struct PreviewComponent<'a> {
    pub(crate) padding: &'a str,
    pub(crate) text: &'a str,
    pub(crate) spinner_color: Option<Color>,
    pub(crate) text_color: Option<Color>,
    pub(crate) thinking_label_frame: Option<usize>,
}

impl ViewportComponent for PreviewComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let line = self.text.split('\n').next_back().unwrap_or("");
        let text_color = self.text_color.unwrap_or(semantic::PREVIEW_FG);
        if let Some(color) = self.spinner_color {
            let full = format!("{}{}", self.padding, line);
            let display = truncate_end_to_width(&full, area.width);
            let padding_len = self.padding.chars().count();
            let (pad_part, text_part) = display.split_at(
                display
                    .char_indices()
                    .nth(padding_len)
                    .map(|(i, _)| i)
                    .unwrap_or(display.len()),
            );
            frame.render_widget(
                Paragraph::new(Line::from(preview_line_spans(
                    pad_part,
                    text_part,
                    Some(color),
                    text_color,
                    self.thinking_label_frame,
                ))),
                area,
            );
        } else {
            let full = format!("{}{}", self.padding, line);
            let display = truncate_end_to_width(&full, area.width);
            frame.render_widget(
                Paragraph::new(Line::from(preview_line_spans(
                    "",
                    &display,
                    None,
                    text_color,
                    self.thinking_label_frame,
                ))),
                area,
            );
        }
    }
}

pub(crate) fn preview_line_spans<'a>(
    pad_part: &'a str,
    text_part: &'a str,
    padding_color: Option<Color>,
    text_color: Color,
    thinking_label_frame: Option<usize>,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    if !pad_part.is_empty() {
        spans.push(Span::styled(
            pad_part.to_string(),
            Style::default().fg(padding_color.unwrap_or(text_color)),
        ));
    }

    if let Some(frame) = thinking_label_frame
        && let Some(rest) = text_part.strip_prefix("thinking")
    {
        spans.extend(thinking_ripple_spans(frame));
        if !rest.is_empty() {
            spans.push(Span::styled(
                rest.to_string(),
                Style::default().fg(text_color),
            ));
        }
        return spans;
    }

    spans.push(Span::styled(
        text_part.to_string(),
        Style::default().fg(text_color),
    ));
    spans
}

pub(crate) fn animated_hold_preview_text(status: &HoldStatus, frame: usize) -> String {
    let base = status.preview_text().trim_end_matches('.');
    let dots = match (frame / 2) % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    };
    format!("{base}{dots}")
}

pub(crate) fn idle_processing_preview_text(frame: usize) -> &'static str {
    match (frame / 2) % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    }
}

pub(crate) fn hold_preview_color(frame: usize) -> Color {
    semantic::HOLD_PREVIEW[(frame / 2) % semantic::HOLD_PREVIEW.len()]
}

pub(crate) fn preview_spinner_padding(processing_frame: usize) -> (String, usize) {
    let n = SPINNER_FRAMES.len();
    let frame_idx = processing_frame % n;
    (format!(" {} ", SPINNER_FRAMES[frame_idx]), frame_idx)
}

fn thinking_ripple_spans(frame: usize) -> [Span<'static>; 3] {
    const LABEL: &str = "thinking";
    const ACTIVE_WIDTHS: [usize; 4] = [2, 4, 6, 4];

    let active_width = ACTIVE_WIDTHS[frame % ACTIVE_WIDTHS.len()];
    let left_width = (LABEL.len() - active_width) / 2;
    let right_start = left_width + active_width;
    let secondary = Style::default()
        .fg(semantic::THINKING_RIPPLE_SECONDARY)
        .add_modifier(Modifier::BOLD);
    let primary = Style::default()
        .fg(semantic::THINKING_RIPPLE_PRIMARY)
        .add_modifier(Modifier::BOLD);

    [
        Span::styled(LABEL[..left_width].to_string(), secondary),
        Span::styled(LABEL[left_width..right_start].to_string(), primary),
        Span::styled(LABEL[right_start..].to_string(), secondary),
    ]
}

pub(crate) struct QueuePreviewComponent<'a> {
    pub(crate) snapshot: Option<&'a talos_conversation::SteeringQueueSnapshot>,
    pub(crate) followup_count: usize,
    pub(crate) max_rows: u16,
}

pub(crate) struct QueuePlan {
    pub(crate) total_rows: u16,
    pub(crate) entries_to_show: usize,
    pub(crate) hidden_count: usize,
    pub(crate) show_summary: bool,
    pub(crate) show_followup: bool,
}

/// Compressed layout budget for constrained terminal heights.
/// Returned by `compress_layout` to guide component construction.
pub(crate) struct CompressedLayout {
    pub(crate) panel_max_height: u16,
    pub(crate) queue_max_rows: u16,
    pub(crate) input_max_height: u16,
}

/// Allocate the height remaining after fixed viewport components.
///
/// Priority: modal panels (approval/credential/slash) > composer (minimum one row
/// whenever the budget permits) > queue preview. The returned allocations always
/// sum to no more than `content_budget`.
pub(crate) fn compress_layout(
    content_budget: u16,
    panel_natural: u16,
    composer_natural: u16,
    queue_natural: u16,
) -> CompressedLayout {
    let composer_floor = u16::from(composer_natural > 0 && content_budget > 0);
    let panel_max_height = panel_natural.min(content_budget.saturating_sub(composer_floor));
    let after_panel = content_budget.saturating_sub(panel_max_height);
    let input_max_height = composer_natural.min(after_panel);
    let queue_max_rows = queue_natural.min(after_panel.saturating_sub(input_max_height));

    CompressedLayout {
        panel_max_height,
        queue_max_rows,
        input_max_height,
    }
}

impl QueuePreviewComponent<'_> {
    pub(crate) fn plan(&self) -> QueuePlan {
        let steering_total = self.snapshot.map(|s| s.total_count).unwrap_or(0);
        let total = steering_total + self.followup_count;
        if total == 0 || self.max_rows == 0 {
            return QueuePlan {
                total_rows: 0,
                entries_to_show: 0,
                hidden_count: 0,
                show_summary: false,
                show_followup: false,
            };
        }

        let max_rows = self.max_rows.min(6);
        let content_budget = ((max_rows - 1) as usize).min(5);
        let available = self.snapshot.map(|s| s.entries.len()).unwrap_or(0);

        let entries_if_full = available.min(content_budget);
        let would_hide = steering_total.saturating_sub(entries_if_full);
        let reserve_summary = would_hide > 0;
        let entry_budget = if reserve_summary {
            content_budget.saturating_sub(1)
        } else {
            content_budget
        };

        let entries_to_show = available.min(entry_budget);
        let hidden_count = steering_total.saturating_sub(entries_to_show);
        let show_summary = hidden_count > 0;

        let remaining = content_budget
            .saturating_sub(entries_to_show)
            .saturating_sub(if show_summary { 1 } else { 0 });
        let show_followup = self.followup_count > 0 && remaining > 0;

        let total_rows = 1
            + entries_to_show as u16
            + if show_summary { 1 } else { 0 }
            + if show_followup { 1 } else { 0 };

        QueuePlan {
            total_rows,
            entries_to_show,
            hidden_count,
            show_summary,
            show_followup,
        }
    }
}

impl ViewportComponent for QueuePreviewComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        self.plan().total_rows
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let plan = self.plan();
        if plan.total_rows == 0 {
            return;
        }

        let dim = Style::default().fg(semantic::DIM_TEXT);
        let steering_total = self.snapshot.map(|s| s.total_count).unwrap_or(0);
        let total = steering_total + self.followup_count;

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(" ", dim),
            Span::styled(
                format!(
                    "{} queued input{}",
                    total,
                    if total == 1 { "" } else { "s" }
                ),
                dim,
            ),
            Span::styled(" (will send after current turn)", dim),
        ]));

        let max_width = (area.width as usize).saturating_sub(4).max(1);

        if let Some(snap) = &self.snapshot {
            for entry in snap.entries.iter().take(plan.entries_to_show) {
                let normalized = normalize_single_line(&entry.text);
                let suffix = if entry.truncated { " ⚠" } else { "" };
                let suffix_width = if entry.truncated { 2 } else { 0 }; // " ⚠" = 2 display cols
                let text_budget = max_width.saturating_sub(suffix_width);
                let display = truncate_to_display_width(&normalized, text_budget);
                lines.push(Line::from(vec![
                    Span::styled("  ", dim),
                    Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                    Span::styled(display, dim),
                    Span::styled(suffix, dim),
                ]));
            }
        }

        if plan.show_summary {
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled(format!("+{} more…", plan.hidden_count), dim),
            ]));
        }

        if plan.show_followup {
            let label = if self.followup_count == 1 {
                "followup".to_string()
            } else {
                format!("followup ×{}", self.followup_count)
            };
            let display = truncate_to_display_width(&label, max_width);
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(display, dim),
            ]));
        }

        debug_assert_eq!(
            lines.len(),
            plan.total_rows as usize,
            "render line count must match height_hint"
        );
        frame.render_widget(Paragraph::new(lines), area);
    }
}

pub(crate) fn normalize_single_line(text: &str) -> String {
    text.replace('\r', "").replace('\n', " ⏎ ")
}

pub(crate) fn truncate_to_display_width(text: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    if text.width() <= max_width {
        return text.to_string();
    }
    let mut result = String::new();
    let mut current = 0usize;
    for ch in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current + w > max_width.saturating_sub(1) {
            result.push('…');
            break;
        }
        result.push(ch);
        current += w;
    }
    result
}

pub(crate) struct TipsComponent<'a> {
    pub(crate) tip: Option<&'a crate::state::Tip>,
}

impl ViewportComponent for TipsComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        use talos_conversation::TipKind;
        let text = if let Some(tip) = self.tip {
            let color = match tip.kind {
                TipKind::ExitHint | TipKind::QueueHint => semantic::TIP_SUCCESS,
                TipKind::ApprovalResult => semantic::TIP_RESULT,
                TipKind::LagWarning | TipKind::Error => semantic::TIP_ERROR,
                TipKind::Info => semantic::TIP_INFO,
            };
            Text::from(Line::from(Span::styled(
                format!(" {}", tip.text),
                Style::default().fg(color),
            )))
        } else {
            Text::from(Line::from(Span::styled(
                " Enter to send, Ctrl+C to interrupt, /skills to list skills, Ctrl+G evolution",
                Style::default().fg(semantic::DIM_TEXT),
            )))
        };
        frame.render_widget(Paragraph::new(text), area);
    }
}

pub(crate) struct InputPadComponent;

impl ViewportComponent for InputPadComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(semantic::INPUT_BG)),
            area,
        );
    }
}

pub(crate) struct InputComponent<'a> {
    pub(crate) state: &'a crate::state::TuiState,
    /// Height allocated by the viewport layout after fixed/modal/queue budgeting.
    pub(crate) max_height: u16,
}

pub(crate) const MAX_COMPOSER_LINES: u16 = 10;

impl ViewportComponent for InputComponent<'_> {
    fn height_hint(&self, width: u16) -> u16 {
        let content_width = composer_content_width(width);
        let content_rows = input_line_count_with_width(&self.state.input_buffer, content_width);
        let cursor_byte_pos = self.state.cursor_byte_pos();
        let cursor_row =
            cursor_line_col_with_width(&self.state.input_buffer[..cursor_byte_pos], content_width)
                .0;

        let natural = content_rows
            .max(cursor_row.saturating_add(1))
            .min(MAX_COMPOSER_LINES);
        natural.min(self.max_height)
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let input_text = crate::scrollback_input::build_input_text_with_max_height(
            self.state,
            composer_content_width(area.width),
            self.max_height,
        );
        let input_block = Block::default()
            .style(Style::default().bg(semantic::INPUT_BG))
            .padding(Padding::new(0, COMPOSER_RIGHT_PAD, 0, 0));
        frame.render_widget(Paragraph::new(input_text).block(input_block), area);
    }
}

pub(crate) struct StatusComponent<'a> {
    pub(crate) status: &'a talos_conversation::StatusSnapshot,
    pub(crate) width: u16,
}

impl ViewportComponent for StatusComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        1
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let text = build_status_text(self.status, self.width);
        frame.render_widget(Paragraph::new(text), area);
    }
}

pub(crate) struct BottomPanelComponent<'a> {
    pub(crate) menu: &'a crate::state::BottomPanelState,
    pub(crate) query: &'a str,
    pub(crate) max_height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BottomPanelPlacement {
    AboveInput,
    BelowInput,
}

pub(crate) const fn bottom_panel_placement(
    screen_height: u16,
    base_height: u16,
    menu_height: u16,
) -> BottomPanelPlacement {
    if base_height.saturating_add(menu_height) <= screen_height {
        BottomPanelPlacement::BelowInput
    } else {
        BottomPanelPlacement::AboveInput
    }
}

pub(crate) fn bottom_panel_rows(
    total: usize,
    area_height: u16,
    header_rows: u16,
) -> (usize, bool, bool) {
    let show_separator = area_height > header_rows;
    let row_capacity = area_height.saturating_sub(header_rows + u16::from(show_separator)) as usize;
    let initial_visible = total
        .min(crate::state::SLASH_MENU_MAX_VISIBLE)
        .min(row_capacity);
    let show_indicator = total > initial_visible && row_capacity >= 2;
    let visible = total
        .min(crate::state::SLASH_MENU_MAX_VISIBLE)
        .min(row_capacity.saturating_sub(usize::from(show_indicator)));
    (visible, show_separator, show_indicator)
}

impl ViewportComponent for BottomPanelComponent<'_> {
    fn height_hint(&self, w: u16) -> u16 {
        if !self.menu.is_open {
            return 0;
        }
        if self.menu.is_approval() {
            let Some(crate::state::PanelKind::Approval { arguments, .. }) = &self.menu.kind else {
                return 0;
            };
            return approval_natural_height(w, arguments).min(self.max_height);
        }
        if self.menu.is_credential_input() {
            let (is_connect, has_default_endpoint) = match &self.menu.kind {
                Some(crate::state::PanelKind::CredentialInput {
                    connect_mode: true,
                    default_base_url,
                    ..
                }) => (true, default_base_url.is_some()),
                _ => (false, false),
            };
            return (if is_connect && !has_default_endpoint {
                4u16
            } else {
                3u16
            })
            .min(self.max_height);
        }
        let filtered = self.menu.filtered_items(self.query).len();
        let extra_header = if self.menu.is_variant_picker() || self.menu.is_model_list() {
            2
        } else {
            0
        };
        let natural_height = if filtered == 0 {
            1 + extra_header
        } else {
            let visible = filtered.min(crate::state::SLASH_MENU_MAX_VISIBLE) as u16;
            let indicator = u16::from(filtered > crate::state::SLASH_MENU_MAX_VISIBLE);
            1 + extra_header + visible + indicator
        };
        natural_height.min(self.max_height)
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        if !self.menu.is_open || area.height == 0 {
            return;
        }

        if self.menu.is_approval() {
            self.render_approval(frame, area);
            return;
        }

        if self.menu.is_credential_input() {
            let (provider, model_id, connect_mode, has_default_endpoint) = match &self.menu.kind {
                Some(crate::state::PanelKind::CredentialInput {
                    provider,
                    model_id,
                    connect_mode,
                    default_base_url,
                }) => (
                    provider.as_str(),
                    model_id.as_deref(),
                    *connect_mode,
                    default_base_url.is_some(),
                ),
                _ => ("?", None, false, false),
            };
            let style = Style::default().bg(semantic::INPUT_BG);
            let dim = Style::default().fg(semantic::DIM_TEXT);
            let header = match model_id {
                Some(id) => format!(" Provider: {provider}  Model: {id}"),
                None => format!(" Provider: {provider}"),
            };

            let api_key_display = credential_display_text(self.menu.credential_buffer.as_str());
            let api_key_color = if self.menu.credential_buffer.is_empty() {
                dim
            } else {
                Style::default().fg(semantic::TEXT_PRIMARY)
            };

            let mut lines = vec![Line::from(Span::styled(
                header,
                Style::default().fg(crate::nord::NORD8).bold(),
            ))];

            if !connect_mode {
                lines.push(Line::from(Span::styled(
                    " Enter the API key for this provider and press Enter (Esc to cancel).",
                    dim,
                )));
                lines.push(Line::from(Span::styled(
                    format!(" ▸ {api_key_display}"),
                    api_key_color,
                )));
                frame.render_widget(Paragraph::new(lines).style(style), area);
                return;
            }

            if has_default_endpoint {
                lines.push(Line::from(Span::styled(
                    " Enter the API key for this provider and press Enter (Esc to cancel).",
                    dim,
                )));
                lines.push(Line::from(Span::styled(
                    format!(" ▸ {api_key_display}"),
                    api_key_color,
                )));
                frame.render_widget(Paragraph::new(lines).style(style), area);
                return;
            }

            let is_base_url_field =
                self.menu.credential_field == crate::state::CredentialField::BaseUrl;
            let base_url_display = if self.menu.base_url_buffer.is_empty() {
                "(required)".to_string()
            } else {
                self.menu.base_url_buffer.clone()
            };

            if is_base_url_field {
                lines.push(Line::from(Span::styled(
                    " Base URL is required for custom providers. Enter to save (Esc to cancel).",
                    dim,
                )));
                lines.push(Line::from(Span::styled(
                    format!("   API key: {api_key_display}"),
                    dim,
                )));
                lines.push(Line::from(Span::styled(
                    format!(" ▸ {base_url_display}"),
                    Style::default().fg(semantic::TEXT_PRIMARY),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    " Enter the API key, then Enter to continue to the required base URL (Esc to cancel).",
                    dim,
                )));
                lines.push(Line::from(Span::styled(
                    format!(" ▸ {api_key_display}"),
                    api_key_color,
                )));
                lines.push(Line::from(Span::styled(
                    format!("   Base URL: {base_url_display}"),
                    dim,
                )));
            }

            frame.render_widget(Paragraph::new(lines).style(style), area);
            return;
        }

        let indices = self.menu.filtered_indices(self.query);
        if indices.is_empty() {
            let dim = Style::default().fg(semantic::DIM_TEXT);
            let text = if self.menu.is_slash() {
                Line::from(Span::styled(" No matching commands", dim))
            } else {
                Line::from(Span::styled(" No matches", dim))
            };
            frame.render_widget(
                Paragraph::new(text).style(Style::default().bg(semantic::INPUT_BG)),
                area,
            );
            return;
        }

        let total = indices.len();
        let extra_header = if self.menu.is_variant_picker() || self.menu.is_model_list() {
            2
        } else {
            0
        };
        let (visible, show_separator, show_indicator) =
            bottom_panel_rows(total, area.height, extra_header);

        let selected_pos = indices
            .iter()
            .position(|&i| i == self.menu.selected_index)
            .unwrap_or(0);

        let scroll_offset = if selected_pos >= visible {
            selected_pos - visible + 1
        } else {
            0
        };

        let dim = Style::default().fg(semantic::DIM_TEXT);
        let primary = Style::default().fg(semantic::TEXT_PRIMARY);
        let selected_style = Style::default()
            .fg(semantic::TEXT_ACCENT)
            .bg(semantic::NORD2);
        let header_style = Style::default()
            .fg(semantic::TEXT_WARNING)
            .bg(semantic::NORD2)
            .bold();
        let header_selected = Style::default()
            .fg(semantic::TEXT_WARNING)
            .bg(semantic::NORD2)
            .bold();

        let mut lines: Vec<Line<'static>> = Vec::with_capacity(area.height as usize);

        if let Some(crate::state::PanelKind::VariantPicker {
            provider, model_id, ..
        }) = &self.menu.kind
        {
            lines.push(Line::from(Span::styled(
                format!(" Model: {model_id}   Provider: {provider}"),
                Style::default().fg(crate::nord::NORD8).bold(),
            )));
            lines.push(Line::from(Span::styled(
                " Select a variant (or press Esc to cancel)",
                dim,
            )));
        } else if let Some(crate::state::PanelKind::ModelList { provider }) = &self.menu.kind {
            lines.push(Line::from(Span::styled(
                format!(" Provider: {provider}"),
                Style::default().fg(crate::nord::NORD8).bold(),
            )));
            lines.push(Line::from(Span::styled(
                " Select a model (or press Esc to cancel)",
                dim,
            )));
        }

        if show_separator {
            let separator = format!(" {}", "─".repeat(area.width.saturating_sub(1) as usize));
            lines.push(Line::from(Span::styled(separator, dim)));
        }

        for i in 0..visible {
            let pos = scroll_offset + i;
            if pos >= total {
                break;
            }
            let raw_idx = indices[pos];
            let item = &self.menu.items[raw_idx];
            let is_selected = raw_idx == self.menu.selected_index;

            if item.action == crate::state::PanelItemAction::Header {
                let style = if is_selected {
                    header_selected
                } else {
                    header_style
                };
                lines.push(Line::from(Span::styled(
                    format!("  {} ───", item.label),
                    style,
                )));
                continue;
            }

            let (name, desc) = if self.menu.is_slash() {
                let command_name = item.label.strip_prefix('/').unwrap_or(&item.label);
                let name = if let crate::state::PanelItemAction::SlashCommand {
                    arg_hint: Some(arg_hint),
                    ..
                } = &item.action
                {
                    format!("  /{command_name} {arg_hint}")
                } else {
                    format!("  /{command_name}")
                };
                let mode = if matches!(
                    &item.action,
                    crate::state::PanelItemAction::SlashCommand {
                        execution_mode: talos_conversation::CommandExecutionMode::DirectExecution,
                        ..
                    }
                ) {
                    "  —  Enter to run"
                } else {
                    "  —  Enter to complete"
                };
                let desc = format!("{mode}; {}", item.description);
                (name, desc)
            } else {
                let marker = if self.menu.is_variant_picker() || self.menu.is_model_list() {
                    if is_selected { "▶ " } else { "  " }
                } else if item.is_current {
                    "▶ "
                } else {
                    "  "
                };
                let name = format!("{marker}{}", item.label);
                let desc = if item.description.is_empty() {
                    String::new()
                } else {
                    format!("  —  {}", item.description)
                };
                (name, desc)
            };

            if is_selected {
                let name_span = Span::styled(name, selected_style);
                let desc_span = Span::styled(desc, selected_style);
                lines.push(Line::from(vec![name_span, desc_span]));
            } else {
                let name_span = Span::styled(name, primary);
                let desc_span = Span::styled(desc, dim);
                lines.push(Line::from(vec![name_span, desc_span]));
            }
        }

        if show_indicator {
            let indicator = format!("  … {}/{}", scroll_offset + visible, total);
            lines.push(Line::from(Span::styled(indicator, dim)));
        }

        let text = Text::from(lines);
        frame.render_widget(
            Paragraph::new(text).style(Style::default().bg(semantic::INPUT_BG)),
            area,
        );
    }
}

impl BottomPanelComponent<'_> {
    fn render_approval(&self, frame: &mut InlineFrame, area: Rect) {
        let Some(crate::state::PanelKind::Approval {
            tool_name,
            arguments,
        }) = &self.menu.kind
        else {
            return;
        };

        let warn = semantic::TEXT_WARNING;
        let accent = semantic::TEXT_ACCENT;
        let dim = semantic::DIM_TEXT;
        let nord2 = semantic::NORD2;
        let input_bg = semantic::INPUT_BG;
        let selected_style = Style::default().fg(accent).bg(nord2);
        let unselected_style = Style::default().fg(dim).bg(input_bg);

        let width = area.width;
        let height = area.height as usize;
        let wide = width >= 60;

        let mut lines: Vec<Line<'static>> = Vec::new();

        let sep = format!(" {}", "─".repeat(width.saturating_sub(1) as usize));
        lines.push(Line::from(Span::styled(sep, Style::default().fg(dim))));

        if wide {
            let prefix = format!("  \u{26a0} {tool_name}: ");
            let arg_w = width
                .saturating_sub(unicode_width::UnicodeWidthStr::width(prefix.as_str()) as u16)
                .max(1) as usize;
            let args_disp = truncate_one_line(arguments, arg_w);
            lines.push(Line::from(Span::styled(
                format!("{prefix}{args_disp}"),
                Style::default().fg(warn).bg(nord2).bold(),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  \u{26a0} {tool_name}"),
                Style::default().fg(warn).bg(nord2).bold(),
            )));
        }

        let options_count = self.menu.items.len();
        let mandatory = 1 + 1 + options_count;
        let budget = height.saturating_sub(mandatory);

        if !wide && !arguments.trim().is_empty() && budget > 0 {
            let arg_w = (width as usize).saturating_sub(4).max(1);
            let arg_max = budget.min(2);
            let wrapped = wrap_text_to_lines(arguments, arg_w, arg_max);
            for wl in &wrapped {
                lines.push(Line::from(Span::styled(
                    format!("  {wl}"),
                    Style::default().fg(warn).bg(input_bg),
                )));
            }
        }

        for (i, item) in self.menu.items.iter().enumerate() {
            if lines.len() >= height {
                break;
            }
            let is_selected = i == self.menu.selected_index;
            let style = if is_selected {
                selected_style
            } else {
                unselected_style
            };
            lines.push(Line::from(Span::styled(format!("  {}", item.label), style)));
        }

        if lines.len() < height {
            lines.push(Line::from(Span::styled(
                "  Up/Down to navigate, Enter to confirm",
                Style::default().fg(dim).bg(input_bg),
            )));
        }

        frame.render_widget(
            Paragraph::new(Text::from(lines)).style(Style::default().bg(input_bg)),
            area,
        );
    }
}

fn truncate_one_line(value: &str, max_width: usize) -> String {
    let single = value.replace('\n', " ");
    if unicode_width::UnicodeWidthStr::width(single.as_str()) <= max_width {
        return single;
    }
    if max_width == 0 {
        return String::new();
    }
    let mut width = 0usize;
    let mut result = String::new();
    for ch in single.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw > max_width.saturating_sub(1) {
            break;
        }
        result.push(ch);
        width += cw;
    }
    result + "…"
}

pub(crate) fn approval_natural_height(width: u16, arguments: &str) -> u16 {
    const BASE: u16 = 6;
    if width >= 60 || arguments.trim().is_empty() {
        BASE
    } else {
        let arg_w = (width as usize).saturating_sub(4).max(1);
        let wrapped = wrap_text_to_lines(arguments, arg_w, 2);
        BASE + wrapped.len() as u16
    }
}

pub(crate) fn wrap_text_to_lines(text: &str, max_width: usize, max_lines: usize) -> Vec<String> {
    if max_width == 0 || max_lines == 0 {
        return Vec::new();
    }
    let flat: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();

    let mut all_lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;

    for ch in flat.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_w + cw > max_width && !current.is_empty() {
            all_lines.push(std::mem::take(&mut current));
            current_w = 0;
        }
        current.push(ch);
        current_w += cw;
    }
    if !current.is_empty() {
        all_lines.push(current);
    }

    if all_lines.is_empty() {
        return Vec::new();
    }

    if all_lines.len() <= max_lines {
        return all_lines;
    }

    let mut result: Vec<String> = all_lines[..max_lines].to_vec();
    if let Some(last) = result.last_mut() {
        while unicode_width::UnicodeWidthStr::width(last.as_str()) >= max_width && !last.is_empty()
        {
            last.pop();
        }
        last.push('…');
    }
    result
}

pub(crate) fn extract_thinking_title(text: &str) -> Option<&str> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut last_title: Option<&str> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(title) = parse_standalone_bold(trimmed) else {
            continue;
        };
        let followed_by_empty_or_eof = match lines.get(i + 1) {
            None => true,
            Some(next) => next.trim().is_empty(),
        };
        if followed_by_empty_or_eof {
            last_title = Some(title);
        }
    }
    last_title
}

fn parse_standalone_bold(line: &str) -> Option<&str> {
    let after_open = line.strip_prefix("**")?;
    let title = after_open.strip_suffix("**")?;
    if title.is_empty() || title.contains('*') {
        return None;
    }
    Some(title)
}

pub(crate) fn truncate_end_to_width(s: &str, max_width: u16) -> String {
    let max = max_width as usize;
    if unicode_width::UnicodeWidthStr::width(s) <= max {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut width = 0usize;
    let mut start = chars.len();
    for (i, ch) in chars.iter().enumerate().rev() {
        let cw = unicode_width::UnicodeWidthChar::width(*ch).unwrap_or(0);
        if width + cw > max {
            break;
        }
        width += cw;
        start = i;
    }
    chars[start..].iter().collect()
}

pub(crate) fn stream_padding_for(
    source: Option<&MessageSource>,
    line_index: usize,
) -> &'static str {
    if line_index > 0 {
        return "   ";
    }

    match source {
        Some(MessageSource::User) => " > ",
        Some(MessageSource::Assistant) => " ● ",
        Some(MessageSource::System) => " # ",
        Some(MessageSource::Error) => " ! ",
        Some(MessageSource::Reasoning) => " ◇ ",
        Some(MessageSource::Tool { .. }) => " ● ",
        None => "   ",
    }
}

pub(crate) fn stream_bg_for(source: Option<&MessageSource>) -> Option<CColor> {
    match source {
        Some(MessageSource::User) => to_crossterm_color(semantic::INPUT_BG),
        _ => None,
    }
}

pub(crate) fn prefix_color_for(
    source: Option<&MessageSource>,
    line_index: usize,
) -> Option<CColor> {
    if line_index > 0 {
        return None;
    }

    match source {
        Some(MessageSource::User) => to_crossterm_color(semantic::PREFIX_USER),
        Some(MessageSource::Assistant)
        | Some(MessageSource::Tool { .. })
        | Some(MessageSource::Reasoning) => to_crossterm_color(semantic::PREFIX_ASSISTANT),
        Some(MessageSource::System) => to_crossterm_color(semantic::PREFIX_SYSTEM),
        Some(MessageSource::Error) => to_crossterm_color(semantic::PREFIX_ERROR),
        None => None,
    }
}

pub(crate) fn stream_opening_lines(
    stream_count: usize,
    opening: Vec<ScrollbackLine>,
) -> Vec<ScrollbackLine> {
    let mut lines = Vec::new();
    if stream_count > 0 {
        lines.push(ScrollbackLine::plain(String::new(), None));
    }
    lines.extend(opening);
    lines
}

pub(crate) fn strip_llm_hints(content: &str) -> String {
    content
        .trim_end_matches("\n\n[Analyze the error above and try a different approach.]")
        .to_string()
}

pub(crate) fn summary_fields_for(tool_name: &str) -> Vec<String> {
    match tool_name {
        "read" | "write" | "edit" | "delete" | "ls" | "stat" => vec!["path".to_string()],
        "bash" => vec!["command".to_string()],
        "grep" => vec!["pattern".to_string()],
        "glob" => vec!["pattern".to_string()],
        "fetch_url" => vec!["url".to_string(), "mode".to_string()],
        "http_request" => vec!["method".to_string(), "url".to_string()],
        "save_url" => vec!["url".to_string(), "destination".to_string()],
        "web_search" => vec!["query".to_string()],
        "git_add" | "git_commit" | "git_push" | "git_pull" | "git_checkout" => {
            vec!["path".to_string()]
        }
        _ => vec![],
    }
}

#[allow(dead_code)]
pub(crate) fn render_history_messages(
    stream_count: &mut usize,
    history: &[Message],
) -> Vec<ScrollbackLine> {
    let mut lines = Vec::new();
    let mut pending_tool_names: Vec<String> = Vec::new();
    for message in history {
        match message {
            Message::Tool { result } => {
                let tool_name = if !pending_tool_names.is_empty() {
                    pending_tool_names.remove(0)
                } else {
                    result.tool_use_id.clone()
                };
                let icon = if result.is_error { "✗" } else { "" };
                let color = if result.is_error {
                    to_crossterm_color(semantic::TEXT_ERROR)
                } else {
                    to_crossterm_color(semantic::TEXT_SUCCESS)
                };
                let content = strip_llm_hints(&result.content);
                let display = talos_conversation::ToolResultDisplay {
                    tool_name: Some(tool_name),
                    is_error: result.is_error,
                    content,
                };
                lines.extend(crate::tool_display::build_tool_result_scrollback_lines(
                    &display, icon, color,
                ));
            }
            Message::Assistant {
                content,
                tool_calls,
                ..
            } => {
                let tool_calls_in_text = talos_core::message::extract_tool_calls_from_text(content);
                let cleaned = talos_core::message::strip_tool_syntax(content);
                let has_tool_calls = !tool_calls.is_empty() || !tool_calls_in_text.is_empty();

                pending_tool_names.clear();
                for tc in tool_calls {
                    pending_tool_names.push(tc.name.clone());
                }

                if !has_tool_calls && !cleaned.is_empty() {
                    lines.extend(render_history_message(
                        stream_count,
                        MessageSource::Assistant,
                        &cleaned,
                    ));
                }

                let calls: Vec<talos_conversation::ToolCallDisplay> = if !tool_calls.is_empty() {
                    tool_calls
                        .iter()
                        .map(|tc| talos_conversation::ToolCallDisplay {
                            tool_name: tc.name.clone(),
                            arguments: tc.input.clone(),
                            provenance: talos_core::tool::ToolProvenance::Native,
                            summary_fields: summary_fields_for(&tc.name),
                        })
                        .collect()
                } else if !tool_calls_in_text.is_empty() {
                    for tc in &tool_calls_in_text {
                        pending_tool_names.push(tc.name.clone());
                    }
                    tool_calls_in_text
                        .iter()
                        .map(|tc| talos_conversation::ToolCallDisplay {
                            tool_name: tc.name.clone(),
                            arguments: tc.input.clone(),
                            provenance: talos_core::tool::ToolProvenance::Native,
                            summary_fields: summary_fields_for(&tc.name),
                        })
                        .collect()
                } else {
                    vec![]
                };

                for call in &calls {
                    lines.push(crate::tool_display::build_tool_call_scrollback_line(call));
                }
            }
            _ => {
                let Some((source, content)) = history_message_parts(message) else {
                    continue;
                };
                if content.is_empty() {
                    continue;
                }
                lines.extend(render_history_message(stream_count, source, &content));
            }
        }
    }
    lines
}

pub(crate) fn render_history_message(
    stream_count: &mut usize,
    source: MessageSource,
    content: &str,
) -> Vec<ScrollbackLine> {
    let mut renderer = StreamRenderState::default();
    let mut lines = stream_opening_lines(*stream_count, renderer.start(source));
    lines.extend(renderer.push_chunk(content));
    if !content.ends_with('\n') {
        lines.extend(renderer.push_chunk("\n"));
    }
    lines.extend(renderer.finish());
    *stream_count += 1;
    lines
}

fn history_message_parts(message: &Message) -> Option<(MessageSource, String)> {
    match message {
        Message::User { content } => Some((MessageSource::User, content.clone())),
        Message::Assistant { content, .. } => Some((MessageSource::Assistant, content.clone())),
        Message::Tool { result } => Some((
            MessageSource::Tool {
                name: result.tool_use_id.clone(),
            },
            result.content.clone(),
        )),
        Message::System { content, .. } => Some((MessageSource::System, content.clone())),
        Message::Context { content } => Some((MessageSource::System, content.clone())),
        // R10: Multimodal messages must render a safe summary instead of
        // being silently dropped on history hydration. We surface only
        // the text content (if any) and a per-image marker containing
        // basename + MIME + byte count. Full canonical paths and image
        // bytes never enter the terminal scrollback.
        Message::Multimodal { parts } => {
            let mut summary = String::new();
            for part in parts {
                match part {
                    talos_core::message::ContentPart::Text { text } => {
                        if !summary.is_empty() {
                            summary.push('\n');
                        }
                        summary.push_str(text);
                    }
                    talos_core::message::ContentPart::Image {
                        path,
                        mime,
                        byte_count,
                    } => {
                        if !summary.is_empty() {
                            summary.push('\n');
                        }
                        let basename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("(unknown)");
                        summary
                            .push_str(&format!("[Image: {basename} ({byte_count} bytes, {mime})]"));
                    }
                }
            }
            if summary.is_empty() {
                None
            } else {
                Some((MessageSource::User, summary))
            }
        }
    }
}

#[cfg(test)]
mod r10_tests {
    use super::*;
    use talos_core::message::{ContentPart, Message};

    fn image_part(name: &str, mime: &str, bytes: u64) -> ContentPart {
        ContentPart::Image {
            path: std::path::PathBuf::from(format!("/tmp/{name}")),
            mime: mime.to_string(),
            byte_count: bytes,
        }
    }

    #[test]
    fn plain_user_message_round_trips() {
        let msg = Message::User {
            content: "hello".to_string(),
        };
        let (source, content) = history_message_parts(&msg).expect("must return Some");
        assert_eq!(source, MessageSource::User);
        assert_eq!(content, "hello");
    }

    /// R10 regression: Multimodal messages must NOT be silently dropped
    /// from history. The summary must include the text part and an
    /// [Image: ...] marker per image. The full canonical path must NOT
    /// appear — only the basename.
    #[test]
    fn multimodal_message_produces_safe_summary_instead_of_none() {
        let msg = Message::Multimodal {
            parts: vec![
                ContentPart::Text {
                    text: "describe these".to_string(),
                },
                image_part("a.png", "image/png", 100),
                image_part("b.jpg", "image/jpeg", 200),
            ],
        };
        let (source, content) =
            history_message_parts(&msg).expect("R10: Multimodal must produce a summary, not None");
        assert_eq!(source, MessageSource::User);
        assert!(content.contains("describe these"));
        assert!(content.contains("[Image: a.png (100 bytes, image/png)]"));
        assert!(content.contains("[Image: b.jpg (200 bytes, image/jpeg)]"));
        // Critical privacy invariant: full canonical path must not leak.
        assert!(
            !content.contains("/tmp/"),
            "summary must not expose the full canonical path, got: {content}"
        );
    }

    /// R10 privacy: image-only Multimodal messages (no text) still
    /// render a safe per-image marker.
    #[test]
    fn image_only_multimodal_produces_summary() {
        let msg = Message::Multimodal {
            parts: vec![image_part("shot.png", "image/png", 4096)],
        };
        let (source, content) = history_message_parts(&msg).expect("must produce summary");
        assert_eq!(source, MessageSource::User);
        assert_eq!(content, "[Image: shot.png (4096 bytes, image/png)]");
    }

    /// R10 edge: empty Multimodal parts vector returns None (nothing
    /// to render), preserving the existing skip-empty contract.
    #[test]
    fn empty_multimodal_returns_none() {
        let msg = Message::Multimodal { parts: vec![] };
        assert!(history_message_parts(&msg).is_none());
    }
}
