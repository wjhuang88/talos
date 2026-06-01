//! MCP client module.

pub mod adapter;
pub mod dispatcher;
pub mod facade;
pub mod manager;
pub mod transport;

pub use adapter::{McpRemoteTool, McpToolAdapter};
pub use dispatcher::McpDispatcher;
pub use manager::{McpClientManager, McpStartupFailure};
