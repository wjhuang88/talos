//! Immutable fork publication intents and host-private inheritance receipts.
//!
//! A receipt never authenticates arbitrary same-UID edits to private custody.
//! Callers must capture proven source bytes under host transition ownership;
//! these storage primitives do not infer provenance from transcript overlap.

use std::{collections::BTreeSet, path::Path, time::Instant};

use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{SessionEntry, SessionError, fork_origins::Origins};

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECORDS: usize = 100_000;
const MAX_PAYLOAD: usize = 32 * 1024;

// Invalid UTF-8 deliberately prevents the old whole-file decoder from compacting
// a materialized child into a legacy, unowned archive. Not an old-append fence.
pub(crate) const ENVELOPE: &[u8] = b"\xffTALOS-FORK\t1\n";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    version: u8,
    parent: Uuid,
    parent_incarnation: Uuid,
    parent_branch: Uuid,
    child: Uuid,
    child_incarnation: Uuid,
    child_branch: Uuid,
    count: usize,
    prefix_end: usize,
    prefix_digest: String,
    records_digest: String,
    tail: String,
    origins: Origins,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runtime_identity: Option<crate::SessionRuntimeIdentity>,
}

fn invalid() -> SessionError {
    SessionError::ParseError("fork inheritance receipt unavailable".into())
}

fn boundary<T>(f: impl FnOnce() -> Result<T, SessionError>) -> Result<T, SessionError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| Err(invalid()))
}

fn check(deadline: Instant) -> Result<(), SessionError> {
    if Instant::now() >= deadline {
        Err(invalid())
    } else {
        Ok(())
    }
}

fn database(head: &Path, child: Uuid) -> Result<std::path::PathBuf, SessionError> {
    if head.file_name().and_then(|s| s.to_str()) != Some(&format!("{child}.tlog")) {
        return Err(invalid());
    }
    Ok(head
        .parent()
        .ok_or_else(invalid)?
        .join("history-bindings.sqlite"))
}

impl crate::Session {
    /// Preserve an interrupted, pending fork outside normal transcript discovery.
    ///
    /// The caller must stop and join its publisher, retain this child's runtime
    /// lease, and coordinate index/lineage metadata separately. Published receipts
    /// are refused, including uncertain commits that actually succeeded. Pending
    /// identities are permanently retired; existing evidence is never overwritten.
    pub fn quarantine_pending_history_fork(
        &self,
        lease: &crate::SessionRuntimeLease,
        deadline: Instant,
    ) -> Result<std::path::PathBuf, SessionError> {
        lease.check_session(self)?;
        quarantine_pending(&self.file_path, deadline)
    }

    /// Restore missing initial runtime identity from a published fork receipt.
    ///
    /// Requires exclusive child runtime ownership. Version-one receipts lack this
    /// snapshot and are refused; a version-two absent identity is a no-op. Existing
    /// later activations are retained, while conflicting generation-zero metadata
    /// is refused. This does not restore index or lineage metadata.
    ///
    /// This operation is not atomic across transcript validation and sidecar
    /// initialization. An error may follow a committed metadata write; preserve
    /// artifacts and retry idempotently. SQLite lock acquisition is nonblocking
    /// and its VM work shares the caller deadline; filesystem I/O is cooperative.
    pub fn restore_history_fork_runtime(
        &self,
        lease: &crate::SessionRuntimeLease,
        deadline: Instant,
    ) -> Result<bool, SessionError> {
        boundary(|| {
            lease.check_session(self)?;
            let evidence = self.inspect_archive_recovery(deadline, || false)?;
            let receipt = evidence.selected_fork_receipt().ok_or_else(invalid)?;
            if receipt.version != 2 {
                return Err(invalid());
            }
            let Some(identity) = &receipt.runtime_identity else {
                return Ok(false);
            };
            let store = crate::PendingSubmissionStore::for_session(self);
            evidence.check_current(deadline, &|| false)?;
            let (_, restored) = store
                .fork_runtime_with_budget(deadline, Some(identity))
                .map_err(|_| invalid())?;
            evidence.check_current(deadline, &|| false)?;
            Ok(restored)
        })
    }

    /// Materialize a new receipt-bound child from validated original history.
    ///
    /// The source execution lease must be held for this entire operation. The
    /// child must be a fresh, absent canonical UUID.tlog in the same directory.
    /// Its current branch is bound into the receipt and must not be replaced by
    /// a later synthetic fork branch. This publishes history only: the host must
    /// finish runtime identity, index and lineage metadata before activating it.
    /// Errors retain pending files/intent for reconciliation; never blindly delete
    /// artifacts after an uncertain commit. No background continuation is started.
    pub fn publish_history_fork(
        &self,
        child: &crate::Session,
        source_lease: &crate::SessionRuntimeLease,
        through_entry: Option<&str>,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<crate::SessionEntry, SessionError> {
        boundary(|| {
            check(deadline)?;
            if stopped() {
                return Err(invalid());
            }
            source_lease.check_session(self)?;
            let database = database(&child.file_path, child.id)?;
            if self.id == child.id
                || child.file_extension() != "tlog"
                || !matches!(
                    child.store.history_encoding(),
                    Some(crate::history_page::HistoryEncoding::Tlog)
                )
                || self.file_path.parent() != child.file_path.parent()
                || std::fs::symlink_metadata(&child.file_path).is_ok()
            {
                return Err(invalid());
            }
            let branch = Uuid::parse_str(&child.current_branch).map_err(|_| invalid())?;
            let incarnation =
                crate::identity_custody::reserve_session_identity(&database, child.id)?;
            let capture = match ForkCapture::from_session(
                self,
                (child.id, incarnation, branch),
                through_entry,
                deadline,
                &stopped,
            ) {
                Ok(capture) => capture,
                Err(error) => {
                    // Capture never creates child intent/artifacts. Retire only
                    // this unused reservation; publication failures differ.
                    crate::identity_custody::retire_session_identity(&database, child.id)?;
                    return Err(error);
                }
            };
            let tail = capture
                .evidence
                .entries()
                .iter()
                .find(|entry| entry.id == capture.receipt.tail)
                .cloned()
                .ok_or_else(invalid)?;
            capture.publish(&child.file_path, deadline, &stopped)?;
            Ok(tail)
        })
    }
}

// A host capture owns its parent identity lease until publication or cancellation.
// This is preparation only: it does not create a child file or receipt intent.
pub(crate) struct ForkCapture {
    receipt: Receipt,
    bytes: Vec<u8>,
    evidence: crate::ArchiveRecoveryEvidence,
    _parent_identity: crate::identity_custody::IdentityLease,
}

pub(crate) fn reconcile_published_metadata(
    session: &crate::Session,
    index_path: &Path,
    deadline: Instant,
) -> Result<bool, SessionError> {
    boundary(|| {
        check(deadline)?;
        let Ok(lease) = crate::SessionRuntimeLease::acquire(&session.file_path) else {
            return Ok(false);
        };
        if published_branch(session, deadline)?.is_none() {
            return Ok(false);
        }
        let evidence = session.inspect_archive_recovery(deadline, || false)?;
        let receipt = evidence.selected_fork_receipt().ok_or_else(invalid)?;
        if receipt.version != 2 {
            return Ok(false);
        }
        session.restore_history_fork_runtime(&lease, deadline)?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(Instant::now()),
        );
        let mut index =
            crate::SessionIndex::open_for_recovery(index_path, &budget).map_err(|_| invalid())?;
        evidence.check_current(deadline, &|| false)?;
        index
            .record_fork_with_budget(
                &receipt.parent.to_string(),
                &receipt.child.to_string(),
                &receipt.tail,
                &budget,
            )
            .map_err(|_| invalid())?;
        evidence.check_current(deadline, &|| false)?;
        Ok(true)
    })
}

// Discovery never infers ownership from file contents or timestamps. Source and
// child runtime fences exclude a cooperating publisher before any retirement.
pub(crate) fn reconcile_pending_in_directory(
    directory: &Path,
    deadline: Instant,
) -> Result<usize, SessionError> {
    boundary(|| {
        check(deadline)?;
        let db = directory.join("history-bindings.sqlite");
        match std::fs::symlink_metadata(&db) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Ok(metadata) if metadata.file_type().is_file() => {}
            _ => return Err(invalid()),
        }
        let mut connection = crate::durable::open_binding_connection_with_flags(
            &db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(Instant::now()),
        );
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='archived_fork_receipts')", [], |row| row.get(0)).map_err(|_| invalid())?;
        if !exists {
            return Ok(0);
        }
        let pending = |child: Uuid| -> Result<Option<Receipt>, SessionError> {
            check(deadline)?;
            let mut statement = connection.prepare("SELECT CASE WHEN length(CAST(payload AS BLOB))<=32768 THEN payload END,published FROM archived_fork_receipts WHERE child=?1 LIMIT 2").map_err(|_| invalid())?;
            let mut rows = statement
                .query([child.to_string()])
                .map_err(|_| invalid())?;
            let Some(row) = rows.next().map_err(|_| invalid())? else {
                return Ok(None);
            };
            let published: i64 = row.get(1).map_err(|_| invalid())?;
            let payload: Option<String> = row.get(0).map_err(|_| invalid())?;
            if rows.next().map_err(|_| invalid())?.is_some() {
                return Err(invalid());
            }
            if published == 1 {
                return Ok(None);
            }
            if published != 0 {
                return Err(invalid());
            }
            let receipt: Receipt =
                serde_json::from_str(&payload.ok_or_else(invalid)?).map_err(|_| invalid())?;
            receipt.validate()?;
            if receipt.child != child {
                return Err(invalid());
            }
            Ok(Some(receipt))
        };
        // Same physical inventory ceiling as archive recovery. Finish inventory
        // before mutating so an oversized directory is not partially interpreted.
        let mut heads = Vec::new();
        for (count, entry) in std::fs::read_dir(directory)?.enumerate() {
            check(deadline)?;
            if count >= 256 {
                return Err(invalid());
            }
            let path = entry?.path();
            let Some(child) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            else {
                continue;
            };
            if path.file_name().and_then(|s| s.to_str()) == Some(&format!("{child}.tlog")) {
                heads.push((child, path));
            }
        }
        let mut preserved = 0;
        for (child, path) in heads {
            let Some(receipt) = pending(child)? else {
                continue;
            };
            let source = directory.join(format!("{}.tlog", receipt.parent));
            let Ok(_source_lease) = crate::SessionRuntimeLease::acquire(&source) else {
                continue;
            };
            let Ok(_child_lease) = crate::SessionRuntimeLease::acquire(&path) else {
                continue;
            };
            if pending(child)?.as_ref() != Some(&receipt) {
                return Err(invalid());
            }
            quarantine_pending(&path, deadline)?;
            preserved += 1;
        }
        Ok(preserved)
    })
}

// Preserve failed publication bytes outside the recognized transcript namespace.
// Idempotent after retirement or hard-link interruption; never touches a committed
// child. The caller must first stop/join its publisher and own the host fence.
pub(crate) fn quarantine_pending(
    head: &Path,
    deadline: Instant,
) -> Result<std::path::PathBuf, SessionError> {
    boundary(|| {
        check(deadline)?;
        let lease = crate::identity_custody::retire_pending_fork(head, deadline)?;
        let quarantine = head.with_extension("fork-unpublished");
        let source = match crate::history_page::open_source_under_identity(head, &lease) {
            Ok(file) => file,
            Err(crate::history_page::HistoryPageError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound =>
            {
                return Ok(quarantine);
            }
            Err(_) => return Err(invalid()),
        };
        drop(source);
        match std::fs::hard_link(head, &quarantine) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                use std::io::Read;
                let read = |path: &Path| -> Result<Vec<u8>, SessionError> {
                    if !std::fs::symlink_metadata(path)?.file_type().is_file() {
                        return Err(invalid());
                    }
                    let mut file = crate::history_page::open_source_under_identity(path, &lease)
                        .map_err(|_| invalid())?;
                    let mut bytes = Vec::new();
                    let mut chunk = [0u8; 64 * 1024];
                    loop {
                        check(deadline)?;
                        let read = file.read(&mut chunk)?;
                        if read == 0 {
                            break;
                        }
                        if bytes.len().checked_add(read).is_none_or(|n| n > MAX_BYTES) {
                            return Err(invalid());
                        }
                        bytes.extend_from_slice(&chunk[..read]);
                    }
                    Ok(bytes)
                };
                if read(head)? != read(&quarantine)? {
                    return Err(invalid());
                }
            }
            Err(error) => return Err(error.into()),
        }
        #[cfg(unix)]
        std::fs::File::open(head.parent().ok_or_else(invalid)?)?.sync_all()?;
        check(deadline)?;
        std::fs::remove_file(head)?;
        #[cfg(unix)]
        std::fs::File::open(head.parent().ok_or_else(invalid)?)?.sync_all()?;
        Ok(quarantine)
    })
}

