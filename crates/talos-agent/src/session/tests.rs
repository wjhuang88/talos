use super::*;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use talos_core::message::{Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderResult};
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
            let mut responses = self.responses.lock().unwrap();
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
        self.captured.lock().unwrap().push(messages.to_vec());
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
        self.captured.lock().unwrap().push(messages.to_vec());
        let events = self
            .responses
            .lock()
            .unwrap()
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
        .unwrap();

    let events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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
        .unwrap();
    sq_tx
        .send(SessionOp::PreviewRequest {
            message: "verify skill".into(),
        })
        .await
        .unwrap();
    let events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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
        .unwrap();

    sq_tx
        .send(SessionOp::Submit {
            message: "again".into(),
        })
        .await
        .unwrap();

    let events = collect_until_completions(&mut eq_rx, 2).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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
        .unwrap();
    let mut events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    while let Ok(event) = eq_rx.try_recv() {
        events.push(event);
    }

    let requests = captured.lock().unwrap();
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
        .unwrap();
    sq_tx
        .send(SessionOp::SubmitStructured { submission })
        .await
        .unwrap();

    let mut events = collect_until_completions(&mut eq_rx, 1).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
    while let Ok(event) = eq_rx.try_recv() {
        events.push(event);
    }

    assert_eq!(captured.lock().unwrap().len(), 1);
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
        .unwrap();

    let receipt = tokio::time::timeout(Duration::from_secs(2), receipt_rx.recv())
        .await
        .expect("tracked durable receipt timeout")
        .expect("tracked durable receipt channel");
    assert!(receipt.disposition.has_durable_custody());

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if captured.lock().unwrap().len() == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Actor should execute accepted work without an EQ observer");

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();

    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 1, "Actor custody must execute exactly once");
    assert!(
        requests[0].iter().any(
            |message| matches!(message, Message::User { content } if content == "must run once")
        )
    );
}

