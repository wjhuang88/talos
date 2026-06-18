//! Mock LLM provider for testing.
//!
//! Provides a configurable [`MockProvider`] that simulates various LLM responses
//! by queuing predefined events and returning them in FIFO order. Useful for
//! unit tests and integration tests that need deterministic LLM behavior.
//!
//! # Example
//!
//! ```rust
//! use talos_provider::mock::MockProvider;
//! use talos_core::provider::LanguageModel;
//!
//! #[tokio::main]
//! async fn main() {
//!     let provider = MockProvider::new()
//!         .with_response("Hello from mock!")
//!         .with_tool_call("read_file", serde_json::json!({"path": "test.rs"}));
//!
//!     let mut rx = provider.stream(&[]).await.unwrap();
//!     // Consume events from rx...
//! }
//! ```

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use talos_core::message::{AgentEvent, Message, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderResult};
use tokio::sync::mpsc;

/// User prompt that makes the mock provider print the would-be model request.
pub const REQUEST_DEBUG_COMMAND: &str = "/mock-request";

const DEFAULT_MOCK_RESPONSE: &str =
    "I'm a mock LLM. I can help with testing and development without making real API calls.";

type RequestDebugBuilder = Arc<dyn Fn(&[Message]) -> String + Send + Sync>;

/// A queued event template that the mock provider will emit.
#[derive(Debug, Clone)]
enum QueuedEvent {
    /// Emit one or more text deltas as streaming chunks.
    Streaming(Vec<String>),
    /// Emit a single text response (wrapped as a single delta).
    Text(String),
    /// Emit a tool_use event.
    ToolCall {
        name: String,
        input: serde_json::Value,
    },
    /// Emit an error event with the given message.
    Error(String),
}

/// Shared state for the mock provider, wrapped in Arc for thread-safety.
#[derive(Debug)]
struct MockState {
    queue: VecDeque<QueuedEvent>,
}

/// Mock implementation of [`LanguageModel`] for testing.
///
/// Responses are queued and returned in FIFO order. When the queue is empty,
/// a default `"I'm a mock LLM"` response is returned.
///
/// # Builder Pattern
///
/// Use the builder methods to configure behavior before calling [`stream`]:
///
/// - [`with_response`](MockProvider::with_response) — queue a text response
/// - [`with_tool_call`](MockProvider::with_tool_call) — queue a tool_use response
/// - [`with_error`](MockProvider::with_error) — queue an error response
/// - [`with_streaming`](MockProvider::with_streaming) — queue streaming text deltas
///
/// # Thread Safety
///
/// `MockProvider` is `Send + Sync` and can be shared across async tasks.
/// Each call to [`stream`] consumes one item from the queue.
#[derive(Clone)]
pub struct MockProvider {
    state: Arc<Mutex<MockState>>,
    request_debug_builder: Option<RequestDebugBuilder>,
}

