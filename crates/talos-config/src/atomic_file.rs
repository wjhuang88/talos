//! Single-file durable atomic replacement.
//!
//! Write-then-rename pattern: create a temp file in the same directory,
//! write content, flush, fsync, then atomically rename over the target.
//! Permissions are preserved from the original file (or 0600 for new files).

use crate::ConfigError;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Atomically replaces `path` with `content`.
///
/// Creates a uniquely-named temp file in the same directory using
/// `create_new` (collision-resistant: pid + nanos + counter), writes +
/// syncs, copies permissions from the original (if it exists), then renames.
/// On any I/O error the temp file is removed and the original is untouched.
/// Pre-existing temp files from other processes are never deleted.
pub(crate) fn durable_replace(path: &Path, content: &[u8]) -> Result<(), ConfigError> {
    let dir = path
        .parent()
        .ok_or_else(|| ConfigError::InvalidConfig("path has no parent directory".into()))?;

    std::fs::create_dir_all(dir)?;

    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut created_tmp = None;
    for attempt in 0..16 {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = dir.join(format!(
            ".{}.tmp.{pid}.{nanos}.{n}",
            path.file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".into())
        ));

        match open_create_new(&tmp) {
            Ok(file) => {
                created_tmp = Some((tmp, file));
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && attempt < 15 => {
                continue;
            }
            Err(e) => return Err(ConfigError::IoError(e)),
        }
    }

    let (tmp, mut file) = created_tmp.ok_or_else(|| {
        ConfigError::IoError(std::io::Error::other(
            "could not create unique temp file after retries",
        ))
    })?;

    file.write_all(content).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        ConfigError::IoError(e)
    })?;
    file.flush().map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        ConfigError::IoError(e)
    })?;
    file.sync_all().map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        ConfigError::IoError(e)
    })?;
    drop(file);

    copy_permissions(&tmp, path)?;

    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(ConfigError::IoError(e));
    }

    sync_dir(dir)?;
    Ok(())
}

/// Writes `content` to `path` with mode 0600 (Unix), then flushes and fsyncs.
pub(crate) fn write_file_synced(path: &Path, content: &[u8]) -> Result<(), ConfigError> {
    open_secure(path)
        .and_then(|mut f| {
            f.write_all(content)?;
            f.flush()?;
            f.sync_all()?;
            Ok(())
        })
        .map_err(|e| {
            let _ = std::fs::remove_file(path);
            cleanup_err(e)
        })
}

/// Creates `path` as a directory with mode 0700 (Unix). No-op if it exists.
pub(crate) fn create_dir_secure(path: &Path) -> Result<(), ConfigError> {
    if path.exists() {
        return Ok(());
    }
    create_dir_impl(path)
}

/// Opens the parent directory of `path` and fsyncs it.
pub(crate) fn sync_dir(dir: &Path) -> Result<(), ConfigError> {
    sync_dir_impl(dir).map_err(ConfigError::IoError)
}

/// Removes `path` and syncs its parent directory (durable removal).
#[allow(dead_code)]
pub(crate) fn durable_remove(path: &Path) -> Result<(), ConfigError> {
    if path.exists() {
        std::fs::remove_file(path).map_err(ConfigError::IoError)?;
    }
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

/// Atomically renames a directory. The destination must not exist.
#[allow(dead_code)]
pub(crate) fn rename_dir(from: &Path, to: &Path) -> Result<(), ConfigError> {
    std::fs::rename(from, to).map_err(ConfigError::IoError)
}

// ---- platform-specific internals ----

#[cfg(unix)]
fn open_secure(path: &Path) -> Result<std::fs::File, std::io::Error> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn open_secure(path: &Path) -> Result<std::fs::File, std::io::Error> {
    std::fs::File::create(path)
}

#[cfg(unix)]
fn open_create_new(path: &Path) -> Result<std::fs::File, std::io::Error> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn open_create_new(path: &Path) -> Result<std::fs::File, std::io::Error> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

#[cfg(unix)]
fn create_dir_impl(path: &Path) -> Result<(), ConfigError> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(path)
        .map_err(ConfigError::IoError)
}

#[cfg(not(unix))]
fn create_dir_impl(path: &Path) -> Result<(), ConfigError> {
    std::fs::create_dir_all(path).map_err(ConfigError::IoError)
}

#[cfg(unix)]
fn copy_permissions(tmp: &Path, orig: &Path) -> Result<(), ConfigError> {
    if let Ok(meta) = std::fs::metadata(orig) {
        std::fs::set_permissions(tmp, meta.permissions()).map_err(ConfigError::IoError)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn copy_permissions(_tmp: &Path, _orig: &Path) -> Result<(), ConfigError> {
    Ok(())
}

#[cfg(unix)]
fn sync_dir_impl(dir: &Path) -> Result<(), std::io::Error> {
    let f = std::fs::File::open(dir)?;
    f.sync_all()
}

#[cfg(not(unix))]
fn sync_dir_impl(_dir: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

fn cleanup_err(e: std::io::Error) -> ConfigError {
    ConfigError::IoError(e)
}
