//! Credential-isolated official Gemini HTTP dispatch.

use std::time::Instant;

use reqwest::{Client, Response};
use serde_json::Value;
use talos_core::provider::{ProviderError, ProviderResult};

use crate::{gemini_auth::VertexTokenSource, gemini_endpoint};

#[derive(Clone)]
enum Authentication {
    AiStudio(reqwest::header::HeaderValue),
    Vertex(VertexTokenSource),
}

/// An admitted destination and its matching credential profile. No custom origin accepts ADC.
#[derive(Clone)]
pub(crate) struct GeminiTransport {
    endpoint: String,
    client: Client,
    authentication: Authentication,
}

fn invalid(message: &'static str) -> ProviderError {
    ProviderError::InvalidResponse(message.into())
}

impl std::fmt::Debug for GeminiTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiTransport").finish_non_exhaustive()
    }
}

impl GeminiTransport {
    fn client() -> ProviderResult<Client> {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| invalid("Gemini HTTP client initialization failed"))
    }

    pub(crate) fn ai_studio(model: &str, key: &str, stream: bool) -> ProviderResult<Self> {
        let endpoint = gemini_endpoint::ai_studio(model, stream).map_err(invalid)?;
        if key.is_empty() {
            return Err(invalid("Gemini API key is empty"));
        }
        let mut key = reqwest::header::HeaderValue::from_str(key)
            .map_err(|_| invalid("invalid Gemini API key"))?;
        key.set_sensitive(true);
        Ok(Self {
            endpoint,
            client: Self::client()?,
            authentication: Authentication::AiStudio(key),
        })
    }

    /// Validate all URL components before any environment/file credential discovery.
    /// Discovery and subsequent dispatch must receive the same caller-owned deadline.
    pub(crate) async fn vertex_adc(
        project: &str,
        location: &str,
        model: &str,
        stream: bool,
        allow_metadata: bool,
        deadline: Instant,
    ) -> ProviderResult<Self> {
        let endpoint =
            gemini_endpoint::vertex(project, location, model, stream).map_err(invalid)?;
        let source = VertexTokenSource::from_environment(allow_metadata, deadline).await?;
        Ok(Self {
            endpoint,
            client: Self::client()?,
            authentication: Authentication::Vertex(source),
        })
    }

    /// Dispatch only this profile; callers own body reading/stream deadlines and protocol retry.
    /// Token acquisition and HTTP headers share one deadline; no cross-protocol fallback occurs.
    pub(crate) async fn dispatch(
        &self,
        body: &Value,
        deadline: Instant,
    ) -> ProviderResult<Response> {
        if deadline <= Instant::now() {
            return Err(invalid("Gemini dispatch timed out"));
        }
        tokio::time::timeout_at(deadline.into(), async {
            let mut request = self.client.post(&self.endpoint).json(body);
            match &self.authentication {
                Authentication::AiStudio(key) => {
                    request = request.header("x-goog-api-key", key);
                }
                Authentication::Vertex(source) => {
                    let token = source.token(deadline).await?;
                    let mut value =
                        reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                            .map_err(|_| invalid("invalid Vertex access token"))?;
                    value.set_sensitive(true);
                    request = request.header(reqwest::header::AUTHORIZATION, value);
                }
            }
            if Instant::now() >= deadline {
                return Err(invalid("Gemini dispatch timed out"));
            }
            let response = request
                .send()
                .await
                .map_err(|_| invalid("Gemini request failed"))?;
            if !response.status().is_success() {
                // Upstream error bodies can contain request text/signatures or credentials.
                return Err(invalid(
                    "Gemini request rejected; verify protocol, endpoint and credentials",
                ));
            }
            Ok(response)
        })
        .await
        .map_err(|_| invalid("Gemini dispatch timed out"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(3)
    }

    #[tokio::test]
    async fn studio_dispatch_uses_key_header_without_bearer_or_query_secret() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/generate")
            .match_header("x-goog-api-key", "fixture-secret")
            .match_header("authorization", mockito::Matcher::Missing)
            .match_body(mockito::Matcher::Json(serde_json::json!({"contents":[]})))
            .with_body("{}")
            .create_async()
            .await;
        let mut transport = GeminiTransport::ai_studio("gemini-2.5-flash", "fixture-secret", false)
            .expect("transport");
        transport.endpoint = format!("{}/generate", server.url());
        assert!(!format!("{transport:?}").contains("fixture-secret"));
        assert_eq!(
            transport
                .dispatch(&serde_json::json!({"contents":[]}), deadline())
                .await
                .expect("response")
                .status(),
            200
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn redirect_does_not_forward_key_and_deadline_prevents_send() {
        let mut server = mockito::Server::new_async().await;
        let mut target = mockito::Server::new_async().await;
        let forbidden = target.mock("POST", "/").expect(0).create_async().await;
        let redirect = server
            .mock("POST", "/")
            .with_status(307)
            .with_header("Location", &target.url())
            .with_body("fixture-secret")
            .expect(1)
            .create_async()
            .await;
        let mut transport =
            GeminiTransport::ai_studio("model", "fixture-secret", true).expect("transport");
        transport.endpoint = server.url();
        assert!(
            transport
                .dispatch(&Value::Null, Instant::now() - Duration::from_secs(1))
                .await
                .is_err()
        );
        let error = transport
            .dispatch(&Value::Null, deadline())
            .await
            .expect_err("redirect");
        assert!(!error.to_string().contains("fixture-secret"));
        redirect.assert_async().await;
        forbidden.assert_async().await;
    }

    #[tokio::test]
    async fn invalid_vertex_destination_is_rejected_before_adc_selection() {
        let error =
            GeminiTransport::vertex_adc("project", "evil.invalid", "model", true, true, deadline())
                .await
                .expect_err("invalid resource");
        assert!(error.to_string().contains("resource component"));
        assert!(GeminiTransport::ai_studio("model", "bad\r\nkey", true).is_err());
    }
}
