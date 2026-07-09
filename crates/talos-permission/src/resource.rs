//! Resource kinds and extraction for permission rules.
//!
//! Defines how resource strings are interpreted (path, domain, command, remote)
//! and extracts resource values from tool input based on [`ToolNature`].

use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{ToolNature, ToolResourceKind};
use url::Url;

/// How to interpret the `resource` field in a [`PermissionRule`].
///
/// Determines whether the resource string is treated as a file path glob
/// or a URL host pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    /// Glob matched against a file path (Read, Write, Execute tools).
    Path,
    /// Glob or exact match against a URL host (Network tools).
    Domain,
    /// Glob matched against an executable or command token.
    Command,
    /// Glob matched against a named remote resource.
    Remote,
}

impl From<ToolResourceKind> for ResourceKind {
    fn from(value: ToolResourceKind) -> Self {
        match value {
            ToolResourceKind::Path => Self::Path,
            ToolResourceKind::Domain => Self::Domain,
            ToolResourceKind::Command => Self::Command,
            ToolResourceKind::Remote => Self::Remote,
        }
    }
}

/// Extracts resource strings from tool input based on [`ToolNature`].
///
/// Each nature maps to specific input fields:
/// - Read/Write → `input["path"]`, `input["file"]`, or `input["destination"]`
/// - Execute → first whitespace-delimited token of `input["command"]`
///   (e.g., `scripts/deploy.sh --arg` → `scripts/deploy.sh`)
/// - Network → host from `input["url"]` (lowercase, no port)
pub struct ResourceExtractor;

impl ResourceExtractor {
    /// Extracts the resource string from tool input based on the tool's nature.
    ///
    /// Returns `None` when the expected field is missing or (for Network)
    /// when the URL cannot be parsed.
    pub fn extract(nature: ToolNature, input: &Value) -> Option<String> {
        match nature {
            ToolNature::Read | ToolNature::Write => input
                .get("path")
                .or_else(|| input.get("file"))
                .or_else(|| input.get("destination"))
                .and_then(Value::as_str)
                .map(String::from),
            ToolNature::Execute => input
                .get("command")
                .and_then(Value::as_str)
                .and_then(|cmd| cmd.split_whitespace().next().map(String::from)),
            ToolNature::Network => input
                .get("url")
                .and_then(Value::as_str)
                .and_then(|url_str| {
                    Url::parse(url_str)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
                }),
            ToolNature::Internal => None,
        }
    }
}
