//! Durable, session-scoped custody for structured submissions (ADR-056).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use talos_core::submission::{
    MAX_PENDING_SUBMISSION_BYTES, MAX_PENDING_SUBMISSIONS, MAX_STEERING_QUEUE_IMAGE_BYTES,
    MAX_STEERING_QUEUE_IMAGES, PendingSubmissionState, StructuredSubmission,
    SubmissionReceiptDisposition, SubmissionRejectionReason,
};
use thiserror::Error;
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 1;
const MAX_TOMBSTONES: usize = MAX_PENDING_SUBMISSIONS * 2;

/// Failure while reading or mutating pending-submission custody.
#[derive(Debug, Error)]
pub enum PendingSubmissionError {
    /// SQLite operation failed.
    #[error("pending submission journal failed: {0}")]
    Storage(#[from] rusqlite::Error),
    /// Structured payload encoding failed.
    #[error("pending submission payload failed: {0}")]
    Payload(#[from] serde_json::Error),
    /// Journal directory creation failed.
    #[error("pending submission journal I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// The in-process journal lock was poisoned.
    #[error("pending submission journal lock poisoned")]
    LockPoisoned,
    /// The on-disk schema cannot be read by this build.
    #[error("unsupported pending submission journal schema version: {0}")]
    UnsupportedSchema(i64),
    /// A lifecycle update addressed no durable record.
    #[error("pending submission record not found")]
    MissingRecord,
    /// A lifecycle update violates the state machine.
    #[error("invalid pending submission state transition")]
    InvalidTransition,
}

/// Exact durable record returned for Actor recovery.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSubmissionRecord {
    /// Durable acceptance receipt.
    pub receipt_id: String,
    /// SHA-256 fingerprint of the immutable serialized submission.
    pub payload_fingerprint: String,
    /// Original immutable structured submission.
    pub submission: StructuredSubmission,
    /// Current durable lifecycle state.
    pub state: PendingSubmissionState,
    /// Correlated Turn after execution starts.
    pub turn_id: Option<String>,
}

/// SQLite sidecar that stores accepted but uncompleted Session work.
#[derive(Debug, Clone)]
pub struct PendingSubmissionStore {
    path: Arc<PathBuf>,
    lock: Arc<Mutex<()>>,
}

impl PendingSubmissionStore {
    /// Opens the sidecar adjacent to a Session transcript.
    #[must_use]
    pub fn for_session(session: &crate::Session) -> Self {
        Self::for_session_file(&session.file_path, &session.id.to_string())
    }

