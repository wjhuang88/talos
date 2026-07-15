//! Host-directory durable session binding and atomic turn persistence.

use std::fs;
use std::sync::Arc;

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use talos_core::message::{
    AssistantReasoning, Message, MessageToolResult, ReasoningBlock, ToolCall,
};
use uuid::Uuid;

use crate::jsonl::message_parts;
use crate::{Session, SessionEntry, SessionError, SessionMetadata};

/// One normalized, cursor-addressable durable transcript entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableTranscriptEntry {
    /// Stable entry identity.
    pub entry_id: String,
    /// Durable turn identity.
    pub turn_id: Option<String>,
    /// Entry timestamp.
    pub timestamp: chrono::DateTime<Utc>,
    /// Transcript role.
    pub role: String,
    /// Redacted model-visible content.
    pub content: String,
    /// Optional displayable reasoning when the policy allowed it.
    pub reasoning: Option<AssistantReasoning>,
    /// Tool call ID for a tool call or result.
    pub tool_call_id: Option<String>,
    /// Tool name for assistant tool calls.
    pub tool_name: Option<String>,
    /// Tool result text when this entry is a result.
    pub tool_result: Option<String>,
    /// Whether the tool result represents an error.
    pub is_error: bool,
    /// Parent entry relationship.
    pub parent_id: Option<String>,
}

/// Controls what a durable embedded transcript is allowed to retain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistencePolicy {
    /// Whether finalized assistant reasoning is retained. Defaults to `false`.
    pub persist_reasoning: bool,
    /// Whether model-visible tool result text is retained after redaction.
    pub persist_raw_tool_output: bool,
}

impl Default for PersistencePolicy {
    fn default() -> Self {
        Self {
            persist_reasoning: false,
            persist_raw_tool_output: true,
        }
    }
}

/// Format capabilities exposed to embedded hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCapabilities {
    /// Format used for all newly created durable sessions.
    pub write_format: String,
    /// Formats this runtime can read.
    pub readable_formats: Vec<String>,
    /// Current TLOG schema version.
    pub schema_version: u8,
}

/// Result of an idempotent durable turn commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCommit {
    /// Stable IDs for entries committed by the turn.
    pub entry_ids: Vec<String>,
    /// Whether this call wrote the turn rather than returning a prior commit.
    pub newly_committed: bool,
}

/// A UUID-backed durable session bound to an opaque host external ID.
#[derive(Debug, Clone)]
pub struct DurableSession {
    external_id: String,
    session: Session,
    bindings_path: std::path::PathBuf,
}

impl DurableSession {
    pub(crate) fn new(
        external_id: String,
        session: Session,
        bindings_path: std::path::PathBuf,
    ) -> Self {
        Self {
            external_id,
            session,
            bindings_path,
        }
    }

