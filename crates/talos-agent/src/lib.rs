//! Talos agent — core orchestration logic and the agent turn loop.

use std::sync::Arc;

use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderError};
use thiserror::Error;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

/// Errors that can occur during agent execution.
#[derive(Debug, Error)]
pub enum AgentError {
    /// An error from the underlying LLM provider.
    #[error("provider error: {0}")]
    ProviderError(#[from] ProviderError),

    /// The turn was cancelled via [`CancellationToken`].
    #[error("turn cancelled")]
    Cancelled,

    /// An unexpected event sequence was received.
    #[error("unexpected event: {0}")]
    UnexpectedEvent(String),
}

/// Result alias for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// The agent orchestrates a single turn: takes a user message, calls the LLM
/// provider, streams events, and returns the assistant response.
///
/// # Example
///
/// ```no_run
/// use talos_agent::Agent;
/// use std::sync::Arc;
/// # use talos_core::provider::{LanguageModel, ProviderResult, Receiver};
/// # use talos_core::message::{AgentEvent, Message};
/// # struct MyModel;
/// # #[async_trait::async_trait]
/// # impl LanguageModel for MyModel {
/// #     async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> { unimplemented!() }
/// # }
/// # async fn example() {
/// let provider: Arc<dyn LanguageModel> = Arc::new(MyModel);
/// let agent = Agent::new(provider);
/// let response = agent.run("Hello!".into()).await.unwrap();
/// # }
/// ```
pub struct Agent {
    /// The language model provider used for this agent.
    provider: Arc<dyn LanguageModel>,
}

impl Agent {
    /// Creates a new agent with the given language model provider.
    #[must_use]
    pub fn new(provider: Arc<dyn LanguageModel>) -> Self {
        Self { provider }
    }

    /// Runs a single turn with the given user message and returns the complete
    /// assistant response.
    ///
    /// This method collects all `TextDelta` events internally and returns the
    /// concatenated response string when `TurnEnd` is received.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError::ProviderError`] if the provider fails,
    /// [`AgentError::Cancelled`] if the cancellation token is triggered,
    /// or [`AgentError::UnexpectedEvent`] if an error event is received.
    pub async fn run(&self, user_message: String) -> AgentResult<String> {
        self.run_inner(user_message, None).await
    }

    /// Runs a single turn with streaming events forwarded to the given
    /// broadcast channel.
    ///
    /// This method behaves like [`Agent::run`] but also sends every
    /// [`AgentEvent`] to `event_tx`, allowing external consumers to receive
    /// real-time updates (e.g., for UI streaming).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError::ProviderError`] if the provider fails,
    /// [`AgentError::Cancelled`] if the cancellation token is triggered,
    /// or [`AgentError::UnexpectedEvent`] if an error event is received.
    pub async fn run_streaming(
        &self,
        user_message: String,
        event_tx: broadcast::Sender<AgentEvent>,
    ) -> AgentResult<String> {
        self.run_inner(user_message, Some(event_tx)).await
    }

    /// Internal implementation shared by [`run`] and [`run_streaming`].
    async fn run_inner(
        &self,
        user_message: String,
        event_tx: Option<broadcast::Sender<AgentEvent>>,
    ) -> AgentResult<String> {
        let message = Message::User {
            content: user_message,
        };

        let mut rx = self.provider.stream(&[message]).await?;

        let mut response = String::new();

        while let Some(event) = rx.recv().await {
            if let Some(ref tx) = event_tx {
                let _ = tx.send(event.clone());
            }

            match event {
                AgentEvent::TextDelta { delta } => {
                    response.push_str(&delta);
                }
                AgentEvent::TurnEnd { .. } => {
                    return Ok(response);
                }
                AgentEvent::Error { message } => {
                    return Err(AgentError::UnexpectedEvent(message));
                }
                AgentEvent::TurnStart | AgentEvent::ToolCall { .. } | AgentEvent::ToolResult { .. } => {}
            }
        }

        Err(AgentError::UnexpectedEvent(
            "channel closed before TurnEnd".into(),
        ))
    }

    /// Returns a [`CancellationToken`] that can be used to cancel the current
    /// turn. The caller is responsible for storing and triggering this token.
    ///
    /// Note: The token itself does not interrupt the provider stream; it is
    /// provided for the caller to coordinate cancellation at a higher level.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        CancellationToken::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{StopReason, Usage};
    use talos_core::provider::ProviderResult;
    use tokio::sync::mpsc;

    /// Mock language model that returns a predefined sequence of events.
    struct MockModel {
        events: Vec<AgentEvent>,
    }

    impl MockModel {
        fn new(events: Vec<AgentEvent>) -> Self {
            Self { events }
        }
    }

    #[async_trait::async_trait]
    impl LanguageModel for MockModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (tx, rx) = mpsc::channel(32);
            for event in &self.events {
                tx.send(event.clone()).await.expect("receiver dropped");
            }
            Ok(rx)
        }
    }

    type Receiver<T> = mpsc::Receiver<T>;

    #[tokio::test]
    async fn test_run_collects_text_deltas() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Hello, ".into(),
            },
            AgentEvent::TextDelta {
                delta: "world!".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(events)));
        let response = agent.run("Hi".into()).await.unwrap();
        assert_eq!(response, "Hello, world!");
    }

    #[tokio::test]
    async fn test_run_handles_error_event() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::Error {
                message: "something went wrong".into(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(events)));
        let result = agent.run("Hi".into()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AgentError::UnexpectedEvent(_)));
    }

    #[tokio::test]
    async fn test_run_handles_channel_close_without_turn_end() {
        let agent = Agent::new(Arc::new(MockModel::new(vec![])));
        let result = agent.run("Hi".into()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AgentError::UnexpectedEvent(_)));
    }

    #[tokio::test]
    async fn test_run_streaming_forwards_events() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Streaming".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(events.clone())));
        let (tx, mut rx) = broadcast::channel::<AgentEvent>(32);

        let response = agent.run_streaming("Hi".into(), tx).await.unwrap();
        assert_eq!(response, "Streaming");

        // Verify events were broadcast
        let mut received = Vec::new();
        while let Ok(event) = rx.try_recv() {
            received.push(event);
        }
        assert_eq!(received.len(), events.len());
        assert_eq!(received, events);
    }

    #[tokio::test]
    async fn test_run_ignores_tool_events_in_basic_loop() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Let me ".into(),
            },
            AgentEvent::ToolCall {
                call: talos_core::message::ToolCall {
                    id: "call_1".into(),
                    name: "read_file".into(),
                    input: serde_json::json!({"path": "test.rs"}),
                },
            },
            AgentEvent::TextDelta {
                delta: "check that.".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(events)));
        let response = agent.run("Read test.rs".into()).await.unwrap();
        // ToolCall events don't contribute to the response string
        assert_eq!(response, "Let me check that.");
    }

    #[tokio::test]
    async fn test_cancellation_token_is_created() {
        let agent = Agent::new(Arc::new(MockModel::new(vec![])));
        let token = agent.cancellation_token();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}