    /// Opens the sidecar adjacent to an arbitrary Session transcript path.
    #[must_use]
    pub fn for_session_file(session_file: &Path, session_id: &str) -> Self {
        let parent = session_file.parent().unwrap_or_else(|| Path::new("."));
        Self {
            path: Arc::new(parent.join(format!("{session_id}.pending.sqlite"))),
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Returns the sidecar path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Durably accepts an exact submission or returns its prior receipt.
    pub fn accept(
        &self,
        submission: &StructuredSubmission,
    ) -> Result<(String, SubmissionReceiptDisposition), PendingSubmissionError> {
        if let Err(reason) = submission.validate() {
            return Ok(rejected(reason));
        }
        let encoded = serde_json::to_string(submission)?;
        let fingerprint = fingerprint(encoded.as_bytes());
        let text_bytes = submission
            .total_text_bytes()
            .ok_or(PendingSubmissionError::InvalidTransition)?;
        let (image_count, image_bytes, _) = submission
            .image_totals()
            .ok_or(PendingSubmissionError::InvalidTransition)?;
        let generation = match i64::try_from(submission.session_generation) {
            Ok(value) => value,
            Err(_) => return Ok(rejected(SubmissionRejectionReason::InvalidStructure)),
        };

        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;

        if let Some(row) = lookup(&transaction, &submission.batch_id)? {
            let disposition = if row.fingerprint == fingerprint && row.json == encoded {
                SubmissionReceiptDisposition::AlreadyAccepted {
                    state: decode_state(&row.state)?,
                    turn_id: row.turn_id,
                }
            } else {
                SubmissionReceiptDisposition::Rejected {
                    reason: SubmissionRejectionReason::IdentityConflict,
                }
            };
            transaction.commit()?;
            return Ok((row.receipt_id, disposition));
        }

        prune_tombstones(&transaction)?;
        if exceeds_pending_bounds(&transaction, text_bytes, image_count, image_bytes)? {
            transaction.commit()?;
            return Ok(rejected(SubmissionRejectionReason::LimitExceeded));
        }

        let receipt_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO pending_submissions (
                batch_id, reservation_id, session_id, session_generation,
                receipt_id, fingerprint, submission_json, text_bytes,
                image_count, image_bytes, state, turn_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                       'accepted_pending', NULL)",
            params![
                submission.batch_id,
                submission.reservation_id,
                submission.session_id,
                generation,
                receipt_id,
                fingerprint,
                encoded,
                to_i64(text_bytes),
                to_i64(image_count),
                to_i64(image_bytes),
            ],
        )?;
        transaction.commit()?;
        Ok((receipt_id, SubmissionReceiptDisposition::AcceptedPending))
    }

    /// Reconciles a previously sent submission without creating custody.
    pub fn reconcile(
        &self,
        submission: &StructuredSubmission,
    ) -> Result<(String, SubmissionReceiptDisposition), PendingSubmissionError> {
        if let Err(reason) = submission.validate() {
            return Ok(rejected(reason));
        }
        let encoded = serde_json::to_string(submission)?;
        let payload_fingerprint = fingerprint(encoded.as_bytes());
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        Ok(match lookup(&connection, &submission.batch_id)? {
            Some(row) if row.fingerprint == payload_fingerprint && row.json == encoded => (
                row.receipt_id,
                SubmissionReceiptDisposition::AlreadyAccepted {
                    state: decode_state(&row.state)?,
                    turn_id: row.turn_id,
                },
            ),
            Some(row) => (
                row.receipt_id,
                SubmissionReceiptDisposition::Rejected {
                    reason: SubmissionRejectionReason::IdentityConflict,
                },
            ),
            None => (String::new(), SubmissionReceiptDisposition::NotAccepted),
        })
    }

    /// Marks an accepted submission as the active Turn.
    pub fn mark_running(&self, batch_id: &str, turn_id: &str) -> Result<(), PendingSubmissionError> {
        self.transition(
            batch_id,
            PendingSubmissionState::Running,
            Some(turn_id),
            &[
                PendingSubmissionState::AcceptedPending,
                PendingSubmissionState::PausedPending,
            ],
        )
    }

    /// Pauses all accepted submissions that have not started.
    pub fn pause_unstarted(&self) -> Result<usize, PendingSubmissionError> {
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let changed = transaction.execute(
            "UPDATE pending_submissions SET state = 'paused_pending'
             WHERE state = 'accepted_pending'",
            [],
        )?;
        transaction.commit()?;
        Ok(changed)
    }

    /// Marks a started submission terminal without making it resumable.
    pub fn mark_terminal(
        &self,
        batch_id: &str,
        state: PendingSubmissionState,
        turn_id: &str,
    ) -> Result<(), PendingSubmissionError> {
        if !matches!(
            state,
            PendingSubmissionState::TerminalCancelled | PendingSubmissionState::TerminalError
        ) {
            return Err(PendingSubmissionError::InvalidTransition);
        }
        self.transition(
            batch_id,
            state,
            Some(turn_id),
            &[PendingSubmissionState::Running],
        )
    }

    /// Finalizes custody after the successful transcript commit.
    pub fn mark_committed(
        &self,
        batch_id: &str,
        turn_id: &str,
    ) -> Result<(), PendingSubmissionError> {
        self.transition(
            batch_id,
            PendingSubmissionState::Committed,
            Some(turn_id),
            &[
                PendingSubmissionState::Running,
                PendingSubmissionState::Committed,
            ],
        )
    }

    /// Returns unstarted work in durable FIFO order.
    pub fn recover_unstarted(&self) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> {
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        let mut statement = connection.prepare(
            "SELECT receipt_id, fingerprint, submission_json, state, turn_id
             FROM pending_submissions
             WHERE state IN ('accepted_pending', 'paused_pending')
             ORDER BY rowid ASC",
        )?;
        let rows = statement.query_map([], read_record_tuple)?;
        rows.map(|row| tuple_to_record(row?)).collect()
    }

    /// Returns one durable batch record.
    pub fn get(
        &self,
        batch_id: &str,
    ) -> Result<Option<PendingSubmissionRecord>, PendingSubmissionError> {
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        connection
            .query_row(
                "SELECT receipt_id, fingerprint, submission_json, state, turn_id
                 FROM pending_submissions WHERE batch_id = ?1",
                params![batch_id],
                read_record_tuple,
            )
            .optional()?
            .map(tuple_to_record)
            .transpose()
    }

    fn transition(
        &self,
        batch_id: &str,
        next: PendingSubmissionState,
        turn_id: Option<&str>,
        expected: &[PendingSubmissionState],
    ) -> Result<(), PendingSubmissionError> {
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let current = transaction
            .query_row(
                "SELECT state FROM pending_submissions WHERE batch_id = ?1",
                params![batch_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(PendingSubmissionError::MissingRecord)?;
        if !expected.contains(&decode_state(&current)?) {
            return Err(PendingSubmissionError::InvalidTransition);
        }
        transaction.execute(
            "UPDATE pending_submissions SET state = ?2, turn_id = ?3 WHERE batch_id = ?1",
            params![batch_id, encode_state(next), turn_id],
        )?;
        prune_tombstones(&transaction)?;
        transaction.commit()?;
        Ok(())
    }

    fn guard(&self) -> Result<std::sync::MutexGuard<'_, ()>, PendingSubmissionError> {
        self.lock
            .lock()
            .map_err(|_| PendingSubmissionError::LockPoisoned)
    }

    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
        connection.busy_timeout(Duration::from_secs(5))?;
        Ok(connection)
    }
}

type RecordTuple = (String, String, String, String, Option<String>);

struct IdentityRow {
    receipt_id: String,
    fingerprint: String,
    json: String,
    state: String,
    turn_id: Option<String>,
}

fn immediate(connection: &mut Connection) -> Result<Transaction<'_>, rusqlite::Error> {
    connection.transaction_with_behavior(TransactionBehavior::Immediate)
}

fn lookup(connection: &Connection, batch_id: &str) -> Result<Option<IdentityRow>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT receipt_id, fingerprint, submission_json, state, turn_id
             FROM pending_submissions WHERE batch_id = ?1",
            params![batch_id],
            |row| {
                Ok(IdentityRow {
                    receipt_id: row.get(0)?,
                    fingerprint: row.get(1)?,
                    json: row.get(2)?,
                    state: row.get(3)?,
                    turn_id: row.get(4)?,
                })
            },
        )
        .optional()
}

