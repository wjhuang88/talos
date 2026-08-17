use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_conversation::{
    ContentOutput, ConversationEngine, MessageSource, ModelInfo, SteeringQueueSnapshot, UiOutput,
    UserInput,
};
use talos_core::message::{AgentEvent, Message, StopReason, Usage};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::session::{
    PendingSubmissionState, RuntimePolicy, SessionConfig, SessionEvent, SessionOp,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionReceiptDisposition,
    SubmissionSource, TurnCompletionStatus, TurnEventPayload,
};
use talos_core::tool::ToolRegistry;
use talos_session::{PendingSubmissionStore, PersistencePolicy, SessionManager};
use tokio::sync::mpsc;

use crate::session_transition::register_generation_bound_sender;
use crate::tui_bridge::{ConversationLoopIo, SessionLifecycleRequest, run_conversation_loop};

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

struct RecordingModel {
    order: Arc<Mutex<Vec<String>>>,
}

struct PendingDispatchModel {
    entered_tx: mpsc::UnboundedSender<()>,
    dropped_tx: mpsc::UnboundedSender<()>,
}

struct DispatchDropGuard(mpsc::UnboundedSender<()>);

impl Drop for DispatchDropGuard {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

#[async_trait]
impl LanguageModel for PendingDispatchModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let _drop_guard = DispatchDropGuard(self.dropped_tx.clone());
        let _ = self.entered_tx.send(());
        std::future::pending().await
    }
}

#[async_trait]
impl LanguageModel for RecordingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let text = messages
            .iter()
            .rev()
            .find_map(|message| match message {
                Message::User { content } => Some(content.clone()),
                Message::Multimodal { parts } => parts.iter().find_map(|part| match part {
                    talos_core::message::ContentPart::Text { text } => Some(text.clone()),
                    talos_core::message::ContentPart::Image { .. } => None,
                }),
                _ => None,
            })
            .unwrap_or_default();
        self.order
            .lock()
            .expect("operation should succeed")
            .push(text.clone());
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx
                .send(AgentEvent::TextDelta {
                    delta: format!("completed:{text}"),
                })
                .await;
            let _ = tx
                .send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                })
                .await;
        });
        Ok(rx)
    }
}

fn runtime_skills() -> Arc<tokio::sync::Mutex<crate::skill_runtime::RuntimeSkills>> {
    let skills_dir = tempfile::tempdir().expect("operation should succeed");
    Arc::new(tokio::sync::Mutex::new(
        crate::skill_runtime::discover_runtime_skills(skills_dir.path(), false)
            .expect("operation should succeed"),
    ))
}

fn model_info(context_limit: u32) -> ModelInfo {
    ModelInfo {
        model_name: "i169-model".into(),
        provider: "test-provider".into(),
        context_limit: Some(context_limit),
        ..Default::default()
    }
}

fn retained_submission(id: &str, sequence: u64, text: &str) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source: SubmissionSource::User,
        sender_generation: 0,
        items: vec![SubmissionItem {
            id: format!("{id}:item"),
            enqueue_sequence: sequence,
            kind: SubmissionKind::UserTurn,
            text: text.into(),
            attachments: Vec::new(),
        }],
    }
}

async fn wait_for_order(order: &Arc<Mutex<Vec<String>>>, expected: &[&str]) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let actual = order.lock().expect("operation should succeed").clone();
            if actual.len() >= expected.len() {
                assert_eq!(
                    &actual[..expected.len()],
                    &expected
                        .iter()
                        .map(|value| (*value).to_string())
                        .collect::<Vec<_>>()
                );
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Provider order timeout");
}

async fn wait_for_visible_user_order(
    ui_rx: &mut mpsc::UnboundedReceiver<UiOutput>,
    expected: &[&str],
) -> Vec<String> {
    let mut visible = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), async {
        while visible.len() < expected.len() {
            match ui_rx.recv().await {
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::User,
                    text,
                })) => {
                    let expected_next = expected[visible.len()];
                    assert!(
                        text.contains(expected_next),
                        "visible user projection was out of order: expected {expected_next:?}, got {text:?}"
                    );
                    visible.push(expected_next.to_string());
                }
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                })) if text.contains("ignored stale structured turn start")
                    || text.contains("ignored stale or out-of-order structured completion")
                    || text.contains("ignored uncorrelated structured submission projection") =>
                {
                    panic!("retained lifecycle was discarded: {text}");
                }
                Some(_) => {}
                None => panic!("Bridge UI channel closed before visible FIFO projection"),
            }
        }
    })
    .await
    .expect("visible user order timeout");
    visible
}

