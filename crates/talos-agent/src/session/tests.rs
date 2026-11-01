use super::*;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use talos_core::message::{Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderProgress, ProviderResult, ToolDefinition};
use talos_core::session::{RuntimePolicy, SessionEvent, TurnCompletionStatus, TurnEventPayload};
use talos_core::submission::PendingSubmissionState;
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
async fn post_turn_pressure_marks_large_response_without_summarizing_or_rewriting_history() {
    let response = "finalized response ".repeat(10_000);
    let model = MockModel::new(vec![success_events(&response), success_events("next")]);
    let remaining = model.responses.clone();
    let workspace = tempfile::tempdir().expect("workspace");
    let (handle, mut actor) = AppServerSession::new(
        make_agent(model),
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: workspace.path().to_path_buf(),
            initial_history: vec![],
            model_context_limit: 20_000,
        },
    );
    let first = actor
        .start_submission(
            structured_submission(
                "pressure-1",
                "item-1",
                0,
                "hi",
                SubmissionSource::Compatibility,
            ),
            1,
            CancellationToken::new(),
        )
        .await
        .unwrap_or_else(|_| panic!("first admission"));
    let record = first.handle.await.expect("join").expect("record");
    actor.commit_turn_record(record);
    assert_eq!(
        remaining.lock().expect("responses").len(),
        1,
        "maintenance must not invoke the provider"
    );
    assert!(actor.history.iter().any(|message| matches!(message,
        Message::Assistant { content, .. } if content == &response)));
    let marked = actor
        .post_turn_pressure
        .take()
        .expect("large finalized reply marks pressure");
    assert!(marked > actor.model_context_limit);
    actor
        .post_turn_pressure
        .record(marked, actor.model_context_limit);
    let prior_slot = actor.post_turn_pressure.clone();
    actor.model_context_limit = u32::MAX;
    let second = actor
        .start_submission(
            structured_submission(
                "pressure-2",
                "item-2",
                0,
                "continue",
                SubmissionSource::Compatibility,
            ),
            2,
            CancellationToken::new(),
        )
        .await
        .unwrap_or_else(|_| panic!("second admission"));
    // Simulate a late result from prior work after the new turn owns its slot.
    prior_slot.record(u32::MAX, 1);
    actor.commit_turn_record(second.handle.await.expect("join").expect("record"));
    assert_eq!(actor.post_turn_pressure.take(), None);
    assert!(remaining.lock().expect("responses").is_empty());
    actor.post_turn_pressure.record(100, 1);
    actor.set_generation(1);
    assert_eq!(actor.post_turn_pressure.take(), None);
    drop(handle);
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
    let names: std::collections::BTreeSet<_> = actor
        .agent
        .tools
        .list()
        .iter()
        .map(|tool| tool.name().to_owned())
        .collect();
    assert_eq!(
        names,
        std::collections::BTreeSet::from(["process".to_owned()]),
        "new Session-owned operations require an explicit I308 disposition"
    );
    assert!(actor.agent.tools.is_builtin_instance("process"));
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
async fn session_process_dispatch_respects_read_and_cancel_policy() {
    use talos_core::tool::ToolNature;
    use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};

    for (action, denied) in [("list", false), ("list", true), ("cancel", true)] {
        let root = tempfile::tempdir().expect("workspace");
        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        engine.add_rule(PermissionRule::new_nature(
            if action == "cancel" {
                ToolNature::Execute
            } else {
                ToolNature::Read
            },
            None,
            None,
            if denied {
                PermissionDecision::Deny("process fixture policy".into())
            } else {
                PermissionDecision::Allow
            },
        ));
        let model = MockModel::new(vec![
            vec![
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "process-policy".into(),
                        name: "process".into(),
                        input: serde_json::json!({"action": action, "job_id": "not-owned"}),
                    },
                    provenance: talos_core::tool::ToolProvenance::Native,
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Default::default(),
                },
            ],
            success_events("done"),
        ]);
        let agent = Agent::with_security_and_hooks(
            Arc::new(model),
            ToolRegistry::new(),
            Some(Arc::new(engine)),
            None,
            root.path().into(),
            Arc::new(talos_plugin::HookRegistry::new()),
        );
        let (handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: root.path().into(),
                initial_history: vec![],
                model_context_limit: 128_000,
            },
        );
        let mut events = handle.eq_rx;
        let task = tokio::spawn(async move { actor.run().await });
        handle
            .sq_tx
            .send(SessionOp::Submit {
                message: "inspect owned jobs".into(),
            })
            .await
            .expect("submit");
        let captured = collect_until_completions(&mut events, 1).await;
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        task.await.expect("actor");
        let results: Vec<_> = captured
            .iter()
            .filter_map(|event| match progress_event(event) {
                Some(AgentEvent::ToolResult { result, .. })
                    if result.tool_use_id == "process-policy" =>
                {
                    Some(result)
                }
                _ => None,
            })
            .collect();
        assert_eq!(results.len(), 1, "{action}: {captured:?}");
        let result = results[0];
        assert_eq!(result.is_error, denied, "{action}: {}", result.content);
        if denied {
            assert!(
                result.content.contains("Permission denied:"),
                "{}",
                result.content
            );
        } else {
            let value: serde_json::Value =
                serde_json::from_str(&result.content).expect("process list");
            assert_eq!(value["jobs"].as_array().map(Vec::len), Some(0));
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn session_process_cancels_only_its_own_running_job() {
    use talos_core::tool::ToolNature;
    use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};

    struct HumanApproval;
    #[async_trait]
    impl crate::permission_pipeline::ApprovalResolver for HumanApproval {
        async fn resolve(
            &self,
            _: crate::permission_pipeline::PermissionApprovalRequest,
            _: Duration,
        ) -> Result<talos_core::ApprovalChoice, crate::permission_pipeline::ApprovalResolverError>
        {
            Ok(talos_core::ApprovalChoice::ApproveOnce)
        }
    }

    async fn collect_job_turn(
        events: &mut tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    ) -> Vec<SessionEvent> {
        tokio::time::timeout(Duration::from_secs(10), async {
            let mut captured = Vec::new();
            loop {
                let event = events.recv().await.expect("session event");
                let completed = completed_status(&event).is_some();
                captured.push(event);
                if completed {
                    return captured;
                }
            }
        })
        .await
        .expect("bounded job-control turn")
    }

    fn tool_events(name: &str, input: serde_json::Value) -> Vec<AgentEvent> {
        vec![
            AgentEvent::ToolCall {
                call: talos_core::message::ToolCall {
                    id: "job-control".into(),
                    name: name.into(),
                    input,
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Default::default(),
            },
        ]
    }
    fn tool_result(events: &[SessionEvent]) -> &talos_core::message::MessageToolResult {
        events
            .iter()
            .find_map(|event| match progress_event(event) {
                Some(AgentEvent::ToolResult { result, .. })
                    if result.tool_use_id == "job-control" =>
                {
                    Some(result)
                }
                _ => None,
            })
            .expect("actual tool result")
    }
    let root = tempfile::tempdir().expect("workspace");
    let mut sessions = Vec::new();
    for _ in 0..2 {
        let model = MockModel::new(vec![]);
        let responses = model.responses.clone();
        let mut tools = ToolRegistry::new();
        tools.register(Arc::new(talos_tools::ExecTool::new(root.path().into())));
        let mut engine = PermissionEngine::empty();
        engine.set_workspace_root(root.path().into());
        for nature in [ToolNature::Read, ToolNature::Execute] {
            engine.add_rule(PermissionRule::new_nature(
                nature,
                None,
                None,
                PermissionDecision::Allow,
            ));
        }
        let agent = Agent::with_security_and_hooks(
            Arc::new(model),
            tools,
            Some(Arc::new(engine)),
            None,
            root.path().into(),
            Arc::new(talos_plugin::HookRegistry::new()),
        )
        .with_approval_resolver(Arc::new(HumanApproval));
        let (handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: root.path().into(),
                initial_history: vec![],
                model_context_limit: 128_000,
            },
        );
        let task = tokio::spawn(async move { actor.run().await });
        sessions.push((handle, responses, task));
    }
    sessions[0].1.lock().expect("responses").extend([
        tool_events("exec", serde_json::json!({"command":"/bin/sleep", "args":["30"], "background":true, "timeout_secs":30})),
        success_events("started"),
    ]);
    sessions[0]
        .0
        .sq_tx
        .send(SessionOp::Submit {
            message: "start bounded job".into(),
        })
        .await
        .expect("submit");
    let events = collect_job_turn(&mut sessions[0].0.eq_rx).await;
    let result = tool_result(&events);
    assert!(!result.is_error, "{}", result.content);
    let started: serde_json::Value = serde_json::from_str(&result.content).expect("receipt");
    let id = started["job_id"].as_str().expect("job id").to_owned();
    for (session, action) in [(1, "status"), (1, "cancel"), (0, "status"), (0, "cancel")] {
        sessions[session].1.lock().expect("responses").extend([
            tool_events(
                "process",
                serde_json::json!({"action":action, "job_id":id, "wait_ms":5000}),
            ),
            success_events("observed"),
        ]);
        sessions[session]
            .0
            .sq_tx
            .send(SessionOp::Submit {
                message: "inspect or cancel job".into(),
            })
            .await
            .expect("submit");
        let events = collect_job_turn(&mut sessions[session].0.eq_rx).await;
        let result = tool_result(&events);
        if session == 1 {
            assert!(result.is_error, "foreign {action}: {}", result.content);
            assert_eq!(result.content, "unknown background job");
        } else {
            assert!(!result.is_error, "owner {action}: {}", result.content);
            let status: serde_json::Value = serde_json::from_str(&result.content).expect("status");
            assert_eq!(
                status["state"],
                if action == "cancel" {
                    "cancelled"
                } else {
                    "running"
                }
            );
            if action == "cancel" {
                assert_eq!(status["cleanup_outcome"], "terminated");
            }
        }
    }
    for (handle, _, task) in sessions {
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        tokio::time::timeout(Duration::from_secs(10), task)
            .await
            .expect("bounded shutdown")
            .expect("actor");
    }
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
async fn context_budget_terminalizes_before_submission_started() {
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
    let pending_store = actor.pending_store.clone();
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
            .expect("budget terminalization timeout")
            .expect("session event channel");
        let terminalized = matches!(
            event,
            SessionEvent::SubmissionResolved {
                state: PendingSubmissionState::TerminalError,
                ..
            }
        );
        events.push(event);
        if terminalized {
            break;
        }
    }
    // Exceed the queue's lifetime capacity to detect leaked admission counters.
    for index in 0..=MAX_STEERING_QUEUE_ITEMS {
        let id = format!("rejected-{index}");
        sq_tx
            .send(SessionOp::SubmitStructured {
                submission: structured_submission(
                    &id,
                    &format!("item-{index}"),
                    13,
                    "request",
                    SubmissionSource::User,
                ),
            })
            .await
            .expect("submit after terminal rejection");
        loop {
            let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
                .await
                .expect("next rejection must not stall")
                .expect("event channel");
            assert!(
                !matches!(&event, SessionEvent::SubmissionRejected { .. }),
                "terminal submissions must not consume queue capacity: {event:?}"
            );
            let resolved = matches!(&event, SessionEvent::SubmissionResolved {
                submission_id, state: PendingSubmissionState::TerminalError, ..
            } if submission_id == &id);
            events.push(event);
            if resolved {
                break;
            }
        }
    }
    sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("operation should succeed");
    actor_task.await.expect("operation should succeed");

    let record = pending_store
        .get("over_budget_batch")
        .expect("durable lookup")
        .expect("terminal record retained");
    assert_eq!(record.state, PendingSubmissionState::TerminalError);
    assert!(
        record.turn_id.is_none(),
        "rejection must not fabricate a Turn"
    );
    assert!(
        pending_store
            .recover_unstarted()
            .expect("recover pending")
            .is_empty()
    );
    assert!(
        pending_store
            .recover_running()
            .expect("recover running")
            .is_empty()
    );

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
    exercise_boundary_injection_with_gate(cancel, provider_error, cancel_before, false, false)
        .await;
}

