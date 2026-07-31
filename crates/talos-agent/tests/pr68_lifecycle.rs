use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::session::AppServerSession;
use talos_agent::{Agent, create_scheduler_tools};
use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, StructuredSubmission, SubmissionItem,
    SubmissionKind, SubmissionRejectionReason, SubmissionSource,
};
use talos_core::tool::ToolRegistry;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

struct MockModel {
    responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
}

impl MockModel {
    fn successful() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from([vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ]]))),
        }
    }
}

#[async_trait]
impl LanguageModel for MockModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let events = self
            .responses
            .lock()
            .expect("mock model lock")
            .pop_front()
            .unwrap_or_default();
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            for event in events {
                let _ = tx.send(event).await;
            }
        });
        Ok(rx)
    }
}

fn make_agent() -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(MockModel::successful()), ToolRegistry::new())
}

fn config(context_limit: u32) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: Vec::new(),
        model_context_limit: context_limit,
    }
}

fn submission(
    batch_id: &str,
    item_id: &str,
    source: SubmissionSource,
) -> StructuredSubmission {
    StructuredSubmission {
        id: batch_id.into(),
        source,
        sender_generation: 0,
        items: vec![SubmissionItem {
            id: item_id.into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "request".into(),
            attachments: Vec::new(),
        }],
    }
}

async fn next_rejection(
    rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
) -> SubmissionRejectionReason {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("session event timeout")
            .expect("session event channel");
        if let SessionEvent::SubmissionRejected { reason, .. } = event {
            return reason;
        }
    }
}

#[tokio::test]
async fn context_rejection_releases_item_identity_for_retry() {
    let (handle, mut actor) = AppServerSession::new(make_agent(), config(64));
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("first_batch", "retryable_item", SubmissionSource::User),
        })
        .await
        .unwrap();
    assert_eq!(
        next_rejection(&mut eq_rx).await,
        SubmissionRejectionReason::ContextBudgetExceeded
    );

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("retry_batch", "retryable_item", SubmissionSource::User),
        })
        .await
        .unwrap();
    assert_eq!(
        next_rejection(&mut eq_rx).await,
        SubmissionRejectionReason::ContextBudgetExceeded,
        "a retry with the retained item identity must reach preflight instead of Duplicate"
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}

#[tokio::test]
async fn scheduler_submission_exposes_bounded_external_projection() {
    let (handle, mut actor) = AppServerSession::new(make_agent(), config(128_000));
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: StructuredSubmission {
                id: "scheduler_batch".into(),
                source: SubmissionSource::Scheduler,
                sender_generation: 0,
                items: vec![SubmissionItem {
                    id: "scheduler_item".into(),
                    enqueue_sequence: 1,
                    kind: SubmissionKind::UserTurn,
                    text: "[scheduled-followup] inspect the build".into(),
                    attachments: Vec::new(),
                }],
            },
        })
        .await
        .unwrap();

    let projection = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(SessionEvent::ExternalSubmissionQueued {
                submission_id,
                source,
                item_texts,
                ..
            }) = eq_rx.recv().await
            {
                break (submission_id, source, item_texts);
            }
        }
    })
    .await
    .expect("external projection event");

    assert_eq!(projection.0, "scheduler_batch");
    assert_eq!(projection.1, SubmissionSource::Scheduler);
    assert_eq!(
        projection.2,
        vec!["[scheduled-followup] inspect the build"]
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn one_shot_waits_for_session_queue_capacity() {
    let (sq_tx, mut sq_rx) = mpsc::channel(1);
    sq_tx.send(SessionOp::Interrupt).await.unwrap();

    let (tools, pending) = create_scheduler_tools();
    let delay_tool = tools[0].clone();
    let list_tool = tools[2].clone();
    let scheduler_task = pending.spawn(sq_tx, CancellationToken::new());

    let registration = delay_tool
        .execute(serde_json::json!({
            "message": "deliver after capacity recovers",
            "delay_secs": 1
        }))
        .await;
    assert!(!registration.is_error, "registration must succeed");

    tokio::time::advance(Duration::from_secs(2)).await;
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    let before = list_tool.execute(serde_json::json!({})).await;
    assert!(
        before.content.contains("1 active task"),
        "the one-shot must remain active while SQ has no capacity: {}",
        before.content
    );

    assert!(matches!(sq_rx.recv().await, Some(SessionOp::Interrupt)));
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }

    let delivered = sq_rx
        .try_recv()
        .expect("one-shot must deliver after capacity recovers");
    assert!(matches!(
        delivered,
        SessionOp::SubmitStructured { submission }
            if submission.source == SubmissionSource::Scheduler
                && submission.items[0].text.contains("capacity recovers")
    ));

    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    let after = list_tool.execute(serde_json::json!({})).await;
    assert!(after.content.contains("No active"));

    scheduler_task.abort();
}
