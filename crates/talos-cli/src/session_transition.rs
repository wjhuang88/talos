//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) will consume.

use talos_agent::session::AppServerSession;
use talos_core::session::{SessionHandle, SessionOp};
use talos_session::Session;

/// A prepared but not-yet-active session replacement.
///
/// Does NOT store the actor (which is `!Send` due to its `Receiver`).
/// The actor is passed to [`SessionTransition::commit`] directly.
#[allow(dead_code)]
struct PreparedSession {
    handle: SessionHandle,
    session: Session,
}

pub struct SessionTransition {
    active_sq_tx: tokio::sync::mpsc::Sender<SessionOp>,
    active_session: Session,
    prepared: Option<PreparedSession>,
}

impl SessionTransition {
    pub fn new(sq_tx: tokio::sync::mpsc::Sender<SessionOp>, session: Session) -> Self {
        Self {
            active_sq_tx: sq_tx,
            active_session: session,
            prepared: None,
        }
    }

    /// Prepare a session transition. Stores the handle and session; the actor
    /// is passed to [`commit`] to avoid storing a `!Send` type.
    pub fn prepare(
        &mut self,
        handle: SessionHandle,
        session: Session,
    ) -> Result<(), String> {
        if self.prepared.is_some() {
            return Err("a session transition is already prepared — commit or rollback first".to_string());
        }
        self.prepared = Some(PreparedSession { handle, session });
        Ok(())
    }

    /// Commit the prepared transition, spawning the new actor and swapping sessions.
    pub fn commit(&mut self, mut actor: AppServerSession) -> Result<Session, String> {
        let prepared = self.prepared.take()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;

        tokio::spawn(async move { actor.run().await });

        let _ = self.active_sq_tx.try_send(SessionOp::Shutdown);

        let old_session = std::mem::replace(&mut self.active_session, prepared.session);
        Ok(old_session)
    }

    pub fn rollback(&mut self) {
        self.prepared = None;
    }

    #[allow(dead_code)]
    pub fn has_prepared(&self) -> bool {
        self.prepared.is_some()
    }

    /// Return a reference to the currently active session.
    pub fn active_session(&self) -> &Session {
        &self.active_session
    }
}