#[tokio::test]
async fn boundary_cancel_after_agent_ack_before_actor_projection_preserves_once() {
    exercise_boundary_injection_with_gate(true, false, false, true, false).await;
}

async fn exercise_boundary_injection_with_gate(
    cancel: bool,
    provider_error: bool,
    cancel_before: bool,
    handoff_cancel: bool,
    compact: bool,
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
    struct RejectCompactObservation {
        calls: AtomicUsize,
    }
    #[async_trait]
    impl talos_plugin::HookHandler for RejectCompactObservation {
        fn name(&self) -> &str {
            "reject-compact-with-steering"
        }
        fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
            &[talos_plugin::HookEventKind::BeforeProviderCall]
        }
        async fn on_event(
            &self,
            _: &talos_plugin::HookContext,
            _: &mut talos_plugin::HookEvent<'_>,
        ) -> talos_plugin::HookResult {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 1 {
                talos_plugin::HookResult::Deny {
                    reason: "reject pre-steering request".into(),
                }
            } else {
                talos_plugin::HookResult::Continue
            }
        }
    }
    let mut hooks = talos_plugin::HookRegistry::new();
    if compact && provider_error {
        hooks.register(Arc::new(RejectCompactObservation {
            calls: AtomicUsize::new(0),
        }));
    }
    let agent = Agent::with_security_and_hooks(
        Arc::new(CapturingSequenceModel {
            captured: captured.clone(),
            responses: Arc::new(Mutex::new(responses.into())),
        }),
        registry,
        None,
        None,
        temp.path().to_path_buf(),
        Arc::new(hooks),
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
    if compact {
        handle
            .sq_tx
            .send(SessionOp::Compact {
                session_generation: 1,
                operation_id: 1,
            })
            .await
            .expect("compact with durable steering");
        loop {
            let event = tokio::time::timeout(Duration::from_secs(5), eq_rx.recv())
                .await
                .expect("queued deadline")
                .expect("event");
            let queued = matches!(
                &event,
                SessionEvent::CompactStatus {
                    state: talos_core::session::CompactState::Queued,
                    ..
                }
            );
            events.push(event);
            if queued {
                break;
            }
        }
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

    if compact {
        let outcomes: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                SessionEvent::CompactStatus {
                    operation_id: 1,
                    state:
                        talos_core::session::CompactState::Completed {
                            outcome,
                            replayed: false,
                        },
                    ..
                } => Some(*outcome),
                _ => None,
            })
            .collect();
        assert_eq!(outcomes.len(), 1, "one terminal for boundary compaction");
        if provider_error {
            assert_eq!(outcomes[0], talos_core::session::CompactOutcome::Failed);
        } else {
            assert!(matches!(
                outcomes[0],
                talos_core::session::CompactOutcome::NotEligible { .. }
            ));
        }
    }
    let requests = captured.lock().expect("captured requests");
    let injected_position = events.iter().position(|event| matches!(event,
        SessionEvent::StructuredSubmissionInjected { submission, .. } if submission.id == "steering"
    )).expect("first injection event");
    let tool_result_position = events
        .iter()
        .position(|event| {
            matches!(progress_event(event),
                Some(AgentEvent::ToolResult { result, .. }) if result.tool_use_id == "probe-1"
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
    assert_recovery_receipts_verify(&durable_session, 1);
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
        matches!(
            reopened.read_messages().expect("replay").last(),
            Some(Message::Assistant { content, tool_calls, .. })
                if content.contains("this turn was interrupted by the user")
                    && tool_calls.is_empty()
        ),
        "cancelled history must close the old request before the next submission"
    );
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
    assert_recovery_receipts_verify(&reopened, 1);
}

#[cfg(unix)]
mod prepared_process_cleanup {
    use super::*;
    use serde_json::{Value, json};
    use std::path::PathBuf;
    use talos_core::tool::*;
    use talos_permission::{AdmittedInvocation, PermissionRequest};
    use talos_sandbox::{OwnedCommandExecutor, OwnedCommandOperation};
    struct ProcessTool {
        root: PathBuf,
        owner: Arc<OwnedCommandExecutor>,
    }
    struct ProcessPlan {
        root: PathBuf,
        operation: OwnedCommandOperation,
        identity: PreparedPlanIdentity,
        provenance: ToolProvenance,
        facets: Vec<ToolPermissionFacet>,
        input: Value,
    }
    #[async_trait]
    impl AgentTool for ProcessTool {
        fn name(&self) -> &str {
            "process_fixture"
        }
        fn description(&self) -> &str {
            "owned process cancellation fixture"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object"})
        }
        fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupOwner>> {
            Some(self.owner.clone())
        }
        async fn execute(&self, _: Value) -> ToolResult {
            panic!("prepared route required")
        }
        fn prepare_normalized_with_provenance(
            &self,
            input: &Value,
            provenance: ToolProvenance,
        ) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
            Ok(Some(Box::new(ProcessPlan {
                root: self.root.clone(),
                operation: self.owner.operation(),
                identity: PreparedPlanIdentity::new(),
                provenance,
                // Test-only auto-admission permits exercising the real execution path.
                facets: vec![ToolPermissionFacet::new(ToolNature::Read)],
                input: input.clone(),
            })))
        }
    }
    #[async_trait]
    impl PreparedNormalToolInvocation for ProcessPlan {
        fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
            Some(&self.identity)
        }
        fn permission_profile(&self) -> &[ToolPermissionFacet] {
            &self.facets
        }
        fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupReceipt>> {
            Some(Arc::new(self.operation.cleanup_receipt()))
        }
        async fn execute(
            self: Box<Self>,
            admission: Box<dyn std::any::Any + Send>,
        ) -> ToolExecutionOutput {
            let proof = admission
                .downcast::<AdmittedInvocation>()
                .expect("real admission");
            proof
                .consume_for_prepared(
                    &PermissionRequest::new(
                        "process_fixture",
                        self.provenance.clone(),
                        &self.facets,
                        &self.input,
                    ),
                    &self.identity,
                )
                .expect("bound proof");
            let mut command = std::process::Command::new("sh");
            command.current_dir(&self.root).args([
                "-c",
                "(sleep 2; printf forbidden > delayed) & printf ready > ready; wait",
            ]);
            let result = self
                .operation
                .execute(command, Duration::from_secs(20), 1024)
                .await;
            ToolExecutionOutput::error(format!("fixture result: {result:?}"))
        }
    }

    #[tokio::test]
    async fn session_interrupt_confirms_real_descendant_cleanup_before_terminal() {
        tokio::time::timeout(Duration::from_secs(12), async {
            let root = tempfile::tempdir().expect("tempdir");
            let owner = Arc::new(OwnedCommandExecutor::default());
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(ProcessTool {
                root: root.path().into(),
                owner: owner.clone(),
            }));
            let model = MockModel::new(vec![vec![
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "process-1".into(),
                        name: "process_fixture".into(),
                        input: json!({}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Default::default(),
                },
            ]]);
            let agent = Agent::with_security_and_hooks(
                Arc::new(model),
                registry,
                Some(Arc::new(
                    talos_permission::PermissionEngine::with_workspace_root(root.path().into()),
                )),
                None,
                root.path().into(),
                Arc::new(talos_plugin::HookRegistry::new()),
            );
            let config = SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: root.path().into(),
                initial_history: vec![],
                model_context_limit: 128_000,
            };
            let (handle, mut actor) = AppServerSession::new(agent, config);
            let mut eq_rx = handle.eq_rx;
            let sq_tx = handle.sq_tx;
            let actor_task = tokio::spawn(async move { actor.run().await });
            sq_tx
                .send(SessionOp::Submit {
                    message: "run fixture".into(),
                })
                .await
                .expect("submit");
            while !root.path().join("ready").exists() {
                while let Ok(event) = eq_rx.try_recv() {
                    assert!(
                        completed_status(&event).is_none(),
                        "turn ended before process startup: {event:?}"
                    );
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            sq_tx.send(SessionOp::Interrupt).await.expect("interrupt");
            loop {
                let event = eq_rx.recv().await.expect("terminal event");
                if let Some(status) = completed_status(&event) {
                    assert!(matches!(status, TurnCompletionStatus::Cancelled));
                    break;
                }
            }
            // Terminal cancellation is the observable barrier; no extra receipt wait.
            tokio::time::sleep(Duration::from_millis(2100)).await;
            assert!(!root.path().join("delayed").exists());
            sq_tx.send(SessionOp::Shutdown).await.expect("shutdown");
            actor_task.await.expect("actor joined");
        })
        .await
        .expect("session deadline");
    }
}

