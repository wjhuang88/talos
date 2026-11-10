//! Native stateless generateContent request assembly.
//! Schema: https://ai.google.dev/api/generate-content (checked 2026-11-01).

use base64::Engine;
use serde_json::{Value, json};
use std::collections::HashMap;
use talos_core::message::{ContentPart, Message};
use talos_core::provider::{ProviderError, ProviderResult, ToolDefinition};

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

/// Assemble unsigned history. Opaque positional signature replay is a separate pending gate.
pub(crate) fn build_request(
    messages: &[Message],
    tools: &[ToolDefinition],
    output_limit: Option<u32>,
) -> ProviderResult<Value> {
    build_request_for_identity("", "", messages, tools, output_limit)
}

/// Replay only envelopes stamped with the currently selected transport identity.
pub(crate) fn build_request_for_identity(
    provider: &str,
    model: &str,
    messages: &[Message],
    tools: &[ToolDefinition],
    output_limit: Option<u32>,
) -> ProviderResult<Value> {
    let mut system = Vec::new();
    let mut contents = Vec::new();
    let mut pending = HashMap::new();
    let mut tool_order = Vec::new();
    let mut tool_results = HashMap::new();
    for message in messages {
        let (role, parts) = match message {
            Message::System { content, .. } => {
                system.push(json!({"text":content}));
                continue;
            }
            Message::User { content } | Message::Context { content } => {
                if !pending.is_empty() {
                    return Err(invalid("Gemini history has unresolved tool calls"));
                }
                ("user", vec![json!({"text":content})])
            }
            Message::Multimodal { parts } => {
                if !pending.is_empty() {
                    return Err(invalid("Gemini history has unresolved tool calls"));
                }
                let mut wire = Vec::new();
                for part in parts {
                    wire.push(match part {
                        ContentPart::Text { text } => json!({"text":text}),
                        ContentPart::Image { path, mime, content_digest, .. } => {
                            match crate::image_io::read_image_with_toctou_guard(path, content_digest).into_bytes() {
                                Some(bytes) => json!({"inlineData":{"mimeType":mime,
                                    "data":base64::engine::general_purpose::STANDARD.encode(bytes)}}),
                                None => json!({"text":"[image omitted: path validation failed]"}),
                            }
                        }
                    });
                }
                ("user", wire)
            }
            Message::Assistant {
                content,
                tool_calls,
                reasoning,
            } => {
                if !pending.is_empty() {
                    return Err(invalid("Gemini history has unresolved tool calls"));
                }
                if let Some(reasoning) = reasoning {
                    if reasoning.provider != provider
                        || reasoning.model != model
                        || reasoning.blocks.len() != 1
                    {
                        return Err(invalid("Gemini replay provider or model mismatch"));
                    }
                    let replay =
                        crate::gemini_replay::decode(provider, model, &reasoning.blocks[0])
                            .map_err(invalid)?;
                    if replay.bindings.len() != tool_calls.len() {
                        return Err(invalid("Gemini replay tool history mismatch"));
                    }
                    let mut ordered_bindings = replay.bindings.iter().collect::<Vec<_>>();
                    ordered_bindings.sort_by_key(|(_, index)| *index);
                    for (id, index) in ordered_bindings {
                        tool_order.push(id.clone());
                        let call = tool_calls
                            .iter()
                            .find(|call| &call.id == id)
                            .ok_or_else(|| invalid("Gemini replay tool history mismatch"))?;
                        let native = &replay.parts[*index]["functionCall"];
                        let args = native.get("args").cloned().unwrap_or_else(|| json!({}));
                        if native["name"].as_str() != Some(call.name.as_str()) || args != call.input
                        {
                            return Err(invalid("Gemini replay tool history mismatch"));
                        }
                        let wire_id = native.get("id").and_then(Value::as_str).map(str::to_owned);
                        pending.insert(id.clone(), (call.name.clone(), wire_id));
                    }
                    let text: String = replay
                        .parts
                        .iter()
                        .filter(|part| part.get("thought").and_then(Value::as_bool) != Some(true))
                        .filter_map(|part| part.get("text").and_then(Value::as_str))
                        .collect();
                    if text != *content {
                        return Err(invalid("Gemini replay text history mismatch"));
                    }
                    contents.push(json!({"role":"model","parts":replay.parts}));
                    continue;
                }
                let mut parts = Vec::new();
                if !content.is_empty() {
                    parts.push(json!({"text":content}));
                }
                for call in tool_calls {
                    tool_order.push(call.id.clone());
                    if call.id.is_empty()
                        || call.name.is_empty()
                        || !call.input.is_object()
                        || pending
                            .insert(call.id.clone(), (call.name.clone(), Some(call.id.clone())))
                            .is_some()
                    {
                        return Err(invalid("invalid or duplicate Gemini tool call"));
                    }
                    parts.push(
                        json!({"functionCall":{"id":call.id,"name":call.name,"args":call.input}}),
                    );
                }
                ("model", parts)
            }
            Message::Tool { result } => {
                let (name, wire_id) = pending
                    .remove(&result.tool_use_id)
                    .ok_or_else(|| invalid("Gemini tool result has no matching call"))?;
                let response = if result.is_error {
                    json!({"error":result.content})
                } else {
                    json!({"output":result.content})
                };
                let mut function = json!({"name":name,"response":response});
                if let Some(id) = wire_id {
                    function["id"] = json!(id);
                }
                tool_results.insert(
                    result.tool_use_id.clone(),
                    json!({"functionResponse":function}),
                );
                if !pending.is_empty() {
                    continue;
                }
                let parts = std::mem::take(&mut tool_order)
                    .into_iter()
                    .map(|id| {
                        tool_results
                            .remove(&id)
                            .ok_or_else(|| invalid("Gemini tool batch incomplete"))
                    })
                    .collect::<ProviderResult<Vec<_>>>()?;
                ("user", parts)
            }
        };
        if !parts.is_empty() {
            contents.push(json!({"role":role,"parts":parts}));
        }
    }
    if !pending.is_empty() {
        return Err(invalid("Gemini history has unresolved tool calls"));
    }
    let mut request = json!({"contents":contents});
    if !system.is_empty() {
        request["systemInstruction"] = json!({"parts":system});
    }
    if let Some(limit) = output_limit {
        if limit == 0 {
            return Err(invalid("Gemini output limit must be positive"));
        }
        request["generationConfig"] = json!({"maxOutputTokens":limit});
    }
    if !tools.is_empty() {
        request["tools"] = json!([{"functionDeclarations":tools.iter().map(|tool|
            json!({"name":tool.name,"description":tool.description,"parametersJsonSchema":tool.parameters})
        ).collect::<Vec<_>>()}]);
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{ContentDigest, MessageToolResult, ToolCall};

    #[test]
    fn idless_parallel_results_are_reordered_to_original_signed_call_positions() {
        use talos_core::message::AssistantReasoning;
        let parts = vec![
            json!({"functionCall":{"name":"lookup","args":{"q":1}},"thoughtSignature":"one"}),
            json!({"functionCall":{"name":"lookup","args":{"q":2}},"thoughtSignature":"two"}),
        ];
        let envelope = crate::gemini_replay::encode(
            "google",
            "model",
            &parts,
            &[("a".into(), 0), ("b".into(), 1)],
        )
        .expect("envelope");
        let calls = Message::Assistant {
            content: String::new(),
            tool_calls: vec![
                ToolCall {
                    id: "a".into(),
                    name: "lookup".into(),
                    input: json!({"q":1}),
                },
                ToolCall {
                    id: "b".into(),
                    name: "lookup".into(),
                    input: json!({"q":2}),
                },
            ],
            reasoning: Some(AssistantReasoning {
                provider: "google".into(),
                model: "model".into(),
                blocks: vec![envelope],
            }),
        };
        let result = |id: &str| Message::Tool {
            result: MessageToolResult {
                tool_use_id: id.into(),
                content: id.into(),
                is_error: false,
            },
        };
        let body = build_request_for_identity(
            "google",
            "model",
            &[calls, result("b"), result("a")],
            &[],
            None,
        )
        .expect("ordered batch");
        assert_eq!(body["contents"][0]["parts"], json!(parts));
        let results = &body["contents"][1]["parts"];
        assert_eq!(results[0]["functionResponse"]["response"]["output"], "a");
        assert_eq!(results[1]["functionResponse"]["response"]["output"], "b");
        assert!(results[0]["functionResponse"].get("id").is_none());
        assert!(results[1]["functionResponse"].get("id").is_none());
    }

    #[test]
    fn signed_parts_survive_storage_and_replay_without_generated_wire_ids() {
        use talos_core::message::AssistantReasoning;
        let parts = vec![
            json!({"text":"summary","thought":true,"thoughtSignature":"fixture-signature-1"}),
            json!({"functionCall":{"name":"lookup","args":{"q":1}},"thoughtSignature":"fixture-signature-2"}),
        ];
        let envelope =
            crate::gemini_replay::encode("google", "model", &parts, &[("internal".into(), 1)])
                .expect("envelope");
        let assistant = Message::Assistant {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: "internal".into(),
                name: "lookup".into(),
                input: json!({"q":1}),
            }],
            reasoning: Some(AssistantReasoning {
                provider: "google".into(),
                model: "model".into(),
                blocks: vec![envelope],
            }),
        };
        let stored = serde_json::to_string(&assistant).expect("store");
        let assistant: Message = serde_json::from_str(&stored).expect("restore");
        let history = vec![
            assistant,
            Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "internal".into(),
                    content: "result".into(),
                    is_error: false,
                },
            },
        ];
        let body =
            build_request_for_identity("google", "model", &history, &[], None).expect("replay");
        assert_eq!(body["contents"][0]["parts"], json!(parts));
        assert!(
            body["contents"][1]["parts"][0]["functionResponse"]
                .get("id")
                .is_none()
        );
        assert!(!format!("{:?}", history).contains("fixture-signature"));
        assert!(build_request_for_identity("foreign", "model", &history, &[], None).is_err());
        assert!(build_request_for_identity("google", "different", &history, &[], None).is_err());
        let mut changed = history;
        if let Message::Assistant { tool_calls, .. } = &mut changed[0] {
            tool_calls[0].input = json!({"q":2});
        }
        assert!(build_request_for_identity("google", "model", &changed, &[], None).is_err());
    }

    #[test]
    fn ordered_two_image_request_preserves_content_and_system_and_limit() {
        let dir = tempfile::tempdir().expect("directory");
        let first = dir.path().join("first.png");
        let second = dir.path().join("second.png");
        std::fs::write(&first, b"first-image").expect("first");
        std::fs::write(&second, b"second-image").expect("second");
        let image = |path| ContentPart::Image {
            path,
            mime: "image/png".into(),
            byte_count: 11,
            content_digest: ContentDigest::default(),
        };
        let body = build_request(
            &[
                Message::System {
                    content: "system".into(),
                    cache_markers: vec![],
                },
                Message::Multimodal {
                    parts: vec![
                        ContentPart::Text {
                            text: "before".into(),
                        },
                        image(first.canonicalize().expect("path")),
                        ContentPart::Text {
                            text: "between".into(),
                        },
                        image(second.canonicalize().expect("path")),
                    ],
                },
            ],
            &[],
            Some(1024),
        )
        .expect("request");
        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "system");
        assert_eq!(body["generationConfig"]["maxOutputTokens"], 1024);
        let parts = &body["contents"][0]["parts"];
        assert_eq!(parts[0]["text"], "before");
        assert_eq!(parts[2]["text"], "between");
        assert_eq!(
            parts[1]["inlineData"]["data"],
            base64::engine::general_purpose::STANDARD.encode(b"first-image")
        );
        assert_eq!(
            parts[3]["inlineData"]["data"],
            base64::engine::general_purpose::STANDARD.encode(b"second-image")
        );
    }

    #[test]
    fn same_name_tools_correlate_by_id_and_reject_orphan_duplicate_results() {
        let calls = Message::Assistant {
            content: String::new(),
            reasoning: None,
            tool_calls: vec![
                ToolCall {
                    id: "a".into(),
                    name: "lookup".into(),
                    input: json!({"q":1}),
                },
                ToolCall {
                    id: "b".into(),
                    name: "lookup".into(),
                    input: json!({"q":2}),
                },
            ],
        };
        let result = |id: &str| Message::Tool {
            result: MessageToolResult {
                tool_use_id: id.into(),
                content: id.into(),
                is_error: false,
            },
        };
        let tool = ToolDefinition {
            name: "lookup".into(),
            description: "lookup".into(),
            parameters: json!({"type":"object","$defs":{}}),
        };
        let body = build_request(&[calls.clone(), result("b"), result("a")], &[tool], None)
            .expect("round trip");
        assert_eq!(
            body["contents"][1]["parts"][0]["functionResponse"]["id"],
            "a"
        );
        assert_eq!(
            body["contents"][1]["parts"][1]["functionResponse"]["id"],
            "b"
        );
        assert_eq!(
            body["tools"][0]["functionDeclarations"][0]["parametersJsonSchema"]["$defs"],
            json!({})
        );
        assert!(build_request(&[result("orphan")], &[], None).is_err());
        assert!(build_request(&[calls.clone(), result("a"), result("a")], &[], None).is_err());
        assert!(build_request(&[calls], &[], None).is_err());
    }
}
