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

    let (cost, label) = estimate_cost(status);
    if let Some(cost) = cost {
        lines.push(ScrollbackLine::plain(String::new(), None));
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("  {label} cost: ${cost:.2}"),
                to_crossterm_color(semantic::TEXT_ACCENT),
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    lines.push(ScrollbackLine::plain(String::new(), None));
    lines
}

const DEFAULT_INPUT_PER_M: f64 = 3.0;
const DEFAULT_OUTPUT_PER_M: f64 = 15.0;

fn estimate_cost(status: &StatusSnapshot) -> (Option<f64>, &'static str) {
    let usage = &status.usage;
    if usage.input_tokens == 0 && usage.output_tokens == 0 {
        return (None, "");
    }

    if let (Some(input_rate), Some(output_rate)) = (
        status.input_price_per_million,
        status.output_price_per_million,
    ) {
        let input_cost = usage.input_tokens as f64 * input_rate / 1_000_000.0;
        let output_cost = usage.output_tokens as f64 * output_rate / 1_000_000.0;
        return (Some(input_cost + output_cost), "Est");
    }

    let input_cost = usage.input_tokens as f64 * DEFAULT_INPUT_PER_M / 1_000_000.0;
    let output_cost = usage.output_tokens as f64 * DEFAULT_OUTPUT_PER_M / 1_000_000.0;
    (Some(input_cost + output_cost), "Est (default)")
}
