//! Runtime-issued recovery provenance stored in the existing terminal marker.
//!
//! Receipts bind final persisted representations. They are not signatures against
//! a host rewriting its own transcript and do not authorize cross-branch access.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use talos_core::message::Message;

use crate::SessionEntry;

/// Runtime-approved model projection for a tool result being committed.
///
/// Only the execution owner may supply this value after tool/privacy projection.
/// Storage accepts it only when it matches the final policy-filtered tool
/// text after the persistence layer's line-ending normalization. Merely observing
/// matching text in a recovered transcript is insufficient.
#[derive(Debug, Clone)]
pub struct RecoveryProjection {
    /// Position in the messages passed to the atomic turn commit.
    pub message_index: usize,
    /// Invocation ID established by the matching assistant call.
    pub tool_call_id: String,
    /// Registered tool identity established by the execution owner.
    pub tool_name: String,
    /// Final model-visible projection; not hidden raw output or arguments.
    pub content: String,
}

/// A committed binding to an eligible tool result's final stored representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRecoveryReceipt {
    version: u8,
    entry_id: String,
    turn_id: String,
    tool_call_id: String,
    tool_name: String,
    stored_digest: String,
    projection_digest: String,
}

impl HistoryRecoveryReceipt {
    /// Decode receipts from a host-validated committed terminal entry.
    /// This parses provenance; it does not establish the marker's branch scope.
    pub fn from_terminal_entry(entry: &SessionEntry) -> Option<Vec<Self>> {
        if entry.role != "system" || entry.content.len() > crate::history_page::HISTORY_PAGE_BYTES {
            return None;
        }
        let marker = crate::turn_outcome::decode_turn_transcript_outcome(&entry.content)?;
        if marker.version != 1 || entry.metadata.turn_id.as_deref() != Some(marker.turn_id.as_str())
        {
            return None;
        }
        let receipts = decode_receipts(&entry.content)?;
        if receipts.len() > 8192
            || receipts
                .iter()
                .any(|r| r.version != 1 || r.turn_id != marker.turn_id)
        {
            return None;
        }
        Some(receipts)
    }

    /// Original authoritative entry ID; this alone is not branch authorization.
    pub fn entry_id(&self) -> &str {
        &self.entry_id
    }

    /// Verify a host-selected source entry before returning its eligible text.
    /// The host must separately establish session/branch scope and marker origin.
    pub fn verified_content(&self, entry: &SessionEntry) -> Option<String> {
        if entry.content.len() > crate::history_page::HISTORY_PAGE_BYTES
            || self.version != 1
            || self.entry_id != entry.id
            || entry.metadata.turn_id.as_deref() != Some(self.turn_id.as_str())
            || entry.role != "system"
            || digest(&entry.content) != self.stored_digest
        {
            return None;
        }
        let (_, call_id, content) = crate::jsonl::parse_tool_result(&entry.content);
        (call_id == self.tool_call_id && digest(&content) == self.projection_digest)
            .then_some(content)
    }
}

