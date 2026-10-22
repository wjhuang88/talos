//! OpenAI Responses request assembly.
//!
//! This module is intentionally separate from the Chat Completions request builder. Responses is a
//! distinct wire family: it uses `input` items, flat function tools, `function_call` /
//! `function_call_output` items, `max_output_tokens`, and stateless `store: false` requests.

use std::collections::HashSet;

use base64::Engine;
use serde_json::{Value, json};
use talos_config::{ReasoningEffort, ReasoningOptions};
use talos_core::message::{ContentPart, Message, ReasoningBlock};
use talos_core::provider::ToolDefinition;

use crate::openai_responses_reasoning::decode_reasoning_item;

pub(crate) const EMPTY_INPUT_MESSAGE: &str = "(silence)";
pub(crate) const EMPTY_ASSISTANT_MESSAGE: &str = "(no response)";
pub(crate) const EMPTY_TOOL_RESULT_MESSAGE: &str = "(done)";

/// Build one stateless OpenAI Responses request body.
///
/// The caller owns HTTP/auth/timeout concerns. This function only projects Talos' durable message
/// model onto the Responses wire contract. It deliberately never emits `previous_response_id` or a
/// server-side conversation id.
pub(crate) fn build_request_body(
    model: &str,
    messages: &[Message],
    tools: &[ToolDefinition],
    reasoning: Option<&ReasoningOptions>,
    output_limit: Option<u32>,
) -> Value {
    let mut input = Vec::new();
    let mut pending_tool_call_ids = HashSet::new();

    for (idx, message) in messages.iter().enumerate() {
        match message {
            Message::System { content, .. } => {
                pending_tool_call_ids.clear();
                input.push(text_message_item(
                    "system",
                    non_empty_content(content, EMPTY_INPUT_MESSAGE),
                ));
            }
            Message::Context { content } | Message::User { content } => {
                pending_tool_call_ids.clear();
                input.push(text_message_item(
                    "user",
                    non_empty_content(content, EMPTY_INPUT_MESSAGE),
                ));
            }
            Message::Multimodal { parts } => {
                pending_tool_call_ids.clear();
                let mut content = parts.iter().map(response_input_part).collect::<Vec<_>>();
                if content.is_empty() {
                    content.push(json!({"type": "input_text", "text": EMPTY_INPUT_MESSAGE}));
                }
                input.push(json!({
                    "type": "message",
                    "role": "user",
                    "content": content,
                }));
            }
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                let matched_tool_call_ids = matched_following_tool_result_ids(messages, idx);
                pending_tool_call_ids.clear();

                let mut replayed_reasoning = false;
                if let Some(assistant_reasoning) = reasoning
                    && assistant_reasoning.model == model
                {
                    for block in &assistant_reasoning.blocks {
                        if let ReasoningBlock::Redacted { data } = block
                            && let Some(item) = decode_reasoning_item(data)
                        {
                            input.push(item);
                            replayed_reasoning = true;
                        }
                    }
                }

                if !content.trim().is_empty() {
                    input.push(text_message_item("assistant", content.clone()));
                }

                for call in tool_calls
                    .iter()
                    .filter(|call| matched_tool_call_ids.contains(&call.id))
                {
                    pending_tool_call_ids.insert(call.id.clone());
                    input.push(json!({
                        "type": "function_call",
                        "call_id": call.id,
                        "name": call.name,
                        "arguments": call.input.to_string(),
                    }));
                }

                if content.trim().is_empty()
                    && pending_tool_call_ids.is_empty()
                    && !replayed_reasoning
                {
                    input.push(text_message_item(
                        "assistant",
                        EMPTY_ASSISTANT_MESSAGE.to_string(),
                    ));
                }
            }
            Message::Tool { result } => {
                if !pending_tool_call_ids.remove(&result.tool_use_id) {
                    tracing::warn!(
                        tool_use_id = %result.tool_use_id,
                        "openai-responses: dropping orphan tool result without preceding function_call"
                    );
                    continue;
                }
                let output = if result.is_error {
                    format!("Error: {}", result.content)
                } else {
                    non_empty_content(&result.content, EMPTY_TOOL_RESULT_MESSAGE)
                };
                input.push(json!({
                    "type": "function_call_output",
                    "call_id": result.tool_use_id,
                    "output": output,
                }));
            }
        }
    }

    let mut body = json!({
        "model": model,
        "input": input,
        "stream": true,
        "store": false,
    });

    if !tools.is_empty() {
        body["tools"] = Value::Array(
            tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                        "strict": false,
                    })
                })
                .collect(),
        );
    }

    if let Some(reasoning) = reasoning {
        let effort = match reasoning.effort.clone().unwrap_or(ReasoningEffort::Medium) {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        };
        body["reasoning"] = json!({
            "effort": effort,
            "summary": "auto",
        });
    }

    if let Some(output_limit) = output_limit {
        body["max_output_tokens"] = json!(output_limit);
    }

    body
}

