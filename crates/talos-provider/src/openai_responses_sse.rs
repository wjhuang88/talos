//! OpenAI Responses semantic stream event state machine.
//!
//! Transport framing is intentionally outside this module. It consumes already-decoded Responses
//! JSON events so the typed wire contract can be tested independently from HTTP/SSE chunking.

use std::collections::HashMap;

use serde_json::Value;
use talos_core::message::{AgentEvent, StopReason, ToolCall, Usage};

/// One parsed Responses stream output.
///
/// Encrypted reasoning is kept opaque and separate from [`AgentEvent`] until PROVIDER-008-A adds a
/// dedicated durable core representation. It must never be rendered or coerced into Anthropic
/// `redacted_thinking` / OpenAI-compatible plain reasoning text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ResponsesStreamOutput {
    Agent(AgentEvent),
    EncryptedReasoning { item_id: String, data: String },
}

#[derive(Debug, Default)]
struct PendingFunctionCall {
    call_id: String,
    name: String,
    arguments: String,
    started: bool,
    emitted: bool,
}

/// Incremental state for one OpenAI Responses stream.
#[derive(Debug, Default)]
pub(crate) struct ResponsesStreamState {
    function_calls: HashMap<String, PendingFunctionCall>,
    terminal: bool,
    saw_tool_call: bool,
}

impl ResponsesStreamState {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Apply one decoded Responses streaming event.
    ///
    /// Unknown event types are ignored for forward compatibility. Terminal error events are
    /// converted into a single Talos error event and mark the stream terminal.
    pub(crate) fn apply(&mut self, event: &Value) -> Vec<ResponsesStreamOutput> {
        let Some(event_type) = event.get("type").and_then(Value::as_str) else {
            return Vec::new();
        };

        match event_type {
            "response.created" => vec![agent(AgentEvent::TurnStart)],
            "response.output_text.delta" => event
                .get("delta")
                .and_then(Value::as_str)
                .filter(|delta| !delta.is_empty())
                .map(|delta| {
                    vec![agent(AgentEvent::TextDelta {
                        delta: delta.to_string(),
                    })]
                })
                .unwrap_or_default(),
            "response.reasoning_summary_text.delta" => event
                .get("delta")
                .and_then(Value::as_str)
                .filter(|delta| !delta.is_empty())
                .map(|delta| {
                    vec![agent(AgentEvent::ThinkingDelta {
                        delta: delta.to_string(),
                    })]
                })
                .unwrap_or_default(),
            "response.output_item.added" => self.output_item_added(event),
            "response.function_call_arguments.delta" => {
                self.function_arguments_delta(event);
                Vec::new()
            }
            "response.function_call_arguments.done" => {
                self.function_arguments_done(event);
                Vec::new()
            }
            "response.output_item.done" => self.output_item_done(event),
            "response.completed" => self.completed(event),
            "response.failed" | "response.incomplete" => self.response_error(event, event_type),
            "error" => self.top_level_error(event),
            _ => Vec::new(),
        }
    }

    /// Finish a stream after transport EOF.
    ///
    /// Responses streams must carry a semantic terminal event. EOF without one is an error rather
    /// than an implicit successful turn end.
    pub(crate) fn finish_eof(&mut self) -> Vec<ResponsesStreamOutput> {
        if self.terminal {
            Vec::new()
        } else {
            self.terminal = true;
            vec![agent(AgentEvent::Error {
                message: "OpenAI Responses stream ended before a terminal event".into(),
            })]
        }
    }

    fn output_item_added(&mut self, event: &Value) -> Vec<ResponsesStreamOutput> {
        let Some(item) = event.get("item") else {
            return Vec::new();
        };
        if item.get("type").and_then(Value::as_str) != Some("function_call") {
            return Vec::new();
        }

        let Some(item_id) = item_identity(event, item) else {
            return Vec::new();
        };
        let pending = self.function_calls.entry(item_id).or_default();
        update_function_call(pending, item);
        if pending.started || pending.name.is_empty() {
            return Vec::new();
        }
        pending.started = true;
        vec![agent(AgentEvent::ToolCallStarted {
            name: pending.name.clone(),
        })]
    }

