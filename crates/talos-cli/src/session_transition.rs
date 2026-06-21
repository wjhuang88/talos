//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) will consume.

#![allow(dead_code)] // Foundation for SESSION-001-B/C, consumed in future iterations

use talos_agent::session::AppServerSession;
use talos_core::session::{SessionHandle, SessionOp};
use talos_session::Session;

/// A prepared but not-yet-active session replacement.
struct PreparedSession {
    handle: SessionHandle,
    actor: AppServerSession,
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

    pub fn prepare(
        &mut self,
        actor: AppServerSession,
        handle: SessionHandle,
        session: Session,
    ) -> Result<(), String> {
        if self.prepared.is_some() {
            return Err("a session transition is already prepared — commit or rollback first".to_string());
        }
        self.prepared = Some(PreparedSession { handle, actor, session });
        Ok(())
    }

    pub fn commit(&mut self) -> Result<Session, String> {
        let prepared = self.prepared.take()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;

        let mut new_actor = prepared.actor;
        tokio::spawn(async move { new_actor.run().await });

        let _ = self.active_sq_tx.try_send(SessionOp::Shutdown);

        let old_session = std::mem::replace(&mut self.active_session, prepared.session);
        Ok(old_session)
    }

    pub fn rollback(&mut self) {
        self.prepared = None;
    }

    pub fn has_prepared(&self) -> bool {
        self.prepared.is_some()
    }
}
