//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) consume.

use std::sync::{Mutex, OnceLock};

use talos_agent::session::AppServerSession;
use talos_core::session::{SessionHandle, SessionOp};
use talos_session::{PendingSubmissionStore, Session};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

type SenderGenerationRegistry = Vec<(mpsc::WeakSender<SessionOp>, u64)>;

fn sender_generation_registry() -> &'static Mutex<SenderGenerationRegistry> {
    static REGISTRY: OnceLock<Mutex<SenderGenerationRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn register_generation_bound_sender(sender: &mpsc::Sender<SessionOp>, generation: u64) {
    let mut registry = sender_generation_registry()
        .lock()
        .expect("session sender generation registry poisoned");
    registry.retain(|(weak, _)| {
        weak.upgrade()
            .is_some_and(|existing| !existing.same_channel(sender))
    });
    registry.push((sender.downgrade(), generation));
}

/// Returns the composition-root generation bound to one exact command Sender.
///
/// The registry stores weak Sender references, so observing a route never keeps
/// a retired Actor command channel alive. Production senders are registered
/// before they are published through the lifecycle watch boundary.
pub(crate) fn authoritative_generation_for_sender(sender: &mpsc::Sender<SessionOp>) -> Option<u64> {
    let mut registry = sender_generation_registry()
        .lock()
        .expect("session sender generation registry poisoned");
    let mut generation = None;
    registry.retain(|(weak, registered_generation)| {
        let Some(existing) = weak.upgrade() else {
            return false;
        };
        if existing.same_channel(sender) {
            generation = Some(*registered_generation);
        }
        true
    });
    generation
}

/// One exact Session Actor command route.
///
/// A target owns both the Actor SQ and the authoritative generation. Consumers
/// receive a normal bounded sender created by [`Self::bind_sender`]; that proxy
/// seals each new structured submission to this exact target before forwarding
/// it. Targeted interrupts retain the caller's generation so the Actor can
/// reject delayed commands instead of having the proxy rewrite their intent.
/// An old proxy remains bound to the old Actor and can never be adopted by a
/// later lifecycle (ADR-056).
#[derive(Clone)]
struct SessionCommandTarget {
    sq_tx: mpsc::Sender<SessionOp>,
    generation: u64,
    route_cancel: CancellationToken,
}

impl SessionCommandTarget {
    #[must_use]
    fn new(sq_tx: mpsc::Sender<SessionOp>, generation: u64) -> Self {
        Self {
            sq_tx,
            generation,
            route_cancel: CancellationToken::new(),
        }
    }

    /// Creates a bounded command proxy permanently bound to this Actor target.
    ///
    /// Reconciliation retains the original immutable generation because it is
    /// observational and may query an older accepted identity. New submissions
    /// are sealed to this target; targeted interrupts remain immutable and are
    /// validated by the Actor against both generation and Turn ID.
    #[must_use]
    fn bind_sender(&self) -> mpsc::Sender<SessionOp> {
        let (proxy_tx, mut proxy_rx) = mpsc::channel(512);
        register_generation_bound_sender(&proxy_tx, self.generation);
        let target = self.clone();
        tokio::spawn(async move {
            loop {
                let operation = tokio::select! {
                    biased;
                    _ = target.route_cancel.cancelled() => break,
                    operation = proxy_rx.recv() => operation,
                };
                let Some(operation) = operation else {
                    break;
                };
                let operation = target.bind_operation(operation);
                if target.sq_tx.send(operation).await.is_err() {
                    break;
                }
            }
        });
        proxy_tx
    }

    fn bind_operation(&self, mut operation: SessionOp) -> SessionOp {
        match &mut operation {
            SessionOp::SubmitStructured { submission }
            | SessionOp::SubmitStructuredTracked { submission, .. } => {
                submission.sender_generation = self.generation;
            }
            SessionOp::Submit { .. }
            | SessionOp::SubmitMultimodal { .. }
            | SessionOp::PreviewRequest { .. }
            | SessionOp::ReconcileStructured { .. }
            | SessionOp::ReconcileStructuredTracked { .. }
            | SessionOp::SubmitStructuredReconcile { .. }
            | SessionOp::SubmitStructuredReconcileTracked { .. }
            | SessionOp::SetSkillContext { .. }
            | SessionOp::InterruptTurn { .. }
            | SessionOp::CancelPausedSubmission { .. }
            | SessionOp::Interrupt
            | SessionOp::Shutdown => {}
        }
        operation
    }

    /// Revokes every generation-bound proxy immediately and reliably queues a
    /// shutdown behind the finite set of commands already accepted by the raw
    /// Actor SQ.
    ///
    /// The durable generation fence is committed before this method is called.
    /// Consequently any structured generation-G command that was queued but
    /// not yet accepted can only receive `WrongGeneration`; it cannot regain
    /// Provider authority while generation G+1 is published.
    fn retire(self) -> JoinHandle<()> {
        self.route_cancel.cancel();
        tokio::spawn(async move {
            if self.sq_tx.send(SessionOp::Shutdown).await.is_ok() {
                self.sq_tx.closed().await;
            }
        })
    }
}

