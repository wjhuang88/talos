use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProtocol {
    /// Automatically select the provider-compatible protocol.
    Auto,
    #[default]
    Native,
    TalosStrict,
    Compat,
}

impl ToolProtocol {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(ToolProtocol::Auto),
            "native" => Some(ToolProtocol::Native),
            "talos-strict" | "talos_xml_json_strict" => Some(ToolProtocol::TalosStrict),
            "compat" | "compatibility" => Some(ToolProtocol::Compat),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ToolProtocol;

    #[test]
    fn parses_automatic_protocol() {
        assert_eq!(ToolProtocol::parse("auto"), Some(ToolProtocol::Auto));
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
            ToolProtocol::Auto => ToolProtocolConfig {
                protocol, strict_prompt: false, stream_filter: false, schema_validate: false,
            },
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
