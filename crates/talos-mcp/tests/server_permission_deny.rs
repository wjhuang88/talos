use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, PaginatedRequestParams};
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{PermissionEngine, PermissionRule};
use talos_plugin::HookRegistry;

struct CountingTool {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl AgentTool for CountingTool {
    fn name(&self) -> &str {
        "counting"
    }

    fn description(&self) -> &str {
        "counts executions"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }

    async fn execute(&self, _input: serde_json::Value) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ToolResult::success("ok")
    }
}

#[tokio::test]
async fn server_permission_deny_returns_error_and_no_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(CountingTool {
        calls: calls.clone(),
    }));

    let mut engine = PermissionEngine::new();
    engine.add_rule(PermissionRule::new(
        "counting",
        None,
        talos_permission::PermissionDecision::Deny("deny all".to_string()),
    ));

    let gate = Arc::new(McpPermissionGate::new(
        Arc::new(engine),
        Arc::new(HookRegistry::new()),
    ));
    let handler = TalosMcpHandler::new(Arc::new(registry), gate);

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

    let _tools = client
        .peer()
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("list tools");

    let error = client
        .peer()
        .call_tool(CallToolRequestParams {
            meta: None,
            name: "counting".into(),
            arguments: Some(serde_json::Map::new()),
            task: None,
        })
        .await
        .expect_err("permission denied must return error");

    let msg = error.to_string();
    assert!(msg.contains("-326") || msg.contains("-320") || msg.contains("-32"), "{msg}");
    assert_eq!(calls.load(Ordering::SeqCst), 0, "tool should not execute");

    let _ = client.cancel().await;
    let _ = server_task.await;
}
