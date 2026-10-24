//! Protocol-resolution precedence shared by provider configuration and catalog inference.
//!
//! The resolver is deliberately generic over the protocol value. That keeps precedence/provenance
//! independent from any one wire family and lets PROVIDER-008-A add future protocol variants without
//! re-encoding the decision tree in CLI/Desktop call sites.

use crate::{
    Config, ProviderProtocol, builtin_provider_config, catalog_protocol::builtin_model_protocol,
    normalize_provider_endpoint,
};

/// Provenance for one effective provider protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolResolutionSource {
    /// Explicit user configuration, at model or provider scope.
    Configured,
    /// Confirmed model-level catalog evidence.
    CatalogInferred,
    /// Confirmed provider default or the conservative global fallback.
    Default,
}

/// Effective protocol plus the evidence class that selected it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolResolution<T> {
    pub protocol: T,
    pub source: ProtocolResolutionSource,
}

/// Maps the explicit model-level transport metadata emitted by models.dev to a Talos wire family.
///
/// This intentionally accepts only confirmed signals. `provider.shape` is authoritative when
/// present. Without a shape, only exact AI SDK package identifiers whose default transport is known
/// are accepted. In particular, package-name substrings and endpoint URLs are not used here, so an
/// arbitrary OpenAI-compatible gateway cannot accidentally opt into Responses.
#[must_use]
pub fn infer_models_dev_model_protocol(
    npm_package: Option<&str>,
    shape: Option<&str>,
) -> Option<ProviderProtocol> {
    if let Some(shape) = shape {
        return match shape.trim() {
            "responses" => Some(ProviderProtocol::OpenAIResponses),
            "completions" => Some(ProviderProtocol::OpenAIChat),
            _ => None,
        };
    }

    match npm_package.map(str::trim) {
        Some("@ai-sdk/anthropic") => Some(ProviderProtocol::AnthropicMessages),
        Some("@ai-sdk/openai") => Some(ProviderProtocol::OpenAIResponses),
        Some("@ai-sdk/openai-compatible") => Some(ProviderProtocol::OpenAIChat),
        _ => None,
    }
}

/// Resolves the effective protocol using ADR-093 / PROVIDER-008-A precedence.
///
/// Precedence is strict and intentionally contains no endpoint/name guessing:
///
/// 1. model explicit;
/// 2. provider explicit;
/// 3. confirmed catalog model signal;
/// 4. confirmed provider default;
/// 5. conservative fallback.
///
/// The first two candidates share `Configured` provenance because both represent explicit user
/// intent. In particular, an explicit Chat value is not equivalent to "unset": callers must pass it
/// as `Some(chat)` so it wins over later Responses defaults and remains a reliable rollback.
pub fn resolve_protocol<T: Clone>(
    model_explicit: Option<T>,
    provider_explicit: Option<T>,
    catalog_model: Option<T>,
    confirmed_provider_default: Option<T>,
    fallback: T,
) -> ProtocolResolution<T> {
    if let Some(protocol) = model_explicit {
        return ProtocolResolution {
            protocol,
            source: ProtocolResolutionSource::Configured,
        };
    }
    if let Some(protocol) = provider_explicit {
        return ProtocolResolution {
            protocol,
            source: ProtocolResolutionSource::Configured,
        };
    }
    if let Some(protocol) = catalog_model {
        return ProtocolResolution {
            protocol,
            source: ProtocolResolutionSource::CatalogInferred,
        };
    }
    if let Some(protocol) = confirmed_provider_default {
        return ProtocolResolution {
            protocol,
            source: ProtocolResolutionSource::Default,
        };
    }
    ProtocolResolution {
        protocol: fallback,
        source: ProtocolResolutionSource::Default,
    }
}

/// Resolves the active config's effective provider protocol, including packaged model evidence.
///
/// This is the runtime resolver used by provider construction. It inspects raw user configuration
/// (rather than the merged provider view) so an omitted protocol remains distinguishable from an
/// explicit `openai-chat` rollback.
#[must_use]
pub fn resolve_config_provider_protocol(config: &Config) -> ProtocolResolution<ProviderProtocol> {
    let catalog_model = builtin_model_protocol(&config.provider, &config.model);
    resolve_config_provider_protocol_with_catalog_signal(config, catalog_model)
}

