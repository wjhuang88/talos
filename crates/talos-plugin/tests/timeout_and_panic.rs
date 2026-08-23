use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use talos_plugin::{
    HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult, TurnId,
};

struct SlowHandler {
    log: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl HookHandler for SlowHandler {
    fn name(&self) -> &str {
        "slow"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::TurnStart]
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(10)
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.log
            .lock()
            .expect("log lock")
            .push("slow-finished".to_string());
        HookResult::Continue
    }
}

struct NextHandler {
    log: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl HookHandler for NextHandler {
    fn name(&self) -> &str {
        "next"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::TurnStart]
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        self.log.lock().expect("log lock").push("next".to_string());
        HookResult::Continue
    }
}

struct PanicHandler;

#[async_trait]
impl HookHandler for PanicHandler {
    fn name(&self) -> &str {
        "panic"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::TurnStart]
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        panic!("boom");
    }
}

struct PermissionHandler {
    result: PermissionHandlerResult,
}

enum PermissionHandlerResult {
    Skip,
    Panic,
    Timeout,
}

#[async_trait]
impl HookHandler for PermissionHandler {
    fn name(&self) -> &str {
        "permission-failure"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::AfterPermissionCheck]
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(5)
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        match self.result {
            PermissionHandlerResult::Skip => HookResult::Skip,
            PermissionHandlerResult::Panic => panic!("permission hook panic"),
            PermissionHandlerResult::Timeout => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                HookResult::Continue
            }
        }
    }
}

fn hook_context() -> HookContext {
    HookContext::new(TurnId::new(), PathBuf::from("."))
}

#[tokio::test]
async fn timeout_fires_per_handler() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(SlowHandler { log: log.clone() }));
    registry.register(Arc::new(NextHandler { log: log.clone() }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::TurnStart {
                turn_id: TurnId::new(),
            },
        )
        .await;

    assert!(matches!(outcome, talos_plugin::HookOutcome::Continue(_)));
    assert_eq!(
        log.lock().expect("log lock").as_slice(),
        ["next".to_string()]
    );
}

#[tokio::test]
async fn panic_aborts_chain_fail_safe() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(PanicHandler));
    registry.register(Arc::new(NextHandler { log: log.clone() }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::TurnStart {
                turn_id: TurnId::new(),
            },
        )
        .await;

    assert!(matches!(outcome, talos_plugin::HookOutcome::Continue(_)));
    assert!(log.lock().expect("log lock").is_empty());
}

#[tokio::test]
async fn final_permission_gate_fails_closed_for_skip_panic_and_timeout() {
    for result in [
        PermissionHandlerResult::Skip,
        PermissionHandlerResult::Panic,
        PermissionHandlerResult::Timeout,
    ] {
        let mut registry = HookRegistry::new();
        registry.register(Arc::new(PermissionHandler { result }));
        let call = talos_core::message::ToolCall {
            id: "call".to_owned(),
            name: "write".to_owned(),
            input: serde_json::json!({"path": "<redacted>"}),
        };
        let outcome = registry
            .dispatch_permission_gate(
                &hook_context(),
                HookEvent::AfterPermissionCheck {
                    call: &call,
                    decision: talos_permission::PermissionDecision::Allow,
                },
            )
            .await;
        assert!(matches!(outcome, talos_plugin::HookOutcome::Deny { .. }));
    }
}
