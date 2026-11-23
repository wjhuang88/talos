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
    if native || text.starts_with("<tool_call>") {
        for delta in [
            json!({"text":"private thought"}),
            json!({"signature":" sig "}),
        ] {
            add(
                "contentBlockDelta",
                json!({"contentBlockIndex":1,"delta":{"reasoningContent":delta}}),
            );
        }
        add("contentBlockStop", json!({"contentBlockIndex":1}));
    }
    if native {
        add(
            "contentBlockStart",
            json!({"contentBlockIndex":0,"start":{"toolUse":{"toolUseId":"r","name":"read_image"}}}),
        );
        add(
            "contentBlockDelta",
            json!({"contentBlockIndex":0,"delta":{"toolUse":{"input":r#"{"path":"tool.png"}"#}}}),
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
    let tool_image = root.join("tool.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([0u8, 255, 0, 255]))
        .save(&tool_image)
        .unwrap();
    let tool_bytes =
        base64::engine::general_purpose::STANDARD.encode(std::fs::read(tool_image).unwrap());
    let first_image = tool_bytes.clone();
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
                && !body.to_string().contains(&first_image)
        })
        .with_header("content-type", "application/vnd.amazon.eventstream")
        .with_body(response(
            native,
            r#"<tool_call>{"name":"read_image","id":"r","args":{"path":"tool.png"}}</tool_call>"#,
        ))
        .create_async()
        .await;
    let second = server
        .mock("POST", path)
        .match_request(move |r| {
            let body: Value = serde_json::from_slice(r.body().unwrap()).unwrap();
            body["messages"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|m| m["content"].as_array().into_iter().flatten())
                .filter(|block| {
                    block["image"]["source"]["bytes"].as_str() == Some(tool_bytes.as_str())
                })
                .count()
                == 1
                && body
                    .to_string()
                    .contains("attached to next provider request")
                && body["messages"].as_array().unwrap().iter().any(|m| {
                    m["content"][1]["reasoningContent"]["reasoningText"]
                        == json!({"text":"private thought","signature":" sig "})
                })
        })
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
    registry.register(Arc::new(talos_tools::read_image_tool::ReadImageTool::new(
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
    agent = agent
        .with_image_input_supported(true)
        .with_reasoning_identity(Some("fixture".into()), Some("example".into()), true);
    agent.set_tool_protocol(protocol);
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let (answer, _) = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        agent.run_streaming(
            "Inspect tool.png".into(),
            vec![History::Multimodal { parts }],
            tx,
        ),
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
            BedrockProvider::new(
                "bedrock",
                "example",
                "us-east-1".into(),
                None,
                true,
                Default::default(),
            )
            .unwrap()
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
    let provider = BedrockProvider::new(
        "bedrock",
        "example",
        "us-east-1".into(),
        None,
        true,
        Default::default(),
    )
    .unwrap();
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

#[tokio::test]
async fn signed_history_requires_live_custody_and_enabled_replay_before_http() {
    use crate::bedrock_provider::BedrockProvider;
    use talos_core::{
        message::{AssistantReasoning, Message as History, ReasoningBlock},
        provider::{LanguageModel, TransientReplay},
    };
    let public = History::Assistant {
        content: "visible".into(),
        tool_calls: vec![],
        reasoning: None,
    };
    let mut private = public.clone();
    if let History::Assistant { reasoning, .. } = &mut private {
        *reasoning = Some(AssistantReasoning {
            provider: "p".into(),
            model: "m".into(),
            blocks: vec![ReasoningBlock::Redacted {
                data: "private-signature".into(),
            }],
        });
    }
    let attachment = Arc::new(TransientReplay::new(public, private).unwrap());
    let history = [attachment.projected_message().clone()];
    assert!(
        !serde_json::to_string(&history)
            .unwrap()
            .contains("private-signature")
    );
    for (enabled, custody) in [(false, vec![attachment]), (true, vec![])] {
        let provider = BedrockProvider::new(
            "p",
            "m",
            "us-east-1".into(),
            None,
            enabled,
            Default::default(),
        )
        .unwrap();
        let (progress, _) = tokio::sync::mpsc::unbounded_channel();
        let result = provider
            .stream_with_replay_context(
                &history,
                &[],
                ToolProtocol::Native,
                progress,
                &[],
                vec![],
                custody,
            )
            .await;
        let Err(error) = result else {
            panic!("accepted unavailable signed history")
        };
        assert!(error.to_string().contains(if enabled {
            "private replay context"
        } else {
            "replay disabled"
        }));
    }
}

#[tokio::test]
async fn ephemeral_image_retries_before_stream_and_reports_progress() {
    use super::bedrock_provider::{BedrockProvider, FIXTURE};
    use std::time::{Duration, Instant};
    use talos_core::{
        message::Message as History,
        provider::{EphemeralImage, LanguageModel, ProviderProgress},
    };
    let mut server = mockito::Server::new_async().await;
    let path = "/model/example/converse-stream";
    let first = server
        .mock("POST", path)
        .with_status(503)
        .expect(1)
        .create_async()
        .await;
    let second = server.mock("POST",path).match_request(|request| {
        let body:Value = serde_json::from_slice(request.body().expect("request bytes")).expect("JSON request");
        body["messages"][0]["content"] == json!([{"text":"observe"},{"image":{"format":"png","source":{"bytes":"iVBORw0KGgo="}}}])
    }).with_body(response(false,"done")).with_header("content-type","application/vnd.amazon.eventstream").expect(1).create_async().await;
    let provider = FIXTURE
        .scope(server.url().parse().expect("fixture URL"), async {
            BedrockProvider::new(
                "p",
                "example",
                "us-east-1".into(),
                None,
                true,
                talos_config::ProviderTimeoutConfig {
                    backoff_base_ms: 0,
                    backoff_max_ms: 0,
                    ..Default::default()
                },
            )
            .expect("provider")
        })
        .await;
    let image = EphemeralImage::png(
        b"\x89PNG\r\n\x1a\n".to_vec(),
        Instant::now() + Duration::from_secs(10),
    )
    .expect("image");
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut stream = provider
        .stream_with_ephemeral_images(
            &[History::User {
                content: "observe".into(),
            }],
            &[],
            ToolProtocol::Native,
            tx,
            &[],
            vec![image],
        )
        .await
        .expect("successful retry");
    while stream.recv().await.is_some() {}
    assert!(matches!(
        rx.recv().await,
        Some(ProviderProgress::InitialDispatch { attempt: 0, .. })
    ));
    assert!(matches!(
        rx.recv().await,
        Some(ProviderProgress::ScheduledBackoff { attempt: 1, .. })
    ));
    assert!(matches!(
        rx.recv().await,
        Some(ProviderProgress::RetryDispatch { attempt: 1, .. })
    ));
    assert!(matches!(
        rx.recv().await,
        Some(ProviderProgress::FirstPacketWait { attempt: 1, .. })
    ));
    first.assert_async().await;
    second.assert_async().await;
}
