//! OpenAI Chat Completions request assembly.

use std::collections::HashSet;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use talos_config::{ReasoningEffort, ReasoningOptions};
use talos_core::message::Message;
use talos_core::message::ReasoningBlock;
use talos_core::provider::ToolDefinition;

pub(crate) const EMPTY_USER_MESSAGE: &str = "(silence)";
pub(crate) const EMPTY_ASSISTANT_MESSAGE: &str = "(no response)";
pub(crate) const EMPTY_ASSISTANT_TOOL_CALL_MESSAGE: &str = "Calling tools…";
pub(crate) const EMPTY_TOOL_RESULT_MESSAGE: &str = "(done)";

/// OpenAI chat message for request serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpenAIMessage {
    pub(crate) role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reasoning_content: Option<String>,
}

/// OpenAI tool call representation in the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpenAIToolCall {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) call_type: String,
    pub(crate) function: OpenAIFunction,
}

/// OpenAI function definition within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpenAIFunction {
    pub(crate) name: String,
    pub(crate) arguments: String,
}

pub(crate) fn build_request_body(
    model: &str,
    messages: &[Message],
    tools: &[ToolDefinition],
    reasoning: Option<&ReasoningOptions>,
    output_limit: Option<u32>,
) -> Value {
    let mut pending_tool_call_ids = HashSet::new();
    let openai_messages: Vec<OpenAIMessage> = messages
        .iter()
        .enumerate()
        .filter_map(|(idx, msg)| match msg {
            Message::System { content, .. } => {
                pending_tool_call_ids.clear();
                OpenAIMessage {
                    role: "system".into(),
                    content: Some(Value::String(non_empty_content(content, EMPTY_USER_MESSAGE))),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                }
                .into()
            }
            Message::Context { content } => {
                pending_tool_call_ids.clear();
                OpenAIMessage {
                    role: "user".into(),
                    content: Some(Value::String(non_empty_content(content, EMPTY_USER_MESSAGE))),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                }
                .into()
            }
            Message::User { content } => {
                pending_tool_call_ids.clear();
                OpenAIMessage {
                    role: "user".into(),
                    content: Some(Value::String(non_empty_content(content, EMPTY_USER_MESSAGE))),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                }
                .into()
            }
            Message::Multimodal { parts } => {
                pending_tool_call_ids.clear();
                let content_parts: Vec<Value> = parts
                    .iter()
                    .map(|p| match p {
                        talos_core::message::ContentPart::Text { text } => {
                            json!({"type": "text", "text": text})
                        }
                        talos_core::message::ContentPart::Image { path, mime, .. } => {
                            let bytes = match crate::image_io::read_image_with_toctou_guard(path)
                                .into_bytes()
                            {
                                Some(b) => b,
                                None => {
                                    return json!({
                                        "type": "text",
                                        "text": "[image omitted: path validation failed]"
                                    });
                                }
                            };
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            json!({"type": "image_url", "image_url": {"url": format!("data:{mime};base64,{b64}")}})
                        }
                    })
                    .collect();
                OpenAIMessage {
                    role: "user".into(),
                    content: Some(Value::Array(content_parts)),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                }
                .into()
            }
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                let matched_tool_call_ids = matched_following_tool_result_ids(messages, idx);
                let mut serialized_tool_call_ids = HashSet::new();
                let openai_tool_calls = if tool_calls.is_empty() {
                    None
                } else {
                    let calls: Vec<OpenAIToolCall> = tool_calls
                        .iter()
                        .filter(|tc| matched_tool_call_ids.contains(&tc.id))
                        .map(|tc| {
                            serialized_tool_call_ids.insert(tc.id.clone());
                            OpenAIToolCall {
                                id: tc.id.clone(),
                                call_type: "function".into(),
                                function: OpenAIFunction {
                                    name: tc.name.clone(),
                                    arguments: tc.input.to_string(),
                                },
                            }
                        })
                        .collect();
                    if calls.is_empty() { None } else { Some(calls) }
                };
                pending_tool_call_ids.clear();
                pending_tool_call_ids.extend(serialized_tool_call_ids);

                let reasoning_content = reasoning.as_ref().and_then(|assistant_reasoning| {
                    if assistant_reasoning.model != model {
                        return None;
                    }

                    let mut combined = String::new();
                    for block in &assistant_reasoning.blocks {
                        if let ReasoningBlock::Plain { text } = block {
                            combined.push_str(text);
                        }
                    }

                    if combined.is_empty() {
                        None
                    } else {
                        Some(combined)
                    }
                });

                OpenAIMessage {
                    role: "assistant".into(),
                    content: if content.trim().is_empty() {
                        Some(Value::String(if openai_tool_calls.is_some() {
                            EMPTY_ASSISTANT_TOOL_CALL_MESSAGE.to_string()
                        } else {
                            EMPTY_ASSISTANT_MESSAGE.to_string()
                        }))
                    } else {
                        Some(Value::String(content.clone()))
                    },
                    tool_calls: openai_tool_calls,
                    tool_call_id: None,
                    reasoning_content,
                }
                .into()
            }
            Message::Tool { result } => {
                if !pending_tool_call_ids.remove(&result.tool_use_id) {
                    tracing::warn!(
                        tool_use_id = %result.tool_use_id,
                        "openai: dropping orphan tool result without preceding assistant tool_call"
                    );
                    return None;
                }
                let content = if result.is_error {
                    format!("Error: {}", result.content)
                } else {
                    non_empty_content(&result.content, EMPTY_TOOL_RESULT_MESSAGE)
                };
                OpenAIMessage {
                    role: "tool".into(),
                    content: Some(Value::String(content)),
                    tool_calls: None,
                    tool_call_id: Some(result.tool_use_id.clone()),
                    reasoning_content: None,
                }
                .into()
            }
        })
        .collect();

    let mut body = json!({
        "model": model,
        "messages": openai_messages,
        "stream": true,
        "stream_options": {
            "include_usage": true,
        },
    });

    if !tools.is_empty() {
        let tools_json: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            })
            .collect();
        body["tools"] = json!(tools_json);
    }

    if let Some(reasoning) = reasoning {
        let effort = match reasoning.effort.clone().unwrap_or(ReasoningEffort::Medium) {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        };
        body["reasoning_effort"] = json!(effort);
        body["max_completion_tokens"] = json!(output_limit.unwrap_or(4096));
    }

    body
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

