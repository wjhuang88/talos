//! Static bearer transport for the reviewed public Copilot HTTP origin.
//!
//! Wire selection stays in the protocol resolver. This module neither acquires tokens nor
//! follows account/enterprise routing, and never copies vendor headers onto another origin.

use reqwest::{Client, Url, header::HeaderMap};
use serde_json::{Map, Value};
use talos_config::ProviderProtocol;
use talos_core::provider::{ProviderError, ProviderResult};

pub(crate) struct CopilotConnection {
    pub(crate) base_url: String,
    pub(crate) client: Client,
    pub(crate) preview_headers: Map<String, Value>,
}

pub(crate) fn connection(
    base_url: Option<&str>,
    protocol: &ProviderProtocol,
) -> ProviderResult<Option<CopilotConnection>> {
    let Some(url) = base_url.and_then(|value| Url::parse(value).ok()) else {
        return Ok(None);
    };
    if url.host_str() != Some("api.githubcopilot.com") {
        return Ok(None);
    }
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ProviderError::InvalidResponse(
            "public Copilot requires its HTTPS origin without credentials, query or fragment"
                .into(),
        ));
    }
    let path = url.path().trim_end_matches('/');
    let final_path = match protocol {
        ProviderProtocol::AnthropicMessages => "/v1/messages",
        ProviderProtocol::OpenAIChat => "/chat/completions",
        ProviderProtocol::OpenAIResponses => "/responses",
        _ => {
            return Err(ProviderError::InvalidResponse(
                "unsupported public Copilot wire protocol".into(),
            ));
        }
    };
    if !matches!(path, "" | "/v1") && path != final_path {
        return Err(ProviderError::InvalidResponse(
            "public Copilot URL does not match the selected wire protocol".into(),
        ));
    }
    let base_url = match protocol {
        ProviderProtocol::AnthropicMessages => "https://api.githubcopilot.com/v1",
        _ => "https://api.githubcopilot.com",
    }
    .to_string();
    let preview_headers = headers()?;
    let mut default_headers = HeaderMap::new();
    for (name, value) in &preview_headers {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| ProviderError::InvalidResponse("invalid Copilot header name".into()))?;
        let value = value
            .as_str()
            .and_then(|value| reqwest::header::HeaderValue::from_str(value).ok())
            .ok_or_else(|| ProviderError::InvalidResponse("invalid Copilot header value".into()))?;
        default_headers.insert(name, value);
    }
    let client = Client::builder()
        .default_headers(default_headers)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| {
            ProviderError::NetworkError("cannot build public Copilot HTTP client".into())
        })?;
    Ok(Some(CopilotConnection {
        base_url,
        client,
        preview_headers,
    }))
}

fn headers() -> ProviderResult<Map<String, Value>> {
    // Stable for one Talos process, never derived from a credential or a user identity.
    static INTERACTION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let interaction = match INTERACTION.get() {
        Some(interaction) => interaction,
        None => {
            let fresh = std::panic::catch_unwind(uuid::Uuid::new_v4).map_err(|_| {
                ProviderError::NetworkError(
                    "cannot generate public Copilot request identity".into(),
                )
            })?;
            INTERACTION.get_or_init(|| fresh.to_string())
        }
    };
    Ok([
        ("Copilot-Integration-Id", "talos".to_string()),
        ("X-Interaction-Id", interaction.clone()),
        ("X-GitHub-Api-Version", "2026-08-01".to_string()),
        (
            "Editor-Version",
            format!("talos/{}", env!("CARGO_PKG_VERSION")),
        ),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), Value::String(value)))
    .collect())
}

// A rejected request may echo its Authorization value in the response body.
pub(crate) fn redact_credential(message: String, credential: &str) -> String {
    if credential.is_empty() { message } else { message.replace(credential, "***") }
}
