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
    owner: Arc<()>,
    session_id: Uuid,
    entries: Vec<(String, Message)>,
}

/// Resumable in-memory history read, committing at most one bounded page per step.
/// The final snapshot retains the full model history; this bounds work per step,
/// not total retained history memory. No partially read snapshot can be published.
pub struct DurableHistoryScan {
    source: DurableRecoverySource,
    stamp: crate::recovery_stamp::FileStamp,
    offset: u64,
    decoder: crate::jsonl::HistoryMessageDecoder,
    entries: Vec<(String, Message)>,
    complete: bool,
}

impl DurableHistoryScan {
    fn new(source: DurableRecoverySource) -> Result<Self, SessionError> {
        let stamp = source
            .recovery_metadata()
            .map_err(|_| SessionError::DurableTurn("history source metadata unavailable".into()))?;
        if crate::recovery_stamp::length(&stamp) != Some(source.source_end()) {
            return Err(SessionError::DurableTurn(
                "history source changed during capture".into(),
            ));
        }
        Ok(Self {
            source,
            stamp,
            offset: 0,
            decoder: Default::default(),
            entries: Vec::new(),
            complete: false,
        })
    }

    /// Read and convert one page with cooperative cancellation/deadline checks.
    /// A stopped step leaves the committed cursor and pairing state unchanged.
    /// Kernel reads and individual record conversions remain synchronous.
    pub fn step(
        &mut self,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<bool, crate::history_page::HistoryPageError> {
        use crate::history_page::HistoryPageError;
        let check = || {
            if stopped() || std::time::Instant::now() >= deadline {
                Err(HistoryPageError::Stopped)
            } else {
                Ok(())
            }
        };
        check()?;
        self.check_current()?;
        if self.complete {
            return Ok(true);
        }
        let page = self.source.read_page(self.offset, deadline, &stopped)?;
        let mut decoder = self.decoder.clone();
        let mut entries = Vec::new();
        for entry in page.entries {
            check()?;
            if let Some(message) = decoder.decode(entry) {
                entries.push(message);
            }
        }
        self.check_current()?;
        check()?;
        self.entries.extend(entries);
        self.decoder = decoder;
        self.offset = page.next_offset;
        self.complete = page.complete;
        Ok(self.complete)
    }

    /// Publish only a complete, freshly revalidated storage-issued snapshot.
    pub fn finish(self) -> Result<DurableHistorySnapshot, crate::history_page::HistoryPageError> {
        if !self.complete {
            return Err(crate::history_page::HistoryPageError::Stopped);
        }
        self.check_current()?;
        Ok(DurableHistorySnapshot {
            owner: self.source.descriptor.owner.clone(),
            session_id: self.source.session_id(),
            entries: self.entries,
        })
    }

    fn check_current(&self) -> Result<(), crate::history_page::HistoryPageError> {
        if crate::recovery_seal::same_file_version(&self.stamp, &self.source.recovery_metadata()?)?
        {
            Ok(())
        } else {
            Err(crate::history_page::HistoryPageError::Stale)
        }
    }
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

/// Storage-issued byte cutoff for bounded host recovery maintenance.
/// This binds the configured durable source, not permission or content freshness.
/// Hosts must still verify prefix digests and current branch before disclosure.
#[derive(Clone)]
pub struct DurableRecoverySource {
    descriptor: Arc<RecoverySourceDescriptor>,
    source_end: u64,
}

struct RecoverySourceDescriptor {
    write_lock: Arc<std::sync::Mutex<()>>,
    owner: Arc<()>,
    session_id: Uuid,
    branch_id: String,
    path: std::path::PathBuf,
}

/// A bounded original record retained from a storage-owned source read.
/// This detects record replacement; it does not grant branch or permission authority.
#[derive(Clone)]
pub struct DurableRecoveryRecord {
    source: DurableRecoverySource,
    range: std::ops::Range<u64>,
    digest: [u8; 32],
}

impl DurableRecoveryRecord {
    /// Reread source and optional committed terminal from the same captured
    /// storage source, then apply receipt eligibility and fixed response bounds.
    /// Combined record bytes cannot exceed one source-page budget. The host must
    /// separately validate current branch/prefix and permission before delivery.
    pub fn recover_text(
        &self,
        terminal: Option<&Self>,
        kind: crate::history_projection::RecoveryTextKind,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<
        Option<crate::history_projection::RecoveryText>,
        crate::history_page::HistoryPageError,
    > {
        use crate::history_page::{HISTORY_PAGE_BYTES, HistoryPageError};
        let own_bytes = self.range.end - self.range.start;
        if let Some(terminal) = terminal {
            if !Arc::ptr_eq(&self.source.descriptor, &terminal.source.descriptor)
                || self.source.source_end != terminal.source.source_end
            {
                return Err(HistoryPageError::Stale);
            }
            let total = own_bytes
                .checked_add(terminal.range.end - terminal.range.start)
                .ok_or(HistoryPageError::RecordTooLarge)?;
            if total > HISTORY_PAGE_BYTES as u64 {
                return Err(HistoryPageError::RecordTooLarge);
            }
        }
        let source = self.reread(deadline, &stopped)?;
        let terminal = terminal
            .map(|record| record.reread(deadline, &stopped))
            .transpose()?;
        let text =
            crate::history_projection::recovery_response_text(&source, terminal.as_ref(), kind);
        if stopped() || std::time::Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        Ok(text)
    }

    /// Reread and compare the original serialized entry before projection.
    pub fn reread(
        &self,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<SessionEntry, crate::history_page::HistoryPageError> {
        use sha2::{Digest, Sha256};
        let entry = crate::history_page::read_history_record(
            &self.source.descriptor.path,
            crate::history_page::HistoryEncoding::Tlog,
            self.range.clone(),
            deadline,
            &stopped,
        )?;
        let encoded = serde_json::to_vec(&entry)
            .map_err(|_| crate::history_page::HistoryPageError::InvalidRecord)?;
        let digest: [u8; 32] = Sha256::digest(encoded).into();
        if digest != self.digest {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        if stopped() || std::time::Instant::now() >= deadline {
            return Err(crate::history_page::HistoryPageError::Stopped);
        }
        Ok(entry)
    }
}

impl DurableRecoverySource {
    /// Try to serialize a bounded synchronous recovery read with this owner's
    /// writes. Returns None on contention or a poisoned lock. This guard grants
    /// no content or permission authority: verify the original prefix separately.
    /// Release all source handles before dropping the guard; never hold it across
    /// provider/tool awaits. Independently reopened owners and external writers
    /// still require the platform's restrictive source handles and revalidation.
    pub fn try_lock_for_recovery_read(&self) -> Option<std::sync::MutexGuard<'_, ()>> {
        self.descriptor.write_lock.try_lock().ok()
    }

    /// Restrict this configured source to an earlier cutoff. This does not verify
    /// bytes or grant branch authority; complete prefix verification is required.
    pub fn recovery_prefix_at(
        &self,
        cutoff: u64,
    ) -> Result<Self, crate::history_page::HistoryPageError> {
        if cutoff > self.source_end {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        let mut source = self.clone();
        source.source_end = cutoff;
        Ok(source)
    }

    pub(crate) fn read_recovery_bytes(
        &self,
        offset: u64,
        end: u64,
        expected: &crate::recovery_stamp::FileStamp,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<Vec<u8>, crate::history_page::HistoryPageError> {
        use crate::history_page::{HISTORY_PAGE_BYTES, HistoryPageError};
        use std::io::{Read, Seek, SeekFrom};
        if offset > end || end > self.source_end {
            return Err(HistoryPageError::Stale);
        }
        let mut file = crate::history_page::open_source(&self.descriptor.path)?;
        if !crate::recovery_seal::same_file_version(
            &crate::recovery_stamp::from_file(&file)?,
            expected,
        )? {
            return Err(HistoryPageError::Stale);
        }
        file.seek(SeekFrom::Start(offset))?;
        let size = (end - offset).min(HISTORY_PAGE_BYTES as u64) as usize;
        let mut bytes = vec![0; size];
        for chunk in bytes.chunks_mut(8192) {
            if stopped() || std::time::Instant::now() >= deadline {
                return Err(HistoryPageError::Stopped);
            }
            file.read_exact(chunk)?;
        }
        if stopped() || std::time::Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        if !crate::recovery_seal::same_file_version(
            &crate::recovery_stamp::from_file(&file)?,
            expected,
        )? {
            return Err(HistoryPageError::Stale);
        }
        Ok(bytes)
    }
    pub(crate) fn recovery_metadata(
        &self,
    ) -> Result<crate::recovery_stamp::FileStamp, crate::history_page::HistoryPageError> {
        let metadata = std::fs::symlink_metadata(&self.descriptor.path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(crate::history_page::HistoryPageError::UnsupportedSource);
        }
        if metadata.len() < self.source_end {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        let file = crate::history_page::open_source(&self.descriptor.path)?;
        if file.metadata()?.len() < self.source_end {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        crate::recovery_stamp::from_file(&file)
    }
    /// Durable owner of the captured source; callers cannot supply another path.
    pub fn session_id(&self) -> Uuid {
        self.descriptor.session_id
    }
    /// Branch identity selected by the configured durable owner.
    pub fn branch_id(&self) -> &str {
        &self.descriptor.branch_id
    }
    /// Captured byte cutoff, excluding future appended turns.
    pub fn source_end(&self) -> u64 {
        self.source_end
    }
    /// Capture one original record within this source cutoff. The locator must
    /// come from host pagination; neither locator nor returned record grants scope.
    pub fn capture_record(
        &self,
        range: std::ops::Range<u64>,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<DurableRecoveryRecord, crate::history_page::HistoryPageError> {
        use sha2::{Digest, Sha256};
        if range.end > self.source_end {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        let entry = crate::history_page::read_history_record(
            &self.descriptor.path,
            crate::history_page::HistoryEncoding::Tlog,
            range.clone(),
            deadline,
            &stopped,
        )?;
        let encoded = serde_json::to_vec(&entry)
            .map_err(|_| crate::history_page::HistoryPageError::InvalidRecord)?;
        let digest = Sha256::digest(encoded).into();
        if stopped() || std::time::Instant::now() >= deadline {
            return Err(crate::history_page::HistoryPageError::Stopped);
        }
        Ok(DurableRecoveryRecord {
            source: self.clone(),
            range,
            digest,
        })
    }

    /// Read an original record within this captured cutoff for host maintenance.
    /// The locator and returned entry do not establish branch or prefix authority.
    pub fn read_record(
        &self,
        range: std::ops::Range<u64>,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<SessionEntry, crate::history_page::HistoryPageError> {
        if range.end > self.source_end {
            return Err(crate::history_page::HistoryPageError::Stale);
        }
        crate::history_page::read_history_record(
            &self.descriptor.path,
            crate::history_page::HistoryEncoding::Tlog,
            range,
            deadline,
            stopped,
        )
    }

    /// Read one bounded original page. Offsets are host maintenance cursors,
    /// never model-controlled paths or authorization capabilities.
    pub fn read_page(
        &self,
        offset: u64,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<crate::history_page::HistoryPage, crate::history_page::HistoryPageError> {
        crate::history_page::read_history_page(
            &self.descriptor.path,
            crate::history_page::HistoryEncoding::Tlog,
            offset,
            self.source_end,
            deadline,
            stopped,
        )
    }
}

/// Opaque capability to remove one successfully saved scan record.
/// Bound to its issuing durable handle; not serializable or query authority.
#[cfg(any(unix, windows))]
pub struct RecoveryScanReceipt {
    owner: Arc<()>,
    payload: String,
}

/// Restored host scan and exact-record cleanup capability.
#[cfg(any(unix, windows))]
pub struct RecoveryScanResume {
    /// Scan requiring completion before any query authorization.
    pub scan: crate::RecoveryPrefixScan,
    /// Cleanup capability bound to the restoring durable handle.
    pub receipt: RecoveryScanReceipt,
}

/// A UUID-backed durable session bound to an opaque host external ID.
#[derive(Debug, Clone)]
pub struct DurableSession {
    recovery_owner: Arc<()>,
    external_id: String,
    session: Session,
    bindings_path: std::path::PathBuf,
}

impl DurableSession {
    /// Bind a host-selected existing TLOG for history preparation without moving
    /// or rewriting its transcript. The host must hold its session execution
    /// lease. This does not certify parent integrity or authorize model reads;
    /// source sealing, indexing, projection and permission checks still apply.
    ///
    /// Bindings are isolated from embedded sessions: reopening requires the
    /// selected Session again, never an external ID interpreted as a path.
    pub fn bind_existing_history(session: Session) -> Result<Self, SessionError> {
        recovery_storage_boundary(|| {
            let expected = format!("{}.tlog", session.id);
            if session.file_path.file_name().and_then(|name| name.to_str()) != Some(&expected)
                || session.store.file_extension() != "tlog"
            {
                return Err(SessionError::DurableTurn(
                    "history binding requires a UUID TLOG".into(),
                ));
            }
            let metadata = fs::symlink_metadata(&session.file_path)?;
            if !metadata.file_type().is_file() {
                return Err(SessionError::DurableTurn(
                    "history binding requires a regular transcript".into(),
                ));
            }
            let directory = session.file_path.parent().ok_or_else(|| {
                SessionError::DurableTurn("history binding has no directory".into())
            })?;
            let bindings_path = directory.join("history-bindings.sqlite");
            if let Ok(metadata) = fs::symlink_metadata(&bindings_path)
                && !metadata.file_type().is_file()
            {
                return Err(SessionError::DurableTurn(
                    "history binding database is not a regular file".into(),
                ));
            }
            let connection = open_bindings(&bindings_path)?;
            let identity = session.id.to_string();
            connection
                .execute(
                    "INSERT OR IGNORE INTO durable_bindings(external_id,session_id) VALUES(?1,?1)",
                    [&identity],
                )
                .map_err(sql_error)?;
            let valid: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM durable_bindings WHERE external_id=?1 AND session_id=?1)",
                [&identity], |row| row.get(0),
            ).map_err(sql_error)?;
            if !valid {
                return Err(SessionError::DurableTurn(
                    "history binding identity conflict".into(),
                ));
            }
            Ok(Self::new(identity, session, bindings_path))
        })
    }

    /// Reconcile at most 32 recorded fork intents under this storage root.
    /// Active locks are skipped; only exact intent-derived unpublished files may
    /// be removed. Returns (scanned, retired, failed, next cursor). Continue with
    /// the cursor, then restart from None on the next maintenance sweep.
    /// Published files are retained; this does not certify their content integrity.
    /// This host maintenance operation does not grant model write authority.
    #[cfg(any(unix, windows))]
    pub fn recover_fork_intents(
        &self,
        budget: &crate::sqlite::RecoveryWorkBudget,
        after: Option<&str>,
    ) -> Result<(usize, usize, usize, Option<String>), SessionError> {
        recovery_storage_boundary(|| {
            let connection = open_binding_connection(&self.bindings_path)?;
            let _guard = crate::sqlite::install_budget(&connection, budget)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            let directory = self
                .file_path()
                .parent()
                .ok_or_else(|| SessionError::DurableTurn("fork directory unavailable".into()))?;
            let ids = connection
                .prepare("SELECT child_id FROM durable_fork_intents WHERE child_id > ?1 ORDER BY child_id LIMIT 32")
                .map_err(sql_error)?
                .query_map([after.unwrap_or("")], |row| row.get::<_, String>(0))
                .map_err(sql_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(sql_error)?;
            let scanned = ids.len();
            let next = if scanned == 32 {
                ids.last().cloned()
            } else {
                None
            };
            let mut failed = 0;
            let mut retired = 0;
            for encoded in ids {
                budget
                    .check()
                    .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
                let result = (|| -> Result<bool, SessionError> {
                    let id = Uuid::parse_str(&encoded).map_err(|_| {
                        SessionError::DurableTurn("fork intent identity invalid".into())
                    })?;
                    if id.to_string() != encoded {
                        return Err(SessionError::DurableTurn(
                            "fork intent identity noncanonical".into(),
                        ));
                    }
                    let lock = open_fork_lock(&directory.join(format!("{id}.fork-lock")), false)?;
                    if !lock.metadata()?.is_file() {
                        return Err(SessionError::DurableTurn("fork lock is not regular".into()));
                    }
                    match lock.try_lock() {
                        Ok(()) => {}
                        Err(std::fs::TryLockError::WouldBlock) => return Ok(false),
                        Err(error) => {
                            return Err(SessionError::DurableTurn(format!(
                                "fork lock failed: {error}"
                            )));
                        }
                    }
                    let tx = connection.unchecked_transaction().map_err(sql_error)?;
                    let intent: Option<(String, String, i64)> = tx.query_row("SELECT parent_id,external_id,version FROM durable_fork_intents WHERE child_id=?1", [&encoded], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(sql_error)?;
                    let Some((parent, external, version)) = intent else {
                        return Ok(false);
                    };
                    if version != 1 {
                        return Err(SessionError::DurableTurn("unsupported fork intent".into()));
                    }
                    let bound: Option<String> = tx
                        .query_row(
                            "SELECT external_id FROM durable_bindings WHERE session_id=?1",
                            [&encoded],
                            |r| r.get(0),
                        )
                        .optional()
                        .map_err(sql_error)?;
                    let path = directory.join(format!("{id}.tlog"));
                    let metadata = match fs::symlink_metadata(&path) {
                        Ok(value) if value.is_file() && !value.file_type().is_symlink() => {
                            Some(value)
                        }
                        Ok(_) => {
                            return Err(SessionError::DurableTurn(
                                "fork recovery source is not regular".into(),
                            ));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                        Err(e) => return Err(e.into()),
                    };
                    #[cfg(windows)]
                    if let Some(metadata) = &metadata {
                        use std::os::windows::fs::MetadataExt;
                        if metadata.file_attributes()
                            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                            != 0
                        {
                            return Err(SessionError::DurableTurn(
                                "fork recovery source is a reparse point".into(),
                            ));
                        }
                    }
                    if let Some(bound) = bound {
                        let cutoff: i64 = tx.query_row("SELECT source_end FROM durable_fork_lineage WHERE child_id=?1 AND parent_id=?2", params![encoded,parent], |r| r.get(0)).map_err(sql_error)?;
                        if bound != external
                            || cutoff < 0
                            || metadata.as_ref().is_none_or(|m| m.len() < cutoff as u64)
                        {
                            return Err(SessionError::DurableTurn(
                                "published fork recovery mismatch".into(),
                            ));
                        }
                    } else if metadata.is_some() {
                        fs::remove_file(&path)?;
                        sync_fork_directory(directory)?;
                    }
                    budget
                        .check()
                        .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
                    tx.execute(
                        "DELETE FROM durable_fork_intents WHERE child_id=?1",
                        [&encoded],
                    )
                    .map_err(sql_error)?;
                    tx.commit().map_err(sql_error)?;
                    Ok(true)
                })();
                match result {
                    Ok(true) => retired += 1,
                    Ok(false) => {}
                    Err(_) => failed += 1,
                }
                budget
                    .check()
                    .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            }
            Ok((scanned, retired, failed, next))
        })
    }

    /// Fork the committed transcript tip into a new host-owned durable identity.
    ///
    /// The host must hold its Actor idle/admission gate for this operation. This
    /// storage operation cannot detect an admitted turn that has not persisted.
    /// No index, checkpoint or scan authority is inherited. Cancellation and the
    /// total deadline are cooperative; regular-file I/O is synchronous.
    /// A process crash before binding publication can leave an unbound file.
    #[cfg(any(unix, windows))]
    pub fn fork_committed_tip(
        &self,
        external_id: &str,
        deadline: std::time::Instant,
        stopped: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Result<Self, SessionError> {
        recovery_storage_boundary(|| {
            validate_external_id(external_id)?;
            let deadline =
                deadline.min(std::time::Instant::now() + std::time::Duration::from_secs(10));
            let stopped = Arc::new(stopped);
            let stop_check = stopped.clone();
            let budget = crate::sqlite::RecoveryWorkBudget::with_cancellation_check(
                deadline.saturating_duration_since(std::time::Instant::now()),
                move || stop_check(),
            );
            let check = || {
                budget
                    .check()
                    .map_err(|e| SessionError::DurableTurn(e.to_string()))
            };
            let source_error =
                |e: crate::history_page::HistoryPageError| SessionError::DurableTurn(e.to_string());
            check()?;
            let source = self.capture_recovery_source()?;
            let mut scan = source.begin_prefix_seal().map_err(source_error)?;
            while !scan.step(deadline, || stopped()).map_err(source_error)? {}
            let seal = scan.finish().map_err(source_error)?;
            // Require contiguous, terminally recorded turns. This is storage
            // integrity validation, not proof that the Actor is idle.
            let mut offset = 0;
            let mut active_turn: Option<String> = None;
            let mut previous_entry: Option<String> = None;
            // Disposable uniqueness state spills to SQLite temporary storage;
            // avoids a transcript-sized Rust set; this is not a hard RSS limit.
            let connection = open_binding_connection(&self.bindings_path)?;
            let _guard = crate::sqlite::install_budget(&connection, &budget)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            connection
                .execute_batch(
                    "PRAGMA temp_store=FILE; PRAGMA temp.cache_size=-256;
                CREATE TEMP TABLE fork_entries(id TEXT PRIMARY KEY);
                CREATE TEMP TABLE fork_turns(id TEXT PRIMARY KEY);
                CREATE TEMP TABLE fork_calls(turn_id TEXT NOT NULL, id TEXT NOT NULL, pending INTEGER NOT NULL, PRIMARY KEY(turn_id,id));",
                )
                .map_err(sql_error)?;
            while offset < source.source_end() {
                let page = source
                    .read_page(offset, deadline, || stopped())
                    .map_err(source_error)?;
                for entry in page.entries {
                    check()?;
                    if entry.id.is_empty() || entry.parent_id != previous_entry {
                        return Err(SessionError::DurableTurn(
                            "fork parent chain is invalid".into(),
                        ));
                    }
                    connection
                        .execute("INSERT INTO fork_entries(id) VALUES(?1)", [&entry.id])
                        .map_err(sql_error)?;
                    previous_entry = Some(entry.id.clone());
                    let turn = entry.metadata.turn_id.as_deref().ok_or_else(|| {
                        SessionError::DurableTurn("fork entry has no turn identity".into())
                    })?;
                    if entry.role == "assistant" {
                        let mut remaining = entry.content.as_str();
                        while let Some(start) = remaining.find("```json-tool") {
                            check()?;
                            let inner = &remaining[start + "```json-tool".len()..];
                            let mut stream = serde_json::Deserializer::from_str(inner)
                                .into_iter::<serde_json::Value>();
                            let object = stream.next().and_then(Result::ok).ok_or_else(|| {
                                SessionError::DurableTurn("fork tool block is malformed".into())
                            })?;
                            let call_id = object
                                .get("id")
                                .and_then(serde_json::Value::as_str)
                                .filter(|id| !id.is_empty())
                                .ok_or_else(|| {
                                    SessionError::DurableTurn(
                                        "fork tool identity is missing".into(),
                                    )
                                })?;
                            if object
                                .get("name")
                                .and_then(serde_json::Value::as_str)
                                .is_none_or(str::is_empty)
                                || object.get("args").is_none()
                            {
                                return Err(SessionError::DurableTurn(
                                    "fork tool block is incomplete".into(),
                                ));
                            }
                            remaining = inner[stream.byte_offset()..]
                                .trim_start()
                                .strip_prefix("```")
                                .ok_or_else(|| {
                                    SessionError::DurableTurn("fork tool block is unclosed".into())
                                })?;
                            connection
                                .execute(
                                    "INSERT INTO fork_calls(turn_id,id,pending) VALUES(?1,?2,1)",
                                    params![turn, call_id],
                                )
                                .map_err(sql_error)?;
                        }
                    } else if entry.role == "system"
                        && (entry.content.starts_with("__OK__:")
                            || entry.content.starts_with("__ERROR__:"))
                    {
                        let (_, call_id, _) = crate::jsonl::parse_tool_result(&entry.content);
                        if connection
                            .execute(
                                "UPDATE fork_calls SET pending=0 WHERE turn_id=?1 AND id=?2 AND pending=1",
                                params![turn, call_id],
                            )
                            .map_err(sql_error)?
                            != 1
                        {
                            return Err(SessionError::DurableTurn(
                                "fork tool result is unpaired".into(),
                            ));
                        }
                    }
                    if active_turn
                        .as_deref()
                        .is_some_and(|current| current != turn)
                    {
                        return Err(SessionError::DurableTurn(
                            "fork contains an incomplete turn".into(),
                        ));
                    }
                    if active_turn.is_none() {
                        connection
                            .execute("INSERT INTO fork_turns(id) VALUES(?1)", [turn])
                            .map_err(sql_error)?;
                    }
                    active_turn = Some(turn.to_owned());
                    if is_turn_transcript_outcome_content(&entry.content) {
                        let pending: bool = connection
                            .query_row(
                                "SELECT EXISTS(SELECT 1 FROM fork_calls WHERE turn_id=?1 AND pending=1)",
                                [turn],
                                |row| row.get(0),
                            )
                            .map_err(sql_error)?;
                        if pending {
                            return Err(SessionError::DurableTurn(
                                "fork contains an incomplete tool batch".into(),
                            ));
                        }
                        let terminal = decode_turn_transcript_outcome(&entry.content)
                            .filter(|terminal| terminal.version == 1 && terminal.turn_id == turn)
                            .ok_or_else(|| {
                                SessionError::DurableTurn("fork terminal is invalid".into())
                            })?;
                        let _ = terminal;
                        crate::HistoryRecoveryReceipt::from_terminal_entry(&entry).ok_or_else(
                            || SessionError::DurableTurn("fork receipt is invalid".into()),
                        )?;
                        active_turn = None;
                    }
                }
                if page.next_offset <= offset {
                    return Err(SessionError::DurableTurn(
                        "fork scan made no progress".into(),
                    ));
                }
                offset = page.next_offset;
            }
            if active_turn.is_some() {
                return Err(SessionError::DurableTurn("fork tip is not terminal".into()));
            }
            seal.check_current().map_err(source_error)?;
            let id = Uuid::new_v4();
            let directory = self.file_path().parent().ok_or_else(|| {
                SessionError::DurableTurn("fork storage directory missing".into())
            })?;
            let path = directory.join(format!("{id}.tlog"));
            let lock_path = directory.join(format!("{id}.fork-lock"));
            let fork_lock = open_fork_lock(&lock_path, true)?;
            fork_lock
                .try_lock()
                .map_err(|_| SessionError::DurableTurn("fork lock unavailable".into()))?;
            fork_lock.sync_all()?;
            sync_fork_directory(directory)?;
            // Persist exact recovery ownership before creating any child bytes.
            // Keep the lock inode even after success: unlink/recreation could
            // let two processes hold different locks under the same name.
            connection.execute("INSERT INTO durable_fork_intents(child_id,parent_id,external_id,version) VALUES(?1,?2,?3,1)",
                params![id.to_string(), self.id().to_string(), external_id]).map_err(sql_error)?;
            #[cfg(test)]
            fork_crash_probe("intent");
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options
                    .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_WRITE_THROUGH);
            }
            let mut file = options.open(&path)?;
            struct UnpublishedFile(std::path::PathBuf, bool);
            impl Drop for UnpublishedFile {
                fn drop(&mut self) {
                    if !self.1 {
                        let _ = fs::remove_file(&self.0);
                    }
                }
            }
            let mut unpublished = UnpublishedFile(path.clone(), false);
            #[cfg(windows)]
            crate::recovery_stamp::verify_local_volume(&file)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            seal.copy_to_staging(&mut file, deadline, || stopped())
                .map_err(source_error)?;
            #[cfg(test)]
            fork_crash_probe("copied");
            file.sync_all()?;
            sync_fork_directory(directory)?;
            #[cfg(test)]
            fork_crash_probe("synced");
            seal.check_current().map_err(source_error)?;

            let tx = connection.unchecked_transaction().map_err(sql_error)?;
            let parent: String = tx
                .query_row(
                    "SELECT session_id FROM durable_bindings WHERE external_id=?1",
                    [&self.external_id],
                    |row| row.get(0),
                )
                .map_err(sql_error)?;
            if parent != self.id().to_string() {
                return Err(SessionError::DurableTurn(
                    "fork parent binding changed".into(),
                ));
            }
            let intent_valid: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM durable_fork_intents WHERE child_id=?1 AND parent_id=?2 AND external_id=?3 AND version=1)",
                params![id.to_string(), parent, external_id], |row| row.get(0)).map_err(sql_error)?;
            if !intent_valid {
                return Err(SessionError::DurableTurn(
                    "fork publication intent changed".into(),
                ));
            }
            // Publish only after the copied file is durable.
            tx.execute(
                "INSERT INTO durable_bindings(external_id,session_id) VALUES(?1,?2)",
                params![external_id, id.to_string()],
            )
            .map_err(sql_error)?;
            tx.execute("INSERT INTO durable_fork_lineage(child_id,parent_id,parent_branch,source_end) VALUES(?1,?2,?3,?4)",
                params![id.to_string(), parent, source.branch_id(), i64::try_from(source.source_end()).map_err(|_| SessionError::DurableTurn("fork source exceeds SQLite range".into()))?]).map_err(sql_error)?;
            #[cfg(test)]
            fork_crash_probe("publication-check");
            check()?;
            // A commit error can have an uncertain outcome. Preserve the file
            // once commit is attempted; recovery must reconcile it with bindings.
            seal.check_current().map_err(source_error)?;
            unpublished.1 = true;
            #[cfg(test)]
            fork_crash_probe("precommit");
            tx.commit().map_err(sql_error)?;
            #[cfg(test)]
            fork_crash_probe("published");
            // Publication is already durable. Cleanup failure must not turn it
            // into an apparent failed fork; a later recovery can retire intent.
            let _ = connection.execute(
                "DELETE FROM durable_fork_intents WHERE child_id=?1",
                [id.to_string()],
            );
            let mut session = Session::with_store(
                id,
                "embedded".into(),
                "embedded".into(),
                path,
                Arc::new(crate::CompactTextSessionStore),
            );
            bind_durable_root_branch(&mut session);
            Ok(Self::new(
                external_id.to_owned(),
                session,
                self.bindings_path.clone(),
            ))
        })
    }

    /// Save completed initial-scan pages in host-owned bindings (ADR-096).
    /// Only scans issued by this exact handle are accepted. Search-cache records
    /// cannot be imported through this API. Supported on Unix and local Windows sources.
    #[cfg(any(unix, windows))]
    pub fn save_recovery_scan(
        &self,
        scan: &crate::RecoveryPrefixScan,
        budget: &crate::sqlite::RecoveryWorkBudget,
    ) -> Result<RecoveryScanReceipt, SessionError> {
        recovery_storage_boundary(|| {
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            let stored = scan
                .stored(self)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            let payload = serde_json::to_string(&stored)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            if payload.len() > 4096 {
                return Err(SessionError::DurableTurn(
                    "scan checkpoint exceeds bound".into(),
                ));
            }
            // Schema was installed when this configured handle was opened. Avoid
            // open_bindings' initialization retries during bounded scan work.
            let connection = open_binding_connection(&self.bindings_path)?;
            let _budget_guard = crate::sqlite::install_budget(&connection, budget)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            connection
                .busy_timeout(std::time::Duration::ZERO)
                .map_err(sql_error)?;
            let transaction = connection.unchecked_transaction().map_err(sql_error)?;
            // The configured binding, not a serialized owner claim, grants storage scope.
            let valid: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM durable_bindings WHERE external_id=?1 AND session_id=?2)",
            params![self.external_id, self.id().to_string()], |row| row.get(0)).map_err(sql_error)?;
            if !valid {
                return Err(SessionError::DurableTurn("stale durable binding".into()));
            }
            let previous: Option<String> = transaction.query_row(
            "SELECT payload FROM durable_recovery_scans WHERE session_id=?1 AND length(payload)<=4096",
            [self.id().to_string()], |row| row.get(0)).optional().map_err(sql_error)?;
            if let Some(previous) = previous
                && let Ok(previous) =
                    serde_json::from_str::<crate::recovery_seal::StoredScan>(&previous)
                && !stored.supersedes(&previous)
            {
                return Err(SessionError::DurableTurn("scan progress superseded".into()));
            }
            scan.stored(self)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            transaction.execute("INSERT INTO durable_recovery_scans(session_id,payload) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET payload=excluded.payload",
            params![self.id().to_string(), payload]).map_err(sql_error)?;
            scan.stored(self)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            transaction.commit().map_err(sql_error)?;
            Ok(RecoveryScanReceipt {
                owner: self.recovery_owner.clone(),
                payload,
            })
        })
    }

    /// Restore host-owned initial scan progress for an unchanged source version.
    /// Missing, malformed or stale records return None and require fresh scanning.
    /// The restored scan still must complete before it can mint a prefix seal.
    #[cfg(any(unix, windows))]
    pub fn restore_recovery_scan(
        &self,
        budget: &crate::sqlite::RecoveryWorkBudget,
    ) -> Result<Option<RecoveryScanResume>, SessionError> {
        let source = self.capture_recovery_source()?;
        self.restore_recovery_scan_for_source(&source, budget)
    }

    /// Restore progress against the exact owner-issued source used by the caller.
    /// This never recaptures a later cutoff between preparation and index creation.
    #[cfg(any(unix, windows))]
    pub fn restore_recovery_scan_for_source(
        &self,
        source: &DurableRecoverySource,
        budget: &crate::sqlite::RecoveryWorkBudget,
    ) -> Result<Option<RecoveryScanResume>, SessionError> {
        if !self.owns_recovery_source(source) {
            return Err(SessionError::DurableTurn("foreign recovery source".into()));
        }
        recovery_storage_boundary(|| {
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            // Schema was installed when this configured handle was opened. Avoid
            // open_bindings' initialization retries during bounded scan work.
            let connection = open_binding_connection(&self.bindings_path)?;
            let _budget_guard = crate::sqlite::install_budget(&connection, budget)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            connection
                .busy_timeout(std::time::Duration::ZERO)
                .map_err(sql_error)?;
            let payload: Option<String> = connection.query_row(
            "SELECT s.payload FROM durable_recovery_scans s JOIN durable_bindings b ON b.session_id=s.session_id WHERE b.external_id=?1 AND b.session_id=?2 AND length(s.payload)<=4096",
            params![self.external_id, self.id().to_string()], |row| row.get(0)).optional().map_err(sql_error)?;
            let Some(payload) = payload else {
                return Ok(None);
            };
            let Ok(stored) = serde_json::from_str::<crate::recovery_seal::StoredScan>(&payload)
            else {
                return Ok(None);
            };
            let result = stored
                .restore(source.clone())
                .ok()
                .map(|scan| RecoveryScanResume {
                    scan,
                    receipt: RecoveryScanReceipt {
                        owner: self.recovery_owner.clone(),
                        payload,
                    },
                });
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            Ok(result)
        })
    }

    /// Remove only the persisted progress identified by this saved-record receipt.
    /// A stale execution cannot erase a newer checkpoint. Returns whether deleted.
    #[cfg(any(unix, windows))]
    pub fn clear_recovery_scan(
        &self,
        receipt: &RecoveryScanReceipt,
        budget: &crate::sqlite::RecoveryWorkBudget,
    ) -> Result<bool, SessionError> {
        recovery_storage_boundary(|| {
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            if !Arc::ptr_eq(&receipt.owner, &self.recovery_owner) {
                return Err(SessionError::DurableTurn("foreign scan receipt".into()));
            }
            let connection = open_binding_connection(&self.bindings_path)?;
            let _guard = crate::sqlite::install_budget(&connection, budget)
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            let tx = connection.unchecked_transaction().map_err(sql_error)?;
            let deleted = tx.execute("DELETE FROM durable_recovery_scans WHERE session_id=?1 AND payload=?2 AND EXISTS(SELECT 1 FROM durable_bindings WHERE external_id=?3 AND session_id=?1)",
                params![self.id().to_string(), receipt.payload, self.external_id]).map_err(sql_error)?;
            budget
                .check()
                .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
            tx.commit().map_err(sql_error)?;
            Ok(deleted != 0)
        })
    }

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
            recovery_owner: Arc::new(()),
            external_id,
            session,
            bindings_path,
        }
    }

    /// Whether this snapshot was issued by this configured durable owner or a
    /// clone of its handle. Independently reopened handles must issue new snapshots
    /// after rebinding; equal IDs, paths or copied files are not owner authority.
    pub fn owns_history_snapshot(&self, snapshot: &DurableHistorySnapshot) -> bool {
        Arc::ptr_eq(&self.recovery_owner, &snapshot.owner) && self.id() == snapshot.session_id
    }

    /// Whether a bounded source was captured by this configured owner/branch.
    /// This proves handle provenance only; current permission and prefix checks
    /// remain mandatory before disclosure. A reopened owner needs a fresh source.
    pub fn owns_recovery_source(&self, source: &DurableRecoverySource) -> bool {
        Arc::ptr_eq(&self.recovery_owner, &source.descriptor.owner)
            && self.id() == source.session_id()
            && self.session.current_branch == source.branch_id()
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

    /// Existing Session index location owned by this durable storage root.
    /// Obtaining this path does not authorize model-triggered creation or repair.
    pub fn recovery_index_path(&self) -> Result<std::path::PathBuf, SessionError> {
        self.bindings_path
            .parent()
            .map(|root| root.join("index.db"))
            .ok_or_else(|| SessionError::DurableTurn("missing storage root".into()))
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
            owner: self.recovery_owner.clone(),
            session_id: self.id(),
            entries: self.read_messages_with_entry_ids()?,
        })
    }

    /// Begin a resumable bounded-page snapshot read of the current durable source.
    /// No transcript bytes are read here. Source changes invalidate the scan;
    /// callers must not dispatch a turn using a partial or failed snapshot.
    pub fn begin_history_scan(&self) -> Result<DurableHistoryScan, SessionError> {
        DurableHistoryScan::new(self.capture_recovery_source()?)
    }

    /// Capture the configured durable source for incremental bounded maintenance.
    /// Capturing a cutoff does not authorize raw disclosure or prove immutability.
    pub fn capture_recovery_source(&self) -> Result<DurableRecoverySource, SessionError> {
        let _guard =
            self.session.write_lock.lock().map_err(|_| {
                SessionError::DurableTurn("recovery source lock unavailable".into())
            })?;
        let metadata = fs::symlink_metadata(self.file_path())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(SessionError::DurableTurn(
                "recovery source is not a regular file".into(),
            ));
        }
        Ok(DurableRecoverySource {
            descriptor: Arc::new(RecoverySourceDescriptor {
                write_lock: self.session.write_lock.clone(),
                owner: self.recovery_owner.clone(),
                session_id: self.id(),
                branch_id: self.session.current_branch.clone(),
                path: self.file_path().to_path_buf(),
            }),
            source_end: metadata.len(),
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
    /// Projections must originate from the live execution owner. Only matches
    /// to final policy-filtered tool text after line-ending normalization receive
    /// a receipt; denied persistence or redacted projections confer no permission.
    /// The receipt is stored in the existing terminal marker, never a second store.
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
                && prior_entries.iter().zip(&filtered).all(|(entry, message)| {
                    let reasoning = match message {
                        Message::Assistant { reasoning, .. } => reasoning.as_ref(),
                        _ => None,
                    };
                    entry.metadata.reasoning.as_ref() == reasoning
                })
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
                    reasoning: match message {
                        Message::Assistant { reasoning, .. } => reasoning.clone(),
                        _ => None,
                    },
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

        let mut tip = self
            .session
            .last_entry_id
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        crate::compact_text::append_entries_atomically(
            &self.session.file_path,
            &existing[..entry_start],
            &existing[entry_start..],
        )?;
        *tip = existing.last().map(|entry| entry.id.clone());
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

// Durable bindings currently expose one root branch per durable UUID. Reopening
// must not mint a new branch identity; ephemeral Session::new roots are unsuitable
// for persistent source provenance. This does not grant authority to copied handles.
fn bind_durable_root_branch(session: &mut Session) {
    let branch = format!("durable-root:{}", session.id);
    if let Some(mut root) = session.branches.remove(&session.current_branch) {
        root.root_id = branch.clone();
        session.branches.insert(branch.clone(), root);
    }
    session.current_branch = branch;
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
    let mut session = Session::with_store(
        id,
        "embedded".into(),
        "embedded".into(),
        file_path,
        Arc::new(crate::CompactTextSessionStore),
    );
    bind_durable_root_branch(&mut session);
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
    let mut session = Session::with_store(
        id,
        "embedded".into(),
        "embedded".into(),
        file_path,
        Arc::new(crate::CompactTextSessionStore),
    );
    bind_durable_root_branch(&mut session);
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

fn open_binding_connection(path: &std::path::Path) -> Result<Connection, SessionError> {
    // Enforce at SQLite open time as well as at the caller's metadata check:
    // a binding path can be replaced between those two operations.
    // SQLite also rejects ancestor symlinks. Resolve only the parent to retain
    // host directory aliases (including macOS /tmp), never the database itself.
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let directory = fs::canonicalize(parent.unwrap_or_else(|| std::path::Path::new(".")))?;
    let name = path
        .file_name()
        .ok_or_else(|| SessionError::DurableTurn("binding database name unavailable".into()))?;
    Connection::open_with_flags(
        directory.join(name),
        rusqlite::OpenFlags::default() | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sql_error)
}

#[cfg(test)]
thread_local! {
    static BINDING_INIT_RETRY_OBSERVER: std::cell::RefCell<Option<Box<dyn FnMut()>>> = const { std::cell::RefCell::new(None) };
}

fn open_bindings(path: &std::path::Path) -> Result<Connection, SessionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = open_binding_connection(path)?;
    // rusqlite opens connections with a five-second busy handler. Disable it
    // during initialization so each explicit retry cannot inherit that wait.
    connection
        .busy_timeout(std::time::Duration::ZERO)
        .map_err(sql_error)?;
    // Initialize WAL mode and schema without busy_timeout so concurrent
    // first-time opens fail fast instead of blocking for 5s each. The
    // 500 ms retry deadline and at most 20 retries let the first writer finish
    // without adding more waits after scheduler delays. Synchronous SQLite/open
    // latency is not a hard real-time bound. Only DatabaseBusy / DatabaseLocked
    // are retried; all other
    // SQLite errors propagate immediately. After init succeeds, apply the
    // 5-second busy_timeout for subsequent transaction operations.
    let init_sql = "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; \
         CREATE TABLE IF NOT EXISTS durable_bindings \
         (external_id TEXT PRIMARY KEY, session_id TEXT NOT NULL UNIQUE); \
         CREATE TABLE IF NOT EXISTS durable_recovery_scans \
         (session_id TEXT PRIMARY KEY, payload TEXT NOT NULL); \
         CREATE TABLE IF NOT EXISTS durable_fork_lineage \
         (child_id TEXT PRIMARY KEY, parent_id TEXT NOT NULL, parent_branch TEXT NOT NULL, source_end INTEGER NOT NULL); \
         CREATE TABLE IF NOT EXISTS durable_fork_intents \
         (child_id TEXT PRIMARY KEY, parent_id TEXT NOT NULL, external_id TEXT NOT NULL, version INTEGER NOT NULL); \
         CREATE TRIGGER IF NOT EXISTS durable_binding_lineage_cleanup AFTER DELETE ON durable_bindings \
         BEGIN DELETE FROM durable_fork_lineage WHERE child_id=OLD.session_id; END; \
         CREATE TRIGGER IF NOT EXISTS durable_binding_scan_cleanup AFTER DELETE ON durable_bindings \
         BEGIN DELETE FROM durable_recovery_scans WHERE session_id=OLD.session_id; END;";
    const MAX_INIT_RETRIES: u32 = 20;
    const RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    let mut retries = 0u32;
    loop {
        match connection.execute_batch(init_sql) {
            Ok(()) => break,
            Err(error @ rusqlite::Error::SqliteFailure(_, _))
                if retries < MAX_INIT_RETRIES
                    && std::time::Instant::now() < deadline
                    && matches!(
                        error.sqlite_error_code(),
                        Some(
                            rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                        )
                    ) =>
            {
                #[cfg(test)]
                BINDING_INIT_RETRY_OBSERVER.with(|observer| {
                    if let Some(observer) = observer.borrow_mut().as_mut() {
                        observer();
                    }
                });
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(sql_error(error));
                }
                retries += 1;
                std::thread::sleep(RETRY_INTERVAL.min(remaining));
                if std::time::Instant::now() >= deadline {
                    return Err(sql_error(error));
                }
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

// The retained lock name is never replaced by this protocol. Windows denies
// delete sharing while open; both platforms reject final symlink/reparse objects.
#[cfg(any(unix, windows))]
fn open_fork_lock(path: &std::path::Path, create: bool) -> Result<fs::File, SessionError> {
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create_new(create);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_FLAG_WRITE_THROUGH, FILE_SHARE_READ,
            FILE_SHARE_WRITE,
        };
        options
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_WRITE_THROUGH);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(SessionError::DurableTurn("fork lock is not regular".into()));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        crate::recovery_stamp::verify_local_volume(&file)
            .map_err(|e| SessionError::DurableTurn(e.to_string()))?;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(SessionError::DurableTurn(
                "fork lock is a reparse point".into(),
            ));
        }
    }
    Ok(file)
}

// Windows uses write-through creation and file sync before SQLite publication.
// There is no portable directory fsync there: process-exit recovery is supported,
// but this does not promise Unix-equivalent namespace durability after power loss.
#[cfg(any(unix, windows))]
fn sync_fork_directory(directory: &std::path::Path) -> Result<(), SessionError> {
    #[cfg(unix)]
    fs::File::open(directory)?.sync_all()?;
    #[cfg(windows)]
    let _ = directory;
    Ok(())
}

#[cfg(any(unix, windows))]
fn recovery_storage_boundary<T>(
    operation: impl FnOnce() -> Result<T, SessionError>,
) -> Result<T, SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        Err(SessionError::DurableTurn(
            "recovery storage operation panicked".into(),
        ))
    })
}

#[cfg(test)]
thread_local! {
    static FORK_CANCEL_PROBE: std::cell::RefCell<Option<(&'static str, Arc<std::sync::atomic::AtomicBool>)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn fork_crash_probe(phase: &str) {
    FORK_CANCEL_PROBE.with(|probe| {
        let mut probe = probe.borrow_mut();
        if probe.as_ref().is_some_and(|(target, _)| *target == phase) {
            let (_, cancelled) = probe.take().expect("matched probe");
            cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
    if std::env::var("TALOS_TEST_FORK_CRASH_PHASE").as_deref() == Ok(phase) {
        // Dedicated child fixture only; process::exit bypasses Rust Drop cleanup.
        std::process::exit(42);
    }
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

pub(crate) fn redact(value: &str) -> String {
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
    #[cfg(unix)]
    #[test]
    fn bindings_open_accepts_parent_directory_alias_but_not_database_alias() {
        let root = tempfile::tempdir().expect("root");
        let directory = root.path().join("real");
        std::fs::create_dir(&directory).expect("directory");
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&directory, &alias).expect("directory alias");
        let path = alias.join("history-bindings.sqlite");
        drop(super::open_bindings(&path).expect("parent alias supported"));
        drop(super::open_binding_connection(&path).expect("reopen via alias"));
        let database_alias = alias.join("other.sqlite");
        std::os::unix::fs::symlink(&path, &database_alias).expect("database alias");
        assert!(super::open_bindings(&database_alias).is_err());
        assert!(super::open_binding_connection(&database_alias).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn bindings_open_refuses_replaced_symlink_without_mutating_target() {
        let root = tempfile::tempdir().expect("root");
        let target = root.path().join("target.sqlite");
        let connection = rusqlite::Connection::open(&target).expect("target");
        connection
            .execute_batch("CREATE TABLE sentinel(value TEXT);")
            .expect("schema");
        drop(connection);
        let original = std::fs::read(&target).expect("original");
        let path = root.path().join("history-bindings.sqlite");
        std::fs::write(&path, []).expect("regular binding path");
        assert!(
            std::fs::symlink_metadata(&path)
                .expect("metadata")
                .is_file()
        );
        // Deterministically replace the path after the caller's metadata check.
        std::fs::remove_file(&path).expect("replace test path");
        std::os::unix::fs::symlink(&target, &path).expect("alias");
        assert!(super::open_bindings(&path).is_err());
        assert!(super::open_binding_connection(&path).is_err());
        assert_eq!(std::fs::read(&target).expect("preserved"), original);
        assert!(!root.path().join("target.sqlite-wal").exists());
        assert!(!root.path().join("target.sqlite-shm").exists());
    }

    #[test]
    fn selected_history_binding_preserves_identity_bytes_and_reopens_without_copying() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().into());
        let session = manager.create_session("selected", "").expect("session");
        session
            .append(&talos_core::message::Message::User {
                content: "original constraint".into(),
            })
            .expect("append");
        let bytes = session.snapshot_bytes().expect("bytes");
        let first = super::DurableSession::bind_existing_history(session.clone()).expect("bind");
        assert_eq!(first.id(), session.id);
        assert_eq!(first.file_path(), session.file_path);
        assert_eq!(first.session().snapshot_bytes().expect("unchanged"), bytes);
        let source = first.capture_recovery_source().expect("source");
        let second = super::DurableSession::bind_existing_history(
            manager.get_session(&session.id).expect("reopen"),
        )
        .expect("rebind");
        assert!(
            !second.owns_recovery_source(&source),
            "reopen cannot inherit live authority"
        );
        assert_eq!(second.read_messages().expect("history").len(), 1);
        assert!(
            !root.path().join("durable").exists(),
            "no replacement embedded session"
        );
        let mut forged = session.clone();
        forged.id = uuid::Uuid::new_v4();
        assert!(super::DurableSession::bind_existing_history(forged).is_err());
        assert_eq!(session.snapshot_bytes().expect("preserved"), bytes);
    }

    #[cfg(unix)]
    #[test]
    fn selected_history_binding_refuses_symlink_transcript_and_database() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().into());
        let session = manager.create_session("selected", "").expect("session");
        let database = session
            .file_path
            .parent()
            .expect("parent")
            .join("history-bindings.sqlite");
        let victim = root.path().join("untouched");
        std::fs::write(&victim, b"preserve").expect("victim");
        std::os::unix::fs::symlink(&victim, &database).expect("alias");
        assert!(super::DurableSession::bind_existing_history(session.clone()).is_err());
        assert_eq!(std::fs::read(&victim).expect("unchanged"), b"preserve");
        std::fs::remove_file(&database).expect("remove test alias");
        std::fs::remove_file(&session.file_path).expect("remove test transcript");
        std::os::unix::fs::symlink(&victim, &session.file_path).expect("alias");
        assert!(super::DurableSession::bind_existing_history(session).is_err());
        assert_eq!(std::fs::read(&victim).expect("unchanged"), b"preserve");
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn fork_cancellation_before_and_after_publication_preserves_authority() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        for phase in ["publication-check", "published"] {
            let root = tempfile::tempdir().expect("root");
            let parent =
                super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
            parent
                .commit_turn(
                    "original",
                    &[talos_core::message::Message::User {
                        content: "original authoritative fact".into(),
                    }],
                    &Default::default(),
                )
                .expect("commit parent");
            let bytes = std::fs::read(parent.file_path()).expect("parent bytes");
            let cancelled = Arc::new(AtomicBool::new(false));
            super::FORK_CANCEL_PROBE
                .with(|probe| *probe.borrow_mut() = Some((phase, cancelled.clone())));
            let stop = cancelled.clone();
            let outcome = parent.fork_committed_tip(
                "child",
                std::time::Instant::now() + std::time::Duration::from_secs(5),
                move || stop.load(Ordering::SeqCst),
            );
            assert!(cancelled.load(Ordering::SeqCst), "probe must execute");
            let child = super::DurableSession::open_existing(root.path(), "child").expect("lookup");
            if phase == "publication-check" {
                assert!(outcome.is_err());
                assert!(
                    child.is_none(),
                    "cancel before final check rolls back publication"
                );
            } else {
                let returned = outcome.expect("committed publication wins late cancellation");
                let child = child.expect("published binding remains");
                assert_eq!(returned.id(), child.id());
                assert_eq!(
                    std::fs::read(child.file_path()).expect("child bytes"),
                    bytes
                );
            }
            assert_eq!(
                std::fs::read(parent.file_path()).expect("unchanged parent"),
                bytes
            );
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn fork_recovery_real_process_crash_boundaries() {
        if let Some(root) = std::env::var_os("TALOS_TEST_FORK_CRASH_ROOT") {
            let owner = super::DurableSession::open_existing(std::path::Path::new(&root), "parent")
                .expect("open")
                .expect("parent");
            owner
                .fork_committed_tip(
                    "child",
                    std::time::Instant::now() + std::time::Duration::from_secs(5),
                    || false,
                )
                .expect("crash probe must exit first");
            panic!("probe was not reached");
        }
        for phase in [
            "intent",
            "midcopy",
            "copied",
            "synced",
            "precommit",
            "published",
        ] {
            let root = tempfile::tempdir().expect("root");
            let owner =
                super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
            owner
                .commit_turn(
                    "seed",
                    &[talos_core::message::Message::User {
                        content: "original fact ".repeat(10_000),
                    }],
                    &Default::default(),
                )
                .expect("seed");
            let original = std::fs::read(owner.file_path()).expect("parent bytes");
            let mut child = std::process::Command::new(std::env::current_exe().expect("binary"))
                .args([
                    "--exact",
                    "durable::tests::fork_recovery_real_process_crash_boundaries",
                    "--nocapture",
                ])
                .env("TALOS_TEST_FORK_CRASH_ROOT", root.path())
                .env("TALOS_TEST_FORK_CRASH_PHASE", phase)
                .stdout(std::process::Stdio::null())
                .spawn()
                .expect("spawn");
            let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let status = loop {
                if let Some(status) = child.try_wait().expect("status") {
                    break status;
                }
                if std::time::Instant::now() >= until {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("crash child timeout");
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(42), "phase {phase}");
            let conn = rusqlite::Connection::open(root.path().join("durable-bindings.sqlite"))
                .expect("db");
            let id: String = conn
                .query_row("SELECT child_id FROM durable_fork_intents", [], |r| {
                    r.get(0)
                })
                .expect("persisted intent");
            if phase == "midcopy" {
                let length =
                    std::fs::metadata(root.path().join("durable").join(format!("{id}.tlog")))
                        .expect("partial metadata")
                        .len();
                assert!(length > 0 && length < original.len() as u64);
            }
            let report = owner
                .recover_fork_intents(
                    &crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2)),
                    None,
                )
                .expect("recover");
            assert_eq!((report.1, report.2), (1, 0), "phase {phase}");
            let reopened =
                super::DurableSession::open_existing(root.path(), "child").expect("child lookup");
            if phase == "published" {
                assert_eq!(
                    std::fs::read(reopened.expect("published child").file_path())
                        .expect("child bytes"),
                    original
                );
            } else {
                assert!(reopened.is_none());
                assert!(
                    !root
                        .path()
                        .join("durable")
                        .join(format!("{id}.tlog"))
                        .exists()
                );
            }
            assert_eq!(
                std::fs::read(owner.file_path()).expect("parent unchanged"),
                original
            );
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn fork_recovery_waits_for_independent_lock_owner_exit() {
        if let Some(root) = std::env::var_os("TALOS_FORK_LOCK_TEST_ROOT") {
            let directory = std::path::PathBuf::from(root);
            let id = std::env::var("TALOS_FORK_LOCK_TEST_ID").expect("id");
            let lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(directory.join("durable").join(format!("{id}.fork-lock")))
                .expect("lock");
            lock.try_lock().expect("hold");
            std::fs::write(directory.join("ready"), b"ready").expect("ready");
            loop {
                std::thread::park_timeout(std::time::Duration::from_secs(1));
            }
        }
        let root = tempfile::tempdir().expect("root");
        let owner = super::DurableSession::open_or_create(root.path(), "parent").expect("owner");
        let id = uuid::Uuid::new_v4();
        let directory = root.path().join("durable");
        std::fs::File::create(directory.join(format!("{id}.fork-lock"))).expect("lock");
        let partial = directory.join(format!("{id}.tlog"));
        std::fs::write(&partial, b"partial").expect("partial");
        let conn =
            rusqlite::Connection::open(root.path().join("durable-bindings.sqlite")).expect("db");
        conn.execute(
            "INSERT INTO durable_fork_intents VALUES(?1,?2,'child',1)",
            rusqlite::params![id.to_string(), owner.id().to_string()],
        )
        .expect("intent");
        struct Child(std::process::Child);
        impl Drop for Child {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let mut child = Child(
            std::process::Command::new(std::env::current_exe().expect("test binary"))
                .args([
                    "--exact",
                    "durable::tests::fork_recovery_waits_for_independent_lock_owner_exit",
                    "--nocapture",
                ])
                .env("TALOS_FORK_LOCK_TEST_ROOT", root.path())
                .env("TALOS_FORK_LOCK_TEST_ID", id.to_string())
                .stdout(std::process::Stdio::null())
                .spawn()
                .expect("child"),
        );
        let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !root.path().join("ready").exists() {
            assert!(std::time::Instant::now() < until, "child ready timeout");
            assert!(
                child.0.try_wait().expect("status").is_none(),
                "child exited early"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let budget = crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2));
        assert_eq!(
            owner
                .recover_fork_intents(&budget, None)
                .expect("active skipped")
                .1,
            0
        );
        assert!(partial.exists());
        child.0.kill().expect("terminate lock owner");
        child.0.wait().expect("reap owner");
        assert_eq!(
            owner
                .recover_fork_intents(&budget, None)
                .expect("recover after exit")
                .1,
            1
        );
        assert!(!partial.exists());
    }

    #[cfg(unix)]
    #[test]
    fn fork_recovery_preserves_symlink_targets_and_keeps_failed_intents() {
        let root = tempfile::tempdir().expect("root");
        let owner = super::DurableSession::open_or_create(root.path(), "parent").expect("owner");
        let directory = root.path().join("durable");
        let target = root.path().join("valuable");
        std::fs::write(&target, b"preserve").expect("target");
        let conn =
            rusqlite::Connection::open(root.path().join("durable-bindings.sqlite")).expect("db");
        for lock_symlink in [false, true] {
            let id = uuid::Uuid::new_v4();
            let lock_path = directory.join(format!("{id}.fork-lock"));
            let child_path = directory.join(format!("{id}.tlog"));
            if lock_symlink {
                std::os::unix::fs::symlink(&target, &lock_path).expect("lock symlink");
                std::fs::write(&child_path, b"partial").expect("partial");
            } else {
                std::fs::File::create(&lock_path).expect("lock");
                std::os::unix::fs::symlink(&target, &child_path).expect("child symlink");
            }
            conn.execute(
                "INSERT INTO durable_fork_intents VALUES(?1,?2,'child',1)",
                rusqlite::params![id.to_string(), owner.id().to_string()],
            )
            .expect("intent");
        }
        let report = owner
            .recover_fork_intents(
                &crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2)),
                None,
            )
            .expect("report");
        assert_eq!((report.0, report.1, report.2), (2, 0, 2));
        assert_eq!(
            std::fs::read(&target).expect("target retained"),
            b"preserve"
        );
        let count: i64 = conn
            .query_row("SELECT count(*) FROM durable_fork_intents", [], |r| {
                r.get(0)
            })
            .expect("count");
        assert_eq!(count, 2);
    }

    #[cfg(unix)]
    #[test]
    fn fork_recovery_pages_past_bad_records_and_preserves_published_child() {
        let root = tempfile::tempdir().expect("root");
        let owner = super::DurableSession::open_or_create(root.path(), "parent").expect("owner");
        let conn =
            rusqlite::Connection::open(root.path().join("durable-bindings.sqlite")).expect("db");
        for n in 1..=33 {
            conn.execute(
                "INSERT INTO durable_fork_intents VALUES(?1,?2,'unused',1)",
                rusqlite::params![uuid::Uuid::from_u128(n).to_string(), owner.id().to_string()],
            )
            .expect("intent");
        }
        let last = uuid::Uuid::from_u128(33);
        std::fs::File::create(
            root.path()
                .join("durable")
                .join(format!("{last}.fork-lock")),
        )
        .expect("lock");
        let budget = crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2));
        let first = owner
            .recover_fork_intents(&budget, None)
            .expect("first page");
        assert_eq!((first.0, first.1, first.2), (32, 0, 32));
        let second = owner
            .recover_fork_intents(&budget, first.3.as_deref())
            .expect("next page");
        assert_eq!((second.0, second.1, second.2), (1, 1, 0));
        let child = owner
            .fork_committed_tip(
                "published",
                std::time::Instant::now() + std::time::Duration::from_secs(2),
                || false,
            )
            .expect("fork");
        let raw = std::fs::read(child.file_path()).expect("raw");
        conn.execute(
            "INSERT INTO durable_fork_intents VALUES(?1,?2,'published',1)",
            rusqlite::params![child.id().to_string(), owner.id().to_string()],
        )
        .expect("simulate postcommit crash");
        let page = owner
            .recover_fork_intents(&budget, first.3.as_deref())
            .expect("published recovery");
        assert_eq!(page.1, 1);
        assert_eq!(std::fs::read(child.file_path()).expect("preserved"), raw);
        assert!(
            super::DurableSession::open_existing(root.path(), "published")
                .expect("open")
                .is_some()
        );
    }

    #[cfg(unix)]
    #[test]
    fn fork_recovery_skips_active_lock_and_retires_only_unpublished_intent() {
        let root = tempfile::tempdir().expect("root");
        let owner = super::DurableSession::open_or_create(root.path(), "parent").expect("owner");
        let id = uuid::Uuid::new_v4();
        let directory = root.path().join("durable");
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{id}.fork-lock")))
            .expect("lock");
        lock.try_lock().expect("hold lock");
        let conn =
            rusqlite::Connection::open(root.path().join("durable-bindings.sqlite")).expect("db");
        conn.execute(
            "INSERT INTO durable_fork_intents VALUES(?1,?2,'child',1)",
            rusqlite::params![id.to_string(), owner.id().to_string()],
        )
        .expect("intent");
        let partial = directory.join(format!("{id}.tlog"));
        std::fs::write(&partial, b"partial").expect("partial");
        let unrelated = directory.join(format!("{}.tlog", uuid::Uuid::new_v4()));
        std::fs::write(&unrelated, b"unrelated").expect("unrelated");
        let budget = crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2));
        assert_eq!(
            owner
                .recover_fork_intents(&budget, None)
                .expect("active skipped")
                .1,
            0
        );
        assert!(partial.exists());
        drop(lock);
        assert_eq!(
            owner
                .recover_fork_intents(&budget, None)
                .expect("recover")
                .1,
            1
        );
        assert!(!partial.exists());
        assert!(unrelated.exists());
        assert_eq!(
            owner
                .recover_fork_intents(&budget, None)
                .expect("idempotent")
                .1,
            0
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn durable_fork_allows_reused_call_id_across_turns_and_refuses_malformed_blocks() {
        use talos_core::message::{Message, MessageToolResult, ToolCall};
        let root = tempfile::tempdir().expect("root");
        let parent = super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
        for turn in ["one", "two"] {
            parent
                .commit_turn(
                    turn,
                    &[
                        Message::Assistant {
                            content: String::new(),
                            reasoning: None,
                            tool_calls: vec![ToolCall {
                                id: "reused".into(),
                                name: "read".into(),
                                input: serde_json::json!({"literal": "```"}),
                            }],
                        },
                        Message::Tool {
                            result: MessageToolResult {
                                tool_use_id: "reused".into(),
                                content: "fact".into(),
                                is_error: false,
                            },
                        },
                    ],
                    &Default::default(),
                )
                .expect("turn");
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        parent
            .fork_committed_tip("valid", deadline, || false)
            .expect("valid reused IDs and embedded fence");
        for (i, bad) in [
            "```json-tool invalid ```",
            "```json-tool {\"name\":\"read\",\"args\":{}} ```",
            "```json-tool {\"id\":\"x\",\"name\":\"read\",\"args\":{}}",
        ]
        .iter()
        .enumerate()
        {
            let owner = super::DurableSession::open_or_create(root.path(), &format!("bad-{i}"))
                .expect("owner");
            owner
                .commit_turn(
                    "turn",
                    &[Message::Assistant {
                        content: (*bad).into(),
                        reasoning: None,
                        tool_calls: vec![],
                    }],
                    &Default::default(),
                )
                .expect("store malformed fixture");
            assert!(
                owner
                    .fork_committed_tip(&format!("child-{i}"), deadline, || false)
                    .is_err()
            );
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn durable_fork_rejects_duplicate_entries_and_reopened_turns() {
        for duplicate_entry in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let parent =
                super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
            for turn in ["first", "second"] {
                parent
                    .commit_turn(
                        turn,
                        &[talos_core::message::Message::User {
                            content: "fact".into(),
                        }],
                        &Default::default(),
                    )
                    .expect("commit");
            }
            let mut entries = parent.session.read_entries().expect("entries");
            if duplicate_entry {
                entries[2].id = entries[0].id.clone();
                entries[3].parent_id = Some(entries[2].id.clone());
            } else {
                entries[2].metadata.turn_id = Some("first".into());
                entries[3].metadata.turn_id = Some("first".into());
                entries[3].content = crate::turn_outcome::encode_turn_transcript_outcome(
                    &crate::TurnTranscriptOutcomeRecord::new(
                        "first",
                        crate::TurnTranscriptOutcome::Success,
                    ),
                )
                .expect("terminal");
            }
            parent
                .session
                .store
                .replace_entries_atomically(parent.file_path(), &entries)
                .expect("alter fixture");
            assert!(
                parent
                    .fork_committed_tip(
                        "child",
                        std::time::Instant::now() + std::time::Duration::from_secs(5),
                        || false
                    )
                    .is_err()
            );
            assert!(
                super::DurableSession::open_existing(root.path(), "child")
                    .expect("lookup")
                    .is_none()
            );
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn durable_fork_refuses_terminal_with_unpaired_tool_call() {
        let root = tempfile::tempdir().expect("root");
        let parent = super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
        let call = talos_core::message::Message::Assistant {
            content: String::new(),
            reasoning: None,
            tool_calls: vec![talos_core::message::ToolCall {
                id: "call".into(),
                name: "read".into(),
                input: serde_json::json!({}),
            }],
        };
        parent
            .commit_turn("turn", &[call], &Default::default())
            .expect("persist");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        assert!(
            parent
                .fork_committed_tip("bad", deadline, || false)
                .is_err()
        );
        assert!(
            super::DurableSession::open_existing(root.path(), "bad")
                .expect("lookup")
                .is_none()
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn durable_fork_reopens_ancestor_without_parent_or_sibling_suffix() {
        let root = tempfile::tempdir().expect("root");
        let parent = super::DurableSession::open_or_create(root.path(), "parent").expect("parent");
        let message = |text: &str| talos_core::message::Message::User {
            content: text.into(),
        };
        parent
            .commit_turn("ancestor", &[message("shared fact")], &Default::default())
            .expect("seed");
        let bytes = std::fs::read(parent.file_path()).expect("raw");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let child = parent
            .fork_committed_tip("child", deadline, || false)
            .expect("fork");
        let sibling = parent
            .fork_committed_tip("sibling", deadline, || false)
            .expect("sibling");
        assert_ne!(parent.id(), child.id());
        assert_eq!(std::fs::read(child.file_path()).expect("child raw"), bytes);
        assert!(
            parent
                .fork_committed_tip("child", deadline, || false)
                .is_err()
        );
        assert!(
            parent
                .fork_committed_tip("cancelled", deadline, || true)
                .is_err()
        );
        assert!(
            super::DurableSession::open_existing(root.path(), "cancelled")
                .expect("lookup")
                .is_none()
        );
        parent
            .commit_turn(
                "parent-later",
                &[message("parent secret")],
                &Default::default(),
            )
            .expect("parent append");
        sibling
            .commit_turn(
                "sibling-later",
                &[message("sibling secret")],
                &Default::default(),
            )
            .expect("sibling append");
        child
            .commit_turn("child-later", &[message("child fact")], &Default::default())
            .expect("child append");
        drop(child);
        let reopened = super::DurableSession::open_existing(root.path(), "child")
            .expect("reopen")
            .expect("child");
        assert_eq!(
            reopened.read_messages().expect("messages"),
            vec![message("shared fact"), message("child fact")]
        );
        assert!(
            std::fs::read(parent.file_path())
                .expect("parent raw")
                .starts_with(&bytes)
        );
        let foreign_source = parent.capture_recovery_source().expect("parent source");
        assert!(!reopened.owns_recovery_source(&foreign_source));
    }

    #[cfg(unix)]
    #[test]
    fn busy_scan_cleanup_keeps_receipt_valid_for_retry() {
        let root = tempfile::tempdir().expect("root");
        let owner =
            super::DurableSession::open_or_create(root.path(), "cleanup-retry").expect("owner");
        let scan = owner
            .capture_recovery_source()
            .expect("source")
            .begin_prefix_seal()
            .expect("scan");
        let budget = crate::sqlite::RecoveryWorkBudget::new(std::time::Duration::from_secs(2));
        let receipt = owner.save_recovery_scan(&scan, &budget).expect("save");
        let blocker = rusqlite::Connection::open(root.path().join("durable-bindings.sqlite"))
            .expect("connection");
        blocker.execute_batch("BEGIN IMMEDIATE").expect("lock");
        assert!(owner.clear_recovery_scan(&receipt, &budget).is_err());
        blocker.execute_batch("ROLLBACK").expect("unlock");
        assert!(owner.clear_recovery_scan(&receipt, &budget).expect("retry"));
        assert!(
            !owner
                .clear_recovery_scan(&receipt, &budget)
                .expect("idempotent")
        );
    }

    #[cfg(unix)]
    #[test]
    fn recovery_storage_panic_rolls_back_and_binding_delete_cleans_scan() {
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "cleanup").expect("owner");
        let result: Result<(), SessionError> = recovery_storage_boundary(|| {
            let connection = open_bindings(&owner.bindings_path)?;
            let tx = connection.unchecked_transaction().map_err(sql_error)?;
            tx.execute(
                "INSERT INTO durable_recovery_scans VALUES(?1,'partial')",
                [owner.id().to_string()],
            )
            .map_err(sql_error)?;
            panic!("injected storage panic before commit");
        });
        assert!(result.is_err());
        let connection = open_bindings(&owner.bindings_path).expect("open");
        let count: i64 = connection
            .query_row("SELECT count(*) FROM durable_recovery_scans", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 0, "panic unwinding must roll back partial progress");
        connection
            .execute(
                "INSERT INTO durable_recovery_scans VALUES(?1,'completed')",
                [owner.id().to_string()],
            )
            .expect("seed");
        connection
            .execute(
                "DELETE FROM durable_bindings WHERE session_id=?1",
                [owner.id().to_string()],
            )
            .expect("delete binding");
        let count: i64 = connection
            .query_row("SELECT count(*) FROM durable_recovery_scans", [], |row| {
                row.get(0)
            })
            .expect("count after delete");
        assert_eq!(count, 0);
    }

    use super::*;

    #[cfg(unix)]
    #[test]
    fn history_scan_resumes_pages_without_publishing_partial_or_stale_history() {
        use crate::history_page::HistoryPageError;
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "scan").expect("owner");
        let messages: Vec<_> = (0..30)
            .map(|i| Message::User {
                content: format!("{i}:{}", "x".repeat(90_000)),
            })
            .collect();
        owner
            .commit_turn("initial", &messages, &PersistencePolicy::default())
            .expect("commit");
        let expected = owner.read_history_snapshot().expect("reference");
        let mut scan = owner.begin_history_scan().expect("begin");
        let deadline = || std::time::Instant::now() + std::time::Duration::from_secs(5);
        assert!(matches!(
            scan.step(deadline(), || true),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!(scan.offset, 0);
        assert!(scan.entries.is_empty());
        let checks = std::cell::Cell::new(0);
        assert!(
            !scan
                .step(deadline(), || {
                    checks.set(checks.get() + 1);
                    false
                })
                .expect("first page")
        );
        let stop_at = checks.get() - 2;
        let mut interrupted = owner.begin_history_scan().expect("interruptible scan");
        let observed = std::cell::Cell::new(0);
        assert!(matches!(
            interrupted.step(deadline(), || {
                observed.set(observed.get() + 1);
                observed.get() >= stop_at
            }),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!(interrupted.offset, 0);
        assert!(interrupted.entries.is_empty());
        while !interrupted
            .step(deadline(), || false)
            .expect("retry interrupted")
        {}
        assert_eq!(
            interrupted.finish().expect("retried snapshot").entries(),
            expected.entries()
        );
        let offset = scan.offset;
        let count = scan.entries.len();
        assert!(matches!(
            scan.step(std::time::Instant::now(), || false),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!((scan.offset, scan.entries.len()), (offset, count));
        while !scan.step(deadline(), || false).expect("resume") {}
        let snapshot = scan.finish().expect("complete");
        assert!(owner.owns_history_snapshot(&snapshot));
        assert_eq!(snapshot.entries(), expected.entries());
        assert!(matches!(
            owner.begin_history_scan().expect("partial").finish(),
            Err(HistoryPageError::Stopped)
        ));

        let mut stale = owner.begin_history_scan().expect("begin stale");
        while !stale.step(deadline(), || false).expect("read") {}
        owner
            .commit_turn(
                "append",
                &[Message::User {
                    content: "changed".into(),
                }],
                &PersistencePolicy::default(),
            )
            .expect("append");
        assert!(matches!(stale.finish(), Err(HistoryPageError::Stale)));
        let captured = owner
            .capture_recovery_source()
            .expect("capture before append");
        owner
            .commit_turn(
                "capture-race",
                &[Message::User {
                    content: "late turn".into(),
                }],
                &PersistencePolicy::default(),
            )
            .expect("concurrent append");
        assert!(DurableHistoryScan::new(captured).is_err());
    }

    #[test]
    fn configured_owner_refuses_copied_identity_and_allows_handle_clones() {
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "owner").expect("owner");
        owner
            .commit_turn(
                "turn",
                &[Message::User {
                    content: "same fact".into(),
                }],
                &PersistencePolicy::default(),
            )
            .expect("commit");
        let snapshot = owner.read_history_snapshot().expect("snapshot");
        let source = owner.capture_recovery_source().expect("source");
        assert!(owner.clone().owns_history_snapshot(&snapshot));
        assert!(owner.clone().owns_recovery_source(&source));
        let copied_path = root.path().join("copied.tlog");
        std::fs::copy(owner.file_path(), &copied_path).expect("copy actual source");
        let mut copied_session = owner.session.clone();
        copied_session.file_path = copied_path;
        let copied =
            DurableSession::new("copied".into(), copied_session, owner.bindings_path.clone());
        let copied_snapshot = copied.read_history_snapshot().expect("copy snapshot");
        assert_eq!(snapshot.session_id(), copied_snapshot.session_id());
        assert_eq!(snapshot.entries(), copied_snapshot.entries());
        assert!(!owner.owns_history_snapshot(&copied_snapshot));
        assert!(
            !owner.owns_recovery_source(&copied.capture_recovery_source().expect("copy source"))
        );
        assert!(!copied.owns_history_snapshot(&snapshot));
        assert!(!copied.owns_recovery_source(&source));
    }

    #[test]
    fn captured_recovery_source_excludes_append_and_is_owned_by_durable_session() {
        let directory = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(directory.path(), "owner").expect("session");
        session
            .commit_turn(
                "first",
                &[Message::User {
                    content: "original fact".into(),
                }],
                &PersistencePolicy::default(),
            )
            .expect("first commit");
        let source = session.capture_recovery_source().expect("source");
        assert_eq!(source.session_id(), session.id());
        assert_eq!(source.branch_id(), session.session().current_branch);
        session
            .commit_turn(
                "second",
                &[Message::User {
                    content: "later fact".into(),
                }],
                &PersistencePolicy::default(),
            )
            .expect("append");
        assert!(
            session
                .capture_recovery_source()
                .expect("new cutoff")
                .source_end()
                > source.source_end()
        );
        let page = source
            .read_page(
                0,
                std::time::Instant::now() + std::time::Duration::from_secs(2),
                || false,
            )
            .expect("page");
        assert!(page.complete);
        let range = page.record_ranges[0].clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let record = source
            .capture_record(range, deadline, || false)
            .expect("capture record");
        assert_eq!(
            record.reread(deadline, || false).expect("unchanged").id,
            page.entries[0].id
        );
        assert!(
            source
                .capture_record(
                    source.source_end()..source.source_end() + 1,
                    deadline,
                    || false
                )
                .is_err()
        );
        assert!(record.reread(deadline, || true).is_err());
        assert!(record.reread(std::time::Instant::now(), || false).is_err());
        let mut bytes = fs::read(session.file_path()).expect("bytes");
        let offset = bytes
            .windows(b"original fact".len())
            .position(|value| value == b"original fact")
            .expect("content bytes");
        bytes[offset..offset + b"original fact".len()].copy_from_slice(b"modified fact");
        fs::write(session.file_path(), bytes).expect("same length replacement");
        assert!(matches!(
            record.reread(deadline, || false),
            Err(crate::history_page::HistoryPageError::Stale)
        ));
        assert!(
            page.entries
                .iter()
                .any(|entry| entry.content == "original fact")
        );
        assert!(
            !page
                .entries
                .iter()
                .any(|entry| entry.content == "later fact")
        );
        assert!(
            source
                .read_page(
                    0,
                    std::time::Instant::now() + std::time::Duration::from_secs(2),
                    || true
                )
                .is_err()
        );
    }

    #[test]
    fn composed_recovery_enforces_combined_source_budget_before_io() {
        let source = DurableRecoverySource {
            descriptor: Arc::new(RecoverySourceDescriptor {
                write_lock: Arc::new(std::sync::Mutex::new(())),
                owner: Arc::new(()),
                session_id: Uuid::new_v4(),
                branch_id: "test".into(),
                path: std::path::PathBuf::from("nonexistent-recovery-budget-fixture"),
            }),
            source_end: 2 * crate::history_page::HISTORY_PAGE_BYTES as u64,
        };
        let record = DurableRecoveryRecord {
            source: source.clone(),
            range: 0..crate::history_page::HISTORY_PAGE_BYTES as u64,
            digest: [0; 32],
        };
        let terminal = DurableRecoveryRecord {
            source,
            range: crate::history_page::HISTORY_PAGE_BYTES as u64
                ..crate::history_page::HISTORY_PAGE_BYTES as u64 + 1,
            digest: [0; 32],
        };
        assert!(matches!(
            record.recover_text(
                Some(&terminal),
                crate::history_projection::RecoveryTextKind::Detail,
                std::time::Instant::now() + std::time::Duration::from_secs(2),
                || false
            ),
            Err(crate::history_page::HistoryPageError::RecordTooLarge)
        ));
    }

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
        let source = reopened.capture_recovery_source().expect("captured source");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let page = source.read_page(0, deadline, || false).expect("page");
        let tool_index = page
            .entries
            .iter()
            .position(|entry| entry.id == first.entry_ids[1])
            .expect("tool record");
        let tool = source
            .capture_record(page.record_ranges[tool_index].clone(), deadline, || false)
            .expect("tool locator");
        let terminal = source
            .capture_record(
                page.record_ranges.last().expect("terminal range").clone(),
                deadline,
                || false,
            )
            .expect("terminal locator");
        let kind = crate::history_projection::RecoveryTextKind::Detail;
        assert!(
            tool.recover_text(None, kind, deadline, || false)
                .expect("no receipt")
                .is_none()
        );
        assert_eq!(
            tool.recover_text(Some(&terminal), kind, deadline, || false)
                .expect("verified rereads")
                .expect("eligible")
                .text,
            "fact: port 8123"
        );
        let other_capture = reopened
            .capture_recovery_source()
            .expect("independent capture");
        let other_terminal = other_capture
            .capture_record(
                page.record_ranges.last().expect("range").clone(),
                deadline,
                || false,
            )
            .expect("other terminal");
        assert!(
            tool.recover_text(Some(&other_terminal), kind, deadline, || false)
                .is_err()
        );
        assert!(
            tool.recover_text(Some(&terminal), kind, deadline, || true)
                .is_err()
        );

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
        // Replace marker bytes on disk while leaving the original tool record intact.
        let mut bytes = fs::read(reopened.file_path()).expect("stored transcript");
        let start = terminal.range.start as usize;
        let end = terminal.range.end as usize;
        let offset = bytes[start..end]
            .windows(b"history_recovery".len())
            .position(|value| value == b"history_recovery")
            .expect("receipt extension");
        bytes[start + offset] = b'H';
        fs::write(reopened.file_path(), bytes).expect("same-length terminal replacement");
        assert!(
            tool.reread(deadline, || false).is_ok(),
            "tool source unchanged"
        );
        assert!(matches!(
            tool.recover_text(Some(&terminal), kind, deadline, || false),
            Err(crate::history_page::HistoryPageError::Stale)
        ));
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
    fn finalize_preserves_original_noncanonical_history_bytes() {
        let directory = tempfile::tempdir().expect("directory");
        let session = create_or_open(directory.path(), "raw-prefix").expect("session");
        session
            .session()
            .append(&Message::User {
                content: "legacy".into(),
            })
            .expect("legacy entry");
        let original = fs::read_to_string(session.file_path()).expect("original");
        let altered = format!("{}\n", original.replacen("\t2:{}\n", "\t3:{ }\n", 1));
        assert_ne!(original, altered);
        fs::write(session.file_path(), &altered).expect("legacy encoding");
        let messages = vec![Message::User {
            content: "next".into(),
        }];
        let policy = PersistencePolicy::default();
        session
            .finalize_turn("next", &messages, &policy, TurnTranscriptOutcome::Success)
            .expect("commit");
        let committed = fs::read(session.file_path()).expect("committed");
        assert!(
            committed.starts_with(altered.as_bytes()),
            "finalization changed inherited raw bytes"
        );
        let reopened = get_by_external_id(directory.path(), "raw-prefix")
            .expect("lookup")
            .expect("reopen");
        let replay = reopened
            .finalize_turn("next", &messages, &policy, TurnTranscriptOutcome::Success)
            .expect("replay");
        assert!(!replay.newly_committed);
        assert_eq!(fs::read(reopened.file_path()).expect("bytes"), committed);
        let mut damaged = committed;
        damaged.extend_from_slice(b"E\t0\tbroken");
        fs::write(reopened.file_path(), &damaged).expect("incomplete tail");
        assert!(
            reopened
                .finalize_turn("later", &messages, &policy, TurnTranscriptOutcome::Success)
                .is_err()
        );
        assert_eq!(
            fs::read(reopened.file_path()).expect("preserved damage"),
            damaged
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
    fn reasoning_persistence_roundtrips_and_retry_checks_filtered_metadata() {
        for persist_reasoning in [false, true] {
            let directory = tempfile::tempdir().expect("directory");
            let session = create_or_open(directory.path(), "reasoning").expect("session");
            let policy = PersistencePolicy {
                persist_reasoning,
                persist_raw_tool_output: true,
            };
            let mut message = Message::Assistant {
                content: "public answer".into(),
                tool_calls: vec![],
                reasoning: Some(AssistantReasoning {
                    provider: "fixture".into(),
                    model: "fixture".into(),
                    blocks: vec![
                        ReasoningBlock::Thinking {
                            text: "private thought\nAuthorization: Bearer reasoning-secret".into(),
                            signature: Some("opaque signature\napi_key=signature-secret".into()),
                        },
                        ReasoningBlock::Redacted {
                            data: "opaque data\nCookie: redacted-block-secret".into(),
                        },
                        ReasoningBlock::Plain {
                            text: "plain thought\nx-api-key: plain-block-secret".into(),
                        },
                    ],
                }),
            };
            session
                .commit_turn("turn", &[message.clone()], &policy)
                .expect("commit");
            let bytes = session.session().snapshot_bytes().expect("bytes");
            let entry = session.session().read_entries().expect("entries").remove(0);
            assert_eq!(entry.metadata.reasoning.is_some(), persist_reasoning);
            let serialized = serde_json::to_string(&entry).expect("serialized entry");
            for secret in [
                "reasoning-secret",
                "signature-secret",
                "redacted-block-secret",
                "plain-block-secret",
            ] {
                assert!(!serialized.contains(secret));
                assert!(!String::from_utf8_lossy(&bytes).contains(secret));
            }
            if persist_reasoning {
                for preserved in [
                    "private thought",
                    "opaque signature",
                    "opaque data",
                    "plain thought",
                ] {
                    assert!(serialized.contains(preserved));
                }
                assert!(serialized.contains("[REDACTED]"));
            }
            assert!(
                !session
                    .commit_turn("turn", &[message.clone()], &policy)
                    .expect("identical retry")
                    .newly_committed
            );
            if let Message::Assistant {
                reasoning: Some(reasoning),
                ..
            } = &mut message
            {
                reasoning.blocks = vec![ReasoningBlock::Plain {
                    text: "different thought".into(),
                }];
            }
            let retry = session.commit_turn("turn", &[message], &policy);
            if persist_reasoning {
                assert!(matches!(
                    retry,
                    Err(SessionError::DurableTurnConflict { .. })
                ));
            } else {
                assert!(
                    !retry
                        .expect("omitted metadata is not persisted identity")
                        .newly_committed
                );
            }
            assert_eq!(
                session.session().snapshot_bytes().expect("unchanged bytes"),
                bytes
            );
            drop(session);
            let reopened = create_or_open(directory.path(), "reasoning").expect("reopen");
            assert_eq!(
                reopened.session().read_entries().expect("reopened entries")[0]
                    .metadata
                    .reasoning,
                entry.metadata.reasoning
            );
        }
    }

    #[test]
    fn reasoning_retry_preserves_legacy_missing_metadata_after_reopen() {
        let directory = tempfile::tempdir().expect("directory");
        let session = create_or_open(directory.path(), "legacy-reasoning").expect("session");
        let mut message = Message::Assistant {
            content: "already committed answer".into(),
            tool_calls: vec![],
            reasoning: None,
        };
        session
            .commit_turn(
                "legacy-turn",
                &[message.clone()],
                &PersistencePolicy::default(),
            )
            .expect("legacy-shaped commit without reasoning metadata");
        let bytes = session.session().snapshot_bytes().expect("bytes");
        drop(session);
        let reopened = create_or_open(directory.path(), "legacy-reasoning").expect("reopen");
        assert!(
            reopened.session().read_entries().expect("entries")[0]
                .metadata
                .reasoning
                .is_none()
        );
        if let Message::Assistant { reasoning, .. } = &mut message {
            *reasoning = Some(AssistantReasoning {
                provider: "fixture".into(),
                model: "fixture".into(),
                blocks: vec![ReasoningBlock::Plain {
                    text: "new metadata".into(),
                }],
            });
        }
        let enabled = PersistencePolicy {
            persist_reasoning: true,
            persist_raw_tool_output: true,
        };
        assert!(matches!(
            reopened.commit_turn("legacy-turn", &[message.clone()], &enabled),
            Err(SessionError::DurableTurnConflict { .. })
        ));
        assert!(
            !reopened
                .commit_turn("legacy-turn", &[message], &PersistencePolicy::default())
                .expect("default-off retry retains the original identity")
                .newly_committed
        );
        assert_eq!(
            reopened
                .session()
                .snapshot_bytes()
                .expect("unchanged bytes"),
            bytes
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
    fn open_bindings_wal_schema_contention_does_not_inherit_driver_busy_timeout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("durable-bindings.sqlite");
        let holder = Connection::open(&db_path).expect("holder");
        holder
            .execute_batch("PRAGMA journal_mode=WAL; BEGIN IMMEDIATE; CREATE TABLE held(x);")
            .expect("hold WAL schema writer");
        let (sender, receiver) = std::sync::mpsc::channel();
        let queued_at = std::time::Instant::now();
        let worker = std::thread::spawn(move || {
            let started_at = std::time::Instant::now();
            let retries = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            let observed = retries.clone();
            BINDING_INIT_RETRY_OBSERVER.with(|observer| {
                *observer.borrow_mut() = Some(Box::new(move || {
                    observed.borrow_mut().push(started_at.elapsed());
                }));
            });
            let result = open_bindings(&db_path).map(|_| ());
            BINDING_INIT_RETRY_OBSERVER.with(|observer| {
                observer.borrow_mut().take();
            });
            sender
                .send((
                    result,
                    started_at.duration_since(queued_at),
                    started_at.elapsed(),
                    retries.borrow().clone(),
                ))
                .expect("send init result");
        });
        // Keep the real write lock past the explicit 500ms retry budget, but
        // release it before joining even if the driver incorrectly waits 5s.
        let bounded = receiver.recv_timeout(std::time::Duration::from_secs(2));
        holder.execute_batch("ROLLBACK").expect("release writer");
        let finished_before_release = bounded.is_ok();
        let (result, queue_delay, initialization_time, retry_times) =
            bounded.unwrap_or_else(|_| {
                receiver
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .expect("init completes after releasing writer")
            });
        worker.join().expect("worker");
        assert!(
            finished_before_release,
            "initialization exceeded its retry budget: queue={queue_delay:?}, init={initialization_time:?}, retries={retry_times:?}, succeeded={}",
            result.is_ok()
        );
        assert!(matches!(result, Err(SessionError::DurableTurn(_))));
        // The failed initializer must not damage the database or poison retry.
        open_bindings(&dir.path().join("durable-bindings.sqlite"))
            .expect("fresh initialization after lock release");
    }

    #[test]
    fn delayed_binding_initialization_does_not_restart_its_retry_budget() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("durable-bindings.sqlite");
        let holder = Connection::open(&path).expect("holder");
        holder
            .execute_batch("PRAGMA journal_mode=WAL; BEGIN IMMEDIATE; CREATE TABLE held(x);")
            .expect("hold real WAL schema writer");
        let attempts = std::rc::Rc::new(std::cell::Cell::new(0));
        let observed = attempts.clone();
        BINDING_INIT_RETRY_OBSERVER.with(|observer| {
            *observer.borrow_mut() = Some(Box::new(move || {
                observed.set(observed.get() + 1);
                if observed.get() == 1 {
                    // Simulate descheduling after the first actual SQLITE_BUSY.
                    // A retry count cannot account for this elapsed wall time.
                    std::thread::sleep(std::time::Duration::from_millis(600));
                }
            }));
        });
        let result = open_bindings(&path);
        BINDING_INIT_RETRY_OBSERVER.with(|observer| {
            observer.borrow_mut().take();
        });
        holder.execute_batch("ROLLBACK").expect("release writer");
        assert!(matches!(result, Err(SessionError::DurableTurn(_))));
        assert_eq!(
            attempts.get(),
            1,
            "expired initialization must not start another retry wait"
        );
        open_bindings(&path).expect("lock release permits fresh initialization");
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
