//! MCP client manager.

use std::sync::Arc;

use talos_core::tool::AgentTool;
use talos_plugin::HookRegistry;
use tokio::process::Child;

use crate::client::adapter::{McpRemoteTool, McpToolAdapter};
use crate::client::dispatcher::McpDispatcher;
use crate::client::transport::{McpTransport, build_remote_headers, spawn_stdio_transport};
use crate::error::{McpError, Result};
use crate::types::{McpClientConfig, McpServerLaunchConfig, McpServerStatus, McpToolDescriptor};

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
    tools: Vec<McpToolDescriptor>,
    _child: Option<Child>,
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
                Ok(mut client) => match client.dispatcher.initialize().await {
                    Ok(()) => match client.dispatcher.list_tools().await {
                        Ok(tools) => {
                            client.tools = tools;
                            clients.push(client);
                        }
                        Err(error) => startup_failures.push(McpStartupFailure {
                            server: server.name.clone(),
                            error: error.to_string(),
                        }),
                    },
                    Err(error) => {
                        startup_failures.push(McpStartupFailure {
                            server: server.name.clone(),
                            error: error.to_string(),
                        });
                    }
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
        match server.transport.as_str() {
            "stdio" => Self::start_stdio(server).await,
            "streamable_http" | "http" => Self::start_streamable_http(server).await,
            "sse" => Self::start_legacy_sse(server).await,
            unsupported => Err(McpError::InvalidConfig(format!(
                "server '{}' has unsupported transport '{}'; supported transports are stdio, sse, streamable_http",
                server.name, unsupported
            ))),
        }
    }

    async fn start_stdio(server: &McpServerLaunchConfig) -> Result<ManagedClient> {
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
            tools: Vec::new(),
            _child: Some(child),
        })
    }

    async fn start_streamable_http(server: &McpServerLaunchConfig) -> Result<ManagedClient> {
        let url = server.url.as_deref().ok_or_else(|| {
            McpError::InvalidConfig(format!(
                "server '{}' missing URL for streamable_http transport",
                server.name
            ))
        })?;
        let headers = build_remote_headers(
            &server.headers,
            server.auth_token_env.as_deref(),
            server.authorization_env.as_deref(),
        )?;
        let transport =
            McpTransport::streamable_http(server.name.clone(), url.to_string(), headers)?;
        let dispatcher = Arc::new(McpDispatcher::new(server.name.clone(), transport));
        Ok(ManagedClient {
            dispatcher,
            tools: Vec::new(),
            _child: None,
        })
    }

    async fn start_legacy_sse(server: &McpServerLaunchConfig) -> Result<ManagedClient> {
        let url = server.url.as_deref().ok_or_else(|| {
            McpError::InvalidConfig(format!(
                "server '{}' missing URL for sse transport",
                server.name
            ))
        })?;
        let headers = build_remote_headers(
            &server.headers,
            server.auth_token_env.as_deref(),
            server.authorization_env.as_deref(),
        )?;
        let transport = McpTransport::connect_legacy_sse(
            server.name.clone(),
            url.to_string(),
            server.sse_post_url.clone(),
            headers,
        )
        .await?;
        let dispatcher = Arc::new(McpDispatcher::new(server.name.clone(), transport));
        Ok(ManagedClient {
            dispatcher,
            tools: Vec::new(),
            _child: None,
        })
    }

    /// Returns startup failures that did not abort manager construction.
    pub fn startup_failures(&self) -> &[McpStartupFailure] {
        &self.startup_failures
    }

    /// Returns a startup-stable status snapshot for all configured servers.
    pub fn server_statuses(&self) -> Vec<McpServerStatus> {
        let mut statuses = self
            .clients
            .iter()
            .map(|client| McpServerStatus {
                server: client.dispatcher.server().to_string(),
                connected: true,
                tool_count: client.tools.len(),
                error: None,
            })
            .collect::<Vec<_>>();
        statuses.extend(self.startup_failures.iter().map(|failure| McpServerStatus {
            server: failure.server.clone(),
            connected: false,
            tool_count: 0,
            error: Some(failure.error.clone()),
        }));
        statuses.sort_by(|left, right| left.server.cmp(&right.server));
        statuses
    }

    /// Discovers all MCP tools and returns Talos tool adapters.
    pub async fn discover_tools(&self) -> Vec<Arc<dyn AgentTool>> {
        let mut tools: Vec<Arc<dyn AgentTool>> = Vec::new();

        for client in &self.clients {
            for original in client.tools.clone() {
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

        tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn invalid_server_is_reported_without_failing_manager_startup() {
        let config = McpClientConfig {
            servers: vec![McpServerLaunchConfig {
                name: "broken".to_string(),
                transport: "stdio".to_string(),
                command: String::new(),
                args: Vec::new(),
                env: HashMap::new(),
                cwd: None,
                ..McpServerLaunchConfig::default()
            }],
        };

        let manager = McpClientManager::start(&config, Arc::new(HookRegistry::new()))
            .await
            .expect("manager startup should degrade per server");

        assert!(manager.discover_tools().await.is_empty());
        assert_eq!(manager.server_statuses().len(), 1);
        let status = &manager.server_statuses()[0];
        assert_eq!(status.server, "broken");
        assert!(!status.connected);
        assert!(
            status
                .error
                .as_deref()
                .is_some_and(|error| error.contains("missing"))
        );
    }

    #[tokio::test]
    async fn remote_server_missing_url_is_reported_without_failing_manager_startup() {
        let config = McpClientConfig {
            servers: vec![McpServerLaunchConfig {
                name: "remote".to_string(),
                transport: "streamable_http".to_string(),
                ..McpServerLaunchConfig::default()
            }],
        };

        let manager = McpClientManager::start(&config, Arc::new(HookRegistry::new()))
            .await
            .expect("manager startup should degrade per server");

        let status = &manager.server_statuses()[0];
        assert_eq!(status.server, "remote");
        assert!(!status.connected);
        assert!(
            status
                .error
                .as_deref()
                .is_some_and(|error| error.contains("missing URL"))
        );
    }
}
