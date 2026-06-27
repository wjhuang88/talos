//! Exit summary formatting for the TUI app.

use std::time::Duration;

use talos_conversation::StatusSnapshot;

use crate::app_stream::ScrollbackLine;
use crate::inline_terminal::{HistoryAttrs, HistorySegment};
use crate::theme::{semantic, to_crossterm_color};

pub(crate) fn build_exit_summary_lines(
    status: &StatusSnapshot,
    elapsed: Duration,
    stream_count: usize,
) -> Vec<ScrollbackLine> {
    let elapsed_secs = elapsed.as_secs();
    let usage = &status.usage;
    let total_tokens = (usage.input_tokens + usage.output_tokens) as u64;

    let mut lines = vec![ScrollbackLine::plain(String::new(), None)];

    let header_sep = "─".repeat(32);
    lines.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            format!("⬡ Talos session complete {header_sep}"),
            to_crossterm_color(semantic::TEXT_ACCENT),
            HistoryAttrs::default(),
        )],
        None,
    ));

    lines.push(ScrollbackLine::plain(String::new(), None));

    if !status.model_name.is_empty() {
        lines.push(ScrollbackLine::styled(
            vec![
                HistorySegment::styled(
                    format!("  {}  ", status.model_name),
                    to_crossterm_color(semantic::TEXT_ACCENT),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    format!("{}  ", crate::formatting::format_duration(elapsed_secs)),
                    to_crossterm_color(semantic::STATUS_VALUE),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    format!("{stream_count} turns"),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                ),
            ],
            None,
        ));
    } else {
        lines.push(ScrollbackLine::styled(
            vec![
                HistorySegment::styled(
                    format!("{}  ", crate::formatting::format_duration(elapsed_secs)),
                    to_crossterm_color(semantic::STATUS_VALUE),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    format!("{stream_count} turns"),
                    to_crossterm_color(semantic::DIM_TEXT),
                    HistoryAttrs::default(),
                ),
            ],
            None,
        ));
    }

    if usage.input_tokens > 0 || usage.output_tokens > 0 {
        lines.push(ScrollbackLine::plain(String::new(), None));
        lines.push(ScrollbackLine::styled(
            vec![
                HistorySegment::styled(
                    format!(
                        "  {} tokens in",
                        crate::formatting::format_tokens(usage.input_tokens as u64)
                    ),
                    to_crossterm_color(semantic::STATUS_VALUE),
                    HistoryAttrs::default(),
                ),
                HistorySegment::styled(
                    format!(
                        "      {} tokens out",
                        crate::formatting::format_tokens(usage.output_tokens as u64)
                    ),
                    to_crossterm_color(semantic::STATUS_VALUE),
                    HistoryAttrs::default(),
                ),
            ],
            None,
        ));
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!(
                    "  {} tokens total",
                    crate::formatting::format_tokens(total_tokens)
                ),
                to_crossterm_color(semantic::DIM_TEXT),
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    if let Some(cost) = estimate_cost(usage) {
        lines.push(ScrollbackLine::plain(String::new(), None));
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("  Est cost: ${cost:.2}"),
                to_crossterm_color(semantic::TEXT_ACCENT),
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    lines.push(ScrollbackLine::plain(String::new(), None));
    lines
}

fn estimate_cost(usage: &talos_core::message::Usage) -> Option<f64> {
    if usage.input_tokens == 0 && usage.output_tokens == 0 {
        return None;
    }
    let input_cost = usage.input_tokens as f64 * 3.0 / 1_000_000.0;
    let output_cost = usage.output_tokens as f64 * 15.0 / 1_000_000.0;
    Some(input_cost + output_cost)
}
