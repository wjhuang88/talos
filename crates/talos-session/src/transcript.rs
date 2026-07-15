//! Transcript export service — format-neutral JSON and Markdown export.
//!
//! Reads session entries via the [`SessionStore`] abstraction, so both
//! `.jsonl` and `.tlog` sessions are supported transparently.

use crate::{SessionEntry, SessionError, SessionMetadata, SessionStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A flattened transcript entry suitable for export.
///
/// Unlike [`SessionEntry`], this omits internal fields (`id`, `parent_id`)
/// and presents only the conversation-relevant data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    /// The role of this entry: `"user"`, `"assistant"`, or `"system"`.
    pub role: String,

    /// The content of this entry.
    pub content: String,

    /// When this entry was created. `None` for entries without a timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    /// Optional metadata (provider, model, token count, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SessionMetadata>,
}

impl From<&SessionEntry> for TranscriptEntry {
    fn from(entry: &SessionEntry) -> Self {
        let metadata = if entry.metadata.is_empty() {
            None
        } else {
            Some(entry.metadata.clone())
        };
        Self {
            role: entry.role.clone(),
            content: entry.content.clone(),
            timestamp: Some(entry.timestamp),
            metadata,
        }
    }
}

/// Export session entries as a structured JSON array.
///
/// Each element contains `role`, `content`, `timestamp`, and optionally `metadata`.
pub fn export_json(entries: &[SessionEntry]) -> Result<String, serde_json::Error> {
    let transcript: Vec<TranscriptEntry> = entries.iter().map(TranscriptEntry::from).collect();
    serde_json::to_string_pretty(&transcript)
}

/// Export session entries as a human-readable Markdown transcript.
///
/// Format:
/// ```text
/// ## User
/// [content]
///
/// ## Assistant
/// [content]
///
/// ## System
/// [content]
/// ```
pub fn export_markdown(entries: &[SessionEntry]) -> String {
    export_markdown_filtered(entries, false)
}

#[allow(dead_code)]
pub fn export_markdown_with_thinking(entries: &[SessionEntry]) -> String {
    export_markdown_filtered(entries, true)
}

fn export_markdown_filtered(entries: &[SessionEntry], include_thinking: bool) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

        if include_thinking
            && let Some(ref reasoning) = entry.metadata.reasoning
            && let Some(text) = talos_core::message::project_displayable_reasoning(reasoning)
        {
            output.push_str("## Thinking\n");
            for line in text.lines() {
                output.push_str(&format!("| {line}\n"));
            }
            output.push_str("\n\n");
        }

        let role_header = match entry.role.as_str() {
            "user" => "User",
            "assistant" => "Assistant",
            "system" => "System",
            other => other,
        };
        output.push_str(&format!("## {}\n{}", role_header, entry.content));
    }
    output.push('\n');
    output
}

