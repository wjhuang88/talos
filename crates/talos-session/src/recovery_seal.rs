//! Incremental immutable-prefix verification for storage-issued recovery sources.

use crate::{DurableRecoverySource, history_page::HistoryPageError};
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Incremental source scan. Each step reads at most one bounded history page.
/// This is content consistency evidence, not branch or permission authority.
pub struct RecoveryPrefixScan {
    source: DurableRecoverySource,
    stamp: crate::recovery_stamp::FileStamp,
    offset: u64,
    pending_page: Vec<u8>,
    digest: [u8; 32],
    expected: Option<[u8; 32]>,
    // A released Windows lease requires bounded content revalidation before reuse.
    revalidate: Option<(u64, [u8; 32])>,
}

// Trusted bindings provenance only. Never deserialize this from the search index
// or model input. Fields cannot be supplied through the public API (ADR-096).
#[cfg(any(unix, windows))]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredScan {
    save_id: uuid::Uuid,
    version: u8,
    session: uuid::Uuid,
    branch: String,
    cutoff: u64,
    offset: u64,
    digest: [u8; 32],
    expected: Option<[u8; 32]>,
    stamp: crate::recovery_stamp::StoredFileVersion,
}

#[cfg(any(unix, windows))]
impl RecoveryPrefixScan {
    pub(crate) fn stored(
        &self,
        owner: &crate::DurableSession,
    ) -> Result<StoredScan, HistoryPageError> {
        if !owner.owns_recovery_source(&self.source) {
            return Err(HistoryPageError::Stale);
        }
        self.check_stamp()?;
        if self.revalidate.is_some() {
            return Err(HistoryPageError::Stale);
        }
        Ok(StoredScan {
            save_id: uuid::Uuid::new_v4(),
            version: 1,
            session: self.source.session_id(),
            branch: self.source.branch_id().into(),
            cutoff: self.source.source_end(),
            offset: self.offset,
            digest: self.digest,
            expected: self.expected,
            stamp: (&self.stamp).into(),
        })
    }
}

#[cfg(any(unix, windows))]
impl StoredScan {
    pub(crate) fn supersedes(&self, previous: &Self) -> bool {
        self.stamp != previous.stamp
            || (self.session == previous.session
                && self.branch == previous.branch
                && self.cutoff == previous.cutoff
                && self.expected == previous.expected
                && self.offset >= previous.offset)
    }

    pub(crate) fn restore(
        self,
        source: DurableRecoverySource,
    ) -> Result<RecoveryPrefixScan, HistoryPageError> {
        self.restore_checked(source, cfg!(windows))
    }

    fn restore_checked(
        self,
        source: DurableRecoverySource,
        revalidate_content: bool,
    ) -> Result<RecoveryPrefixScan, HistoryPageError> {
        if self.version != 1
            || self.session != source.session_id()
            || self.branch != source.branch_id()
            || self.cutoff != source.source_end()
            || self.offset > self.cutoff
            || (self.offset != self.cutoff
                && !self
                    .offset
                    .is_multiple_of(crate::history_page::HISTORY_PAGE_BYTES as u64))
            || (self.offset == 0 && self.digest != [0; 32])
        {
            return Err(HistoryPageError::Stale);
        }
        let stamp = source.recovery_metadata()?;
        if crate::recovery_stamp::StoredFileVersion::from(&stamp) != self.stamp {
            return Err(HistoryPageError::Stale);
        }
        let scan = RecoveryPrefixScan {
            source,
            stamp,
            offset: if revalidate_content { 0 } else { self.offset },
            pending_page: Vec::new(),
            digest: if revalidate_content {
                [0; 32]
            } else {
                self.digest
            },
            expected: self.expected,
            revalidate: (revalidate_content && self.offset != 0)
                .then_some((self.offset, self.digest)),
        };
        scan.check_stamp()?;
        Ok(scan)
    }
}

/// Persistable consistency expectation for one previously verified raw prefix.
/// This is untrusted cache metadata, never branch or permission authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecoveryPrefixExpectation {
    version: u8,
    cutoff: u64,
    digest: [u8; 32],
}

impl RecoveryPrefixExpectation {
    /// Captured byte cutoff; not authorization to access a source.
    pub fn cutoff(&self) -> u64 {
        self.cutoff
    }
}

/// Storage-issued rolling digest for all raw bytes of one captured prefix. Later suffix appends
/// are compatible; replacing any covered record invalidates verification.
#[derive(Clone)]
pub struct RecoveryPrefixSeal {
    source: DurableRecoverySource,
    stamp: crate::recovery_stamp::FileStamp,
    digest: [u8; 32],
}

impl DurableRecoverySource {
    /// Revalidate an untrusted persisted prefix expectation against this configured
    /// storage source. Only a completed scan grants content-consistency evidence;
    /// the caller still binds owner, branch and permission independently.
    pub fn verify_prefix_expectation(
        &self,
        expectation: &RecoveryPrefixExpectation,
    ) -> Result<RecoveryPrefixScan, HistoryPageError> {
        if expectation.version != 1 || expectation.cutoff > self.source_end() {
            return Err(HistoryPageError::Stale);
        }
        let source = self.recovery_prefix_at(expectation.cutoff)?;
        let mut scan = source.begin_prefix_seal()?;
        scan.expected = Some(expectation.digest);
        Ok(scan)
    }

    /// Begin incrementally sealing this source's captured prefix.
    /// No source content is read until the first step.
    pub fn begin_prefix_seal(&self) -> Result<RecoveryPrefixScan, HistoryPageError> {
        Ok(RecoveryPrefixScan {
            source: self.clone(),
            stamp: self.recovery_metadata()?,
            offset: 0,
            pending_page: Vec::new(),
            digest: [0; 32],
            expected: None,
            revalidate: None,
        })
    }
}

