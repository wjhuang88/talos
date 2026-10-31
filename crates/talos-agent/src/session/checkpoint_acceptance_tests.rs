//! End-to-end deterministic semantic-retention fixtures (not model-quality benchmarks).
use super::*;
use serde_json::{Value, json};
use talos_core::tool::{AgentTool, ToolResult};
use talos_session::{PersistencePolicy, SessionManager};

const LIMIT: u32 = 24_000;
const MARKERS: [(&str, &str); 11] = [
    ("goal", "DELIVER_OFFLINE_REPORT"),
    ("constraints", "NEVER_SEND_NETWORK"),
    ("decisions", "USE_LOCAL_CACHE"),
    ("open_loops", "FAILED_REMOTE_ROUTE"),
    ("pending", "VERIFY_FINAL_TOTAL"),
    ("next_step", "RUN_LOCAL_CHECK"),
    ("completed", "BASELINE_SCAN_DONE"),
    ("in_progress", "LOCAL_RECOUNT_ACTIVE"),
    ("facts", "LOCAL_COUNT_CONFIRMED_42"),
    ("assumptions", "MAY_REQUIRE_RECOUNT"),
    ("resources", "REPORT_PATH_LOCAL"),
];

struct RetentionTool(Arc<AtomicUsize>);
#[async_trait]
impl AgentTool for RetentionTool {
    fn name(&self) -> &str {
        "retention_probe"
    }
    fn description(&self) -> &str {
        "Return bounded local fixture evidence"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{"turn":{"type":"integer"}},"required":["turn"]})
    }
    async fn execute(&self, input: Value) -> ToolResult {
        self.0.fetch_add(1, Ordering::SeqCst);
        let turn = input["turn"].as_u64().expect("turn");
        let content = format!(
            "{} PROGRESS_{turn} {}",
            if turn == 0 {
                "FAILED_REMOTE_ROUTE TOOL_ONLY_FAILURE"
            } else {
                "LOCAL_PROBE_OK"
            },
            "e".repeat(3000)
        );
        if turn == 0 {
            ToolResult::error(content)
        } else {
            ToolResult::success(content)
        }
    }
}

