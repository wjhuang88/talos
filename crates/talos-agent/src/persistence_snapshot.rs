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
                    let content = private.project(&result.content);
                    // This stage supports only identical eligible/model/persisted text.
                    // A different projection needs an explicit permitted representation.
                    if content != stored.content
                        || content.len() > 8192
                        || result.tool_use_id.len() > 1024
                        || name.len() > 256
                    {
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
        for different in [false, true] {
            RECOVERY_EXECUTION.sync_scope(uuid::Uuid::new_v4(), || {
                let mut registry = ToolRegistry::new();
                registry.register(Arc::new(Tool { different }));
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
                assert_eq!(snapshot.recovery.len(), usize::from(!different));
                if !different {
                    assert_eq!(snapshot.recovery[0].message_index, 1);
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
                        usize::from(!different)
                    );
                });
                agent.tools.register(Arc::new(Tool { different }));
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
