use talos_config::{Config, ProtocolResolutionSource, ProviderConfig, ProviderProtocol};

fn config_with_provider(name: &str, provider: ProviderConfig) -> Config {
    let mut config = Config::default();
    config.provider = name.to_string();
    config.model = "test-model".to_string();
    config.providers.insert(name.to_string(), provider);
    config
}

#[test]
fn omitted_protocol_roundtrips_as_unset_without_exposing_internal_marker() {
    let provider = ProviderConfig {
        api_key_env: Some("TEST_API_KEY".into()),
        ..Default::default()
    };
    assert!(provider.protocol.is_unspecified());

    let encoded = toml::to_string(&provider).expect("serialize provider");
    assert!(
        !encoded
            .lines()
            .any(|line| line.trim_start().starts_with("protocol ="))
    );
    assert!(!encoded.contains("unspecified"));

    let decoded: ProviderConfig = toml::from_str(&encoded).expect("deserialize provider");
    assert!(decoded.protocol.is_unspecified());
    assert!(toml::from_str::<ProviderConfig>("protocol = \"unspecified\"").is_err());
}

#[test]
fn official_openai_resolves_responses_without_changing_legacy_dispatch_yet() {
    let config = config_with_provider(
        "openai",
        ProviderConfig {
            api_key_env: Some("OPENAI_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIResponses);
    assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    assert_eq!(config.provider_protocol(), ProviderProtocol::OpenAIChat);
}

#[test]
fn explicit_chat_remains_a_configured_rollback_for_official_openai() {
    let config = config_with_provider(
        "openai",
        ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            api_key_env: Some("OPENAI_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
}

#[test]
fn official_openai_url_is_still_confirmed_for_responses() {
    let config = config_with_provider(
        "openai",
        ProviderConfig {
            base_url: Some("https://api.openai.com/v1/".into()),
            api_key_env: Some("OPENAI_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIResponses);
    assert_eq!(resolved.source, ProtocolResolutionSource::Default);
}

#[test]
fn unconfirmed_openai_gateway_stays_chat_without_explicit_protocol() {
    let config = config_with_provider(
        "openai",
        ProviderConfig {
            base_url: Some("https://gateway.example.test/v1".into()),
            api_key_env: Some("OPENAI_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(resolved.source, ProtocolResolutionSource::Default);
}

#[test]
fn custom_gateway_can_explicitly_select_responses() {
    let config = config_with_provider(
        "gateway",
        ProviderConfig {
            protocol: ProviderProtocol::OpenAIResponses,
            base_url: Some("https://gateway.example.test/v1".into()),
            api_key_env: Some("GATEWAY_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::OpenAIResponses);
    assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
}

#[test]
fn legacy_anthropic_endpoint_shape_remains_conservative_confirmed_evidence() {
    let config = config_with_provider(
        "gateway",
        ProviderConfig {
            base_url: Some("https://api.example.test/anthropic/v1".into()),
            api_key_env: Some("GATEWAY_API_KEY".into()),
            ..Default::default()
        },
    );

    let resolved = config.resolve_provider_protocol();
    assert_eq!(resolved.protocol, ProviderProtocol::AnthropicMessages);
    assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
}
