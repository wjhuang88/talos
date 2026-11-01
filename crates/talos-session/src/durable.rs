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

use crate::diagnostic::is_terminal_diagnostic_content;
use crate::jsonl::message_parts;
use crate::turn_outcome::{
    decode_turn_transcript_outcome, encode_turn_transcript_outcome,
    is_turn_transcript_outcome_content,
};
use crate::{
    Session, SessionEntry, SessionError, SessionMetadata, TurnTranscriptOutcome,
    TurnTranscriptOutcomeRecord,
};

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
    /// Stable IDs for model-visible entries committed by the turn.
    pub entry_ids: Vec<String>,
    /// Whether this call wrote the turn rather than returning a prior commit.
    pub newly_committed: bool,
}

/// A storage-issued recovery snapshot with immutable session/entry identities.
/// Only a durable read can construct it; caller-authored message/ID pairs cannot
/// be relabeled as provenance from another configured session.
#[derive(Clone)]
pub struct DurableHistorySnapshot {
    session_id: Uuid,
    entries: Vec<(String, Message)>,
}

impl DurableHistorySnapshot {
    /// Identity of the session read to produce this snapshot.
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Model messages paired with original entry IDs in recovery order.
    pub fn entries(&self) -> &[(String, Message)] {
        &self.entries
    }
}

/// A UUID-backed durable session bound to an opaque host external ID.
#[derive(Debug, Clone)]
pub struct DurableSession {
    external_id: String,
    session: Session,
    bindings_path: std::path::PathBuf,
}

impl DurableSession {
    /// Lists existing host-owned durable identities without creating bindings.
    pub fn list_external_ids(root: &std::path::Path) -> Result<Vec<String>, SessionError> {
        let bindings_path = root.join("durable-bindings.sqlite");
        if !bindings_path.exists() {
            return Ok(Vec::new());
        }
        let connection = open_bindings(&bindings_path)?;
        let mut statement = connection
            .prepare("SELECT external_id FROM durable_bindings ORDER BY external_id")
            .map_err(sql_error)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(sql_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
    }

    /// Opens or creates a durable session for a host-owned external identity.
    pub fn open_or_create(root: &std::path::Path, external_id: &str) -> Result<Self, SessionError> {
        create_or_open(root, external_id)
    }

    /// Opens an existing durable session without creating a new binding.
    pub fn open_existing(
        root: &std::path::Path,
        external_id: &str,
    ) -> Result<Option<Self>, SessionError> {
        get_by_external_id(root, external_id)
    }

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

    /// Read recovered model messages with their original TLOG entry identities.
    /// Entries omitted from model history are omitted from this mapping as well.
    pub fn read_messages_with_entry_ids(&self) -> Result<Vec<(String, Message)>, SessionError> {
        self.session.read_messages_with_entry_ids()
    }

    /// Read one immutable session-qualified recovery snapshot.
    pub fn read_history_snapshot(&self) -> Result<DurableHistorySnapshot, SessionError> {
        Ok(DurableHistorySnapshot {
            session_id: self.id(),
            entries: self.read_messages_with_entry_ids()?,
        })
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
    /// provider payloads, HTTP headers, `raw_content` metadata, or hidden
    /// terminal outcome markers.
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
            .filter(|entry| {
                !is_terminal_diagnostic_content(&entry.content)
                    && !is_turn_transcript_outcome_content(&entry.content)
            })
            .take(limit.min(200))
            .map(transcript_entry)
            .collect())
    }

    /// Returns the durable terminal outcome for a turn without exposing marker entries.
    pub fn turn_outcome(
        &self,
        turn_id: &str,
    ) -> Result<Option<TurnTranscriptOutcome>, SessionError> {
        Ok(self
            .session
            .read_turn_transcript_outcomes()?
            .into_iter()
            .find(|record| record.turn_id == turn_id)
            .map(|record| record.outcome))
    }

    /// Atomically commits every model-visible message and the hidden Success
    /// marker of one completed turn.
    ///
    /// Repeating a committed `turn_id` returns the original model-visible IDs
    /// without writing duplicates. The outcome marker is written in the same
    /// atomic replacement as the messages so startup can distinguish a proven
    /// Success from partial or ambiguous transcript state.
    pub fn commit_turn(
        &self,
        turn_id: &str,
        messages: &[Message],
        policy: &PersistencePolicy,
    ) -> Result<TurnCommit, SessionError> {
        self.finalize_turn(turn_id, messages, policy, TurnTranscriptOutcome::Success)
    }

