//! Controlled HTTP tests of the real factory composition; no production routing override exists.

use super::*;
use mockito::{Matcher, Server};
use talos_config::{ProviderConfig, ProviderTimeoutConfig};
use talos_core::message::{AgentEvent, Message};

#[path = "factory_copilot_agent_tests.rs"]
mod agent_tests;

const KEY: &str = "copilot-controlled-test-secret";

fn config(protocol: ProviderProtocol, root: &str) -> Config {
    Config {
        provider: "github-copilot".into(),
        model: "claude-sonnet-4.6".into(),
        providers: std::collections::HashMap::from([(
            "github-copilot".into(),
            ProviderConfig {
                protocol,
                base_url: Some(root.into()),
                timeout: ProviderTimeoutConfig {
                    max_attempts: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
        )]),
        ..Default::default()
    }
}

fn controlled(config: &Config, server: &Server) -> Arc<dyn LanguageModel> {
    build_with_connection(config, KEY, |base, protocol| {
        // Validate the production origin/route and construct the production headers/client first.
        // Only this private unit-test composition relocates the already-selected connection.
        let mut connection = crate::copilot::connection(base, protocol)?.expect("Copilot selected");
        let url = reqwest::Url::parse(&connection.base_url).expect("canonical URL");
        connection.base_url = format!("{}{}", server.url(), url.path().trim_end_matches('/'));
        Ok(Some(connection))
    })
    .expect("controlled factory")
}

#[test]
fn public_copilot_factory_selects_wire_without_exposing_a_credential() {
    let mut interactions = Vec::new();
    for (protocol, expected) in [
        (ProviderProtocol::AnthropicMessages, "/v1/messages"),
        (ProviderProtocol::OpenAIChat, "/chat/completions"),
        (ProviderProtocol::OpenAIResponses, "/responses"),
    ] {
        for root in [
            "https://api.githubcopilot.com",
            "https://api.githubcopilot.com/v1",
        ] {
            let preview = build_configured_provider(&config(protocol.clone(), root), KEY)
                .expect("public factory")
                .request_preview(&[])
                .expect("preview");
            assert_eq!(
                preview["url"],
                format!("https://api.githubcopilot.com{expected}")
            );
            assert_eq!(preview["headers"]["Authorization"], "Bearer ***");
            assert_eq!(preview["headers"]["Copilot-Integration-Id"], "talos");
            assert_eq!(preview["headers"]["X-GitHub-Api-Version"], "2026-08-01");
            assert!(preview["headers"].get("x-api-key").is_none());
            assert!(!preview.to_string().contains(KEY));
            let interaction = preview["headers"]["X-Interaction-Id"]
                .as_str()
                .expect("identity");
            uuid::Uuid::parse_str(interaction).expect("random process identity");
            interactions.push(interaction.to_string());
            if protocol == ProviderProtocol::AnthropicMessages {
                assert_eq!(preview["headers"]["anthropic-version"], "2023-06-01");
                assert!(preview["body"].get("stream_options").is_none());
            }
        }
    }
    assert!(interactions.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn public_copilot_default_root_and_replaced_gateway_have_distinct_auth() {
    let mut public = config(
        ProviderProtocol::AnthropicMessages,
        "https://api.githubcopilot.com",
    );
    public
        .providers
        .get_mut("github-copilot")
        .expect("host")
        .base_url = None;
    let preview = build_configured_provider(&public, KEY)
        .expect("builtin default")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(preview["url"], "https://api.githubcopilot.com/v1/messages");
    assert_eq!(preview["headers"]["Authorization"], "Bearer ***");

    for root in [
        "https://gateway.test/v1",
        "https://api.githubcopilot.com.gateway.test/v1",
    ] {
        let preview =
            build_configured_provider(&config(ProviderProtocol::AnthropicMessages, root), KEY)
                .expect("custom gateway")
                .request_preview(&[])
                .expect("preview");
        assert_eq!(preview["url"], format!("{root}/messages"));
        assert!(preview["headers"].get("x-api-key").is_some());
        assert!(preview["headers"].get("Authorization").is_none());
        assert!(preview["headers"].get("Copilot-Integration-Id").is_none());
    }
}

#[test]
fn public_copilot_refuses_unreviewed_origin_parts_or_wire_paths() {
    for root in [
        "http://api.githubcopilot.com/v1",
        "https://api.githubcopilot.com:8443/v1",
        "https://user:secret@api.githubcopilot.com/v1",
        "https://api.githubcopilot.com/v1?secret=hidden",
        "https://api.githubcopilot.com/v1#hidden",
        "https://api.githubcopilot.com/other",
        "https://api.githubcopilot.com/responses",
    ] {
        let result =
            build_configured_provider(&config(ProviderProtocol::AnthropicMessages, root), KEY);
        assert!(result.is_err(), "{root}");
        let error = result.err().expect("error").to_string();
        assert!(!error.contains(KEY));
        assert!(!error.contains("user:secret"));
    }
}

fn success_wire(protocol: &ProviderProtocol) -> String {
    match protocol {
        ProviderProtocol::AnthropicMessages => concat!(
            "event: message_start\ndata: {\"message\":{\"usage\":{\"input_tokens\":1}}}\n\n",
            "event: content_block_start\ndata: {\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "event: content_block_delta\ndata: {\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n",
            "event: message_delta\ndata: {\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
            "event: message_stop\ndata: {}\n\n"
        ).into(),
        ProviderProtocol::OpenAIChat => concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        ).into(),
        ProviderProtocol::OpenAIResponses => concat!(
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_test\",\"status\":\"in_progress\"}}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ok\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_test\",\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":1,\"total_tokens\":2}}}\n\n"
        ).into(),
        _ => panic!("test wire"),
    }
}

#[tokio::test]
async fn three_real_adapters_send_bearer_headers_and_the_selected_wire() {
    for (protocol, path) in [
        (ProviderProtocol::AnthropicMessages, "/v1/messages"),
        (ProviderProtocol::OpenAIChat, "/chat/completions"),
        (ProviderProtocol::OpenAIResponses, "/responses"),
    ] {
        let mut server = Server::new_async().await;
        let provider = controlled(
            &config(protocol.clone(), "https://api.githubcopilot.com"),
            &server,
        );
        let preview = provider.request_preview(&[]).expect("preview");
        let expected = server
            .mock("POST", path)
            .match_header("authorization", format!("Bearer {KEY}").as_str())
            .match_header("x-api-key", Matcher::Missing)
            .match_header("copilot-integration-id", "talos")
            .match_header(
                "x-interaction-id",
                preview["headers"]["X-Interaction-Id"]
                    .as_str()
                    .expect("identity"),
            )
            .match_header("x-github-api-version", "2026-08-01")
            .match_body(Matcher::PartialJson(
                serde_json::json!({"model":"claude-sonnet-4.6", "stream":true}),
            ))
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(success_wire(&protocol))
            .expect(1)
            .create_async()
            .await;
        let mut stream = provider
            .stream(&[Message::User {
                content: "hello".into(),
            }])
            .await
            .expect("dispatch");
        let mut text = String::new();
        while let Some(event) = stream.recv().await {
            match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::Error { message } => panic!("stream error: {message}"),
                _ => {}
            }
        }
        assert_eq!(text, "ok", "{protocol:?}");
        expected.assert_async().await;
    }
}

#[tokio::test]
async fn copilot_redirects_and_auth_errors_do_not_replay_or_change_wire() {
    for protocol in [
        ProviderProtocol::AnthropicMessages,
        ProviderProtocol::OpenAIChat,
        ProviderProtocol::OpenAIResponses,
    ] {
        for status in [307, 401, 403] {
            let mut server = Server::new_async().await;
            let mut elsewhere = Server::new_async().await;
            let forbidden = elsewhere
                .mock("POST", Matcher::Any)
                .expect(0)
                .create_async()
                .await;
            let mut configured = config(protocol.clone(), "https://api.githubcopilot.com");
            let timeout = &mut configured
                .providers
                .get_mut("github-copilot")
                .expect("host")
                .timeout;
            timeout.max_attempts = 3;
            timeout.backoff_base_ms = 1;
            timeout.backoff_max_ms = 1;
            let provider = controlled(&configured, &server);
            let path = reqwest::Url::parse(
                provider.request_preview(&[]).expect("preview")["url"]
                    .as_str()
                    .expect("url"),
            )
            .expect("url")
            .path()
            .to_string();
            let response = server
                .mock("POST", path.as_str())
                .match_header("authorization", format!("Bearer {KEY}").as_str())
                .match_header("x-api-key", Matcher::Missing)
                .with_status(status)
                .with_header("location", &elsewhere.url())
                .with_body(format!("controlled refusal: Authorization: Bearer {KEY}"))
                .expect(1)
                .create_async()
                .await;
            let result = provider
                .stream(&[Message::User {
                    content: "hello".into(),
                }])
                .await;
            assert!(result.is_err(), "{protocol:?}/{status}");
            let error = result.expect_err("refusal");
            if status == 401 {
                assert!(matches!(error, ProviderError::AuthenticationFailed(_)));
            }
            assert!(!error.to_string().contains(KEY));
            assert!(error.to_string().contains("controlled refusal"));
            assert!(error.to_string().contains("Bearer ***"));
            response.assert_async().await;
            forbidden.assert_async().await;
        }
    }
}

#[tokio::test]
async fn copilot_bounded_decisions_preserve_authentication_and_output_limits() {
    for (protocol, path, token_field) in [
        (
            ProviderProtocol::AnthropicMessages,
            "/v1/messages",
            "max_tokens",
        ),
        (
            ProviderProtocol::OpenAIChat,
            "/chat/completions",
            "max_completion_tokens",
        ),
        (
            ProviderProtocol::OpenAIResponses,
            "/responses",
            "max_output_tokens",
        ),
    ] {
        let mut server = Server::new_async().await;
        let provider = controlled(
            &config(protocol.clone(), "https://api.githubcopilot.com"),
            &server,
        );
        let mut expected_body = serde_json::json!({"model":"claude-sonnet-4.6"});
        expected_body[token_field] = serde_json::json!(17);
        let expected = server
            .mock("POST", path)
            .match_header("authorization", format!("Bearer {KEY}").as_str())
            .match_header("x-api-key", Matcher::Missing)
            .match_header("copilot-integration-id", "talos")
            .match_body(Matcher::PartialJson(expected_body))
            .match_request(|request| {
                let body: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body.get("tools")
                    .is_none_or(|tools| tools.as_array().is_some_and(Vec::is_empty))
            })
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(success_wire(&protocol))
            .expect(1)
            .create_async()
            .await;
        let mut stream = provider
            .stream_decision(
                &[Message::User {
                    content: "bounded decision".into(),
                }],
                talos_core::provider::DecisionRequestLimits {
                    max_output_tokens: 17,
                    max_retries: 0,
                },
            )
            .await
            .expect("bounded dispatch");
        while let Some(event) = stream.recv().await {
            assert!(!matches!(event, AgentEvent::Error { .. }), "{event:?}");
        }
        expected.assert_async().await;
    }
}

#[test]
fn copilot_full_wire_paths_survive_model_override_validation() {
    for provider_protocol in [ProviderProtocol::Unspecified, ProviderProtocol::OpenAIChat] {
        for model_protocol in [
            ProviderProtocol::AnthropicMessages,
            ProviderProtocol::OpenAIResponses,
        ] {
            let mut configured = config(
                provider_protocol.clone(),
                "https://api.githubcopilot.com/chat/completions",
            );
            configured
                .providers
                .get_mut("github-copilot")
                .expect("host")
                .models
                .insert(
                    configured.model.clone(),
                    talos_config::ModelConfig {
                        protocol: Some(model_protocol),
                        ..Default::default()
                    },
                );
            assert!(build_configured_provider(&configured, KEY).is_err());
            for (url, accepted) in [
                ("https://api.githubcopilot.com/v1/messages", true),
                ("https://api.githubcopilot.com/chat/completions", false),
                ("https://gateway.test/v1", true),
            ] {
                let model = configured
                    .providers
                    .get_mut("github-copilot")
                    .expect("host")
                    .models
                    .get_mut(&configured.model)
                    .expect("model");
                model.protocol = Some(ProviderProtocol::AnthropicMessages);
                model.base_url = Some(url.into());
                let result = build_configured_provider(&configured, KEY);
                assert_eq!(result.is_ok(), accepted);
                if accepted {
                    let snapshot = result
                        .expect("provider")
                        .request_preview(&[])
                        .expect("preview");
                    if url.contains("gateway.test") {
                        assert_eq!(snapshot["url"], "https://gateway.test/v1/messages");
                        assert!(snapshot["headers"].get("Copilot-Integration-Id").is_none());
                        assert!(snapshot["headers"].get("x-api-key").is_some());
                    } else {
                        assert_eq!(snapshot["url"], url);
                        assert_eq!(snapshot["headers"]["Authorization"], "Bearer ***");
                    }
                }
            }
        }
    }
}

#[test]
fn reviewed_copilot_catalog_rows_choose_native_wire_and_preserve_rollback() {
    for model in [
        "claude-fable-5",
        "claude-haiku-4.5",
        "claude-opus-4.7",
        "claude-opus-4.8",
        "claude-opus-5",
        "claude-sonnet-4.6",
        "claude-sonnet-5",
    ] {
        for root in [None, Some("https://api.githubcopilot.com")] {
            let mut configured = config(
                ProviderProtocol::Unspecified,
                "https://api.githubcopilot.com",
            );
            configured.model = model.into();
            configured
                .providers
                .get_mut("github-copilot")
                .expect("host")
                .base_url = root.map(str::to_string);
            let resolution = configured.resolve_provider_protocol();
            assert_eq!(resolution.protocol, ProviderProtocol::AnthropicMessages);
            assert_eq!(
                resolution.source,
                talos_config::ProtocolResolutionSource::CatalogInferred
            );
            let snapshot = build_configured_provider(&configured, KEY)
                .expect("catalog factory")
                .request_preview(&[])
                .expect("preview");
            assert_eq!(snapshot["url"], "https://api.githubcopilot.com/v1/messages");
            assert_eq!(snapshot["headers"]["Authorization"], "Bearer ***");
            configured
                .providers
                .get_mut("github-copilot")
                .expect("host")
                .protocol = ProviderProtocol::OpenAIChat;
            let snapshot = build_configured_provider(&configured, KEY)
                .expect("rollback factory")
                .request_preview(&[])
                .expect("preview");
            assert_eq!(
                snapshot["url"],
                "https://api.githubcopilot.com/chat/completions"
            );
            assert_eq!(snapshot["headers"]["Authorization"], "Bearer ***");
        }
        for root in [
            "https://gateway.test/v1",
            "https://api.githubcopilot.com/v1",
        ] {
            let mut configured = config(ProviderProtocol::Unspecified, root);
            configured.model = model.into();
            assert_eq!(configured.provider_protocol(), ProviderProtocol::OpenAIChat);
            let snapshot = build_configured_provider(&configured, KEY)
                .expect("changed-root factory")
                .request_preview(&[])
                .expect("preview");
            if root.contains("gateway.test") {
                assert_eq!(snapshot["url"], "https://gateway.test/v1/chat/completions");
                assert!(snapshot["headers"].get("Copilot-Integration-Id").is_none());
            } else {
                assert_eq!(
                    snapshot["url"],
                    "https://api.githubcopilot.com/chat/completions"
                );
            }
        }
    }
}
