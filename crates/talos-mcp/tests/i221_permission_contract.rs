use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use talos_core::tool::{ToolNature, ToolPermissionFacet};
use talos_mcp::McpCallRequest;
use talos_mcp::server::McpPermissionGate;
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
use talos_plugin::{
    HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult, TurnId,
};

struct CapturePermissionHooks {
    payloads: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl HookHandler for CapturePermissionHooks {
    fn name(&self) -> &str {
        "capture-permission-hooks"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[
            HookEventKind::OnToolCallProposed,
            HookEventKind::BeforePermissionCheck,
            HookEventKind::AfterPermissionCheck,
        ]
    }

    async fn on_event(&self, _context: &HookContext, event: &mut HookEvent<'_>) -> HookResult {
        let call = match event {
            HookEvent::OnToolCallProposed { call }
            | HookEvent::BeforePermissionCheck { call }
            | HookEvent::AfterPermissionCheck { call, .. } => call,
            _ => return HookResult::Continue,
        };
        self.payloads
            .lock()
            .expect("capture lock")
            .push(call.input.to_string());
        HookResult::Continue
    }
}

struct SlowPermissionHook;

#[async_trait]
impl HookHandler for SlowPermissionHook {
    fn name(&self) -> &str {
        "slow-permission-hook"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::BeforePermissionCheck]
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(1)
    }

    async fn on_event(&self, _context: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        tokio::time::sleep(Duration::from_secs(1)).await;
        HookResult::Continue
    }
}

fn allow_engine() -> Arc<PermissionEngine> {
    let mut engine = PermissionEngine::with_workspace_root(PathBuf::from("/tmp"));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Internal,
        None,
        Some(talos_permission::ResourceKind::Path),
        PermissionDecision::Allow,
    ));
    Arc::new(engine)
}

fn request() -> McpCallRequest {
    McpCallRequest {
        name: "write".to_owned(),
        arguments: Some(serde_json::Map::from_iter([
            (
                "path".to_owned(),
                serde_json::Value::String("/tmp/private/sentinel.txt".to_owned()),
            ),
            (
                "secret".to_owned(),
                serde_json::Value::String("sentinel-secret".to_owned()),
            ),
        ])),
    }
}

fn profile() -> Vec<ToolPermissionFacet> {
    vec![ToolPermissionFacet::new(ToolNature::Internal)]
}

#[tokio::test]
async fn legacy_evaluate_call_remains_source_compatible_and_hooks_are_redacted() {
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(CapturePermissionHooks {
        payloads: payloads.clone(),
    }));
    let gate = McpPermissionGate::new(allow_engine(), Arc::new(hooks));
    let context = HookContext::new(TurnId::new(), PathBuf::from("/tmp"));

    let result: Result<(), rmcp::ErrorData> =
        gate.evaluate_call(&context, &request(), &profile()).await;
    result.expect("legacy API authorizes");

    let payloads = payloads.lock().expect("capture lock");
    assert_eq!(payloads.len(), 3);
    for payload in payloads.iter() {
        assert!(payload.contains("<redacted>"));
        assert!(!payload.contains("sentinel-secret"));
        assert!(!payload.contains("/tmp/private"));
    }
}

#[tokio::test]
async fn one_total_mcp_deadline_fails_closed() {
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(SlowPermissionHook));
    let gate = McpPermissionGate::new(allow_engine(), Arc::new(hooks))
        .with_deadline(Duration::from_millis(5));
    let context = HookContext::new(TurnId::new(), PathBuf::from("/tmp"));

    gate.authorize_call(&context, &request(), &profile())
        .await
        .expect_err("deadline must deny authorization");
}