impl MockProvider {
    /// Create a new empty mock provider.
    ///
    /// The provider starts with an empty queue. When [`stream`] is called
    /// with no queued events, it returns a default `"I'm a mock LLM"` response.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                queue: VecDeque::new(),
            })),
            request_debug_builder: None,
        }
    }

    /// Attach a renderer for provider-specific request diagnostics.
    #[must_use]
    pub fn with_request_debug_builder(
        mut self,
        builder: impl Fn(&[Message]) -> String + Send + Sync + 'static,
    ) -> Self {
        self.request_debug_builder = Some(Arc::new(builder));
        self
    }

    /// Queue a text response.
    ///
    /// When consumed, this will emit: `TurnStart` → `TextDelta { delta: text }` → `TurnEnd`.
    #[must_use]
    pub fn with_response(self, text: impl Into<String>) -> Self {
        self.state
            .lock()
            .expect("mock state lock poisoned")
            .queue
            .push_back(QueuedEvent::Text(text.into()));
        self
    }

    /// Queue a tool_use response.
    ///
    /// When consumed, this will emit: `TurnStart` → `ToolCall { call }` → `TurnEnd`
    /// with [`StopReason::ToolUse`].
    ///
    /// The tool call ID is auto-generated as `"mock_call_N"` where N is an
    /// incrementing counter.
    #[must_use]
    pub fn with_tool_call(self, name: impl Into<String>, input: serde_json::Value) -> Self {
        self.state
            .lock()
            .expect("mock state lock poisoned")
            .queue
            .push_back(QueuedEvent::ToolCall {
                name: name.into(),
                input,
            });
        self
    }

    /// Queue an error response.
    ///
    /// When consumed, this will emit: `TurnStart` → `Error { message }` → `TurnEnd`.
    ///
    /// The `status_code` parameter is used to generate a realistic error message:
    /// - `401` → "authentication failed"
    /// - `429` → "rate limited"
    /// - `500` → "internal server error"
    /// - Other → "mock error (status: N)"
    #[must_use]
    pub fn with_error(self, status_code: u16) -> Self {
        let message = match status_code {
            401 => "authentication failed: invalid API key".into(),
            429 => "rate limited: too many requests".into(),
            500 => "internal server error".into(),
            other => format!("mock error (status: {other})"),
        };
        self.state
            .lock()
            .expect("mock state lock poisoned")
            .queue
            .push_back(QueuedEvent::Error(message));
        self
    }

    /// Queue a streaming text response with multiple deltas.
    ///
    /// When consumed, this will emit: `TurnStart` → `TextDelta` for each chunk → `TurnEnd`.
    #[must_use]
    pub fn with_streaming(self, chunks: Vec<String>) -> Self {
        self.state
            .lock()
            .expect("mock state lock poisoned")
            .queue
            .push_back(QueuedEvent::Streaming(chunks));
        self
    }

    /// Drain all queued events and return them as a vector.
    ///
    /// Useful for inspecting the current queue state in tests.
    pub fn queue_len(&self) -> usize {
        self.state
            .lock()
            .expect("mock state lock poisoned")
            .queue
            .len()
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LanguageModel for MockProvider {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(32);

        let event = {
            let mut state = self.state.lock().expect("mock state lock poisoned");
            state.queue.pop_front()
        };
        let debug_messages = request_debug_messages(messages);
        let request_debug = if event.is_none() {
            self.request_debug_builder
                .as_ref()
                .and_then(|builder| debug_messages.as_ref().map(|messages| builder(messages)))
                .or_else(|| debug_messages.map(|_| "request diagnostics are not configured".into()))
        } else {
            None
        };

        tokio::spawn(async move {
            match (event, request_debug) {
                (Some(QueuedEvent::Text(text)), _) => {
                    emit_text_response(&tx, &text).await;
                }
                (Some(QueuedEvent::ToolCall { name, input }), _) => {
                    emit_tool_call_response(&tx, &name, input).await;
                }
                (Some(QueuedEvent::Error(message)), _) => {
                    emit_error_response(&tx, &message).await;
                }
                (Some(QueuedEvent::Streaming(chunks)), _) => {
                    emit_streaming_response(&tx, &chunks).await;
                }
                (None, Some(debug_text)) => {
                    emit_text_response(&tx, &debug_text).await;
                }
                (None, None) => {
                    emit_default_response(&tx).await;
                }
            }
        });

        Ok(rx)
    }

    fn request_preview(&self, messages: &[Message]) -> Option<Value> {
        let debug_messages = request_debug_messages(messages)?;
        self.request_debug_builder.as_ref().map(|builder| {
            serde_json::from_str::<Value>(&builder(&debug_messages)).unwrap_or(Value::Null)
        })
    }
}

fn request_debug_messages(messages: &[Message]) -> Option<Vec<Message>> {
    let mut debug_messages = messages.to_vec();
    let Message::User { content } = debug_messages.last_mut()? else {
        return None;
    };

    let stripped_content = strip_request_debug_command(content)?;
    *content = stripped_content;

    Some(debug_messages)
}

fn strip_request_debug_command(content: &str) -> Option<String> {
    let mut lines: Vec<&str> = content.lines().collect();
    let command_line_index = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with(REQUEST_DEBUG_COMMAND))?;
    let command_line = lines[command_line_index];
    let command_offset = command_line.find(REQUEST_DEBUG_COMMAND)?;
    let after_command = command_line[command_offset + REQUEST_DEBUG_COMMAND.len()..].trim_start();

    if after_command.is_empty() {
        lines.remove(command_line_index);
    } else {
        lines[command_line_index] = after_command;
    }

    Some(lines.join("\n").trim_end().to_string())
}

async fn emit_text_response(tx: &mpsc::Sender<AgentEvent>, text: &str) {
    let _ = tx.send(AgentEvent::TurnStart).await;
    let _ = tx
        .send(AgentEvent::TextDelta {
            delta: text.to_string(),
        })
        .await;
    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        })
        .await;
}

