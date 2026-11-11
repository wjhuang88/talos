//! Terminal-only private signed browser history; no raw payload is a conversation event.

use crate::gemini_stream::GeminiOutput;
use crate::openai_sse::InvocationSender;
use std::sync::Arc;
use talos_core::message::{AgentEvent, AssistantReasoning, Message, ToolCall};
use talos_core::provider::{
    ProviderError, ProviderInvocationEvent, ProviderResult, TransientReplay,
};

fn invalid() -> ProviderError {
    ProviderError::InvalidResponse("invalid Gemini private replay response".into())
}

#[allow(clippy::too_many_arguments)] // One terminal response and its replay routing context.
pub(crate) fn prepare(
    provider: &str,
    model: &str,
    text: &str,
    thoughts: &str,
    terminal: Vec<GeminiOutput>,
    sink: &InvocationSender,
    protocol: talos_core::tool::ToolProtocol,
    had_private_history: bool,
) -> ProviderResult<Vec<ProviderInvocationEvent>> {
    let mut blocks = None;
    let mut end = None;
    let mut raw_calls = Vec::new();
    let mut safe_calls = Vec::new();
    let mut calls = Vec::new();
    let textual = protocol != talos_core::tool::ToolProtocol::Native;
    if textual {
        calls = crate::anthropic_stream::parse_invocation_text_calls(text, protocol, sink)
            .map_err(|_| invalid())?;
        let mut ids = std::collections::HashSet::new();
        for event in &calls {
            let raw = match event {
                ProviderInvocationEvent::BrowserToolCall {
                    id,
                    name,
                    arguments,
                } => ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    input: serde_json::to_value(arguments.fields()).map_err(|_| invalid())?,
                },
                ProviderInvocationEvent::Legacy(AgentEvent::ToolCall { call, .. }) => call.clone(),
                _ => return Err(invalid()),
            };
            if !ids.insert(raw.id.clone()) {
                return Err(invalid());
            }
            let mut safe = raw.clone();
            if sink.is_browser(&raw.name) {
                safe.input = serde_json::json!({"protocolVersion":2});
            }
            raw_calls.push(raw);
            safe_calls.push(safe);
        }
    }
    for event in terminal {
        match event {
            GeminiOutput::Event(AgentEvent::ReasoningComplete { blocks: reasoning })
                if blocks.is_none() && end.is_none() =>
            {
                blocks = Some(reasoning)
            }
            GeminiOutput::Event(event @ AgentEvent::TurnEnd { .. }) if end.is_none() => {
                end = Some(event)
            }
            GeminiOutput::FunctionCall {
                id,
                name,
                arguments,
            } if end.is_none() && !textual => {
                let raw_input = serde_json::from_str(&arguments).map_err(|_| invalid())?;
                let raw = ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    input: raw_input,
                };
                raw_calls.push(raw.clone());
                if sink.is_browser(&name) {
                    let arguments =
                        talos_core::tool::BrowserRawArguments::parse_original(&arguments)
                            .map_err(|_| invalid())?;
                    safe_calls.push(ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input: serde_json::json!({"protocolVersion":2}),
                    });
                    calls.push(ProviderInvocationEvent::BrowserToolCall {
                        id,
                        name,
                        arguments,
                    });
                } else {
                    safe_calls.push(raw.clone());
                    calls.push(ProviderInvocationEvent::Legacy(AgentEvent::ToolCall {
                        call: raw,
                        provenance: Default::default(),
                        summary_fields: vec![],
                    }));
                }
            }
            _ => return Err(invalid()),
        }
    }
    let blocks = blocks.ok_or_else(invalid)?;
    let mut end = end.ok_or_else(invalid)?;
    if !calls.is_empty()
        && !matches!(
            &end,
            AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn
                    | talos_core::message::StopReason::ToolUse,
                ..
            }
        )
    {
        return Err(invalid());
    }
    if !calls.is_empty()
        && let AgentEvent::TurnEnd { stop_reason, .. } = &mut end
        && *stop_reason == talos_core::message::StopReason::EndTurn
    {
        *stop_reason = talos_core::message::StopReason::ToolUse;
    }
    if textual && !had_private_history && calls.is_empty() {
        let mut output = vec![
            ProviderInvocationEvent::Legacy(AgentEvent::ReasoningComplete { blocks }),
            ProviderInvocationEvent::Legacy(AgentEvent::ThinkingDelta {
                delta: thoughts.into(),
            }),
            ProviderInvocationEvent::Legacy(AgentEvent::TextDelta { delta: text.into() }),
        ];
        output.extend(calls);
        output.push(ProviderInvocationEvent::Legacy(end));
        return Ok(output);
    }
    let safe_text = if textual {
        crate::browser_text_arguments::outside_tool_blocks(text)
            .map_err(|_| invalid())?
            .trim()
            .to_owned()
    } else {
        talos_core::message::strip_tool_syntax(text)
    };
    let public = Message::Assistant {
        content: safe_text.clone(),
        tool_calls: safe_calls,
        reasoning: None,
    };
    let private = Message::Assistant {
        content: text.into(),
        tool_calls: raw_calls,
        reasoning: Some(AssistantReasoning {
            provider: provider.into(),
            model: model.into(),
            blocks,
        }),
    };
    let replay = Arc::new(TransientReplay::new(public, private)?);
    // Publish custody first. The Agent registers declared private fields before observing
    // any text, thought or ordinary call, and retains that filter across live user turns.
    let mut output = vec![ProviderInvocationEvent::TransientReplay(replay)];
    if !thoughts.is_empty() {
        output.push(ProviderInvocationEvent::Legacy(AgentEvent::ThinkingDelta {
            delta: thoughts.into(),
        }));
    }
    if !safe_text.is_empty() {
        output.push(ProviderInvocationEvent::Legacy(AgentEvent::TextDelta {
            delta: safe_text,
        }));
    }
    output.extend(calls);
    output.push(ProviderInvocationEvent::Legacy(end));
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::{
        message::{StopReason, Usage},
        tool::ToolProtocol,
    };

    fn prepare_text(
        text: &str,
        protocol: ToolProtocol,
        stop_reason: StopReason,
    ) -> ProviderResult<Vec<ProviderInvocationEvent>> {
        let (tx, _rx) = tokio::sync::mpsc::channel(32);
        let sink = InvocationSender::Original {
            tx,
            browser_tools: vec!["browser".into()],
        };
        let blocks = vec![
            crate::gemini_replay::encode(
                "google",
                "model",
                &[serde_json::json!({"text":text,"thoughtSignature":"opaque"})],
                &[],
            )
            .expect("signed text"),
        ];
        prepare(
            "google",
            "model",
            text,
            "",
            vec![
                GeminiOutput::Event(AgentEvent::ReasoningComplete { blocks }),
                GeminiOutput::Event(AgentEvent::TurnEnd {
                    stop_reason,
                    usage: Usage::default(),
                }),
            ],
            &sink,
            protocol,
            false,
        )
    }

    #[test]
    fn textual_browser_removes_raw_blocks_and_refuses_truncated_calls() {
        let text = r#"<tool_call>{"name":"browser","id":"b","args":{"protocolVersion":2,"operation":"fill","text":"secret\nvalue"}}</tool_call>after"#;
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            let events = prepare_text(text, protocol, StopReason::EndTurn).expect("complete");
            assert!(matches!(
                events.first(),
                Some(ProviderInvocationEvent::TransientReplay(_))
            ));
            let visible = events
                .iter()
                .filter_map(|event| match event {
                    ProviderInvocationEvent::Legacy(AgentEvent::TextDelta { delta }) => {
                        Some(delta.as_str())
                    }
                    _ => None,
                })
                .collect::<String>();
            assert_eq!(visible, "after");
            assert!(prepare_text(text, protocol, StopReason::MaxTokens).is_err());
        }
    }

    #[test]
    fn ordinary_signed_tool_frames_restore_exactly_with_browser_registered() {
        for (protocol, text) in [
            (
                ToolProtocol::Compat,
                "  ```json-tool\n{\"name\":\"ordinary\",\"id\":\"o\",\"args\":{}}\n```  ",
            ),
            (
                ToolProtocol::TalosStrict,
                "  <tool_call>{\"name\":\"ordinary\",\"id\":\"o\",\"args\":{}}</tool_call>  ",
            ),
        ] {
            let events = prepare_text(text, protocol, StopReason::EndTurn).expect("complete");
            let ProviderInvocationEvent::TransientReplay(replay) = &events[0] else {
                panic!("signed tools require exact original custody")
            };
            let restored = replay
                .restore(replay.projected_message(), "google", "model")
                .expect("restore");
            let result = Message::Tool {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "o".into(),
                    content: "done".into(),
                    is_error: false,
                },
            };
            let projected =
                crate::gemini_compat::project("google", "model", &[restored, result], protocol)
                    .expect("signed projection");
            assert!(matches!(&projected[0], Message::Assistant { content, .. } if content == text));
        }
    }

    #[test]
    fn malformed_browser_batches_release_no_events() {
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            for text in [
                r#"<tool_call>{"name":"browser","id":"b","args":{"text":"secret","te\u0078t":"duplicate"}}</tool_call>"#,
                r#"<tool_call>{"name":"browser","name":"ordinary","id":"b","args":{"text":"secret"}}</tool_call>"#,
                r#"<tool_call>{"name":"browser","id":"b","args":{"text":"secret"}}"#,
                r#"<tool_call>{"name":"browser","id":"b","args":{"text":"secret"}}</tool_call><tool_call>{"name":"browser","id":"b","args":{}}</tool_call>"#,
            ] {
                assert!(prepare_text(text, protocol, StopReason::EndTurn).is_err());
            }
        }
    }

    #[test]
    fn ordinary_text_does_not_acquire_private_custody() {
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            let events =
                prepare_text("ordinary response", protocol, StopReason::EndTurn).expect("ordinary");
            assert!(
                events
                    .iter()
                    .all(|event| !matches!(event, ProviderInvocationEvent::TransientReplay(_)))
            );
        }
    }
}
