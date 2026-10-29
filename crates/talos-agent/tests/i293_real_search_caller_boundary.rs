//! Real I293 prepared SearchRouter through Agent permission dispatch and Session interruption.
//!
//! The talos-tools integration feature holds the actual prepared running composition before any
//! candidate network future is polled, so these tests exercise the real caller/permission/drop path
//! with zero DNS/HTTP and without production WebSearch registration.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
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
use talos_core::tool::ToolRegistry;
use talos_permission::PermissionEngine;
use talos_tools::{SearchRouterIntegrationProbe, i293_guarded_search_router_tool_fixture};
use tokio::sync::mpsc;

const WAIT: Duration = Duration::from_secs(5);

#[derive(Default)]
struct Observations {
    approvals: AtomicUsize,
    denied_results: AtomicUsize,
    denial: Mutex<Option<String>>,
}

struct SearchModel {
    calls: AtomicUsize,
    observed: Arc<Observations>,
}

#[async_trait]
impl LanguageModel for SearchModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let first = self.calls.fetch_add(1, Ordering::SeqCst) == 0;
        if !first
            && let Some(error) = messages.iter().find_map(|message| match message {
                Message::Tool { result } if result.is_error => Some(result.content.clone()),
                _ => None,
            })
        {
            self.observed.denied_results.fetch_add(1, Ordering::SeqCst);
            *self.observed.denial.lock().expect("search observation") = Some(error);
        }

        let (tx, rx) = mpsc::channel(4);
        tx.try_send(AgentEvent::TurnStart)
            .expect("bounded search fixture channel");
        if first {
            tx.try_send(AgentEvent::ToolCall {
                call: ToolCall {
                    id: "i293-real-search".into(),
                    name: "web_search".into(),
                    input: json!({
                        "query": "offline caller boundary",
                        "max_results": 1,
                        "include_snippets": true
                    }),
                },
                provenance: Default::default(),
                summary_fields: vec!["query".into()],
            })
            .expect("search tool call");
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
        Ok(rx)
    }
}

struct ApprovePreparedSearch(Arc<Observations>);

#[async_trait]
impl ApprovalResolver for ApprovePreparedSearch {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        assert_eq!(request.tool_name, "web_search");
        let scope = request
            .preview
            .network_scope()
            .expect("prepared search network scope");
        assert_eq!(scope.endpoints().len(), 4);
        assert_eq!(
            scope
                .endpoints()
                .iter()
                .filter(|endpoint| endpoint.method() == "GET")
                .count(),
            3
        );
        assert_eq!(
            scope
                .endpoints()
                .iter()
                .filter(|endpoint| endpoint.method() == "POST")
                .count(),
            1
        );
        self.0.approvals.fetch_add(1, Ordering::SeqCst);
        Ok(ApprovalChoice::ApproveOnce)
    }
}

fn agent(
    root: &std::path::Path,
    probe: Arc<SearchRouterIntegrationProbe>,
    observed: Arc<Observations>,
    approve: bool,
) -> Agent {
    let mut registry = ToolRegistry::new();
    registry.register(
        i293_guarded_search_router_tool_fixture(probe).expect("real guarded search fixture"),
    );
    let agent = Agent::with_security(
        Arc::new(SearchModel {
            calls: AtomicUsize::new(0),
            observed: Arc::clone(&observed),
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
        agent.with_approval_resolver(Arc::new(ApprovePreparedSearch(observed)))
    } else {
        agent
    }
}

#[tokio::test]
async fn real_prepared_search_without_resolver_is_denied_before_execution() {
    let root = tempfile::tempdir().expect("owned workspace");
    let probe = Arc::new(SearchRouterIntegrationProbe::default());
    let observed = Arc::new(Observations::default());
    let agent = agent(
        root.path(),
        Arc::clone(&probe),
        Arc::clone(&observed),
        false,
    );

    tokio::time::timeout(WAIT, agent.run("search".into()))
        .await
        .expect("bounded turn")
        .expect("permission denial returned to model");

    assert_eq!(observed.approvals.load(Ordering::SeqCst), 0);
    assert_eq!(observed.denied_results.load(Ordering::SeqCst), 1);
    let denial = observed.denial.lock().expect("search observation");
    assert!(
        denial.as_deref().is_some_and(
            |value| value.contains("approval") && !value.contains("permission state error")
        ),
        "actual permission rejection: {denial:?}"
    );
    assert_eq!(
        probe.starts(),
        0,
        "permission denial must precede execution"
    );
    assert_eq!(probe.drops(), 0, "no prepared execution was owned");
}

#[tokio::test(start_paused = true)]
async fn real_prepared_search_session_interrupt_drops_before_cancelled_terminal() {
    let root = tempfile::tempdir().expect("owned workspace");
    let probe = Arc::new(SearchRouterIntegrationProbe::default());
    let observed = Arc::new(Observations::default());
    let agent = agent(root.path(), Arc::clone(&probe), Arc::clone(&observed), true);
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
            message: "search".into(),
        })
        .await
        .expect("submit");
    tokio::time::timeout(WAIT, probe.wait_started())
        .await
        .expect("real prepared search starts");
    assert_eq!(observed.approvals.load(Ordering::SeqCst), 1);
    assert_eq!(probe.starts(), 1);
    assert_eq!(probe.drops(), 0);

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
                "dropped real search execution has no returned tool result"
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
                    "cancelled terminal belongs to the started turn"
                );
                assert!(matches!(status, TurnCompletionStatus::Cancelled));
                assert_eq!(
                    probe.drops(),
                    1,
                    "real running search ownership is dropped before terminal publication"
                );
                break;
            }
        }
    })
    .await
    .expect("cancelled terminal");

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
            "no delayed tool result after actor shutdown and join"
        );
    }
    assert_eq!(probe.starts(), 1);
    assert_eq!(probe.drops(), 1);
}
