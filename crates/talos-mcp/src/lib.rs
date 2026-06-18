//! Talos MCP integration crate.

// I009-S3 begin
pub mod client;
pub mod error;
// I009-S3 end

// I009-S4 begin
/// MCP server implementation for exposing Talos tools.
pub mod server;
// I009-S4 end

pub mod types;

pub use error::{McpError, Result};
pub use types::{McpCallRequest, McpClientConfig, McpServerLaunchConfig, McpToolDescriptor};
