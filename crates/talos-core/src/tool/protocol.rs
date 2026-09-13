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
