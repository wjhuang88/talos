//! Acceptance for the separately reviewed protocol-only catalog snapshot.

use talos_config::{
    Config, ModelConfig, ProtocolResolutionSource, ProviderConfig, ProviderProtocol,
};

fn selected(provider: &str, model: &str) -> Config {
    Config {
        provider: provider.into(),
        model: model.into(),
        ..Default::default()
    }
}

#[test]
fn packaged_model_evidence_selects_wire_shape_without_name_guessing() {
    for (provider, model, expected) in [
        ("openai", "o3", ProviderProtocol::OpenAIResponses),
        (
            "github-copilot",
            "gpt-5.3-codex",
            ProviderProtocol::OpenAIResponses,
        ),
        (
            "github-copilot",
            "gemini-3.5-flash",
            ProviderProtocol::OpenAIChat,
        ),
    ] {
        let config = selected(provider, model);
        let resolved = config.resolve_provider_protocol();
        assert_eq!(resolved.protocol, expected, "{provider}/{model}");
        assert_eq!(resolved.source, ProtocolResolutionSource::CatalogInferred);
    }
}

#[test]
fn replaced_endpoint_requires_explicit_intent_to_use_responses() {
    let mut config = selected("openai", "o3");
    config.providers.insert(
        "openai".into(),
        ProviderConfig {
            base_url: Some("https://unconfirmed.test/v1".into()),
            ..Default::default()
        },
    );
    assert_eq!(config.provider_protocol(), ProviderProtocol::OpenAIChat);
    config
        .providers
        .get_mut("openai")
        .expect("provider")
        .models
        .insert(
            "o3".into(),
            ModelConfig {
                protocol: Some(ProviderProtocol::OpenAIResponses),
                ..Default::default()
            },
        );
    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIResponses);
    assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
}

#[test]
fn unsupported_transport_and_changed_endpoint_are_not_auto_activated() {
    for (provider, model) in [("amazon-bedrock", "openai.gpt-5.4"), ("opencode", "gpt-5")] {
        let resolved = selected(provider, model).resolve_provider_protocol();
        assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    }
}

#[test]
fn frozen_projection_counts_and_existing_provider_root_are_preserved() {
    let data: toml::Value = toml::from_str(include_str!("../src/models.toml")).expect("catalog");
    let models = data["models"].as_array().expect("models");
    assert_eq!(models.len(), 6474);
    for (protocol, count) in [
        ("openai-chat", 3853),
        ("anthropic-messages", 38),
        ("openai-responses", 119),
    ] {
        assert_eq!(
            models
                .iter()
                .filter(
                    |model| model.get("protocol").and_then(toml::Value::as_str) == Some(protocol)
                )
                .count(),
            count
        );
    }
    let opencode = data["providers"]
        .as_array()
        .expect("providers")
        .iter()
        .find(|provider| provider["id"].as_str() == Some("opencode"))
        .expect("opencode");
    assert_eq!(
        opencode["api_base_url"].as_str(),
        Some("https://opencode.ai/zen/v1")
    );
}
