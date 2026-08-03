use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
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

struct RecordingDelayModel {
    calls: Arc<AtomicUsize>,
    inputs: Arc<Mutex<Vec<String>>>,
    delay: Duration,
}

#[async_trait]
impl LanguageModel for RecordingDelayModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let input = messages
            .iter()
            .rev()
            .find_map(|message| match message {
                Message::User { content } => Some(content.clone()),
                _ => None,
            })
            .unwrap_or_default();
        self.inputs
            .lock()
            .expect("recorded input lock poisoned")
            .push(input);

        let delay = self.delay;
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::TurnStart).await;
            tokio::time::sleep(delay).await;
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

fn make_agent(calls: Arc<AtomicUsize>, inputs: Arc<Mutex<Vec<String>>>, delay: Duration) -> Agent {
    #[allow(deprecated)]
    Agent::new(
        Arc::new(RecordingDelayModel {
            calls,
            inputs,
            delay,
        }),
        ToolRegistry::new(),
    )
}

fn session_config(workspace_root: &std::path::Path) -> SessionConfig {
    SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 128_000,
    }
}

fn submission(index: usize, text: impl Into<String>) -> StructuredSubmission {
    StructuredSubmission {
        id: format!("stress_batch_{index}"),
        source: SubmissionSource::User,
        sender_generation: 0,
        items: vec![SubmissionItem {
            id: format!("stress_item_{index}"),
            enqueue_sequence: index as u64,
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
        let event = tokio::time::timeout(Duration::from_secs(10), eq_rx.recv())
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

async fn wait_for_completions(
    eq_rx: &mut mpsc::UnboundedReceiver<SessionEvent>,
    wanted: usize,
) -> Vec<String> {
    let mut completed = Vec::with_capacity(wanted);
    tokio::time::timeout(Duration::from_secs(20), async {
        while completed.len() < wanted {
            let event = eq_rx
                .recv()
                .await
                .expect("session event channel closed before completion");
            if let SessionEvent::StructuredTurnEvent {
                submission_id,
                payload:
                    TurnEventPayload::Completed {
                        status: TurnCompletionStatus::Success { .. },
                    },
                ..
            } = event
            {
                completed.push(submission_id);
            }
        }
    })
    .await
    .expect("timed out waiting for structured completions");
    completed
}

async fn wait_for_state(
    store: &PendingSubmissionStore,
    submission_id: &str,
    wanted: PendingSubmissionState,
) {
    tokio::time::timeout(Duration::from_secs(10), async {
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
async fn duplicate_submit_and_reconcile_storm_executes_one_identity_once() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-stress-identity")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let calls = Arc::new(AtomicUsize::new(0));
    let inputs = Arc::new(Mutex::new(Vec::new()));
    let (handle, mut actor) = AppServerSession::new(
        make_agent(calls.clone(), inputs.clone(), Duration::from_millis(75)),
        session_config(temp.path()),
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    let work = submission(0, "execute exactly once");
    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: work.clone(),
        })
        .await
        .unwrap();
    assert_eq!(
        wait_for_receipt(&mut eq_rx, &work.id).await,
        SubmissionReceiptDisposition::AcceptedPending
    );

    let mut senders = Vec::new();
    for attempt in 0..32 {
        let tx = sq_tx.clone();
        let duplicate = work.clone();
        senders.push(tokio::spawn(async move {
            let op = if attempt % 2 == 0 {
                SessionOp::SubmitStructured {
                    submission: duplicate,
                }
            } else {
                SessionOp::ReconcileStructured {
                    submission: duplicate,
                }
            };
            tx.send(op).await.expect("send duplicate stress operation");
        }));
    }
    for sender in senders {
        sender.await.unwrap();
    }

    assert_eq!(
        wait_for_completions(&mut eq_rx, 1).await,
        vec![work.id.clone()]
    );
    wait_for_state(&store, &work.id, PendingSubmissionState::Committed).await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        inputs
            .lock()
            .expect("recorded input lock poisoned")
            .as_slice(),
        ["execute exactly once".to_string()]
    );

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}

#[tokio::test]
async fn distinct_submissions_reach_provider_in_fifo_order_under_burst_load() {
    const COUNT: usize = 8;

    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager.create_or_open_session("i169-stress-fifo").unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let calls = Arc::new(AtomicUsize::new(0));
    let inputs = Arc::new(Mutex::new(Vec::new()));
    let (handle, mut actor) = AppServerSession::new(
        make_agent(calls.clone(), inputs.clone(), Duration::from_millis(10)),
        session_config(temp.path()),
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    let works = (0..COUNT)
        .map(|index| submission(index, format!("fifo-{index}")))
        .collect::<Vec<_>>();
    for work in &works {
        sq_tx
            .send(SessionOp::SubmitStructured {
                submission: work.clone(),
            })
            .await
            .unwrap();
    }

    let completed = wait_for_completions(&mut eq_rx, COUNT).await;
    let expected_ids = works.iter().map(|work| work.id.clone()).collect::<Vec<_>>();
    assert_eq!(completed, expected_ids);
    assert_eq!(calls.load(Ordering::SeqCst), COUNT);

    let expected_inputs = (0..COUNT)
        .map(|index| format!("fifo-{index}"))
        .collect::<Vec<_>>();
    assert_eq!(
        *inputs.lock().expect("recorded input lock poisoned"),
        expected_inputs
    );

    for work in &works {
        wait_for_state(&store, &work.id, PendingSubmissionState::Committed).await;
    }

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}

#[tokio::test]
async fn fixed_seed_interleaving_preserves_single_execution_and_fifo_custody() {
    const COUNT: usize = 12;
    const STEPS: usize = 128;
    const SEED: u64 = 0x5eed_0169_c057_0d1e;

    let temp = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-fixed-seed-interleaving")
        .unwrap();
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let calls = Arc::new(AtomicUsize::new(0));
    let inputs = Arc::new(Mutex::new(Vec::new()));
    let (handle, mut actor) = AppServerSession::new(
        make_agent(calls.clone(), inputs.clone(), Duration::from_millis(2)),
        session_config(temp.path()),
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    let works = (0..COUNT)
        .map(|index| submission(index + 100, format!("seeded-{index}")))
        .collect::<Vec<_>>();
    let mut first_submit_seen = vec![false; COUNT];
    let mut expected_order = Vec::with_capacity(COUNT);
    let mut state = SEED;

    for step in 0..STEPS {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let index = ((state >> 32) as usize) % COUNT;
        let force_submit = !first_submit_seen[index] || (state & 0b11) == 0;
        let work = works[index].clone();
        let operation = if force_submit {
            if !first_submit_seen[index] {
                first_submit_seen[index] = true;
                expected_order.push(index);
            }
            SessionOp::SubmitStructured { submission: work }
        } else {
            SessionOp::ReconcileStructured { submission: work }
        };
        sq_tx.send(operation).await.unwrap();
        if step % 7 == 0 {
            tokio::task::yield_now().await;
        }
    }

    for index in 0..COUNT {
        if !first_submit_seen[index] {
            first_submit_seen[index] = true;
            expected_order.push(index);
            sq_tx
                .send(SessionOp::SubmitStructured {
                    submission: works[index].clone(),
                })
                .await
                .unwrap();
        }
    }

    let completed = wait_for_completions(&mut eq_rx, COUNT).await;
    let expected_ids = expected_order
        .iter()
        .map(|index| works[*index].id.clone())
        .collect::<Vec<_>>();
    assert_eq!(completed, expected_ids, "fixed seed: {SEED:#x}");
    assert_eq!(calls.load(Ordering::SeqCst), COUNT, "fixed seed: {SEED:#x}");

    let expected_inputs = expected_order
        .iter()
        .map(|index| format!("seeded-{index}"))
        .collect::<Vec<_>>();
    assert_eq!(
        *inputs.lock().expect("recorded input lock poisoned"),
        expected_inputs,
        "fixed seed: {SEED:#x}"
    );
    for work in &works {
        wait_for_state(&store, &work.id, PendingSubmissionState::Committed).await;
    }

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