/// Result of a successful [`SessionTransition::commit`].
pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The handle for the newly active session actor. Its SQ sender is the
    /// generation-binding proxy, not the raw Actor sender.
    pub new_handle: SessionHandle,
    /// Durable authoritative generation assigned to the replacement Actor.
    pub generation: u64,
}

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
    /// Creates the transition owner by rehydrating the durable generation for
    /// the active logical Session. A process restart therefore preserves the
    /// authority that owns accepted pending work.
    pub fn new(sq_tx: mpsc::Sender<SessionOp>, session: Session) -> Result<Self, String> {
        let generation = PendingSubmissionStore::for_session(&session)
            .runtime_generation()
            .map_err(|error| format!("failed to load Session runtime generation: {error}"))?;
        register_generation_bound_sender(&sq_tx, generation);
        Ok(Self {
            active_target: SessionCommandTarget::new(sq_tx, generation),
            active_session: session,
            prepared: None,
        })
    }

    /// Returns the authoritative generation of the currently active Actor.
    #[must_use]
    pub fn active_generation(&self) -> u64 {
        self.active_target.generation
    }

    pub fn prepare(&mut self, handle: SessionHandle, session: Session) -> Result<(), String> {
        if self.prepared.is_some() {
            return Err(
                "a session transition is already prepared — commit or rollback first".to_string(),
            );
        }
        self.prepared = Some(PreparedSession { handle, session });
        Ok(())
    }

    /// Commits one Actor replacement behind an atomic durable generation fence.
    ///
    /// Same-Session replacement first proves that generation G owns no
    /// non-terminal durable custody and advances the journal to G+1 in the same
    /// SQLite transaction that serializes admission. Only after that point are
    /// old proxies revoked and reliable Actor shutdown queued. The old Actor may
    /// finish rejecting already-buffered stale commands, but it cannot acquire
    /// new custody or enter the Provider while the replacement is published.
    pub fn commit(&mut self, mut actor: AppServerSession) -> Result<CommitResult, String> {
        let prepared = self
            .prepared
            .as_ref()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;
        let same_logical_session = prepared.session.id == self.active_session.id;
        let pending_store = PendingSubmissionStore::for_session(&prepared.session);
        let next_generation = if same_logical_session {
            pending_store
                .advance_runtime_generation(self.active_generation())
                .map_err(|error| format!("failed to fence Session runtime generation: {error}"))?
        } else {
            pending_store.runtime_generation().map_err(|error| {
                format!("failed to load target Session runtime generation: {error}")
            })?
        };
        let mut prepared = self
            .prepared
            .take()
            .ok_or_else(|| "prepared transition disappeared during commit".to_string())?;

        let _retirement = self.active_target.clone().retire();

        actor.set_generation(next_generation);
        tokio::spawn(async move { actor.run().await });

        let new_target = SessionCommandTarget::new(prepared.handle.sq_tx.clone(), next_generation);
        prepared.handle.sq_tx = new_target.bind_sender();
        self.active_target = new_target;
        let old_session = std::mem::replace(&mut self.active_session, prepared.session);

        Ok(CommitResult {
            old_session,
            new_handle: prepared.handle,
            generation: next_generation,
        })
    }

    pub fn rollback(&mut self) {
        self.prepared = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn coalesced_watch_updates_resolve_latest_authoritative_generation() {
        let (sender_zero, _rx_zero) = mpsc::channel(1);
        let (sender_one, _rx_one) = mpsc::channel(1);
        let (sender_two, _rx_two) = mpsc::channel(1);
        register_generation_bound_sender(&sender_zero, 0);
        register_generation_bound_sender(&sender_one, 1);
        register_generation_bound_sender(&sender_two, 2);

        let (watch_tx, mut watch_rx) = tokio::sync::watch::channel(sender_zero);
        watch_tx.send(sender_one).unwrap();
        watch_tx.send(sender_two).unwrap();

        watch_rx.changed().await.unwrap();
        let current = watch_rx.borrow().clone();
        assert_eq!(authoritative_generation_for_sender(&current), Some(2));
    }

    #[test]
    fn transition_rehydrates_the_durable_session_generation() {
        let temp = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-transition-generation")
            .unwrap();
        let store = PendingSubmissionStore::for_session(durable.session());
        assert_eq!(store.advance_runtime_generation(0).unwrap(), 1);
        let (sender, _receiver) = mpsc::channel(1);

        let transition = SessionTransition::new(sender, durable.session().clone()).unwrap();
        assert_eq!(transition.active_generation(), 1);
    }

    #[tokio::test]
    async fn retirement_revokes_proxy_and_waits_through_a_full_raw_queue() {
        let (raw_tx, mut raw_rx) = mpsc::channel(1);
        raw_tx.send(SessionOp::Interrupt).await.unwrap();
        let target = SessionCommandTarget::new(raw_tx, 7);
        let proxy = target.bind_sender();
        let retirement = target.retire();

        tokio::time::timeout(std::time::Duration::from_secs(1), proxy.closed())
            .await
            .expect("retirement must revoke every old generation-bound proxy");

        assert!(matches!(raw_rx.recv().await, Some(SessionOp::Interrupt)));
        assert!(matches!(raw_rx.recv().await, Some(SessionOp::Shutdown)));
        drop(raw_rx);
        tokio::time::timeout(std::time::Duration::from_secs(1), retirement)
            .await
            .expect("retirement must complete after the old Actor receiver closes")
            .unwrap();
    }

    #[test]
    fn targeted_interrupt_generation_is_not_rewritten_by_proxy_binding() {
        let (sender, _receiver) = mpsc::channel(1);
        let target = SessionCommandTarget::new(sender, 2);
        let operation = target.bind_operation(SessionOp::InterruptTurn {
            session_generation: 1,
            turn_id: "old-turn".into(),
        });
        assert!(matches!(
            operation,
            SessionOp::InterruptTurn {
                session_generation: 1,
                turn_id,
            } if turn_id == "old-turn"
        ));
    }
}
