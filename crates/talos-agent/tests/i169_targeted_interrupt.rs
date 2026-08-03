use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionEvent, SessionOp,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionSource, TurnCompletionStatus,
    TurnEventPayload,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::mpsc;

struct BlockingModel {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LanguageModel for BlockingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _keep_open = tx;
            std::future::pending::<()>().await;
        });
        Ok(rx)
    }
}

fn submission() -> StructuredSubmission {
    StructuredSubmission {
        id: "targeted-interrupt-batch".into(),
        source: SubmissionSource::User,
        sender_generation: 7,
        items: vec![SubmissionItem {
            id: "targeted-interrupt-item".into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "block until the exact targeted interrupt".into(),
            attachments: Vec::new(),
        }],
    }
}

async fn wait_for_started(eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>) -> String {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), eq_rx.recv())
            .await
            .expect("structured start timeout")
            .expect("session event channel closed before structured start");
        if let SessionEvent::StructuredTurnEvent {
            session_generation: 7,
            submission_id,
            turn_id,
            sequence: 0,
            payload: TurnEventPayload::Started,
            ..
        } = event
            && submission_id == "targeted-interrupt-batch"
        {
            return turn_id;
        }
    }
}

async fn assert_no_structured_cancellation(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    turn_id: &str,
) {
    let quiet = tokio::time::sleep(Duration::from_millis(100));
    tokio::pin!(quiet);
    loop {
        tokio::select! {
            _ = &mut quiet => break,
            event = eq_rx.recv() => {
                let event = event.expect("session event channel closed while checking mismatch interrupt");
                assert!(
                    !matches!(
                        event,
                        SessionEvent::StructuredTurnEvent {
                            ref turn_id: completed_turn,
                            payload: TurnEventPayload::Completed {
                                status: TurnCompletionStatus::Cancelled,
                            },
                            ..
                        } if completed_turn == turn_id
                    ),
                    "a non-matching targeted interrupt must not cancel the active structured Turn"
                );
            }
        }
    }
}

#[tokio::test]
async fn only_exact_generation_and_turn_cancel_structured_work() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-targeted-interrupt")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let calls = Arc::new(AtomicUsize::new(0));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(BlockingModel {
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
    actor.set_generation(7);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::InterruptTurn {
            session_generation: 7,
            turn_id: "no-active-turn".into(),
        })
        .await
        .unwrap();

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission(),
        })
        .await
        .unwrap();
    let turn_id = wait_for_started(&mut eq_rx).await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        store
            .get("targeted-interrupt-batch")
            .unwrap()
            .expect("durable running record")
            .state,
        PendingSubmissionState::Running
    );

    sq_tx
        .send(SessionOp::InterruptTurn {
            session_generation: 6,
            turn_id: turn_id.clone(),
        })
        .await
        .unwrap();
    assert_no_structured_cancellation(&mut eq_rx, &turn_id).await;
    assert_eq!(
        store
            .get("targeted-interrupt-batch")
            .unwrap()
            .expect("record after stale-generation interrupt")
            .state,
        PendingSubmissionState::Running
    );

    sq_tx
        .send(SessionOp::InterruptTurn {
            session_generation: 7,
            turn_id: "wrong-turn-id".into(),
        })
        .await
        .unwrap();
    assert_no_structured_cancellation(&mut eq_rx, &turn_id).await;
    assert_eq!(
        store
            .get("targeted-interrupt-batch")
            .unwrap()
            .expect("record after wrong-turn interrupt")
            .state,
        PendingSubmissionState::Running
    );

    sq_tx
        .send(SessionOp::InterruptTurn {
            session_generation: 7,
            turn_id: turn_id.clone(),
        })
        .await
        .unwrap();

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = eq_rx
                .recv()
                .await
                .expect("session event channel closed before exact cancellation");
            if matches!(
                event,
                SessionEvent::StructuredTurnEvent {
                    ref turn_id: completed_turn,
                    payload: TurnEventPayload::Completed {
                        status: TurnCompletionStatus::Cancelled,
                    },
                    ..
                } if completed_turn == &turn_id
            ) {
                break;
            }
        }
    })
    .await
    .expect("exact targeted interrupt must complete as Cancelled");

    let terminal = store
        .get("targeted-interrupt-batch")
        .unwrap()
        .expect("durable terminal record");
    assert_eq!(terminal.state, PendingSubmissionState::TerminalCancelled);
    assert_eq!(terminal.turn_id.as_deref(), Some(turn_id.as_str()));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
