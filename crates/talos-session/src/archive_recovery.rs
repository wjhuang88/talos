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
    entries: Vec<SessionEntry>,
    raw_records: Vec<Vec<u8>>,
    sources: Vec<ArchiveRecoverySource>,
    excluded_archives: usize,
    removed_placeholders: usize,
    versions: Vec<(PathBuf, ArchiveFileVersion)>,
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
    /// Check all captured file versions before a host publishes restoration or
    /// query evidence. No decoded records are published on a stale source.
    pub fn check_all_sources(&self, deadline: Instant) -> Result<(), SessionError> {
        self.check_current(deadline, &|| false)
    }
    /// Revalidate supporting archive/other-head versions. The query owner must
    /// separately validate the selected head's captured prefix, allowing only
    /// verified append-only growth there. This alone does not authorize queries.
    pub fn check_supporting_sources(&self, deadline: Instant) -> Result<(), SessionError> {
        self.check_versions(self.versions.iter().skip(1), deadline, &|| false)
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

    fn check_current(
        &self,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        self.check_versions(self.versions.iter(), deadline, stopped)
    }

    fn check_versions<'a>(
        &self,
        versions: impl Iterator<Item = &'a (PathBuf, ArchiveFileVersion)>,
        deadline: Instant,
        stopped: &impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        let versions: Vec<_> = versions.collect();
        let head = self
            .versions
            .first()
            .ok_or_else(|| invalid("missing selected head"))?;
        let directory = head
            .0
            .parent()
            .ok_or_else(|| invalid("missing directory"))?;
        let mut expected_paths: BTreeSet<_> =
            self.versions.iter().map(|(path, _)| path.clone()).collect();
        for (index, item) in std::fs::read_dir(directory)?.enumerate() {
            check(deadline, stopped)?;
            if index >= MAX_FILES {
                return Err(invalid("directory bound exceeded"));
            }
            let path = item?.path();
            if recovery_source_kind(&path).is_some() && !expected_paths.remove(&path) {
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
            let file =
                crate::history_page::open_source(path).map_err(|e| invalid(&e.to_string()))?;
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
                if recovery_source_kind(&path).is_some() && !remaining_paths.remove(&path) {
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
            let file =
                crate::history_page::open_source(path).map_err(|e| invalid(&e.to_string()))?;
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
    let archive = name
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
        let mut selected = BTreeSet::from([0]);
        let mut known: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        insert_records(&files[0], &mut known)?;
        loop {
            let mut added = false;
            for (index, file) in files.iter().enumerate().skip(1) {
                check(deadline, stopped)?;
                if selected.contains(&index) || !file.archive {
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
        for file in files.iter().skip(1).filter(|file| !file.archive) {
            if file
                .records
                .iter()
                .any(|(entry, _)| known.contains_key(&entry.id))
            {
                return Err(invalid("candidate overlaps another live session"));
            }
        }
        let mut records = BTreeMap::new();
        let mut successors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut incoming: BTreeMap<String, usize> = BTreeMap::new();
        let mut placeholders = BTreeSet::new();
        for &index in &selected {
            let mut previous: Option<String> = None;
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
        // Recheck every inspected file: an excluded head/archive must not change
        // while its absence of overlap is used as evidence.
        for file in &files {
            check(deadline, stopped)?;
            let current = crate::history_page::open_source(&file.source.path)
                .map_err(|e| invalid(&e.to_string()))?;
            let stamp =
                crate::recovery_stamp::from_file(&current).map_err(|e| invalid(&e.to_string()))?;
            if !crate::recovery_seal::same_file_version(&file.stamp, &stamp)
                .map_err(|e| invalid(&e.to_string()))?
            {
                return Err(invalid("source changed"));
            }
        }
        let raw_records = ordered
            .iter()
            .map(|entry| {
                known
                    .remove(&entry.id)
                    .ok_or_else(|| invalid("missing original record bytes"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ArchiveRecoveryEvidence {
            entries: ordered,
            raw_records,
            sources: selected.iter().map(|&i| files[i].source.clone()).collect(),
            excluded_archives: files
                .iter()
                .enumerate()
                .filter(|(i, f)| f.archive && !selected.contains(i))
                .count(),
            removed_placeholders: placeholders.len(),
            versions: files
                .into_iter()
                .map(ArchiveFileVersion::from_records)
                .collect(),
        })
    }
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
    let mut file = crate::history_page::open_source(path).map_err(|e| invalid(&e.to_string()))?;
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
    let mut offset = 0;
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
    while offset < decoded.len() {
        check(deadline, stopped)?;
        if matches!(decoded[offset], b'\n' | b'\r') {
            offset += 1;
            continue;
        }
        let (entry, length) = crate::history_page::tlog_record(&decoded[offset..])
            .map_err(|e| invalid(&e.to_string()))?
            .ok_or_else(|| invalid("incomplete record"))?;
        *count += 1;
        if *count > MAX_ENTRIES {
            return Err(invalid("record bound exceeded"));
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
        let before = read_file(
            &session.file_path,
            false,
            &mut remaining,
            &mut count,
            Instant::now() + Duration::from_secs(5),
            &|| false,
        )
        .expect("original records");
        let before_records = before.records;
        drop(before.stamp);
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
        // A database/lock sidecar is not transcript ownership evidence.
        std::fs::write(
            session.file_path.with_file_name("history-bindings.sqlite"),
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