#[tokio::test]
async fn fresh_actor_admits_submit_without_recovery_sidecar_creation() {
    let dir = tempfile::tempdir().expect("isolated workspace");
    let agent = make_agent(MockModel::new(vec![success_events("hello")]));
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: dir.path().to_path_buf(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let sidecar = actor.pending_store.path().to_path_buf();
    let actor_task = tokio::spawn(async move { actor.run().await });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "hi".into(),
        })
        .await
        .expect("submit");
    let mut events = handle.eq_rx;
    let received = collect_until_completions(&mut events, 1).await;
    assert!(received.iter().any(|event| matches!(
        completed_status(event),
        Some(TurnCompletionStatus::Success { .. })
    )));
    assert!(
        !sidecar.exists(),
        "startup must not create a recovery-only journal"
    );
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    tokio::time::timeout(Duration::from_secs(2), actor_task)
        .await
        .expect("bounded shutdown")
        .expect("actor task");
    assert!(
        !sidecar.exists(),
        "shutdown must not initialize an absent pending journal"
    );
}

#[tokio::test]
async fn actor_compacts_long_recovered_text_without_replacing_raw_history() {
    struct CheckpointModel {
        summaries: AtomicUsize,
        ordinary: AtomicUsize,
    }
    #[async_trait]
    impl LanguageModel for CheckpointModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let summary = match messages {
                [Message::User { content }]
                    if content.starts_with("Produce only a JSON checkpoint") =>
                {
                    Some(content)
                }
                _ => None,
            };
            let text = if let Some(prompt) = summary {
                self.summaries.fetch_add(1, Ordering::SeqCst);
                assert!(!prompt.contains("WORKSPACE_CONTEXT_MUST_REMAIN_VERBATIM"));
                let sources: Vec<serde_json::Value> =
                    serde_json::from_str(prompt.split("\nSources: ").last().expect("sources"))
                        .expect("JSON");
                let item = serde_json::json!({"text":"Preserve original task", "sources":[sources[0]["source"]]});
                serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
                    "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                    "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
                .to_string()
            } else {
                self.ordinary.fetch_add(1, Ordering::SeqCst);
                assert!(
                    messages
                        .iter()
                        .any(|message| matches!(message, Message::System { .. }))
                );
                assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content == "WORKSPACE_CONTEXT_MUST_REMAIN_VERBATIM")));
                assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.contains("Derived checkpoint"))));
                "continued after checkpoint".into()
            };
            let (tx, rx) = mpsc::channel(8);
            for event in success_events(&text) {
                tx.send(event).await.expect("fixture stream");
            }
            Ok(rx)
        }
    }
    let model = Arc::new(CheckpointModel {
        summaries: AtomicUsize::new(0),
        ordinary: AtomicUsize::new(0),
    });
    let mut agent = Agent::with_security_and_hooks(
        model.clone(),
        ToolRegistry::new(),
        None,
        None,
        "/tmp".into(),
        Arc::new(talos_plugin::HookRegistry::new()),
    );
    agent.workspace_context = Some("WORKSPACE_CONTEXT_MUST_REMAIN_VERBATIM".into());
    let raw: Vec<_> = (0..50)
        .flat_map(|index| {
            [
                Message::User {
                    content: format!("task {index}: {}", "x".repeat(4000)),
                },
                Message::Assistant {
                    content: "recorded".into(),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ]
        })
        .collect();
    let config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: "/tmp".into(),
        initial_history: raw.clone(),
        model_context_limit: 20_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, config);
    let mut eq_rx = handle.eq_rx;
    let sq_tx = handle.sq_tx;
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("manual compact");
    let outcome = loop {
        let event = tokio::time::timeout(Duration::from_secs(10), eq_rx.recv())
            .await
            .expect("manual deadline")
            .expect("event");
        assert!(
            !is_turn_started(&event),
            "compaction must not create a Turn"
        );
        if let SessionEvent::CompactStatus {
            state:
                talos_core::session::CompactState::Completed {
                    outcome,
                    replayed: false,
                },
            ..
        } = event
        {
            break outcome;
        }
    };
    assert!(
        matches!(outcome, talos_core::session::CompactOutcome::Applied { before, after } if after < before && after <= 20_000)
    );
    let summary_calls = model.summaries.load(Ordering::SeqCst);
    sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("duplicate");
    assert!(
        matches!(eq_rx.recv().await, Some(SessionEvent::CompactStatus {
        state: talos_core::session::CompactState::Completed { outcome: repeated, replayed: true }, ..
    }) if repeated == outcome)
    );
    assert_eq!(model.summaries.load(Ordering::SeqCst), summary_calls);
    sq_tx
        .send(SessionOp::Submit {
            message: "continue".into(),
        })
        .await
        .expect("submit");
    let events = collect_until_completions(&mut eq_rx, 1).await;
    assert!(events.iter().any(|event| matches!(
        completed_status(event),
        Some(TurnCompletionStatus::Success { .. })
    )));
    sq_tx.send(SessionOp::Shutdown).await.expect("shutdown");
    let history = task.await.expect("actor exit");
    assert_eq!(&history[..raw.len()], raw);
    assert!(history.len() > raw.len());
    assert!(model.summaries.load(Ordering::SeqCst) > 1);
    assert_eq!(model.ordinary.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn manual_compact_below_threshold_is_visible_and_never_dispatches_a_turn() {
    use talos_core::session::{CompactOutcome, CompactState};
    let model = Arc::new(MockModel::new(vec![]));
    let agent = Agent::new(model, ToolRegistry::new());
    let config = SessionConfig {
        workspace_root: std::env::temp_dir(),
        runtime_policy: RuntimePolicy::interactive(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (mut handle, mut actor) = AppServerSession::new(agent, config);
    actor.set_generation(3);
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 2,
            operation_id: 1,
        })
        .await
        .expect("stale");
    assert!(matches!(
        handle.eq_rx.recv().await,
        Some(SessionEvent::CompactStatus {
            state: CompactState::StaleGeneration,
            ..
        })
    ));
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 3,
            operation_id: 1,
        })
        .await
        .expect("compact");
    let mut states = vec![];
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), handle.eq_rx.recv())
            .await
            .expect("deadline")
            .expect("event");
        assert!(!is_turn_started(&event));
        if let SessionEvent::CompactStatus { state, .. } = event {
            states.push(state);
            if matches!(state, CompactState::Completed { .. }) {
                break;
            }
        }
    }
    assert_eq!(states.len(), 3);
    assert_eq!(states[0], CompactState::Queued);
    assert_eq!(states[1], CompactState::Started);
    assert!(matches!(
        states[2],
        CompactState::Completed {
            outcome: CompactOutcome::NotEligible { .. },
            replayed: false
        }
    ));
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    assert!(task.await.expect("actor").is_empty());
}

