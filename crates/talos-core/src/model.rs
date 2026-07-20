//! Shared model catalog types — provider/model metadata, pricing, and capabilities.
//!
//! These types live at the `talos-core` boundary so that multiple crates
//! (`talos-config`, `talos-models`, CLI, TUI) can share a single definition
//! without creating cyclic dependencies.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

/// Image input capability provenance for a model (ADR-050).
///
/// `Supported` and `Unsupported` are resolved from confirmed catalog
/// metadata. `Unknown` applies to custom/discovered models with no
/// confirmed capability. Both `Unknown` and `Unsupported` fail-closed
/// for the attachment UI; the distinction is diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageInputCapability {
    /// Catalog metadata confirms `image_input = true`.
    Supported,
    /// Catalog metadata confirms `image_input = false`.
    Unsupported,
    /// No confirmed capability (custom/discovered model).
    Unknown,
}

impl ImageInputCapability {
    /// Resolves the capability from a model's catalog metadata.
    ///
    /// Returns `Supported` when `image_input = true`, `Unsupported` when
    /// `image_input = false`, and `Unknown` when no metadata is available
    /// (custom/discovered models).
    pub fn from_metadata(metadata: Option<&ModelMetadata>) -> Self {
        match metadata {
            Some(m) if m.capabilities.image_input => Self::Supported,
            Some(_) => Self::Unsupported,
            None => Self::Unknown,
        }
    }

    /// Returns `true` when image attachment is allowed.
    pub fn allows_attachment(self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// Reasoning effort levels for OpenAI o-series models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort.
    Medium,
    /// High reasoning effort.
    High,
}

/// Provider API protocol advertised by catalog metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum CatalogProviderProtocol {
    /// Anthropic Messages-compatible API.
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
    /// OpenAI Chat Completions-compatible API.
    #[serde(rename = "openai-chat")]
    OpenAIChat,
}

/// Static metadata for a known model.
///
/// Represents model knowledge (context limits, pricing, capabilities)
/// independent of runtime configuration. Used to inform the agent about
/// model properties without hardcoded fallbacks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelMetadata {
    /// Unique model identifier (e.g., "claude-sonnet-4-5-20250929").
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
    /// Named invocation presets for this model (ADR-048).
    #[serde(default)]
    pub variants: Vec<VariantDef>,
}

/// Look up a model by id in a dataset.
///
/// Returns the first model whose `id` matches. When multiple providers share
/// the same model id (e.g. `glm-5.2` under `zhipu`, `zai`, etc.), this returns
/// an arbitrary first match. Use [`find_model_by_provider`] for unambiguous
/// resolution in those cases.
pub fn find_model<'a>(models: &'a [ModelMetadata], id: &str) -> Option<&'a ModelMetadata> {
    models.iter().find(|m| m.id == id)
}

/// Look up a model by `(provider, id)` in a dataset.
///
/// Use this instead of [`find_model`] whenever the active provider is known,
/// so that duplicate model ids across providers resolve unambiguously.
pub fn find_model_by_provider<'a>(
    models: &'a [ModelMetadata],
    provider: &str,
    id: &str,
) -> Option<&'a ModelMetadata> {
    models.iter().find(|m| m.provider == provider && m.id == id)
}

/// Collects all models whose `id` matches, regardless of provider.
///
/// Returns an empty vector when the id is unique or absent. Use this to detect
/// ambiguity before resolving a bare model id.
pub fn models_with_id<'a>(models: &'a [ModelMetadata], id: &str) -> Vec<&'a ModelMetadata> {
    models.iter().filter(|m| m.id == id).collect()
}

/// Lightweight provider metadata for catalog queries.
///
/// Unlike [`talos_config::ProviderConfig`] (which includes credentials and
/// protocol details), this struct carries only the catalog-level identity and
/// display information needed by `/model` and `/connect` pickers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderInfo {
    /// Provider identifier (e.g., "anthropic", "openai").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Default API base URL, if known.
    pub api_base_url: Option<String>,
    /// API protocol, if known from catalog metadata.
    #[serde(default)]
    pub protocol: Option<CatalogProviderProtocol>,
    /// Environment variable name for the API key, if conventional.
    pub env_var: Option<String>,
    /// Documentation URL, if known.
    pub doc_url: Option<String>,
    /// Source of this provider entry.
    #[serde(default)]
    pub source: ProviderSource,
}

