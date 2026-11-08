use super::*;

fn call(index: usize, name: &str, input: Value) -> Vec<AgentEvent> {
    vec![
        AgentEvent::TurnStart,
        AgentEvent::ToolCall {
            call: ToolCall {
                id: format!("guard_{index}"),
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

fn registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TimedMockTool {
        tool_name: "echo".into(),
        read_only: true,
        delay_ms: 0,
        result: ToolExecutionResult::success("ok"),
        execution_log: Arc::new(Mutex::new(Vec::new())),
    }));
    registry
}

fn editing_registry(root: &std::path::Path) -> ToolRegistry {
    use talos_core::tool::{ToolContribution, ToolContributionSource};
    let mut registry = registry();
    registry
        .register_builtin_contributions([
            ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(talos_tools::file_tools::EditTool::new(root.to_owned()))
                    as Arc<dyn AgentTool>,
            ),
            ToolContribution::new(
                ToolContributionSource::new("fixture:command"),
                Arc::new(TimedMockTool {
                    tool_name: "exec".into(),
                    read_only: false,
                    delay_ms: 0,
                    result: ToolExecutionResult::success("validated"),
                    execution_log: Arc::new(Mutex::new(vec![])),
                }) as Arc<dyn AgentTool>,
            ),
        ])
        .expect("builtins");
    registry
}

struct ProgressSpoofHook;
#[async_trait]
impl talos_plugin::HookHandler for ProgressSpoofHook {
    fn name(&self) -> &str {
        "progress-spoof"
    }
    fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
        &[talos_plugin::HookEventKind::AfterToolCall]
    }
    async fn on_event(
        &self,
        _: &talos_plugin::HookContext,
        _: &mut talos_plugin::HookEvent<'_>,
    ) -> talos_plugin::HookResult {
        talos_core::tool::record_file_mutation();
        talos_plugin::HookResult::Continue
    }
}

#[tokio::test]
async fn hook_activity_does_not_manufacture_file_progress() {
    let root = tempfile::tempdir().expect("root");
    let mut hooks = talos_plugin::HookRegistry::new();
    hooks.register(Arc::new(ProgressSpoofHook));
    let agent = Agent::with_security_and_hooks(
        Arc::new(MockModel::new(
            (0..3)
                .map(|i| call(i, "exec", serde_json::json!({"message":"same"})))
                .collect(),
        )),
        editing_registry(root.path()),
        None,
        None,
        root.path().into(),
        Arc::new(hooks),
    );
    assert!(
        agent
            .run("work".into())
            .await
            .expect("guard")
            .contains("repeated")
    );
}

#[tokio::test]
async fn progress_preserves_total_budget_and_noncommand_repeat_limits() {
    for scenario in ["budget", "edit_loop", "read_loop", "one_change"] {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("source.txt"), "0").expect("source");
        let mut responses = vec![];
        for i in 0..if scenario == "budget" { 26 } else { 4 } {
            if scenario != "one_change" || i == 0 {
                responses.push(call(i*2, "edit", if scenario == "edit_loop" {
                    serde_json::json!({"path":"source.txt","old_string":"","new_string":"a"})
                } else {
                    serde_json::json!({"path":"source.txt","old_string":i.to_string(),"new_string":(i+1).to_string()})
                }));
            }
            responses.push(call(
                i * 2 + 1,
                if scenario == "read_loop" {
                    "echo"
                } else {
                    "exec"
                },
                serde_json::json!({"message":"same"}),
            ));
        }
        let agent = Agent::new(
            Arc::new(MockModel::new(responses)),
            editing_registry(root.path()),
        );
        // Exercise non-streaming batch execution too.
        let text = agent.run("work".into()).await.expect("bounded");
        assert!(
            text.contains(if scenario == "budget" {
                "limit"
            } else {
                "repeated"
            }),
            "{scenario}: {text}"
        );
        if scenario == "edit_loop" {
            assert!(text.contains("'edit'"));
        }
        if scenario == "read_loop" {
            assert!(text.contains("'echo'"));
        }
        if scenario == "budget" {
            assert_eq!(
                std::fs::read_to_string(root.path().join("source.txt")).expect("source"),
                "25"
            );
        }
    }
}

#[tokio::test]
async fn proposed_same_batch_edit_is_not_completed_progress() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("source.txt"), "0").expect("source");
    let input = serde_json::json!({"message":"same"});
    let mut batch = call(
        2,
        "edit",
        serde_json::json!({"path":"source.txt","old_string":"0","new_string":"1"}),
    );
    batch.insert(2, call(3, "exec", input.clone()).remove(1));
    let agent = Agent::new(
        Arc::new(MockModel::new(vec![
            call(0, "exec", input.clone()),
            call(1, "exec", input),
            batch,
        ])),
        editing_registry(root.path()),
    );
    let text = agent.run("work".into()).await.expect("guard");
    assert!(text.contains("repeated"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.txt")).expect("source"),
        "0"
    );
}

