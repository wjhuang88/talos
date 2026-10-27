//! Projection from OpenAI Responses semantic outputs into Talos' shared invocation ingress.
//!
//! The semantic parser intentionally preserves native function-call argument bytes. This bridge is
//! the only place that turns those outputs into Talos invocation events, reusing `InvocationSender`
//! so browser tools retain trusted original-syntax evidence and ordinary tools share the existing
//! JSON validation path.

use tokio::sync::mpsc;

use crate::openai_responses_sse::ResponsesStreamOutput;
use crate::openai_sse::InvocationSender;

/// Forward one Responses semantic stream into the selected Talos invocation sink.
///
/// Channel closure is treated as cancellation and stops forwarding without manufacturing another
/// provider error. Semantic/provider errors are already represented as `AgentEvent::Error` by the
/// Responses state machine or transport.
pub(crate) async fn forward_response_stream(
    mut source: mpsc::Receiver<ResponsesStreamOutput>,
    sink: InvocationSender,
) {
    while let Some(output) = source.recv().await {
        let sent = match output {
            ResponsesStreamOutput::Agent(event) => sink.send(event).await,
            ResponsesStreamOutput::FunctionCall {
                id,
                name,
                arguments,
            } => sink.native_call(id, name, &arguments).await,
        };
        if sent.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use talos_core::message::{AgentEvent, StopReason, Usage};
    use talos_core::provider::{ProviderInvocationEvent, ProviderInvocationStream};

    #[tokio::test]
    async fn forwards_agent_events_without_rewriting_them() {
        let (source_tx, source_rx) = mpsc::channel(4);
        let (target_tx, target_rx) = mpsc::channel(4);
        let task = tokio::spawn(forward_response_stream(
            source_rx,
            InvocationSender::Original {
                tx: target_tx,
                browser_tools: vec![],
            },
        ));

        source_tx
            .send(ResponsesStreamOutput::Agent(AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 3,
                    output_tokens: 5,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                    reasoning_tokens: 1,
                },
            }))
            .await
            .expect("source receiver live");
        drop(source_tx);

        let mut stream = ProviderInvocationStream::original(target_rx);
        let Some(ProviderInvocationEvent::Legacy(AgentEvent::TurnEnd { stop_reason, usage })) =
            stream.recv().await
        else {
            panic!("expected forwarded turn end");
        };
        assert_eq!(stop_reason, StopReason::EndTurn);
        assert_eq!(usage.input_tokens, 3);
        assert_eq!(usage.reasoning_tokens, 1);
        task.await.expect("forwarder task");
    }

    #[tokio::test]
    async fn ordinary_function_call_uses_shared_native_json_projection() {
        let (source_tx, source_rx) = mpsc::channel(4);
        let (target_tx, target_rx) = mpsc::channel(4);
        let task = tokio::spawn(forward_response_stream(
            source_rx,
            InvocationSender::Original {
                tx: target_tx,
                browser_tools: vec![],
            },
        ));

        source_tx
            .send(ResponsesStreamOutput::FunctionCall {
                id: "call-1".into(),
                name: "lookup".into(),
                arguments: "{ \"key\" : \"value\" }".into(),
            })
            .await
            .expect("source receiver live");
        drop(source_tx);

        let mut stream = ProviderInvocationStream::original(target_rx);
        let Some(ProviderInvocationEvent::Legacy(AgentEvent::ToolCall {
            call, provenance, ..
        })) = stream.recv().await
        else {
            panic!("expected native tool call");
        };
        assert_eq!(call.id, "call-1");
        assert_eq!(call.name, "lookup");
        assert_eq!(call.input, json!({"key": "value"}));
        assert_eq!(provenance, talos_core::tool::ToolProvenance::Native);
        task.await.expect("forwarder task");
    }

    #[tokio::test]
    async fn browser_function_call_preserves_original_argument_syntax_evidence() {
        let (source_tx, source_rx) = mpsc::channel(4);
        let (target_tx, target_rx) = mpsc::channel(4);
        let task = tokio::spawn(forward_response_stream(
            source_rx,
            InvocationSender::Original {
                tx: target_tx,
                browser_tools: vec!["browser".into()],
            },
        ));

        let original = "{ \"url\" : \"https://example.com\", \"wait\" : 1 }";
        source_tx
            .send(ResponsesStreamOutput::FunctionCall {
                id: "browser-1".into(),
                name: "browser".into(),
                arguments: original.into(),
            })
            .await
            .expect("source receiver live");
        drop(source_tx);

        let mut stream = ProviderInvocationStream::original(target_rx);
        let Some(ProviderInvocationEvent::BrowserToolCall {
            id,
            name,
            arguments,
        }) = stream.recv().await
        else {
            panic!("expected browser evidence event");
        };
        assert_eq!(id, "browser-1");
        assert_eq!(name, "browser");
        assert_eq!(arguments.fields()["url"], "https://example.com");
        assert_eq!(arguments.fields()["wait"], 1);
        task.await.expect("forwarder task");
    }

    #[tokio::test]
    async fn invalid_function_json_fails_closed_and_stops_forwarding() {
        let (source_tx, source_rx) = mpsc::channel(4);
        let (target_tx, target_rx) = mpsc::channel(4);
        let task = tokio::spawn(forward_response_stream(
            source_rx,
            InvocationSender::Original {
                tx: target_tx,
                browser_tools: vec![],
            },
        ));

        source_tx
            .send(ResponsesStreamOutput::FunctionCall {
                id: "bad".into(),
                name: "lookup".into(),
                arguments: "{not-json".into(),
            })
            .await
            .expect("source receiver live");
        source_tx
            .send(ResponsesStreamOutput::Agent(AgentEvent::TextDelta {
                delta: "must not leak after invalid call".into(),
            }))
            .await
            .expect("source receiver live");
        drop(source_tx);

        let mut stream = ProviderInvocationStream::original(target_rx);
        let Some(ProviderInvocationEvent::Legacy(AgentEvent::Error { message })) =
            stream.recv().await
        else {
            panic!("expected validation error");
        };
        assert!(message.contains("invalid tool arguments JSON"));
        assert!(stream.recv().await.is_none());
        task.await.expect("forwarder task");
    }
}