    /// Atomically finalizes one turn with its admitted display-safe prefix.
    ///
    /// The first terminal outcome wins. Retrying the same outcome with the same
    /// filtered role/content sequence returns the original entry IDs without a
    /// write. A conflicting outcome or payload is rejected without mutation.
    pub fn finalize_turn(
        &self,
        turn_id: &str,
        messages: &[Message],
        policy: &PersistencePolicy,
        outcome: TurnTranscriptOutcome,
    ) -> Result<TurnCommit, SessionError> {
        self.finalize_turn_with_recovery(turn_id, messages, policy, outcome, &[])
    }

    /// Atomically finalize a turn and runtime-approved recovery provenance.
    ///
    /// Projections must originate from the live execution owner. Only exact matches
    /// to final policy-filtered tool text receive a receipt; denied persistence or
    /// changed/redacted projections cannot confer recovery permission. The receipt
    /// is stored in the existing terminal marker, never a second transcript store.
    pub fn finalize_turn_with_recovery(
        &self,
        turn_id: &str,
        messages: &[Message],
        policy: &PersistencePolicy,
        outcome: TurnTranscriptOutcome,
        projections: &[crate::RecoveryProjection],
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
        let prior_entries = existing
            .iter()
            .filter(|entry| {
                entry.metadata.turn_id.as_deref() == Some(turn_id)
                    && !is_turn_transcript_outcome_content(&entry.content)
            })
            .collect::<Vec<_>>();
        let prior_ids = prior_entries
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();
        let mut markers = existing.iter().filter(|entry| {
            is_turn_transcript_outcome_content(&entry.content)
                && (entry.metadata.turn_id.as_deref() == Some(turn_id)
                    || decode_turn_transcript_outcome(&entry.content)
                        .is_some_and(|record| record.turn_id == turn_id))
        });
        let marker = markers.next();
        if markers.next().is_some() {
            return Err(SessionError::DurableTurn(
                "duplicate terminal markers".into(),
            ));
        }

        let filtered = messages
            .iter()
            .map(|message| filtered_message(message, policy))
            .collect::<Vec<_>>();
        let requested_parts = filtered.iter().map(message_parts).collect::<Vec<_>>();
        let prior_parts = prior_entries
            .iter()
            .map(|entry| (entry.role.clone(), entry.content.clone()))
            .collect::<Vec<_>>();

        if let Some(marker_entry) = marker {
            let prior = decode_turn_transcript_outcome(&marker_entry.content)
                .filter(|record| record.version == 1 && record.turn_id == turn_id)
                .ok_or_else(|| SessionError::DurableTurn("invalid terminal marker".into()))?;
            let stored_receipts = crate::HistoryRecoveryReceipt::from_terminal_entry(marker_entry)
                .ok_or_else(|| {
                    SessionError::DurableTurn("invalid terminal recovery marker".into())
                })?;
            let prior_source = prior_entries
                .iter()
                .map(|entry| (*entry).clone())
                .collect::<Vec<_>>();
            let requested_receipts = crate::history_receipt::bind_receipts(
                &filtered,
                &prior_source,
                projections,
                policy.persist_raw_tool_output,
            )?;
            if prior.outcome == outcome
                && prior_parts == requested_parts
                && stored_receipts == requested_receipts
            {
                return Ok(TurnCommit {
                    entry_ids: prior_ids,
                    newly_committed: false,
                });
            }
            return Err(SessionError::DurableTurnConflict {
                turn_id: turn_id.to_string(),
                existing: format!("{:?}", prior.outcome),
                requested: format!("{:?}", outcome),
            });
        }
        if !prior_entries.is_empty() {
            return Err(SessionError::DurableTurnConflict {
                turn_id: turn_id.to_string(),
                existing: "ambiguous_unfinalized_entries".into(),
                requested: format!("{:?}", outcome),
            });
        }

        let mut parent_id = existing.last().map(|entry| entry.id.clone());
        let mut entry_ids = Vec::with_capacity(messages.len());
        let entry_start = existing.len();
        for message in &filtered {
            let (role, content) = message_parts(message);
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

        let marker = TurnTranscriptOutcomeRecord::new(turn_id, outcome);
        let receipts = crate::history_receipt::bind_receipts(
            &filtered,
            &existing[entry_start..],
            projections,
            policy.persist_raw_tool_output,
        )?;
        let mut marker_content = encode_turn_transcript_outcome(&marker)
            .map_err(|error| SessionError::InvalidJson(error.to_string()))?;
        if !receipts.is_empty() {
            let mut value = serde_json::to_value(&marker)
                .map_err(|error| SessionError::InvalidJson(error.to_string()))?;
            value["history_recovery"] = serde_json::to_value(receipts)
                .map_err(|error| SessionError::InvalidJson(error.to_string()))?;
            marker_content = format!(
                "{}{}",
                crate::turn_outcome::TURN_TRANSCRIPT_OUTCOME_PREFIX,
                value
            );
        }
        let marker_id = Uuid::new_v4().to_string();
        let marker_entry = SessionEntry {
            id: marker_id,
            parent_id,
            timestamp: Utc::now(),
            role: "system".into(),
            content: marker_content,
            metadata: SessionMetadata {
                turn_id: Some(turn_id.to_string()),
                ..SessionMetadata::default()
            },
        };
        let json_len = serde_json::to_vec(&marker_entry)
            .map_err(|error| SessionError::InvalidJson(error.to_string()))?
            .len()
            + 1;
        if json_len > crate::history_page::HISTORY_PAGE_BYTES
            || crate::compact_text::encode_entry(&marker_entry).len()
                > crate::history_page::HISTORY_PAGE_BYTES
        {
            return Err(SessionError::DurableTurn(
                "encoded recovery marker exceeds source page bound".into(),
            ));
        }
        existing.push(marker_entry);

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
    // Initialize WAL mode and schema without busy_timeout so concurrent
    // first-time opens fail fast instead of blocking for 5s each. The
    // bounded retry loop (20 × 25 ms ≤ 500 ms total) lets the first writer
    // complete. Only DatabaseBusy / DatabaseLocked are retried; all other
    // SQLite errors propagate immediately. After init succeeds, apply the
    // 5-second busy_timeout for subsequent transaction operations.
    let init_sql = "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; \
         CREATE TABLE IF NOT EXISTS durable_bindings \
         (external_id TEXT PRIMARY KEY, session_id TEXT NOT NULL UNIQUE);";
    const MAX_INIT_RETRIES: u32 = 20;
    const RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);
    let mut retries = 0u32;
    loop {
        match connection.execute_batch(init_sql) {
            Ok(()) => break,
            Err(rusqlite::Error::SqliteFailure(err, _))
                if retries < MAX_INIT_RETRIES
                    && matches!(
                        err.code,
                        rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                    ) =>
            {
                retries += 1;
                std::thread::sleep(RETRY_INTERVAL);
            }
            // Non-retryable error or retries exhausted: return the
            // structured SQLite error without panicking.
            Err(e) => return Err(sql_error(e)),
        }
    }
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(sql_error)?;
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
        Message::Multimodal { parts } => Message::Multimodal {
            parts: parts
                .iter()
                .map(|part| match part {
                    talos_core::message::ContentPart::Text { text } => {
                        talos_core::message::ContentPart::Text { text: redact(text) }
                    }
                    talos_core::message::ContentPart::Image {
                        path,
                        mime,
                        byte_count,
                        content_digest,
                    } => talos_core::message::ContentPart::Image {
                        path: path.clone(),
                        mime: mime.clone(),
                        byte_count: *byte_count,
                        content_digest: content_digest.clone(),
                    },
                })
                .collect(),
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
    fn recovery_receipts_commit_atomically_and_survive_restart() {
        let directory = tempfile::tempdir().expect("tempdir");
        let session = create_or_open(directory.path(), "recovery").expect("session");
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
                result: MessageToolResult {
                    tool_use_id: "call".into(),
                    content: "fact: port 8123".into(),
                    is_error: false,
                },
            },
        ];
        let projections = vec![crate::RecoveryProjection {
            message_index: 1,
            tool_call_id: "call".into(),
            tool_name: "read".into(),
            content: "fact: port 8123".into(),
        }];
        let first = session
            .finalize_turn_with_recovery(
                "turn",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &projections,
            )
            .expect("commit");
        let retry = session
            .finalize_turn_with_recovery(
                "turn",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &projections,
            )
            .expect("retry");
        assert_eq!(first.entry_ids, retry.entry_ids);
        assert!(!retry.newly_committed);
        drop(session);
        let reopened = create_or_open(directory.path(), "recovery").expect("reopen");
        let entries = reopened.session().read_entries().expect("entries");
        let receipts =
            crate::HistoryRecoveryReceipt::from_terminal_entry(entries.last().expect("marker"))
                .expect("receipts");
        assert_eq!(receipts.len(), 1);
        assert!(
            !crate::export_json(&entries)
                .expect("export")
                .contains("history_recovery")
        );
        assert!(!crate::export_markdown(&entries).contains("history_recovery"));
        assert_eq!(
            crate::read_transcript(&crate::CompactTextSessionStore, reopened.file_path())
                .expect("read export")
                .len(),
            2
        );
        assert_eq!(receipts[0].entry_id(), first.entry_ids[1]);
        assert_eq!(
            receipts[0].verified_content(&entries[1]).as_deref(),
            Some("fact: port 8123")
        );
        let mut corrupt = entries[1].clone();
        corrupt.content.push_str("changed");
        assert!(receipts[0].verified_content(&corrupt).is_none());
        assert_eq!(reopened.transcript(None, 20).expect("transcript").len(), 2);
        assert!(
            reopened
                .finalize_turn(
                    "turn",
                    &messages,
                    &PersistencePolicy::default(),
                    TurnTranscriptOutcome::Success
                )
                .is_err()
        );
    }

