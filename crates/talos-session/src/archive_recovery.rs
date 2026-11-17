//! Read-only reconstruction evidence for legacy overlapping archive segments.
//! This does not grant branch authority or install a model checkpoint.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sha2::{Digest, Sha256};

use crate::{Session, SessionEntry, SessionError};

const MAX_FILES: usize = 256;
const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

#[cfg(test)]
thread_local! {
    static VALIDATION_FINISH_OBSERVER: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

/// Original records joined by exact identity/content overlap and unique order.
/// Parent IDs are preserved, including legacy writer defects. This evidence is
/// not a valid linear transcript and must not be passed off as a checkpoint.
pub struct ArchiveRecoveryEvidence {
    input_bytes: usize,
    input_records: usize,
    directory_entries: usize,
    head_records: usize,
    entries: Vec<SessionEntry>,
    raw_records: Vec<Vec<u8>>,
    sources: Vec<ArchiveRecoverySource>,
    excluded_archives: usize,
    removed_placeholders: usize,
    versions: Vec<(PathBuf, ArchiveFileVersion)>,
    ownership: Vec<crate::archive_publication::Ownership>,
    receipts: Vec<(PathBuf, crate::fork_receipt::Receipt)>,
}

// Windows stamps own restrictive read handles. Keep only inert identity and
// content expectations between recovery operations, never a model-turn lease.
struct ArchiveFileVersion {
    #[cfg(not(windows))]
    stamp: crate::recovery_stamp::FileStamp,
    #[cfg(windows)]
    stamp: crate::recovery_stamp::StoredFileVersion,
    #[cfg(windows)]
    sha256: String,
}

impl ArchiveFileVersion {
    fn from_records(file: FileRecords) -> (PathBuf, Self) {
        let version = Self {
            #[cfg(not(windows))]
            stamp: file.stamp,
            #[cfg(windows)]
            stamp: (&file.stamp).into(),
            #[cfg(windows)]
            sha256: file.source.sha256,
        };
        (file.source.path, version)
    }
}

/// A selected source file and the digest of its exact stored bytes.
#[derive(Debug, Clone)]
pub struct ArchiveRecoverySource {
    /// Host-selected local source, never supplied by a model query.
    pub path: PathBuf,
    /// SHA-256 of the compressed or uncompressed original file bytes.
    pub sha256: String,
}

impl ArchiveRecoveryEvidence {
    // Reserve the crash window where both the original head and new archive
    // are visible. Recovery charges compressed input and decoded bytes together.
    pub(crate) fn check_archive_addition(
        &self,
        stored_bytes: usize,
        decoded_bytes: usize,
    ) -> Result<(), SessionError> {
        self.check_archive_addition_limit(stored_bytes, decoded_bytes, MAX_BYTES)
    }

    #[cfg(test)]
    pub(crate) fn check_archive_addition_with_limit(
        &self,
        stored_bytes: usize,
        decoded_bytes: usize,
        byte_limit: usize,
    ) -> Result<(), SessionError> {
        self.check_archive_addition_limit(stored_bytes, decoded_bytes, byte_limit)
    }

    fn check_archive_addition_limit(
        &self,
        stored_bytes: usize,
        decoded_bytes: usize,
        byte_limit: usize,
    ) -> Result<(), SessionError> {
        if byte_limit == 0 || byte_limit > MAX_BYTES {
            return Err(invalid("invalid archive publication budget"));
        }
        if self.input_bytes.checked_add(stored_bytes)
            .and_then(|n| n.checked_add(decoded_bytes)).is_none_or(|n| n > byte_limit)
            || self.input_records.checked_add(self.head_records).is_none_or(|n| n > MAX_ENTRIES)
            // Archive, temporary alias, chain metadata, and SQLite WAL/SHM.
            || self.directory_entries.checked_add(5).is_none_or(|n| n > MAX_FILES)
        {
            return Err(invalid("archive publication exceeds recovery budget"));
        }
        Ok(())
    }

    /// Check all captured file versions before a host publishes restoration or
    /// query evidence. No decoded records are published on a stale source.
    pub fn check_all_sources(&self, deadline: Instant) -> Result<(), SessionError> {
        self.check_current(deadline, &|| false)
    }
    /// Revalidate supporting archive/other-head versions. The query owner must
    /// separately validate the selected head's captured prefix, allowing only
    /// verified append-only growth there. This alone does not authorize queries.
    pub fn check_supporting_sources(&self, deadline: Instant) -> Result<(), SessionError> {
        self.check_versions(self.versions.iter().skip(1), deadline, &|| false, None)
    }
    /// Decode a complete, unchanged candidate through the normal host restore
    /// decoder. This preserves tool pairing across former archive cuts; it does
    /// not create a summary, checkpoint receipt or model-search authority.
    pub fn into_restored_history(
        mut self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<(Vec<talos_core::message::Message>, Option<SessionEntry>), SessionError> {
        self.check_current(deadline, &stopped)?;
        let tail = self.entries.last().cloned();
        let mut decoder = crate::jsonl::HistoryMessageDecoder::default();
        let mut messages = Vec::new();
        for entry in self.entries.drain(..) {
            check(deadline, &stopped)?;
            if let Some((_, message)) = decoder.decode(entry) {
                messages.push(message);
            }
        }
        self.check_current(deadline, &stopped)?;
        Ok((messages, tail))
    }

    pub(crate) fn check_current(
        &self,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        self.check_versions(self.versions.iter(), deadline, stopped, None)
    }

    // Only the receipt publisher may add this one previously absent sibling.
    // It separately verifies exact child bytes before committing the receipt.
    pub(crate) fn check_with_unpublished_child(
        &self,
        child: &Path,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        if self.versions.iter().any(|(path, _)| path == child)
            || self.versions.first().and_then(|(path, _)| path.parent()) != child.parent()
            || recovery_source_kind(child) != Some(false)
        {
            return Err(invalid("invalid publication child"));
        }
        self.check_versions(self.versions.iter(), deadline, stopped, Some(child))
    }

    pub(crate) fn check_fork_addition(
        &self,
        bytes: usize,
        records: usize,
    ) -> Result<(), SessionError> {
        if self
            .input_bytes
            .checked_add(bytes)
            .is_none_or(|n| n > MAX_BYTES)
            || self
                .input_records
                .checked_add(records)
                .is_none_or(|n| n > MAX_ENTRIES)
            || self
                .directory_entries
                .checked_add(4)
                .is_none_or(|n| n > MAX_FILES)
        {
            return Err(invalid("fork publication exceeds recovery budget"));
        }
        Ok(())
    }

    fn check_versions<'a>(
        &self,
        versions: impl Iterator<Item = &'a (PathBuf, ArchiveFileVersion)>,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
        unpublished_child: Option<&Path>,
    ) -> Result<(), SessionError> {
        let versions: Vec<_> = versions.collect();
        let head = self
            .versions
            .first()
            .ok_or_else(|| invalid("missing selected head"))?;
        // Supporting-only validation deliberately permits head append, but must
        // never retain disclosure authority after the selected identity retires.
        check(deadline, stopped)?;
        let _identity = crate::identity_custody::require_not_retired_until(&head.0, deadline)?;
        for (path, expected) in &self.receipts {
            check(deadline, stopped)?;
            if crate::fork_receipt::load(path, deadline)? != *expected {
                return Err(invalid("fork receipt changed"));
            }
        }
        check(deadline, stopped)?;
        let directory = head
            .0
            .parent()
            .ok_or_else(|| invalid("missing directory"))?;
        let names: Vec<_> = self
            .ownership
            .iter()
            .map(|record| record.name.clone())
            .collect();
        if crate::archive_publication::load(&head.0, &names, deadline, stopped)? != self.ownership {
            return Err(invalid("archive ownership changed"));
        }
        let mut expected_paths: BTreeSet<_> =
            self.versions.iter().map(|(path, _)| path.clone()).collect();
        for (index, item) in std::fs::read_dir(directory)?.enumerate() {
            check(deadline, stopped)?;
            if index >= MAX_FILES {
                return Err(invalid("directory bound exceeded"));
            }
            let path = item?.path();
            if recovery_source_kind(&path).is_some()
                && unpublished_child != Some(path.as_path())
                && !expected_paths.remove(&path)
            {
                return Err(invalid("source inventory changed"));
            }
        }
        if !expected_paths.is_empty() {
            return Err(invalid("source inventory changed"));
        }
        // Hold fresh Windows leases only for this bounded validation, so every
        // checked file stays stable until the inventory has been rechecked.
        #[cfg(windows)]
        let mut leases = Vec::new();
        #[cfg(windows)]
        let mut remaining = MAX_BYTES;
        for (path, expected) in &versions {
            check(deadline, stopped)?;
            let file = crate::history_page::open_source_until(path, deadline)
                .map_err(|e| invalid(&e.to_string()))?;
            let stamp =
                crate::recovery_stamp::from_file(&file).map_err(|e| invalid(&e.to_string()))?;
            #[cfg(not(windows))]
            if !crate::recovery_seal::same_file_version(&expected.stamp, &stamp)
                .map_err(|e| invalid(&e.to_string()))?
            {
                return Err(invalid("source changed before restoration"));
            }
            #[cfg(windows)]
            {
                if crate::recovery_stamp::StoredFileVersion::from(&stamp) != expected.stamp {
                    return Err(invalid("source changed before restoration"));
                }
                // Metadata equality after releasing a Windows lease is not
                // content authority (ADR-096). Rehash every stored byte under
                // the new restrictive handle, including excluded live heads.
                let mut file = file;
                let mut digest = Sha256::new();
                let mut buffer = [0_u8; 64 * 1024];
                loop {
                    check(deadline, stopped)?;
                    let read = file.read(&mut buffer)?;
                    if read == 0 {
                        break;
                    }
                    remaining = remaining
                        .checked_sub(read)
                        .ok_or_else(|| invalid("source byte bound exceeded"))?;
                    digest.update(&buffer[..read]);
                }
                if hex::encode(digest.finalize()) != expected.sha256 {
                    return Err(invalid("source content changed before restoration"));
                }
                leases.push(stamp);
            }
        }
        #[cfg(test)]
        VALIDATION_FINISH_OBSERVER.with(|observer| {
            if let Some(observer) = observer.borrow_mut().take() {
                observer();
            }
        });
        {
            let mut remaining_paths: BTreeSet<_> =
                self.versions.iter().map(|(path, _)| path.clone()).collect();
            for (index, item) in std::fs::read_dir(directory)?.enumerate() {
                check(deadline, stopped)?;
                if index >= MAX_FILES {
                    return Err(invalid("directory bound exceeded"));
                }
                let path = item?.path();
                if recovery_source_kind(&path).is_some()
                    && unpublished_child != Some(path.as_path())
                    && !remaining_paths.remove(&path)
                {
                    return Err(invalid("source inventory changed"));
                }
            }
            if !remaining_paths.is_empty() {
                return Err(invalid("source inventory changed"));
            }
        }
        // An open handle stabilizes its object, not necessarily its pathname:
        // atomic replacement can publish another object at the same name.
        // Rebind every checked path after all source reads and inventory checks.
        for (path, expected) in versions {
            check(deadline, stopped)?;
            let file = crate::history_page::open_source_until(path, deadline)
                .map_err(|e| invalid(&e.to_string()))?;
            let current =
                crate::recovery_stamp::from_file(&file).map_err(|e| invalid(&e.to_string()))?;
            #[cfg(not(windows))]
            let unchanged = crate::recovery_seal::same_file_version(&expected.stamp, &current)
                .map_err(|e| invalid(&e.to_string()))?;
            #[cfg(windows)]
            let unchanged =
                crate::recovery_stamp::StoredFileVersion::from(&current) == expected.stamp;
            if !unchanged {
                return Err(invalid("source changed before restoration"));
            }
        }
        if crate::archive_publication::load(&head.0, &names, deadline, stopped)? != self.ownership {
            return Err(invalid("archive ownership changed"));
        }
        check(deadline, stopped)?;
        Ok(())
    }
    /// Copy original TLOG record bytes through an inclusive host-selected entry.
    ///
    /// This is preparation data for host fork publication, not a fork receipt or
    /// model-disclosure authority. No file or binding is created. The returned
    /// records omit proven count placeholders, preserve original IDs/parents and
    /// encoding, and contain no file header. A publisher must add one TLOG header
    /// and persist its own trusted inheritance receipt before exposing a child.
    /// All captured sources are checked before and after the bounded copy.
    pub fn original_record_prefix(
        &self,
        through_entry: &str,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<Vec<Vec<u8>>, SessionError> {
        self.check_current(deadline, &stopped)?;
        let mut prefix = Vec::new();
        let mut found = false;
        let mut remaining = MAX_BYTES;
        for (entry, raw) in self.entries.iter().zip(&self.raw_records) {
            check(deadline, &stopped)?;
            remaining = remaining
                .checked_sub(raw.len())
                .ok_or_else(|| invalid("prefix byte bound exceeded"))?;
            prefix.push(raw.clone());
            if entry.id == through_entry {
                found = true;
                break;
            }
        }
        if !found {
            return Err(SessionError::EntryNotFound(through_entry.into()));
        }
        self.check_current(deadline, &stopped)?;
        Ok(prefix)
    }

    /// Whether the selected head has a validated private fork receipt.
    /// This evidence still requires source revalidation before query delivery.
    pub fn has_fork_receipt(&self) -> bool {
        self.selected_fork_receipt().is_some()
    }

    // The selected head's receipt has already validated the complete logical
    // stream; callers must recheck this evidence before publishing a descendant.
    pub(crate) fn selected_fork_receipt(&self) -> Option<&crate::fork_receipt::Receipt> {
        let head = &self.versions.first()?.0;
        self.receipts
            .iter()
            .find(|(path, _)| path == head)
            .map(|(_, receipt)| receipt)
    }

    /// Original records in uniquely determined storage order, without synthetic
    /// count placeholders. These are host inspection data, not model disclosure.
    pub fn entries(&self) -> &[SessionEntry] {
        &self.entries
    }
    /// Files selected by exact-record overlap with the selected head.
    pub fn sources(&self) -> &[ArchiveRecoverySource] {
        &self.sources
    }
    /// Archives with no proven overlap; they were not silently concatenated.
    pub fn excluded_archives(&self) -> usize {
        self.excluded_archives
    }
    /// Exact legacy count placeholders omitted from the candidate record stream.
    pub fn removed_placeholders(&self) -> usize {
        self.removed_placeholders
    }
}

struct FileRecords {
    source: ArchiveRecoverySource,
    stamp: crate::recovery_stamp::FileStamp,
    records: Vec<(SessionEntry, Vec<u8>)>,
    archive: bool,
    receipt: Option<crate::fork_receipt::Receipt>,
    envelope: bool,
}

fn invalid(reason: &str) -> SessionError {
    SessionError::ParseError(format!("archive recovery: {reason}"))
}
fn check(deadline: Instant, stopped: &impl Fn() -> bool) -> Result<(), SessionError> {
    if stopped() || Instant::now() >= deadline {
        Err(invalid("stopped"))
    } else {
        Ok(())
    }
}

// Classify only formats that can participate in recovery ownership evidence.
fn recovery_source_kind(path: &Path) -> Option<bool> {
    let name = path.file_name()?.to_str()?;
    // Reserved owned names cannot silently disappear into the legacy namespace
    // when malformed or missing their private publication record.
    let archive = crate::archive_publication::owned(name)
        || name
            .strip_prefix("s-")
            .and_then(|name| {
                name.strip_suffix(".tlog.zst")
                    .or_else(|| name.strip_suffix(".tlog"))
            })
            .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok());
    let head = name
        .strip_suffix(".tlog")
        .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok());
    (archive || head).then_some(archive)
}

