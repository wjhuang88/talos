//! A downstream provider implementing only runtime-exported protocol types.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use talos_runtime::{
    AgentEvent, CapabilityProbe, LanguageModel, Message, ProtocolCapabilities, ProviderError,
    ProviderResult, Receiver, StopReason, ToolCall, ToolProvenance, Usage,
};

#[derive(Default)]
pub struct FixtureProvider {
    responses: Mutex<VecDeque<(AgentEvent, StopReason)>>,
}

impl FixtureProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_response(mut self, text: &str) -> Self {
        self.responses.get_mut().expect("fixture mutex").push_back((
            AgentEvent::TextDelta { delta: text.into() },
            StopReason::EndTurn,
        ));
        self
    }

    pub fn with_tool_call(mut self, name: &str, input: serde_json::Value) -> Self {
        self.responses.get_mut().expect("fixture mutex").push_back((
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "fixture-call".into(),
                    name: name.into(),
                    input,
                },
                provenance: ToolProvenance::Native,
                summary_fields: vec![],
            },
            StopReason::ToolUse,
        ));
        self
    }
}

#[async_trait]
impl LanguageModel for FixtureProvider {
    fn protocol_capabilities(&self) -> CapabilityProbe {
        // This deterministic adapter emits structured calls; it does not parse text tools.
        CapabilityProbe::Known(ProtocolCapabilities {
            native_tools: true,
            compatibility: false,
        })
    }

    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (event, stop_reason) = self
            .responses
            .lock()
            .map_err(|_| ProviderError::InvalidResponse("fixture mutex poisoned".into()))?
            .pop_front()
            .ok_or_else(|| ProviderError::InvalidResponse("fixture response exhausted".into()))?;
        let (tx, rx) = tokio::sync::mpsc::channel(3);
        for event in [
            AgentEvent::TurnStart,
            event,
            AgentEvent::TurnEnd {
                stop_reason,
                usage: Usage::default(),
            },
        ] {
            tx.try_send(event).map_err(|_| {
                ProviderError::InvalidResponse("fixture channel unavailable".into())
            })?;
        }
        Ok(rx)
    }
}
