//! Bounded decisions must enforce limits at the real HTTP request boundary.
use talos_core::message::Message;
use talos_core::provider::{DecisionRequestLimits, LanguageModel, ProviderError};
use talos_provider::{AnthropicProvider, openai::OpenAIProvider};

#[tokio::test]
async fn glm_auto_review_uses_low_reasoning_without_affecting_other_requests() {
    let mut server = mockito::Server::new_async().await;
    let auto = server
        .mock("POST", "/chat/completions")
        .match_request(|request| {
            let body: serde_json::Value =
                serde_json::from_slice(request.body().expect("body")).expect("JSON");
            body["reasoning_effort"] == "low"
                && body["thinking"]["type"] == "enabled"
                && body["max_completion_tokens"] == 17
                && body.get("tools").is_none()
        })
        .with_status(500)
        .expect(1)
        .create_async()
        .await;
    let generic = server
        .mock("POST", "/chat/completions")
        .match_request(|request| {
            let body: serde_json::Value =
                serde_json::from_slice(request.body().expect("body")).expect("JSON");
            body.get("thinking").is_none() && body.get("reasoning_effort").is_none()
        })
        .with_status(500)
        .expect(2)
        .create_async()
        .await;
    let provider = OpenAIProvider::new("test", "glm-5.3")
        .with_base_url(server.url())
        .with_reasoning(
            Some(talos_config::ReasoningOptions {
                effort: Some(talos_config::ReasoningEffort::High),
                budget_tokens: None,
                replay: true,
            }),
            Some(8192),
        );
    let limits = DecisionRequestLimits {
        max_output_tokens: 17,
        max_retries: 0,
    };
    assert!(provider.stream_auto_review(&[], limits).await.is_err());
    assert!(provider.stream_decision(&[], limits).await.is_err());
    let other = OpenAIProvider::new("test", "unrelated-model").with_base_url(server.url());
    assert!(other.stream_auto_review(&[], limits).await.is_err());
    // Request-local Auto shaping must not overwrite configured conversation reasoning.
    let normal = server
        .mock("POST", "/chat/completions")
        .match_request(|request| {
            let body: serde_json::Value =
                serde_json::from_slice(request.body().expect("body")).expect("JSON");
            body["reasoning_effort"] == "high" && body.get("thinking").is_none()
        })
        .with_status(400)
        .expect(1)
        .create_async()
        .await;
    assert!(provider.stream(&[]).await.is_err());
    auto.assert_async().await;
    generic.assert_async().await;
    normal.assert_async().await;
}

#[tokio::test]
async fn decision_wire_request_caps_tokens_disables_reasoning_and_does_not_retry() {
    for anthropic in [false, true] {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", if anthropic { "/" } else { "/chat/completions" })
            .match_request(move |request| {
                let body: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body[if anthropic {
                    "max_tokens"
                } else {
                    "max_completion_tokens"
                }] == 17
                    && body.get("tools").is_none()
                    && body.get("thinking").is_none()
                    && body.get("reasoning_effort").is_none()
            })
            .with_status(500)
            .with_body("retryable failure")
            .expect(1)
            .create_async()
            .await;
        let reasoning = talos_config::ReasoningOptions {
            effort: None,
            budget_tokens: Some(1024),
            replay: true,
        };
        let provider: Box<dyn LanguageModel> = if anthropic {
            Box::new(
                AnthropicProvider::new("test", "fixture")
                    .with_base_url(server.url())
                    .with_reasoning(Some(reasoning), Some(8192)),
            )
        } else {
            Box::new(
                OpenAIProvider::new("test", "fixture")
                    .with_base_url(server.url())
                    .with_reasoning(Some(reasoning), Some(8192)),
            )
        };
        let result = provider
            .stream_decision(
                &[Message::User {
                    content: "isolated input".into(),
                }],
                DecisionRequestLimits {
                    max_output_tokens: 17,
                    max_retries: 0,
                },
            )
            .await;
        assert!(matches!(result, Err(ProviderError::ServerError(_))));
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn decision_rejects_history_and_zero_budget_before_dispatch() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let providers: Vec<Box<dyn LanguageModel>> = vec![
        Box::new(OpenAIProvider::new("test", "fixture").with_base_url(server.url())),
        Box::new(AnthropicProvider::new("test", "fixture").with_base_url(server.url())),
    ];
    for provider in providers {
        assert!(
            provider
                .stream_decision(
                    &[],
                    DecisionRequestLimits {
                        max_output_tokens: 0,
                        max_retries: 0
                    }
                )
                .await
                .is_err()
        );
        let history = [Message::Assistant {
            content: "session history".into(),
            tool_calls: vec![],
            reasoning: None,
        }];
        assert!(
            provider
                .stream_decision(
                    &history,
                    DecisionRequestLimits {
                        max_output_tokens: 17,
                        max_retries: 0
                    }
                )
                .await
                .is_err()
        );
    }
    mock.assert_async().await;
}
