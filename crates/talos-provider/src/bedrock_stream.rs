//! Translate ConverseStream events. Tool calls become actionable only after clean EOF.
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use talos_core::message::{AgentEvent, StopReason, ToolCall, Usage};
use talos_core::provider::{ProviderError, ProviderResult};
use talos_core::tool::{ToolProtocol, ToolProvenance};

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

enum Block {
    Text,
    Reasoning {
        text: String,
        signature: String,
    },
    Tool {
        id: String,
        name: String,
        input: String,
    },
}

#[derive(Default)]
pub(crate) struct ConverseStream {
    textual: Option<(ToolProtocol, String)>,
    started: bool,
    open: BTreeMap<u32, Block>,
    closed: HashSet<u32>,
    ids: HashSet<String>,
    signed: BTreeMap<u32, (String, String)>,
    reasoning_bytes: usize,
    calls: BTreeMap<u32, ToolCall>,
    stop: Option<StopReason>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenCounts {
    input_tokens: u32,
    output_tokens: u32,
    #[serde(default)]
    cache_read_input_tokens: u32,
    #[serde(default)]
    cache_write_input_tokens: u32,
}

impl ConverseStream {
    pub(crate) fn new(protocol: ToolProtocol) -> Self {
        Self {
            textual: matches!(protocol, ToolProtocol::Compat | ToolProtocol::TalosStrict)
                .then(|| (protocol, String::new())),
            ..Self::default()
        }
    }