impl ForkCapture {
    pub(crate) fn from_session(
        source: &crate::Session,
        child: (Uuid, Uuid, Uuid),
        through_entry: Option<&str>,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<Self, SessionError> {
        boundary(|| {
            check(deadline)?;
            if stopped()
                || child.0 == source.id
                || !matches!(
                    source.store.history_encoding(),
                    Some(crate::history_page::HistoryEncoding::Tlog)
                )
            {
                return Err(invalid());
            }
            let (incarnation, lease) =
                crate::identity_custody::adopt_fork_source(&source.file_path, deadline)?;
            let evidence = source.inspect_archive_recovery(deadline, &stopped)?;
            let tail = through_entry
                .or_else(|| evidence.entries().last().map(|e| e.id.as_str()))
                .ok_or_else(invalid)?;
            let records = evidence.original_record_prefix(tail, deadline, &stopped)?;
            let inherited = evidence.selected_fork_receipt();
            let origins = Origins::inherit(
                source.id,
                inherited.map(|r| &r.origins),
                evidence.entries().len(),
                records.len(),
            )?;
            // Parent branch is the host-selected branch; original record parent
            // links remain unchanged even for a non-linear legacy prefix.
            let branch = Uuid::parse_str(&source.current_branch).map_err(|_| invalid())?;
            let header = format!("TALOS\t1\t{}\n", source.created_at.timestamp_millis());
            let (mut receipt, bytes) = Receipt::capture(
                (source.id, incarnation, branch),
                child,
                header.as_bytes(),
                &records,
                origins,
                deadline,
            )?;
            // Version two records the publication-time identity, including an
            // explicit absence. Never infer it later from a mutable/deleted parent.
            let runtime = crate::PendingSubmissionStore::for_session(source);
            receipt.version = 2;
            receipt.runtime_identity = if runtime.path().exists() {
                match runtime
                    .fork_runtime_with_budget(deadline, None)
                    .map_err(|_| invalid())?
                    .0
                {
                    Some(state)
                        if state.status == crate::SessionRuntimeActivationStatus::Committed
                            && state.activation.is_valid() =>
                    {
                        Some(state.activation.target)
                    }
                    Some(_) => return Err(invalid()),
                    None => None,
                }
            } else {
                None
            };
            receipt.validate()?;
            evidence.check_current(deadline, &stopped)?;
            Ok(Self {
                receipt,
                bytes,
                evidence,
                _parent_identity: lease,
            })
        })
    }

    // Publish under the caller's source/runtime transition fence. Failure keeps
    // intent/artifacts for explicit reconciliation; never erase uncertain commit.
    pub(crate) fn publish(
        &self,
        destination: &Path,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        boundary(|| {
            use std::io::Write;
            check(deadline)?;
            if stopped() {
                return Err(invalid());
            }
            database(destination, self.receipt.child)?;
            self.check_source(deadline, &stopped)?;
            self.evidence
                .check_fork_addition(self.bytes.len(), self.receipt.count)?;
            // Bind the allowed new sibling before creating any intent or file.
            self.evidence
                .check_with_unpublished_child(destination, deadline, &stopped)?;
            if std::fs::symlink_metadata(destination).is_ok() {
                return Err(invalid());
            }
            self.receipt.record_intent(destination, deadline)?;
            let temporary = crate::store::temporary_sibling(destination);
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            for chunk in self.bytes.chunks(64 * 1024) {
                check(deadline)?;
                if stopped() {
                    return Err(invalid());
                }
                file.write_all(chunk)?;
            }
            file.sync_all()?;
            drop(file);
            self.check_source(deadline, &stopped)?;
            std::fs::hard_link(&temporary, destination)?;
            #[cfg(test)]
            crate::durable::fork_crash_probe("receipt_child_linked");
            #[cfg(unix)]
            std::fs::File::open(destination.parent().ok_or_else(invalid)?)?.sync_all()?;
            std::fs::remove_file(&temporary)?;
            self.evidence
                .check_with_unpublished_child(destination, deadline, &stopped)?;
            if stopped() {
                return Err(invalid());
            }
            self.receipt.publish(destination, deadline)?;
            #[cfg(test)]
            crate::durable::fork_crash_probe("receipt_committed_before_metadata");
            Ok(())
        })
    }

    // Call before any child artifact becomes visible. Once publication adds a
    // sibling, inventory equality intentionally changes; the host must retain
    // its content/transition fence through the remaining publication steps.
    pub(crate) fn check_source(
        &self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        self.evidence.check_current(deadline, &stopped)
    }
}

impl Receipt {
    /// Validate the current head, resolving a compaction marker through owned
    /// archives. The returned logical originals are used to validate additions;
    /// callers still preserve the physical head bytes when writing.
    pub(crate) fn current_records(
        &self,
        head: &Path,
        bytes: &[u8],
        deadline: Instant,
    ) -> Result<Vec<Vec<u8>>, SessionError> {
        check(deadline)?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid());
        }
        let offset = envelope_header_length(bytes)?;
        let (first, _) = crate::history_page::decode_tlog_record(&bytes[offset..], MAX_BYTES)
            .map_err(|_| invalid())?
            .ok_or_else(invalid)?;
        if !crate::archive_recovery::placeholder(&first) {
            return self.validate_materialized(bytes, deadline);
        }
        let session = crate::Session::new(self.child, String::new(), String::new(), head.into());
        let evidence = session.inspect_archive_recovery(deadline, || false)?;
        let tail = evidence.entries().last().ok_or_else(invalid)?;
        let records = evidence.original_record_prefix(&tail.id, deadline, || false)?;
        self.validate_records(&records, deadline)?;
        Ok(records)
    }

    /// Validate an uncompacted materialized head, including its exact physical
    /// prefix and subsequent linear suffix. Compacted heads must instead prove
    /// the original stream using owned archives before `validate_records`.
    pub(crate) fn validate_materialized(
        &self,
        bytes: &[u8],
        deadline: Instant,
    ) -> Result<Vec<Vec<u8>>, SessionError> {
        self.validate()?;
        check(deadline)?;
        if bytes.len() > MAX_BYTES
            || bytes.len() < self.prefix_end
            || crate::compaction_locator::digest(&bytes[..self.prefix_end]) != self.prefix_digest
        {
            return Err(invalid());
        }
        let mut offset = envelope_header_length(bytes)?;
        let mut records = Vec::new();
        let mut inherited_end = None;
        while offset < bytes.len() {
            check(deadline)?;
            if records.len() >= MAX_RECORDS {
                return Err(invalid());
            }
            let (_, length) = crate::history_page::decode_tlog_record(&bytes[offset..], MAX_BYTES)
                .map_err(|_| invalid())?
                .ok_or_else(invalid)?;
            records.push(bytes[offset..offset + length].to_vec());
            offset += length;
            if records.len() == self.count {
                inherited_end = Some(offset);
            }
        }
        if inherited_end != Some(self.prefix_end) {
            return Err(invalid());
        }
        self.validate_records(&records, deadline)?;
        Ok(records)
    }

    /// Build commitments only from a proven ordered raw prefix. Identity tuples
    /// are (session UUID, incarnation, branch UUID), supplied by host custody.
    /// This does not persist or publish authority.
    pub(crate) fn capture(
        parent: (Uuid, Uuid, Uuid),
        child: (Uuid, Uuid, Uuid),
        header: &[u8],
        records: &[Vec<u8>],
        origins: Origins,
        deadline: Instant,
    ) -> Result<(Self, Vec<u8>), SessionError> {
        check(deadline)?;
        if records.is_empty() || records.len() > MAX_RECORDS {
            return Err(invalid());
        }
        // Exactly one canonical TLOG header follows the binary gate. Record
        // bytes themselves are never normalized or re-encoded.
        let text = std::str::from_utf8(header).map_err(|_| invalid())?;
        let timestamp = text
            .strip_prefix("TALOS\t1\t")
            .and_then(|s| s.strip_suffix('\n'))
            .ok_or_else(invalid)?;
        timestamp.parse::<i64>().map_err(|_| invalid())?;
        if timestamp.contains('\n') {
            return Err(invalid());
        }
        let length = records
            .iter()
            .try_fold(ENVELOPE.len() + header.len(), |sum, raw| {
                sum.checked_add(raw.len())
            })
            .ok_or_else(invalid)?;
        if length > MAX_BYTES {
            return Err(invalid());
        }
        let mut bytes = Vec::with_capacity(length);
        bytes.extend_from_slice(ENVELOPE);
        bytes.extend_from_slice(header);
        let mut hash = Sha256::new();
        let mut tail = String::new();
        let mut ids = BTreeSet::new();
        for raw in records {
            check(deadline)?;
            let entry = decode(raw)?;
            if !ids.insert(entry.id.clone()) {
                return Err(invalid());
            }
            tail = entry.id;
            hash.update(raw);
            bytes.extend_from_slice(raw);
        }
        let receipt = Self {
            version: 1,
            parent: parent.0,
            parent_incarnation: parent.1,
            parent_branch: parent.2,
            child: child.0,
            child_incarnation: child.1,
            child_branch: child.2,
            count: records.len(),
            prefix_end: bytes.len(),
            prefix_digest: crate::compaction_locator::digest(&bytes),
            records_digest: hex::encode(hash.finalize()),
            tail,
            origins,
            runtime_identity: None,
        };
        receipt.validate()?;
        Ok((receipt, bytes))
    }

    fn validate(&self) -> Result<(), SessionError> {
        if !matches!(self.version, 1 | 2)
            || (self.version == 1 && self.runtime_identity.is_some())
            || self.parent == self.child
            || self.parent_incarnation == self.child_incarnation
            || [
                self.parent,
                self.parent_incarnation,
                self.parent_branch,
                self.child,
                self.child_incarnation,
                self.child_branch,
            ]
            .iter()
            .any(Uuid::is_nil)
            || self.prefix_end <= ENVELOPE.len()
            || self.prefix_end > MAX_BYTES
            || self.tail.is_empty()
            || self.tail.len() > 512
        {
            return Err(invalid());
        }
        for hash in [&self.prefix_digest, &self.records_digest] {
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                return Err(invalid());
            }
        }
        self.origins.validate(self.count)
    }

    /// Immutable intent before the caller creates staging or final files. The
    /// identity reservation is permanent even if this insertion fails.
    pub(crate) fn record_intent(&self, head: &Path, deadline: Instant) -> Result<(), SessionError> {
        boundary(|| {
            check(deadline)?;
            self.validate()?;
            let db = database(head, self.child)?;
            #[cfg(test)]
            crate::identity_custody::custody_stage("intent-child");
            let _lease = crate::identity_custody::lock_reserved_identity(
                head,
                self.child_incarnation,
                deadline,
            )?;
            #[cfg(test)]
            crate::identity_custody::custody_stage("intent-parent");
            crate::identity_custody::require_active_session_identity_until(
                &db,
                self.parent,
                self.parent_incarnation,
                deadline,
            )?;
            #[cfg(test)]
            crate::identity_custody::custody_stage("intent-identities-verified");
            let payload = serde_json::to_string(self).map_err(|_| invalid())?;
            if payload.len() > MAX_PAYLOAD {
                return Err(invalid());
            }
            // Identity reservation already initialized this database in WAL
            // mode. Do not restart its independent initialization retry budget.
            let mut connection = crate::durable::open_binding_connection_with_flags(
                &db,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )
            .map_err(|_| invalid())?;
            connection
                .busy_timeout(std::time::Duration::ZERO)
                .map_err(|_| invalid())?;
            let budget = crate::sqlite::RecoveryWorkBudget::new(
                deadline.saturating_duration_since(Instant::now()),
            );
            let connection =
                crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
            connection.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS archived_fork_receipts (child TEXT PRIMARY KEY, payload TEXT NOT NULL, published INTEGER NOT NULL CHECK(published IN (0,1)))").map_err(|_| invalid())?;
            connection
                .execute(
                    "INSERT INTO archived_fork_receipts(child,payload,published) VALUES(?1,?2,0)",
                    params![self.child.to_string(), payload],
                )
                .map_err(|_| invalid())?;
            check(deadline)
        })
    }

    /// Commit only after the exact final head has been synced. The caller owns
    /// publication, holds its source/child transition fences, and retains files
    /// on an uncertain commit. This function never overwrites transcript bytes.
    pub(crate) fn publish(&self, head: &Path, deadline: Instant) -> Result<(), SessionError> {
        boundary(|| {
            use std::io::Read;
            check(deadline)?;
            self.validate()?;
            let db = database(head, self.child)?;
            let lease = crate::identity_custody::lock_reserved_identity(
                head,
                self.child_incarnation,
                deadline,
            )?;
            let mut file = crate::history_page::open_source_under_identity(head, &lease)
                .map_err(|_| invalid())?;
            let before = crate::recovery_stamp::from_file(&file).map_err(|_| invalid())?;
            let mut bytes = Vec::new();
            (&mut file)
                .take(self.prefix_end as u64 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() != self.prefix_end
                || !bytes.starts_with(ENVELOPE)
                || crate::compaction_locator::digest(&bytes) != self.prefix_digest
            {
                return Err(invalid());
            }
            // File synchronization is done through the writer before this read
            // lease. Recheck the pathname before granting the private receipt.
            let rebound = crate::history_page::open_source_under_identity(head, &lease)
                .map_err(|_| invalid())?;
            let after = crate::recovery_stamp::from_file(&rebound).map_err(|_| invalid())?;
            if !crate::recovery_seal::same_file_version(&before, &after).map_err(|_| invalid())? {
                return Err(invalid());
            }
            check(deadline)?;
            let mut connection = crate::durable::open_binding_connection_with_flags(
                db,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )
            .map_err(|_| invalid())?;
            connection
                .busy_timeout(std::time::Duration::ZERO)
                .map_err(|_| invalid())?;
            let budget = crate::sqlite::RecoveryWorkBudget::new(
                deadline.saturating_duration_since(Instant::now()),
            );
            let connection =
                crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
            connection
                .execute_batch("PRAGMA synchronous=FULL")
                .map_err(|_| invalid())?;
            let payload = serde_json::to_string(self).map_err(|_| invalid())?;
            if connection.execute("UPDATE archived_fork_receipts SET published=1 WHERE child=?1 AND payload=?2 AND published=0", params![self.child.to_string(),payload]).map_err(|_| invalid())? != 1 { return Err(invalid()); }
            check(deadline)
        })
    }

    /// Verify the logical original stream, also after supported compaction has
    /// moved inherited records into owned archives. No live ancestor is read.
    /// Suffix records must be unique and strictly linear from inherited tail.
    pub(crate) fn validate_records(
        &self,
        records: &[Vec<u8>],
        deadline: Instant,
    ) -> Result<(), SessionError> {
        self.validate()?;
        if records.len() < self.count || records.len() > MAX_RECORDS {
            return Err(invalid());
        }
        let mut hash = Sha256::new();
        let mut ids = BTreeSet::new();
        let mut previous = None;
        let mut bytes = 0usize;
        for (index, raw) in records.iter().enumerate() {
            check(deadline)?;
            bytes = bytes
                .checked_add(raw.len())
                .filter(|n| *n <= MAX_BYTES)
                .ok_or_else(invalid)?;
            let entry = decode(raw)?;
            if !ids.insert(entry.id.clone()) || (index >= self.count && entry.parent_id != previous)
            {
                return Err(invalid());
            }
            if index < self.count {
                hash.update(raw);
            }
            if index + 1 == self.count && entry.id != self.tail {
                return Err(invalid());
            }
            previous = Some(entry.id);
        }
        if hex::encode(hash.finalize()) != self.records_digest {
            return Err(invalid());
        }
        check(deadline)
    }

    /// This relation must only be used after validating both actual streams.
    /// Raw byte equality and unique IDs remain required at the intersection site.
    pub(crate) fn origin(&self, index: usize) -> Uuid {
        self.origins.at(index).unwrap_or(self.child)
    }

    pub(crate) fn inherited(&self, index: usize) -> bool {
        index < self.count
    }
}

