//! Protocol-resolution precedence shared by provider configuration and catalog inference.
//!
//! The resolver is deliberately generic over the protocol value. That keeps precedence/provenance
//! independent from any one wire family and lets PROVIDER-008-A add future protocol variants without
//! re-encoding the decision tree in CLI/Desktop call sites.

use crate::ProviderProtocol;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