    pub(crate) fn accept(
        &mut self,
        event: &str,
        body: Value,
    ) -> ProviderResult<Option<AgentEvent>> {
        if !self.started {
            if event != "messageStart" || body["role"] != "assistant" {
                return Err(invalid("missing Bedrock assistant message start"));
            }
            self.started = true;
            return Ok(None);
        }
        if self.stop.is_some() {
            if event != "metadata" || self.usage.is_some() {
                return Err(invalid("unexpected event after Bedrock message stop"));
            }
            let counts: TokenCounts = serde_json::from_value(body["usage"].clone())
                .map_err(|_| invalid("invalid Bedrock token usage"))?;
            self.usage = Some(Usage {
                input_tokens: counts.input_tokens,
                output_tokens: counts.output_tokens,
                cache_read_tokens: counts.cache_read_input_tokens,
                cache_write_tokens: counts.cache_write_input_tokens,
                ..Usage::default()
            });
            return Ok(None);
        }
        if event == "messageStop" {
            if !self.open.is_empty() {
                return Err(invalid("Bedrock stopped with incomplete content blocks"));
            }
            let reason = match body["stopReason"].as_str() {
                Some("end_turn" | "stop_sequence") if self.calls.is_empty() => StopReason::EndTurn,
                Some("tool_use") if !self.calls.is_empty() => StopReason::ToolUse,
                Some("max_tokens" | "model_context_window_exceeded") if self.calls.is_empty() => {
                    StopReason::MaxTokens
                }
                _ => return Err(invalid("unsupported or inconsistent Bedrock stop reason")),
            };
            self.stop = Some(reason);
            return Ok(None);
        }
        let index = body["contentBlockIndex"]
            .as_u64()
            .and_then(|i| u32::try_from(i).ok())
            .ok_or_else(|| invalid("missing Bedrock content block index"))?;
        if self.closed.contains(&index) {
            return Err(invalid("Bedrock content block already closed"));
        }
        match event {
            "contentBlockStart" => {
                if self.textual.is_some() {
                    return Err(invalid("unexpected native tool in Bedrock textual mode"));
                }
                let tool = &body["start"]["toolUse"];
                let id = tool["toolUseId"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| invalid("missing Bedrock tool ID"))?;
                let name = tool["name"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| invalid("missing Bedrock tool name"))?;
                if self.open.contains_key(&index) || !self.ids.insert(id.into()) {
                    return Err(invalid("duplicate Bedrock tool start"));
                }
                self.open.insert(
                    index,
                    Block::Tool {
                        id: id.into(),
                        name: name.into(),
                        input: String::new(),
                    },
                );
            }
            "contentBlockDelta" => {
                let delta = &body["delta"];
                if delta.as_object().map(|d| d.len()) != Some(1) {
                    return Err(invalid("invalid Bedrock delta"));
                }
                if let Some(text) = delta["text"].as_str() {
                    if !matches!(self.open.entry(index).or_insert(Block::Text), Block::Text) {
                        return Err(invalid("Bedrock block kind changed"));
                    }
                    if let Some((_, buffer)) = &mut self.textual {
                        if buffer.len().saturating_add(text.len()) > 8 * 1024 * 1024 {
                            return Err(invalid("Bedrock textual tool response too large"));
                        }
                        buffer.push_str(text);
                    }
                    return Ok(Some(AgentEvent::TextDelta { delta: text.into() }));
                }
                if let Some(reasoning) = delta.get("reasoningContent") {
                    let fields = reasoning
                        .as_object()
                        .filter(|fields| fields.len() == 1)
                        .ok_or_else(|| invalid("invalid Bedrock reasoning delta"))?;
                    let (key, value) = fields
                        .iter()
                        .next()
                        .ok_or_else(|| invalid("empty Bedrock reasoning delta"))?;
                    let fragment = value
                        .as_str()
                        .ok_or_else(|| invalid("invalid Bedrock reasoning fragment"))?;
                    self.reasoning_bytes = self.reasoning_bytes.saturating_add(fragment.len());
                    if self.reasoning_bytes > 8 * 1024 * 1024 {
                        return Err(invalid("Bedrock reasoning exceeds turn limit"));
                    }
                    let Block::Reasoning { text, signature } =
                        self.open.entry(index).or_insert_with(|| Block::Reasoning {
                            text: String::new(),
                            signature: String::new(),
                        })
                    else {
                        return Err(invalid("Bedrock block kind changed"));
                    };
                    match key.as_str() {
                        "text" => text.push_str(fragment),
                        "signature" => signature.push_str(fragment),
                        _ => return Err(invalid("unsupported Bedrock reasoning fragment")),
                    }
                    return Ok(None);
                }
                let fragment = delta["toolUse"]["input"]
                    .as_str()
                    .ok_or_else(|| invalid("unsupported Bedrock content delta"))?;
                let Some(Block::Tool { input, .. }) = self.open.get_mut(&index) else {
                    return Err(invalid("missing Bedrock tool start"));
                };
                if input.len().saturating_add(fragment.len()) > 16 * 1024 * 1024 {
                    return Err(invalid("Bedrock tool input too large"));
                }
                input.push_str(fragment);
            }
            "contentBlockStop" => {
                match self
                    .open
                    .remove(&index)
                    .ok_or_else(|| invalid("unknown Bedrock content block"))?
                {
                    Block::Text => {}
                    Block::Reasoning { text, signature } => {
                        if signature.is_empty() {
                            return Err(invalid("missing Bedrock reasoning signature"));
                        }
                        self.signed.insert(index, (text, signature));
                    }
                    Block::Tool { id, name, input } => {
                        let input: Value = serde_json::from_str(&input)
                            .map_err(|_| invalid("invalid Bedrock tool JSON"))?;
                        if !input.is_object() {
                            return Err(invalid("Bedrock tool input must be an object"));
                        }
                        self.calls.insert(index, ToolCall { id, name, input });
                    }
                }
                self.closed.insert(index);
            }
            _ => return Err(invalid("unexpected Bedrock conversation event")),
        }
        Ok(None)
    }

