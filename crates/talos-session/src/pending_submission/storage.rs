use std::time::{Duration, Instant};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use talos_core::submission::{
    MAX_PENDING_SUBMISSION_BYTES, MAX_PENDING_SUBMISSIONS, MAX_STEERING_QUEUE_IMAGE_BYTES,
    MAX_STEERING_QUEUE_IMAGES,
};

use super::codec::to_i64;
use super::{PendingSubmissionError, RUNTIME_GENERATION_KEY, SCHEMA_VERSION};
use crate::runtime_state::{
    SessionRuntimeActivation, SessionRuntimeActivationStatus, SessionRuntimeState,
};

pub(super) const MAX_TOMBSTONES: usize = MAX_PENDING_SUBMISSIONS * 2;
pub(super) const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const SQLITE_BUSY_RETRY_DELAY: Duration = Duration::from_millis(10);

pub(super) fn retry_sqlite_busy<T>(
    timeout: Duration,
    mut operation: impl FnMut() -> Result<T, rusqlite::Error>,
) -> Result<T, rusqlite::Error> {
    let deadline = Instant::now() + timeout;
    loop {
        match operation() {
            Err(error) if sqlite_is_busy_or_locked(&error) && Instant::now() < deadline => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                std::thread::sleep(SQLITE_BUSY_RETRY_DELAY.min(remaining));
            }
            result => return result,
        }
    }
}

fn sqlite_is_busy_or_locked(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}

pub(super) fn immediate(connection: &mut Connection) -> Result<Transaction<'_>, rusqlite::Error> {
    connection.transaction_with_behavior(TransactionBehavior::Immediate)
}

pub(super) fn exceeds_pending_bounds(
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
    Ok(
        current.0.saturating_add(1) > to_i64(MAX_PENDING_SUBMISSIONS)
            || current.1.saturating_add(to_i64(text_bytes)) > to_i64(MAX_PENDING_SUBMISSION_BYTES)
            || current.2.saturating_add(to_i64(image_count)) > to_i64(MAX_STEERING_QUEUE_IMAGES)
            || current.3.saturating_add(to_i64(image_bytes))
                > to_i64(MAX_STEERING_QUEUE_IMAGE_BYTES),
    )
}

pub(super) fn ensure_schema(connection: &Connection) -> Result<(), PendingSubmissionError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS pending_journal_meta (
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
         CREATE TABLE IF NOT EXISTS submission_idempotency (
             batch_id TEXT PRIMARY KEY,
             receipt_id TEXT NOT NULL,
             fingerprint TEXT NOT NULL,
             state TEXT NOT NULL,
             turn_id TEXT
         );
         CREATE TABLE IF NOT EXISTS session_runtime_state (
             singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
             activation_id TEXT NOT NULL,
             generation INTEGER NOT NULL,
             activation_json TEXT NOT NULL,
             status TEXT NOT NULL CHECK(status IN ('pending_marker', 'committed'))
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

pub(super) fn count_nonterminal(connection: &Connection) -> Result<usize, rusqlite::Error> {
    let pending = connection.query_row(
        "SELECT COUNT(*) FROM pending_submissions
         WHERE state IN ('accepted_pending', 'running', 'paused_pending')",
        [],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(usize::try_from(pending).unwrap_or(usize::MAX))
}

pub(super) fn load_runtime_state(
    connection: &Connection,
) -> Result<Option<SessionRuntimeState>, PendingSubmissionError> {
    let row = connection
        .query_row(
            "SELECT activation_id, generation, activation_json, status
             FROM session_runtime_state WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((activation_id, generation, encoded, status)) = row else {
        return Ok(None);
    };
    let activation: SessionRuntimeActivation = serde_json::from_str(&encoded)?;
    let status = SessionRuntimeActivationStatus::parse(&status).ok_or_else(|| {
        PendingSubmissionError::InvalidRuntimeActivation(
            "unsupported runtime activation status".to_string(),
        )
    })?;
    if !activation.is_valid()
        || activation.activation_id != activation_id
        || to_i64(activation.generation) != generation
    {
        return Err(PendingSubmissionError::InvalidRuntimeActivation(
            "stored runtime activation failed identity validation".to_string(),
        ));
    }
    Ok(Some(SessionRuntimeState { activation, status }))
}

pub(super) fn write_runtime_state(
    connection: &Connection,
    state: &SessionRuntimeState,
) -> Result<(), PendingSubmissionError> {
    if !state.activation.is_valid() {
        return Err(PendingSubmissionError::InvalidRuntimeActivation(
            "attempted to persist an invalid activation".to_string(),
        ));
    }
    let encoded = serde_json::to_string(&state.activation)?;
    connection.execute(
        "INSERT INTO session_runtime_state (
             singleton, activation_id, generation, activation_json, status
         ) VALUES (1, ?1, ?2, ?3, ?4)
         ON CONFLICT(singleton) DO UPDATE SET
             activation_id = excluded.activation_id,
             generation = excluded.generation,
             activation_json = excluded.activation_json,
             status = excluded.status",
        params![
            state.activation.activation_id,
            to_i64(state.activation.generation),
            encoded,
            state.status.as_str(),
        ],
    )?;
    Ok(())
}

pub(super) fn load_or_initialize_runtime_generation(
    connection: &Connection,
) -> Result<u64, PendingSubmissionError> {
    connection.execute(
        "INSERT OR IGNORE INTO pending_journal_meta (key, value) VALUES (?1, 0)",
        params![RUNTIME_GENERATION_KEY],
    )?;
    let value = connection.query_row(
        "SELECT value FROM pending_journal_meta WHERE key = ?1",
        params![RUNTIME_GENERATION_KEY],
        |row| row.get::<_, i64>(0),
    )?;
    u64::try_from(value).map_err(|_| PendingSubmissionError::UnsupportedSchema(value))
}

pub(super) fn prune_tombstones(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    // Preserve the permanent idempotency identity and terminal summary before
    // pruning the large serialized payload. Delayed retries can therefore
    // never become a fresh Provider execution merely because payload storage
    // was compacted.
    transaction.execute(
        "INSERT INTO submission_idempotency (
             batch_id, receipt_id, fingerprint, state, turn_id
         )
         SELECT batch_id, receipt_id, fingerprint, state, turn_id
         FROM pending_submissions
         WHERE state IN ('committed', 'terminal_cancelled', 'terminal_error')
         ON CONFLICT(batch_id) DO UPDATE SET
             receipt_id = excluded.receipt_id,
             fingerprint = excluded.fingerprint,
             state = excluded.state,
             turn_id = excluded.turn_id",
        [],
    )?;

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