#[tokio::test]
async fn bridge_and_actor_retain_durable_custody_when_request_plan_exceeds_budget() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-bridge-budget")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let pending_store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);

    let provider_calls = Arc::new(AtomicUsize::new(0));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(CountingModel {
            calls: provider_calls.clone(),
        }),
        ToolRegistry::new(),
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 64,
    };
    let (session_handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let actor_sq_tx = session_handle.sq_tx.clone();
    let actor_task = tokio::spawn(async move { actor.run().await });

    let engine = ConversationEngine::new("budget-model".into(), "test-provider".into());
    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_sq_watch_tx, sq_watch_rx) = tokio::sync::watch::channel(session_handle.sq_tx);
    let (_model_tx, model_rx) = tokio::sync::watch::channel(model_info(64));
    let (session_tx, _session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();

    let bridge_task = tokio::spawn(run_conversation_loop(
        engine,
        ConversationLoopIo {
            agent_rx: session_handle.eq_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: sq_watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills: runtime_skills(),
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("preserve this exact user input".into()))
        .expect("operation should succeed");

    let mut last_snapshot = SteeringQueueSnapshot {
        entries: Vec::new(),
        total_count: 0,
        omitted_count: 0,
    };
    let pause_message = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match ui_rx.recv().await {
                Some(UiOutput::SteeringQueueSnapshot(snapshot)) => last_snapshot = snapshot,
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                })) if text.contains("paused before Provider start") => break text,
                Some(_) => {}
                None => panic!("bridge UI channel closed before durable pause"),
            }
        }
    })
    .await
    .expect("Bridge must observe Actor pre-Provider pause");

    assert!(pause_message.contains("ContextBudgetExceeded"));
    assert_eq!(
        last_snapshot.total_count, 0,
        "Engine reservation must be removed only after Actor durable custody"
    );
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);

    let recovered = pending_store
        .recover_unstarted()
        .expect("recover durable paused submission");
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].state, PendingSubmissionState::PausedPending);
    assert_eq!(recovered[0].submission.items.len(), 1);
    assert_eq!(
        recovered[0].submission.items[0].text,
        "preserve this exact user input"
    );

    user_tx
        .send(UserInput::Exit)
        .expect("operation should succeed");
    tokio::time::timeout(Duration::from_secs(2), bridge_task)
        .await
        .expect("bridge exit timeout")
        .expect("bridge task");
    actor_sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    tokio::time::timeout(Duration::from_secs(2), actor_task)
        .await
        .expect("Actor shutdown timeout")
        .expect("Actor task");
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn resumed_structured_turn_cancel_crosses_bridge_actor_and_durable_boundaries() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let seeded = manager
        .create_or_open_session("i209-resumed-cancel")
        .expect("durable session");
    for index in 0..2_000 {
        seeded
            .session()
            .append(&Message::User {
                content: format!("persisted history row {index:03} {}", "x".repeat(128)),
            })
            .expect("seed persisted history");
    }
    let session_id = seeded.id().to_string();
    drop(seeded);

    let resumed = manager
        .create_or_open_session("i209-resumed-cancel")
        .expect("resume durable session");
    let initial_history = resumed.read_messages().expect("read resumed history");
    assert_eq!(initial_history.len(), 2_000);
    let store = PendingSubmissionStore::for_session_file(resumed.file_path(), &session_id);
    assert_eq!(
        store
            .advance_runtime_generation(0)
            .expect("advance resumed runtime generation"),
        1
    );

    let (entered_tx, mut entered_rx) = mpsc::unbounded_channel();
    let (dropped_tx, mut dropped_rx) = mpsc::unbounded_channel();
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(PendingDispatchModel {
            entered_tx,
            dropped_tx,
        }),
        ToolRegistry::new(),
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp.path().to_path_buf(),
        initial_history,
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_generation(1);
    actor.set_durable_persistence(resumed, PersistencePolicy::default());
    let actor_sq_tx = handle.sq_tx.clone();
    let actor_task = tokio::spawn(async move { actor.run().await });

    // The proxy preserves Actor custody while making the exact bridge command
    // observable as an independent cancellation boundary.
    let (proxy_tx, mut proxy_rx) = mpsc::channel(8);
    register_generation_bound_sender(&proxy_tx, 1);
    let (interrupt_seen_tx, mut interrupt_seen_rx) = mpsc::unbounded_channel();
    let (submission_seen_tx, mut submission_seen_rx) = mpsc::unbounded_channel();
    let actor_commands = actor_sq_tx.clone();
    let proxy_task = tokio::spawn(async move {
        while let Some(operation) = proxy_rx.recv().await {
            if let SessionOp::SubmitStructured { submission } = &operation {
                let _ = submission_seen_tx.send(submission.id.clone());
            }
            if let SessionOp::InterruptTurn {
                session_generation,
                turn_id,
            } = &operation
            {
                let _ = interrupt_seen_tx.send((*session_generation, turn_id.clone()));
            }
            if actor_commands.send(operation).await.is_err() {
                break;
            }
        }
    });

    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_watch_tx, watch_rx) = tokio::sync::watch::channel(proxy_tx);
    let (_model_tx, model_rx) = tokio::sync::watch::channel(model_info(128_000));
    let (session_tx, _session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let bridge_task = tokio::spawn(run_conversation_loop(
        ConversationEngine::new("i209-model".into(), "test-provider".into()),
        ConversationLoopIo {
            agent_rx: handle.eq_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills: runtime_skills(),
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("cancel this resumed turn".into()))
        .expect("submit resumed turn");
    let submission_id = tokio::time::timeout(Duration::from_secs(2), submission_seen_rx.recv())
        .await
        .expect("bridge submits structured turn")
        .expect("structured submission observation");
    tokio::time::timeout(Duration::from_secs(5), entered_rx.recv())
        .await
        .expect("provider dispatch starts")
        .expect("provider dispatch signal");

    // This is the message emitted by the TUI Esc entry point.
    user_tx
        .send(UserInput::Cancel)
        .expect("send Esc cancellation message");
    let (generation, turn_id) =
        tokio::time::timeout(Duration::from_secs(2), interrupt_seen_rx.recv())
            .await
            .expect("bridge emits targeted interrupt promptly")
            .expect("targeted interrupt observation");
    assert_eq!(generation, 1);
    assert!(!turn_id.is_empty());
    tokio::time::timeout(Duration::from_secs(2), dropped_rx.recv())
        .await
        .expect("Actor cancellation drops pending provider dispatch")
        .expect("provider dispatch drop signal");

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if store
                .get(&submission_id)
                .expect("read pending submission")
                .is_some_and(|record| {
                    record.state == PendingSubmissionState::TerminalCancelled
                        && record.turn_id.as_deref() == Some(turn_id.as_str())
                })
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("durable terminal cancellation");

    let mut saw_terminal_cancelled = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(output) = ui_rx.recv().await {
            if matches!(
                output,
                UiOutput::Status(status)
                    if !status.is_processing
                        && status.phase == Some(talos_conversation::TurnPhase::Cancelled)
            ) {
                saw_terminal_cancelled = true;
                break;
            }
        }
    })
    .await
    .expect("terminal cancelled status reaches TUI");
    assert!(saw_terminal_cancelled);

    user_tx.send(UserInput::Exit).expect("exit bridge");
    bridge_task.await.expect("bridge task");
    actor_sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown Actor");
    actor_task.await.expect("Actor task");
    proxy_task.abort();
}