impl Session {
    /// Inspect this head and bounded sibling archives without changing files.
    /// The host must hold its execution lease or transition fence. Exact original bytes
    /// establish overlap; a unique adjacency order is required. Other live heads
    /// sharing candidate IDs, conflicting IDs, cycles, missing/forward parents,
    /// malformed data and source changes fail closed. Legacy non-adjacent parents
    /// are retained for explicit subsequent recovery review, never repaired here.
    pub fn inspect_archive_recovery(
        &self,
        deadline: Instant,
        stopped: impl Fn() -> bool,
    ) -> Result<ArchiveRecoveryEvidence, SessionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.inspect_archive_recovery_inner(deadline, &stopped)
        }))
        .unwrap_or_else(|_| Err(invalid("decoder failed")))
    }

    fn inspect_archive_recovery_inner(
        &self,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
    ) -> Result<ArchiveRecoveryEvidence, SessionError> {
        if self.file_path.file_name().and_then(|s| s.to_str()) != Some(&format!("{}.tlog", self.id))
        {
            return Err(invalid("selected head identity mismatch"));
        }
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let directory = self
            .file_path
            .parent()
            .ok_or_else(|| invalid("missing directory"))?;
        let mut paths = vec![(self.file_path.clone(), false)];
        let mut inspected = 0;
        for item in std::fs::read_dir(directory)? {
            check(deadline, stopped)?;
            inspected += 1;
            if inspected > MAX_FILES {
                return Err(invalid("directory bound exceeded"));
            }
            let path = item?.path();
            if path == self.file_path {
                continue;
            }
            if let Some(archive) = recovery_source_kind(&path) {
                paths.push((path, archive));
            }
        }
        paths[1..].sort_by(|a, b| a.0.cmp(&b.0));
        let mut remaining = MAX_BYTES;
        let mut count = 0;
        let mut files = Vec::new();
        for (path, archive) in paths {
            files.push(read_file(
                &path,
                archive,
                &mut remaining,
                &mut count,
                deadline,
                stopped,
            )?);
        }
        let names: Vec<_> = files
            .iter()
            .filter_map(|file| {
                let name = file.source.path.file_name()?.to_str()?;
                crate::archive_publication::owned(name).then(|| name.to_owned())
            })
            .collect();
        let ownership =
            crate::archive_publication::load(&self.file_path, &names, deadline, stopped)?;
        let captured: Vec<_> = files.iter().collect();
        let rebuilt = reconstruct(&captured, &ownership, self.id, deadline, stopped)?;
        let ordered = rebuilt.entries;
        let raw_records = rebuilt.raw;
        let selected = rebuilt.selected;
        let removed_placeholders = rebuilt.placeholders;
        let own_positions: BTreeMap<_, _> = ordered
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.id.as_str(), index))
            .collect();
        for (file_index, file) in files
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, file)| !file.archive)
        {
            let other_id = file
                .source
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .ok_or_else(|| invalid("other head identity invalid"))?;
            let sibling = if file.receipt.is_some()
                && file
                    .records
                    .first()
                    .is_some_and(|(entry, _)| placeholder(entry))
            {
                let mut reordered = captured.clone();
                reordered.swap(0, file_index);
                Some(reconstruct(
                    &reordered, &ownership, other_id, deadline, stopped,
                )?)
            } else {
                None
            };
            let other_records: Vec<_> = match &sibling {
                Some(rebuilt) => rebuilt.entries.iter().zip(&rebuilt.raw).collect(),
                None => file
                    .records
                    .iter()
                    .map(|(entry, raw)| (entry, raw))
                    .collect(),
            };
            for (other_index, (entry, raw)) in other_records.into_iter().enumerate() {
                check(deadline, stopped)?;
                let own_index = own_positions.get(entry.id.as_str()).copied();
                let Some(own_raw) = own_index.map(|index| &raw_records[index]) else {
                    continue;
                };
                let own_origin = match (&files[0].receipt, own_index) {
                    (Some(receipt), Some(index)) => receipt.origin(index),
                    (Some(_), None) => return Err(invalid("fork inherited record missing")),
                    (None, _) => self.id,
                };
                let other_origin = file
                    .receipt
                    .as_ref()
                    .map_or(other_id, |receipt| receipt.origin(other_index));
                let inherited = file
                    .receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.inherited(other_index))
                    || files[0]
                        .receipt
                        .as_ref()
                        .zip(own_index)
                        .is_some_and(|(receipt, index)| receipt.inherited(index));
                if raw != own_raw || own_origin != other_origin || !inherited {
                    return Err(invalid("candidate overlaps another live session"));
                }
            }
        }
        // Recheck every inspected file: an excluded head/archive must not change
        // while its absence of overlap is used as evidence.
        for file in &files {
            check(deadline, stopped)?;
            let current = crate::history_page::open_source_until(&file.source.path, deadline)
                .map_err(|e| invalid(&e.to_string()))?;
            let stamp =
                crate::recovery_stamp::from_file(&current).map_err(|e| invalid(&e.to_string()))?;
            if !crate::recovery_seal::same_file_version(&file.stamp, &stamp)
                .map_err(|e| invalid(&e.to_string()))?
            {
                return Err(invalid("source changed"));
            }
        }
        if crate::archive_publication::load(&self.file_path, &names, deadline, stopped)?
            != ownership
        {
            return Err(invalid("archive ownership changed"));
        }
        Ok(ArchiveRecoveryEvidence {
            input_bytes: MAX_BYTES - remaining,
            input_records: count,
            directory_entries: inspected,
            head_records: files[0].records.len(),
            receipts: files
                .iter()
                .filter_map(|file| {
                    file.receipt
                        .clone()
                        .map(|receipt| (file.source.path.clone(), receipt))
                })
                .collect(),
            ownership,
            entries: ordered,
            raw_records,
            sources: selected.iter().map(|&i| files[i].source.clone()).collect(),
            excluded_archives: files
                .iter()
                .enumerate()
                .filter(|(i, f)| f.archive && !selected.contains(i))
                .count(),
            removed_placeholders,
            versions: files
                .into_iter()
                .map(ArchiveFileVersion::from_records)
                .collect(),
        })
    }
}

