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

use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::{AgentEvent, Message, ReasoningBlock, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;

use crate::openai_request::{build_request_body, redact_secret};
use crate::parse_text_tool_calls;
use crate::retry::{RetryDecision, classify_retry_with_backoff};

const OPENAI_API_URL: &str = "https://api.openai.com/v1";
const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";
/// OpenAI Chat Completions provider implementing [`LanguageModel`].
///
/// Streams text deltas and tool calls via SSE from the OpenAI Chat Completions API.
/// Supports custom base URLs for compatible APIs (e.g., Azure OpenAI, local LLMs).
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
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
    fn endpoint_url(&self) -> String {
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
        tokio::spawn(parse_sse_stream(
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
        tokio::spawn(parse_sse_stream(
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

/// OpenAI stream chunk for SSE response parsing.
#[derive(Debug, Clone, Deserialize)]
struct OpenAIStreamChunk {
    #[serde(default)]
    choices: Vec<OpenAIChoice>,
}

/// Choice within an OpenAI stream chunk.
#[derive(Debug, Clone, Deserialize)]
struct OpenAIChoice {
    #[serde(default)]
    delta: OpenAIDelta,
    #[serde(default, rename = "finish_reason")]
    finish_reason: Option<String>,
}

/// Delta content within an OpenAI stream choice.
#[derive(Debug, Clone, Default, Deserialize)]
struct OpenAIDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAIDeltaToolCall>>,
}

/// Tool call within an OpenAI stream delta.
#[derive(Debug, Clone, Deserialize)]
struct OpenAIDeltaToolCall {
    #[serde(default)]
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default, rename = "type")]
    #[allow(dead_code)]
    call_type: Option<String>,
    #[serde(default)]
    function: Option<OpenAIDeltaFunction>,
}

/// Function within an OpenAI stream delta tool call.
#[derive(Debug, Clone, Deserialize)]
struct OpenAIDeltaFunction {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

async fn parse_sse_stream(
    response: reqwest::Response,
    tx: mpsc::Sender<AgentEvent>,
    first_packet_timeout: Duration,
    idle_timeout: Duration,
) {
    let _ = tx.send(AgentEvent::TurnStart).await;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut input_tokens: u32 = 0;
    let mut output_tokens: u32 = 0;
    let mut reasoning_tokens: u32 = 0;

    let mut tool_call_ids: Vec<String> = Vec::new();
    let mut tool_call_names: Vec<String> = Vec::new();
    let mut tool_call_args: Vec<String> = Vec::new();
    let mut text_accumulator = String::new();
    let mut reasoning_text = String::new();
    let mut saw_first_packet = false;
    while let Some(chunk_result) = {
        let next_chunk = stream.next();
        let wait_result = if saw_first_packet {
            tokio::time::timeout(idle_timeout, next_chunk).await
        } else {
            tokio::time::timeout(first_packet_timeout, next_chunk).await
        };

        match wait_result {
            Ok(next) => next,
            Err(_) => {
                let message = if saw_first_packet {
                    format!(
                        "stream-idle timeout: provider stopped sending data for {}s",
                        idle_timeout.as_secs()
                    )
                } else {
                    format!(
                        "first-packet timeout: no response from provider within {}s",
                        first_packet_timeout.as_secs()
                    )
                };
                let _ = tx.send(AgentEvent::Error { message }).await;
                return;
            }
        }
    } {
        saw_first_packet = true;
        let chunk = match chunk_result {
            Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
                Ok(s) => s,
                Err(_) => continue,
            },
            Err(_) => break,
        };

        buffer.push_str(&chunk);

        while let Some(pos) = buffer.find("\n\n") {
            let event_text = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            if event_text.trim().is_empty() {
                continue;
            }

            let data = extract_event_data(&event_text);

            // OpenAI sends `data: [DONE]` at the end
            if data.as_str().map(|s| s.trim()) == Some("[DONE]") {
                let text_calls = parse_text_tool_calls(&text_accumulator);
                for call in text_calls {
                    let _ = tx
                        .send(AgentEvent::ToolCall {
                            call,
                            provenance: ToolProvenance::Native,
                            summary_fields: vec![],
                        })
                        .await;
                }
                // Emit accumulated native tool calls. Some OpenAI-compatible
                // providers stream function name/arguments but close the stream
                // with `[DONE]` and no `finish_reason` chunk; without this
                // fallback the accumulated tool calls would be silently dropped
                // after `ToolCallStarted`, leaving the UI in a stuck
                // `ToolCallStarted -> TurnEnd` state with no `ToolCall`.
                let has_native_tool_calls = tool_call_names.iter().any(|name| !name.is_empty());
                for i in 0..tool_call_ids.len() {
                    if !tool_call_names[i].is_empty() {
                        let tool_call_id = finalized_tool_call_id(&tool_call_ids[i], i);
                        let args: Value =
                            serde_json::from_str(&tool_call_args[i]).unwrap_or_else(|_| json!({}));
                        let _ = tx
                            .send(AgentEvent::ToolCall {
                                call: ToolCall {
                                    id: tool_call_id,
                                    name: tool_call_names[i].clone(),
                                    input: args,
                                },
                                provenance: Default::default(),
                                summary_fields: vec![],
                            })
                            .await;
                    }
                }
                if !reasoning_text.is_empty() {
                    let _ = tx
                        .send(AgentEvent::ReasoningComplete {
                            blocks: vec![ReasoningBlock::Plain {
                                text: std::mem::take(&mut reasoning_text),
                            }],
                        })
                        .await;
                }
                let _ = tx
                    .send(AgentEvent::TurnEnd {
                        stop_reason: if has_native_tool_calls {
                            StopReason::ToolUse
                        } else {
                            StopReason::EndTurn
                        },
                        usage: Usage {
                            input_tokens,
                            output_tokens,
                            cache_read_tokens: 0,
                            cache_write_tokens: 0,
                            reasoning_tokens,
                        },
                    })
                    .await;
                return;
            }

            if let Some(message) = extract_openai_stream_error(&data) {
                let _ = tx.send(AgentEvent::Error { message }).await;
                return;
            }

            let chunk: OpenAIStreamChunk = match serde_json::from_value(data.clone()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Some((input, output, reasoning)) = extract_openai_usage(&data) {
                input_tokens = input;
                output_tokens = output;
                reasoning_tokens = reasoning;
            }

            if chunk.choices.is_empty() {
                continue;
            }

            let choice = &chunk.choices[0];

            if let Some(ref text) = choice.delta.content
                && !text.is_empty()
            {
                text_accumulator.push_str(text);
                let _ = tx
                    .send(AgentEvent::TextDelta {
                        delta: text.clone(),
                    })
                    .await;
            }

            if let Some(ref reasoning_content) = choice.delta.reasoning_content
                && !reasoning_content.is_empty()
            {
                reasoning_text.push_str(reasoning_content);
                let _ = tx
                    .send(AgentEvent::ThinkingDelta {
                        delta: reasoning_content.clone(),
                    })
                    .await;
            }

            // Extract tool calls
            if let Some(ref tool_calls) = choice.delta.tool_calls {
                for tc in tool_calls {
                    let idx = tc.index;

                    // Ensure vectors are large enough
                    while tool_call_ids.len() <= idx {
                        tool_call_ids.push(String::new());
                        tool_call_names.push(String::new());
                        tool_call_args.push(String::new());
                    }

                    if let Some(ref id) = tc.id {
                        tool_call_ids[idx] = id.clone();
                    }
                    if let Some(ref func) = tc.function {
                        if let Some(ref name) = func.name {
                            if tool_call_names[idx].is_empty() {
                                let _ = tx
                                    .send(AgentEvent::ToolCallStarted { name: name.clone() })
                                    .await;
                            }
                            tool_call_names[idx] = name.clone();
                        }
                        if let Some(ref args) = func.arguments {
                            tool_call_args[idx].push_str(args);
                        }
                    }
                }
            }

            if let Some(ref finish_reason) = choice.finish_reason {
                let stop_reason = match finish_reason.as_str() {
                    "stop" => StopReason::EndTurn,
                    "tool_calls" => StopReason::ToolUse,
                    "length" => StopReason::MaxTokens,
                    _ => StopReason::EndTurn,
                };

                // Emit accumulated tool calls. Some OpenAI-compatible providers stream
                // function name/arguments but omit the tool call id; synthesize a stable
                // per-response id so the following tool result can still be paired.
                for i in 0..tool_call_ids.len() {
                    if !tool_call_names[i].is_empty() {
                        let tool_call_id = finalized_tool_call_id(&tool_call_ids[i], i);
                        let args: Value =
                            serde_json::from_str(&tool_call_args[i]).unwrap_or_else(|_| json!({}));
                        let _ = tx
                            .send(AgentEvent::ToolCall {
                                call: ToolCall {
                                    id: tool_call_id,
                                    name: tool_call_names[i].clone(),
                                    input: args,
                                },
                                provenance: Default::default(),
                                summary_fields: vec![],
                            })
                            .await;
                    }
                }

                let text_calls = parse_text_tool_calls(&text_accumulator);
                for call in text_calls {
                    let _ = tx
                        .send(AgentEvent::ToolCall {
                            call,
                            provenance: ToolProvenance::Native,
                            summary_fields: vec![],
                        })
                        .await;
                }

                if !reasoning_text.is_empty() {
                    let _ = tx
                        .send(AgentEvent::ReasoningComplete {
                            blocks: vec![ReasoningBlock::Plain {
                                text: std::mem::take(&mut reasoning_text),
                            }],
                        })
                        .await;
                }

                let _ = tx
                    .send(AgentEvent::TurnEnd {
                        stop_reason,
                        usage: Usage {
                            input_tokens,
                            output_tokens,
                            cache_read_tokens: 0,
                            cache_write_tokens: 0,
                            reasoning_tokens,
                        },
                    })
                    .await;
                return;
            }
        }
    }

    // Stream ended without explicit [DONE] or finish_reason
    let text_calls = parse_text_tool_calls(&text_accumulator);
    for call in text_calls {
        let _ = tx
            .send(AgentEvent::ToolCall {
                call,
                provenance: ToolProvenance::Native,
                summary_fields: vec![],
            })
            .await;
    }

    if !reasoning_text.is_empty() {
        let _ = tx
            .send(AgentEvent::ReasoningComplete {
                blocks: vec![ReasoningBlock::Plain {
                    text: std::mem::take(&mut reasoning_text),
                }],
            })
            .await;
    }

    // Emit any accumulated native tool calls.
    for i in 0..tool_call_ids.len() {
        if !tool_call_names[i].is_empty() {
            let tool_call_id = finalized_tool_call_id(&tool_call_ids[i], i);
            let args: Value =
                serde_json::from_str(&tool_call_args[i]).unwrap_or_else(|_| json!({}));
            let _ = tx
                .send(AgentEvent::ToolCall {
                    call: ToolCall {
                        id: tool_call_id,
                        name: tool_call_names[i].clone(),
                        input: args,
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                })
                .await;
        }
    }

    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens,
                output_tokens,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
                reasoning_tokens,
            },
        })
        .await;
}

fn finalized_tool_call_id(raw_id: &str, index: usize) -> String {
    if raw_id.is_empty() {
        format!("call_{index}")
    } else {
        raw_id.to_string()
    }
}

fn extract_event_data(event_text: &str) -> Value {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("data:") {
            let trimmed = rest.trim();
            // Return as string for [DONE] check
            if trimmed == "[DONE]" {
                return Value::String(trimmed.to_string());
            }
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                return value;
            }
        }
    }
    Value::Null
}

