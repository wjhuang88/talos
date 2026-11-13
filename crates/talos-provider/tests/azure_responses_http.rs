use std::time::Duration;

use serde_json::{Value, json};
use std::sync::Arc;
use talos_config::{
    AzureApiMode, AzureAuthentication, AzureBodyShape, AzureTransportConfig, Config, ModelConfig,
    ProviderConfig, ProviderProtocol,
};
use talos_core::message::{
    AgentEvent, ContentDigest, ContentPart, Message, MessageToolResult, StopReason, ToolCall,
};
use talos_core::provider::{LanguageModel, ToolDefinition};
fn responses_provider(endpoint: String) -> Arc<dyn LanguageModel> {
    azure_provider(endpoint, 0)
}
fn azure_provider(endpoint: String, retries: u32) -> Arc<dyn LanguageModel> {
    let mut config = Config::default();
    config.provider = "azure-fixture".into();
    config.model = "gpt-test".into();
    config.providers.insert(
        config.provider.clone(),
        ProviderConfig {
            protocol: ProviderProtocol::AzureOpenAI,
            azure: Some(AzureTransportConfig {
                endpoint,
                api: AzureApiMode::V1,
                shape: Some(AzureBodyShape::Responses),
                authentication: AzureAuthentication::ApiKey,
            }),
            models: [(
                config.model.clone(),
                ModelConfig {
                    azure_deployment: Some("azure-deployment".into()),
                    ..Default::default()
                },
            )]
            .into(),
            timeout: talos_config::ProviderTimeoutConfig {
                dispatch_timeout_secs: 5,
                first_packet_timeout_secs: 5,
                stream_idle_timeout_secs: 5,
                max_attempts: retries,
                backoff_base_ms: 1,
                backoff_max_ms: 1,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    talos_provider::openai_responses::factory::build_configured_provider(&config, "sk-test")
        .expect("Azure provider")
}

async fn collect_events(mut rx: tokio::sync::mpsc::Receiver<AgentEvent>) -> Vec<AgentEvent> {
    tokio::time::timeout(Duration::from_secs(5), async move {
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    })
    .await
    .expect("Responses stream should finish within fixture timeout")
}

#[tokio::test]
async fn mock_http_two_round_function_call_preserves_call_id() {
    let mut server = mockito::Server::new_async().await;
    let tool = ToolDefinition::new(
        "probe",
        "read-only fixture",
        json!({"type": "object", "properties": {"value": {"type": "integer"}}}),
    );

    let first = server
        .mock("POST", "/openai/v1/responses")
        .match_header("api-key", "sk-test")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_request(|request| {
            let body: Value = serde_json::from_slice(request.body().expect("request body"))
                .expect("Responses request JSON");
            body["model"] == "azure-deployment" && body["store"] == false
                && body["stream"] == true
                && body["tools"][0]["type"] == "function"
                && body["tools"][0]["name"] == "probe"
                && body.get("previous_response_id").is_none()
        })
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(concat!(
            "data: {\"type\":\"response.created\"}\n\n",
            "data: {\"type\":\"response.output_item.added\",\"item\":{\"id\":\"fc_1\",\"type\":\"function_call\",\"call_id\":\"call_1\",\"name\":\"probe\",\"arguments\":\"\"}}\n\n",
            "data: {\"type\":\"response.function_call_arguments.done\",\"item_id\":\"fc_1\",\"arguments\":\"{\\\"value\\\":1}\"}\n\n",
            "data: {\"type\":\"response.output_item.done\",\"item\":{\"id\":\"fc_1\",\"type\":\"function_call\",\"call_id\":\"call_1\",\"name\":\"probe\",\"arguments\":\"{\\\"value\\\":1}\"}}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":4,\"output_tokens\":2}}}\n\n"
        ))
        .expect(1)
        .create_async()
        .await;

    let provider = responses_provider(server.url());
    let user = Message::User {
        content: "probe value".into(),
    };
    let first_events = collect_events(
        provider
            .stream_with_tools(std::slice::from_ref(&user), std::slice::from_ref(&tool))
            .await
            .expect("first Responses round"),
    )
    .await;

    let call = first_events
        .iter()
        .find_map(|event| match event {
            AgentEvent::ToolCall { call, .. } => Some(call.clone()),
            _ => None,
        })
        .expect("native function call event");
    assert_eq!(call.id, "call_1");
    assert_eq!(call.name, "probe");
    assert_eq!(call.input, json!({"value": 1}));
    assert!(first_events.iter().any(|event| matches!(
        event,
        AgentEvent::TurnEnd {
            stop_reason: StopReason::ToolUse,
            ..
        }
    )));
    first.assert_async().await;

    let second = server
        .mock("POST", "/openai/v1/responses")
        .match_header("api-key", "sk-test")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_request(|request| {
            let body: Value = serde_json::from_slice(request.body().expect("request body"))
                .expect("Responses request JSON");
            let Some(input) = body["input"].as_array() else {
                return false;
            };
            let replayed_call = input.iter().any(|item| {
                item["type"] == "function_call"
                    && item["call_id"] == "call_1"
                    && item["name"] == "probe"
            });
            let output = input.iter().any(|item| {
                item["type"] == "function_call_output"
                    && item["call_id"] == "call_1"
                    && item["output"] == "tool-ok"
            });
            replayed_call && output && body.get("previous_response_id").is_none()
        })
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(concat!(
            "data: {\"type\":\"response.created\"}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"done\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":8,\"output_tokens\":1}}}\n\n"
        ))
        .expect(1)
        .create_async()
        .await;

    let messages = vec![
        user,
        Message::Assistant {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: call.id,
                name: call.name,
                input: call.input,
            }],
            reasoning: None,
        },
        Message::Tool {
            result: MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "tool-ok".into(),
                is_error: false,
            },
        },
    ];
    let second_events = collect_events(
        provider
            .stream_with_tools(&messages, &[tool])
            .await
            .expect("second Responses round"),
    )
    .await;
    assert!(second_events.iter().any(|event| matches!(
        event,
        AgentEvent::TextDelta { delta } if delta == "done"
    )));
    assert!(second_events.iter().any(|event| matches!(
        event,
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            ..
        }
    )));
    second.assert_async().await;
}

