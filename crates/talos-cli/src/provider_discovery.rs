//! Provider model discovery (MODEL-008-B/I148).
//!
//! Protocol-specific model discovery from OpenAI-compatible and
//! Anthropic-compatible provider endpoints.

#![allow(dead_code)]

use std::time::Duration;

use serde::Deserialize;

const MAX_RESPONSE_BYTES: usize = 1_048_576;
const MAX_MODEL_COUNT: usize = 1000;
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
struct ModelsListResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

#[derive(Debug)]
pub(crate) enum DiscoveryError {
    Timeout,
    AuthFailure,
    Malformed,
    Oversize,
    Empty,
    NotSupported,
    Network(String),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "Request timed out"),
            Self::AuthFailure => write!(f, "Authentication failed"),
            Self::Malformed => write!(f, "Malformed response"),
            Self::Oversize => write!(f, "Response too large"),
            Self::Empty => write!(f, "No models returned"),
            Self::NotSupported => write!(f, "Provider does not support /models endpoint"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

pub(crate) async fn discover_provider_models(
    base_url: &str,
    api_key: &str,
    protocol: talos_config::ProviderProtocol,
) -> Result<Vec<String>, DiscoveryError> {
    let client = reqwest::Client::builder()
        .timeout(DISCOVERY_TIMEOUT)
        .build()
        .map_err(|e| DiscoveryError::Network(e.to_string()))?;

    let request = match protocol {
        talos_config::ProviderProtocol::OpenAIChat => {
            let models_url = format!("{}/models", base_url.trim_end_matches('/'));
            client
                .get(&models_url)
                .header("Authorization", format!("Bearer {api_key}"))
        }
        talos_config::ProviderProtocol::AnthropicMessages => {
            let models_url = base_url.trim_end_matches("/messages").trim_end_matches('/');
            let models_url = format!("{models_url}/models");
            client
                .get(&models_url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
        }
    };

    let response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            DiscoveryError::Timeout
        } else {
            DiscoveryError::Network(e.to_string())
        }
    })?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(DiscoveryError::AuthFailure);
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(DiscoveryError::NotSupported);
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let body = body.chars().take(200).collect::<String>();
        return Err(DiscoveryError::Network(format!("HTTP {status}: {body}")));
    }

    let content_length = response.content_length().unwrap_or(0) as usize;
    if content_length > MAX_RESPONSE_BYTES {
        return Err(DiscoveryError::Oversize);
    }

    let body = response
        .text()
        .await
        .map_err(|_| DiscoveryError::Malformed)?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(DiscoveryError::Oversize);
    }

    let parsed: ModelsListResponse =
        serde_json::from_str(&body).map_err(|_| DiscoveryError::Malformed)?;

    if parsed.data.is_empty() {
        return Err(DiscoveryError::Empty);
    }

    let model_ids: Vec<String> = parsed
        .data
        .into_iter()
        .take(MAX_MODEL_COUNT)
        .map(|m| m.id)
        .collect();

    Ok(model_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn mock_server(status: &str, content_type: &str, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = body.to_string();
        let status = status.to_string();
        let content_type = content_type.to_string();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut req_buf = [0u8; 4096];
            let _ = socket.read(&mut req_buf).await;

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
            socket.flush().await.unwrap();
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn discover_openai_chat_models_success() {
        let body = r#"{"data":[{"id":"gpt-4o"},{"id":"gpt-4o-mini"}]}"#;
        let url = mock_server("200 OK", "application/json", body).await;

        let result =
            discover_provider_models(&url, "test-key", talos_config::ProviderProtocol::OpenAIChat)
                .await;

        assert!(result.is_ok());
        let models = result.unwrap();
        assert_eq!(models, vec!["gpt-4o", "gpt-4o-mini"]);
    }

    #[tokio::test]
    async fn discover_anthropic_models_success() {
        let body =
            r#"{"data":[{"id":"claude-sonnet-4-20250514"},{"id":"claude-3-5-haiku-20241022"}]}"#;
        let url = mock_server("200 OK", "application/json", body).await;

        let result = discover_provider_models(
            &format!("{url}/messages"),
            "test-key",
            talos_config::ProviderProtocol::AnthropicMessages,
        )
        .await;

        assert!(result.is_ok());
        let models = result.unwrap();
        assert_eq!(
            models,
            vec!["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"]
        );
    }

    #[tokio::test]
    async fn discover_auth_failure() {
        let body = r#"{"error":{"message":"invalid api key"}}"#;
        let url = mock_server("401 Unauthorized", "application/json", body).await;

        let result =
            discover_provider_models(&url, "bad-key", talos_config::ProviderProtocol::OpenAIChat)
                .await;

        assert!(matches!(result, Err(DiscoveryError::AuthFailure)));
    }

    #[tokio::test]
    async fn discover_not_supported() {
        let url = mock_server("404 Not Found", "application/json", "{}").await;

        let result =
            discover_provider_models(&url, "test-key", talos_config::ProviderProtocol::OpenAIChat)
                .await;

        assert!(matches!(result, Err(DiscoveryError::NotSupported)));
    }

    #[tokio::test]
    async fn discover_empty_model_list() {
        let body = r#"{"data":[]}"#;
        let url = mock_server("200 OK", "application/json", body).await;

        let result =
            discover_provider_models(&url, "test-key", talos_config::ProviderProtocol::OpenAIChat)
                .await;

        assert!(matches!(result, Err(DiscoveryError::Empty)));
    }

    #[tokio::test]
    async fn discover_malformed_response() {
        let body = "not valid json";
        let url = mock_server("200 OK", "application/json", body).await;

        let result =
            discover_provider_models(&url, "test-key", talos_config::ProviderProtocol::OpenAIChat)
                .await;

        assert!(matches!(result, Err(DiscoveryError::Malformed)));
    }

    #[tokio::test]
    async fn discover_network_error() {
        let result = discover_provider_models(
            "http://127.0.0.1:1",
            "test-key",
            talos_config::ProviderProtocol::OpenAIChat,
        )
        .await;

        assert!(matches!(result, Err(DiscoveryError::Network(_))));
    }

    #[tokio::test]
    async fn discover_openai_chat_url_derivation() {
        let body = r#"{"data":[{"id":"test-model"}]}"#;
        let url = mock_server("200 OK", "application/json", body).await;

        let result = discover_provider_models(
            &format!("{url}/v1"),
            "test-key",
            talos_config::ProviderProtocol::OpenAIChat,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn discover_anthropic_url_derivation() {
        let body = r#"{"data":[{"id":"test-model"}]}"#;
        let url = mock_server("200 OK", "application/json", body).await;

        let result = discover_provider_models(
            &format!("{url}/v1/messages"),
            "test-key",
            talos_config::ProviderProtocol::AnthropicMessages,
        )
        .await;

        assert!(result.is_ok());
    }
}