#[tokio::test]
async fn manual_compact_shutdown_cancels_queued_work_once_at_active_turn() {
    use talos_core::session::{CompactOutcome, CompactState};
    let model = Arc::new(SlowModel {
        delay: Duration::from_secs(60),
        events: success_events("late"),
    });
    let agent = Agent::new(model, ToolRegistry::new());
    let config = SessionConfig {
        workspace_root: std::env::temp_dir(),
        runtime_policy: RuntimePolicy::interactive(),
        initial_history: vec![],
        model_context_limit: 128_000,
    };
    let (mut handle, mut actor) = AppServerSession::new(agent, config);
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "active".into(),
        })
        .await
        .expect("submit");
    while let Some(event) = handle.eq_rx.recv().await {
        if is_turn_started(&event) {
            break;
        }
    }
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("compact");
    loop {
        if matches!(
            handle.eq_rx.recv().await,
            Some(SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            })
        ) {
            break;
        }
    }
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 2,
        })
        .await
        .expect("busy request");
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let mut terminals = 0;
    let mut busy = 0;
    while let Some(event) = tokio::time::timeout(Duration::from_secs(5), handle.eq_rx.recv())
        .await
        .expect("shutdown deadline")
    {
        match event {
            SessionEvent::CompactStatus {
                state:
                    CompactState::Completed {
                        outcome: CompactOutcome::Cancelled,
                        replayed: false,
                    },
                operation_id: 1,
                ..
            } => terminals += 1,
            SessionEvent::CompactStatus {
                state: CompactState::Busy,
                operation_id: 2,
                ..
            } => busy += 1,
            SessionEvent::CompactStatus {
                state: CompactState::Started,
                ..
            } => panic!("must not compact an incomplete turn"),
            _ => {}
        }
    }
    assert_eq!((terminals, busy), (1, 1));
    let _ = task.await.expect("actor");
}

