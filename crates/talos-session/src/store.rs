//! Session storage abstraction.
//!
//! The [`SessionStore`] trait separates session entry persistence from the [`Session`] type,
//! enabling future compact text format support alongside JSONL.

use crate::{SessionEntry, SessionError, SessionInfo, SessionMetadata};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use talos_core::message::Message;
use uuid::Uuid;

pub use crate::compact_text::CompactTextSessionStore;

// Retry only acquisition/publication operations that have not written bytes.
// Windows recovery read leases deny writes/deletion; waiting here also covers
// independently opened owners/processes without weakening the read lease.
// Never put write_all/flush or an entire append/commit inside this helper.
pub(crate) fn retry_sharing_conflict<T>(
    operation: impl FnMut() -> std::io::Result<T>,
) -> std::io::Result<T> {
    #[cfg(windows)]
    {
        retry_sharing_conflict_until(
            operation,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
        )
    }
    #[cfg(not(windows))]
    {
        let mut operation = operation;
        operation()
    }
}

// A Windows target read lease can make rename report ACCESS_DENIED rather
// than SHARING_VIOLATION. Acquire write access without modifying bytes first:
// this waits at the reliable sharing boundary and excludes new recovery leases
// through publication. Default Windows sharing permits our own atomic rename.
pub(crate) fn publish_atomic_file(temporary: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        retry_sharing_conflict(|| {
            let target = open_publication_target(destination)?;
            match fs::rename(temporary, destination) {
                Err(error)
                    if target.is_none()
                        && error.raw_os_error()
                            == Some(windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED as i32) =>
                {
                    // A target may have appeared since the NotFound observation.
                    // Retry only after acquiring that current target; its real
                    // sharing error uses the same outer deadline. ACL errors do not.
                    let Some(_target) = open_publication_target(destination)? else {
                        return Err(error);
                    };
                    fs::rename(temporary, destination)
                }
                result => result,
            }
        })
    }
    #[cfg(not(windows))]
    fs::rename(temporary, destination)
}

#[cfg(windows)]
fn open_publication_target(path: &Path) -> std::io::Result<Option<fs::File>> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    let file = match OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(std::io::Error::other(
            "publication target is not a regular file",
        ));
    }
    crate::recovery_stamp::verify_local_volume(&file)
        .map_err(|_| std::io::Error::other("publication target is not local storage"))?;
    Ok(Some(file))
}

#[cfg(windows)]
fn retry_sharing_conflict_until<T>(
    mut operation: impl FnMut() -> std::io::Result<T>,
    deadline: std::time::Instant,
) -> std::io::Result<T> {
    loop {
        match operation() {
            Err(error)
                if error.raw_os_error()
                    == Some(windows_sys::Win32::Foundation::ERROR_SHARING_VIOLATION as i32) =>
            {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(error);
                }
                #[cfg(test)]
                SHARING_CONFLICT_OBSERVER.with(|observer| {
                    if let Some(observer) = observer.borrow_mut().take() {
                        observer();
                    }
                });
                std::thread::sleep(remaining.min(std::time::Duration::from_millis(10)));
            }
            result => return result,
        }
    }
}

