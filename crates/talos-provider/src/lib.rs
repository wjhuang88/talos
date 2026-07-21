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

mod anthropic_request;
mod anthropic_stream;

use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use tokio::sync::mpsc;

use crate::retry::{RetryDecision, classify_retry_with_backoff};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider implementing [`LanguageModel`].
///
/// Streams text deltas via SSE from the Anthropic Messages API,
/// handles errors gracefully, and supports exponential backoff retry.
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
        self.send_request(&body).await
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
        self.send_request(&body).await
    }

    async fn send_request(&self, body: &Value) -> ProviderResult<reqwest::Response> {
        let max_attempts = self.timeout_config.max_attempts;
        let dispatch_timeout = Duration::from_secs(self.timeout_config.dispatch_timeout_secs);
        let mut attempt = 0u32;
        loop {
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
                Ok(resp) if resp.status().is_success() => return Ok(resp),
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

pub(crate) use anthropic_stream::parse_text_tool_calls;
