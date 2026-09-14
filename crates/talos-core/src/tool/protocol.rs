use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProtocol {
    #[default]
    Native,
    TalosStrict,
    Compat,
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

/// Classifies protocol failures without inspecting secrets or tool arguments.
pub fn classify_protocol_failure(message: &str) -> ProtocolFailureDisposition {
    let value = message.to_ascii_lowercase();
    if value.contains("timeout") || value.contains("cancel") {
        ProtocolFailureDisposition::Stop
    } else if value.contains("malformed") || value.contains("invalid") {
        ProtocolFailureDisposition::Correct
    } else if value.contains("unsupported") || value.contains("protocol") {
        ProtocolFailureDisposition::Fallback
    } else {
        ProtocolFailureDisposition::HumanReview
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
}

impl ToolProtocol {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" => Some(ToolProtocol::Native),
            "talos-strict" | "talos_xml_json_strict" => Some(ToolProtocol::TalosStrict),
            "compat" | "compatibility" => Some(ToolProtocol::Compat),
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
        }
    }
}
