//! Endpoint identity must not manufacture model capability evidence.

use talos_core::provider::LanguageModel;
use talos_core::tool::CapabilityProbe;
use talos_provider::{AnthropicProvider, openai::OpenAIProvider};

#[tokio::test]
async fn unresolved_auto_is_rejected_without_network_dispatch() {
    let mut server = mockito::Server::new_async().await;
    let forbidden = server
        .mock("POST", mockito::Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let providers: Vec<Box<dyn LanguageModel>> = vec![
        Box::new(OpenAIProvider::new("test", "fixture").with_base_url(server.url())),
        Box::new(AnthropicProvider::new("test", "fixture").with_base_url(server.url())),
    ];
    for provider in providers {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let result = provider
            .stream_with_protocol(&[], &[], talos_core::tool::ToolProtocol::Auto, tx)
            .await;
        assert!(result.is_err());
        assert_eq!(rx.recv().await, None);
    }
    forbidden.assert_async().await;
}

#[test]
fn scope_is_opaque_unambiguous_and_bound_to_request_configuration() {
    let make = |url: &str, model: &str, output| {
        OpenAIProvider::new("test-key", model)
            .with_base_url(url)
            .with_reasoning(None, output)
            .protocol_capability_scope()
    };
    let scope = make("https://gateway.invalid/v1", "private-model", None).expect("safe scope");
    assert!(!scope.contains("gateway"));
    assert!(!scope.contains("private-model"));
    assert_eq!(
        Some(scope.clone()),
        make("https://gateway.invalid/v1", "private-model", None)
    );
    assert_ne!(
        Some(scope.clone()),
        make("https://gateway.invalid/v1", "other-model", None)
    );
    assert_ne!(
        Some(scope.clone()),
        make("https://other.invalid/v1", "private-model", None)
    );
    assert_ne!(
        Some(scope),
        make("https://gateway.invalid/v1", "private-model", Some(256))
    );
    assert_ne!(
        make("https://gateway.invalid/a|b", "c", None),
        make("https://gateway.invalid/a", "b|c", None)
    );
    for url in [
        "https://user:password@gateway.invalid/v1",
        "https://gateway.invalid/v1?token=secret",
        "https://gateway.invalid/v1#secret",
    ] {
        assert_eq!(make(url, "model", None), None);
        assert_eq!(
            AnthropicProvider::new("test-key", "model")
                .with_base_url(url)
                .protocol_capability_scope(),
            None
        );
    }
}

#[test]
fn official_and_custom_endpoints_require_model_evidence() {
    for model in [
        "",
        "nonexistent-model",
        "gpt-4o",
        "claude-sonnet-4-20250514",
    ] {
        let openai = OpenAIProvider::new("test-only", model);
        let anthropic = AnthropicProvider::new("test-only", model);
        assert_eq!(openai.protocol_capabilities(), CapabilityProbe::Unknown);
        assert_eq!(anthropic.protocol_capabilities(), CapabilityProbe::Unknown);
        assert_eq!(
            openai
                .with_base_url("https://gateway.invalid/v1")
                .protocol_capabilities(),
            CapabilityProbe::Unknown
        );
        assert_eq!(
            anthropic
                .with_base_url("https://gateway.invalid/messages")
                .protocol_capabilities(),
            CapabilityProbe::Unknown
        );
    }
}