fn response_input_part(part: &ContentPart) -> Value {
    match part {
        ContentPart::Text { text } => json!({
            "type": "input_text",
            "text": text,
        }),
        ContentPart::Image {
            path,
            mime,
            content_digest,
            ..
        } => {
            let Some(bytes) =
                crate::image_io::read_image_with_toctou_guard(path, content_digest).into_bytes()
            else {
                return json!({
                    "type": "input_text",
                    "text": "[image omitted: path validation failed]",
                });
            };
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            json!({
                "type": "input_image",
                "image_url": format!("data:{mime};base64,{encoded}"),
                "detail": "auto",
            })
        }
    }
}

fn text_message_item(role: &str, text: String) -> Value {
    json!({
        "type": "message",
        "role": role,
        "content": [{
            "type": "input_text",
            "text": text,
        }],
    })
}

fn matched_following_tool_result_ids(
    messages: &[Message],
    assistant_idx: usize,
) -> HashSet<String> {
    let mut matched = HashSet::new();
    for message in messages.iter().skip(assistant_idx + 1) {
        match message {
            Message::Tool { result } => {
                matched.insert(result.tool_use_id.clone());
            }
            _ => break,
        }
    }
    matched
}

fn non_empty_content(content: &str, fallback: &str) -> String {
    if content.trim().is_empty() {
        fallback.to_string()
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai_responses_reasoning::encode_completed_reasoning_item;
    use talos_core::message::{AssistantReasoning, ContentDigest, MessageToolResult, ToolCall};

    fn tool_definition() -> ToolDefinition {
        ToolDefinition::new(
            "lookup",
            "Look up one value",
            json!({
                "type": "object",
                "properties": {"key": {"type": "string"}},
                "required": ["key"]
            }),
        )
    }

    #[test]
    fn request_is_streaming_and_stateless() {
        let body = build_request_body(
            "gpt-test",
            &[Message::User {
                content: "hello".into(),
            }],
            &[],
            None,
            Some(2048),
        );

        assert_eq!(body["model"], "gpt-test");
        assert_eq!(body["stream"], true);
        assert_eq!(body["store"], false);
        assert_eq!(body["max_output_tokens"], 2048);
        assert!(body.get("previous_response_id").is_none());
        assert!(body.get("conversation").is_none());
    }

    #[test]
    fn text_roles_map_to_responses_message_items() {
        let body = build_request_body(
            "gpt-test",
            &[
                Message::System {
                    content: "system".into(),
                    cache_markers: vec![],
                },
                Message::Context {
                    content: "context".into(),
                },
                Message::User {
                    content: "user".into(),
                },
                Message::Assistant {
                    content: "assistant".into(),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ],
            &[],
            None,
            None,
        );

        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 4);
        assert_eq!(input[0]["role"], "system");
        assert_eq!(input[1]["role"], "user");
        assert_eq!(input[2]["role"], "user");
        assert_eq!(input[3]["role"], "assistant");
        assert_eq!(input[0]["content"][0]["type"], "input_text");
    }

    #[test]
    fn function_tools_are_flat_and_call_results_reuse_call_id() {
        let call = ToolCall {
            id: "call-1".into(),
            name: "lookup".into(),
            input: json!({"key": "value"}),
        };
        let body = build_request_body(
            "gpt-test",
            &[
                Message::Assistant {
                    content: String::new(),
                    tool_calls: vec![call],
                    reasoning: None,
                },
                Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: "call-1".into(),
                        content: "found".into(),
                        is_error: false,
                    },
                },
            ],
            &[tool_definition()],
            None,
            None,
        );

        let tools = body["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["name"], "lookup");
        assert_eq!(tools[0]["strict"], false);
        assert!(tools[0].get("function").is_none());

        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 2);
        assert_eq!(input[0]["type"], "function_call");
        assert_eq!(input[0]["call_id"], "call-1");
        assert_eq!(input[0]["name"], "lookup");
        assert_eq!(input[1]["type"], "function_call_output");
        assert_eq!(input[1]["call_id"], "call-1");
        assert_eq!(input[1]["output"], "found");
    }

    #[test]
    fn encrypted_reasoning_replays_as_full_item_before_tool_call() {
        let envelope = encode_completed_reasoning_item(&json!({
            "type": "reasoning",
            "id": "rs_1",
            "summary": [{"type": "summary_text", "text": "safe summary"}],
            "encrypted_content": "ciphertext"
        }))
        .expect("valid reasoning item")
        .expect("encrypted envelope");
        let body = build_request_body(
            "gpt-test",
            &[
                Message::Assistant {
                    content: String::new(),
                    tool_calls: vec![ToolCall {
                        id: "call-1".into(),
                        name: "lookup".into(),
                        input: json!({"key": "value"}),
                    }],
                    reasoning: Some(AssistantReasoning {
                        provider: "openai".into(),
                        model: "gpt-test".into(),
                        blocks: vec![ReasoningBlock::Redacted { data: envelope }],
                    }),
                },
                Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: "call-1".into(),
                        content: "found".into(),
                        is_error: false,
                    },
                },
            ],
            &[],
            None,
            None,
        );

        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["type"], "reasoning");
        assert_eq!(input[0]["id"], "rs_1");
        assert_eq!(input[0]["summary"][0]["text"], "safe summary");
        assert_eq!(input[0]["encrypted_content"], "ciphertext");
        assert_eq!(input[1]["type"], "function_call");
        assert_eq!(input[2]["type"], "function_call_output");
    }

    #[test]
    fn reasoning_replay_requires_same_model_and_owned_envelope() {
        let body = build_request_body(
            "gpt-test",
            &[Message::Assistant {
                content: String::new(),
                tool_calls: vec![],
                reasoning: Some(AssistantReasoning {
                    provider: "openai".into(),
                    model: "different-model".into(),
                    blocks: vec![ReasoningBlock::Redacted {
                        data: "foreign-ciphertext".into(),
                    }],
                }),
            }],
            &[],
            None,
            None,
        );
        let input = body["input"].as_array().expect("input array");
        assert_eq!(input.len(), 1);
        assert_eq!(input[0]["role"], "assistant");
        assert_eq!(input[0]["content"][0]["text"], EMPTY_ASSISTANT_MESSAGE);
    }

    #[test]
    fn unmatched_calls_and_orphan_results_are_not_replayed() {
        let body = build_request_body(
            "gpt-test",
            &[
                Message::Assistant {
                    content: "done".into(),
                    tool_calls: vec![ToolCall {
                        id: "never-returned".into(),
                        name: "lookup".into(),
                        input: json!({}),
                    }],
                    reasoning: None,
                },
                Message::User {
                    content: "next".into(),
                },
                Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: "orphan".into(),
                        content: "ignored".into(),
                        is_error: false,
                    },
                },
            ],
            &[],
            None,
            None,
        );

        let input = body["input"].as_array().expect("input array");
        assert!(input.iter().all(|item| item["type"] != "function_call"));
        assert!(
            input
                .iter()
                .all(|item| item["type"] != "function_call_output")
        );
    }

    #[test]
    fn reasoning_and_output_limit_use_responses_fields() {
        let reasoning = ReasoningOptions {
            effort: Some(ReasoningEffort::High),
            ..Default::default()
        };
        let body = build_request_body(
            "gpt-test",
            &[Message::User {
                content: "think".into(),
            }],
            &[],
            Some(&reasoning),
            Some(8192),
        );

        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["reasoning"]["summary"], "auto");
        assert_eq!(body["max_output_tokens"], 8192);
        assert!(body.get("reasoning_effort").is_none());
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn multimodal_input_preserves_text_image_text_order() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("image.png");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\nfixture").expect("write fixture");
        let canonical = path.canonicalize().expect("canonical path");

        let body = build_request_body(
            "gpt-test",
            &[Message::Multimodal {
                parts: vec![
                    ContentPart::Text {
                        text: "before".into(),
                    },
                    ContentPart::Image {
                        path: canonical,
                        mime: "image/png".into(),
                        byte_count: 15,
                        content_digest: ContentDigest::default(),
                    },
                    ContentPart::Text {
                        text: "after".into(),
                    },
                ],
            }],
            &[],
            None,
            None,
        );

        let content = body["input"][0]["content"]
            .as_array()
            .expect("content array");
        assert_eq!(content.len(), 3);
        assert_eq!(content[0]["type"], "input_text");
        assert_eq!(content[0]["text"], "before");
        assert_eq!(content[1]["type"], "input_image");
        assert_eq!(content[1]["detail"], "auto");
        assert!(
            content[1]["image_url"]
                .as_str()
                .expect("data url")
                .starts_with("data:image/png;base64,")
        );
        assert_eq!(content[2]["type"], "input_text");
        assert_eq!(content[2]["text"], "after");
    }
}
