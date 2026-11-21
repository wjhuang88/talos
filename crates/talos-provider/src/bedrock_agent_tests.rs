//! Public factory and permission-gated Agent acceptance against signed local HTTP.
use aws_smithy_eventstream::frame::write_message_to;
use aws_smithy_types::event_stream::{Header, HeaderValue, Message};
use serde_json::{Value, json};
use std::sync::Arc;
use talos_core::tool::{ToolProtocol, ToolRegistry};

fn response(native: bool, text: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut add = |name: &'static str, body: Value| {
        let mut frame = Message::new(serde_json::to_vec(&body).unwrap());
        for (k, v) in [
            (":message-type", "event"),
            (":event-type", name),
            (":content-type", "application/json"),
        ] {
            frame = frame.add_header(Header::new(k, HeaderValue::String(v.into())));
        }
        write_message_to(&frame, &mut bytes).unwrap();
    };
    add("messageStart", json!({"role":"assistant"}));
    if native {
        add(
            "contentBlockStart",
            json!({"contentBlockIndex":0,"start":{"toolUse":{"toolUseId":"r","name":"read"}}}),
        );
        add(
            "contentBlockDelta",
            json!({"contentBlockIndex":0,"delta":{"toolUse":{"input":r#"{"path":"note"}"#}}}),
        );
    } else {
        add(
            "contentBlockDelta",
            json!({"contentBlockIndex":0,"delta":{"text":text}}),
        );
    }
    add("contentBlockStop", json!({"contentBlockIndex":0}));
    add(
        "messageStop",
        json!({"stopReason":if native {"tool_use"} else {"end_turn"}}),
    );
    add(
        "metadata",
        json!({"usage":{"inputTokens":7,"outputTokens":3}}),
    );
    bytes
}

#[test]
fn bedrock_factory_agent_rounds() {
    // Scope region and the test marker to a child process, including parallel test runs.
    if std::env::var_os("TALOS_TEST_BEDROCK_AGENT_CHILD").is_none() {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "bedrock_agent_tests::bedrock_factory_agent_rounds",
                "--nocapture",
            ])
            .env("AWS_REGION", "us-east-1")
            .env("TALOS_TEST_BEDROCK_AGENT_CHILD", "1")
            .status()
            .unwrap();
        assert!(result.success());
        return;
    }
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for protocol in [
            ToolProtocol::Native,
            ToolProtocol::Compat,
            ToolProtocol::TalosStrict,
        ] {
            round(protocol).await;
        }
    });
}

