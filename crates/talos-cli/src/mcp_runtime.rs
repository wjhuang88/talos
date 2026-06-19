//! Session-level MCP startup composition.

use std::sync::Arc;

use anyhow::Result;
use talos_config::McpConfig;
use talos_conversation::McpServerDiagnostic;
use talos_core::tool::AgentTool;
use talos_mcp::client::McpClientManager;
use talos_plugin::HookRegistry;

use crate::provider_setup::config_to_mcp_client_config;

pub(crate) struct McpSessionRuntime {
    _manager: McpClientManager,
    tools: Vec<Arc<dyn AgentTool>>,
    diagnostics: Vec<McpServerDiagnostic>,
}

impl McpSessionRuntime {
    pub(crate) async fn start(config: &McpConfig, hooks: Arc<HookRegistry>) -> Result<Self> {
        let manager = McpClientManager::start(&config_to_mcp_client_config(config), hooks).await?;
        let diagnostics = manager
            .server_statuses()
            .into_iter()
            .map(|status| McpServerDiagnostic {
                name: status.server,
                connected: status.connected,
                tool_count: status.tool_count,
                error: status.error,
            })
            .collect();
        let tools = manager.discover_tools().await;

        Ok(Self {
            _manager: manager,
            tools,
            diagnostics,
        })
    }

    pub(crate) fn tools(&self) -> &[Arc<dyn AgentTool>] {
        &self.tools
    }

    pub(crate) fn diagnostics(&self) -> &[McpServerDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn report_startup_failures(&self) {
        for status in &self.diagnostics {
            if !status.connected {
                eprintln!(
                    "Warning: MCP server '{}' failed to start: {}",
                    status.name,
                    status.error.as_deref().unwrap_or("unavailable")
                );
            }
        }
    }
}
