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

pub mod mock;
pub mod openai;
mod openai_request;

use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{Value, json};
use talos_config::{ProviderTimeoutConfig, ReasoningOptions};
use talos_core::message::ToolCall;
use talos_core::message::{
    AgentEvent, AssistantReasoning, Message, ReasoningBlock, StopReason, SystemCacheMarker,
    SystemCacheType, Usage,
};
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
        "body": build_request_body(model, messages, &[], None, None),
    })
}

fn build_request_body(
    model: &str,
    messages: &[Message],
    tools: &[ToolDefinition],
    reasoning: Option<&ReasoningOptions>,
    output_limit: Option<u32>,
) -> Value {
    let max_tokens = output_limit.unwrap_or(4096);
    let mut system_blocks = Vec::new();
    for msg in messages {
        if let Message::System {
            content,
            cache_markers,
        } = msg
        {
            system_blocks.extend(anthropic_system_blocks(content, cache_markers));
        }
    }

    let anthropic_messages: Vec<Value> = messages
        .iter()
        .filter(|msg| !matches!(msg, Message::System { .. }))
        .map(|msg| match msg {
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
                reasoning,
            } => {
                let mut blocks = Vec::new();

                if let Some(AssistantReasoning {
                    provider,
                    model: reasoning_model,
                    blocks: reasoning_blocks,
                }) = reasoning
                    && provider == "anthropic"
                    && reasoning_model == model
                {
                    for block in reasoning_blocks {
                        match block {
                            ReasoningBlock::Thinking { text, signature } => {
                                blocks.push(json!({
                                    "type": "thinking",
                                    "thinking": text,
                                    "signature": signature,
                                }));
                            }
                            ReasoningBlock::Redacted { data } => {
                                blocks.push(json!({
                                    "type": "redacted_thinking",
                                    "data": data,
                                }));
                            }
                            ReasoningBlock::Plain { .. } => {}
                        }
                    }
                }

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
            Message::Tool { result } => {
                let mut block = json!({
                    "type": "tool_result",
                    "tool_use_id": result.tool_use_id,
                    "content": result.content,
                });
                if result.is_error {
                    block["is_error"] = json!(true);
                }
                json!({
                    "role": "user",
                    "content": [block],
                })
            }
            Message::System { .. } => unreachable!("system messages are filtered above"),
        })
        .collect();

    let mut body = json!({
        "model": model,
        "messages": anthropic_messages,
        "max_tokens": max_tokens,
        "stream": true,
    });

    if let Some(reasoning) = reasoning {
        let mut budget_tokens = reasoning
            .budget_tokens
            .unwrap_or_else(|| max_tokens.saturating_mul(80) / 100);
        if budget_tokens >= max_tokens {
            budget_tokens = max_tokens.saturating_sub(1);
        }

        let mut include_thinking_param = true;
        if let Some(Message::Assistant {
            tool_calls,
            reasoning,
            ..
        }) = messages.last()
            && !tool_calls.is_empty()
        {
            let has_reasoning_blocks = reasoning.as_ref().is_some_and(|assistant_reasoning| {
                assistant_reasoning.provider == "anthropic"
                    && assistant_reasoning.model == model
                    && !assistant_reasoning.blocks.is_empty()
            });

            if !has_reasoning_blocks {
                include_thinking_param = false;
                tracing::warn!(
                    "anthropic: omitting thinking parameter for trailing tool_use assistant message without replayable reasoning blocks"
                );
            }
        }

        if include_thinking_param {
            body["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": budget_tokens,
            });
            body["temperature"] = json!(1);
        }
    }

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

    if !system_blocks.is_empty() {
        body["system"] = json!(system_blocks);
    }

    body
}