    /// Returns Talos's UUID identity used for the safe TLOG filename.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.session.id
    }

    /// Returns the host-provided logical key; it is never used as a filename.
    #[must_use]
    pub fn external_id(&self) -> &str {
        &self.external_id
    }

    /// Returns the UUID-named TLOG path under the host-selected directory.
    #[must_use]
    pub fn file_path(&self) -> &std::path::Path {
        &self.session.file_path
    }

    /// Returns model messages suitable for automatic Runtime history recovery.
    pub fn read_messages(&self) -> Result<Vec<Message>, SessionError> {
        self.session.read_messages()
    }

    /// Returns the underlying session for read-only inspection.
    #[must_use]
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Returns durable format capabilities without inferring support from extensions.
    #[must_use]
    pub fn capabilities(&self) -> SessionCapabilities {
        SessionCapabilities {
            write_format: "tlog".into(),
            readable_formats: vec!["tlog".into(), "jsonl".into()],
            schema_version: 1,
        }
    }

    /// Returns a normalized transcript page after an optional entry-ID cursor.
    ///
    /// The cursor entry is excluded. The method exposes no raw TLOG lines,
    /// provider payloads, HTTP headers, or `raw_content` metadata.
    pub fn transcript(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<DurableTranscriptEntry>, SessionError> {
        let entries = self.session.read_entries()?;
        let start = cursor
            .and_then(|cursor| {
                entries
                    .iter()
                    .position(|entry| entry.id == cursor)
                    .map(|index| index + 1)
            })
            .unwrap_or(0);
        Ok(entries
            .into_iter()
            .skip(start)
            .take(limit.min(200))
            .map(transcript_entry)
            .collect())
    }

    /// Atomically commits every model-visible message of one successful turn.
    ///
    /// `begin_turn` is intentionally implicit and non-durable: a cancelled,
    /// denied, or failed turn produces no transcript entry. Repeating a
    /// committed `turn_id` returns the original IDs without writing duplicates.
    pub fn commit_turn(
        &self,
        turn_id: &str,
        messages: &[Message],
        policy: &PersistencePolicy,
    ) -> Result<TurnCommit, SessionError> {
        if turn_id.is_empty() {
            return Err(SessionError::DurableTurn(
                "turn_id must not be empty".into(),
            ));
        }
        let _lock = self
            .session
            .write_lock
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let mut existing = self.session.store.read_entries(&self.session.file_path)?;
        let prior_ids = existing
            .iter()
            .filter(|entry| entry.metadata.turn_id.as_deref() == Some(turn_id))
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();
        if !prior_ids.is_empty() {
            return Ok(TurnCommit {
                entry_ids: prior_ids,
                newly_committed: false,
            });
        }

        let mut parent_id = existing.last().map(|entry| entry.id.clone());
        let mut entry_ids = Vec::with_capacity(messages.len());
        for message in messages {
            let message = filtered_message(message, policy);
            let (role, content) = message_parts(&message);
            let id = Uuid::new_v4().to_string();
            existing.push(SessionEntry {
                id: id.clone(),
                parent_id: parent_id.clone(),
                timestamp: Utc::now(),
                role,
                content,
                metadata: SessionMetadata {
                    turn_id: Some(turn_id.to_string()),
                    ..SessionMetadata::default()
                },
            });
            parent_id = Some(id.clone());
            entry_ids.push(id);
        }
        self.session
            .store
            .replace_entries_atomically(&self.session.file_path, &existing)?;
        Ok(TurnCommit {
            entry_ids,
            newly_committed: true,
        })
    }

    /// Marks an uncommitted turn as aborted. No transcript state is written.
    pub fn abort_turn(&self, turn_id: &str, _reason: &str) -> Result<(), SessionError> {
        if turn_id.is_empty() {
            return Err(SessionError::DurableTurn(
                "turn_id must not be empty".into(),
            ));
        }
        Ok(())
    }

    /// Deletes the TLOG and its external-ID binding.
    pub fn delete(self) -> Result<(), SessionError> {
        if self.session.file_path.exists() {
            fs::remove_file(&self.session.file_path)?;
        }
        let connection = open_bindings(&self.bindings_path)?;
        connection
            .execute(
                "DELETE FROM durable_bindings WHERE external_id = ?1",
                params![self.external_id],
            )
            .map_err(sql_error)?;
        Ok(())
    }
}

fn transcript_entry(entry: SessionEntry) -> DurableTranscriptEntry {
    let (is_error, tool_call_id, tool_result) = if entry.role == "system" {
        let (is_error, id, content) = crate::jsonl::parse_tool_result(&entry.content);
        (
            is_error,
            (id != "unknown").then_some(id),
            (entry.content.starts_with("__OK__:") || entry.content.starts_with("__ERROR__:"))
                .then_some(content),
        )
    } else {
        (false, None, None)
    };
    let tool_name = if entry.role == "assistant" {
        talos_core::message::extract_tool_calls_from_text(&entry.content)
            .first()
            .map(|call| call.name.clone())
    } else {
        None
    };
    DurableTranscriptEntry {
        entry_id: entry.id,
        turn_id: entry.metadata.turn_id.clone(),
        timestamp: entry.timestamp,
        role: entry.role,
        content: entry.content,
        reasoning: entry.metadata.reasoning,
        tool_call_id,
        tool_name,
        tool_result,
        is_error,
        parent_id: entry.parent_id,
    }
}