    #[test]
    fn encoded_terminal_marker_bound_includes_json_escaping_and_metadata() {
        use crate::history_page::{HISTORY_PAGE_BYTES, HistoryEncoding, read_history_page};
        for (count, accepted) in [
            ((HISTORY_PAGE_BYTES - 2048) / 6, true),
            (HISTORY_PAGE_BYTES / 6 + 2048, false),
        ] {
            let directory = tempfile::tempdir().expect("tempdir");
            let session = create_or_open(directory.path(), "marker-bound").expect("session");
            let turn = "\"".repeat(count);
            let before = std::fs::read(session.file_path()).expect("initial");
            let result = session.finalize_turn_with_recovery(
                &turn,
                &[],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            );
            assert_eq!(result.is_ok(), accepted);
            if !accepted {
                assert_eq!(
                    std::fs::read(session.file_path()).expect("unchanged"),
                    before
                );
                continue;
            }
            let entries = session.session().read_entries().expect("entries");
            let json_path = directory.path().join("receipt.jsonl");
            crate::SessionStore::replace_entries_atomically(
                &crate::JsonlSessionStore,
                &json_path,
                &entries,
            )
            .expect("json encode");
            for (path, encoding) in [
                (session.file_path(), HistoryEncoding::Tlog),
                (json_path.as_path(), HistoryEncoding::Jsonl),
            ] {
                let end = std::fs::metadata(path).expect("metadata").len();
                let page = read_history_page(
                    path,
                    encoding,
                    0,
                    end,
                    std::time::Instant::now() + std::time::Duration::from_secs(5),
                    || false,
                )
                .expect("bounded page");
                assert!(page.complete);
                assert_eq!(page.entries.len(), 1);
            }
        }
    }

