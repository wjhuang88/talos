use super::*;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use talos_core::message::{Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderProgress, ProviderResult, ToolDefinition};
use talos_core::session::{RuntimePolicy, SessionEvent, TurnCompletionStatus, TurnEventPayload};
use talos_core::tool::ToolRegistry;
use tokio::sync::mpsc;

type Receiver<T> = mpsc::Receiver<T>;

fn is_turn_started(event: &SessionEvent) -> bool {
    matches!(
        event,
        SessionEvent::TurnEvent {
            payload: TurnEventPayload::Started,
            ..
        }
    )
}

fn progress_event(event: &SessionEvent) -> Option<&AgentEvent> {
    match event {
        SessionEvent::TurnEvent {
            payload: TurnEventPayload::Progress { event },
            ..
        } => Some(event),
        _ => None,
    }
}

fn completed_status(event: &SessionEvent) -> Option<&TurnCompletionStatus> {
    match event {
        SessionEvent::TurnEvent {
            payload: TurnEventPayload::Completed { status },
            ..
        } => Some(status),
        _ => None,
    }
}

struct MockModel {
    responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
}

impl MockModel {
    fn new(responses: Vec<Vec<AgentEvent>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
        }
    }
}

#[async_trait]
impl LanguageModel for MockModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let events = {
            let mut responses = self.responses.lock().expect("operation should succeed");
            responses.pop_front().unwrap_or_default()
        };
        tokio::spawn(async move {
            for event in events {
                let _ = tx.send(event).await;
            }
        });
        Ok(rx)
    }
}

struct SlowModel {
    delay: Duration,
    events: Vec<AgentEvent>,
}

struct CountingModel {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LanguageModel for CountingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }
}

struct BlockingProgressModel {
    progress: ProviderProgress,
}

#[async_trait]
impl LanguageModel for BlockingProgressModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        unreachable!("session agent must use progress-aware provider entrypoint")
    }

    async fn stream_with_tools_and_progress(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        progress_tx
            .send(self.progress.clone())
            .expect("progress receiver available");
        if matches!(self.progress, ProviderProgress::FirstPacketWait { .. }) {
            let (tx, rx) = mpsc::channel(1);
            tokio::spawn(async move {
                std::future::pending::<()>().await;
                drop(tx);
            });
            return Ok(rx);
        }
        std::future::pending::<ProviderResult<Receiver<AgentEvent>>>().await
    }
}

#[async_trait]
impl LanguageModel for SlowModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let events = self.events.clone();
        let delay = self.delay;
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            for event in events {
                let _ = tx.send(event).await;
            }
        });
        Ok(rx)
    }
}

struct PanicModel;

#[async_trait]
impl LanguageModel for PanicModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        panic!("intentional panic for testing");
    }
}

struct PreviewModel;

#[async_trait]
impl LanguageModel for PreviewModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(rx)
    }

    fn request_preview(&self, messages: &[Message]) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "messages": messages }))
    }
}

struct CapturingModel {
    captured: Arc<Mutex<Vec<Vec<Message>>>>,
}

#[async_trait]
impl LanguageModel for CapturingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        self.captured
            .lock()
            .expect("operation should succeed")
            .push(messages.to_vec());
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            for event in success_events("captured") {
                let _ = tx.send(event).await;
            }
        });
        Ok(rx)
    }
}

struct CapturingSequenceModel {
    captured: Arc<Mutex<Vec<Vec<Message>>>>,
    responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
}

#[async_trait]
impl LanguageModel for CapturingSequenceModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        self.captured
            .lock()
            .expect("operation should succeed")
            .push(messages.to_vec());
        let events = self
            .responses
            .lock()
            .expect("operation should succeed")
            .pop_front()
            .unwrap_or_default();
        let (tx, rx) = mpsc::channel(8);
        tokio::spawn(async move {
            for event in events {
                let _ = tx.send(event).await;
            }
        });
        Ok(rx)
    }
}

fn make_agent(model: impl LanguageModel + 'static) -> Agent {
    #[allow(deprecated)]
    Agent::new(Arc::new(model), ToolRegistry::new())
}

fn success_events(text: &str) -> Vec<AgentEvent> {
    vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta { delta: text.into() },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ]
}

async fn collect_events(
    mut eq_rx: tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    timeout: Duration,
) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    loop {
        tokio::select! {
            event = eq_rx.recv() => {
                match event {
                    Some(e) => events.push(e),
                    None => break,
                }
            }
            _ = tokio::time::sleep(timeout) => break,
        }
    }
    events
}

fn structured_submission(
    id: &str,
    item_id: &str,
    sender_generation: u64,
    text: &str,
    source: SubmissionSource,
) -> StructuredSubmission {
    StructuredSubmission {
        id: id.into(),
        source,
        sender_generation,
        items: vec![SubmissionItem {
            id: item_id.into(),
            enqueue_sequence: sender_generation,
            kind: SubmissionKind::UserTurn,
            text: text.into(),
            attachments: Vec::new(),
        }],
    }
}

fn set_authoritative_generation(actor: &mut AppServerSession, generation: u64) {
    let store = actor.pending_store.clone();
    let current = store
        .runtime_generation()
        .expect("operation should succeed");
    for expected in current..generation {
        assert_eq!(
            store
                .advance_runtime_generation(expected)
                .expect("operation should succeed"),
            expected + 1
        );
    }
    actor.set_generation(generation);
}

#[test]
fn structured_submission_rejects_unbounded_image_metadata() {
    let images = (0..=MAX_SUBMISSION_IMAGE_COUNT)
        .map(|index| talos_core::message::ContentPart::Image {
            path: std::path::PathBuf::from(format!("image_{index}.png")),
            mime: "image/png".into(),
            byte_count: 1,
            content_digest: talos_core::message::ContentDigest::default(),
        })
        .collect();
    let submission = StructuredSubmission {
        id: "image_batch".into(),
        source: SubmissionSource::User,
        sender_generation: 1,
        items: vec![SubmissionItem {
            id: "image_item".into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::UserTurn,
            text: "images".into(),
            attachments: images,
        }],
    };

    assert_eq!(
        validate_submission(&submission),
        Err(SubmissionRejectionReason::LimitExceeded)
    );
}

async fn collect_until_completions(
    eq_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    completion_count: usize,
) -> Vec<SessionEvent> {
    let mut events = Vec::new();
    while events
        .iter()
        .filter(|event| completed_status(event).is_some())
        .count()
        < completion_count
    {
        let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
            .await
            .expect("turn completion event timeout")
            .expect("session event channel closed before completion");
        events.push(event);
    }
    events
}

#[tokio::test]
async fn test_submit_and_receive() {
    let agent = make_agent(MockModel::new(vec![success_events("hello")]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "hi".into(),
        })
        .await
        .expect("operation should succeed");

    let events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    assert!(
        events.iter().any(is_turn_started),
        "Should have TurnStarted"
    );
    assert!(
            events.iter().any(|e| matches!(progress_event(e), Some(AgentEvent::TextDelta { delta }) if delta == "hello")),
            "Should have TextDelta with 'hello'"
        );
    assert!(
        events.iter().any(|e| matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Success { .. })
        )),
        "Should have TurnCompleted(Success)"
    );
}

