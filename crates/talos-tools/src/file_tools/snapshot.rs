use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::FileToolError;

const SNAPSHOT_TTL: Duration = Duration::from_secs(15 * 60);
const MAX_SNAPSHOTS: usize = 64;
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_FILE_LINES: usize = 50_000;
const MAX_REGISTRY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LineSpan {
    pub(crate) content_start: usize,
    pub(crate) content_end: usize,
    pub(crate) full_end: usize,
}

#[derive(Clone)]
pub(crate) struct SnapshotRecord {
    pub(crate) path: PathBuf,
    pub(crate) file_revision: [u8; 32],
    pub(crate) line_digests: Vec<[u8; 32]>,
    pub(crate) created_at: Instant,
    accounted_bytes: usize,
}

/// Bounded Runtime-local registry for model-private file snapshots.
///
/// The registry is intentionally memory-only. Clones share the same bounded
/// state so read, write, edit, and delete tools can coordinate invalidation.
#[derive(Clone)]
pub struct FileSnapshotRegistry {
    state: Arc<Mutex<RegistryState>>,
    next_id: Arc<AtomicU64>,
    namespace: Arc<str>,
}

struct RegistryState {
    records: HashMap<String, SnapshotRecord>,
    order: VecDeque<String>,
    accounted_bytes: usize,
    path_locks: HashMap<PathBuf, Weak<Mutex<()>>>,
}

impl Default for FileSnapshotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSnapshotRegistry {
    /// Creates an empty bounded registry.
    #[must_use]
    pub fn new() -> Self {
        let uuid = Uuid::new_v4().simple().to_string();
        Self {
            state: Arc::new(Mutex::new(RegistryState {
                records: HashMap::new(),
                order: VecDeque::new(),
                accounted_bytes: 0,
                path_locks: HashMap::new(),
            })),
            next_id: Arc::new(AtomicU64::new(1)),
            namespace: Arc::from(&uuid[..8]),
        }
    }

    pub(crate) fn capture(
        &self,
        path: &Path,
        bytes: &[u8],
    ) -> Result<(String, Vec<String>), FileToolError> {
        if bytes.len() > MAX_FILE_BYTES {
            return Err(FileToolError::SnapshotLimit(format!(
                "file exceeds {MAX_FILE_BYTES} byte snapshot limit"
            )));
        }
        std::str::from_utf8(bytes)
            .map_err(|_| FileToolError::BinaryFile(path.display().to_string()))?;
        let spans = line_spans(bytes);
        if spans.len() > MAX_FILE_LINES {
            return Err(FileToolError::SnapshotLimit(format!(
                "file exceeds {MAX_FILE_LINES} line snapshot limit"
            )));
        }

        let line_digests = spans
            .iter()
            .map(|span| digest(&bytes[span.content_start..span.content_end]))
            .collect::<Vec<_>>();
        let check_codes = line_digests
            .iter()
            .map(|value| format!("{:02x}", value[0]))
            .collect::<Vec<_>>();
        let id = format!(
            "s{}{}",
            self.namespace,
            to_base36(self.next_id.fetch_add(1, Ordering::Relaxed))
        );
        let accounted_bytes = path.as_os_str().len() + line_digests.len() * 32 + 64;
        if accounted_bytes > MAX_REGISTRY_BYTES {
            return Err(FileToolError::SnapshotLimit(
                "snapshot metadata exceeds registry budget".into(),
            ));
        }

        let record = SnapshotRecord {
            path: path.to_path_buf(),
            file_revision: digest(bytes),
            line_digests,
            created_at: Instant::now(),
            accounted_bytes,
        };
        let mut state = self.lock_state()?;
        state.evict_expired();
        while state.records.len() >= MAX_SNAPSHOTS
            || state.accounted_bytes.saturating_add(accounted_bytes) > MAX_REGISTRY_BYTES
        {
            if !state.evict_oldest() {
                break;
            }
        }
        state.accounted_bytes = state.accounted_bytes.saturating_add(accounted_bytes);
        state.order.push_back(id.clone());
        state.records.insert(id.clone(), record);
        Ok((id, check_codes))
    }

    pub(crate) fn get(&self, id: &str, path: &Path) -> Result<SnapshotRecord, FileToolError> {
        let mut state = self.lock_state()?;
        state.evict_expired();
        let record = state
            .records
            .get(id)
            .ok_or(FileToolError::SnapshotNotFound)?;
        if record.path != path {
            return Err(FileToolError::SnapshotPathMismatch);
        }
        Ok(record.clone())
    }

    pub(crate) fn invalidate_path(&self, path: &Path) -> Result<(), FileToolError> {
        let mut state = self.lock_state()?;
        let ids = state
            .records
            .iter()
            .filter(|(_, record)| record.path == path || record.path.starts_with(path))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            state.remove(&id);
        }
        Ok(())
    }

    pub(crate) fn path_lock(&self, path: &Path) -> Result<Arc<Mutex<()>>, FileToolError> {
        let mut state = self.lock_state()?;
        state.path_locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = state.path_locks.get(path).and_then(Weak::upgrade) {
            return Ok(lock);
        }
        let lock = Arc::new(Mutex::new(()));
        state
            .path_locks
            .insert(path.to_path_buf(), Arc::downgrade(&lock));
        Ok(lock)
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, RegistryState>, FileToolError> {
        self.state
            .lock()
            .map_err(|_| FileToolError::SnapshotRegistryUnavailable)
    }
}

