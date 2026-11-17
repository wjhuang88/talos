//! External-crate Azure SDK and catalog precedence tests (ADR-103).
use talos_config::{
    AzureApiMode, AzureAuthentication, AzureBodyShape, AzureTransportConfig, Config, ModelConfig,
    ProtocolResolutionSource, ProviderConfig, ProviderProtocol,
};

fn configured() -> Config {
    let mut config = Config::default();
    config.providers.insert(
        "azure".into(),
        ProviderConfig {
            azure: Some(AzureTransportConfig {
                endpoint: "https://resource.openai.azure.com".into(),
                api: AzureApiMode::V1,
                shape: None,
                authentication: AzureAuthentication::ApiKey,
            }),
            ..Default::default()
        },
    );
    config
}

#[test]
fn azure_catalog_admission_and_negative_matrix() {
    let config = configured();
    let source = ProtocolResolutionSource::CatalogInferred;
    assert_eq!(
        config.resolve_model_protocol("azure", "gpt-4.1").source,
        source
    );
    assert_eq!(
        config.resolve_azure_body_shape("azure", "gpt-4.1").source,
        source
    );
    let mut admitted = 0;
    for model in talos_config::model::builtin_models()
        .into_iter()
        .filter(|m| m.provider == "azure")
    {
        if config.resolve_model_protocol("azure", &model.id).source == source {
            assert_eq!(
                config.resolve_model_protocol("azure", &model.id).protocol,
                ProviderProtocol::AzureOpenAI
            );
            assert_eq!(
                config.resolve_azure_body_shape("azure", &model.id).protocol,
                AzureBodyShape::Responses
            );
            admitted += 1;
        }
    }
    assert_eq!(admitted, 22);
    for provider in ["gateway", "azure-cognitive-services"] {
        let mut other = config.clone();
        other
            .providers
            .insert(provider.into(), config.providers["azure"].clone());
        assert_ne!(
            other.resolve_azure_body_shape(provider, "gpt-4.1").source,
            source
        );
    }
    for model in [
        "unknown",
        "gateway/gpt-4.1",
        "deepseek-v4-pro",
        "claude-fable-5",
    ] {
        assert_ne!(
            config.resolve_azure_body_shape("azure", model).source,
            source
        );
    }
    for scenario in 0..6 {
        let mut invalid = config.clone();
        let p = invalid.providers.get_mut("azure").unwrap();
        match scenario {
            0 => p.base_url = Some("https://override.invalid".into()),
            1 => {
                p.models.insert(
                    "gpt-4.1".into(),
                    ModelConfig {
                        base_url: Some("https://override.invalid".into()),
                        ..Default::default()
                    },
                );
            }
            2 => p.azure.as_mut().unwrap().endpoint = "https://user:secret@example.com".into(),
            3 => p.azure = None,
            4 => {
                p.azure.as_mut().unwrap().api = AzureApiMode::Deployment {
                    api_version: "2024-10-21".into(),
                }
            }
            _ => {
                p.models.insert(
                    "gpt-4.1".into(),
                    ModelConfig {
                        protocol: Some(ProviderProtocol::OpenAIChat),
                        ..Default::default()
                    },
                );
            }
        }
        assert_ne!(
            invalid.resolve_azure_body_shape("azure", "gpt-4.1").source,
            source,
            "scenario {scenario}"
        );
    }
}

#[test]
fn azure_public_overrides_schema_and_roundtrip() {
    let mut config = configured();
    config
        .providers
        .get_mut("azure")
        .unwrap()
        .azure
        .as_mut()
        .unwrap()
        .shape = Some(AzureBodyShape::Chat);
    assert_eq!(
        config.resolve_azure_body_shape("azure", "gpt-4.1").protocol,
        AzureBodyShape::Chat
    );
    config.providers.get_mut("azure").unwrap().models.insert(
        "gpt-4.1".into(),
        ModelConfig {
            azure_deployment: Some("deployed-model".into()),
            azure_shape: Some(AzureBodyShape::Responses),
            ..Default::default()
        },
    );
    assert_eq!(
        config.resolve_azure_body_shape("azure", "gpt-4.1").protocol,
        AzureBodyShape::Responses
    );
    let encoded = toml::to_string(&config).expect("serialize");
    let decoded: Config = toml::from_str(&encoded).expect("reload");
    assert_eq!(
        decoded.providers["azure"].azure,
        config.providers["azure"].azure
    );
    let schema = serde_json::to_value(schemars::schema_for!(AzureTransportConfig)).unwrap();
    assert_eq!(schema["additionalProperties"], false);
    let mut value =
        serde_json::to_value(config.providers["azure"].azure.as_ref().unwrap()).unwrap();
    value["token"] = serde_json::json!("inline-token-not-allowed");
    assert!(serde_json::from_value::<AzureTransportConfig>(value).is_err());
    let profile = config.providers["azure"].azure.as_ref().unwrap();
    assert!(
        profile
            .request_url(AzureBodyShape::Responses, "gpt-4.1")
            .is_ok()
    );
    for bad in [".", "..", "../x", "x/y", "x%2fy"] {
        assert!(profile.request_url(AzureBodyShape::Responses, bad).is_err());
    }
}