#[cfg(all(test, windows))]
thread_local! {
    static SHARING_CONFLICT_OBSERVER: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

/// Trait abstracting session entry persistence.
///
/// Enables different storage formats (JSONL, compact text, segment chains)
/// while keeping the `Session` and `SessionManager` types format-agnostic.
pub trait SessionStore: Send + Sync + std::fmt::Debug {
    /// Read all entries from a session file.
    fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError>;

    /// Append a single entry to a session file.
    fn append_entry(&self, file_path: &Path, entry: &SessionEntry) -> Result<(), SessionError>;

    /// Replace a complete log using a temporary sibling file and atomic rename.
    ///
    /// Used only by durable turn commits, where all entries for a successful
    /// turn must become visible together.
    fn replace_entries_atomically(
        &self,
        file_path: &Path,
        entries: &[SessionEntry],
    ) -> Result<(), SessionError>;

    /// Atomically replace a transcript with new entries followed by a retained
    /// suffix of the caller's original snapshot. The caller owns the write lease.
    /// Custom stores retain their normal atomic replacement behavior; the built-in
    /// TLOG store additionally preserves the retained records' original encoding.
    fn replace_with_retained_suffix(
        &self,
        file_path: &Path,
        original: &[SessionEntry],
        retain_from: usize,
        prepended: &[SessionEntry],
    ) -> Result<(), SessionError> {
        let suffix = original
            .get(retain_from..)
            .ok_or_else(|| SessionError::ParseError("invalid retained record cutoff".into()))?;
        let mut replacement = prepended.to_vec();
        replacement.extend_from_slice(suffix);
        self.replace_entries_atomically(file_path, &replacement)
    }

    /// Read the ID of the last entry in the file.
    fn read_last_entry_id(&self, file_path: &Path) -> Option<String>;

    /// Scan a session file for message count and last preview text.
    fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError>;

    /// Read the session file as raw bytes.
    fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError>;

    /// Built-in encoding available to bounded message recovery. Custom stores
    /// default to unavailable; this never falls back to a whole-file read.
    fn history_encoding(&self) -> Option<crate::history_page::HistoryEncoding> {
        None
    }

    /// The file extension for this store's format (e.g., `"jsonl"`).
    fn file_extension(&self) -> &'static str;
}

/// JSONL-based session store implementation.
///
/// This is the default store, preserving backward compatibility with
/// existing `.jsonl` session files.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonlSessionStore;

impl SessionStore for JsonlSessionStore {
    fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
        read_entries_from_path(file_path)
    }

    fn append_entry(&self, file_path: &Path, entry: &SessionEntry) -> Result<(), SessionError> {
        let line =
            serde_json::to_string(entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;

        if !file_path.exists()
            && let Some(parent) = file_path.parent()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = retry_sharing_conflict(|| {
            OpenOptions::new().create(true).append(true).open(file_path)
        })?;
        writeln!(file, "{line}")?;

        Ok(())
    }

    fn replace_entries_atomically(
        &self,
        file_path: &Path,
        entries: &[SessionEntry],
    ) -> Result<(), SessionError> {
        let parent = file_path.parent().ok_or_else(|| {
            SessionError::ParseError("session file has no parent directory".into())
        })?;
        fs::create_dir_all(parent)?;
        let temporary = temporary_sibling(file_path);
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        for entry in entries {
            let line = serde_json::to_string(entry)
                .map_err(|error| SessionError::InvalidJson(error.to_string()))?;
            writeln!(file, "{line}")?;
        }
        file.sync_all()?;
        drop(file);
        publish_atomic_file(&temporary, file_path)?;
        Ok(())
    }

    fn read_last_entry_id(&self, file_path: &Path) -> Option<String> {
        read_last_entry_id(file_path)
    }

    fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError> {
        let file = fs::File::open(file_path)?;
        let metadata = file.metadata()?;
        let timestamp = metadata
            .modified()
            .ok()
            .map(chrono::DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        let id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);

        let reader = BufReader::new(file);
        let mut count = 0;
        let mut last_preview = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
                count += 1;
                last_preview = crate::jsonl::preview_text(&entry.content);
                continue;
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
                && value.get("type").and_then(|t| t.as_str()) == Some("message")
                && let Some(data) = value.get("data")
                && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
            {
                count += 1;
                let (_, content) = crate::jsonl::message_parts(&msg);
                last_preview = crate::jsonl::preview_text(&content);
            }
        }

        Ok(SessionInfo {
            id,
            project: String::new(),
            workspace_root: String::new(),
            last_message_preview: last_preview,
            timestamp,
            message_count: count,
        })
    }

    fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError> {
        std::fs::read(file_path).map_err(SessionError::IoError)
    }

    fn history_encoding(&self) -> Option<crate::history_page::HistoryEncoding> {
        Some(crate::history_page::HistoryEncoding::Jsonl)
    }

    fn file_extension(&self) -> &'static str {
        "jsonl"
    }
}

/// Returns a UUID-named temporary file next to a session file.
pub(crate) fn temporary_sibling(file_path: &Path) -> PathBuf {
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("session");
    file_path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()))
}

fn read_last_entry_id(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    if file_size == 0 {
        return None;
    }
    let read_size = std::cmp::min(file_size, 8192) as usize;
    let seek_pos = file_size.saturating_sub(read_size as u64);
    file.seek(SeekFrom::Start(seek_pos)).ok()?;
    let mut buf = vec![0u8; read_size];
    file.read_exact(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    let last_line = text.lines().rev().find(|l| !l.is_empty())?;
    let entry: SessionEntry = serde_json::from_str(last_line).ok()?;
    Some(entry.id)
}

fn read_entries_from_path(path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut synthetic_counter: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Some(entry) = decode_jsonl_history_line(&line, &mut synthetic_counter) {
            entries.push(entry);
        }
        // Invalid lines are silently skipped (crash-safety guarantee)
    }

    Ok(entries)
}

