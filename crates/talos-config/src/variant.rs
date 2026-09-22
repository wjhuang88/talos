//! Shared projection of configured model variants into request options.

use crate::{Config, ReasoningOptions};
use talos_core::model::{ModelCapabilities, ReasoningEffort, VariantDef};

/// The safe runtime projection of a selected catalog variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantResolution {
    /// Reasoning effort to apply to the provider request, when supported.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Bounded note for a selected variant absent from the active model catalog.
    pub diagnostic: Option<String>,
}

/// Resolves a selected variant against the active model's catalog metadata.
///
/// The legacy `"default"` identity preserves baseline behavior without a
/// diagnostic. Reasoning overrides are silently omitted when unsupported.
pub fn resolve_variant(
    variant_id: Option<&str>,
    model_variants: &[VariantDef],
    model_capabilities: &ModelCapabilities,
) -> VariantResolution {
    let Some(variant_id) = variant_id.filter(|id| *id != "default") else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: None,
        };
    };

    let Some(variant) = model_variants
        .iter()
        .find(|variant| variant.id == variant_id)
    else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: Some(format!(
                "Variant '{variant_id}' not found; using no variant"
            )),
        };
    };

    VariantResolution {
        reasoning_effort: variant
            .reasoning_effort
            .clone()
            .filter(|_| model_capabilities.reasoning),
        diagnostic: None,
    }
}

/// Materializes the declarative provider/model/variant identity into the exact
/// effective runtime configuration consumed by Provider construction.
///
/// Persisted configuration keeps the stable variant identity separate from
/// derived request options. Every runtime reconstruction must call this helper
/// so live switching, startup, resume, new/fork and headless modes cannot
/// disagree about the first Provider request.
pub fn materialize_runtime_model_config(config: &Config) -> (Config, VariantResolution) {
    let mut runtime_config = config.clone();
    let all_models = config.all_models();
    let metadata =
        crate::model::find_model_by_provider(&all_models, &config.provider, &config.model);
    let resolution = metadata.map_or_else(
        || {
            resolve_variant(
                config.variant.as_deref(),
                &[],
                &ModelCapabilities::default(),
            )
        },
        |model| {
            resolve_variant(
                config.variant.as_deref(),
                &model.variants,
                &model.capabilities,
            )
        },
    );

    if let Some(reasoning_effort) = resolution.reasoning_effort.clone() {
        let provider = runtime_config
            .providers
            .entry(runtime_config.provider.clone())
            .or_default();
        let reasoning = provider
            .models
            .entry(runtime_config.model.clone())
            .or_default()
            .reasoning
            .get_or_insert_with(ReasoningOptions::default);
        reasoning.effort = Some(reasoning_effort);
    }

    (runtime_config, resolution)
}
