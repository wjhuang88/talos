//! Public contract tests for ADR-093 protocol resolution.
//!
//! These tests intentionally exercise the exported `talos_config` API instead of path-including
//! implementation modules. That keeps the integration-test crate aligned with the same public
//! surface used by CLI/Desktop consumers.

use std::collections::HashMap;

use talos_config::{
    Config, ModelConfig, ProtocolResolutionSource, ProviderConfig, ProviderProtocol,
    infer_models_dev_model_protocol, resolve_config_provider_protocol, resolve_protocol,
};

#[test]
fn public_resolver_preserves_explicit_chat_rollback() {
    let resolved = resolve_protocol(
        None,
        Some(ProviderProtocol::OpenAIChat),
        Some(ProviderProtocol::OpenAIResponses),
        Some(ProviderProtocol::OpenAIResponses),
        ProviderProtocol::OpenAIChat,
    );

    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
}

#[test]
fn public_config_resolver_prefers_model_override_to_provider_override() {
    let mut config = Config {
        provider: "gateway".into(),
        model: "custom-model".into(),
        ..Config::default()
    };
    config.providers.insert(
        "gateway".into(),
        ProviderConfig {
            protocol: ProviderProtocol::OpenAIResponses,
            models: HashMap::from([(
                "custom-model".into(),
                ModelConfig {
                    protocol: Some(ProviderProtocol::OpenAIChat),
                    ..ModelConfig::default()
                },
            )]),
            ..ProviderConfig::default()
        },
    );

    let resolved = resolve_config_provider_protocol(&config);
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
}

#[test]
fn public_config_resolver_keeps_unknown_gateway_on_chat_fallback() {
    let config = Config {
        provider: "gateway".into(),
        model: "unknown-model".into(),
        ..Config::default()
    };

    let resolved = resolve_config_provider_protocol(&config);
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(resolved.source, ProtocolResolutionSource::Default);
}

#[test]
fn public_models_dev_signal_mapping_requires_confirmed_metadata() {
    assert_eq!(
        infer_models_dev_model_protocol(Some("@ai-sdk/openai"), Some("responses")),
        Some(ProviderProtocol::OpenAIResponses)
    );
    assert_eq!(
        infer_models_dev_model_protocol(Some("@ai-sdk/openai"), Some("completions")),
        Some(ProviderProtocol::OpenAIChat)
    );
    assert_eq!(
        infer_models_dev_model_protocol(Some("@vendor/openai"), None),
        None
    );
}
