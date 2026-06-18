//! Talos-owned DTOs for MCP types.
//!
//! These types replace external `rmcp::` and `talos_config::` types
//! in the public API surface, ensuring Talos controls its own ABI.

use std::collections::HashMap;
use std::path::PathBuf;

/// Talos-owned replacement for `rmcp::model::Tool`.
///
/// A descriptor for one MCP tool exposed by a remote server.
#[derive(Debug, Clone)]
pub struct McpToolDescriptor {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
    /// Hint that the tool is read-only (no side effects).
    pub read_only_hint: bool,
}

/// Talos-owned replacement for `rmcp::model::CallToolRequestParams`.
///
/// Used internally by the permission gate to evaluate tool calls.
#[derive(Debug, Clone)]
pub struct McpCallRequest {
    /// Tool name to call.
    pub name: String,
    /// Optional arguments map.
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Talos-owned replacement for `talos_config::McpServerConfig`.
///
/// Launch configuration for one MCP server.
#[derive(Debug, Clone)]
pub struct McpServerLaunchConfig {
    /// Stable MCP server name.
    pub name: String,
    /// Transport kind (`stdio` or `http`).
    pub transport: String,
    /// Executable command for stdio transport.
    pub command: String,
    /// Command arguments for stdio transport.
    pub args: Vec<String>,
    /// Environment variables for stdio transport.
    pub env: HashMap<String, String>,
    /// Working directory for stdio transport.
    pub cwd: Option<PathBuf>,
}

/// Talos-owned replacement for `talos_config::McpConfig`.
///
/// Configuration for the MCP client manager.
#[derive(Debug, Clone, Default)]
pub struct McpClientConfig {
    /// Declared MCP servers to launch.
    pub servers: Vec<McpServerLaunchConfig>,
}
