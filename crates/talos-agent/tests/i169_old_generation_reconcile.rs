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
    SubmissionSource,
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
                    delta: "generation-two".into(),
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

fn submission(id: &str, generation: u64, text: &str) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source: SubmissionSource::User,
        sender_generation: generation,
        items: vec![SubmissionItem {
            id: format!("{id}:item"),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: text.into(),
            attachments: Vec::new(),
        }],
    }
}

async fn tracked_receipt(
    receipt_rx: &mut mpsc::UnboundedReceiver<SubmissionReceipt>,
) -> SubmissionReceipt {
    tokio::time::timeout(Duration::from_secs(3), receipt_rx.recv())
        .await
        .expect("tracked receipt timeout")
        .expect("tracked receipt channel closed")
}

#[tokio::test]
async fn generation_two_can_observe_but_never_execute_generation_one_custody() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-cross-generation")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    let old = submission("generation-one", 1, "must remain frozen");
    assert_eq!(
        store.accept(&old).unwrap().1,
        SubmissionReceiptDisposition::AcceptedPending
    );
    assert_eq!(store.pause_unstarted().unwrap(), 1);

    let calls = Arc::new(AtomicUsize::new(0));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(CountingModel {
            calls: calls.clone(),
        }),
        ToolRegistry::new(),
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_generation(2);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let actor_task = tokio::spawn(async move { actor.run().await });

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let (old_receipt_tx, mut old_receipt_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::ReconcileStructuredTracked {
            submission: old.clone(),
            receipt_tx: Some(old_receipt_tx),
        })
        .await
        .unwrap();
    let receipt = tracked_receipt(&mut old_receipt_rx).await;
    assert_eq!(receipt.session_generation, 1);
    assert_eq!(
        receipt.disposition,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::PausedPending,
            turn_id: None,
        }
    );

    let current = submission("generation-two", 2, "may execute");
    let (current_receipt_tx, mut current_receipt_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: current.clone(),
            receipt_tx: Some(current_receipt_tx),
        })
        .await
        .unwrap();
    let receipt = tracked_receipt(&mut current_receipt_rx).await;
    assert_eq!(receipt.session_generation, 2);
    assert_eq!(
        receipt.disposition,
        SubmissionReceiptDisposition::AcceptedPending
    );

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if calls.load(Ordering::SeqCst) == 1
                && store
                    .get(&current.id)
                    .unwrap()
                    .is_some_and(|record| record.state == PendingSubmissionState::Committed)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("current generation work must commit");

    assert_eq!(
        store.get(&old.id).unwrap().unwrap().state,
        PendingSubmissionState::PausedPending
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
