//! Bounded, instance-local backups for explicitly recoverable edits.

use std::collections::HashMap;
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
    expected: [u8; 32],
    created: Instant,
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
        if original.len() > MAX_FILE_BYTES {
            return Err(FileToolError::SnapshotLimit(
                "recoverable edit exceeds 2 MiB backup limit".into(),
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
                    expected: digest(updated),
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
        atomic_replace(
            path,
            &backup.original,
            backup.expected,
            path_lock.unwrap_or(&self.write_lock),
        )
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

#[cfg(test)]
mod tests;
