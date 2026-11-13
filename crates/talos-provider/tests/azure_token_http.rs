//! Entra credentials are injected only into a child test process, never the shared environment.
use std::time::Duration;
use talos_config::{
    AzureApiMode, AzureAuthentication, AzureBodyShape, AzureTransportConfig, Config,
    ProviderConfig, ProviderProtocol,
};
use talos_core::message::Message;

#[tokio::test]
async fn supplied_token_sends_only_bearer_without_api_key_fallback() {
    if std::env::var_os("TALOS_AZURE_TOKEN_CHILD").is_none() {
        let output = tokio::time::timeout(
            Duration::from_secs(30),
            tokio::process::Command::new(std::env::current_exe().expect("test exe"))
                .args([
                    "--exact",
                    "supplied_token_sends_only_bearer_without_api_key_fallback",
                    "--nocapture",
                ])
                .env("TALOS_AZURE_TOKEN_CHILD", "1")
                .env("TALOS_AZURE_TOKEN_FIXTURE", "fixture-bearer")
                .kill_on_drop(true)
                .output(),
        )
        .await
        .expect("bounded child")
        .expect("child");
        assert!(
            output.status.success(),
            "child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    for shape in [AzureBodyShape::Chat, AzureBodyShape::Responses] {
        let mut server = mockito::Server::new_async().await;
        let path = if shape == AzureBodyShape::Chat {
            "/openai/v1/chat/completions"
        } else {
            "/openai/v1/responses"
        };
        let fixture = server
            .mock("POST", path)
            .match_header("authorization", "Bearer fixture-bearer")
            .match_header("api-key", mockito::Matcher::Missing)
            .with_status(401)
            .with_body("fixture-bearer echoed credential")
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
                api_key: Some("must-not-fallback".into()),
                azure: Some(AzureTransportConfig {
                    endpoint: server.url(),
                    api: AzureApiMode::V1,
                    shape: Some(shape),
                    authentication: AzureAuthentication::EntraToken {
                        token_env: "TALOS_AZURE_TOKEN_FIXTURE".into(),
                    },
                }),
                timeout: talos_config::ProviderTimeoutConfig {
                    max_attempts: 0,
                    dispatch_timeout_secs: 5,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        assert_eq!(config.api_key().expect("dispatch credential"), "");
        let provider = talos_provider::openai_responses::factory::build_configured_provider(
            &config,
            "must-not-fallback",
        )
        .expect("factory");
        let error = provider
            .stream(&[Message::User {
                content: "fixture".into(),
            }])
            .await
            .expect_err("401");
        assert!(!error.to_string().contains("fixture-bearer"));
        fixture.assert_async().await;
    }
}
