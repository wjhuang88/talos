use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionEvent, SessionOp,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionReceipt,
    SubmissionReceiptDisposition, SubmissionSource, TurnEventPayload,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::mpsc;

struct CountingModel {
    calls: Arc<AtomicUsize>,
    delay: Duration,
}

#[async_trait]
impl LanguageModel for CountingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(8);
        let delay = self.delay;
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx
                .send(AgentEvent::TextDelta {
                    delta: "done".into(),
                })
                .await;
            let _ = tx
                .send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                })
                .await;
        });
        Ok(rx)
    }
}

fn make_agent(model: impl LanguageModel + 'static) -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(model), ToolRegistry::new())
}

fn session_config(workspace_root: &std::path::Path) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    }
}

fn submission(id: &str, item_id: &str, generation: u64, text: &str) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source: SubmissionSource::User,
        sender_generation: generation,
        items: vec![SubmissionItem {
            id: item_id.into(),
            enqueue_sequence: generation,
            kind: SubmissionKind::UserTurn,
            text: text.into(),
            attachments: Vec::new(),
        }],
    }
}

fn advance_runtime_generation(store: &PendingSubmissionStore, target: u64) {
    let mut current = store.runtime_generation().unwrap();
    while current < target {
        current = store.advance_runtime_generation(current).unwrap();
    }
    assert_eq!(current, target);
}

async fn wait_for_receipt(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    wanted_submission_id: &str,
) {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(3), eq_rx.recv())
            .await
            .expect("timed out waiting for durable receipt")
            .expect("session event channel closed before durable receipt");
        if matches!(
            event,
            SessionEvent::SubmissionReceipt {
                submission_id,
                disposition: SubmissionReceiptDisposition::AcceptedPending,
                ..
            } if submission_id == wanted_submission_id
        ) {
            return;
        }
    }
}

async fn wait_for_structured_start(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    wanted_submission_id: &str,
) {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(3), eq_rx.recv())
            .await
            .expect("timed out waiting for structured turn start")
            .expect("session event channel closed before structured turn start");
        if matches!(
            event,
            SessionEvent::StructuredTurnEvent {
                submission_id,
                sequence: 0,
                payload: TurnEventPayload::Started,
                ..
            } if submission_id == wanted_submission_id
        ) {
            return;
        }
    }
}

async fn wait_for_tracked_receipt(
    receipt_rx: &mut mpsc::UnboundedReceiver<SubmissionReceipt>,
) -> SubmissionReceipt {
    tokio::time::timeout(Duration::from_secs(3), receipt_rx.recv())
        .await
        .expect("timed out waiting for tracked durable receipt")
        .expect("tracked durable receipt channel closed")
}

#[tokio::test]
async fn lost_ack_reconciles_committed_custody_without_duplicate_execution() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager.create_or_open_session("i169-lost-ack").unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    advance_runtime_generation(&store, 7);
    let calls = Arc::new(AtomicUsize::new(0));
    let agent = make_agent(CountingModel {
        calls: calls.clone(),
        delay: Duration::from_millis(10),
    });
    let (handle, mut actor) = AppServerSession::new(agent, session_config(temp.path()));
    actor.set_generation(7);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let actor_task = tokio::spawn(async move { actor.run().await });
    let immutable = submission("lost_ack_batch", "lost_ack_item", 7, "retain me");

    // The EQ projection is intentionally absent. Durable Actor custody must
    // still execute the accepted submission and commit it exactly once.
    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: immutable.clone(),
        })
        .await
        .unwrap();

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let committed = store
                .get("lost_ack_batch")
                .unwrap()
                .is_some_and(|record| record.state == PendingSubmissionState::Committed);
            if committed && calls.load(Ordering::SeqCst) == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("accepted lost-Ack submission should commit once");

    // Reconciliation asks the same authority about the exact immutable
    // identity. It must not enqueue or execute a second Turn.
    let (reconcile_tx, mut reconcile_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::ReconcileStructuredTracked {
            submission: immutable.clone(),
            receipt_tx: Some(reconcile_tx),
        })
        .await
        .unwrap();
    let reconciled = wait_for_tracked_receipt(&mut reconcile_rx).await;
    assert_eq!(reconciled.submission_id, "lost_ack_batch");
    assert!(matches!(
        reconciled.disposition,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Committed,
            turn_id: Some(_),
        }
    ));

    // Even an accidental resend of the exact payload resolves to the existing
    // durable identity rather than creating another execution authority.
    let (resend_tx, mut resend_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: immutable,
            receipt_tx: Some(resend_tx),
        })
        .await
        .unwrap();
    let resent = wait_for_tracked_receipt(&mut resend_rx).await;
    assert!(matches!(
        resent.disposition,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Committed,
            turn_id: Some(_),
        }
    ));

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        store.get("lost_ack_batch").unwrap().unwrap().state,
        PendingSubmissionState::Committed
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn shutdown_pauses_unstarted_durable_submissions() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-shutdown-pause")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    advance_runtime_generation(&store, 1);
    let calls = Arc::new(AtomicUsize::new(0));
    let agent = make_agent(CountingModel {
        calls: calls.clone(),
        delay: Duration::from_secs(10),
    });
    let (handle, mut actor) = AppServerSession::new(agent, session_config(temp.path()));
    actor.set_generation(1);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("running_batch", "running_item", 1, "run"),
        })
        .await
        .unwrap();
    wait_for_structured_start(&mut eq_rx, "running_batch").await;

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("pending_batch_1", "pending_item_1", 1, "later one"),
        })
        .await
        .unwrap();
    wait_for_receipt(&mut eq_rx, "pending_batch_1").await;
    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission("pending_batch_2", "pending_item_2", 1, "later two"),
        })
        .await
        .unwrap();
    wait_for_receipt(&mut eq_rx, "pending_batch_2").await;

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        store.get("running_batch").unwrap().unwrap().state,
        PendingSubmissionState::TerminalCancelled
    );
    assert_eq!(
        store.get("pending_batch_1").unwrap().unwrap().state,
        PendingSubmissionState::PausedPending
    );
    assert_eq!(
        store.get("pending_batch_2").unwrap().unwrap().state,
        PendingSubmissionState::PausedPending
    );
}
