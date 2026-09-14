use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProtocol {
    #[default]
    Native,
    TalosStrict,
    Compat,
    /// Select Native only when the provider supplies explicit capability evidence.
    Auto,
}

/// Provider-advertised evidence used for automatic protocol selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolCapabilities {
    /// Whether provider-native tool calls are supported.
    pub native_tools: bool,
    /// Whether the compatibility text parser is available.
    pub compatibility: bool,
}

/// Result of a provider capability probe; unknown evidence must not assume native support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityProbe {
    Known(ProtocolCapabilities),
    Unknown,
}

/// Request-scoped cache for endpoint/model capability evidence.
///
/// Keys are supplied by the caller and must include every configuration value that can
/// affect protocol support (normally endpoint, provider and model). Entries are never
/// persisted or shared across agents, preventing cross-endpoint capability leakage.
#[derive(Clone, Default)]
pub struct ProtocolCapabilityCache {
    entries: Arc<RwLock<HashMap<String, CapabilityProbe>>>,
}

impl ProtocolCapabilityCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns cached evidence for an exact endpoint/model scope.
    pub fn get(&self, scope: &str) -> Option<CapabilityProbe> {
        self.entries.read().ok()?.get(scope).copied()
    }

    /// Stores evidence for an exact endpoint/model scope.
    pub fn insert(&self, scope: impl Into<String>, probe: CapabilityProbe) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(scope.into(), probe);
        }
    }

    /// Removes all cached evidence.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }
}

/// Safe disposition after a provider protocol failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFailureDisposition {
    /// Retry the same request using the compatibility parser.
    Fallback,
    /// Ask for a corrected provider response without executing tools.
    Correct,
    /// Stop because execution state is not trustworthy.
    Stop,
    /// Require a human decision.
    HumanReview,
}

/// Whether a request may be safely attempted again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionOutcome {
    /// No provider request was dispatched.
    NotStarted,
    /// Provider confirmed completion.
    Completed,
    /// Dispatch may have reached the provider; replay is unsafe.
    Unknown,
}

impl ExecutionOutcome {
    /// Returns true only when replay cannot duplicate an operation.
    pub const fn permits_retry(self) -> bool {
        matches!(self, Self::NotStarted)
    }
}

/// Classifies transport failures without trusting provider-controlled error text.
///
/// An invalid response alone does not prove a tool-protocol mismatch. A separate
/// validated protocol observation is required before correction or fallback.
pub fn classify_protocol_failure(
    error: &crate::provider::ProviderError,
) -> ProtocolFailureDisposition {
    use crate::provider::ProviderError;
    match error {
        ProviderError::AuthenticationFailed(_)
        | ProviderError::RateLimited(_)
        | ProviderError::ServerError(_)
        | ProviderError::NetworkError(_) => ProtocolFailureDisposition::Stop,
        ProviderError::InvalidResponse(_) => ProtocolFailureDisposition::HumanReview,
    }
}

impl CapabilityProbe {
    /// Resolve a protocol conservatively when probing is unavailable.
    pub fn select(self) -> ToolProtocol {
        match self {
            Self::Known(capabilities) => capabilities.select(),
            Self::Unknown => ToolProtocol::Compat,
        }
    }
}

impl ProtocolCapabilities {
    /// Select the safest deterministic protocol, preferring native calls.
    pub fn select(self) -> ToolProtocol {
        if self.native_tools {
            ToolProtocol::Native
        } else if self.compatibility {
            ToolProtocol::Compat
        } else {
            ToolProtocol::TalosStrict
        }
    }
}

#[cfg(test)]
mod capability_tests {
    use super::*;

    #[test]
    fn selection_prefers_native_then_compat_then_strict() {
        assert_eq!(
            ProtocolCapabilities {
                native_tools: true,
                compatibility: true
            }
            .select(),
            ToolProtocol::Native
        );
        assert_eq!(
            ProtocolCapabilities {
                native_tools: false,
                compatibility: true
            }
            .select(),
            ToolProtocol::Compat
        );
        assert_eq!(
            ProtocolCapabilities {
                native_tools: false,
                compatibility: false
            }
            .select(),
            ToolProtocol::TalosStrict
        );
    }

    #[test]
    fn unknown_probe_uses_compatibility_recovery() {
        assert_eq!(CapabilityProbe::Unknown.select(), ToolProtocol::Compat);
    }

    #[test]
    fn failure_classification_is_fail_closed() {
        use crate::provider::ProviderError;
        for message in [
            "invalid API key",
            "unsupported authentication protocol",
            "malformed",
            "timeout",
        ] {
            for error in [
                ProviderError::AuthenticationFailed(message.into()),
                ProviderError::RateLimited(message.into()),
                ProviderError::ServerError(message.into()),
                ProviderError::NetworkError(message.into()),
            ] {
                assert_eq!(
                    classify_protocol_failure(&error),
                    ProtocolFailureDisposition::Stop
                );
            }
            assert_eq!(
                classify_protocol_failure(&ProviderError::InvalidResponse(message.into())),
                ProtocolFailureDisposition::HumanReview
            );
        }
    }

    #[test]
    fn unknown_execution_never_permits_retry() {
        assert!(ExecutionOutcome::NotStarted.permits_retry());
        assert!(!ExecutionOutcome::Completed.permits_retry());
        assert!(!ExecutionOutcome::Unknown.permits_retry());
    }

    #[test]
    fn capability_cache_is_scoped_and_can_cache_unknown() {
        let cache = ProtocolCapabilityCache::new();
        assert_eq!(cache.get("openai|gpt"), None);
        cache.insert("openai|gpt", CapabilityProbe::Unknown);
        assert_eq!(cache.get("openai|gpt"), Some(CapabilityProbe::Unknown));
        assert_eq!(cache.get("other|gpt"), None);
    }
}

impl ToolProtocol {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" => Some(ToolProtocol::Native),
            "talos-strict" | "talos_xml_json_strict" => Some(ToolProtocol::TalosStrict),
            "compat" | "compatibility" => Some(ToolProtocol::Compat),
            "auto" => Some(ToolProtocol::Auto),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolProtocolConfig {
    pub protocol: ToolProtocol,
    pub strict_prompt: bool,
    pub stream_filter: bool,
    pub schema_validate: bool,
}

impl ToolProtocolConfig {
    pub fn for_protocol(protocol: ToolProtocol) -> Self {
        match protocol {
            ToolProtocol::Native => ToolProtocolConfig {
                protocol,
                strict_prompt: false,
                stream_filter: false,
                schema_validate: false,
            },
            ToolProtocol::TalosStrict => ToolProtocolConfig {
                protocol,
                strict_prompt: true,
                stream_filter: true,
                schema_validate: true,
            },
            ToolProtocol::Compat => ToolProtocolConfig {
                protocol,
                strict_prompt: false,
                stream_filter: true,
                schema_validate: false,
            },
            ToolProtocol::Auto => ToolProtocolConfig {
                protocol,
                strict_prompt: false,
                stream_filter: true,
                schema_validate: true,
            },
        }
    }
}
