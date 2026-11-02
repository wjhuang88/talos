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
    assert!(!result.is_error, "{}", result.content);
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
    let model = Arc::new(HugeModel(AtomicUsize::new(0)));
    let mut runtime = RuntimeBuilder::new()
        .provider(model.clone())
        .workspace_root(root.path())
        .model_context_limit(32_000)
        .tool(Arc::new(HugeTool))
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
}
