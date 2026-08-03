//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) consume.

use talos_agent::session::AppServerSession;
use talos_core::session::{SessionHandle, SessionOp};
use talos_session::Session;
use tokio::sync::mpsc;

/// Atomically published command route for one exact Session Actor generation.
///
/// A sender and its generation must travel together. Publishing only a new
/// sender and letting consumers infer an epoch locally permits delayed commands
/// from an old lifecycle to target a newer Actor (ADR-056).
#[derive(Clone)]
pub(crate) struct SessionCommandTarget {
    pub(crate) sq_tx: mpsc::Sender<SessionOp>,
    pub(crate) generation: u64,
}

impl SessionCommandTarget {
    #[must_use]
    pub(crate) fn new(sq_tx: mpsc::Sender<SessionOp>, generation: u64) -> Self {
        Self { sq_tx, generation }
    }

    #[must_use]
    pub(crate) fn same_actor(&self, other: &Self) -> bool {
        self.generation == other.generation && self.sq_tx.same_channel(&other.sq_tx)
    }

    pub(crate) fn try_send(
        &self,
        operation: SessionOp,
    ) -> Result<(), mpsc::error::TrySendError<SessionOp>> {
        self.sq_tx.try_send(operation)
    }
}

/// Result of a successful [`SessionTransition::commit`].
///
/// Contains the old session (for cleanup or fork source access), the new
/// [`SessionHandle`], and the exact command target that must be published to
/// Bridge and lifecycle callers as one atomic value.
pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The handle for the newly active session actor.
    pub new_handle: SessionHandle,
    /// Sender and authoritative generation for the newly active Actor.
    pub new_target: SessionCommandTarget,
}

/// A prepared but not-yet-active session replacement.
struct PreparedSession {
    handle: SessionHandle,
    session: Session,
}

pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    prepared: Option<PreparedSession>,
}

impl SessionTransition {
    /// Creates the transition owner for the initial generation-zero Actor.
    pub fn new(sq_tx: mpsc::Sender<SessionOp>, session: Session) -> Self {
        Self {
            active_target: SessionCommandTarget::new(sq_tx, 0),
            active_session: session,
            prepared: None,
        }
    }

    /// Returns an atomic snapshot of the currently active Actor route.
    #[must_use]
    pub(crate) fn active_target(&self) -> SessionCommandTarget {
        self.active_target.clone()
    }

    /// Returns the authoritative generation of the currently active Actor.
    #[must_use]
    pub fn active_generation(&self) -> u64 {
        self.active_target.generation
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
    /// The composition root assigns the next generation before the Actor starts
    /// and publishes the sender and generation together. Actor admission remains
    /// the authoritative generation check (ADR-056).
    pub fn commit(&mut self, mut actor: AppServerSession) -> Result<CommitResult, String> {
        let next_generation = self
            .active_generation()
            .checked_add(1)
            .ok_or_else(|| "session generation exhausted".to_string())?;
        let prepared = self
            .prepared
            .take()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;

        actor.set_generation(next_generation);
        tokio::spawn(async move { actor.run().await });
        let _ = self.active_target.try_send(SessionOp::Shutdown);

        let new_target = SessionCommandTarget::new(prepared.handle.sq_tx.clone(), next_generation);
        self.active_target = new_target.clone();
        let old_session = std::mem::replace(&mut self.active_session, prepared.session);

        Ok(CommitResult {
            old_session,
            new_handle: prepared.handle,
            new_target,
        })
    }

    pub fn rollback(&mut self) {
        self.prepared = None;
    }
}
