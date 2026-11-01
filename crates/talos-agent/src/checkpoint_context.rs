//! Summarizer-only eligibility projection; never a persistence or UI projection.

use std::collections::{HashMap, HashSet};

use talos_core::message::{ContentPart, Message, MessageToolResult};
use talos_core::tool::ToolResult;

use crate::{Agent, transient_text::PrivateTokens};

fn result_identity(name: &str, result: &MessageToolResult) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for field in [name, result.tool_use_id.as_str(), result.content.as_str()] {
        hash.update(field.len().to_le_bytes());
        hash.update(field.as_bytes());
    }
    hash.update([u8::from(result.is_error)]);
    hash.finalize().into()
}

pub(crate) struct ResultReceipt {
    execution: Option<uuid::Uuid>,
    digest: [u8; 32],
    instance: std::sync::Weak<dyn talos_core::tool::AgentTool>,
}

impl Agent {
    pub(crate) fn has_checkpoint_result(&self, name: &str, result: &MessageToolResult) -> bool {
        let Ok(execution) = crate::persistence_snapshot::RECOVERY_EXECUTION.try_with(|id| *id)
        else {
            return false;
        };
        let Some(instance) = self.tools.instance_handle(name) else {
            return false;
        };
        let digest = result_identity(name, result);
        self.checkpoint_result_receipts
            .lock()
            .map(|receipts| {
                receipts.iter().any(|receipt| {
                    receipt.execution == Some(execution)
                        && receipt.digest == digest
                        && receipt.instance.ptr_eq(&instance)
                })
            })
            .unwrap_or(false)
    }

    /// Authorize only exact model-facing bytes observed at the execution boundary.
    /// Receipts are process-local and bounded; recovered history has no authority.
    pub(crate) fn record_checkpoint_result(&self, name: &str, result: &MessageToolResult) {
        let Some(instance) = self.tools.instance_handle(name) else {
            return;
        };
        if let Ok(mut receipts) = self.checkpoint_result_receipts.lock() {
            if receipts.len() == 1024 {
                receipts.pop_front();
            }
            receipts.push_back(ResultReceipt {
                execution: crate::persistence_snapshot::RECOVERY_EXECUTION
                    .try_with(|id| *id)
                    .ok(),
                digest: result_identity(name, result),
                instance,
            });
        }
    }

    /// Project canonical messages for checkpoint generation. Arguments, reasoning
    /// and unverifiable tool payloads are excluded, including on recovered history.
    pub(crate) fn checkpoint_projection(
        &self,
        messages: &[Message],
        private_tokens: &PrivateTokens,
    ) -> crate::compaction::CompactionResult<Vec<Message>> {
        let mut private = private_tokens.clone();
        let mut pending = HashMap::new();
        let mut seen = HashSet::new();
        let mut ambiguous = HashSet::new();
        for message in messages {
            if let Message::Assistant { tool_calls, .. } = message {
                for call in tool_calls {
                    if !seen.insert(call.id.as_str()) {
                        ambiguous.insert(call.id.as_str());
                    }
                }
            }
        }
        // Gather declared private tokens before projecting any echoed assistant text.
        let mut results = HashMap::new();
        let mut projection_bytes = 0usize;
        for (index, message) in messages.iter().enumerate() {
            match message {
                Message::Assistant { tool_calls, .. } => {
                    pending.clear();
                    for call in tool_calls {
                        if !ambiguous.contains(call.id.as_str()) {
                            pending.insert(call.id.as_str(), call.name.as_str());
                        }
                    }
                }
                Message::Tool { result } => {
                    if let Some(tool) =
                        pending
                            .remove(result.tool_use_id.as_str())
                            .and_then(|name| {
                                let instance = self.tools.instance_handle(name)?;
                                let verified = self
                                    .checkpoint_result_receipts
                                    .lock()
                                    .map(|receipts| {
                                        let digest = result_identity(name, result);
                                        receipts.iter().any(|receipt| {
                                            receipt.digest == digest
                                                && receipt.instance.ptr_eq(&instance)
                                        })
                                    })
                                    .unwrap_or(false);
                                verified.then(|| self.tools.get(name)).flatten()
                            })
                    {
                        let execution = ToolResult {
                            content: result.content.clone(),
                            is_error: result.is_error,
                            continuations: Vec::new(),
                            todo_presentation: None,
                        };
                        let tokens = tool.model_private_tokens(&execution);
                        for token in &tokens {
                            charge_projection_bytes(&mut projection_bytes, token.len())?;
                        }
                        private.extend(tokens);
                        // This is already canonical model-facing input. Never read
                        // external raw_content or display/export-only data here.
                        let content = tool.project_result(&execution).model_content;
                        charge_projection_bytes(&mut projection_bytes, content.len())?;
                        results.insert(index, content);
                    }
                }
                _ => pending.clear(),
            }
        }
        Ok(messages
            .iter()
            .enumerate()
            .map(|(index, message)| match message {
                Message::Assistant {
                    content,
                    tool_calls,
                    ..
                } => Message::Assistant {
                    content: private.project(content),
                    tool_calls: tool_calls
                        .iter()
                        .map(|call| talos_core::message::ToolCall {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: serde_json::json!({}),
                        })
                        .collect(),
                    reasoning: None,
                },
                Message::Tool { result } => Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: result.tool_use_id.clone(),
                        content: results
                            .get(&index)
                            .map(|content| private.project(content))
                            .unwrap_or_else(|| {
                                "[tool payload excluded: unverifiable provenance]".into()
                            }),
                        is_error: result.is_error,
                    },
                },
                Message::User { content } => Message::User {
                    content: private.project(content),
                },
                Message::Context { content } => Message::Context {
                    content: private.project(content),
                },
                Message::System { content, .. } => {
                    let mut projected = message.clone();
                    if let Message::System { content: text, .. } = &mut projected {
                        *text = private.project(content);
                    }
                    projected
                }
                Message::Multimodal { parts } => Message::Multimodal {
                    // Summarization sees text only; it does not reload image payloads.
                    parts: parts
                        .iter()
                        .filter_map(|part| match part {
                            ContentPart::Text { text } => Some(ContentPart::Text {
                                text: private.project(text),
                            }),
                            _ => None,
                        })
                        .collect(),
                },
            })
            .collect())
    }
}

