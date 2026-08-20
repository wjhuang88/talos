//! Provider trait and error types for LLM backends.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::message::{AgentEvent, Message};

pub type Receiver<T> = mpsc::Receiver<T>;

/// Non-secret, request-local progress reported by a language-model provider.
///
/// Retry ordinals are zero-based: `0` is the initial dispatch and positive values are the exact
/// ordinals returned by the provider's retry policy. Progress is transient and must not be
/// persisted as conversation history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProviderProgress {
    /// The initial request is being dispatched.
    InitialDispatch {
        /// Zero-based retry ordinal. This is always `0` for the initial dispatch.
        attempt: u32,
        /// Configured maximum retry ordinal.
        max_attempts: u32,
    },
    /// A retry dispatch is being attempted after its scheduled backoff.
    RetryDispatch {
        /// Zero-based retry ordinal returned by the provider retry decision.
        attempt: u32,
        /// Configured maximum retry ordinal.
        max_attempts: u32,
    },
    /// A bounded retry backoff has been scheduled.
    ScheduledBackoff {
        /// Retry ordinal that will be dispatched after the backoff.
        attempt: u32,
        /// Configured maximum retry ordinal.
        max_attempts: u32,
        /// Actual bounded delay selected by the provider retry policy.
        delay_ms: u64,
    },
    /// Response headers arrived and the provider is waiting for the first stream packet.
    FirstPacketWait {
        /// Zero-based retry ordinal whose response is being streamed.
        attempt: u32,
        /// Configured maximum retry ordinal.
        max_attempts: u32,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl ToolDefinition {
    /// Creates a new tool definition.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    /// Formats this tool definition as a text block suitable for inclusion
    /// in the system prompt.
    #[must_use]
    pub fn to_prompt_text(&self) -> String {
        format!(
            "## {}\n{}\nParameters: {}",
            self.name,
            self.description,
            serde_json::to_string_pretty(&self.parameters).unwrap_or_default()
        )
    }
}

#[async_trait::async_trait]
pub trait LanguageModel: Send + Sync {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>>;

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<Receiver<AgentEvent>> {
        let _ = tools;
        self.stream(messages).await
    }

    /// Streams a response while optionally reporting typed request-local provider progress.
    ///
    /// The default preserves source compatibility for third-party providers by delegating to
    /// [`LanguageModel::stream_with_tools`] without emitting progress.
    async fn stream_with_tools_and_progress(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        drop(progress_tx);
        self.stream_with_tools(messages, tools).await
    }

    fn request_preview(&self, _messages: &[Message]) -> Option<Value> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LegacyModel;

    #[async_trait::async_trait]
    impl LanguageModel for LegacyModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (_tx, rx) = mpsc::channel(1);
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn legacy_provider_uses_default_progress_aware_entrypoint() {
        let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();
        let result = LegacyModel
            .stream_with_tools_and_progress(&[], &[], progress_tx)
            .await;

        assert!(result.is_ok());
        assert_eq!(progress_rx.recv().await, None);
    }

    #[test]
    fn provider_progress_roundtrips_without_unbounded_diagnostics() {
        let progress = ProviderProgress::ScheduledBackoff {
            attempt: 2,
            max_attempts: 3,
            delay_ms: 750,
        };
        let encoded = serde_json::to_string(&progress).expect("serialize progress");
        assert_eq!(
            encoded,
            r#"{"stage":"scheduled_backoff","attempt":2,"max_attempts":3,"delay_ms":750}"#
        );
        let decoded: ProviderProgress =
            serde_json::from_str(&encoded).expect("deserialize progress");
        assert_eq!(decoded, progress);
    }
}