    pub(crate) fn finish(self) -> ProviderResult<Vec<AgentEvent>> {
        if !self.signed.is_empty() {
            return Err(invalid(
                "Bedrock signed reasoning requires a verified replay binding",
            ));
        }
        let mut stop_reason = self
            .stop
            .ok_or_else(|| invalid("Bedrock stream ended before messageStop"))?;
        let calls = if let Some((protocol, text)) = self.textual {
            let mut parsed = Vec::new();
            if stop_reason == StopReason::EndTurn {
                parsed = crate::anthropic_stream::parse_protocol_text_calls(&text, protocol)
                    .map_err(invalid)?;
                let mut ids = HashSet::new();
                if parsed.iter().any(|call| !ids.insert(&call.id)) {
                    return Err(invalid("duplicate Bedrock textual tool ID"));
                }
                if !parsed.is_empty() {
                    stop_reason = StopReason::ToolUse;
                }
            }
            parsed
        } else {
            self.calls.into_values().collect()
        };
        let mut events: Vec<_> = calls
            .into_iter()
            .map(|call| AgentEvent::ToolCall {
                call,
                provenance: ToolProvenance::Native,
                summary_fields: vec![],
            })
            .collect();
        events.push(AgentEvent::TurnEnd {
            stop_reason,
            usage: self.usage.unwrap_or_default(),
        });
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn started() -> ConverseStream {
        let mut stream = ConverseStream::default();
        stream
            .accept("messageStart", json!({"role":"assistant"}))
            .unwrap();
        stream
    }

    #[test]
    fn interleaved_calls_wait_for_stop_metadata_and_clean_eof() {
        let mut stream = started();
        for (index, id) in [(0, "first"), (1, "second")] {
            assert!(stream.accept("contentBlockStart",json!({"contentBlockIndex":index,"start":{"toolUse":{"toolUseId":id,"name":"read"}}})).unwrap().is_none());
        }
        for index in [1, 0] {
            for fragment in ["{", "}"] {
                assert!(stream.accept("contentBlockDelta",json!({"contentBlockIndex":index,"delta":{"toolUse":{"input":fragment}}})).unwrap().is_none());
            }
            stream
                .accept("contentBlockStop", json!({"contentBlockIndex":index}))
                .unwrap();
        }
        stream
            .accept("messageStop", json!({"stopReason":"tool_use"}))
            .unwrap();
        stream
            .accept(
                "metadata",
                json!({"usage":{"inputTokens":11,"outputTokens":3,"cacheReadInputTokens":7}}),
            )
            .unwrap();
        let events = stream.finish().unwrap();
        assert!(
            matches!(&events[..],[AgentEvent::ToolCall{call:a,..},AgentEvent::ToolCall{call:b,..},AgentEvent::TurnEnd{stop_reason:StopReason::ToolUse,usage}] if a.id=="first" && b.id=="second" && usage.input_tokens==11 && usage.cache_read_tokens==7)
        );
    }

    #[test]
    fn textual_calls_require_a_complete_successful_turn() {
        let text = r#"<tool_call>{"name":"read","id":"r","args":{"path":"note"}}</tool_call>"#;
        for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
            for reason in ["end_turn", "max_tokens"] {
                let mut stream = ConverseStream::new(protocol);
                stream
                    .accept("messageStart", json!({"role":"assistant"}))
                    .unwrap();
                for fragment in [&text[..10], &text[10..]] {
                    stream
                        .accept(
                            "contentBlockDelta",
                            json!({"contentBlockIndex":0,"delta":{"text":fragment}}),
                        )
                        .unwrap();
                }
                stream
                    .accept("contentBlockStop", json!({"contentBlockIndex":0}))
                    .unwrap();
                stream
                    .accept("messageStop", json!({"stopReason":reason}))
                    .unwrap();
                let events = stream.finish().unwrap();
                if reason == "end_turn" {
                    assert!(
                        matches!(&events[..], [AgentEvent::ToolCall{call,..},AgentEvent::TurnEnd{stop_reason:StopReason::ToolUse,..}] if call.id=="r" && call.name=="read")
                    );
                } else {
                    assert!(matches!(
                        &events[..],
                        [AgentEvent::TurnEnd {
                            stop_reason: StopReason::MaxTokens,
                            ..
                        }]
                    ));
                }
            }
        }
    }

    #[test]
    fn signed_fragments_preserve_bytes_without_emitting_private_text() {
        let mut stream = started();
        for fragment in [
            json!({"text":" thought "}),
            json!({"signature":" A"}),
            json!({"signature":"B "}),
        ] {
            assert!(
                stream
                    .accept(
                        "contentBlockDelta",
                        json!({"contentBlockIndex":0,"delta":{"reasoningContent":fragment}})
                    )
                    .unwrap()
                    .is_none()
            );
        }
        stream
            .accept("contentBlockStop", json!({"contentBlockIndex":0}))
            .unwrap();
        assert_eq!(
            stream.signed.get(&0),
            Some(&(" thought ".into(), " AB ".into()))
        );
        stream
            .accept("messageStop", json!({"stopReason":"end_turn"}))
            .unwrap();
        assert!(stream.finish().is_err());
    }

    #[test]
    fn incomplete_or_inconsistent_turn_cannot_complete() {
        assert!(started().finish().is_err());
        assert!(
            started()
                .accept("messageStop", json!({"stopReason":"tool_use"}))
                .is_err()
        );
        let mut stream = started();
        stream
            .accept(
                "contentBlockDelta",
                json!({"contentBlockIndex":0,"delta":{"text":"partial"}}),
            )
            .unwrap();
        assert!(
            stream
                .accept("messageStop", json!({"stopReason":"end_turn"}))
                .is_err()
        );
    }
}
