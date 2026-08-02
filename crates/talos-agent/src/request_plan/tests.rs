use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_core::provider::{LanguageModel, ProviderResult, ToolDefinition};
use talos_core::session::{SubmissionItem, SubmissionKind};
use talos_core::tool::ToolRegistry;
use talos_plugin::{
    HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult,
};
use tokio::sync::mpsc;

use super::*;

#[derive(Clone)]
struct CapturedRequest {
    messages: Vec<Message>,
    tools: Vec<ToolDefinition>,
}

struct CapturingModel {
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
}

impl CapturingModel {
    fn new() -> (Self, Arc<Mutex<Vec<CapturedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                requests: requests.clone(),
            },
            requests,
        )
    }

    fn response() -> mpsc::Receiver<AgentEvent> {
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx
                .send(AgentEvent::TextDelta {
                    delta: "done".into(),
                })
                .await;
            let _ = tx
                .send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                })
                .await;
        });
        rx
    }
}

#[async_trait]
impl LanguageModel for CapturingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.requests
            .lock()
            .expect("captured request lock poisoned")
            .push(CapturedRequest {
                messages: messages.to_vec(),
                tools: Vec::new(),
            });
        Ok(Self::response())
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.requests
            .lock()
            .expect("captured request lock poisoned")
            .push(CapturedRequest {
                messages: messages.to_vec(),
                tools: tools.to_vec(),
            });
        Ok(Self::response())
    }
}

struct BeforeProviderCounter {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl HookHandler for BeforeProviderCounter {
    fn name(&self) -> &str {
        "i169-before-provider-counter"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::BeforeProviderCall]
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        HookResult::Continue
    }
}

#[tokio::test]
async fn initial_provider_dispatch_consumes_the_exact_sealed_plan_once() {
    let (model, captured_requests) = CapturingModel::new();
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(BeforeProviderCounter {
        calls: hook_calls.clone(),
    }));
    let agent = Agent::with_security_and_hooks(
        Arc::new(model),
        ToolRegistry::new(),
        None,
        None,
        PathBuf::from("/tmp"),
        Arc::new(hooks),
    );
    let items = vec![SubmissionItem {
        id: "sealed_item".into(),
        enqueue_sequence: 0,
        kind: SubmissionKind::UserTurn,
        text: "seal this request".into(),
        attachments: Vec::new(),
    }];

    let prepared = agent
        .prepare_session_turn(&items, Vec::new(), 128_000)
        .await
        .expect("prepare sealed request plan");
    let expected_messages = prepared.initial_plan.messages.clone();
    let expected_tools = prepared.initial_plan.tool_definitions.clone();
    let expected_estimate = prepared.initial_plan.estimated_tokens;
    assert_eq!(
        expected_estimate,
        Agent::estimate_provider_request_tokens(&expected_messages, &expected_tools)
    );
    assert_eq!(hook_calls.load(Ordering::SeqCst), 1);

    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let (result, _messages) = agent.run_prepared_session_turn(prepared, event_tx).await;
    assert_eq!(result.expect("run prepared request"), "done");
    assert_eq!(hook_calls.load(Ordering::SeqCst), 1);

    let captured = captured_requests
        .lock()
        .expect("captured request lock poisoned");
    assert_eq!(captured.len(), 1);
    assert_eq!(
        serde_json::to_value(&captured[0].messages).unwrap(),
        serde_json::to_value(&expected_messages).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&captured[0].tools).unwrap(),
        serde_json::to_value(&expected_tools).unwrap()
    );
}

#[tokio::test]
async fn over_budget_sealed_plan_never_reaches_the_provider() {
    let (model, captured_requests) = CapturingModel::new();
    let agent = Agent::with_security_and_hooks(
        Arc::new(model),
        ToolRegistry::new(),
        None,
        None,
        PathBuf::from("/tmp"),
        Arc::new(HookRegistry::new()),
    );
    let items = vec![SubmissionItem {
        id: "over_budget_item".into(),
        enqueue_sequence: 0,
        kind: SubmissionKind::UserTurn,
        text: "this cannot fit into a one-token request".into(),
        attachments: Vec::new(),
    }];

    let result = agent.prepare_session_turn(&items, Vec::new(), 1).await;
    assert!(matches!(
        result,
        Err(AgentError::ContextBudgetExceeded { limit: 1, .. })
    ));
    assert!(
        captured_requests
            .lock()
            .expect("captured request lock poisoned")
            .is_empty(),
        "a rejected sealed plan must never dispatch"
    );
}
