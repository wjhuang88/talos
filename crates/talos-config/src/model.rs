//! Talos model metadata — static model knowledge with pricing, capabilities, and source provenance.
//!
//! Provides a built-in dataset of mainstream models and supports importing
//! additional model data from models.dev JSON endpoints.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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

/// Source of model metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelSource {
    /// From the built-in dataset embedded at compile time.
    #[default]
    Builtin,
    /// Manually added by the user.
    Manual,
    /// Imported from models.dev, with a refresh timestamp.
    ModelsDev { refreshed_at: String },
}

/// Pricing information for a model (per 1M tokens, USD).
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ModelPricing {
    /// Input token price per 1M tokens.
    pub input_per_1m: Option<f64>,
    /// Output token price per 1M tokens.
    pub output_per_1m: Option<f64>,
    /// Cache read token price per 1M tokens.
    pub cache_read_per_1m: Option<f64>,
}

/// Capability flags for a model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Supports tool/function calling.
    #[serde(default)]
    pub tools: bool,
    /// Supports structured/JSON output.
    #[serde(default)]
    pub structured_output: bool,
    /// Supports reasoning/thinking mode.
    #[serde(default)]
    pub reasoning: bool,
    /// Accepts image input.
    #[serde(default)]
    pub image_input: bool,
}

/// Static metadata for a known model.
///
/// Represents model knowledge (context limits, pricing, capabilities)
/// independent of runtime configuration. Used to inform the agent about
/// model properties without hardcoded fallbacks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelMetadata {
    /// Unique model identifier (e.g., "claude-sonnet-4-20250514").
    pub id: String,
    /// Provider name (e.g., "anthropic", "openai", "google").
    pub provider: String,
    /// Maximum context window in tokens.
    pub context_limit: Option<u32>,
    /// Maximum output tokens.
    pub output_limit: Option<u32>,
    /// Pricing information (per 1M tokens, USD).
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    /// Model capability flags.
    #[serde(default)]
    pub capabilities: ModelCapabilities,
    /// Model release date (ISO 8601 or similar).
    pub release_date: Option<String>,
    /// Where this metadata originated.
    #[serde(default)]
    pub source: ModelSource,
}

/// Load the built-in model dataset embedded at compile time.
pub fn builtin_models() -> Vec<ModelMetadata> {
    let toml_str = include_str!("models.toml");
    let dataset: ModelDataset = toml::from_str(toml_str)
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

/// Look up a model by id in a dataset.
pub fn find_model<'a>(models: &'a [ModelMetadata], id: &str) -> Option<&'a ModelMetadata> {
    models.iter().find(|m| m.id == id)
}

/// Internal TOML dataset wrapper.
#[derive(Debug, Deserialize)]
struct ModelDataset {
    models: Vec<ModelMetadata>,
}

/// Import model data from models.dev JSON format.
///
/// Accepts the `models.json` array format from models.dev and maps fields:
/// - `context_length` → `context_limit`
/// - `max_tokens` → `output_limit`
/// - `pricing.input` → `pricing.input_per_1m`
/// - `pricing.output` → `pricing.output_per_1m`
/// - `pricing.cache_read` → `pricing.cache_read_per_1m`
/// - `capabilities.*` → `capabilities.*`
///
/// # Errors
///
/// Returns [`ModelError::ImportError`] if the JSON is invalid or cannot be parsed.
pub fn import_models_dev(json: &str) -> Result<Vec<ModelMetadata>, ModelError> {
    // Try parsing as the canonical models.dev format first.
    let raw: Vec<serde_json::Value> =
        serde_json::from_str(json).map_err(|e| ModelError::ImportError(e.to_string()))?;

    let now = chrono_utc();
    let mut models = Vec::with_capacity(raw.len());

    for value in raw {
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
                refreshed_at: now.clone(),
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
        assert!(
            models.len() >= 15,
            "expected at least 15 builtin models, got {}",
            models.len()
        );
        // All should have Builtin source
        for m in &models {
            assert_eq!(m.source, ModelSource::Builtin);
        }
    }

    #[test]
    fn test_find_model_by_id() {
        let models = builtin_models();
        // Should find a known model
        let found = find_model(&models, "claude-sonnet-4-20250514");
        assert!(found.is_some());
        let m = found.unwrap();
        assert_eq!(m.provider, "anthropic");
        assert_eq!(m.context_limit, Some(200_000));

        // Should not find a nonexistent model
        let not_found = find_model(&models, "nonexistent-model-xyz");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_import_models_dev_parses() {
        let json = r#"[
            {
                "id": "claude-sonnet-4-20250514",
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
        assert_eq!(m.id, "claude-sonnet-4-20250514");
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
        assert!(find_model(&models, "claude-sonnet-4-20250514").is_some());
        assert!(find_model(&models, "Claude-Sonnet-4-20250514").is_none());
    }

    #[test]
    fn test_builtin_models_have_reasonable_data() {
        let models = builtin_models();
        // Every model should have at least context_limit set
        for m in &models {
            assert!(
                m.context_limit.is_some(),
                "model {} missing context_limit",
                m.id
            );
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
}
