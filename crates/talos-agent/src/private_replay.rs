//! Live Actor custody. No persistence, provider-global cache or cross-session discovery.

use crate::{AgentError, AgentResult};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use talos_core::message::Message;
use talos_core::provider::{ProviderError, TransientReplay};

const MAX_BLOCKS: usize = 32;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct PrivateReplayStore(Mutex<HashMap<String, Arc<TransientReplay>>>);

fn invalid() -> AgentError {
    AgentError::ProviderError(ProviderError::InvalidResponse(
        "private replay context is invalid or unavailable; start a new conversation".into(),
    ))
}

impl PrivateReplayStore {
    pub(crate) fn seed_tokens(&self) -> AgentResult<crate::transient_text::PrivateTokens> {
        let mut tokens = crate::transient_text::PrivateTokens::default();
        let store = self.0.lock().map_err(|_| invalid())?;
        if !tokens.extend_private_replay(
            store
                .values()
                .flat_map(|item| item.private_values().map(str::to_owned)),
        ) {
            return Err(invalid());
        }
        Ok(tokens)
    }

    /// Keep only committed/canonical markers; request-only compaction does not free custody.
    pub(crate) fn prune(&self, history: &[Message]) -> AgentResult<()> {
        let retained = history
            .iter()
            .filter_map(|message| {
                TransientReplay::marker(message)
                    .ok()
                    .flatten()
                    .map(|(marker, _)| marker)
            })
            .collect::<HashSet<_>>();
        self.0
            .lock()
            .map_err(|_| invalid())?
            .retain(|marker, _| retained.contains(marker.as_str()));
        Ok(())
    }

    pub(crate) fn insert(
        &self,
        replay: Arc<TransientReplay>,
        effective: &Message,
    ) -> AgentResult<()> {
        if !replay.matches(effective) {
            return Err(invalid());
        }
        let (marker, _) = TransientReplay::marker(effective)?.ok_or_else(invalid)?;
        let mut store = self.0.lock().map_err(|_| invalid())?;
        if store.contains_key(marker)
            || store.len() >= MAX_BLOCKS
            || store
                .values()
                .map(|item| item.byte_count() as usize)
                .sum::<usize>()
                .saturating_add(replay.byte_count() as usize)
                > MAX_BYTES
        {
            return Err(invalid());
        }
        store.insert(marker.into(), replay);
        Ok(())
    }

    /// Bind the final sealed request before provider selection/HTTP. A missing whole exchange
    /// may have been compacted; surviving partial IDs/results must still be refused.
    pub(crate) fn for_request(
        &self,
        messages: &[Message],
        provider: Option<&str>,
        model: Option<&str>,
    ) -> AgentResult<Vec<Arc<TransientReplay>>> {
        let store = self.0.lock().map_err(|_| invalid())?;
        let mut seen = HashSet::new();
        let mut replay = Vec::new();
        for message in messages {
            if let Some((marker, _)) = TransientReplay::marker(message)? {
                if !seen.insert(marker) {
                    return Err(invalid());
                }
                let item = store.get(marker).ok_or_else(invalid)?;
                let Message::Assistant {
                    reasoning: Some(identity),
                    ..
                } = message
                else {
                    return Err(invalid());
                };
                if !item.matches(message)
                    || Some(identity.provider.as_str()) != provider
                    || Some(identity.model.as_str()) != model
                {
                    return Err(invalid());
                }
                replay.push(item.clone());
            }
        }
        for (marker, item) in store.iter() {
            if seen.contains(marker.as_str()) {
                continue;
            }
            let Message::Assistant { tool_calls, .. } = item.projected_message() else {
                return Err(invalid());
            };
            if messages.iter().any(|message| match message {
                Message::Assistant {
                    tool_calls: surviving,
                    ..
                } => surviving
                    .iter()
                    .any(|call| tool_calls.iter().any(|bound| bound.id == call.id)),
                Message::Tool { result } => tool_calls
                    .iter()
                    .any(|bound| bound.id == result.tool_use_id),
                _ => false,
            }) {
                return Err(invalid());
            }
        }
        Ok(replay)
    }
}

