use std::path::PathBuf;

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An I/O error occurred (file read/write, directory creation, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Removing one artifact from a Session-owned artifact set failed.
    #[error(
        "failed to remove session artifact {path}: {source}; removed={removed:?}; remaining={remaining:?}; retryable=true"
    )]
    ArtifactCleanup {
        /// Exact artifact path whose removal failed.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: std::io::Error,
        /// Paths already removed before the failure.
        removed: Vec<PathBuf>,
        /// Paths still present and safe to retry.
        remaining: Vec<PathBuf>,
    },

    /// Session index cleanup failed while the transcript remained discoverable.
    #[error("failed to remove Session {session_id} from the index: {message}")]
    IndexCleanup {
        /// Session whose supplementary index/fork rows could not be removed.
        session_id: Uuid,
        /// Content-free index diagnostic.
        message: String,
    },

    /// Orphan-sidecar validation or SQLite ownership probing failed.
    #[error("failed to reconcile orphan Session sidecar {path}: {message}")]
    OrphanReconciliation {
        /// Exact validated sidecar path.
        path: PathBuf,
        /// Content-free validation or SQLite diagnostic.
        message: String,
    },

    /// A line in the JSONL file is not valid JSON.
    #[error("invalid JSON in session file: {0}")]
    InvalidJson(String),

    /// The requested session was not found.
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),

    /// The requested entry ID was not found in the session.
    #[error("entry not found: {0}")]
    EntryNotFound(String),

    /// The requested branch ID was not found.
    #[error("branch not found: {0}")]
    BranchNotFound(String),

    /// Failed to parse a session file.
    #[error("failed to parse session file: {0}")]
    ParseError(String),

    /// A per-session internal mutex was poisoned by a panicking thread.
    #[error("session lock poisoned")]
    LockPoisoned,

    /// A host-provided external session identifier is invalid.
    #[error("invalid external session identifier: {0}")]
    InvalidExternalId(String),

    /// A durable turn cannot be committed because its persisted state is inconsistent.
    #[error("durable turn error: {0}")]
    DurableTurn(String),
}