/// The fixed binary marker alone grants no authority. Callers must validate a
/// private receipt and preserve these bytes through every supported rewrite.
pub(crate) fn envelope_header_length(bytes: &[u8]) -> Result<usize, SessionError> {
    let body = bytes.strip_prefix(ENVELOPE).ok_or_else(invalid)?;
    let end = body
        .iter()
        .take(128)
        .position(|b| *b == b'\n')
        .ok_or_else(invalid)?;
    let text = std::str::from_utf8(&body[..end]).map_err(|_| invalid())?;
    let mut fields = text.split('\t');
    if fields.next() != Some("TALOS")
        || fields.next() != Some("1")
        || fields
            .next()
            .and_then(|value| value.parse::<i64>().ok())
            .is_none()
        || fields.next().is_some()
    {
        return Err(invalid());
    }
    Ok(ENVELOPE.len() + end + 1)
}

fn decode(raw: &[u8]) -> Result<SessionEntry, SessionError> {
    let (entry, length) = crate::history_page::decode_tlog_record(raw, MAX_BYTES)
        .map_err(|_| invalid())?
        .ok_or_else(invalid)?;
    if length != raw.len()
        || entry.id.is_empty()
        || entry.id.len() > 512
        || crate::archive_recovery::placeholder(&entry)
    {
        return Err(invalid());
    }
    Ok(entry)
}

/// Missing tables/rows are legacy; an existing pending, duplicate, or malformed
/// publication state never is. This is not payload validation: envelope readers
/// must additionally load the private receipt and validate the actual records.
pub(crate) fn check_publication(
    connection: &Connection,
    child: Uuid,
) -> Result<bool, SessionError> {
    let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='archived_fork_receipts')", [], |row| row.get(0)).map_err(|_| invalid())?;
    if !exists {
        return Ok(false);
    }
    let mut query = connection
        .prepare("SELECT published FROM archived_fork_receipts WHERE child=?1 LIMIT 2")
        .map_err(|_| invalid())?;
    let mut rows = query.query([child.to_string()]).map_err(|_| invalid())?;
    if let Some(row) = rows.next().map_err(|_| invalid())? {
        let published: i64 = row.get(0).map_err(|_| invalid())?;
        if published != 1 || rows.next().map_err(|_| invalid())?.is_some() {
            return Err(invalid());
        }
        return Ok(true);
    }
    Ok(false)
}

/// Recover the published identity instead of inventing a new branch at reopen.
/// Does not authorize history: the caller still validates the transcript normally.
pub(crate) fn published_branch(
    session: &crate::Session,
    deadline: Instant,
) -> Result<Option<String>, SessionError> {
    boundary(|| {
        check(deadline)?;
        if session.file_extension() != "tlog" {
            return Ok(None);
        }
        let db = database(&session.file_path, session.id)?;
        match std::fs::symlink_metadata(&db) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Ok(metadata) if metadata.file_type().is_file() => {}
            _ => return Err(invalid()),
        }
        let mut connection = crate::durable::open_binding_connection_with_flags(
            &db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(Instant::now()),
        );
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        if !check_publication(&connection, session.id)? {
            return Ok(None);
        }
        let receipt = load(&session.file_path, deadline)?;
        Ok(Some(receipt.child_branch.to_string()))
    })
}

pub(crate) fn restore_branch(
    session: &mut crate::Session,
    deadline: Instant,
) -> Result<(), SessionError> {
    if let Some(branch) = published_branch(session, deadline)? {
        session.branches.clear();
        session.branches.insert(
            branch.clone(),
            crate::SessionBranch {
                root_id: branch.clone(),
                entries: Vec::new(),
            },
        );
        session.current_branch = branch;
    }
    Ok(())
}

