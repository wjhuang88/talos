//! Controlled real Agent image/read/signature rounds for both credential profiles.
use crate::{gemini_provider::GeminiProvider, gemini_transport::GeminiTransport};
use base64::Engine;
use serde_json::json;
use sha2::Digest;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use talos_core::message::{ContentDigest, ContentPart, Message};
use talos_core::tool::ToolRegistry;

#[tokio::test]
async fn studio_real_agent_round_preserves_images_tools_and_signed_history() {
    run(false).await;
}
#[tokio::test]
async fn vertex_real_agent_round_refreshes_explicit_adc_and_sends_only_bearer() {
    run(true).await;
}

async fn run(vertex: bool) {
    let temp = tempfile::tempdir().expect("workspace");
    let workspace = temp.path().canonicalize().expect("canonical");
    std::fs::write(workspace.join("fixture.txt"), "GEMINI_READ_OK\n").expect("text");
    let mut parts = vec![ContentPart::Text {
        text: "Compare two images".into(),
    }];
    let mut encoded = Vec::new();
    for (index, color) in [[20, 40, 60, 255], [90, 120, 150, 255]]
        .into_iter()
        .enumerate()
    {
        let path = workspace.join(format!("image-{index}.png"));
        image::RgbaImage::from_pixel(1, 1, image::Rgba(color))
            .save(&path)
            .expect("image");
        let bytes = std::fs::read(&path).expect("bytes");
        encoded.push(base64::engine::general_purpose::STANDARD.encode(&bytes));
        parts.push(ContentPart::Image {
            path,
            mime: "image/png".into(),
            byte_count: bytes.len() as u64,
            content_digest: ContentDigest::from_raw(sha2::Sha256::digest(&bytes).into()),
        });
    }
    let mut server = mockito::Server::new_async().await;
    let mut token_server = mockito::Server::new_async().await;
    let token = token_server
        .mock("POST", "/")
        .match_body(mockito::Matcher::PartialJson(
            json!({"grant_type":"refresh_token","client_id":"fixture-client"}),
        ))
        .with_body(r#"{"access_token":"fixture-access","token_type":"Bearer","expires_in":3600}"#)
        .expect(if vertex { 1 } else { 0 })
        .create_async()
        .await;
    let mut transport =
        GeminiTransport::ai_studio("model", "fixture-api-key", true).expect("transport");
    if vertex {
        let path = workspace.join("adc.json");
        std::fs::write(
            &path,
            serde_json::to_vec(
                &json!({"type":"authorized_user","client_id":"fixture-client",
            "client_secret":"fixture-client-secret","refresh_token":"fixture-refresh"}),
            )
            .expect("JSON"),
        )
        .expect("ADC");
        let source = crate::gemini_auth::VertexTokenSource::from_adc(
            Some(&path),
            &workspace.join("absent.json"),
            false,
            Instant::now() + Duration::from_secs(3),
        )
        .await
        .expect("explicit ADC")
        .with_test_endpoint(token_server.url());
        transport = transport.with_test_vertex_source(source);
    }
    let transport = transport.with_test_endpoint(server.url());
    let call_part = json!({"functionCall":{"name":"read","args":{"path":"fixture.txt"}},"thoughtSignature":"fixture-private-signature"});
    let first_wire = format!(
        "data: {}\n\n",
        json!({"candidates":[{"content":{"parts":[call_part.clone()]},"finishReason":"STOP"}]})
    );
    let expected_images = encoded
        .iter()
        .map(|data| json!({"mimeType":"image/png","data":data}))
        .collect::<Vec<_>>();
    let mut first = server
        .mock("POST", "/")
        .match_request(move |request| {
            let bytes = request.body().expect("body");
            let body: serde_json::Value = serde_json::from_slice(bytes).expect("JSON");
            let images = body["contents"]
                .as_array()
                .expect("contents")
                .iter()
                .flat_map(|content| content["parts"].as_array().expect("parts"))
                .filter_map(|part| part.get("inlineData").cloned())
                .collect::<Vec<_>>();
            !String::from_utf8_lossy(bytes).contains("functionResponse")
                && images == expected_images
        })
        .match_body(mockito::Matcher::PartialJson(
            json!({"contents":[{"role":"user","parts":[
            {"text":"Compare two images"},{"inlineData":{"mimeType":"image/png","data":encoded[0]}},
            {"inlineData":{"mimeType":"image/png","data":encoded[1]}}]}]}),
        ))
        .with_body(first_wire)
        .expect(1);
    let mut second = server
        .mock("POST", "/")
        .match_request(move |request| {
            let body: serde_json::Value =
                serde_json::from_slice(request.body().expect("body")).expect("JSON");
            let contents = body["contents"].as_array().expect("contents");
            let signed = contents.iter().any(|entry| {
                entry["role"] == "model" && entry["parts"] == json!([call_part.clone()])
            });
            let result = contents.iter().any(|entry| {
                entry["parts"].as_array().is_some_and(|parts| {
                    parts.iter().any(|part| {
                        let response = &part["functionResponse"];
                        response["name"] == "read"
                            && response.get("id").is_none()
                            && response["response"]["output"]
                                .as_str()
                                .is_some_and(|text| text.contains("GEMINI_READ_OK"))
                    })
                })
            });
            signed && result
        })
        .with_body(format!(
            "data: {}\n\n",
            json!({"candidates":[{"content":{"parts":[{"text":"ok"}]},"finishReason":"STOP"}]})
        ))
        .expect(1);
    if vertex {
        first = first
            .match_header("authorization", "Bearer fixture-access")
            .match_header("x-goog-api-key", mockito::Matcher::Missing);
        second = second
            .match_header("authorization", "Bearer fixture-access")
            .match_header("x-goog-api-key", mockito::Matcher::Missing);
    } else {
        first = first
            .match_header("x-goog-api-key", "fixture-api-key")
            .match_header("authorization", mockito::Matcher::Missing);
        second = second
            .match_header("x-goog-api-key", "fixture-api-key")
            .match_header("authorization", mockito::Matcher::Missing);
    }
    let first = first.create_async().await;
    let second = second.create_async().await;
    let provider = GeminiProvider::new(
        transport,
        "fixture-google".into(),
        "model".into(),
        Some(1024),
        Default::default(),
    )
    .expect("provider");
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(talos_tools::file_tools::ReadTool::new(
        workspace.clone(),
    )));
    let engine = talos_permission::PermissionEngine::with_workspace_root(workspace.clone());
    let agent = talos_agent::Agent::with_security(
        Arc::new(provider),
        tools,
        Some(Arc::new(engine)),
        None,
        workspace,
    )
    .with_image_input_supported(true)
    .with_reasoning_identity(Some("fixture-google".into()), Some("model".into()), true);
    let (events, _receiver) = tokio::sync::mpsc::unbounded_channel();
    let (answer, history) = tokio::time::timeout(
        Duration::from_secs(20),
        agent.run_streaming(
            "Read fixture.txt".into(),
            vec![Message::Multimodal { parts }],
            events,
        ),
    )
    .await
    .expect("bounded round")
    .expect("Agent round");
    assert_eq!(answer, "ok");
    assert!(history.iter().any(|message|matches!(message,Message::Tool{result} if !result.is_error && result.content.contains("GEMINI_READ_OK"))));
    assert!(!format!("{history:?}").contains("fixture-private-signature"));
    let durable = serde_json::to_string(&history).expect("history");
    for credential in [
        "fixture-api-key",
        "fixture-access",
        "fixture-client-secret",
        "fixture-refresh",
    ] {
        assert!(!durable.contains(credential));
    }
    first.assert_async().await;
    second.assert_async().await;
    token.assert_async().await;
}

