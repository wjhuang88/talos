//! Textual tools over the same Gemini wire, retaining signed model text unchanged.

use std::collections::HashSet;
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::{ProviderError, ProviderResult};
use talos_core::tool::ToolProtocol;
use tokio::sync::mpsc;

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

// Shared parsing has already validated frames; retain only the distinction between
// a wire-supplied ID and the parser's request-local synthetic ID.
fn explicit_ids(text: &str) -> Vec<Option<String>> {
    let mut remaining = text;
    let mut ids = Vec::new();
    while let Some((start, open, close)) = [
        ("```json-tool", "```"),
        ("<tool_call>", "</tool_call>"),
        ("<toolcall>", "</toolcall>"),
    ]
    .into_iter()
    .filter_map(|(open, close)| remaining.find(open).map(|start| (start, open, close)))
    .min_by_key(|(start, _, _)| *start)
    {
        let inner = &remaining[start + open.len()..];
        let Some(end) = inner.find(close) else { break };
        let id = serde_json::from_str::<serde_json::Value>(inner[..end].trim())
            .ok()
            .and_then(|value| {
                value
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .filter(|id| !id.is_empty())
                    .map(str::to_owned)
            });
        ids.push(id);
        remaining = &inner[end + close.len()..];
    }
    ids
}

pub(crate) fn project(
    provider: &str,
    model: &str,
    messages: &[Message],
    protocol: ToolProtocol,
) -> ProviderResult<Vec<Message>> {
    let mut pending = HashSet::new();
    for message in messages {
        match message {
            Message::Assistant { tool_calls, .. } => {
                if !pending.is_empty() {
                    return Err(invalid("Gemini compatibility history has unresolved calls"));
                }
                for call in tool_calls {
                    if call.id.is_empty() || !pending.insert(call.id.clone()) {
                        return Err(invalid(
                            "Gemini compatibility tool identity is invalid or duplicated",
                        ));
                    }
                }
            }
            Message::Tool { result } => {
                if !pending.remove(&result.tool_use_id) {
                    return Err(invalid("Gemini compatibility result has no matching call"));
                }
            }
            Message::System { .. } => {}
            _ if !pending.is_empty() => {
                return Err(invalid("Gemini compatibility history has unresolved calls"));
            }
            _ => {}
        }
    }
    if !pending.is_empty() {
        return Err(invalid("Gemini compatibility history has unresolved calls"));
    }
    let mut projected = crate::compatibility_messages(messages);
    for (original, projected) in messages.iter().zip(&mut projected) {
        if let Message::Assistant {
            content,
            tool_calls,
            reasoning: Some(reasoning),
        } = original
        {
            if reasoning.provider != provider
                || reasoning.model != model
                || reasoning.blocks.len() != 1
            {
                return Err(invalid("Gemini compatibility replay identity mismatch"));
            }
            let replay = crate::gemini_replay::decode(provider, model, &reasoning.blocks[0])
                .map_err(invalid)?;
            // Native signed calls cannot be rewritten into textual instructions.
            if !replay.bindings.is_empty() {
                return Err(invalid(
                    "Gemini signed native history cannot switch to textual tools",
                ));
            }
            let calls = crate::anthropic_stream::parse_protocol_text_calls(content, protocol)
                .map_err(invalid)?;
            let ids = explicit_ids(content);
            if ids.len() != tool_calls.len()
                || ids
                    .iter()
                    .zip(tool_calls)
                    .any(|(id, call)| id.as_ref().is_some_and(|id| id != &call.id))
                || calls.len() != tool_calls.len()
                || calls
                    .iter()
                    .zip(tool_calls)
                    .any(|(a, b)| a.name != b.name || a.input != b.input)
            {
                return Err(invalid("Gemini compatibility tool history mismatch"));
            }
            *projected = Message::Assistant {
                content: content.clone(),
                tool_calls: vec![],
                reasoning: Some(reasoning.clone()),
            };
        }
    }
    // The native builder checks signed text equality and identity before any credentials/HTTP.
    Ok(projected)
}

