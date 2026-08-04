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

#[test]
fn generation_advance_refuses_to_orphan_generation_one_custody() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-cross-generation")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    assert_eq!(store.advance_runtime_generation(0).unwrap(), 1);

    let retained = submission("generation-one", 1, "must retain custody");
    assert_eq!(
        store.accept(&retained).unwrap().1,
        SubmissionReceiptDisposition::AcceptedPending
    );
    assert_eq!(store.pause_unstarted().unwrap(), 1);
    assert!(matches!(
        store.advance_runtime_generation(1),
        Err(talos_session::PendingSubmissionError::GenerationBusy {
            generation: 1,
            pending: 1,
        })
    ));
    assert_eq!(store.runtime_generation().unwrap(), 1);

    store.cancel_unstarted(&retained.id).unwrap();
    assert_eq!(store.advance_runtime_generation(1).unwrap(), 2);
    let stale = submission("late-generation-one", 1, "must reject after fence");
    assert_eq!(
        store.accept(&stale).unwrap().1,
        SubmissionReceiptDisposition::Rejected {
            reason: talos_core::session::SubmissionRejectionReason::WrongGeneration,
        }
    );
    assert!(store.get(&stale.id).unwrap().is_none());
}

#[tokio::test]
async fn process_reconstruction_rehydrates_generation_one_and_resumes_custody() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-generation-restart")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    assert_eq!(store.advance_runtime_generation(0).unwrap(), 1);

    let retained = submission("generation-one-retained", 1, "resume after restart");
    assert_eq!(
        store.accept(&retained).unwrap().1,
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
    let recovered_generation = store.runtime_generation().unwrap();
    assert_eq!(recovered_generation, 1);
    actor.set_generation(recovered_generation);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let resume = submission("generation-one-resume", 1, "explicit resume authority");
    let (resume_receipt_tx, mut resume_receipt_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: resume.clone(),
            receipt_tx: Some(resume_receipt_tx),
        })
        .await
        .unwrap();
    let receipt = tracked_receipt(&mut resume_receipt_rx).await;
    assert!(receipt.disposition.has_durable_custody());
    assert_eq!(receipt.session_generation, 1);

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let retained_committed = store
                .get(&retained.id)
                .unwrap()
                .is_some_and(|record| record.state == PendingSubmissionState::Committed);
            let resume_committed = store
                .get(&resume.id)
                .unwrap()
                .is_some_and(|record| record.state == PendingSubmissionState::Committed);
            if retained_committed && resume_committed && calls.load(Ordering::SeqCst) == 2 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("reconstructed generation must resume retained custody exactly once");

    let stale = submission("generation-zero-stale-after-restart", 0, "must reject");
    let (stale_receipt_tx, mut stale_receipt_rx) = mpsc::unbounded_channel();
    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: stale.clone(),
            receipt_tx: Some(stale_receipt_tx),
        })
        .await
        .unwrap();
    let stale_receipt = tracked_receipt(&mut stale_receipt_rx).await;
    assert!(matches!(
        stale_receipt.disposition,
        SubmissionReceiptDisposition::Rejected { .. }
    ));
    assert!(store.get(&stale.id).unwrap().is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    while eq_rx.try_recv().is_ok() {}
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
