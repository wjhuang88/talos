//! Talos provider — LLM client abstractions and provider-specific implementations.

pub mod mock;
pub mod openai;

use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{Value, json};
use talos_core::message::ToolCall;
use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;
use uuid::Uuid;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 500;

/// Anthropic Claude provider implementing [`LanguageModel`].
///
/// Streams text deltas via SSE from the Anthropic Messages API,
/// handles errors gracefully, and supports exponential backoff retry.
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
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
        }
    }

    /// Set a custom base URL (useful for testing or enterprise proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn make_request(&self, messages: &[Message]) -> ProviderResult<reqwest::Response> {
        let body = build_request_body(&self.model, messages, &[]);
        self.send_request(&body).await
    }

    async fn make_request_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<reqwest::Response> {
        let body = build_request_body(&self.model, messages, tools);
        self.send_request(&body).await
    }

    async fn send_request(&self, body: &Value) -> ProviderResult<reqwest::Response> {
        let mut attempt = 0;
        loop {
            let response = self
                .client
                .post(&self.base_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let status = response.status();

            if status.is_success() {
                return Ok(response);
            }

            let body_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
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
impl LanguageModel for AnthropicProvider {
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
        let body = build_request_body(&self.model, messages, &[]);
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

/// Build a redacted Anthropic Messages API request snapshot for mock diagnostics.
pub fn anthropic_request_debug_snapshot(
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    messages: &[Message],
) -> Value {
    json!({
        "method": "POST",
        "url": base_url.unwrap_or(ANTHROPIC_API_URL),
        "headers": {
            "x-api-key": redact_secret(api_key),
            "anthropic-version": ANTHROPIC_VERSION,
            "content-type": "application/json",
        },
        "body": build_request_body(model, messages, &[]),
    })
}

fn build_request_body(model: &str, messages: &[Message], tools: &[ToolDefinition]) -> Value {
    let anthropic_messages: Vec<Value> = messages
        .iter()
        .map(|msg| match msg {
            Message::System { content } => json!({
                "role": "user",
                "content": content,
            }),
            Message::Context { content } => json!({
                "role": "user",
                "content": content,
            }),
            Message::User { content } => json!({
                "role": "user",
                "content": content,
            }),
            Message::Assistant {
                content,
                tool_calls,
            } => {
                let mut blocks = Vec::new();
                if !content.is_empty() {
                    blocks.push(json!({
                        "type": "text",
                        "text": content,
                    }));
                }
                for tc in tool_calls {
                    blocks.push(json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.name,
                        "input": tc.input,
                    }));
                }
                json!({
                    "role": "assistant",
                    "content": blocks,
                })
            }
            Message::Tool { result } => json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": result.tool_use_id,
                    "content": result.content,
                }],
            }),
        })
        .collect();

    let mut body = json!({
        "model": model,
        "messages": anthropic_messages,
        "max_tokens": 4096,
        "stream": true,
    });

    if !tools.is_empty() {
        let tools_json: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect();
        body["tools"] = json!(tools_json);
    }

    body
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

struct ToolUseBlock {
    id: String,
    name: String,
    input_json: String,
}

