//! Real Agent/Session dispatch with an offline Network tool stand-in.
//! No WebSearchTool, HTTP/DNS, backend admission or transport-cancellation proof.
//! Characterizes current caller boundaries without changing production behavior.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};
use talos_agent::Agent;
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest,
};
use talos_agent::session::AppServerSession;
use talos_core::ApprovalChoice;
use talos_core::message::{AgentEvent, Message, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnCompletionStatus, TurnEventPayload,
};
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolRegistry, ToolResourceKind,
    ToolResult,
};
use talos_permission::PermissionEngine;
use tokio::sync::{Notify, mpsc};

const WAIT: Duration = Duration::from_secs(5);

#[derive(Default)]
struct Observations {
    starts: AtomicUsize,
    drops: AtomicUsize,
    completes: AtomicUsize,
    approvals: AtomicUsize,
    denied_results: AtomicUsize,
    denial: Mutex<Option<String>>,
    started: Notify,
    release: Notify,
}

struct OwnedWork(Arc<Observations>);
impl Drop for OwnedWork {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct NetworkStandIn(Arc<Observations>);
#[async_trait]
impl AgentTool for NetworkStandIn {
    fn name(&self) -> &str {
        "i293_network_stand_in"
    }
    fn description(&self) -> &str {
        "Offline owned-future fixture; performs no network I/O"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{},"additionalProperties":false})
    }
    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }
    fn family(&self) -> ToolFamily {
        ToolFamily::Network
    }
    fn permission_profile(&self, _: &Value) -> Vec<ToolPermissionFacet> {
        vec![ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "example.invalid",
            ToolResourceKind::Domain,
        )]
    }
    fn is_always_on(&self) -> bool {
        true
    }
    fn is_read_only(&self) -> bool {
        true
    }
    async fn execute(&self, _: Value) -> ToolResult {
        let _owned = OwnedWork(self.0.clone());
        self.0.starts.fetch_add(1, Ordering::SeqCst);
        self.0.started.notify_one();
        self.0.release.notified().await;
        self.0.completes.fetch_add(1, Ordering::SeqCst);
        ToolResult::success("offline fixture released")
    }
}

struct Model {
    calls: AtomicUsize,
    observed: Arc<Observations>,
}
#[async_trait]
impl LanguageModel for Model {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let first = self.calls.fetch_add(1, Ordering::SeqCst) == 0;
        if !first
            && let Some(error) = messages.iter().find_map(|m| match m {
                Message::Tool { result } if result.is_error => Some(result.content.clone()),
                _ => None,
            })
        {
            self.observed.denied_results.fetch_add(1, Ordering::SeqCst);
            *self.observed.denial.lock().expect("fixture observation") = Some(error);
        }
        let (tx, rx) = mpsc::channel(4);
        tx.try_send(AgentEvent::TurnStart)
            .expect("bounded fixture channel");
        if first {
            tx.try_send(AgentEvent::ToolCall {
                call: ToolCall {
                    id: "i293-call".into(),
                    name: "i293_network_stand_in".into(),
                    input: json!({}),
                },
                provenance: Default::default(),
                summary_fields: Vec::new(),
            })
            .expect("fixture tool call");
        } else {
            tx.try_send(AgentEvent::TextDelta {
                delta: "fixture finished".into(),
            })
            .expect("fixture text");
        }
        tx.try_send(AgentEvent::TurnEnd {
            stop_reason: if first {
                StopReason::ToolUse
            } else {
                StopReason::EndTurn
            },
            usage: Usage::default(),
        })
        .expect("fixture end");
        // No spawned provider producer: all events fit, sender drops here.
        Ok(rx)
    }
}

struct ApproveOnce(Arc<Observations>);
#[async_trait]
impl ApprovalResolver for ApproveOnce {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(request.tool_name, "i293_network_stand_in");
        self.0.approvals.fetch_add(1, Ordering::SeqCst);
        Ok(ApprovalChoice::ApproveOnce)
    }
}

fn agent(root: &std::path::Path, observed: Arc<Observations>, approve: bool) -> Agent {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkStandIn(observed.clone())));
    let agent = Agent::with_security(
        Arc::new(Model {
            calls: AtomicUsize::new(0),
            observed: observed.clone(),
        }),
        registry,
        Some(Arc::new(PermissionEngine::with_workspace_root(
            root.to_path_buf(),
        ))),
        None,
        root.to_path_buf(),
    )
    .with_permission_deadline(Duration::from_millis(10));
    if approve {
        agent.with_approval_resolver(Arc::new(ApproveOnce(observed)))
    } else {
        agent
    }
}

