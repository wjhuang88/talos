//! OpenAI provider implementing [`LanguageModel`] via the Chat Completions API.
//!
//! Streams text deltas and tool calls via SSE from the OpenAI Chat Completions API,
//! handles errors gracefully, and supports exponential backoff retry.
//!
//! # Example
//!
//! ```no_run
//! use talos_provider::openai::OpenAIProvider;
//! use talos_core::provider::LanguageModel;
//!
//! let provider = OpenAIProvider::new("sk-...", "gpt-4o");
//! ```

use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use tokio::sync::mpsc;

use crate::openai_request::{build_request_body, redact_secret};
use crate::retry::{RetryDecision, classify_retry_with_backoff};

pub(crate) const OPENAI_API_URL: &str = "https://api.openai.com/v1";
pub(crate) const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";
/// OpenAI Chat Completions provider implementing [`LanguageModel`].
///
/// Streams text deltas and tool calls via SSE from the OpenAI Chat Completions API.
/// Supports custom base URLs for compatible APIs (e.g., Azure OpenAI, local LLMs).
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    pub(crate) base_url: String,
    client: Client,
    reasoning: Option<ReasoningOptions>,
    output_limit: Option<u32>,
    timeout_config: ProviderTimeoutConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` — OpenAI API key (must not be empty).
    /// * `model` — Model identifier (e.g., `"gpt-4o"`, `"gpt-4-turbo"`).
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: OPENAI_API_URL.into(),
            client: Client::new(),
            reasoning: None,
            output_limit: None,
            timeout_config: ProviderTimeoutConfig::default(),
        }
    }

    /// Set a custom base URL (the OpenAI-compatible gateway root).
    ///
    /// `base_url` is the gateway root (e.g. `https://gateway.example.com/v1` or
    /// `https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1`).
    /// The provider automatically appends `/chat/completions` to the URL when
    /// making the request, matching the OpenAI SDK convention.
    ///
    /// # Arguments
    ///
    /// * `base_url` — Gateway root, no trailing `/chat/completions`.
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

    /// Compose the chat completions endpoint URL by appending the standard
    /// path to the configured base.
    pub(crate) fn endpoint_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}{CHAT_COMPLETIONS_PATH}")
    }

    async fn make_request(&self, messages: &[Message]) -> ProviderResult<reqwest::Response> {
        let body = build_request_body(
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
        let body = build_request_body(
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
                .post(self.endpoint_url())
                .header("Authorization", format!("Bearer {}", &self.api_key))
                .header("Content-Type", "application/json")
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
                                "retrying openai dispatch timeout"
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
                                "retrying openai provider request"
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
                                "retrying openai network error"
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
impl LanguageModel for OpenAIProvider {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let response = self.make_request(messages).await?;
        let (tx, rx) = mpsc::channel(32);
        let timeout_config = self.timeout_config.clone();
        tokio::spawn(crate::openai_sse::parse_sse_stream(
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
        tokio::spawn(crate::openai_sse::parse_sse_stream(
            response,
            tx,
            Duration::from_secs(timeout_config.first_packet_timeout_secs),
            Duration::from_secs(timeout_config.stream_idle_timeout_secs),
        ));
        Ok(rx)
    }

    fn request_preview(&self, messages: &[Message]) -> Option<Value> {
        let body = build_request_body(
            &self.model,
            messages,
            &[],
            self.reasoning.as_ref(),
            self.output_limit,
        );
        Some(json!({
            "method": "POST",
            "url": self.endpoint_url(),
            "headers": {
                "Authorization": format!("Bearer {}", redact_secret(&self.api_key)),
                "Content-Type": "application/json",
            },
            "body": body,
        }))
    }
}

/// Build a redacted OpenAI-compatible chat completions request snapshot for mock diagnostics.
pub fn openai_request_debug_snapshot(
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    messages: &[Message],
) -> Value {
    let endpoint_url = match base_url {
        Some(url) => format!("{}{}", url.trim_end_matches('/'), CHAT_COMPLETIONS_PATH),
        None => format!("{OPENAI_API_URL}{CHAT_COMPLETIONS_PATH}"),
    };

    json!({
        "method": "POST",
        "url": endpoint_url,
        "headers": {
            "Authorization": format!("Bearer {}", redact_secret(api_key)),
            "Content-Type": "application/json",
        },
        "body": build_request_body(model, messages, &[], None, None),
    })
}
