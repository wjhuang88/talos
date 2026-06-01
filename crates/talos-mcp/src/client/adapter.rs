//! MCP-to-agent tool adapter.

use std::sync::Arc;

use async_trait::async_trait;
use rmcp::model::Tool;
use serde_json::Value;
use talos_core::message::ToolCall;
use talos_core::tool::{AgentTool, ToolResult};
use talos_plugin::{HookEvent, HookRegistry, ToolObservation};

use crate::client::dispatcher::McpDispatcher;
use crate::client::facade;

/// MCP remote tool metadata stored for adapter construction.
#[derive(Clone)]
pub struct McpRemoteTool {
    /// MCP server name.
    pub server: String,
    /// Original rmcp tool descriptor.
    pub original: Tool,
}

/// A small adapter that bridges MCP remote tools into Talos `AgentTool`.
pub struct McpToolAdapter {
    prefixed_name: String,
    description: String,
    schema: Value,
    read_only: bool,
    remote: McpRemoteTool,
    dispatcher: Arc<McpDispatcher>,
    hook_registry: Arc<HookRegistry>,
}

impl McpToolAdapter {
    /// Creates a new MCP tool adapter from remote metadata.
    pub fn new(
        remote: McpRemoteTool,
        dispatcher: Arc<McpDispatcher>,
        hook_registry: Arc<HookRegistry>,
    ) -> Option<Self> {
        let name = facade::tool_name(&remote.original)?;
        let prefixed_name = format!("mcp:{}:{}", remote.server, name);
        let description = facade::tool_description(&remote.original).unwrap_or_else(|| {
            format!("Remote MCP tool '{}' from server '{}'", name, remote.server)
        });
        let schema = facade::tool_input_schema(&remote.original);
        let read_only = facade::tool_is_read_only(&remote.original);

        Some(Self {
            prefixed_name,
            description,
            schema,
            read_only,
            remote,
            dispatcher,
            hook_registry,
        })
    }

    fn original_name(&self) -> Option<String> {
        facade::tool_name(&self.remote.original)
    }
}

#[async_trait]
impl AgentTool for McpToolAdapter {
    fn name(&self) -> &str {
        &self.prefixed_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> Value {
        self.schema.clone()
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let call = ToolCall {
            id: "mcp-adapter-call".to_string(),
            name: self.prefixed_name.clone(),
            input: input.clone(),
        };

        let ctx = talos_plugin::HookContext::new(
            talos_plugin::TurnId::new(),
            std::path::PathBuf::from("."),
        );
        let _ = self
            .hook_registry
            .dispatch(
                &ctx,
                HookEvent::OnToolCallProposed {
                    call: &call,
                },
            )
            .await;

        let result = match self.original_name() {
            Some(original_name) => match self.dispatcher.call_tool(&original_name, input).await {
                Ok(payload) => {
                    let content = facade::call_result_to_text(payload);
                    tracing::info!(tool = %self.prefixed_name, content = %content, "MCP remote tool call succeeded");
                    ToolResult::success(content)
                }
                Err(error) => {
                    tracing::warn!(tool = %self.prefixed_name, %error, "MCP remote tool call failed");
                    ToolResult::error(error.to_string())
                }
            },
            None => ToolResult::error("MCP tool missing name"),
        };

        let observation = ToolObservation {
            call,
            result: result.clone(),
        };
        let _ = self
            .hook_registry
            .dispatch(
                &ctx,
                HookEvent::OnToolResultObserved {
                    observation: &observation,
                },
            )
            .await;

        result
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn provenance(&self) -> talos_core::tool::ToolProvenance {
        talos_core::tool::ToolProvenance::McpRemote {
            server: self.remote.server.clone(),
        }
    }
}