#[tokio::test]
async fn normal_text_is_not_duplicated_and_disconnected_ui_keeps_guard_history() {
    let agent = Agent::new(
        Arc::new(MockModel::new(vec![vec![
            AgentEvent::TextDelta {
                delta: "done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Default::default(),
            },
        ]])),
        registry(),
    );
    let (tx, mut rx) = mpsc::unbounded_channel();
    let (text, _) = agent
        .run_streaming("run".into(), vec![], tx)
        .await
        .expect("normal");
    let mut streamed = String::new();
    while let Ok(event) = rx.try_recv() {
        if let AgentEvent::TextDelta { delta } = event {
            streamed.push_str(&delta);
        }
    }
    assert_eq!(streamed, text);
    let agent = Agent::new(
        Arc::new(MockModel::new(
            (0..3)
                .map(|i| call(i, "echo", serde_json::json!({"message":"same"})))
                .collect(),
        )),
        registry(),
    );
    let (tx, rx) = mpsc::unbounded_channel();
    drop(rx);
    let (text, history) = agent
        .run_streaming("run".into(), vec![], tx)
        .await
        .expect("guard");
    assert!(matches!(history.last(), Some(Message::Assistant {content,..}) if content == &text));
}

#[tokio::test]
async fn completed_real_edits_allow_repeated_command_validation() {
    use talos_core::tool::{ToolContribution, ToolContributionSource};
    for mutation in ["changed", "noop", "failed", "untrusted"] {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("source.txt"), "0").expect("source");
        let mut registry = ToolRegistry::new();
        let exec: Arc<dyn AgentTool> = Arc::new(talos_tools::ExecTool::new(root.path().to_owned()));
        let edit: Arc<dyn AgentTool> = Arc::new(talos_tools::file_tools::EditTool::new(
            root.path().to_owned(),
        ));
        registry
            .register_builtin_contributions([ToolContribution::new(
                ToolContributionSource::new("talos-tools:exec"),
                exec,
            )])
            .expect("exec");
        if mutation == "untrusted" {
            registry.register(edit);
        } else {
            registry
                .register_builtin_contributions([ToolContribution::new(
                    ToolContributionSource::new("talos-tools:file"),
                    edit,
                )])
                .expect("edit");
        }
        let mut responses = vec![];
        for i in 0..3 {
            responses.push(call(
                i * 2,
                "exec",
                serde_json::json!({"command":"rustc", "args":["--version"]}),
            ));
            if i < 2 {
                responses.push(call(i * 2 + 1, "edit", serde_json::json!({"path":"source.txt", "old_string": if mutation == "failed" { "missing".into() } else if mutation == "noop" { "0".into() } else { i.to_string() }, "new_string": if mutation == "noop" { "0".into() } else { (i+1).to_string() }})));
            }
        }
        responses.push(vec![
            AgentEvent::TextDelta {
                delta: "verified".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Default::default(),
            },
        ]);
        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let (tx, _rx) = mpsc::unbounded_channel();
        let (text, messages) = agent
            .run_streaming("edit and validate".into(), vec![], tx)
            .await
            .expect("turn");
        if mutation == "changed" {
            assert_eq!(text, "verified");
            for message in &messages {
                if let Message::Tool { result } = message {
                    assert!(
                        !result.is_error,
                        "real command/edit must succeed: {}",
                        result.content
                    );
                }
            }
        } else {
            assert!(text.contains("repeated"), "{mutation}: {text}");
        }
        let executions = messages.iter().filter(|m| matches!(m, Message::Tool { result } if result.tool_use_id.ends_with('0') || result.tool_use_id.ends_with('2') || result.tool_use_id.ends_with('4'))).count();
        assert_eq!(
            executions,
            if mutation == "changed" { 3 } else { 2 },
            "{mutation}"
        );
    }
}

#[tokio::test]
async fn protection_stops_are_streamed_and_persisted_once() {
    for budget in [false, true] {
        let responses = (0..if budget { 51 } else { 3 })
            .map(|i| {
                call(
                    i,
                    "echo",
                    serde_json::json!({"message": if budget { i } else { 0 }}),
                )
            })
            .collect();
        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry());
        let prepared = agent
            .prepare_turn_start(
                "run".into(),
                vec![Message::User {
                    content: "run".into(),
                }],
                vec![],
                None,
            )
            .await
            .expect("prepare");
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (snap_tx, mut snap_rx) = mpsc::unbounded_channel();
        let (result, messages) = agent
            .run_prepared_session_turn(prepared, tx, Some(snap_tx), None, None)
            .await;
        let reason = result.expect("guard preserves completed work");
        assert!(reason.contains(if budget { "limit" } else { "repeated" }));
        let mut texts = vec![];
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::TextDelta { delta } = event {
                texts.push(delta);
            }
        }
        assert_eq!(
            texts,
            vec![reason.clone()],
            "live guard explanation must appear once"
        );
        assert_eq!(messages.iter().filter(|m| matches!(m, Message::Assistant { content, tool_calls, .. } if content == &reason && tool_calls.is_empty())).count(), 1);
        let mut latest = None;
        while let Ok(snapshot) = snap_rx.try_recv() {
            latest = Some(snapshot);
        }
        assert!(
            matches!(latest.expect("durable snapshot").messages.last(), Some(Message::Assistant { content, tool_calls, .. }) if content == &reason && tool_calls.is_empty())
        );
        assert_eq!(
            messages
                .iter()
                .filter(|m| matches!(m, Message::Tool { .. }))
                .count(),
            if budget { 50 } else { 2 }
        );
    }
}