#[tokio::test]
async fn textual_modes_complete_real_permission_gated_agent_with_signed_continuation() {
    use talos_core::tool::ToolProtocol;
    for protocol in [ToolProtocol::Compat, ToolProtocol::TalosStrict] {
        let temp = tempfile::tempdir().expect("workspace");
        let workspace = temp.path().canonicalize().expect("canonical");
        std::fs::write(workspace.join("note"), "GEMINI_COMPAT_OK").expect("note");
        let text = r#"<tool_call>{"name":"read","id":"r","args":{"path":"note"}}</tool_call>"#;
        let signed = json!({"text":text,"thoughtSignature":"compat-private-signature"});
        let mut server = mockito::Server::new_async().await;
        let first = server
            .mock("POST", "/")
            .match_header("x-goog-api-key", "fixture-key")
            .match_request(|request| {
                let body: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body.get("tools").is_none() && !body.to_string().contains("<tool_result")
            })
            .with_body(format!(
                "data: {}\n\n",
                json!({"candidates":[{"content":{"parts":[signed.clone()]},"finishReason":"STOP"}]})
            ))
            .expect(1)
            .create_async()
            .await;
        let second = server.mock("POST", "/").match_header("x-goog-api-key", "fixture-key")
            .match_request(move |request| {
                let body: serde_json::Value = serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body.get("tools").is_none() && body["contents"].as_array().expect("contents").iter().any(|item| item["role"] == "model" && item["parts"] == json!([signed]))
                    && body.to_string().contains("GEMINI_COMPAT_OK") && !body.to_string().contains("functionResponse")
            })
            .with_body(format!("data: {}\n\n",json!({"candidates":[{"content":{"parts":[{"text":"done"}]},"finishReason":"STOP"}]})))
            .expect(1).create_async().await;
        let provider = GeminiProvider::new(
            GeminiTransport::ai_studio("model", "fixture-key", true)
                .expect("transport")
                .with_test_endpoint(server.url()),
            "google".into(),
            "model".into(),
            Some(512),
            Default::default(),
        )
        .expect("provider");
        let mut tools = ToolRegistry::new();
        tools.register(Arc::new(talos_tools::file_tools::ReadTool::new(
            workspace.clone(),
        )));
        let mut agent = talos_agent::Agent::with_security(
            Arc::new(provider),
            tools,
            Some(Arc::new(
                talos_permission::PermissionEngine::with_workspace_root(workspace.clone()),
            )),
            None,
            workspace,
        )
        .with_reasoning_identity(Some("google".into()), Some("model".into()), true);
        agent.set_tool_protocol(protocol);
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let (answer, history) = tokio::time::timeout(
            Duration::from_secs(10),
            agent.run_streaming("Read note".into(), vec![], tx),
        )
        .await
        .expect("bounded")
        .expect("round");
        assert_eq!(answer, "done");
        assert!(history.iter().any(|item| matches!(item, Message::Tool {result} if !result.is_error && result.content.contains("GEMINI_COMPAT_OK"))));
        first.assert_async().await;
        second.assert_async().await;
    }
}
