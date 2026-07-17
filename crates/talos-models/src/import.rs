use crate::error::CatalogError;
use talos_core::model::{
    CatalogProviderProtocol, ModelCapabilities, ModelMetadata, ModelPricing, ModelSource,
    ProviderInfo, ProviderSource,
};

/// Result of importing models.dev data — contains both provider and model metadata.
pub struct ImportResult {
    pub providers: Vec<ProviderInfo>,
    pub models: Vec<ModelMetadata>,
}

/// Parses the models.dev `api.json` format.
///
/// Returns both provider metadata (name, env var, API base URL, docs URL) and
/// model metadata (limits, pricing, capabilities). The top level is an object
/// keyed by provider id; each provider value contains `name`, `env`, `npm`,
/// `api`, `doc`, and a nested `models` object map. The `npm` package is used
/// as the primary provider protocol signal when present.
pub fn import_models_dev_api(json: &str, refreshed_at: &str) -> Result<ImportResult, CatalogError> {
    let root: serde_json::Value =
        serde_json::from_str(json).map_err(|e| CatalogError::ParseError(e.to_string()))?;

    let provider_map = root.as_object().ok_or_else(|| {
        CatalogError::ParseError("expected JSON object keyed by provider id".to_string())
    })?;

    let mut providers = Vec::new();
    let mut models = Vec::new();

    for (provider_id, provider_val) in provider_map {
        let provider_obj = provider_val.as_object().ok_or_else(|| {
            CatalogError::ParseError(format!("provider '{provider_id}' is not an object"))
        })?;

        providers.push(parse_provider_info(provider_obj, provider_id, refreshed_at));

        let models_map = provider_obj
            .get("models")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                CatalogError::ParseError(format!(
                    "provider '{provider_id}' missing 'models' object"
                ))
            })?;

        for (model_id, model_val) in models_map {
            let model_obj = model_val.as_object().ok_or_else(|| {
                CatalogError::ParseError(format!(
                    "model '{provider_id}/{model_id}' is not an object"
                ))
            })?;

            let limit = model_obj.get("limit");
            let context_limit = limit
                .and_then(|l| l.get("context"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
            let output_limit = limit
                .and_then(|l| l.get("output"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            let pricing = model_obj.get("cost").map(|c| ModelPricing {
                input_per_1m: c.get("input").and_then(|v| v.as_f64()),
                output_per_1m: c.get("output").and_then(|v| v.as_f64()),
                cache_read_per_1m: c.get("cache_read").and_then(|v| v.as_f64()),
            });

            let capabilities = parse_capabilities(model_obj);

            let release_date = model_obj
                .get("release_date")
                .and_then(|v| v.as_str())
                .map(String::from);

            models.push(ModelMetadata {
                id: model_id.clone(),
                provider: provider_id.clone(),
                context_limit,
                output_limit,
                pricing,
                capabilities,
                release_date,
                variants: vec![],
                source: ModelSource::ModelsDev {
                    refreshed_at: refreshed_at.to_string(),
                },
            });
        }
    }

    Ok(ImportResult { providers, models })
}

/// Parses the models.dev `models.json` format (capabilities-focused, no pricing).
///
/// Returns the same [`ImportResult`] shape as [`import_models_dev_api`]; the
/// difference is that model entries will not have pricing data. Provider
/// metadata is parsed identically.
pub fn import_models_dev_models(
    json: &str,
    refreshed_at: &str,
) -> Result<ImportResult, CatalogError> {
    let root: serde_json::Value =
        serde_json::from_str(json).map_err(|e| CatalogError::ParseError(e.to_string()))?;

    let provider_map = root.as_object().ok_or_else(|| {
        CatalogError::ParseError("expected JSON object keyed by provider id".to_string())
    })?;

    let mut providers = Vec::new();
    let mut models = Vec::new();

    for (provider_id, provider_val) in provider_map {
        let provider_obj = provider_val.as_object().ok_or_else(|| {
            CatalogError::ParseError(format!("provider '{provider_id}' is not an object"))
        })?;

        providers.push(parse_provider_info(provider_obj, provider_id, refreshed_at));

        let models_map = provider_obj
            .get("models")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                CatalogError::ParseError(format!(
                    "provider '{provider_id}' missing 'models' object"
                ))
            })?;

        for (model_id, model_val) in models_map {
            let model_obj = model_val.as_object().ok_or_else(|| {
                CatalogError::ParseError(format!(
                    "model '{provider_id}/{model_id}' is not an object"
                ))
            })?;

            let limit = model_obj.get("limit");
            let context_limit = limit
                .and_then(|l| l.get("context"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);
            let output_limit = limit
                .and_then(|l| l.get("output"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            let capabilities = parse_capabilities(model_obj);

            let release_date = model_obj
                .get("release_date")
                .and_then(|v| v.as_str())
                .map(String::from);

            models.push(ModelMetadata {
                id: model_id.clone(),
                provider: provider_id.clone(),
                context_limit,
                output_limit,
                pricing: None,
                capabilities,
                release_date,
                variants: vec![],
                source: ModelSource::ModelsDev {
                    refreshed_at: refreshed_at.to_string(),
                },
            });
        }
    }

    Ok(ImportResult { providers, models })
}

fn parse_provider_info(
    obj: &serde_json::Map<String, serde_json::Value>,
    provider_id: &str,
    refreshed_at: &str,
) -> ProviderInfo {
    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(provider_id)
        .to_string();

    let api_base_url = obj.get("api").and_then(|v| v.as_str()).map(String::from);
    let npm_package = obj.get("npm").and_then(|v| v.as_str());
    let protocol = infer_provider_protocol(npm_package, api_base_url.as_deref());

    let env_var = obj
        .get("env")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    let doc_url = obj.get("doc").and_then(|v| v.as_str()).map(String::from);

    ProviderInfo {
        id: provider_id.to_string(),
        name,
        api_base_url,
        protocol,
        env_var,
        doc_url,
        source: ProviderSource::ModelsDev {
            refreshed_at: refreshed_at.to_string(),
        },
    }
}

fn infer_provider_protocol(
    npm_package: Option<&str>,
    api_base_url: Option<&str>,
) -> Option<CatalogProviderProtocol> {
    let package = npm_package.unwrap_or_default().to_ascii_lowercase();
    if package.contains("anthropic") {
        return Some(CatalogProviderProtocol::AnthropicMessages);
    }

    let url = api_base_url.unwrap_or_default().to_ascii_lowercase();
    if url.contains("/anthropic/") {
        return Some(CatalogProviderProtocol::AnthropicMessages);
    }

    if npm_package.is_some() || api_base_url.is_some() {
        return Some(CatalogProviderProtocol::OpenAIChat);
    }

    None
}

fn parse_capabilities(obj: &serde_json::Map<String, serde_json::Value>) -> ModelCapabilities {
    ModelCapabilities {
        tools: obj
            .get("tool_call")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        structured_output: obj
            .get("structured_output")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        reasoning: obj
            .get("reasoning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        image_input: obj
            .get("attachment")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const REFRESHED_AT: &str = "2025-07-03T00:00:00Z";

    #[test]
    fn test_import_api_json_minimal() {
        let json = r#"{
            "anthropic": {
                "name": "Anthropic",
                "env": ["ANTHROPIC_API_KEY"],
                "npm": "@ai-sdk/anthropic",
                "doc": "https://docs.anthropic.com",
                "models": {
                    "claude-sonnet-4-5": {
                        "limit": { "context": 200000, "output": 8192 },
                        "cost": { "input": 3.0, "output": 15.0 },
                        "tool_call": true,
                        "reasoning": true
                    }
                }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.providers.len(), 1);

        let m = &result.models[0];
        assert_eq!(m.id, "claude-sonnet-4-5");
        assert_eq!(m.provider, "anthropic");
        assert_eq!(m.context_limit, Some(200_000));
        assert_eq!(m.output_limit, Some(8192));
        assert!(m.capabilities.tools);
        assert!(m.capabilities.reasoning);
        let p = m.pricing.as_ref().unwrap();
        assert_eq!(p.input_per_1m, Some(3.0));
        assert_eq!(p.output_per_1m, Some(15.0));

        let pi = &result.providers[0];
        assert_eq!(pi.id, "anthropic");
        assert_eq!(pi.name, "Anthropic");
        assert_eq!(pi.env_var.as_deref(), Some("ANTHROPIC_API_KEY"));
        assert_eq!(pi.doc_url.as_deref(), Some("https://docs.anthropic.com"));
    }

    #[test]
    fn test_import_api_json_provider_with_api_base_url() {
        let json = r#"{
            "openai": {
                "name": "OpenAI",
                "env": ["OPENAI_API_KEY"],
                "api": "https://api.openai.com/v1",
                "doc": "https://platform.openai.com/docs",
                "models": {
                    "gpt-4o": { "limit": { "context": 128000 }, "tool_call": true }
                }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        let pi = &result.providers[0];
        assert_eq!(
            pi.api_base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn test_import_api_json_provider_protocol_from_npm_package() {
        let json = r#"{
            "kimi-for-coding": {
                "name": "Kimi For Coding",
                "env": ["KIMI_API_KEY"],
                "npm": "@ai-sdk/anthropic",
                "api": "https://api.kimi.com/coding/v1",
                "doc": "https://www.kimi.com/code/docs",
                "models": {
                    "k2p7": { "limit": { "context": 262144 }, "tool_call": true }
                }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        let pi = &result.providers[0];
        assert_eq!(
            pi.protocol,
            Some(CatalogProviderProtocol::AnthropicMessages)
        );
        assert_eq!(
            pi.api_base_url.as_deref(),
            Some("https://api.kimi.com/coding/v1")
        );
    }

    #[test]
    fn test_import_api_json_multiple_providers() {
        let json = r#"{
            "anthropic": {
                "models": { "claude-sonnet-4-5": { "limit": { "context": 200000 } } }
            },
            "openai": {
                "models": { "gpt-4o": { "limit": { "context": 128000 } } }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        assert_eq!(result.models.len(), 2);
        assert_eq!(result.providers.len(), 2);
    }

    #[test]
    fn test_import_api_json_provider_without_name_uses_id() {
        let json = r#"{
            "minimal-provider": {
                "models": {
                    "m1": { "limit": { "context": 100000 }, "tool_call": true }
                }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        let pi = &result.providers[0];
        assert_eq!(pi.id, "minimal-provider");
        assert_eq!(pi.name, "minimal-provider");
        assert!(pi.api_base_url.is_none());
        assert!(pi.env_var.is_none());
    }

    #[test]
    fn test_import_api_json_missing_models_field() {
        let json = r#"{"anthropic": {}}"#;
        let result = import_models_dev_api(json, REFRESHED_AT);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_api_json_not_object() {
        let json = r#"[]"#;
        let result = import_models_dev_api(json, REFRESHED_AT);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_models_json_minimal() {
        let json = r#"{
            "openai": {
                "name": "OpenAI",
                "models": {
                    "o3": {
                        "limit": { "context": 200000, "output": 100000 },
                        "reasoning": true,
                        "tool_call": true
                    }
                }
            }
        }"#;

        let result = import_models_dev_models(json, REFRESHED_AT).expect("parse");
        assert_eq!(result.models.len(), 1);
        let m = &result.models[0];
        assert_eq!(m.id, "o3");
        assert_eq!(m.provider, "openai");
        assert!(m.capabilities.reasoning);
        assert!(m.pricing.is_none());

        let pi = &result.providers[0];
        assert_eq!(pi.name, "OpenAI");
    }

    #[test]
    fn test_import_models_json_unknown_fields_ignored() {
        let json = r#"{
            "test": {
                "models": {
                    "model-1": {
                        "limit": { "context": 100000 },
                        "unknown_field": "ignored",
                        "modalities": ["text", "image"]
                    }
                }
            }
        }"#;

        let result = import_models_dev_models(json, REFRESHED_AT).expect("parse");
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].context_limit, Some(100_000));
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_models_dev_api("not json", REFRESHED_AT);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_api_json_env_array_first_element() {
        let json = r#"{
            "multi-env": {
                "env": ["PRIMARY_KEY", "FALLBACK_KEY"],
                "models": {
                    "m1": { "tool_call": true }
                }
            }
        }"#;

        let result = import_models_dev_api(json, REFRESHED_AT).expect("parse");
        assert_eq!(result.providers[0].env_var.as_deref(), Some("PRIMARY_KEY"));
    }
}
