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
