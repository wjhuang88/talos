//! Model/provider endpoint precedence without network discovery or implicit credential relocation.

use crate::{Config, ConfigError, ProviderProtocol, catalog_protocol::builtin_model_endpoint};

/// Evidence that selected the effective provider connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointResolutionSource {
    /// Explicit per-model connection override.
    ModelConfigured,
    /// Explicit provider connection (or the configured rollback from a different catalog wire).
    ProviderConfigured,
    /// Reviewed, packaged endpoint for this exact built-in provider/model and effective wire.
    CatalogInferred,
    /// Existing provider/adapter default.
    Default,
}

/// Effective API root and its provenance. Credentials remain owned by the selected provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEndpointResolution {
    /// API root consumed by the selected adapter; `None` retains its hard-coded default.
    pub base_url: Option<String>,
    /// Why this connection was selected.
    pub source: EndpointResolutionSource,
}

impl Config {
    /// Resolves the active model's connection without changing protocol or retrying a request.
    ///
    /// Explicit model endpoints win. Catalog endpoints require an exact built-in provider/model,
    /// the unchanged provider root, and a matching effective protocol. Custom gateways never inherit
    /// a catalog vendor's endpoint through unique model-name matching. Packaged model endpoints are
    /// reviewed connection defaults, including their credential destination, like provider defaults.
    /// A wire rollback on a routed catalog model retains its vendor root even if the host did not
    /// save a provider-level URL, instead of falling through to a different vendor's adapter default.
    ///
    /// # Errors
    ///
    /// Rejects unsupported or unresolved model endpoints before constructing an HTTP request.
    pub fn resolve_provider_endpoint(&self) -> Result<ProviderEndpointResolution, ConfigError> {
        let provider = self.providers.get(&self.provider);
        let explicit_model = provider
            .and_then(|p| p.models.get(&self.model))
            .and_then(|m| m.base_url.as_deref());
        let protocol = self.provider_protocol();
        let catalog = explicit_model
            .is_none()
            .then(|| {
                builtin_model_endpoint(
                    &self.provider,
                    &self.model,
                    provider.and_then(|p| p.base_url.as_deref()),
                    &protocol,
                )
            })
            .flatten();
        self.resolve_endpoint_with_catalog(explicit_model, catalog.as_deref(), &protocol)
    }

    fn resolve_endpoint_with_catalog(
        &self,
        explicit_model: Option<&str>,
        catalog: Option<&str>,
        protocol: &ProviderProtocol,
    ) -> Result<ProviderEndpointResolution, ConfigError> {
        if let Some(endpoint) = explicit_model.or(catalog) {
            let base_url = checked_model_endpoint(endpoint, protocol)?;
            return Ok(ProviderEndpointResolution {
                base_url: Some(base_url),
                source: if explicit_model.is_some() {
                    EndpointResolutionSource::ModelConfigured
                } else {
                    EndpointResolutionSource::CatalogInferred
                },
            });
        }
        Ok(ProviderEndpointResolution {
            base_url: match self.base_url() {
                Some(root) => Some(root),
                None => crate::catalog_protocol::builtin_model_provider_root(
                    &self.provider,
                    &self.model,
                )
                .map(|root| checked_model_endpoint(&root, protocol))
                .transpose()?,
            },
            source: if self
                .providers
                .get(&self.provider)
                .is_some_and(|p| p.base_url.is_some())
            {
                EndpointResolutionSource::ProviderConfigured
            } else {
                EndpointResolutionSource::Default
            },
        })
    }
}