async fn round(protocol: ToolProtocol) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note"), "BEDROCK_READ_OK").unwrap();
    let mut server = mockito::Server::new_async().await;
    let native = protocol == ToolProtocol::Native;
    use base64::Engine;
    use sha2::Digest;
    use talos_core::message::{ContentDigest, ContentPart, Message as History};
    let mut parts = Vec::new();
    let mut expected = Vec::new();
    for (label, color) in [("first", [255, 0, 0, 255]), ("second", [0, 0, 255, 255])] {
        let image_path = root.join(format!("{label}.png"));
        image::RgbaImage::from_pixel(1, 1, image::Rgba(color))
            .save(&image_path)
            .unwrap();
        let bytes = std::fs::read(&image_path).unwrap();
        parts.push(ContentPart::Text { text: label.into() });
        parts.push(ContentPart::Image {
            path: image_path,
            mime: "image/png".into(),
            byte_count: bytes.len() as u64,
            content_digest: ContentDigest::from_raw(sha2::Sha256::digest(&bytes).into()),
        });
        expected.push(json!({"text":label}));
        expected.push(json!({"image":{"format":"png","source":{"bytes":base64::engine::general_purpose::STANDARD.encode(bytes)}}}));
    }
    let path = "/model/example/converse-stream";
    let first = server
        .mock("POST", path)
        .match_header("x-amz-security-token", "fixture-token")
        .match_request(move |r| {
            let body: Value = serde_json::from_slice(r.body().unwrap()).unwrap();
            let ordered = body["messages"].as_array().unwrap().iter().any(|m| {
                m["content"].as_array().is_some_and(|blocks| {
                    blocks
                        .windows(expected.len())
                        .any(|window| window == expected.as_slice())
                })
            });
            ordered
                && body.get("toolConfig").is_some() == native
                && !body.to_string().contains("BEDROCK_READ_OK")
        })
        .with_header("content-type", "application/vnd.amazon.eventstream")
        .with_body(response(
            native,
            r#"<tool_call>{"name":"read","id":"r","args":{"path":"note"}}</tool_call>"#,
        ))
        .create_async()
        .await;
    let second = server
        .mock("POST", path)
        .match_request(|r| String::from_utf8_lossy(r.body().unwrap()).contains("BEDROCK_READ_OK"))
        .with_header("content-type", "application/vnd.amazon.eventstream")
        .with_body(response(false, "done"))
        .create_async()
        .await;
    let mut config = talos_config::Config {
        provider: "fixture".into(),
        model: "example".into(),
        ..Default::default()
    };
    config.providers.insert(
        "fixture".into(),
        talos_config::ProviderConfig {
            protocol: talos_config::ProviderProtocol::AmazonBedrock,
            ..Default::default()
        },
    );
    let provider = crate::bedrock_provider::FIXTURE
        .scope(server.url().parse().unwrap(), async {
            crate::openai_responses::factory::build_configured_provider(
                &config,
                &config.api_key().unwrap(),
            )
            .unwrap()
        })
        .await;
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(talos_tools::file_tools::ReadTool::new(
        root.clone(),
    )));
    let permission = talos_permission::PermissionEngine::with_workspace_root(root.clone());
    let mut agent = talos_agent::Agent::with_security(
        provider,
        registry,
        Some(Arc::new(permission)),
        None,
        root,
    );
    agent = agent.with_image_input_supported(true);
    agent.set_tool_protocol(protocol);
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let (answer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        agent.run_streaming("Read note".into(), vec![History::Multimodal { parts }], tx),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(answer, "done");
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn dropping_receiver_closes_incomplete_http_response() {
    use crate::bedrock_provider::{BedrockProvider, FIXTURE};
    use std::time::Duration;
    use talos_core::{message::Message as History, provider::LanguageModel};
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut socket = BufReader::new(socket);
        let mut length = None;
        loop {
            let mut line = String::new();
            assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
            if line == "\r\n" {
                break;
            }
            if let Some((key, value)) = line.split_once(':')
                && key.eq_ignore_ascii_case("content-length")
            {
                length = Some(value.trim().parse::<usize>().unwrap());
            }
        }
        socket
            .read_exact(&mut vec![0; length.unwrap()])
            .await
            .unwrap();
        socket.get_mut().write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.amazon.eventstream\r\nContent-Length: 1024\r\n\r\n\x00").await.unwrap();
        // Hold a partial frame open until the client's cancellation reaches the wire.
        match socket.read(&mut [0]).await {
            Ok(0) => {}
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {}
            other => panic!("expected client disconnect, got {other:?}"),
        }
    });
    let provider = FIXTURE
        .scope(endpoint, async {
            BedrockProvider::new("example", "us-east-1".into(), None, Default::default()).unwrap()
        })
        .await;
    let receiver = provider
        .stream(&[History::User {
            content: "hello".into(),
        }])
        .await
        .unwrap();
    drop(receiver);
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn textual_modes_reject_orphan_results_before_dispatch() {
    use crate::bedrock_provider::BedrockProvider;
    use talos_core::{
        message::{Message as History, MessageToolResult},
        provider::LanguageModel,
    };
    let provider =
        BedrockProvider::new("example", "us-east-1".into(), None, Default::default()).unwrap();
    let history = [History::Tool {
        result: MessageToolResult {
            tool_use_id: "orphan".into(),
            content: "unused".into(),
            is_error: false,
        },
    }];
    for mode in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
        let (progress, _) = tokio::sync::mpsc::unbounded_channel();
        let error = provider
            .stream_with_protocol(&history, &[], mode, progress)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("no matching call"));
    }
}