async fn parse_sse_stream(response: reqwest::Response, tx: mpsc::Sender<AgentEvent>) {
    let _ = tx.send(AgentEvent::TurnStart).await;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut input_tokens: u32 = 0;
    let mut output_tokens: u32 = 0;
    let mut cache_read_tokens: u32 = 0;
    let mut cache_write_tokens: u32 = 0;
    let mut text_accumulator = String::new();
    let mut syntax_filter = talos_core::tool_filter::ToolSyntaxFilter::new();
    let mut tool_use_blocks: std::collections::HashMap<u32, ToolUseBlock> =
        std::collections::HashMap::new();

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

            let event_type = extract_event_type(&event_text);
            let data = extract_event_data(&event_text);

            match event_type.as_deref() {
                Some("message_start") => {
                    if let Some(usage) = extract_usage_from_message_start(&data) {
                        input_tokens = usage.input_tokens;
                        cache_read_tokens = usage.cache_read_tokens;
                        cache_write_tokens = usage.cache_write_tokens;
                    }
                }
                Some("content_block_start") => {
                    if let Some(block) = data.get("content_block")
                        && block.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                    {
                        let index = data.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32;
                        let id = block
                            .get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();
                        let name = block
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string();
                        tool_use_blocks.insert(
                            index,
                            ToolUseBlock {
                                id,
                                name,
                                input_json: String::new(),
                            },
                        );
                    }
                }
                Some("content_block_delta") => {
                    if let Some(text) = extract_text_delta(&data) {
                        text_accumulator.push_str(&text);
                        let clean = syntax_filter.push_chunk(&text);
                        if !clean.is_empty() {
                            let _ = tx.send(AgentEvent::TextDelta { delta: clean }).await;
                        }
                    }
                    if let Some(partial) = data.get("delta")
                        && partial.get("type").and_then(|t| t.as_str()) == Some("input_json_delta")
                    {
                        let index = data.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32;
                        if let Some(json_str) = partial.get("partial_json").and_then(|p| p.as_str())
                            && let Some(block) = tool_use_blocks.get_mut(&index)
                        {
                            block.input_json.push_str(json_str);
                        }
                    }
                }
                Some("content_block_stop") => {
                    let index = data.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32;
                    if let Some(block) = tool_use_blocks.remove(&index) {
                        let input_json: serde_json::Value = serde_json::from_str(&block.input_json)
                            .unwrap_or(serde_json::json!({}));
                        let _ = tx
                            .send(AgentEvent::ToolCall {
                                call: ToolCall {
                                    id: block.id,
                                    name: block.name,
                                    input: input_json,
                                },
                                provenance: ToolProvenance::Native,
                            })
                            .await;
                    }
                }
                Some("message_delta") => {
                    if let Some(usage) = extract_usage_from_message_delta(&data) {
                        output_tokens = usage.output_tokens;
                    }
                    if let Some(stop_reason) = extract_stop_reason(&data) {
                        let remaining = syntax_filter.finish();
                        if !remaining.is_empty() {
                            let _ = tx.send(AgentEvent::TextDelta { delta: remaining }).await;
                        }
                        let tool_calls = parse_text_tool_calls(&text_accumulator);
                        for call in tool_calls {
                            let _ = tx
                                .send(AgentEvent::ToolCall {
                                    call,
                                    provenance: ToolProvenance::Native,
                                })
                                .await;
                        }
                        let _ = tx
                            .send(AgentEvent::TurnEnd {
                                stop_reason,
                                usage: Usage {
                                    input_tokens,
                                    output_tokens,
                                    cache_read_tokens,
                                    cache_write_tokens,
                                },
                            })
                            .await;
                        return;
                    }
                }
                Some("error") => {
                    let message = extract_error_message(&data);
                    let _ = tx
                        .send(AgentEvent::Error {
                            message: message.unwrap_or_else(|| "unknown error".into()),
                        })
                        .await;
                    return;
                }
                _ => {}
            }
        }
    }

    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_write_tokens,
            },
        })
        .await;
}

pub(crate) fn parse_text_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("```json-tool") {
        let inner_start = start + "```json-tool".len();
        let inner = remaining[inner_start..].trim_start();
        let end = inner.find("```").unwrap_or(inner.len());
        let content = inner[..end].trim();

        if let Some(call) = parse_json_tool_call(content) {
            calls.push(call);
        }

        remaining = &inner[end..];
        if end + 3 < remaining.len() {
            remaining = &remaining[3..];
        } else {
            break;
        }
    }

    // Fallback: also check for <tool_call> / <toolcall> XML tags
    while let Some(start) = remaining
        .find("<tool_call>")
        .or_else(|| remaining.find("<toolcall>"))
    {
        let tag_len = if remaining[start..].starts_with("<tool_call>") {
            "<tool_call>".len()
        } else {
            "<toolcall>".len()
        };
        let inner_start = start + tag_len;
        let inner = &remaining[inner_start..];
        let end = inner.find("</tool_call>").unwrap_or(inner.len());
        let content = inner[..end].trim();

        if let Some(call) = parse_json_tool_call(content) {
            calls.push(call);
        }

        remaining = &inner[end..];
        let close_len = "</tool_call>".len();
        if end + close_len < remaining.len() {
            remaining = &remaining[close_len..];
        } else {
            break;
        }
    }

    calls
}