#[tokio::test]
async fn app_server_session_registers_session_owned_process_tool() {
    let agent = make_agent(MockModel::new(vec![]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (_handle, actor) = AppServerSession::new(agent, config);
    let process = actor
        .agent
        .tools
        .get("process")
        .expect("session composition must register process");

    let result = process
        .execute(serde_json::json!({"action": "list", "max_bytes": 1024}))
        .await;
    assert!(!result.is_error);
    let value: serde_json::Value = serde_json::from_str(&result.content).unwrap();
    assert_eq!(value["jobs"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn shutdown_before_start_commit_performs_no_provider_work() {
    let calls = Arc::new(AtomicUsize::new(0));
    let agent = make_agent(CountingModel {
        calls: calls.clone(),
    });
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let control = RuntimeAdmissionControl::new();
    actor.set_runtime_admission(control.clone());
    let reached = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    actor.set_start_commit_gate(reached.clone(), release.clone());
    let actor_task = tokio::spawn(async move { actor.run().await });

    let permit = handle.sq_tx.reserve().await.expect("channel remains open");
    control
        .commit_reserved(
            permit,
            SessionOp::Submit {
                message: "must-not-run".into(),
            },
        )
        .expect("pre-fence submit commits");
    reached.notified().await;
    assert_eq!(
        control.begin_shutdown(31, RuntimeShutdownTurnPolicy::Interrupt),
        RuntimeAdmissionClose::Accepted {
            active_at_fence: false
        }
    );
    release.notify_one();
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown signal sends");
    tokio::time::timeout(Duration::from_secs(1), actor_task)
        .await
        .expect("actor exits within bound")
        .expect("actor joins");

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        control.snapshot().active_turn,
        RuntimeActiveTurnOutcome::Idle
    );
}

#[tokio::test]
async fn set_skill_context_reaches_request_preview() {
    let agent = make_agent(PreviewModel);
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SetSkillContext {
            name: Some("review".into()),
            content: Some("Review instructions from activated skill.".into()),
        })
        .await
        .expect("operation should succeed");
    sq_tx
        .send(SessionOp::PreviewRequest {
            message: "verify skill".into(),
        })
        .await
        .expect("operation should succeed");
    let events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;
    let preview_text = events
        .iter()
        .find_map(|event| match progress_event(event) {
            Some(AgentEvent::TextDelta { delta }) => Some(delta.as_str()),
            _ => None,
        })
        .expect("request preview text");

    assert!(preview_text.contains("# Activated Skill: review"));
    assert!(preview_text.contains("Review instructions from activated skill."));
}

#[tokio::test]
async fn test_multi_turn() {
    let agent = make_agent(MockModel::new(vec![
        success_events("first"),
        success_events("second"),
    ]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "hi".into(),
        })
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Submit {
            message: "again".into(),
        })
        .await
        .expect("operation should succeed");

    let events = collect_until_completions(&mut eq_rx, 2).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let turn_started_count = events.iter().filter(|e| is_turn_started(e)).count();
    assert_eq!(turn_started_count, 2, "Should have 2 TurnStarted events");

    let success_count = events
        .iter()
        .filter(|e| {
            matches!(
                completed_status(e),
                Some(TurnCompletionStatus::Success { .. })
            )
        })
        .count();
    assert!(
        success_count >= 1,
        "Should have at least 1 TurnCompleted(Success)"
    );
}

#[tokio::test]
async fn structured_batch_preserves_distinct_user_messages_and_correlation() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let agent = make_agent(CapturingModel {
        captured: captured.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 7);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: StructuredSubmission {
                id: "batch_a".into(),
                source: SubmissionSource::User,
                sender_generation: 7,
                items: vec![
                    SubmissionItem {
                        id: "item_a".into(),
                        enqueue_sequence: 1,
                        kind: SubmissionKind::UserTurn,
                        text: "first".into(),
                        attachments: Vec::new(),
                    },
                    SubmissionItem {
                        id: "item_b".into(),
                        enqueue_sequence: 2,
                        kind: SubmissionKind::UserTurn,
                        text: "second".into(),
                        attachments: Vec::new(),
                    },
                ],
            },
        })
        .await
        .expect("operation should succeed");
    let mut events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");
    while let Ok(event) = eq_rx.try_recv() {
        events.push(event);
    }

    let requests = captured.lock().expect("operation should succeed");
    let users = requests[0]
        .iter()
        .filter_map(|message| match message {
            Message::User { content } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(users, vec!["first", "second"]);
    drop(requests);

    assert!(events.iter().any(|event| matches!(
        event,
        SessionEvent::SubmissionQueued { submission_id, .. } if submission_id == "batch_a"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        SessionEvent::SubmissionStarted { submission_id, .. } if submission_id == "batch_a"
    )));
}

#[tokio::test]
async fn duplicate_submission_reconciles_and_executes_at_most_once() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let agent = make_agent(CapturingModel {
        captured: captured.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 11);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });
    let submission = structured_submission(
        "duplicate_batch",
        "duplicate_item",
        11,
        "once",
        SubmissionSource::User,
    );

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission.clone(),
        })
        .await
        .expect("operation should succeed");
    sq_tx
        .send(SessionOp::SubmitStructured { submission })
        .await
        .expect("operation should succeed");

    let mut events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");
    while let Ok(event) = eq_rx.try_recv() {
        events.push(event);
    }

    assert_eq!(captured.lock().expect("operation should succeed").len(), 1);
    assert!(events.iter().any(|event| matches!(
        event,
        SessionEvent::SubmissionReceipt {
            submission_id,
            disposition: SubmissionReceiptDisposition::AlreadyAccepted { .. },
            ..
        } if submission_id == "duplicate_batch"
    )));
    assert!(!events.iter().any(|event| matches!(
        event,
        SessionEvent::SubmissionRejected { submission_id, .. }
            if submission_id == "duplicate_batch"
    )));
}

#[tokio::test]
async fn closed_eq_does_not_revoke_actor_custody_or_duplicate_execution() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let agent = make_agent(CapturingModel {
        captured: captured.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 12);
    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);
    let actor_task = tokio::spawn(async move { actor.run().await });
    let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();

    sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: structured_submission(
                "lost_ack_batch",
                "lost_ack_item",
                12,
                "must run once",
                SubmissionSource::User,
            ),
            receipt_tx: Some(receipt_tx),
        })
        .await
        .expect("operation should succeed");

    let receipt = tokio::time::timeout(Duration::from_secs(2), receipt_rx.recv())
        .await
        .expect("tracked durable receipt timeout")
        .expect("tracked durable receipt channel");
    assert!(receipt.disposition.has_durable_custody());

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if captured.lock().expect("operation should succeed").len() == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Actor should execute accepted work without an EQ observer");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    let requests = captured.lock().expect("operation should succeed");
    assert_eq!(requests.len(), 1, "Actor custody must execute exactly once");
    assert!(
        requests[0].iter().any(
            |message| matches!(message, Message::User { content } if content == "must run once")
        )
    );
}

#[tokio::test]
async fn context_budget_pauses_before_submission_started() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let agent = make_agent(CapturingModel {
        captured: captured.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 64,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 13);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "over_budget_batch",
                "over_budget_item",
                13,
                "request",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("operation should succeed");

    let mut events = Vec::new();
    loop {
        let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
            .await
            .expect("budget pause timeout")
            .expect("session event channel");
        let paused = matches!(
            event,
            SessionEvent::SubmissionPaused {
                reason: SubmissionRejectionReason::ContextBudgetExceeded,
                ..
            }
        );
        events.push(event);
        if paused {
            break;
        }
    }
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::SubmissionStarted { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::SubmissionRejected { .. }))
    );
    assert!(
        captured
            .lock()
            .expect("operation should succeed")
            .is_empty()
    );
}

