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
    session_id: Option<&str>,
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

    if let Some(session_id) = session_id.filter(|id| !id.is_empty()) {
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("  session {session_id}"),
                to_crossterm_color(semantic::DIM_TEXT),
                HistoryAttrs::default(),
            )],
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

    // No pricing metadata available — omit cost line rather than fabricate.
    (None, "")
}

#[cfg(test)]
mod tests {
    use talos_conversation::StatusSnapshot;
    use talos_core::message::Usage;

    use super::estimate_cost;

    fn snapshot_with_pricing(
        input_tokens: u32,
        output_tokens: u32,
        input_price: Option<f64>,
        output_price: Option<f64>,
    ) -> StatusSnapshot {
        StatusSnapshot {
            usage: Usage {
                input_tokens,
                output_tokens,
                ..Default::default()
            },
            input_price_per_million: input_price,
            output_price_per_million: output_price,
            ..Default::default()
        }
    }

    #[test]
    fn estimate_cost_with_pricing_returns_value() {
        let status = snapshot_with_pricing(1000, 500, Some(3.0), Some(15.0));
        let (cost, label) = estimate_cost(&status);
        assert!(cost.is_some());
        assert_eq!(label, "Est");
        let cost = cost.unwrap();
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn estimate_cost_without_pricing_returns_none() {
        let status = snapshot_with_pricing(1000, 500, None, None);
        let (cost, label) = estimate_cost(&status);
        assert!(cost.is_none());
        assert_eq!(label, "");
    }

    #[test]
    fn estimate_cost_zero_tokens_returns_none() {
        let status = snapshot_with_pricing(0, 0, Some(3.0), Some(15.0));
        let (cost, label) = estimate_cost(&status);
        assert!(cost.is_none());
        assert_eq!(label, "");
    }

    #[test]
    fn estimate_cost_partial_pricing_returns_none() {
        let status = snapshot_with_pricing(1000, 500, Some(3.0), None);
        let (cost, label) = estimate_cost(&status);
        assert!(cost.is_none());
        assert_eq!(label, "");
    }
}
