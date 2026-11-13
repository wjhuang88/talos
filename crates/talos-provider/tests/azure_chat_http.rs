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
