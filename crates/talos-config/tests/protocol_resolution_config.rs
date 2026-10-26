use talos_config::{Config, ProtocolResolutionSource, ProviderConfig, ProviderProtocol};

#[test]
fn selecting_and_connecting_openai_do_not_persist_an_explicit_chat_override() {
    let mut config = Config::default();
    config
        .set_active_model("openai/o3")
        .expect("packaged model");
    config.set_provider_credential("openai", "test-key");
    assert!(config.providers["openai"].protocol.is_unspecified());
    let encoded = toml::to_string(&config).expect("persist config");
    let restored: Config = toml::from_str(&encoded).expect("load config");
    assert_eq!(
        restored.provider_protocol(),
        ProviderProtocol::OpenAIResponses
    );
    // Either the packaged snapshot or the confirmed provider default can supply evidence;
    // selecting a model and inserting a credential must never create explicit user intent.
    assert!(matches!(
        restored.resolve_provider_protocol().source,
        ProtocolResolutionSource::Default | ProtocolResolutionSource::CatalogInferred
    ));
}

#[test]
fn packaged_model_signal_cannot_confirm_a_replaced_builtin_endpoint() {
    let mut config = Config {
        provider: "openai".into(),
        model: "o3".into(),
        ..Default::default()
    };
    assert!(matches!(
        config.resolve_provider_protocol().source,
        ProtocolResolutionSource::Default | ProtocolResolutionSource::CatalogInferred
    ));
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
            talos_config::ModelConfig {
                protocol: Some(ProviderProtocol::OpenAIResponses),
                ..Default::default()
            },
        );
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::OpenAIResponses
    );
    assert_eq!(
        config.resolve_provider_protocol().source,
        ProtocolResolutionSource::Configured
    );
}

#[test]
fn preview_and_active_protocol_use_the_same_model_override_without_mutation() {
    let mut config = Config::default();
    config.set_provider_credential("gateway", "test-key");
    config
        .providers
        .get_mut("gateway")
        .expect("provider")
        .protocol = ProviderProtocol::OpenAIChat;
    config
        .providers
        .get_mut("gateway")
        .expect("provider")
        .models
        .insert(
            "test-model".into(),
            talos_config::ModelConfig {
                protocol: Some(ProviderProtocol::OpenAIResponses),
                ..Default::default()
            },
        );
    let preview = config.resolve_model_protocol("gateway", "test-model");
    assert_eq!(preview.protocol, ProviderProtocol::OpenAIResponses);
    assert_eq!(preview.source, ProtocolResolutionSource::Configured);
    assert_eq!(config.provider, "anthropic");
    config.provider = "gateway".into();
    config.model = "test-model".into();
    assert_eq!(config.resolve_provider_protocol(), preview);
    assert_eq!(config.provider_protocol(), preview.protocol);
}

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
fn official_openai_resolves_responses_consistently_with_runtime_dispatch() {
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
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::OpenAIResponses
    );
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