#[tokio::test]
async fn coalesced_sender_replacements_submit_and_ack_exact_generation_two() {
    let (sender_zero, mut receiver_zero) = mpsc::channel(8);
    let (sender_one, mut receiver_one) = mpsc::channel(8);
    let (sender_two, mut receiver_two) = mpsc::channel(8);
    register_generation_bound_sender(&sender_zero, 0);
    register_generation_bound_sender(&sender_one, 1);
    register_generation_bound_sender(&sender_two, 2);

    let (watch_tx, watch_rx) = tokio::sync::watch::channel(sender_zero);
    watch_tx.send(sender_one).expect("operation should succeed");
    watch_tx.send(sender_two).expect("operation should succeed");

    let (agent_tx, agent_rx) = mpsc::unbounded_channel();
    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_model_tx, model_rx) = tokio::sync::watch::channel(model_info(128_000));
    let (session_tx, _session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let bridge_task = tokio::spawn(run_conversation_loop(
        ConversationEngine::new("i169-model".into(), "test-provider".into()),
        ConversationLoopIo {
            agent_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills: runtime_skills(),
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("generation two".into()))
        .expect("operation should succeed");
    let operation = tokio::time::timeout(Duration::from_secs(5), receiver_two.recv())
        .await
        .expect("G2 dispatch timeout")
        .expect("G2 sender remains open");
    let SessionOp::SubmitStructured { submission } = operation else {
        panic!("expected structured submission on G2");
    };
    assert_eq!(submission.sender_generation, 2);
    assert!(receiver_zero.try_recv().is_err());
    assert!(receiver_one.try_recv().is_err());

    let receipt_id = "receipt-generation-two".to_string();
    agent_tx
        .send(SessionEvent::SubmissionReceipt {
            session_id: "session-generation-two".into(),
            session_generation: 2,
            source: SubmissionSource::User,
            submission_id: submission.id.clone(),
            reservation_id: format!("reservation:{}", submission.id),
            receipt_id: receipt_id.clone(),
            item_count: 1,
            total_text_bytes: submission.total_text_bytes(),
            disposition: SubmissionReceiptDisposition::AcceptedPending,
        })
        .expect("operation should succeed");
    agent_tx
        .send(SessionEvent::StructuredTurnEvent {
            session_id: "session-generation-two".into(),
            session_generation: 2,
            source: SubmissionSource::User,
            submission_id: submission.id.clone(),
            receipt_id: receipt_id.clone(),
            turn_id: "turn-generation-two".into(),
            sequence: 0,
            payload: TurnEventPayload::Started,
        })
        .expect("operation should succeed");
    agent_tx
        .send(SessionEvent::StructuredTurnEvent {
            session_id: "session-generation-two".into(),
            session_generation: 2,
            source: SubmissionSource::User,
            submission_id: submission.id,
            receipt_id,
            turn_id: "turn-generation-two".into(),
            sequence: 1,
            payload: TurnEventPayload::Completed {
                status: TurnCompletionStatus::Success {
                    final_text: String::new(),
                    new_messages: Vec::new(),
                },
            },
        })
        .expect("operation should succeed");

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match ui_rx.recv().await {
                Some(UiOutput::SteeringQueueSnapshot(snapshot)) if snapshot.total_count == 0 => {
                    break;
                }
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                })) => panic!("unexpected Bridge error: {text}"),
                Some(_) => {}
                None => panic!("Bridge closed before G2 receipt committed escrow"),
            }
        }
    })
    .await
    .expect("G2 receipt projection timeout");

    user_tx
        .send(UserInput::Exit)
        .expect("operation should succeed");
    bridge_task.await.expect("operation should succeed");
}