fn parse_json_tool_call(content: &str) -> Option<ToolCall> {
    let content = content.trim().trim_matches('`').trim();

    // Try: entire content is JSON: {"name":"bash","args":{...}}
    if content.starts_with('{') {
        let obj: serde_json::Value = serde_json::from_str(content).ok()?;
        let name = obj.get("name")?.as_str()?.to_string();
        let args = obj.get("args")?.clone();
        return Some(ToolCall {
            id: Uuid::new_v4().to_string(),
            name,
            input: args,
        });
    }

    // Try: name is first word, JSON follows somewhere in content
    let first_space = content.find(|c: char| c.is_whitespace())?;
    let name = content[..first_space].trim().to_string();

    let rest = content[first_space..].trim();
    if let Some(brace_start) = rest.find('{') {
        let json_str = &rest[brace_start..];
        if let Some(brace_end) = json_str.rfind('}') {
            let json_str = &json_str[..=brace_end];
            if let Ok(args) = serde_json::from_str::<serde_json::Value>(json_str) {
                return Some(ToolCall {
                    id: Uuid::new_v4().to_string(),
                    name,
                    input: args,
                });
            }
        }
    }

    // Try: name is first word, rest is key=value pairs
    if let Ok(args) = serde_json::from_str::<serde_json::Value>(rest) {
        return Some(ToolCall {
            id: Uuid::new_v4().to_string(),
            name,
            input: args,
        });
    }

    None
}

fn extract_event_type(event_text: &str) -> Option<String> {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("event:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn extract_event_data(event_text: &str) -> Value {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("data:")
            && let Ok(value) = serde_json::from_str::<Value>(rest.trim())
        {
            return value;
        }
    }
    Value::Null
}

fn extract_usage_from_message_start(data: &Value) -> Option<Usage> {
    data.get("message")
        .and_then(|msg| msg.get("usage"))
        .map(|usage| Usage {
            input_tokens: usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            output_tokens: usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            cache_read_tokens: usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            cache_write_tokens: usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
}

fn extract_usage_from_message_delta(data: &Value) -> Option<Usage> {
    data.get("usage").map(|usage| Usage {
        input_tokens: 0,
        output_tokens: usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
    })
}

fn extract_text_delta(data: &Value) -> Option<String> {
    data.get("delta")
        .and_then(|delta| delta.get("text"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn extract_stop_reason(data: &Value) -> Option<StopReason> {
    data.get("delta")
        .and_then(|delta| delta.get("stop_reason"))
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "end_turn" => StopReason::EndTurn,
            "tool_use" => StopReason::ToolUse,
            "max_tokens" => StopReason::MaxTokens,
            _ => StopReason::EndTurn,
        })
}

fn extract_error_message(data: &Value) -> Option<String> {
    data.get("error")
        .and_then(|err| err.get("message"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn exponential_backoff(attempt: u32) -> Duration {
    let delay_ms = BASE_RETRY_DELAY_MS * 2_u64.pow(attempt);
    Duration::from_millis(delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_body_user_only() {
        let messages = vec![Message::User {
            content: "Hello".into(),
        }];
        let body = build_request_body("claude-sonnet-4-20250514", &messages);

        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 4096);
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello");
    }

    #[test]
    fn build_request_body_with_tool_calls() {
        let messages = vec![
            Message::User {
                content: "List files".into(),
            },
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![talos_core::message::ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "ls"}),
                }],
            },
        ];
        let body = build_request_body("claude-sonnet-4-20250514", &messages);

        assert_eq!(body["messages"][1]["role"], "assistant");
        let blocks = body["messages"][1]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "tool_use");
        assert_eq!(blocks[0]["id"], "call_1");
    }

    #[test]
    fn extract_text_delta_valid() {
        let data = json!({
            "delta": {
                "type": "text_delta",
                "text": "Hello, world!"
            }
        });
        assert_eq!(extract_text_delta(&data), Some("Hello, world!".into()));
    }

    #[test]
    fn extract_stop_reason_end_turn() {
        let data = json!({
            "delta": {
                "stop_reason": "end_turn",
                "stop_sequence": null
            }
        });
        assert_eq!(extract_stop_reason(&data), Some(StopReason::EndTurn));
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
}
