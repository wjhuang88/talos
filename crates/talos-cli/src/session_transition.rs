//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) will consume.

use talos_agent::session::AppServerSession;
use talos_core::session::SessionHandle;
use talos_session::Session;

/// Result of a successful [`SessionTransition::commit`].
///
/// Contains the old session (for cleanup or fork source access) and the new
/// [`SessionHandle`] (whose `eq_rx` and `sq_tx` the caller must wire into the
/// bridge forwarder and user persister so persistence follows the new session).
pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The handle for the newly active session actor.
    pub new_handle: SessionHandle,
}

/// A prepared but not-yet-active session replacement.
struct PreparedSession {
    handle: SessionHandle,
    session: Session,
}

pub struct SessionTransition {
    active_sq_tx: tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
    active_session: Session,
    prepared: Option<PreparedSession>,
}

impl SessionTransition {
    pub fn new(
        sq_tx: tokio::sync::mpsc::Sender<talos_core::session::SessionOp>,
        session: Session,
    ) -> Self {
        Self {
            active_sq_tx: sq_tx,
            active_session: session,
            prepared: None,
        }
    }

    /// Prepare a session transition. Stores the handle and session; the actor
    /// is passed to [`commit`](Self::commit) to avoid storing a `!Send` type.
    pub fn prepare(&mut self, handle: SessionHandle, session: Session) -> Result<(), String> {
        if self.prepared.is_some() {
            return Err(
                "a session transition is already prepared — commit or rollback first".to_string(),
            );
        }
        self.prepared = Some(PreparedSession { handle, session });
        Ok(())
    }

    /// Commit the prepared transition, spawning the new actor and swapping sessions.
    ///
    /// Returns a [`CommitResult`] containing the old session and the new
    /// [`SessionHandle`]. The caller MUST use `new_handle.eq_rx` and
    /// `new_handle.sq_tx` to update the bridge forwarder and user persister;
    /// otherwise persistence will continue targeting the old session.
    pub fn commit(&mut self, mut actor: AppServerSession) -> Result<CommitResult, String> {
        let prepared = self
            .prepared
            .take()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;

        tokio::spawn(async move { actor.run().await });
        let _ = self
            .active_sq_tx
            .try_send(talos_core::session::SessionOp::Shutdown);

        self.active_sq_tx = prepared.handle.sq_tx.clone();
        let old_session = std::mem::replace(&mut self.active_session, prepared.session);

        Ok(CommitResult {
            old_session,
            new_handle: prepared.handle,
        })
    }

    pub fn rollback(&mut self) {
        self.prepared = None;
    }
}