fn checked_model_endpoint(
    endpoint: &str,
    protocol: &ProviderProtocol,
) -> Result<String, ConfigError> {
    let endpoint = crate::substitute_env_vars(endpoint);
    if endpoint.contains("${") {
        return Err(ConfigError::InvalidConfig(
            "model endpoint has an unresolved environment variable; configure its base URL".into(),
        ));
    }
    let normalized = crate::validate_provider_base_url(&endpoint).map_err(|reason| {
        ConfigError::InvalidConfig(format!("invalid model endpoint: {reason}"))
    })?;
    // Normalization is not protocol inference. Keep other wire roots for their selected adapter.
    Ok(
        if *protocol == ProviderProtocol::OpenAIChat
            && normalized.protocol == ProviderProtocol::OpenAIChat
        {
            normalized.base_url
        } else {
            endpoint.trim().trim_end_matches('/').to_string()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ModelConfig, ProtocolResolutionSource, ProviderConfig};

    fn configured(protocol: Option<ProviderProtocol>, model_url: Option<&str>) -> Config {
        Config {
            provider: "openai".into(),
            model: "o3".into(),
            providers: std::collections::HashMap::from([(
                "openai".into(),
                ProviderConfig {
                    base_url: Some("https://provider.test/v1".into()),
                    models: std::collections::HashMap::from([(
                        "o3".into(),
                        ModelConfig {
                            base_url: model_url.map(str::to_string),
                            protocol,
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
    fn model_override_wins_without_persisting_or_relocating_other_models() {
        let mut config = configured(
            Some(ProviderProtocol::OpenAIResponses),
            Some("https://model.test/openai/v1"),
        );
        let before = toml::to_string(&config).expect("serialize");
        let endpoint = config.resolve_provider_endpoint().expect("model route");
        assert_eq!(
            endpoint.base_url.as_deref(),
            Some("https://model.test/openai/v1")
        );
        assert_eq!(endpoint.source, EndpointResolutionSource::ModelConfigured);
        assert_eq!(
            config.provider_protocol(),
            ProviderProtocol::OpenAIResponses
        );
        config.model = "other".into();
        assert_eq!(
            config
                .resolve_provider_endpoint()
                .expect("provider route")
                .base_url
                .as_deref(),
            Some("https://provider.test/v1")
        );
        config.model = "o3".into();
        assert_eq!(toml::to_string(&config).expect("serialize"), before);
        let roundtrip: Config = toml::from_str(&before).expect("parse");
        assert_eq!(
            roundtrip.resolve_provider_endpoint().expect("restored"),
            endpoint
        );
    }

    #[test]
    fn new_model_endpoint_is_not_openai_official_protocol_evidence() {
        let mut config = configured(None, Some("https://unconfirmed.test/v1"));
        config
            .providers
            .get_mut("openai")
            .expect("provider")
            .base_url = None;
        let protocol = config.resolve_provider_protocol();
        assert_eq!(protocol.protocol, ProviderProtocol::OpenAIChat);
        assert_eq!(protocol.source, ProtocolResolutionSource::Default);
    }

    #[test]
    fn endpoint_normalization_does_not_infer_another_wire() {
        for (url, expected) in [
            (
                "https://model.test/v1/chat/completions/",
                "https://model.test/v1",
            ),
            (
                "https://model.test/anthropic/v1",
                "https://model.test/anthropic/v1",
            ),
        ] {
            let config = configured(Some(ProviderProtocol::OpenAIChat), Some(url));
            assert_eq!(
                config
                    .resolve_provider_endpoint()
                    .expect("route")
                    .base_url
                    .as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn catalog_environment_and_transport_errors_fail_before_adapter_construction() {
        let config = configured(None, None);
        for invalid in [
            "wss://example.test/realtime",
            "http://remote.test/v1",
            "http://localhost:80@remote.test/v1",
            "http://[::1]@remote.test/v1",
            "${TALOS_UNSET_ENDPOINT_008_748}/openai/v1",
        ] {
            assert!(
                config
                    .resolve_endpoint_with_catalog(
                        None,
                        Some(invalid),
                        &ProviderProtocol::OpenAIResponses
                    )
                    .is_err()
            );
        }
        for invalid in [
            "http://localhost:80@remote.test/v1",
            "http://[::1]@remote.test/v1",
        ] {
            assert!(
                configured(Some(ProviderProtocol::OpenAIResponses), Some(invalid))
                    .resolve_provider_endpoint()
                    .is_err()
            );
        }
        let route = config
            .resolve_endpoint_with_catalog(
                None,
                Some("https://catalog.test/openai/v1"),
                &ProviderProtocol::OpenAIResponses,
            )
            .expect("catalog route");
        assert_eq!(route.source, EndpointResolutionSource::CatalogInferred);
    }
}
