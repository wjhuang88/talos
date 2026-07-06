//! Talos model metadata — static model knowledge with pricing, capabilities, and source provenance.
//!
//! Provides a built-in dataset of mainstream models and supports importing
//! additional model data from models.dev JSON endpoints.
//!
//! The shared data types ([`ModelMetadata`], [`ModelSource`], [`ModelPricing`],
//! [`ModelCapabilities`]) and lookup helpers ([`find_model`],
//! [`find_model_by_provider`], [`models_with_id`]) are defined in
//! `talos_core::model` and re-exported here for backward compatibility.

pub use talos_core::model::{
    CatalogProviderProtocol, ModelCapabilities, ModelMetadata, ModelPricing, ModelSource,
    find_model, find_model_by_provider, models_with_id,
};

use serde::Deserialize;
use thiserror::Error;

/// Error types for model metadata operations.
#[derive(Debug, Error)]
pub enum ModelError {
    /// Failed to parse the built-in model dataset.
    #[error("failed to parse built-in models: {0}")]
    ParseError(String),

    /// Failed to parse models.dev JSON import data.
    #[error("failed to parse models.dev import: {0}")]
    ImportError(String),
}

/// Load the built-in model dataset embedded at compile time.
pub fn builtin_models() -> Vec<ModelMetadata> {
    let toml_str = include_str!("models.toml");
    let dataset: TomlDataset = toml::from_str(toml_str)
        .unwrap_or_else(|e| panic!("built-in models.toml failed to parse: {e}"));
    dataset
        .models
        .into_iter()
        .map(|mut m| {
            m.source = ModelSource::Builtin;
            m
        })
        .collect()
}

/// Provider metadata from the built-in `models.toml` `[[providers]]` section.
#[derive(Debug, Clone)]
pub struct BuiltinProvider {
    /// Provider id, e.g. "anthropic".
    pub id: String,
    /// Display name, e.g. "Anthropic".
    pub name: String,
    /// Default API endpoint from models.dev, e.g. "https://api.anthropic.com/v1/messages".
    pub api_base_url: Option<String>,
    /// API protocol advertised by the catalog, if known.
    pub protocol: Option<crate::ProviderProtocol>,
    /// Canonical env var for the API key, e.g. "ANTHROPIC_API_KEY".
    pub env_var: Option<String>,
    /// Documentation URL.
    pub doc_url: Option<String>,
}

/// Load the built-in provider metadata embedded at compile time.
pub fn builtin_providers() -> Vec<BuiltinProvider> {
    let toml_str = include_str!("models.toml");
    let dataset: TomlDataset = toml::from_str(toml_str)
        .unwrap_or_else(|e| panic!("built-in models.toml failed to parse: {e}"));
    dataset
        .providers
        .into_iter()
        .map(|p| BuiltinProvider {
            id: p.id,
            name: p.name,
            api_base_url: p.api_base_url,
            protocol: p.protocol.map(catalog_protocol_to_config),
            env_var: p.env_var,
            doc_url: p.doc_url,
        })
        .collect()
}

/// Internal TOML dataset wrapper.
#[derive(Debug, Deserialize)]
struct TomlDataset {
    #[serde(default)]
    providers: Vec<TomlProviderEntry>,
    models: Vec<ModelMetadata>,
}

#[derive(Debug, Deserialize)]
struct TomlProviderEntry {
    id: String,
    #[serde(default)]
    name: String,
    api_base_url: Option<String>,
    protocol: Option<CatalogProviderProtocol>,
    env_var: Option<String>,
    doc_url: Option<String>,
}

fn catalog_protocol_to_config(protocol: CatalogProviderProtocol) -> crate::ProviderProtocol {
    match protocol {
        CatalogProviderProtocol::AnthropicMessages => crate::ProviderProtocol::AnthropicMessages,
        CatalogProviderProtocol::OpenAIChat => crate::ProviderProtocol::OpenAIChat,
    }
}

