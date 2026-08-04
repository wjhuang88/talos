use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionOp, StructuredSubmission,
    SubmissionItem, SubmissionKind, SubmissionReceipt, SubmissionReceiptDisposition,
    SubmissionRejectionReason, SubmissionSource,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::mpsc;

struct CountingModel {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LanguageModel for CountingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
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
                    usage: talos_core::message::Usage::default(),
                })
                .await;
        });
        Ok(rx)
    }
}

fn make_agent(calls: Arc<AtomicUsize>) -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(CountingModel { calls }), ToolRegistry::new())
}

fn session_config(workspace_root: &std::path::Path) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    }
}

fn submission(
    id: &str,
    item_id: &str,
    generation: u64,
    source: SubmissionSource,
) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source,
        sender_generation: generation,
        items: vec![SubmissionItem {
            id: item_id.into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: id.into(),
            attachments: Vec::new(),
        }],
    }
}

fn advance_store_to(store: &PendingSubmissionStore, generation: u64) {
    for expected in 0..generation {
        assert_eq!(store.advance_runtime_generation(expected).unwrap(), expected + 1);
    }
}

async fn receipt(receiver: &mut mpsc::UnboundedReceiver<SubmissionReceipt>) -> SubmissionReceipt {
    tokio::time::timeout(Duration::from_secs(3), receiver.recv())
        .await
        .expect("tracked receipt timeout")
        .expect("tracked receipt channel closed")
}

async fn wait_for_committed(
    store: &PendingSubmissionStore,
    submission_id: &str,
    calls: &AtomicUsize,
    expected_calls: usize,
) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let committed = store
                .get(submission_id)
                .unwrap()
                .is_some_and(|record| record.state == PendingSubmissionState::Committed);
            if committed && calls.load(Ordering::SeqCst) == expected_calls {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("submission should commit under the authoritative generation");
}

#[tokio::test]
async fn stale_generation_is_rejected_before_durable_or_provider_custody() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-generation-reject")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    advance_store_to(&store, 5);
    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) =
        AppServerSession::new(make_agent(calls.clone()), session_config(temp.path()));
    actor.set_generation(5);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    assert_eq!(actor.generation(), 5);

    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let actor_task = tokio::spawn(async move { actor.run().await });
    let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();

    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: submission(
                "stale_generation_batch",
                "stale_generation_item",
                4,
                SubmissionSource::User,
            ),
            receipt_tx: Some(receipt_tx),
        })
        .await
        .unwrap();

    let rejected = receipt(&mut receipt_rx).await;
    assert_eq!(
        rejected.session_generation, 4,
        "a rejection receipt must echo the addressed stale generation"
    );
    assert!(matches!(
        rejected.disposition,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::WrongGeneration,
        }
    ));
    assert!(store.get("stale_generation_batch").unwrap().is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn user_and_scheduler_require_the_exact_authoritative_generation() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-generation-accept")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    advance_store_to(&store, 9);
    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) =
        AppServerSession::new(make_agent(calls.clone()), session_config(temp.path()));
    actor.set_generation(9);
    actor.set_durable_persistence(durable, PersistencePolicy::default());

    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let actor_task = tokio::spawn(async move { actor.run().await });

    let (user_tx, mut user_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: submission(
                "current_generation_batch",
                "current_generation_item",
                9,
                SubmissionSource::User,
            ),
            receipt_tx: Some(user_tx),
        })
        .await
        .unwrap();
    let user_receipt = receipt(&mut user_rx).await;
    assert_eq!(user_receipt.session_generation, 9);
    assert!(user_receipt.disposition.has_durable_custody());
    wait_for_committed(&store, "current_generation_batch", &calls, 1).await;

    for (id, generation) in [("scheduler_zero", 0), ("scheduler_old", 8)] {
        let (stale_tx, mut stale_rx) = mpsc::unbounded_channel();
        sq_tx
            .send(SessionOp::SubmitStructuredTracked {
                submission: submission(
                    id,
                    &format!("{id}_item"),
                    generation,
                    SubmissionSource::Scheduler,
                ),
                receipt_tx: Some(stale_tx),
            })
            .await
            .unwrap();
        let stale = receipt(&mut stale_rx).await;
        assert_eq!(
            stale.session_generation, generation,
            "a rejection receipt must remain correlated to its addressed generation"
        );
        assert!(matches!(
            stale.disposition,
            SubmissionReceiptDisposition::Rejected {
                reason: SubmissionRejectionReason::WrongGeneration,
            }
        ));
        assert!(store.get(id).unwrap().is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    let (scheduler_tx, mut scheduler_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: submission(
                "scheduler_generation_batch",
                "scheduler_generation_item",
                9,
                SubmissionSource::Scheduler,
            ),
            receipt_tx: Some(scheduler_tx),
        })
        .await
        .unwrap();
    let scheduler_receipt = receipt(&mut scheduler_rx).await;
    assert_eq!(scheduler_receipt.session_generation, 9);
    assert!(scheduler_receipt.disposition.has_durable_custody());
    wait_for_committed(&store, "scheduler_generation_batch", &calls, 2).await;

    let scheduler_record = store
        .get("scheduler_generation_batch")
        .unwrap()
        .expect("scheduler submission journal record");
    assert_eq!(scheduler_record.submission.sender_generation, 9);

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