impl RegistryState {
    fn evict_expired(&mut self) {
        let expired = self
            .records
            .iter()
            .filter(|(_, record)| record.created_at.elapsed() >= SNAPSHOT_TTL)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in expired {
            self.remove(&id);
        }
    }

    fn evict_oldest(&mut self) -> bool {
        while let Some(id) = self.order.pop_front() {
            if let Some(record) = self.records.remove(&id) {
                self.accounted_bytes = self.accounted_bytes.saturating_sub(record.accounted_bytes);
                return true;
            }
        }
        false
    }

    fn remove(&mut self, id: &str) {
        if let Some(record) = self.records.remove(id) {
            self.accounted_bytes = self.accounted_bytes.saturating_sub(record.accounted_bytes);
        }
        self.order.retain(|candidate| candidate != id);
    }
}

pub(crate) fn line_spans(bytes: &[u8]) -> Vec<LineSpan> {
    let mut spans = Vec::new();
    let mut start = 0usize;
    while start < bytes.len() {
        let newline = bytes[start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| start + offset);
        match newline {
            Some(index) => {
                let content_end = if index > start && bytes[index - 1] == b'\r' {
                    index - 1
                } else {
                    index
                };
                spans.push(LineSpan {
                    content_start: start,
                    content_end,
                    full_end: index + 1,
                });
                start = index + 1;
            }
            None => {
                spans.push(LineSpan {
                    content_start: start,
                    content_end: bytes.len(),
                    full_end: bytes.len(),
                });
                break;
            }
        }
    }
    spans
}

pub(crate) fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(crate) fn atomic_replace(
    path: &Path,
    bytes: &[u8],
    expected_revision: [u8; 32],
    path_lock: &Mutex<()>,
) -> Result<(), FileToolError> {
    let _guard = path_lock
        .lock()
        .map_err(|_| FileToolError::SnapshotRegistryUnavailable)?;
    verify_path_identity(path)?;
    let current = fs::read(path)?;
    verify_path_identity(path)?;
    if digest(&current) != expected_revision {
        return Err(FileToolError::FileRevisionMismatch);
    }

    let parent = path
        .parent()
        .ok_or_else(|| FileToolError::AtomicWriteFailed("target has no parent directory".into()))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let temp = parent.join(format!(".{file_name}.talos-{}.tmp", Uuid::new_v4()));
    let result = (|| -> Result<(), FileToolError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        file.set_permissions(fs::metadata(path)?.permissions())?;
        drop(file);
        verify_path_identity(path)?;
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|error| match error {
        FileToolError::FileRevisionMismatch | FileToolError::PathIdentityChanged => error,
        other => FileToolError::AtomicWriteFailed(other.to_string()),
    })
}

fn verify_path_identity(path: &Path) -> Result<(), FileToolError> {
    if path.canonicalize()? == path {
        Ok(())
    } else {
        Err(FileToolError::PathIdentityChanged)
    }
}

fn to_base36(mut value: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut output = Vec::new();
    loop {
        output.push(DIGITS[(value % 36) as usize] as char);
        value /= 36;
        if value == 0 {
            break;
        }
    }
    output.iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_spans_preserve_terminators_and_final_line() {
        let bytes = b"a\r\n\nlast";
        let spans = line_spans(bytes);
        assert_eq!(spans.len(), 3);
        assert_eq!(&bytes[spans[0].content_start..spans[0].content_end], b"a");
        assert_eq!(&bytes[spans[0].content_end..spans[0].full_end], b"\r\n");
        assert_eq!(&bytes[spans[1].content_start..spans[1].content_end], b"");
        assert_eq!(&bytes[spans[2].content_start..spans[2].full_end], b"last");
    }

    #[test]
    fn short_handle_is_base36_and_monotonic() {
        assert_eq!(to_base36(1), "1");
        assert_eq!(to_base36(35), "z");
        assert_eq!(to_base36(36), "10");
    }

    #[test]
    fn registry_evicts_oldest_snapshot_at_fixed_capacity() {
        let registry = FileSnapshotRegistry::new();
        let path = Path::new("/bounded/source.txt");
        let mut ids = Vec::new();
        for value in 0..=MAX_SNAPSHOTS {
            let bytes = format!("line-{value}");
            let (id, _) = registry.capture(path, bytes.as_bytes()).expect("capture");
            ids.push(id);
        }
        assert!(matches!(
            registry.get(&ids[0], path),
            Err(FileToolError::SnapshotNotFound)
        ));
        assert!(registry.get(ids.last().expect("last id"), path).is_ok());
    }
}
