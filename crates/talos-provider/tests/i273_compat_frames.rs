//! Invalid compatibility frames must not produce a partially executable response.
use serde_json::json;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::LanguageModel;
use talos_core::tool::ToolProtocol;
use talos_provider::{AnthropicProvider, openai::OpenAIProvider};

#[tokio::test]
async fn protocol_frame_matrix_rejects_invalid_batches_and_preserves_valid_calls() {
    let valid = "```json-tool\n{\"name\":\"probe\",\"args\":{}}\n```";
    let strict = "<tool_call>{\"name\":\"probe\",\"args\":{}}</tool_call>";
    for (text, protocol, expected_calls) in [
        (
            "```json-tool\n{\"name\":\"probe\",\"args\":{}}".to_owned(),
            ToolProtocol::Compat,
            None,
        ),
        (
            "<tool_call>{\"name\":\"probe\",\"args\":{}}".to_owned(),
            ToolProtocol::Compat,
            None,
        ),
        (
            format!("{valid}\n```json-tool\nnot JSON\n```"),
            ToolProtocol::Compat,
            None,
        ),
        (
            format!("{valid}\n<toolcall>{{\"name\":\"probe\",\"args\":null}}</toolcall>"),
            ToolProtocol::Compat,
            None,
        ),
        (valid.to_owned(), ToolProtocol::Compat, Some(1)),
        (valid.to_owned(), ToolProtocol::Native, Some(0)),
        (valid.to_owned(), ToolProtocol::TalosStrict, None),
        (strict.to_owned(), ToolProtocol::TalosStrict, Some(1)),
        (
            format!("{strict} explanation"),
            ToolProtocol::TalosStrict,
            Some(1),
        ),
        (
            format!("explanation {strict}"),
            ToolProtocol::TalosStrict,
            None,
        ),
        (format!("{strict}{strict}"), ToolProtocol::TalosStrict, None),
        (
            "<tool_call>probe {}</tool_call>".into(),
            ToolProtocol::TalosStrict,
            None,
        ),
        ("ordinary answer".into(), ToolProtocol::TalosStrict, Some(0)),
    ] {
        for anthropic in [false, true] {
            let mut server = mockito::Server::new_async().await;
            let body = if anthropic {
                format!(
                    "event: content_block_delta\ndata: {}\n\nevent: message_delta\ndata: {}\n\n",
                    json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":text}}),
                    json!({"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":10}})
                )
            } else {
                format!(
                    "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
                    json!({"choices":[{"delta":{"content":text},"finish_reason":null}]}),
                    json!({"choices":[{"delta":{},"finish_reason":"stop"}]})
                )
            };
            let mock = server
                .mock("POST", if anthropic { "/" } else { "/chat/completions" })
                .with_header("content-type", "text/event-stream")
                .with_body(body)
                .create_async()
                .await;
            let provider: Box<dyn LanguageModel> = if anthropic {
                Box::new(AnthropicProvider::new("test", "fixture").with_base_url(server.url()))
            } else {
                Box::new(OpenAIProvider::new("test", "fixture").with_base_url(server.url()))
            };
            let (tx, _) = tokio::sync::mpsc::unbounded_channel();
            let mut events = provider
                .stream_with_protocol(
                    &[Message::User {
                        content: "fixture".into(),
                    }],
                    &[],
                    protocol,
                    tx,
                )
                .await
                .expect("dispatch");
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                let mut failed = false;
                let mut calls = 0;
                let mut completed = false;
                while let Some(event) = events.recv().await {
                    calls += usize::from(matches!(event, AgentEvent::ToolCall { .. }));
                    completed |= matches!(event, AgentEvent::TurnEnd { .. });
                    failed |= matches!(event, AgentEvent::Error { .. });
                }
                if let Some(expected) = expected_calls {
                    assert!(!failed, "valid response rejected: {protocol:?} {text}");
                    assert!(completed);
                    assert_eq!(calls, expected);
                } else {
                    assert!(failed, "invalid response accepted: {protocol:?} {text}");
                    assert!(!completed);
                    assert_eq!(calls, 0);
                }
            })
            .await
            .expect("bounded fixture");
            mock.assert_async().await;
        }
    }
}
