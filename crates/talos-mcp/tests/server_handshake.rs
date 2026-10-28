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
async fn standalone_native_files_default_to_read_allow_and_write_headless_deny() {
    use rmcp::model::CallToolRequestParams;
    use talos_tools::ordinary_file_tool_contributions;

    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("document.txt");
    std::fs::write(&path, "original MCP content").expect("initial file");
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions(ordinary_file_tool_contributions(root.path().to_path_buf()))
        .expect("native file registration");
    let handler = TalosMcpHandler::new(
        Arc::new(registry),
        Arc::new(McpPermissionGate::new(
            Arc::new(PermissionEngine::new()),
            Arc::new(HookRegistry::new()),
        )),
    );
    let (client_io, server_io) = tokio::io::duplex(1024 * 64);
    let server_task = tokio::spawn(async move {
        let running = handler.serve(server_io).await.expect("server starts");
        let _ = running.waiting().await;
    });
    let client = ().serve(client_io).await.expect("client starts");
    let arguments = |value: serde_json::Value| value.as_object().expect("arguments").clone();
    let read = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("read")
                .with_arguments(arguments(serde_json::json!({"path": path}))),
        )
        .await
        .expect("default read admitted");
    assert_ne!(read.is_error, Some(true));
    assert!(
        serde_json::to_string(&read)
            .expect("response")
            .contains("original MCP content")
    );
    let destination = root.path().join("new-file.txt");
    client
        .peer()
        .call_tool(
            CallToolRequestParams::new("write").with_arguments(arguments(
                serde_json::json!({"path": destination, "content": "must not be created"}),
            )),
        )
        .await
        .expect_err("native ownership does not install a model or human resolver");
    assert!(!destination.exists(), "denied write must not create a file");
    assert_eq!(
        std::fs::read_to_string(&path).expect("unchanged file"),
        "original MCP content"
    );
    let _ = client.cancel().await;
    server_task.await.expect("server stopped");
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
        let running = handler
            .serve((server_read, server_write))
            .await
            .expect("server starts");
        let _ = running.waiting().await;
    });

    let client = ().serve((client_read, client_write)).await.expect("client starts");
    let tools = client
        .peer()
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("list tools succeeds");

    assert!(tools.tools.len() >= 5, "tool count: {}", tools.tools.len());

    let _ = client.cancel().await;
    let _ = server_task.await;
}
