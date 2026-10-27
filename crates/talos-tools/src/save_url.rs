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
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
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
    recovery: crate::file_tools::FileRecoveryStore,
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
            recovery: Default::default(),
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
         file-write permission. Optional retain_backup enables recoverable overwrites of existing \
         regular files up to 2 MiB and requires Read. Restore with recovery_id and destination \
         in this tool instance within 15 minutes; url is ignored for restoration."
    }

    fn parameters(&self) -> Value {
        let mut schema = talos_core::tool_parameters!(SaveUrlInput);
        schema["properties"]["retain_backup"] = serde_json::json!({
            "type": "boolean",
            "description": "Opt in to a recoverable overwrite of an existing regular file. Before/after images must fit 2 MiB. Backup expires after 15 minutes in this tool instance."
        });
        schema["properties"]["recovery_id"] = serde_json::json!({
            "type": "string",
            "description": "Restore a previous recoverable overwrite of destination without downloading. URL is ignored. Requires read and write permission."
        });
        schema
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Write
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Network
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let mut profile = Vec::new();
        if input.get("recovery_id").is_some() {
            profile.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                input
                    .get("destination")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                ToolResourceKind::Path,
            ));
        } else if let Some(url) = input.get("url").and_then(Value::as_str)
            && let Ok(parsed) = reqwest::Url::parse(url)
            && let Some(host) = parsed.host_str()
        {
            profile.push(ToolPermissionFacet::with_resource(
                ToolNature::Network,
                host.to_lowercase(),
                ToolResourceKind::Domain,
            ));
        } else {
            profile.push(ToolPermissionFacet::new(ToolNature::Network));
        }

        if input.get("retain_backup").and_then(Value::as_bool) == Some(true)
            && input.get("recovery_id").is_none()
        {
            profile.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                input
                    .get("destination")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                ToolResourceKind::Path,
            ));
        }

        if let Some(destination) = input.get("destination").and_then(Value::as_str) {
            profile.push(ToolPermissionFacet::with_resource(
                ToolNature::Write,
                destination,
                ToolResourceKind::Path,
            ));
        } else {
            profile.push(ToolPermissionFacet::new(ToolNature::Write));
        }

        profile
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["url", "destination"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let retain_backup = match input.get("retain_backup") {
            None | Some(Value::Bool(false)) => false,
            Some(Value::Bool(true)) => true,
            _ => return ToolResult::error("retain_backup must be a boolean"),
        };
        let recovery_id = match input.get("recovery_id") {
            None => None,
            Some(Value::String(id)) if !id.is_empty() => Some(id.clone()),
            _ => return ToolResult::error("recovery_id must be a nonempty string"),
        };
        if retain_backup && recovery_id.is_some() {
            return ToolResult::error("retain_backup and recovery_id cannot be combined");
        }
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

        if let Some(id) = recovery_id {
            let path = match std::fs::canonicalize(&parsed.destination) {
                Ok(path) => path,
                Err(error) => return ToolResult::error(error.to_string()),
            };
            return match self.recovery.restore(&path, &id, None) {
                Ok(()) => ToolResult::success(format!("restored {}", parsed.destination)),
                Err(error) => ToolResult::error(error.to_string()),
            };
        }

        let captured = if retain_backup {
            let path = match std::fs::canonicalize(&parsed.destination) {
                Ok(path) => path,
                Err(error) => return ToolResult::error(error.to_string()),
            };
            let before = match crate::file_tools::read_bounded_regular_file(&path) {
                Ok((bytes, _)) => bytes,
                Err(error) => return ToolResult::error(error.to_string()),
            };
            Some((path, before))
        } else {
            None
        };

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

        let limit = if retain_backup {
            self.max_body_bytes
                .min(talos_core::tool::MAX_FILE_MUTATION_REVIEW_BYTES)
        } else {
            self.max_body_bytes
        };
        let body_bytes = match read_bounded_body(response, limit).await {
            Ok(b) => b,
            Err(e) => {
                return ToolResult::error(format!("failed to read response body: {e}"));
            }
        };

        if let Some((path, before)) = captured {
            if std::fs::canonicalize(&parsed.destination).ok().as_ref() != Some(&path) {
                return ToolResult::error("destination identity changed during download");
            }
            return match self.recovery.replace(&path, &before, &body_bytes, None) {
                Ok(id) => ToolResult::success(format!(
                    "saved {} bytes to {}; recovery_id: {id}",
                    body_bytes.len(),
                    parsed.destination
                )),
                Err(error) => ToolResult::error(error.to_string()),
            };
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

async fn read_bounded_body(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if chunk.len() > limit.saturating_sub(body.len()) {
            return Err(format!("response too large (max {limit} bytes)"));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;

#[cfg(test)]
mod recovery_tests;
