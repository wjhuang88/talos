//! Durable, session-scoped custody for structured submissions (ADR-056).

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OptionalExtension, params};
use talos_core::submission::{
    PendingSubmissionState, StructuredSubmission, SubmissionKind, SubmissionReceiptDisposition,
    SubmissionRejectionReason,
};
use thiserror::Error;
use uuid::Uuid;

use crate::runtime_state::{
    SessionRuntimeActivation, SessionRuntimeActivationStatus, SessionRuntimeIdentity,
    SessionRuntimeState,
};
use crate::turn_outcome::decode_turn_transcript_outcome;
use crate::{CompactTextSessionStore, JsonlSessionStore, SessionStore, TurnTranscriptOutcome};

const SCHEMA_VERSION: i64 = 1;
const RUNTIME_GENERATION_KEY: &str = "runtime_generation";
mod codec;
mod storage;

use codec::*;
use storage::*;

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
    /// Transcript inspection failed while reconciling a Running record.
    #[error("pending submission transcript inspection failed: {0}")]
    Transcript(#[from] crate::SessionError),
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
    /// The durable runtime generation changed outside the expected lifecycle owner.
    #[error("runtime generation conflict: expected {expected}, found {actual}")]
    GenerationConflict {
        /// Generation held by the caller.
        expected: u64,
        /// Generation stored durably for the Session.
        actual: u64,
    },
    /// Non-terminal durable custody still belongs to the current generation.
    #[error("runtime generation {generation} still owns {pending} non-terminal submission(s)")]
    GenerationBusy {
        /// Generation that still owns custody.
        generation: u64,
        /// Number of non-terminal durable records.
        pending: usize,
    },
    /// The durable runtime generation cannot advance further.
    #[error("runtime generation exhausted")]
    GenerationExhausted,
    /// A runtime activation record failed validation or contradicted the durable generation.
    #[error("invalid Session runtime activation: {0}")]
    InvalidRuntimeActivation(String),
    /// A different runtime identity already owns this generation.
    #[error("Session runtime activation conflict: existing {existing}, requested {requested}")]
    RuntimeActivationConflict { existing: String, requested: String },
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
    session_file: Arc<PathBuf>,
    session_id: Arc<String>,
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
            session_file: Arc::new(session_file.to_path_buf()),
            session_id: Arc::new(session_id.to_owned()),
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Returns the sidecar path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Loads the durable runtime generation for this logical Session.
    ///
    /// Legacy Sessions without the metadata key are initialized at generation
    /// zero. Process reconstruction rehydrates this exact value so accepted
    /// envelopes remain addressable after memory loss.
    pub fn runtime_generation(&self) -> Result<u64, PendingSubmissionError> {
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let generation = load_or_initialize_runtime_generation(&transaction)?;
        transaction.commit()?;
        Ok(generation)
    }

    /// Loads the compaction-independent current Session runtime identity.
    pub fn runtime_state(&self) -> Result<Option<SessionRuntimeState>, PendingSubmissionError> {
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        load_runtime_state(&connection)
    }

    /// Initializes a new/legacy Session with an exact committed runtime identity.
    ///
    /// Repeating the same initialization is idempotent. A contradictory identity
    /// at the same durable generation is rejected rather than silently changing
    /// the Session's reconstruction authority.
    pub fn initialize_runtime_identity(
        &self,
        identity: SessionRuntimeIdentity,
    ) -> Result<SessionRuntimeState, PendingSubmissionError> {
        self.ensure_transcript_owner_marker()?;
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let generation = load_or_initialize_runtime_generation(&transaction)?;
        let activation = SessionRuntimeActivation::new(generation, identity.clone(), identity);
        if let Some(existing) = load_runtime_state(&transaction)? {
            if existing.activation == activation
                && existing.status == SessionRuntimeActivationStatus::Committed
            {
                transaction.commit()?;
                return Ok(existing);
            }
            return Err(PendingSubmissionError::RuntimeActivationConflict {
                existing: existing.activation.activation_id,
                requested: activation.activation_id,
            });
        }
        write_runtime_state(
            &transaction,
            &SessionRuntimeState {
                activation: activation.clone(),
                status: SessionRuntimeActivationStatus::Committed,
            },
        )?;
        transaction.commit()?;
        Ok(SessionRuntimeState {
            activation,
            status: SessionRuntimeActivationStatus::Committed,
        })
    }

    /// Atomically fences generation `G` and stages the exact `G+1` activation.
    ///
    /// The transcript marker is committed in a separate append-only store, so
    /// the sidecar records `pending_marker` until the caller proves that marker
    /// durable and calls [`Self::commit_runtime_activation`]. A crash at that
    /// cut point is therefore recoverable without starting a runtime from an
    /// unproven activation.
    pub fn stage_runtime_activation(
        &self,
        expected: u64,
        activation: &SessionRuntimeActivation,
    ) -> Result<u64, PendingSubmissionError> {
        if !activation.is_valid() {
            return Err(PendingSubmissionError::InvalidRuntimeActivation(
                "activation digest/version mismatch".to_string(),
            ));
        }
        let expected_next = expected
            .checked_add(1)
            .ok_or(PendingSubmissionError::GenerationExhausted)?;
        if activation.generation != expected_next {
            return Err(PendingSubmissionError::InvalidRuntimeActivation(format!(
                "activation generation {} does not follow expected generation {expected}",
                activation.generation
            )));
        }

        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let current = load_or_initialize_runtime_generation(&transaction)?;

        if current == activation.generation {
            let existing = load_runtime_state(&transaction)?.ok_or_else(|| {
                PendingSubmissionError::InvalidRuntimeActivation(
                    "generation advanced without runtime activation state".to_string(),
                )
            })?;
            if existing.activation == *activation {
                transaction.commit()?;
                return Ok(current);
            }
            return Err(PendingSubmissionError::RuntimeActivationConflict {
                existing: existing.activation.activation_id,
                requested: activation.activation_id.clone(),
            });
        }
        if current != expected {
            return Err(PendingSubmissionError::GenerationConflict {
                expected,
                actual: current,
            });
        }
        let pending = count_nonterminal(&transaction)?;
        if pending > 0 {
            return Err(PendingSubmissionError::GenerationBusy {
                generation: current,
                pending,
            });
        }
        transaction.execute(
            "UPDATE pending_journal_meta SET value = ?2 WHERE key = ?1",
            params![RUNTIME_GENERATION_KEY, to_i64(activation.generation)],
        )?;
        write_runtime_state(
            &transaction,
            &SessionRuntimeState {
                activation: activation.clone(),
                status: SessionRuntimeActivationStatus::PendingMarker,
            },
        )?;
        transaction.commit()?;
        Ok(activation.generation)
    }

    /// Marks a staged activation committed after its exact transcript marker is durable.
    pub fn commit_runtime_activation(
        &self,
        activation_id: &str,
    ) -> Result<SessionRuntimeState, PendingSubmissionError> {
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let mut state = load_runtime_state(&transaction)?.ok_or_else(|| {
            PendingSubmissionError::InvalidRuntimeActivation(
                "no staged runtime activation exists".to_string(),
            )
        })?;
        if state.activation.activation_id != activation_id {
            return Err(PendingSubmissionError::RuntimeActivationConflict {
                existing: state.activation.activation_id,
                requested: activation_id.to_string(),
            });
        }
        let generation = load_or_initialize_runtime_generation(&transaction)?;
        if generation != state.activation.generation {
            return Err(PendingSubmissionError::InvalidRuntimeActivation(format!(
                "runtime state generation {} differs from durable generation {generation}",
                state.activation.generation
            )));
        }
        state.status = SessionRuntimeActivationStatus::Committed;
        write_runtime_state(&transaction, &state)?;
        transaction.commit()?;
        Ok(state)
    }

    /// Atomically advances the durable generation for a live replacement of
    /// the same logical Session.
    ///
    /// The quiescence check and metadata update execute under the same SQLite
    /// immediate transaction used by [`Self::accept`]. Therefore either an
    /// in-flight admission wins and the fence remains at the old generation,
    /// or the fence wins and every new old-generation admission is rejected.
    pub fn advance_runtime_generation(&self, expected: u64) -> Result<u64, PendingSubmissionError> {
        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;
        let current = load_or_initialize_runtime_generation(&transaction)?;
        if current != expected {
            return Err(PendingSubmissionError::GenerationConflict {
                expected,
                actual: current,
            });
        }
        let pending = count_nonterminal(&transaction)?;
        if pending > 0 {
            return Err(PendingSubmissionError::GenerationBusy {
                generation: current,
                pending,
            });
        }
        let next = current
            .checked_add(1)
            .ok_or(PendingSubmissionError::GenerationExhausted)?;
        transaction.execute(
            "UPDATE pending_journal_meta SET value = ?2 WHERE key = ?1",
            params![RUNTIME_GENERATION_KEY, to_i64(next)],
        )?;
        transaction.commit()?;
        Ok(next)
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
        let text_bytes = submission.total_text_bytes();
        let (image_count, image_bytes) = submission.image_totals();
        let generation = match i64::try_from(submission.sender_generation) {
            Ok(value) => value,
            Err(_) => return Ok(rejected(SubmissionRejectionReason::InvalidStructure)),
        };

        let _guard = self.guard()?;
        let mut connection = self.connection()?;
        let transaction = immediate(&mut connection)?;
        ensure_schema(&transaction)?;

        if let Some(row) = lookup(&transaction, &submission.id)? {
            let disposition = if identity_matches(&row, &fingerprint, &encoded) {
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

        let runtime_generation = load_or_initialize_runtime_generation(&transaction)?;
        if submission.sender_generation != runtime_generation {
            transaction.commit()?;
            return Ok(rejected(SubmissionRejectionReason::WrongGeneration));
        }

        prune_tombstones(&transaction)?;
        if exceeds_pending_bounds(&transaction, text_bytes, image_count, image_bytes)? {
            transaction.commit()?;
            return Ok(rejected(SubmissionRejectionReason::LimitExceeded));
        }

        let receipt_id = Uuid::new_v4().to_string();
        let reservation_id = format!("reservation:{}", submission.id);
        transaction.execute(
            "INSERT INTO pending_submissions (
                batch_id, reservation_id, session_id, session_generation,
                receipt_id, fingerprint, submission_json, text_bytes,
                image_count, image_bytes, state, turn_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                       'accepted_pending', NULL)",
            params![
                submission.id,
                reservation_id,
                self.session_id.as_str(),
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
        Ok(match lookup(&connection, &submission.id)? {
            Some(row) if identity_matches(&row, &payload_fingerprint, &encoded) => (
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
    pub fn mark_running(
        &self,
        submission_id: &str,
        turn_id: &str,
    ) -> Result<(), PendingSubmissionError> {
        self.transition(
            submission_id,
            PendingSubmissionState::Running,
            Some(turn_id),
            &[
                PendingSubmissionState::AcceptedPending,
                PendingSubmissionState::PausedPending,
            ],
        )
    }

    /// Pauses one durably accepted submission before it starts.
    pub fn mark_paused(&self, submission_id: &str) -> Result<(), PendingSubmissionError> {
        self.transition(
            submission_id,
            PendingSubmissionState::PausedPending,
            None,
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

    /// Explicitly terminalizes one accepted submission before Provider start.
    ///
    /// This is the recovery action for deterministic pre-start failures. The
    /// original identity remains in the permanent idempotency ledger and can
    /// never execute as fresh work later.
    pub fn cancel_unstarted(&self, submission_id: &str) -> Result<(), PendingSubmissionError> {
        self.transition(
            submission_id,
            PendingSubmissionState::TerminalCancelled,
            None,
            &[
                PendingSubmissionState::AcceptedPending,
                PendingSubmissionState::PausedPending,
                PendingSubmissionState::TerminalCancelled,
            ],
        )
    }

    /// Marks a started submission terminal without making it resumable.
    pub fn mark_terminal(
        &self,
        submission_id: &str,
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
            submission_id,
            state,
            Some(turn_id),
            &[PendingSubmissionState::Running, state],
        )
    }

    /// Finalizes custody from the authoritative terminal transcript outcome.
    ///
    /// Success becomes Committed. Error and Cancelled markers are mapped to
    /// their matching terminal states even when the caller is the legacy
    /// startup path named `mark_committed`. A real transcript file with no
    /// marker is ambiguous and remains Running. Preview requests are the only
    /// transcript-free successful structured kind and are validated from the
    /// immutable journal payload before taking that compatibility path.
    pub fn mark_committed(
        &self,
        submission_id: &str,
        turn_id: &str,
    ) -> Result<(), PendingSubmissionError> {
        let preview = self.get(submission_id)?.is_some_and(|record| {
            record.submission.common_kind() == Some(SubmissionKind::PreviewRequest)
        });
        if preview {
            return self.transition(
                submission_id,
                PendingSubmissionState::Committed,
                Some(turn_id),
                &[
                    PendingSubmissionState::Running,
                    PendingSubmissionState::Committed,
                ],
            );
        }

        let outcome = self.transcript_outcome_for_turn(turn_id)?;
        match outcome {
            Some(TurnTranscriptOutcome::Success) => self.transition(
                submission_id,
                PendingSubmissionState::Committed,
                Some(turn_id),
                &[
                    PendingSubmissionState::Running,
                    PendingSubmissionState::Committed,
                ],
            ),
            Some(TurnTranscriptOutcome::Cancelled) => self.mark_terminal(
                submission_id,
                PendingSubmissionState::TerminalCancelled,
                turn_id,
            ),
            Some(TurnTranscriptOutcome::Error) => self.mark_terminal(
                submission_id,
                PendingSubmissionState::TerminalError,
                turn_id,
            ),
            None if !self.session_file.exists() => self.transition(
                submission_id,
                PendingSubmissionState::Committed,
                Some(turn_id),
                &[
                    PendingSubmissionState::Running,
                    PendingSubmissionState::Committed,
                ],
            ),
            None => Err(PendingSubmissionError::InvalidTransition),
        }
    }

    /// Returns unstarted work in durable FIFO order.
    pub fn recover_unstarted(
        &self,
    ) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> {
        self.recover_states(&["accepted_pending", "paused_pending"])
    }

    /// Returns Running records for transcript-backed crash reconciliation.
    pub fn recover_running(&self) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> {
        self.recover_states(&["running"])
    }

    fn recover_states(
        &self,
        states: &[&str],
    ) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> {
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        let state_filter = states
            .iter()
            .map(|state| format!("'{state}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT receipt_id, fingerprint, submission_json, state, turn_id
             FROM pending_submissions
             WHERE state IN ({state_filter})
             ORDER BY rowid ASC"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map([], read_record_tuple)?;
        rows.map(|row| tuple_to_record(row?)).collect()
    }

    /// Returns one durable submission record that still retains its payload.
    pub fn get(
        &self,
        submission_id: &str,
    ) -> Result<Option<PendingSubmissionRecord>, PendingSubmissionError> {
        let _guard = self.guard()?;
        let connection = self.connection()?;
        ensure_schema(&connection)?;
        connection
            .query_row(
                "SELECT receipt_id, fingerprint, submission_json, state, turn_id
                 FROM pending_submissions WHERE batch_id = ?1",
                params![submission_id],
                read_record_tuple,
            )
            .optional()?
            .map(tuple_to_record)
            .transpose()
    }

    fn transcript_outcome_for_turn(
        &self,
        turn_id: &str,
    ) -> Result<Option<TurnTranscriptOutcome>, PendingSubmissionError> {
        if !self.session_file.exists() {
            return Ok(None);
        }
        let entries = if self
            .session_file
            .extension()
            .and_then(|value| value.to_str())
            == Some("jsonl")
        {
            JsonlSessionStore.read_entries(self.session_file.as_ref())?
        } else {
            CompactTextSessionStore.read_entries(self.session_file.as_ref())?
        };
        Ok(entries
            .into_iter()
            .filter_map(|entry| decode_turn_transcript_outcome(&entry.content))
            .filter(|record| record.turn_id == turn_id)
            .map(|record| record.outcome)
            .next_back())
    }

    fn transition(
        &self,
        submission_id: &str,
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
                params![submission_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(PendingSubmissionError::MissingRecord)?;
        if !expected.contains(&decode_state(&current)?) {
            return Err(PendingSubmissionError::InvalidTransition);
        }
        transaction.execute(
            "UPDATE pending_submissions SET state = ?2, turn_id = ?3 WHERE batch_id = ?1",
            params![submission_id, encode_state(next), turn_id],
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

    fn ensure_transcript_owner_marker(&self) -> Result<(), PendingSubmissionError> {
        if self.session_file.exists() {
            return Ok(());
        }
        if let Some(parent) = self.session_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.session_file.as_ref())
        {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
        connection.busy_timeout(SQLITE_BUSY_TIMEOUT)?;
        retry_sqlite_busy(SQLITE_BUSY_TIMEOUT, || {
            connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")
        })?;
        Ok(connection)
    }
}

#[cfg(test)]
mod tests;