#[tokio::test]
async fn aggregate_queue_limit_counts_running_and_pending_submissions() {
    let agent = make_agent(SlowModel {
        delay: Duration::from_secs(30),
        events: success_events("too late"),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 21);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    for batch in 0..=4_u64 {
        let submission = StructuredSubmission {
            id: format!("bounded_batch_{batch}"),
            source: SubmissionSource::User,
            sender_generation: 21,
            items: (0..MAX_SUBMISSION_BATCH_ITEMS)
                .map(|item| SubmissionItem {
                    id: format!("bounded_item_{batch}_{item}"),
                    enqueue_sequence: batch * MAX_SUBMISSION_BATCH_ITEMS as u64 + item as u64,
                    kind: SubmissionKind::UserTurn,
                    text: "x".into(),
                    attachments: Vec::new(),
                })
                .collect(),
        };
        sq_tx
            .send(SessionOp::SubmitStructured { submission })
            .await
            .expect("operation should succeed");
    }

    let rejected = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(SessionEvent::SubmissionRejected {
                submission_id,
                reason: SubmissionRejectionReason::LimitExceeded,
                ..
            }) = eq_rx.recv().await
            {
                break submission_id;
            }
        }
    })
    .await
    .expect("aggregate limit rejection");
    assert_eq!(rejected, "bounded_batch_4");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");
    let mut closed = Vec::new();
    while let Ok(event) = eq_rx.try_recv() {
        if let SessionEvent::SubmissionRejected {
            submission_id,
            reason: SubmissionRejectionReason::SessionClosed,
            ..
        } = event
        {
            closed.push(submission_id);
        }
    }
    assert_eq!(closed.len(), 3, "shutdown must reject every pending batch");
}

#[tokio::test]
async fn paused_user_submission_runs_before_retained_scheduler_work() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let responses = Arc::new(Mutex::new(VecDeque::from(vec![
        vec![AgentEvent::Error {
            message: "pause".into(),
        }],
        success_events("user resumed"),
        success_events("scheduler resumed"),
    ])));
    let agent = make_agent(CapturingSequenceModel {
        captured: captured.clone(),
        responses,
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    set_authoritative_generation(&mut actor, 31);
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "failing_batch",
                "failing_item",
                31,
                "fail first",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("operation should succeed");
    let _ = collect_until_completions(&mut eq_rx, 1).await;

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "scheduler_batch",
                "scheduler_item",
                31,
                "scheduler retained",
                SubmissionSource::Scheduler,
            ),
        })
        .await
        .expect("operation should succeed");
    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "user_batch",
                "user_item",
                31,
                "user resumes",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("operation should succeed");
    let _ = collect_until_completions(&mut eq_rx, 2).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    let calls = captured.lock().expect("operation should succeed");
    let last_users = calls
        .iter()
        .map(|messages| {
            messages
                .iter()
                .rev()
                .find_map(|message| match message {
                    Message::User { content } => Some(content.as_str()),
                    _ => None,
                })
                .expect("provider call has user message")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        last_users,
        vec!["fail first", "user resumes", "scheduler retained"]
    );
}

#[tokio::test]
async fn test_interrupt() {
    let slow_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta {
            delta: "slow response".into(),
        },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ];
    let agent = make_agent(SlowModel {
        delay: Duration::from_millis(500),
        events: slow_events,
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "hi".into(),
        })
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Interrupt)
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let events = collect_events(eq_rx, Duration::from_secs(3)).await;

    assert!(
        events.iter().any(is_turn_started),
        "Should have TurnStarted"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(completed_status(e), Some(TurnCompletionStatus::Cancelled))),
        "Should have TurnCompleted(Cancelled)"
    );
}

#[tokio::test]
async fn provider_wait_boundaries_cancel_through_existing_session_lifecycle() {
    let stages = [
        ProviderProgress::InitialDispatch {
            attempt: 0,
            max_attempts: 3,
        },
        ProviderProgress::ScheduledBackoff {
            attempt: 1,
            max_attempts: 3,
            delay_ms: 500,
        },
        ProviderProgress::FirstPacketWait {
            attempt: 1,
            max_attempts: 3,
        },
    ];

    for expected_progress in stages {
        let agent = make_agent(BlockingProgressModel {
            progress: expected_progress.clone(),
        });
        let config = SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);
        let mut eq_rx = handle.eq_rx;
        let sq_tx = handle.sq_tx;
        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "wait".into(),
            })
            .await
            .expect("submit");

        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let event = eq_rx.recv().await.expect("session event");
                if matches!(
                    progress_event(&event),
                    Some(AgentEvent::ProviderProgress { progress }) if progress == &expected_progress
                ) {
                    break;
                }
            }
        })
        .await
        .expect("typed provider progress should arrive");

        sq_tx.send(SessionOp::Interrupt).await.expect("interrupt");
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let event = eq_rx.recv().await.expect("terminal event");
                if matches!(
                    completed_status(&event),
                    Some(TurnCompletionStatus::Cancelled)
                ) {
                    break;
                }
            }
        })
        .await
        .expect("provider wait should cancel promptly");

        sq_tx.send(SessionOp::Shutdown).await.expect("shutdown");
        actor_task.await.expect("actor");
    }
}

#[tokio::test]
async fn test_shutdown() {
    let agent = make_agent(MockModel::new(vec![]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");

    let result = tokio::time::timeout(Duration::from_secs(2), actor_task).await;
    assert!(result.is_ok(), "Actor should exit cleanly on Shutdown");
}

#[tokio::test]
async fn test_eq_consumer_disconnect() {
    let agent = make_agent(MockModel::new(vec![success_events("hello")]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let sq_tx = handle.sq_tx;
    drop(handle.eq_rx);

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "hi".into(),
        })
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");

    let result = tokio::time::timeout(Duration::from_secs(2), actor_task).await;
    assert!(
        result.is_ok(),
        "Actor should handle EQ disconnect gracefully"
    );
}

#[tokio::test]
async fn test_sq_backpressure() {
    let agent = make_agent(MockModel::new(vec![success_events("hello")]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, _actor) = AppServerSession::new(agent, config);

    let sq_tx = handle.sq_tx;

    for _ in 0..512 {
        sq_tx
            .try_send(SessionOp::Submit {
                message: "fill".into(),
            })
            .expect("operation should succeed");
    }

    let result = sq_tx.try_send(SessionOp::Submit {
        message: "overflow".into(),
    });
    assert!(
        result.is_err(),
        "try_send should fail when SQ is at capacity"
    );
    assert!(
        matches!(
            result.expect_err("operation should fail"),
            tokio::sync::mpsc::error::TrySendError::Full(_)
        ),
        "Error should be Full, not Closed"
    );
}

#[tokio::test]
async fn test_panic_recovery() {
    let agent = make_agent(PanicModel);
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "panic me".into(),
        })
        .await
        .expect("operation should succeed");

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "still here?".into(),
        })
        .await
        .expect("operation should succeed");

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let events = collect_events(eq_rx, Duration::from_secs(3)).await;

    let turn_started_count = events.iter().filter(|e| is_turn_started(e)).count();
    assert_eq!(turn_started_count, 2, "Should have 2 TurnStarted events");

    let error_count = events
        .iter()
        .filter(|e| {
            matches!(
                completed_status(e),
                Some(TurnCompletionStatus::Error { .. })
            )
        })
        .count();
    assert_eq!(error_count, 2, "Should have 2 TurnCompleted(Error) events");
}

