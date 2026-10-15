//! Content-free lifecycle evidence, independent of execution authorization.

use std::time::Duration;

use async_trait::async_trait;

/// A cleanup result that never exposes command output or configuration values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ToolCleanupError {
    /// The caller's waiting deadline elapsed; cleanup may still be running.
    #[error("tool cleanup deadline exceeded")]
    Timeout,
    /// Cleanup could not be confirmed. This failure remains visible on later waits.
    #[error("tool cleanup is unconfirmed")]
    Unconfirmed,
}

/// Waitable cleanup evidence for one invocation or a tool owner.
///
/// Waiting grants no execution authority, does not cancel execution, and does
/// not close admission. Callers must drop and join the actual execution before
/// waiting after cancellation, keeping the async runtime alive throughout.
/// Operation-local implementations span every continuation but exclude unrelated
/// invocations. A timeout must not clear a previously recorded cleanup failure.
#[async_trait]
pub trait ToolCleanupReceipt: Send + Sync {
    /// Waits within the supplied duration for confirmed cleanup.
    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError>;
}

/// A retained tool-owner shutdown handle, separate from invocation cancellation.
///
/// Composition roots retain this handle even after registry replacement/removal.
/// Shutdown seals admission, cancels and joins execution, then waits using the
/// remaining shared shutdown deadline. It must never report success merely
/// because the async runtime was destroyed.
pub trait ToolCleanupOwner: ToolCleanupReceipt {
    /// Irreversibly and idempotently prevents new operation/process registration.
    ///
    /// Registration and sealing must be atomic with respect to each other. Even
    /// existing invocations and their continuations cannot register after sealing.
    /// Previously registered commands may still start: callers must cancel and
    /// join execution before waiting. Sealing does not itself cancel that work.
    fn seal(&self);
}
