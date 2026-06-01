//! Error types for hook execution.

use std::time::Duration;

use thiserror::Error;

/// Errors emitted by the hook runtime.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum HookError {
    /// A hook handler exceeded its execution timeout.
    #[error("hook handler '{handler}' timed out after {timeout:?}")]
    Timeout {
        /// Handler name.
        handler: String,
        /// Timeout duration.
        timeout: Duration,
    },

    /// A hook handler panicked while processing an event.
    #[error("hook handler '{handler}' panicked")]
    Panic {
        /// Handler name.
        handler: String,
    },

    /// A hook handler denied the current event.
    #[error("hook handler '{handler}' denied event: {reason}")]
    Denied {
        /// Handler name.
        handler: String,
        /// Denial reason.
        reason: String,
    },
}
