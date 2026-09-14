//! Talos provider — LLM client abstractions and provider-specific implementations.
//!
//! This crate contains provider adapters and request-shaping helpers used by Talos.
//! The public boundary is intentionally narrow in the pre-1.0 line:
//!
//! - provider types implement the [`talos_core::provider::LanguageModel`] trait;
//! - request previews are diagnostic snapshots and must redact credentials;
//! - network calls return typed provider errors instead of panicking;
//! - retry behavior is bounded and provider-specific;
//! - model catalogs, credential storage, and runtime selection live outside this crate.
//!
//! Publishing this crate does not make Talos provider configuration stable. Consumers should treat
//! concrete provider structs as pre-1.0 adapters and prefer the `talos-core` provider traits for
//! long-lived integration code.

mod image_io;
pub mod mock;
pub mod openai;
mod openai_request;
mod openai_sse;
pub mod retry;
mod stream_utf8;

mod anthropic_request;
mod anthropic_stream;

use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{
    LanguageModel, ProviderError, ProviderProgress, ProviderResult, ToolDefinition,
};

fn validate_decision_messages(
    messages: &[Message],
    limits: talos_core::provider::DecisionRequestLimits,
) -> ProviderResult<()> {
    if limits.max_output_tokens == 0
        || messages
            .iter()
            .any(|message| !matches!(message, Message::System { .. } | Message::User { .. }))
    {
        return Err(ProviderError::InvalidResponse(
            "invalid isolated decision request".into(),
        ));
    }
    Ok(())
}

/// Opaque cache identity; URLs carrying credentials or query data disable caching.
fn protocol_scope(
    adapter: &str,
    endpoint: &str,
    model: &str,
    reasoning: Option<&ReasoningOptions>,
    output_limit: Option<u32>,
) -> Option<String> {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let url = reqwest::Url::parse(endpoint).ok()?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    // JSON tuple encoding preserves field boundaries, including embedded delimiters.
    let encoded = serde_json::to_vec(&(adapter, endpoint, model, reasoning, output_limit)).ok()?;
    let digest = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(encoded));
    Some(format!("protocol-v1:{digest}"))
}

/// Projects durable native tool blocks into bounded compatibility text while preserving IDs.
/// This is used only for non-native protocol requests; native requests retain structured blocks.
pub(crate) fn compatibility_messages(messages: &[Message]) -> Vec<Message> {
    messages
        .iter()
        .map(|message| match message {
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } if !tool_calls.is_empty() => {
                let calls = tool_calls
                    .iter()
                    .map(|call| format!("{} {}", call.name, call.input))
                    .collect::<Vec<_>>()
                    .join("\n");
                Message::Assistant {
                    content: format!("{content}\n<tool_calls>\n{calls}\n</tool_calls>"),
                    tool_calls: Vec::new(),
                    reasoning: reasoning.clone(),
                }
            }
            Message::Tool { result } => Message::User {
                content: format!(
                    "<tool_result id={} error={}>\n{}\n</tool_result>",
                    result.tool_use_id, result.is_error, result.content
                ),
            },
            other => other.clone(),
        })
        .collect()
}
use tokio::sync::mpsc;