    fn function_arguments_delta(&mut self, event: &Value) {
        let Some(item_id) = event.get("item_id").and_then(Value::as_str) else {
            return;
        };
        let Some(delta) = event.get("delta").and_then(Value::as_str) else {
            return;
        };
        self.function_calls
            .entry(item_id.to_string())
            .or_default()
            .arguments
            .push_str(delta);
    }

    fn function_arguments_done(&mut self, event: &Value) {
        let Some(item_id) = event.get("item_id").and_then(Value::as_str) else {
            return;
        };
        let Some(arguments) = event.get("arguments").and_then(Value::as_str) else {
            return;
        };
        self.function_calls
            .entry(item_id.to_string())
            .or_default()
            .arguments = arguments.to_string();
    }

    fn output_item_done(&mut self, event: &Value) -> Vec<ResponsesStreamOutput> {
        let Some(item) = event.get("item") else {
            return Vec::new();
        };
        match item.get("type").and_then(Value::as_str) {
            Some("function_call") => self.function_call_done(event, item),
            Some("reasoning") => reasoning_done(event, item),
            _ => Vec::new(),
        }
    }

    fn function_call_done(&mut self, event: &Value, item: &Value) -> Vec<ResponsesStreamOutput> {
        let Some(item_id) = item_identity(event, item) else {
            return vec![agent(AgentEvent::Error {
                message: "OpenAI Responses function call completed without an item id".into(),
            })];
        };
        let pending = self.function_calls.entry(item_id).or_default();
        update_function_call(pending, item);
        if pending.emitted {
            return Vec::new();
        }
        if pending.call_id.is_empty() || pending.name.is_empty() {
            return vec![agent(AgentEvent::Error {
                message: "OpenAI Responses function call completed without call_id or name".into(),
            })];
        }
        let input = match serde_json::from_str(&pending.arguments) {
            Ok(input) => input,
            Err(_) => {
                return vec![agent(AgentEvent::Error {
                    message: "invalid OpenAI Responses function arguments JSON".into(),
                })];
            }
        };
        pending.emitted = true;
        self.saw_tool_call = true;
        vec![agent(AgentEvent::ToolCall {
            call: ToolCall {
                id: pending.call_id.clone(),
                name: pending.name.clone(),
                input,
            },
            provenance: Default::default(),
            summary_fields: vec![],
        })]
    }

    fn completed(&mut self, event: &Value) -> Vec<ResponsesStreamOutput> {
        if self.terminal {
            return Vec::new();
        }
        self.terminal = true;
        let response = event.get("response").unwrap_or(event);
        vec![agent(AgentEvent::TurnEnd {
            stop_reason: if self.saw_tool_call {
                StopReason::ToolUse
            } else {
                StopReason::EndTurn
            },
            usage: response_usage(response),
        })]
    }

    fn response_error(&mut self, event: &Value, event_type: &str) -> Vec<ResponsesStreamOutput> {
        if self.terminal {
            return Vec::new();
        }
        self.terminal = true;
        let response = event.get("response").unwrap_or(event);
        let message = nested_error_message(response).unwrap_or_else(|| event_type.to_string());
        vec![agent(AgentEvent::Error {
            message: format!("OpenAI Responses {event_type}: {message}"),
        })]
    }

    fn top_level_error(&mut self, event: &Value) -> Vec<ResponsesStreamOutput> {
        if self.terminal {
            return Vec::new();
        }
        self.terminal = true;
        let message = nested_error_message(event)
            .or_else(|| event.get("message").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "unknown streaming error".to_string());
        vec![agent(AgentEvent::Error {
            message: format!("OpenAI Responses error: {message}"),
        })]
    }
}

