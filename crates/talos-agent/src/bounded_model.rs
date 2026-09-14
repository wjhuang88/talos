//! Shared bounded, tool-free model invocation for isolated decisions.

use std::time::Duration;
use talos_core::message::{AgentEvent, Message, ReasoningBlock, StopReason};
use talos_core::provider::LanguageModel;
use tokio_util::sync::CancellationToken;

/// Outcome of an isolated bounded model decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedDecision {
    /// The model returned a bounded textual decision.
    Decision(String),
    /// The model produced no usable decision within the contract.
    Abstain(String),
    /// The invocation failed before a decision could be trusted.
    Failure(String),
}

/// Invoke a model with no tools and a strict output/deadline boundary.
pub async fn invoke_text(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    max_output_bytes: usize,
) -> Result<String, String> {
    match invoke_text_bounded(
        provider,
        messages,
        deadline,
        max_output_bytes,
        CancellationToken::new(),
    )
    .await
    {
        BoundedDecision::Decision(output) => Ok(output),
        BoundedDecision::Abstain(reason) | BoundedDecision::Failure(reason) => Err(reason),
    }
}

/// Invoke a model with explicit cancellation and a typed, fail-closed outcome.
pub async fn invoke_text_bounded(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    max_output_bytes: usize,
    cancellation: CancellationToken,
) -> BoundedDecision {
    let mut output = String::new();
    let mut thinking_bytes = 0usize;
    let mut reasoning_bytes = 0usize;
    let result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            return BoundedDecision::Failure("bounded model invocation cancelled".to_owned());
        }
        result = tokio::time::timeout(deadline, async {
        let mut events = provider.stream(messages).await.map_err(|_| "bounded model dispatch failed".to_owned())?;
        while let Some(event) = events.recv().await {
            let mut text_bytes = output.len();
            match &event {
                AgentEvent::TextDelta { delta } => text_bytes = text_bytes.saturating_add(delta.len()),
                AgentEvent::ThinkingDelta { delta } => thinking_bytes = thinking_bytes.saturating_add(delta.len()),
                AgentEvent::ReasoningComplete { blocks } => {
                    for block in blocks {
                        let size = match block {
                            ReasoningBlock::Thinking { text, signature } => text.len().saturating_add(signature.as_ref().map_or(0, String::len)),
                            ReasoningBlock::Redacted { data } => data.len(),
                            ReasoningBlock::Plain { text } => text.len(),
                        };
                        reasoning_bytes = reasoning_bytes.saturating_add(size);
                    }
                }
                _ => {}
            }
            // Thinking deltas and their completed replay blocks describe the same output.
            // Count the larger representation, including signatures and opaque payloads.
            if text_bytes.saturating_add(thinking_bytes.max(reasoning_bytes)) > max_output_bytes {
                return Err("bounded model output exceeded limit".to_owned());
            }
            match event {
                AgentEvent::TextDelta { delta } => {
                    output.push_str(&delta);
                }
                AgentEvent::ToolCall { .. } | AgentEvent::ToolCallStarted { .. } | AgentEvent::ToolResult { .. } => {
                    return Err("tool use is forbidden in auto assessment".to_owned());
                }
                AgentEvent::Error { .. } => return Err("bounded model stream failed".to_owned()),
                AgentEvent::TurnEnd { stop_reason: StopReason::EndTurn, .. } => return Ok(()),
                AgentEvent::TurnEnd { .. } => return Err("bounded model response incomplete".to_owned()),
                _ => {}
            }
        }
        Err("bounded model stream closed before completion".to_owned())
        }) => result,
    };
    match result {
        Err(_) => return BoundedDecision::Failure("bounded model deadline exceeded".to_owned()),
        Ok(Err(error)) => return BoundedDecision::Failure(error),
        Ok(Ok(())) => {}
    }
    if output.trim().is_empty() {
        return BoundedDecision::Abstain("bounded model returned no output".to_owned());
    }
    BoundedDecision::Decision(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::provider::{ProviderResult, Receiver};

    struct ResponseModel(Vec<AgentEvent>);

    #[async_trait::async_trait]
    impl LanguageModel for ResponseModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (tx, rx) = tokio::sync::mpsc::channel(self.0.len().max(1));
            for event in &self.0 {
                tx.send(event.clone()).await.expect("receiver alive");
            }
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn partial_decision_requires_successful_terminal_event() {
        for terminal in [None, Some(StopReason::MaxTokens), Some(StopReason::ToolUse)] {
            let mut events = vec![AgentEvent::TextDelta {
                delta: "fallback".into(),
            }];
            if let Some(stop_reason) = terminal {
                events.push(AgentEvent::TurnEnd {
                    stop_reason,
                    usage: Default::default(),
                });
            }
            assert!(matches!(
                invoke_text_bounded(
                    &ResponseModel(events),
                    &[],
                    Duration::from_secs(1),
                    128,
                    CancellationToken::new()
                )
                .await,
                BoundedDecision::Failure(_)
            ));
        }
        let model = ResponseModel(vec![
            AgentEvent::TextDelta {
                delta: "fallback".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Default::default(),
            },
        ]);
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 128).await,
            Ok("fallback".into())
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert_eq!(
            invoke_text_bounded(&model, &[], Duration::from_secs(1), 128, cancelled).await,
            BoundedDecision::Failure("bounded model invocation cancelled".into())
        );
    }

    #[tokio::test]
    async fn provider_error_text_is_not_a_decision_or_diagnostic() {
        let model = ResponseModel(vec![AgentEvent::Error {
            message: "no output: credential=secret-test-marker".into(),
        }]);
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 128).await,
            Err("bounded model stream failed".into())
        );
    }

    struct DelayedModel {
        dispatch: Duration,
        response: Duration,
    }

    #[tokio::test]
    async fn reasoning_and_text_share_a_budget_without_double_counting_deltas() {
        let end = AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Default::default(),
        };
        for reasoning in [
            AgentEvent::ThinkingDelta {
                delta: "12345678".into(),
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Plain {
                    text: "12345678".into(),
                }],
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Redacted {
                    data: "12345678".into(),
                }],
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Thinking {
                    text: "1".into(),
                    signature: Some("2345678".into()),
                }],
            },
        ] {
            let model = ResponseModel(vec![
                reasoning,
                AgentEvent::TextDelta { delta: "ok".into() },
                end.clone(),
            ]);
            assert_eq!(
                invoke_text(&model, &[], Duration::from_secs(1), 9).await,
                Err("bounded model output exceeded limit".into())
            );
        }
        let model = ResponseModel(vec![
            AgentEvent::ThinkingDelta {
                delta: "12345678".into(),
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Plain {
                    text: "12345678".into(),
                }],
            },
            AgentEvent::TextDelta { delta: "ok".into() },
            end,
        ]);
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 10).await,
            Ok("ok".into())
        );
    }

    #[async_trait::async_trait]
    impl LanguageModel for DelayedModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            tokio::time::sleep(self.dispatch).await;
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let response = self.response;
            tokio::spawn(async move {
                tokio::time::sleep(response).await;
                let _ = tx
                    .send(AgentEvent::TextDelta {
                        delta: "decision".into(),
                    })
                    .await;
            });
            Ok(rx)
        }
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_includes_initial_dispatch() {
        let model = DelayedModel {
            dispatch: Duration::from_secs(60),
            response: Duration::ZERO,
        };
        let started = tokio::time::Instant::now();
        let result = invoke_text(&model, &[], Duration::from_secs(1), 128).await;
        assert_eq!(result, Err("bounded model deadline exceeded".into()));
        assert_eq!(started.elapsed(), Duration::from_secs(1));
    }

    #[tokio::test(start_paused = true)]
    async fn dispatch_and_response_share_one_deadline() {
        let model = DelayedModel {
            dispatch: Duration::from_millis(600),
            response: Duration::from_millis(600),
        };
        let started = tokio::time::Instant::now();
        let result = invoke_text(&model, &[], Duration::from_secs(1), 128).await;
        assert_eq!(result, Err("bounded model deadline exceeded".into()));
        assert_eq!(started.elapsed(), Duration::from_secs(1));
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_is_distinguished_from_deadline() {
        let model = DelayedModel {
            dispatch: Duration::from_secs(60),
            response: Duration::ZERO,
        };
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result =
            invoke_text_bounded(&model, &[], Duration::from_secs(1), 128, cancellation).await;
        assert_eq!(
            result,
            BoundedDecision::Failure("bounded model invocation cancelled".into())
        );
    }
}