pub(crate) fn create_or_open(
    root: &std::path::Path,
    external_id: &str,
) -> Result<DurableSession, SessionError> {
    validate_external_id(external_id)?;
    fs::create_dir_all(root)?;
    let bindings_path = root.join("durable-bindings.sqlite");
    let mut connection = open_bindings(&bindings_path)?;
    let transaction = connection.transaction().map_err(sql_error)?;
    transaction
        .execute(
            "INSERT OR IGNORE INTO durable_bindings (external_id, session_id) VALUES (?1, ?2)",
            params![external_id, Uuid::new_v4().to_string()],
        )
        .map_err(sql_error)?;
    let bound_id: String = transaction
        .query_row(
            "SELECT session_id FROM durable_bindings WHERE external_id = ?1",
            params![external_id],
            |row| row.get(0),
        )
        .map_err(sql_error)?;
    let id = Uuid::parse_str(&bound_id)
        .map_err(|_| SessionError::DurableTurn("binding contains invalid UUID".into()))?;
    let session_dir = root.join("durable");
    fs::create_dir_all(&session_dir)?;
    let file_path = session_dir.join(format!("{id}.tlog"));
    let session = Session::with_store(
        id,
        "embedded".into(),
        "embedded".into(),
        file_path,
        Arc::new(crate::CompactTextSessionStore),
    );
    if !session.file_path.exists() {
        session
            .store
            .replace_entries_atomically(&session.file_path, &[])?;
    }
    transaction.commit().map_err(sql_error)?;
    Ok(DurableSession::new(
        external_id.to_string(),
        session,
        bindings_path,
    ))
}

