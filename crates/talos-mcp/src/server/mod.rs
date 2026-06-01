//! MCP server surface for Talos tools.

mod handler;
mod permission;

pub use handler::TalosMcpHandler;
pub use permission::McpPermissionGate;
