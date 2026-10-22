//! Protocol-resolution precedence shared by provider configuration and catalog inference.
//!
//! The resolver is deliberately generic over the protocol value. That keeps precedence/provenance
//! independent from any one wire family and lets PROVIDER-008-A add future protocol variants without
//! re-encoding the decision tree in CLI/Desktop call sites.

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
        let unset =
            resolve_protocol(None, None, None, Some(Protocol::Responses), Protocol::Chat);
        assert_eq!(explicit.protocol, Protocol::Chat);
        assert_eq!(explicit.source, ProtocolResolutionSource::Configured);
        assert_eq!(unset.protocol, Protocol::Responses);
        assert_eq!(unset.source, ProtocolResolutionSource::Default);
    }
}
