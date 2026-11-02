//! Deterministic Runtime composition over a real coding artifact, not model quality.
use super::*;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use talos_core::message::{AgentEvent, Message, StopReason};
use talos_core::provider::{LanguageModel, ProviderResult};
use talos_core::tool::{AgentTool, ToolNature, ToolResult};

struct CodingTool {
    path: PathBuf,
    repair: bool,
    calls: Arc<AtomicUsize>,
}
#[async_trait]
impl AgentTool for CodingTool {
    fn name(&self) -> &str {
        if self.repair {
            "repair_fixture"
        } else {
            "verify_fixture"
        }
    }
    fn description(&self) -> &str {
        "Operate on the local coding acceptance fixture"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object"})
    }
    fn is_read_only(&self) -> bool {
        !self.repair
    }
    fn recovery_projection(&self, result: &ToolResult) -> Option<String> {
        Some(result.content.clone())
    }
    async fn execute(&self, _: Value) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let source = tokio::fs::read_to_string(&self.path).await.expect("source");
        if self.repair {
            assert!(
                !source.contains("return a - b"),
                "known failed patch must not match"
            );
            assert!(source.contains("return a + a"));
            tokio::fs::write(&self.path, source.replace("return a + a", "return a + b"))
                .await
                .expect("repair");
            ToolResult::success(
                "codingfact repaired add.ts; subtraction patch did not match; pending verify addition; assumption deployment unverified",
            )
        } else {
            assert_eq!(
                source,
                "export function add(a: number, b: number) { return a + b; }\n"
            );
            ToolResult::success("addition source verified; deployment remains unverified")
        }
    }
}
fn call(id: &str, name: &str, input: Value) -> Vec<AgentEvent> {
    vec![
        AgentEvent::ToolCall {
            call: talos_core::message::ToolCall {
                id: id.into(),
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
fn result(messages: &[Message], id: &str) -> Value {
    let result = messages
        .iter()
        .rev()
        .find_map(|message| match message {
            Message::Tool { result } if result.tool_use_id == id => Some(result),
            _ => None,
        })
        .expect("real tool result");
    assert!(!result.is_error, "{id}: {}", result.content);
    serde_json::from_str(&result.content).expect("JSON result")
}
struct CodingModel {
    rejected_probe: AtomicUsize,
    installed_a: std::sync::Mutex<Option<String>>,
    step: AtomicUsize,
    recovering: bool,
}
#[async_trait]
impl LanguageModel for CodingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let step = self.step.fetch_add(1, Ordering::SeqCst);
        if !self.recovering && matches!(step, 2 | 4) {
            let actual = messages
                .iter()
                .find_map(|message| match message {
                    Message::Context { content } if content.starts_with("[Derived checkpoint;") => {
                        Some(content)
                    }
                    _ => None,
                })
                .expect("registered checkpoint installed");
            // Runtime resolves model aliases to authoritative source identities before
            // rendering. Test canonical bytes at that installed boundary.
            let payload = actual
                .strip_prefix("[Derived checkpoint; source history remains authoritative]\n")
                .expect("projection envelope");
            let parsed = talos_agent::compaction::Checkpoint::parse(payload, |_| true)
                .expect("installed schema");
            assert_eq!(
                parsed.summary.text,
                if step == 2 {
                    "coding checkpoint A"
                } else {
                    "coding checkpoint B"
                }
            );
            assert_eq!(parsed.render(1_000_000).expect("render installed"), payload);
            let reordered: Value = serde_json::from_str(payload).expect("reordered object");
            let roundtrip = talos_agent::compaction::Checkpoint::parse(
                &serde_json::to_string(&reordered).expect("serialize object"),
                |_| true,
            )
            .expect("same semantic input");
            assert_eq!(
                roundtrip.render(1_000_000).expect("render roundtrip"),
                payload
            );
        }
        let events = if !self.recovering
            && matches!(step, 1 | 3)
            && self.rejected_probe.load(Ordering::SeqCst) < step
        {
            self.step.store(step, Ordering::SeqCst);
            self.rejected_probe.store(step, Ordering::SeqCst);
            let discovery = result(messages, &format!("discover-{}", step - 1));
            let item = |text: &str| json!({"text":text,"sources":[discovery["sources"][0]]});
            call(
                &format!("oversized-{step}"),
                "compact_context",
                json!({
                    "snapshot":discovery["snapshot"],
                    "checkpoint":{"schema_version":1,"summary":item(&"x".repeat(40 * 1024)),
                        "goal":item("repair addition"),"next_step":item("verify addition"),
                        "completed":[],"open_loops":[],"assumptions":[],"facts":[],
                        "constraints":[],"in_progress":[],"pending":[],"decisions":[],"resources":[] }
                }),
            )
        } else if self.recovering {
            match step {
                0 => call("discover", "history_search", json!({"mode":"discover"})),
                1 => call(
                    "search",
                    "history_search",
                    json!({"mode":"search","snapshot":result(messages,"discover")["snapshot"],"query":"codingfact"}),
                ),
                2 => {
                    let found = result(messages, "search");
                    assert_eq!(found["results"].as_array().expect("results").len(), 1);
                    call(
                        "detail",
                        "history_search",
                        json!({"mode":"detail","snapshot":result(messages,"discover")["snapshot"],"reference":found["results"][0]["reference"]}),
                    )
                }
                3 => {
                    let detail = result(messages, "detail");
                    assert!(
                        detail["text"]
                            .as_str()
                            .expect("text")
                            .contains("pending verify addition")
                    );
                    assert!(
                        detail["text"]
                            .as_str()
                            .expect("text")
                            .contains("deployment unverified")
                    );
                    call("verify", "verify_fixture", json!({}))
                }
                4 => call(
                    "long-search",
                    "history_search",
                    json!({"mode":"search", "snapshot":result(messages,"discover")["snapshot"], "query":"longdetailfact"}),
                ),
                5 => {
                    let found = result(messages, "long-search");
                    call(
                        "long-detail",
                        "history_search",
                        json!({"mode":"detail", "snapshot":result(messages,"discover")["snapshot"], "reference":found["results"][0]["reference"]}),
                    )
                }
                6 => {
                    let detail = result(messages, "long-detail");
                    let text = detail["text"].as_str().expect("large JSON detail intact");
                    assert!(text.len() > 4096 && text.len() <= 8192);
                    assert!(text.starts_with("longdetailfact"));
                    assert!(!text.contains("[truncated,"));
                    assert!(messages.iter().any(|message| matches!(message, Message::Tool { result } if result.tool_use_id == "verify" && !result.is_error && result.content == "addition source verified; deployment remains unverified")));
                    vec![
                        AgentEvent::TextDelta {
                            delta: "verified repaired source; deployment unverified".into(),
                        },
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            usage: Default::default(),
                        },
                    ]
                }
                _ => panic!("unexpected recovery request"),
            }
        } else {
            match step {
                0 | 2 => {
                    if step == 2 {
                        assert_eq!(result(messages, "compact-1")["status"], "Admitted");
                        let installed = messages
                            .iter()
                            .find_map(|message| match message {
                                Message::Context { content }
                                    if content.starts_with("[Derived checkpoint;") =>
                                {
                                    Some(content.clone())
                                }
                                _ => None,
                            })
                            .expect("installed A");
                        *self.installed_a.lock().expect("projection") = Some(installed);
                    }
                    call(
                        &format!("discover-{step}"),
                        "history_search",
                        json!({"mode":"discover"}),
                    )
                }
                1 | 3 => {
                    {
                        let refusal = messages
                            .iter()
                            .find_map(|message| match message {
                                Message::Tool { result }
                                    if result.tool_use_id == format!("oversized-{step}") =>
                                {
                                    Some(result)
                                }
                                _ => None,
                            })
                            .expect("registered oversized refusal");
                        assert!(refusal.is_error);
                        assert!(
                            refusal
                                .content
                                .contains("checkpoint payload exceeds byte limit")
                        );
                        if step == 1 {
                            assert!(!messages.iter().any(|message| matches!(message, Message::Context { content } if content.starts_with("[Derived checkpoint;"))), "refusal must not install a projection");
                            assert!(
                                serde_json::to_string(messages)
                                    .expect("request")
                                    .contains("codingfact repaired"),
                                "original source survives rejected input"
                            );
                        } else {
                            assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.contains("coding checkpoint A"))), "oversized replacement retains installed A");
                            let installed = self.installed_a.lock().expect("projection");
                            assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if Some(content) == installed.as_ref())), "refusal preserves A byte-for-byte");
                        }
                    }
                    if step == 3 {
                        assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.contains("coding checkpoint A"))));
                    }
                    let evidence = if step == 1 {
                        messages
                            .iter()
                            .find_map(|message| match message {
                                Message::Tool { result }
                                    if result.content.starts_with("codingfact repaired") =>
                                {
                                    Some(result.content.as_str())
                                }
                                _ => None,
                            })
                            .expect("original executed repair evidence")
                    } else {
                        messages
                            .iter()
                            .find_map(|message| match message {
                                Message::Context { content }
                                    if content.contains("coding checkpoint A") =>
                                {
                                    Some(content.as_str())
                                }
                                _ => None,
                            })
                            .expect("installed checkpoint evidence")
                    };
                    assert!(evidence.contains("subtraction patch did not match"));
                    assert!(evidence.contains("deployment unverified"));
                    assert!(evidence.contains("verify addition"));
                    let discovery = result(messages, &format!("discover-{}", step - 1));
                    let item =
                        |text: &str| json!({"text":text,"sources":[discovery["sources"][0]]});
                    let checkpoint = json!({"schema_version":1,"summary":item(if step==1 {"coding checkpoint A"} else {"coding checkpoint B"}),"goal":item("repair addition"),"next_step":item("verify addition"),"completed":[item("repair applied")],"open_loops":[item("subtraction patch did not match")],"assumptions":[item("deployment unverified")],"facts":[],"constraints":[],"in_progress":[],"pending":[item("verify addition")],"decisions":[],"resources":[]});
                    call(
                        &format!("compact-{step}"),
                        "compact_context",
                        json!({"snapshot":discovery["snapshot"],"checkpoint":checkpoint}),
                    )
                }
                4 => {
                    assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.contains("coding checkpoint B"))));
                    assert!(
                        !serde_json::to_string(messages)
                            .expect("request")
                            .contains("codingfact repaired")
                    );
                    vec![
                        AgentEvent::TextDelta {
                            delta: "checkpoints installed".into(),
                        },
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            usage: Default::default(),
                        },
                    ]
                }
                _ => panic!("unexpected checkpoint request"),
            }
        };
        let (tx, rx) = mpsc::channel(16);
        for event in events {
            tx.send(event).await.expect("receiver");
        }
        Ok(rx)
    }
}