#[tokio::test]
async fn test_concurrent_submit_and_interrupt() {
    let slow_events = vec![
        AgentEvent::TurnStart,
        AgentEvent::TextDelta {
            delta: "slow".into(),
        },
        AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: talos_core::message::Usage::default(),
        },
    ];
    let agent = make_agent(SlowModel {
        delay: Duration::from_millis(500),
        events: slow_events,
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "slow turn".into(),
        })
        .await
        .expect("operation should succeed");

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx
        .send(SessionOp::Interrupt)
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Submit {
            message: "after interrupt".into(),
        })
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let events = collect_events(eq_rx, Duration::from_secs(3)).await;

    assert!(
        events.iter().any(is_turn_started),
        "Should have TurnStarted"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(completed_status(e), Some(TurnCompletionStatus::Cancelled))),
        "First turn should be Cancelled"
    );
}

#[tokio::test]
async fn test_multi_turn_with_history() {
    use talos_core::message::Message;

    let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
    let responses = Arc::new(Mutex::new(VecDeque::from(vec![
        success_events("first response"),
        success_events("second response"),
        success_events("third response"),
    ])));
    let _captured = captured_messages.clone();

    struct CapturingModel {
        responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
        captured: Arc<Mutex<Vec<Vec<Message>>>>,
    }

    #[async_trait]
    impl LanguageModel for CapturingModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            self.captured
                .lock()
                .expect("operation should succeed")
                .push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().expect("operation should succeed");
                responses.pop_front().unwrap_or_default()
            };
            tokio::spawn(async move {
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    let agent = make_agent(CapturingModel {
        responses,
        captured: captured_messages.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;

    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 1".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 2".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 3".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let events = collect_until_completions(&mut eq_rx, 3).await;
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;
    let success_count = events
        .iter()
        .filter(|e| {
            matches!(
                completed_status(e),
                Some(TurnCompletionStatus::Success { .. })
            )
        })
        .count();
    assert!(success_count >= 1, "Should have at least 1 Success");

    let captured = captured_messages.lock().expect("operation should succeed");
    assert!(captured.len() >= 3, "Should have captured at least 3 calls");

    let third_call_messages = &captured[2];
    let user_messages: Vec<_> = third_call_messages
        .iter()
        .filter(|m| matches!(m, Message::User { .. }))
        .collect();
    assert!(
        user_messages.len() >= 3,
        "Third turn should have at least 3 user messages (turns 1, 2, 3), got {}",
        user_messages.len()
    );
}

#[tokio::test]
async fn test_interrupt_after_success_preserves_history() {
    use talos_core::message::Message;

    let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
    let responses = Arc::new(Mutex::new(VecDeque::from(vec![
        success_events("first response"),
        success_events("second response"),
    ])));

    struct CapturingModel {
        responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
        captured: Arc<Mutex<Vec<Vec<Message>>>>,
    }

    #[async_trait]
    impl LanguageModel for CapturingModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            self.captured
                .lock()
                .expect("operation should succeed")
                .push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().expect("operation should succeed");
                responses.pop_front().unwrap_or_default()
            };
            tokio::spawn(async move {
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    let agent = make_agent(CapturingModel {
        responses,
        captured: captured_messages.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 1".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(event) = eq_rx.recv().await {
            if matches!(
                completed_status(&event),
                Some(TurnCompletionStatus::Success { .. })
            ) {
                break;
            }
        }
    })
    .await
    .expect("first turn should complete before timeout");

    sq_tx
        .send(SessionOp::Interrupt)
        .await
        .expect("operation should succeed");

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 2".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::timeout(Duration::from_secs(1), async {
        while let Some(event) = eq_rx.recv().await {
            if matches!(
                completed_status(&event),
                Some(TurnCompletionStatus::Success { .. })
            ) {
                break;
            }
        }
    })
    .await
    .expect("second turn should complete before timeout");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let captured = captured_messages.lock().expect("operation should succeed");
    assert!(captured.len() >= 2, "Should have captured 2 calls");

    let second_call_messages = &captured[1];
    assert!(
        second_call_messages
            .iter()
            .any(|m| matches!(m, Message::User { content } if content == "turn 1")),
        "Second turn should retain first user message after interrupt"
    );
    assert!(
        second_call_messages.iter().any(
            |m| matches!(m, Message::Assistant { content, .. } if content == "first response")
        ),
        "Second turn should retain first assistant response after interrupt"
    );
}

#[tokio::test]
async fn test_initial_history_from_jsonl_resume() {
    use talos_core::message::Message;
    use talos_session::SessionManager;

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("resume-test", "")
        .expect("operation should succeed");
    let session_id = session.id.to_string();
    session
        .append(&Message::User {
            content: "prior question".into(),
        })
        .expect("operation should succeed");
    session
        .append(&Message::Assistant {
            content: "prior answer".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .expect("operation should succeed");
    let resumed = manager
        .resume_session(&session_id)
        .expect("operation should succeed");
    let prior_history = resumed.read_messages().expect("operation should succeed");

    let captured_messages = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));
    let responses = Arc::new(Mutex::new(VecDeque::from(vec![success_events(
        "new response",
    )])));

    struct CapturingModel {
        responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
        captured: Arc<Mutex<Vec<Vec<Message>>>>,
    }

    #[async_trait]
    impl LanguageModel for CapturingModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            self.captured
                .lock()
                .expect("operation should succeed")
                .push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().expect("operation should succeed");
                responses.pop_front().unwrap_or_default()
            };
            tokio::spawn(async move {
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    let agent = make_agent(CapturingModel {
        responses,
        captured: captured_messages.clone(),
    });
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: prior_history,
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let sq_tx = handle.sq_tx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "new question".into(),
        })
        .await
        .expect("operation should succeed");
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    let _ = actor_task.await;

    let captured = captured_messages.lock().expect("operation should succeed");
    assert_eq!(captured.len(), 1, "Should have captured exactly 1 call");

    let messages = &captured[0];
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::User { content } if content == "prior question")),
        "Resumed session should include prior user message"
    );
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::Assistant { content, .. } if content == "prior answer")),
        "Resumed session should include prior assistant response"
    );
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, Message::User { content } if content.contains("new question"))),
        "Resumed session should include new user message"
    );
}

#[tokio::test]
async fn canonical_turn_events_are_contiguous_and_actor_persistence_replays_messages() {
    use talos_session::{SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("single-flow", "")
        .expect("operation should succeed");
    let agent = make_agent(MockModel::new(vec![success_events("persisted answer")]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_persistence(
        session.clone(),
        SessionMetadata {
            provider: Some("mock".into()),
            model: Some("mock-model".into()),
            ..SessionMetadata::default()
        },
    );
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "persist this question".into(),
        })
        .await
        .expect("operation should succeed");

    let mut sequences = Vec::new();
    let mut session_ids = Vec::new();
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(event) = eq_rx.recv().await {
            if let SessionEvent::TurnEvent {
                session_id,
                sequence,
                payload,
                ..
            } = event
            {
                session_ids.push(session_id);
                sequences.push(sequence);
                if matches!(payload, TurnEventPayload::Completed { .. }) {
                    break;
                }
            }
        }
    })
    .await
    .expect("canonical turn completion");

    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    assert_eq!(sequences, (0..sequences.len() as u64).collect::<Vec<_>>());
    assert!(
        session_ids
            .iter()
            .all(|event_session_id| event_session_id == &session.id.to_string()),
        "every canonical event must carry the durable session identity"
    );
    assert_eq!(
        session.read_messages().expect("operation should succeed"),
        vec![
            Message::User {
                content: "persist this question".into(),
            },
            Message::Assistant {
                content: "persisted answer".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ]
    );
    assert!(
        session
            .read_events()
            .expect("operation should succeed")
            .is_empty(),
        "canonical persistence must not duplicate transient AgentEvents"
    );
}