/// Internal TOML dataset wrapper (legacy name kept for the old struct above).
///
/// Import model data from models.dev JSON format for the old catalog pipeline.
///
/// Handles the canonical models.dev format: a JSON object keyed by `"provider/model-id"`,
/// where each value contains fields like `name`, `reasoning`, `tool_call`, `limit.context`,
/// `limit.output`, `attachment`, etc.
///
/// Also accepts the legacy array format (`[{id, provider, ...}]`) for backward compatibility.
///
/// # Errors
///
/// Returns [`ModelError::ImportError`] if the JSON is invalid or cannot be parsed.
pub fn import_models_dev(json: &str) -> Result<Vec<ModelMetadata>, ModelError> {
    let now = chrono_utc();

    if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(json) {
        return import_models_dev_object(&map, &now);
    }

    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
        return import_models_dev_array(&arr, &now);
    }

    Err(ModelError::ImportError(
        "expected JSON object (models.dev canonical) or array (legacy)".to_string(),
    ))
}

fn import_models_dev_object(
    map: &serde_json::Map<String, serde_json::Value>,
    now: &str,
) -> Result<Vec<ModelMetadata>, ModelError> {
    let mut models = Vec::with_capacity(map.len());

    for (full_id, value) in map {
        let obj = value.as_object().ok_or_else(|| {
            ModelError::ImportError(format!("entry '{full_id}' is not an object"))
        })?;

        let (provider, model_id) = full_id.split_once('/').unwrap_or(("unknown", full_id));

        let limit = obj.get("limit");
        let context_limit = limit
            .and_then(|l| l.get("context"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let output_limit = limit
            .and_then(|l| l.get("output"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let pricing = obj.get("pricing").map(|p| ModelPricing {
            input_per_1m: p.get("input").and_then(|v| v.as_f64()),
            output_per_1m: p.get("output").and_then(|v| v.as_f64()),
            cache_read_per_1m: p.get("cache_read").and_then(|v| v.as_f64()),
        });

        let capabilities = ModelCapabilities {
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
        };

        let release_date = obj
            .get("release_date")
            .and_then(|v| v.as_str())
            .map(String::from);

        models.push(ModelMetadata {
            id: model_id.to_string(),
            provider: provider.to_string(),
            context_limit,
            output_limit,
            pricing,
            capabilities,
            release_date,
            source: ModelSource::ModelsDev {
                refreshed_at: now.to_string(),
            },
        });
    }

    Ok(models)
}

fn import_models_dev_array(
    arr: &[serde_json::Value],
    now: &str,
) -> Result<Vec<ModelMetadata>, ModelError> {
    let mut models = Vec::with_capacity(arr.len());

    for value in arr {
        let obj = value
            .as_object()
            .ok_or_else(|| ModelError::ImportError("expected array of objects".to_string()))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ModelError::ImportError("missing 'id' field".to_string()))?
            .to_string();

        let provider = obj
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let context_limit = obj
            .get("context_length")
            .or_else(|| obj.get("context_limit"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let output_limit = obj
            .get("max_tokens")
            .or_else(|| obj.get("output_limit"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let pricing = obj.get("pricing").map(|p| ModelPricing {
            input_per_1m: p
                .get("input")
                .or_else(|| p.get("input_per_1m"))
                .and_then(|v| v.as_f64()),
            output_per_1m: p
                .get("output")
                .or_else(|| p.get("output_per_1m"))
                .and_then(|v| v.as_f64()),
            cache_read_per_1m: p
                .get("cache_read")
                .or_else(|| p.get("cache_read_per_1m"))
                .and_then(|v| v.as_f64()),
        });

        let capabilities = obj
            .get("capabilities")
            .map(|c| ModelCapabilities {
                tools: c.get("tools").and_then(|v| v.as_bool()).unwrap_or(false),
                structured_output: c
                    .get("structured_output")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                reasoning: c
                    .get("reasoning")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                image_input: c
                    .get("image_input")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            })
            .unwrap_or_default();

        let release_date = obj
            .get("release_date")
            .or_else(|| obj.get("released"))
            .and_then(|v| v.as_str())
            .map(String::from);

        models.push(ModelMetadata {
            id,
            provider,
            context_limit,
            output_limit,
            pricing,
            capabilities,
            release_date,
            source: ModelSource::ModelsDev {
                refreshed_at: now.to_string(),
            },
        });
    }

    Ok(models)
}

/// Returns the current UTC time as an ISO 8601 string.
fn chrono_utc() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO 8601 without external deps
    let days = now / 86400;
    let secs = now % 86400;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    // Days since epoch to Y-M-D (simplified algorithm)
    let mut y = 1970;
    let mut d = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1;
    let mut day = d + 1;
    for &md in &month_days {
        if day <= md {
            break;
        }
        day -= md;
        m += 1;
    }

    format!("{y:04}-{m:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn is_leap(year: u64) -> bool {
    year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_models_loads() {
        let models = builtin_models();
        assert!(!models.is_empty(), "expected builtin models, got 0");
        // All should have Builtin source
        for m in &models {
            assert_eq!(m.source, ModelSource::Builtin);
        }
    }

    #[test]
    fn test_find_model_by_id() {
        let models = builtin_models();
        // Should find some model with the given ID (note: bare ID lookup returns
        // the first match across all providers).
        let found = find_model(&models, "claude-sonnet-4-5");
        assert!(found.is_some());

        // Should not find a nonexistent model
        let not_found = find_model(&models, "nonexistent-model-xyz");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_import_models_dev_parses() {
        let json = r#"[
            {
                "id": "claude-sonnet-4-5-20250929",
                "provider": "anthropic",
                "context_length": 200000,
                "max_tokens": 8192,
                "pricing": {
                    "input": 3.0,
                    "output": 15.0,
                    "cache_read": 0.30
                },
                "capabilities": {
                    "tools": true,
                    "structured_output": false,
                    "reasoning": true,
                    "image_input": true
                },
                "release_date": "2025-05-14"
            }
        ]"#;

        let models = import_models_dev(json).expect("should parse");
        assert_eq!(models.len(), 1);
        let m = &models[0];
        assert_eq!(m.id, "claude-sonnet-4-5-20250929");
        assert_eq!(m.provider, "anthropic");
        assert_eq!(m.context_limit, Some(200_000));
        assert_eq!(m.output_limit, Some(8192));
        assert!(m.pricing.is_some());
        let p = m.pricing.as_ref().unwrap();
        assert_eq!(p.input_per_1m, Some(3.0));
        assert_eq!(p.output_per_1m, Some(15.0));
        assert_eq!(p.cache_read_per_1m, Some(0.30));
        assert!(m.capabilities.tools);
        assert!(!m.capabilities.structured_output);
        assert!(m.capabilities.reasoning);
        assert!(m.capabilities.image_input);
        assert_eq!(m.release_date, Some("2025-05-14".to_string()));
        assert!(matches!(m.source, ModelSource::ModelsDev { .. }));
    }

    #[test]
    fn test_model_pricing_optional() {
        let json = r#"[
            {
                "id": "some-model",
                "provider": "unknown"
            }
        ]"#;

        let models = import_models_dev(json).expect("should parse");
        let m = &models[0];
        assert!(m.pricing.is_none());
        assert!(m.context_limit.is_none());
        assert!(m.output_limit.is_none());
    }

    #[test]
    fn test_builtin_source_is_builtin() {
        let models = builtin_models();
        for m in &models {
            assert_eq!(
                m.source,
                ModelSource::Builtin,
                "model {} should have Builtin source",
                m.id
            );
        }
    }

    #[test]
    fn test_import_models_dev_field_aliases() {
        // Test that context_limit and output_limit aliases work
        let json = r#"[
            {
                "id": "test-model",
                "provider": "test",
                "context_limit": 100000,
                "output_limit": 4096
            }
        ]"#;

        let models = import_models_dev(json).expect("should parse");
        let m = &models[0];
        assert_eq!(m.context_limit, Some(100_000));
        assert_eq!(m.output_limit, Some(4096));
    }

    #[test]
    fn test_import_models_dev_invalid_json() {
        let result = import_models_dev("not json");
        assert!(result.is_err());

        let result = import_models_dev(r#"{"not": "an array"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_models_dev_missing_id() {
        let json = r#"[{"provider": "test"}]"#;
        let result = import_models_dev(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_capabilities_defaults() {
        let caps = ModelCapabilities::default();
        assert!(!caps.tools);
        assert!(!caps.structured_output);
        assert!(!caps.reasoning);
        assert!(!caps.image_input);
    }

    #[test]
    fn test_model_pricing_defaults() {
        let pricing = ModelPricing::default();
        assert!(pricing.input_per_1m.is_none());
        assert!(pricing.output_per_1m.is_none());
        assert!(pricing.cache_read_per_1m.is_none());
    }

    #[test]
    fn test_find_model_case_sensitive() {
        let models = builtin_models();
        // Exact match required
        assert!(find_model(&models, "claude-sonnet-4-5").is_some());
        assert!(find_model(&models, "Claude-Sonnet-4-5").is_none());
    }

    #[test]
    fn test_builtin_models_have_reasonable_data() {
        let models = builtin_models();
        // Every model should have id and provider set.
        for m in &models {
            assert!(!m.id.is_empty(), "model has empty id");
            assert!(!m.provider.is_empty(), "model {} has empty provider", m.id);
        }
    }

    #[test]
    fn test_model_metadata_serialization_roundtrip() {
        let models = builtin_models();
        for m in &models {
            let toml_str = toml::to_string(m).expect("should serialize");
            let roundtrip: ModelMetadata = toml::from_str(&toml_str).expect("should deserialize");
            assert_eq!(m.id, roundtrip.id);
            assert_eq!(m.provider, roundtrip.provider);
            assert_eq!(m.context_limit, roundtrip.context_limit);
            assert_eq!(m.output_limit, roundtrip.output_limit);
            assert_eq!(m.capabilities, roundtrip.capabilities);
        }
    }

    #[test]
    fn test_import_models_dev_empty_array() {
        let models = import_models_dev("[]").expect("should parse empty array");
        assert!(models.is_empty());
    }

    #[test]
    fn test_find_model_by_provider_resolves_duplicates() {
        let models = builtin_models();
        // glm-5.2 exists under many providers (models.dev).
        let aihubmix = find_model_by_provider(&models, "aihubmix", "glm-5.2");
        let cortecs = find_model_by_provider(&models, "cortecs", "glm-5.2");
        assert!(aihubmix.is_some());
        assert!(cortecs.is_some());
        assert_eq!(aihubmix.unwrap().provider, "aihubmix");
        assert_eq!(cortecs.unwrap().provider, "cortecs");
    }

    #[test]
    fn test_find_model_by_provider_returns_none_for_wrong_provider() {
        let models = builtin_models();
        assert!(find_model_by_provider(&models, "openai", "claude-sonnet-4-5").is_none());
        assert!(find_model_by_provider(&models, "anthropic", "claude-sonnet-4-5").is_some());
    }

    #[test]
    fn test_models_with_id_detects_ambiguity() {
        let models = builtin_models();
        let unique = models_with_id(&models, "gpt-4o-2024-05-13");
        assert_eq!(unique.len(), 1);

        let ambiguous = models_with_id(&models, "glm-5.2");
        assert!(
            ambiguous.len() >= 2,
            "glm-5.2 should appear under multiple providers"
        );

        let missing = models_with_id(&models, "nonexistent-model");
        assert!(missing.is_empty());
    }
}