#[tokio::test]
async fn targeted_compact_cancel_preserves_active_turn_and_replays_terminal_receipt() {
    use talos_core::session::{CompactOutcome, CompactState};
    let model = Arc::new(SlowModel {
        delay: Duration::from_millis(300),
        events: success_events("active turn survives compact cancellation"),
    });
    let agent = Agent::new(model, ToolRegistry::new());
    let (mut handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            workspace_root: std::env::temp_dir(),
            runtime_policy: RuntimePolicy::interactive(),
            initial_history: vec![],
            model_context_limit: 128_000,
        },
    );
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "continue current work".into(),
        })
        .await
        .expect("submit");
    while let Some(event) = handle.eq_rx.recv().await {
        if is_turn_started(&event) {
            break;
        }
    }
    for operation in [
        SessionOp::Compact {
            session_generation: 0,
            operation_id: 10,
        },
        SessionOp::CancelCompact {
            session_generation: 0,
            operation_id: 10,
        },
        SessionOp::Compact {
            session_generation: 0,
            operation_id: 10,
        },
    ] {
        handle.sq_tx.send(operation).await.expect("operation");
    }
    let mut completed = false;
    let mut terminals = Vec::new();
    while !completed || terminals.len() < 2 {
        let event = tokio::time::timeout(Duration::from_secs(5), handle.eq_rx.recv())
            .await
            .expect("bounded completion")
            .expect("event");
        match event {
            SessionEvent::CompactStatus {
                operation_id: 10,
                state: CompactState::Completed { outcome, replayed },
                ..
            } => {
                assert_eq!(outcome, CompactOutcome::Cancelled);
                terminals.push(replayed);
            }
            SessionEvent::CompactStatus {
                state: CompactState::Started,
                ..
            } => panic!("cancelled queued compact must never run"),
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => {
                assert!(matches!(status, TurnCompletionStatus::Success { .. }));
                completed = true;
            }
            _ => {}
        }
    }
    assert_eq!(terminals, vec![false, true]);
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let history = task.await.expect("actor");
    assert!(history.iter().any(
        |message| matches!(message, Message::Assistant { content, .. }
        if content == "active turn survives compact cancellation")
    ));
}

#[tokio::test]
async fn manual_compact_runs_at_completed_tool_batch_before_turn_completion() {
    exercise_manual_boundary_hook_timeout(false, false, false).await;
}

#[tokio::test(start_paused = true)]
async fn hanging_manual_boundary_hook_times_out_without_retrying_observation() {
    exercise_manual_boundary_hook_timeout(true, false, false).await;
}

#[tokio::test(start_paused = true)]
async fn cancelling_incomplete_boundary_observation_does_not_retry_hook() {
    exercise_manual_boundary_hook_timeout(true, true, false).await;
}

#[tokio::test]
async fn denied_boundary_observation_is_not_retried_as_allowed() {
    exercise_manual_boundary_hook_timeout(false, false, true).await;
}

async fn exercise_manual_boundary_hook_timeout(hang: bool, cancel: bool, deny: bool) {
    use talos_core::session::{CompactOutcome, CompactState};
    struct BoundaryHook {
        calls: AtomicUsize,
        hang: bool,
        deny: bool,
        entered: Arc<tokio::sync::Notify>,
    }
    #[async_trait]
    impl talos_plugin::HookHandler for BoundaryHook {
        fn name(&self) -> &str {
            "boundary-hanging-hook"
        }
        fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
            &[talos_plugin::HookEventKind::BeforeProviderCall]
        }
        fn timeout(&self) -> Duration {
            Duration::from_secs(360)
        }
        async fn on_event(
            &self,
            _: &talos_plugin::HookContext,
            _: &mut talos_plugin::HookEvent<'_>,
        ) -> talos_plugin::HookResult {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 1 {
                if self.deny {
                    return talos_plugin::HookResult::Deny {
                        reason: "reject this continuation once".into(),
                    };
                }
                if self.hang {
                    self.entered.notify_one();
                    std::future::pending::<()>().await;
                }
            }
            talos_plugin::HookResult::Continue
        }
    }
    let hook_entered = Arc::new(tokio::sync::Notify::new());
    let mut hooks = talos_plugin::HookRegistry::new();
    let hook = Arc::new(BoundaryHook {
        calls: AtomicUsize::new(0),
        hang,
        deny,
        entered: hook_entered.clone(),
    });
    hooks.register(hook.clone());
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(BoundaryBlockingTool {
        entered: entered.clone(),
        release: release.clone(),
    }));
    let model = MockModel::new(vec![
        vec![
            AgentEvent::ToolCall {
                call: talos_core::message::ToolCall {
                    id: "boundary".into(),
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
        success_events("continued after manual boundary"),
    ]);
    let agent = Agent::with_security_and_hooks(
        Arc::new(model),
        tools,
        None,
        None,
        std::env::temp_dir(),
        Arc::new(hooks),
    );
    let (mut handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: std::env::temp_dir(),
            initial_history: vec![],
            model_context_limit: 128_000,
        },
    );
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "run tool".into(),
        })
        .await
        .expect("submit");
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .expect("tool entered");
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("compact");
    loop {
        if matches!(
            handle.eq_rx.recv().await,
            Some(SessionEvent::CompactStatus {
                state: CompactState::Queued,
                ..
            })
        ) {
            break;
        }
    }
    release.notify_one();
    if hang {
        hook_entered.notified().await;
        if cancel {
            handle
                .sq_tx
                .send(SessionOp::CancelCompact {
                    session_generation: 0,
                    operation_id: 1,
                })
                .await
                .expect("targeted cancellation");
        } else {
            tokio::time::advance(Duration::from_secs(121)).await;
        }
    }
    let mut compact_finished = false;
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), handle.eq_rx.recv())
            .await
            .expect("boundary deadline")
            .expect("event");
        match event {
            SessionEvent::CompactStatus {
                state:
                    CompactState::Completed {
                        outcome,
                        replayed: false,
                    },
                ..
            } => {
                if cancel {
                    assert_eq!(outcome, CompactOutcome::Cancelled);
                } else if hang || deny {
                    assert_eq!(outcome, CompactOutcome::Failed);
                } else {
                    assert!(matches!(outcome, CompactOutcome::NotEligible { .. }));
                }
                compact_finished = true;
            }
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => {
                assert!(
                    compact_finished,
                    "manual compaction must finish before ordinary continuation completes"
                );
                if hang || deny {
                    assert!(matches!(status, TurnCompletionStatus::Error { .. }));
                } else {
                    assert!(matches!(status, TurnCompletionStatus::Success { .. }));
                }
                break;
            }
            _ => {}
        }
    }
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let history = task.await.expect("actor");
    assert_eq!(
        hook.calls.load(Ordering::SeqCst),
        2,
        "failed observation must not be retried"
    );
    assert!(
        history
            .iter()
            .any(|message| matches!(message, Message::Tool { .. }))
    );
}