struct EchoTool;

struct BoundaryBlockingTool {
    entered: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
}

#[tokio::test]
async fn boundary_injection_ambiguous_running_recovery_fences_new_execution() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let manager = talos_session::SessionManager::with_dir(temp.path().to_path_buf());
    let session = manager
        .create_session("ambiguous-boundary", "")
        .expect("session");
    let calls = Arc::new(AtomicUsize::new(0));
    let (handle, mut actor) = AppServerSession::new(
        make_agent(CountingModel {
            calls: calls.clone(),
        }),
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: temp.path().to_path_buf(),
            initial_history: vec![],
            model_context_limit: 128_000,
        },
    );
    actor.set_persistence(session, talos_session::SessionMetadata::default());
    set_authoritative_generation(&mut actor, 1);
    let store = actor.pending_store.clone();
    for id in ["original", "injected"] {
        store
            .accept(&structured_submission(
                id,
                &format!("{id}-item"),
                1,
                "ambiguous",
                SubmissionSource::User,
            ))
            .expect("accept");
        store
            .mark_running(id, "interrupted-before-transcript")
            .expect("running");
    }
    handle
        .sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "new",
                "new-item",
                1,
                "must not start",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("queued before recovery");
    let task = tokio::spawn(async move { actor.run().await });
    let (receipt_tx, mut receipt_rx) = tokio::sync::mpsc::unbounded_channel();
    handle
        .sq_tx
        .send(SessionOp::SubmitStructuredTracked {
            submission: structured_submission(
                "tracked-new",
                "tracked-new-item",
                1,
                "must reject",
                SubmissionSource::User,
            ),
            receipt_tx: Some(receipt_tx),
        })
        .await
        .expect("tracked submit");
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("recovery terminates")
        .expect("actor joins");
    let receipt = receipt_rx.try_recv().expect("explicit tracked rejection");
    assert!(matches!(
        receipt.disposition,
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::SessionClosed
        }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(store.get("new").expect("lookup").is_none());
    assert_eq!(store.recover_running().expect("frozen identities").len(), 2);
}

#[tokio::test]
async fn boundary_handoff_without_ack_publishes_correlated_terminal_resolution() {
    use talos_core::session::PendingSubmissionState;
    let temp = tempfile::tempdir().expect("temporary directory");
    let (handle, actor) = AppServerSession::new(
        make_agent(CountingModel {
            calls: Arc::new(AtomicUsize::new(0)),
        }),
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: temp.path().to_path_buf(),
            initial_history: vec![],
            model_context_limit: 128_000,
        },
    );
    let submission = structured_submission(
        "unack",
        "unack-item",
        0,
        "not delivered",
        SubmissionSource::User,
    );
    let (receipt, _) = actor.pending_store.accept(&submission).expect("accept");
    actor
        .pending_store
        .mark_running(&submission.id, "outer")
        .expect("running");
    let active = ActiveStructuredTurn {
        submission_id: submission.id.clone(),
        receipt_id: receipt.clone(),
        session_generation: 0,
        source: SubmissionSource::User,
        turn_id: "outer".into(),
    };
    assert!(actor.finish_unacknowledged_boundary(&active));
    let record = actor
        .pending_store
        .get(&submission.id)
        .expect("lookup")
        .expect("record");
    assert_eq!(record.state, PendingSubmissionState::TerminalError);
    let mut events = handle.eq_rx;
    assert!(
        matches!(events.try_recv(), Ok(SessionEvent::SubmissionResolved {
        submission_id, receipt_id, state: PendingSubmissionState::TerminalError, ..
    }) if submission_id == submission.id && receipt_id == receipt)
    );
    assert!(
        events.try_recv().is_err(),
        "no fabricated injection or outer Turn event"
    );
}

#[async_trait]
impl talos_core::tool::AgentTool for BoundaryBlockingTool {
    fn name(&self) -> &str {
        "boundary_probe"
    }
    fn description(&self) -> &str {
        "Wait for a deterministic test boundary"
    }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _: serde_json::Value) -> talos_core::tool::ToolResult {
        self.entered.notify_one();
        self.release.notified().await;
        talos_core::tool::ToolResult::success("boundary complete")
    }
}

#[tokio::test]
async fn boundary_injection_reaches_next_provider_request_in_same_turn() {
    exercise_boundary_injection(false, false, false).await;
}

#[tokio::test]
async fn boundary_injection_cancel_preserves_injected_input_once() {
    exercise_boundary_injection(true, false, false).await;
}

#[tokio::test]
async fn boundary_injection_provider_error_preserves_both_fifo_inputs() {
    exercise_boundary_injection(false, true, false).await;
}

#[tokio::test]
async fn boundary_injection_targeted_cancel_resumes_uninjected_user_once() {
    exercise_boundary_injection(false, false, true).await;
}

async fn exercise_boundary_injection(cancel: bool, provider_error: bool, cancel_before: bool) {
    exercise_boundary_injection_with_gate(cancel, provider_error, cancel_before, false).await;
}

#[tokio::test]
async fn boundary_cancel_after_agent_ack_before_actor_projection_preserves_once() {
    exercise_boundary_injection_with_gate(true, false, false, true).await;
}

