//! Shared bounded, tool-free model invocation for isolated decisions.

use futures_util::FutureExt;
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

/// Explicit provenance and isolation contract for one bounded decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedDecisionContext {
    /// Stable identifier used to correlate diagnostics without exposing prompt data.
    pub correlation_id: String,
    /// Caller-owned purpose label, such as `permission-review` or `protocol-recovery`.
    pub purpose: String,
    /// Explicitly allowed tools. An empty list is the normal tool-free policy.
    pub dedicated_tools: Vec<String>,
    /// Maximum provider retry dispatches for this isolated request.
    pub max_retries: u32,
}

impl BoundedDecisionContext {
    /// Creates a tool-free, non-retrying context with a caller-supplied correlation ID.
    pub fn new(correlation_id: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            correlation_id: correlation_id.into(),
            purpose: purpose.into(),
            dedicated_tools: Vec::new(),
            max_retries: 0,
        }
    }
}

/// Invoke a model with no tools and a strict output/deadline boundary.
///
/// The provider receives a 4096-token generation budget. `max_output_bytes`
/// independently bounds UTF-8 text plus reasoning (including opaque signatures).
#[cfg(test)]
async fn invoke_text(
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

/// Invoke a tool-free Auto review using the adapter's request-local reasoning policy.
pub(crate) async fn invoke_auto_review(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    max_output_bytes: usize,
) -> Result<String, String> {
    match invoke_text_bounded_with_limits(
        provider,
        messages,
        deadline,
        talos_core::provider::DecisionRequestLimits {
            max_output_tokens: 4096,
            max_retries: 0,
        },
        max_output_bytes,
        CancellationToken::new(),
        true,
    )
    .await
    {
        BoundedDecision::Decision(output) => Ok(output),
        BoundedDecision::Abstain(reason) | BoundedDecision::Failure(reason) => Err(reason),
    }
}

/// Invoke a bounded decision with an explicit isolation and provenance contract.
pub async fn invoke_text_with_context(
    provider: &dyn LanguageModel,
    messages: &[Message],
    context: &BoundedDecisionContext,
    deadline: Duration,
    max_output_bytes: usize,
    cancellation: CancellationToken,
) -> BoundedDecision {
    let limits = talos_core::provider::DecisionRequestLimits {
        max_output_tokens: 4096,
        max_retries: context.max_retries,
    };
    // The provider receives only caller-supplied messages; session history and tools are
    // never inherited. Reject empty correlation/purpose labels before dispatch.
    if context.correlation_id.trim().is_empty() || context.purpose.trim().is_empty() {
        return BoundedDecision::Failure("bounded decision context is incomplete".to_owned());
    }
    if !context.dedicated_tools.is_empty() {
        return BoundedDecision::Failure(
            "bounded decision dedicated tools are not enabled for this caller".to_owned(),
        );
    }
    invoke_text_bounded_with_limits(
        provider,
        messages,
        deadline,
        limits,
        max_output_bytes,
        cancellation,
        false,
    )
    .await
}

/// Invoke a model with explicit cancellation and a typed, fail-closed outcome.
#[cfg(test)]
async fn invoke_text_bounded(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    max_output_bytes: usize,
    cancellation: CancellationToken,
) -> BoundedDecision {
    invoke_text_bounded_with_limits(
        provider,
        messages,
        deadline,
        talos_core::provider::DecisionRequestLimits {
            max_output_tokens: 4096,
            max_retries: 0,
        },
        max_output_bytes,
        cancellation,
        false,
    )
    .await
}

async fn invoke_text_bounded_with_limits(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    limits: talos_core::provider::DecisionRequestLimits,
    max_output_bytes: usize,
    cancellation: CancellationToken,
    auto_review: bool,
) -> BoundedDecision {
    let mut output = String::new();
    let mut text_bytes = 0usize;
    let mut thinking_bytes = 0usize;
    let mut reasoning_bytes = 0usize;
    let result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            return BoundedDecision::Failure("bounded model invocation cancelled".to_owned());
        }
        result = tokio::time::timeout(deadline, async {
        // Catch both construction and polling panics from third-party implementations.
        // Never expose panic payloads or retry a request whose dispatch state is unknown.
        let mut events = std::panic::AssertUnwindSafe(async {
            if auto_review {
                provider.stream_auto_review(messages, limits).await
            } else {
                provider.stream_decision(messages, limits).await
            }
        }).catch_unwind().await
            .map_err(|_| "bounded model provider panicked".to_owned())?
            .map_err(|_| "bounded model dispatch failed".to_owned())?;
        while let Some(event) = events.recv().await {
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
            if text_bytes.saturating_add(thinking_bytes.max(reasoning_bytes))
                > max_output_bytes
            {
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
        Err(_) => {
            tracing::warn!(
                reason = "deadline",
                text_bytes,
                thinking_bytes,
                reasoning_bytes,
                max_output_bytes,
                max_output_tokens = limits.max_output_tokens,
                "Bounded decision failed"
            );
            return BoundedDecision::Failure("bounded model deadline exceeded".to_owned());
        }
        Ok(Err(error)) => {
            // `error` is constructed locally from fixed strings, never provider text.
            tracing::warn!(reason = %error, text_bytes, thinking_bytes, reasoning_bytes, max_output_bytes, max_output_tokens = limits.max_output_tokens, "Bounded decision failed");
            return BoundedDecision::Failure(error);
        }
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

    struct AutoOnlyModel;

    #[async_trait::async_trait]
    impl LanguageModel for AutoOnlyModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            panic!("Auto must not call unrestricted stream")
        }

        async fn stream_auto_review(
            &self,
            _: &[Message],
            limits: talos_core::provider::DecisionRequestLimits,
        ) -> ProviderResult<Receiver<AgentEvent>> {
            assert_eq!(limits.max_retries, 0);
            assert_eq!(limits.max_output_tokens, 4096);
            ResponseModel(vec![
                AgentEvent::TextDelta {
                    delta: "auto decision".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Default::default(),
                },
            ])
            .stream(&[])
            .await
        }
    }

    #[tokio::test]
    async fn auto_review_dispatch_is_distinct_from_generic_decisions() {
        assert_eq!(
            invoke_auto_review(&AutoOnlyModel, &[], Duration::from_secs(1), 128).await,
            Ok("auto decision".into())
        );
        assert_eq!(
            invoke_text(&AutoOnlyModel, &[], Duration::from_secs(1), 128).await,
            Err("bounded model dispatch failed".into())
        );
    }

    #[tokio::test]
    async fn utf8_byte_budget_is_independent_of_provider_token_limit() {
        // 6 KiB of UTF-8 is valid under an 8 KiB byte budget, even though
        // the provider token budget is 4096. Reasoning replay is not counted twice.
        let model = ResponseModel(vec![
            AgentEvent::ThinkingDelta {
                delta: "思".repeat(2048),
            },
            AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Plain {
                    text: "思".repeat(2048),
                }],
            },
            AgentEvent::TextDelta { delta: "ok".into() },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Default::default(),
            },
        ]);
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 8192).await,
            Ok("ok".into())
        );
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 6145).await,
            Err("bounded model output exceeded limit".into())
        );
        let context = BoundedDecisionContext::new("test-request", "permission-review");
        assert_eq!(
            invoke_text_with_context(
                &model,
                &[],
                &context,
                Duration::from_secs(1),
                8192,
                CancellationToken::new()
            )
            .await,
            BoundedDecision::Decision("ok".into())
        );
    }

    #[async_trait::async_trait]
    impl LanguageModel for ResponseModel {
        async fn stream_decision(
            &self,
            messages: &[Message],
            limits: talos_core::provider::DecisionRequestLimits,
        ) -> ProviderResult<Receiver<AgentEvent>> {
            assert_eq!(limits.max_retries, 0);
            assert!(limits.max_output_tokens > 0 && limits.max_output_tokens <= 4096);
            self.stream(messages).await
        }

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

    struct PanickingModel(std::sync::atomic::AtomicUsize);

    #[async_trait::async_trait]
    impl LanguageModel for PanickingModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            panic!("unbounded entry must not be called")
        }

        async fn stream_decision(
            &self,
            _: &[Message],
            _: talos_core::provider::DecisionRequestLimits,
        ) -> ProviderResult<Receiver<AgentEvent>> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tokio::task::yield_now().await;
            panic!("untrusted provider payload")
        }
    }

    #[tokio::test]
    async fn provider_panic_is_a_sanitized_failure_without_retry() {
        let model = PanickingModel(std::sync::atomic::AtomicUsize::new(0));
        assert_eq!(
            invoke_text(&model, &[], Duration::from_secs(1), 128).await,
            Err("bounded model provider panicked".into())
        );
        assert_eq!(model.0.load(std::sync::atomic::Ordering::SeqCst), 1);
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
        async fn stream_decision(
            &self,
            messages: &[Message],
            limits: talos_core::provider::DecisionRequestLimits,
        ) -> ProviderResult<Receiver<AgentEvent>> {
            assert_eq!(limits.max_retries, 0);
            assert!(limits.max_output_tokens > 0 && limits.max_output_tokens <= 4096);
            self.stream(messages).await
        }

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
