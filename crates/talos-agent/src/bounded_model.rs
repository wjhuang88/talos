//! Shared bounded, tool-free model invocation for isolated decisions.

use std::time::Duration;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::LanguageModel;

/// Invoke a model with no tools and a strict output/deadline boundary.
pub async fn invoke_text(
    provider: &dyn LanguageModel,
    messages: &[Message],
    deadline: Duration,
    max_output_bytes: usize,
) -> Result<String, String> {
    let mut output = String::new();
    tokio::time::timeout(deadline, async {
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
    })
    .await
    .map_err(|_| "bounded model deadline exceeded".to_owned())??;
    if output.trim().is_empty() {
        return Err("bounded model returned no output".to_owned());
    }
    Ok(output)
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
}