#[tokio::test]
async fn mock_http_accepts_adr051_multimodal_continuation_without_path_leak() {
    let temp = tempfile::tempdir().expect("temporary image directory");
    let path = temp.path().join("continuation.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 255]))
        .save(&path)
        .expect("fixture png");

    // Durable image parts carry grant-time canonical paths. Exercise an alias on Unix so Linux
    // also covers the precondition exposed by macOS's temporary-directory aliases.
    #[cfg(unix)]
    let path = {
        let alias = temp.path().join("continuation-alias.png");
        std::os::unix::fs::symlink(&path, &alias).expect("fixture image alias");
        alias
    };
    let path = path
        .canonicalize()
        .expect("grant-time canonical image path");

    let mut server = mockito::Server::new_async().await;
    let path_text = path.to_string_lossy().into_owned();
    let mock = server
        .mock("POST", "/openai/v1/responses")
        .match_header("api-key", "sk-test")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_request(move |request| {
            let raw = request.body().expect("request body");
            let body_text = String::from_utf8_lossy(raw);
            if body_text.contains(&path_text)
                || body_text.contains("continuation.png")
                || body_text.contains("continuation-alias.png")
            {
                return false;
            }
            let body: Value = serde_json::from_slice(raw).expect("Responses request JSON");
            body["input"].as_array().is_some_and(|input| {
                input.iter().any(|item| {
                    item["content"].as_array().is_some_and(|content| {
                        content.iter().any(|part| {
                            part["type"] == "input_image"
                                && part["image_url"]
                                    .as_str()
                                    .is_some_and(|url| url.starts_with("data:image/png;base64,"))
                        })
                    })
                })
            })
        })
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(concat!(
            "data: {\"type\":\"response.created\"}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"image received\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":6,\"output_tokens\":2}}}\n\n"
        ))
        .expect(1)
        .create_async()
        .await;

    let provider = responses_provider(server.url());
    let messages = vec![Message::Multimodal {
        parts: vec![
            ContentPart::Text {
                text: "describe this".into(),
            },
            ContentPart::Image {
                path,
                mime: "image/png".into(),
                byte_count: 0,
                content_digest: ContentDigest::default(),
            },
        ],
    }];
    let events = collect_events(
        provider
            .stream(&messages)
            .await
            .expect("Responses multimodal round"),
    )
    .await;
    assert!(events.iter().any(|event| matches!(
        event,
        AgentEvent::TextDelta { delta } if delta == "image received"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            ..
        }
    )));
    mock.assert_async().await;
}