pub(crate) fn bind_receipts(
    messages: &[Message],
    entries: &[SessionEntry],
    projections: &[RecoveryProjection],
    permit_tool_persistence: bool,
) -> Result<Vec<HistoryRecoveryReceipt>, crate::SessionError> {
    if !permit_tool_persistence {
        return Ok(Vec::new());
    }
    if projections.len() > 8192 {
        return Err(crate::SessionError::DurableTurn(
            "too many recovery projections".into(),
        ));
    }
    let mut receipts = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for projection in projections {
        if projection.content.len() > 8192
            || projection.tool_call_id.len() > 1024
            || projection.tool_name.len() > 256
            || !seen.insert(projection.message_index)
        {
            return Err(crate::SessionError::DurableTurn(
                "invalid recovery projection bounds".into(),
            ));
        }
        let Some(Message::Tool { result }) = messages.get(projection.message_index) else {
            return Err(crate::SessionError::DurableTurn(
                "recovery projection does not name a tool result".into(),
            ));
        };
        let Some(entry) = entries.get(projection.message_index) else {
            return Err(crate::SessionError::DurableTurn(
                "recovery source entry is missing".into(),
            ));
        };
        // The durable filter canonicalizes line endings even without redaction.
        // Permit only that normalization; sensitive-line replacement still differs
        // and cannot promote omitted/redacted content into recovery authority.
        let canonical = projection.content.lines().collect::<Vec<_>>().join("\n");
        if canonical != result.content || projection.tool_call_id != result.tool_use_id {
            continue;
        }
        let matching_call = messages[..projection.message_index]
            .iter()
            .rev()
            .find_map(|message| {
                if let Message::Assistant { tool_calls, .. } = message {
                    Some(
                        tool_calls
                            .iter()
                            .filter(|call| call.id == projection.tool_call_id)
                            .count()
                            == 1
                            && tool_calls.iter().any(|call| {
                                call.id == projection.tool_call_id
                                    && call.name == projection.tool_name
                            }),
                    )
                } else if let Message::Tool { result } = message {
                    (result.tool_use_id == projection.tool_call_id).then_some(false)
                } else {
                    Some(false)
                }
            })
            .unwrap_or(false);
        if !matching_call {
            return Err(crate::SessionError::DurableTurn(
                "recovery invocation does not match".into(),
            ));
        }
        receipts.push(HistoryRecoveryReceipt {
            version: 1,
            entry_id: entry.id.clone(),
            turn_id: entry.metadata.turn_id.clone().unwrap_or_default(),
            tool_call_id: projection.tool_call_id.clone(),
            tool_name: projection.tool_name.clone(),
            stored_digest: digest(&entry.content),
            projection_digest: digest(&result.content),
        });
    }
    Ok(receipts)
}

pub(crate) fn decode_receipts(marker: &str) -> Option<Vec<HistoryRecoveryReceipt>> {
    let encoded = marker.strip_prefix(crate::turn_outcome::TURN_TRANSCRIPT_OUTCOME_PREFIX)?;
    let value: serde_json::Value = serde_json::from_str(encoded).ok()?;
    if value.get("version")?.as_u64()? != 1 {
        return None;
    }
    let Some(receipts) = value.get("history_recovery") else {
        return Some(Vec::new());
    };
    serde_json::from_value(receipts.clone()).ok()
}

fn digest(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{MessageToolResult, ToolCall};

    #[test]
    fn ambiguous_or_cross_user_tool_pairing_cannot_mint_receipts() {
        let call = ToolCall {
            id: "call".into(),
            name: "read".into(),
            input: serde_json::json!({}),
        };
        let result = Message::Tool {
            result: MessageToolResult {
                tool_use_id: "call".into(),
                content: "fact".into(),
                is_error: false,
            },
        };
        let assistant = Message::Assistant {
            content: String::new(),
            reasoning: None,
            tool_calls: vec![call.clone()],
        };
        let cases = vec![
            vec![
                Message::Assistant {
                    content: String::new(),
                    reasoning: None,
                    tool_calls: vec![call.clone(), call],
                },
                result.clone(),
            ],
            vec![
                assistant.clone(),
                Message::User {
                    content: "new request".into(),
                },
                result.clone(),
            ],
            vec![assistant, result.clone(), result],
        ];
        for messages in cases {
            let entries = messages
                .iter()
                .enumerate()
                .map(|(i, message)| {
                    let (role, content) = crate::jsonl::message_parts(message);
                    SessionEntry {
                        id: i.to_string(),
                        parent_id: None,
                        timestamp: chrono::Utc::now(),
                        role,
                        content,
                        metadata: crate::SessionMetadata {
                            turn_id: Some("turn".into()),
                            ..Default::default()
                        },
                    }
                })
                .collect::<Vec<_>>();
            let projection = RecoveryProjection {
                message_index: messages.len() - 1,
                tool_call_id: "call".into(),
                tool_name: "read".into(),
                content: "fact".into(),
            };
            assert!(bind_receipts(&messages, &entries, &[projection], true).is_err());
        }
    }
}
