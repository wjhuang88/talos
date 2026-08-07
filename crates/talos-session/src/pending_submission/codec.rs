use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use talos_core::submission::{
    PendingSubmissionState, SubmissionReceiptDisposition, SubmissionRejectionReason,
};

use super::{PendingSubmissionError, PendingSubmissionRecord};

pub(super) type RecordTuple = (String, String, String, String, Option<String>);

pub(super) struct IdentityRow {
    pub(super) receipt_id: String,
    pub(super) fingerprint: String,
    pub(super) json: Option<String>,
    pub(super) state: String,
    pub(super) turn_id: Option<String>,
}

pub(super) fn identity_matches(row: &IdentityRow, fingerprint: &str, encoded: &str) -> bool {
    row.fingerprint == fingerprint && row.json.as_deref().is_none_or(|json| json == encoded)
}

pub(super) fn lookup(
    connection: &Connection,
    submission_id: &str,
) -> Result<Option<IdentityRow>, rusqlite::Error> {
    let active = connection
        .query_row(
            "SELECT receipt_id, fingerprint, submission_json, state, turn_id
             FROM pending_submissions WHERE batch_id = ?1",
            params![submission_id],
            |row| {
                Ok(IdentityRow {
                    receipt_id: row.get(0)?,
                    fingerprint: row.get(1)?,
                    json: Some(row.get(2)?),
                    state: row.get(3)?,
                    turn_id: row.get(4)?,
                })
            },
        )
        .optional()?;
    if active.is_some() {
        return Ok(active);
    }
    connection
        .query_row(
            "SELECT receipt_id, fingerprint, state, turn_id
             FROM submission_idempotency WHERE batch_id = ?1",
            params![submission_id],
            |row| {
                Ok(IdentityRow {
                    receipt_id: row.get(0)?,
                    fingerprint: row.get(1)?,
                    json: None,
                    state: row.get(2)?,
                    turn_id: row.get(3)?,
                })
            },
        )
        .optional()
}

pub(super) fn read_record_tuple(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecordTuple> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

pub(super) fn tuple_to_record(
    tuple: RecordTuple,
) -> Result<PendingSubmissionRecord, PendingSubmissionError> {
    let (receipt_id, payload_fingerprint, json, state, turn_id) = tuple;
    Ok(PendingSubmissionRecord {
        receipt_id,
        payload_fingerprint,
        submission: serde_json::from_str(&json)?,
        state: decode_state(&state)?,
        turn_id,
    })
}

pub(super) fn rejected(
    reason: SubmissionRejectionReason,
) -> (String, SubmissionReceiptDisposition) {
    (
        String::new(),
        SubmissionReceiptDisposition::Rejected { reason },
    )
}

pub(super) fn fingerprint(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(super) fn to_i64<T>(value: T) -> i64
where
    i64: TryFrom<T>,
{
    i64::try_from(value).unwrap_or(i64::MAX)
}

pub(super) fn encode_state(state: PendingSubmissionState) -> &'static str {
    match state {
        PendingSubmissionState::AcceptedPending => "accepted_pending",
        PendingSubmissionState::Running => "running",
        PendingSubmissionState::PausedPending => "paused_pending",
        PendingSubmissionState::TerminalCancelled => "terminal_cancelled",
        PendingSubmissionState::TerminalError => "terminal_error",
        PendingSubmissionState::Committed => "committed",
    }
}

pub(super) fn decode_state(state: &str) -> Result<PendingSubmissionState, PendingSubmissionError> {
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
