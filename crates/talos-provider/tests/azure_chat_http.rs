use serde_json::json;
use talos_config::{
    AzureApiMode, AzureAuthentication, AzureBodyShape, AzureTransportConfig, Config, ModelConfig,
    ProviderConfig, ProviderProtocol,
};
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::ToolDefinition;
use talos_core::tool::ToolProtocol;

#[tokio::test]
async fn azure_chat_modes_preserve_tool_protocol_and_deployment_identity() {
    for api in [
        AzureApiMode::V1,
        AzureApiMode::Deployment {
            api_version: "2024-10-21".into(),
        },
    ] {
        for protocol in [
            ToolProtocol::Native,
            ToolProtocol::Compat,
            ToolProtocol::TalosStrict,
        ] {
            let mut server = mockito::Server::new_async().await;
            let legacy = matches!(api, AzureApiMode::Deployment { .. });
            let path = if legacy {
                "/openai/deployments/deployment/chat/completions"
            } else {
                "/openai/v1/chat/completions"
            };
            let fixture = server.mock("POST", path)
                .match_query(if legacy { mockito::Matcher::UrlEncoded("api-version".into(), "2024-10-21".into()) } else { mockito::Matcher::Missing })
                .match_header("api-key", "fixture-key")
                .match_header("authorization", mockito::Matcher::Missing)
                .match_request(move |request| {
                    let body: serde_json::Value = serde_json::from_slice(request.body().expect("body")).expect("JSON");
                    body["model"] == "deployment" && body["stream"] == true && if protocol == ToolProtocol::Native { body["tools"][0]["function"]["name"] == "probe" } else { body.get("tools").is_none() }
                })
                .with_status(200).with_header("content-type", "text/event-stream")
                .with_body("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n")
                .expect(1).create_async().await;
            let mut config = Config::default();
            config.provider = "azure-fixture".into();
            config.model = "logical-model".into();
            config.providers.insert(
                config.provider.clone(),
                ProviderConfig {
                    protocol: ProviderProtocol::AzureOpenAI,
                    azure: Some(AzureTransportConfig {
                        endpoint: server.url(),
                        api: api.clone(),
                        shape: Some(AzureBodyShape::Chat),
                        authentication: AzureAuthentication::ApiKey,
                    }),
                    models: [(
                        config.model.clone(),
                        ModelConfig {
                            azure_deployment: Some("deployment".into()),
                            ..Default::default()
                        },
                    )]
                    .into(),
                    timeout: talos_config::ProviderTimeoutConfig {
                        max_attempts: 0,
                        dispatch_timeout_secs: 5,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            let provider = talos_provider::openai_responses::factory::build_configured_provider(
                &config,
                "fixture-key",
            )
            .expect("factory");
            let (tx, _) = tokio::sync::mpsc::unbounded_channel();
            let mut events = provider
                .stream_with_protocol(
                    &[Message::User {
                        content: "probe".into(),
                    }],
                    &[ToolDefinition::new(
                        "probe",
                        "fixture",
                        json!({"type":"object"}),
                    )],
                    protocol,
                    tx,
                )
                .await
                .expect("dispatch");
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                let mut ended = false;
                while let Some(event) = events.recv().await {
                    assert!(!matches!(event, AgentEvent::Error { .. }));
                    ended |= matches!(
                        event,
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            ..
                        }
                    );
                }
                assert!(ended);
            })
            .await
            .expect("bounded stream");
            fixture.assert_async().await;
        }
    }
}

