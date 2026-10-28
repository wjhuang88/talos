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
    #[serde(default)]
    api_base_url: Option<String>,
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
pub(crate) fn builtin_model_protocol(
    provider: &str,
    model_id: &str,
    configured_endpoint: Option<&str>,
) -> Option<ProviderProtocol> {
    let builtin = model::builtin_providers()
        .into_iter()
        .find(|entry| entry.id == provider);
    if let Some(builtin) = &builtin
        && let Some(configured) = configured_endpoint
    {
        let expected = builtin.api_base_url.as_deref().or(match provider {
            "openai" => Some("https://api.openai.com/v1"),
            "anthropic" => Some("https://api.anthropic.com/v1"),
            _ => None,
        });
        if !expected.is_some_and(|expected| {
            endpoint_root(&crate::substitute_env_vars(configured))
                == endpoint_root(&crate::substitute_env_vars(expected))
        }) {
            return None;
        }
    }
    infer_model_protocol(
        &PROTOCOL_DATASET.models,
        provider,
        model_id,
        builtin.is_some(),
    )
}

/// Endpoint evidence is provider-scoped: a unique model name on a custom gateway never authorizes
/// sending that gateway's credentials to the catalog vendor. Keep the endpoint paired to its wire.
pub(crate) fn builtin_model_endpoint(
    provider: &str,
    model_id: &str,
    configured_endpoint: Option<&str>,
    protocol: &ProviderProtocol,
) -> Option<String> {
    let builtin = model::builtin_providers()
        .into_iter()
        .find(|entry| entry.id == provider)?;
    infer_model_endpoint(
        &PROTOCOL_DATASET.models,
        provider,
        model_id,
        builtin.api_base_url.as_deref(),
        configured_endpoint,
        protocol,
    )
}

// A wire rollback must keep the selected vendor's default root even when the host did not persist
// that root explicitly. Otherwise an adapter's hard-coded official endpoint could receive a
// gateway credential. Limit this fallback to the exact reviewed model endpoint records.
pub(crate) fn builtin_model_provider_root(provider: &str, model_id: &str) -> Option<String> {
    let entry = PROTOCOL_DATASET
        .models
        .iter()
        .find(|entry| entry.provider == provider && entry.id == model_id)?;
    entry.api_base_url.as_ref()?;
    entry_protocol(entry)?;
    model::builtin_providers()
        .into_iter()
        .find(|entry| entry.id == provider)?
        .api_base_url
}

fn infer_model_endpoint(
    catalog: &[ModelProtocolEntry],
    provider: &str,
    model_id: &str,
    builtin_root: Option<&str>,
    configured_endpoint: Option<&str>,
    protocol: &ProviderProtocol,
) -> Option<String> {
    let expected = builtin_root?;
    if let Some(configured) = configured_endpoint
        && endpoint_root(&crate::substitute_env_vars(configured))
            != endpoint_root(&crate::substitute_env_vars(expected))
    {
        return None;
    }
    let entry = catalog
        .iter()
        .find(|entry| entry.provider == provider && entry.id == model_id)?;
    (entry_protocol(entry).as_ref() == Some(protocol))
        .then(|| entry.api_base_url.clone())
        .flatten()
}

fn endpoint_root(url: &str) -> &str {
    let root = url.trim().trim_end_matches('/');
    for suffix in ["/chat/completions", "/responses", "/messages"] {
        if let Some(root) = root.strip_suffix(suffix) {
            return root;
        }
    }
    root
}

fn entry_protocol(entry: &ModelProtocolEntry) -> Option<ProviderProtocol> {
    if entry
        .api_base_url
        .as_deref()
        .is_some_and(|api| !crate::protocol_signal::is_http_model_endpoint(api))
    {
        return None;
    }
    entry.protocol.clone()
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
            .and_then(entry_protocol);
    }

    let exact: Vec<_> = catalog
        .iter()
        .filter(|entry| entry.id == model_id)
        .collect();
    match exact.as_slice() {
        [entry] => return entry_protocol(entry),
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
            .then(|| entry_protocol(matches[0]))
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
            api_base_url: None,
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
    #[test]
    fn catalog_endpoint_requires_exact_owner_root_and_selected_wire() {
        let mut routed = entry("vendor", "model", Some(ProviderProtocol::OpenAIResponses));
        routed.api_base_url = Some("https://wire.vendor.test/openai/v1".into());
        let catalog = [routed];
        let root = Some("https://vendor.test/v1");
        let responses = ProviderProtocol::OpenAIResponses;
        assert_eq!(
            infer_model_endpoint(&catalog, "vendor", "model", root, root, &responses).as_deref(),
            Some("https://wire.vendor.test/openai/v1")
        );
        // Reviewed built-in metadata can select a distinct origin; unique custom model matching
        // is not connection authority. Nor can replacing a built-in gateway move its credentials.
        assert!(
            infer_model_endpoint(&catalog, "custom", "model", None, root, &responses).is_none()
        );
        assert!(
            infer_model_endpoint(
                &catalog,
                "vendor",
                "model",
                root,
                Some("https://custom.test/v1"),
                &responses
            )
            .is_none()
        );
        assert!(
            infer_model_endpoint(
                &catalog,
                "vendor",
                "model",
                root,
                root,
                &ProviderProtocol::OpenAIChat
            )
            .is_none()
        );
        assert!(
            infer_model_endpoint(&catalog, "vendor", "missing", root, root, &responses).is_none()
        );
        assert!(
            infer_model_endpoint(&catalog, "vendor", "model", None, None, &responses).is_none()
        );
    }
}