fn read_record_tuple(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecordTuple> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn tuple_to_record(tuple: RecordTuple) -> Result<PendingSubmissionRecord, PendingSubmissionError> {
    let (receipt_id, payload_fingerprint, json, state, turn_id) = tuple;
    Ok(PendingSubmissionRecord {
        receipt_id,
        payload_fingerprint,
        submission: serde_json::from_str(&json)?,
        state: decode_state(&state)?,
        turn_id,
    })
}

fn rejected(reason: SubmissionRejectionReason) -> (String, SubmissionReceiptDisposition) {
    (
        String::new(),
        SubmissionReceiptDisposition::Rejected { reason },
    )
}

fn fingerprint(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn to_i64<T>(value: T) -> i64
where
    i64: TryFrom<T>,
{
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn exceeds_pending_bounds(
    transaction: &Transaction<'_>,
    text_bytes: usize,
    image_count: usize,
    image_bytes: u64,
) -> Result<bool, rusqlite::Error> {
    let current: (i64, i64, i64, i64) = transaction.query_row(
        "SELECT COUNT(*), COALESCE(SUM(text_bytes), 0),
                COALESCE(SUM(image_count), 0), COALESCE(SUM(image_bytes), 0)
         FROM pending_submissions
         WHERE state IN ('accepted_pending', 'running', 'paused_pending')",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    Ok(current.0.saturating_add(1) > to_i64(MAX_PENDING_SUBMISSIONS)
        || current.1.saturating_add(to_i64(text_bytes)) > to_i64(MAX_PENDING_SUBMISSION_BYTES)
        || current.2.saturating_add(to_i64(image_count)) > to_i64(MAX_STEERING_QUEUE_IMAGES)
        || current.3.saturating_add(to_i64(image_bytes)) > to_i64(MAX_STEERING_QUEUE_IMAGE_BYTES))
}

fn ensure_schema(connection: &Connection) -> Result<(), PendingSubmissionError> {
    connection.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = FULL;
         CREATE TABLE IF NOT EXISTS pending_journal_meta (
             key TEXT PRIMARY KEY, value INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS pending_submissions (
             batch_id TEXT PRIMARY KEY,
             reservation_id TEXT NOT NULL,
             session_id TEXT NOT NULL,
             session_generation INTEGER NOT NULL,
             receipt_id TEXT NOT NULL UNIQUE,
             fingerprint TEXT NOT NULL,
             submission_json TEXT NOT NULL,
             text_bytes INTEGER NOT NULL,
             image_count INTEGER NOT NULL,
             image_bytes INTEGER NOT NULL,
             state TEXT NOT NULL,
             turn_id TEXT
         );
         INSERT OR IGNORE INTO pending_journal_meta (key, value)
         VALUES ('schema_version', 1);",
    )?;
    let version = connection.query_row(
        "SELECT value FROM pending_journal_meta WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, i64>(0),
    )?;
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PendingSubmissionError::UnsupportedSchema(version))
    }
}

fn prune_tombstones(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    let count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM pending_submissions
         WHERE state IN ('committed', 'terminal_cancelled', 'terminal_error')",
        [],
        |row| row.get(0),
    )?;
    let remove = count.saturating_sub(to_i64(MAX_TOMBSTONES));
    if remove > 0 {
        transaction.execute(
            "DELETE FROM pending_submissions WHERE rowid IN (
                 SELECT rowid FROM pending_submissions
                 WHERE state IN ('committed', 'terminal_cancelled', 'terminal_error')
                 ORDER BY rowid ASC LIMIT ?1
             )",
            params![remove],
        )?;
    }
    Ok(())
}

fn encode_state(state: PendingSubmissionState) -> &'static str {
    match state {
        PendingSubmissionState::AcceptedPending => "accepted_pending",
        PendingSubmissionState::Running => "running",
        PendingSubmissionState::PausedPending => "paused_pending",
        PendingSubmissionState::TerminalCancelled => "terminal_cancelled",
        PendingSubmissionState::TerminalError => "terminal_error",
        PendingSubmissionState::Committed => "committed",
    }
}

fn decode_state(state: &str) -> Result<PendingSubmissionState, PendingSubmissionError> {
    match state {
        "accepted_pending" => Ok(PendingSubmissionState::AcceptedPending),
        "running" => Ok(PendingSubmissionState::Running),
        "paused_pending" => Ok(PendingSubmissionState::PausedPending),
        "terminal_cancelled" => Ok(PendingSubmissionState::TerminalCancelled),
        "terminal_error" => Ok(PendingSubmissionState::TerminalError),
        "committed" => Ok(PendingSubmissionState::Committed),
        _ => Err(PendingSubmissionError::UnsupportedSchema(-1)),
    }
}

#[cfg(test)]
mod tests;