async fn emit_tool_call_response(
    tx: &mpsc::Sender<AgentEvent>,
    name: &str,
    input: serde_json::Value,
) {
    let _ = tx.send(AgentEvent::TurnStart).await;
    let _ = tx
        .send(AgentEvent::ToolCall {
            call: ToolCall {
                id: "mock_call_1".into(),
                name: name.to_string(),
                input,
            },
            provenance: Default::default(),
            summary_fields: vec![],
        })
        .await;
    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
        })
        .await;
}

async fn emit_error_response(tx: &mpsc::Sender<AgentEvent>, message: &str) {
    let _ = tx.send(AgentEvent::TurnStart).await;
    let _ = tx
        .send(AgentEvent::Error {
            message: message.to_string(),
        })
        .await;
    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        })
        .await;
}

async fn emit_streaming_response(tx: &mpsc::Sender<AgentEvent>, chunks: &[String]) {
    let _ = tx.send(AgentEvent::TurnStart).await;
    for chunk in chunks {
        let _ = tx
            .send(AgentEvent::TextDelta {
                delta: chunk.clone(),
            })
            .await;
    }
    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        })
        .await;
}

async fn emit_default_response(tx: &mpsc::Sender<AgentEvent>) {
    let _ = tx.send(AgentEvent::TurnStart).await;
    let _ = tx
        .send(AgentEvent::TextDelta {
            delta: DEFAULT_MOCK_RESPONSE.into(),
        })
        .await;
    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::provider::LanguageModel;

    async fn collect_events(mut rx: mpsc::Receiver<AgentEvent>) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    }

    #[tokio::test]
    async fn test_text_response() {
        let provider = MockProvider::new().with_response("Hello, world!");
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], AgentEvent::TurnStart));
        assert!(matches!(
            &events[1],
            AgentEvent::TextDelta { delta } if delta == "Hello, world!"
        ));
        assert!(matches!(
            &events[2],
            AgentEvent::TurnEnd { stop_reason, .. } if matches!(stop_reason, StopReason::EndTurn)
        ));
    }

    #[tokio::test]
    async fn test_tool_call_response() {
        let provider =
            MockProvider::new().with_tool_call("read_file", serde_json::json!({"path": "test.rs"}));
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], AgentEvent::TurnStart));
        match &events[1] {
            AgentEvent::ToolCall { call, .. } => {
                assert_eq!(call.name, "read_file");
                assert_eq!(call.input, serde_json::json!({"path": "test.rs"}));
            }
            other => panic!("expected ToolCall, got {other:?}"),
        }
        assert!(matches!(
            &events[2],
            AgentEvent::TurnEnd { stop_reason, .. } if matches!(stop_reason, StopReason::ToolUse)
        ));
    }

    #[tokio::test]
    async fn test_error_response_401() {
        let provider = MockProvider::new().with_error(401);
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        match &events[1] {
            AgentEvent::Error { message } => {
                assert!(message.contains("authentication failed"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_error_response_429() {
        let provider = MockProvider::new().with_error(429);
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        match &events[1] {
            AgentEvent::Error { message } => {
                assert!(message.contains("rate limited"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_error_response_500() {
        let provider = MockProvider::new().with_error(500);
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        match &events[1] {
            AgentEvent::Error { message } => {
                assert!(message.contains("internal server error"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let provider =
            MockProvider::new().with_streaming(vec!["Hello".into(), ", ".into(), "world!".into()]);
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 5); // TurnStart + 3 deltas + TurnEnd
        assert!(matches!(events[0], AgentEvent::TurnStart));
        assert!(matches!(
            &events[1], AgentEvent::TextDelta { delta } if delta == "Hello"
        ));
        assert!(matches!(
            &events[2], AgentEvent::TextDelta { delta } if delta == ", "
        ));
        assert!(matches!(
            &events[3], AgentEvent::TextDelta { delta } if delta == "world!"
        ));
        assert!(matches!(events[4], AgentEvent::TurnEnd { .. }));
    }

    #[tokio::test]
    async fn test_queue_fifo_ordering() {
        let provider = MockProvider::new()
            .with_response("first")
            .with_response("second")
            .with_response("third");

        // First call
        let rx1 = provider.stream(&[]).await.expect("stream should succeed");
        let events1 = collect_events(rx1).await;
        assert!(matches!(
            &events1[1], AgentEvent::TextDelta { delta } if delta == "first"
        ));

        // Second call
        let rx2 = provider.stream(&[]).await.expect("stream should succeed");
        let events2 = collect_events(rx2).await;
        assert!(matches!(
            &events2[1], AgentEvent::TextDelta { delta } if delta == "second"
        ));

        // Third call
        let rx3 = provider.stream(&[]).await.expect("stream should succeed");
        let events3 = collect_events(rx3).await;
        assert!(matches!(
            &events3[1], AgentEvent::TextDelta { delta } if delta == "third"
        ));
    }

    #[tokio::test]
    async fn test_empty_queue_default_response() {
        let provider = MockProvider::new();
        let rx = provider.stream(&[]).await.expect("stream should succeed");
        let events = collect_events(rx).await;

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], AgentEvent::TurnStart));
        assert!(matches!(
            &events[1],
            AgentEvent::TextDelta { delta } if delta == DEFAULT_MOCK_RESPONSE
        ));
        assert!(matches!(events[2], AgentEvent::TurnEnd { .. }));
    }

    #[tokio::test]
    async fn request_debug_command_uses_configured_builder() {
        let provider =
            MockProvider::new().with_request_debug_builder(|messages| match messages.last() {
                Some(Message::User { content }) => content.clone(),
                other => format!("unexpected: {other:?}"),
            });
        let messages = vec![Message::User {
            content: format!("system prompt\n\n{REQUEST_DEBUG_COMMAND} explain this code"),
        }];

        let rx = provider
            .stream(&messages)
            .await
            .expect("stream should succeed");
        let events = collect_events(rx).await;

        assert!(matches!(
            &events[1],
            AgentEvent::TextDelta { delta } if delta == "system prompt\n\nexplain this code"
        ));
    }

    #[tokio::test]
    async fn request_debug_command_strips_multiline_wrapper() {
        let provider =
            MockProvider::new().with_request_debug_builder(|messages| match messages.last() {
                Some(Message::User { content }) => content.clone(),
                other => format!("unexpected: {other:?}"),
            });
        let messages = vec![Message::User {
            content: format!("system prompt\n\n{REQUEST_DEBUG_COMMAND}\nexplain this code"),
        }];

        let rx = provider
            .stream(&messages)
            .await
            .expect("stream should succeed");
        let events = collect_events(rx).await;

        assert!(matches!(
            &events[1],
            AgentEvent::TextDelta { delta } if delta == "system prompt\n\nexplain this code"
        ));
    }

    #[tokio::test]
    async fn request_debug_command_does_not_override_queued_events() {
        let provider = MockProvider::new()
            .with_response("queued")
            .with_request_debug_builder(|_| "debug".into());
        let messages = vec![Message::User {
            content: REQUEST_DEBUG_COMMAND.into(),
        }];

        let rx = provider
            .stream(&messages)
            .await
            .expect("stream should succeed");
        let events = collect_events(rx).await;

        assert!(matches!(
            &events[1],
            AgentEvent::TextDelta { delta } if delta == "queued"
        ));
    }

    #[tokio::test]
    async fn test_mixed_queue() {
        let provider = MockProvider::new()
            .with_response("text response")
            .with_tool_call("bash", serde_json::json!({"command": "ls"}))
            .with_error(401)
            .with_streaming(vec!["streaming".into(), " response".into()]);

        // 1: text response
        let rx = provider.stream(&[]).await.unwrap();
        let events = collect_events(rx).await;
        assert!(matches!(
            &events[1], AgentEvent::TextDelta { delta } if delta == "text response"
        ));

        // 2: tool call
        let rx = provider.stream(&[]).await.unwrap();
        let events = collect_events(rx).await;
        assert!(matches!(&events[1], AgentEvent::ToolCall { call, .. } if call.name == "bash"));

        // 3: error
        let rx = provider.stream(&[]).await.unwrap();
        let events = collect_events(rx).await;
        assert!(
            matches!(&events[1], AgentEvent::Error { message } if message.contains("authentication"))
        );

        // 4: streaming
        let rx = provider.stream(&[]).await.unwrap();
        let events = collect_events(rx).await;
        assert_eq!(events.len(), 4); // TurnStart + 2 deltas + TurnEnd
    }

    #[tokio::test]
    async fn test_queue_len() {
        let provider = MockProvider::new().with_response("a").with_response("b");

        assert_eq!(provider.queue_len(), 2);

        // Consume one
        let rx = provider.stream(&[]).await.unwrap();
        let _ = collect_events(rx).await;
        assert_eq!(provider.queue_len(), 1);
    }
}
