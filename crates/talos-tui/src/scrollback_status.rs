use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::scrollback::truncate_end_to_width;
use crate::scrollback_status_git;
use crate::theme::semantic;

/// Width-aware thresholds:
/// - Expanded (>= 100 cols): Shows all fields.
/// - Standard (>= 80 cols): Drops platform label.
/// - Narrow (>= 60 cols): Drops platform label and Git details.
/// - Minimal (< 60 cols): Truncates workspace heavily, drops variant and noncritical metrics.
pub(crate) fn build_status_text(
    status: &talos_conversation::StatusSnapshot,
    width: u16,
) -> Text<'static> {
    let dim = Style::default().fg(semantic::DIM_TEXT);
    let accent = Style::default().fg(semantic::TEXT_ACCENT);
    let val = Style::default().fg(semantic::STATUS_VALUE);

    let expanded = width >= 100;
    let standard = width >= 80;
    let narrow = width >= 60;

    let model_name = &status.model_name;
    let provider = status_provider_for_display(model_name, &status.provider);
    let workspace = &status.workspace_path;
    let variant = &status.variant;

    let total_tokens = (status.usage.input_tokens + status.usage.output_tokens) as u64;
    let context_label = format_context_label(status.context_limit, total_tokens);
    let output_usage_label =
        format_output_usage(status.usage.output_tokens, status.usage.reasoning_tokens);

    let queue_total = status.steering_count + status.followup_count;
    let phase_label = phase_status_label(status);

    let ctx_len = if context_label.is_empty() {
        0
    } else {
        display_width(&context_label) + 2
    };

    let platform_part = if expanded {
        let os = match std::env::consts::OS {
            "macos" => "macOS",
            "linux" => "Linux",
            "windows" => "Windows",
            other => other,
        };
        format!(" · {os}")
    } else {
        String::new()
    };

    let git_part = if standard && !workspace.is_empty() {
        if let Some(git_summary) = scrollback_status_git::get_git_status(workspace) {
            let branch = git_summary.branch.as_deref().unwrap_or("");
            let dirty = if git_summary.dirty { "*" } else { "" };
            if branch.is_empty() && dirty.is_empty() {
                String::new()
            } else if branch.is_empty() {
                format!(" · {dirty}")
            } else {
                format!(" · ⎇ {branch}{dirty}")
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let variant_part = if narrow {
        if let Some(v) = variant {
            format!(" · {v}")
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let metrics_part = if narrow {
        let attachments = if status.attachment_count > 0 {
            let suffix = if status.attachment_count == 1 {
                ""
            } else {
                "s"
            };
            format!(" · {} image{suffix}", status.attachment_count)
        } else {
            String::new()
        };
        let q = if queue_total > 0 {
            format!(" · ⬡ {queue_total} queued")
        } else {
            String::new()
        };
        let p = phase_label
            .as_deref()
            .map(|l| format!(" · {l}"))
            .unwrap_or_default();
        format!("{output_usage_label}{attachments}{q}{p}")
    } else {
        String::new()
    };

    let right_part = if narrow {
        metrics_part.to_string()
    } else {
        String::new()
    };

    let reserved = 1
        + display_width(&right_part)
        + 5
        + ctx_len
        + display_width(&platform_part)
        + display_width(&git_part)
        + display_width(&variant_part);
    let available = (width as usize).saturating_sub(reserved);

    let provider_budget = if provider.is_empty() || !narrow {
        0
    } else {
        display_width(provider).min(18) + 3
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

    let ctx_part = if context_label.is_empty() {
        String::new()
    } else {
        format!(" ({context_label})")
    };
    let model_part = format!("⬡ {}{}", truncate_str(model_name, model_limit), ctx_part);

    let provider_part = if narrow && !provider.is_empty() {
        format!(" ({})", truncate_str(provider, 18))
    } else {
        String::new()
    };

    let workspace_part = if workspace.is_empty() {
        String::new()
    } else {
        format!(
            " ▸ {}",
            truncate_end_to_width(workspace, workspace_limit as u16)
        )
    };

    let left_spans = vec![
        Span::styled(" ", dim),
        Span::styled(format!("{model_part}{provider_part}"), accent),
        Span::styled(workspace_part, val),
        Span::styled(git_part, dim),
        Span::styled(platform_part, dim),
        Span::styled(variant_part, accent),
    ];

    let right_spans = vec![Span::styled("     ", dim), Span::styled(right_part, val)];

    let mut all_spans = left_spans;
    if narrow {
        all_spans.extend(right_spans);
    }

    Text::from(Line::from(all_spans))
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

fn status_provider_for_display<'a>(model_name: &str, provider: &'a str) -> &'a str {
    if provider.is_empty() || model_name.starts_with(&format!("{provider}/")) {
        ""
    } else {
        provider
    }
}

pub(crate) fn truncate_str(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if display_width(s) <= max_len {
        return s.to_string();
    }
    let mut width = 0;
    let mut truncated = String::new();
    for ch in s.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + char_width + 1 > max_len {
            break;
        }
        truncated.push(ch);
        width += char_width;
    }
    truncated.push('…');
    truncated
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
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