    #[test]
    fn duplicate_or_unknown_terminal_markers_refuse_retry_without_writing() {
        for duplicate in [false, true] {
            let directory = tempfile::tempdir().expect("tempdir");
            let session = create_or_open(directory.path(), "markers").expect("session");
            let messages = vec![Message::User {
                content: "original".into(),
            }];
            session
                .commit_turn("turn", &messages, &PersistencePolicy::default())
                .expect("commit");
            let mut entries = session.session().read_entries().expect("read");
            if duplicate {
                let mut marker = entries.last().expect("marker").clone();
                marker.id = "duplicate".into();
                entries.push(marker);
            } else {
                let marker = entries.last_mut().expect("marker");
                marker.content = marker.content.replace("\"version\":1", "\"version\":99");
                assert!(crate::HistoryRecoveryReceipt::from_terminal_entry(marker).is_none());
            }
            session
                .session()
                .store
                .replace_entries_atomically(session.file_path(), &entries)
                .expect("corrupt fixture");
            let before = std::fs::read(session.file_path()).expect("bytes");
            assert!(
                session
                    .commit_turn("turn", &messages, &PersistencePolicy::default())
                    .is_err()
            );
            assert_eq!(std::fs::read(session.file_path()).expect("bytes"), before);
        }
    }

    #[test]
    fn persistence_denial_or_redaction_never_mints_recovery_receipts() {
        for (permit, content) in [(false, "private result"), (true, "api_key=sk-secret")] {
            let directory = tempfile::tempdir().expect("tempdir");
            let session = create_or_open(directory.path(), "denial").expect("session");
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
                    result: MessageToolResult {
                        tool_use_id: "call".into(),
                        content: content.into(),
                        is_error: false,
                    },
                },
            ];
            let projection = crate::RecoveryProjection {
                message_index: 1,
                tool_call_id: "call".into(),
                tool_name: "read".into(),
                content: content.into(),
            };
            session
                .finalize_turn_with_recovery(
                    "turn",
                    &messages,
                    &PersistencePolicy {
                        persist_raw_tool_output: permit,
                        ..Default::default()
                    },
                    TurnTranscriptOutcome::Success,
                    &[projection],
                )
                .expect("commit");
            let entries = session.session().read_entries().expect("entries");
            assert!(
                crate::HistoryRecoveryReceipt::from_terminal_entry(entries.last().expect("marker"))
                    .expect("receipts")
                    .is_empty()
            );
            assert!(
                !std::fs::read_to_string(session.file_path())
                    .expect("disk")
                    .contains(content)
            );
        }
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
        assert_eq!(
            session
                .session()
                .read_turn_transcript_outcomes()
                .expect("outcomes"),
            vec![TurnTranscriptOutcomeRecord::new(
                "turn-1",
                TurnTranscriptOutcome::Success,
            )]
        );
    }

    #[test]
    fn partial_finalization_is_atomic_idempotent_and_reconstructs_on_reopen() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:partial").expect("session");
        let messages = vec![
            Message::User {
                content: "read file".into(),
            },
            Message::Assistant {
                content: "tool result follows".into(),
                tool_calls: vec![],
                reasoning: None,
            },
        ];
        let first = session
            .finalize_turn(
                "turn-cancelled",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Cancelled,
            )
            .expect("partial finalization");
        let retry = session
            .finalize_turn(
                "turn-cancelled",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Cancelled,
            )
            .expect("idempotent retry");
        assert!(first.newly_committed);
        assert!(!retry.newly_committed);
        assert_eq!(first.entry_ids, retry.entry_ids);
        assert_eq!(session.read_messages().expect("messages"), messages);
        assert_eq!(
            session
                .session()
                .read_turn_transcript_outcomes()
                .expect("outcome"),
            vec![TurnTranscriptOutcomeRecord::new(
                "turn-cancelled",
                TurnTranscriptOutcome::Cancelled,
            )]
        );
        let reopened = get_by_external_id(directory.path(), "task:partial")
            .expect("lookup")
            .expect("reopen");
        assert_eq!(
            reopened.read_messages().expect("replayed messages"),
            messages
        );
    }

    #[test]
    fn partial_finalization_conflicting_retry_preserves_original_state() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:conflict").expect("session");
        let original = [Message::User {
            content: "original".into(),
        }];
        session
            .finalize_turn(
                "turn-conflict",
                &original,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Error,
            )
            .expect("initial finalization");
        let before = std::fs::read(session.file_path()).expect("snapshot");
        let conflict = session.finalize_turn(
            "turn-conflict",
            &[Message::User {
                content: "different".into(),
            }],
            &PersistencePolicy::default(),
            TurnTranscriptOutcome::Cancelled,
        );
        assert!(matches!(
            conflict,
            Err(SessionError::DurableTurnConflict { .. })
        ));
        assert_eq!(std::fs::read(session.file_path()).expect("after"), before);
    }

    #[test]
    fn partial_finalization_rejects_ambiguous_legacy_entries() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:ambiguous").expect("session");
        session
            .session()
            .append_with_metadata(
                &Message::User {
                    content: "legacy partial".into(),
                },
                SessionMetadata {
                    turn_id: Some("turn-ambiguous".into()),
                    ..SessionMetadata::default()
                },
            )
            .expect("legacy entry");
        let result = session.finalize_turn(
            "turn-ambiguous",
            &[],
            &PersistencePolicy::default(),
            TurnTranscriptOutcome::Cancelled,
        );
        assert!(matches!(
            result,
            Err(SessionError::DurableTurnConflict { .. })
        ));
        assert!(
            session
                .session()
                .read_turn_transcript_outcomes()
                .expect("outcomes")
                .is_empty()
        );
    }

    #[test]
    fn empty_partial_finalization_writes_only_hidden_cancelled_evidence() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:empty-cancel").expect("session");

        let commit = session
            .finalize_turn(
                "turn-empty-cancel",
                &[],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Cancelled,
            )
            .expect("empty cancellation finalization");

        assert!(commit.newly_committed);
        assert!(commit.entry_ids.is_empty());
        assert!(session.transcript(None, 10).expect("transcript").is_empty());
        assert!(session.read_messages().expect("messages").is_empty());
        assert_eq!(
            session
                .session()
                .read_turn_transcript_outcomes()
                .expect("outcome"),
            vec![TurnTranscriptOutcomeRecord::new(
                "turn-empty-cancel",
                TurnTranscriptOutcome::Cancelled,
            )]
        );
    }

    #[test]
    fn partial_finalization_applies_reasoning_secret_and_tool_output_policy() {
        let directory = tempfile::tempdir().expect("temporary host directory");
        let session = create_or_open(directory.path(), "task:partial-policy").expect("session");
        let messages = vec![
            Message::Assistant {
                content: "Authorization: Bearer secret".into(),
                tool_calls: vec![ToolCall {
                    id: "call-policy".into(),
                    name: "lookup".into(),
                    input: serde_json::json!({"api_key": "sk-private", "query": "safe"}),
                }],
                reasoning: Some(AssistantReasoning {
                    provider: "provider".into(),
                    model: "model".into(),
                    blocks: vec![ReasoningBlock::Plain {
                        text: "private reasoning".into(),
                    }],
                }),
            },
            Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "call-policy".into(),
                    content: "raw private output".into(),
                    is_error: false,
                },
            },
        ];
        let policy = PersistencePolicy {
            persist_reasoning: false,
            persist_raw_tool_output: false,
        };

        session
            .finalize_turn(
                "turn-policy",
                &messages,
                &policy,
                TurnTranscriptOutcome::Error,
            )
            .expect("filtered partial finalization");

        let replay = session.read_messages().expect("replay");
        let serialized = serde_json::to_string(&replay).expect("serialize replay");
        assert!(!serialized.contains("private reasoning"));
        assert!(!serialized.contains("raw private output"));
        assert!(!serialized.contains("sk-private"));
        assert!(!serialized.contains("Bearer secret"));
        assert!(serialized.contains("[REDACTED]"));
        assert!(serialized.contains("[tool output omitted by persistence policy]"));
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

    #[test]
    fn open_bindings_non_busy_error_returns_immediately() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("durable-bindings.sqlite");
        // Write invalid content so Connection::open succeeds but the PRAGMA
        // fails with SQLITE_NOTADB — not BUSY/LOCKED. The retry loop must
        // NOT engage; the error returns in well under the 500 ms retry floor.
        std::fs::write(&db_path, b"not a database").expect("write");
        let start = std::time::Instant::now();
        let result = open_bindings(&db_path);
        let elapsed = start.elapsed();
        assert!(result.is_err(), "should fail on corrupt database");
        assert!(
            matches!(result, Err(SessionError::DurableTurn(_))),
            "expected DurableTurn, got {result:?}"
        );
        assert!(
            elapsed < std::time::Duration::from_millis(400),
            "non-BUSY error took {elapsed:?}; should return immediately"
        );
    }

    #[test]
    fn open_bindings_busy_exhaustion_returns_structured_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("durable-bindings.sqlite");
        // Create the database in default (rollback journal) mode and hold
        // a write lock via BEGIN IMMEDIATE so open_bindings cannot change
        // to WAL or create the schema table. With busy_timeout unset
        // during init, every retry fails immediately; after 20 retries
        // the function must return a structured SessionError, not panic.
        let holder = Connection::open(&db_path).expect("open holder");
        holder
            .execute_batch("BEGIN IMMEDIATE; CREATE TABLE t(x);")
            .expect("begin + create");
        let result = open_bindings(&db_path);
        assert!(
            result.is_err(),
            "should fail after retry exhaustion, got Ok"
        );
        match result {
            Err(SessionError::DurableTurn(msg)) => {
                assert!(
                    msg.contains("binding index error"),
                    "unexpected error message: {msg}"
                );
            }
            other => panic!("expected DurableTurn, got {other:?}"),
        }
    }
}