impl crate::Agent {
    pub(crate) fn install_private_replay(
        &self,
        store: &PrivateReplayStore,
        replay: Arc<TransientReplay>,
        effective: &mut Message,
        private_tokens: &crate::transient_text::PrivateTokens,
        history: &[Message],
    ) -> AgentResult<()> {
        let Message::Assistant {
            reasoning: Some(identity),
            ..
        } = &*effective
        else {
            return Err(invalid());
        };
        if !replay.matches(effective)
            || Some(identity.provider.as_str()) != self.provider_key.as_deref()
            || Some(identity.model.as_str()) != self.model_id.as_deref()
        {
            return Err(invalid());
        }
        // Only the trusted persistence/privacy projection may change this view. Hook edits
        // were already compared above, before any tool or durable effect.
        let projected = self
            .persistence_projection_with_tokens(std::slice::from_ref(effective), private_tokens)
            .into_iter()
            .next()
            .ok_or_else(invalid)?;
        let replay = Arc::new(
            replay
                .with_public_projection(projected)?
                .with_private_values(private_tokens.values().map(str::to_owned))?,
        );
        *effective = replay.projected_message().clone();
        store.prune(history)?;
        store.insert(replay, effective)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{AssistantReasoning, MessageToolResult, ReasoningBlock, ToolCall};
    fn replay(id: &str) -> Arc<TransientReplay> {
        let public_call = ToolCall {
            id: id.into(),
            name: "browser".into(),
            input: serde_json::json!({"protocolVersion":2}),
        };
        let raw_call = ToolCall {
            input: serde_json::json!({"text":"private-sentinel"}),
            ..public_call.clone()
        };
        Arc::new(
            TransientReplay::new(
                Message::Assistant {
                    content: "safe".into(),
                    tool_calls: vec![public_call],
                    reasoning: None,
                },
                Message::Assistant {
                    content: "safe".into(),
                    tool_calls: vec![raw_call],
                    reasoning: Some(AssistantReasoning {
                        provider: "p".into(),
                        model: "m".into(),
                        blocks: vec![ReasoningBlock::Redacted {
                            data: "private-signed-parts".into(),
                        }],
                    }),
                },
            )
            .expect("replay"),
        )
    }
    #[test]
    fn live_custody_restores_identity_but_fresh_actor_rejects_markers() {
        let store = PrivateReplayStore::default();
        let replay = replay("b");
        let message = replay.projected_message().clone();
        store.insert(replay, &message).expect("insert");
        assert_eq!(
            store
                .for_request(std::slice::from_ref(&message), Some("p"), Some("m"))
                .expect("live")
                .len(),
            1
        );
        assert!(
            PrivateReplayStore::default()
                .for_request(std::slice::from_ref(&message), Some("p"), Some("m"))
                .is_err()
        );
        assert!(
            store
                .for_request(std::slice::from_ref(&message), Some("other"), Some("m"))
                .is_err()
        );
        assert!(
            store
                .for_request(&[message.clone(), message.clone()], Some("p"), Some("m"))
                .is_err()
        );
        let mut edited = message.clone();
        if let Message::Assistant { content, .. } = &mut edited {
            content.push_str(" hook edit");
        }
        assert!(store.for_request(&[edited], Some("p"), Some("m")).is_err());
        let mut forged = message;
        if let Message::Assistant {
            reasoning: Some(identity),
            ..
        } = &mut forged
        {
            if let ReasoningBlock::Redacted { data } = &mut identity.blocks[0] {
                data.push('0');
            }
        }
        assert!(store.for_request(&[forged], Some("p"), Some("m")).is_err());
    }
    #[test]
    fn canonical_pruning_drops_uncommitted_context_but_partial_request_fails() {
        let store = PrivateReplayStore::default();
        let replay = replay("b");
        let message = replay.projected_message().clone();
        store.insert(replay, &message).expect("insert");
        let result = Message::Tool {
            result: MessageToolResult {
                tool_use_id: "b".into(),
                content: "ok".into(),
                is_error: false,
            },
        };
        assert!(store.for_request(&[result], Some("p"), Some("m")).is_err());
        assert!(
            store
                .for_request(&[], Some("p"), Some("m"))
                .expect("whole exchange omitted")
                .is_empty()
        );
        store.prune(&[]).expect("cancel uncommitted");
        assert!(store.for_request(&[message], Some("p"), Some("m")).is_err());
    }
    #[test]
    fn hard_block_quota_refuses_without_replacing_live_entries() {
        let store = PrivateReplayStore::default();
        let mut history = Vec::new();
        for index in 0..MAX_BLOCKS {
            let item = replay(&format!("b{index}"));
            let message = item.projected_message().clone();
            store.insert(item, &message).expect("bounded");
            history.push(message);
        }
        let extra = replay("extra");
        let message = extra.projected_message().clone();
        assert!(store.insert(extra.clone(), &message).is_err());
        assert_eq!(
            store
                .for_request(&history, Some("p"), Some("m"))
                .expect("intact")
                .len(),
            MAX_BLOCKS
        );
        store.prune(&history[..1]).expect("canonical eviction");
        store
            .insert(extra, &message)
            .expect("freed canonical entries");
    }
    #[test]
    fn byte_quota_and_actor_private_filter_are_bounded() {
        let store = PrivateReplayStore::default();
        for index in 0..3 {
            let item = Arc::new(
                TransientReplay::new(
                    Message::Assistant {
                        content: "safe".into(),
                        tool_calls: vec![],
                        reasoning: None,
                    },
                    Message::Assistant {
                        content: String::new(),
                        tool_calls: vec![],
                        reasoning: Some(AssistantReasoning {
                            provider: "p".into(),
                            model: "m".into(),
                            blocks: vec![ReasoningBlock::Redacted {
                                data: "s".repeat(7 * 1024 * 1024),
                            }],
                        }),
                    },
                )
                .expect("bounded attachment"),
            );
            let message = item.projected_message().clone();
            let inserted = store.insert(item, &message);
            assert_eq!(inserted.is_ok(), index < 2);
        }
        let store = PrivateReplayStore::default();
        let item = replay("b");
        let message = item.projected_message().clone();
        store.insert(item, &message).expect("custody");
        let tokens = store.seed_tokens().expect("filter");
        assert_eq!(tokens.project("echo private-sentinel"), "echo [private]");
        assert!(
            crate::token::TokenEstimator::default().estimate(&[message])
                > "private-sentinel".len() as u32
        );
    }
}