fn matched_following_tool_result_ids(
    messages: &[Message],
    assistant_idx: usize,
) -> HashSet<String> {
    let mut matched = HashSet::new();
    for msg in messages.iter().skip(assistant_idx + 1) {
        match msg {
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
    use talos_core::message::ContentPart;

    #[test]
    fn multimodal_message_produces_array_content_with_image_url() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.png");
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        std::fs::write(&img_path, &png_header).unwrap();
        // ContentPart::Image.path contract: stored path MUST be the
        // canonical path produced at grant time. The TOCTOU guard in
        // image_io rejects any non-canonical stored path.
        let canonical = img_path.canonicalize().unwrap();

        let messages = vec![Message::Multimodal {
            parts: vec![
                ContentPart::Text {
                    text: "What is this?".into(),
                },
                ContentPart::Image {
                    path: canonical,
                    mime: "image/png".into(),
                    byte_count: 8,
                },
            ],
        }];

        let body = build_request_body("gpt-4o", &messages, &[], None, None);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");

        let content = msgs[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "What is this?");
        assert_eq!(content[1]["type"], "image_url");
        assert!(
            content[1]["image_url"]["url"]
                .as_str()
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
    }

    #[test]
    fn text_only_multimodal_produces_array_with_text_parts() {
        let messages = vec![Message::Multimodal {
            parts: vec![
                ContentPart::Text {
                    text: "Hello".into(),
                },
                ContentPart::Text {
                    text: "World".into(),
                },
            ],
        }];

        let body = build_request_body("gpt-4o", &messages, &[], None, None);
        let msgs = body["messages"].as_array().unwrap();
        let content = msgs[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[1]["type"], "text");
    }
}
