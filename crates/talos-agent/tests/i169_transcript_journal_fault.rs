use std::path::{Path, PathBuf};
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
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionSource, TurnEventPayload,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::{Notify, mpsc};

struct GatedModel {
    calls: Arc<AtomicUsize>,
    release: Arc<Notify>,
}

#[async_trait]
impl LanguageModel for GatedModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(8);
        let release = self.release.clone();
        tokio::spawn(async move {
            release.notified().await;
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx
                .send(AgentEvent::TextDelta {
                    delta: "committed once".into(),
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

fn make_agent(calls: Arc<AtomicUsize>, release: Arc<Notify>) -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(GatedModel { calls, release }), ToolRegistry::new())
}

fn session_config(workspace_root: &Path) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    }
}

fn submission() -> StructuredSubmission {
    StructuredSubmission {
        id: "transcript-before-journal".into(),
        source: SubmissionSource::User,
        sender_generation: 1,
        items: vec![SubmissionItem {
            id: "transcript-before-journal:item".into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "persist before finalization".into(),
            attachments: Vec::new(),
        }],
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .expect("pending journal path has file name")
        .to_string_lossy();
    path.with_file_name(format!("{file_name}.fault-backup"))
}

async fn wait_for_structured_start(eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>) -> String {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), eq_rx.recv())
            .await
            .expect("structured start timeout")
            .expect("session event channel closed before start");
        if let SessionEvent::StructuredTurnEvent {
            submission_id,
            turn_id,
            sequence: 0,
            payload: TurnEventPayload::Started,
            ..
        } = event
            && submission_id == "transcript-before-journal"
        {
            return turn_id;
        }
    }
}

#[tokio::test]
async fn transcript_commit_survives_journal_finalization_failure_without_provider_replay() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let external_id = "i169-transcript-journal-fault";
    let durable = manager
        .create_or_open_session(external_id)
        .expect("durable session");
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    assert_eq!(
        store
            .advance_runtime_generation(0)
            .expect("operation should succeed"),
        1
    );

    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(Notify::new());
    let (handle, mut actor) = AppServerSession::new(
        make_agent(calls.clone(), release.clone()),
        session_config(temp.path()),
    );
    actor.set_generation(1);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission(),
        })
        .await
        .expect("operation should succeed");
    let turn_id = wait_for_structured_start(&mut eq_rx).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        store
            .get("transcript-before-journal")
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::Running
    );

    let journal_path = store.path().to_path_buf();
    let journal_backup = backup_path(&journal_path);
    std::fs::rename(&journal_path, &journal_backup).expect("move pending journal aside");
    std::fs::create_dir(&journal_path).expect("block pending journal reopen with a directory");

    release.notify_one();
    let mut entries_committed = false;
    let mut finalization_failed = false;
    tokio::time::timeout(Duration::from_secs(5), async {
        while !(entries_committed && finalization_failed) {
            match eq_rx.recv().await {
                Some(SessionEvent::EntriesCommitted {
                    turn_id: committed_turn,
                    entry_ids,
                    ..
                }) if committed_turn == turn_id => {
                    assert!(!entry_ids.is_empty());
                    entries_committed = true;
                }
                Some(SessionEvent::Error { message })
                    if message.contains("failed to finalize structured turn custody") =>
                {
                    finalization_failed = true;
                }
                Some(_) => {}
                None => panic!("session event channel closed before fault evidence"),
            }
        }
    })
    .await
    .expect("transcript commit and journal failure evidence timeout");

    std::fs::remove_dir(&journal_path).expect("remove pending journal blocker");
    std::fs::rename(&journal_backup, &journal_path).expect("restore pending journal");
    assert_eq!(
        store
            .get("transcript-before-journal")
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::Running,
        "journal finalization failure must leave recoverable Running custody"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    let reopened = manager
        .create_or_open_session(external_id)
        .expect("reopen durable session");
    let transcript = reopened.read_messages().expect("read committed transcript");
    assert!(transcript.iter().any(
        |message| matches!(message, Message::User { content } if content == "persist before finalization")
    ));
    assert!(transcript.iter().any(
        |message| matches!(message, Message::Assistant { content, .. } if content == "committed once")
    ));

    let restart_release = Arc::new(Notify::new());
    let (restart_handle, mut restarted_actor) = AppServerSession::new(
        make_agent(calls.clone(), restart_release),
        session_config(temp.path()),
    );
    restarted_actor.set_generation(1);
    restarted_actor.set_durable_persistence(reopened, PersistencePolicy::default());
    let restart_sq_tx = restart_handle.sq_tx;
    drop(restart_handle.eq_rx);
    let restart_task = tokio::spawn(async move { restarted_actor.run().await });

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let record = store
                .get("transcript-before-journal")
                .expect("operation should succeed")
                .expect("pending journal record");
            if record.state == PendingSubmissionState::Committed {
                assert_eq!(record.turn_id.as_deref(), Some(turn_id.as_str()));
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("restart must finalize transcript-backed Running custody");

    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "restart reconciliation must not replay the Provider turn"
    );
    restart_sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    restart_task.await.expect("operation should succeed");
}
