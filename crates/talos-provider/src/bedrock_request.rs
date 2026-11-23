//! Encode stateless ConverseStream messages. The model ID belongs in the signed URL.
use base64::Engine;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use talos_core::message::{ContentPart, Message};
use talos_core::provider::{ProviderError, ProviderResult, ToolDefinition};

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

#[derive(Serialize)]
struct Turn {
    role: &'static str,
    content: Vec<Value>,
}

fn encode_part(part: &ContentPart) -> ProviderResult<Value> {
    match part {
        ContentPart::Text { text } => Ok(json!({"text":text})),
        ContentPart::Image {
            path,
            mime,
            content_digest,
            ..
        } => {
            let format = match mime.as_str() {
                "image/png" => "png",
                "image/jpeg" => "jpeg",
                "image/gif" => "gif",
                "image/webp" => "webp",
                _ => return Err(invalid("unsupported Bedrock image format")),
            };
            let Some(bytes) =
                crate::image_io::read_image_with_toctou_guard(path, content_digest).into_bytes()
            else {
                return Ok(json!({"text":"[image omitted: path validation failed]"}));
            };
            Ok(
                json!({"image":{"format":format,"source":{"bytes":base64::engine::general_purpose::STANDARD.encode(bytes)}}}),
            )
        }
    }
}

pub(crate) fn validate_tool_history(messages: &[Message]) -> ProviderResult<()> {
    let mut pending = HashSet::new();
    for message in messages {
        if !pending.is_empty() && !matches!(message, Message::Tool { .. }) {
            return Err(invalid("Bedrock history has unresolved tool calls"));
        }
        match message {
            Message::Assistant { tool_calls, .. } => {
                for call in tool_calls {
                    if call.name.is_empty()
                        || call.id.is_empty()
                        || !call.input.is_object()
                        || !pending.insert(call.id.as_str())
                    {
                        return Err(invalid("invalid or duplicate Bedrock tool use"));
                    }
                }
            }
            Message::Tool { result } if !pending.remove(result.tool_use_id.as_str()) => {
                return Err(invalid("Bedrock tool result has no matching call"));
            }
            _ => {}
        }
    }
    if !pending.is_empty() {
        return Err(invalid("Bedrock history has unresolved tool calls"));
    }
    Ok(())
}

pub(crate) fn build(
    messages: &[Message],
    tools: &[ToolDefinition],
    max_tokens: Option<u32>,
) -> ProviderResult<Value> {
    build_bound(messages, tools, max_tokens, None)
}

pub(crate) fn fingerprint(system: &Value, messages: &Value, route: &str) -> String {
    use sha2::{Digest, Sha256};
    let encoded = json!([route, system, messages]).to_string();
    base64::engine::general_purpose::STANDARD.encode(Sha256::digest(encoded.as_bytes()))
}

pub(crate) fn build_bound(
    messages: &[Message],
    tools: &[ToolDefinition],
    max_tokens: Option<u32>,
    route: Option<&str>,
) -> ProviderResult<Value> {
    validate_tool_history(messages)?;
    let system: Vec<_> = messages
        .iter()
        .filter_map(|message| match message {
            Message::System { content, .. } => Some(json!({"text":content})),
            _ => None,
        })
        .collect();
    let system_value = json!(system);
    let mut turns = Vec::<Turn>::new();
    for message in messages {
        let (role, blocks) = match message {
            Message::System { .. } => {
                continue;
            }
            Message::User { content } | Message::Context { content } => {
                ("user", vec![json!({"text":content})])
            }
            Message::Multimodal { parts } => (
                "user",
                parts
                    .iter()
                    .map(encode_part)
                    .collect::<ProviderResult<_>>()?,
            ),
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                if let Some(reasoning) = reasoning {
                    let mismatch = || invalid("Bedrock signed history binding mismatch");
                    let route = route.ok_or_else(mismatch)?;
                    let [talos_core::message::ReasoningBlock::Redacted { data }] =
                        reasoning.blocks.as_slice()
                    else {
                        return Err(mismatch());
                    };
                    let (expected, blocks): (String, Vec<Value>) =
                        serde_json::from_str(data).map_err(|_| mismatch())?;
                    if expected != fingerprint(&system_value, &json!(turns), route)
                        || turns.last().is_some_and(|turn| turn.role == "assistant")
                    {
                        return Err(mismatch());
                    }
                    turns.push(Turn {
                        role: "assistant",
                        content: blocks,
                    });
                    continue;
                }
                let mut blocks = Vec::new();
                if !content.is_empty() {
                    blocks.push(json!({"text":content}));
                }
                for call in tool_calls {
                    blocks.push(json!({"toolUse":{"toolUseId":call.id,"name":call.name,"input":call.input}}));
                }
                ("assistant", blocks)
            }
            Message::Tool { result } => {
                let status = if result.is_error { "error" } else { "success" };
                (
                    "user",
                    vec![
                        json!({"toolResult":{"toolUseId":result.tool_use_id,"status":status,"content":[{"text":result.content}]}}),
                    ],
                )
            }
        };
        if blocks.is_empty() {
            return Err(invalid("empty Bedrock message"));
        }
        if let Some(last) = turns.last_mut().filter(|last| last.role == role) {
            last.content.extend(blocks);
        } else {
            turns.push(Turn {
                role,
                content: blocks,
            });
        }
    }
    let mut body = json!({"messages":turns});
    if !system.is_empty() {
        body["system"] = json!(system);
    }
    if !tools.is_empty() {
        let specs: Vec<_> = tools.iter().map(|tool| json!({"toolSpec":{"name":tool.name,"description":tool.description,"inputSchema":{"json":tool.parameters}}})).collect();
        body["toolConfig"] = json!({"tools":specs});
    }
    if let Some(limit) = max_tokens {
        body["inferenceConfig"] = json!({"maxTokens":limit});
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{MessageToolResult, ToolCall};

    #[test]
    fn associates_reversed_results_in_one_user_turn() {
        let calls = ["a", "b"].map(|id| ToolCall {
            id: id.into(),
            name: "read".into(),
            input: json!({"path":id}),
        });
        let mut messages = vec![
            Message::User {
                content: "check".into(),
            },
            Message::Assistant {
                content: String::new(),
                tool_calls: calls.to_vec(),
                reasoning: None,
            },
        ];
        for id in ["b", "a"] {
            messages.push(Message::Tool {
                result: MessageToolResult {
                    tool_use_id: id.into(),
                    content: id.into(),
                    is_error: id == "b",
                },
            });
        }
        let body = build(&messages, &[], Some(42)).unwrap();
        let blocks = &body["messages"][2]["content"];
        assert_eq!(blocks[0]["toolResult"]["toolUseId"], "b");
        assert_eq!(blocks[0]["toolResult"]["status"], "error");
        assert_eq!(blocks[1]["toolResult"]["toolUseId"], "a");
        assert_eq!(body["inferenceConfig"]["maxTokens"], 42);
        assert!(build(&messages[..3], &[], None).is_err());
        messages.push(messages[2].clone());
        assert!(build(&messages, &[], None).is_err());
    }
}
