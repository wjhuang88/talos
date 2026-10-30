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
    let mut resolution = metadata.map_or_else(
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

    if config.resolve_model_fields().reasoning_supported.value == Some(false)
        && let Some(model) = runtime_config
            .providers
            .get_mut(&runtime_config.provider)
            .and_then(|provider| provider.models.get_mut(&runtime_config.model))
    {
        if model.reasoning.is_some() {
            append_diagnostic(
                &mut resolution,
                "Reasoning options and replay omitted: resolved capability is disabled.",
            );
        }
        model.reasoning = None;
    }

    if let Some(options) = runtime_config
        .providers
        .get(&runtime_config.provider)
        .and_then(|provider| provider.models.get(&runtime_config.model))
        .and_then(|model| model.reasoning.as_ref())
    {
        use crate::ProviderProtocol;
        match config
            .resolve_model_protocol(&config.provider, &config.model)
            .protocol
        {
            ProviderProtocol::OpenAIChat | ProviderProtocol::OpenAIResponses
                if options.budget_tokens.is_some() =>
            {
                append_diagnostic(
                    &mut resolution,
                    "Reasoning budget_tokens omitted: this protocol uses effort.",
                );
            }
            ProviderProtocol::AnthropicMessages if options.effort.is_some() => {
                append_diagnostic(
                    &mut resolution,
                    "Reasoning effort omitted: this protocol uses a thinking budget.",
                );
            }
            _ => {}
        }
    }
    (runtime_config, resolution)
}

fn append_diagnostic(resolution: &mut VariantResolution, message: &str) {
    match resolution.diagnostic.as_mut() {
        Some(existing) => {
            existing.push(' ');
            existing.push_str(message);
        }
        None => resolution.diagnostic = Some(message.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ModelConfig, ProviderConfig};
    use std::collections::HashMap;

    fn configured(model: &str, supported: Option<bool>) -> Config {
        Config {
            provider: "gateway".into(),
            model: model.into(),
            providers: HashMap::from([(
                "gateway".into(),
                ProviderConfig {
                    models: HashMap::from([(
                        model.into(),
                        ModelConfig {
                            reasoning_supported: supported,
                            reasoning: Some(ReasoningOptions {
                                effort: Some(ReasoningEffort::High),
                                budget_tokens: Some(2048),
                                replay: true,
                            }),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        }
    }

    #[test]
    fn negative_reasoning_materialization_preserves_persisted_options() {
        let config = configured("unknown-model014-fixture", Some(false));
        let before = toml::to_string(&config).expect("serialize");
        let (runtime, _) = materialize_runtime_model_config(&config);
        assert!(
            runtime.providers["gateway"]
                .models
                .get(&runtime.model)
                .expect("model")
                .reasoning
                .is_none()
        );
        assert!(
            config.providers["gateway"]
                .models
                .get(&config.model)
                .expect("model")
                .reasoning
                .is_some()
        );
        assert_eq!(toml::to_string(&config).expect("serialize"), before);
    }

    #[test]
    fn materialization_reports_disabled_and_protocol_omissions_without_persisting() {
        let config = configured("unknown-model014-fixture", Some(false));
        let (_, resolution) = materialize_runtime_model_config(&config);
        assert!(
            resolution
                .diagnostic
                .expect("disabled diagnostic")
                .contains("capability is disabled")
        );
        for (protocol, expected) in [
            (crate::ProviderProtocol::OpenAIChat, "budget_tokens omitted"),
            (
                crate::ProviderProtocol::OpenAIResponses,
                "budget_tokens omitted",
            ),
            (crate::ProviderProtocol::AnthropicMessages, "effort omitted"),
        ] {
            let mut config = configured("unknown-model014-fixture", Some(true));
            let provider = config.providers.get_mut("gateway").expect("provider");
            provider.protocol = protocol;
            let options = provider
                .models
                .get_mut("unknown-model014-fixture")
                .expect("model")
                .reasoning
                .as_mut()
                .expect("options");
            options.effort = Some(ReasoningEffort::High);
            options.budget_tokens = Some(2048);
            let before = toml::to_string(&config).expect("config");
            let (_, resolution) = materialize_runtime_model_config(&config);
            let diagnostic = resolution.diagnostic.expect("protocol diagnostic");
            assert!(diagnostic.contains(expected), "{diagnostic}");
            assert!(diagnostic.len() < 200);
            assert_eq!(toml::to_string(&config).expect("config"), before);
        }
    }

    #[test]
    fn unknown_reasoning_preserves_explicit_legacy_options() {
        let config = configured("unknown-model014-fixture", None);
        let (runtime, _) = materialize_runtime_model_config(&config);
        assert_eq!(
            runtime.providers["gateway"]
                .models
                .get(&runtime.model)
                .expect("model")
                .reasoning,
            config.providers["gateway"]
                .models
                .get(&config.model)
                .expect("model")
                .reasoning
        );
    }

    #[test]
    fn catalog_negative_suppresses_options_unless_explicitly_overridden() {
        let catalog = crate::model::builtin_models();
        let model = catalog
            .iter()
            .find(|model| {
                !model.capabilities.reasoning
                    && catalog
                        .iter()
                        .filter(|candidate| candidate.id == model.id)
                        .count()
                        == 1
            })
            .expect("unique non-reasoning catalog model");
        let config = configured(&model.id, None);
        assert_eq!(
            config.resolve_model_fields().reasoning_supported.value,
            Some(false)
        );
        let (runtime, _) = materialize_runtime_model_config(&config);
        assert!(
            runtime.providers["gateway"]
                .models
                .get(&runtime.model)
                .expect("model")
                .reasoning
                .is_none()
        );
        let config = configured(&model.id, Some(true));
        let (runtime, _) = materialize_runtime_model_config(&config);
        assert!(
            runtime.providers["gateway"]
                .models
                .get(&runtime.model)
                .expect("model")
                .reasoning
                .is_some()
        );
    }
}