fn extract_openai_usage(data: &Value) -> Option<(u32, u32, u32)> {
    let usage_data = data.get("usage").filter(|usage| !usage.is_null())?;
    let input_tokens = usage_data
        .get("prompt_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = usage_data
        .get("completion_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let reasoning_tokens = usage_data
        .get("completion_tokens_details")
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    Some((input_tokens, output_tokens, reasoning_tokens))
}

fn extract_openai_stream_error(data: &Value) -> Option<String> {
    let error = data.get("error")?;
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| error.as_str())
        .unwrap_or("unknown provider stream error");
    let kind = error.get("type").and_then(Value::as_str);

    Some(match kind {
        Some(kind) if !kind.is_empty() => {
            format!("provider stream error ({kind}): {message}")
        }
        _ => format!("provider stream error: {message}"),
    })
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::openai_request::{
        EMPTY_ASSISTANT_MESSAGE, EMPTY_ASSISTANT_TOOL_CALL_MESSAGE, EMPTY_TOOL_RESULT_MESSAGE,
        EMPTY_USER_MESSAGE, OpenAIFunction, OpenAIMessage, OpenAIToolCall,
    };
    use serde_json::json;
    use talos_config::{ReasoningEffort, ReasoningOptions};
    use talos_core::message::{AssistantReasoning, ReasoningBlock};

    async fn spawn_chunked_sse_server(
        chunks: Vec<(Duration, String)>,
        close_after: Option<Duration>,
    ) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut req_buf = [0_u8; 1024];
            let _ = socket.read(&mut req_buf).await;

            let headers = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Type: text/event-stream\r\n",
                "Transfer-Encoding: chunked\r\n",
                "Connection: close\r\n\r\n"
            );
            socket.write_all(headers.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();

            for (delay, payload) in chunks {
                tokio::time::sleep(delay).await;
                let frame = format!("{:X}\r\n{}\r\n", payload.len(), payload);
                socket.write_all(frame.as_bytes()).await.unwrap();
                socket.flush().await.unwrap();
            }

            if let Some(delay) = close_after {
                tokio::time::sleep(delay).await;
            }

            let _ = socket.write_all(b"0\r\n\r\n").await;
            let _ = socket.flush().await;
        });

        format!("http://{addr}")
    }

    #[test]
    fn build_request_body_user_only() {
        let messages = vec![Message::User {
            content: "Hello".into(),
        }];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello");
    }

    #[test]
    fn build_request_body_requests_streaming_usage() {
        let body = build_request_body("gpt-4o", &[], &[], None, None);

        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
    }

    #[test]
    fn test_openai_reasoning_effort_request() {
        let body = build_request_body(
            "o3",
            &[],
            &[],
            Some(&ReasoningOptions {
                effort: Some(ReasoningEffort::High),
                budget_tokens: None,
                replay: true,
            }),
            Some(2048),
        );

        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["max_completion_tokens"], 2048);
    }

    #[test]
    fn test_openai_non_reasoning_keeps_body() {
        let body = build_request_body("gpt-4o", &[], &[], None, Some(2048));

        assert!(body.get("reasoning_effort").is_none());
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn build_request_body_keeps_system_message_first() {
        let messages = vec![
            Message::System {
                content: "Stable system prompt".into(),
                cache_markers: Vec::new(),
            },
            Message::User {
                content: "Hello".into(),
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "Stable system prompt");
        assert_eq!(body["messages"][1]["role"], "user");
    }

    #[test]
    fn build_request_body_with_tool_calls() {
        let messages = vec![
            Message::User {
                content: "List files".into(),
            },
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "ls"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: "file1.rs\nfile2.rs".into(),
                    is_error: false,
                },
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][1]["role"], "assistant");
        let tool_calls = body["messages"][1]["tool_calls"].as_array().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0]["id"], "call_1");
        assert_eq!(tool_calls[0]["type"], "function");
        assert_eq!(tool_calls[0]["function"]["name"], "bash");
    }

    #[test]
    fn build_request_body_strips_unmatched_assistant_tool_calls() {
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "ls"}),
                }],
                reasoning: None,
            },
            Message::User {
                content: "continue".into(),
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["role"], "assistant");
        assert!(body["messages"][0].get("tool_calls").is_none());
        assert_eq!(body["messages"][0]["content"], EMPTY_ASSISTANT_MESSAGE);
        assert_eq!(body["messages"][1]["role"], "user");
    }

    #[test]
    fn build_request_body_drops_orphan_tool_result() {
        let messages = vec![Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "file1.rs\nfile2.rs".into(),
                is_error: false,
            },
        }];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert!(body["messages"].as_array().unwrap().is_empty());
    }

    #[test]
    fn build_request_body_keeps_matched_tool_result() {
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "ls"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: "file1.rs\nfile2.rs".into(),
                    is_error: false,
                },
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][1]["role"], "tool");
        assert_eq!(body["messages"][1]["tool_call_id"], "call_1");
        assert_eq!(body["messages"][1]["content"], "file1.rs\nfile2.rs");
    }

    #[test]
    fn build_request_body_assistant_with_text() {
        let messages = vec![Message::Assistant {
            content: "I'll help with that.".into(),
            tool_calls: vec![],
            reasoning: None,
        }];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["role"], "assistant");
        assert_eq!(body["messages"][0]["content"], "I'll help with that.");
        assert!(body["messages"][0]["tool_calls"].is_null());
    }

    #[test]
    fn build_request_body_replaces_empty_text_content() {
        let messages = vec![
            Message::User {
                content: " ".into(),
            },
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "true"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: String::new(),
                    is_error: false,
                },
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["content"], EMPTY_USER_MESSAGE);
        assert_eq!(
            body["messages"][1]["content"],
            EMPTY_ASSISTANT_TOOL_CALL_MESSAGE
        );
        assert_eq!(body["messages"][2]["content"], EMPTY_TOOL_RESULT_MESSAGE);
    }

    #[test]
    fn build_request_body_assistant_tool_call_has_non_empty_content() {
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "true"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: "ok".into(),
                    is_error: false,
                },
            },
        ];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(
            body["messages"][0]["content"],
            EMPTY_ASSISTANT_TOOL_CALL_MESSAGE
        );
        assert!(body["messages"][0]["tool_calls"].is_array());
    }

    #[test]
    fn test_openai_reasoning_content_replay() {
        let messages = vec![Message::Assistant {
            content: "Result".into(),
            tool_calls: vec![],
            reasoning: Some(AssistantReasoning {
                provider: "my-gateway".into(),
                model: "glm-5".into(),
                blocks: vec![
                    ReasoningBlock::Plain {
                        text: "first ".into(),
                    },
                    ReasoningBlock::Thinking {
                        text: "ignored".into(),
                        signature: Some("sig".into()),
                    },
                    ReasoningBlock::Plain {
                        text: "second".into(),
                    },
                ],
            }),
        }];

        let body = build_request_body("glm-5", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["reasoning_content"], "first second");
    }

    #[test]
    fn openai_provider_default_base_url() {
        let provider = OpenAIProvider::new("sk-test", "gpt-4o");
        assert_eq!(provider.base_url, OPENAI_API_URL);
    }

    #[test]
    fn openai_provider_custom_base_url() {
        let provider =
            OpenAIProvider::new("sk-test", "gpt-4o").with_base_url("http://localhost:8080/v1");
        assert_eq!(provider.base_url, "http://localhost:8080/v1");
    }

    #[test]
    fn endpoint_url_appends_chat_completions_to_default_base() {
        let provider = OpenAIProvider::new("sk-test", "gpt-4o");
        assert_eq!(
            provider.endpoint_url(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn endpoint_url_appends_chat_completions_to_custom_base() {
        let provider = OpenAIProvider::new("sk-test", "glm-5")
            .with_base_url("https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1");
        assert_eq!(
            provider.endpoint_url(),
            "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
    }

    #[test]
    fn endpoint_url_strips_trailing_slash_before_appending() {
        let provider = OpenAIProvider::new("sk-test", "gpt-4o")
            .with_base_url("https://gateway.example.com/v1/");
        assert_eq!(
            provider.endpoint_url(),
            "https://gateway.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn extract_event_data_valid_json() {
        let event_text = "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n";
        let data = extract_event_data(event_text);
        assert!(data.is_object());
        assert_eq!(data["choices"][0]["delta"]["content"], "hello");
    }

    #[test]
    fn extract_event_data_done() {
        let event_text = "data: [DONE]\n\n";
        let data = extract_event_data(event_text);
        assert_eq!(data.as_str(), Some("[DONE]"));
    }

    #[test]
    fn extract_event_data_empty() {
        let event_text = "\n\n";
        let data = extract_event_data(event_text);
        assert!(data.is_null());
    }

    #[test]
    fn openai_stream_chunk_deserialize_text_only() {
        let json_str = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {"content": "Hello"},
                "finish_reason": null
            }]
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".into()));
        assert!(chunk.choices[0].delta.tool_calls.is_none());
        assert!(chunk.choices[0].delta.reasoning_content.is_none());
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn openai_stream_chunk_deserialize_reasoning_content() {
        let json_str = r#"{
            "choices": [{
                "delta": {"reasoning_content": "thinking chunk"},
                "finish_reason": null
            }]
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            chunk.choices[0].delta.reasoning_content,
            Some("thinking chunk".into())
        );
    }

    #[test]
    fn openai_stream_chunk_deserialize_tool_call() {
        let json_str = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "bash",
                            "arguments": "{\"command\": \"ls\"}"
                        }
                    }]
                },
                "finish_reason": null
            }]
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        let tool_calls = chunk.choices[0].delta.tool_calls.clone().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, Some("call_abc123".into()));
        assert_eq!(
            tool_calls[0].function.as_ref().unwrap().name,
            Some("bash".into())
        );
        assert_eq!(
            tool_calls[0].function.as_ref().unwrap().arguments,
            Some("{\"command\": \"ls\"}".into())
        );
    }

    #[test]
    fn openai_stream_chunk_deserialize_finish_reason() {
        let json_str = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }]
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        assert_eq!(chunk.choices[0].finish_reason, Some("stop".into()));
    }

    #[test]
    fn openai_stream_chunk_deserialize_finish_tool_calls() {
        let json_str = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "tool_calls"
            }]
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        assert_eq!(chunk.choices[0].finish_reason, Some("tool_calls".into()));
    }

    #[test]
    fn openai_stream_chunk_deserialize_empty_choices() {
        let json_str = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4o",
            "choices": []
        }"#;
        let chunk: OpenAIStreamChunk = serde_json::from_str(json_str).unwrap();
        assert!(chunk.choices.is_empty());
    }

    #[test]
    fn extract_openai_usage_reads_usage_only_chunk() {
        let data = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "choices": [],
            "usage": {
                "prompt_tokens": 123,
                "completion_tokens": 45,
                "completion_tokens_details": {
                    "reasoning_tokens": 12
                },
                "total_tokens": 168
            }
        });

        assert_eq!(extract_openai_usage(&data), Some((123, 45, 12)));
    }

    #[test]
    fn test_openai_reasoning_tokens_extraction() {
        let data = json!({
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "completion_tokens_details": {
                    "reasoning_tokens": 7
                }
            }
        });

        assert_eq!(extract_openai_usage(&data), Some((10, 20, 7)));
    }

    #[test]
    fn extract_openai_usage_ignores_null_usage() {
        let data = json!({
            "choices": [],
            "usage": null
        });

        assert_eq!(extract_openai_usage(&data), None);
    }

    #[test]
    fn extract_openai_stream_error_reads_object_error() {
        let data = json!({
            "error": {
                "message": "upstream failed",
                "type": "server_error"
            },
            "choices": []
        });

        assert_eq!(
            extract_openai_stream_error(&data),
            Some("provider stream error (server_error): upstream failed".to_string())
        );
    }

    #[tokio::test]
    async fn parse_sse_stream_retains_usage_only_chunk() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":123,\"completion_tokens\":45,\"completion_tokens_details\":{\"reasoning_tokens\":9},\"total_tokens\":168}}\n\n",
            "data: [DONE]\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(8);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut final_usage = None;
        while let Some(event) = rx.recv().await {
            if let AgentEvent::TurnEnd { usage, .. } = event {
                final_usage = Some(usage);
            }
        }

        let usage = final_usage.unwrap();
        assert_eq!(usage.input_tokens, 123);
        assert_eq!(usage.output_tokens, 45);
        assert_eq!(usage.reasoning_tokens, 9);
    }

    #[tokio::test]
    async fn parse_sse_stream_error_chunk_emits_terminal_error() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"partial\"},\"finish_reason\":null}]}\n\n",
            "data: {\"error\":{\"message\":\"gateway aborted\",\"type\":\"server_error\"},\"choices\":[]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut error = None;
        let mut saw_turn_end = false;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::Error { message } => error = Some(message),
                AgentEvent::TurnEnd { .. } => saw_turn_end = true,
                _ => {}
            }
        }

        assert_eq!(text, "partial");
        assert_eq!(
            error.as_deref(),
            Some("provider stream error (server_error): gateway aborted")
        );
        assert!(
            !saw_turn_end,
            "provider error chunks must not be converted to successful TurnEnd"
        );
    }

    #[tokio::test]
    async fn parse_sse_stream_synthesizes_missing_tool_call_id() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"type\":\"function\",\"function\":{\"name\":\"tree\",\"arguments\":\"{\\\"path\\\":\\\".\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut tool_call = None;
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolCall { call, .. } => tool_call = Some(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        let call = tool_call.expect("missing-id tool call should still be emitted");
        assert_eq!(call.id, "call_0");
        assert_eq!(call.name, "tree");
        assert_eq!(call.input, json!({ "path": "." }));
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn parse_sse_stream_accumulates_split_id_name_args_chunks() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_abc\",\"type\":\"function\"}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"get_weather\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"loc\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"ation\\\":\\\"Paris\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut tool_call = None;
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolCall { call, .. } => tool_call = Some(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        let call = tool_call.expect("split-chunk tool call should be accumulated and emitted");
        assert_eq!(call.id, "call_abc");
        assert_eq!(call.name, "get_weather");
        assert_eq!(call.input, json!({ "location": "Paris" }));
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn parse_sse_stream_empty_final_delta_clean_end_turn() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut stop_reason = None;
        let mut tool_call_started = false;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                AgentEvent::ToolCallStarted { .. } => tool_call_started = true,
                _ => {}
            }
        }

        assert_eq!(text, "hello");
        assert_eq!(stop_reason, Some(StopReason::EndTurn));
        assert!(
            !tool_call_started,
            "empty final delta must not produce a dangling ToolCallStarted"
        );
    }

    #[tokio::test]
    async fn parse_sse_stream_done_after_tool_calls_emits_tool_use() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_xyz\",\"type\":\"function\",\"function\":{\"name\":\"tree\",\"arguments\":\"{\\\"path\\\":\\\".\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let tool_call_started_count = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ToolCallStarted { .. }))
            .count();
        let tool_call = events
            .iter()
            .find_map(|e| match e {
                AgentEvent::ToolCall { call, .. } => Some(call.clone()),
                _ => None,
            })
            .expect("[DONE] after tool_calls must still emit a complete ToolCall");
        let stop_reason = events.iter().find_map(|e| match e {
            AgentEvent::TurnEnd { stop_reason, .. } => Some(stop_reason.clone()),
            _ => None,
        });

        assert_eq!(tool_call_started_count, 1);
        assert_eq!(tool_call.id, "call_xyz");
        assert_eq!(tool_call.name, "tree");
        assert_eq!(tool_call.input, json!({ "path": "." }));
        assert_eq!(
            stop_reason,
            Some(StopReason::ToolUse),
            "[DONE] after native tool calls must set ToolUse, not EndTurn"
        );
    }

    #[tokio::test]
    async fn parse_sse_stream_malformed_tool_arguments_becomes_empty_object() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_bad\",\"type\":\"function\",\"function\":{\"name\":\"bash\",\"arguments\":\"not valid json{\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut tool_call = None;
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolCall { call, .. } => tool_call = Some(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        let call = tool_call.expect("malformed-args tool call should still be emitted");
        assert_eq!(call.id, "call_bad");
        assert_eq!(call.name, "bash");
        assert_eq!(
            call.input,
            json!({}),
            "malformed JSON arguments should degrade to empty object, not panic"
        );
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn parse_sse_stream_usage_chunk_interleaved_with_tool_calls() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_u\",\"type\":\"function\",\"function\":{\"name\":\"read\",\"arguments\":\"{\\\"path\\\":\\\"a\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":50,\"completion_tokens\":5,\"completion_tokens_details\":{\"reasoning_tokens\":0},\"total_tokens\":55}}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut tool_call = None;
        let mut final_usage = None;
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolCall { call, .. } => tool_call = Some(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    usage,
                } => {
                    stop_reason = Some(reason);
                    final_usage = Some(usage);
                }
                _ => {}
            }
        }

        let call = tool_call.expect("tool call should be emitted despite interleaved usage chunk");
        assert_eq!(call.id, "call_u");
        assert_eq!(call.name, "read");
        assert_eq!(call.input, json!({ "path": "a" }));
        let usage = final_usage.expect("usage should be retained");
        assert_eq!(usage.input_tokens, 50);
        assert_eq!(usage.output_tokens, 5);
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn parse_sse_stream_multi_tool_missing_ids_synthesizes_unique_indices() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"type\":\"function\",\"function\":{\"name\":\"read\",\"arguments\":\"{\\\"p\\\":1}\"}},{\"index\":1,\"type\":\"function\",\"function\":{\"name\":\"write\",\"arguments\":\"{\\\"p\\\":2}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut tool_calls = Vec::new();
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolCall { call, .. } => tool_calls.push(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        assert_eq!(
            tool_calls.len(),
            2,
            "both missing-id tool calls should be emitted"
        );
        assert_eq!(tool_calls[0].id, "call_0");
        assert_eq!(tool_calls[0].name, "read");
        assert_eq!(tool_calls[1].id, "call_1");
        assert_eq!(tool_calls[1].name, "write");
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn test_openai_reasoning_content_stream() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"step 1 \"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"step 2\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut thinking_deltas = Vec::new();
        let mut reasoning_blocks = None;

        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ThinkingDelta { delta } => thinking_deltas.push(delta),
                AgentEvent::ReasoningComplete { blocks } => reasoning_blocks = Some(blocks),
                _ => {}
            }
        }

        assert_eq!(
            thinking_deltas,
            vec!["step 1 ".to_string(), "step 2".to_string()]
        );
        assert_eq!(
            reasoning_blocks,
            Some(vec![ReasoningBlock::Plain {
                text: "step 1 step 2".to_string(),
            }])
        );
    }

    #[tokio::test]
    async fn test_first_packet_timeout() {
        let url = spawn_chunked_sse_server(vec![], Some(Duration::from_secs(3))).await;
        let response = reqwest::get(url).await.unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(response, tx, Duration::from_secs(1), Duration::from_secs(2)).await;

        let mut timeout_error = None;
        while let Some(event) = rx.recv().await {
            if let AgentEvent::Error { message } = event {
                timeout_error = Some(message);
                break;
            }
        }

        assert_eq!(
            timeout_error.as_deref(),
            Some("first-packet timeout: no response from provider within 1s")
        );
    }

    #[tokio::test]
    async fn test_stream_idle_timeout() {
        let url = spawn_chunked_sse_server(
            vec![(
                Duration::from_millis(0),
                "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n"
                    .to_string(),
            )],
            Some(Duration::from_secs(3)),
        )
        .await;
        let response = reqwest::get(url).await.unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(response, tx, Duration::from_secs(1), Duration::from_secs(1)).await;

        let mut timeout_error = None;
        while let Some(event) = rx.recv().await {
            if let AgentEvent::Error { message } = event {
                timeout_error = Some(message);
                break;
            }
        }

        assert_eq!(
            timeout_error.as_deref(),
            Some("stream-idle timeout: provider stopped sending data for 1s")
        );
    }

    #[tokio::test]
    async fn test_normal_stream_not_timed_out() {
        let url = spawn_chunked_sse_server(
            vec![
                (
                    Duration::from_millis(0),
                    "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"},\"finish_reason\":null}]}\n\n"
                        .to_string(),
                ),
                (
                    Duration::from_millis(150),
                    "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":2,\"completion_tokens\":1}}\n\n"
                        .to_string(),
                ),
            ],
            None,
        )
        .await;
        let response = reqwest::get(url).await.unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(response, tx, Duration::from_secs(1), Duration::from_secs(1)).await;

        let mut saw_turn_end = false;
        let mut saw_timeout_error = false;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TurnEnd { .. } => saw_turn_end = true,
                AgentEvent::Error { message } if message.contains("timeout") => {
                    saw_timeout_error = true
                }
                _ => {}
            }
        }

        assert!(saw_turn_end);
        assert!(!saw_timeout_error);
    }

    // --- D101 coverage-extension fixtures (I102 / PROVIDER-002 / RUNTIME-002) ---
    // These lock already-implemented parser paths that FP1-FP2 did not have an
    // explicit fixture for. No production-code change should be required to make
    // any of these pass — they are deterministic regression guards.

    #[tokio::test]
    async fn parse_sse_stream_finish_reason_length_emits_max_tokens() {
        // `finish_reason: "length"` must surface as StopReason::MaxTokens so the
        // engine MaxTokens fix (FS04) actually fires when a provider hits the
        // token cap instead of `stop`.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"truncated\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"length\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut stop_reason = None;
        let mut text = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        assert_eq!(text, "truncated");
        assert_eq!(
            stop_reason,
            Some(StopReason::MaxTokens),
            "finish_reason=length must surface as StopReason::MaxTokens"
        );
    }

    #[tokio::test]
    async fn parse_sse_stream_role_only_first_chunk_does_not_emit_or_hang() {
        // Many OpenAI-compatible providers send a leading delta carrying only
        // `role: "assistant"`. The parser must consume it without emitting a
        // spurious event and without blocking the rest of the stream.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let text: String = events
            .iter()
            .filter_map(|e| match e {
                AgentEvent::TextDelta { delta } => Some(delta.clone()),
                _ => None,
            })
            .collect();
        let stop_reason = events.iter().find_map(|e| match e {
            AgentEvent::TurnEnd { stop_reason, .. } => Some(stop_reason.clone()),
            _ => None,
        });

        assert_eq!(text, "ok");
        assert_eq!(stop_reason, Some(StopReason::EndTurn));
    }

    #[tokio::test]
    async fn parse_sse_stream_keepalive_comment_lines_pass_through() {
        // SSE spec allows `: comment` lines and proxies often inject
        // `: keepalive` / `retry: 1500` directives. They must not break parsing
        // or be mistaken for an event payload.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            ": keepalive\n\n",
            "retry: 1500\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
            ": still alive\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        assert_eq!(text, "hi");
        assert_eq!(stop_reason, Some(StopReason::EndTurn));
    }

    #[tokio::test]
    async fn parse_sse_stream_empty_data_event_is_skipped() {
        // `data: ` (with no payload) or empty event blocks must not crash the
        // parser nor produce a spurious TurnEnd.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: \n\n",
            "\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut stop_reason = None;
        let mut error_events = 0;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                AgentEvent::Error { .. } => error_events += 1,
                _ => {}
            }
        }

        assert_eq!(text, "ok");
        assert_eq!(stop_reason, Some(StopReason::EndTurn));
        assert_eq!(error_events, 0, "empty data events must not raise errors");
    }

    #[tokio::test]
    async fn parse_sse_stream_mixed_text_and_tool_call_in_same_delta_emits_both() {
        // Some providers stream content and tool_calls in the same delta chunk.
        // The parser must emit both the text delta and the tool call.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"thinking...\",\"tool_calls\":[{\"index\":0,\"id\":\"call_mix\",\"type\":\"function\",\"function\":{\"name\":\"read\",\"arguments\":\"{\\\"p\\\":1}\"}}]},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut tool_call = None;
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::ToolCall { call, .. } => tool_call = Some(call),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        assert_eq!(text, "thinking...");
        let call = tool_call.expect("mixed-delta tool call must be emitted");
        assert_eq!(call.id, "call_mix");
        assert_eq!(call.name, "read");
        assert_eq!(call.input, json!({ "p": 1 }));
        assert_eq!(stop_reason, Some(StopReason::ToolUse));
    }

    #[tokio::test]
    async fn parse_sse_stream_utf8_multibyte_content_round_trips() {
        // Multi-byte UTF-8 content split across chunks must re-assemble without
        // mojibake. The parser reads `String::from_utf8(bytes)` per network
        // chunk, so a chunk boundary inside a multi-byte code point would
        // historically produce `continue`. This fixture confirms the current
        // behavior: if a chunk ends mid-code-point, that partial bytes are
        // dropped (lossy), but complete code points are preserved. We feed
        // whole-code-point chunks to keep this a regression guard, not a bug
        // hunt.
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"你好\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"世界\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        );
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;
        let response = reqwest::get(format!("{}/stream", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(16);

        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut text = String::new();
        let mut stop_reason = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::TurnEnd {
                    stop_reason: reason,
                    ..
                } => stop_reason = Some(reason),
                _ => {}
            }
        }

        assert_eq!(text, "你好世界");
        assert_eq!(stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn openai_message_struct_serialization() {
        let msg = OpenAIMessage {
            role: "user".into(),
            content: Some("Hello".into()),
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"], "Hello");
        assert!(json["tool_calls"].is_null());
        assert!(json["tool_call_id"].is_null());
        assert!(json["reasoning_content"].is_null());
    }

    #[test]
    fn openai_tool_call_struct_serialization() {
        let tc = OpenAIToolCall {
            id: "call_1".into(),
            call_type: "function".into(),
            function: OpenAIFunction {
                name: "bash".into(),
                arguments: "{\"command\": \"ls\"}".into(),
            },
        };
        let json = serde_json::to_value(&tc).unwrap();
        assert_eq!(json["id"], "call_1");
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "bash");
        assert_eq!(json["function"]["arguments"], "{\"command\": \"ls\"}");
    }

    async fn spawn_never_responding_server() -> String {
        use tokio::io::AsyncReadExt;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    tokio::time::sleep(Duration::from_secs(300)).await;
                }
            }
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn test_dispatch_timeout_openai() {
        let url = spawn_never_responding_server().await;
        let timeout_config = ProviderTimeoutConfig {
            dispatch_timeout_secs: 1,
            first_packet_timeout_secs: 30,
            stream_idle_timeout_secs: 90,
            max_attempts: 1,
            backoff_base_ms: 500,
            backoff_max_ms: 8_000,
        };
        let provider = OpenAIProvider::new("sk-test", "gpt-4o")
            .with_base_url(&url)
            .with_timeout_config(timeout_config);

        let messages = vec![Message::User {
            content: "hello".into(),
        }];
        let result = provider.stream(&messages).await;

        assert!(result.is_err(), "dispatch timeout must produce an error");
        let err = result.unwrap_err();
        match err {
            ProviderError::NetworkError(msg) => {
                assert!(
                    msg.contains("dispatch timeout"),
                    "error must mention dispatch timeout, got: {msg}"
                );
                assert!(
                    msg.contains("1s"),
                    "error must mention the configured timeout duration, got: {msg}"
                );
            }
            other => panic!("expected NetworkError for dispatch timeout, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_normal_request_not_dispatch_timed_out_openai() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        );
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(stream_body)
            .create_async()
            .await;

        let timeout_config = ProviderTimeoutConfig {
            dispatch_timeout_secs: 5,
            first_packet_timeout_secs: 30,
            stream_idle_timeout_secs: 90,
            max_attempts: 1,
            backoff_base_ms: 500,
            backoff_max_ms: 8_000,
        };
        let provider = OpenAIProvider::new("sk-test", "gpt-4o")
            .with_base_url(server.url())
            .with_timeout_config(timeout_config);

        let messages = vec![Message::User {
            content: "hello".into(),
        }];
        let result = provider.stream(&messages).await;
        assert!(
            result.is_ok(),
            "normal request must not be dispatch-timed-out"
        );
    }
}
