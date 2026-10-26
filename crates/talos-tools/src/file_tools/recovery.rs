//! Bounded, instance-local backups for explicitly recoverable file mutations.

use std::collections::HashMap;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::FileToolError;
use super::snapshot::{atomic_replace, digest};

const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_RECORDS: usize = 64;
const TTL: Duration = Duration::from_secs(15 * 60);

struct Backup {
    path: PathBuf,
    original: Arc<[u8]>,
    expected: ExpectedState,
    created: Instant,
}

enum ExpectedState {
    Contents([u8; 32]),
    Deleted(Permissions),
}

#[derive(Default)]
pub(super) struct FileRecoveryStore {
    records: Mutex<HashMap<String, Backup>>,
    write_lock: Mutex<()>,
}

impl FileRecoveryStore {
    fn records(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Backup>>, FileToolError> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| FileToolError::SnapshotRegistryUnavailable)?;
        records.retain(|_, record| record.created.elapsed() < TTL);
        Ok(records)
    }

    pub(super) fn replace(
        &self,
        path: &Path,
        original: &[u8],
        updated: &[u8],
        path_lock: Option<&Mutex<()>>,
    ) -> Result<String, FileToolError> {
        if original.len() > MAX_FILE_BYTES || updated.len() > MAX_FILE_BYTES {
            return Err(FileToolError::SnapshotLimit(
                "recoverable edit before/after content exceeds 2 MiB limit".into(),
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        {
            let mut records = self.records()?;
            let total = records
                .values()
                .map(|record| record.original.len())
                .sum::<usize>();
            if records.len() >= MAX_RECORDS
                || total.saturating_add(original.len()) > MAX_TOTAL_BYTES
            {
                return Err(FileToolError::SnapshotLimit(
                    "edit backup store is full; no file was changed".into(),
                ));
            }
            records.insert(
                id.clone(),
                Backup {
                    path: path.to_path_buf(),
                    original: Arc::from(original),
                    expected: ExpectedState::Contents(digest(updated)),
                    created: Instant::now(),
                },
            );
        }
        // Keep the backup on errors too: a parent-directory sync error can occur
        // after rename. Recovery still requires the exact expected edited content.
        atomic_replace(
            path,
            updated,
            digest(original),
            path_lock.unwrap_or(&self.write_lock),
        )
        .map_err(|error| FileToolError::AtomicWriteFailed(format!("{error}; recovery_id: {id}")))?;
        Ok(id)
    }

    pub(super) fn remove(
        &self,
        path: &Path,
        path_lock: Option<&Mutex<()>>,
    ) -> Result<String, FileToolError> {
        let mut records = self.records()?;
        let _guard = path_lock
            .unwrap_or(&self.write_lock)
            .lock()
            .map_err(|_| FileToolError::SnapshotRegistryUnavailable)?;
        if path.canonicalize()? != path {
            return Err(FileToolError::PathIdentityChanged);
        }
        let (original, permissions) = read_bounded_regular_file(path)?;
        let total = records
            .values()
            .map(|record| record.original.len())
            .sum::<usize>();
        if original.len() > MAX_FILE_BYTES
            || records.len() >= MAX_RECORDS
            || total.saturating_add(original.len()) > MAX_TOTAL_BYTES
        {
            return Err(FileToolError::SnapshotLimit(
                "deletion backup limit exceeded; no file was deleted".into(),
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        records.insert(
            id.clone(),
            Backup {
                path: path.to_path_buf(),
                original: Arc::from(original),
                expected: ExpectedState::Deleted(permissions),
                created: Instant::now(),
            },
        );
        // Retain a receipt on uncertain failure, just as for recoverable edits.
        fs::remove_file(path).map_err(|error| {
            FileToolError::AtomicWriteFailed(format!("deletion failed; recovery_id: {id}; {error}"))
        })?;
        Ok(id)
    }

    pub(super) fn restore(
        &self,
        path: &Path,
        id: &str,
        path_lock: Option<&Mutex<()>>,
    ) -> Result<(), FileToolError> {
        // Serializing restores with backup lookup prevents concurrent consumption.
        let mut records = self.records()?;
        let backup = records.get(id).ok_or_else(|| {
            FileToolError::InvalidInput(
                "recovery id is missing, expired or already consumed".into(),
            )
        })?;
        if backup.path != path {
            return Err(FileToolError::SnapshotPathMismatch);
        }
        let result = match &backup.expected {
            ExpectedState::Contents(expected) => atomic_replace(
                path,
                &backup.original,
                *expected,
                path_lock.unwrap_or(&self.write_lock),
            ),
            ExpectedState::Deleted(permissions) => restore_deleted(
                path,
                &backup.original,
                permissions,
                path_lock.unwrap_or(&self.write_lock),
            ),
        };
        result
        .map_err(|error| match error {
            FileToolError::AtomicWriteFailed(_) => FileToolError::AtomicWriteFailed(format!(
                "restoration may already have completed; recovery_id: {id} retained; inspect current file contents before retrying: {error}"
            )),
            other => other,
        })?;
        records.remove(id);
        Ok(())
    }
}

/// Reads only a bounded regular file through the same checked handle.
pub(super) fn read_bounded_regular_file(
    path: &Path,
) -> Result<(Vec<u8>, Permissions), FileToolError> {
    let file = open_backup_file(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > MAX_FILE_BYTES as u64 {
        return Err(FileToolError::SnapshotLimit(
            "recoverable mutation exceeds 2 MiB backup limit".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(FileToolError::SnapshotLimit(
            "recoverable mutation exceeds 2 MiB backup limit".into(),
        ));
    }
    Ok((bytes, metadata.permissions()))
}

fn open_backup_file(path: &Path) -> Result<fs::File, FileToolError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    #[cfg(not(any(unix, windows)))]
    return Err(FileToolError::InvalidInput(
        "recoverable deletion is unsupported on this platform".into(),
    ));
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(FileToolError::PathIdentityChanged);
        }
    }
    if !metadata.is_file() {
        return Err(FileToolError::InvalidInput(
            "deletion backup requires a regular file".into(),
        ));
    }
    Ok(file)
}

fn restore_deleted(
    path: &Path,
    bytes: &[u8],
    permissions: &Permissions,
    path_lock: &Mutex<()>,
) -> Result<(), FileToolError> {
    let _guard = path_lock
        .lock()
        .map_err(|_| FileToolError::SnapshotRegistryUnavailable)?;
    let parent = path.parent().ok_or(FileToolError::PathIdentityChanged)?;
    if parent.canonicalize()? != parent {
        return Err(FileToolError::PathIdentityChanged);
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // Exclusive creation never overwrites a recreated file or follows a leaf link.
    let mut file = options.open(path)?;
    (|| -> std::io::Result<()> {
        file.write_all(bytes)?;
        file.set_permissions(permissions.clone())?;
        file.sync_all()
    })()
    .map_err(|error| FileToolError::AtomicWriteFailed(error.to_string()))
}

#[cfg(test)]
mod tests;