pub(crate) fn get_by_external_id(
    root: &std::path::Path,
    external_id: &str,
) -> Result<Option<DurableSession>, SessionError> {
    validate_external_id(external_id)?;
    let bindings_path = root.join("durable-bindings.sqlite");
    if !bindings_path.exists() {
        return Ok(None);
    }
    let connection = open_bindings(&bindings_path)?;
    let id: Option<String> = connection
        .query_row(
            "SELECT session_id FROM durable_bindings WHERE external_id = ?1",
            params![external_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(sql_error)?;
    let Some(id) = id else {
        return Ok(None);
    };
    let id = Uuid::parse_str(&id)
        .map_err(|_| SessionError::DurableTurn("binding contains invalid UUID".into()))?;
    let file_path = root.join("durable").join(format!("{id}.tlog"));
    if !file_path.exists() {
        return Err(SessionError::SessionNotFound(id));
    }
    let session = Session::with_store(
        id,
        "embedded".into(),
        "embedded".into(),
        file_path,
        Arc::new(crate::CompactTextSessionStore),
    );
    Ok(Some(DurableSession::new(
        external_id.to_string(),
        session,
        bindings_path,
    )))
}

pub(crate) fn remove_binding_for_session(
    root: &std::path::Path,
    session_id: &Uuid,
) -> Result<(), SessionError> {
    let bindings_path = root.join("durable-bindings.sqlite");
    if !bindings_path.exists() {
        return Ok(());
    }
    let connection = open_bindings(&bindings_path)?;
    connection
        .execute(
            "DELETE FROM durable_bindings WHERE session_id = ?1",
            params![session_id.to_string()],
        )
        .map_err(sql_error)?;
    Ok(())
}

fn open_bindings(path: &std::path::Path) -> Result<Connection, SessionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = Connection::open(path).map_err(sql_error)?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(sql_error)?;
    connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS durable_bindings (external_id TEXT PRIMARY KEY, session_id TEXT NOT NULL UNIQUE);").map_err(sql_error)?;
    Ok(connection)
}

fn validate_external_id(external_id: &str) -> Result<(), SessionError> {
    if external_id.is_empty()
        || external_id.len() > 1024
        || external_id.contains('\0')
        || external_id.contains(['/', '\\'])
        || external_id.contains("..")
    {
        return Err(SessionError::InvalidExternalId(
            "must be non-empty, at most 1024 bytes, contain no NUL or path separators, and not contain '..'".into(),
        ));
    }
    Ok(())
}

fn sql_error(error: rusqlite::Error) -> SessionError {
    SessionError::DurableTurn(format!("binding index error: {error}"))
}

fn filtered_message(message: &Message, policy: &PersistencePolicy) -> Message {
    match message {
        Message::User { content } => Message::User {
            content: redact(content),
        },
        Message::Context { content } => Message::Context {
            content: redact(content),
        },
        Message::System {
            content,
            cache_markers,
        } => Message::System {
            content: redact(content),
            cache_markers: cache_markers.clone(),
        },
        Message::Assistant {
            content,
            tool_calls,
            reasoning,
        } => Message::Assistant {
            content: redact(content),
            tool_calls: tool_calls
                .iter()
                .map(|call| ToolCall {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    input: redact_json(&call.input),
                })
                .collect(),
            reasoning: if policy.persist_reasoning {
                reasoning.clone().map(redact_reasoning)
            } else {
                None
            },
        },
        Message::Tool { result } => Message::Tool {
            result: MessageToolResult {
                tool_use_id: result.tool_use_id.clone(),
                content: if policy.persist_raw_tool_output {
                    redact(&result.content)
                } else {
                    "[tool output omitted by persistence policy]".into()
                },
                is_error: result.is_error,
            },
        },
    }
}

fn redact_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(value) => serde_json::Value::String(redact(value)),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(redact_json).collect())
        }
        serde_json::Value::Object(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        if is_sensitive_key(key) {
                            serde_json::Value::String("[REDACTED]".into())
                        } else {
                            redact_json(value)
                        },
                    )
                })
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn redact_reasoning(reasoning: AssistantReasoning) -> AssistantReasoning {
    AssistantReasoning {
        provider: redact(&reasoning.provider),
        model: redact(&reasoning.model),
        blocks: reasoning
            .blocks
            .into_iter()
            .map(|block| match block {
                ReasoningBlock::Thinking { text, signature } => ReasoningBlock::Thinking {
                    text: redact(&text),
                    signature: signature.map(|value| redact(&value)),
                },
                ReasoningBlock::Redacted { data } => ReasoningBlock::Redacted {
                    data: redact(&data),
                },
                ReasoningBlock::Plain { text } => ReasoningBlock::Plain {
                    text: redact(&text),
                },
            })
            .collect(),
    }
}