fn resolve_config_provider_protocol_with_catalog_signal(
    config: &Config,
    catalog_model: Option<ProviderProtocol>,
) -> ProtocolResolution<ProviderProtocol> {
    let user_provider = config.providers.get(&config.provider);
    let model_explicit = user_provider
        .and_then(|provider| provider.models.get(&config.model))
        .and_then(|model| model.protocol.clone())
        .filter(|protocol| !protocol.is_unspecified());
    let provider_explicit = user_provider
        .map(|provider| provider.protocol.clone())
        .filter(|protocol| !protocol.is_unspecified());

    let endpoint_signal = user_provider
        .and_then(|provider| provider.base_url.as_deref())
        .map(normalize_provider_endpoint)
        .and_then(|endpoint| {
            (endpoint.protocol == ProviderProtocol::AnthropicMessages)
                .then_some(ProviderProtocol::AnthropicMessages)
        });

    let confirmed_provider_default = if config.provider == "openai" {
        let official_endpoint = user_provider
            .and_then(|provider| provider.base_url.as_deref())
            .is_none_or(|url| url.trim().trim_end_matches('/') == "https://api.openai.com/v1");
        if official_endpoint {
            Some(ProviderProtocol::OpenAIResponses)
        } else {
            endpoint_signal
        }
    } else {
        builtin_provider_config(&config.provider)
            .map(|provider| provider.protocol)
            .filter(|protocol| !protocol.is_unspecified())
            .or(endpoint_signal)
    };

    resolve_protocol(
        model_explicit,
        provider_explicit,
        catalog_model.filter(|protocol| !protocol.is_unspecified()),
        confirmed_provider_default,
        ProviderProtocol::OpenAIChat,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{ModelConfig, ProviderConfig};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Protocol {
        Chat,
        Responses,
        Anthropic,
    }

    #[test]
    fn model_explicit_beats_every_lower_precedence_candidate() {
        let resolved = resolve_protocol(
            Some(Protocol::Chat),
            Some(Protocol::Responses),
            Some(Protocol::Anthropic),
            Some(Protocol::Responses),
            Protocol::Responses,
        );
        assert_eq!(resolved.protocol, Protocol::Chat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
    }

    #[test]
    fn provider_explicit_beats_catalog_and_defaults() {
        let resolved = resolve_protocol(
            None,
            Some(Protocol::Chat),
            Some(Protocol::Responses),
            Some(Protocol::Responses),
            Protocol::Responses,
        );
        assert_eq!(resolved.protocol, Protocol::Chat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
    }

    #[test]
    fn catalog_model_beats_confirmed_provider_default() {
        let resolved = resolve_protocol(
            None,
            None,
            Some(Protocol::Chat),
            Some(Protocol::Responses),
            Protocol::Responses,
        );
        assert_eq!(resolved.protocol, Protocol::Chat);
        assert_eq!(resolved.source, ProtocolResolutionSource::CatalogInferred);
    }

    #[test]
    fn confirmed_provider_default_precedes_global_fallback() {
        let resolved =
            resolve_protocol(None, None, None, Some(Protocol::Anthropic), Protocol::Chat);
        assert_eq!(resolved.protocol, Protocol::Anthropic);
        assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    }

    #[test]
    fn unconfirmed_provider_uses_conservative_fallback() {
        let resolved = resolve_protocol::<Protocol>(None, None, None, None, Protocol::Chat);
        assert_eq!(resolved.protocol, Protocol::Chat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    }

    #[test]
    fn explicit_chat_is_not_conflated_with_unset() {
        let explicit = resolve_protocol(
            None,
            Some(Protocol::Chat),
            None,
            Some(Protocol::Responses),
            Protocol::Chat,
        );
        let unset = resolve_protocol(None, None, None, Some(Protocol::Responses), Protocol::Chat);
        assert_eq!(explicit.protocol, Protocol::Chat);
        assert_eq!(explicit.source, ProtocolResolutionSource::Configured);
        assert_eq!(unset.protocol, Protocol::Responses);
        assert_eq!(unset.source, ProtocolResolutionSource::Default);
    }

    #[test]
    fn models_dev_shape_is_authoritative() {
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/openai"), Some("completions")),
            Some(ProviderProtocol::OpenAIChat)
        );
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/openai-compatible"), Some("responses")),
            Some(ProviderProtocol::OpenAIResponses)
        );
    }

    #[test]
    fn models_dev_known_sdk_defaults_are_exact() {
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/openai"), None),
            Some(ProviderProtocol::OpenAIResponses)
        );
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/anthropic"), None),
            Some(ProviderProtocol::AnthropicMessages)
        );
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/openai-compatible"), None),
            Some(ProviderProtocol::OpenAIChat)
        );
        assert_eq!(
            infer_models_dev_model_protocol(Some("@vendor/openai"), None),
            None
        );
    }

    #[test]
    fn models_dev_unknown_shape_fails_closed() {
        assert_eq!(
            infer_models_dev_model_protocol(Some("@ai-sdk/openai"), Some("future-shape")),
            None
        );
    }

    fn config_with_provider(provider: &str, model: &str) -> Config {
        Config {
            provider: provider.into(),
            model: model.into(),
            ..Config::default()
        }
    }

    #[test]
    fn config_model_explicit_beats_provider_explicit_and_catalog() {
        let mut config = config_with_provider("openai", "o3");
        config.providers.insert(
            "openai".into(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIResponses,
                models: HashMap::from([(
                    "o3".into(),
                    ModelConfig {
                        protocol: Some(ProviderProtocol::OpenAIChat),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
        );
        let resolved = resolve_config_provider_protocol_with_catalog_signal(
            &config,
            Some(ProviderProtocol::AnthropicMessages),
        );
        assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
    }

    #[test]
    fn config_provider_explicit_chat_beats_catalog_responses() {
        let mut config = config_with_provider("gateway", "o3");
        config.providers.insert(
            "gateway".into(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                ..Default::default()
            },
        );
        let resolved = resolve_config_provider_protocol_with_catalog_signal(
            &config,
            Some(ProviderProtocol::OpenAIResponses),
        );
        assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Configured);
    }

    #[test]
    fn config_catalog_signal_beats_official_openai_default() {
        let config = config_with_provider("openai", "legacy-model");
        let resolved = resolve_config_provider_protocol_with_catalog_signal(
            &config,
            Some(ProviderProtocol::OpenAIChat),
        );
        assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(resolved.source, ProtocolResolutionSource::CatalogInferred);
    }

    #[test]
    fn config_unconfirmed_gateway_falls_back_to_chat() {
        let config = config_with_provider("gateway", "unknown-model");
        let resolved = resolve_config_provider_protocol_with_catalog_signal(&config, None);
        assert_eq!(resolved.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(resolved.source, ProtocolResolutionSource::Default);
    }
}
