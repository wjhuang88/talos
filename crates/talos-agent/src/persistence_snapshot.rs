//! Execution-owned persistence snapshots; raw history cannot mint provenance.

use crate::{Agent, transient_text::PrivateTokens};
use std::collections::{HashMap, HashSet};
use talos_core::message::Message;
use talos_session::RecoveryProjection;

tokio::task_local! {
    /// Unique execution scope; not inherited by unrelated spawned tasks.
    pub(crate) static RECOVERY_EXECUTION: uuid::Uuid;
}

pub(crate) struct PersistenceSnapshot {
    pub messages: Vec<Message>,
    pub recovery: Vec<RecoveryProjection>,
}

impl Agent {
    pub(crate) fn persistence_snapshot(
        &self,
        raw: &[Message],
        private: &PrivateTokens,
    ) -> PersistenceSnapshot {
        let messages = self.persistence_projection_with_tokens(raw, private);
        let mut pending = HashMap::new();
        let mut seen = HashSet::new();
        let mut recovery = Vec::new();
        for (index, message) in raw.iter().enumerate() {
            match message {
                Message::Assistant { tool_calls, .. } => {
                    pending.clear();
                    for call in tool_calls {
                        if seen.insert(call.id.as_str()) {
                            pending.insert(call.id.as_str(), call.name.as_str());
                        } else {
                            pending.remove(call.id.as_str());
                        }
                    }
                }
                Message::Tool { result } => {
                    let Some(name) = pending.remove(result.tool_use_id.as_str()) else {
                        continue;
                    };
                    if !self.has_checkpoint_result(name, result) {
                        continue;
                    }
                    let Some(Message::Tool { result: stored }) = messages.get(index) else {
                        continue;
                    };
                    let mut content = private.project(&result.content);
                    if content != stored.content {
                        let Some(tool) = self.tools.get(name) else {
                            continue;
                        };
                        let execution = talos_core::tool::ToolResult {
                            content: result.content.clone(),
                            is_error: result.is_error,
                            continuations: Vec::new(),
                            todo_presentation: None,
                        };
                        let Ok(Some(permitted)) =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                tool.recovery_projection(&execution)
                            }))
                        else {
                            continue;
                        };
                        content = private.project(&permitted);
                    }
                    // Different views require the explicit tool opt-in above;
                    // durable storage still enforces policy and redaction.
                    if content.len() > 8192 || result.tool_use_id.len() > 1024 || name.len() > 256 {
                        continue;
                    }
                    recovery.push(RecoveryProjection {
                        message_index: index,
                        tool_call_id: result.tool_use_id.clone(),
                        tool_name: name.into(),
                        content,
                    });
                }
                _ => pending.clear(),
            }
        }
        PersistenceSnapshot { messages, recovery }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use talos_core::{
        message::{MessageToolResult, ToolCall},
        tool::{AgentTool, ToolRegistry, ToolResult, ToolResultProjection},
    };
    struct Model;
    #[async_trait::async_trait]
    impl talos_core::provider::LanguageModel for Model {
        async fn stream(
            &self,
            _: &[Message],
        ) -> talos_core::provider::ProviderResult<
            talos_core::provider::Receiver<talos_core::message::AgentEvent>,
        > {
            panic!("no provider needed")
        }
    }
    struct Tool {
        different: bool,
        opt_in: bool,
    }
    #[async_trait::async_trait]
    impl AgentTool for Tool {
        fn name(&self) -> &str {
            "read"
        }
        fn description(&self) -> &str {
            "fixture"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn execute(&self, _: serde_json::Value) -> ToolResult {
            ToolResult::success("fact")
        }
        fn recovery_projection(&self, result: &ToolResult) -> Option<String> {
            self.opt_in.then(|| result.content.clone())
        }
        fn project_result(&self, result: &ToolResult) -> ToolResultProjection {
            ToolResultProjection {
                model_content: result.content.clone(),
                display_content: String::new(),
                persistence_content: if self.different {
                    "omitted".into()
                } else {
                    result.content.clone()
                },
            }
        }
    }
    #[tokio::test]
    async fn real_read_snapshot_recovers_only_persisted_numbered_text_after_restart() {
        RECOVERY_EXECUTION
            .scope(uuid::Uuid::new_v4(), async {
                let dir = tempfile::tempdir().expect("tempdir");
                std::fs::write(dir.path().join("fact.txt"), "durable fact\n").expect("file");
                let read = Arc::new(talos_tools::ReadTool::with_snapshot_registry(
                    dir.path().to_path_buf(),
                    talos_tools::FileSnapshotRegistry::new(),
                ));
                let output = read.execute(serde_json::json!({"path": "fact.txt"})).await;
                assert!(!output.is_error, "read fixture failed");
                assert!(output.content.starts_with("[snapshot:"));
                let mut private = PrivateTokens::default();
                private.extend(read.model_private_tokens(&output));
                let expected = private.project(&read.project_result(&output).persistence_content);
                let mut registry = ToolRegistry::new();
                registry.register(read);
                let agent = Agent::new(Arc::new(Model), registry);
                let result = MessageToolResult {
                    tool_use_id: "call".into(),
                    content: output.content,
                    is_error: false,
                };
                let messages = vec![
                    Message::Assistant {
                        content: String::new(),
                        reasoning: None,
                        tool_calls: vec![ToolCall {
                            id: "call".into(),
                            name: "read".into(),
                            input: serde_json::json!({"path":"fact.txt"}),
                        }],
                    },
                    Message::Tool {
                        result: result.clone(),
                    },
                ];
                agent.record_checkpoint_result("read", &result);
                let snapshot = agent.persistence_snapshot(&messages, &private);
                assert_eq!(snapshot.recovery.len(), 1);
                assert_eq!(snapshot.recovery[0].content, expected);
                assert!(!expected.contains("snapshot:"));
                for permit in [false, true] {
                    let key = format!("read-{permit}");
                    let session = talos_session::DurableSession::open_or_create(dir.path(), &key)
                        .expect("session");
                    session
                        .finalize_turn_with_recovery(
                            "turn",
                            &snapshot.messages,
                            &talos_session::PersistencePolicy {
                                persist_raw_tool_output: permit,
                                persist_reasoning: false,
                            },
                            talos_session::TurnTranscriptOutcome::Success,
                            &snapshot.recovery,
                        )
                        .expect("commit");
                    drop(session);
                    let reopened = talos_session::DurableSession::open_existing(dir.path(), &key)
                        .expect("reopen")
                        .expect("session");
                    let entries = reopened.session().read_entries().expect("entries");
                    let receipts = talos_session::HistoryRecoveryReceipt::from_terminal_entry(
                        entries.last().expect("terminal"),
                    )
                    .expect("receipts");
                    assert_eq!(receipts.len(), usize::from(permit));
                    if permit {
                        assert_eq!(
                            receipts[0].verified_content(&entries[1]),
                            Some(expected.lines().collect::<Vec<_>>().join("\n"))
                        );
                    }
                }
            })
            .await;
    }

    #[tokio::test]
    async fn execution_scope_does_not_leak_to_unrelated_tasks_or_after_return() {
        assert!(RECOVERY_EXECUTION.try_with(|id| *id).is_err());
        let first = uuid::Uuid::new_v4();
        let second = uuid::Uuid::new_v4();
        RECOVERY_EXECUTION
            .scope(first, async {
                assert_eq!(RECOVERY_EXECUTION.with(|id| *id), first);
                let child = tokio::spawn(async { RECOVERY_EXECUTION.try_with(|id| *id).ok() });
                assert_eq!(child.await.expect("child"), None);
                RECOVERY_EXECUTION
                    .scope(second, async {
                        tokio::task::yield_now().await;
                        assert_eq!(RECOVERY_EXECUTION.with(|id| *id), second);
                    })
                    .await;
                assert_eq!(RECOVERY_EXECUTION.with(|id| *id), first);
            })
            .await;
        assert!(RECOVERY_EXECUTION.try_with(|id| *id).is_err());
    }

    #[test]
    fn only_live_matching_execution_projection_can_be_persisted_for_recovery() {
        for (different, opt_in) in [(false, false), (true, false), (true, true)] {
            RECOVERY_EXECUTION.sync_scope(uuid::Uuid::new_v4(), || {
                let mut registry = ToolRegistry::new();
                registry.register(Arc::new(Tool { different, opt_in }));
                let mut agent = Agent::new(Arc::new(Model), registry);
                let result = MessageToolResult {
                    tool_use_id: "call".into(),
                    content: "fact".into(),
                    is_error: false,
                };
                let messages = vec![
                    Message::Assistant {
                        content: String::new(),
                        reasoning: None,
                        tool_calls: vec![ToolCall {
                            id: "call".into(),
                            name: "read".into(),
                            input: serde_json::json!({}),
                        }],
                    },
                    Message::Tool {
                        result: result.clone(),
                    },
                ];
                let private = PrivateTokens::default();
                assert!(
                    agent
                        .persistence_snapshot(&messages, &private)
                        .recovery
                        .is_empty()
                );
                agent.record_checkpoint_result("read", &result);
                let snapshot = agent.persistence_snapshot(&messages, &private);
                assert_eq!(snapshot.recovery.len(), usize::from(!different || opt_in));
                if !different || opt_in {
                    assert_eq!(snapshot.recovery[0].message_index, 1);
                    for permit in [false, true] {
                        let dir = tempfile::tempdir().expect("directory");
                        let session =
                            talos_session::DurableSession::open_or_create(dir.path(), "projection")
                                .expect("session");
                        session
                            .finalize_turn_with_recovery(
                                "turn",
                                &snapshot.messages,
                                &talos_session::PersistencePolicy {
                                    persist_raw_tool_output: permit,
                                    ..Default::default()
                                },
                                talos_session::TurnTranscriptOutcome::Success,
                                &snapshot.recovery,
                            )
                            .expect("atomic commit");
                        let retried = session
                            .finalize_turn_with_recovery(
                                "turn",
                                &snapshot.messages,
                                &talos_session::PersistencePolicy {
                                    persist_raw_tool_output: permit,
                                    ..Default::default()
                                },
                                talos_session::TurnTranscriptOutcome::Success,
                                &snapshot.recovery,
                            )
                            .expect("idempotent same projection");
                        assert!(!retried.newly_committed);
                        drop(session);
                        let reopened =
                            talos_session::DurableSession::open_existing(dir.path(), "projection")
                                .expect("reopen")
                                .expect("present");
                        let entries = reopened.session().read_entries().expect("entries");
                        let receipts = talos_session::HistoryRecoveryReceipt::from_terminal_entry(
                            entries.last().expect("marker"),
                        )
                        .expect("receipts");
                        assert_eq!(receipts.len(), usize::from(permit));
                        if permit {
                            assert_eq!(
                                receipts[0].verified_content(&entries[1]),
                                Some("fact".into())
                            );
                            if different {
                                assert!(entries[1].content.contains("omitted"));
                                let receipt = serde_json::to_value(&receipts[0]).expect("receipt");
                                assert_eq!(receipt["version"], 2);
                                for (field, value) in [
                                    ("version", serde_json::json!(99)),
                                    ("version", serde_json::json!(1)),
                                    ("retained_projection", serde_json::Value::Null),
                                    ("retained_projection", serde_json::json!("x".repeat(8193))),
                                    ("retained_projection", serde_json::json!("tampered")),
                                ] {
                                    let mut changed = receipt.clone();
                                    changed[field] = value;
                                    let changed: talos_session::HistoryRecoveryReceipt =
                                        serde_json::from_value(changed).expect("decode");
                                    assert!(changed.verified_content(&entries[1]).is_none());
                                }
                            }
                            let mut changed = entries[1].clone();
                            changed.content.push('x');
                            assert!(receipts[0].verified_content(&changed).is_none());
                        }
                    }
                }
                RECOVERY_EXECUTION.sync_scope(uuid::Uuid::new_v4(), || {
                    assert!(
                        agent
                            .persistence_snapshot(&messages, &private)
                            .recovery
                            .is_empty(),
                        "old execution cannot authorize another turn"
                    );
                    agent.record_checkpoint_result("read", &result);
                    assert_eq!(
                        agent
                            .persistence_snapshot(&messages, &private)
                            .recovery
                            .len(),
                        usize::from(!different || opt_in)
                    );
                });
                agent.tools.register(Arc::new(Tool { different, opt_in }));
                assert!(
                    agent
                        .persistence_snapshot(&messages, &private)
                        .recovery
                        .is_empty()
                );
            });
        }
    }
}
