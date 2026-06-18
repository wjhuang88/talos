use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An I/O error occurred (file read/write, directory creation, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

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
}
