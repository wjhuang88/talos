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
use talos_config::ReasoningOptions;
use talos_core::message::{AgentEvent, Message, ReasoningBlock, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;

use crate::openai_request::{build_request_body, redact_secret};
use crate::parse_text_tool_calls;

const OPENAI_API_URL: &str = "https://api.openai.com/v1";
const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";
const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 500;

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
        let mut attempt = 0;
        loop {
            let response = self
                .client
                .post(self.endpoint_url())
                .header("Authorization", format!("Bearer {}", &self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let status = response.status();

            if status.is_success() {
                return Ok(response);
            }

            let body_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(ProviderError::AuthenticationFailed(body_text));
            }

            if status.as_u16() == 429 {
                if attempt >= MAX_RETRIES {
                    return Err(ProviderError::RateLimited(body_text));
                }
                let delay = exponential_backoff(attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }

            if status.is_server_error() {
                if attempt >= MAX_RETRIES {
                    return Err(ProviderError::ServerError(body_text));
                }
                let delay = exponential_backoff(attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }

            return Err(ProviderError::InvalidResponse(format!(
                "unexpected status {}: {}",
                status, body_text
            )));
        }
    }
}

#[async_trait::async_trait]
impl LanguageModel for OpenAIProvider {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let response = self.make_request(messages).await?;
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(parse_sse_stream(response, tx));
        Ok(rx)
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let response = self.make_request_with_tools(messages, tools).await?;
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(parse_sse_stream(response, tx));
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

async fn parse_sse_stream(response: reqwest::Response, tx: mpsc::Sender<AgentEvent>) {
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
    while let Some(chunk_result) = stream.next().await {
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

                // Emit accumulated tool calls
                for i in 0..tool_call_ids.len() {
                    if !tool_call_ids[i].is_empty() && !tool_call_names[i].is_empty() {
                        let args: Value =
                            serde_json::from_str(&tool_call_args[i]).unwrap_or_else(|_| json!({}));
                        let _ = tx
                            .send(AgentEvent::ToolCall {
                                call: ToolCall {
                                    id: tool_call_ids[i].clone(),
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

    // Emit any accumulated native tool calls
    for i in 0..tool_call_ids.len() {
        if !tool_call_ids[i].is_empty() && !tool_call_names[i].is_empty() {
            let args: Value =
                serde_json::from_str(&tool_call_args[i]).unwrap_or_else(|_| json!({}));
            let _ = tx
                .send(AgentEvent::ToolCall {
                    call: ToolCall {
                        id: tool_call_ids[i].clone(),
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

fn exponential_backoff(attempt: u32) -> Duration {
    let delay_ms = BASE_RETRY_DELAY_MS * 2_u64.pow(attempt);
    Duration::from_millis(delay_ms)
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
    fn build_request_body_tool_result() {
        let messages = vec![Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "file1.rs\nfile2.rs".into(),
                is_error: false,
            },
        }];
        let body = build_request_body("gpt-4o", &messages, &[], None, None);

        assert_eq!(body["messages"][0]["role"], "tool");
        assert_eq!(body["messages"][0]["tool_call_id"], "call_1");
        assert_eq!(body["messages"][0]["content"], "file1.rs\nfile2.rs");
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
                tool_calls: vec![],
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
        assert_eq!(body["messages"][1]["content"], EMPTY_ASSISTANT_MESSAGE);
        assert_eq!(body["messages"][2]["content"], EMPTY_TOOL_RESULT_MESSAGE);
    }

    #[test]
    fn build_request_body_assistant_tool_call_has_non_empty_content() {
        let messages = vec![Message::Assistant {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "bash".into(),
                input: json!({"command": "true"}),
            }],
            reasoning: None,
        }];
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
    fn exponential_backoff_increases() {
        let d0 = exponential_backoff(0);
        let d1 = exponential_backoff(1);
        let d2 = exponential_backoff(2);
        assert!(d0 < d1);
        assert!(d1 < d2);
        assert_eq!(d0, Duration::from_millis(500));
        assert_eq!(d1, Duration::from_millis(1000));
        assert_eq!(d2, Duration::from_millis(2000));
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

        parse_sse_stream(response, tx).await;

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

        parse_sse_stream(response, tx).await;

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
}