fn anthropic_system_blocks(content: &str, markers: &[SystemCacheMarker]) -> Vec<Value> {
    if content.is_empty() {
        return Vec::new();
    }

    if markers.is_empty() {
        return vec![json!({
            "type": "text",
            "text": content,
        })];
    }

    let mut blocks = Vec::new();
    let mut cursor = 0;
    let mut sorted_markers = markers.to_vec();
    sorted_markers.sort_by_key(|marker| marker.offset);

    for marker in sorted_markers {
        if marker.offset < cursor {
            continue;
        }
        let Some(marker_end) = marker.offset.checked_add(marker.length) else {
            return vec![json!({"type": "text", "text": content})];
        };
        if marker_end > content.len()
            || !content.is_char_boundary(marker.offset)
            || !content.is_char_boundary(marker_end)
        {
            return vec![json!({"type": "text", "text": content})];
        }
        if cursor < marker.offset
            && let Some(text) = content.get(cursor..marker.offset)
            && !text.is_empty()
        {
            blocks.push(json!({
                "type": "text",
                "text": text,
            }));
        }
        if let Some(text) = content.get(marker.offset..marker_end)
            && !text.is_empty()
        {
            let mut block = json!({
                "type": "text",
                "text": text,
            });
            if matches!(marker.cache_type, SystemCacheType::Ephemeral) {
                block["cache_control"] = json!({ "type": "ephemeral" });
            }
            blocks.push(block);
        }
        cursor = marker_end;
    }

    if cursor < content.len()
        && let Some(text) = content.get(cursor..)
        && !text.is_empty()
    {
        blocks.push(json!({
            "type": "text",
            "text": text,
        }));
    }

    if blocks.is_empty() {
        vec![json!({
            "type": "text",
            "text": content,
        })]
    } else {
        blocks
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

struct ToolUseBlock {
    id: String,
    name: String,
    input_json: String,
}

struct ThinkingBlockState {
    text: String,
    signature: String,
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
    let mut cache_read_tokens: u32 = 0;
    let mut cache_write_tokens: u32 = 0;
    let mut text_accumulator = String::new();
    let mut tool_use_blocks: std::collections::HashMap<u32, ToolUseBlock> =
        std::collections::HashMap::new();
    let mut current_thinking: Option<ThinkingBlockState> = None;
    let mut reasoning_blocks: Vec<ReasoningBlock> = Vec::new();
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
                    if let Some(block) = data.get("content_block") {
                        let block_type = block.get("type").and_then(|t| t.as_str());
                        if block_type == Some("tool_use") {
                            let index =
                                data.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32;
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
                                    name: name.clone(),
                                    input_json: String::new(),
                                },
                            );
                            let _ = tx.send(AgentEvent::ToolCallStarted { name }).await;
                        } else if block_type == Some("thinking") {
                            current_thinking = Some(ThinkingBlockState {
                                text: String::new(),
                                signature: String::new(),
                            });
                        } else if block_type == Some("redacted_thinking") {
                            let data = block
                                .get("data")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string();
                            reasoning_blocks.push(ReasoningBlock::Redacted { data });
                        }
                    }
                }
                Some("content_block_delta") => {
                    if let Some(text) = extract_text_delta(&data) {
                        text_accumulator.push_str(&text);
                        let _ = tx
                            .send(AgentEvent::TextDelta {
                                delta: text.clone(),
                            })
                            .await;
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
                    if let Some(delta) = data.get("delta") {
                        if delta.get("type").and_then(|t| t.as_str()) == Some("thinking_delta") {
                            if let Some(thinking_text) =
                                delta.get("thinking").and_then(|t| t.as_str())
                            {
                                if let Some(current) = current_thinking.as_mut() {
                                    current.text.push_str(thinking_text);
                                }
                                let _ = tx
                                    .send(AgentEvent::ThinkingDelta {
                                        delta: thinking_text.to_string(),
                                    })
                                    .await;
                            }
                        } else if delta.get("type").and_then(|t| t.as_str())
                            == Some("signature_delta")
                            && let Some(signature_text) =
                                delta.get("signature").and_then(|s| s.as_str())
                            && let Some(current) = current_thinking.as_mut()
                        {
                            current.signature.push_str(signature_text);
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
                                summary_fields: vec![],
                            })
                            .await;
                    }
                    if let Some(thinking) = current_thinking.take() {
                        let signature = if thinking.signature.is_empty() {
                            None
                        } else {
                            Some(thinking.signature)
                        };
                        reasoning_blocks.push(ReasoningBlock::Thinking {
                            text: thinking.text,
                            signature,
                        });
                    }
                }
                Some("message_delta") => {
                    if let Some(usage) = extract_usage_from_message_delta(&data) {
                        output_tokens = usage.output_tokens;
                        reasoning_tokens = usage.reasoning_tokens;
                    }
                    if let Some(stop_reason) = extract_stop_reason(&data) {
                        if !reasoning_blocks.is_empty() {
                            let _ = tx
                                .send(AgentEvent::ReasoningComplete {
                                    blocks: std::mem::take(&mut reasoning_blocks),
                                })
                                .await;
                        }
                        let tool_calls = parse_text_tool_calls(&text_accumulator);
                        for call in tool_calls {
                            let _ = tx
                                .send(AgentEvent::ToolCall {
                                    call,
                                    provenance: ToolProvenance::Native,
                                    summary_fields: vec![],
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
                                    reasoning_tokens,
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

    if !reasoning_blocks.is_empty() {
        let _ = tx
            .send(AgentEvent::ReasoningComplete {
                blocks: std::mem::take(&mut reasoning_blocks),
            })
            .await;
    }

    let _ = tx
        .send(AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_write_tokens,
                reasoning_tokens,
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

pub(crate) fn parse_json_tool_call(content: &str) -> Option<ToolCall> {
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
            reasoning_tokens: 0,
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
        reasoning_tokens: usage
            .get("output_tokens_details")
            .and_then(|details| details.get("thinking_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
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
#[allow(warnings)]
mod tests {
    use super::*;

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

    fn sse_event(event_type: &str, data: &Value) -> String {
        format!("event: {event_type}\ndata: {}\n\n", data)
    }

    #[test]
    fn build_request_body_user_only() {
        let messages = vec![Message::User {
            content: "Hello".into(),
        }];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

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
                reasoning: None,
            },
        ];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        assert_eq!(body["messages"][1]["role"], "assistant");
        let blocks = body["messages"][1]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "tool_use");
        assert_eq!(blocks[0]["id"], "call_1");
    }

    #[test]
    fn build_request_body_system_cache_control() {
        let content = "# Identity\nstable\n\n# Runtime Context\ndynamic\n";
        let messages = vec![
            Message::System {
                content: content.into(),
                cache_markers: vec![SystemCacheMarker {
                    offset: 0,
                    length: "# Identity\nstable\n".len(),
                    cache_type: SystemCacheType::Ephemeral,
                }],
            },
            Message::User {
                content: "Hello".into(),
            },
        ];

        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        assert_eq!(body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(body["messages"][0]["role"], "user");
        let system = body["system"].as_array().unwrap();
        assert_eq!(system[0]["type"], "text");
        assert_eq!(system[0]["cache_control"]["type"], "ephemeral");
        assert!(system[0]["text"].as_str().unwrap().contains("# Identity"));
        assert!(
            system[1]["text"]
                .as_str()
                .unwrap()
                .contains("# Runtime Context")
        );
        assert!(system[1].get("cache_control").is_none());
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

    #[tokio::test]
    async fn test_anthropic_thinking_delta_parsing() {
        let mut server = mockito::Server::new_async().await;
        let signature = "sig:abc+/=\n==";
        let body = format!(
            "{}{}{}{}{}{}",
            sse_event(
                "message_start",
                &json!({
                    "message": {
                        "usage": {
                            "input_tokens": 3,
                            "output_tokens": 0,
                            "cache_read_input_tokens": 0,
                            "cache_creation_input_tokens": 0
                        }
                    }
                })
            ),
            sse_event(
                "content_block_start",
                &json!({
                    "index": 0,
                    "content_block": { "type": "thinking" }
                })
            ),
            sse_event(
                "content_block_delta",
                &json!({
                    "index": 0,
                    "delta": { "type": "thinking_delta", "thinking": "step-1 " }
                })
            ),
            sse_event(
                "content_block_delta",
                &json!({
                    "index": 0,
                    "delta": { "type": "signature_delta", "signature": signature }
                })
            ),
            sse_event("content_block_stop", &json!({ "index": 0 })),
            sse_event(
                "message_delta",
                &json!({
                    "delta": { "stop_reason": "end_turn" },
                    "usage": {
                        "output_tokens": 4,
                        "output_tokens_details": { "thinking_tokens": 2 }
                    }
                })
            )
        );

        let _mock = server
            .mock("GET", "/thinking")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(body)
            .create_async()
            .await;

        let response = reqwest::get(format!("{}/thinking", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(32);
        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut thinking_deltas = Vec::new();
        let mut reasoning_complete = None;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ThinkingDelta { delta } => thinking_deltas.push(delta),
                AgentEvent::ReasoningComplete { blocks } => reasoning_complete = Some(blocks),
                _ => {}
            }
        }

        assert_eq!(thinking_deltas, vec!["step-1 ".to_string()]);
        let blocks = reasoning_complete.expect("missing reasoning complete event");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            ReasoningBlock::Thinking { text, signature: s } => {
                assert_eq!(text, "step-1 ");
                assert_eq!(s.as_deref(), Some(signature));
            }
            other => panic!("expected thinking block, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_anthropic_redacted_thinking_capture() {
        let mut server = mockito::Server::new_async().await;
        let redacted_data = "eyJyZWRhY3RlZCI6dHJ1ZSwic2lnIjoiKysvPSJ9";
        let body = format!(
            "{}{}{}{}",
            sse_event(
                "message_start",
                &json!({
                    "message": {
                        "usage": {
                            "input_tokens": 1,
                            "output_tokens": 0,
                            "cache_read_input_tokens": 0,
                            "cache_creation_input_tokens": 0
                        }
                    }
                })
            ),
            sse_event(
                "content_block_start",
                &json!({
                    "index": 0,
                    "content_block": {
                        "type": "redacted_thinking",
                        "data": redacted_data
                    }
                })
            ),
            sse_event("content_block_stop", &json!({ "index": 0 })),
            sse_event(
                "message_delta",
                &json!({
                    "delta": { "stop_reason": "end_turn" },
                    "usage": { "output_tokens": 1 }
                })
            )
        );

        let _mock = server
            .mock("GET", "/redacted")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(body)
            .create_async()
            .await;

        let response = reqwest::get(format!("{}/redacted", server.url()))
            .await
            .unwrap();
        let (tx, mut rx) = mpsc::channel(32);
        parse_sse_stream(
            response,
            tx,
            Duration::from_secs(30),
            Duration::from_secs(90),
        )
        .await;

        let mut reasoning_complete = None;
        while let Some(event) = rx.recv().await {
            if let AgentEvent::ReasoningComplete { blocks } = event {
                reasoning_complete = Some(blocks);
            }
        }

        let blocks = reasoning_complete.expect("missing reasoning complete event");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            ReasoningBlock::Redacted { data } => assert_eq!(data, redacted_data),
            other => panic!("expected redacted block, got {other:?}"),
        }
    }

    #[test]
    fn test_anthropic_reasoning_replay() {
        let signature = "sig-byte-identical+/=";
        let messages = vec![
            Message::User {
                content: "use tool".into(),
            },
            Message::Assistant {
                content: "done".into(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "ls"}),
                }],
                reasoning: Some(AssistantReasoning {
                    provider: "anthropic".into(),
                    model: "claude-sonnet-4-20250514".into(),
                    blocks: vec![
                        ReasoningBlock::Thinking {
                            text: "reason-step".into(),
                            signature: Some(signature.into()),
                        },
                        ReasoningBlock::Redacted {
                            data: "opaque-data".into(),
                        },
                    ],
                }),
            },
        ];

        let body = build_request_body(
            "claude-sonnet-4-20250514",
            &messages,
            &[],
            Some(&ReasoningOptions {
                effort: None,
                budget_tokens: Some(1000),
                replay: true,
            }),
            Some(4096),
        );

        let blocks = body["messages"][1]["content"].as_array().unwrap();
        assert_eq!(blocks[0]["type"], "thinking");
        assert_eq!(blocks[0]["thinking"], "reason-step");
        assert_eq!(blocks[0]["signature"], signature);
        assert_eq!(blocks[1]["type"], "redacted_thinking");
        assert_eq!(blocks[1]["data"], "opaque-data");
        assert_eq!(blocks[2]["type"], "text");
        assert_eq!(blocks[3]["type"], "tool_use");
    }

    #[test]
    fn test_anthropic_degradation_guardrail() {
        let messages = vec![Message::Assistant {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "bash".into(),
                input: json!({"command": "pwd"}),
            }],
            reasoning: None,
        }];

        let body = build_request_body(
            "claude-sonnet-4-20250514",
            &messages,
            &[],
            Some(&ReasoningOptions {
                effort: None,
                budget_tokens: Some(200),
                replay: true,
            }),
            Some(1024),
        );

        assert!(body.get("thinking").is_none());
        assert!(body.get("temperature").is_none());
    }

    #[test]
    fn test_anthropic_max_tokens_from_config() {
        let messages = vec![Message::User {
            content: "Hello".into(),
        }];

        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, Some(8192));
        assert_eq!(body["max_tokens"], 8192);
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
        let start_event = sse_event(
            "message_start",
            &json!({
                "message": {
                    "usage": {
                        "input_tokens": 1,
                        "output_tokens": 0,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0
                    }
                }
            }),
        );
        let url = spawn_chunked_sse_server(
            vec![(Duration::from_millis(0), start_event)],
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
        let stream = vec![
            (
                Duration::from_millis(0),
                sse_event(
                    "message_start",
                    &json!({
                        "message": {
                            "usage": {
                                "input_tokens": 1,
                                "output_tokens": 0,
                                "cache_read_input_tokens": 0,
                                "cache_creation_input_tokens": 0
                            }
                        }
                    }),
                ),
            ),
            (
                Duration::from_millis(150),
                sse_event(
                    "content_block_delta",
                    &json!({
                        "index": 0,
                        "delta": {
                            "type": "text_delta",
                            "text": "hello"
                        }
                    }),
                ),
            ),
            (
                Duration::from_millis(150),
                sse_event(
                    "message_delta",
                    &json!({
                        "delta": {"stop_reason": "end_turn"},
                        "usage": {"output_tokens": 2}
                    }),
                ),
            ),
        ];
        let url = spawn_chunked_sse_server(stream, None).await;
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
}