// Tool callbacks own their returned allocation. Reject excessive aggregate
// output before building the token trie or copying it into request projections.
fn charge_projection_bytes(
    total: &mut usize,
    bytes: usize,
) -> crate::compaction::CompactionResult<()> {
    *total = total
        .checked_add(bytes)
        .filter(|value| *value <= 2 * 1024 * 1024)
        .ok_or_else(|| {
            crate::compaction::CompactionError::CompactionFailed(
                "checkpoint tool projection limit exceeded".into(),
            )
        })?;
    Ok(())
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use talos_core::message::ToolCall;
    use talos_core::tool::{AgentTool, ToolRegistry, ToolResultProjection};

    struct NeverCalledModel;
    #[async_trait::async_trait]
    impl talos_core::provider::LanguageModel for NeverCalledModel {
        async fn stream(
            &self,
            _: &[Message],
        ) -> talos_core::provider::ProviderResult<
            talos_core::provider::Receiver<talos_core::message::AgentEvent>,
        > {
            panic!("projection validation must not call a provider")
        }
    }

    struct ExpandingTool {
        tokens: bool,
    }
    #[async_trait::async_trait]
    impl AgentTool for ExpandingTool {
        fn name(&self) -> &str {
            "expanding"
        }
        fn description(&self) -> &str {
            "projection bound fixture"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({})
        }
        async fn execute(&self, _: serde_json::Value) -> ToolResult {
            ToolResult::success("small")
        }
        fn model_private_tokens(&self, _: &ToolResult) -> Vec<String> {
            if self.tokens {
                vec!["x".repeat(2 * 1024 * 1024 + 1)]
            } else {
                vec![]
            }
        }
        fn project_result(&self, _: &ToolResult) -> ToolResultProjection {
            ToolResultProjection {
                model_content: "x".repeat(1_100_000),
                display_content: String::new(),
                persistence_content: String::new(),
            }
        }
    }

    #[test]
    fn rejects_tool_projection_expansion_and_private_token_expansion() {
        for tokens in [false, true] {
            let mut registry = ToolRegistry::new();
            registry.register(Arc::new(ExpandingTool { tokens }));
            let agent = Agent::new(Arc::new(NeverCalledModel), registry);
            let mut messages = Vec::new();
            for id in ["one", "two"] {
                let result = MessageToolResult {
                    tool_use_id: id.into(),
                    content: "small".into(),
                    is_error: false,
                };
                agent.record_checkpoint_result("expanding", &result);
                messages.push(Message::Assistant {
                    content: String::new(),
                    reasoning: None,
                    tool_calls: vec![ToolCall {
                        id: id.into(),
                        name: "expanding".into(),
                        input: serde_json::json!({}),
                    }],
                });
                messages.push(Message::Tool { result });
            }
            let original = messages.clone();
            let error = agent
                .checkpoint_projection(&messages, &PrivateTokens::default())
                .expect_err("aggregate bound");
            assert!(
                error
                    .to_string()
                    .contains("checkpoint tool projection limit exceeded")
            );
            assert_eq!(messages, original);
        }
    }
}
