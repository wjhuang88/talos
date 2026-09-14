//! Anthropic Messages API SSE stream parsing.

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;
use talos_core::message::ToolCall;
use talos_core::message::{AgentEvent, ReasoningBlock, StopReason, Usage};
use talos_core::tool::ToolProvenance;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::stream_utf8::Utf8StreamDecoder;

struct ToolUseBlock {
    id: String,
    name: String,
    input_json: String,
    initial_input: Value,
}

impl ToolUseBlock {
    fn arguments(&self) -> Result<Value, ()> {
        let input = if self.input_json.is_empty() {
            self.initial_input.clone()
        } else {
            serde_json::from_str(&self.input_json).map_err(|_| ())?
        };
        if input.is_object() {
            Ok(input)
        } else {
            Err(())
        }
    }
}

struct ThinkingBlockState {
    text: String,
    signature: String,
}

pub(crate) async fn parse_sse_stream(
    response: reqwest::Response,
    tx: mpsc::Sender<AgentEvent>,
    first_packet_timeout: Duration,
    idle_timeout: Duration,
) {
    parse_sse_stream_with_mode(
        response,
        tx,
        first_packet_timeout,
        idle_timeout,
        talos_core::tool::ToolProtocol::Compat,
    )
    .await;
}

pub(crate) async fn parse_sse_stream_with_mode(
    response: reqwest::Response,
    tx: mpsc::Sender<AgentEvent>,
    first_packet_timeout: Duration,
    idle_timeout: Duration,
    protocol: talos_core::tool::ToolProtocol,
) {
    let _ = tx.send(AgentEvent::TurnStart).await;

    let mut stream = response.bytes_stream();
    let mut utf8_decoder = Utf8StreamDecoder::default();
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
        let timeout = if saw_first_packet {
            idle_timeout
        } else {
            first_packet_timeout
        };
        let wait_result = tokio::select! {
            biased;
            _ = tx.closed() => return,
            result = tokio::time::timeout(timeout, next_chunk) => result,
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
            Ok(bytes) => match utf8_decoder.push(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    let _ = tx
                        .send(AgentEvent::Error {
                            message: "provider stream decode error: invalid UTF-8 byte stream"
                                .into(),
                        })
                        .await;
                    return;
                }
            },
            Err(_) => {
                let _ = tx
                    .send(AgentEvent::Error {
                        message: "provider stream transport read error".into(),
                    })
                    .await;
                return;
            }
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
                                    initial_input: block
                                        .get("input")
                                        .cloned()
                                        .unwrap_or(Value::Null),
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
                        let input_json = match block.arguments() {
                            Ok(input) => input,
                            Err(()) => {
                                let _ = tx
                                    .send(AgentEvent::Error {
                                        message: "invalid tool arguments JSON".into(),
                                    })
                                    .await;
                                return;
                            }
                        };
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
                    if let Some(stop_reason) = match extract_stop_reason(&data) {
                        Ok(reason) => reason,
                        Err(message) => {
                            let _ = tx.send(AgentEvent::Error { message }).await;
                            return;
                        }
                    } {
                        // Some Anthropic-compatible providers (including MiniMax)
                        // omit `content_block_stop` before the final tool_use
                        // message delta. Flush any started blocks so the agent
                        // never observes ToolUse without a corresponding call.
                        for (_, block) in std::mem::take(&mut tool_use_blocks) {
                            if block.name.is_empty() {
                                continue;
                            }
                            let input = match block.arguments() {
                                Ok(input) => input,
                                Err(_) => {
                                    let _ = tx
                                        .send(AgentEvent::Error {
                                            message: "invalid tool arguments JSON".into(),
                                        })
                                        .await;
                                    return;
                                }
                            };
                            let _ = tx
                                .send(AgentEvent::ToolCall {
                                    call: ToolCall {
                                        id: if block.id.is_empty() {
                                            format!("call_{}", Uuid::new_v4())
                                        } else {
                                            block.id
                                        },
                                        name: block.name,
                                        input,
                                    },
                                    provenance: ToolProvenance::Native,
                                    summary_fields: vec![],
                                })
                                .await;
                        }
                        if !reasoning_blocks.is_empty() {
                            let _ = tx
                                .send(AgentEvent::ReasoningComplete {
                                    blocks: std::mem::take(&mut reasoning_blocks),
                                })
                                .await;
                        }
                        let tool_calls = if protocol != talos_core::tool::ToolProtocol::Native {
                            match parse_protocol_text_calls(&text_accumulator, protocol) {
                                Ok(calls) => calls,
                                Err(message) => {
                                    let _ = tx
                                        .send(AgentEvent::Error {
                                            message: message.into(),
                                        })
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            Vec::new()
                        };
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

    if utf8_decoder.finish().is_err() {
        let _ = tx
            .send(AgentEvent::Error {
                message: "provider stream decode error: invalid UTF-8 byte stream".into(),
            })
            .await;
        return;
    }

    let _ = tx
        .send(AgentEvent::Error {
            message: "provider stream closed without explicit terminal signal (message_delta stop_reason)".into(),
        })
        .await;
}

pub(crate) fn parse_protocol_text_calls(
    text: &str,
    protocol: talos_core::tool::ToolProtocol,
) -> Result<Vec<ToolCall>, &'static str> {
    use talos_core::tool::ToolProtocol;
    if protocol == ToolProtocol::Native {
        return Ok(Vec::new());
    }
    if protocol == ToolProtocol::Auto {
        return Err("automatic protocol must be resolved before parsing");
    }
    let calls = parse_text_tool_calls(text)?;
    if protocol == ToolProtocol::TalosStrict && !calls.is_empty() {
        let content = text
            .trim_start()
            .strip_prefix("<tool_call>")
            .ok_or("strict tool call must precede text and use the canonical XML tag")?;
        let end = content
            .find("</tool_call>")
            .ok_or("unterminated strict tool block")?;
        let object: Value = serde_json::from_str(content[..end].trim())
            .map_err(|_| "strict tool payload must be JSON")?;
        if calls.len() != 1 || !object.is_object() {
            return Err("strict protocol requires exactly one JSON tool block");
        }
    }
    Ok(calls)
}

pub(crate) fn parse_text_tool_calls(text: &str) -> Result<Vec<ToolCall>, &'static str> {
    let mut calls = Vec::new();
    let mut remaining = text;
    let frames = [
        ("```json-tool", "```"),
        ("<tool_call>", "</tool_call>"),
        ("<toolcall>", "</toolcall>"),
    ];
    while let Some((start, open, close)) = frames
        .iter()
        .filter_map(|(open, close)| remaining.find(open).map(|start| (start, *open, *close)))
        .min_by_key(|(start, _, _)| *start)
    {
        let inner = &remaining[start + open.len()..];
        let end = inner
            .find(close)
            .ok_or("unterminated compatibility tool block")?;
        let call = parse_json_tool_call(inner[..end].trim())
            .ok_or("invalid compatibility tool payload")?;
        if call.name.trim().is_empty() || !call.input.is_object() {
            return Err("invalid compatibility tool name or arguments");
        }
        calls.push(call);
        remaining = &inner[end + close.len()..];
    }
    Ok(calls)
}

pub(crate) fn parse_json_tool_call(content: &str) -> Option<ToolCall> {
    let content = content.trim();

    // Try: entire content is JSON: {"name":"bash","args":{...}}
    if content.starts_with('{') {
        let obj: serde_json::Value = serde_json::from_str(content).ok()?;
        let name = obj.get("name")?.as_str()?.to_string();
        let args = obj.get("args")?.clone();
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        return Some(ToolCall {
            id,
            name,
            input: args,
        });
    }

    // Legacy compatibility form: a tool name followed by one complete JSON object.
    let first_space = content.find(|c: char| c.is_whitespace())?;
    let name = content[..first_space].trim().to_string();

    let rest = content[first_space..].trim();
    if let Ok(args) = serde_json::from_str::<serde_json::Value>(rest) {
        return Some(ToolCall {
            id: Uuid::new_v4().to_string(),
            name,
            input: args,
        });
    }

    None
}

#[cfg(test)]
mod compatibility_batch_tests {
    use super::parse_text_tool_calls;

    #[test]
    fn complete_mixed_frames_preserve_order_and_reject_partial_batches() {
        let first = "<toolcall>{\"name\":\"first\",\"args\":{}}</toolcall>";
        let second = "```json-tool\n{\"name\":\"second\",\"args\":{}}\n```";
        let text = format!("{first}\n{second}");
        let calls = parse_text_tool_calls(&text).expect("valid frames");
        assert_eq!(
            calls
                .iter()
                .map(|call| call.name.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        for bad in [
            "<tool_call>probe {} ignored</tool_call>",
            "<toolcall>probe {}</tool_call>",
            "```json-tool\n{\"name\":\"\",\"args\":{}}\n```",
        ] {
            assert!(parse_text_tool_calls(&format!("{text}{bad}")).is_err());
        }
        assert!(
            parse_text_tool_calls("ordinary prose")
                .expect("no frames")
                .is_empty()
        );
    }
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

fn extract_stop_reason(data: &Value) -> Result<Option<StopReason>, String> {
    let Some(reason) = data
        .get("delta")
        .and_then(|delta| delta.get("stop_reason"))
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };
    let stop_reason = match reason {
        "end_turn" | "stop_sequence" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        "pause_turn" => {
            return Err(
                "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported"
                    .into(),
            );
        }
        "refusal" => {
            return Err("provider refused request (stop_reason=refusal)".into());
        }
        unknown => {
            return Err(format!(
                "unsupported provider stop_reason: {}",
                bounded_terminal_reason(unknown)
            ));
        }
    };
    Ok(Some(stop_reason))
}

fn bounded_terminal_reason(reason: &str) -> String {
    const MAX_REASON_CHARS: usize = 120;
    let bounded = reason
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_REASON_CHARS)
        .collect::<String>();
    if bounded.is_empty() {
        "unknown".into()
    } else {
        bounded
    }
}

fn extract_error_message(data: &Value) -> Option<String> {
    data.get("error")
        .and_then(|err| err.get("message"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::AnthropicProvider;
    use serde_json::json;
    use talos_config::ProviderTimeoutConfig;
    use talos_core::message::Message;
    use talos_core::provider::LanguageModel;
    use talos_core::provider::ProviderError;

    async fn spawn_chunked_sse_server(
        chunks: Vec<(Duration, String)>,
        close_after: Option<Duration>,
    ) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("operation should succeed");
        let addr = listener.local_addr().expect("operation should succeed");

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("operation should succeed");
            let mut req_buf = [0_u8; 1024];
            let _ = socket.read(&mut req_buf).await;

            let headers = concat!(
                "HTTP/1.1 200 OK\r\n",
                "Content-Type: text/event-stream\r\n",
                "Transfer-Encoding: chunked\r\n",
                "Connection: close\r\n\r\n"
            );
            socket
                .write_all(headers.as_bytes())
                .await
                .expect("operation should succeed");
            socket.flush().await.expect("operation should succeed");

            for (delay, payload) in chunks {
                tokio::time::sleep(delay).await;
                let frame = format!("{:X}\r\n{}\r\n", payload.len(), payload);
                socket
                    .write_all(frame.as_bytes())
                    .await
                    .expect("operation should succeed");
                socket.flush().await.expect("operation should succeed");
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
        assert_eq!(extract_stop_reason(&data), Ok(Some(StopReason::EndTurn)));
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
            .expect("operation should succeed");
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
            .expect("operation should succeed");
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

    #[tokio::test]
    async fn test_first_packet_timeout() {
        let url = spawn_chunked_sse_server(vec![], Some(Duration::from_secs(3))).await;
        let response = reqwest::get(url).await.expect("operation should succeed");
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
        let response = reqwest::get(url).await.expect("operation should succeed");
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
        let response = reqwest::get(url).await.expect("operation should succeed");
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

    async fn spawn_never_responding_server() -> String {
        use tokio::io::AsyncReadExt;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("operation should succeed");
        let addr = listener.local_addr().expect("operation should succeed");

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
    async fn test_dispatch_timeout_anthropic() {
        let url = spawn_never_responding_server().await;
        let timeout_config = ProviderTimeoutConfig {
            dispatch_timeout_secs: 1,
            first_packet_timeout_secs: 30,
            stream_idle_timeout_secs: 90,
            max_attempts: 1,
            backoff_base_ms: 500,
            backoff_max_ms: 8_000,
        };
        let provider = AnthropicProvider::new("sk-test", "claude-sonnet-4-20250514")
            .with_base_url(&url)
            .with_timeout_config(timeout_config);

        let messages = vec![Message::User {
            content: "hello".into(),
        }];
        let result = provider.stream(&messages).await;

        assert!(result.is_err(), "dispatch timeout must produce an error");
        let err = result.expect_err("operation should fail");
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
    async fn test_normal_request_not_dispatch_timed_out_anthropic() {
        let mut server = mockito::Server::new_async().await;
        let stream_body = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-sonnet-4-20250514\",\"stop_reason\":null,\"usage\":{\"input_tokens\":10,\"output_tokens\":1}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n"
        );
        let _mock = server
            .mock("POST", "/")
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
        let provider = AnthropicProvider::new("sk-test", "claude-sonnet-4-20250514")
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

#[cfg(test)]
mod i168_terminal_outcome_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn parse_body(body: String) -> Vec<AgentEvent> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("operation should succeed");
        let addr = listener.local_addr().expect("operation should succeed");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("operation should succeed");
            let mut request = [0_u8; 2048];
            let _ = socket.read(&mut request).await;
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket
                .write_all(headers.as_bytes())
                .await
                .expect("operation should succeed");
            socket
                .write_all(body.as_bytes())
                .await
                .expect("operation should succeed");
            socket.flush().await.expect("operation should succeed");
        });

        let response = reqwest::get(format!("http://{addr}/stream"))
            .await
            .expect("operation should succeed");
        let (tx, mut rx) = mpsc::channel(32);
        parse_sse_stream(response, tx, Duration::from_secs(5), Duration::from_secs(5)).await;
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    }

    fn terminal_error(events: &[AgentEvent]) -> &str {
        events
            .iter()
            .find_map(|event| match event {
                AgentEvent::Error { message } => Some(message.as_str()),
                _ => None,
            })
            .expect("terminal error")
    }

    #[tokio::test]
    async fn malformed_native_tool_arguments_stop_before_tool_event() {
        for block_stop in [false, true] {
            for arguments in ["not json{", "[]", "null"] {
                let events = parse_body(tool_body(Some(arguments), block_stop)).await;
                assert_eq!(terminal_error(&events), "invalid tool arguments JSON");
                assert!(!events.iter().any(|event| matches!(
                    event,
                    AgentEvent::ToolCall { .. } | AgentEvent::TurnEnd { .. }
                )));
            }
        }
    }

    fn tool_body(arguments: Option<&str>, block_stop: bool) -> String {
        use serde_json::json;
        let event = |kind: &str, data: Value| format!("event: {kind}\ndata: {data}\n\n");
        let mut body = event(
            "content_block_start",
            json!({"index":0,"content_block":{
                "type":"tool_use", "id":"call_test", "name":"fixture", "input":{}
            }}),
        );
        if let Some(arguments) = arguments {
            body.push_str(&event(
                "content_block_delta",
                json!({"index":0,"delta":{
                    "type":"input_json_delta", "partial_json":arguments
                }}),
            ));
        }
        if block_stop {
            body.push_str(&event("content_block_stop", json!({"index":0})));
        }
        body.push_str(&terminal_event("tool_use"));
        body
    }

    #[tokio::test]
    async fn native_tool_arguments_preserve_valid_objects_and_empty_input() {
        for block_stop in [false, true] {
            for arguments in [None, Some("{}"), Some(r#"{"path":"fixture"}"#)] {
                let events = parse_body(tool_body(arguments, block_stop)).await;
                let calls: Vec<_> = events
                    .iter()
                    .filter_map(|event| match event {
                        AgentEvent::ToolCall { call, .. } => Some(call),
                        _ => None,
                    })
                    .collect();
                assert_eq!(calls.len(), 1);
                assert_eq!(
                    calls[0].input,
                    serde_json::from_str::<Value>(arguments.unwrap_or("{}")).expect("fixture JSON")
                );
                assert!(
                    !events
                        .iter()
                        .any(|event| matches!(event, AgentEvent::Error { .. }))
                );
                assert!(
                    events
                        .iter()
                        .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
                );
            }
        }
    }

    fn text_event(text: &str) -> String {
        format!(
            "event: content_block_delta\ndata: {{\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":\"{text}\"}}}}\n\n"
        )
    }

    fn terminal_event(reason: &str) -> String {
        format!(
            "event: message_delta\ndata: {{\"delta\":{{\"stop_reason\":\"{reason}\",\"stop_sequence\":null}},\"usage\":{{\"output_tokens\":1}}}}\n\n"
        )
    }

    #[tokio::test]
    async fn anthropic_eof_after_partial_text_is_terminal_error() {
        let events = parse_body(text_event("partial")).await;
        assert!(
            events.iter().any(
                |event| matches!(event, AgentEvent::TextDelta { delta } if delta == "partial")
            )
        );
        assert!(events.iter().any(|event| matches!(event, AgentEvent::Error { message } if message.contains("explicit terminal"))));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_stop_sequence_is_explicit_completion() {
        let events = parse_body(format!(
            "{}{}",
            text_event("complete"),
            terminal_event("stop_sequence")
        ))
        .await;
        assert!(events.iter().any(|event| matches!(
            event,
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                ..
            }
        )));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::Error { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_pause_turn_uses_known_paused_policy() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("pause_turn")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("paused"));
        assert!(message.contains("pause_turn"));
        assert!(!message.contains("unsupported provider stop_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_refusal_uses_known_refusal_policy() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("refusal")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("refused"));
        assert!(message.contains("refusal"));
        assert!(!message.contains("unsupported provider stop_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_unknown_stop_reason_is_not_end_turn() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("fixture_unknown_reason")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("unsupported provider stop_reason"));
        assert!(message.contains("fixture_unknown_reason"));
        assert!(!message.contains("pause_turn"));
        assert!(!message.contains("refusal"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn explicit_provider_completion_regression_is_unchanged() {
        let events = parse_body(format!(
            "{}{}",
            text_event("complete"),
            terminal_event("end_turn")
        ))
        .await;
        assert!(events.iter().any(|event| matches!(
            event,
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                ..
            }
        )));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::Error { .. }))
        );
    }
}