/// A receipt is loaded only from private custody; the file envelope or a matching
/// digest alone can never produce one. Parent retirement deliberately is ignored.
pub(crate) fn load(head: &Path, deadline: Instant) -> Result<Receipt, SessionError> {
    boundary(|| {
        check(deadline)?;
        let child = head
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(invalid)?;
        let db = database(head, child)?;
        let _lease = crate::identity_custody::require_not_retired_until(head, deadline)?;
        let mut connection = crate::durable::open_binding_connection_with_flags(
            &db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(|_| invalid())?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| invalid())?;
        let budget = crate::sqlite::RecoveryWorkBudget::new(
            deadline.saturating_duration_since(Instant::now()),
        );
        let connection =
            crate::sqlite::install_budget(&mut connection, &budget).map_err(|_| invalid())?;
        let mut query = connection.prepare("SELECT CASE WHEN length(CAST(payload AS BLOB))<=32768 THEN payload END,published FROM archived_fork_receipts WHERE child=?1 LIMIT 2").map_err(|_| invalid())?;
        let mut rows = query.query([child.to_string()]).map_err(|_| invalid())?;
        let row = rows.next().map_err(|_| invalid())?.ok_or_else(invalid)?;
        let payload: String = row.get(0).map_err(|_| invalid())?;
        let published: i64 = row.get(1).map_err(|_| invalid())?;
        if published != 1 || rows.next().map_err(|_| invalid())?.is_some() {
            return Err(invalid());
        }
        let receipt: Receipt = serde_json::from_str(&payload).map_err(|_| invalid())?;
        receipt.validate()?;
        if receipt.child != child {
            return Err(invalid());
        }
        crate::identity_custody::require_active_session_identity_until(
            &db,
            child,
            receipt.child_incarnation,
            deadline,
        )?;
        check(deadline)?;
        Ok(receipt)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{identity_custody::*, store::SessionStore};
    use std::io::Write;

    fn deadline() -> Instant {
        Instant::now() + std::time::Duration::from_secs(10)
    }

    fn raw(id: &str, parent: Option<&str>, text: &str) -> Vec<u8> {
        crate::compact_text::encode_entry(&SessionEntry {
            id: id.into(),
            parent_id: parent.map(str::to_owned),
            timestamp: chrono::Utc::now(),
            role: "user".into(),
            content: text.into(),
            metadata: Default::default(),
        })
        .into_bytes()
    }

    fn fixture(root: &Path) -> (std::path::PathBuf, Receipt, Vec<u8>, Vec<Vec<u8>>) {
        let db = root.join("history-bindings.sqlite");
        let parent = Uuid::new_v4();
        let child = Uuid::new_v4();
        let p = reserve_session_identity(&db, parent).expect("parent reservation");
        let c = reserve_session_identity(&db, child).expect("child reservation");
        // Real legacy parent defect: two subsequent writes refer to the same
        // earlier parent. Exact bytes must survive; never normalize this chain.
        let records = vec![
            raw("a", None, "one"),
            raw("b", Some("a"), "two"),
            raw("c", Some("a"), "three"),
        ];
        let origins = Origins::inherit(parent, None, 3, 3).expect("origins");
        let (receipt, bytes) = Receipt::capture(
            (parent, p, Uuid::new_v4()),
            (child, c, Uuid::new_v4()),
            b"TALOS\t1\t123\n",
            &records,
            origins,
            deadline(),
        )
        .expect("capture");
        (root.join(format!("{child}.tlog")), receipt, bytes, records)
    }

    fn write_synced(path: &Path, bytes: &[u8]) {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .expect("new file");
        file.write_all(bytes).expect("write");
        file.sync_all().expect("sync");
    }

    #[test]
    fn private_receipt_survives_parent_retirement_but_not_child_retirement() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, records) = fixture(root.path());
        assert!(
            load(&head, deadline()).is_err(),
            "reservation alone is not a receipt"
        );
        start_custody_trace();
        let intent = receipt.record_intent(&head, deadline());
        let trace = finish_custody_trace();
        eprintln!("fork-intent ok={} stages={trace:?}", intent.is_ok());
        intent.expect("intent before artifacts");
        assert!(!head.exists());
        write_synced(&head, &bytes);
        assert!(
            crate::history_page::open_source(&head).is_err(),
            "pending final head unavailable"
        );
        assert!(crate::CompactTextSessionStore.read_entries(&head).is_err());
        assert!(
            crate::CompactTextSessionStore
                .append_entry(
                    &head,
                    &decode(&raw("d", Some("c"), "suffix")).expect("entry")
                )
                .is_err()
        );
        assert_eq!(std::fs::read(&head).expect("bytes"), bytes);
        receipt.publish(&head, deadline()).expect("publish");
        assert_eq!(load(&head, deadline()).expect("private receipt"), receipt);
        let db = root.path().join("history-bindings.sqlite");
        let parent_head = root.path().join(format!("{}.tlog", receipt.parent));
        let mut parent_bytes = b"TALOS\t1\t123\n".to_vec();
        for raw in &records {
            parent_bytes.extend_from_slice(raw);
        }
        write_synced(&parent_head, &parent_bytes);
        crate::remove_session_artifacts_for_transcript(&parent_head)
            .expect("managed parent deletion");
        assert!(!parent_head.exists());
        let loaded = load(&head, deadline()).expect("child independent of parent");
        loaded
            .validate_records(&records, deadline())
            .expect("nonlinear inherited prefix");
        assert_eq!(loaded.origin(0), receipt.parent);
        assert_eq!(loaded.origin(3), receipt.child);
        assert!(
            receipt.publish(&head, deadline()).is_err(),
            "cannot republish"
        );
        assert!(
            receipt.record_intent(&head, deadline()).is_err(),
            "cannot overwrite"
        );
        retire_session_identity(&db, receipt.child).expect("retire child");
        assert!(load(&head, deadline()).is_err());
        assert!(reserve_session_identity(&db, receipt.child).is_err());
    }

    #[test]
    fn receipt_rejects_changed_prefix_colliding_suffix_and_wrong_parent() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, originals) = fixture(root.path());
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let loaded = load(&head, deadline()).expect("load");
        let mut records = originals.clone();
        records.push(raw("d", Some("c"), "legal suffix"));
        loaded
            .validate_records(&records, deadline())
            .expect("valid continuation");
        records[3] = raw("d", Some("a"), "wrong parent");
        assert!(loaded.validate_records(&records, deadline()).is_err());
        records[3] = raw("a", Some("c"), "colliding suffix");
        assert!(loaded.validate_records(&records, deadline()).is_err());
        records = originals.clone();
        records[1] = raw("b", Some("a"), "changed content");
        assert!(loaded.validate_records(&records, deadline()).is_err());
        assert!(
            loaded
                .validate_records(&originals[..2], deadline())
                .is_err()
        );
        assert!(loaded.validate_records(&originals, Instant::now()).is_err());
        // Copying the envelope and exact records under an unrelated UUID grants
        // no receipt, even if every public byte/hash matches.
        let forged = root.path().join(format!("{}.tlog", Uuid::new_v4()));
        write_synced(&forged, &bytes);
        assert!(load(&forged, deadline()).is_err());
    }

    #[test]
    fn changed_or_extended_head_cannot_commit_pending_receipt() {
        for suffix in [b"x".as_slice(), b"".as_slice()] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, mut bytes, _) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            if suffix.is_empty() {
                let last = bytes.len() - 2;
                bytes[last] ^= 1;
            } else {
                bytes.extend_from_slice(suffix);
            }
            write_synced(&head, &bytes);
            assert!(receipt.publish(&head, deadline()).is_err());
            assert!(load(&head, deadline()).is_err());
            assert!(crate::history_page::open_source(&head).is_err());
            assert_eq!(std::fs::read(&head).expect("unchanged"), bytes);
        }
    }

    #[test]
    fn large_original_record_survives_capture_publication_and_validation() {
        let root = tempfile::tempdir().expect("root");
        let (head, template, _, _) = fixture(root.path());
        let records = vec![raw("large", None, &"x".repeat(3 * 1024 * 1024))];
        let (receipt, bytes) = Receipt::capture(
            (
                template.parent,
                template.parent_incarnation,
                template.parent_branch,
            ),
            (
                template.child,
                template.child_incarnation,
                template.child_branch,
            ),
            b"TALOS\t1\t123\n",
            &records,
            Origins::inherit(template.parent, None, 1, 1).expect("origin"),
            deadline(),
        )
        .expect("large original");
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        load(&head, deadline())
            .expect("receipt")
            .validate_records(&records, deadline())
            .expect("exact raw record");
        assert_eq!(std::fs::read(&head).expect("bytes"), bytes);
    }

    #[test]
    fn intent_and_publication_exclude_existing_identity_readers() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, _) = fixture(root.path());
        let reader = require_not_retired(&head).expect("existing reader");
        assert!(receipt.record_intent(&head, deadline()).is_err());
        drop(reader);
        receipt
            .record_intent(&head, deadline())
            .expect("exclusive intent");
        assert!(require_not_retired(&head).is_err());
        write_synced(&head, &bytes);
        receipt
            .publish(&head, deadline())
            .expect("exclusive publication");
        let reader = require_not_retired(&head).expect("published reader");
        assert!(lock_reserved_identity(&head, receipt.child_incarnation, deadline()).is_err());
        drop(reader);
    }

    #[test]
    fn materialized_store_read_scan_append_and_atomic_commit_preserve_raw_prefix() {
        let root = tempfile::tempdir().expect("root");
        let (head, template, _, mut records) = fixture(root.path());
        records[0] = String::from_utf8(records[0].clone())
            .expect("text")
            .replacen("\t2:{}\n", "\t3:{ }\n", 1)
            .into_bytes();
        let (receipt, bytes) = Receipt::capture(
            (
                template.parent,
                template.parent_incarnation,
                template.parent_branch,
            ),
            (
                template.child,
                template.child_incarnation,
                template.child_branch,
            ),
            b"TALOS\t1\t123\n",
            &records,
            template.origins.clone(),
            deadline(),
        )
        .expect("noncanonical raw prefix");
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let store = crate::CompactTextSessionStore;
        let original = store.read_entries(&head).expect("receipt-aware store");
        assert_eq!(original.len(), 3);
        assert_eq!(original[2].parent_id.as_deref(), Some("a"));
        assert_eq!(
            store
                .scan_file(&head)
                .expect("discovery preview")
                .message_count,
            3
        );
        store
            .append_entry(
                &head,
                &decode(&raw("d", Some("c"), "append")).expect("record"),
            )
            .expect("append");
        let mut entries = store.read_entries(&head).expect("reopen");
        entries.push(decode(&raw("e", Some("d"), "atomic commit")).expect("record"));
        store
            .replace_entries_atomically(&head, &entries)
            .expect("atomic prefix preservation");
        let current = std::fs::read(&head).expect("raw head");
        assert!(
            current.starts_with(&bytes),
            "original noncanonical record bytes retained"
        );
        receipt
            .validate_materialized(&current, deadline())
            .expect("physical cutoff and logical suffix");
        assert_eq!(store.read_entries(&head).expect("final reopen").len(), 5);
        entries[0].content = "changed inherited record".into();
        assert!(store.replace_entries_atomically(&head, &entries).is_err());
        assert_eq!(std::fs::read(&head).expect("unchanged"), current);
        let entries = store.read_entries(&head).expect("valid head");
        for bad in [
            raw("bad", Some("a"), "wrong parent"),
            raw("a", Some("e"), "duplicate"),
        ] {
            let bad = decode(&bad).expect("record");
            assert!(store.append_entry(&head, &bad).is_err());
            let mut candidate = entries.clone();
            candidate.push(bad);
            assert!(store.replace_entries_atomically(&head, &candidate).is_err());
            assert_eq!(
                std::fs::read(&head).expect("failed writes preserve bytes"),
                current
            );
        }
        assert!(
            store
                .replace_with_retained_suffix(&head, &entries, 1, &[])
                .is_err()
        );
        assert!(
            store
                .replace_with_retained_suffix(&head, &entries, 0, &entries[..1])
                .is_err()
        );
        let engine = crate::compaction_engine::CompactionEngine::new(std::sync::Arc::new(store));
        engine
            .compact_segment(&head, root.path(), 1)
            .expect("proved compaction");
        let session =
            crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
        let restored = session
            .inspect_archive_recovery(deadline(), || false)
            .expect("compacted original bytes");
        assert_eq!(
            restored
                .original_record_prefix("c", deadline(), || false)
                .expect("raw inherited"),
            records
        );
        assert_eq!(restored.entries().len(), 5);
    }

    #[test]
    fn stripped_or_missing_envelope_cannot_fall_back_to_legacy_read_or_write() {
        for missing in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, _) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            if missing {
                std::fs::remove_file(&head).expect("external deletion fixture");
            } else {
                std::fs::write(&head, &bytes[ENVELOPE.len()..])
                    .expect("external downgrade fixture");
            }
            let before = std::fs::read(&head).ok();
            let store = crate::CompactTextSessionStore;
            assert!(store.read_entries(&head).is_err());
            assert!(store.scan_file(&head).is_err());
            let entry = decode(&raw("d", Some("c"), "suffix")).expect("entry");
            assert!(store.append_entry(&head, &entry).is_err());
            assert!(store.replace_entries_atomically(&head, &[entry]).is_err());
            assert!(load(&head, deadline()).is_err());
            assert_eq!(std::fs::read(&head).ok(), before);
        }
    }

    #[test]
    fn materialized_recovery_accepts_only_receipt_bound_exact_shared_records() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, records) = fixture(root.path());
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let parent_head = root.path().join(format!("{}.tlog", receipt.parent));
        let mut parent_bytes = b"TALOS\t1\t123\n".to_vec();
        for raw in &records {
            parent_bytes.extend_from_slice(raw);
        }
        write_synced(&parent_head, &parent_bytes);
        let parent = crate::Session::new(
            receipt.parent,
            String::new(),
            String::new(),
            parent_head.clone(),
        );
        let child = crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
        let misbound =
            crate::Session::new(receipt.parent, String::new(), String::new(), head.clone());
        assert!(
            misbound
                .inspect_archive_recovery(deadline(), || false)
                .is_err()
        );
        for session in [&parent, &child] {
            let evidence = session
                .inspect_archive_recovery(deadline(), || false)
                .expect("exact inherited intersection");
            assert_eq!(evidence.entries().len(), 3);
            assert_eq!(
                evidence
                    .original_record_prefix("c", deadline(), || false)
                    .expect("raw"),
                records
            );
        }
        // Same ID in the two independent ordinary suffixes is never covered by
        // inherited overlap, even if its bytes and parent happen to be identical.
        let suffix = raw("collision", Some("c"), "same raw suffix");
        crate::CompactTextSessionStore
            .append_entry(&head, &decode(&suffix).expect("entry"))
            .expect("child suffix");
        std::fs::OpenOptions::new()
            .append(true)
            .open(&parent_head)
            .expect("parent writer")
            .write_all(&suffix)
            .expect("parent suffix");
        assert!(
            parent
                .inspect_archive_recovery(deadline(), || false)
                .is_err()
        );
        assert!(
            child
                .inspect_archive_recovery(deadline(), || false)
                .is_err()
        );
        crate::remove_session_artifacts_for_transcript(&parent_head).expect("delete parent");
        let evidence = child
            .inspect_archive_recovery(deadline(), || false)
            .expect("child owns all bytes after parent deletion");
        assert_eq!(evidence.entries().len(), 4);
        let connection =
            Connection::open(root.path().join("history-bindings.sqlite")).expect("custody fixture");
        connection
            .execute("UPDATE archived_fork_receipts SET payload='{}'", [])
            .expect("damage custody");
        assert!(
            evidence.check_all_sources(deadline()).is_err(),
            "cached evidence cannot outlive its receipt"
        );
    }

    #[test]
    fn owned_binary_archive_never_extends_parent_even_after_child_deletion() {
        for compressed in [false, true] {
            if compressed && !cfg!(feature = "archive-compress-zstd") {
                continue;
            }
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, records) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let parent_head = root.path().join(format!("{}.tlog", receipt.parent));
            let mut parent_bytes = b"TALOS\t1\t123\n".to_vec();
            for raw in &records {
                parent_bytes.extend_from_slice(raw);
            }
            write_synced(&parent_head, &parent_bytes);
            let parent =
                crate::Session::new(receipt.parent, String::new(), String::new(), parent_head);
            let suffix =
                decode(&raw("child-only", Some("c"), "never parent history")).expect("record");
            crate::CompactTextSessionStore
                .append_entry(&head, &suffix)
                .expect("child append");
            let archive_bytes = std::fs::read(&head).expect("child raw bytes");
            #[cfg(feature = "archive-compress-zstd")]
            let archive_bytes = if compressed {
                zstd::stream::encode_all(archive_bytes.as_slice(), 1).expect("compressed bytes")
            } else {
                archive_bytes
            };
            let extension = if compressed { "tlog.zst" } else { "tlog" };
            let archive = root
                .path()
                .join(format!("s-owned-{}.{extension}", Uuid::new_v4()));
            crate::archive_publication::record(&head, &archive, &archive_bytes)
                .expect("ownership before publication");
            write_synced(&archive, &archive_bytes);
            for delete_child in [false, true] {
                if delete_child {
                    crate::remove_session_artifacts_for_transcript(&head)
                        .expect("managed child deletion");
                }
                let evidence = parent
                    .inspect_archive_recovery(deadline(), || false)
                    .expect("parent excludes child archive");
                assert_eq!(evidence.entries().len(), 3);
                assert!(
                    !evidence
                        .entries()
                        .iter()
                        .any(|entry| entry.id == "child-only")
                );
            }
            // Replacing the immutable owned archive, even with syntactically valid
            // binary contents, must invalidate its physical digest commitment.
            std::fs::write(&archive, &bytes).expect("external alteration fixture");
            assert!(
                parent
                    .inspect_archive_recovery(deadline(), || false)
                    .is_err()
            );
            std::fs::remove_file(&archive).expect("fixture replacement");
            let legacy_name = root
                .path()
                .join(format!("s-{}.{extension}", Uuid::new_v4()));
            write_synced(&legacy_name, &archive_bytes);
            assert!(
                parent
                    .inspect_archive_recovery(deadline(), || false)
                    .is_err(),
                "envelope archive cannot masquerade as legacy overlap"
            );
        }
    }

    #[test]
    fn compacted_child_reconstructs_owned_raw_prefix_for_keep_some_and_zero() {
        for keep in [0usize, 1] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, records) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let archive = root.path().join(format!("s-owned-{}.tlog", Uuid::new_v4()));
            crate::archive_publication::record(&head, &archive, &bytes).expect("archive intent");
            write_synced(&archive, &bytes);
            let marker = SessionEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "system".into(),
                content: format!(
                    "[Compaction summary: {} earlier entries summarized, {keep} recent entries preserved]",
                    3 - keep
                ),
                metadata: Default::default(),
            };
            if keep == 0 {
                crate::compaction_locator::record(&head, &archive, &marker, "c")
                    .expect("zero retention bridge");
            }
            // Construct the exact on-disk result of keepN/keep0. Production
            // compaction remains gated until its writer can prove this output.
            let mut compacted = bytes[..envelope_header_length(&bytes).expect("header")].to_vec();
            compacted.extend_from_slice(crate::compact_text::encode_entry(&marker).as_bytes());
            for raw in &records[3 - keep..] {
                compacted.extend_from_slice(raw);
            }
            compacted.extend_from_slice(&raw("d", Some("c"), "post-compaction suffix"));
            std::fs::write(&head, &compacted).expect("compacted fixture");
            let session =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            let parent_head = root.path().join(format!("{}.tlog", receipt.parent));
            let mut parent_bytes = b"TALOS\t1\t123\n".to_vec();
            for raw in &records {
                parent_bytes.extend_from_slice(raw);
            }
            write_synced(&parent_head, &parent_bytes);
            let parent = crate::Session::new(
                receipt.parent,
                String::new(),
                String::new(),
                parent_head.clone(),
            );
            let parent_evidence = parent
                .inspect_archive_recovery(deadline(), || false)
                .expect("compacted child sibling does not block parent");
            assert_eq!(parent_evidence.entries().len(), 3);
            assert_eq!(
                parent_evidence
                    .original_record_prefix("c", deadline(), || false)
                    .expect("parent raw"),
                records
            );
            let evidence = session
                .inspect_archive_recovery(deadline(), || false)
                .expect("reconstruct child");
            assert_eq!(evidence.entries().len(), 4);
            assert_eq!(
                evidence
                    .original_record_prefix("c", deadline(), || false)
                    .expect("exact inherited prefix"),
                records
            );
            let (messages, tail) = evidence
                .into_restored_history(deadline(), || false)
                .expect("normal decoder");
            assert_eq!(messages.len(), 4);
            assert_eq!(tail.expect("tail").id, "d");
            let store = crate::CompactTextSessionStore;
            let physical = store
                .read_entries(&head)
                .expect("validated compacted store read");
            assert_eq!(physical.len(), keep + 2);
            let next = decode(&raw("e", Some("d"), "continued after restart")).expect("entry");
            store
                .append_entry(&head, &next)
                .expect("append through owned archive proof");
            let after = std::fs::read(&head).expect("physical bytes");
            assert!(
                after.starts_with(&compacted),
                "marker/envelope/retained bytes survive append"
            );
            let reopened =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            assert_eq!(
                reopened
                    .inspect_archive_recovery(deadline(), || false)
                    .expect("reopened history")
                    .entries()
                    .len(),
                5
            );
            let collision =
                decode(&raw("a", Some("e"), "collides with archived prefix")).expect("entry");
            assert!(store.append_entry(&head, &collision).is_err());
            assert_eq!(
                std::fs::read(&head).expect("failed write preserves head"),
                after
            );
            assert_eq!(std::fs::read(&archive).expect("archive preserved"), bytes);
            // Valid ownership of altered bytes cannot substitute for the child
            // inheritance receipt. Change one original while updating only the
            // physical archive fixture commitment.
            let altered = bytes
                .windows(3)
                .position(|part| part == b"one")
                .expect("content");
            let mut changed = bytes.clone();
            changed[altered] = b'O';
            std::fs::write(&archive, &changed).expect("alter archive");
            let connection =
                Connection::open(root.path().join("history-bindings.sqlite")).expect("fixture db");
            connection
                .execute(
                    "UPDATE archive_publication_intents SET digest=?1",
                    [crate::compaction_locator::digest(&changed)],
                )
                .expect("alter physical commitment");
            assert!(
                session
                    .inspect_archive_recovery(deadline(), || false)
                    .is_err()
            );
        }
    }

    #[test]
    fn compacted_receipt_never_accepts_wrong_archive_authority_or_record_order() {
        for mutation in ["missing-envelope", "foreign-owner", "reordered-records"] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, records) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let archive = root.path().join(format!("s-owned-{}.tlog", Uuid::new_v4()));
            let archive_bytes = match mutation {
                "missing-envelope" => bytes[ENVELOPE.len()..].to_vec(),
                "reordered-records" => {
                    let mut changed =
                        bytes[..envelope_header_length(&bytes).expect("header")].to_vec();
                    // Both b and c refer to a. The changed order remains a
                    // plausible legacy parent graph, but contradicts receipt.
                    for index in [0, 2, 1] {
                        changed.extend_from_slice(&records[index]);
                    }
                    changed
                }
                _ => bytes.clone(),
            };
            let owner_head = if mutation == "foreign-owner" {
                root.path().join(format!("{}.tlog", Uuid::new_v4()))
            } else {
                head.clone()
            };
            crate::archive_publication::record(&owner_head, &archive, &archive_bytes)
                .expect("real physical binding");
            write_synced(&archive, &archive_bytes);
            let marker = SessionEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: chrono::Utc::now(),
                role: "system".into(),
                content:
                    "[Compaction summary: 3 earlier entries summarized, 0 recent entries preserved]"
                        .into(),
                metadata: Default::default(),
            };
            let tail = if mutation == "reordered-records" {
                "b"
            } else {
                "c"
            };
            crate::compaction_locator::record(&head, &archive, &marker, tail)
                .expect("bridge fixture");
            let mut compacted = bytes[..envelope_header_length(&bytes).expect("header")].to_vec();
            compacted.extend_from_slice(crate::compact_text::encode_entry(&marker).as_bytes());
            std::fs::write(&head, &compacted).expect("keep zero fixture");
            let session =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            let error = session
                .inspect_archive_recovery(deadline(), || false)
                .err()
                .expect("invalid proof refused");
            let reason = error.to_string();
            let expected = match mutation {
                "missing-envelope" => "inherited archive lacks owned envelope",
                "foreign-owner" => "compaction archive belongs to another owner",
                _ => "fork inheritance receipt unavailable",
            };
            assert!(reason.contains(expected), "{mutation}: {reason}");
            assert_eq!(std::fs::read(&head).expect("head preserved"), compacted);
            assert_eq!(
                std::fs::read(&archive).expect("archive preserved"),
                archive_bytes
            );
        }
    }

    #[test]
    fn append_cannot_exceed_physical_limit_when_logical_records_still_fit() {
        let root = tempfile::tempdir().expect("root");
        let (head, template, _, _) = fixture(root.path());
        let suffix = raw("next", Some("large"), "small suffix");
        let header = b"TALOS\t1\t123\n";
        let overhead = ENVELOPE.len() + header.len();
        let target_raw_len = MAX_BYTES - suffix.len() - overhead / 2;
        let mut content = "x".repeat(target_raw_len);
        let initial = raw("large", None, &content);
        content.truncate(content.len() - (initial.len() - target_raw_len));
        drop(initial);
        let records = vec![raw("large", None, &content)];
        drop(content);
        assert_eq!(records[0].len(), target_raw_len);
        assert!(target_raw_len + suffix.len() <= MAX_BYTES);
        assert!(target_raw_len + overhead + suffix.len() > MAX_BYTES);
        let (receipt, bytes) = Receipt::capture(
            (
                template.parent,
                template.parent_incarnation,
                template.parent_branch,
            ),
            (
                template.child,
                template.child_incarnation,
                template.child_branch,
            ),
            header,
            &records,
            Origins::inherit(template.parent, None, 1, 1).expect("origin"),
            deadline(),
        )
        .expect("near-limit valid head");
        drop(records);
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let expected = crate::compaction_locator::digest(&bytes);
        drop(bytes);
        let error = crate::CompactTextSessionStore
            .append_entry(&head, &decode(&suffix).expect("suffix"))
            .expect_err("physical limit rejects before publication");
        assert!(
            error.to_string().contains("physical transcript byte bound"),
            "{error}"
        );
        assert_eq!(
            crate::compaction_locator::digest(&std::fs::read(&head).expect("unchanged head")),
            expected
        );
    }

    #[test]
    fn cold_session_append_uses_validated_logical_tail_across_keep_zero() {
        for keep in [None, Some(0usize), Some(1)] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, records) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            if let Some(keep) = keep {
                let archive = root.path().join(format!("s-owned-{}.tlog", Uuid::new_v4()));
                crate::archive_publication::record(&head, &archive, &bytes)
                    .expect("archive intent");
                write_synced(&archive, &bytes);
                let marker = SessionEntry {
                    id: Uuid::new_v4().to_string(),
                    parent_id: None,
                    timestamp: chrono::Utc::now(),
                    role: "system".into(),
                    content: format!(
                        "[Compaction summary: {} earlier entries summarized, {keep} recent entries preserved]",
                        3 - keep
                    ),
                    metadata: Default::default(),
                };
                if keep == 0 {
                    crate::compaction_locator::record(&head, &archive, &marker, "c")
                        .expect("bridge");
                }
                let mut compacted =
                    bytes[..envelope_header_length(&bytes).expect("header")].to_vec();
                compacted.extend_from_slice(crate::compact_text::encode_entry(&marker).as_bytes());
                for raw in &records[3 - keep..] {
                    compacted.extend_from_slice(raw);
                }
                std::fs::write(&head, compacted).expect("compacted fixture");
            }
            let physical = std::fs::read(&head).expect("before");
            let mut previous = "c".to_owned();
            for round in 0..2 {
                let reopened =
                    crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
                reopened
                    .append(&talos_core::message::Message::User {
                        content: format!("cold continuation {round}"),
                    })
                    .expect("cold session append");
                let evidence = reopened
                    .inspect_archive_recovery(deadline(), || false)
                    .expect("reopen recovery");
                let tail = evidence.entries().last().expect("tail");
                assert_eq!(tail.parent_id.as_deref(), Some(previous.as_str()));
                previous = tail.id.clone();
                assert_eq!(evidence.entries().len(), 4 + round);
            }
            assert!(std::fs::read(&head).expect("after").starts_with(&physical));
            let connection = Connection::open(root.path().join("history-bindings.sqlite"))
                .expect("fixture custody");
            connection
                .execute("UPDATE archived_fork_receipts SET payload='{}'", [])
                .expect("damage receipt");
            let before = std::fs::read(&head).expect("before rejection");
            let reopened =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            assert!(
                reopened
                    .append(&talos_core::message::Message::User {
                        content: "must refuse".into()
                    })
                    .is_err()
            );
            assert_eq!(std::fs::read(&head).expect("unchanged"), before);
        }
    }

    #[test]
    fn zero_compaction_rejects_corrupt_locator_digest_before_head_replacement() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, records) = fixture(root.path());
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let archive = root.path().join(format!("s-owned-{}.tlog", Uuid::new_v4()));
        crate::archive_publication::record(&head, &archive, &bytes).expect("archive intent");
        write_synced(&archive, &bytes);
        let marker = SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            timestamp: chrono::Utc::now(),
            role: "system".into(),
            content:
                "[Compaction summary: 3 earlier entries summarized, 0 recent entries preserved]"
                    .into(),
            metadata: Default::default(),
        };
        crate::compaction_locator::record(&head, &archive, &marker, "c").expect("locator");
        let connection = Connection::open(database(&head, receipt.child).expect("database"))
            .expect("connection");
        connection
            .execute(
                "UPDATE compaction_zero_locators SET archive_digest=?1",
                ["0".repeat(64)],
            )
            .expect("corrupt only archive digest");
        drop(connection);
        let prior: Vec<_> = records
            .iter()
            .map(|raw| decode(raw).expect("entry"))
            .collect();
        let error = crate::compact_text::rewrite_archived_records(
            &head,
            &prior,
            prior.len(),
            &marker,
            &archive,
        )
        .expect_err("bad locator must not replace head");
        assert!(
            error
                .to_string()
                .contains("fork compaction bridge unavailable"),
            "{error}"
        );
        assert_eq!(std::fs::read(&head).expect("unchanged"), bytes);
    }

    #[test]
    fn production_compaction_budget_refusal_preserves_readable_head() {
        for marker_growth in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let (head, template, _, _) = fixture(root.path());
            let records = if marker_growth {
                let first = raw("a", None, "");
                let target =
                    MAX_BYTES / 2 - ENVELOPE.len() - b"TALOS\t1\t123\n".len() - first.len();
                let mut content = "x".repeat(target);
                let overhead = raw("large", Some("a"), &content).len() - target;
                content.truncate(target - overhead);
                vec![first, raw("large", Some("a"), &content)]
            } else {
                vec![raw("large", None, &"x".repeat(33 * 1024 * 1024))]
            };
            let (receipt, bytes) = Receipt::capture(
                (
                    template.parent,
                    template.parent_incarnation,
                    template.parent_branch,
                ),
                (
                    template.child,
                    template.child_incarnation,
                    template.child_branch,
                ),
                b"TALOS\t1\t123\n",
                &records,
                Origins::inherit(template.parent, None, records.len(), records.len())
                    .expect("origin"),
                deadline(),
            )
            .expect("capture");
            if marker_growth {
                // Old head plus raw archive exactly fits. Replacing the short
                // first record with the longer count marker would exceed the cap.
                assert_eq!(bytes.len() * 2, MAX_BYTES);
                assert!(records[0].len() < 80);
            }
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let inventory = || {
                let mut names: Vec<_> = std::fs::read_dir(head.parent().expect("directory"))
                    .expect("inventory")
                    .map(|e| e.expect("entry").file_name())
                    .collect();
                names.sort();
                names
            };
            let before = inventory();
            let engine = crate::compaction_engine::CompactionEngine::new(std::sync::Arc::new(
                crate::CompactTextSessionStore,
            ))
            .with_compressor(Box::new(crate::compression::NoCompressor));
            let error = engine
                .compact_segment(
                    &head,
                    head.parent().expect("directory"),
                    usize::from(marker_growth),
                )
                .expect_err("duplicate archive exceeds shared budget");
            assert!(
                error
                    .to_string()
                    .contains("archive publication exceeds recovery budget"),
                "{error}"
            );
            let after: Vec<_> = inventory()
                .into_iter()
                .filter(|name| {
                    name != "history-bindings.sqlite-shm" && name != "history-bindings.sqlite-wal"
                })
                .collect();
            assert_eq!(after, before);
            let connection = Connection::open(database(&head, receipt.child).expect("database"))
                .expect("inspect database");
            let intents: i64 = connection
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name='archive_publication_intents'",
                    [],
                    |row| row.get(0),
                )
                .expect("no archive intent table");
            assert_eq!(intents, 0);
            assert_eq!(std::fs::read(&head).expect("head"), bytes);
            assert_eq!(
                receipt
                    .current_records(&head, &bytes, deadline())
                    .expect("readable"),
                records
            );
        }
    }

    #[test]
    fn failed_host_capture_retires_reservation_without_child_intent() {
        let root = tempfile::tempdir().expect("root");
        let (head, parent, bytes, _) = fixture(root.path());
        parent.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        parent.publish(&head, deadline()).expect("publish");
        let source = crate::Session::new(parent.child, String::new(), String::new(), head.clone());
        let id = Uuid::new_v4();
        let child = crate::Session::new(
            id,
            String::new(),
            String::new(),
            root.path().join(format!("{id}.tlog")),
        );
        let lease = crate::SessionRuntimeLease::acquire(&source.file_path).expect("source lease");
        assert!(
            source
                .publish_history_fork(&child, &lease, Some("absent"), deadline(), || false)
                .is_err()
        );
        assert!(!child.file_path.exists());
        assert_eq!(std::fs::read(&head).expect("source preserved"), bytes);
        let db = root.path().join("history-bindings.sqlite");
        let connection = Connection::open(&db).expect("db");
        let retired: i64 = connection
            .query_row(
                "SELECT retired FROM session_identity_custody WHERE session_id=?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .expect("reservation tombstone");
        assert_eq!(retired, 1);
        let intents: i64 = connection
            .query_row(
                "SELECT count(*) FROM archived_fork_receipts WHERE child=?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .expect("intent count");
        assert_eq!(intents, 0);
        assert!(reserve_session_identity(&db, id).is_err());
    }

    #[test]
    fn published_runtime_snapshot_survives_parent_removal_and_preserves_later_activation() {
        use talos_core::message::Message;
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let parent = manager.create_session("runtime", "").expect("parent");
        parent
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        let identity =
            crate::SessionRuntimeIdentity::new("provider", "model-at-fork", Some("variant"));
        crate::PendingSubmissionStore::for_session(&parent)
            .initialize_runtime_identity(identity.clone())
            .expect("parent identity");
        let source_lease =
            crate::SessionRuntimeLease::acquire(&parent.file_path).expect("parent lease");
        let mut children = Vec::new();
        for _ in 0..2 {
            let id = Uuid::new_v4();
            let child = crate::Session::new(
                id,
                String::new(),
                String::new(),
                parent.file_path.with_file_name(format!("{id}.tlog")),
            );
            parent
                .publish_history_fork(&child, &source_lease, None, deadline(), || false)
                .expect("publish");
            let receipt = load(&child.file_path, deadline()).expect("receipt");
            assert_eq!(receipt.version, 2);
            assert_eq!(receipt.runtime_identity, Some(identity.clone()));
            children.push(child);
        }
        drop(source_lease);
        std::fs::remove_file(&parent.file_path).expect("remove fixture parent");
        let child = &children[0];
        child.compact_archived(0).expect("keep0");
        let lease = crate::SessionRuntimeLease::acquire(&child.file_path).expect("child lease");
        let bytes = std::fs::read(&child.file_path).expect("head");
        assert!(
            child
                .restore_history_fork_runtime(&lease, deadline())
                .expect("restore")
        );
        assert!(
            !child
                .restore_history_fork_runtime(&lease, deadline())
                .expect("retry")
        );
        let store = crate::PendingSubmissionStore::for_session(child);
        assert_eq!(
            store
                .runtime_state()
                .expect("state")
                .expect("identity")
                .activation
                .target,
            identity
        );
        let later = crate::SessionRuntimeIdentity::new("provider", "later-model", None);
        let activation = crate::SessionRuntimeActivation::new(1, identity.clone(), later);
        store
            .stage_runtime_activation(0, &activation)
            .expect("later activation");
        let before = store.runtime_state().expect("state");
        assert!(
            !child
                .restore_history_fork_runtime(&lease, deadline())
                .expect("preserve later pending activation")
        );
        assert_eq!(store.runtime_state().expect("after"), before);
        assert_eq!(std::fs::read(&child.file_path).expect("unchanged"), bytes);
        let conflict = &children[1];
        let conflict_store = crate::PendingSubmissionStore::for_session(conflict);
        conflict_store
            .initialize_runtime_identity(crate::SessionRuntimeIdentity::new(
                "other",
                "conflicting",
                None,
            ))
            .expect("fixture conflict");
        let before = conflict_store.runtime_state().expect("before conflict");
        let lease =
            crate::SessionRuntimeLease::acquire(&conflict.file_path).expect("conflict lease");
        assert!(
            conflict
                .restore_history_fork_runtime(&lease, deadline())
                .is_err()
        );
        assert_eq!(
            conflict_store.runtime_state().expect("after conflict"),
            before
        );
    }

    #[test]
    fn manager_reopen_restores_receipt_branch_after_compaction_and_parent_removal() {
        use talos_core::message::Message;
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let parent = manager.create_session("branch", "").expect("parent");
        for i in 0..4 {
            parent
                .append(&Message::User {
                    content: format!("original {i}"),
                })
                .expect("append");
        }
        parent.compact_archived(0).expect("parent compact");
        let id = Uuid::new_v4();
        let child = crate::Session::new(
            id,
            String::new(),
            String::new(),
            parent.file_path.with_file_name(format!("{id}.tlog")),
        );
        let lease = crate::SessionRuntimeLease::acquire(&parent.file_path).expect("source lease");
        parent
            .publish_history_fork(&child, &lease, None, deadline(), || false)
            .expect("publish");
        drop(lease);
        std::fs::remove_file(&parent.file_path).expect("remove disposable parent");
        for keep in [None, Some(1), Some(0)] {
            if let Some(keep) = keep {
                child.compact_archived(keep).expect("compact child");
            }
            let reopened = manager.get_session(&id).expect("reopen");
            assert_eq!(reopened.current_branch, child.current_branch);
            assert_eq!(reopened.branches.len(), 1);
            let before = std::fs::read(&reopened.file_path).expect("before synthetic fork");
            let mut synthetic = reopened.clone();
            let tail = load(&reopened.file_path, deadline()).expect("receipt").tail;
            assert!(synthetic.fork(&tail).is_err());
            assert_eq!(synthetic.current_branch, child.current_branch);
            assert_eq!(
                std::fs::read(&reopened.file_path).expect("unchanged"),
                before
            );
            assert!(reopened.branches.contains_key(&child.current_branch));
            let nested_id = Uuid::new_v4();
            let nested = crate::Session::new(
                nested_id,
                String::new(),
                String::new(),
                child.file_path.with_file_name(format!("{nested_id}.tlog")),
            );
            let lease =
                crate::SessionRuntimeLease::acquire(&reopened.file_path).expect("child lease");
            reopened
                .publish_history_fork(&nested, &lease, None, deadline(), || false)
                .expect("nested publication");
            let receipt = load(&nested.file_path, deadline()).expect("nested receipt");
            assert_eq!(receipt.parent_branch.to_string(), child.current_branch);
            assert_eq!(
                manager
                    .get_session(&nested_id)
                    .expect("nested reopen")
                    .current_branch,
                nested.current_branch
            );
        }
    }

    #[test]
    fn host_fork_publication_requires_matching_runtime_lease() {
        let root = tempfile::tempdir().expect("root");
        let (head, parent, bytes, _) = fixture(root.path());
        parent.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        parent.publish(&head, deadline()).expect("publish");
        let source = crate::Session::new(parent.child, String::new(), String::new(), head);
        source.compact_archived(0).expect("archive");
        let id = Uuid::new_v4();
        let child = crate::Session::new(
            id,
            String::new(),
            String::new(),
            root.path().join(format!("{id}.tlog")),
        );
        let wrong = crate::SessionRuntimeLease::acquire(&child.file_path).expect("wrong lease");
        assert!(
            source
                .publish_history_fork(&child, &wrong, None, deadline(), || false)
                .is_err()
        );
        drop(wrong);
        assert!(!child.file_path.exists());
        let lease = crate::SessionRuntimeLease::acquire(&source.file_path).expect("source lease");
        let incompatible = crate::Session::with_store(
            child.id,
            String::new(),
            String::new(),
            child.file_path.clone(),
            std::sync::Arc::new(crate::JsonlSessionStore),
        );
        assert!(
            source
                .publish_history_fork(&incompatible, &lease, None, deadline(), || false)
                .is_err()
        );
        assert!(!child.file_path.exists());
        let tail = source
            .publish_history_fork(&child, &lease, Some("b"), deadline(), || false)
            .expect("host publication");
        assert_eq!(tail.id, "b");
        let receipt = load(&child.file_path, deadline()).expect("child receipt");
        assert_eq!(receipt.child_branch.to_string(), child.current_branch);
        let mut scan = child.begin_message_scan().expect("child scan");
        while !scan.step(deadline(), || false).expect("page") {}
        assert_eq!(scan.finish().expect("history").len(), 2);
    }

    #[test]
    fn captured_archive_publisher_creates_independent_readable_child() {
        let root = tempfile::tempdir().expect("root");
        let (head, parent, bytes, records) = fixture(root.path());
        parent.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        parent.publish(&head, deadline()).expect("publish");
        let source = crate::Session::new(parent.child, String::new(), String::new(), head.clone());
        source.compact_archived(0).expect("archive parent");
        let child = Uuid::new_v4();
        let db = root.path().join("history-bindings.sqlite");
        let incarnation = reserve_session_identity(&db, child).expect("reserve child");
        let capture = ForkCapture::from_session(
            &source,
            (child, incarnation, Uuid::new_v4()),
            None,
            deadline(),
            || false,
        )
        .expect("capture");
        let destination = root.path().join(format!("{child}.tlog"));
        capture
            .publish(&destination, deadline(), || false)
            .expect("publish actual child");
        assert_eq!(std::fs::read(&destination).expect("child"), capture.bytes);
        assert!(
            capture.publish(&destination, deadline(), || false).is_err(),
            "no overwrite"
        );
        drop(capture);
        crate::identity_custody::retire_session_identity(&db, source.id).expect("retire parent");
        std::fs::remove_file(&head).expect("remove disposable parent");
        let receipt = load(&destination, deadline()).expect("independent receipt");
        let child_session =
            crate::Session::new(child, String::new(), String::new(), destination.clone());
        child_session
            .append(&talos_core::message::Message::User {
                content: "child suffix".into(),
            })
            .expect("continue child");
        let bytes = std::fs::read(&destination).expect("child bytes");
        let current = receipt
            .current_records(&destination, &bytes, deadline())
            .expect("logical originals");
        assert_eq!(&current[..3], &records);
        assert_eq!(current.len(), 4);
        let mut scan = child_session.begin_message_scan().expect("restore child");
        while !scan.step(deadline(), || false).expect("page") {}
        assert_eq!(scan.finish().expect("independent history").len(), 4);
    }

    #[test]
    fn source_capture_binds_archived_prefix_and_retains_parent_identity() {
        let root = tempfile::tempdir().expect("root");
        let (head, parent, bytes, records) = fixture(root.path());
        parent.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        parent.publish(&head, deadline()).expect("publish");
        let source = crate::Session::new(parent.child, String::new(), String::new(), head.clone());
        source.compact_archived(0).expect("archive parent");
        let child = Uuid::new_v4();
        let db = root.path().join("history-bindings.sqlite");
        let incarnation = reserve_session_identity(&db, child).expect("reserve child");
        assert!(
            ForkCapture::from_session(
                &source,
                (child, incarnation, Uuid::new_v4()),
                None,
                deadline(),
                || true
            )
            .is_err()
        );
        let capture = ForkCapture::from_session(
            &source,
            (child, incarnation, Uuid::new_v4()),
            Some("b"),
            deadline(),
            || false,
        )
        .expect("capture archived prefix");
        assert_eq!(capture.receipt.count, 2);
        assert_eq!(capture.receipt.tail, "b");
        assert_eq!(capture.receipt.origin(0), parent.parent);
        assert_eq!(
            capture
                .receipt
                .validate_materialized(&capture.bytes, deadline())
                .expect("raw"),
            records[..2]
        );
        assert!(crate::identity_custody::retire_session_identity(&db, source.id).is_err());
        capture
            .check_source(deadline(), || false)
            .expect("unchanged");
        source
            .append(&talos_core::message::Message::User {
                content: "new suffix".into(),
            })
            .expect("source append");
        assert!(capture.check_source(deadline(), || false).is_err());
        drop(capture);
        crate::identity_custody::retire_session_identity(&db, source.id).expect("released");
    }

    #[test]
    fn receipt_message_scan_spans_entry_pages_without_duplicate_delivery() {
        let root = tempfile::tempdir().expect("root");
        let (head, template, _, _) = fixture(root.path());
        let count = crate::history_page::HISTORY_PAGE_ENTRIES + 3;
        let records: Vec<_> = (0..count)
            .map(|i| {
                raw(
                    &format!("r{i}"),
                    if i == 0 { None } else { Some("r0") },
                    &format!("message-{i}"),
                )
            })
            .collect();
        let (receipt, bytes) = Receipt::capture(
            (
                template.parent,
                template.parent_incarnation,
                template.parent_branch,
            ),
            (
                template.child,
                template.child_incarnation,
                template.child_branch,
            ),
            b"TALOS\t1\t123\n",
            &records,
            Origins::inherit(template.parent, None, count, count).expect("origins"),
            deadline(),
        )
        .expect("capture");
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let session = crate::Session::new(receipt.child, String::new(), String::new(), head);
        session.compact_archived(1).expect("compact");
        let mut scan = session.begin_message_scan().expect("scan");
        assert!(!scan.step(deadline(), || false).expect("first page"));
        assert!(matches!(
            scan.step(deadline(), || true),
            Err(crate::history_page::HistoryPageError::Stopped)
        ));
        assert!(scan.step(deadline(), || false).expect("retry second page"));
        let (messages, tail) = scan
            .finish_with_tail_cancellable(deadline(), || false)
            .expect("finish");
        assert_eq!(messages.len(), count);
        for (i, message) in messages.iter().enumerate() {
            assert!(
                matches!(message, talos_core::message::Message::User { content } if content == &format!("message-{i}"))
            );
        }
        assert_eq!(tail.expect("tail").id, format!("r{}", count - 1));
    }

    #[test]
    fn message_scan_restores_receipt_bound_history_and_rechecks_custody() {
        for keep in [None, Some(0), Some(1)] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, _) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let session =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            if let Some(keep) = keep {
                session.compact_archived(keep).expect("compact");
            }
            let mut scan = session.begin_message_scan().expect("capture");
            assert!(matches!(
                scan.step(deadline(), || true),
                Err(crate::history_page::HistoryPageError::Stopped)
            ));
            while !scan.step(deadline(), || false).expect("restore page") {}
            let (messages, tail) = scan.finish_with_tail().expect("restored");
            assert_eq!(tail.expect("tail").id, "c");
            let content: Vec<_> = messages
                .iter()
                .filter_map(|m| match m {
                    talos_core::message::Message::User { content } => Some(content.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(content, ["one", "two", "three"]);
            let mut cancelled = session.begin_message_scan().expect("capture cancellation");
            while !cancelled
                .step(deadline(), || false)
                .expect("decode cancellation")
            {}
            assert!(matches!(
                cancelled.finish_with_tail_cancellable(deadline(), || true),
                Err(crate::history_page::HistoryPageError::Stopped)
            ));
            if keep.is_some() {
                let archive = std::fs::read_dir(root.path())
                    .expect("inventory")
                    .map(|e| e.expect("entry").path())
                    .find(|p| {
                        p.file_name()
                            .and_then(|s| s.to_str())
                            .is_some_and(crate::archive_publication::owned)
                    })
                    .expect("archive");
                let saved = std::fs::read(&archive).expect("save fixture archive");
                let mut stale = session.begin_message_scan().expect("capture archive");
                while !stale.step(deadline(), || false).expect("decode archive") {}
                std::fs::remove_file(&archive).expect("remove disposable archive");
                assert!(
                    stale.finish().is_err(),
                    "archive removal invalidates finish"
                );
                let mut missing = session.begin_message_scan().expect("capture missing");
                assert!(
                    missing.step(deadline(), || false).is_err(),
                    "never return retained tail only"
                );
                write_synced(&archive, &saved);
            }
            let mut scan = session.begin_message_scan().expect("capture again");
            while !scan.step(deadline(), || false).expect("restore again") {}
            let connection =
                Connection::open(database(&head, receipt.child).expect("db")).expect("connection");
            connection
                .execute("UPDATE archived_fork_receipts SET payload='{}'", [])
                .expect("corrupt custody");
            assert!(
                scan.finish().is_err(),
                "custody changes before finish must reject"
            );
            let mut scan = session.begin_message_scan().expect("capture corrupt");
            assert!(scan.step(deadline(), || false).is_err());
        }
    }

    #[test]
    fn production_compaction_preserves_receipt_across_repeated_keep_some_and_zero() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, records) = fixture(root.path());
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        receipt.publish(&head, deadline()).expect("publish");
        let mut expected_count = 3;
        for keep in [1, 0, 1, 0] {
            let session =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            session
                .compact_archived(keep)
                .expect("production compaction");
            assert!(std::fs::read(&head).expect("head").starts_with(ENVELOPE));
            let before = session
                .inspect_archive_recovery(deadline(), || false)
                .expect("compacted recovery");
            assert_eq!(before.entries().len(), expected_count);
            assert_eq!(
                before
                    .original_record_prefix("c", deadline(), || false)
                    .expect("original prefix"),
                records
            );
            let tail = before.entries().last().expect("logical tail").id.clone();
            let reopened =
                crate::Session::new(receipt.child, String::new(), String::new(), head.clone());
            reopened
                .append(&talos_core::message::Message::User {
                    content: format!("continuation {expected_count}"),
                })
                .expect("cold continuation");
            expected_count += 1;
            let after = reopened
                .inspect_archive_recovery(deadline(), || false)
                .expect("appended history");
            assert_eq!(after.entries().len(), expected_count);
            assert_eq!(
                after.entries().last().expect("tail").parent_id.as_deref(),
                Some(tail.as_str())
            );
        }
    }

    #[test]
    fn missing_malformed_or_wrong_incarnation_receipts_never_load() {
        for mutation in [
            "DELETE FROM archived_fork_receipts",
            "UPDATE archived_fork_receipts SET payload='{}'",
            "UPDATE archived_fork_receipts SET payload=printf('%.*c',32769,'x')",
            "UPDATE archived_fork_receipts SET payload=json_set(payload,'$.count',4)",
            "UPDATE archived_fork_receipts SET payload=json_set(payload,'$.child_incarnation','00000000-0000-4000-8000-000000000001')",
            "UPDATE archived_fork_receipts SET payload=json_set(payload,'$.child','00000000-0000-4000-8000-000000000001')",
        ] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, _) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            receipt.publish(&head, deadline()).expect("publish");
            let connection = Connection::open(root.path().join("history-bindings.sqlite"))
                .expect("private fixture mutation");
            connection.execute_batch(mutation).expect("mutate");
            drop(connection);
            assert!(load(&head, deadline()).is_err(), "{mutation}");
            assert_eq!(std::fs::read(&head).expect("head preserved"), bytes);
        }
    }

    #[test]
    fn expired_operations_never_publish_or_create_receipt_table() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, _) = fixture(root.path());
        assert!(receipt.record_intent(&head, Instant::now()).is_err());
        let db = root.path().join("history-bindings.sqlite");
        let connection = Connection::open(&db).expect("inspect");
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='archived_fork_receipts'",
                [],
                |row| row.get(0),
            )
            .expect("schema");
        assert_eq!(count, 0);
        drop(connection);
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        assert!(receipt.publish(&head, Instant::now()).is_err());
        assert!(load(&head, deadline()).is_err());
        assert!(lock_reserved_identity(&head, receipt.child_incarnation, Instant::now()).is_err());
        receipt.publish(&head, deadline()).expect("publish");
        assert!(load(&head, Instant::now()).is_err());
    }

    #[test]
    fn quarantine_conflict_preserves_both_files_and_retry_handles_existing_alias() {
        for identical in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let (head, receipt, bytes, _) = fixture(root.path());
            receipt.record_intent(&head, deadline()).expect("intent");
            write_synced(&head, &bytes);
            let destination = head.with_extension("fork-unpublished");
            if identical {
                std::fs::hard_link(&head, &destination).expect("interrupted quarantine alias");
            } else {
                write_synced(&destination, b"existing evidence must survive");
            }
            let existing = std::fs::read(&destination).expect("before");
            let result = quarantine_pending(&head, deadline());
            assert_eq!(
                std::fs::read(&destination).expect("evidence retained"),
                existing
            );
            if identical {
                assert_eq!(result.expect("resume"), destination);
                assert!(!head.exists());
            } else {
                assert!(result.is_err());
                assert_eq!(std::fs::read(&head).expect("head retained"), bytes);
            }
            assert!(
                reserve_session_identity(
                    &root.path().join("history-bindings.sqlite"),
                    receipt.child
                )
                .is_err()
            );
        }
    }

    #[test]
    fn quarantine_refuses_duplicate_private_publication_rows() {
        let root = tempfile::tempdir().expect("root");
        let (head, receipt, bytes, _) = fixture(root.path());
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        let db = database(&head, receipt.child).expect("database");
        let connection = Connection::open(&db).expect("connection");
        connection
            .execute_batch(
                "ALTER TABLE archived_fork_receipts RENAME TO original_receipts;
            CREATE TABLE archived_fork_receipts(child TEXT,published INTEGER);
            INSERT INTO archived_fork_receipts SELECT child,0 FROM original_receipts;
            INSERT INTO archived_fork_receipts SELECT child,1 FROM original_receipts;",
            )
            .expect("corrupt duplicate states");
        drop(connection);
        assert!(quarantine_pending(&head, deadline()).is_err());
        assert_eq!(std::fs::read(&head).expect("preserved head"), bytes);
        assert!(!head.with_extension("fork-unpublished").exists());
        crate::identity_custody::require_active_session_identity(
            &db,
            receipt.child,
            receipt.child_incarnation,
        )
        .expect("must not retire ambiguous child");
    }

    #[test]
    fn published_metadata_crash_child() {
        let Some(root) = std::env::var_os("TALOS_METADATA_CRASH_ROOT") else {
            return;
        };
        let root = Path::new(&root);
        let manager = crate::SessionManager::with_dir(root.join("sessions"));
        let source = manager.create_session("crash", "").expect("source");
        source
            .append(&talos_core::message::Message::User {
                content: "durable original".into(),
            })
            .expect("append");
        crate::PendingSubmissionStore::for_session(&source)
            .initialize_runtime_identity(crate::SessionRuntimeIdentity::new(
                "crash-provider",
                "publication-model",
                Some("exact-variant"),
            ))
            .expect("source runtime");
        manager
            .update_index(&source)
            .expect("parent indexed before crash");
        let child_id = Uuid::new_v4();
        let child = crate::Session::new(
            child_id,
            String::new(),
            String::new(),
            source.file_path.with_file_name(format!("{child_id}.tlog")),
        );
        std::fs::write(root.join("parent-id"), source.id.to_string()).expect("parent id");
        std::fs::write(root.join("child-id"), child_id.to_string()).expect("child id");
        let lease = crate::SessionRuntimeLease::acquire(&source.file_path).expect("source lease");
        source
            .publish_history_fork(&child, &lease, None, deadline(), || false)
            .expect("crash before return");
        panic!("crash probe did not run");
    }

    #[test]
    fn process_death_after_receipt_commit_restores_runtime_without_parent() {
        let root = tempfile::tempdir().expect("root");
        let status = std::process::Command::new(std::env::current_exe().expect("executable"))
            .args([
                "--exact",
                "fork_receipt::tests::published_metadata_crash_child",
                "--nocapture",
            ])
            .env("TALOS_METADATA_CRASH_ROOT", root.path())
            .env(
                "TALOS_TEST_FORK_CRASH_PHASE",
                "receipt_committed_before_metadata",
            )
            .status()
            .expect("child process");
        assert_eq!(status.code(), Some(42));
        let manager = crate::SessionManager::with_dir(root.path().join("sessions"));
        let parent_id = std::fs::read_to_string(root.path().join("parent-id")).expect("parent id");
        let child_id = std::fs::read_to_string(root.path().join("child-id")).expect("child id");
        let parent = manager.resume_session(&parent_id).expect("parent");
        let child = manager.resume_session(&child_id).expect("published child");
        let store = crate::PendingSubmissionStore::for_session(&child);
        assert!(!store.path().exists(), "crash precedes child metadata");
        assert_eq!(
            manager
                .reconcile_pending_history_forks(deadline())
                .expect("pending discovery"),
            0
        );
        let bytes = std::fs::read(&child.file_path).expect("child bytes");
        std::fs::remove_file(&parent.file_path).expect("remove fixture parent");
        let parent_store = crate::PendingSubmissionStore::for_session(&parent);
        std::fs::remove_file(parent_store.path()).expect("remove fixture parent identity");
        manager
            .reconcile_index()
            .expect("startup index cleanup before metadata replay");
        let lease = crate::SessionRuntimeLease::acquire(&child.file_path).expect("busy child");
        assert_eq!(
            manager
                .reconcile_published_history_forks(deadline())
                .expect("skip busy"),
            0
        );
        assert!(!store.path().exists());
        drop(lease);
        assert_eq!(
            manager
                .reconcile_published_history_forks(deadline())
                .expect("automatic restore"),
            1
        );
        let lineage = manager.get_forks(&parent_id).expect("lineage");
        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0].forked_session_id, child_id);
        let timestamp = lineage[0].forked_at;
        assert_eq!(
            manager
                .reconcile_published_history_forks(deadline())
                .expect("retry"),
            1
        );
        assert_eq!(
            manager.get_forks(&parent_id).expect("retry lineage")[0].forked_at,
            timestamp
        );
        manager
            .reconcile_index()
            .expect("subsequent index reconciliation");
        assert_eq!(
            manager
                .get_forks(&parent_id)
                .expect("lineage after cleanup")
                .len(),
            1
        );
        assert_eq!(
            store
                .runtime_state()
                .expect("state")
                .expect("runtime")
                .activation
                .target,
            crate::SessionRuntimeIdentity::new(
                "crash-provider",
                "publication-model",
                Some("exact-variant")
            )
        );
        assert_eq!(std::fs::read(&child.file_path).expect("preserved"), bytes);
    }

    #[test]
    fn captured_publisher_crash_child() {
        let Some(root) = std::env::var_os("TALOS_CAPTURED_PUBLISHER_CRASH_ROOT") else {
            return;
        };
        let root = Path::new(&root);
        let (head, parent, bytes, _) = fixture(root);
        parent.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        parent.publish(&head, deadline()).expect("parent publish");
        let source = crate::Session::new(parent.child, String::new(), String::new(), head);
        source.compact_archived(0).expect("compact parent");
        let child = Uuid::new_v4();
        let incarnation = reserve_session_identity(&root.join("history-bindings.sqlite"), child)
            .expect("reserve");
        std::fs::write(root.join("parent-id"), source.id.to_string()).expect("parent handoff");
        std::fs::write(root.join("child-id"), child.to_string()).expect("child handoff");
        let capture = ForkCapture::from_session(
            &source,
            (child, incarnation, Uuid::new_v4()),
            None,
            deadline(),
            || false,
        )
        .expect("capture");
        capture
            .publish(&root.join(format!("{child}.tlog")), deadline(), || false)
            .expect("crash before return");
        panic!("crash probe did not execute");
    }

    #[test]
    fn automatic_pending_discovery_respects_both_runtime_fences_after_process_death() {
        let root = tempfile::tempdir().expect("root");
        let workspace = root.path().join("workspace");
        std::fs::create_dir(&workspace).expect("workspace");
        let status = std::process::Command::new(std::env::current_exe().expect("executable"))
            .args([
                "--exact",
                "fork_receipt::tests::captured_publisher_crash_child",
                "--nocapture",
            ])
            .env("TALOS_CAPTURED_PUBLISHER_CRASH_ROOT", &workspace)
            .env("TALOS_TEST_FORK_CRASH_PHASE", "receipt_child_linked")
            .status()
            .expect("child process");
        assert_eq!(status.code(), Some(42));
        let parent =
            Uuid::parse_str(&std::fs::read_to_string(workspace.join("parent-id")).expect("parent"))
                .expect("uuid");
        let child =
            Uuid::parse_str(&std::fs::read_to_string(workspace.join("child-id")).expect("child"))
                .expect("uuid");
        let head = workspace.join(format!("{child}.tlog"));
        let parent_path = workspace.join(format!("{parent}.tlog"));
        let original = std::fs::read(&parent_path).expect("parent bytes");
        let bytes = std::fs::read(&head).expect("pending bytes");
        let manager = crate::SessionManager::with_dir(root.path().to_path_buf());
        let parent_lease =
            crate::SessionRuntimeLease::acquire(&parent_path).expect("live source fence");
        assert_eq!(
            manager
                .reconcile_pending_history_forks(deadline())
                .expect("skip parent"),
            0
        );
        assert_eq!(std::fs::read(&head).expect("head untouched"), bytes);
        drop(parent_lease);
        let child_lease = crate::SessionRuntimeLease::acquire(&head).expect("live child fence");
        assert_eq!(
            manager
                .reconcile_pending_history_forks(deadline())
                .expect("skip child"),
            0
        );
        assert_eq!(std::fs::read(&head).expect("head untouched"), bytes);
        drop(child_lease);
        assert_eq!(
            manager
                .reconcile_pending_history_forks(deadline())
                .expect("reconcile"),
            1
        );
        assert!(!head.exists());
        assert_eq!(
            std::fs::read(head.with_extension("fork-unpublished")).expect("preserved"),
            bytes
        );
        assert_eq!(
            manager
                .reconcile_pending_history_forks(deadline())
                .expect("retry"),
            0
        );
        assert_eq!(
            std::fs::read(&parent_path).expect("parent untouched"),
            original
        );
        let source = crate::Session::new(parent, String::new(), String::new(), parent_path);
        assert_eq!(
            source
                .inspect_archive_recovery(deadline(), || false)
                .expect("parent recovery")
                .entries()
                .len(),
            3
        );
        // Startup runs orphan maintenance after quarantine/index reconciliation.
        // Use valid SQLite so a database error cannot falsely preserve evidence.
        let sidecar = workspace.join(format!("{child}.pending.sqlite"));
        let wal = workspace.join(format!("{child}.pending.sqlite-wal"));
        let shm = workspace.join(format!("{child}.pending.sqlite-shm"));
        let db = Connection::open(&sidecar).expect("sidecar");
        db.execute_batch(
            "CREATE TABLE evidence(value TEXT); INSERT INTO evidence VALUES('retained');",
        )
        .expect("valid evidence");
        drop(db);
        std::fs::write(&wal, []).expect("wal");
        std::fs::write(&shm, []).expect("shm");
        let paths = [&sidecar, &wal, &shm];
        let original_sidecars: Vec<_> = paths
            .iter()
            .map(|p| std::fs::read(p).expect("original"))
            .collect();
        manager.reconcile_index().expect("startup index");
        let policy = crate::OrphanSidecarReconciliationPolicy {
            minimum_age: std::time::Duration::ZERO,
            ..Default::default()
        };
        let report = manager
            .reconcile_orphan_sidecars(&policy)
            .expect("startup orphan maintenance");
        assert_eq!(report.removed_artifacts, 0);
        assert!(report.failures.is_empty());
        assert_eq!(report.skipped_sets, 1);
        for (path, expected) in paths.iter().zip(original_sidecars) {
            assert_eq!(std::fs::read(path).expect("sidecar retained"), expected);
        }
        // Counterfactual in this disposable fixture: without the marker, the
        // same healthy sidecar set is actually eligible for orphan cleanup.
        std::fs::rename(
            head.with_extension("fork-unpublished"),
            head.with_extension("preserved-evidence"),
        )
        .expect("move fixture marker");
        let control = manager
            .reconcile_orphan_sidecars(&policy)
            .expect("control cleanup");
        assert!(control.failures.is_empty());
        assert_eq!(control.removed_artifacts, 3);
        assert_eq!(
            std::fs::read(head.with_extension("preserved-evidence"))
                .expect("fork evidence retained"),
            bytes
        );
        assert!(
            reserve_session_identity(&workspace.join("history-bindings.sqlite"), child).is_err()
        );
    }

    #[test]
    fn crash_after_child_link_quarantine_preserves_bytes_and_restores_parent() {
        let root = tempfile::tempdir().expect("root");
        let status = std::process::Command::new(std::env::current_exe().expect("executable"))
            .args([
                "--exact",
                "fork_receipt::tests::captured_publisher_crash_child",
                "--nocapture",
            ])
            .env("TALOS_CAPTURED_PUBLISHER_CRASH_ROOT", root.path())
            .env("TALOS_TEST_FORK_CRASH_PHASE", "receipt_child_linked")
            .status()
            .expect("real child process");
        assert_eq!(status.code(), Some(42));
        let parent = Uuid::parse_str(
            &std::fs::read_to_string(root.path().join("parent-id")).expect("parent"),
        )
        .expect("uuid");
        let child =
            Uuid::parse_str(&std::fs::read_to_string(root.path().join("child-id")).expect("child"))
                .expect("uuid");
        let head = root.path().join(format!("{child}.tlog"));
        let bytes = std::fs::read(&head).expect("pending bytes");
        let source = crate::Session::new(
            parent,
            String::new(),
            String::new(),
            root.path().join(format!("{parent}.tlog")),
        );
        let original = std::fs::read(&source.file_path).expect("parent bytes");
        assert!(
            source
                .inspect_archive_recovery(deadline(), || false)
                .is_err(),
            "pending sibling is unavailable"
        );
        let child_session = crate::Session::new(child, String::new(), String::new(), head.clone());
        let child_lease =
            crate::SessionRuntimeLease::acquire(&head).expect("child recovery ownership");
        let preserved = child_session
            .quarantine_pending_history_fork(&child_lease, deadline())
            .expect("reconcile pending");
        assert!(!head.exists());
        assert_eq!(
            std::fs::read(&preserved).expect("preserved evidence"),
            bytes
        );
        assert_eq!(
            quarantine_pending(&head, deadline()).expect("idempotent"),
            preserved
        );
        assert_eq!(
            std::fs::read(&source.file_path).expect("parent unchanged"),
            original
        );
        assert_eq!(
            source
                .inspect_archive_recovery(deadline(), || false)
                .expect("parent restored")
                .entries()
                .len(),
            3
        );
        assert!(
            reserve_session_identity(&root.path().join("history-bindings.sqlite"), child).is_err()
        );
        assert!(
            quarantine_pending(&source.file_path, deadline()).is_err(),
            "published parent cannot be retired here"
        );
        assert!(
            source
                .inspect_archive_recovery(deadline(), || false)
                .is_ok()
        );
    }

    #[test]
    fn fork_receipt_crash_child() {
        let Some(root) = std::env::var_os("TALOS_FORK_RECEIPT_CRASH_ROOT") else {
            return;
        };
        let root = Path::new(&root);
        let (head, receipt, bytes, _) = fixture(root);
        receipt.record_intent(&head, deadline()).expect("intent");
        write_synced(&head, &bytes);
        std::fs::write(root.join("child-id"), receipt.child.to_string()).expect("handoff");
        if std::env::var_os("TALOS_FORK_RECEIPT_CRASH_AFTER_COMMIT").is_some() {
            receipt.publish(&head, deadline()).expect("commit");
        }
        std::process::exit(73);
    }

    #[test]
    fn process_death_preserves_pending_and_committed_publication_states() {
        for committed in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let mut command =
                std::process::Command::new(std::env::current_exe().expect("test executable"));
            command
                .args([
                    "--exact",
                    "fork_receipt::tests::fork_receipt_crash_child",
                    "--nocapture",
                ])
                .env("TALOS_FORK_RECEIPT_CRASH_ROOT", root.path())
                .env_remove("TALOS_FORK_RECEIPT_CRASH_AFTER_COMMIT");
            if committed {
                command.env("TALOS_FORK_RECEIPT_CRASH_AFTER_COMMIT", "1");
            }
            let status = command.status().expect("child");
            assert_eq!(status.code(), Some(73));
            let id = std::fs::read_to_string(root.path().join("child-id")).expect("child id");
            let head = root.path().join(format!("{id}.tlog"));
            let before = std::fs::read(&head).expect("head retained");
            assert_eq!(load(&head, deadline()).is_ok(), committed);
            assert_eq!(crate::history_page::open_source(&head).is_ok(), committed);
            assert_eq!(std::fs::read(&head).expect("no read mutation"), before);
            let id = Uuid::parse_str(&id).expect("uuid");
            retire_session_identity(&root.path().join("history-bindings.sqlite"), id)
                .expect("restart retirement");
            assert!(
                reserve_session_identity(&root.path().join("history-bindings.sqlite"), id).is_err()
            );
        }
    }
}