#[tokio::test]
async fn manual_boundary_applies_checkpoint_before_continuation_and_preserves_raw_history() {
    struct BoundaryHookCounter(Arc<AtomicUsize>);
    #[async_trait]
    impl talos_plugin::HookHandler for BoundaryHookCounter {
        fn name(&self) -> &str {
            "boundary-provider-counter"
        }
        fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
            &[talos_plugin::HookEventKind::BeforeProviderCall]
        }
        async fn on_event(
            &self,
            _: &talos_plugin::HookContext,
            _: &mut talos_plugin::HookEvent<'_>,
        ) -> talos_plugin::HookResult {
            self.0.fetch_add(1, Ordering::SeqCst);
            talos_plugin::HookResult::Continue
        }
    }
    let hook_calls = Arc::new(AtomicUsize::new(0));
    let mut hooks = talos_plugin::HookRegistry::new();
    hooks.register(Arc::new(BoundaryHookCounter(hook_calls.clone())));
    struct BoundarySummaryModel {
        ordinary: AtomicUsize,
        summaries: AtomicUsize,
    }
    #[async_trait]
    impl LanguageModel for BoundarySummaryModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let events = if let [Message::User { content }] = messages
                && content.starts_with("Produce only a JSON checkpoint")
            {
                self.summaries.fetch_add(1, Ordering::SeqCst);
                let sources: Vec<serde_json::Value> =
                    serde_json::from_str(content.split("\nSources: ").last().expect("sources"))
                        .expect("JSON");
                let item = serde_json::json!({"text":"retain original task", "sources":[sources[0]["source"]]});
                success_events(&serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
                    "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                    "facts":[],"assumptions":[],"resources":[],"open_loops":[]}).to_string())
            } else if self.ordinary.fetch_add(1, Ordering::SeqCst) == 0 {
                vec![
                    AgentEvent::ToolCall {
                        call: talos_core::message::ToolCall {
                            id: "boundary".into(),
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
            } else {
                assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.starts_with("[Derived checkpoint;"))));
                success_events("continued with checkpoint")
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("fixture event");
            }
            Ok(rx)
        }
    }
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let model = Arc::new(BoundarySummaryModel {
        ordinary: AtomicUsize::new(0),
        summaries: AtomicUsize::new(0),
    });
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BoundaryBlockingTool {
        entered: entered.clone(),
        release: release.clone(),
    }));
    let agent = Agent::with_security_and_hooks(
        model.clone(),
        registry,
        None,
        None,
        std::env::temp_dir(),
        Arc::new(hooks),
    );
    let raw: Vec<_> = (0..30)
        .flat_map(|index| {
            [
                Message::User {
                    content: format!("task {index}: {}", "x".repeat(4000)),
                },
                Message::Assistant {
                    content: "recorded".into(),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ]
        })
        .collect();
    let prepared = agent
        .prepare_session_turn_at_revision(
            &structured_submission("probe", "probe-item", 0, "run tool", SubmissionSource::User)
                .items,
            raw.clone(),
            u32::MAX,
            0,
            raw.len() as u64,
            None,
            None,
        )
        .await
        .expect("estimate initial request");
    let limit = prepared.initial_plan.estimated_tokens * 100 / 85;
    hook_calls.store(0, Ordering::SeqCst);
    let (mut handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: std::env::temp_dir(),
            initial_history: raw.clone(),
            model_context_limit: limit,
        },
    );
    let task = tokio::spawn(async move {
        actor.run().await;
        actor.history.clone()
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "run tool".into(),
        })
        .await
        .expect("submit");
    tokio::time::timeout(Duration::from_secs(5), entered.notified())
        .await
        .expect("tool boundary");
    assert_eq!(
        model.summaries.load(Ordering::SeqCst),
        0,
        "initial advisory pressure must not auto-compact"
    );
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("compact");
    loop {
        if matches!(
            handle.eq_rx.recv().await,
            Some(SessionEvent::CompactStatus {
                state: talos_core::session::CompactState::Queued,
                ..
            })
        ) {
            break;
        }
    }
    release.notify_one();
    let mut applied = false;
    loop {
        let event = tokio::time::timeout(Duration::from_secs(10), handle.eq_rx.recv())
            .await
            .expect("completion deadline")
            .expect("event");
        match event {
            SessionEvent::CompactStatus {
                state: talos_core::session::CompactState::Completed { outcome, .. },
                ..
            } => {
                assert!(
                    matches!(outcome, talos_core::session::CompactOutcome::Applied { before, after } if after < before && after <= limit),
                    "{outcome:?}"
                );
                applied = true;
            }
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => {
                assert!(
                    applied,
                    "checkpoint must be applied before continuation completes"
                );
                assert!(matches!(status, TurnCompletionStatus::Success { .. }));
                break;
            }
            _ => {}
        }
    }
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let history = task.await.expect("actor");
    assert_eq!(&history[..raw.len()], raw);
    assert_eq!(model.ordinary.load(Ordering::SeqCst), 2);
    assert_eq!(
        hook_calls.load(Ordering::SeqCst),
        2,
        "one observation per actual ordinary request, including manual boundary"
    );
    assert!(model.summaries.load(Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn manual_compact_with_durable_steering_preserves_cancelled_custody() {
    exercise_boundary_injection_with_gate(true, false, false, true, true).await;
}

#[tokio::test]
async fn failed_manual_compact_with_durable_steering_preserves_error_custody() {
    exercise_boundary_injection_with_gate(false, true, false, false, true).await;
}

#[tokio::test]
async fn recovered_actor_sources_are_session_scoped_and_clear_on_rebinding() {
    use talos_session::{PersistencePolicy, SessionManager};
    let directory = tempfile::tempdir().expect("directory");
    let manager = SessionManager::with_dir(directory.path().to_path_buf());
    let durable = manager
        .create_or_open_session("source-original")
        .expect("durable");
    let raw = vec![Message::User {
        content: "retained constraint".into(),
    }];
    durable
        .commit_turn("source-turn", &raw, &PersistencePolicy::default())
        .expect("commit");
    let entries = durable.read_history_snapshot().expect("recovery");
    let expected = format!("tlog:{}:{}", durable.id(), entries.entries()[0].0);
    #[allow(deprecated)]
    let agent = Agent::new(Arc::new(ToolCallThenErrorModel::new()), ToolRegistry::new());
    let (_handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: directory.path().to_path_buf(),
            initial_history: raw.clone(),
            model_context_limit: 128_000,
        },
    );
    actor.set_durable_persistence(durable.clone(), PersistencePolicy::default());
    let reopened =
        talos_session::DurableSession::open_existing(directory.path(), "source-original")
            .expect("open")
            .expect("existing");
    let reopened_snapshot = reopened.read_history_snapshot().expect("reopened snapshot");
    assert_eq!(reopened_snapshot.session_id(), entries.session_id());
    assert_eq!(reopened_snapshot.entries(), entries.entries());
    actor.set_recovered_history_sources(reopened_snapshot.clone());
    assert!(
        actor.recovered_source.is_none(),
        "equal IDs, entries and path are not configured-owner provenance"
    );
    actor.set_durable_persistence(reopened, PersistencePolicy::default());
    actor.set_recovered_history_sources(reopened_snapshot);
    assert!(
        actor.recovered_source.is_some(),
        "restart works when snapshot and configured owner agree"
    );
    actor.set_durable_persistence(durable, PersistencePolicy::default());
    let foreign = manager
        .create_or_open_session("source-foreign")
        .expect("foreign");
    foreign
        .commit_turn("source-turn", &raw, &PersistencePolicy::default())
        .expect("same contents in foreign session");
    actor.set_recovered_history_sources(foreign.read_history_snapshot().expect("foreign snapshot"));
    assert!(
        actor.recovered_source.is_none(),
        "same bytes do not authenticate foreign entry IDs"
    );
    actor.set_recovered_history_sources(entries.clone());
    assert_eq!(
        actor
            .recovered_source
            .as_ref()
            .expect("source")
            .resolve(0, 1, 0, &raw[0]),
        Some(expected.as_str())
    );
    actor.history[0] = Message::User {
        content: "replacement".into(),
    };
    actor.set_recovered_history_sources(entries.clone());
    assert!(actor.recovered_source.is_none());
    actor.history = raw;
    actor.set_recovered_history_sources(entries);
    assert!(actor.recovered_source.is_some());
    let other = manager
        .create_or_open_session("source-other")
        .expect("other session");
    actor.set_durable_persistence(other, PersistencePolicy::default());
    assert!(actor.recovered_source.is_none());
}

#[tokio::test]
async fn durable_turn_receipts_bind_live_actor_history_for_success_error_and_cancel() {
    use talos_session::{PersistencePolicy, SessionManager};
    for outcome in ["success", "error", "cancel"] {
        let directory = tempfile::tempdir().expect("directory");
        let manager = SessionManager::with_dir(directory.path().to_path_buf());
        let durable = manager.create_or_open_session(outcome).expect("durable");
        let model: Arc<dyn LanguageModel> = match outcome {
            "success" => Arc::new(MockModel::new(vec![vec![
                AgentEvent::TextDelta {
                    delta: "retained result".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Default::default(),
                },
            ]])),
            "error" => Arc::new(ToolCallThenErrorModel::new()),
            _ => Arc::new(ToolCallThenBlockingModel {
                call_count: Arc::new(std::sync::atomic::AtomicU8::new(0)),
            }),
        };
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool));
        #[allow(deprecated)]
        let agent = Agent::new(model, registry);
        let (mut handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: directory.path().to_path_buf(),
                initial_history: vec![],
                model_context_limit: 128_000,
            },
        );
        actor.set_durable_persistence(durable.clone(), PersistencePolicy::default());
        let task = tokio::spawn(async move {
            actor.run().await;
            actor
        });
        handle
            .sq_tx
            .send(SessionOp::Submit {
                message: "retain this constraint".into(),
            })
            .await
            .expect("submit");
        let mut cancelled = false;
        loop {
            let event = tokio::time::timeout(Duration::from_secs(10), handle.eq_rx.recv())
                .await
                .expect("event deadline")
                .expect("event");
            if outcome == "cancel"
                && !cancelled
                && matches!(progress_event(&event), Some(AgentEvent::ToolResult { .. }))
            {
                handle
                    .sq_tx
                    .send(SessionOp::Interrupt)
                    .await
                    .expect("interrupt");
                cancelled = true;
            }
            if let Some(status) = completed_status(&event) {
                assert!(
                    matches!(
                        (outcome, status),
                        ("success", TurnCompletionStatus::Success { .. })
                            | ("error", TurnCompletionStatus::Error { .. })
                            | ("cancel", TurnCompletionStatus::Cancelled)
                    ),
                    "{outcome}: {status:?}"
                );
                break;
            }
        }
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        let actor = task.await.expect("actor");
        let recovered = durable.read_messages_with_entry_ids().expect("recovery");
        assert!(!recovered.is_empty());
        assert_eq!(actor.history.len(), recovered.len(), "{outcome}");
        let source = actor.recovered_source.as_ref().expect("committed source");
        for (index, ((id, _), message)) in recovered.iter().zip(&actor.history).enumerate() {
            let expected = format!("tlog:{}:{id}", durable.id());
            assert_eq!(
                source.resolve(
                    actor.session_generation,
                    actor.history.len() as u64,
                    index,
                    message
                ),
                Some(expected.as_str()),
                "{outcome}, entry {index}"
            );
        }
    }
}