struct Reconstructed {
    entries: Vec<SessionEntry>,
    raw: Vec<Vec<u8>>,
    selected: BTreeSet<usize>,
    placeholders: usize,
}

// Reuse the single captured inventory. Sibling reconstruction never recursively
// scans the directory or resets its input bounds/deadline.
fn reconstruct(
    files: &[&FileRecords],
    ownership: &[crate::archive_publication::Ownership],
    owner: uuid::Uuid,
    deadline: Instant,
    stopped: &impl Fn() -> bool,
) -> Result<Reconstructed, SessionError> {
    let mut foreign = BTreeSet::new();
    for (index, file) in files.iter().enumerate() {
        let binding = ownership.iter().find(|record| {
            file.source.path.file_name().and_then(|s| s.to_str()) == Some(record.name.as_str())
        });
        // Binary archives cannot enter the legacy overlap namespace. Their
        // physical owner/digest must have been committed before visibility,
        // including when that owner has since been deleted.
        if file.archive && file.envelope && binding.is_none() {
            return Err(invalid("fork archive ownership missing"));
        }
        if let Some(record) = binding {
            if record.digest != file.source.sha256 {
                return Err(invalid("archive ownership digest mismatch"));
            }
            if record.owner != owner.to_string() {
                foreign.insert(index);
            } else if file.envelope && files[0].receipt.is_none() {
                return Err(invalid("fork archive selected without inheritance receipt"));
            }
        }
    }
    let locators = crate::compaction_locator::load(&files[0].source.path, deadline, stopped)?;
    let mut bridges = BTreeMap::new();
    for (index, file) in files.iter().enumerate() {
        let Some((marker, raw)) = file.records.first() else {
            continue;
        };
        if let Some(locator) = locators.iter().find(|locator| locator.marker == marker.id) {
            if !placeholder(marker)
                || crate::compaction_locator::digest(raw) != locator.marker_digest
            {
                return Err(invalid("compaction marker changed"));
            }
            let target = files
                .iter()
                .position(|candidate| {
                    candidate.archive
                        && candidate.source.path.file_name().and_then(|s| s.to_str())
                            == Some(locator.archive.as_str())
                })
                .ok_or_else(|| invalid("compaction archive missing"))?;
            if foreign.contains(&target) {
                return Err(invalid("compaction archive belongs to another owner"));
            }
            if files[target].source.sha256 != locator.archive_digest {
                return Err(invalid("compaction archive changed"));
            }
            let mut previous = locator.tail.as_str();
            for (entry, _) in file.records.iter().skip(1) {
                check(deadline, stopped)?;
                if placeholder(entry) || entry.parent_id.as_deref() != Some(previous) {
                    return Err(invalid("compaction suffix parent mismatch"));
                }
                previous = &entry.id;
            }
            bridges.insert(index, (target, locator.tail.clone()));
        }
    }
    for (index, (target, tail)) in &bridges {
        check(deadline, stopped)?;
        let valid_tail = files[*target].records.last().is_some_and(|(entry, _)| {
            if placeholder(entry) {
                files[*target].records.len() == 1
                    && bridges
                        .get(target)
                        .is_some_and(|(_, prior_tail)| prior_tail == tail)
            } else {
                entry.id == *tail
            }
        });
        if !valid_tail {
            return Err(invalid("compaction tail commitment mismatch"));
        }
        let mut seen = BTreeSet::new();
        let mut current = *index;
        while let Some((next, _)) = bridges.get(&current) {
            check(deadline, stopped)?;
            if !seen.insert(current) {
                return Err(invalid("compaction locator cycle"));
            }
            current = *next;
        }
    }
    let mut selected = BTreeSet::from([0]);
    let mut known: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    insert_records(files[0], &mut known)?;
    loop {
        let mut added = false;
        for index in selected.clone() {
            if let Some((target, _)) = bridges.get(&index)
                && selected.insert(*target)
            {
                insert_records(files[*target], &mut known)?;
                added = true;
            }
        }
        for (index, file) in files.iter().enumerate().skip(1) {
            check(deadline, stopped)?;
            if selected.contains(&index)
                || !file.archive
                || foreign.contains(&index)
                || (files[0].receipt.is_some()
                    && !ownership.iter().any(|record| {
                        record.owner == owner.to_string()
                            && file.source.path.file_name().and_then(|name| name.to_str())
                                == Some(record.name.as_str())
                    }))
            {
                continue;
            }
            if file
                .records
                .iter()
                .any(|(entry, _)| known.contains_key(&entry.id))
            {
                insert_records(file, &mut known)?;
                selected.insert(index);
                added = true;
            }
        }
        if !added {
            break;
        }
    }
    if files[0].receipt.is_some() {
        for &index in selected.iter().filter(|&&index| index != 0) {
            let file = &files[index];
            if !file.envelope
                || !ownership.iter().any(|record| {
                    record.owner == owner.to_string()
                        && file.source.path.file_name().and_then(|name| name.to_str())
                            == Some(record.name.as_str())
                })
            {
                return Err(invalid("inherited archive lacks owned envelope"));
            }
        }
    }
    let mut records = BTreeMap::new();
    let mut successors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut incoming: BTreeMap<String, usize> = BTreeMap::new();
    let mut placeholders = BTreeSet::new();
    for &index in &selected {
        if files[index].records.first().is_some_and(|(entry, _)| {
            placeholder(entry) && entry.content.ends_with(", 0 recent entries preserved]")
        }) && !bridges.contains_key(&index)
        {
            return Err(invalid("zero-retention compaction receipt missing"));
        }
        let mut previous = bridges.get(&index).map(|(_, tail)| tail.clone());
        if previous
            .as_ref()
            .is_some_and(|tail| !known.contains_key(tail))
        {
            return Err(invalid("compaction original tail missing"));
        }
        for (entry, _) in &files[index].records {
            check(deadline, stopped)?;
            if placeholder(entry) {
                placeholders.insert(entry.id.clone());
                continue;
            }
            records.insert(entry.id.clone(), entry.clone());
            incoming.entry(entry.id.clone()).or_default();
            if let Some(parent) = previous
                && successors
                    .entry(parent)
                    .or_default()
                    .insert(entry.id.clone())
            {
                *incoming.entry(entry.id.clone()).or_default() += 1;
            }
            previous = Some(entry.id.clone());
        }
    }
    let mut ready: BTreeSet<_> = incoming
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut ordered = Vec::new();
    let mut seen = BTreeSet::new();
    while !ready.is_empty() {
        check(deadline, stopped)?;
        if ready.len() != 1 {
            return Err(invalid("ambiguous record order"));
        }
        let id = ready
            .pop_first()
            .ok_or_else(|| invalid("missing ready record"))?;
        let entry = records
            .remove(&id)
            .ok_or_else(|| invalid("missing record"))?;
        if entry
            .parent_id
            .as_ref()
            .is_some_and(|parent| !seen.contains(parent))
        {
            return Err(invalid("missing or forward original parent"));
        }
        seen.insert(id.clone());
        ordered.push(entry);
        if let Some(children) = successors.get(&id) {
            for child in children {
                let degree = incoming
                    .get_mut(child)
                    .ok_or_else(|| invalid("missing adjacency"))?;
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(child.clone());
                }
            }
        }
    }
    if !records.is_empty() {
        return Err(invalid("record order cycle"));
    }
    let raw_records = ordered
        .iter()
        .map(|entry| {
            known
                .get(&entry.id)
                .cloned()
                .ok_or_else(|| invalid("missing original record bytes"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(receipt) = &files[0].receipt {
        receipt.validate_records(&raw_records, deadline)?;
    }
    Ok(Reconstructed {
        entries: ordered,
        raw: raw_records,
        selected,
        placeholders: placeholders.len(),
    })
}

fn insert_records(
    file: &FileRecords,
    known: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), SessionError> {
    let mut local = BTreeSet::new();
    for (entry, bytes) in &file.records {
        if entry.id.is_empty() || !local.insert(&entry.id) {
            return Err(invalid("duplicate or empty record ID"));
        }
        if let Some(prior) = known.get(&entry.id) {
            if prior != bytes {
                return Err(invalid("conflicting stable record ID"));
            }
        } else {
            known.insert(entry.id.clone(), bytes.clone());
        }
    }
    Ok(())
}

pub(crate) fn placeholder(entry: &SessionEntry) -> bool {
    if entry.role != "system" || entry.parent_id.is_some() || !entry.metadata.is_empty() {
        return false;
    }
    let Some(body) = entry
        .content
        .strip_prefix("[Compaction summary: ")
        .and_then(|s| s.strip_suffix(" recent entries preserved]"))
    else {
        return false;
    };
    let Some((old, recent)) = body.split_once(" earlier entries summarized, ") else {
        return false;
    };
    old.parse::<usize>().is_ok() && recent.parse::<usize>().is_ok()
}

fn read_file(
    path: &Path,
    archive: bool,
    remaining: &mut usize,
    count: &mut usize,
    deadline: Instant,
    stopped: &impl Fn() -> bool,
) -> Result<FileRecords, SessionError> {
    check(deadline, stopped)?;
    let mut file = crate::history_page::open_source_until(path, deadline)
        .map_err(|e| invalid(&e.to_string()))?;
    let stamp = crate::recovery_stamp::from_file(&file).map_err(|e| invalid(&e.to_string()))?;
    let mut stored = Vec::new();
    read_bounded(&mut file, &mut stored, remaining, deadline, stopped)?;
    let sha256 = hex::encode(Sha256::digest(&stored));
    let decoded = if path.extension().is_some_and(|s| s == "zst") {
        #[cfg(feature = "archive-compress-zstd")]
        {
            let mut decoder = zstd::stream::read::Decoder::new(stored.as_slice())
                .map_err(|_| invalid("compressed archive invalid"))?;
            // A small frame may request a large native decoder window even when
            // its output fits the byte budget. Bound that allocation separately.
            decoder
                .window_log_max(MAX_BYTES.ilog2())
                .map_err(|_| invalid("compressed archive window limit unavailable"))?;
            let mut decoded = Vec::new();
            read_bounded(&mut decoder, &mut decoded, remaining, deadline, stopped)?;
            decoded
        }
        #[cfg(not(feature = "archive-compress-zstd"))]
        {
            return Err(invalid("compressed archives unsupported by this build"));
        }
    } else {
        stored
    };
    let envelope = decoded.starts_with(crate::fork_receipt::ENVELOPE);
    let receipt = if envelope && !archive {
        let receipt = crate::fork_receipt::load(path, deadline)?;
        let offset = crate::fork_receipt::envelope_header_length(&decoded)?;
        let compacted = crate::history_page::decode_tlog_record(&decoded[offset..], MAX_BYTES)
            .map_err(|_| invalid("invalid fork first record"))?
            .is_some_and(|(entry, _)| placeholder(&entry));
        if !compacted {
            receipt.validate_materialized(&decoded, deadline)?;
        }
        Some(receipt)
    } else {
        None
    };
    let mut offset = if envelope {
        crate::fork_receipt::envelope_header_length(&decoded)?
    } else {
        0
    };
    if decoded.starts_with(b"TALOS\t") {
        offset = decoded
            .iter()
            .position(|b| *b == b'\n')
            .ok_or_else(|| invalid("invalid header"))?
            + 1;
        let header =
            std::str::from_utf8(&decoded[..offset - 1]).map_err(|_| invalid("invalid header"))?;
        let parts: Vec<_> = header.split('\t').collect();
        if parts.len() != 3 || parts[1] != "1" || parts[2].parse::<i64>().is_err() {
            return Err(invalid("unsupported header"));
        }
    }
    let mut records = Vec::new();
    let mut ids = BTreeSet::new();
    while offset < decoded.len() {
        check(deadline, stopped)?;
        if matches!(decoded[offset], b'\n' | b'\r') {
            offset += 1;
            continue;
        }
        let (entry, length) =
            crate::history_page::decode_tlog_record(&decoded[offset..], MAX_BYTES)
                .map_err(|e| invalid(&e.to_string()))?
                .ok_or_else(|| invalid("incomplete record"))?;
        *count += 1;
        if *count > MAX_ENTRIES {
            return Err(invalid("record bound exceeded"));
        }
        if entry.id.is_empty() || !ids.insert(entry.id.clone()) {
            return Err(invalid("duplicate or empty record ID"));
        }
        records.push((entry, decoded[offset..offset + length].to_vec()));
        offset += length;
    }
    Ok(FileRecords {
        source: ArchiveRecoverySource {
            path: path.into(),
            sha256,
        },
        stamp,
        records,
        archive,
        receipt,
        envelope,
    })
}

fn read_bounded(
    reader: &mut impl Read,
    bytes: &mut Vec<u8>,
    remaining: &mut usize,
    deadline: Instant,
    stopped: &impl Fn() -> bool,
) -> Result<(), SessionError> {
    let mut chunk = [0; 8192];
    loop {
        check(deadline, stopped)?;
        let length = reader
            .read(&mut chunk)
            .map_err(|_| invalid("source read failed"))?;
        if length == 0 {
            return Ok(());
        }
        *remaining = remaining
            .checked_sub(length)
            .ok_or_else(|| invalid("byte bound exceeded"))?;
        bytes.extend_from_slice(&chunk[..length]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SessionStore;
    use std::time::Duration;
    use talos_core::message::{Message, MessageToolResult, ToolCall};

    fn append_users(session: &Session, start: usize, end: usize) {
        for i in start..end {
            session
                .append(&Message::User {
                    content: format!("fact {i}"),
                })
                .expect("append");
        }
    }
    fn inspect(session: &Session) -> Result<ArchiveRecoveryEvidence, SessionError> {
        session.inspect_archive_recovery(Instant::now() + Duration::from_secs(5), || false)
    }
    fn fixture() -> (tempfile::TempDir, crate::SessionManager, Session) {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().into());
        let session = manager.create_session("archive", "").expect("session");
        (root, manager, session)
    }

    #[test]
    fn legacy_noncanonical_uuid_archive_names_keep_existing_recovery_semantics() {
        for compact_id in [false, true] {
            let (_root, _manager, session) = fixture();
            append_users(&session, 0, 205);
            session.compact_archived(50).expect("archive");
            let evidence = inspect(&session).expect("owned evidence");
            let archive = evidence
                .sources()
                .iter()
                .find(|source| source.path != session.file_path)
                .expect("archive")
                .path
                .clone();
            let id = uuid::Uuid::new_v4();
            let name = if compact_id {
                id.simple().to_string()
            } else {
                id.to_string().to_uppercase()
            };
            let extension = if archive.extension().is_some_and(|s| s == "zst") {
                "tlog.zst"
            } else {
                "tlog"
            };
            // Model files written under the historical legacy name acceptance rule.
            std::fs::rename(
                &archive,
                archive.with_file_name(format!("s-{name}.{extension}")),
            )
            .expect("legacy fixture");
            assert_eq!(
                inspect(&session).expect("legacy recovery").entries().len(),
                205
            );
        }
    }

    #[test]
    fn foreign_owned_archive_does_not_extend_parent_after_child_deletion() {
        let (_root, manager, parent) = fixture();
        append_users(&parent, 0, 205);
        let original = parent.snapshot_bytes().expect("parent bytes");
        parent.compact_archived(50).expect("parent archive");
        let child_id = uuid::Uuid::new_v4();
        let child_path = parent
            .file_path
            .parent()
            .expect("dir")
            .join(format!("{child_id}.tlog"));
        std::fs::write(&child_path, original).expect("materialized inherited prefix fixture");
        let child = Session::new(child_id, "archive".into(), String::new(), child_path);
        append_users(&child, 205, 206);
        child.compact_archived(1).expect("child archive");
        manager
            .delete_session(&child_id)
            .expect("delete child head through lifecycle");
        let evidence = inspect(&parent).expect("parent recovery");
        assert_eq!(evidence.entries().len(), 205);
        assert_eq!(evidence.entries().last().expect("tail").content, "fact 204");
        assert_eq!(evidence.excluded_archives(), 1);
    }

    #[test]
    fn owned_archive_missing_or_changed_registration_never_falls_back_to_legacy() {
        for corruption in ["row", "table", "digest"] {
            let (_root, _manager, session) = fixture();
            append_users(&session, 0, 205);
            session.compact_archived(50).expect("archive");
            let evidence = inspect(&session).expect("evidence");
            let db = session
                .file_path
                .parent()
                .expect("dir")
                .join("history-bindings.sqlite");
            let connection = crate::durable::open_bindings(&db).expect("db");
            connection
                .execute_batch(match corruption {
                    "row" => "DELETE FROM archive_publication_intents",
                    "table" => "DROP TABLE archive_publication_intents",
                    _ => "UPDATE archive_publication_intents SET digest=printf('%064d',0)",
                })
                .expect("corrupt private fixture");
            assert!(inspect(&session).is_err(), "{corruption}");
            assert!(
                evidence
                    .check_supporting_sources(Instant::now() + Duration::from_secs(5))
                    .is_err(),
                "{corruption}"
            );
        }
    }

    #[test]
    fn archive_publication_crash_child() {
        let Some(path) = std::env::var_os("TALOS_TEST_OWNED_ARCHIVE_HEAD") else {
            return;
        };
        let path = PathBuf::from(path);
        let id = uuid::Uuid::parse_str(path.file_stem().expect("stem").to_str().expect("id"))
            .expect("uuid");
        let _lease = crate::SessionRuntimeLease::acquire(&path).expect("lease");
        Session::new(id, "archive".into(), String::new(), path)
            .compact_archived(1)
            .expect("crash phase");
        panic!("crash hook not reached");
    }

    #[test]
    fn foreign_archive_publication_crash_keeps_parent_original_history() {
        for phase in ["archive_intent", "archive_published"] {
            let (_root, manager, parent) = fixture();
            append_users(&parent, 0, 205);
            let original = parent.snapshot_bytes().expect("parent bytes");
            parent.compact_archived(50).expect("parent archive");
            let id = uuid::Uuid::new_v4();
            let path = parent
                .file_path
                .parent()
                .expect("dir")
                .join(format!("{id}.tlog"));
            std::fs::write(&path, original).expect("inherited fixture");
            let child_session = Session::new(id, "archive".into(), String::new(), path.clone());
            append_users(&child_session, 205, 206);
            let before = child_session.snapshot_bytes().expect("child before");
            let mut process = std::process::Command::new(std::env::current_exe().expect("exe"))
                .args([
                    "--exact",
                    "archive_recovery::tests::archive_publication_crash_child",
                ])
                .env("TALOS_TEST_OWNED_ARCHIVE_HEAD", &path)
                .env("TALOS_TEST_FORK_CRASH_PHASE", phase)
                .stdout(std::process::Stdio::null())
                .spawn()
                .expect("spawn");
            let deadline = Instant::now() + Duration::from_secs(15);
            let status = loop {
                if let Some(status) = process.try_wait().expect("wait") {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = process.kill();
                    let _ = process.wait();
                    panic!("crash child timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(42));
            assert_eq!(
                child_session.snapshot_bytes().expect("unchanged child"),
                before
            );
            manager
                .delete_session(&id)
                .expect("delete unpublished child fixture");
            assert_eq!(
                inspect(&parent).expect("unpolluted parent").entries().len(),
                205,
                "{phase}"
            );
        }
    }

    #[test]
    fn keep_zero_restart_append_and_repeat_recovers_original_order() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        let originals = session.read_entries().expect("originals");
        session.compact_archived(0).expect("keep zero");
        let reopened = manager.get_session(&session.id).expect("restart");
        append_users(&reopened, 205, 207);
        let head = reopened.read_entries().expect("head");
        assert_eq!(
            head[1].parent_id.as_deref(),
            Some(originals[204].id.as_str()),
            "a synthetic placeholder must not become the original history parent"
        );
        let recovered = inspect(&reopened).expect("zero-overlap recovery");
        assert_eq!(recovered.entries().len(), 207);
        assert_eq!(recovered.entries()[204].id, originals[204].id);
        reopened.compact_archived(0).expect("second keep zero");
        let reopened = manager.get_session(&session.id).expect("second restart");
        append_users(&reopened, 207, 208);
        let recovered = inspect(&reopened).expect("ordered zero-overlap chain");
        assert_eq!(recovered.entries().len(), 208);
        for (i, entry) in recovered.entries().iter().enumerate() {
            assert_eq!(entry.content, format!("fact {i}"));
        }
    }

    #[test]
    fn keep_zero_missing_receipt_or_archive_prevents_append_without_mutation() {
        for remove_receipt in [true, false] {
            let (_root, manager, session) = fixture();
            append_users(&session, 0, 205);
            session.compact_archived(0).expect("compact");
            let locator = crate::compaction_locator::load(
                &session.file_path,
                Instant::now() + Duration::from_secs(5),
                &|| false,
            )
            .expect("locators")
            .remove(0);
            let directory = session.file_path.parent().expect("directory");
            if remove_receipt {
                let connection =
                    crate::durable::open_bindings(&directory.join("history-bindings.sqlite"))
                        .expect("db");
                connection
                    .execute("DELETE FROM compaction_zero_locators", [])
                    .expect("remove receipt fixture");
            } else {
                std::fs::remove_file(directory.join(locator.archive))
                    .expect("remove archive fixture");
            }
            let before = session.snapshot_bytes().expect("before");
            let reopened = manager.get_session(&session.id).expect("reopen");
            assert!(
                reopened
                    .append(&Message::User {
                        content: "must not append".into()
                    })
                    .is_err()
            );
            assert!(inspect(&reopened).is_err());
            assert_eq!(before, reopened.snapshot_bytes().expect("unchanged"));
        }
    }

    #[test]
    fn keep_zero_recovery_excludes_unrelated_zero_retention_session() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(0).expect("selected compact");
        let other = manager.create_session("archive", "").expect("other");
        append_users(&other, 1000, 1205);
        other.compact_archived(0).expect("other compact");
        assert_eq!(inspect(&session).expect("selected").entries().len(), 205);
        assert_eq!(
            inspect(&other).expect("other").entries()[0].content,
            "fact 1000"
        );
    }

    #[test]
    fn keep_zero_without_append_preserves_tail_and_rejects_wrong_commitment() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        let original = session.read_entries().expect("original");
        session.compact_archived(0).expect("first");
        session.compact_archived(0).expect("no append second");
        assert_eq!(inspect(&session).expect("recover").entries().len(), 205);
        let db = session
            .file_path
            .parent()
            .expect("dir")
            .join("history-bindings.sqlite");
        let connection = crate::durable::open_bindings(&db).expect("db");
        connection
            .execute(
                "UPDATE compaction_zero_locators SET tail=?1",
                [&original[0].id],
            )
            .expect("wrong tail fixture");
        let reopened = manager.get_session(&session.id).expect("restart");
        let before = reopened.snapshot_bytes().expect("before");
        assert!(inspect(&reopened).is_err());
        assert!(
            reopened
                .append(&Message::User {
                    content: "no".into()
                })
                .is_err()
        );
        assert_eq!(before, reopened.snapshot_bytes().expect("unchanged"));
    }

    #[test]
    fn keep_zero_marker_archive_and_suffix_tampering_are_rejected() {
        for corruption in ["marker", "archive", "suffix"] {
            let (_root, manager, session) = fixture();
            append_users(&session, 0, 205);
            session.compact_archived(0).expect("compact");
            match corruption {
                "marker" => {
                    let text = std::fs::read_to_string(&session.file_path).expect("head");
                    std::fs::write(&session.file_path, text.replace("\t2:{}\n", "\t3:{ }\n"))
                        .expect("change raw marker");
                }
                "archive" => {
                    let locator = crate::compaction_locator::load(
                        &session.file_path,
                        Instant::now() + Duration::from_secs(5),
                        &|| false,
                    )
                    .expect("locator")
                    .remove(0);
                    std::fs::write(
                        session
                            .file_path
                            .parent()
                            .expect("dir")
                            .join(locator.archive),
                        b"broken archive",
                    )
                    .expect("corrupt archive");
                }
                _ => {
                    let mut entry = session.read_entries().expect("marker").remove(0);
                    entry.id = uuid::Uuid::new_v4().to_string();
                    entry.parent_id = None;
                    entry.role = "user".into();
                    entry.content = "unrelated suffix".into();
                    session
                        .store
                        .append_entry(&session.file_path, &entry)
                        .expect("inject wrong suffix");
                }
            }
            let reopened = manager.get_session(&session.id).expect("restart");
            assert!(inspect(&reopened).is_err(), "{corruption}");
        }
    }

    #[test]
    fn keep_zero_then_keep_one_then_zero_preserves_cold_append_order() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(0).expect("first zero");
        let reopened = manager.get_session(&session.id).expect("restart");
        append_users(&reopened, 205, 210);
        reopened.compact_archived(1).expect("retain one");
        let reopened = manager.get_session(&session.id).expect("restart after one");
        append_users(&reopened, 210, 212);
        reopened.compact_archived(0).expect("zero again");
        let reopened = manager
            .get_session(&session.id)
            .expect("restart after zero");
        append_users(&reopened, 212, 213);
        let evidence = inspect(&reopened).expect("mixed recovery");
        assert_eq!(evidence.entries().len(), 213);
        for (i, entry) in evidence.entries().iter().enumerate() {
            assert_eq!(entry.content, format!("fact {i}"));
        }
    }

    #[test]
    fn keep_zero_crash_child() {
        let Some(path) = std::env::var_os("TALOS_TEST_ZERO_HEAD") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let id = uuid::Uuid::parse_str(path.file_stem().expect("stem").to_str().expect("id"))
            .expect("uuid");
        let _lease = crate::SessionRuntimeLease::acquire(&path).expect("lease");
        let session = Session::new(id, "archive".into(), String::new(), path);
        session
            .compact_archived(0)
            .expect("compact reaches crash probe");
        panic!("expected crash probe did not run");
    }

    #[test]
    fn keep_zero_process_crash_before_and_after_head_publication_recovers() {
        for phase in ["zero_registered", "zero_published"] {
            let (_root, manager, session) = fixture();
            append_users(&session, 0, 205);
            let original_bytes = session.snapshot_bytes().expect("original bytes");
            let original = session.read_entries().expect("original entries");
            let mut child =
                std::process::Command::new(std::env::current_exe().expect("test executable"))
                    .args(["--exact", "archive_recovery::tests::keep_zero_crash_child"])
                    .env("TALOS_TEST_ZERO_HEAD", &session.file_path)
                    .env("TALOS_TEST_FORK_CRASH_PHASE", phase)
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .expect("spawn child");
            let deadline = Instant::now() + Duration::from_secs(15);
            let status = loop {
                if let Some(status) = child.try_wait().expect("wait") {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("crash child timed out");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(42), "{phase}");
            let reopened = manager.get_session(&session.id).expect("restart");
            if phase == "zero_registered" {
                assert_eq!(reopened.snapshot_bytes().expect("old head"), original_bytes);
            } else {
                assert_eq!(reopened.read_entries().expect("new head").len(), 1);
            }
            let restored = inspect(&reopened).expect("recover after process death");
            assert_eq!(restored.entries().len(), 205);
            assert_eq!(
                restored.entries().last().expect("tail").id,
                original[204].id
            );
            append_users(&reopened, 205, 206);
            let entries = reopened.read_entries().expect("continued");
            assert_eq!(
                entries.last().expect("new").parent_id.as_deref(),
                Some(original[204].id.as_str())
            );
            assert_eq!(
                inspect(&reopened)
                    .expect("continued recovery")
                    .entries()
                    .len(),
                206
            );
        }
    }

    #[test]
    fn real_repeated_compaction_reconstructs_tool_boundary_and_excludes_other_session() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 154);
        session
            .append(&Message::Assistant {
                content: "inspect".into(),
                reasoning: None,
                tool_calls: vec![ToolCall {
                    id: "boundary-call".into(),
                    name: "read".into(),
                    input: serde_json::json!({"path":"fixture"}),
                }],
            })
            .expect("call");
        session
            .append(&Message::Tool {
                result: MessageToolResult {
                    tool_use_id: "boundary-call".into(),
                    content: "boundary result".into(),
                    is_error: false,
                },
            })
            .expect("result");
        append_users(&session, 156, 205);
        let first = session.read_entries().expect("first");
        session.compact_archived(50).expect("first archive");
        assert_eq!(session.read_entries().expect("head")[1].id, first[155].id);
        append_users(&session, 205, 360);
        let second = session.read_entries().expect("second");
        session.compact_archived(50).expect("second archive");
        let other = manager.create_session("archive", "").expect("other");
        append_users(&other, 0, 205);
        other
            .compact_archived(50)
            .expect("unrelated archive in same directory");
        let reopened = manager.get_session(&session.id).expect("restart");
        let before = reopened.snapshot_bytes().expect("before");
        let evidence = inspect(&reopened).expect("unique recovery");
        assert_eq!(evidence.entries.len(), 360);
        assert_eq!(evidence.sources.len(), 3);
        assert_eq!(evidence.excluded_archives, 1);
        assert_eq!(evidence.removed_placeholders, 2);
        let expected: Vec<_> = first
            .iter()
            .chain(second[51..].iter())
            .map(|entry| entry.id.as_str())
            .collect();
        assert_eq!(
            evidence
                .entries
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        let mut decoder = crate::jsonl::HistoryMessageDecoder::default();
        let messages: Vec<_> = evidence
            .entries
            .into_iter()
            .filter_map(|entry| decoder.decode(entry))
            .collect();
        assert!(messages.iter().any(|(_, message)| matches!(message, Message::Tool { result } if result.tool_use_id == "boundary-call" && result.content == "boundary result")));
        assert_eq!(reopened.snapshot_bytes().expect("after"), before);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_archive_replacement_during_validation_revokes_evidence() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let evidence = inspect(&session).expect("prepared recovery");
        let archive = evidence
            .sources
            .iter()
            .find(|source| source.path != session.file_path)
            .expect("selected archive")
            .path
            .clone();
        let bytes = std::fs::read(&archive).expect("original bytes");
        let replacement = archive.with_extension("replacement");
        std::fs::write(&replacement, &bytes).expect("replacement bytes");
        VALIDATION_FINISH_OBSERVER.with(|observer| {
            *observer.borrow_mut() = Some(Box::new(move || {
                // Identical contents do not preserve the captured file identity.
                // The replacement occurs after source reads, before publication.
                std::fs::rename(replacement, archive).expect("atomic replacement");
            }));
        });
        assert!(
            evidence
                .check_supporting_sources(Instant::now() + Duration::from_secs(5))
                .is_err(),
            "a replaced source must revoke the captured evidence"
        );
        assert_eq!(
            inspect(&session).expect("fresh recovery").entries().len(),
            205
        );
    }

    #[test]
    fn compaction_preserves_noncanonical_retained_record_bytes() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        let original = std::fs::read_to_string(&session.file_path).expect("original");
        let altered = original.replace("\t2:{}\n", "\t3:{ }\n");
        assert_ne!(original, altered);
        std::fs::write(&session.file_path, &altered).expect("legacy encoding");
        session.compact_archived(50).expect("real compaction");
        let evidence = inspect(&session).expect("raw overlap remains valid");
        assert_eq!(evidence.entries().len(), 205);
        assert!(
            evidence
                .raw_records
                .iter()
                .all(|raw| raw.ends_with(b"\t3:{ }\n"))
        );
        append_users(&session, 205, 210);
        session.compact_archived(50).expect("repeat compaction");
        assert_eq!(
            inspect(&session).expect("repeated overlap").entries().len(),
            210
        );
    }

    #[test]
    fn fork_preparation_preserves_original_encoding_and_inclusive_archived_cutoff() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        // Noncanonical but valid metadata in an old record must survive exactly;
        // re-encoding a SessionEntry would silently normalize these bytes.
        let original = std::fs::read_to_string(&session.file_path).expect("original");
        let altered = original.replacen("\t2:{}\n", "\t3:{ }\n", 1);
        assert_ne!(
            altered, original,
            "fixture changes one raw metadata encoding"
        );
        std::fs::write(&session.file_path, altered).expect("legacy encoding");
        let mut remaining = MAX_BYTES;
        let mut count = 0;
        let before_records = {
            let before = read_file(
                &session.file_path,
                false,
                &mut remaining,
                &mut count,
                Instant::now() + Duration::from_secs(5),
                &|| false,
            )
            .expect("original records");
            before.records
        };
        session.compact_archived(50).expect("real archive");
        append_users(&session, 205, 210);
        let evidence = inspect(&session).expect("recovery");
        let prefix = evidence
            .original_record_prefix(
                &before_records[19].0.id,
                Instant::now() + Duration::from_secs(5),
                || false,
            )
            .expect("archived cutoff");
        assert_eq!(prefix.len(), 20);
        for (raw, (_, original)) in prefix.iter().zip(&before_records) {
            assert_eq!(raw, original);
        }
        assert!(prefix[0].ends_with(b"\t3:{ }\n"));
        let tail = evidence.entries().last().expect("tail");
        assert_eq!(
            evidence
                .original_record_prefix(&tail.id, Instant::now() + Duration::from_secs(5), || false)
                .expect("tip")
                .len(),
            210
        );
        assert!(
            evidence
                .original_record_prefix(
                    "not-an-entry",
                    Instant::now() + Duration::from_secs(5),
                    || false
                )
                .is_err()
        );
        assert!(
            evidence
                .original_record_prefix(&tail.id, Instant::now() + Duration::from_secs(5), || true)
                .is_err()
        );
        assert!(
            evidence
                .original_record_prefix(&tail.id, Instant::now(), || false)
                .is_err()
        );
        append_users(&session, 210, 211);
        assert!(
            evidence
                .original_record_prefix(&tail.id, Instant::now() + Duration::from_secs(5), || false)
                .is_err(),
            "stale capture cannot prepare a fork"
        );
    }

    #[test]
    fn retained_archive_evidence_does_not_block_unrelated_session_append() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let other = manager.create_session("archive", "").expect("other");
        append_users(&other, 0, 1);
        let evidence = inspect(&session).expect("prepared recovery");
        // Keep the evidence alive, just as History does during provider work.
        // An unrelated live head participates in the ownership inventory but
        // must not be write-locked for the lifetime of another model turn.
        append_users(&other, 1, 2);
        assert_eq!(other.read_entries().expect("other persisted").len(), 2);
        assert_eq!(evidence.entries().len(), 205);
        // Changed ownership evidence cannot continue authorizing disclosure.
        assert!(
            evidence
                .check_supporting_sources(Instant::now() + Duration::from_secs(5))
                .is_err()
        );
    }

    #[test]
    fn retired_head_revokes_prepared_supporting_evidence_without_changing_files() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let database = session
            .file_path
            .parent()
            .expect("directory")
            .join("history-bindings.sqlite");
        crate::reserve_session_identity(&database, session.id).expect("reserve");
        let evidence = inspect(&session).expect("prepared evidence");
        let before = std::fs::read(&session.file_path).expect("head");
        let deadline = Instant::now() + Duration::from_secs(5);
        evidence
            .check_supporting_sources(deadline)
            .expect("active evidence");
        crate::retire_session_identity(&database, session.id).expect("retire");
        assert_eq!(
            std::fs::read(&session.file_path).expect("unchanged head"),
            before
        );
        assert!(evidence.check_supporting_sources(deadline).is_err());
        assert!(evidence.check_all_sources(deadline).is_err());
    }

    #[test]
    fn retained_archive_evidence_allows_selected_head_append_without_extending_history() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let evidence = inspect(&session).expect("prepare");
        append_users(&session, 205, 206);
        assert_eq!(evidence.entries().len(), 205);
        let deadline = Instant::now() + Duration::from_secs(5);
        evidence
            .check_supporting_sources(deadline)
            .expect("unchanged supporting sources");
        assert!(evidence.check_all_sources(deadline).is_err());
        assert_eq!(
            inspect(&session).expect("new evidence").entries().len(),
            206
        );
    }

    #[cfg(windows)]
    #[test]
    fn released_archive_leases_require_content_even_when_metadata_matches() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let other = manager.create_session("archive", "").expect("other");
        append_users(&other, 0, 1);
        let mut evidence = inspect(&session).expect("prepare");
        let bytes = std::fs::read(&other.file_path).expect("original other head");
        let changed = String::from_utf8(bytes)
            .expect("UTF-8")
            .replace("fact 0", "fact 9");
        std::fs::write(&other.file_path, changed).expect("write after lease release");
        let source = crate::history_page::open_source(&other.file_path).expect("fresh lease");
        let stamp = crate::recovery_stamp::from_file(&source).expect("fresh stamp");
        // Native Windows has demonstrated equal metadata after equal-length
        // writes (ADR-096). Force that equality here so this regression doesn't
        // depend on filesystem timestamp granularity or scheduling.
        let (_, expected) = evidence
            .versions
            .iter_mut()
            .find(|(path, _)| path == &other.file_path)
            .expect("other head inventory");
        expected.stamp = (&stamp).into();
        drop(stamp);
        drop(source);
        assert!(
            evidence
                .check_supporting_sources(Instant::now() + Duration::from_secs(5))
                .is_err()
        );
    }

    #[test]
    fn newly_added_source_revokes_prepared_archive_evidence() {
        let root = tempfile::tempdir().expect("root");
        let manager = crate::SessionManager::with_dir(root.path().into());
        let session = manager.create_session("inventory", "").expect("session");
        append_users(&session, 0, 205);
        session.compact_archived(50).expect("archive");
        let evidence = inspect(&session).expect("prepare");
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        evidence.check_all_sources(deadline).expect("unchanged");
        // An unrelated database/lock sidecar is not transcript ownership evidence.
        // history-bindings.sqlite now carries immutable publication authority.
        std::fs::write(
            session.file_path.with_file_name("unrelated.sqlite"),
            b"sidecar",
        )
        .expect("sidecar");
        evidence
            .check_supporting_sources(deadline)
            .expect("unrelated sidecar");
        let child = session
            .file_path
            .with_file_name(format!("{}.tlog", uuid::Uuid::new_v4()));
        std::fs::copy(&session.file_path, &child).expect("new overlapping head");
        assert!(evidence.check_all_sources(deadline).is_err());
        assert!(evidence.check_supporting_sources(deadline).is_err());
        assert!(evidence.into_restored_history(deadline, || false).is_err());
    }

    #[test]
    fn original_legacy_parent_values_are_retained_without_relabeling() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 205);
        let mut entries = session.read_entries().expect("entries");
        for i in 2..entries.len() {
            entries[i].parent_id = Some(entries[0].id.clone());
        }
        crate::CompactTextSessionStore
            .replace_entries_atomically(&session.file_path, &entries)
            .expect("legacy writer fixture");
        session.compact_archived(50).expect("archive");
        let evidence = inspect(&session).expect("candidate");
        assert_eq!(evidence.entries.len(), 205);
        assert_eq!(evidence.entries[204].parent_id, Some(entries[0].id.clone()));
        assert_ne!(
            evidence.entries[204].parent_id,
            Some(entries[203].id.clone())
        );
    }

    fn archive(session: &Session, entries: &[SessionEntry]) -> PathBuf {
        let path = session
            .file_path
            .parent()
            .expect("parent")
            .join(format!("s-{}.tlog", uuid::Uuid::new_v4()));
        crate::CompactTextSessionStore
            .replace_entries_atomically(&path, entries)
            .expect("archive fixture");
        path
    }

    #[test]
    fn conflicting_stable_ids_and_ambiguous_branch_order_are_refused() {
        for conflict in [true, false] {
            let (_root, _manager, session) = fixture();
            append_users(&session, 0, 3);
            let entries = session.read_entries().expect("entries");
            let mut candidate = entries.clone();
            if conflict {
                candidate[1].content = "changed content under same ID".into();
            } else {
                candidate[1].id = uuid::Uuid::new_v4().to_string();
                candidate[1].content = "divergent branch".into();
            }
            archive(&session, &candidate);
            assert!(inspect(&session).is_err());
        }
    }

    #[test]
    fn shared_live_head_is_not_inferred_as_same_session() {
        let (_root, manager, session) = fixture();
        append_users(&session, 0, 3);
        let other = manager.create_session("archive", "").expect("other");
        let entries = session.read_entries().expect("entries");
        crate::CompactTextSessionStore
            .replace_entries_atomically(&other.file_path, &entries)
            .expect("fork copy");
        assert!(inspect(&session).is_err());
    }

    #[test]
    fn cancellation_corrupt_archive_and_missing_parent_fail_without_writes() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 3);
        let before = session.snapshot_bytes().expect("before");
        assert!(
            session
                .inspect_archive_recovery(Instant::now() + Duration::from_secs(5), || true)
                .is_err()
        );
        assert!(
            session
                .inspect_archive_recovery(Instant::now(), || false)
                .is_err()
        );
        let path = archive(&session, &session.read_entries().expect("entries"));
        std::fs::write(&path, b"E\tbroken").expect("corrupt");
        assert!(inspect(&session).is_err());
        assert_eq!(session.snapshot_bytes().expect("after"), before);
        let mut entries = session.read_entries().expect("entries");
        entries[0].parent_id = Some(uuid::Uuid::new_v4().to_string());
        std::fs::remove_file(path).expect("remove disposable archive");
        crate::CompactTextSessionStore
            .replace_entries_atomically(&session.file_path, &entries)
            .expect("missing parent fixture");
        assert!(inspect(&session).is_err());
    }

    #[cfg(feature = "archive-compress-zstd")]
    #[test]
    fn compressed_window_limit_rejects_tiny_frame_without_changing_sources() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 3);
        let before = session.snapshot_bytes().expect("before");
        let path = session
            .file_path
            .parent()
            .expect("directory")
            .join(format!("s-{}.tlog.zst", uuid::Uuid::new_v4()));
        // Zstandard frame: unknown content size, 128 MiB window, final empty raw block.
        // Its decoded output is empty, so a decoded-byte limit alone cannot reject it.
        let frame = [0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x88, 0x01, 0x00, 0x00];
        assert!(
            zstd::stream::decode_all(frame.as_slice())
                .expect("valid frame")
                .is_empty()
        );
        std::fs::write(&path, frame).expect("archive");
        assert!(
            inspect(&session).is_err(),
            "oversized decoder window must be refused"
        );
        assert_eq!(std::fs::read(path).expect("archive unchanged"), frame);
        assert_eq!(session.snapshot_bytes().expect("head unchanged"), before);
    }

    #[test]
    fn bounded_reader_refuses_expansion_past_budget() {
        let mut remaining = 16;
        let mut output = Vec::new();
        let mut input = std::io::Cursor::new(vec![b'x'; 17]);
        assert!(
            read_bounded(
                &mut input,
                &mut output,
                &mut remaining,
                Instant::now() + Duration::from_secs(1),
                &|| false
            )
            .is_err()
        );
        assert!(output.is_empty(), "oversized chunk is not retained");
    }

    #[cfg(unix)]
    #[test]
    fn archive_symlink_is_refused_without_reading_its_target() {
        let (_root, _manager, session) = fixture();
        append_users(&session, 0, 3);
        let alias = session
            .file_path
            .parent()
            .expect("parent")
            .join(format!("s-{}.tlog", uuid::Uuid::new_v4()));
        std::os::unix::fs::symlink(&session.file_path, alias).expect("alias");
        assert!(inspect(&session).is_err());
    }
}
