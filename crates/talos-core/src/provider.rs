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

/// Provider-enforced limits for an isolated, tool-free decision request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DecisionRequestLimits {
    /// Positive maximum generated tokens, including any provider reasoning.
    pub max_output_tokens: u32,
    /// Maximum additional transport dispatches; zero disables retries.
    pub max_retries: u32,
}

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
    /// Dispatches an isolated text decision with no tools or inherited reasoning settings.
    ///
    /// Implementations must enforce the supplied token and retry limits. Unsupported
    /// providers fail before dispatch; falling back to unrestricted `stream` is unsafe.
    async fn stream_decision(
        &self,
        _messages: &[Message],
        _limits: DecisionRequestLimits,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        Err(ProviderError::InvalidResponse(
            "provider does not support bounded decisions".into(),
        ))
    }
    /// Returns a stable, non-secret scope for capability evidence caching.
    /// `None` disables caching when the provider cannot describe its endpoint/model safely.
    fn protocol_capability_scope(&self) -> Option<String> {
        None
    }

    /// Reports non-secret evidence for automatic tool-protocol selection.
    ///
    /// Implementations must return [`crate::tool::CapabilityProbe::Unknown`] unless
    /// the evidence is tied to the configured endpoint and model. Unknown evidence
    /// is handled conservatively by selecting the validated compatibility path.
    fn protocol_capabilities(&self) -> crate::tool::CapabilityProbe {
        crate::tool::CapabilityProbe::Unknown
    }

    /// Performs a request-scoped native capability probe.
    ///
    /// Providers must return `Unknown` unless the response was validated as native
    /// wire evidence for the configured endpoint and model. The conservative default
    /// preserves compatibility for existing third-party implementations.
    async fn probe_protocol_capabilities(
        &self,
        _tools: &[ToolDefinition],
    ) -> crate::tool::CapabilityProbe {
        crate::tool::CapabilityProbe::Unknown
    }

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

    /// Streams a response with an explicit tool protocol selected by the caller.
    ///
    /// Legacy providers retain their Native behavior. Compatibility modes require an
    /// override implementing both request projection and validated response parsing;
    /// the default rejects them before dispatch rather than silently sending native tools.
    /// Auto must be resolved by the caller before dispatch.
    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: crate::tool::ToolProtocol,
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        if protocol != crate::tool::ToolProtocol::Native {
            return Err(ProviderError::InvalidResponse(
                "provider has no adapter for the selected tool protocol".into(),
            ));
        }
        self.stream_with_tools_and_progress(messages, tools, progress_tx)
            .await
    }

    fn request_preview(&self, _messages: &[Message]) -> Option<Value> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LegacyModel;

    struct CountingLegacyModel(std::sync::atomic::AtomicUsize);

    #[async_trait::async_trait]
    impl LanguageModel for CountingLegacyModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let (_, rx) = mpsc::channel(1);
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn legacy_protocol_adapter_rejects_unsupported_modes_before_dispatch() {
        use crate::tool::ToolProtocol;
        let model = CountingLegacyModel(std::sync::atomic::AtomicUsize::new(0));
        for mode in [
            ToolProtocol::Compat,
            ToolProtocol::TalosStrict,
            ToolProtocol::Auto,
        ] {
            let (tx, mut rx) = mpsc::unbounded_channel();
            assert!(
                model
                    .stream_with_protocol(&[], &[], mode, tx)
                    .await
                    .is_err()
            );
            assert_eq!(rx.recv().await, None);
            assert_eq!(model.0.load(std::sync::atomic::Ordering::SeqCst), 0);
        }
        let (tx, _) = mpsc::unbounded_channel();
        assert!(
            model
                .stream_with_protocol(&[], &[], ToolProtocol::Native, tx)
                .await
                .is_ok()
        );
        assert_eq!(model.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

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