// Compatibility decoder for message restoration only. Synthetic IDs and skipped
// corrupt lines must never become durable search/projection authorization.
pub(crate) fn decode_jsonl_history_line(
    line: &str,
    synthetic_counter: &mut u64,
) -> Option<SessionEntry> {
    if let Ok(entry) = serde_json::from_str::<SessionEntry>(line) {
        return Some(entry);
    }
    let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
    if value.get("type").and_then(|t| t.as_str()) != Some("message") {
        return None;
    }
    let msg = serde_json::from_value::<Message>(value.get("data")?.clone()).ok()?;
    let (role, content) = crate::jsonl::message_parts(&msg);
    let id = format!("synthetic-{synthetic_counter}");
    let parent_id = synthetic_counter
        .checked_sub(1)
        .map(|n| format!("synthetic-{n}"));
    *synthetic_counter += 1;
    Some(SessionEntry {
        id,
        parent_id,
        timestamp: Utc::now(),
        role,
        content,
        metadata: SessionMetadata::default(),
    })
}

#[cfg(all(test, windows))]
mod recovery_writer_wait_tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    // The notification comes from the actual OS sharing-conflict branch, not
    // a sleep intended to guess when the writer has reached the protected file.
    fn wait_for_conflict<T: Send + std::fmt::Debug + 'static>(
        operation: impl FnOnce() -> T + Send + 'static,
    ) -> (std::thread::JoinHandle<()>, mpsc::Receiver<T>) {
        let (conflict_tx, conflict_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let task = std::thread::spawn(move || {
            SHARING_CONFLICT_OBSERVER.with(|slot| {
                *slot.borrow_mut() = Some(Box::new(move || {
                    let _ = conflict_tx.send(());
                }));
            });
            let _ = result_tx.send(operation());
        });
        if let Err(error) = conflict_rx.recv_timeout(Duration::from_secs(3)) {
            let result = result_rx.recv_timeout(Duration::from_secs(1));
            panic!("real sharing conflict not reached: {error:?}; writer result: {result:?}");
        }
        assert!(
            matches!(result_rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "writer must wait instead of returning a persistence failure"
        );
        (task, result_rx)
    }

    #[test]
    fn independently_opened_append_waits_for_recovery_lease_without_duplicate_records() {
        for extension in ["tlog", "jsonl"] {
            let root = tempfile::tempdir().expect("root");
            let id = Uuid::new_v4();
            let path = root.path().join(format!("{id}.{extension}"));
            let original = crate::Session::new(id, "fixture".into(), String::new(), path.clone());
            original
                .append(&Message::User {
                    content: "original".into(),
                })
                .expect("seed");
            let before = original.read_entries().expect("entries");
            let independent =
                crate::Session::new(id, "fixture".into(), String::new(), path.clone());
            let lease = crate::history_page::open_source(&path).expect("recovery lease");
            let (task, result) = wait_for_conflict(move || {
                independent.append(&Message::User {
                    content: "once".into(),
                })
            });
            assert_eq!(original.read_entries().expect("still original").len(), 1);
            drop(lease);
            result
                .recv_timeout(Duration::from_secs(3))
                .expect("writer finished")
                .expect("append succeeded");
            task.join().expect("writer task");
            let entries = original.read_entries().expect("after");
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[1].parent_id.as_deref(), Some(before[0].id.as_str()));
            assert_eq!(entries[1].content, "once");
        }
    }

    #[test]
    fn delayed_header_acquisition_never_truncates_an_initialized_transcript() {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("header.tlog");
        std::fs::write(&path, []).expect("empty source");
        let lease = crate::history_page::open_source(&path).expect("read lease");
        let (conflict_tx, conflict_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let writer_path = path.clone();
        let task = std::thread::spawn(move || {
            SHARING_CONFLICT_OBSERVER.with(|slot| {
                *slot.borrow_mut() = Some(Box::new(move || {
                    conflict_tx.send(()).expect("real conflict");
                    resume_rx
                        .recv_timeout(Duration::from_secs(3))
                        .expect("resume acquisition");
                }));
            });
            let entry = SessionEntry {
                id: Uuid::new_v4().to_string(),
                parent_id: None,
                timestamp: Utc::now(),
                role: "user".into(),
                content: "delayed append".into(),
                metadata: Default::default(),
            };
            CompactTextSessionStore.append_entry(&writer_path, &entry)
        });
        conflict_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("header open conflicted");
        drop(lease);
        let seeded =
            crate::Session::new(Uuid::new_v4(), "seed".into(), String::new(), path.clone());
        seeded
            .append(&Message::User {
                content: "must survive".into(),
            })
            .expect("initialize before retry");
        let before = std::fs::read(&path).expect("initialized bytes");
        resume_tx.send(()).expect("resume writer");
        task.join()
            .expect("writer joined")
            .expect("append succeeded");
        assert!(std::fs::read(&path).expect("after").starts_with(&before));
        let entries = seeded.read_entries().expect("one header, both records");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, "must survive");
        assert_eq!(entries[1].content, "delayed append");
    }

    #[test]
    fn atomic_publication_coordinates_at_write_open_without_retrying_access_denied() {
        let root = tempfile::tempdir().expect("root");
        let target = root.path().join("target.tlog");
        let temporary = root.path().join("prepared.tlog");
        fs::write(&target, b"original").expect("target");
        fs::write(&temporary, b"replacement").expect("temporary");
        let lease = crate::history_page::open_source(&target).expect("read lease");
        assert_eq!(
            open_publication_target(&target)
                .expect_err("lease excludes writer")
                .raw_os_error(),
            Some(32)
        );
        let error =
            fs::rename(&temporary, &target).expect_err("held target blocks direct replacement");
        assert!(
            matches!(error.raw_os_error(), Some(5 | 32)),
            "unexpected native rename error: {error}"
        );
        assert_eq!(fs::read(&target).expect("unchanged"), b"original");
        drop(lease);
        let target_guard = open_publication_target(&target)
            .expect("write acquisition")
            .expect("existing target");
        assert!(
            crate::history_page::open_source(&target).is_err(),
            "publication excludes a new recovery lease"
        );
        fs::rename(&temporary, &target).expect("rename while holding delete-sharing write handle");
        drop(target_guard);
        assert_eq!(fs::read(&target).expect("published"), b"replacement");

        fs::write(&temporary, b"must not publish").expect("next temporary");
        let mut permissions = fs::metadata(&target).expect("metadata").permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&target, permissions.clone()).expect("deny writes");
        let error = publish_atomic_file(&temporary, &target).expect_err("readonly target refused");
        assert_eq!(error.raw_os_error(), Some(5));
        assert_eq!(fs::read(&target).expect("preserved"), b"replacement");
        assert_eq!(
            fs::read(&temporary).expect("unpublished bytes retained"),
            b"must not publish"
        );
        permissions.set_readonly(false);
        fs::set_permissions(&target, permissions).expect("restore fixture permissions");
    }

    #[test]
    fn durable_atomic_commit_waits_for_recovery_lease_and_reopens_once() {
        let root = tempfile::tempdir().expect("root");
        let original = crate::DurableSession::open_or_create(root.path(), "writer").expect("owner");
        original
            .commit_turn(
                "seed",
                &[Message::User {
                    content: "original".into(),
                }],
                &Default::default(),
            )
            .expect("seed");
        let independent = crate::DurableSession::open_or_create(root.path(), "writer")
            .expect("independent owner");
        let before = std::fs::read(original.file_path()).expect("before");
        let lease = crate::history_page::open_source(original.file_path()).expect("recovery lease");
        let (task, result) = wait_for_conflict(move || {
            independent.commit_turn(
                "next",
                &[Message::User {
                    content: "committed once".into(),
                }],
                &Default::default(),
            )
        });
        assert_eq!(
            std::fs::read(original.file_path()).expect("unchanged before publish"),
            before
        );
        drop(lease);
        result
            .recv_timeout(Duration::from_secs(3))
            .expect("writer finished")
            .expect("commit succeeded");
        task.join().expect("writer task");
        let reopened =
            crate::DurableSession::open_or_create(root.path(), "writer").expect("reopen");
        let messages = reopened.read_messages().expect("messages");
        assert_eq!(messages.iter().filter(|message| matches!(message, Message::User { content } if content == "committed once")).count(), 1);
        reopened
            .commit_turn(
                "next",
                &[Message::User {
                    content: "committed once".into(),
                }],
                &Default::default(),
            )
            .expect("idempotent replay");
        assert_eq!(
            reopened.read_messages().expect("replayed").len(),
            messages.len()
        );
    }

    #[test]
    fn permanent_sharing_conflict_is_bounded_and_other_errors_are_not_retried() {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("source.tlog");
        std::fs::write(&path, b"unchanged").expect("source");
        let _lease = crate::history_page::open_source(&path).expect("lease");
        let start = Instant::now();
        let error = retry_sharing_conflict_until(
            || OpenOptions::new().append(true).open(&path),
            start + Duration::from_millis(30),
        )
        .expect_err("bounded conflict");
        assert_eq!(
            error.raw_os_error(),
            Some(windows_sys::Win32::Foundation::ERROR_SHARING_VIOLATION as i32)
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(std::fs::read(&path).expect("unchanged"), b"unchanged");
        let mut attempts = 0;
        let result: std::io::Result<()> = retry_sharing_conflict_until(
            || {
                attempts += 1;
                Err(std::io::Error::from_raw_os_error(
                    windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED as i32,
                ))
            },
            Instant::now() + Duration::from_secs(5),
        );
        assert!(result.is_err());
        assert_eq!(attempts, 1, "permission failures are not transient leases");
    }
}