#[tokio::test(start_paused = true)]
async fn permission_deadline_and_detached_token_do_not_bound_active_tool_execution() {
    let root = tempfile::tempdir().expect("owned workspace");
    let observed = Arc::new(Observations::default());
    let agent = Arc::new(agent(root.path(), observed.clone(), true));
    let token = agent.cancellation_token();
    let task = tokio::spawn(async move { agent.run("fixture".into()).await });
    tokio::time::timeout(WAIT, observed.started.notified())
        .await
        .unwrap_or_else(|e| {
            panic!(
                "tool did not start: {e}; denial={:?}",
                observed.denial.lock().expect("fixture observation")
            )
        });
    assert_eq!(observed.approvals.load(Ordering::SeqCst), 1);
    token.cancel();
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert!(
        !task.is_finished(),
        "approval TTL and detached token are not execution controls"
    );
    assert_eq!(observed.drops.load(Ordering::SeqCst), 0);
    observed.release.notify_one();
    assert_eq!(
        tokio::time::timeout(WAIT, task)
            .await
            .expect("join deadline")
            .expect("agent task")
            .expect("agent result"),
        "fixture finished"
    );
    assert_eq!(observed.starts.load(Ordering::SeqCst), 1);
    assert_eq!(observed.completes.load(Ordering::SeqCst), 1);
    assert_eq!(observed.drops.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn network_ask_without_resolver_denies_before_starting_owned_work() {
    let root = tempfile::tempdir().expect("owned workspace");
    let observed = Arc::new(Observations::default());
    let agent = agent(root.path(), observed.clone(), false);
    tokio::time::timeout(WAIT, agent.run("fixture".into()))
        .await
        .expect("bounded turn")
        .expect("denial feedback turn");
    assert_eq!(observed.denied_results.load(Ordering::SeqCst), 1);
    let denial = observed.denial.lock().expect("fixture observation");
    assert!(
        denial
            .as_deref()
            .is_some_and(|s| s.contains("approval") && !s.contains("permission state error")),
        "actual permission rejection: {denial:?}"
    );
    assert_eq!(observed.approvals.load(Ordering::SeqCst), 0);
    assert_eq!(observed.starts.load(Ordering::SeqCst), 0);
    assert_eq!(observed.drops.load(Ordering::SeqCst), 0);
}

// This offline cancellation fixture must reach execution before it tests interruption.
// Keep its short approval budget in logical time so host scheduling cannot consume it.
#[tokio::test(start_paused = true)]
async fn session_interrupt_releases_owned_tool_before_cancelled_terminal_event() {
    let root = tempfile::tempdir().expect("owned workspace");
    let observed = Arc::new(Observations::default());
    let agent = agent(root.path(), observed.clone(), true);
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: root.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let mut events = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "fixture".into(),
        })
        .await
        .expect("submit");
    tokio::time::timeout(WAIT, observed.started.notified())
        .await
        .unwrap_or_else(|e| {
            panic!(
                "tool did not start: {e}; denial={:?}",
                observed.denial.lock().expect("fixture observation")
            )
        });
    handle
        .sq_tx
        .send(SessionOp::Interrupt)
        .await
        .expect("interrupt");
    tokio::time::timeout(WAIT, async {
        let mut started_turn = None;
        loop {
            let event = events.recv().await.expect("terminal channel");
            assert!(
                !matches!(
                    event,
                    SessionEvent::TurnEvent {
                        payload: TurnEventPayload::Progress {
                            event: AgentEvent::ToolResult { .. }
                        },
                        ..
                    }
                ),
                "dropped execution has no returned tool result"
            );
            if let SessionEvent::TurnEvent {
                turn_id,
                payload: TurnEventPayload::Started,
                ..
            } = &event
            {
                started_turn = Some(turn_id.clone());
            }
            if let SessionEvent::TurnEvent {
                turn_id,
                payload: TurnEventPayload::Completed { status },
                ..
            } = event
            {
                assert_eq!(
                    Some(turn_id),
                    started_turn,
                    "terminal belongs to the started turn"
                );
                assert!(matches!(status, TurnCompletionStatus::Cancelled));
                assert_eq!(
                    observed.drops.load(Ordering::SeqCst),
                    1,
                    "cleanup observed before terminal publication"
                );
                break;
            }
        }
    })
    .await
    .expect("cancelled terminal");
    assert_eq!(observed.approvals.load(Ordering::SeqCst), 1);
    assert_eq!(observed.starts.load(Ordering::SeqCst), 1);
    assert_eq!(observed.completes.load(Ordering::SeqCst), 0);
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    tokio::time::timeout(WAIT, actor_task)
        .await
        .expect("actor shutdown")
        .expect("actor join");
    while let Ok(event) = events.try_recv() {
        assert!(
            !matches!(
                event,
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Progress {
                        event: AgentEvent::ToolResult { .. }
                    },
                    ..
                }
            ),
            "no delayed result after actor shutdown and join"
        );
    }
    assert_eq!(observed.drops.load(Ordering::SeqCst), 1);
}