async fn exercise_boundary_injection_with_gate(
    cancel: bool,
    provider_error: bool,
    cancel_before: bool,
    handoff_cancel: bool,
) {
    use talos_core::session::PendingSubmissionState;
    use talos_session::{SessionManager, SessionMetadata};

    let temp = tempfile::tempdir().expect("temp directory");
    let manager = SessionManager::with_dir(temp.path().to_path_buf());
    let session = manager.create_session("boundary", "").expect("session");
    let ack_reached = Arc::new(tokio::sync::Notify::new());
    let captured = Arc::new(Mutex::new(Vec::new()));
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BoundaryBlockingTool {
        entered: entered.clone(),
        release: release.clone(),
    }));
    let tool_response = |id: &str| {
        vec![
            AgentEvent::ToolCall {
                call: talos_core::message::ToolCall {
                    id: id.into(),
                    name: "boundary_probe".into(),
                    input: serde_json::json!({}),
                },
                provenance: talos_core::tool::ToolProvenance::Native,
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Default::default(),
            },
        ]
    };
    let responses = vec![
        tool_response("probe-1"),
        if cancel_before {
            success_events("queued turn resumed")
        } else {
            tool_response("probe-2")
        },
        if provider_error {
            vec![AgentEvent::Error {
                message: "boundary fixture failure".into(),
            }]
        } else {
            success_events("steering received")
        },
    ];
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(CapturingSequenceModel {
            captured: captured.clone(),
            responses: Arc::new(Mutex::new(responses.into())),
        }),
        registry,
    );
    let (handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: temp.path().to_path_buf(),
            initial_history: vec![],
            model_context_limit: 128_000,
        },
    );
    actor.set_persistence(session.clone(), SessionMetadata::default());
    if handoff_cancel {
        actor.boundary_ack_cancel_gate = Some(ack_reached.clone());
    }
    set_authoritative_generation(&mut actor, 1);
    let store = actor.pending_store.clone();
    let task = tokio::spawn(async move { actor.run().await });
    let mut events = Vec::new();
    let mut eq_rx = handle.eq_rx;
    handle
        .sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "initial",
                "initial-item",
                1,
                "original task",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("initial submit");
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .expect("tool entered");
    handle
        .sq_tx
        .send(SessionOp::SubmitStructured {
            submission: structured_submission(
                "steering",
                "steering-item",
                1,
                "change the next step",
                SubmissionSource::User,
            ),
        })
        .await
        .expect("steering submit");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = eq_rx.recv().await.expect("event");
            let accepted = matches!(&event, SessionEvent::SubmissionReceipt { submission_id, disposition: SubmissionReceiptDisposition::AcceptedPending, .. } if submission_id == "steering");
            events.push(event);
            if accepted { break; }
        }
    }).await.expect("durable receipt");
    if cancel_before {
        let turn_id = events
            .iter()
            .find_map(|event| match event {
                SessionEvent::StructuredSubmissionStarted { turn_id, .. } => Some(turn_id.clone()),
                _ => None,
            })
            .expect("original turn identity");
        handle
            .sq_tx
            .send(SessionOp::InterruptTurn {
                session_generation: 1,
                turn_id,
            })
            .await
            .expect("targeted Esc");
        events.extend(collect_until_completions(&mut eq_rx, 2).await);
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("actor stopped")
            .expect("actor joined");
        while let Ok(event) = eq_rx.try_recv() {
            events.push(event);
        }
        let original = store.get("initial").expect("lookup").expect("original");
        let steering = store.get("steering").expect("lookup").expect("steering");
        assert_eq!(original.state, PendingSubmissionState::TerminalCancelled);
        assert_eq!(steering.state, PendingSubmissionState::Committed);
        assert_ne!(original.turn_id, steering.turn_id);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, SessionEvent::StructuredSubmissionStarted { .. }))
                .count(),
            2
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, SessionEvent::StructuredSubmissionInjected { .. }))
        );
        let requests = captured.lock().expect("requests");
        assert_eq!(requests.len(), 2);
        assert!(requests[1].iter().any(|message| matches!(message, Message::User { content } if content == "change the next step")));
        return;
    }
    release.notify_one();
    if handoff_cancel {
        tokio::time::timeout(Duration::from_secs(5), ack_reached.notified())
            .await
            .expect("Agent ack held before Actor projection");
    } else {
        tokio::time::timeout(Duration::from_secs(5), entered.notified())
            .await
            .expect("second tool entered");
    }
    if cancel {
        // Both the original and the injected item must still consume quota.
        // Fill the rest while the second tool is held at an explicit gate.
        let mut remaining = MAX_STEERING_QUEUE_ITEMS - 2;
        let mut batch = 0;
        while remaining > 0 {
            let count = remaining.min(MAX_SUBMISSION_BATCH_ITEMS);
            handle
                .sq_tx
                .send(SessionOp::SubmitStructured {
                    submission: StructuredSubmission {
                        id: format!("quota-{batch}"),
                        source: SubmissionSource::Scheduler,
                        sender_generation: 1,
                        items: (0..count)
                            .map(|item| SubmissionItem {
                                id: format!("quota-{batch}-{item}"),
                                enqueue_sequence: item as u64,
                                kind: SubmissionKind::UserTurn,
                                text: "queued".into(),
                                attachments: vec![],
                            })
                            .collect(),
                    },
                })
                .await
                .expect("fill remaining quota");
            remaining -= count;
            batch += 1;
        }
        handle
            .sq_tx
            .send(SessionOp::SubmitStructured {
                submission: structured_submission(
                    "overflow",
                    "overflow-item",
                    1,
                    "must reject",
                    SubmissionSource::User,
                ),
            })
            .await
            .expect("overflow probe");
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let event = eq_rx.recv().await.expect("quota event");
                let rejected = matches!(&event, SessionEvent::SubmissionRejected {
                    submission_id, reason: SubmissionRejectionReason::LimitExceeded, ..
                } if submission_id == "overflow");
                assert!(
                    !matches!(&event, SessionEvent::SubmissionReceipt {
                    submission_id, disposition: SubmissionReceiptDisposition::AcceptedPending, ..
                } if submission_id == "overflow"),
                    "injected Running item lost its quota"
                );
                events.push(event);
                if rejected {
                    break;
                }
            }
        })
        .await
        .expect("injected quota rejection");
        let interrupt = if handoff_cancel {
            SessionOp::InterruptTurn {
                session_generation: 1,
                turn_id: store
                    .get("initial")
                    .expect("lookup")
                    .expect("record")
                    .turn_id
                    .expect("outer Turn"),
            }
        } else {
            SessionOp::Interrupt
        };
        handle.sq_tx.send(interrupt).await.expect("interrupt");
    } else {
        handle
            .sq_tx
            .send(SessionOp::SubmitStructured {
                submission: structured_submission(
                    "later",
                    "later-item",
                    1,
                    "second steering",
                    SubmissionSource::User,
                ),
            })
            .await
            .expect("second steering submit");
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let event = eq_rx.recv().await.expect("event");
                let accepted = matches!(&event, SessionEvent::SubmissionReceipt { submission_id, disposition: SubmissionReceiptDisposition::AcceptedPending, .. } if submission_id == "later");
                events.push(event);
                if accepted { break; }
            }
        }).await.expect("second receipt");
        release.notify_one();
    }
    events.extend(collect_until_completions(&mut eq_rx, 1).await);
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .expect("actor stopped")
        .expect("actor joined");
    while let Ok(event) = eq_rx.try_recv() {
        events.push(event);
    }

    let requests = captured.lock().expect("captured requests");
    let injected_position = events.iter().position(|event| matches!(event,
        SessionEvent::StructuredSubmissionInjected { submission, .. } if submission.id == "steering"
    )).expect("first injection event");
    let tool_result_position = events
        .iter()
        .position(|event| {
            matches!(progress_event(event),
                Some(AgentEvent::ToolResult { result }) if result.tool_use_id == "probe-1"
            )
        })
        .expect("first tool result event");
    assert!(
        tool_result_position < injected_position,
        "injection must follow the preceding tool result in UI order"
    );
    if let Some(next_call_position) = events.iter().position(|event| {
        matches!(progress_event(event),
            Some(AgentEvent::ToolCall { call, .. }) if call.id == "probe-2"
        )
    }) {
        assert!(
            injected_position < next_call_position,
            "injection must precede the next response in UI order"
        );
    }
    assert_eq!(
        requests.len(),
        if handoff_cancel {
            1
        } else if cancel {
            2
        } else {
            3
        },
        "steering must not start a second outer Turn"
    );
    if !handoff_cancel {
        let next = &requests[1];
        let tool_index = next.iter().position(|message| matches!(message, Message::Tool { result } if result.tool_use_id == "probe-1")).expect("tool result");
        assert!(
            matches!(&next[tool_index + 1], Message::User { content } if content == "change the next step")
        );
    }
    assert!(
        !events.iter().any(|event| matches!(event,
            SessionEvent::SubmissionResolved { submission_id, .. } if submission_id == "steering"
        )),
        "acknowledged input must not also resolve as an unacknowledged failure"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, SessionEvent::StructuredSubmissionInjected { .. }))
            .count(),
        if cancel { 1 } else { 2 }
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, SessionEvent::StructuredSubmissionStarted { .. }))
            .count(),
        1
    );
    let first = store
        .get("initial")
        .expect("lookup")
        .expect("initial record");
    let second = store
        .get("steering")
        .expect("lookup")
        .expect("steering record");
    let expected = if cancel {
        PendingSubmissionState::TerminalCancelled
    } else if provider_error {
        PendingSubmissionState::TerminalError
    } else {
        PendingSubmissionState::Committed
    };
    assert_eq!(first.state, expected);
    assert_eq!(second.state, expected);
    assert_eq!(first.turn_id, second.turn_id);
    if !cancel {
        let third = store.get("later").expect("lookup").expect("later record");
        assert_eq!(third.state, expected);
        assert_eq!(third.turn_id, first.turn_id);
        let users: Vec<_> = requests[2]
            .iter()
            .filter_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        assert!(users.ends_with(&["change the next step", "second steering"]));
    }
    assert_eq!(session.read_messages().expect("transcript").iter().filter(|message| matches!(message, Message::User { content } if content == "change the next step")).count(), 1);
    drop(requests);

    // Reconstruct the real Session actor and retry the exact accepted identity.
    // Its terminal receipt must survive reconstruction without another model call.
    let replay_calls = Arc::new(AtomicUsize::new(0));
    let (reopened, mut recovered_actor) = AppServerSession::new(
        make_agent(CountingModel {
            calls: replay_calls.clone(),
        }),
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: temp.path().to_path_buf(),
            initial_history: session.read_messages().expect("recovered transcript"),
            model_context_limit: 128_000,
        },
    );
    recovered_actor.set_persistence(session, SessionMetadata::default());
    set_authoritative_generation(&mut recovered_actor, 1);
    let recovered_task = tokio::spawn(async move { recovered_actor.run().await });
    reopened
        .sq_tx
        .send(SessionOp::SubmitStructured {
            submission: second.submission,
        })
        .await
        .expect("retry accepted identity");
    let mut recovered_events = reopened.eq_rx;
    let recovered_state = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(SessionEvent::SubmissionReceipt {
                submission_id,
                disposition: SubmissionReceiptDisposition::AlreadyAccepted { state, .. },
                ..
            }) = recovered_events.recv().await
            {
                if submission_id == "steering" {
                    break state;
                }
            }
        }
    })
    .await
    .expect("recovered terminal receipt");
    assert_eq!(recovered_state, expected);
    reopened
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("recovered shutdown");
    tokio::time::timeout(Duration::from_secs(5), recovered_task)
        .await
        .expect("recovered actor stopped")
        .expect("recovered actor joined");
    assert_eq!(replay_calls.load(Ordering::SeqCst), 0);
}