#[tokio::test]
async fn bridge_adopts_retained_user_fifo_before_new_resume_submission() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-retained-bridge-order")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    let u1 = retained_submission("retained-u1", 1, "U1");
    let u2 = retained_submission("retained-u2", 2, "U2");
    store.accept(&u1).expect("operation should succeed");
    store.mark_paused(&u1.id).expect("operation should succeed");
    store.accept(&u2).expect("operation should succeed");
    store.mark_paused(&u2.id).expect("operation should succeed");

    let order = Arc::new(Mutex::new(Vec::new()));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(RecordingModel {
            order: order.clone(),
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
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let actor_sq_tx = handle.sq_tx.clone();
    let actor_task = tokio::spawn(async move { actor.run().await });

    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_watch_tx, watch_rx) = tokio::sync::watch::channel(handle.sq_tx);
    let (_model_tx, model_rx) = tokio::sync::watch::channel(model_info(128_000));
    let (session_tx, _session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let bridge_task = tokio::spawn(run_conversation_loop(
        ConversationEngine::new("i169-model".into(), "test-provider".into()),
        ConversationLoopIo {
            agent_rx: handle.eq_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills: runtime_skills(),
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("R".into()))
        .expect("operation should succeed");
    wait_for_order(&order, &["U1", "U2", "R"]).await;
    assert_eq!(
        store
            .get(&u1.id)
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::Committed
    );
    assert_eq!(
        store
            .get(&u2.id)
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::Committed
    );

    user_tx
        .send(UserInput::Message("NEXT".into()))
        .expect("operation should succeed");
    wait_for_order(&order, &["U1", "U2", "R", "NEXT"]).await;
    let visible = wait_for_visible_user_order(&mut ui_rx, &["U1", "U2", "R", "NEXT"]).await;
    assert_eq!(visible, vec!["U1", "U2", "R", "NEXT"]);

    user_tx
        .send(UserInput::Exit)
        .expect("operation should succeed");
    bridge_task.await.expect("operation should succeed");
    actor_sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");
}

#[tokio::test]
async fn cancelling_deterministic_prestart_pause_releases_newer_bridge_acceptance() {
    let temp = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp.path().join("sessions"));
    let durable = manager
        .create_or_open_session("i169-retained-cancel-pause")
        .expect("durable session");
    let session_id = durable.id().to_string();
    let store = PendingSubmissionStore::for_session_file(durable.file_path(), &session_id);
    let oversized = "U".repeat(60_000);
    let retained = retained_submission("retained-cancel-u1", 1, &oversized);
    store.accept(&retained).expect("operation should succeed");
    store
        .mark_paused(&retained.id)
        .expect("operation should succeed");

    let order = Arc::new(Mutex::new(Vec::new()));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(RecordingModel {
            order: order.clone(),
        }),
        ToolRegistry::new(),
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp.path().to_path_buf(),
        initial_history: Vec::new(),
        model_context_limit: 8_192,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let actor_sq_tx = handle.sq_tx.clone();
    let actor_task = tokio::spawn(async move { actor.run().await });

    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (_watch_tx, watch_rx) = tokio::sync::watch::channel(handle.sq_tx);
    let (_model_tx, model_rx) = tokio::sync::watch::channel(model_info(8_192));
    let (session_tx, _session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let bridge_task = tokio::spawn(run_conversation_loop(
        ConversationEngine::new("i169-model".into(), "test-provider".into()),
        ConversationLoopIo {
            agent_rx: handle.eq_rx,
            user_rx,
            ui_tx,
            sq_tx_watch: watch_rx,
            model_info_watch: model_rx,
            session_tx,
            runtime_skills: runtime_skills(),
            permission_engine: None,
        },
    ));

    user_tx
        .send(UserInput::Message("R".into()))
        .expect("operation should succeed");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match ui_rx.recv().await {
                Some(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                })) if text
                    .contains("older retained submission retained-cancel-u1 paused before")
                    && text.contains("press cancel") =>
                {
                    break;
                }
                Some(_) => {}
                None => panic!("Bridge closed before retained pre-start pause evidence"),
            }
        }
    })
    .await
    .expect("retained pre-start pause must expose a resolution action");
    assert_eq!(
        store
            .get(&retained.id)
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::PausedPending
    );
    assert!(order.lock().expect("operation should succeed").is_empty());

    user_tx
        .send(UserInput::Cancel)
        .expect("operation should succeed");
    wait_for_order(&order, &["R"]).await;
    assert_eq!(
        store
            .get(&retained.id)
            .expect("operation should succeed")
            .expect("operation should succeed")
            .state,
        PendingSubmissionState::TerminalCancelled
    );

    user_tx
        .send(UserInput::Message("NEXT".into()))
        .expect("operation should succeed");
    wait_for_order(&order, &["R", "NEXT"]).await;
    let visible = wait_for_visible_user_order(&mut ui_rx, &["R", "NEXT"]).await;
    assert_eq!(visible, vec!["R", "NEXT"]);

    user_tx
        .send(UserInput::Exit)
        .expect("operation should succeed");
    bridge_task.await.expect("operation should succeed");
    actor_sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");
}