#[tokio::test]
async fn azure_chat_agent_executes_read_in_all_transport_and_tool_modes() {
    use base64::Engine as _;
    use sha2::Digest as _;
    use std::{sync::Arc, time::Duration};
    use talos_core::message::{ContentDigest, ContentPart};
    use talos_core::tool::ToolRegistry;
    for api in [
        AzureApiMode::V1,
        AzureApiMode::Deployment {
            api_version: "2024-10-21".into(),
        },
    ] {
        for protocol in [
            ToolProtocol::Native,
            ToolProtocol::Compat,
            ToolProtocol::TalosStrict,
        ] {
            let directory = tempfile::tempdir().expect("workspace");
            let workspace = directory.path().canonicalize().expect("canonical");
            std::fs::write(workspace.join("fixture.txt"), "AZURE_CHAT_READ_OK").expect("fixture");
            let mut parts = Vec::new();
            let mut urls = Vec::new();
            for (index, color) in [[255, 0, 0, 255], [0, 0, 255, 255]].into_iter().enumerate() {
                let path = workspace.join(format!("private-image-{index}.png"));
                image::RgbaImage::from_pixel(1, 1, image::Rgba(color))
                    .save(&path)
                    .expect("PNG");
                let bytes = std::fs::read(&path).expect("bytes");
                urls.push(format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                ));
                parts.push(ContentPart::Image {
                    path,
                    mime: "image/png".into(),
                    byte_count: bytes.len() as u64,
                    content_digest: ContentDigest::from_raw(sha2::Sha256::digest(&bytes).into()),
                });
            }
            let mut server = mockito::Server::new_async().await;
            let legacy = matches!(api, AzureApiMode::Deployment { .. });
            let path = if legacy {
                "/openai/deployments/deployment/chat/completions"
            } else {
                "/openai/v1/chat/completions"
            };
            let delta = if protocol == ToolProtocol::Native {
                json!({"tool_calls":[{"index":0,"id":"call_read","type":"function","function":{"name":"read","arguments":"{\"path\":\"fixture.txt\"}"}}]})
            } else {
                json!({"content":"<tool_call>{\"name\":\"read\",\"args\":{\"path\":\"fixture.txt\"}}</tool_call>"})
            };
            let mut fixtures = Vec::new();
            for second in [false, true] {
                let expected_images = urls.clone();
                let mut response = if second {
                    json!({"content":"done"})
                } else {
                    delta.clone()
                };
                if !second {
                    response["reasoning_content"] = json!("azure-reasoning-fixture");
                }
                let finish = if !second && protocol == ToolProtocol::Native {
                    "tool_calls"
                } else {
                    "stop"
                };
                fixtures.push(server.mock("POST", path)
                    .match_query(if legacy { mockito::Matcher::UrlEncoded("api-version".into(), "2024-10-21".into()) } else { mockito::Matcher::Missing })
                    .match_header("api-key", "fixture-key")
                    .match_header("authorization", mockito::Matcher::Missing)
                    .match_request(move |request| {
                        let raw = String::from_utf8_lossy(request.body().expect("body"));
                        let body: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
                        let images: Vec<&str> = body["messages"].as_array().expect("messages").iter()
                            .filter_map(|message| message["content"].as_array()).flatten()
                            .filter(|part| part["type"] == "image_url")
                            .filter_map(|part| part["image_url"]["url"].as_str()).collect();
                        images == expected_images && !raw.contains("private-image-")
                            && body["reasoning_effort"] == "high" && body["model"] == "deployment" && raw.contains("AZURE_CHAT_READ_OK") == second
                            && (!second || protocol != ToolProtocol::Native || body["messages"].as_array().expect("messages").iter().any(|m| m["role"] == "tool" && m["tool_call_id"] == "call_read"))
                    })
                    .with_status(200).with_header("content-type", "text/event-stream")
                    .with_body(format!("data: {}\n\ndata: {}\n\ndata: [DONE]\n\n", json!({"choices":[{"delta":response,"finish_reason":null}]}), json!({"choices":[{"delta":{},"finish_reason":finish}],"usage":{"prompt_tokens":4,"completion_tokens":2}})))
                    .expect(1).create_async().await);
            }
            let mut config = Config::default();
            config.provider = "azure-fixture".into();
            config.model = "logical-model".into();
            config.providers.insert(
                config.provider.clone(),
                ProviderConfig {
                    protocol: ProviderProtocol::AzureOpenAI,
                    azure: Some(AzureTransportConfig {
                        endpoint: server.url(),
                        api: api.clone(),
                        shape: Some(AzureBodyShape::Chat),
                        authentication: AzureAuthentication::ApiKey,
                    }),
                    models: [(
                        config.model.clone(),
                        ModelConfig {
                            azure_deployment: Some("deployment".into()),
                            reasoning_supported: Some(true),
                            reasoning: Some(talos_config::ReasoningOptions {
                                effort: Some(talos_core::model::ReasoningEffort::High),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )]
                    .into(),
                    timeout: talos_config::ProviderTimeoutConfig {
                        max_attempts: 0,
                        dispatch_timeout_secs: 5,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            let provider = talos_provider::openai_responses::factory::build_configured_provider(
                &config,
                "fixture-key",
            )
            .expect("factory");
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(talos_tools::file_tools::ReadTool::new(
                workspace.clone(),
            )));
            let engine = talos_permission::PermissionEngine::with_workspace_root(workspace.clone());
            let mut agent = talos_agent::Agent::with_security(
                provider,
                registry,
                Some(Arc::new(engine)),
                None,
                workspace,
            )
            .with_reasoning_identity(
                Some("azure-fixture".into()),
                Some("logical-model".into()),
                true,
            );
            agent.set_tool_protocol(protocol);
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (answer, history) = tokio::time::timeout(
                Duration::from_secs(20),
                agent.run_streaming(
                    "Read fixture.txt".into(),
                    vec![Message::Multimodal { parts }],
                    tx,
                ),
            )
            .await
            .expect("bounded Agent")
            .expect("Agent");
            assert_eq!(answer, "done");
            assert!(history.iter().any(|m| matches!(m, Message::Assistant { reasoning: Some(r), .. } if !r.blocks.is_empty())));
            let mut input = 0;
            let mut output = 0;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::TurnEnd { usage, .. } = event {
                    input += usage.input_tokens;
                    output += usage.output_tokens;
                }
            }
            assert_eq!((input, output), (8, 4));
            assert!(history.iter().any(|m| matches!(m, Message::Tool { result } if !result.is_error && result.content.contains("AZURE_CHAT_READ_OK"))));
            for fixture in fixtures {
                fixture.assert_async().await;
            }
        }
    }
}

#[tokio::test]
async fn azure_chat_terminal_failures_are_explicit_and_redacted() {
    let cases: Vec<(Vec<u8>, bool)> = vec![
        (b"data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n".to_vec(), false),
        (b"data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"fixture-key\"}]}\n\n".to_vec(), false),
        (b"data: {\"error\":{\"message\":\"fixture-key\"}}\n\n".to_vec(), false),
        (vec![b'd', b'a', b't', b'a', b':', b' ', 0xe4, 0xbd], false),
        (b"data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"length\"}],\"usage\":{\"prompt_tokens\":3,\"completion_tokens\":2}}\n\n".to_vec(), true),
    ];
    for (body, length) in cases {
        let mut server = mockito::Server::new_async().await;
        let fixture = server
            .mock("POST", "/openai/v1/chat/completions")
            .with_header("content-type", "text/event-stream")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        let mut config = Config::default();
        config.provider = "azure-fixture".into();
        config.model = "deployment".into();
        config.providers.insert(
            config.provider.clone(),
            ProviderConfig {
                protocol: ProviderProtocol::AzureOpenAI,
                azure: Some(AzureTransportConfig {
                    endpoint: server.url(),
                    api: AzureApiMode::V1,
                    shape: Some(AzureBodyShape::Chat),
                    authentication: AzureAuthentication::ApiKey,
                }),
                timeout: talos_config::ProviderTimeoutConfig {
                    max_attempts: 0,
                    dispatch_timeout_secs: 5,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let provider = talos_provider::openai_responses::factory::build_configured_provider(
            &config,
            "fixture-key",
        )
        .expect("factory");
        let mut rx = provider
            .stream(&[Message::User {
                content: "test".into(),
            }])
            .await
            .expect("dispatch");
        tokio::time::timeout(std::time::Duration::from_secs(6), async {
            let (mut errors, mut ends) = (0, 0);
            while let Some(event) = rx.recv().await {
                assert!(!format!("{event:?}").contains("fixture-key"));
                match event {
                    AgentEvent::Error { message, .. } => {
                        assert_eq!(message, "Azure response stream failed");
                        errors += 1;
                    }
                    AgentEvent::TurnEnd { stop_reason, usage } => {
                        assert!(length);
                        assert_eq!(stop_reason, StopReason::MaxTokens);
                        assert_eq!(usage.input_tokens, 3);
                        assert_eq!(usage.output_tokens, 2);
                        ends += 1;
                    }
                    _ => {}
                }
            }
            assert_eq!(errors, usize::from(!length));
            assert_eq!(ends, usize::from(length));
        })
        .await
        .expect("bounded terminal");
        fixture.assert_async().await;
    }
}
