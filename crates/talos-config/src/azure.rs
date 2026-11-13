//! Explicit Azure transport configuration; never discovers credentials or endpoints.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Azure message body, independent from deployment naming.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum AzureBodyShape {
    /// Chat Completions body and SSE events.
    #[default]
    Chat,
    /// Stateless Responses body and SSE events.
    Responses,
}

/// Azure URL convention; selecting a mode never triggers a fallback.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum AzureApiMode {
    /// Current v1 API with deployment in body `model`.
    #[default]
    V1,
    /// Dated deployment Chat API with an explicit service version.
    Deployment {
        /// Explicit API version, for example `2024-10-21`.
        api_version: String,
    },
}

/// Credential selection without ambient account discovery.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum AzureAuthentication {
    /// Existing configured API-key source, sent as `api-key` only.
    #[default]
    ApiKey,
    /// Externally acquired token read from this process's environment at dispatch.
    EntraToken {
        /// Environment variable name, not the token itself.
        token_env: String,
    },
}

/// Explicit resource binding for Azure; no catalog entry can supply this authority.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AzureTransportConfig {
    /// Resource origin, using HTTPS or literal loopback HTTP for local fixtures.
    pub endpoint: String,
    /// URL convention. Defaults to v1.
    #[serde(default)]
    pub api: AzureApiMode,
    /// Provider-level body choice; omission permits separately reviewed model evidence.
    #[serde(default)]
    pub shape: Option<AzureBodyShape>,
    /// Explicit credential mechanism.
    #[serde(default)]
    pub authentication: AzureAuthentication,
}

impl std::fmt::Debug for AzureTransportConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureTransportConfig")
            .field("endpoint", &"[redacted]")
            .field("shape", &self.shape)
            .finish_non_exhaustive()
    }
}

