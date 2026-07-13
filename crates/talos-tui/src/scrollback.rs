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

pub(crate) use crate::scrollback_input::{
    build_input_text, credential_cursor_col, credential_display_text, cursor_line_col,
    input_line_count,
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

pub(crate) struct QueuePreviewComponent {
    pub(crate) count: usize,
    pub(crate) steering: usize,
    pub(crate) followup: usize,
}

impl ViewportComponent for QueuePreviewComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        if self.count == 0 {
            0
        } else {
            1 + (self.count as u16).min(2)
        }
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let dim = Style::default().fg(semantic::DIM_TEXT);
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(" ", dim),
            Span::styled(
                format!(
                    "{} queued input{}",
                    self.count,
                    if self.count == 1 { "" } else { "s" }
                ),
                dim,
            ),
            Span::styled(" (will send after current turn)", dim),
        ]));
        let max_width = (area.width as usize).saturating_sub(4);
        let show_steering = self.steering.min(2);
        for i in 0..show_steering {
            let label = if i == 0 { "steering" } else { "…" };
            let text = if label.len() > max_width {
                format!("{}…", &label[..max_width - 1])
            } else {
                label.to_string()
            };
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(text, dim),
            ]));
        }
        if self.followup > 0 && lines.len() < 3 {
            let label = if self.followup == 1 {
                "followup".to_string()
            } else {
                format!("followup x{}", self.followup)
            };
            let text = if label.len() > max_width {
                format!("{}…", &label[..max_width - 1])
            } else {
                label
            };
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(text, dim),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), area);
    }
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
}

impl ViewportComponent for InputComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 {
        input_line_count(&self.state.input_buffer)
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let input_text = build_input_text(self.state);
        let input_block = Block::default()
            .style(Style::default().bg(semantic::INPUT_BG))
            .padding(Padding::new(0, 1, 0, 0));
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

pub(crate) fn bottom_panel_rows(total: usize, area_height: u16) -> (usize, bool, bool) {
    let show_separator = area_height >= 2;
    let row_capacity = area_height.saturating_sub(u16::from(show_separator)) as usize;
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
        let natural_height = if filtered == 0 {
            1
        } else {
            let visible = filtered.min(crate::state::SLASH_MENU_MAX_VISIBLE) as u16;
            let indicator = u16::from(filtered > crate::state::SLASH_MENU_MAX_VISIBLE);
            1 + visible + indicator
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
        let (visible, show_separator, show_indicator) = bottom_panel_rows(total, area.height);

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
                let marker = if item.is_current { "▶ " } else { "  " };
                let name = format!("{marker}{}", item.label);
                let desc = format!("  —  {}", item.description);
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
            let arg_w = width.saturating_sub(prefix.chars().count() as u16).max(1) as usize;
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

fn truncate_one_line(value: &str, max_chars: usize) -> String {
    let single = value.replace('\n', " ");
    let chars: Vec<char> = single.chars().collect();
    if chars.len() <= max_chars {
        single
    } else if max_chars == 0 {
        String::new()
    } else {
        format!(
            "{}…",
            chars[..max_chars.saturating_sub(1)]
                .iter()
                .collect::<String>()
        )
    }
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

pub(crate) fn wrap_text_to_lines(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    if width == 0 || max_lines == 0 {
        return Vec::new();
    }
    let flat: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    let chars: Vec<char> = flat.chars().collect();
    let total = chars.len().div_ceil(width);
    if total == 0 {
        return Vec::new();
    }
    let truncated = total > max_lines;
    let take = total.min(max_lines);
    let mut result = Vec::with_capacity(take);
    for i in 0..take {
        let start = i * width;
        let end = (start + width).min(chars.len());
        let mut line: String = chars[start..end].iter().collect();
        if truncated && i == take - 1 {
            let lc: Vec<char> = line.chars().collect();
            line = if lc.len() > 1 {
                lc[..lc.len() - 1].iter().collect::<String>() + "…"
            } else {
                "…".to_string()
            };
        }
        result.push(line);
    }
    result
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
                lines.extend(render_history_message(stream_count, source, content));
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

fn history_message_parts(message: &Message) -> Option<(MessageSource, &str)> {
    match message {
        Message::User { content } => Some((MessageSource::User, content.as_str())),
        Message::Assistant { content, .. } => Some((MessageSource::Assistant, content.as_str())),
        Message::Tool { result } => Some((
            MessageSource::Tool {
                name: result.tool_use_id.clone(),
            },
            result.content.as_str(),
        )),
        Message::System { content, .. } => Some((MessageSource::System, content.as_str())),
        Message::Context { content } => Some((MessageSource::System, content.as_str())),
    }
}