struct RetentionModel {
    next_turn: AtomicUsize,
    summary_calls: AtomicUsize,
    checkpoint_dispatches: AtomicUsize,
    segmented_sources: AtomicUsize,
    require_tool_failure: bool,
}
impl RetentionModel {
    fn new(next_turn: usize, require_tool_failure: bool) -> Self {
        Self {
            next_turn: AtomicUsize::new(next_turn),
            summary_calls: AtomicUsize::new(0),
            checkpoint_dispatches: AtomicUsize::new(0),
            segmented_sources: AtomicUsize::new(0),
            require_tool_failure,
        }
    }
    fn summarize(&self, content: &str) -> String {
        self.summary_calls.fetch_add(1, Ordering::SeqCst);
        let (previous, sources) = content
            .split_once("Prior checkpoint: ")
            .expect("prompt contract")
            .1
            .split_once("\nSources: ")
            .expect("sources");
        let sources: Vec<Value> = serde_json::from_str(sources).expect("source JSON");
        let fallback =
            json!({"text":"Await supplied task evidence", "sources":[sources[0]["source"]]});
        let mut checkpoint: Value = if previous.is_empty() {
            json!({"schema_version":1,"summary":fallback,"goal":fallback,"next_step":fallback,
                "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
        } else {
            serde_json::from_str(previous).expect("provisional checkpoint")
        };
        // Extract only facts actually present in this source chunk. Later chunks
        // must carry the prior checkpoint; hardcoded fixture output cannot pass.
        for source in &sources {
            if source.get("byte_start").is_some() || source.get("projection_byte_start").is_some() {
                self.segmented_sources.fetch_add(1, Ordering::SeqCst);
            }
            let text = serde_json::to_string(source).expect("source");
            for (field, marker) in MARKERS {
                if text.contains(marker) {
                    let item = json!({"text":marker, "sources":[source["source"]]});
                    checkpoint[field] = if matches!(field, "goal" | "next_step") {
                        item
                    } else {
                        json!([item])
                    };
                }
            }
            if text.contains("TOOL_ONLY_FAILURE") {
                let failures = checkpoint["open_loops"].as_array_mut().expect("open loops");
                if !failures
                    .iter()
                    .any(|item| item["text"] == "TOOL_ONLY_FAILURE")
                {
                    failures
                        .push(json!({"text":"TOOL_ONLY_FAILURE", "sources":[source["source"]]}));
                }
            }
        }
        checkpoint["summary"] = checkpoint["goal"].clone();
        serde_json::to_string(&checkpoint).expect("checkpoint")
    }
}
#[async_trait]
impl LanguageModel for RetentionModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let events = if let [Message::User { content }] = messages
            && content.starts_with("Produce only a JSON checkpoint")
        {
            success_events(&self.summarize(content))
        } else {
            let mut pending = std::collections::HashSet::new();
            for message in messages {
                match message {
                    Message::Assistant { tool_calls, .. } => {
                        assert!(pending.is_empty(), "no incomplete prior batch");
                        pending.extend(tool_calls.iter().map(|call| call.id.clone()));
                    }
                    Message::Tool { result } => {
                        assert!(pending.remove(&result.tool_use_id), "paired tool result");
                    }
                    Message::User { .. } => assert!(pending.is_empty()),
                    _ => {}
                }
                if let Message::Context { content } = message
                    && let Some(checkpoint) = content.strip_prefix(
                        "[Derived checkpoint; source history remains authoritative]\n",
                    )
                {
                    let checkpoint: Value =
                        serde_json::from_str(checkpoint).expect("checkpoint JSON");
                    for (field, marker) in MARKERS {
                        assert!(
                            checkpoint[field].to_string().contains(marker),
                            "lost {field}: {checkpoint}"
                        );
                    }
                    assert_eq!(
                        checkpoint["open_loops"]
                            .to_string()
                            .contains("TOOL_ONLY_FAILURE"),
                        self.require_tool_failure,
                        "live verified tool failures are retained; unverified recovered payloads are excluded"
                    );
                    self.checkpoint_dispatches.fetch_add(1, Ordering::SeqCst);
                }
            }
            assert!(pending.is_empty());
            if matches!(messages.last(), Some(Message::Tool { .. })) {
                success_events("Local evidence recorded; final verification remains pending")
            } else {
                let turn = self.next_turn.fetch_add(1, Ordering::SeqCst);
                vec![
                    AgentEvent::ToolCall {
                        call: talos_core::message::ToolCall {
                            id: format!("retention-{turn}"),
                            name: "retention_probe".into(),
                            input: json!({"turn":turn}),
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
        };
        let (tx, rx) = mpsc::channel(8);
        for event in events {
            tx.send(event).await.expect("fixture stream");
        }
        Ok(rx)
    }
}

fn actor_for_history(
    model: Arc<RetentionModel>,
    calls: Arc<AtomicUsize>,
    raw: Vec<Message>,
    root: &std::path::Path,
) -> (SessionHandle, AppServerSession) {
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(RetentionTool(calls)));
    #[allow(deprecated)]
    let agent = Agent::new(model, tools);
    AppServerSession::new(
        agent,
        SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: root.to_path_buf(),
            initial_history: raw,
            model_context_limit: LIMIT,
        },
    )
}

async fn complete_turn(handle: &mut SessionHandle, text: String) -> usize {
    handle
        .sq_tx
        .send(SessionOp::Submit { message: text })
        .await
        .expect("submit");
    let mut applied = 0;
    loop {
        let event = tokio::time::timeout(Duration::from_secs(20), handle.eq_rx.recv())
            .await
            .expect("turn deadline")
            .expect("event");
        if let Some(AgentEvent::ContextBudget { budget }) = progress_event(&event) {
            assert!(
                budget.estimated_tokens <= LIMIT,
                "ordinary dispatch must fit"
            );
        }
        if matches!(
            &event,
            SessionEvent::AutomaticCompactStatus {
                state: talos_core::session::AutomaticCompactState::Applied { .. },
                ..
            }
        ) {
            applied += 1;
        }
        if let Some(status) = completed_status(&event) {
            assert!(
                matches!(status, TurnCompletionStatus::Success { .. }),
                "{status:?}"
            );
            return applied;
        }
    }
}

#[tokio::test]
async fn fifty_tool_turns_preserve_semantics_raw_export_and_durable_restart() {
    let directory = tempfile::tempdir().expect("directory");
    let manager = SessionManager::with_dir(directory.path().to_path_buf());
    let durable = manager
        .create_or_open_session("checkpoint-retention")
        .expect("session");
    let calls = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(RetentionModel::new(0, true));
    let (mut handle, mut actor) =
        actor_for_history(model.clone(), calls.clone(), vec![], directory.path());
    actor.set_durable_persistence(durable.clone(), PersistencePolicy::default());
    let task = tokio::spawn(async move {
        actor.run().await;
        actor
    });
    let mut applied = 0;
    for turn in 0..50 {
        let text = if turn == 0 {
            format!(
                "{} {}",
                MARKERS
                    .iter()
                    .map(|(_, marker)| *marker)
                    .collect::<Vec<_>>()
                    .join(" "),
                "u".repeat(1500)
            )
        } else {
            format!("Continue local step {turn}; {}", "u".repeat(1500))
        };
        applied += complete_turn(&mut handle, text).await;
    }
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let actor = task.await.expect("actor");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        50,
        "no tool replay for compaction"
    );
    assert!(applied >= 2, "repeated checkpoints required, got {applied}");
    assert!(
        model.summary_calls.load(Ordering::SeqCst) > applied,
        "multi-chunk folding required"
    );
    assert!(model.checkpoint_dispatches.load(Ordering::SeqCst) > 0);
    let raw = durable.read_messages().expect("raw recovery");
    assert_eq!(raw, actor.history);
    assert_eq!(
        raw.iter()
            .filter(|message| matches!(message, Message::User { .. }))
            .count(),
        50
    );
    assert!(
        !serde_json::to_string(&raw)
            .expect("raw JSON")
            .contains("[Derived checkpoint;")
    );
    {
        let cached = actor
            .agent
            .checkpoint_projection_state
            .lock()
            .expect("cache");
        let projection = cached.as_ref().expect("repeated projection");
        assert!(projection.sequence >= 2);
        assert!(
            projection
                .parent_sequence
                .is_some_and(|parent| parent < projection.sequence)
        );
        let mapped = durable.read_messages_with_entry_ids().expect("durable IDs");
        for &index in &projection.covered {
            let expected = format!("tlog:{}:{}", durable.id(), mapped[index].0);
            assert_eq!(
                projection.source.resolve(
                    projection.source.generation(),
                    projection.source.revision(),
                    index,
                    &raw[index]
                ),
                Some(expected.as_str())
            );
        }
    }
    let entries = durable.session().read_entries().expect("entries");
    let exported = talos_session::export_json(&entries).expect("export");
    for (_, marker) in MARKERS {
        assert!(exported.contains(marker));
    }
    let saved = durable.session().snapshot_bytes().expect("raw bytes");
    drop(actor);
    let reopened = manager
        .get_session_by_external_id("checkpoint-retention")
        .expect("lookup")
        .expect("reopen");
    assert_eq!(reopened.session().snapshot_bytes().expect("bytes"), saved);
    let model = Arc::new(RetentionModel::new(50, false));
    let (mut handle, mut actor) =
        actor_for_history(model.clone(), calls.clone(), raw.clone(), directory.path());
    actor.set_durable_persistence(reopened.clone(), PersistencePolicy::default());
    actor.set_recovered_history_sources(reopened.read_history_snapshot().expect("snapshot"));
    assert!(
        actor
            .agent
            .checkpoint_projection_state
            .lock()
            .expect("cache")
            .is_none()
    );
    let task = tokio::spawn(async move {
        actor.run().await;
        actor
    });
    assert!(complete_turn(&mut handle, "Resume local verification".into()).await > 0);
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let actor = task.await.expect("actor");
    assert_eq!(&actor.history[..raw.len()], raw);
    assert_eq!(calls.load(Ordering::SeqCst), 51);
    assert!(model.checkpoint_dispatches.load(Ordering::SeqCst) > 0);
    assert!(model.summary_calls.load(Ordering::SeqCst) > 1);
    let cached = actor
        .agent
        .checkpoint_projection_state
        .lock()
        .expect("reconstructed cache");
    let projection = cached.as_ref().expect("reconstructed projection");
    assert!(
        projection.parent_sequence.is_none(),
        "restart must not invent an in-memory predecessor"
    );
    let mapped = reopened
        .read_messages_with_entry_ids()
        .expect("recovered IDs");
    for &index in &projection.covered {
        let expected = format!("tlog:{}:{}", reopened.id(), mapped[index].0);
        assert_eq!(
            projection.source.resolve(
                projection.source.generation(),
                projection.source.revision(),
                index,
                &actor.history[index]
            ),
            Some(expected.as_str())
        );
    }
}

#[tokio::test]
async fn text_heavy_restart_manual_fold_preserves_early_semantics_and_raw_bytes() {
    text_heavy_manual_fold(false, false).await;
}

#[tokio::test]
async fn text_heavy_fork_manual_fold_rebinds_sources_without_mutating_parent() {
    text_heavy_manual_fold(true, false).await;
}

#[tokio::test]
async fn cli_session_persistence_compacts_recovered_history_with_content_bound_sources() {
    text_heavy_manual_fold(false, true).await;
}

async fn text_heavy_manual_fold(fork: bool, cli_persistence: bool) {
    use talos_core::session::{CompactOutcome, CompactState};
    let directory = tempfile::tempdir().expect("directory");
    let manager = SessionManager::with_dir(directory.path().to_path_buf());
    let durable = manager
        .create_or_open_session("text-restart")
        .expect("session");
    for index in 0..20 {
        let content = if index == 0 {
            format!(
                "{} {}",
                MARKERS
                    .iter()
                    .map(|(_, marker)| *marker)
                    .collect::<Vec<_>>()
                    .join(" "),
                "界".repeat(40_000)
            )
        } else {
            format!("Local step {index}: {}", "s".repeat(1000))
        };
        let pair = [
            Message::User { content },
            Message::Assistant {
                content: "recorded".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ];
        durable
            .commit_turn(
                &format!("seed-{index}"),
                &pair,
                &PersistencePolicy::default(),
            )
            .expect("seed durable history");
    }
    let saved = durable.session().snapshot_bytes().expect("raw bytes");
    let external_id = if fork {
        let child = manager
            .create_or_open_session("text-fork")
            .expect("fork destination");
        assert_ne!(child.id(), durable.id());
        std::fs::write(child.file_path(), &saved).expect("copy raw fork history");
        "text-fork"
    } else {
        "text-restart"
    };
    let reopened = manager
        .get_session_by_external_id(external_id)
        .expect("lookup")
        .expect("reopen");
    let snapshot = reopened.read_history_snapshot().expect("snapshot");
    let raw = snapshot
        .entries()
        .iter()
        .map(|(_, message)| message.clone())
        .collect::<Vec<_>>();
    let calls = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(RetentionModel::new(20, false));
    let (mut handle, mut actor) =
        actor_for_history(model.clone(), calls.clone(), raw.clone(), directory.path());
    if cli_persistence {
        // CLI/TUI use this Session persistence path, without an atomic durable
        // receipt. Its source IDs must stay explicitly content-bound.
        actor.set_persistence(reopened.session().clone(), Default::default());
    } else {
        actor.set_durable_persistence(reopened.clone(), PersistencePolicy::default());
        actor.set_recovered_history_sources(snapshot);
    }
    let task = tokio::spawn(async move {
        actor.run().await;
        actor
    });
    handle
        .sq_tx
        .send(SessionOp::Compact {
            session_generation: 0,
            operation_id: 1,
        })
        .await
        .expect("manual compact");
    loop {
        let event = tokio::time::timeout(Duration::from_secs(20), handle.eq_rx.recv())
            .await
            .expect("compact deadline")
            .expect("event");
        if let SessionEvent::CompactStatus {
            state:
                CompactState::Completed {
                    outcome,
                    replayed: false,
                },
            ..
        } = event
        {
            assert!(
                matches!(outcome, CompactOutcome::Applied { before, after } if before > LIMIT && after <= LIMIT),
                "{outcome:?}"
            );
            break;
        }
    }
    assert_eq!(
        reopened
            .session()
            .snapshot_bytes()
            .expect("unchanged bytes"),
        saved
    );
    assert!(
        model.segmented_sources.load(Ordering::SeqCst) > 1,
        "oversized single text must be segmented"
    );
    assert!(model.summary_calls.load(Ordering::SeqCst) > 1);
    complete_turn(&mut handle, "Continue the local report".into()).await;
    handle
        .sq_tx
        .send(SessionOp::Shutdown)
        .await
        .expect("shutdown");
    let actor = task.await.expect("actor");
    assert_eq!(&actor.history[..raw.len()], raw);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(model.checkpoint_dispatches.load(Ordering::SeqCst) > 0);
    let cache = actor
        .agent
        .checkpoint_projection_state
        .lock()
        .expect("cache");
    let projection = cache.as_ref().expect("manual projection");
    let mapped = reopened.read_messages_with_entry_ids().expect("entry IDs");
    for &index in &projection.covered {
        let expected = if cli_persistence {
            crate::compaction::HistorySource::stable_entry_id(0, index, &raw[index])
                .expect("content identity")
        } else {
            format!("tlog:{}:{}", reopened.id(), mapped[index].0)
        };
        assert_eq!(
            projection.source.resolve(
                projection.source.generation(),
                projection.source.revision(),
                index,
                &raw[index],
            ),
            Some(expected.as_str()),
        );
    }
    if fork {
        assert_eq!(
            durable.session().snapshot_bytes().expect("parent bytes"),
            saved
        );
        assert_eq!(durable.read_messages().expect("parent history"), raw);
    }
}