#[tokio::test]
async fn automatic_checkpoint_reports_terminal_outcomes_without_transcript_messages() {
    use talos_core::session::AutomaticCompactState;
    struct ProgressModel {
        invalid: bool,
        block: bool,
        entered: tokio::sync::Notify,
    }
    #[async_trait]
    impl LanguageModel for ProgressModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let events = if let [Message::User { content }] = messages
                && content.starts_with("Produce only a JSON checkpoint")
            {
                self.entered.notify_one();
                if self.block {
                    return std::future::pending().await;
                }
                if self.invalid {
                    success_events("invalid checkpoint")
                } else {
                    let sources: Vec<serde_json::Value> =
                        serde_json::from_str(content.split("\nSources: ").last().expect("sources"))
                            .expect("source JSON");
                    let item = serde_json::json!({"text":"retained task", "sources":[sources[0]["source"]]});
                    success_events(&serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
                        "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                        "facts":[],"assumptions":[],"resources":[],"open_loops":[]}).to_string())
                }
            } else {
                success_events("ordinary reply")
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("event");
            }
            Ok(rx)
        }
    }
    for outcome in ["applied", "failed", "cancelled"] {
        let directory = tempfile::tempdir().expect("directory");
        let model = Arc::new(ProgressModel {
            invalid: outcome == "failed",
            block: outcome == "cancelled",
            entered: tokio::sync::Notify::new(),
        });
        #[allow(deprecated)]
        let agent = Agent::new(model.clone(), ToolRegistry::new());
        let raw: Vec<_> = (0..20)
            .flat_map(|index| {
                [
                    Message::User {
                        content: format!("original constraint {index}: {}", "a".repeat(4000)),
                    },
                    Message::Assistant {
                        content: "ack".into(),
                        tool_calls: vec![],
                        reasoning: None,
                    },
                ]
            })
            .collect();
        let probe = agent
            .prepare_turn_start(
                "continue".into(),
                vec![Message::User {
                    content: "continue".into(),
                }],
                raw.clone(),
                Some(u32::MAX),
            )
            .await
            .expect("probe");
        let limit = probe.initial_plan.estimated_tokens * 100 / 96;
        let (mut handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: directory.path().to_path_buf(),
                initial_history: raw.clone(),
                model_context_limit: limit,
            },
        );
        let control = RuntimeAdmissionControl::new();
        actor.set_runtime_admission(control.clone());
        let task = tokio::spawn(async move {
            actor.run().await;
            actor
        });
        handle
            .sq_tx
            .send(SessionOp::Submit {
                message: "continue".into(),
            })
            .await
            .expect("submit");
        let mut states = vec![];
        loop {
            let event = tokio::time::timeout(Duration::from_secs(10), handle.eq_rx.recv())
                .await
                .expect("deadline")
                .expect("event");
            if let SessionEvent::AutomaticCompactStatus { state, .. } = &event {
                states.push(*state);
                if outcome == "cancelled" && matches!(state, AutomaticCompactState::Started { .. })
                {
                    tokio::time::timeout(Duration::from_secs(5), model.entered.notified())
                        .await
                        .expect("summary started");
                    control.interrupt_active();
                }
            }
            if let Some(status) = completed_status(&event) {
                assert!(
                    matches!(
                        (outcome, status),
                        ("cancelled", TurnCompletionStatus::Cancelled)
                            | ("applied" | "failed", TurnCompletionStatus::Success { .. })
                    ),
                    "{outcome}: {status:?}"
                );
                break;
            }
        }
        handle
            .sq_tx
            .send(SessionOp::Shutdown)
            .await
            .expect("shutdown");
        let actor = task.await.expect("actor");
        assert_eq!(states.len(), 2, "one start and one terminal: {states:?}");
        let AutomaticCompactState::Started { before } = states[0] else {
            panic!("start first");
        };
        assert!(before > 0);
        match (outcome, states[1]) {
            (
                "applied",
                AutomaticCompactState::Applied {
                    before: prior,
                    after,
                },
            ) => {
                assert_eq!(prior, before);
                assert!(after < before && after <= limit);
            }
            ("failed", AutomaticCompactState::Failed { before: prior, .. })
            | ("cancelled", AutomaticCompactState::Cancelled { before: prior }) => {
                assert_eq!(prior, before)
            }
            _ => panic!("unexpected outcome: {outcome}, {states:?}"),
        }
        assert_eq!(&actor.history[..raw.len()], raw);
        assert!(
            !serde_json::to_string(&actor.history)
                .expect("history")
                .contains("Automatic compaction")
        );
        if outcome != "applied" {
            assert!(
                actor
                    .agent
                    .checkpoint_projection_state
                    .lock()
                    .expect("cache")
                    .is_none()
            );
        }
    }
}

