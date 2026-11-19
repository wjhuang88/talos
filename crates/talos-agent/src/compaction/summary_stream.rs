//! Bounded stream decoder shared by the authoritative checkpoint pipeline.

use super::{CompactionError, CompactionResult};
use talos_core::message::AgentEvent;

/// Accept only a bounded, successfully terminated summary; provider text never
/// becomes a diagnostic, including when a partial response subsequently fails.
pub(crate) async fn read_summary_stream(
    rx: &mut talos_core::provider::Receiver<AgentEvent>,
) -> CompactionResult<String> {
    const MAX_SUMMARY_BYTES: usize = 32 * 1024;
    let mut timing = SummaryTiming::new();
    let mut summary = String::new();
    let mut completed = false;
    while let Some(event) = rx.recv().await {
        timing.event(&event);
        match event {
            AgentEvent::TextDelta { delta } if !completed => {
                if delta.len() > MAX_SUMMARY_BYTES.saturating_sub(summary.len()) {
                    return Err(CompactionError::CompactionFailed(
                        "summary exceeds byte limit".into(),
                    ));
                }
                summary.push_str(&delta);
            }
            AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                ..
            } if !completed => completed = true,
            AgentEvent::Error { .. } => {
                return Err(CompactionError::ProviderError(
                    "summary stream failed".into(),
                ));
            }
            AgentEvent::ToolCallStarted { .. }
            | AgentEvent::ToolCall { .. }
            | AgentEvent::ToolResult { .. } => {
                return Err(CompactionError::CompactionFailed(
                    "summary stream contains tool activity".into(),
                ));
            }
            AgentEvent::TurnEnd { .. } | AgentEvent::TextDelta { .. } => {
                return Err(CompactionError::CompactionFailed(
                    "invalid summary terminal sequence".into(),
                ));
            }
            _ => {}
        }
    }
    if !completed || summary.trim().is_empty() {
        return Err(CompactionError::CompactionFailed(
            "summary missing successful terminal or content".into(),
        ));
    }
    timing.completed = true;
    Ok(summary)
}

// Drop also records deadline cancellation; counters never retain streamed text.
struct SummaryTiming {
    started: tokio::time::Instant,
    first_text_ms: Option<u64>,
    text_bytes: usize,
    text_events: usize,
    completed: bool,
}

impl SummaryTiming {
    fn new() -> Self {
        Self {
            started: tokio::time::Instant::now(),
            first_text_ms: None,
            text_bytes: 0,
            text_events: 0,
            completed: false,
        }
    }

    fn event(&mut self, event: &AgentEvent) {
        if let AgentEvent::TextDelta { delta } = event {
            if !delta.is_empty() && self.first_text_ms.is_none() {
                self.first_text_ms = Some(self.started.elapsed().as_millis() as u64);
            }
            self.text_bytes = self.text_bytes.saturating_add(delta.len());
            self.text_events = self.text_events.saturating_add(1);
        }
    }
}

impl Drop for SummaryTiming {
    fn drop(&mut self) {
        tracing::info!(
            stage = "checkpoint_summary_stream",
            elapsed_ms = self.started.elapsed().as_millis() as u64,
            first_text_ms = ?self.first_text_ms,
            text_bytes = self.text_bytes,
            text_events = self.text_events,
            completed = self.completed,
            "checkpoint summary stream finished"
        );
    }
}

#[cfg(test)]
mod summary_stream_tests {
    use super::*;
    use talos_core::message::{StopReason, Usage};

    async fn read(events: Vec<AgentEvent>) -> CompactionResult<String> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(16);
        for event in events {
            tx.send(event).await.expect("fixture channel");
        }
        drop(tx);
        read_summary_stream(&mut rx).await
    }
    fn text(value: &str) -> AgentEvent {
        AgentEvent::TextDelta {
            delta: value.into(),
        }
    }
    fn end(reason: StopReason) -> AgentEvent {
        AgentEvent::TurnEnd {
            stop_reason: reason,
            usage: Usage::default(),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn summary_timing_counts_bytes_without_retaining_content() {
        let mut timing = SummaryTiming::new();
        timing.event(&text(""));
        assert_eq!(timing.first_text_ms, None);
        tokio::time::advance(std::time::Duration::from_millis(25)).await;
        timing.event(&text("界"));
        tokio::time::advance(std::time::Duration::from_millis(75)).await;
        timing.event(&text("abc"));
        assert_eq!(timing.first_text_ms, Some(25));
        assert_eq!(timing.text_bytes, 6);
        assert_eq!(timing.text_events, 3);
        assert!(!timing.completed);
    }

    #[tokio::test]
    async fn summary_rejects_partial_error_empty_missing_and_failed_terminal() {
        for events in [
            vec![AgentEvent::Error {
                message: "private provider detail".into(),
            }],
            vec![
                text("partial"),
                AgentEvent::Error {
                    message: "private provider detail".into(),
                },
            ],
            vec![text("partial")],
            vec![text("  "), end(StopReason::EndTurn)],
            vec![text("partial"), end(StopReason::MaxTokens)],
            vec![text("partial"), end(StopReason::ToolUse)],
            vec![text("done"), end(StopReason::EndTurn), text("late")],
            vec![
                text("done"),
                end(StopReason::EndTurn),
                AgentEvent::Error {
                    message: "private provider detail".into(),
                },
            ],
        ] {
            let error = read(events)
                .await
                .expect_err("invalid stream must not replace context");
            assert!(!error.to_string().contains("private provider detail"));
        }
    }
    #[tokio::test]
    async fn summary_rejects_tool_activity_even_with_success_terminal() {
        for after_terminal in [false, true] {
            let call = AgentEvent::ToolCall {
                provenance: talos_core::tool::ToolProvenance::Native,
                summary_fields: Vec::new(),
                call: talos_core::message::ToolCall {
                    id: "private-call-id".into(),
                    name: "private-tool".into(),
                    input: serde_json::json!({"secret":"private-argument"}),
                },
            };
            let events = if after_terminal {
                vec![text("checkpoint"), end(StopReason::EndTurn), call]
            } else {
                vec![text("checkpoint"), call, end(StopReason::EndTurn)]
            };
            let error = read(events)
                .await
                .expect_err("tool activity is not a summary");
            assert!(!error.to_string().contains("private"));
        }
    }

    #[tokio::test]
    async fn summary_enforces_bytes_and_preserves_valid_content() {
        assert!(
            read(vec![text(&"界".repeat(11_000)), end(StopReason::EndTurn)])
                .await
                .is_err()
        );
        assert_eq!(
            read(vec![
                text("goal"),
                text(" preserved"),
                end(StopReason::EndTurn)
            ])
            .await
            .expect("valid"),
            "goal preserved"
        );
    }
}
