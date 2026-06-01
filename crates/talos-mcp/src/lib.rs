//! Talos MCP integration crate.

// I009-S3 begin
pub mod client;
pub mod error;
// I009-S3 end

pub use error::{McpError, Result};