use crate::retry::{RetryDecision, classify_retry_with_backoff};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider implementing [`LanguageModel`].
///
/// Streams text deltas via SSE from the Anthropic Messages API,
/// handles errors gracefully, and supports exponential backoff retry.
#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
    reasoning: Option<ReasoningOptions>,
    output_limit: Option<u32>,
    timeout_config: ProviderTimeoutConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` — Anthropic API key (must not be empty).
    /// * `model` — Model identifier (e.g., `"claude-sonnet-4-20250514"`).
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: ANTHROPIC_API_URL.into(),
            client: Client::new(),
            reasoning: None,
            output_limit: None,
            timeout_config: ProviderTimeoutConfig::default(),
        }
    }

    /// Set a custom base URL (useful for testing or enterprise proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set per-model reasoning and output token configuration.
    pub fn with_reasoning(
        mut self,
        reasoning: Option<ReasoningOptions>,
        output_limit: Option<u32>,
    ) -> Self {
        self.reasoning = reasoning;
        self.output_limit = output_limit;
        self
    }

    /// Set provider stream timeout configuration.
    pub fn with_timeout_config(mut self, config: ProviderTimeoutConfig) -> Self {
        self.timeout_config = config;
        self
    }

    async fn make_request(&self, messages: &[Message]) -> ProviderResult<reqwest::Response> {
        let body = anthropic_request::build_request_body(
            &self.model,
            messages,
            &[],
            self.reasoning.as_ref(),
            self.output_limit,
        );
        self.send_request(&body, None).await
    }

    async fn make_request_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<reqwest::Response> {
        let body = anthropic_request::build_request_body(
            &self.model,
            messages,
            tools,
            self.reasoning.as_ref(),
            self.output_limit,
        );
        self.send_request(&body, None).await
    }

    async fn send_request(
        &self,
        body: &Value,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<reqwest::Response> {
        let max_attempts = self.timeout_config.max_attempts;
        let dispatch_timeout = Duration::from_secs(self.timeout_config.dispatch_timeout_secs);
        let mut attempt = 0u32;
        loop {
            emit_progress(
                progress_tx,
                if attempt == 0 {
                    ProviderProgress::InitialDispatch {
                        attempt,
                        max_attempts,
                    }
                } else {
                    ProviderProgress::RetryDispatch {
                        attempt,
                        max_attempts,
                    }
                },
            );
            let request_fut = self
                .client
                .post(&self.base_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send();

            let response = match tokio::time::timeout(dispatch_timeout, request_fut).await {
                Ok(result) => result,
                Err(_) => {
                    let error = ProviderError::NetworkError(format!(
                        "request dispatch timeout: no response headers within {}s",
                        self.timeout_config.dispatch_timeout_secs
                    ));
                    match classify_retry_with_backoff(
                        &error,
                        attempt,
                        max_attempts,
                        self.timeout_config.backoff_base_ms,
                        self.timeout_config.backoff_max_ms,
                    ) {
                        RetryDecision::Retry {
                            attempt: new_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: new_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tracing::warn!(
                                attempt = new_attempt,
                                delay_ms,
                                "retrying anthropic dispatch timeout"
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = new_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
            };

            match response {
                Ok(resp) if resp.status().is_success() => {
                    emit_progress(
                        progress_tx,
                        ProviderProgress::FirstPacketWait {
                            attempt,
                            max_attempts,
                        },
                    );
                    return Ok(resp);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body_text = resp.text().await.unwrap_or_default();
                    let error = status_to_error(status, body_text);
                    match classify_retry_with_backoff(
                        &error,
                        attempt,
                        max_attempts,
                        self.timeout_config.backoff_base_ms,
                        self.timeout_config.backoff_max_ms,
                    ) {
                        RetryDecision::Retry {
                            attempt: new_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: new_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tracing::warn!(
                                attempt = new_attempt,
                                delay_ms,
                                "retrying anthropic provider request"
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = new_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
                Err(e) => {
                    let error = ProviderError::NetworkError(e.to_string());
                    match classify_retry_with_backoff(
                        &error,
                        attempt,
                        max_attempts,
                        self.timeout_config.backoff_base_ms,
                        self.timeout_config.backoff_max_ms,
                    ) {
                        RetryDecision::Retry {
                            attempt: new_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: new_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tracing::warn!(
                                attempt = new_attempt,
                                delay_ms,
                                "retrying anthropic network error"
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = new_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
            }
        }
    }
}

fn emit_progress(
    progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    progress: ProviderProgress,
) {
    if let Some(progress_tx) = progress_tx {
        let _ = progress_tx.send(progress);
    }
}

fn status_to_error(status: reqwest::StatusCode, body: String) -> ProviderError {
    match status.as_u16() {
        401 | 403 => ProviderError::AuthenticationFailed(body),
        408 | 409 | 425 | 429 => ProviderError::RateLimited(body),
        s if s >= 500 => ProviderError::ServerError(body),
        _ => ProviderError::InvalidResponse(format!("unexpected status {status}: {body}")),
    }
}

#[async_trait::async_trait]
impl LanguageModel for AnthropicProvider {
    async fn stream_decision(
        &self,
        messages: &[Message],
        limits: talos_core::provider::DecisionRequestLimits,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        validate_decision_messages(messages, limits)?;
        let mut isolated = self.clone();
        isolated.reasoning = None;
        isolated.output_limit = Some(limits.max_output_tokens);
        isolated.timeout_config.max_attempts =
            limits.max_retries.min(self.timeout_config.max_attempts);
        let (tx, _) = mpsc::unbounded_channel();
        isolated
            .stream_with_protocol(messages, &[], talos_core::tool::ToolProtocol::Native, tx)
            .await
    }

    fn protocol_capability_scope(&self) -> Option<String> {
        protocol_scope(
            "anthropic",
            &self.base_url,
            &self.model,
            self.reasoning.as_ref(),
            self.output_limit,
        )
    }

    fn protocol_capabilities(&self) -> talos_core::tool::CapabilityProbe {
        // The endpoint alone is not model-specific capability evidence.
        talos_core::tool::CapabilityProbe::Unknown
    }

    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: talos_core::tool::ToolProtocol,
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        if protocol == talos_core::tool::ToolProtocol::Auto {
            return Err(ProviderError::InvalidResponse(
                "automatic protocol must be resolved before dispatch".into(),
            ));
        }
        if matches!(protocol, talos_core::tool::ToolProtocol::Native) {
            let body = anthropic_request::build_request_body(
                &self.model,
                messages,
                tools,
                self.reasoning.as_ref(),
                self.output_limit,
            );
            let response = self.send_request(&body, Some(&progress_tx)).await?;
            let (tx, rx) = mpsc::channel(32);
            let timeout_config = self.timeout_config.clone();
            tokio::spawn(anthropic_stream::parse_sse_stream_with_mode(
                response,
                tx,
                Duration::from_secs(timeout_config.first_packet_timeout_secs),
                Duration::from_secs(timeout_config.stream_idle_timeout_secs),
                protocol,
            ));
            Ok(rx)
        } else {
            let projected = compatibility_messages(messages);
            let body = anthropic_request::build_request_body(
                &self.model,
                &projected,
                &[],
                self.reasoning.as_ref(),
                self.output_limit,
            );
            let response = self.send_request(&body, Some(&progress_tx)).await?;
            let (tx, rx) = mpsc::channel(32);
            let timeout_config = self.timeout_config.clone();
            tokio::spawn(anthropic_stream::parse_sse_stream_with_mode(
                response,
                tx,
                Duration::from_secs(timeout_config.first_packet_timeout_secs),
                Duration::from_secs(timeout_config.stream_idle_timeout_secs),
                protocol,
            ));
            Ok(rx)
        }
    }

    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let response = self.make_request(messages).await?;
        let (tx, rx) = mpsc::channel(32);
        let timeout_config = self.timeout_config.clone();
        tokio::spawn(anthropic_stream::parse_sse_stream(
            response,
            tx,
            Duration::from_secs(timeout_config.first_packet_timeout_secs),
            Duration::from_secs(timeout_config.stream_idle_timeout_secs),
        ));
        Ok(rx)
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let response = self.make_request_with_tools(messages, tools).await?;
        let (tx, rx) = mpsc::channel(32);
        let timeout_config = self.timeout_config.clone();
        tokio::spawn(anthropic_stream::parse_sse_stream(
            response,
            tx,
            Duration::from_secs(timeout_config.first_packet_timeout_secs),
            Duration::from_secs(timeout_config.stream_idle_timeout_secs),
        ));
        Ok(rx)
    }

    async fn stream_with_tools_and_progress(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let body = anthropic_request::build_request_body(
            &self.model,
            messages,
            tools,
            self.reasoning.as_ref(),
            self.output_limit,
        );
        let response = self.send_request(&body, Some(&progress_tx)).await?;
        let (tx, rx) = mpsc::channel(32);
        let timeout_config = self.timeout_config.clone();
        tokio::spawn(anthropic_stream::parse_sse_stream(
            response,
            tx,
            Duration::from_secs(timeout_config.first_packet_timeout_secs),
            Duration::from_secs(timeout_config.stream_idle_timeout_secs),
        ));
        Ok(rx)
    }

    fn request_preview(&self, messages: &[Message]) -> Option<Value> {
        let body = anthropic_request::build_request_body(
            &self.model,
            messages,
            &[],
            self.reasoning.as_ref(),
            self.output_limit,
        );
        Some(json!({
            "method": "POST",
            "url": &self.base_url,
            "headers": {
                "x-api-key": redact_secret(&self.api_key),
                "anthropic-version": ANTHROPIC_VERSION,
                "content-type": "application/json",
            },
            "body": body,
        }))
    }
}

pub use anthropic_request::anthropic_request_debug_snapshot;
use anthropic_request::redact_secret;
