//! Anthropic Messages API request assembly.

use serde_json::{Value, json};
use talos_config::ReasoningOptions;
use talos_core::message::{
    AssistantReasoning, Message, ReasoningBlock, SystemCacheMarker, SystemCacheType,
};
use talos_core::provider::ToolDefinition;

use crate::ANTHROPIC_API_URL;
use crate::ANTHROPIC_VERSION;

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

pub(crate) fn build_request_body(
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
            Message::Multimodal { parts } => {
                let text: String = parts
                    .iter()
                    .filter_map(|p| match p {
                        talos_core::message::ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                json!({
                    "role": "user",
                    "content": text,
                })
            }
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

pub(crate) fn redact_secret(secret: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use talos_core::message::ToolCall;

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

    // ── TOOL-021 error propagation fixtures ──────────────────────────────

    #[test]
    fn fixture_anthropic_error_result_has_is_error_flag() {
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "bash".into(),
                    input: json!({"command": "false"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: "command failed".into(),
                    is_error: true,
                },
            },
        ];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        // F2: error result must have is_error: true
        let tool_block = &body["messages"][1]["content"][0];
        assert_eq!(tool_block["type"], "tool_result");
        assert_eq!(
            tool_block["is_error"], true,
            "Anthropic must set is_error: true"
        );
        assert_eq!(
            tool_block["content"], "command failed",
            "error content must be preserved without modification"
        );
    }

    #[test]
    fn fixture_anthropic_success_result_no_is_error() {
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "read_file".into(),
                    input: json!({"path": "test.rs"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "call_1".into(),
                    content: "fn main() {}".into(),
                    is_error: false,
                },
            },
        ];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        // F1/F3: success result must NOT have is_error flag
        let tool_block = &body["messages"][1]["content"][0];
        assert_eq!(tool_block["type"], "tool_result");
        assert!(
            tool_block.get("is_error").is_none(),
            "success result must not set is_error"
        );
    }

    #[test]
    fn fixture_anthropic_orphan_result_not_filtered() {
        // F4: Anthropic does NOT filter orphan tool results (unlike OpenAI).
        // The result is sent as-is — the API may accept or reject it.
        let messages = vec![Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "orphan_1".into(),
                content: "orphan result".into(),
                is_error: false,
            },
        }];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        // Anthropic serializes orphan tool results without filtering.
        let user_msgs: Vec<_> = body["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["role"] == "user")
            .collect();
        assert!(
            !user_msgs.is_empty(),
            "Anthropic must NOT filter orphan tool results (provider difference from OpenAI)"
        );
    }

    #[test]
    fn fixture_anthropic_orphan_error_result_not_filtered() {
        // F4-error: Anthropic does NOT filter orphan ERROR tool results either.
        let messages = vec![Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "orphan_err".into(),
                content: "command not found".into(),
                is_error: true,
            },
        }];
        let body = build_request_body("claude-sonnet-4-20250514", &messages, &[], None, None);

        let user_msgs: Vec<_> = body["messages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["role"] == "user")
            .collect();
        assert!(
            !user_msgs.is_empty(),
            "Anthropic must NOT filter orphan ERROR tool results"
        );
        // Verify is_error is still set on the orphan
        let tool_block = &user_msgs[0]["content"][0];
        assert_eq!(
            tool_block["is_error"], true,
            "orphan error result must preserve is_error flag"
        );
    }
}
