//! URL-to-file save tool: downloads remote content to a local file.
//!
//! The `save_url` tool is write-capable (ToolNature::Write) and requires
//! both network and file-write permission. It reuses the same SSRF guard
//! and size limits as `http_request`.

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use thiserror::Error;

use crate::http_request::is_private_ip;

const DEFAULT_MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const DEFAULT_TIMEOUT_SECS: u64 = 60;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SaveUrlError {
    #[error("invalid save_url input: {0}")]
    InvalidInput(String),
    #[error("download failed: {0}")]
    DownloadFailed(String),
    #[error("file write failed: {0}")]
    FileWriteFailed(String),
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SaveUrlInput {
    /// The URL to download.
    pub url: String,

    /// Destination file path within the workspace.
    pub destination: String,
}

// ---------------------------------------------------------------------------
// Tool
// ---------------------------------------------------------------------------

pub struct SaveUrlTool {
    client: reqwest::Client,
    max_body_bytes: usize,
}

impl SaveUrlTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }
}

impl Default for SaveUrlTool {
    fn default() -> Self {
        Self::new()
    }
}

async fn check_ssrf(host: &str) -> Result<(), String> {
    use std::net::IpAddr;

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(ip) {
            return Err(format!("URL resolves to private IP ({ip})"));
        }
        return Ok(());
    }

    let addr = format!("{host}:0");
    let addrs = tokio::net::lookup_host(&addr)
        .await
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    for sock_addr in addrs {
        if is_private_ip(sock_addr.ip()) {
            return Err(format!(
                "{host} resolves to private IP ({})",
                sock_addr.ip()
            ));
        }
    }
    Ok(())
}

#[async_trait]
impl AgentTool for SaveUrlTool {
    fn name(&self) -> &str {
        "save_url"
    }

    fn description(&self) -> &str {
        "Download a URL and save it to a file. Saves raw bytes to the specified destination path. \
         SSRF-protected (private IPs blocked). Size limited to 10 MB. Requires network and \
         file-write permission."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(SaveUrlInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Write
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["url", "destination"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: SaveUrlInput = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(SaveUrlError::InvalidInput(e.to_string()).to_string());
            }
        };

        if parsed.destination.is_empty() {
            return ToolResult::error(
                SaveUrlError::InvalidInput("destination must not be empty".to_string()).to_string(),
            );
        }

        // Validate URL.
        let parsed_url = match reqwest::Url::parse(&parsed.url) {
            Ok(url) => url,
            Err(e) => return ToolResult::error(format!("invalid URL: {e}")),
        };

        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return ToolResult::error(format!("unsupported URL scheme '{scheme}'"));
        }

        // SSRF guard.
        let host = match parsed_url.host_str() {
            Some(h) => h.to_string(),
            None => return ToolResult::error("URL has no host".to_string()),
        };

        if let Err(e) = check_ssrf(&host).await {
            return ToolResult::error(format!("SSRF guard blocked: {e}"));
        }

        // Download.
        let response = match self.client.get(parsed.url.as_str()).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return ToolResult::error(SaveUrlError::DownloadFailed(e.to_string()).to_string());
            }
        };

        let status = response.status();
        if !status.is_success() {
            return ToolResult::error(format!("server returned status {status}"));
        }

        let body_bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return ToolResult::error(format!("failed to read response body: {e}"));
            }
        };

        if body_bytes.len() > self.max_body_bytes {
            return ToolResult::error(format!(
                "response too large ({} bytes, max {})",
                body_bytes.len(),
                self.max_body_bytes
            ));
        }

        // Write to file.
        let dest_path = PathBuf::from(&parsed.destination);
        if let Some(parent) = dest_path.parent()
            && let Err(e) = tokio::fs::create_dir_all(parent).await
        {
            return ToolResult::error(SaveUrlError::FileWriteFailed(e.to_string()).to_string());
        }

        if let Err(e) = tokio::fs::write(&dest_path, &body_bytes).await {
            return ToolResult::error(SaveUrlError::FileWriteFailed(e.to_string()).to_string());
        }

        ToolResult::success(format!(
            "saved {} bytes to {}",
            body_bytes.len(),
            parsed.destination
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        let tool = SaveUrlTool::new();
        assert_eq!(tool.name(), "save_url");
    }

    #[test]
    fn test_tool_not_read_only() {
        let tool = SaveUrlTool::new();
        assert!(!tool.is_read_only());
    }

    #[test]
    fn test_tool_nature_is_write() {
        let tool = SaveUrlTool::new();
        assert!(matches!(tool.nature(), ToolNature::Write));
    }

    #[test]
    fn test_tool_summary_fields() {
        let tool = SaveUrlTool::new();
        assert_eq!(tool.summary_fields(), &["url", "destination"]);
    }

    #[test]
    fn test_tool_emits_parameters_schema() {
        let tool = SaveUrlTool::new();
        let schema = tool.parameters();
        assert!(schema.is_object());
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_deserialize_input() {
        let json = r#"{"url": "https://example.com/file.zip", "destination": "/tmp/file.zip"}"#;
        let input: SaveUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.url, "https://example.com/file.zip");
        assert_eq!(input.destination, "/tmp/file.zip");
    }
}
