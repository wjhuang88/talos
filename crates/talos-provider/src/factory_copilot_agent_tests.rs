//! Real native Agent image/read round using the private controlled factory composition.

use super::*;
use serde_json::json;
use sha2::Digest;
use talos_core::message::{ContentDigest, ContentPart};
use talos_core::tool::ToolRegistry;

#[tokio::test]
async fn copilot_messages_image_and_native_read_complete_through_real_agent() {
    run_copilot_agent_round(false).await;
}

#[tokio::test]
async fn catalog_copilot_messages_image_and_native_read_complete_through_real_agent() {
    run_copilot_agent_round(true).await;
}

async fn run_copilot_agent_round(catalog: bool) {
    let temp = tempfile::tempdir().expect("workspace");
    let workspace = temp.path().canonicalize().expect("canonical workspace");
    std::fs::write(workspace.join("fixture.txt"), "COPILOT_READ_OK\n").expect("read fixture");
    let image_path = workspace.join("fixture.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([20, 40, 60, 255]))
        .save(&image_path)
        .expect("image fixture");
    let bytes = std::fs::read(&image_path).expect("image bytes");
    let history = vec![Message::Multimodal {
        parts: vec![
            ContentPart::Text {
                text: "Image context".into(),
            },
            ContentPart::Image {
                path: image_path,
                mime: "image/png".into(),
                byte_count: u64::try_from(bytes.len()).expect("image length"),
                content_digest: ContentDigest::from_raw(sha2::Sha256::digest(&bytes).into()),
            },
        ],
    }];
    let mut server = Server::new_async().await;
    let protocol = ProviderProtocol::AnthropicMessages;
    let provider = controlled(
        &config(
            if catalog {
                ProviderProtocol::Unspecified
            } else {
                protocol.clone()
            },
            "https://api.githubcopilot.com",
        ),
        &server,
    );
    let preview = provider.request_preview(&[]).expect("preview");
    let interaction = preview["headers"]["X-Interaction-Id"]
        .as_str()
        .expect("identity");
    let tool_wire: String = [
        ("message_start", json!({"message":{"usage":{"input_tokens":2}}})),
        ("content_block_start", json!({"index":0,"content_block":{"type":"tool_use","id":"call_copilot_read","name":"read","input":{}}})),
        ("content_block_delta", json!({"index":0,"delta":{"type":"input_json_delta","partial_json":"{\"path\":\"fixture.txt\"}"}})),
        ("message_delta", json!({"delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":2}})),
        ("message_stop", json!({})),
    ].into_iter().map(|(event,data)|format!("event: {event}\ndata: {data}\n\n")).collect();
    let first = server
        .mock("POST", "/v1/messages")
        .match_header("authorization", format!("Bearer {KEY}").as_str())
        .match_header("x-api-key", Matcher::Missing)
        .match_header("copilot-integration-id", "talos")
        .match_header("x-interaction-id", interaction)
        .match_header("anthropic-version", "2023-06-01")
        .match_body(Matcher::AllOf(vec![
            Matcher::PartialJson(json!({"model":"claude-sonnet-4.6","tools":[{"name":"read"}]})),
            Matcher::Regex(r#""type"\s*:\s*"image""#.into()),
        ]))
        .match_request(|request| {
            !request
                .body()
                .expect("request body")
                .windows(b"tool_result".len())
                .any(|s| s == b"tool_result")
        })
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(tool_wire)
        .expect(1)
        .create_async()
        .await;
    let second = server
        .mock("POST", "/v1/messages")
        .match_header("authorization", format!("Bearer {KEY}").as_str())
        .match_header("x-api-key", Matcher::Missing)
        .match_header("copilot-integration-id", "talos")
        .match_header("x-interaction-id", interaction)
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex("tool_result".into()),
            Matcher::Regex("call_copilot_read".into()),
            Matcher::Regex("COPILOT_READ_OK".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(success_wire(&protocol))
        .expect(1)
        .create_async()
        .await;
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(talos_tools::file_tools::ReadTool::new(
        workspace.clone(),
    )));
    let engine = talos_permission::PermissionEngine::with_workspace_root(workspace.clone());
    let agent =
        talos_agent::Agent::with_security(provider, tools, Some(Arc::new(engine)), None, workspace)
            .with_image_input_supported(true);
    let (events, _receiver) = tokio::sync::mpsc::unbounded_channel();
    let (answer, messages) = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        agent.run_streaming("Read fixture.txt".into(), history, events),
    )
    .await
    .expect("bounded Agent round")
    .expect("native Agent round");
    assert_eq!(answer, "ok");
    assert!(messages.iter().any(|message|matches!(message, Message::Tool {result}
        if result.tool_use_id=="call_copilot_read" && !result.is_error && result.content.contains("COPILOT_READ_OK"))));
    assert!(
        !serde_json::to_string(&messages)
            .expect("messages")
            .contains(KEY)
    );
    first.assert_async().await;
    second.assert_async().await;
}