fn agent(event: AgentEvent) -> ResponsesStreamOutput {
    ResponsesStreamOutput::Agent(event)
}

fn item_identity(event: &Value, item: &Value) -> Option<String> {
    item.get("id")
        .and_then(Value::as_str)
        .or_else(|| event.get("item_id").and_then(Value::as_str))
        .map(str::to_string)
}

fn update_function_call(pending: &mut PendingFunctionCall, item: &Value) {
    if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
        pending.call_id = call_id.to_string();
    }
    if let Some(name) = item.get("name").and_then(Value::as_str) {
        pending.name = name.to_string();
    }
    if let Some(arguments) = item.get("arguments").and_then(Value::as_str) {
        pending.arguments = arguments.to_string();
    }
}

fn reasoning_done(event: &Value, item: &Value) -> Vec<ResponsesStreamOutput> {
    let Some(data) = item
        .get("encrypted_content")
        .and_then(Value::as_str)
        .filter(|data| !data.is_empty())
    else {
        return Vec::new();
    };
    let Some(item_id) = item_identity(event, item) else {
        return Vec::new();
    };
    vec![ResponsesStreamOutput::EncryptedReasoning {
        item_id,
        data: data.to_string(),
    }]
}

fn response_usage(response: &Value) -> Usage {
    let usage = response.get("usage").unwrap_or(&Value::Null);
    let input_tokens = u32_field(usage, "input_tokens");
    let output_tokens = u32_field(usage, "output_tokens");
    let cache_read_tokens = usage
        .get("input_tokens_details")
        .map(|details| u32_field(details, "cached_tokens"))
        .unwrap_or_default();
    let reasoning_tokens = usage
        .get("output_tokens_details")
        .map(|details| u32_field(details, "reasoning_tokens"))
        .unwrap_or_default();
    Usage {
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens: 0,
        reasoning_tokens,
    }
}

fn u32_field(value: &Value, field: &str) -> u32 {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default()
}

