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
    use std::{sync::Arc, time::Duration};
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
                let response = if second {
                    json!({"content":"done"})
                } else {
                    delta.clone()
                };
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
                        body["model"] == "deployment" && raw.contains("AZURE_CHAT_READ_OK") == second
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
            );
            agent.set_tool_protocol(protocol);
            let (tx, _) = tokio::sync::mpsc::unbounded_channel();
            let (answer, history) = tokio::time::timeout(
                Duration::from_secs(20),
                agent.run_streaming("Read fixture.txt".into(), vec![], tx),
            )
            .await
            .expect("bounded Agent")
            .expect("Agent");
            assert_eq!(answer, "done");
            assert!(history.iter().any(|m| matches!(m, Message::Tool { result } if !result.is_error && result.content.contains("AZURE_CHAT_READ_OK"))));
            for fixture in fixtures {
                fixture.assert_async().await;
            }
        }
    }
}
