//! Eligible index text from host-authorized source entries.
//!
//! This module does not establish branch membership. The host must supply both
//! source and terminal marker from its validated branch snapshot before indexing.

use crate::{HistoryRecoveryReceipt, SessionEntry};

/// Build model-searchable text without indexing hidden metadata or tool inputs.
///
/// User and assistant text comes from the persisted representation. Assistant
/// tool-call blocks are removed; reasoning and raw metadata are never read. Tool
/// results require a receipt from the same committed turn. System instructions,
/// internal markers, unknown roles and unverifiable tools are ineligible.
///
/// This function is bounded by the source-page limit, not a model response limit.
/// The query layer must bound snippets/detail and revalidate current branch scope.
pub fn recovery_index_text(
    source: &SessionEntry,
    terminal: Option<&SessionEntry>,
) -> Option<String> {
    if source.content.len() > crate::history_page::HISTORY_PAGE_BYTES {
        return None;
    }
    match source.role.as_str() {
        "user" => Some(crate::durable::redact(&source.content)),
        "assistant" => Some(crate::durable::redact(
            &talos_core::message::strip_tool_syntax(&source.content),
        )),
        "system" => {
            let terminal = terminal?;
            if source.metadata.turn_id.is_none()
                || source.metadata.turn_id != terminal.metadata.turn_id
            {
                return None;
            }
            let receipts = HistoryRecoveryReceipt::from_terminal_entry(terminal)?;
            let mut matching = receipts.iter().filter(|r| r.entry_id() == source.id);
            let receipt = matching.next()?;
            if matching.next().is_some() {
                return None;
            }
            receipt.verified_content(source)
        }
        _ => None,
    }
}

/// Bounded eligible text for a recovery snippet or explicit detail response.
/// Branch authority and source freshness must be verified by the query host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryText {
    /// UTF-8 text with hidden tool syntax and unverified tool output excluded.
    pub text: String,
    /// Whether eligible text exceeded the requested byte limit.
    pub truncated: bool,
}

/// Requested disclosure shape, with fixed byte ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTextKind {
    /// Search result preview, at most 512 UTF-8 bytes.
    Snippet,
    /// Explicit original-entry detail, at most 8192 UTF-8 bytes.
    Detail,
}

/// Project an authoritative reread into a bounded snippet (up to 512 bytes) or
/// detail (up to 8192 bytes). The source/receipt rules are identical to indexing;
/// callers must never fall back to raw text when this returns None.
///
/// The kind selects a fixed ceiling. Truncation preserves UTF-8 boundaries and is
/// explicit; it does not claim that a truncated detail is the full original fact.
pub fn recovery_response_text(
    source: &SessionEntry,
    terminal: Option<&SessionEntry>,
    kind: RecoveryTextKind,
) -> Option<RecoveryText> {
    let max_bytes = match kind {
        RecoveryTextKind::Snippet => 512,
        RecoveryTextKind::Detail => 8192,
    };
    let mut text = recovery_index_text(source, terminal)?;
    let truncated = text.len() > max_bytes;
    if truncated {
        let mut end = max_bytes;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    Some(RecoveryText { text, truncated })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(role: &str, content: &str) -> SessionEntry {
        SessionEntry {
            id: "entry".into(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: crate::SessionMetadata {
                raw_content: Some("HIDDEN_RAW".into()),
                ..Default::default()
            },
        }
    }
    #[test]
    fn response_projection_bounds_utf8_and_never_falls_back_to_unverified_tool_text() {
        let source = entry("user", &"汉字".repeat(2000));
        for (kind, limit) in [
            (RecoveryTextKind::Snippet, 512),
            (RecoveryTextKind::Detail, 8192),
        ] {
            let response = recovery_response_text(&source, None, kind).expect("eligible");
            assert!(response.text.len() <= limit);
            assert!(response.truncated);
            assert!(source.content.starts_with(&response.text));
        }
        assert!(
            recovery_response_text(
                &entry("system", "unverified tool secret"),
                None,
                RecoveryTextKind::Snippet
            )
            .is_none()
        );
        let response = recovery_response_text(
            &entry("assistant", "visible decision"),
            None,
            RecoveryTextKind::Snippet,
        )
        .expect("assistant");
        assert_eq!(response.text, "visible decision");
        assert!(!response.truncated);
        assert!(!response.text.contains("HIDDEN_RAW"));
    }

    #[test]
    fn serialized_tool_arguments_with_fences_and_quotes_never_enter_index() {
        use talos_core::message::{Message, ToolCall};
        for secret in [
            "```HIDDEN_INPUT",
            "quote \" and ```HIDDEN_INPUT",
            "\n```\nHIDDEN_INPUT",
            "backslash \\ ```HIDDEN_INPUT",
        ] {
            let message = Message::Assistant {
                content: "visible decision".into(),
                reasoning: None,
                tool_calls: vec![ToolCall {
                    id: "call".into(),
                    name: "read".into(),
                    input: serde_json::json!({"text": secret}),
                }],
            };
            let (role, content) = crate::jsonl::message_parts(&message);
            assert!(content.contains("HIDDEN_INPUT"));
            let source = entry(&role, &content);
            assert_eq!(
                recovery_index_text(&source, None).as_deref(),
                Some("visible decision")
            );
            let truncated = content.strip_suffix("\n```").expect("closing fence");
            assert_eq!(
                recovery_index_text(&entry(&role, truncated), None).as_deref(),
                Some("visible decision")
            );
        }
    }

    #[test]
    fn index_projection_excludes_arguments_metadata_and_unverified_results() {
        let source = entry(
            "assistant",
            "public decision\n```json-tool\n{\"id\":\"call\",\"name\":\"read\",\"args\":{\"secret\":\"HIDDEN_INPUT\"}}\n```",
        );
        let text = recovery_index_text(&source, None).expect("assistant");
        assert!(text.contains("public decision"));
        assert_eq!(
            recovery_index_text(
                &entry("assistant", "kept```json-tool\nHIDDEN_TRUNCATED"),
                None
            )
            .as_deref(),
            Some("kept")
        );
        assert!(!text.contains("HIDDEN_INPUT"));
        assert!(!text.contains("HIDDEN_RAW"));
        assert!(
            recovery_index_text(&entry("system", "__OK__:call__\nHIDDEN_TOOL"), None).is_none()
        );
        assert!(recovery_index_text(&entry("system", "__SYSTEM__:instruction"), None).is_none());
        assert!(recovery_index_text(&entry("unknown-3", "private"), None).is_none());
    }
}
