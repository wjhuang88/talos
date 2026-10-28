//! Centralized construction of configured wire-protocol adapters.
//!
//! Hosts remain responsible for credential lookup, runtime model/variant materialization, mocks,
//! and local failure injection. This module owns only the protocol-to-adapter boundary so adding a
//! wire family does not require parallel construction logic in CLI and Desktop hosts.

use std::sync::Arc;

use talos_config::{Config, ProviderProtocol, resolve_config_provider_protocol};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};

use crate::{AnthropicProvider, OpenAIResponsesProvider, openai::OpenAIProvider};

/// Exact output-token limit shared by provider construction and request admission.
///
/// Models without an explicit configured limit receive the same conservative fallback used by the
/// existing CLI/Desktop hosts: one quarter of context, bounded to 4K..32K and never above context.
#[must_use]
pub fn effective_output_limit(config: &Config) -> u32 {
    config.output_limit().unwrap_or_else(|| {
        let (context_limit, _) = config.resolve_model_limits();
        (context_limit / 4).clamp(4096, 32_768).min(context_limit)
    })
}

/// Build the adapter selected by the effective wire-protocol resolver.
///
/// `config` is expected to have already gone through host-specific model/variant materialization.
/// Protocol selection itself is centralized here and never probes an endpoint or retries using a
/// different wire family.
pub fn build_configured_provider(
    config: &Config,
    api_key: &str,
) -> ProviderResult<Arc<dyn LanguageModel>> {
    let provider_config = config.active_provider_config();
    let model_config = provider_config.models.get(&config.model).cloned();
    let reasoning = model_config.and_then(|model| model.reasoning);
    let output_limit = Some(effective_output_limit(config));
    let timeout = provider_config.timeout;
    let endpoint = config
        .resolve_provider_endpoint()
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
    let base_url = endpoint.base_url;
    let resolution = resolve_config_provider_protocol(config);

    tracing::debug!(
        provider = %config.provider,
        model = %config.model,
        effective_protocol = ?resolution.protocol,
        protocol_source = ?resolution.source,
        endpoint_source = ?endpoint.source,
        "resolved provider wire protocol"
    );

    match resolution.protocol {
        ProviderProtocol::AnthropicMessages => {
            let mut provider = AnthropicProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                // Mixed-protocol catalogs advertise an API root; this adapter consumes a full
                // Messages endpoint. Preserve existing full endpoints and append only once.
                let base_url = base_url.trim().trim_end_matches('/');
                let endpoint = if base_url.ends_with("/messages") {
                    base_url.to_string()
                } else {
                    format!("{base_url}/messages")
                };
                provider = provider.with_base_url(endpoint);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::OpenAIResponses => {
            let mut provider = OpenAIResponsesProvider::new(api_key, &config.model)
                .with_provider_key(&config.provider);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        _ => Err(ProviderError::InvalidResponse(format!(
            "unsupported provider protocol for {} / {}",
            config.provider, config.model
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{CapabilityProbe, ProtocolCapabilities};

    #[test]
    fn model_messages_override_constructs_a_full_endpoint_from_a_gateway_root() {
        for base_url in [
            "https://gateway.test/v1",
            "https://gateway.test/v1/messages/",
        ] {
            let config = Config {
                provider: "gateway".into(),
                model: "test-model".into(),
                providers: std::collections::HashMap::from([(
                    "gateway".into(),
                    talos_config::ProviderConfig {
                        base_url: Some(base_url.into()),
                        models: std::collections::HashMap::from([(
                            "test-model".into(),
                            talos_config::ModelConfig {
                                protocol: Some(ProviderProtocol::AnthropicMessages),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let provider = build_configured_provider(&config, "test-key").expect("adapter");
            let preview = provider.request_preview(&[]).expect("request preview");
            assert_eq!(preview["url"], "https://gateway.test/v1/messages");
        }
    }

    #[test]
    fn official_openai_default_constructs_responses_adapter() {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("packaged OpenAI model");
        let provider = build_configured_provider(&config, "sk-test").expect("provider builds");
        assert_eq!(
            provider.protocol_capabilities(),
            CapabilityProbe::Known(ProtocolCapabilities {
                native_tools: true,
                compatibility: true,
            })
        );
    }

    #[test]
    fn explicit_openai_chat_remains_a_rollback_override() {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("packaged OpenAI model");
        config
            .providers
            .entry("openai".into())
            .or_default()
            .protocol = ProviderProtocol::OpenAIChat;
        let provider = build_configured_provider(&config, "sk-test").expect("provider builds");
        let preview = provider.request_preview(&[]).expect("request preview");
        assert!(preview["body"].get("messages").is_some());
        assert!(preview["body"].get("input").is_none());
    }
    #[test]
    fn model_endpoint_reaches_each_real_adapter_and_never_changes_wire() {
        for (protocol, suffix) in [
            (ProviderProtocol::OpenAIResponses, "/responses"),
            (ProviderProtocol::OpenAIChat, "/chat/completions"),
            (ProviderProtocol::AnthropicMessages, "/messages"),
        ] {
            let config = Config {
                provider: "fixture".into(),
                model: "model".into(),
                providers: std::collections::HashMap::from([(
                    "fixture".into(),
                    talos_config::ProviderConfig {
                        base_url: Some("https://provider.test/v1".into()),
                        models: std::collections::HashMap::from([(
                            "model".into(),
                            talos_config::ModelConfig {
                                base_url: Some("https://model.test/wire/v1".into()),
                                protocol: Some(protocol),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let preview = build_configured_provider(&config, "test-key")
                .expect("model adapter")
                .request_preview(&[])
                .expect("preview");
            assert_eq!(
                preview["url"],
                format!("https://model.test/wire/v1{suffix}")
            );
        }
    }

    #[test]
    fn unsupported_model_transport_is_rejected_without_building_a_fallback_adapter() {
        let config = Config {
            provider: "openai".into(),
            model: "o3".into(),
            providers: std::collections::HashMap::from([(
                "openai".into(),
                talos_config::ProviderConfig {
                    models: std::collections::HashMap::from([(
                        "o3".into(),
                        talos_config::ModelConfig {
                            base_url: Some("wss://model.test/realtime".into()),
                            protocol: Some(ProviderProtocol::OpenAIResponses),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(build_configured_provider(&config, "test-key").is_err());
    }
}