pub(crate) fn forward(
    mut source: mpsc::Receiver<AgentEvent>,
    protocol: ToolProtocol,
    deadline: std::time::Instant,
) -> mpsc::Receiver<AgentEvent> {
    let (tx, rx) = mpsc::channel(32);
    tokio::spawn(async move {
        let _ = tokio::time::timeout_at(deadline.into(), async {
        let mut text = String::new();
        loop {
            let event = tokio::select! {
                _ = tx.closed() => return,
                event = source.recv() => match event {
                    Some(event) => event,
                    None => {
                        let _ = tx.send(AgentEvent::Error { message: "Gemini compatibility stream ended without terminal outcome".into() }).await;
                        return;
                    }
                },
            };
            match event {
                AgentEvent::TextDelta { delta } => {
                    if text.len().saturating_add(delta.len()) > 8 * 1024 * 1024 {
                        let _ = tx
                            .send(AgentEvent::Error {
                                message: "Gemini compatibility text exceeds limit".into(),
                            })
                            .await;
                        return;
                    }
                    text.push_str(&delta);
                    if tx.send(AgentEvent::TextDelta { delta }).await.is_err() {
                        return;
                    }
                }
                AgentEvent::ToolCall { .. } | AgentEvent::ToolCallStarted { .. } => {
                    let _ = tx
                        .send(AgentEvent::Error {
                            message: "unexpected native tool in Gemini textual tool mode".into(),
                        })
                        .await;
                    return;
                }
                AgentEvent::TurnEnd {
                    mut stop_reason,
                    usage,
                } => {
                    let calls =
                        match crate::anthropic_stream::parse_protocol_text_calls(&text, protocol) {
                            Ok(calls) => calls,
                            Err(message) => {
                                let _ = tx
                                    .send(AgentEvent::Error {
                                        message: message.into(),
                                    })
                                    .await;
                                return;
                            }
                        };
                    let mut ids = HashSet::new();
                    if calls.iter().any(|call| !ids.insert(&call.id)) {
                        let _ = tx.send(AgentEvent::Error { message: "duplicate Gemini compatibility tool identity".into() }).await;
                        return;
                    }
                    if !calls.is_empty() && stop_reason == StopReason::EndTurn {
                        stop_reason = StopReason::ToolUse;
                    }
                    for call in calls {
                        if tx
                            .send(AgentEvent::ToolCall {
                                call,
                                provenance: Default::default(),
                                summary_fields: vec![],
                            })
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    let _ = tx.send(AgentEvent::TurnEnd { stop_reason, usage }).await;
                    return;
                }
                event => {
                    let failed = matches!(event, AgentEvent::Error { .. });
                    if tx.send(event).await.is_err() || failed {
                        return;
                    }
                }
            }
        }
        }).await;
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use talos_core::message::{AssistantReasoning, ToolCall, Usage};

    #[test]
    fn signed_text_projection_retains_parts_and_rejects_edits_or_native_switch() {
        let text = r#"<tool_call>{"name":"read","id":"r","args":{"path":"note"}}</tool_call>"#;
        let parts = vec![json!({"text":text,"thoughtSignature":"private-signature"})];
        let reasoning = AssistantReasoning {
            provider: "google".into(),
            model: "model".into(),
            blocks: vec![
                crate::gemini_replay::encode("google", "model", &parts, &[]).expect("envelope"),
            ],
        };
        let mut history = vec![Message::Assistant {
            content: text.into(),
            tool_calls: vec![ToolCall {
                id: "r".into(),
                name: "read".into(),
                input: json!({"path":"note"}),
            }],
            reasoning: Some(reasoning),
        }];
        history.push(Message::Tool {
            result: talos_core::message::MessageToolResult {
                tool_use_id: "r".into(),
                content: "ok".into(),
                is_error: false,
            },
        });
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            let projected = project("google", "model", &history, protocol).expect("projection");
            let body = crate::gemini_request::build_request_for_identity(
                "google",
                "model",
                &projected,
                &[],
                None,
            )
            .expect("signed request");
            assert_eq!(body["contents"][0]["parts"], json!(parts));
            assert!(project("other", "model", &history, protocol).is_err());
        }
        if let Message::Assistant { tool_calls, .. } = &mut history[0] {
            tool_calls[0].id = "other".into();
        }
        if let Message::Tool { result } = &mut history[1] {
            result.tool_use_id = "other".into();
        }
        assert!(project("google", "model", &history, ToolProtocol::Compat).is_err());
        if let Message::Assistant { tool_calls, .. } = &mut history[0] {
            tool_calls[0].id = "r".into();
            tool_calls[0].input = json!({"path":"changed"});
        }
        if let Message::Tool { result } = &mut history[1] {
            result.tool_use_id = "r".into();
        }
        assert!(project("google", "model", &history, ToolProtocol::Compat).is_err());
        let native =
            vec![json!({"functionCall":{"name":"read","args":{}},"thoughtSignature":"private"})];
        if let Message::Assistant {
            reasoning: Some(reasoning),
            ..
        } = &mut history[0]
        {
            reasoning.blocks = vec![
                crate::gemini_replay::encode("google", "model", &native, &[("r".into(), 0)])
                    .expect("native"),
            ];
        }
        assert!(project("google", "model", &history, ToolProtocol::Compat).is_err());
    }

    #[tokio::test]
    async fn bridge_deadline_and_receiver_drop_cancel_upstream() {
        for drop_receiver in [false, true] {
            let (tx, source) = mpsc::channel(64);
            for _ in 0..64 {
                tx.send(AgentEvent::TextDelta { delta: "x".into() })
                    .await
                    .expect("queued");
            }
            let rx = forward(
                source,
                ToolProtocol::Compat,
                std::time::Instant::now() + std::time::Duration::from_millis(50),
            );
            if drop_receiver {
                drop(rx);
            } else {
                // Keep the output receiver alive and deliberately do not drain its full queue.
                tokio::time::timeout(std::time::Duration::from_secs(2), tx.closed())
                    .await
                    .expect("deadline closes source");
                drop(rx);
            }
            tokio::time::timeout(std::time::Duration::from_secs(2), tx.closed())
                .await
                .expect("drop closes source");
        }
    }

    #[test]
    fn result_association_rejects_missing_duplicate_or_orphan_and_accepts_idless_text() {
        use talos_core::message::MessageToolResult;
        let text = "<toolcall>read {}</toolcall>";
        let block = crate::gemini_replay::encode(
            "google",
            "model",
            &[json!({"text":text,"thoughtSignature":"private"})],
            &[],
        )
        .expect("envelope");
        let call = ToolCall {
            id: "generated".into(),
            name: "read".into(),
            input: json!({}),
        };
        let assistant = Message::Assistant {
            content: text.into(),
            tool_calls: vec![call],
            reasoning: Some(AssistantReasoning {
                provider: "google".into(),
                model: "model".into(),
                blocks: vec![block],
            }),
        };
        let result = Message::Tool {
            result: MessageToolResult {
                tool_use_id: "generated".into(),
                content: "ok".into(),
                is_error: false,
            },
        };
        assert!(
            project(
                "google",
                "model",
                &[assistant.clone(), result.clone()],
                ToolProtocol::Compat
            )
            .is_ok()
        );
        assert!(
            project(
                "google",
                "model",
                std::slice::from_ref(&assistant),
                ToolProtocol::Compat
            )
            .is_err()
        );
        assert!(
            project(
                "google",
                "model",
                std::slice::from_ref(&result),
                ToolProtocol::Compat
            )
            .is_err()
        );
        assert!(
            project(
                "google",
                "model",
                &[assistant.clone(), result.clone(), result],
                ToolProtocol::Compat
            )
            .is_err()
        );
        if let Message::Assistant {
            mut tool_calls,
            content,
            reasoning,
        } = assistant
        {
            tool_calls.push(tool_calls[0].clone());
            assert!(
                project(
                    "google",
                    "model",
                    &[Message::Assistant {
                        content,
                        tool_calls,
                        reasoning
                    }],
                    ToolProtocol::Compat
                )
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn invalid_strict_batches_and_native_calls_release_no_tools() {
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            for scenario in ["malformed", "native", "duplicate"] {
                let (tx, source) = mpsc::channel(8);
                let mut output = forward(
                    source,
                    protocol,
                    std::time::Instant::now() + std::time::Duration::from_secs(5),
                );
                if scenario == "native" {
                    tx.send(AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "n".into(),
                            name: "read".into(),
                            input: json!({}),
                        },
                        provenance: Default::default(),
                        summary_fields: vec![],
                    })
                    .await
                    .expect("native");
                } else {
                    let text = if scenario == "duplicate" {
                        r#"<tool_call>{"name":"read","id":"same","args":{}}</tool_call><tool_call>{"name":"read","id":"same","args":{}}</tool_call>"#
                    } else if protocol == ToolProtocol::TalosStrict {
                        "prose<toolcall>read {}</toolcall>"
                    } else {
                        "<tool_call>read {}</tool_call><tool_call>broken"
                    };
                    tx.send(AgentEvent::TextDelta { delta: text.into() })
                        .await
                        .expect("text");
                }
                tx.send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                })
                .await
                .expect("end");
                drop(tx);
                let mut errors = 0;
                while let Some(event) = output.recv().await {
                    assert!(!matches!(
                        event,
                        AgentEvent::ToolCall { .. } | AgentEvent::TurnEnd { .. }
                    ));
                    errors += usize::from(matches!(event, AgentEvent::Error { .. }));
                }
                assert_eq!(errors, 1);
            }
        }
    }
}
