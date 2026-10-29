//! Actual adapter construction from the independently reviewed packaged endpoint projection.

use talos_config::{Config, EndpointResolutionSource, ProviderConfig, ProviderProtocol};
use talos_provider::openai_responses::factory::build_configured_provider;

fn configured(provider: &str, model: &str, root: &str) -> Config {
    Config {
        provider: provider.into(),
        model: model.into(),
        providers: std::collections::HashMap::from([(
            provider.into(),
            ProviderConfig {
                base_url: Some(root.into()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    }
}

#[test]
fn packaged_endpoints_reach_real_adapters_including_a_reviewed_origin_change() {
    for (provider, model, root, expected) in [
        (
            "freemodel",
            "gpt-5.4",
            "https://cc.freemodel.dev/v1",
            "https://api.freemodel.dev/v1/chat/completions",
        ),
        (
            "zenmux",
            "anthropic/claude-sonnet-4.5",
            "https://zenmux.ai/api/v1",
            "https://zenmux.ai/api/anthropic/v1/messages",
        ),
        (
            "ofox",
            "anthropic/claude-sonnet-4.5",
            "https://api.ofox.ai/v1",
            "https://api.ofox.ai/anthropic/v1/messages",
        ),
    ] {
        let config = configured(provider, model, root);
        assert_eq!(
            config
                .resolve_provider_endpoint()
                .expect("catalog endpoint")
                .source,
            EndpointResolutionSource::CatalogInferred
        );
        let preview = build_configured_provider(&config, "endpoint-test-secret")
            .expect("adapter")
            .request_preview(&[])
            .expect("preview");
        assert_eq!(preview["url"], expected, "{provider}/{model}");
        assert!(!preview.to_string().contains("endpoint-test-secret"));
    }
}

#[test]
fn catalog_routing_never_relocates_a_replaced_or_custom_gateway() {
    let replacement = configured("freemodel", "gpt-5.4", "https://replacement.test/v1");
    let preview = build_configured_provider(&replacement, "gateway-key")
        .expect("adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(
        preview["url"],
        "https://replacement.test/v1/chat/completions"
    );
    assert_eq!(
        replacement
            .resolve_provider_endpoint()
            .expect("route")
            .source,
        EndpointResolutionSource::ProviderConfigured
    );

    // The frozen Neon model is unique, so protocol inheritance really occurs. URL inheritance must
    // still fail closed: the custom gateway's key is never sent to Neon's catalog destination.
    assert_eq!(
        talos_config::model::builtin_models()
            .iter()
            .filter(|model| model.id == "gpt-5-2")
            .count(),
        1
    );
    let custom = configured("custom", "gpt-5-2", "https://custom.test/v1");
    assert_eq!(
        custom.provider_protocol(),
        ProviderProtocol::OpenAIResponses
    );
    let preview = build_configured_provider(&custom, "gateway-key")
        .expect("custom adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(preview["url"], "https://custom.test/v1/responses");
}

#[test]
fn explicit_wire_rollback_does_not_reuse_a_different_catalog_wire_endpoint() {
    let mut config = configured(
        "zenmux",
        "anthropic/claude-sonnet-4.5",
        "https://zenmux.ai/api/v1",
    );
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
    config
        .providers
        .get_mut("zenmux")
        .expect("provider")
        .protocol = ProviderProtocol::OpenAIChat;
    let preview = build_configured_provider(&config, "key")
        .expect("rollback adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(preview["url"], "https://zenmux.ai/api/v1/chat/completions");
}

#[test]
fn omitted_provider_root_cannot_relocate_credentials_to_an_adapter_default_on_rollback() {
    let mut config = configured("freemodel", "gpt-5.4", "https://cc.freemodel.dev/v1");
    config
        .providers
        .get_mut("freemodel")
        .expect("provider")
        .base_url = None;
    let preview = build_configured_provider(&config, "vendor-key")
        .expect("catalog adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(
        preview["url"],
        "https://api.freemodel.dev/v1/chat/completions"
    );
    config
        .providers
        .get_mut("freemodel")
        .expect("provider")
        .protocol = ProviderProtocol::AnthropicMessages;
    let preview = build_configured_provider(&config, "vendor-key")
        .expect("rollback adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(preview["url"], "https://cc.freemodel.dev/v1/messages");
    assert!(
        !preview["url"]
            .as_str()
            .expect("url")
            .contains("api.anthropic.com")
    );
}

#[test]
fn reviewed_copilot_messages_endpoint_is_auto_activated() {
    let config = configured(
        "github-copilot",
        "claude-sonnet-4.6",
        "https://api.githubcopilot.com",
    );
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
    assert_eq!(
        config
            .resolve_provider_endpoint()
            .expect("provider route")
            .source,
        EndpointResolutionSource::CatalogInferred
    );
    let preview = build_configured_provider(&config, "key")
        .expect("existing adapter")
        .request_preview(&[])
        .expect("preview");
    assert_eq!(preview["url"], "https://api.githubcopilot.com/v1/messages");
    assert_eq!(preview["headers"]["Authorization"], "Bearer ***");
    assert!(preview["headers"].get("x-api-key").is_none());
}