/// Safe validation error that never embeds endpoint or credential input.
#[derive(Debug, thiserror::Error)]
#[error("invalid Azure transport configuration: {0}")]
pub struct AzureTransportError(&'static str);

impl AzureTransportConfig {
    /// Resolve model body/deployment overrides without acquiring credentials.
    pub fn resolve_model(
        &self,
        logical_model: &str,
        model: Option<&crate::ModelConfig>,
    ) -> Result<(AzureBodyShape, String, url::Url), AzureTransportError> {
        if model.is_some_and(|m| m.base_url.is_some()) {
            return Err(AzureTransportError("model endpoint override is forbidden"));
        }
        if let AzureAuthentication::EntraToken { token_env } = &self.authentication {
            if token_env.is_empty()
                || !token_env.bytes().enumerate().all(|(i, b)| {
                    b == b'_' || b.is_ascii_alphabetic() || i > 0 && b.is_ascii_digit()
                })
            {
                return Err(AzureTransportError(
                    "invalid token environment variable name",
                ));
            }
        }
        let shape = model
            .and_then(|m| m.azure_shape)
            .or(self.shape)
            .unwrap_or_default();
        let deployment = model
            .and_then(|m| m.azure_deployment.as_deref())
            .unwrap_or(logical_model);
        Ok((
            shape,
            deployment.into(),
            self.request_url(shape, deployment)?,
        ))
    }

    /// Construct the full request URL without reading or transmitting credentials.
    pub fn request_url(
        &self,
        shape: AzureBodyShape,
        deployment: &str,
    ) -> Result<url::Url, AzureTransportError> {
        let mut url = url::Url::parse(&self.endpoint)
            .map_err(|_| AzureTransportError("expected resource origin"))?;
        let loopback = match url.host() {
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            _ => false,
        };
        if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(AzureTransportError(
                "expected HTTPS resource origin without path, credentials, query or fragment",
            ));
        }
        if !identifier(deployment) {
            return Err(AzureTransportError("invalid deployment identifier"));
        }
        match &self.api {
            AzureApiMode::V1 => url.set_path(match shape {
                AzureBodyShape::Chat => "/openai/v1/chat/completions",
                AzureBodyShape::Responses => "/openai/v1/responses",
            }),
            AzureApiMode::Deployment { api_version } => {
                if shape != AzureBodyShape::Chat || !identifier(api_version) {
                    return Err(AzureTransportError(
                        "deployment mode requires Chat and an explicit API version",
                    ));
                }
                url.set_path(&format!(
                    "/openai/deployments/{deployment}/chat/completions"
                ));
                url.query_pairs_mut()
                    .append_pair("api-version", api_version);
            }
        }
        Ok(url)
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(endpoint: &str) -> AzureTransportConfig {
        AzureTransportConfig {
            endpoint: endpoint.into(),
            api: AzureApiMode::V1,
            shape: None,
            authentication: AzureAuthentication::ApiKey,
        }
    }

    #[test]
    fn routes_are_distinct_and_deployment_is_not_a_path() {
        let mut p = profile("https://example.openai.azure.com");
        assert_eq!(
            p.request_url(AzureBodyShape::Responses, "deployment-1")
                .expect("URL")
                .as_str(),
            "https://example.openai.azure.com/openai/v1/responses"
        );
        p.api = AzureApiMode::Deployment {
            api_version: "2024-10-21".into(),
        };
        assert_eq!(
            p.request_url(AzureBodyShape::Chat, "deployment-1")
                .expect("URL")
                .as_str(),
            "https://example.openai.azure.com/openai/deployments/deployment-1/chat/completions?api-version=2024-10-21"
        );
        assert!(
            p.request_url(AzureBodyShape::Responses, "deployment-1")
                .is_err()
        );
        for id in ["", "../x", "x/y", "x?key=secret", "x%2fy"] {
            assert!(p.request_url(AzureBodyShape::Chat, id).is_err());
        }
    }

    #[test]
    fn rejects_credential_relocation_and_secret_bearing_urls() {
        for endpoint in [
            "http://example.com",
            "https://user:secret@example.com",
            "https://example.com/path",
            "https://example.com?key=secret",
            "https://example.com#secret",
        ] {
            let error = profile(endpoint)
                .request_url(AzureBodyShape::Chat, "model")
                .expect_err("invalid origin");
            assert!(!error.to_string().contains("secret"));
        }
        assert!(
            profile("http://127.0.0.1:1234")
                .request_url(AzureBodyShape::Chat, "model")
                .is_ok()
        );
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::{Config, ProviderConfig};

    #[test]
    fn old_config_omits_azure_and_new_profile_round_trips() {
        let old: ProviderConfig = toml::from_str("protocol = 'openai-chat'").expect("old config");
        assert!(old.azure.is_none());
        let text = r#"
provider = "azure-test"
model = "logical-model"
[providers.azure-test.azure]
endpoint = "https://resource.openai.azure.com"
shape = "responses"
[providers.azure-test.azure.authentication]
kind = "entra-token"
token_env = "AZURE_ACCESS_TOKEN"
[providers.azure-test.models.logical-model]
azure_deployment = "production-deployment"
azure_shape = "chat"
"#;
        let config: Config = toml::from_str(text).expect("Azure config");
        let p = config.active_provider_config();
        assert_eq!(
            p.azure.as_ref().expect("profile").shape,
            Some(AzureBodyShape::Responses)
        );
        assert_eq!(
            p.models["logical-model"].azure_shape,
            Some(AzureBodyShape::Chat)
        );
        assert_eq!(
            p.models["logical-model"].azure_deployment.as_deref(),
            Some("production-deployment")
        );
        let restored: Config =
            toml::from_str(&toml::to_string(&config).expect("serialize")).expect("restore");
        assert_eq!(restored.active_provider_config().azure, p.azure);
    }
}

#[cfg(test)]
mod resolution_tests {
    use super::*;
    #[test]
    fn model_override_cannot_relocate_resource_or_acquire_credentials() {
        let mut profile = AzureTransportConfig {
            endpoint: "https://example.com".into(),
            api: AzureApiMode::V1,
            shape: Some(AzureBodyShape::Responses),
            authentication: AzureAuthentication::ApiKey,
        };
        let mut model = crate::ModelConfig {
            azure_shape: Some(AzureBodyShape::Chat),
            azure_deployment: Some("production".into()),
            ..Default::default()
        };
        let (shape, deployment, url) = profile
            .resolve_model("logical", Some(&model))
            .expect("resolve");
        assert_eq!(shape, AzureBodyShape::Chat);
        assert_eq!(deployment, "production");
        assert_eq!(url.path(), "/openai/v1/chat/completions");
        assert_eq!(
            profile.resolve_model("logical", None).expect("fallback").0,
            AzureBodyShape::Responses
        );
        model.base_url = Some("https://other.example.com".into());
        assert!(profile.resolve_model("logical", Some(&model)).is_err());
        profile.authentication = AzureAuthentication::EntraToken {
            token_env: "INVALID=TOKEN".into(),
        };
        assert!(profile.resolve_model("logical", None).is_err());
        profile.authentication = AzureAuthentication::EntraToken {
            token_env: "TALOS_UNSET_AZURE_TEST".into(),
        };
        assert!(profile.resolve_model("logical", None).is_ok());
    }
}
