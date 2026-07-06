use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use crate::scrollback::truncate_end_to_width;
use crate::theme::semantic;

struct StatusFlags {
    queue_total: usize,
    phase_label: Option<String>,
}

pub(crate) fn build_status_text(
    status: &talos_conversation::StatusSnapshot,
    width: u16,
) -> Text<'static> {
    let compact = width < 80;

    let model_name = &status.model_name;
    let provider = &status.provider;
    let workspace = &status.workspace_path;
    let total_tokens = (status.usage.input_tokens + status.usage.output_tokens) as u64;
    let output_tokens = status.usage.output_tokens;
    let reasoning_tokens = status.usage.reasoning_tokens;
    let output_usage_label = format_output_usage(output_tokens, reasoning_tokens);
    let flags = StatusFlags {
        queue_total: status.steering_count + status.followup_count,
        phase_label: phase_status_label(status),
    };
    let context_label = format_context_label(status.context_limit, total_tokens);

    if compact {
        return build_compact_status(
            model_name,
            &context_label,
            status_provider_for_display(model_name, provider),
            workspace,
            &output_usage_label,
            &flags,
            width,
        );
    }

    build_expanded_status(
        model_name,
        &context_label,
        status_provider_for_display(model_name, provider),
        workspace,
        &output_usage_label,
        &flags,
        width,
    )
}

fn format_output_usage(output_tokens: u32, reasoning_tokens: u32) -> String {
    let out = crate::formatting::format_tokens(output_tokens as u64);
    if reasoning_tokens > 0 {
        let thinking = crate::formatting::format_tokens(reasoning_tokens as u64);
        format!("{out} out ({thinking} thinking)")
    } else {
        format!("{out} out")
    }
}

fn format_context_limit(limit: Option<u32>) -> String {
    match limit {
        Some(tokens) if tokens >= 1_000_000 => {
            let m = tokens / 1_000_000;
            format!("{m}M ctx")
        }
        Some(tokens) if tokens >= 1000 => {
            let k = tokens / 1000;
            format!("{k}k ctx")
        }
        Some(tokens) => format!("{tokens} ctx"),
        None => String::new(),
    }
}

fn format_context_label(limit: Option<u32>, total_tokens: u64) -> String {
    let limit_label = format_context_limit(limit);
    if limit_label.is_empty() {
        return limit_label;
    }

    match context_usage_percent(limit, total_tokens) {
        Some(percent) => format!("{limit_label} · {percent}%"),
        None => limit_label,
    }
}

fn context_usage_percent(limit: Option<u32>, total_tokens: u64) -> Option<u64> {
    let limit = u64::from(limit?);
    if limit == 0 {
        return None;
    }

    Some((total_tokens.saturating_mul(100) + (limit / 2)) / limit)
}

fn build_compact_status(
    model_name: &str,
    context_label: &str,
    provider: &str,
    workspace: &str,
    output_usage_label: &str,
    flags: &StatusFlags,
    width: u16,
) -> Text<'static> {
    let dim = Style::default().fg(semantic::DIM_TEXT);
    let accent = Style::default().fg(semantic::TEXT_ACCENT);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let tokens_part = format!(" {output_usage_label}");
    let queue_part = if flags.queue_total > 0 {
        format!(" · ⬡{}", flags.queue_total)
    } else {
        String::new()
    };
    let phase_part = flags
        .phase_label
        .as_deref()
        .map(|label| format!(" · {label}"))
        .unwrap_or_default();
    let ctx_len = if context_label.is_empty() {
        0
    } else {
        context_label.chars().count() + 2
    };
    let reserved = tokens_part.chars().count()
        + queue_part.chars().count()
        + phase_part.chars().count()
        + ctx_len
        + 1;
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
    let ctx_part = if context_label.is_empty() {
        String::new()
    } else {
        format!(" ({context_label})")
    };
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
        Span::styled(format!("{model_part}{ctx_part}{provider_part}"), accent),
        Span::styled(workspace_part, val),
        Span::styled("", dim),
        Span::styled(tokens_part, val),
        Span::styled(queue_part, dim),
        Span::styled(phase_part, dim),
    ]))
}

fn build_expanded_status(
    model_name: &str,
    context_label: &str,
    provider: &str,
    workspace: &str,
    output_usage_label: &str,
    flags: &StatusFlags,
    width: u16,
) -> Text<'static> {
    let dim = Style::default().fg(semantic::DIM_TEXT);
    let accent = Style::default().fg(semantic::TEXT_ACCENT);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let tokens_part = output_usage_label.to_string();
    let queue_part = if flags.queue_total > 0 {
        format!(" · ⬡ {} queued", flags.queue_total)
    } else {
        String::new()
    };
    let phase_part = flags
        .phase_label
        .as_deref()
        .map(|label| format!(" · {label}"))
        .unwrap_or_default();

    let ctx_len = if context_label.is_empty() {
        0
    } else {
        context_label.chars().count() + 2
    };
    let right_part = format!("{tokens_part}{queue_part}{phase_part}");
    let reserved = 1 + right_part.chars().count() + 5 + ctx_len;
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
    let ctx_part = if context_label.is_empty() {
        String::new()
    } else {
        format!(" ({context_label})")
    };
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
        Span::styled(format!("{model_part}{ctx_part}{provider_part}"), accent),
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

fn phase_status_label(status: &talos_conversation::StatusSnapshot) -> Option<String> {
    match status.phase.as_ref() {
        Some(talos_conversation::TurnPhase::TimedOut) => Some("timed out".to_string()),
        Some(talos_conversation::TurnPhase::Failed) => Some("failed".to_string()),
        Some(talos_conversation::TurnPhase::Cancelled) => Some("cancelled".to_string()),
        Some(talos_conversation::TurnPhase::RunningTool { name }) => Some(format!("tool: {name}")),
        _ => None,
    }
}