impl RecoveryPrefixSeal {
    /// Copy this fully verified prefix into a host-owned staging writer.
    ///
    /// Uses at most 64 KiB per read and rechecks source identity between chunks.
    /// This does not publish a fork or grant permission to write: the host must
    /// hold its admission gate and discard the staging output on any error.
    /// The writer can contain a partial prefix on failure. Kernel I/O is synchronous;
    /// cancellation and deadlines are cooperative between reads and writes.
    pub fn copy_to_staging(
        &self,
        writer: &mut impl std::io::Write,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<u64, HistoryPageError> {
        self.check_current()?;
        let mut offset = 0_u64;
        while offset < self.source.source_end() {
            if stopped() || Instant::now() >= deadline {
                return Err(HistoryPageError::Stopped);
            }
            let end = offset
                .saturating_add(64 * 1024)
                .min(self.source.source_end());
            let bytes =
                self.source
                    .read_recovery_bytes(offset, end, &self.stamp, deadline, &stopped)?;
            writer.write_all(&bytes)?;
            #[cfg(test)]
            crate::durable::fork_crash_probe("midcopy");
            self.check_current()?;
            offset = end;
        }
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        self.check_current()?;
        Ok(offset)
    }

    /// Export bounded cache metadata for a future full source revalidation.
    pub fn expectation(&self) -> RecoveryPrefixExpectation {
        RecoveryPrefixExpectation {
            version: 1,
            cutoff: self.source.source_end(),
            digest: self.digest,
        }
    }

    /// Begin a bounded-page verification of the entire captured prefix.
    /// The scan must finish before this evidence can authorize a later read.
    pub fn begin_verification(&self) -> Result<RecoveryPrefixScan, HistoryPageError> {
        let mut scan = self.source.begin_prefix_seal()?;
        scan.expected = Some(self.digest);
        Ok(scan)
    }

    /// Read one exact original record only while the fully verified file version
    /// remains current. Appends/replacements require another completed verification.
    /// Neither the seal nor a record locator establishes branch/permission authority.
    pub fn read_record(
        &self,
        range: std::ops::Range<u64>,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<crate::SessionEntry, HistoryPageError> {
        if range.start >= range.end || range.end > self.source.source_end() {
            return Err(HistoryPageError::Stale);
        }
        if range.end - range.start > crate::history_page::HISTORY_PAGE_BYTES as u64 {
            return Err(HistoryPageError::RecordTooLarge);
        }
        self.check_current()?;
        let bytes = self.source.read_recovery_bytes(
            range.start,
            range.end,
            &self.stamp,
            deadline,
            &stopped,
        )?;
        let (entry, consumed) =
            crate::history_page::tlog_record(&bytes)?.ok_or(HistoryPageError::InvalidRecord)?;
        if consumed != bytes.len() {
            return Err(HistoryPageError::InvalidRecord);
        }
        self.check_current()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        Ok(entry)
    }

    /// Check that the verified file version has not changed. Call again inside
    /// the host's final permission/generation delivery fence.
    pub fn check_current(&self) -> Result<(), HistoryPageError> {
        if !same_file_version(&self.source.recovery_metadata()?, &self.stamp)? {
            return Err(HistoryPageError::Stale);
        }
        Ok(())
    }

    /// Captured source cutoff, unaffected by later append-only writes.
    pub fn source_end(&self) -> u64 {
        self.source.source_end()
    }
}

impl RecoveryPrefixScan {
    /// Whether this scan is verifying exactly the supplied cache expectation.
    /// Matching does not certify completion or authorize history queries.
    pub fn verifies_expectation(&self, expectation: &RecoveryPrefixExpectation) -> bool {
        expectation.version == 1
            && self.source.source_end() == expectation.cutoff
            && self.expected == Some(expectation.digest)
    }

    /// Number of bytes committed to complete-page scan progress.
    /// Partial page bytes do not count and cannot authorize a query.
    pub fn completed_bytes(&self) -> u64 {
        if self.revalidate.is_some() {
            return 0;
        }
        self.offset
    }

    /// Read at most 64 KiB, retaining at most one 2 MiB page between steps.
    /// Completed pages extend the version-one rolling digest. Verification
    /// compares the digest only at completion, never certifies a partial page.
    /// Cancellation/deadline do not advance the committed offset or digest;
    /// a verified pending page may already have grown before the final check.
    /// Source changes require discarding the scan. A host may resume a deadline
    /// interruption, but must discard an operation it has cancelled.
    pub fn step(
        &mut self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<bool, HistoryPageError> {
        self.check_stamp()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        if self.offset == self.source.source_end() {
            return Ok(true);
        }
        // Preserve verified sub-page reads across cooperative budget exhaustion,
        // while retaining the existing fixed-page digest format.
        let page_end = self.source.source_end().min(
            self.offset
                .saturating_add(crate::history_page::HISTORY_PAGE_BYTES as u64),
        );
        let read_offset = self.offset + self.pending_page.len() as u64;
        let read_end = page_end.min(read_offset.saturating_add(64 * 1024));
        let bytes = self.source.read_recovery_bytes(
            read_offset,
            read_end,
            &self.stamp,
            deadline,
            &stopped,
        )?;
        self.check_stamp()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        self.pending_page.extend_from_slice(&bytes);
        if read_end < page_end {
            return Ok(false);
        }
        let next_offset = page_end;
        let mut hash = Sha256::new();
        hash.update(self.digest);
        hash.update(self.offset.to_le_bytes());
        hash.update(next_offset.to_le_bytes());
        hash.update(&self.pending_page);
        let digest: [u8; 32] = hash.finalize().into();
        let complete = next_offset == self.source.source_end();
        if self
            .revalidate
            .is_some_and(|(end, expected)| next_offset == end && digest != expected)
        {
            return Err(HistoryPageError::Stale);
        }
        if complete && self.expected.is_some_and(|expected| expected != digest) {
            return Err(HistoryPageError::Stale);
        }
        self.check_stamp()?;
        if stopped() || Instant::now() >= deadline {
            return Err(HistoryPageError::Stopped);
        }
        if self.revalidate.is_some_and(|(end, _)| next_offset == end) {
            self.revalidate = None;
        }
        self.digest = digest;
        self.offset = next_offset;
        self.pending_page.clear();
        Ok(complete)
    }

    /// Finish only after every covered page has been read successfully.
    /// Source stability is checked again; no partial digest grants evidence.
    pub fn finish(self) -> Result<RecoveryPrefixSeal, HistoryPageError> {
        self.check_stamp()?;
        if self.revalidate.is_some()
            || self.offset != self.source.source_end()
            || self
                .expected
                .as_ref()
                .is_some_and(|expected| expected != &self.digest)
        {
            return Err(HistoryPageError::Stale);
        }
        Ok(RecoveryPrefixSeal {
            source: self.source,
            stamp: self.stamp,
            digest: self.digest,
        })
    }

    fn check_stamp(&self) -> Result<(), HistoryPageError> {
        let current = self.source.recovery_metadata()?;
        if !same_file_version(&current, &self.stamp)? {
            return Err(HistoryPageError::Stale);
        }
        Ok(())
    }
}

// Platforms without stable file identity/change markers cannot mint this evidence.
#[cfg(unix)]
pub(crate) fn same_file_version(
    a: &std::fs::Metadata,
    b: &std::fs::Metadata,
) -> Result<bool, HistoryPageError> {
    Ok(crate::recovery_stamp::StoredFileVersion::from(a)
        == crate::recovery_stamp::StoredFileVersion::from(b))
}
#[cfg(windows)]
pub(crate) fn same_file_version(
    a: &crate::recovery_stamp::FileStamp,
    b: &crate::recovery_stamp::FileStamp,
) -> Result<bool, HistoryPageError> {
    // Both stamps retain restrictive read leases minted only from validated
    // local SourceFile handles (ADR-096). Identity/timestamps are compared while
    // those leases exclude writers and deletion, not as standalone authority.
    Ok(a == b)
}
#[cfg(not(any(unix, windows)))]
pub(crate) fn same_file_version(
    _: &crate::recovery_stamp::FileStamp,
    _: &crate::recovery_stamp::FileStamp,
) -> Result<bool, HistoryPageError> {
    Err(HistoryPageError::UnsupportedSource)
}

#[cfg(all(test, any(unix, windows)))]
mod process_resume_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn restored_prefix_revalidation_refuses_equal_metadata_content_change() {
        use std::io::{Seek, Write};
        for pages in [1u64, 2, 3] {
            for changed in [false, true] {
                let root = tempfile::tempdir().expect("root");
                let owner = crate::DurableSession::open_or_create(root.path(), "revalidate")
                    .expect("owner");
                owner
                    .commit_turn(
                        "seed",
                        &[talos_core::message::Message::User {
                            content: "fact ".repeat(1_000_000),
                        }],
                        &Default::default(),
                    )
                    .expect("seed");
                let mut scan = owner
                    .capture_recovery_source()
                    .expect("source")
                    .begin_prefix_seal()
                    .expect("scan");
                let deadline = Instant::now() + Duration::from_secs(10);
                let target = (pages * crate::history_page::HISTORY_PAGE_BYTES as u64)
                    .min(scan.source.source_end());
                while scan.completed_bytes() < target {
                    scan.step(deadline, || false).expect("page");
                }
                let mut stored = scan.stored(&owner).expect("stored");
                let saved_offset = stored.offset;
                drop(scan);
                if changed {
                    let mut writer = std::fs::OpenOptions::new()
                        .write(true)
                        .open(owner.file_path())
                        .expect("writer");
                    writer.seek(std::io::SeekFrom::Start(128)).expect("seek");
                    writer.write_all(b"DIFFERENT").expect("overwrite same size");
                    writer.sync_all().expect("sync");
                }
                let source = owner.capture_recovery_source().expect("fresh lease");
                // Deterministically model the observed Windows metadata collision on any host.
                stored.stamp = (&source.recovery_metadata().expect("metadata")).into();
                let mut resumed = stored
                    .restore_checked(source, true)
                    .expect("pending verification");
                assert_eq!(resumed.completed_bytes(), 0);
                assert!(
                    resumed.stored(&owner).is_err(),
                    "unverified offset cannot replace saved progress"
                );
                assert!(resumed.step(deadline, || true).is_err());
                assert_eq!(resumed.completed_bytes(), 0);
                let mut refused = false;
                while resumed.completed_bytes() == 0 {
                    match resumed.step(deadline, || false) {
                        Ok(false) => {}
                        Ok(true) => {
                            assert_eq!(saved_offset, resumed.source.source_end());
                            break;
                        }
                        Err(HistoryPageError::Stale) => {
                            refused = true;
                            break;
                        }
                        Err(error) => panic!("unexpected {error}"),
                    }
                }
                assert_eq!(refused, changed);
                if changed {
                    assert!(resumed.finish().is_err(), "mismatch cannot mint a seal");
                } else {
                    assert_eq!(resumed.completed_bytes(), saved_offset);
                    while !resumed.step(deadline, || false).expect("remaining page") {}
                    resumed.finish().expect("fully verified source");
                }
            }
        }
    }
    #[test]
    fn saved_receipt_cleans_after_source_change_without_deleting_a_resave() {
        let root = tempfile::tempdir().expect("root");
        let owner = crate::DurableSession::open_or_create(root.path(), "receipt").expect("owner");
        owner
            .commit_turn(
                "seed",
                &[talos_core::message::Message::User {
                    content: "fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let scan = owner
            .capture_recovery_source()
            .expect("source")
            .begin_prefix_seal()
            .expect("scan");
        let budget = crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2));
        let first = owner.save_recovery_scan(&scan, &budget).expect("save");
        let second = owner
            .save_recovery_scan(&scan, &budget)
            .expect("same offset resave");
        assert!(
            !owner
                .clear_recovery_scan(&first, &budget)
                .expect("obsolete receipt")
        );
        let foreign = crate::DurableSession::open_existing(root.path(), "receipt")
            .expect("open")
            .expect("foreign handle");
        assert!(foreign.clear_recovery_scan(&second, &budget).is_err());
        drop(scan); // Release the Windows source lease before the host appends.
        owner
            .commit_turn(
                "later",
                &[talos_core::message::Message::User {
                    content: "new suffix".into(),
                }],
                &Default::default(),
            )
            .expect("append");
        assert!(
            owner
                .clear_recovery_scan(&second, &budget)
                .expect("cleanup after source changed")
        );
        assert!(
            !owner
                .clear_recovery_scan(&second, &budget)
                .expect("cleanup idempotent")
        );
    }

    #[test]
    fn saved_scan_rejects_foreign_owner_and_corrupt_payload() {
        let root = tempfile::tempdir().expect("root");
        let owner = crate::DurableSession::open_or_create(root.path(), "owner").expect("owner");
        let foreign =
            crate::DurableSession::open_or_create(root.path(), "foreign").expect("foreign");
        let budget = || crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(5));
        let source = owner.capture_recovery_source().expect("source");
        let scan = source.begin_prefix_seal().expect("scan");
        assert!(foreign.save_recovery_scan(&scan, &budget()).is_err());
        assert!(
            foreign
                .restore_recovery_scan_for_source(&source, &budget())
                .is_err()
        );
        let receipt = owner.save_recovery_scan(&scan, &budget()).expect("save");
        assert!(foreign.clear_recovery_scan(&receipt, &budget()).is_err());
        let cancelled = budget();
        cancelled.cancel();
        assert!(owner.restore_recovery_scan(&cancelled).is_err());
        assert!(owner.clear_recovery_scan(&receipt, &cancelled).is_err());
        assert!(
            owner
                .restore_recovery_scan(&budget())
                .expect("uncancelled restore")
                .is_some(),
            "cancelled restore/cleanup must retain saved progress"
        );
        drop(scan);
        drop(source);
        let db = rusqlite::Connection::open(root.path().join("durable-bindings.sqlite"))
            .expect("trusted db fixture");
        let payload: String = db
            .query_row(
                "SELECT payload FROM durable_recovery_scans WHERE session_id=?1",
                [owner.id().to_string()],
                |row| row.get(0),
            )
            .expect("payload");
        for field in ["version", "session", "branch", "cutoff", "offset", "digest"] {
            let mut value: serde_json::Value = serde_json::from_str(&payload).expect("json");
            value[field] = match field {
                "session" => serde_json::json!(foreign.id()),
                "branch" => serde_json::json!("foreign-branch"),
                "digest" => serde_json::json!(vec![9u8; 32]),
                _ => serde_json::json!(999999),
            };
            db.execute(
                "UPDATE durable_recovery_scans SET payload=?1 WHERE session_id=?2",
                rusqlite::params![value.to_string(), owner.id().to_string()],
            )
            .expect("corrupt");
            assert!(
                owner
                    .restore_recovery_scan(&budget())
                    .expect("restore")
                    .is_none(),
                "{field}"
            );
        }
        db.execute(
            "UPDATE durable_recovery_scans SET payload='invalid-json' WHERE session_id=?1",
            [owner.id().to_string()],
        )
        .expect("malformed");
        assert!(
            owner
                .restore_recovery_scan(&budget())
                .expect("malformed restore")
                .is_none()
        );
    }

    #[cfg(windows)]
    #[test]
    fn restored_scan_retains_new_lease_until_dropped() {
        let root = tempfile::tempdir().expect("root");
        let owner =
            crate::DurableSession::open_or_create(root.path(), "restored-lease").expect("owner");
        let budget = || crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(5));
        {
            let scan = owner
                .capture_recovery_source()
                .expect("source")
                .begin_prefix_seal()
                .expect("scan");
            owner.save_recovery_scan(&scan, &budget()).expect("save");
        }
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(owner.file_path())
            .expect("old leases released");
        drop(writer);
        let resumed = owner
            .restore_recovery_scan(&budget())
            .expect("restore")
            .expect("scan");
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open(owner.file_path())
                .is_err(),
            "new scan must exclude a writer"
        );
        drop(resumed);
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(owner.file_path())
            .expect("new lease released");
        assert!(
            !matches!(owner.restore_recovery_scan(&budget()), Ok(Some(_))),
            "sharing conflict cannot reuse saved offset"
        );
        drop(writer);
    }

