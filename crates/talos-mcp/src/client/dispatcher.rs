//! MCP method dispatcher.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::client::facade;
use crate::client::transport::McpTransport;
use crate::error::{McpError, Result};
use crate::types::McpToolDescriptor;

/// Method dispatcher bound to one MCP server transport.
#[derive(Clone)]
pub struct McpDispatcher {
    server: String,
    transport: Arc<McpTransport>,
}

impl McpDispatcher {
    /// Creates a dispatcher for one MCP server transport.
    pub fn new(server: String, transport: McpTransport) -> Self {
        Self {
            server,
            transport: Arc::new(transport),
        }
    }

    /// Lists tools exposed by the remote server.
    pub async fn list_tools(&self) -> Result<Vec<McpToolDescriptor>> {
        let result = self
            .transport
            .request("tools/list", Some(json!({})))
            .await
            .map_err(|error| self.method_error("tools/list", error))?;
        facade::decode_tools_list(result, &self.server)
    }

    /// Calls one remote tool by name with JSON input.
    pub async fn call_tool(&self, tool_name: &str, input: Value) -> Result<Value> {
        self.transport
            .request(
                "tools/call",
                Some(json!({
                    "name": tool_name,
                    "arguments": input,
                })),
            )
            .await
            .map_err(|error| self.method_error("tools/call", error))
    }

    /// Returns the bound server name.
    pub fn server(&self) -> &str {
        &self.server
    }

    fn method_error(&self, method: &str, error: McpError) -> McpError {
        match error {
            McpError::Timeout { .. } => error,
            _ => McpError::Rpc {
                server: self.server.clone(),
                method: method.to_string(),
                message: error.to_string(),
            },
        }
    }
}
