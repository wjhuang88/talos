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
    let mut events = provider.stream(messages).await.map_err(|e| e.to_string())?;
    let mut output = String::new();
    tokio::time::timeout(deadline, async {
        while let Some(event) = events.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => {
                    if output.len().saturating_add(delta.len()) > max_output_bytes {
                        return Err("bounded model output exceeded limit".to_owned());
                    }
                    output.push_str(&delta);
                }
                AgentEvent::ToolCall { .. } => return Err("tool use is forbidden in auto assessment".to_owned()),
                AgentEvent::Error { message } => return Err(message),
                AgentEvent::TurnEnd { .. } => break,
                _ => {}
            }
        }
        Ok(())
    }).await.map_err(|_| "bounded model deadline exceeded".to_owned())??;
    if output.trim().is_empty() { return Err("bounded model returned no output".to_owned()); }
    Ok(output)
}
