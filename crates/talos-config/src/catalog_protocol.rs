//! Model-level wire-protocol evidence embedded alongside the packaged model catalog.
//!
//! `ModelMetadata` intentionally remains focused on model limits/capabilities. Protocol routing is
//! separate evidence, so this module reads the optional `protocol` key from the same committed
//! `models.toml` without widening the shared model/store schema.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::{ProviderProtocol, model};

const MODELS_TOML_ZST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/models_data.zst"));

#[derive(Debug, Deserialize)]
struct ProtocolDataset {
    #[serde(default)]
    models: Vec<ModelProtocolEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModelProtocolEntry {
    id: String,
    provider: String,
    #[serde(default)]
    protocol: Option<ProviderProtocol>,
}

static PROTOCOL_DATASET: LazyLock<ProtocolDataset> = LazyLock::new(|| {
    let toml_bytes =
        zstd::decode_all(MODELS_TOML_ZST).expect("failed to decompress embedded models.toml");
    let toml_str = String::from_utf8(toml_bytes).expect("embedded models.toml must be valid UTF-8");
    toml::from_str(&toml_str).expect("embedded model protocol metadata must be valid TOML")
});

/// Resolves confirmed model-level protocol evidence from the packaged catalog.
///
/// Built-in providers require an exact `(provider, model)` match. Custom providers follow the same
/// conservative matching rule as context-limit inference: exact bare model id first, then exactly
/// one stripped gateway prefix (`gateway/model` or `gateway:model`). Any ambiguity fails closed.
pub(crate) fn builtin_model_protocol(provider: &str, model_id: &str) -> Option<ProviderProtocol> {
    let provider_is_builtin = model::builtin_providers()
        .iter()
        .any(|entry| entry.id == provider);
    infer_model_protocol(
        &PROTOCOL_DATASET.models,
        provider,
        model_id,
        provider_is_builtin,
    )
}

fn infer_model_protocol(
    catalog: &[ModelProtocolEntry],
    provider: &str,
    model_id: &str,
    provider_is_builtin: bool,
) -> Option<ProviderProtocol> {
    if provider_is_builtin {
        return catalog
            .iter()
            .find(|entry| entry.provider == provider && entry.id == model_id)
            .and_then(|entry| entry.protocol.clone());
    }

    let exact: Vec<_> = catalog
        .iter()
        .filter(|entry| entry.id == model_id)
        .collect();
    match exact.as_slice() {
        [entry] => return entry.protocol.clone(),
        [] => {}
        _ => return None,
    }

    let normalized = model_id
        .split_once(['/', ':'])
        .and_then(|(_, remainder)| (!remainder.is_empty()).then_some(remainder));
    normalized.and_then(|normalized| {
        let matches: Vec<_> = catalog
            .iter()
            .filter(|entry| entry.id == normalized)
            .collect();
        (matches.len() == 1)
            .then(|| matches[0].protocol.clone())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(provider: &str, id: &str, protocol: Option<ProviderProtocol>) -> ModelProtocolEntry {
        ModelProtocolEntry {
            id: id.into(),
            provider: provider.into(),
            protocol,
        }
    }

    #[test]
    fn builtin_provider_requires_exact_provider_and_model() {
        let catalog = vec![
            entry("openai", "shared", Some(ProviderProtocol::OpenAIResponses)),
            entry("other", "shared", Some(ProviderProtocol::OpenAIChat)),
        ];
        assert_eq!(
            infer_model_protocol(&catalog, "openai", "shared", true),
            Some(ProviderProtocol::OpenAIResponses)
        );
        assert_eq!(
            infer_model_protocol(&catalog, "openai", "other", true),
            None
        );
    }

    #[test]
    fn custom_provider_accepts_unique_exact_model_signal() {
        let catalog = vec![entry(
            "openai",
            "o3",
            Some(ProviderProtocol::OpenAIResponses),
        )];
        assert_eq!(
            infer_model_protocol(&catalog, "gateway", "o3", false),
            Some(ProviderProtocol::OpenAIResponses)
        );
    }

    #[test]
    fn custom_provider_strips_exactly_one_gateway_prefix() {
        let catalog = vec![entry(
            "openai",
            "o3",
            Some(ProviderProtocol::OpenAIResponses),
        )];
        assert_eq!(
            infer_model_protocol(&catalog, "gateway", "upstream/o3", false),
            Some(ProviderProtocol::OpenAIResponses)
        );
        assert_eq!(
            infer_model_protocol(&catalog, "gateway", "upstream:o3", false),
            Some(ProviderProtocol::OpenAIResponses)
        );
        assert_eq!(
            infer_model_protocol(&catalog, "gateway", "a/b/o3", false),
            None
        );
    }

    #[test]
    fn ambiguous_custom_model_inherits_no_protocol() {
        let catalog = vec![
            entry("openai", "shared", Some(ProviderProtocol::OpenAIResponses)),
            entry("gateway-a", "shared", Some(ProviderProtocol::OpenAIChat)),
        ];
        assert_eq!(
            infer_model_protocol(&catalog, "custom", "shared", false),
            None
        );
        assert_eq!(
            infer_model_protocol(&catalog, "custom", "prefix/shared", false),
            None
        );
    }

    #[test]
    fn absent_or_unsignaled_model_fails_closed() {
        let catalog = vec![entry("openai", "o3", None)];
        assert_eq!(infer_model_protocol(&catalog, "custom", "o3", false), None);
        assert_eq!(
            infer_model_protocol(&catalog, "custom", "missing", false),
            None
        );
    }
}