#[tokio::test]
async fn runtime_coding_artifact_survives_two_checkpoints_and_fork_recovery() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("add.ts");
    std::fs::write(
        &path,
        "export function add(a: number, b: number) { return a + a; }\n",
    )
    .expect("fixture");
    let session =
        DurableSession::open_or_create(&root.path().join("sessions"), "coding").expect("session");
    let repairs = Arc::new(AtomicUsize::new(0));
    let mut producer = RuntimeBuilder::new()
        .provider(Arc::new(
            talos_provider::mock::MockProvider::new()
                .with_tool_call("repair_fixture", json!({}))
                .with_response("repair recorded"),
        ))
        .workspace_root(root.path())
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .tool(Arc::new(CodingTool {
            path: path.clone(),
            repair: true,
            calls: repairs.clone(),
        }))
        .durable_session(session.clone())
        .build()
        .expect("producer");
    producer.submit("repair addition").await.expect("submit");
    assert!(matches!(
        tokio::time::timeout(
            Duration::from_secs(10),
            collect_until_turn_completed(&mut producer)
        )
        .await
        .expect("producer deadline")
        .expect("completion"),
        TurnCompletionStatus::Success { .. }
    ));
    producer.shutdown().await.expect("shutdown");
    let filler: Vec<_> = (0..8300)
        .map(|i| Message::User {
            content: format!("prior {i} {}", "x".repeat(256)),
        })
        .collect();
    let mut filler = filler;
    filler[0] = Message::User {
        content: format!("longdetailfact {}", "evidence ".repeat(650)),
    };
    session
        .commit_turn("older-context", &filler, &Default::default())
        .expect("filler");
    let raw = std::fs::read(session.file_path()).expect("raw");
    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(CodingModel {
            rejected_probe: AtomicUsize::new(0),
            installed_a: std::sync::Mutex::new(None),
            step: AtomicUsize::new(0),
            recovering: false,
        }))
        .workspace_root(root.path())
        .model_context_limit(1_000_000)
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .durable_session(session.clone())
        .build()
        .expect("runtime");
    runtime
        .submit("checkpoint work twice")
        .await
        .expect("submit");
    let mut applied = 0;
    let mut completed = false;
    tokio::time::timeout(Duration::from_secs(40), async {
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::ModelCheckpointTerminal { outcome, .. } => {
                    let talos_core::session::CompactOutcome::Applied { before, after } = outcome
                    else {
                        panic!("{outcome:?}")
                    };
                    assert!(after < before);
                    applied += 1;
                    println!("coding_checkpoint before={before} after={after}");
                }
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { status },
                    ..
                } => {
                    assert!(matches!(status, TurnCompletionStatus::Success { .. }));
                    completed = true;
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("checkpoint deadline");
    assert!(
        completed,
        "checkpoint stage requires successful completion, not channel EOF"
    );
    assert_eq!(applied, 2);
    runtime.fork_durable("coding-child").await.expect("fork");
    runtime.shutdown().await.expect("shutdown");
    let child = DurableSession::open_existing(&root.path().join("sessions"), "coding-child")
        .expect("lookup")
        .expect("child");
    assert!(
        std::fs::read(child.file_path())
            .expect("child raw")
            .starts_with(&raw)
    );
    let verifies = Arc::new(AtomicUsize::new(0));
    let recovery_model = Arc::new(CodingModel {
        rejected_probe: AtomicUsize::new(0),
        installed_a: std::sync::Mutex::new(None),
        step: AtomicUsize::new(0),
        recovering: true,
    });
    let child_before = std::fs::read(child.file_path()).expect("before recovery");
    let child_path = child.file_path().to_path_buf();
    let mut resumed = RuntimeBuilder::new()
        .provider(recovery_model.clone())
        .workspace_root(root.path())
        .model_context_limit(1_000_000)
        .tool(Arc::new(CodingTool {
            path,
            repair: false,
            calls: verifies.clone(),
        }))
        .durable_session(child)
        .build()
        .expect("reopen");
    resumed
        .submit("recover pending work")
        .await
        .expect("submit");
    let status = tokio::time::timeout(
        Duration::from_secs(30),
        collect_until_turn_completed(&mut resumed),
    )
    .await
    .expect("recovery deadline")
    .expect("complete");
    assert!(
        matches!(status,TurnCompletionStatus::Success {final_text,..} if final_text=="verified repaired source; deployment unverified")
    );
    resumed.shutdown().await.expect("shutdown");
    assert_eq!(repairs.load(Ordering::SeqCst), 1);
    assert_eq!(verifies.load(Ordering::SeqCst), 1);
    assert_eq!(recovery_model.step.load(Ordering::SeqCst), 7);
    assert!(
        std::fs::read(child_path)
            .expect("after recovery")
            .starts_with(&child_before)
    );
    assert!(
        std::fs::read(session.file_path())
            .expect("parent")
            .starts_with(&raw)
    );
    println!("coding repairs=1 verifications=1 checkpoints=2 recovery_calls=5");
}

#[tokio::test]
async fn registered_checkpoint_runtime_bounds_huge_tool_before_provider_continuation() {
    struct HugeTool;
    #[async_trait]
    impl AgentTool for HugeTool {
        fn name(&self) -> &str {
            "huge_fixture"
        }
        fn description(&self) -> &str {
            "Return bounded-fixture evidence"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object"})
        }
        fn is_read_only(&self) -> bool {
            true
        }
        async fn execute(&self, _: Value) -> ToolResult {
            ToolResult::success(format!(
                "visible-fact {} hidden-tail",
                "x".repeat(2 * 1024 * 1024)
            ))
        }
    }
    struct HugeModel(AtomicUsize);
    #[async_trait]
    impl LanguageModel for HugeModel {
        async fn stream(&self, _: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            panic!("normal requests must carry tool definitions")
        }
        async fn stream_with_tools(
            &self,
            messages: &[Message],
            tools: &[talos_core::provider::ToolDefinition],
        ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            for name in ["compact_context", "history_search", "huge_fixture"] {
                assert_eq!(tools.iter().filter(|tool| tool.name == name).count(), 1);
            }
            let events = match self.0.fetch_add(1, Ordering::SeqCst) {
                0 => call("huge", "huge_fixture", json!({})),
                1 => {
                    let results: Vec<_> = messages
                        .iter()
                        .filter_map(|message| match message {
                            Message::Tool { result } if result.tool_use_id == "huge" => {
                                Some(result)
                            }
                            _ => None,
                        })
                        .collect();
                    assert_eq!(results.len(), 1);
                    let content = &results[0].content;
                    assert!(content.starts_with("visible-fact"));
                    assert!(content.len() < 4200);
                    assert_eq!(content.matches("[truncated,").count(), 1);
                    assert!(!content.contains("hidden-tail"));
                    assert_eq!(messages.iter().filter(|message| matches!(message, Message::Assistant { tool_calls, .. } if tool_calls.iter().any(|call| call.id == "huge"))).count(), 1);
                    vec![
                        AgentEvent::TextDelta {
                            delta: "bounded continuation".into(),
                        },
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            usage: Default::default(),
                        },
                    ]
                }
                _ => panic!("unexpected provider call"),
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("receiver");
            }
            Ok(rx)
        }
    }
    let root = tempfile::tempdir().expect("workspace");
    let durable = DurableSession::open_or_create(&root.path().join("sessions"), "huge-output")
        .expect("durable");
    let model = Arc::new(HugeModel(AtomicUsize::new(0)));
    let mut runtime = RuntimeBuilder::new()
        .provider(model.clone())
        .workspace_root(root.path())
        .model_context_limit(32_000)
        .tool(Arc::new(HugeTool))
        .durable_session(durable.clone())
        .build()
        .expect("runtime");
    runtime.submit("read huge result").await.expect("submit");
    let status = tokio::time::timeout(
        Duration::from_secs(10),
        collect_until_turn_completed(&mut runtime),
    )
    .await
    .expect("deadline")
    .expect("completion");
    assert!(
        matches!(status, TurnCompletionStatus::Success { .. }),
        "{status:?}"
    );
    assert_eq!(model.0.load(Ordering::SeqCst), 2);
    runtime.shutdown().await.expect("shutdown");
    // A completed old tool batch becomes eligible only after the protected
    // recent exchanges. Preserve it in authoritative storage while compacting it.
    let later: Vec<_> = (0..12)
        .flat_map(|index| {
            [
                Message::User {
                    content: format!("later safe task {index}"),
                },
                Message::Assistant {
                    content: "completed".into(),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ]
        })
        .collect();
    durable
        .commit_turn("later-safe-boundaries", &later, &Default::default())
        .expect("later history");
    let reopened = DurableSession::open_existing(&root.path().join("sessions"), "huge-output")
        .expect("reopen")
        .expect("session");
    let history = reopened.read_messages().expect("persisted history");
    let results: Vec<_> = history
        .iter()
        .filter_map(|message| match message {
            Message::Tool { result } if result.tool_use_id == "huge" => Some(result),
            _ => None,
        })
        .collect();
    assert_eq!(results.len(), 1, "one authoritative tool result");
    assert!(results[0].content.len() < 4200);
    assert_eq!(results[0].content.matches("[truncated,").count(), 1);
    assert!(!results[0].content.contains("hidden-tail"));
    struct RecoverHuge(AtomicUsize);
    #[async_trait]
    impl LanguageModel for RecoverHuge {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let events = match self.0.fetch_add(1, Ordering::SeqCst) {
                0 => call("discover", "history_search", json!({"mode":"discover"})),
                1 => {
                    let discovery = result(messages, "discover");
                    let item =
                        |text: &str| json!({"text":text,"sources":[discovery["sources"][0]]});
                    call(
                        "compact-huge",
                        "compact_context",
                        json!({"snapshot":discovery["snapshot"],
                        "checkpoint":{"schema_version":1,"summary":item("Huge tool completed; original bounded fact remains recoverable"),
                        "goal":item("recover prior fact"),"next_step":item("search authoritative history"),
                        "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                        "facts":[],"assumptions":[],"resources":[],"open_loops":[]}}),
                    )
                }
                2 => {
                    assert_eq!(result(messages, "compact-huge")["status"], "Admitted");
                    assert!(messages.iter().any(|message| matches!(message, Message::Context { content } if content.starts_with("[Derived checkpoint;"))));
                    assert!(!messages.iter().any(|message| matches!(message, Message::Tool { result } if result.tool_use_id == "huge")), "huge batch must actually leave the provider projection");
                    call(
                        "discover-recovery",
                        "history_search",
                        json!({"mode":"discover"}),
                    )
                }
                3 => call(
                    "search",
                    "history_search",
                    // Recovery search accepts FTS syntax; quote a hyphenated
                    // literal so it is not parsed as an FTS column expression.
                    json!({"mode":"search", "snapshot":result(messages,"discover-recovery")["snapshot"], "query":"\"visible-fact\""}),
                ),
                4 => {
                    let found = result(messages, "search");
                    assert_eq!(found["results"].as_array().expect("results").len(), 1);
                    call(
                        "detail",
                        "history_search",
                        json!({"mode":"detail", "snapshot":result(messages,"discover-recovery")["snapshot"], "reference":found["results"][0]["reference"]}),
                    )
                }
                5 => {
                    let detail = result(messages, "detail");
                    let text = detail["text"].as_str().expect("recovered text");
                    assert!(text.starts_with("visible-fact"));
                    assert_eq!(text.matches("[truncated,").count(), 1);
                    assert!(text.len() < 4200);
                    assert!(!text.contains("hidden-tail"));
                    vec![
                        AgentEvent::TextDelta {
                            delta: "bounded fact recovered".into(),
                        },
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            usage: Default::default(),
                        },
                    ]
                }
                _ => panic!("unexpected recovery request"),
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("receiver");
            }
            Ok(rx)
        }
    }
    let storage_projection = |session: &DurableSession| {
        let entries = session
            .session()
            .read_entries()
            .expect("actual stored records");
        let receipts: Vec<_> = entries
            .iter()
            .flat_map(|entry| {
                talos_session::HistoryRecoveryReceipt::from_terminal_entry(entry)
                    .unwrap_or_default()
            })
            .collect();
        let matches: Vec<_> = receipts
            .iter()
            .filter_map(|receipt| {
                let entry = entries
                    .iter()
                    .find(|entry| entry.id == receipt.entry_id())?;
                let content = receipt.verified_content(entry)?;
                content
                    .starts_with("visible-fact")
                    .then_some((entry, content))
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "one persisted recovery receipt for the bounded source"
        );
        let (entry, content) = &matches[0];
        assert!(
            entry.metadata.raw_content.is_none(),
            "durable projection must not copy hidden producer bytes into raw metadata"
        );
        assert_eq!(content.matches("[truncated,").count(), 1);
        assert!(
            !serde_json::to_string(&entries)
                .expect("stored metadata")
                .contains("hidden-tail")
        );
        (entry.id.clone(), content.clone())
    };
    let original_projection = storage_projection(&reopened);
    let original_bytes = std::fs::read(reopened.file_path()).expect("original transcript");
    let recovery = Arc::new(RecoverHuge(AtomicUsize::new(0)));
    let mut runtime = RuntimeBuilder::new()
        .provider(recovery.clone())
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .workspace_root(root.path())
        .model_context_limit(32_000)
        .durable_session(reopened.clone())
        .build()
        .expect("recovered runtime");
    runtime
        .submit("recover bounded fact")
        .await
        .expect("submit");
    let mut applied = 0;
    let mut completed = false;
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::ModelCheckpointTerminal { outcome, .. } => {
                    let talos_core::session::CompactOutcome::Applied { before, after } = outcome
                    else {
                        panic!("{outcome:?}")
                    };
                    assert!(after < before);
                    println!("huge_checkpoint before={before} after={after}");
                    applied += 1;
                }
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { status },
                    ..
                } => {
                    assert!(
                        matches!(status, TurnCompletionStatus::Success { .. }),
                        "{status:?}"
                    );
                    completed = true;
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("recovery deadline");
    assert!(completed);
    assert_eq!(applied, 1, "one confirmed installed checkpoint");
    assert_eq!(recovery.0.load(Ordering::SeqCst), 6);
    runtime.shutdown().await.expect("shutdown");
    assert!(
        std::fs::read(reopened.file_path())
            .expect("transcript after checkpoint")
            .starts_with(&original_bytes)
    );
    assert_eq!(
        storage_projection(&reopened),
        original_projection,
        "checkpoint and recovery retain the same source and receipt projection"
    );
    let final_history = reopened.read_messages().expect("authoritative history");
    let original_results: Vec<_> = final_history
        .iter()
        .filter_map(|message| match message {
            Message::Tool { result } if result.tool_use_id == "huge" => Some(result),
            _ => None,
        })
        .collect();
    assert_eq!(original_results.len(), 1);
    assert_eq!(
        original_results[0].content.matches("[truncated,").count(),
        1
    );
    assert!(!original_results[0].content.contains("hidden-tail"));
}

#[tokio::test]
async fn registered_checkpoint_rejects_insufficient_reclaim_preserving_installed_projection() {
    struct BatchGate(Arc<tokio::sync::Notify>);
    #[async_trait]
    impl AgentTool for BatchGate {
        fn name(&self) -> &str {
            "batch_gate"
        }
        fn description(&self) -> &str {
            "Controlled unfinished tool batch"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object"})
        }
        fn is_read_only(&self) -> bool {
            true
        }
        async fn execute(&self, _: Value) -> ToolResult {
            self.0.notified().await;
            ToolResult::success("batch completed")
        }
    }
    struct Model {
        step: AtomicUsize,
        installed: std::sync::Mutex<Option<String>>,
    }
    #[async_trait]
    impl LanguageModel for Model {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let step = self.step.fetch_add(1, Ordering::SeqCst);
            let projection = || {
                messages.iter().find_map(|message| match message {
                    Message::Context { content } if content.starts_with("[Derived checkpoint;") => {
                        Some(content.clone())
                    }
                    _ => None,
                })
            };
            let events = match step {
                0 | 2 | 4 => {
                    if step == 2 {
                        assert!(messages.iter().any(|message| matches!(message, Message::Tool { result } if result.tool_use_id == "gate" && result.content == "batch completed")), "checkpoint continuation keeps complete paired batch");
                        *self.installed.lock().expect("projection") =
                            Some(projection().expect("A installed"));
                    }
                    if step == 4 {
                        assert_eq!(
                            projection(),
                            *self.installed.lock().expect("projection"),
                            "failed replacement preserves exact installed A"
                        );
                        assert_eq!(result(messages, "compact-3")["status"], "Admitted");
                        let terminal = messages
                            .iter()
                            .rev()
                            .find_map(|message| match message {
                                Message::Multimodal { parts } => {
                                    parts.iter().find_map(|part| match part {
                                        talos_core::message::ContentPart::Text { text } => {
                                            serde_json::from_str::<Value>(text).ok().filter(
                                                |value| value.get("checkpoint_operation").is_some(),
                                            )
                                        }
                                        _ => None,
                                    })
                                }
                                _ => None,
                            })
                            .expect("terminal metadata");
                        let outcome = terminal["outcome"].as_str().expect("outcome");
                        assert!(
                            outcome.contains("FailedWithEstimate")
                                && outcome.contains("Preparation"),
                            "{terminal}"
                        );
                    }
                    call(
                        &format!("discover-{step}"),
                        "history_search",
                        json!({"mode":"discover"}),
                    )
                }
                1 | 3 | 5 => {
                    let discovery = result(messages, &format!("discover-{}", step - 1));
                    let item =
                        |text: &str| json!({"text":text,"sources":[discovery["sources"][0]]});
                    let text = if step == 3 {
                        "x".repeat(3900)
                    } else {
                        format!("checkpoint {step}")
                    };
                    let checkpoint = json!({"schema_version":1,"summary":item(&text),"goal":item(&text),"next_step":item(&text),"completed":[],"open_loops":[],"assumptions":[],"facts":[],"constraints":[],"in_progress":[],"pending":[],"decisions":[],"resources":[]});
                    // This probe is valid schema and fits the render cap. It must
                    // fail the reclamation gate, not an input-size shortcut.
                    let parsed =
                        talos_agent::compaction::Checkpoint::parse(&checkpoint.to_string(), |_| {
                            true
                        })
                        .expect("valid schema");
                    parsed.render(64_000).expect("within render cap");
                    let mut events = call(
                        &format!("compact-{step}"),
                        "compact_context",
                        json!({"snapshot":discovery["snapshot"],"checkpoint":checkpoint}),
                    );
                    if step == 1 {
                        let gate = call("gate", "batch_gate", json!({}));
                        events.insert(1, gate[0].clone());
                    }
                    events
                }
                6 => {
                    let current = projection().expect("B installed");
                    assert!(current.contains("checkpoint 5"));
                    assert_ne!(Some(current), *self.installed.lock().expect("projection"));
                    vec![AgentEvent::TurnEnd {
                        stop_reason: StopReason::EndTurn,
                        usage: Default::default(),
                    }]
                }
                _ => panic!("unexpected request {step}"),
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("receiver");
            }
            Ok(rx)
        }
    }
    let root = tempfile::tempdir().expect("workspace");
    let durable =
        DurableSession::open_or_create(&root.path().join("sessions"), "reclaim").expect("session");
    let seed: Vec<_> = (0..20)
        .flat_map(|index| {
            [
                Message::User {
                    content: format!("task {index} {}", "x".repeat(120)),
                },
                Message::Assistant {
                    content: "completed".repeat(12),
                    tool_calls: vec![],
                    reasoning: None,
                },
            ]
        })
        .collect();
    durable
        .commit_turn("seed", &seed, &Default::default())
        .expect("seed");
    let release = Arc::new(tokio::sync::Notify::new());
    let model = Arc::new(Model {
        step: AtomicUsize::new(0),
        installed: std::sync::Mutex::new(None),
    });
    let mut runtime = RuntimeBuilder::new()
        .provider(model.clone())
        .tool(Arc::new(BatchGate(release.clone())))
        .workspace_root(root.path())
        .model_context_limit(64_000)
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .durable_session(durable)
        .build()
        .expect("runtime");
    runtime
        .submit("exercise reclamation rejection and recovery")
        .await
        .expect("submit");
    let mut outcomes = Vec::new();
    let mut manual_busy = 0;
    let mut gate_completed = false;
    let mut completed_success = false;
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::ModelCheckpointTerminal { outcome, .. } => {
                    assert!(
                        gate_completed,
                        "installation cannot precede the complete tool batch"
                    );
                    outcomes.push(outcome);
                }
                SessionEvent::TurnEvent {
                    payload:
                        talos_core::session::TurnEventPayload::Progress {
                            event: AgentEvent::ToolResult { result, .. },
                        },
                    ..
                } if result.tool_use_id == "compact-1" => {
                    assert!(!result.is_error);
                    assert_eq!(
                        serde_json::from_str::<Value>(&result.content).expect("admission receipt")
                            ["status"],
                        "Admitted"
                    );
                    assert!(outcomes.is_empty());
                    runtime
                        .compact(99)
                        .await
                        .expect("manual contention request");
                }
                SessionEvent::CompactStatus {
                    operation_id: 99,
                    state,
                    ..
                } => {
                    assert!(
                        matches!(state, talos_core::session::CompactState::Busy),
                        "{state:?}"
                    );
                    assert!(
                        outcomes.is_empty(),
                        "model operation remains pending during the blocked batch"
                    );
                    manual_busy += 1;
                    release.notify_one();
                }
                SessionEvent::TurnEvent {
                    payload:
                        talos_core::session::TurnEventPayload::Progress {
                            event: AgentEvent::ToolResult { result, .. },
                        },
                    ..
                } if result.tool_use_id == "gate" => {
                    assert_eq!(result.content, "batch completed");
                    gate_completed = true;
                }
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { status },
                    ..
                } => {
                    assert!(
                        matches!(status, TurnCompletionStatus::Success { .. }),
                        "{status:?}"
                    );
                    completed_success = true;
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("deadline");
    assert!(completed_success, "successful Turn completion is required");
    assert_eq!(model.step.load(Ordering::SeqCst), 7);
    assert_eq!(manual_busy, 1);
    assert_eq!(outcomes.len(), 3);
    assert!(matches!(
        outcomes[0],
        talos_core::session::CompactOutcome::Applied { .. }
    ));
    assert!(matches!(
        outcomes[1],
        talos_core::session::CompactOutcome::FailedWithEstimate {
            reason: talos_core::session::CompactFailureReason::Preparation,
            ..
        }
    ));
    assert!(matches!(
        outcomes[2],
        talos_core::session::CompactOutcome::Applied { .. }
    ));
    runtime.shutdown().await.expect("shutdown");
}