    #[test]
    fn saved_cursor_rejects_mutations_after_all_source_leases_are_dropped() {
        use std::io::{Seek, SeekFrom, Write};
        for mutation in ["append", "overwrite", "truncate", "replace"] {
            let root = tempfile::tempdir().expect("root");
            let owner =
                crate::DurableSession::open_or_create(root.path(), "mutation").expect("owner");
            owner
                .commit_turn(
                    "seed",
                    &[talos_core::message::Message::User {
                        content: "fact ".repeat(600_000),
                    }],
                    &Default::default(),
                )
                .expect("commit");
            let budget = || crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(5));
            {
                let mut scan = owner
                    .capture_recovery_source()
                    .expect("source")
                    .begin_prefix_seal()
                    .expect("scan");
                while scan.offset == 0 {
                    assert!(
                        !scan
                            .step(Instant::now() + Duration::from_secs(5), || false)
                            .expect("step")
                    );
                }
                owner.save_recovery_scan(&scan, &budget()).expect("save");
            }
            let path = owner.file_path();
            let original = std::fs::read(path).expect("bytes");
            match mutation {
                "append" => std::fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .expect("writer")
                    .write_all(b"\n")
                    .expect("append"),
                "overwrite" => {
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .open(path)
                        .expect("writer");
                    file.seek(SeekFrom::Start(128)).expect("seek");
                    file.write_all(b"X").expect("write");
                    file.sync_all().expect("flush");
                }
                "truncate" => {
                    let mut bytes = original.clone();
                    let index = 128;
                    bytes[index] ^= 1;
                    std::fs::write(path, bytes).expect("truncate and rewrite");
                }
                "replace" => {
                    std::fs::remove_file(path).expect("old lease released");
                    std::fs::write(path, &original).expect("same bytes new identity");
                }
                _ => unreachable!(),
            }
            let restored = owner
                .restore_recovery_scan(&budget())
                .expect("restore attempt");
            #[cfg(unix)]
            assert!(
                restored.is_none(),
                "Unix metadata change must refuse immediately"
            );
            #[cfg(windows)]
            if let Some(mut resumed) = restored {
                assert_eq!(resumed.scan.completed_bytes(), 0);
                loop {
                    match resumed
                        .scan
                        .step(Instant::now() + Duration::from_secs(5), || false)
                    {
                        Ok(false) => {}
                        Ok(true) => panic!("{mutation} must not certify the changed prefix"),
                        Err(HistoryPageError::Stale) => break,
                        Err(error) => panic!("unexpected {error}"),
                    }
                }
                assert!(resumed.scan.finish().is_err());
            }
        }
    }

    #[test]
    fn saved_prefix_resumes_in_new_process_without_old_leases() {
        const ROOT: &str = "TALOS_TEST_SAVED_PREFIX_ROOT";
        let budget = || crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(5));
        if let Some(root) = std::env::var_os(ROOT) {
            let owner =
                crate::DurableSession::open_existing(std::path::Path::new(&root), "scan-process")
                    .expect("open")
                    .expect("persisted owner");
            let mut resumed = owner
                .restore_recovery_scan(&budget())
                .expect("restore")
                .expect("saved progress");
            if cfg!(windows) {
                assert_eq!(
                    resumed.scan.completed_bytes(),
                    0,
                    "fresh lease must revalidate saved prefix"
                );
                assert_eq!(
                    resumed.scan.revalidate.map(|(offset, _)| offset),
                    Some(crate::history_page::HISTORY_PAGE_BYTES as u64)
                );
            } else {
                assert_eq!(
                    resumed.scan.offset,
                    crate::history_page::HISTORY_PAGE_BYTES as u64
                );
            }
            while !resumed
                .scan
                .step(Instant::now() + Duration::from_secs(5), || false)
                .expect("page")
            {}
            let seal = resumed.scan.finish().expect("complete seal");
            drop(seal);
            assert!(
                owner
                    .clear_recovery_scan(&resumed.receipt, &budget())
                    .expect("clear")
            );
            println!("SAVED_PREFIX_PROCESS_VERIFIED");
            return;
        }
        let root = tempfile::tempdir().expect("root");
        {
            let owner =
                crate::DurableSession::open_or_create(root.path(), "scan-process").expect("owner");
            owner
                .commit_turn(
                    "seed",
                    &[talos_core::message::Message::User {
                        content: "fact ".repeat(600_000),
                    }],
                    &Default::default(),
                )
                .expect("commit");
            let mut scan = owner
                .capture_recovery_source()
                .expect("source")
                .begin_prefix_seal()
                .expect("scan");
            while scan.offset == 0 {
                assert!(
                    !scan
                        .step(Instant::now() + Duration::from_secs(5), || false)
                        .expect("step")
                );
            }
            assert_eq!(scan.offset, crate::history_page::HISTORY_PAGE_BYTES as u64);
            owner
                .save_recovery_scan(&scan, &budget())
                .expect("save nonzero cursor");
        } // owner, scan, source and all Windows leases are gone before spawn
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", "recovery_seal::process_resume_tests::saved_prefix_resumes_in_new_process_without_old_leases", "--nocapture"])
            .env(ROOT, root.path()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
            .spawn().expect("child");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if child.try_wait().expect("wait").is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("child recovery deadline");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().expect("output");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("SAVED_PREFIX_PROCESS_VERIFIED"));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::{DurableSession, PersistencePolicy, TurnTranscriptOutcome};
    use std::time::Duration;
    use talos_core::message::Message;

    #[test]
    fn staging_copy_preserves_raw_prefix_and_refuses_mutation_or_cancel() {
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "fork-copy").expect("owner");
        owner
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "fact ".repeat(30_000),
                }],
                &Default::default(),
            )
            .expect("seed");
        let mut scan = owner
            .capture_recovery_source()
            .expect("source")
            .begin_prefix_seal()
            .expect("scan");
        let deadline = Instant::now() + Duration::from_secs(5);
        while !scan.step(deadline, || false).expect("scan step") {}
        let seal = scan.finish().expect("seal");
        let original = std::fs::read(owner.file_path()).expect("original");
        let mut copied = Vec::new();
        assert_eq!(
            seal.copy_to_staging(&mut copied, deadline, || false)
                .expect("copy"),
            original.len() as u64
        );
        assert_eq!(copied, original);
        let mut cancelled = Vec::new();
        assert!(matches!(
            seal.copy_to_staging(&mut cancelled, deadline, || true),
            Err(HistoryPageError::Stopped)
        ));
        assert!(cancelled.is_empty());
        struct MutatingWriter<'a> {
            owner: &'a DurableSession,
            wrote: bool,
        }
        impl std::io::Write for MutatingWriter<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                assert!(!self.wrote, "source mutation must stop before second chunk");
                self.wrote = true;
                self.owner
                    .commit_turn(
                        "append",
                        &[Message::User {
                            content: "new".into(),
                        }],
                        &Default::default(),
                    )
                    .expect("append");
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut writer = MutatingWriter {
            owner: &owner,
            wrote: false,
        };
        assert!(matches!(
            seal.copy_to_staging(&mut writer, deadline, || false),
            Err(HistoryPageError::Stale)
        ));
        assert!(writer.wrote);
    }

    #[test]
    fn resumed_verification_requires_exact_expectation() {
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "expectation").expect("owner");
        let source = owner.capture_recovery_source().expect("source");
        let expectation = RecoveryPrefixExpectation {
            version: 1,
            cutoff: source.source_end(),
            digest: [7; 32],
        };
        let initial = source.begin_prefix_seal().expect("initial");
        assert!(!initial.verifies_expectation(&expectation));
        let scan = source
            .verify_prefix_expectation(&expectation)
            .expect("verify");
        assert!(scan.verifies_expectation(&expectation));
        let mut changed = expectation.clone();
        changed.digest[0] ^= 1;
        assert!(!scan.verifies_expectation(&changed));
        changed = expectation.clone();
        changed.cutoff += 1;
        assert!(!scan.verifies_expectation(&changed));
        changed = expectation;
        changed.version += 1;
        assert!(!scan.verifies_expectation(&changed));
    }

    #[test]
    fn reopened_owners_cannot_regress_committed_scan_progress() {
        let root = tempfile::tempdir().expect("root");
        let first = DurableSession::open_or_create(root.path(), "owners").expect("first");
        first
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "fact ".repeat(600_000),
                }],
                &Default::default(),
            )
            .expect("seed");
        let second = DurableSession::open_existing(root.path(), "owners")
            .expect("open")
            .expect("second");
        let older = first
            .capture_recovery_source()
            .expect("older")
            .begin_prefix_seal()
            .expect("scan");
        let mut newer = second
            .capture_recovery_source()
            .expect("newer")
            .begin_prefix_seal()
            .expect("scan");
        for _ in 0..32 {
            newer
                .step(Instant::now() + Duration::from_secs(2), || false)
                .expect("advance");
        }
        let (saved, wait_for_save) = std::sync::mpsc::channel();
        let old_writer = std::thread::spawn(move || {
            wait_for_save
                .recv_timeout(Duration::from_secs(5))
                .expect("new owner committed");
            first
                .save_recovery_scan(
                    &older,
                    &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2)),
                )
                .is_err()
        });
        let new_writer = std::thread::spawn(move || {
            second
                .save_recovery_scan(
                    &newer,
                    &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2)),
                )
                .expect("new owner save");
            saved.send(()).expect("release old owner");
        });
        new_writer.join().expect("new writer");
        assert!(
            old_writer.join().expect("old writer"),
            "old owner must not overwrite newer page"
        );
        let observer = DurableSession::open_existing(root.path(), "owners")
            .expect("observer")
            .expect("bound");
        assert_eq!(
            observer
                .restore_recovery_scan(&crate::sqlite::RecoveryWorkBudget::new(
                    Duration::from_secs(2)
                ))
                .expect("restore")
                .expect("progress")
                .scan
                .completed_bytes(),
            2 * 1024 * 1024
        );
    }

    #[test]
    fn restore_never_substitutes_new_cutoff_for_captured_source() {
        let root = tempfile::tempdir().expect("root");
        let owner = DurableSession::open_or_create(root.path(), "capture-race").expect("owner");
        owner
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "old".into(),
                }],
                &Default::default(),
            )
            .expect("seed");
        let original = owner.capture_recovery_source().expect("original cutoff");
        owner
            .commit_turn(
                "append",
                &[Message::User {
                    content: "new".into(),
                }],
                &Default::default(),
            )
            .expect("append");
        let concurrent = DurableSession::open_existing(root.path(), "capture-race")
            .expect("open")
            .expect("concurrent owner");
        let fresh = concurrent.capture_recovery_source().expect("new cutoff");
        assert!(fresh.source_end() > original.source_end());
        let scan = fresh.begin_prefix_seal().expect("scan");
        let budget = crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2));
        concurrent
            .save_recovery_scan(&scan, &budget)
            .expect("new version progress");
        assert!(
            owner
                .restore_recovery_scan_for_source(&original, &budget)
                .expect("old cutoff refuses")
                .is_none()
        );
        assert!(
            owner
                .restore_recovery_scan_for_source(&fresh, &budget)
                .is_err(),
            "equal session ID is not handle provenance"
        );
        let current = owner
            .capture_recovery_source()
            .expect("explicit fresh capture");
        assert!(
            owner
                .restore_recovery_scan_for_source(&current, &budget)
                .expect("matching cutoff")
                .is_some()
        );
    }

    #[test]
    fn trusted_scan_store_resumes_completed_pages_and_refuses_changed_source() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "scan-store").expect("owner");
        session
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "fact ".repeat(600_000),
                }],
                &Default::default(),
            )
            .expect("commit");
        let mut scan = session
            .capture_recovery_source()
            .expect("source")
            .begin_prefix_seal()
            .expect("scan");
        let old_scan = session
            .capture_recovery_source()
            .expect("old source")
            .begin_prefix_seal()
            .expect("old scan");
        let old_receipt = session
            .save_recovery_scan(
                &old_scan,
                &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2)),
            )
            .expect("old checkpoint");
        for _ in 0..32 {
            assert!(
                !scan
                    .step(Instant::now() + Duration::from_secs(2), || false)
                    .expect("page chunk")
            );
        }
        assert_eq!(scan.offset, crate::history_page::HISTORY_PAGE_BYTES as u64);
        let receipt = session
            .save_recovery_scan(
                &scan,
                &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2)),
            )
            .expect("save actual scan");
        assert!(
            session
                .save_recovery_scan(
                    &old_scan,
                    &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2))
                )
                .is_err(),
            "older progress must not overwrite the committed page"
        );
        let budget = crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2));
        assert!(
            !session
                .clear_recovery_scan(&old_receipt, &budget)
                .expect("stale cleanup")
        );
        assert!(
            session
                .restore_recovery_scan(&budget)
                .expect("preserved newer")
                .is_some()
        );
        assert!(
            session
                .clear_recovery_scan(&receipt, &budget)
                .expect("exact cleanup")
        );
        assert!(
            session
                .restore_recovery_scan(&budget)
                .expect("removed")
                .is_none()
        );
        session
            .save_recovery_scan(&scan, &budget)
            .expect("save for reopen");
        let stopped = crate::sqlite::RecoveryWorkBudget::with_cancellation_check(
            Duration::from_secs(2),
            || true,
        );
        assert!(session.save_recovery_scan(&scan, &stopped).is_err());
        assert!(session.restore_recovery_scan(&stopped).is_err());
        let other = DurableSession::open_or_create(root.path(), "other").expect("other");
        assert!(
            other
                .save_recovery_scan(
                    &scan,
                    &crate::sqlite::RecoveryWorkBudget::new(Duration::from_secs(2))
                )
                .is_err()
        );
        drop(scan);
        drop(session);
        let reopened = DurableSession::open_existing(root.path(), "scan-store")
            .expect("reopen")
            .expect("owner");
        let mut restored = reopened
            .restore_recovery_scan(&crate::sqlite::RecoveryWorkBudget::new(
                Duration::from_secs(2),
            ))
            .expect("restore")
            .expect("saved progress")
            .scan;
        assert_eq!(
            restored.offset,
            crate::history_page::HISTORY_PAGE_BYTES as u64
        );
        while !restored
            .step(Instant::now() + Duration::from_secs(2), || false)
            .expect("continue")
        {}
        restored.finish().expect("complete seal");
        reopened
            .commit_turn(
                "append",
                &[Message::User {
                    content: "later".into(),
                }],
                &Default::default(),
            )
            .expect("append");
        assert!(
            reopened
                .restore_recovery_scan(&crate::sqlite::RecoveryWorkBudget::new(
                    Duration::from_secs(2)
                ))
                .expect("stale check")
                .is_none()
        );
    }

    #[test]
    fn interruption_after_full_page_hash_retries_without_duplicating_bytes() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "final-check").expect("session");
        session
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "small fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        assert!(source.source_end() < 8192);
        let mut scan = source.begin_prefix_seal().expect("scan");
        let checks = std::cell::Cell::new(0);
        let stopped = || {
            let next = checks.get() + 1;
            checks.set(next);
            next == 5
        };
        assert!(matches!(
            scan.step(Instant::now() + Duration::from_secs(2), stopped),
            Err(HistoryPageError::Stopped)
        ));
        assert_eq!(checks.get(), 5, "stop at post-hash commit check");
        assert_eq!(scan.offset, 0);
        assert_eq!(scan.digest, [0; 32]);
        assert_eq!(scan.pending_page.len() as u64, source.source_end());
        assert!(
            scan.step(Instant::now() + Duration::from_secs(2), || false)
                .expect("retry")
        );
        assert!(scan.pending_page.is_empty());
        let actual = scan.finish().expect("retried seal");
        let mut uninterrupted = source.begin_prefix_seal().expect("fresh scan");
        assert!(
            uninterrupted
                .step(Instant::now() + Duration::from_secs(2), || false)
                .expect("fresh step")
        );
        assert_eq!(
            actual.digest,
            uninterrupted.finish().expect("fresh seal").digest
        );
    }

    #[test]
    fn subpage_progress_survives_expiry_and_preserves_version_one_digest() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "subpage").expect("session");
        session
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "fact ".repeat(600_000),
                }],
                &Default::default(),
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        let bytes = std::fs::read(session.file_path()).expect("original bytes");
        let mut expected = [0u8; 32];
        let mut offset = 0u64;
        for page in bytes.chunks(crate::history_page::HISTORY_PAGE_BYTES) {
            let end = offset + page.len() as u64;
            let mut hash = Sha256::new();
            hash.update(expected);
            hash.update(offset.to_le_bytes());
            hash.update(end.to_le_bytes());
            hash.update(page);
            expected = hash.finalize().into();
            offset = end;
        }
        let mut scan = source.begin_prefix_seal().expect("scan");
        let mut steps = 0;
        loop {
            let complete = scan
                .step(Instant::now() + Duration::from_secs(2), || false)
                .expect("bounded step");
            steps += 1;
            assert!(scan.pending_page.len() <= crate::history_page::HISTORY_PAGE_BYTES);
            if complete {
                break;
            }
            let progress = (scan.offset, scan.pending_page.len(), scan.digest);
            assert!(matches!(
                scan.step(Instant::now(), || false),
                Err(HistoryPageError::Stopped)
            ));
            assert_eq!(
                (scan.offset, scan.pending_page.len(), scan.digest),
                progress,
                "expiry must preserve prior verified sub-page progress"
            );
        }
        assert!(
            steps > 32,
            "multiple original pages split into bounded reads"
        );
        assert_eq!(
            scan.finish().expect("complete seal").expectation().digest,
            expected
        );
    }

    #[test]
    fn persisted_expectation_requires_full_reopened_source_verification() {
        let root = tempfile::tempdir().expect("root");
        let session = DurableSession::open_or_create(root.path(), "restart-seal").expect("session");
        session
            .commit_turn(
                "first",
                &[Message::User {
                    content: "original fact".into(),
                }],
                &Default::default(),
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut scan = source.begin_prefix_seal().expect("scan");
        while !scan.step(deadline, || false).expect("step") {}
        let seal = scan.finish().expect("seal");
        let bytes = serde_json::to_vec(&seal.expectation()).expect("persist metadata");
        session
            .commit_turn(
                "next",
                &[Message::User {
                    content: "new suffix".into(),
                }],
                &Default::default(),
            )
            .expect("append");
        drop(session);
        let reopened = DurableSession::open_or_create(root.path(), "restart-seal").expect("reopen");
        let source = reopened.capture_recovery_source().expect("new source");
        let expectation: RecoveryPrefixExpectation =
            serde_json::from_slice(&bytes).expect("restore metadata");
        assert!(
            source
                .verify_prefix_expectation(&expectation)
                .expect("begin only")
                .finish()
                .is_err()
        );
        let mut scan = source
            .verify_prefix_expectation(&expectation)
            .expect("verification");
        while !scan.step(deadline, || false).expect("step") {}
        assert_eq!(
            scan.finish().expect("verified").source_end(),
            expectation.cutoff
        );
        let mut bad = expectation.clone();
        bad.digest[0] ^= 1;
        let mut scan = source
            .verify_prefix_expectation(&bad)
            .expect("untrusted expectation");
        assert!(scan.step(deadline, || false).is_err());
        bad = expectation.clone();
        bad.version = 2;
        assert!(source.verify_prefix_expectation(&bad).is_err());
        bad = expectation;
        bad.cutoff = source.source_end() + 1;
        assert!(source.verify_prefix_expectation(&bad).is_err());
    }

    #[test]
    fn multi_page_prefix_verifies_after_append_and_refuses_replacement_or_partial_scan() {
        let dir = tempfile::tempdir().expect("directory");
        let session = DurableSession::open_or_create(dir.path(), "owner").expect("session");
        let messages: Vec<_> = (0..600)
            .map(|i| Message::User {
                content: format!("source {i} {}", "bounded record ".repeat(300)),
            })
            .collect();
        session
            .finalize_turn_with_recovery(
                "first",
                &messages,
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        assert!(source.source_end() > crate::history_page::HISTORY_PAGE_BYTES as u64);
        let mut interrupted = source.begin_prefix_seal().expect("interrupted scan");
        assert!(
            !interrupted
                .step(Instant::now() + Duration::from_secs(10), || false)
                .expect("first chunk")
        );
        let original_meta = std::fs::metadata(session.file_path()).expect("metadata");
        let replacement = dir.path().join("replacement");
        std::fs::copy(session.file_path(), &replacement).expect("same length clone");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&replacement)
            .expect("replacement");
        file.set_times(
            std::fs::FileTimes::new().set_modified(original_meta.modified().expect("mtime")),
        )
        .expect("preserve mtime");
        std::fs::rename(replacement, session.file_path()).expect("atomic replacement");
        assert!(
            matches!(
                source.read_recovery_bytes(
                    0,
                    source.source_end(),
                    &original_meta,
                    Instant::now() + Duration::from_secs(2),
                    || false
                ),
                Err(HistoryPageError::Stale)
            ),
            "opened handle must match expected inode even if length/mtime match"
        );
        assert!(matches!(
            interrupted.step(Instant::now() + Duration::from_secs(10), || false),
            Err(HistoryPageError::Stale)
        ));
        let deadline = Instant::now() + Duration::from_secs(10);
        assert!(source.begin_prefix_seal().expect("scan").finish().is_err());
        let mut scan = source.begin_prefix_seal().expect("scan");
        assert!(matches!(
            scan.step(deadline, || true),
            Err(HistoryPageError::Stopped)
        ));
        let mut steps = 1;
        while !scan.step(deadline, || false).expect("page") {
            steps += 1;
        }
        assert!(steps > 1);
        let seal = scan.finish().expect("seal");
        let original_bytes = std::fs::read(session.file_path()).expect("original bytes");
        session
            .finalize_turn_with_recovery(
                "second",
                &[Message::User {
                    content: "later suffix".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("durable append");
        assert!(
            std::fs::read(session.file_path())
                .expect("committed bytes")
                .starts_with(&original_bytes)
        );
        let mut verify = seal.begin_verification().expect("verify appended source");
        while !verify.step(deadline, || false).expect("same prefix") {}
        assert_eq!(
            verify.finish().expect("verified").source_end(),
            source.source_end()
        );
        // Actual same-length replacement retains IDs and ranges, but not content.
        let mut bytes = std::fs::read(session.file_path()).expect("source bytes");
        let at = bytes
            .windows(b"bounded record".len())
            .position(|b| b == b"bounded record")
            .expect("record text");
        bytes[at] = b'X';
        std::fs::write(session.file_path(), bytes).expect("replace prefix");
        let mut verify = seal.begin_verification().expect("scan changed source");
        loop {
            match verify.step(deadline, || false) {
                Ok(false) => continue,
                Err(HistoryPageError::Stale) => break,
                other => panic!("changed prefix must fail: {other:?}"),
            }
        }
    }

    #[test]
    fn mutation_between_pages_invalidates_scan_instead_of_sealing_mixed_versions() {
        let dir = tempfile::tempdir().expect("directory");
        let session = DurableSession::open_or_create(dir.path(), "owner").expect("session");
        session
            .finalize_turn_with_recovery(
                "turn",
                &[Message::User {
                    content: "original".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("commit");
        let source = session.capture_recovery_source().expect("source");
        let mut scan = source.begin_prefix_seal().expect("scan");
        assert!(
            scan.step(Instant::now() + Duration::from_secs(2), || false)
                .expect("page")
        );
        session
            .finalize_turn_with_recovery(
                "append",
                &[Message::User {
                    content: "suffix".into(),
                }],
                &PersistencePolicy::default(),
                TurnTranscriptOutcome::Success,
                &[],
            )
            .expect("append");
        assert!(matches!(scan.finish(), Err(HistoryPageError::Stale)));
    }
}
