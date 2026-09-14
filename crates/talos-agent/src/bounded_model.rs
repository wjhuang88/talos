//! Shared bounded, tool-free model invocation for isolated decisions.

use std::time::Duration;
use talos_core::message::{AgentEvent, Message};
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
    let result = tokio::select! {
        _ = cancellation.cancelled() => {
            return BoundedDecision::Failure("bounded model invocation cancelled".to_owned());
        }
        result = tokio::time::timeout(deadline, async {
        let mut events = provider.stream(messages).await.map_err(|e| e.to_string())?;
        while let Some(event) = events.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => {
                    if output.len().saturating_add(delta.len()) > max_output_bytes {
                        return Err("bounded model output exceeded limit".to_owned());
                    }
                    output.push_str(&delta);
                }
                AgentEvent::ToolCall { .. } => {
                    return Err("tool use is forbidden in auto assessment".to_owned());
                }
                AgentEvent::Error { message } => return Err(message),
                AgentEvent::TurnEnd { .. } => break,
                _ => {}
            }
        }
        Ok(())
        }) => result,
    };
    match result {
        Err(_) => return BoundedDecision::Failure("bounded model deadline exceeded".to_owned()),
        Ok(Err(error)) => {
            return if error.contains("no output") {
                BoundedDecision::Abstain(error)
            } else {
                BoundedDecision::Failure(error)
            };
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

    struct DelayedModel {
        dispatch: Duration,
        response: Duration,
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
