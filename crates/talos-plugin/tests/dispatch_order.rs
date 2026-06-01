use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talos_plugin::{HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult, TurnId};

struct RecordingHandler {
    name: &'static str,
    log: Arc<Mutex<Vec<String>>>,
    result: HookResult,
}

#[async_trait]
impl HookHandler for RecordingHandler {
    fn name(&self) -> &str {
        self.name
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::OnSystemPromptBuilt]
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        self.log.lock().expect("log lock").push(self.name.to_string());
        match &self.result {
            HookResult::Continue => HookResult::Continue,
            HookResult::Skip => HookResult::Skip,
            HookResult::Deny { reason } => HookResult::Deny {
                reason: reason.clone(),
            },
            HookResult::Modify(_) => HookResult::Modify(HookEvent::OnSystemPromptBuilt {
                prompt: Box::leak(String::from("modified prompt").into_boxed_str()),
            }),
        }
    }
}

fn hook_context() -> HookContext {
    HookContext::new(TurnId::new(), PathBuf::from("."))
}

#[tokio::test]
async fn dispatch_is_sequential() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(RecordingHandler {
        name: "first",
        log: log.clone(),
        result: HookResult::Continue,
    }));
    registry.register(Arc::new(RecordingHandler {
        name: "second",
        log: log.clone(),
        result: HookResult::Continue,
    }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::OnSystemPromptBuilt {
                prompt: "original prompt",
            },
        )
        .await;

    assert!(matches!(outcome, talos_plugin::HookOutcome::Continue(_)));
    assert_eq!(
        log.lock().expect("log lock").as_slice(),
        ["first".to_string(), "second".to_string()]
    );
}

#[tokio::test]
async fn skip_short_circuits() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(RecordingHandler {
        name: "first",
        log: log.clone(),
        result: HookResult::Skip,
    }));
    registry.register(Arc::new(RecordingHandler {
        name: "second",
        log: log.clone(),
        result: HookResult::Continue,
    }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::OnSystemPromptBuilt {
                prompt: "original prompt",
            },
        )
        .await;

    assert!(matches!(outcome, talos_plugin::HookOutcome::Skip(_)));
    assert_eq!(log.lock().expect("log lock").as_slice(), ["first".to_string()]);
}

#[tokio::test]
async fn deny_propagates() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(RecordingHandler {
        name: "deny",
        log,
        result: HookResult::Deny {
            reason: "blocked".to_string(),
        },
    }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::OnSystemPromptBuilt {
                prompt: "original prompt",
            },
        )
        .await;

    match outcome {
        talos_plugin::HookOutcome::Deny { reason, .. } => assert_eq!(reason, "blocked"),
        other => panic!("expected deny outcome, got {other:?}"),
    }
}

#[tokio::test]
async fn modify_replaces_event() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(RecordingHandler {
        name: "modify",
        log,
        result: HookResult::Modify(HookEvent::OnSystemPromptBuilt {
            prompt: Box::leak(String::from("modified prompt").into_boxed_str()),
        }),
    }));

    let outcome = registry
        .dispatch(
            &hook_context(),
            HookEvent::OnSystemPromptBuilt {
                prompt: "original prompt",
            },
        )
        .await;

    match outcome.into_event() {
        HookEvent::OnSystemPromptBuilt { prompt } => assert_eq!(prompt, "modified prompt"),
        other => panic!("unexpected event: {other:?}"),
    }
}