fn redact(value: &str) -> String {
    value
        .lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if lower.contains("authorization:")
                || lower.contains("cookie:")
                || lower.contains("set-cookie:")
                || lower.contains("x-api-key:")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("api-key")
                || lower.contains("token=")
                || lower.contains("bearer ")
                || lower.contains("sk-")
            {
                "[REDACTED]".into()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("token")
        || key.contains("api_key")
        || key.contains("apikey")
        || key.contains("authorization")
        || key.contains("cookie")
        || key.contains("password")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colon_external_id_maps_to_uuid_tlog_and_is_idempotent() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let first = create_or_open(
            directory.path(),
            "assistant:8c8cb03a-9a54-43c2-93a9-42f5d0a3bf56",
        )
        .expect("first open");
        let second = create_or_open(
            directory.path(),
            "assistant:8c8cb03a-9a54-43c2-93a9-42f5d0a3bf56",
        )
        .expect("second open");
        assert_eq!(first.id(), second.id());
        assert_eq!(
            first
                .file_path()
                .extension()
                .and_then(|value| value.to_str()),
            Some("tlog")
        );
        let filename = first.id().to_string();
        assert_eq!(
            first
                .file_path()
                .file_stem()
                .and_then(|value| value.to_str()),
            Some(filename.as_str())
        );
        assert!(!first.file_path().to_string_lossy().contains("assistant:"));
    }

    #[test]
    fn atomic_turn_is_idempotent_and_redacts_credentials() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:one").expect("session");
        let messages = vec![
            Message::User {
                content: "Authorization: Bearer token-value\napi_key=sk-secret\nx-api-key: key-value\nplain sk-live-secret".into(),
            },
            Message::Assistant {
                content: "Cookie: session=secret".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ];
        let first = session
            .commit_turn("turn-1", &messages, &PersistencePolicy::default())
            .expect("commit");
        let retry = session
            .commit_turn("turn-1", &messages, &PersistencePolicy::default())
            .expect("retry");
        assert!(first.newly_committed);
        assert!(!retry.newly_committed);
        assert_eq!(first.entry_ids, retry.entry_ids);
        let disk = std::fs::read_to_string(session.file_path()).expect("TLOG readable");
        assert!(!disk.contains("token-value"));
        assert!(!disk.contains("sk-secret"));
        assert!(!disk.contains("key-value"));
        assert!(!disk.contains("sk-live-secret"));
        assert!(!disk.contains("session=secret"));
        assert_eq!(session.transcript(None, 20).expect("transcript").len(), 2);
    }

    #[test]
    fn external_id_rejects_path_traversal_and_concurrent_opens_share_one_uuid() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        for invalid in ["../task", "task/one", "task\\one"] {
            assert!(matches!(
                create_or_open(directory.path(), invalid),
                Err(SessionError::InvalidExternalId(_))
            ));
        }

        let root = directory.path().to_path_buf();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(6));
        let handles = (0..6)
            .map(|_| {
                let root = root.clone();
                let barrier = std::sync::Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    create_or_open(&root, "task:concurrent")
                        .expect("concurrent open")
                        .id()
                })
            })
            .collect::<Vec<_>>();
        let ids = handles
            .into_iter()
            .map(|handle| handle.join().expect("worker did not panic"))
            .collect::<Vec<_>>();
        assert!(ids.iter().all(|id| *id == ids[0]));
    }

    #[test]
    fn abort_and_delete_leave_no_entries_or_stale_binding() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:abort").expect("session");
        session
            .abort_turn("turn-aborted", "cancelled")
            .expect("abort");
        assert!(session.transcript(None, 10).expect("transcript").is_empty());
        let id = session.id();
        session.delete().expect("delete");
        assert!(
            get_by_external_id(directory.path(), "task:abort")
                .expect("lookup")
                .is_none()
        );
        assert!(
            !directory
                .path()
                .join("durable")
                .join(format!("{id}.tlog"))
                .exists()
        );
    }

    #[test]
    fn transcript_reconstructs_tool_messages_and_policy_can_omit_output() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:tools").expect("session");
        let messages = vec![
            Message::Assistant {
                content: String::new(),
                tool_calls: vec![ToolCall {
                    id: "call-1".into(),
                    name: "fixture".into(),
                    input: serde_json::json!({"Authorization": "Bearer secret"}),
                }],
                reasoning: None,
            },
            Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "call-1".into(),
                    content: "Cookie: secret".into(),
                    is_error: false,
                },
            },
        ];
        session
            .commit_turn(
                "turn-tools",
                &messages,
                &PersistencePolicy {
                    persist_raw_tool_output: false,
                    ..PersistencePolicy::default()
                },
            )
            .expect("commit");
        let entries = session.transcript(None, 10).expect("transcript");
        assert_eq!(entries[0].tool_name.as_deref(), Some("fixture"));
        assert_eq!(entries[1].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(
            entries[1].tool_result.as_deref(),
            Some("[tool output omitted by persistence policy]")
        );
    }

    #[test]
    fn persistence_failure_is_returned_without_a_successful_commit() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:write-failure").expect("session");
        std::fs::remove_file(session.file_path()).expect("remove initial TLOG");
        std::fs::remove_dir(directory.path().join("durable")).expect("remove durable directory");
        std::fs::write(directory.path().join("durable"), "not a directory")
            .expect("block durable directory");

        let result = session.commit_turn(
            "turn-failure",
            &[Message::User {
                content: "must not commit".into(),
            }],
            &PersistencePolicy::default(),
        );
        assert!(result.is_err());
    }
}
