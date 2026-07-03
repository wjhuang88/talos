//! OpenAI Chat Completions request assembly.

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
    pub(crate) content: Option<String>,
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
    let openai_messages: Vec<OpenAIMessage> = messages
        .iter()
        .map(|msg| match msg {
            Message::System { content, .. } => OpenAIMessage {
                role: "system".into(),
                content: Some(non_empty_content(content, EMPTY_USER_MESSAGE)),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            Message::Context { content } => OpenAIMessage {
                role: "user".into(),
                content: Some(non_empty_content(content, EMPTY_USER_MESSAGE)),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            Message::User { content } => OpenAIMessage {
                role: "user".into(),
                content: Some(non_empty_content(content, EMPTY_USER_MESSAGE)),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                let openai_tool_calls = if tool_calls.is_empty() {
                    None
                } else {
                    Some(
                        tool_calls
                            .iter()
                            .map(|tc| OpenAIToolCall {
                                id: tc.id.clone(),
                                call_type: "function".into(),
                                function: OpenAIFunction {
                                    name: tc.name.clone(),
                                    arguments: tc.input.to_string(),
                                },
                            })
                            .collect(),
                    )
                };

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
                        Some(if openai_tool_calls.is_some() {
                            EMPTY_ASSISTANT_TOOL_CALL_MESSAGE.to_string()
                        } else {
                            EMPTY_ASSISTANT_MESSAGE.to_string()
                        })
                    } else {
                        Some(content.clone())
                    },
                    tool_calls: openai_tool_calls,
                    tool_call_id: None,
                    reasoning_content,
                }
            }
            Message::Tool { result } => {
                let content = if result.is_error {
                    format!("Error: {}", result.content)
                } else {
                    non_empty_content(&result.content, EMPTY_TOOL_RESULT_MESSAGE)
                };
                OpenAIMessage {
                    role: "tool".into(),
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: Some(result.tool_use_id.clone()),
                    reasoning_content: None,
                }
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

fn non_empty_content(content: &str, fallback: &str) -> String {
    if content.trim().is_empty() {
        fallback.to_string()
    } else {
        content.to_string()
    }
}
