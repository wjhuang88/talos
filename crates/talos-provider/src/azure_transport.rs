//! Explicit Azure destination and exclusive credential construction.
use reqwest::{Client, RequestBuilder, header::HeaderValue};
use serde_json::{Value, json};
use talos_config::{AzureAuthentication, AzureTransportConfig, ModelConfig};
use talos_core::provider::{ProviderError, ProviderResult};

#[derive(Clone)]
pub(crate) struct AzureConnection {
    pub(crate) endpoint: String,
    deployment: String,
    authentication: AzureAuthentication,
}

impl AzureConnection {
    pub(crate) fn new(
        profile: &AzureTransportConfig,
        model: &str,
        settings: Option<&ModelConfig>,
    ) -> ProviderResult<Self> {
        let (_, deployment, endpoint) = profile
            .resolve_model(model, settings)
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;
        Ok(Self {
            endpoint: endpoint.to_string(),
            deployment,
            authentication: profile.authentication.clone(),
        })
    }

    pub(crate) fn identity(&self) -> String {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        for field in [self.endpoint.as_bytes(), self.deployment.as_bytes()] {
            digest.update((field.len() as u64).to_be_bytes());
            digest.update(field);
        }
        format!(
            "azure-v1:{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest.finalize())
        )
    }

    pub(crate) fn request(
        &self,
        client: &Client,
        api_key: &str,
        body: &Value,
    ) -> ProviderResult<RequestBuilder> {
        let (name, credential) = match &self.authentication {
            AzureAuthentication::ApiKey => ("api-key", api_key.to_owned()),
            AzureAuthentication::EntraToken { token_env } => {
                let token = std::env::var(token_env).map_err(|_| invalid_credential())?;
                if token.is_empty() || token.len() > 16384 {
                    return Err(invalid_credential());
                }
                ("Authorization", format!("Bearer {token}"))
            }
            _ => return Err(invalid_credential()),
        };
        if credential.is_empty() || credential.len() > 16400 {
            return Err(invalid_credential());
        }
        let mut header = HeaderValue::from_str(&credential).map_err(|_| invalid_credential())?;
        header.set_sensitive(true);
        let mut body = body.clone();
        body["model"] = json!(self.deployment);
        Ok(client.post(&self.endpoint).header(name, header).json(&body))
    }

    pub(crate) fn preview(&self, preview: &mut Value) {
        preview["body"]["model"] = json!(self.deployment);
        preview["headers"] = match self.authentication {
            AzureAuthentication::ApiKey => {
                json!({"Content-Type":"application/json", "api-key":"***"})
            }
            _ => json!({"Content-Type":"application/json", "Authorization":"Bearer ***"}),
        };
    }
}

fn invalid_credential() -> ProviderError {
    ProviderError::InvalidResponse("Azure credential is missing or invalid".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_config::{AzureApiMode, AzureBodyShape};
    #[test]
    fn destination_identity_and_headers_are_separate_from_logical_model() {
        let profile = AzureTransportConfig {
            endpoint: "https://resource.example.com".into(),
            api: AzureApiMode::V1,
            shape: Some(AzureBodyShape::Chat),
            authentication: AzureAuthentication::ApiKey,
        };
        let connection = AzureConnection::new(
            &profile,
            "logical",
            Some(&ModelConfig {
                azure_deployment: Some("production".into()),
                ..Default::default()
            }),
        )
        .expect("profile");
        let original = json!({"model":"logical"});
        let request = connection
            .request(&Client::new(), "fixture-key", &original)
            .expect("request")
            .build()
            .expect("build");
        assert_eq!(request.headers()["api-key"], "fixture-key");
        assert!(request.headers()["api-key"].is_sensitive());
        assert!(!request.headers().contains_key("authorization"));
        let body: Value =
            serde_json::from_slice(request.body().expect("body").as_bytes().expect("bytes"))
                .expect("JSON");
        assert_eq!(body["model"], "production");
        assert_eq!(original["model"], "logical");
        let mut other = connection.clone();
        other.deployment = "different".into();
        assert_ne!(connection.identity(), other.identity());
        let mut preview = json!({"body":original});
        connection.preview(&mut preview);
        assert_eq!(preview["headers"]["api-key"], "***");
        assert!(
            connection
                .request(&Client::new(), "bad\r\nheader", &json!({}))
                .is_err()
        );
    }
}