#[async_trait::async_trait]
impl talos_core::tool::AgentTool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes input back"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        })
    }
    async fn execute(&self, input: serde_json::Value) -> talos_core::tool::ToolResult {
        let msg = input
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("empty");
        talos_core::tool::ToolResult::success(format!("echo: {msg}"))
    }
}

struct ToolCallThenErrorModel {
    call_count: Arc<std::sync::atomic::AtomicU8>,
    trailing_fragment: bool,
}

struct ToolCallThenBlockingModel {
    call_count: Arc<std::sync::atomic::AtomicU8>,
}

#[async_trait]
impl LanguageModel for ToolCallThenBlockingModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let count = self.call_count.clone();
        tokio::spawn(async move {
            if count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                let _ = tx
                    .send(AgentEvent::ToolCall {
                        call: talos_core::message::ToolCall {
                            id: "call_echo_cancelled".into(),
                            name: "echo".into(),
                            input: serde_json::json!({"message": "cancelled"}),
                        },
                        provenance: talos_core::tool::ToolProvenance::Native,
                        summary_fields: vec![],
                    })
                    .await;
                let _ = tx
                    .send(AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: talos_core::message::Usage::default(),
                    })
                    .await;
            } else {
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });
        Ok(rx)
    }
}

impl ToolCallThenErrorModel {
    fn new() -> Self {
        Self {
            call_count: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            trailing_fragment: false,
        }
    }

    fn with_trailing_fragment() -> Self {
        Self {
            call_count: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            trailing_fragment: true,
        }
    }
}
#[async_trait]
impl LanguageModel for ToolCallThenErrorModel {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = mpsc::channel(64);
        let count = self.call_count.clone();
        let trailing_fragment = self.trailing_fragment;
        tokio::spawn(async move {
            let n = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n == 0 {
                let _ = tx
                    .send(AgentEvent::ToolCall {
                        call: talos_core::message::ToolCall {
                            id: "call_echo_1".into(),
                            name: "echo".into(),
                            input: serde_json::json!({"message": "hello"}),
                        },
                        provenance: talos_core::tool::ToolProvenance::Native,
                        summary_fields: vec![],
                    })
                    .await;
                let _ = tx
                    .send(AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: talos_core::message::Usage::default(),
                    })
                    .await;
            } else if trailing_fragment {
                let _ = tx
                    .send(AgentEvent::TextDelta {
                        delta: "trailing half-streamed fragment".into(),
                    })
                    .await;
                let _ = tx
                    .send(AgentEvent::Error {
                        message: "provider stream closed without explicit terminal signal ([DONE] or finish_reason)".into(),
                    })
                    .await;
            } else {
                let _ = tx
                    .send(AgentEvent::Error {
                        message: "provider server error".into(),
                    })
                    .await;
            }
        });
        Ok(rx)
    }
}

#[tokio::test]
async fn failed_continuation_preserves_completed_tool_prefix_without_trailing_fragment() {
    use talos_session::{SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("error-path", "")
        .expect("operation should succeed");

    let mut registry = ToolRegistry::new();
    registry.register(std::sync::Arc::new(EchoTool));
    #[allow(deprecated)]
    let agent = Agent::new(
        std::sync::Arc::new(ToolCallThenErrorModel::with_trailing_fragment()),
        registry,
    );

    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_persistence(
        session.clone(),
        SessionMetadata {
            provider: Some("mock".into()),
            model: Some("test".into()),
            ..SessionMetadata::default()
        },
    );

    let sq_tx = handle.sq_tx;
    let eq_rx = handle.eq_rx;
    let _actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "echo hello".into(),
        })
        .await
        .expect("operation should succeed");

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    let has_error_completion = events.iter().any(|e| {
        matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Error { .. })
        )
    });
    assert!(
        has_error_completion,
        "turn should complete with error status"
    );
    let persisted = session.read_messages().expect("operation should succeed");
    let has_tool_result = persisted
        .iter()
        .any(|m| matches!(m, Message::Tool { result } if result.content.contains("echo: hello")));
    assert!(
        has_tool_result,
        "SESSION-006 FIX: tool result must be persisted after provider error"
    );
    let has_user_msg = persisted
        .iter()
        .any(|m| matches!(m, Message::User { content } if content.contains("echo hello")));
    assert!(
        has_user_msg,
        "user message must be persisted in the partial turn prefix"
    );
    assert!(
        persisted.iter().all(|message| !matches!(
            message,
            Message::Assistant { content, .. }
                if content.contains("trailing half-streamed fragment")
        )),
        "the failed continuation fragment must not become a completed assistant fact"
    );
    let diagnostics = session
        .read_terminal_diagnostics()
        .expect("operation should succeed");
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(
        diagnostics[0].outcome,
        talos_session::ProviderTerminalOutcome::ToolUse
    );
    assert_eq!(diagnostics[0].response_ordinal, 1);
    assert_eq!(
        diagnostics[1].outcome,
        talos_session::ProviderTerminalOutcome::Error
    );
    assert_eq!(diagnostics[1].response_ordinal, 2);
}