/// Source of provider metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSource {
    /// From the built-in dataset.
    #[default]
    Builtin,
    /// Imported from models.dev, with a refresh timestamp.
    ModelsDev { refreshed_at: String },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_model_source_default_is_builtin() {
        assert_eq!(ModelSource::default(), ModelSource::Builtin);
    }

    #[test]
    fn test_model_capabilities_default_all_false() {
        let caps = ModelCapabilities::default();
        assert!(!caps.tools);
        assert!(!caps.structured_output);
        assert!(!caps.reasoning);
        assert!(!caps.image_input);
    }

    #[test]
    fn test_model_pricing_default_all_none() {
        let pricing = ModelPricing::default();
        assert!(pricing.input_per_1m.is_none());
        assert!(pricing.output_per_1m.is_none());
        assert!(pricing.cache_read_per_1m.is_none());
    }

    #[test]
    fn test_find_model_by_provider_resolves() {
        let models = vec![
            ModelMetadata {
                id: "glm-5.2".to_string(),
                provider: "zhipu".to_string(),
                context_limit: Some(128_000),
                output_limit: None,
                pricing: None,
                capabilities: ModelCapabilities::default(),
                release_date: None,
                variants: vec![],
                source: ModelSource::Builtin,
            },
            ModelMetadata {
                id: "glm-5.2".to_string(),
                provider: "zai".to_string(),
                context_limit: Some(128_000),
                output_limit: None,
                pricing: None,
                capabilities: ModelCapabilities::default(),
                release_date: None,
                variants: vec![],
                source: ModelSource::Builtin,
            },
        ];

        let zhipu = find_model_by_provider(&models, "zhipu", "glm-5.2");
        assert!(zhipu.is_some());
        assert_eq!(zhipu.unwrap().provider, "zhipu");

        let zai = find_model_by_provider(&models, "zai", "glm-5.2");
        assert!(zai.is_some());
        assert_eq!(zai.unwrap().provider, "zai");

        assert!(find_model_by_provider(&models, "openai", "glm-5.2").is_none());
    }

    #[test]
    fn test_models_with_id_detects_ambiguity() {
        let models = vec![
            ModelMetadata {
                id: "shared".to_string(),
                provider: "a".to_string(),
                context_limit: None,
                output_limit: None,
                pricing: None,
                capabilities: ModelCapabilities::default(),
                release_date: None,
                variants: vec![],
                source: ModelSource::Builtin,
            },
            ModelMetadata {
                id: "shared".to_string(),
                provider: "b".to_string(),
                context_limit: None,
                output_limit: None,
                pricing: None,
                capabilities: ModelCapabilities::default(),
                release_date: None,
                variants: vec![],
                source: ModelSource::Builtin,
            },
            ModelMetadata {
                id: "unique".to_string(),
                provider: "a".to_string(),
                context_limit: None,
                output_limit: None,
                pricing: None,
                capabilities: ModelCapabilities::default(),
                release_date: None,
                variants: vec![],
                source: ModelSource::Builtin,
            },
        ];

        assert_eq!(models_with_id(&models, "shared").len(), 2);
        assert_eq!(models_with_id(&models, "unique").len(), 1);
        assert!(models_with_id(&models, "missing").is_empty());
    }

    #[test]
    fn test_provider_info_default() {
        let info = ProviderInfo::default();
        assert!(info.id.is_empty());
        assert!(info.name.is_empty());
        assert!(info.api_base_url.is_none());
        assert!(info.protocol.is_none());
        assert!(info.env_var.is_none());
        assert!(info.doc_url.is_none());
        assert_eq!(info.source, ProviderSource::Builtin);
    }

    #[test]
    fn test_model_metadata_serde_roundtrip() {
        let meta = ModelMetadata {
            id: "test-model".to_string(),
            provider: "test".to_string(),
            context_limit: Some(200_000),
            output_limit: Some(8_192),
            pricing: Some(ModelPricing {
                input_per_1m: Some(3.0),
                output_per_1m: Some(15.0),
                cache_read_per_1m: Some(0.3),
            }),
            capabilities: ModelCapabilities {
                tools: true,
                structured_output: false,
                reasoning: true,
                image_input: true,
            },
            release_date: Some("2025-01-01".to_string()),
            variants: vec![],
            source: ModelSource::ModelsDev {
                refreshed_at: "2025-07-03T00:00:00Z".to_string(),
            },
        };

        let json = serde_json::to_string(&meta).expect("serialize");
        let roundtrip: ModelMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(meta.id, roundtrip.id);
        assert_eq!(meta.provider, roundtrip.provider);
        assert_eq!(meta.context_limit, roundtrip.context_limit);
        assert_eq!(meta.output_limit, roundtrip.output_limit);
        assert_eq!(meta.capabilities, roundtrip.capabilities);
        assert_eq!(meta.source, roundtrip.source);
    }

    #[test]
    fn image_input_capability_supported_when_metadata_image_input_true() {
        let metadata = ModelMetadata {
            id: "test-model".into(),
            provider: "test".into(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: ModelCapabilities {
                image_input: true,
                ..Default::default()
            },
            release_date: None,
            source: ModelSource::default(),
            variants: vec![],
        };
        let cap = ImageInputCapability::from_metadata(Some(&metadata));
        assert_eq!(cap, ImageInputCapability::Supported);
        assert!(cap.allows_attachment());
    }

    #[test]
    fn image_input_capability_unsupported_when_metadata_image_input_false() {
        let metadata = ModelMetadata {
            id: "test-model".into(),
            provider: "test".into(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: ModelCapabilities {
                image_input: false,
                ..Default::default()
            },
            release_date: None,
            source: ModelSource::default(),
            variants: vec![],
        };
        let cap = ImageInputCapability::from_metadata(Some(&metadata));
        assert_eq!(cap, ImageInputCapability::Unsupported);
        assert!(!cap.allows_attachment());
    }

    #[test]
    fn image_input_capability_unknown_when_no_metadata() {
        let cap = ImageInputCapability::from_metadata(None);
        assert_eq!(cap, ImageInputCapability::Unknown);
        assert!(!cap.allows_attachment());
    }
}

/// A named invocation preset (ADR-048). Lives in talos-core to avoid
/// a talos-config dependency from talos-conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct VariantDef {
    /// Stable identifier, e.g. "default", "high-reasoning".
    pub id: String,
    /// Display label, e.g. "High Reasoning".
    pub label: String,
    /// Optional reasoning effort override.
    #[serde(default)]
    pub reasoning_effort: Option<ReasoningEffort>,
}
