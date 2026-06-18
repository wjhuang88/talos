//! MCP client manager.

use std::sync::Arc;

use talos_core::tool::AgentTool;
use talos_plugin::HookRegistry;
use tokio::process::Child;
use tracing::warn;

use crate::client::adapter::{McpRemoteTool, McpToolAdapter};
use crate::client::dispatcher::McpDispatcher;
use crate::client::transport::spawn_stdio_transport;
use crate::error::{McpError, Result};
use crate::types::{McpClientConfig, McpServerLaunchConfig};

/// Non-fatal startup failure for one MCP server.
#[derive(Debug, Clone)]
pub struct McpStartupFailure {
    /// Server name.
    pub server: String,
    /// Error details.
    pub error: String,
}

struct ManagedClient {
    dispatcher: Arc<McpDispatcher>,
    _child: Child,
}

/// Owns MCP clients and exposes remote tools as Talos tools.
pub struct McpClientManager {
    clients: Vec<ManagedClient>,
    startup_failures: Vec<McpStartupFailure>,
    hook_registry: Arc<HookRegistry>,
}

impl McpClientManager {
    /// Starts all configured MCP clients.
    pub async fn start(config: &McpClientConfig, hook_registry: Arc<HookRegistry>) -> Result<Self> {
        let mut clients = Vec::new();
        let mut startup_failures = Vec::new();

        for server in &config.servers {
            match Self::start_one(server).await {
                Ok(client) => match client.dispatcher.list_tools().await {
                    Ok(_) => clients.push(client),
                    Err(error) => startup_failures.push(McpStartupFailure {
                        server: server.name.clone(),
                        error: error.to_string(),
                    }),
                },
                Err(error) => startup_failures.push(McpStartupFailure {
                    server: server.name.clone(),
                    error: error.to_string(),
                }),
            }
        }

        Ok(Self {
            clients,
            startup_failures,
            hook_registry,
        })
    }

    async fn start_one(server: &McpServerLaunchConfig) -> Result<ManagedClient> {
        if server.transport != "stdio" {
            // TODO: I009-future support HTTP transport.
            return Err(McpError::InvalidConfig(format!(
                "server '{}' has unsupported transport '{}'; only stdio is supported",
                server.name, server.transport
            )));
        }
        if server.command.trim().is_empty() {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' missing stdio command",
                server.name
            )));
        }

        let (transport, child) = spawn_stdio_transport(
            &server.name,
            &server.command,
            &server.args,
            &server.env,
            server.cwd.as_deref(),
        )
        .await?;

        let mut child = child;
        if let Some(status) = child.try_wait()? {
            return Err(McpError::InvalidConfig(format!(
                "server '{}' exited immediately with status {}",
                server.name, status
            )));
        }

        let dispatcher = Arc::new(McpDispatcher::new(server.name.clone(), transport));
        Ok(ManagedClient {
            dispatcher,
            _child: child,
        })
    }

    /// Returns startup failures that did not abort manager construction.
    pub fn startup_failures(&self) -> &[McpStartupFailure] {
        &self.startup_failures
    }

    /// Discovers all MCP tools and returns Talos tool adapters.
    pub async fn discover_tools(&self) -> Vec<Arc<dyn AgentTool>> {
        let mut tools: Vec<Arc<dyn AgentTool>> = Vec::new();

        for client in &self.clients {
            match client.dispatcher.list_tools().await {
                Ok(remote_tools) => {
                    for original in remote_tools {
                        let remote = McpRemoteTool {
                            server: client.dispatcher.server().to_string(),
                            original,
                        };
                        let adapter = McpToolAdapter::new(
                            remote,
                            client.dispatcher.clone(),
                            self.hook_registry.clone(),
                        );
                        tools.push(Arc::new(adapter));
                    }
                }
                Err(error) => {
                    warn!(
                        server = %client.dispatcher.server(),
                        %error,
                        "failed to discover MCP tools"
                    );
                }
            }
        }

        tools
    }
}