fn nested_error_message(value: &Value) -> Option<String> {
    value
        .get("error")
        .and_then(|error| {
            error
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| error.as_str())
        })
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn single_agent(outputs: Vec<ResponsesStreamOutput>) -> AgentEvent {
        match outputs.as_slice() {
            [ResponsesStreamOutput::Agent(event)] => event.clone(),
            other => panic!("expected one agent event, got {other:?}"),
        }
    }

    #[test]
    fn text_and_reasoning_summary_deltas_map_to_distinct_talos_events() {
        let mut state = ResponsesStreamState::new();
        assert!(matches!(
            single_agent(state.apply(&json!({"type": "response.created"}))),
            AgentEvent::TurnStart
        ));
        assert!(matches!(
            single_agent(state.apply(&json!({
                "type": "response.output_text.delta",
                "delta": "hello"
            }))),
            AgentEvent::TextDelta { delta } if delta == "hello"
        ));
        assert!(matches!(
            single_agent(state.apply(&json!({
                "type": "response.reasoning_summary_text.delta",
                "delta": "checking"
            }))),
            AgentEvent::ThinkingDelta { delta } if delta == "checking"
        ));
    }

    #[test]
    fn function_call_accumulates_arguments_and_emits_on_completed_item() {
        let mut state = ResponsesStreamState::new();
        let started = single_agent(state.apply(&json!({
            "type": "response.output_item.added",
            "item": {
                "id": "fc_item_1",
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": ""
            }
        })));
        assert!(matches!(started, AgentEvent::ToolCallStarted { name } if name == "lookup"));
        assert!(state
            .apply(&json!({
                "type": "response.function_call_arguments.delta",
                "item_id": "fc_item_1",
                "delta": "{\"key\":"
            }))
            .is_empty());
        assert!(state
            .apply(&json!({
                "type": "response.function_call_arguments.done",
                "item_id": "fc_item_1",
                "arguments": "{\"key\":\"value\"}"
            }))
            .is_empty());
        let completed = single_agent(state.apply(&json!({
            "type": "response.output_item.done",
            "item": {
                "id": "fc_item_1",
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{\"key\":\"value\"}"
            }
        })));
        assert!(matches!(
            completed,
            AgentEvent::ToolCall { call, .. }
                if call.id == "call_1"
                    && call.name == "lookup"
                    && call.input == json!({"key": "value"})
        ));
    }

    #[test]
    fn encrypted_reasoning_is_taken_only_from_completed_output_item() {
        let mut state = ResponsesStreamState::new();
        assert!(state
            .apply(&json!({
                "type": "response.output_item.added",
                "item": {
                    "id": "rs_1",
                    "type": "reasoning",
                    "encrypted_content": "partial-secret"
                }
            }))
            .is_empty());
        let outputs = state.apply(&json!({
            "type": "response.output_item.done",
            "item": {
                "id": "rs_1",
                "type": "reasoning",
                "encrypted_content": "complete-secret"
            }
        }));
        assert_eq!(
            outputs,
            vec![ResponsesStreamOutput::EncryptedReasoning {
                item_id: "rs_1".into(),
                data: "complete-secret".into()
            }]
        );
    }

    #[test]
    fn completed_event_maps_usage_and_tool_stop_reason() {
        let mut state = ResponsesStreamState::new();
        state.apply(&json!({
            "type": "response.output_item.done",
            "item": {
                "id": "fc_1",
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{}"
            }
        }));
        let end = single_agent(state.apply(&json!({
            "type": "response.completed",
            "response": {
                "usage": {
                    "input_tokens": 120,
                    "output_tokens": 30,
                    "input_tokens_details": {"cached_tokens": 20},
                    "output_tokens_details": {"reasoning_tokens": 7}
                }
            }
        })));
        match end {
            AgentEvent::TurnEnd { stop_reason, usage } => {
                assert_eq!(stop_reason, StopReason::ToolUse);
                assert_eq!(usage.input_tokens, 120);
                assert_eq!(usage.output_tokens, 30);
                assert_eq!(usage.cache_read_tokens, 20);
                assert_eq!(usage.reasoning_tokens, 7);
            }
            other => panic!("expected TurnEnd, got {other:?}"),
        }
        assert!(state.finish_eof().is_empty());
    }

    #[test]
    fn unknown_events_are_ignored_and_eof_without_terminal_is_error() {
        let mut state = ResponsesStreamState::new();
        assert!(state
            .apply(&json!({"type": "response.future_event", "value": 1}))
            .is_empty());
        assert!(matches!(
            single_agent(state.finish_eof()),
            AgentEvent::Error { message } if message.contains("before a terminal event")
        ));
        assert!(state.finish_eof().is_empty());
    }

    #[test]
    fn failed_incomplete_and_top_level_error_are_terminal() {
        for event in [
            json!({
                "type": "response.failed",
                "response": {"error": {"message": "failed fixture"}}
            }),
            json!({
                "type": "response.incomplete",
                "response": {"error": {"message": "incomplete fixture"}}
            }),
            json!({
                "type": "error",
                "error": {"message": "stream fixture"}
            }),
        ] {
            let mut state = ResponsesStreamState::new();
            assert!(matches!(
                single_agent(state.apply(&event)),
                AgentEvent::Error { .. }
            ));
            assert!(state.finish_eof().is_empty());
        }
    }

    #[test]
    fn invalid_function_arguments_surface_error_without_panicking() {
        let mut state = ResponsesStreamState::new();
        assert!(matches!(
            single_agent(state.apply(&json!({
                "type": "response.output_item.done",
                "item": {
                    "id": "fc_bad",
                    "type": "function_call",
                    "call_id": "call_bad",
                    "name": "lookup",
                    "arguments": "{not-json"
                }
            }))),
            AgentEvent::Error { message } if message.contains("function arguments JSON")
        ));
    }
}
