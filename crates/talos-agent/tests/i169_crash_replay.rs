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
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionReceiptDisposition,
    SubmissionSource, TurnCompletionStatus, TurnEventPayload,
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

async fn wait_for_receipt(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    wanted_submission_id: &str,
) -> SubmissionReceiptDisposition {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), eq_rx.recv())
            .await
            .expect("timed out waiting for durable receipt")
            .expect("session event channel closed before durable receipt");
        if let SessionEvent::SubmissionReceipt {
            submission_id,
            disposition,
            ..
        } = event
            && submission_id == wanted_submission_id
        {
            return disposition;
        }
    }
}

async fn wait_for_success(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    wanted_submission_id: &str,
) {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), eq_rx.recv())
            .await
            .expect("timed out waiting for structured completion")
            .expect("session event channel closed before structured completion");
        if matches!(
            event,
            SessionEvent::StructuredTurnEvent {
                submission_id,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Success { .. },
                },
                ..
            } if submission_id == wanted_submission_id
        ) {
            return;
        }
    }
}

async fn wait_for_state(
    store: &PendingSubmissionStore,
    submission_id: &str,
    wanted: PendingSubmissionState,
) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let state = store
                .get(submission_id)
                .expect("read pending journal")
                .expect("pending submission record")
                .state;
            if state == wanted {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("timed out waiting for pending journal state");
}

#[tokio::test]
async fn orphan_running_submission_is_never_auto_replayed() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-running-orphan")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    let work = submission("orphan_batch", "orphan_item", 1, "do not replay");

    let (_, accepted) = store.accept(&work).unwrap();
    assert_eq!(accepted, SubmissionReceiptDisposition::AcceptedPending);
    store.mark_running(&work.id, "orphan_turn").unwrap();

    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) =
        AppServerSession::new(make_agent(calls.clone()), session_config(temp.path()));
    actor.set_generation(1);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    sq_tx
        .send(SessionOp::ReconcileStructured {
            submission: work.clone(),
        })
        .await
        .unwrap();
    let disposition = wait_for_receipt(&mut eq_rx, &work.id).await;
    assert_eq!(
        disposition,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Running,
            turn_id: Some("orphan_turn".into()),
        }
    );

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        store.get(&work.id).unwrap().unwrap().state,
        PendingSubmissionState::Running
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}

#[tokio::test]
async fn paused_reconcile_is_observational_until_explicit_user_resume() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-paused-recovery")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    let work = submission("paused_batch", "paused_item", 2, "recover me once");

    let (_, accepted) = store.accept(&work).unwrap();
    assert_eq!(accepted, SubmissionReceiptDisposition::AcceptedPending);
    assert_eq!(store.pause_unstarted().unwrap(), 1);

    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) =
        AppServerSession::new(make_agent(calls.clone()), session_config(temp.path()));
    actor.set_generation(2);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        store.get(&work.id).unwrap().unwrap().state,
        PendingSubmissionState::PausedPending
    );

    sq_tx
        .send(SessionOp::ReconcileStructured {
            submission: work.clone(),
        })
        .await
        .unwrap();
    let disposition = wait_for_receipt(&mut eq_rx, &work.id).await;
    assert_eq!(
        disposition,
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::PausedPending,
            turn_id: None,
        }
    );

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "observational reconciliation must not resume paused work"
    );
    assert_eq!(
        store.get(&work.id).unwrap().unwrap().state,
        PendingSubmissionState::PausedPending
    );

    let resume = submission(
        "resume_batch",
        "resume_item",
        2,
        "explicitly resume retained work",
    );
    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: resume.clone(),
        })
        .await
        .unwrap();
    assert_eq!(
        wait_for_receipt(&mut eq_rx, &resume.id).await,
        SubmissionReceiptDisposition::AcceptedPending
    );

    wait_for_success(&mut eq_rx, &work.id).await;
    wait_for_success(&mut eq_rx, &resume.id).await;
    wait_for_state(&store, &work.id, PendingSubmissionState::Committed).await;
    wait_for_state(&store, &resume.id, PendingSubmissionState::Committed).await;
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let reopened = manager
        .create_or_open_session("i169-paused-recovery")
        .unwrap();
    let messages = reopened.read_messages().unwrap();
    let user_messages = messages
        .iter()
        .filter_map(|message| match message {
            Message::User { content } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        user_messages,
        vec!["recover me once", "explicitly resume retained work"],
        "older retained user work must run before the explicit resuming item"
    );
    assert_eq!(
        messages
            .iter()
            .filter(|message| {
                matches!(message, Message::Assistant { content, .. } if content == "done")
            })
            .count(),
        2
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