#[tokio::test]
async fn context_budget_rejects_before_submission_started() {
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
        .unwrap();

    let mut events = Vec::new();
    loop {
        let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
            .await
            .expect("budget rejection timeout")
            .expect("session event channel");
        let rejected = matches!(
            event,
            SessionEvent::SubmissionRejected {
                reason: SubmissionRejectionReason::ContextBudgetExceeded,
                ..
            }
        );
        events.push(event);
        if rejected {
            break;
        }
    }
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();

    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::SubmissionStarted { .. }))
    );
    assert!(captured.lock().unwrap().is_empty());
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
            .unwrap();
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

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
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
        .unwrap();
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
        .unwrap();
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
        .unwrap();
    let _ = collect_until_completions(&mut eq_rx, 2).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();

    let calls = captured.lock().unwrap();
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
        .unwrap();

    sq_tx.send(SessionOp::Interrupt).await.unwrap();

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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

    sq_tx.send(SessionOp::Shutdown).await.unwrap();

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
        .unwrap();

    sq_tx.send(SessionOp::Shutdown).await.unwrap();

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
            .unwrap();
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
            result.unwrap_err(),
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
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "still here?".into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    sq_tx.send(SessionOp::Interrupt).await.unwrap();

    sq_tx
        .send(SessionOp::Submit {
            message: "after interrupt".into(),
        })
        .await
        .unwrap();

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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
            self.captured.lock().unwrap().push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().unwrap();
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

    // Submit 3 turns
    sq_tx
        .send(SessionOp::Submit {
            message: "turn 1".into(),
        })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 2".into(),
        })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 3".into(),
        })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let events = collect_until_completions(&mut eq_rx, 3).await;
    sq_tx.send(SessionOp::Shutdown).await.unwrap();
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

    // Verify the 3rd turn received history from turns 1 and 2
    let captured = captured_messages.lock().unwrap();
    assert!(captured.len() >= 3, "Should have captured at least 3 calls");

    // 3rd call should have messages from turns 1 and 2
    let third_call_messages = &captured[2];
    // Should have: User(turn 1), Assistant(first response), User(turn 2), Assistant(second response), User(turn 3 with system prompt)
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
            self.captured.lock().unwrap().push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().unwrap();
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
        .unwrap();
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

    sq_tx.send(SessionOp::Interrupt).await.unwrap();

    sq_tx
        .send(SessionOp::Submit {
            message: "turn 2".into(),
        })
        .await
        .unwrap();
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

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    let _ = actor_task.await;

    let captured = captured_messages.lock().unwrap();
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

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("resume-test", "").unwrap();
    let session_id = session.id.to_string();
    session
        .append(&Message::User {
            content: "prior question".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "prior answer".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();
    let resumed = manager.resume_session(&session_id).unwrap();
    let prior_history = resumed.read_messages().unwrap();

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
            self.captured.lock().unwrap().push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                let mut responses = self.responses.lock().unwrap();
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
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    let _ = actor_task.await;

    let captured = captured_messages.lock().unwrap();
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

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("single-flow", "").unwrap();
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
        .unwrap();

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

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();

    assert_eq!(sequences, (0..sequences.len() as u64).collect::<Vec<_>>());
    assert!(
        session_ids
            .iter()
            .all(|event_session_id| event_session_id == &session.id.to_string()),
        "every canonical event must carry the durable session identity"
    );
    assert_eq!(
        session.read_messages().unwrap(),
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
        session.read_events().unwrap().is_empty(),
        "canonical persistence must not duplicate transient AgentEvents"
    );
}

// ── TOOL-021 fixture: provider error after tool execution drops tool results ──

struct EchoTool;

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

/// Model that sends a tool call, then on second call sends an error.
struct ToolCallThenErrorModel {
    call_count: Arc<std::sync::atomic::AtomicU8>,
    trailing_fragment: bool,
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
                // First call: produce a tool call
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

/// Proves SESSION-006 / I135 FIX: when a provider error occurs after tool execution,
/// the session NOW persists the completed tool exchange for resume.
#[tokio::test]
async fn failed_continuation_preserves_completed_tool_prefix_without_trailing_fragment() {
    use talos_session::{SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("error-path", "").unwrap();

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
        .unwrap();

    // Wait for turn completion
    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    // Verify the turn completed with an error
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
    // SESSION-006 / I135 FIX: The tool result IS NOW persisted because the
    // error branch in turn.rs calls persist_turn_messages with partial_messages.
    let persisted = session.read_messages().unwrap();
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
    let diagnostics = session.read_terminal_diagnostics().unwrap();
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

/// Proves ADR-042 is preserved with REAL durable persistence: when both
/// interactive and durable persistence are configured, a provider error
/// still results in NO durable commit (no EntriesCommitted) while the
/// interactive session retains the completed tool exchange.
#[tokio::test]
async fn fixture_adr042_durable_failed_turn_aborts_with_real_durable() {
    use talos_session::{DurableSession, PersistencePolicy, SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("adr042-real", "").unwrap();

    // Create a real durable session
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

    // Set BOTH interactive and durable persistence
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
        .unwrap();

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    // Turn must error
    let has_error = events.iter().any(|e| {
        matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Error { .. })
        )
    });
    assert!(has_error, "turn should error");

    // ADR-042: NO EntriesCommitted on error path — durable failed turns abort
    let has_entries_committed = events
        .iter()
        .any(|e| matches!(e, SessionEvent::EntriesCommitted { .. }));
    assert!(
        !has_entries_committed,
        "ADR-042: no EntriesCommitted on error path — durable failed turns abort"
    );

    // Interactive persistence DOES have the tool result (SESSION-006 fix)
    let persisted = session.read_messages().unwrap();
    let has_tool_result = persisted
        .iter()
        .any(|m| matches!(m, Message::Tool { result } if result.content.contains("echo: hello")));
    assert!(
        has_tool_result,
        "interactive session persists tool result on error (SESSION-006 fix)"
    );
    // The actor owns the DurableSession. We verify ADR-042 via the
    // EntriesCommitted event absence (checked above), which is the
    // authoritative signal that commit_turn was never called.
}

/// Proves persistence failure is observable: when persist_turn_messages
/// fails, the error message includes the persistence failure warning.
#[tokio::test]
async fn fixture_persistence_failure_is_observable_in_error() {
    use talos_session::{SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("persist-fail", "").unwrap();

    // Deterministically block the concrete session path on every platform:
    // replace its parent directory with a regular file after creation.
    let session_parent = session
        .file_path
        .parent()
        .expect("session file has parent")
        .to_path_buf();
    std::fs::remove_file(&session.file_path).expect("remove initial session log");
    std::fs::remove_dir_all(&session_parent).expect("remove session parent");
    std::fs::write(&session_parent, "not a directory").expect("block session parent");

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

    let sq_tx = handle.sq_tx;
    let eq_rx = handle.eq_rx;
    let _actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::Submit {
            message: "echo hello".into(),
        })
        .await
        .unwrap();

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    // The turn should complete with error (provider error from the model)
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

/// Proves ADR-042 durable transcript is empty after failed turn:
/// reopens the durable session and verifies no committed entries.
#[tokio::test]
async fn fixture_durable_transcript_empty_after_failed_turn() {
    use talos_session::{PersistencePolicy, SessionManager, SessionMetadata};

    let temp_dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(temp_dir.path().to_path_buf());
    let session = manager.create_session("durable-empty", "").unwrap();

    // Create durable session and keep the directory for later reopening
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
        .unwrap();

    let events = collect_events(eq_rx, Duration::from_secs(5)).await;

    // Verify turn errored
    let has_error = events.iter().any(|e| {
        matches!(
            completed_status(e),
            Some(TurnCompletionStatus::Error { .. })
        )
    });
    assert!(has_error, "turn should error");

    // Reopen the durable session and verify transcript is empty
    // (ADR-042: failed turns abort, leaving no committed entries)
    let reopened = manager
        .get_session_by_external_id(durable_external_id)
        .expect("durable lookup");
    let durable_session = reopened.expect("durable session must exist after failed turn");
    let transcript = durable_session
        .transcript(None, 100)
        .expect("transcript read");
    assert!(
        transcript.is_empty(),
        "ADR-042: durable transcript must be empty after failed turn (got {} entries)",
        transcript.len()
    );
}
