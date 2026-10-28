//! OpenAI Responses HTTP client boundary.
//!
//! This is intentionally not yet a public `LanguageModel` adapter. It owns request dispatch,
//! same-protocol retry, typed progress, and handoff to the Responses SSE transport while preserving
//! opaque reasoning replay only for the same configured provider/model identity.

use std::time::Duration;

use reqwest::Client;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::Message;
use talos_core::provider::{ProviderError, ProviderProgress, ProviderResult, ToolDefinition};
use tokio::sync::mpsc;

use crate::openai_responses_request::build_request_body;
use crate::openai_responses_sse::ResponsesStreamOutput;
use crate::retry::{RetryDecision, classify_retry_with_backoff};

pub(crate) const OPENAI_RESPONSES_API_URL: &str = "https://api.openai.com/v1";
pub(crate) const RESPONSES_PATH: &str = "/responses";

#[derive(Clone)]
pub(crate) struct OpenAIResponsesClient {
    api_key: String,
    provider_key: String,
    model: String,
    base_url: String,
    client: Client,
    copilot_headers: serde_json::Map<String, Value>,
    reasoning: Option<ReasoningOptions>,
    output_limit: Option<u32>,
    timeout_config: ProviderTimeoutConfig,
}

impl OpenAIResponsesClient {
    pub(crate) fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            provider_key: "openai".into(),
            model: model.into(),
            base_url: OPENAI_RESPONSES_API_URL.into(),
            client: Client::new(),
            copilot_headers: serde_json::Map::new(),
            reasoning: None,
            output_limit: None,
            timeout_config: ProviderTimeoutConfig::default(),
        }
    }

    pub(crate) fn with_copilot_client(
        mut self,
        client: Client,
        headers: serde_json::Map<String, Value>,
    ) -> Self {
        self.client = client;
        self.copilot_headers = headers;
        self
    }

    /// Stamp the config provider identity used by ADR-034 reasoning replay checks.
    #[must_use]
    pub(crate) fn with_provider_key(mut self, provider_key: impl Into<String>) -> Self {
        self.provider_key = provider_key.into();
        self
    }

    #[must_use]
    pub(crate) fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    #[must_use]
    pub(crate) fn with_reasoning(
        mut self,
        reasoning: Option<ReasoningOptions>,
        output_limit: Option<u32>,
    ) -> Self {
        self.reasoning = reasoning;
        self.output_limit = output_limit;
        self
    }

    #[must_use]
    pub(crate) fn with_timeout_config(mut self, config: ProviderTimeoutConfig) -> Self {
        self.timeout_config = config;
        self
    }

    #[must_use]
    pub(crate) fn endpoint_url(&self) -> String {
        let base = self.base_url.trim().trim_end_matches('/');
        if base.to_ascii_lowercase().ends_with(RESPONSES_PATH) {
            base.to_string()
        } else {
            format!("{base}{RESPONSES_PATH}")
        }
    }

    /// Build the same stateless request shape used for dispatch without touching the network.
    ///
    /// Credentials are always redacted and image content follows the existing preview projection so
    /// diagnostics cannot turn a request preview into an implicit file read.
    pub(crate) fn request_preview(&self, messages: &[Message]) -> Value {
        let replay = self
            .reasoning
            .as_ref()
            .is_none_or(|reasoning| reasoning.replay);
        let preview_messages = crate::image_io::preview_messages(messages);
        let messages =
            replay_safe_messages(&self.provider_key, &self.model, replay, &preview_messages);
        let mut body = build_request_body(
            &self.model,
            &messages,
            &[],
            self.reasoning.as_ref(),
            self.output_limit,
        );
        // Replay ciphertext belongs only on the wire, never in a readable diagnostic snapshot.
        if let Some(input) = body.get_mut("input").and_then(Value::as_array_mut) {
            for item in input {
                if item.get("type").and_then(Value::as_str) == Some("reasoning")
                    && let Some(ciphertext) = item.get_mut("encrypted_content")
                {
                    *ciphertext = Value::String("[redacted]".into());
                }
            }
        }
        let mut preview = json!({
            "method": "POST",
            "url": self.endpoint_url(),
            "headers": {
                "Authorization": format!("Bearer {}", redact_secret(&self.api_key)),
                "Content-Type": "application/json",
            },
            "body": body,
        });
        if let Some(headers) = preview["headers"].as_object_mut() {
            if !self.copilot_headers.is_empty() {
                headers.insert("Authorization".into(), json!("Bearer ***"));
            }
            headers.extend(self.copilot_headers.clone());
        }
        preview
    }

    pub(crate) async fn stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<mpsc::Receiver<ResponsesStreamOutput>> {
        self.stream_with_images(messages, tools, progress_tx, Vec::new())
            .await
    }

    pub(crate) async fn stream_with_images(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
        images: Vec<talos_core::provider::EphemeralImage>,
    ) -> ProviderResult<mpsc::Receiver<ResponsesStreamOutput>> {
        let replay = self
            .reasoning
            .as_ref()
            .is_none_or(|reasoning| reasoning.replay);
        let messages = replay_safe_messages(&self.provider_key, &self.model, replay, messages);
        let mut body = build_request_body(
            &self.model,
            &messages,
            tools,
            self.reasoning.as_ref(),
            self.output_limit,
        );
        let deadline = crate::image_io::append_responses_ephemeral_images(&mut body, images)?;
        let response = match deadline {
            Some(_) => {
                self.send_request_with_deadline(&body, progress_tx, deadline)
                    .await?
            }
            None => self.send_request(&body, progress_tx).await?,
        };
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(crate::openai_responses_transport::parse_response_stream(
            response,
            tx,
            Duration::from_secs(self.timeout_config.first_packet_timeout_secs),
            Duration::from_secs(self.timeout_config.stream_idle_timeout_secs),
        ));
        Ok(rx)
    }

    pub(crate) async fn send_request(
        &self,
        body: &Value,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<reqwest::Response> {
        self.send_request_with_deadline(body, progress_tx, None)
            .await
    }

    async fn send_request_with_deadline(
        &self,
        body: &Value,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
        deadline: Option<std::time::Instant>,
    ) -> ProviderResult<reqwest::Response> {
        let request = self.send_request_before_deadline(body, progress_tx, deadline);
        match deadline {
            Some(expires) => tokio::time::timeout_at(expires.into(), request)
                .await
                .map_err(|_| {
                    ProviderError::InvalidResponse("ephemeral image expired during request".into())
                })?,
            None => request.await,
        }
    }

    async fn send_request_before_deadline(
        &self,
        body: &Value,
        progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
        deadline: Option<std::time::Instant>,
    ) -> ProviderResult<reqwest::Response> {
        let max_attempts = if deadline.is_some() {
            0
        } else {
            self.timeout_config.max_attempts
        };
        // ADR-051: an ephemeral image may not be replayed by automatic retries.
        let retry_decision = |error: &ProviderError, attempt| {
            if deadline.is_some() {
                RetryDecision::DoNotRetry
            } else {
                self.retry_decision(error, attempt)
            }
        };
        let dispatch_timeout = Duration::from_secs(self.timeout_config.dispatch_timeout_secs);
        let mut attempt = 0u32;

        loop {
            if deadline.is_some_and(|expires| std::time::Instant::now() >= expires) {
                return Err(ProviderError::InvalidResponse(
                    "ephemeral image expired before dispatch".into(),
                ));
            }
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

            let request = self
                .client
                .post(self.endpoint_url())
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .json(body)
                .send();

            let response = match tokio::time::timeout(dispatch_timeout, request).await {
                Ok(result) => result,
                Err(_) => {
                    let error = ProviderError::NetworkError(format!(
                        "request dispatch timeout: no response headers within {}s",
                        self.timeout_config.dispatch_timeout_secs
                    ));
                    match retry_decision(&error, attempt) {
                        RetryDecision::Retry {
                            attempt: next_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: next_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = next_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
            };

            match response {
                Ok(response) if response.status().is_success() => {
                    emit_progress(
                        progress_tx,
                        ProviderProgress::FirstPacketWait {
                            attempt,
                            max_attempts,
                        },
                    );
                    return Ok(response);
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let body = if !self.copilot_headers.is_empty() { crate::copilot::redact_credential(body, &self.api_key) } else { body };
                    let error = responses_status_to_error(status, body);
                    match retry_decision(&error, attempt) {
                        RetryDecision::Retry {
                            attempt: next_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: next_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = next_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
                Err(error) => {
                    let error = ProviderError::NetworkError(error.to_string());
                    match retry_decision(&error, attempt) {
                        RetryDecision::Retry {
                            attempt: next_attempt,
                            delay_ms,
                        } => {
                            emit_progress(
                                progress_tx,
                                ProviderProgress::ScheduledBackoff {
                                    attempt: next_attempt,
                                    max_attempts,
                                    delay_ms,
                                },
                            );
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = next_attempt;
                            continue;
                        }
                        RetryDecision::DoNotRetry => return Err(error),
                    }
                }
            }
        }
    }

    fn retry_decision(&self, error: &ProviderError, attempt: u32) -> RetryDecision {
        classify_retry_with_backoff(
            error,
            attempt,
            self.timeout_config.max_attempts,
            self.timeout_config.backoff_base_ms,
            self.timeout_config.backoff_max_ms,
        )
    }
}

fn redact_secret(secret: &str) -> String {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return "<empty>".into();
    }

    let prefix: String = trimmed.chars().take(4).collect();
    let suffix: String = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

fn replay_safe_messages(
    provider_key: &str,
    model: &str,
    replay: bool,
    messages: &[Message],
) -> Vec<Message> {
    messages
        .iter()
        .cloned()
        .map(|message| match message {
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                let reasoning = reasoning.filter(|reasoning| {
                    replay && reasoning.provider == provider_key && reasoning.model == model
                });
                Message::Assistant {
                    content,
                    tool_calls,
                    reasoning,
                }
            }
            message => message,
        })
        .collect()
}

fn emit_progress(
    progress_tx: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    progress: ProviderProgress,
) {
    if let Some(progress_tx) = progress_tx {
        let _ = progress_tx.send(progress);
    }
}

fn responses_status_to_error(status: reqwest::StatusCode, body: String) -> ProviderError {
    match status.as_u16() {
        401 | 403 => ProviderError::AuthenticationFailed(body),
        404 | 405 => ProviderError::InvalidResponse(format!(
            "OpenAI Responses endpoint is unavailable ({status}); this gateway may not support \
             /responses. Set protocol = \"openai-chat\" explicitly to use Chat Completions."
        )),
        408 | 409 | 425 | 429 => ProviderError::RateLimited(body),
        code if code >= 500 => ProviderError::ServerError(body),
        _ => ProviderError::InvalidResponse(format!(
            "OpenAI Responses request rejected with status {status}: {body}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{AssistantReasoning, ReasoningBlock};

    #[test]
    fn endpoint_appends_responses_once() {
        let default_client = OpenAIResponsesClient::new("sk-test", "gpt-test");
        assert_eq!(
            default_client.endpoint_url(),
            "https://api.openai.com/v1/responses"
        );

        let explicit = OpenAIResponsesClient::new("sk-test", "gpt-test")
            .with_base_url("https://gateway.example/v1/responses/");
        assert_eq!(
            explicit.endpoint_url(),
            "https://gateway.example/v1/responses"
        );
    }

    #[test]
    fn request_preview_is_stateless_and_redacts_credentials() {
        let client = OpenAIResponsesClient::new("sk-super-secret", "gpt-test")
            .with_reasoning(Some(ReasoningOptions::default()), Some(1234));
        let preview = client.request_preview(&[Message::User {
            content: "hello".into(),
        }]);
        assert_eq!(preview["url"], "https://api.openai.com/v1/responses");
        assert_eq!(preview["body"]["store"], false);
        assert!(preview["body"].get("previous_response_id").is_none());
        assert_eq!(preview["body"]["max_output_tokens"], 1234);
        let authorization = preview["headers"]["Authorization"]
            .as_str()
            .expect("authorization preview");
        assert!(!authorization.contains("sk-super-secret"));
    }

    #[test]
    fn request_preview_redacts_replay_ciphertext_without_disabling_wire_replay() {
        let item = json!({
            "type": "reasoning", "id": "rs_test", "summary": [],
            "encrypted_content": "CIPHERTEXT-DO-NOT-DISPLAY",
        });
        let data = crate::openai_responses_reasoning::encode_completed_reasoning_item(&item)
            .expect("valid reasoning item")
            .expect("opaque replay envelope");
        let messages = [Message::Assistant {
            content: "answer".into(),
            tool_calls: vec![],
            reasoning: Some(AssistantReasoning {
                provider: "openai".into(),
                model: "gpt-test".into(),
                blocks: vec![ReasoningBlock::Redacted { data }],
            }),
        }];
        let client = OpenAIResponsesClient::new("test-key", "gpt-test");
        let preview = client.request_preview(&messages);
        assert_eq!(
            preview["body"]["input"][0]["encrypted_content"],
            "[redacted]"
        );
        assert!(!preview.to_string().contains("CIPHERTEXT-DO-NOT-DISPLAY"));
        let wire = build_request_body("gpt-test", &messages, &[], None, None);
        assert_eq!(
            wire["input"][0]["encrypted_content"],
            "CIPHERTEXT-DO-NOT-DISPLAY"
        );
    }

    #[test]
    fn replay_history_requires_same_provider_and_model() {
        let message = Message::Assistant {
            content: String::new(),
            tool_calls: vec![],
            reasoning: Some(AssistantReasoning {
                provider: "gateway-a".into(),
                model: "same-model".into(),
                blocks: vec![ReasoningBlock::Redacted {
                    data: "opaque".into(),
                }],
            }),
        };

        let same = replay_safe_messages(
            "gateway-a",
            "same-model",
            true,
            std::slice::from_ref(&message),
        );
        assert!(matches!(
            &same[0],
            Message::Assistant {
                reasoning: Some(_),
                ..
            }
        ));
        let other_provider = replay_safe_messages(
            "gateway-b",
            "same-model",
            true,
            std::slice::from_ref(&message),
        );
        assert!(matches!(
            &other_provider[0],
            Message::Assistant {
                reasoning: None,
                ..
            }
        ));
        let other_model = replay_safe_messages(
            "gateway-a",
            "other-model",
            true,
            std::slice::from_ref(&message),
        );
        assert!(matches!(
            &other_model[0],
            Message::Assistant {
                reasoning: None,
                ..
            }
        ));
        let disabled = replay_safe_messages(
            "gateway-a",
            "same-model",
            false,
            std::slice::from_ref(&message),
        );
        assert!(matches!(
            &disabled[0],
            Message::Assistant {
                reasoning: None,
                ..
            }
        ));
    }

    #[test]
    fn unsupported_endpoint_error_is_actionable_and_not_retryable() {
        let error = responses_status_to_error(reqwest::StatusCode::NOT_FOUND, "not found".into());
        assert!(matches!(
            &error,
            ProviderError::InvalidResponse(message)
                if message.contains("protocol = \"openai-chat\"")
                    && message.contains("/responses")
        ));
        let client = OpenAIResponsesClient::new("sk-test", "gpt-test");
        assert!(matches!(
            client.retry_decision(&error, 0),
            RetryDecision::DoNotRetry
        ));
    }

    #[test]
    fn same_protocol_server_errors_remain_eligible_for_bounded_retry() {
        let client = OpenAIResponsesClient::new("sk-test", "gpt-test");
        let error =
            responses_status_to_error(reqwest::StatusCode::SERVICE_UNAVAILABLE, "temporary".into());
        assert!(matches!(
            client.retry_decision(&error, 0),
            RetryDecision::Retry { .. }
        ));
    }
}
