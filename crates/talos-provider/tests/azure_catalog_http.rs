//! Catalog admission must select the wire without supplying a credential destination.
use serde_json::Value;
use std::time::Duration;
use talos_config::{
    AzureApiMode, AzureAuthentication, AzureBodyShape, AzureTransportConfig, Config, ModelConfig,
    ProviderConfig,
};
use talos_core::message::{AgentEvent, Message};

#[tokio::test]
async fn catalog_default_and_explicit_shapes_route_through_factory() {
    for (provider_shape, model_shape, responses) in [
        (None, None, true),
        (Some(AzureBodyShape::Chat), None, false),
        (
            Some(AzureBodyShape::Responses),
            Some(AzureBodyShape::Chat),
            false,
        ),
        (
            Some(AzureBodyShape::Chat),
            Some(AzureBodyShape::Responses),
            true,
        ),
    ] {
        let mut server = mockito::Server::new_async().await;
        let path = if responses {
            "/openai/v1/responses"
        } else {
            "/openai/v1/chat/completions"
        };
        let body = if responses {
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ok\"}\n\ndata: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n"
        } else {
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n"
        };
        let fixture = server
            .mock("POST", path)
            .match_header("api-key", "fixture-key")
            .match_header("authorization", mockito::Matcher::Missing)
            .match_request(move |request| {
                let body: Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body["model"] == "deployed.model"
                    && if responses {
                        body["store"] == false && body["input"].is_array()
                    } else {
                        body["messages"].is_array()
                    }
            })
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        let mut config = Config::default();
        config.provider = "azure".into();
        config.model = "gpt-4.1".into();
        config.providers.insert(
            "azure".into(),
            ProviderConfig {
                azure: Some(AzureTransportConfig {
                    endpoint: server.url(),
                    api: AzureApiMode::V1,
                    shape: provider_shape,
                    authentication: AzureAuthentication::ApiKey,
                }),
                models: [(
                    "gpt-4.1".into(),
                    ModelConfig {
                        azure_deployment: Some("deployed.model".into()),
                        azure_shape: model_shape,
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
        let mut receiver = provider
            .stream(&[Message::User {
                content: "fixture".into(),
            }])
            .await
            .expect("stream");
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut text = String::new();
            while let Some(event) = receiver.recv().await {
                match event {
                    AgentEvent::TextDelta { delta } => text.push_str(&delta),
                    AgentEvent::Error { message } => panic!("unexpected error: {message}"),
                    _ => {}
                }
            }
            assert_eq!(text, "ok");
        })
        .await
        .expect("bounded stream");
        fixture.assert_async().await;
    }
}
