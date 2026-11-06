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

pub(crate) fn prepare(
    provider: &str,
    model: &str,
    text: &str,
    thoughts: &str,
    terminal: Vec<GeminiOutput>,
    sink: &InvocationSender,
) -> ProviderResult<Vec<ProviderInvocationEvent>> {
    let mut blocks = None;
    let mut end = None;
    let mut raw_calls = Vec::new();
    let mut safe_calls = Vec::new();
    let mut calls = Vec::new();
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
            } if end.is_none() => {
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
    let end = end.ok_or_else(invalid)?;
    let safe_text = talos_core::message::strip_tool_syntax(text);
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