#[path = "checkpoint_acceptance_tests.rs"]
mod checkpoint_acceptance;

fn assert_recovery_receipts_verify(durable: &talos_session::DurableSession, expected: usize) {
    let entries = durable
        .session()
        .read_entries()
        .expect("read receipt source");
    let receipts = entries
        .iter()
        .filter_map(talos_session::HistoryRecoveryReceipt::from_terminal_entry)
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(receipts.len(), expected);
    for receipt in receipts {
        let entry = entries
            .iter()
            .find(|entry| entry.id == receipt.entry_id())
            .expect("original entry");
        assert!(receipt.verified_content(entry).is_some());
    }
}

#[tokio::test]
async fn registered_model_checkpoint_discovers_admits_and_applies_through_actor() {
    use talos_core::tool::ToolNature;
    use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
    struct CheckpointModel {
        step: AtomicUsize,
    }
    #[async_trait]
    impl LanguageModel for CheckpointModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let step = self.step.fetch_add(1, Ordering::SeqCst);
            let events = match step {
                0 | 1 => {
                    let (name, input) = if step == 0 {
                        ("history_search", serde_json::json!({"mode":"discover"}))
                    } else {
                        let result = messages
                            .iter()
                            .rev()
                            .find_map(|message| match message {
                                Message::Tool { result }
                                    if result.tool_use_id == "checkpoint-0" =>
                                {
                                    Some(result)
                                }
                                _ => None,
                            })
                            .expect("discovery result in real continuation");
                        assert!(!result.is_error, "{}", result.content);
                        let discovery: serde_json::Value =
                            serde_json::from_str(&result.content).expect("discovery JSON");
                        let item = serde_json::json!({"text":"Keep the constraint and proceed", "sources":[discovery["sources"][0]]});
                        (
                            "compact_context",
                            serde_json::json!({"snapshot":discovery["snapshot"], "checkpoint": {
                                "schema_version":1,"summary":item,"goal":item,"next_step":item,
                                "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                                "facts":[],"assumptions":[],"resources":[],"open_loops":[]
                            }}),
                        )
                    };
                    vec![
                        AgentEvent::ToolCall {
                            call: talos_core::message::ToolCall {
                                id: format!("checkpoint-{step}"),
                                name: name.into(),
                                input,
                            },
                            provenance: talos_core::tool::ToolProvenance::Native,
                            summary_fields: vec![],
                        },
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::ToolUse,
                            usage: Default::default(),
                        },
                    ]
                }
                2 => {
                    let request = serde_json::to_string(messages).expect("request");
                    assert!(
                        request.contains("Admitted"),
                        "admission must remain in paired batch: {request}"
                    );
                    let terminal = messages
                        .iter()
                        .find_map(|message| match message {
                            Message::Multimodal { parts } => {
                                parts.iter().find_map(|part| match part {
                                    talos_core::message::ContentPart::Text { text } => {
                                        serde_json::from_str::<serde_json::Value>(text).ok().filter(
                                            |value| value.get("checkpoint_operation").is_some(),
                                        )
                                    }
                                    _ => None,
                                })
                            }
                            _ => None,
                        })
                        .expect("actual terminal metadata, not tool description");
                    assert_eq!(terminal["checkpoint_operation"], 0);
                    assert!(
                        terminal["outcome"]
                            .as_str()
                            .expect("outcome")
                            .starts_with("Applied {"),
                        "{terminal}"
                    );
                    success_events("checkpoint complete")
                }
                _ => panic!("unexpected extra provider request"),
            };
            let (tx, rx) = mpsc::channel(16);
            for event in events {
                tx.send(event).await.expect("receiver");
            }
            Ok(rx)
        }
    }
    let root = tempfile::tempdir().expect("workspace");
    let mut engine = PermissionEngine::empty();
    engine.set_workspace_root(root.path().into());
    for nature in [ToolNature::Read, ToolNature::Write] {
        engine.add_rule(PermissionRule::new_nature(
            nature,
            None,
            None,
            PermissionDecision::Allow,
        ));
    }
    let raw: Vec<_> = (0..30)
        .flat_map(|_| {
            [
                Message::User {
                    content: "constraint ".repeat(400),
                },
                Message::Assistant {
                    content: "recorded".into(),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ]
        })
        .collect();
    let mut agent = Agent::with_security_and_hooks(
        Arc::new(CheckpointModel {
            step: AtomicUsize::new(0),
        }),
        ToolRegistry::new(),
        Some(Arc::new(engine)),
        None,
        root.path().into(),
        Arc::new(talos_plugin::HookRegistry::new()),
    );
    agent
        .enable_model_checkpoint_tools()
        .expect("enable model tools");
    let (mut handle, mut actor) = AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: root.path().into(),
            initial_history: raw.clone(),
            model_context_limit: 128_000,
        },
    );
    let task = tokio::spawn(async move {
        actor.run().await;
        actor
    });
    handle
        .sq_tx
        .send(SessionOp::Submit {
            message: "checkpoint and continue".into(),
        })
        .await
        .expect("submit");
    let captured = tokio::time::timeout(
        Duration::from_secs(10),
        collect_until_completions(&mut handle.eq_rx, 1),
    )
    .await
    .expect("no circular admission/batch wait");
    assert!(captured.iter().any(|event| matches!(
        completed_status(event),
        Some(TurnCompletionStatus::Success { .. })
    )));
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let actor = task.await.expect("actor");
    assert_eq!(&actor.history[..raw.len()], raw.as_slice());
    assert!(
        actor
            .agent
            .checkpoint_projection_state
            .lock()
            .expect("projection")
            .is_some()
    );
}

#[test]
fn model_checkpoint_enable_reports_collision_without_partial_registration() {
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(crate::checkpoint_tools::HistorySearchTool));
    let mut agent = Agent::new(Arc::new(MockModel::new(vec![])), tools);
    assert!(agent.enable_model_checkpoint_tools().is_err());
    assert!(agent.tools.get("history_search").is_some());
    assert!(!agent.tools.is_builtin_instance("history_search"));
    assert!(agent.tools.get("compact_context").is_none());
}
