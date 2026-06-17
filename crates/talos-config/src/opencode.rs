//! Opencode-style provider configuration import.
//!
//! One-way migration from opencode JSON provider blocks into Talos
//! [`ProviderConfig`](crate::ProviderConfig) values.

use std::collections::HashMap;

use serde::Deserialize;

use crate::{ConfigError, ModelConfig, ProviderConfig, ProviderProtocol};

fn npm_to_protocol(npm: &str) -> ProviderProtocol {
    if npm.contains("anthropic") {
        ProviderProtocol::AnthropicMessages
    } else {
        ProviderProtocol::OpenAIChat
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct OpencodeProvider {
    npm: Option<String>,
    options: OpencodeProviderOptions,
    models: HashMap<String, OpencodeModel>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct OpencodeProviderOptions {
    #[serde(rename = "baseURL")]
    base_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct OpencodeModel {
    limit: OpencodeModelLimit,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct OpencodeModelLimit {
    context: Option<u32>,
    output: Option<u32>,
}

/// Import provider definitions from an opencode-style JSON configuration.
///
/// Accepts either a full opencode config object (with a top-level `provider`
/// field) or the provider object directly.
///
/// # Examples
///
/// ```
/// let json = r#"{
///   "provider": {
///     "custom": {
///       "npm": "@ai-sdk/openai-compatible",
///       "options": { "baseURL": "https://api.example.com/v1" },
///       "models": {
///         "gpt-4o": { "limit": { "context": 128000, "output": 4096 } }
///       }
///     }
///   }
/// }"#;
/// let providers = talos_config::opencode::import_opencode_providers(json).unwrap();
/// assert!(providers.contains_key("custom"));
/// ```
pub fn import_opencode_providers(
    input: &str,
) -> Result<HashMap<String, ProviderConfig>, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| ConfigError::ParseError(format!("invalid JSON: {e}")))?;

    let provider_value = value.get("provider").cloned().unwrap_or(value);

    let providers: HashMap<String, OpencodeProvider> = serde_json::from_value(provider_value)
        .map_err(|e| ConfigError::ParseError(format!("invalid opencode provider schema: {e}")))?;

    let mut result = HashMap::with_capacity(providers.len());
    for (name, op) in providers {
        let mut config = ProviderConfig {
            protocol: op.npm.as_deref().map(npm_to_protocol).unwrap_or_default(),
            tool_protocol: Default::default(),
            base_url: op.options.base_url,
            ..ProviderConfig::default()
        };

        for (model_name, model) in op.models {
            let model_config = ModelConfig {
                context_limit: model.limit.context,
                output_limit: model.limit.output,
            };
            config.models.insert(model_name, model_config);
        }

        result.insert(name, config);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_single_provider() {
        let json = r#"{
            "provider": {
                "bailian": {
                    "npm": "@ai-sdk/openai-compatible",
                    "options": { "baseURL": "https://example.com/v1" },
                    "models": {
                        "glm-5": { "limit": { "context": 202752, "output": 4096 } }
                    }
                }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        assert_eq!(providers.len(), 1);

        let bailian = providers.get("bailian").unwrap();
        assert_eq!(bailian.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(bailian.base_url.as_deref(), Some("https://example.com/v1"));
        assert!(bailian.api_key.is_none());
        assert!(bailian.api_key_env.is_none());

        let glm5 = bailian.models.get("glm-5").unwrap();
        assert_eq!(glm5.context_limit, Some(202_752));
        assert_eq!(glm5.output_limit, Some(4096));
    }

    #[test]
    fn test_import_multiple_providers() {
        let json = r#"{
            "provider": {
                "anthropic": {
                    "npm": "@ai-sdk/anthropic",
                    "models": {
                        "claude-sonnet": { "limit": { "context": 200000 } }
                    }
                },
                "openai": {
                    "npm": "@ai-sdk/openai-compatible",
                    "models": {
                        "gpt-4o": { "limit": { "output": 4096 } }
                    }
                }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        assert_eq!(providers.len(), 2);

        let anthropic = providers.get("anthropic").unwrap();
        assert_eq!(anthropic.protocol, ProviderProtocol::AnthropicMessages);
        assert_eq!(
            anthropic.models.get("claude-sonnet").unwrap().context_limit,
            Some(200_000)
        );

        let openai = providers.get("openai").unwrap();
        assert_eq!(openai.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(
            openai.models.get("gpt-4o").unwrap().output_limit,
            Some(4096)
        );
    }

    #[test]
    fn test_import_bare_provider_object_without_wrapper() {
        let json = r#"{
            "custom": {
                "npm": "@ai-sdk/openai-compatible",
                "options": { "baseURL": "https://custom.example.com" }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(
            providers.get("custom").unwrap().base_url.as_deref(),
            Some("https://custom.example.com")
        );
    }

    #[test]
    fn test_import_missing_optional_fields() {
        let json = r#"{
            "provider": {
                "minimal": {}
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        assert_eq!(providers.len(), 1);

        let minimal = providers.get("minimal").unwrap();
        assert_eq!(minimal.protocol, ProviderProtocol::OpenAIChat);
        assert!(minimal.base_url.is_none());
        assert!(minimal.models.is_empty());
    }

    #[test]
    fn test_import_unknown_npm_defaults_to_openai_chat() {
        let json = r#"{
            "provider": {
                "unknown": { "npm": "@some-vendor/custom-adapter" }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        assert_eq!(
            providers.get("unknown").unwrap().protocol,
            ProviderProtocol::OpenAIChat
        );
    }

    #[test]
    fn test_import_invalid_json_fails() {
        let result = import_opencode_providers("not json");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid JSON"));
    }

    #[test]
    fn test_import_invalid_schema_fails() {
        let json = r#"{ "provider": { "bad": { "options": "should_be_object" } } }"#;
        let result = import_opencode_providers(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid opencode provider schema"));
    }

    #[test]
    fn test_import_model_without_limit() {
        let json = r#"{
            "provider": {
                "p": {
                    "models": { "m": {} }
                }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        let model = providers.get("p").unwrap().models.get("m").unwrap();
        assert!(model.context_limit.is_none());
        assert!(model.output_limit.is_none());
    }

    #[test]
    fn test_import_model_with_partial_limit() {
        let json = r#"{
            "provider": {
                "p": {
                    "models": { "m": { "limit": { "context": 1000 } } }
                }
            }
        }"#;

        let providers = import_opencode_providers(json).unwrap();
        let model = providers.get("p").unwrap().models.get("m").unwrap();
        assert_eq!(model.context_limit, Some(1000));
        assert!(model.output_limit.is_none());
    }
}