#[tokio::test]
async fn fixture_adr042_durable_failed_turn_aborts_with_real_durable() {
    use talos_session::{PersistencePolicy, SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("adr042-real", "")
        .expect("operation should succeed");

    let durable = manager
        .create_or_open_session("adr042-real-durable")
        .expect("durable session");

    let mut registry = ToolRegistry::new();
    registry.register(std::sync::Arc::new(EchoTool));
    #[allow(deprecated)]
    let agent = Agent::new(std::sync::Arc::new(ToolCallThenErrorModel::new()), registry);

    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);

    actor.set_persistence(
        session.clone(),
        SessionMetadata {
            provider: Some("mock".into()),
            model: Some("test".into()),
            ..SessionMetadata::default()
        },
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());

    let sq_tx = handle.sq_tx;
    let eq_rx = handle.eq_rx;
    let _actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "echo hello".into(),
        })
        .await
        .expect("operation should succeed");

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    let has_error = events.iter().any(|e| {
        matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Error { .. })
        )
    });
    assert!(has_error, "turn should error");

    let has_entries_committed = events
        .iter()
        .any(|e| matches!(e, SessionEvent::EntriesCommitted { .. }));
    assert!(
        !has_entries_committed,
        "ADR-042: no EntriesCommitted on error path — durable failed turns abort"
    );

    let persisted = session.read_messages().expect("operation should succeed");
    let has_tool_result = persisted
        .iter()
        .any(|m| matches!(m, Message::Tool { result } if result.content.contains("echo: hello")));
    assert!(
        has_tool_result,
        "interactive session persists tool result on error (SESSION-006 fix)"
    );
}

#[tokio::test]
async fn fixture_persistence_failure_is_observable_in_error() {
    use talos_session::{SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("persist-fail", "")
        .expect("operation should succeed");

    let session_parent = session
        .file_path
        .parent()
        .expect("session file has parent")
        .to_path_buf();
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BoundaryBlockingTool {
        entered: entered.clone(),
        release: release.clone(),
    }));
    #[allow(deprecated)]
    let agent = Agent::new(
        Arc::new(CapturingSequenceModel {
            captured: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(VecDeque::from(vec![
                vec![
                    AgentEvent::ToolCall {
                        call: talos_core::message::ToolCall {
                            id: "persist-probe".into(),
                            name: "boundary_probe".into(),
                            input: serde_json::json!({}),
                        },
                        provenance: talos_core::tool::ToolProvenance::Native,
                        summary_fields: vec![],
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: Default::default(),
                    },
                ],
                vec![AgentEvent::Error {
                    message: "provider server error".into(),
                }],
            ]))),
        }),
        registry,
    );

    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_persistence(
        session.clone(),
        SessionMetadata {
            provider: Some("mock".into()),
            model: Some("test".into()),
            ..SessionMetadata::default()
        },
    );

    let sq_tx = handle.sq_tx;
    let eq_rx = handle.eq_rx;
    let _actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "echo hello".into(),
        })
        .await
        .expect("operation should succeed");

    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .expect("persistence fixture tool entered");
    // Fail finalization after admission, not startup custody inspection.
    std::fs::remove_file(&session.file_path).expect("remove initial session log");
    std::fs::remove_dir_all(&session_parent).expect("remove session parent");
    std::fs::write(&session_parent, "not a directory").expect("block session parent");
    release.notify_one();
    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    let error_message = events.iter().find_map(|e| match completed_status(e) {
        Some(TurnCompletionStatus::Error { message }) => Some(message.clone()),
        _ => None,
    });

    let message = error_message.expect("turn must emit a structured terminal error");
    assert!(
        message.contains("provider server error"),
        "provider error remains observable: {message}"
    );
    assert!(
        message.contains("failed to persist partial turn messages"),
        "persistence failure must be appended to the terminal error: {message}"
    );
}

#[tokio::test]
async fn fixture_durable_failed_turn_replays_closed_prefix_and_error_outcome() {
    use talos_session::{PersistencePolicy, SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager
        .create_session("durable-empty", "")
        .expect("operation should succeed");

    let durable_external_id = "durable-empty-check";
    let durable = manager
        .create_or_open_session(durable_external_id)
        .expect("durable session");

    let mut registry = ToolRegistry::new();
    registry.register(std::sync::Arc::new(EchoTool));
    #[allow(deprecated)]
    let agent = Agent::new(std::sync::Arc::new(ToolCallThenErrorModel::new()), registry);

    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_persistence(
        session.clone(),
        SessionMetadata {
            provider: Some("mock".into()),
            model: Some("test".into()),
            ..SessionMetadata::default()
        },
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());

    let sq_tx = handle.sq_tx;
    let eq_rx = handle.eq_rx;
    let _actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "echo hello".into(),
        })
        .await
        .expect("operation should succeed");

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    let has_error = events.iter().any(|e| {
        matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Error { .. })
        )
    });
    assert!(has_error, "turn should error");

    let reopened = manager
        .get_session_by_external_id(durable_external_id)
        .expect("durable lookup");
    let durable_session = reopened.expect("durable session must exist after failed turn");
    let transcript = durable_session
        .transcript(None, 100)
        .expect("transcript read");
    assert!(
        !transcript.is_empty(),
        "ADR-058: durable transcript must retain the admitted closed prefix"
    );
    assert!(
        transcript.iter().all(|entry| entry.turn_id.is_some()),
        "partial durable entries must bind to the failed turn"
    );
    assert_eq!(
        durable_session
            .session()
            .read_turn_transcript_outcomes()
            .expect("outcomes"),
        vec![talos_session::TurnTranscriptOutcomeRecord::new(
            transcript[0].turn_id.as_deref().expect("turn id"),
            talos_session::TurnTranscriptOutcome::Error,
        )]
    );
}

#[tokio::test]
async fn fixture_durable_cancelled_turn_replays_latest_closed_prefix() {
    use talos_session::{PersistencePolicy, SessionManager};

    let temp_dir = tempfile::tempdir().expect("operation should succeed");
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let durable_external_id = "durable-cancelled-check";
    let durable = manager
        .create_or_open_session(durable_external_id)
        .expect("durable session");
    let mut registry = ToolRegistry::new();
    registry.register(std::sync::Arc::new(EchoTool));
    #[allow(deprecated)]
    let agent = Agent::new(
        std::sync::Arc::new(ToolCallThenBlockingModel {
            call_count: Arc::new(std::sync::atomic::AtomicU8::new(0)),
        }),
        registry,
    );
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: temp_dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });
    sq_tx
        .send(SessionOp::Submit {
            message: "run echo then stop".into(),
        })
        .await
        .expect("submit");

    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = eq_rx.recv().await.expect("session event");
            if matches!(progress_event(&event), Some(AgentEvent::ToolResult { .. })) {
                break;
            }
        }
    })
    .await
    .expect("tool result timeout");
    sq_tx.send(SessionOp::Interrupt).await.expect("interrupt");
    let events = collect_events(eq_rx, Duration::from_secs(5)).await;
    assert!(events.iter().any(|event| {
        matches!(
            completed_status(event),
            Some(TurnCompletionStatus::Cancelled)
        )
    }));
    sq_tx.send(SessionOp::Shutdown).await.expect("shutdown");
    actor_task.await.expect("actor");

    let reopened = manager
        .get_session_by_external_id(durable_external_id)
        .expect("lookup")
        .expect("durable session");
    let transcript = reopened.transcript(None, 100).expect("transcript");
    assert!(
        !transcript.is_empty(),
        "closed tool exchange must survive cancel"
    );
    let turn_id = transcript[0].turn_id.clone().expect("turn id");
    assert!(
        transcript
            .iter()
            .all(|entry| entry.turn_id.as_deref() == Some(&turn_id))
    );
    assert_eq!(
        reopened.read_messages().expect("replay").len(),
        transcript.len()
    );
    assert_eq!(
        reopened
            .session()
            .read_turn_transcript_outcomes()
            .expect("outcome"),
        vec![talos_session::TurnTranscriptOutcomeRecord::new(
            turn_id,
            talos_session::TurnTranscriptOutcome::Cancelled,
        )]
    );
}
