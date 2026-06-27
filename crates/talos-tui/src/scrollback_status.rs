use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use talos_core::message::Usage;

use crate::scrollback::truncate_end_to_width;
use crate::theme::semantic;

pub(crate) fn build_status_text(
    status: &talos_conversation::StatusSnapshot,
    width: u16,
) -> Text<'static> {
    let compact = width < 80;

    let model_name = &status.model_name;
    let provider = &status.provider;
    let workspace = &status.workspace_path;
    let total_tokens = (status.usage.input_tokens + status.usage.output_tokens) as u64;
    let queue_total = status.steering_count + status.followup_count;

    if compact {
        return build_compact_status(
            model_name,
            status_provider_for_display(model_name, provider),
            workspace,
            total_tokens,
            queue_total,
            width,
        );
    }

    build_expanded_status(
        model_name,
        status_provider_for_display(model_name, provider),
        workspace,
        total_tokens,
        queue_total,
        width,
    )
}

fn build_compact_status(
    model_name: &str,
    provider: &str,
    workspace: &str,
    total_tokens: u64,
    queue_total: usize,
    width: u16,
) -> Text<'static> {
    let dim = Style::default().fg(semantic::DIM_TEXT);
    let accent = Style::default().fg(semantic::TEXT_ACCENT);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let tokens_part = format!(" {}t", crate::formatting::format_tokens(total_tokens));
    let queue_part = if queue_total > 0 {
        format!(" · ⬡{queue_total}")
    } else {
        String::new()
    };
    let reserved = tokens_part.chars().count() + queue_part.chars().count() + 1;
    let available = (width as usize).saturating_sub(reserved);
    let model_limit = if workspace.is_empty() {
        available.saturating_sub(2).clamp(8, 20)
    } else {
        (available / 2).clamp(8, 20)
    };
    let workspace_limit = available
        .saturating_sub(model_limit)
        .saturating_sub(provider.chars().count().min(14))
        .clamp(8, 16);

    let model_part = format!("⬡ {}", truncate_str(model_name, model_limit));
    let provider_part = if provider.is_empty() {
        String::new()
    } else {
        format!(" ({})", truncate_str(provider, 12))
    };
    let workspace_part = if workspace.is_empty() {
        String::new()
    } else {
        format!(
            " ▸ {}",
            truncate_end_to_width(workspace, workspace_limit as u16)
        )
    };

    Text::from(Line::from(vec![
        Span::styled(" ", dim),
        Span::styled(format!("{model_part}{provider_part}"), accent),
        Span::styled(workspace_part, val),
        Span::styled("", dim),
        Span::styled(tokens_part, val),
        Span::styled(queue_part, dim),
    ]))
}

fn build_expanded_status(
    model_name: &str,
    provider: &str,
    workspace: &str,
    total_tokens: u64,
    queue_total: usize,
    width: u16,
) -> Text<'static> {
    let dim = Style::default().fg(semantic::DIM_TEXT);
    let accent = Style::default().fg(semantic::TEXT_ACCENT);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let tokens_part = format!("{} tokens", crate::formatting::format_tokens(total_tokens));
    let queue_part = if queue_total > 0 {
        format!(" · ⬡ {} queued", queue_total)
    } else {
        String::new()
    };

    let right_part = format!("{tokens_part}{queue_part}");
    let reserved = 1 + right_part.chars().count() + 5;
    let available = (width as usize).saturating_sub(reserved);
    let provider_budget = if provider.is_empty() {
        0
    } else {
        provider.chars().count().min(18) + 3
    };
    let model_limit = if workspace.is_empty() {
        available.saturating_sub(provider_budget).clamp(10, 40)
    } else {
        (available / 2)
            .saturating_sub(provider_budget / 2)
            .clamp(10, 36)
    };
    let workspace_limit = available
        .saturating_sub(model_limit)
        .saturating_sub(provider_budget)
        .clamp(12, 48);

    let model_part = format!("⬡ {}", truncate_str(model_name, model_limit));
    let provider_part = if provider.is_empty() {
        String::new()
    } else {
        format!(" ({})", truncate_str(provider, 18))
    };
    let workspace_part = if workspace.is_empty() {
        String::new()
    } else {
        format!(
            " ▸ {}",
            truncate_end_to_width(workspace, workspace_limit as u16)
        )
    };

    Text::from(Line::from(vec![
        Span::styled(" ", dim),
        Span::styled(format!("{model_part}{provider_part}"), accent),
        Span::styled(workspace_part, val),
        Span::styled("", dim),
        Span::styled("     ", dim),
        Span::styled(right_part, val),
    ]))
}

fn status_provider_for_display<'a>(model_name: &str, provider: &'a str) -> &'a str {
    if provider.is_empty() || model_name.starts_with(&format!("{provider}/")) {
        ""
    } else {
        provider
    }
}

pub(crate) fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        return s.to_string();
    }
    let truncated: String = chars[..max_len - 1].iter().collect();
    format!("{truncated}…")
}

#[allow(dead_code)]
pub(crate) fn calculate_cost(usage: &Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}
