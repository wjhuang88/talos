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

    let requested = Path::new(relative);
    let joined = if requested.is_absolute() {
        requested.to_path_buf()
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
    tool_name: &str,
    nature: talos_core::tool::ToolNature,
    authorizations: &[talos_core::tool::ToolExecutionAuthorization],
) -> Result<PathBuf, FileToolError> {
    if let Ok(path) = resolve_workspace_path(workspace_root, relative) {
        return Ok(path);
    }
    let authorization = authorizations
        .iter()
        .find(|authorization| {
            authorization.authorizes_path(tool_name, nature, workspace_root, relative)
        })
        .ok_or_else(|| FileToolError::PathEscape(relative.to_owned()))?;
    let path = authorization.normalized_path();
    if path.parent().is_none() {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }
    Ok(path.to_path_buf())
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