/// Read a session file via the given store and return transcript entries.
///
/// Works with any [`SessionStore`] implementation, so both `.jsonl` and
/// `.tlog` sessions are supported.
pub fn read_transcript(
    store: &dyn SessionStore,
    file_path: &Path,
) -> Result<Vec<TranscriptEntry>, SessionError> {
    let entries = store.read_entries(file_path)?;
    Ok(entries.iter().map(TranscriptEntry::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionEntry;
    use crate::store::{CompactTextSessionStore, JsonlSessionStore};
    use chrono::Utc;
    use talos_core::message::{AssistantReasoning, ReasoningBlock};
    use uuid::Uuid;

    fn make_entry(role: &str, content: &str) -> SessionEntry {
        SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: SessionMetadata::default(),
        }
    }

    fn make_entry_with_metadata(role: &str, content: &str) -> SessionEntry {
        SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: Utc::now(),
            role: role.into(),
            content: content.into(),
            metadata: SessionMetadata {
                turn_id: None,
                provider: Some("anthropic".into()),
                model: Some("claude-sonnet-4-20250514".into()),
                token_count: Some(42),
                working_directory: Some("/tmp/project".into()),
                reasoning: None,
                raw_content: None,
            },
        }
    }

    fn make_entry_with_reasoning(content: &str, reasoning_text: &str) -> SessionEntry {
        let mut entry = make_entry("assistant", content);
        entry.metadata.reasoning = Some(AssistantReasoning {
            provider: "fixture-provider".into(),
            model: "fixture-model".into(),
            blocks: vec![ReasoningBlock::Plain {
                text: reasoning_text.into(),
            }],
        });
        entry
    }

    // --- export_json tests ---

    #[test]
    fn export_json_produces_valid_json() {
        let entries = vec![
            make_entry("user", "Hello"),
            make_entry("assistant", "Hi there"),
        ];
        let json = export_json(&entries).unwrap();
        // Should parse back as a JSON array.
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn export_json_has_correct_fields() {
        let entries = vec![make_entry("user", "test content")];
        let json = export_json(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

        let entry = &parsed[0];
        assert_eq!(entry["role"], "user");
        assert_eq!(entry["content"], "test content");
        assert!(entry.get("timestamp").is_some());
    }

    #[test]
    fn export_json_empty_produces_empty_array() {
        let entries: Vec<SessionEntry> = Vec::new();
        let json = export_json(&entries).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn export_json_preserves_metadata() {
        let entries = vec![make_entry_with_metadata("assistant", "response")];
        let json = export_json(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

        let meta = &parsed[0]["metadata"];
        assert_eq!(meta["provider"], "anthropic");
        assert_eq!(meta["model"], "claude-sonnet-4-20250514");
        assert_eq!(meta["token_count"], 42);
        assert_eq!(meta["working_directory"], "/tmp/project");
    }

    #[test]
    fn export_json_omits_empty_metadata() {
        let entries = vec![make_entry("user", "no metadata")];
        let json = export_json(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

        // metadata should be absent when empty
        assert!(parsed[0].get("metadata").is_none());
    }

    // --- export_markdown tests ---

    #[test]
    fn export_markdown_produces_role_headers() {
        let entries = vec![
            make_entry("user", "Hello"),
            make_entry("assistant", "Hi there"),
            make_entry("system", "Event"),
        ];
        let md = export_markdown(&entries);

        assert!(md.contains("## User"));
        assert!(md.contains("## Assistant"));
        assert!(md.contains("## System"));
    }

    #[test]
    fn export_markdown_contains_content() {
        let entries = vec![make_entry("user", "Hello, world!")];
        let md = export_markdown(&entries);
        assert!(md.contains("Hello, world!"));
    }

    #[test]
    fn export_markdown_empty_produces_empty_string() {
        let entries: Vec<SessionEntry> = Vec::new();
        let md = export_markdown(&entries);
        assert!(md.is_empty());
    }

    #[test]
    fn export_markdown_separates_entries() {
        let entries = vec![
            make_entry("user", "first"),
            make_entry("assistant", "second"),
        ];
        let md = export_markdown(&entries);
        // Second entry should be separated by blank lines
        assert!(md.contains("\n\n## Assistant"));
    }

    #[test]
    fn export_markdown_excludes_reasoning_and_derived_thinking_title_by_default() {
        let entries = vec![make_entry_with_reasoning(
            "Visible answer",
            "**Secret Title**\n\nreasoning about api_key=sk-leak",
        )];

        let md = export_markdown(&entries);

        assert!(md.contains("Visible answer"));
        assert!(!md.contains("## Thinking"));
        assert!(!md.contains("Secret Title"));
        assert!(!md.contains("reasoning about"));
        assert!(!md.contains("sk-leak"));
    }

    #[test]
    fn export_markdown_with_thinking_keeps_explicit_opt_in_behavior() {
        let entries = vec![make_entry_with_reasoning(
            "Visible answer",
            "**Section Title**\n\nreasoning details",
        )];

        let md = export_markdown_with_thinking(&entries);

        assert!(md.contains("## Thinking"));
        assert!(md.contains("**Section Title**"));
        assert!(md.contains("reasoning details"));
        assert!(md.contains("Visible answer"));
    }

    // --- read_transcript tests ---

    #[test]
    fn read_transcript_works_with_jsonl_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");

        let store = JsonlSessionStore;
        let entry = make_entry("user", "Hello from JSONL");
        store.append_entry(&path, &entry).unwrap();

        let transcript = read_transcript(&store, &path).unwrap();
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].role, "user");
        assert_eq!(transcript[0].content, "Hello from JSONL");
    }

    #[test]
    fn read_transcript_works_with_tlog_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.tlog");

        let store = CompactTextSessionStore;
        let entry = make_entry("assistant", "Hello from TLOG");
        store.append_entry(&path, &entry).unwrap();

        let transcript = read_transcript(&store, &path).unwrap();
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].role, "assistant");
        assert_eq!(transcript[0].content, "Hello from TLOG");
    }

    #[test]
    fn read_transcript_empty_session() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.jsonl");

        let store = JsonlSessionStore;
        let transcript = read_transcript(&store, &path).unwrap();
        assert!(transcript.is_empty());
    }

    #[test]
    fn read_transcript_multiple_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.tlog");

        let store = CompactTextSessionStore;
        let entries = vec![
            make_entry("user", "Question"),
            make_entry("assistant", "Answer"),
            make_entry("system", "Event"),
        ];
        for e in &entries {
            store.append_entry(&path, e).unwrap();
        }

        let transcript = read_transcript(&store, &path).unwrap();
        assert_eq!(transcript.len(), 3);
        assert_eq!(transcript[0].role, "user");
        assert_eq!(transcript[1].role, "assistant");
        assert_eq!(transcript[2].role, "system");
    }

    #[test]
    fn read_transcript_preserves_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.jsonl");

        let store = JsonlSessionStore;
        let entry = make_entry_with_metadata("assistant", "with metadata");
        store.append_entry(&path, &entry).unwrap();

        let transcript = read_transcript(&store, &path).unwrap();
        assert_eq!(transcript.len(), 1);
        let meta = transcript[0].metadata.as_ref().unwrap();
        assert_eq!(meta.provider, Some("anthropic".into()));
        assert_eq!(meta.model, Some("claude-sonnet-4-20250514".into()));
    }

    // --- round-trip tests ---

    #[test]
    fn export_json_round_trip() {
        let entries = vec![
            make_entry("user", "Hello"),
            make_entry_with_metadata("assistant", "Response"),
        ];
        let json = export_json(&entries).unwrap();
        let parsed: Vec<TranscriptEntry> = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].role, "user");
        assert_eq!(parsed[0].content, "Hello");
        assert_eq!(parsed[1].role, "assistant");
        assert_eq!(parsed[1].content, "Response");
        assert!(parsed[1].metadata.is_some());
    }

    #[test]
    fn export_markdown_round_trip_readability() {
        let entries = vec![
            make_entry("user", "What is Rust?"),
            make_entry("assistant", "Rust is a systems programming language."),
        ];
        let md = export_markdown(&entries);

        // Should be human-readable
        assert!(md.contains("## User"));
        assert!(md.contains("What is Rust?"));
        assert!(md.contains("## Assistant"));
        assert!(md.contains("Rust is a systems programming language."));
    }
}
