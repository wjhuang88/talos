//! File operation tools: read, write, edit, delete, ls.

use std::path::{Path, PathBuf};

use thiserror::Error;

mod delete_tool;
mod ls_tool;
mod read_tool;
mod snapshot;
mod write_edit_tools;

pub use delete_tool::{DeleteInput, DeleteTool};
pub use ls_tool::{LsInput, LsTool};
pub use read_tool::{ReadInput, ReadTool};
pub use snapshot::FileSnapshotRegistry;
pub use write_edit_tools::{EditInput, EditTool, WriteInput, WriteTool};

/// Creates the four core file tools with one shared model-private snapshot registry.
#[must_use]
pub fn snapshot_aware_file_tools(
    workspace_root: PathBuf,
) -> (ReadTool, WriteTool, EditTool, DeleteTool) {
    let snapshots = FileSnapshotRegistry::new();
    (
        ReadTool::with_snapshot_registry(workspace_root.clone(), snapshots.clone()),
        WriteTool::with_snapshot_registry(workspace_root.clone(), snapshots.clone()),
        EditTool::with_snapshot_registry(workspace_root.clone(), snapshots.clone()),
        DeleteTool::with_snapshot_registry(workspace_root, snapshots),
    )
}

/// Size threshold for binary file detection (8KB).
const BINARY_CHECK_SIZE: usize = 8 * 1024;

/// Errors that can occur during file tool operations.
#[derive(Debug, Error)]
pub enum FileToolError {
    #[error("path escapes workspace root: {0}")]
    PathEscape(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("file appears to be binary: {0}")]
    BinaryFile(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("string not found in file: {0}")]
    StringNotFound(String),

    #[error("invalid line range: start_line ({start}) > end_line ({end})")]
    InvalidLineRange { start: u32, end: u32 },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("file already exists: {0}. Use the edit tool to modify existing files")]
    FileExists(String),

    #[error("SNAPSHOT_NOT_FOUND: snapshot expired or is unavailable; read the file again")]
    SnapshotNotFound,

    #[error("SNAPSHOT_PATH_MISMATCH: snapshot belongs to a different file")]
    SnapshotPathMismatch,

    #[error("FILE_REV_MISMATCH: file changed since it was read; read the file again")]
    FileRevisionMismatch,

    #[error("PATH_IDENTITY_CHANGED: file path changed since it was read; read the file again")]
    PathIdentityChanged,

    #[error("HASH_MISMATCH: line check code does not match the snapshot")]
    HashMismatch,

    #[error("INVALID_REF: {0}")]
    InvalidRef(String),

    #[error("INVALID_RANGE: {0}")]
    InvalidEditRange(String),

    #[error("SNAPSHOT_LIMIT: {0}")]
    SnapshotLimit(String),

    #[error("snapshot registry unavailable")]
    SnapshotRegistryUnavailable,

    #[error("ATOMIC_WRITE_FAILED: {0}")]
    AtomicWriteFailed(String),
}

/// Errors specific to the delete tool.
#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("refusing to delete workspace root")]
    WorkspaceRoot,
}

/// Resolves a relative path against the workspace root and validates
/// that the resulting path stays within the workspace.
pub(crate) fn resolve_workspace_path(
    workspace_root: &Path,
    relative: &str,
) -> Result<PathBuf, FileToolError> {
    let canon_root = workspace_root.canonicalize()?;

    let joined = if relative.starts_with('/') {
        PathBuf::from(relative)
    } else {
        workspace_root.join(relative)
    };

    let canonical = if joined.exists() {
        joined.canonicalize()?
    } else if let Some(parent) = joined.parent() {
        if parent.exists() {
            let canon_parent = parent.canonicalize()?;
            let file_name = joined
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            canon_parent.join(file_name)
        } else {
            canon_root.join(relative)
        }
    } else {
        canon_root.join(relative)
    };

    if !canonical.starts_with(&canon_root) {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }

    Ok(canonical)
}

/// Resolves a path that may be outside the workspace, with security checks.
///
/// This is used when the permission engine has authorized an external path
/// (SEC-001). The path is canonicalized to prevent symlink/TOCTOU attacks,
/// and path traversal (`..`) is rejected. Unlike `resolve_workspace_path`,
/// the path does not need to be within the workspace root.
///
/// # Security
///
/// - Symlink resolution: the path is canonicalized if it exists.
/// - Path traversal: `..` components that escape to root are rejected.
/// - Delete-root protection: refuses to resolve `/` as a target.
pub(crate) fn resolve_authorized_path(
    workspace_root: &Path,
    relative: &str,
) -> Result<PathBuf, FileToolError> {
    // First try workspace resolution (fast path for in-workspace files).
    if let Ok(path) = resolve_workspace_path(workspace_root, relative) {
        return Ok(path);
    }
    // SEC-001: External path authorization.
    // Only ABSOLUTE paths may resolve outside the workspace. Relative paths
    // that escape via `..` are still rejected (they use resolve_workspace_path
    // which already failed above).
    if !relative.starts_with('/') {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }

    // Check for symlink escape: if the absolute path is inside the workspace
    // but canonicalizes outside, it's a symlink attack — reject it.
    let canon_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let abs_path = PathBuf::from(relative);
    if abs_path.starts_with(&canon_root) && abs_path.exists() {
        let canon = abs_path.canonicalize()?;
        if !canon.starts_with(&canon_root) {
            return Err(FileToolError::PathEscape(relative.to_owned()));
        }
    }

    // External path: resolve with security checks but no workspace restriction.
    let joined = if relative.starts_with('/') {
        PathBuf::from(relative)
    } else {
        workspace_root.join(relative)
    };

    // Reject path traversal that escapes to filesystem root.
    let normalized = {
        let mut buf = PathBuf::new();
        for component in joined.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if !buf.pop() {
                        return Err(FileToolError::PathEscape(relative.to_owned()));
                    }
                }
                other => buf.push(other.as_os_str()),
            }
        }
        buf
    };

    // Refuse to resolve filesystem root (delete-root protection).
    let normalized_str = normalized.to_string_lossy();
    if normalized_str == "/" || normalized.parent().is_none() {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }

    // Canonicalize if the path exists (symlink resolution).
    let canonical = if normalized.exists() {
        normalized.canonicalize()?
    } else if let Some(parent) = normalized.parent() {
        if parent.exists() {
            let canon_parent = parent.canonicalize()?;
            let file_name = normalized
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            canon_parent.join(file_name)
        } else {
            normalized
        }
    } else {
        normalized
    };

    Ok(canonical)
}

/// Checks if a file appears to be binary by looking for null bytes
/// in the first 8KB of content.
pub(crate) fn is_binary_file(path: &Path) -> Result<bool, FileToolError> {
    let bytes = std::fs::read(path)?;
    let check_bytes = &bytes[..bytes.len().min(BINARY_CHECK_SIZE)];
    Ok(check_bytes.contains(&0u8))
}

/// Directories that are skipped during recursive search.
pub fn is_skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

#[cfg(test)]
#[allow(warnings)]
mod tests;
