use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionOp, StructuredSubmission,
    SubmissionItem, SubmissionKind, SubmissionSource,
};
use talos_core::tool::ToolRegistry;
use talos_session::{
    PendingSubmissionStore, SessionManager, SessionMetadata, TurnTranscriptOutcome,
    TurnTranscriptOutcomeRecord,
};
use tokio::sync::mpsc;

struct CountingModel {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LanguageModel for CountingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }
}

fn make_agent(calls: Arc<AtomicUsize>) -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(CountingModel { calls }), ToolRegistry::new())
}

fn config(workspace_root: &Path) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    }
}

fn submission(id: &str) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source: SubmissionSource::User,
        sender_generation: 1,
        items: vec![SubmissionItem {
            id: format!("{id}:item"),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "must never replay".into(),
            attachments: Vec::new(),
        }],
    }
}

async fn assert_terminal_recovery(
    external_id: &str,
    submission_id: &str,
    turn_id: &str,
    outcome: TurnTranscriptOutcome,
    expected: PendingSubmissionState,
) {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
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
    let frozen = submission(submission_id);
    store.accept(&frozen).expect("durable acceptance");
    store
        .mark_running(submission_id, turn_id)
        .expect("mark running");
    durable
        .session()
        .append_turn_transcript_outcome(&TurnTranscriptOutcomeRecord::new(turn_id, outcome))
        .expect("terminal outcome marker");

    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) = AppServerSession::new(make_agent(calls.clone()), config(temp.path()));
    actor.set_generation(1);
    actor.set_persistence(durable.session().clone(), SessionMetadata::default());
    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let task = tokio::spawn(async move { actor.run().await });

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let record = store
                .get(submission_id)
                .expect("read pending journal")
                .expect("running record remains addressable");
            if record.state == expected {
                assert_eq!(record.turn_id.as_deref(), Some(turn_id));
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("terminal recovery timeout");

    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "terminal transcript evidence must never replay Provider execution"
    );
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    task.await.expect("operation should succeed");
}

#[tokio::test]
async fn running_error_outcome_recovers_as_terminal_error_without_replay() {
    assert_terminal_recovery(
        "i169-running-error-outcome",
        "running-error-submission",
        "running-error-turn",
        TurnTranscriptOutcome::Error,
        PendingSubmissionState::TerminalError,
    )
    .await;
}

#[tokio::test]
async fn running_cancelled_outcome_recovers_as_terminal_cancelled_without_replay() {
    assert_terminal_recovery(
        "i169-running-cancelled-outcome",
        "running-cancelled-submission",
        "running-cancelled-turn",
        TurnTranscriptOutcome::Cancelled,
        PendingSubmissionState::TerminalCancelled,
    )
    .await;
}

#[tokio::test]
async fn running_success_outcome_recovers_as_committed_without_replay() {
    assert_terminal_recovery(
        "i169-running-success-outcome",
        "running-success-submission",
        "running-success-turn",
        TurnTranscriptOutcome::Success,
        PendingSubmissionState::Committed,
    )
    .await;
}

#[tokio::test]
async fn ordinary_transcript_entry_without_terminal_outcome_remains_frozen_running() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-running-ambiguous-outcome")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    assert_eq!(
        store
            .advance_runtime_generation(0)
            .expect("operation should succeed"),
        1
    );
    let frozen = submission("running-ambiguous-submission");
    let turn_id = "running-ambiguous-turn";
    store.accept(&frozen).expect("durable acceptance");
    store
        .mark_running(&frozen.id, turn_id)
        .expect("mark running");
    durable
        .session()
        .append_with_metadata(
            &Message::Assistant {
                content: "partial provider output".into(),
                tool_calls: Vec::new(),
                reasoning: None,
            },
            SessionMetadata {
                turn_id: Some(turn_id.into()),
                ..SessionMetadata::default()
            },
        )
        .expect("partial transcript entry");

    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) = AppServerSession::new(make_agent(calls.clone()), config(temp.path()));
    actor.set_generation(1);
    actor.set_persistence(durable.session().clone(), SessionMetadata::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let task = tokio::spawn(async move { actor.run().await });

    let event = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = eq_rx.recv().await.expect("session event channel");
            if let talos_core::session::SessionEvent::Error { message } = event
                && message.contains("remains frozen in Running state")
            {
                break message;
            }
        }
    })
    .await
    .expect("ambiguous recovery diagnostic timeout");
    assert!(event.contains(turn_id));
    let record = store
        .get(&frozen.id)
        .expect("read pending journal")
        .expect("running record remains addressable");
    assert_eq!(record.state, PendingSubmissionState::Running);
    assert_eq!(record.turn_id.as_deref(), Some(turn_id));
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    task.await.expect("operation should succeed");
}
