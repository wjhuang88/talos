//! Public OpenAI Responses `LanguageModel` adapter.
//!
//! This wrapper composes the dedicated Responses request/client/transport/state-machine layers with
//! Talos' shared invocation-integrity ingress. It does not reuse Chat Completions request or SSE
//! parsing, and it never performs a cross-protocol fallback.

use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{
    LanguageModel, ProviderError, ProviderInvocationEvent, ProviderInvocationStream,
    ProviderProgress, ProviderResult, ToolDefinition,
};
use talos_core::tool::{CapabilityProbe, ProtocolCapabilities, ToolProtocol};
use tokio::sync::mpsc;

use crate::openai_responses_bridge::forward_response_stream;
use crate::openai_responses_client::OpenAIResponsesClient;
use crate::openai_sse::InvocationSender;

/// OpenAI Responses provider implementing Talos' provider boundary.
#[derive(Clone)]
pub struct OpenAIResponsesProvider {
    client: OpenAIResponsesClient,
}

impl OpenAIResponsesProvider {
    /// Create a Responses provider for one model.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: OpenAIResponsesClient::new(api_key, model),
        }
    }

    /// Bind the Talos provider identity used for durable reasoning replay isolation.
    #[must_use]
    pub fn with_provider_key(mut self, provider_key: impl Into<String>) -> Self {
        self.client = self.client.with_provider_key(provider_key);
        self
    }

    /// Override the OpenAI-compatible API root. The client appends `/responses` exactly once.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.client = self.client.with_base_url(base_url);
        self
    }

    /// Configure reasoning and the Responses `max_output_tokens` limit.
    #[must_use]
    pub fn with_reasoning(
        mut self,
        reasoning: Option<ReasoningOptions>,
        output_limit: Option<u32>,
    ) -> Self {
        self.client = self.client.with_reasoning(reasoning, output_limit);
        self
    }

    /// Configure dispatch/retry/stream timeouts.
    #[must_use]
    pub fn with_timeout_config(mut self, config: ProviderTimeoutConfig) -> Self {
        self.client = self.client.with_timeout_config(config);
        self
    }

    async fn semantic_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<mpsc::Receiver<crate::openai_responses_sse::ResponsesStreamOutput>> {
        match protocol {
            ToolProtocol::Native => self.client.stream(messages, tools, progress_tx).await,
            ToolProtocol::Auto => Err(ProviderError::InvalidResponse(
                "automatic tool protocol must be resolved before OpenAI Responses dispatch".into(),
            )),
            ToolProtocol::Compat | ToolProtocol::TalosStrict => Err(ProviderError::InvalidResponse(
                "OpenAI Responses adapter requires native function tools; textual compatibility tool protocols are not enabled for this wire family"
                    .into(),
            )),
        }
    }

    async fn legacy_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let source = self
            .semantic_stream(messages, tools, protocol, progress_tx)
            .await?;
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(forward_response_stream(source, InvocationSender::Legacy(tx)));
        Ok(rx)
    }
}

#[async_trait::async_trait]
impl LanguageModel for OpenAIResponsesProvider {
    fn protocol_capabilities(&self) -> CapabilityProbe {
        // Selecting the Responses wire family is itself explicit/confirmed evidence that native
        // function items are available. Textual Talos compatibility framing is intentionally not
        // mixed into this adapter.
        CapabilityProbe::Known(ProtocolCapabilities {
            native_tools: true,
            compatibility: false,
        })
    }

    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.legacy_stream(messages, &[], ToolProtocol::Native, None)
            .await
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.legacy_stream(messages, tools, ToolProtocol::Native, None)
            .await
    }

    async fn stream_with_tools_and_progress(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.legacy_stream(messages, tools, ToolProtocol::Native, Some(&progress_tx))
            .await
    }

    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.legacy_stream(messages, tools, protocol, Some(&progress_tx))
            .await
    }

    async fn stream_with_invocation_integrity(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
        browser_tools: &[String],
    ) -> ProviderResult<ProviderInvocationStream> {
        let source = self
            .semantic_stream(messages, tools, protocol, Some(&progress_tx))
            .await?;
        let (tx, rx) = mpsc::channel::<ProviderInvocationEvent>(32);
        tokio::spawn(forward_response_stream(
            source,
            InvocationSender::Original {
                tx,
                browser_tools: browser_tools.to_vec(),
            },
        ));
        Ok(ProviderInvocationStream::original(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses_wire_family_declares_native_tool_evidence() {
        let provider = OpenAIResponsesProvider::new("sk-test", "gpt-test");
        assert_eq!(
            provider.protocol_capabilities(),
            CapabilityProbe::Known(ProtocolCapabilities {
                native_tools: true,
                compatibility: false,
            })
        );
        assert_eq!(provider.protocol_capabilities().select(), ToolProtocol::Native);
    }

    #[tokio::test]
    async fn auto_and_textual_tool_protocols_fail_before_network_dispatch() {
        let provider = OpenAIResponsesProvider::new("sk-test", "gpt-test");
        for protocol in [
            ToolProtocol::Auto,
            ToolProtocol::Compat,
            ToolProtocol::TalosStrict,
        ] {
            let (progress, _rx) = mpsc::unbounded_channel();
            let error = provider
                .stream_with_protocol(&[], &[], protocol, progress)
                .await
                .expect_err("non-native protocol must fail before dispatch");
            assert!(matches!(error, ProviderError::InvalidResponse(_)));
        }
    }
}