#[tokio::test]
async fn semantic_terminal_closes_stream_without_waiting_for_http_eof() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    for failed in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("fixture listener");
        let address = listener.local_addr().expect("fixture address");
        let terminal = if failed {
            r#"{"type":"response.failed","response":{"error":{"message":"fixture failure"}}}"#
        } else {
            r#"{"type":"response.completed","response":{"usage":{"input_tokens":1,"output_tokens":2}}}"#
        };
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("fixture connection");
            let mut request = [0; 8192];
            let received = socket.read(&mut request).await.expect("fixture request");
            assert!(received > 0, "fixture request must contain bytes");
            let body = format!("data: {terminal}\n\n");
            socket
                .write_all(format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{body}\r\n",
                    body.len(),
                ).as_bytes())
                .await
                .expect("fixture terminal chunk");
            // Deliberately omit the terminating HTTP chunk and keep the socket alive.
            std::future::pending::<()>().await;
        });
        let provider = responses_provider(format!("http://{address}"));
        let mut events = provider
            .stream(&[Message::User {
                content: "test".into(),
            }])
            .await
            .expect("fixture stream");
        let result = tokio::time::timeout(Duration::from_secs(2), async {
            let mut received = Vec::new();
            while let Some(event) = events.recv().await {
                received.push(event);
            }
            received
        })
        .await;
        server.abort();
        let received =
            result.expect("semantic terminal must close before idle timeout or HTTP EOF");
        assert_eq!(received.len(), 1);
        if failed {
            assert!(
                matches!(&received[0], AgentEvent::Error { message } if message == "Azure response stream failed")
            );
        } else {
            assert!(
                matches!(&received[0], AgentEvent::TurnEnd { stop_reason: StopReason::EndTurn, usage } if usage.input_tokens == 1 && usage.output_tokens == 2)
            );
        }
    }
}

#[tokio::test]
async fn tool_image_is_never_automatically_reposted_after_503() {
    use talos_core::provider::EphemeralImage;
    use talos_core::tool::ToolProtocol;
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/openai/v1/responses")
        .match_header("api-key", "sk-test")
        .match_header("authorization", mockito::Matcher::Missing)
        .with_status(503)
        .with_body("unavailable")
        .expect(1)
        .create_async()
        .await;
    let provider = azure_provider(server.url(), 3);
    let image = EphemeralImage::png(
        b"\x89PNG\r\n\x1a\nfixture".to_vec(),
        std::time::Instant::now() + Duration::from_secs(120),
    )
    .expect("image");
    let (progress, _) = tokio::sync::mpsc::unbounded_channel();
    let result = provider
        .stream_with_ephemeral_images(
            &[Message::User {
                content: "fixture".into(),
            }],
            &[],
            ToolProtocol::Native,
            progress,
            &[],
            vec![image],
        )
        .await;
    assert!(matches!(
        result,
        Err(talos_core::provider::ProviderError::ServerError(_))
    ));
    mock.assert_async().await;
}

