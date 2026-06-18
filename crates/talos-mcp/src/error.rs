//! MCP error types.

use thiserror::Error;

/// Result alias used throughout the MCP crate.
pub type Result<T> = std::result::Result<T, McpError>;

/// Errors produced by MCP client operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Configuration is invalid.
    #[error("invalid MCP config: {0}")]
    InvalidConfig(String),

    /// Child process spawn failed.
    #[error("failed to spawn MCP server '{server}': {source}")]
    Spawn {
        /// Server name.
        server: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// MCP server disconnected unexpectedly.
    #[error("MCP server '{0}' disconnected")]
    Disconnected(String),

    /// JSON-RPC request/response failed.
    #[error("MCP RPC error from '{server}' method '{method}': {message}")]
    Rpc {
        /// Server name.
        server: String,
        /// Method name.
        method: String,
        /// Error message.
        message: String,
    },

    /// Serialization/deserialization failure.
    #[error("MCP JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO failure.
    #[error("MCP IO error: {0}")]
    Io(#[from] std::io::Error),

    /// MCP protocol-level error (code + message from the remote).
    #[error("MCP protocol error: {message}")]
    Protocol {
        /// Protocol error code.
        code: i64,
        /// Error message.
        message: String,
    },
}
