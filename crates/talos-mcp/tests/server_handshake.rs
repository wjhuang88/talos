use std::sync::Arc;

use async_trait::async_trait;
use rmcp::ServiceExt;
use rmcp::model::PaginatedRequestParams;
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::PermissionEngine;
use talos_plugin::HookRegistry;
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};

struct PingTool;

#[async_trait]
impl AgentTool for PingTool {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "returns pong"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }

    async fn execute(&self, _input: serde_json::Value) -> ToolResult {
        ToolResult::success("pong")
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

fn build_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new(std::path::PathBuf::from("."))));
    registry.register(Arc::new(ReadTool::new(std::path::PathBuf::from("."))));
    registry.register(Arc::new(WriteTool::new(std::path::PathBuf::from("."))));
    registry.register(Arc::new(EditTool::new(std::path::PathBuf::from("."))));
    registry.register(Arc::new(PingTool));
    registry
}

#[tokio::test]
async fn server_handshake_lists_tools() {
    let server_registry = Arc::new(build_registry());
    let gate = Arc::new(McpPermissionGate::new(
        Arc::new(PermissionEngine::new()),
        Arc::new(HookRegistry::new()),
    ));
    let handler = TalosMcpHandler::new(server_registry, gate);

    let (client_io, server_io) = tokio::io::duplex(1024 * 64);
    let (client_read, client_write) = tokio::io::split(client_io);
    let (server_read, server_write) = tokio::io::split(server_io);

    let server_task = tokio::spawn(async move {
        let running = handler.serve((server_read, server_write)).await.expect("server starts");
        let _ = running.waiting().await;
    });

    let client = ()
        .serve((client_read, client_write))
        .await
        .expect("client starts");
    let tools = client
        .peer()
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("list tools succeeds");

    assert!(tools.tools.len() >= 5, "tool count: {}", tools.tools.len());

    let _ = client.cancel().await;
    let _ = server_task.await;
}