#[tokio::test]
async fn explicit_text_protocols_dispatch_and_parse_tools_on_responses() {
    use talos_core::tool::ToolProtocol;
    for (protocol, truncated) in [
        (ToolProtocol::Compat, false),
        (ToolProtocol::TalosStrict, false),
        (ToolProtocol::Compat, true),
        (ToolProtocol::TalosStrict, true),
    ] {
        let mut server = mockito::Server::new_async().await;
        let text = "<tool_call>{\"name\":\"probe\",\"args\":{\"value\":1}}</tool_call>";
        let terminal = if truncated {
            json!({"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"},"usage":{"input_tokens":4,"output_tokens":2}}})
        } else {
            json!({"type":"response.completed","response":{"usage":{"input_tokens":4,"output_tokens":2}}})
        };
        let body = format!(
            "data: {}\n\ndata: {}\n\n",
            json!({"type":"response.output_text.delta","delta":text}),
            terminal
        );
        let mock = server
            .mock("POST", "/openai/v1/responses")
            .match_header("api-key", "sk-test")
            .match_header("authorization", mockito::Matcher::Missing)
            .match_request(|r| {
                let body: Value =
                    serde_json::from_slice(r.body().expect("request body")).expect("request JSON");
                body["tools"].as_array().is_none_or(Vec::is_empty)
                    && body["model"] == "azure-deployment"
                    && body["store"] == false
            })
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        let provider = responses_provider(server.url());
        let (progress, _) = tokio::sync::mpsc::unbounded_channel();
        let events = collect_events(
            provider
                .stream_with_protocol(
                    &[Message::User {
                        content: "fixture".into(),
                    }],
                    &[],
                    protocol,
                    progress,
                )
                .await
                .expect("text protocol dispatch"),
        )
        .await;
        assert!(events.iter().any(|event| matches!(event, AgentEvent::ToolCall { call, .. } if call.name == "probe" && call.input == json!({"value":1}))));
        let expected = if truncated {
            StopReason::MaxTokens
        } else {
            StopReason::ToolUse
        };
        assert!(events.iter().any(|event| matches!(event, AgentEvent::TurnEnd { stop_reason, .. } if *stop_reason == expected)));
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn azure_redirect_does_not_relocate_credentials() {
    let mut origin = mockito::Server::new_async().await;
    let mut destination = mockito::Server::new_async().await;
    let leak = destination
        .mock("POST", "/capture")
        .expect(0)
        .create_async()
        .await;
    let redirect = origin
        .mock("POST", "/openai/v1/responses")
        .match_header("api-key", "sk-test")
        .with_status(307)
        .with_header("location", &format!("{}/capture", destination.url()))
        .with_body("sk-test echoed credential")
        .expect(1)
        .create_async()
        .await;
    let error = responses_provider(origin.url())
        .stream(&[Message::User {
            content: "fixture".into(),
        }])
        .await
        .expect_err("redirect refused");
    assert!(!error.to_string().contains("sk-test"));
    redirect.assert_async().await;
    leak.assert_async().await;
}

#[tokio::test]
async fn azure_success_status_error_event_is_redacted() {
    let mut server = mockito::Server::new_async().await;
    let fixture = server
        .mock("POST", "/openai/v1/responses")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: {\"type\":\"error\",\"message\":\"sk-test echoed credential\"}\n\n")
        .expect(1)
        .create_async()
        .await;
    let events = collect_events(
        responses_provider(server.url())
            .stream(&[Message::User {
                content: "fixture".into(),
            }])
            .await
            .expect("HTTP response"),
    )
    .await;
    assert!(events.iter().any(|event| matches!(event, AgentEvent::Error { message } if message == "Azure response stream failed")));
    assert!(!format!("{events:?}").contains("sk-test"));
    fixture.assert_async().await;
}
