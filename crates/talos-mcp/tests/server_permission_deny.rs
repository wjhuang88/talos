use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, PaginatedRequestParams};
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{PermissionEngine, PermissionRule};
use talos_plugin::{HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult};

struct CountingTool {
    calls: Arc<AtomicUsize>,
}

struct SkipFinalPermissionHook;

struct SkipProposalPermissionHook;

struct PanicProposalPermissionHook;

struct TimeoutProposalPermissionHook;

#[async_trait]
impl HookHandler for SkipProposalPermissionHook {
    fn name(&self) -> &str {
        "skip-proposal-permission"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::OnToolCallProposed]
    }

    async fn on_event(&self, _context: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        HookResult::Skip
    }
}

#[async_trait]
impl HookHandler for PanicProposalPermissionHook {
    fn name(&self) -> &str {
        "panic-proposal-permission"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::OnToolCallProposed]
    }

    async fn on_event(&self, _context: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        panic!("proposal hook panic")
    }
}

#[async_trait]
impl HookHandler for TimeoutProposalPermissionHook {
    fn name(&self) -> &str {
        "timeout-proposal-permission"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::OnToolCallProposed]
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(5)
    }

    async fn on_event(&self, _context: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        tokio::time::sleep(Duration::from_secs(1)).await;
        HookResult::Continue
    }
}

#[async_trait]
impl HookHandler for SkipFinalPermissionHook {
    fn name(&self) -> &str {
        "skip-final-permission"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::AfterPermissionCheck]
    }

    async fn on_event(&self, _context: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        HookResult::Skip
    }
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

    fn is_read_only(&self) -> bool {
        true
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
        let running = handler
            .serve((server_read, server_write))
            .await
            .expect("server starts");
        let _ = running.waiting().await;
    });

    let client = ().serve((client_read, client_write)).await.expect("client starts");

    let _tools = client
        .peer()
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("list tools");

    let error = client
        .peer()
        .call_tool(CallToolRequestParams::new("counting").with_arguments(serde_json::Map::new()))
        .await
        .expect_err("permission denied must return error");

    let msg = error.to_string();
    assert!(
        msg.contains("-326") || msg.contains("-320") || msg.contains("-32"),
        "{msg}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0, "tool should not execute");

    let _ = client.cancel().await;
    let _ = server_task.await;
}

#[tokio::test]
async fn server_final_permission_hook_skip_returns_error_and_no_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(CountingTool {
        calls: calls.clone(),
    }));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(SkipFinalPermissionHook));
    let gate = Arc::new(McpPermissionGate::new(
        Arc::new(PermissionEngine::new()),
        Arc::new(hooks),
    ));
    let handler = TalosMcpHandler::new(Arc::new(registry), gate);
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

    client
        .peer()
        .call_tool(CallToolRequestParams::new("counting").with_arguments(serde_json::Map::new()))
        .await
        .expect_err("final permission hook must fail closed");
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let _ = client.cancel().await;
    let _ = server_task.await;
}

#[tokio::test]
async fn server_proposal_permission_hook_skip_returns_error_and_no_execution() {
    assert_proposal_hook_fails_closed(Arc::new(SkipProposalPermissionHook)).await;
}

#[tokio::test]
async fn server_proposal_permission_hook_panic_returns_error_and_no_execution() {
    assert_proposal_hook_fails_closed(Arc::new(PanicProposalPermissionHook)).await;
}

#[tokio::test]
async fn server_proposal_permission_hook_timeout_returns_error_and_no_execution() {
    assert_proposal_hook_fails_closed(Arc::new(TimeoutProposalPermissionHook)).await;
}

async fn assert_proposal_hook_fails_closed(hook: Arc<dyn HookHandler>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(CountingTool {
        calls: calls.clone(),
    }));
    let mut hooks = HookRegistry::new();
    hooks.register(hook);
    let gate = Arc::new(McpPermissionGate::new(
        Arc::new(PermissionEngine::new()),
        Arc::new(hooks),
    ));
    let handler = TalosMcpHandler::new(Arc::new(registry), gate);
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

    client
        .peer()
        .call_tool(CallToolRequestParams::new("counting").with_arguments(serde_json::Map::new()))
        .await
        .expect_err("proposal permission hook must fail closed");
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let _ = client.cancel().await;
    let _ = server_task.await;
}
