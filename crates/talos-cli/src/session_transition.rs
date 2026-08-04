//! Session runtime transition service.
//!
//! Provides atomic prepare/commit/rollback for replacing the active
//! [`AppServerSession`] without splitting Agent context, persistence,
//! conversation state, or visible history across different sessions.
//!
//! This is SESSION-001-A: the infrastructure that SESSION-001-B (new/resume)
//! and SESSION-001-C (fork) consume.

use std::sync::{Mutex, OnceLock};

use talos_agent::{PendingSchedulerActor, session::AppServerSession};
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

    fn cancel_routes(&self) {
        self.route_cancel.cancel();
    }

    /// Reliably queues shutdown behind the finite raw SQ backlog and waits
    /// until the Actor receiver has closed. No bounded-queue `Full` result is
    /// discarded at this ownership boundary.
    async fn shutdown_actor(&self) {
        if self.sq_tx.send(SessionOp::Shutdown).await.is_ok() {
            self.sq_tx.closed().await;
        }
    }
}

/// Result of a successful [`SessionTransition::commit`].
pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The handle for the newly active session actor. Its SQ sender is the
    /// generation-binding proxy, not the raw Actor sender.
    pub new_handle: SessionHandle,
}

struct ActiveRuntime {
    actor_join: JoinHandle<()>,
    scheduler_cancel: CancellationToken,
    scheduler_join: JoinHandle<()>,
}

struct PreparedSession {
    handle: SessionHandle,
    session: Session,
}

pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    active_runtime: Option<ActiveRuntime>,
    prepared: Option<PreparedSession>,
    quiesced_generation: Option<u64>,
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
            active_runtime: None,
            prepared: None,
            quiesced_generation: None,
        })
    }

    /// Returns the authoritative generation of the currently active Actor.
    #[must_use]
    pub fn active_generation(&self) -> u64 {
        self.active_target.generation
    }

    /// Creates the exact generation-bound route published to the Bridge and
    /// Scheduler for the current Actor.
    #[must_use]
    pub fn bind_active_sender(&self) -> mpsc::Sender<SessionOp> {
        self.active_target.bind_sender()
    }

    /// Attaches the initial production Actor and Scheduler tasks so a later
    /// replacement can cancel and await both owners before publication.
    pub fn attach_active_runtime(
        &mut self,
        actor_join: JoinHandle<()>,
        scheduler_cancel: CancellationToken,
        scheduler_join: JoinHandle<()>,
    ) -> Result<(), String> {
        if self.active_runtime.is_some() {
            return Err("the active Session runtime is already attached".to_string());
        }
        self.active_runtime = Some(ActiveRuntime {
            actor_join,
            scheduler_cancel,
            scheduler_join,
        });
        Ok(())
    }

    /// Durably fences and retires the active runtime before a same-Session
    /// replacement reads its canonical final transcript.
    ///
    /// Callers must complete Provider, MCP, tool, skill and context preparation
    /// before entering this boundary. Once it returns, generation G cannot
    /// accept new durable custody, old command routes are revoked, and the old
    /// Scheduler and Actor have terminated. Reading the transcript afterwards
    /// therefore observes every old-generation Turn completed during retirement.
    pub async fn quiesce_same_session(&mut self, session: &Session) -> Result<u64, String> {
        if session.id != self.active_session.id {
            return Err("quiesce requires the currently active logical Session".to_string());
        }
        if self.prepared.is_some() {
            return Err(
                "cannot quiesce while a session transition is already prepared".to_string(),
            );
        }
        if let Some(generation) = self.quiesced_generation {
            return Ok(generation);
        }

        let next_generation = PendingSubmissionStore::for_session(session)
            .advance_runtime_generation(self.active_generation())
            .map_err(|error| format!("failed to fence Session runtime generation: {error}"))?;
        self.retire_active_runtime().await;
        self.quiesced_generation = Some(next_generation);
        Ok(next_generation)
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

    /// Commits one Actor replacement with an acknowledged fence-and-handoff.
    ///
    /// For the same logical Session, durable admission and generation advance
    /// are serialized in one SQLite transaction. Callers that need canonical
    /// final history first use [`Self::quiesce_same_session`], then prepare the
    /// replacement from the post-retirement transcript. Other transitions fence
    /// and retire here. In both paths the generation-G+1 Actor is published only
    /// after generation G has lost process-local authority.
    pub async fn commit(
        &mut self,
        mut actor: AppServerSession,
        pending_scheduler: PendingSchedulerActor,
    ) -> Result<CommitResult, String> {
        let prepared = self
            .prepared
            .as_ref()
            .ok_or_else(|| "no prepared transition to commit".to_string())?;
        let same_logical_session = prepared.session.id == self.active_session.id;
        let quiesced_generation = self.quiesced_generation;
        let pending_store = PendingSubmissionStore::for_session(&prepared.session);
        let next_generation = if same_logical_session {
            if let Some(generation) = quiesced_generation {
                generation
            } else {
                pending_store
                    .advance_runtime_generation(self.active_generation())
                    .map_err(|error| {
                        format!("failed to fence Session runtime generation: {error}")
                    })?
            }
        } else {
            pending_store.runtime_generation().map_err(|error| {
                format!("failed to load target Session runtime generation: {error}")
            })?
        };
        let mut prepared = self
            .prepared
            .take()
            .expect("prepared transition was checked before the durable fence");

        if quiesced_generation.is_none() {
            self.retire_active_runtime().await;
        }
        self.quiesced_generation = None;

        actor.set_generation(next_generation);
        let actor_join = tokio::spawn(async move { actor.run().await });

        let new_target = SessionCommandTarget::new(prepared.handle.sq_tx.clone(), next_generation);
        prepared.handle.sq_tx = new_target.bind_sender();
        let scheduler_cancel = CancellationToken::new();
        let scheduler_join = pending_scheduler.spawn(
            prepared.handle.sq_tx.clone(),
            next_generation,
            scheduler_cancel.clone(),
        );
        self.active_runtime = Some(ActiveRuntime {
            actor_join,
            scheduler_cancel,
            scheduler_join,
        });
        self.active_target = new_target;
        let old_session = std::mem::replace(&mut self.active_session, prepared.session);

        Ok(CommitResult {
            old_session,
            new_handle: prepared.handle,
        })
    }

    async fn retire_active_runtime(&mut self) {
        self.active_target.cancel_routes();
        if let Some(runtime) = self.active_runtime.take() {
            runtime.scheduler_cancel.cancel();
            if let Err(error) = runtime.scheduler_join.await {
                tracing::warn!(%error, "retired Session Scheduler task did not join cleanly");
            }
            self.active_target.shutdown_actor().await;
            if let Err(error) = runtime.actor_join.await {
                tracing::warn!(%error, "retired Session Actor task did not join cleanly");
            }
        } else {
            self.active_target.shutdown_actor().await;
        }
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
    async fn runtime_retirement_waits_through_full_queue_for_scheduler_and_actor() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let temp = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-awaited-runtime-retirement")
            .unwrap();
        let (raw_tx, mut raw_rx) = mpsc::channel(1);
        raw_tx.send(SessionOp::Interrupt).await.unwrap();
        let mut transition = SessionTransition::new(raw_tx, durable.session().clone()).unwrap();
        let proxy = transition.bind_active_sender();

        let scheduler_cancel = CancellationToken::new();
        let scheduler_retired = Arc::new(AtomicBool::new(false));
        let scheduler_retired_task = scheduler_retired.clone();
        let scheduler_token = scheduler_cancel.clone();
        let scheduler_join = tokio::spawn(async move {
            scheduler_token.cancelled().await;
            scheduler_retired_task.store(true, Ordering::SeqCst);
        });

        let actor_retired = Arc::new(AtomicBool::new(false));
        let actor_retired_task = actor_retired.clone();
        let actor_join = tokio::spawn(async move {
            assert!(matches!(raw_rx.recv().await, Some(SessionOp::Interrupt)));
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            assert!(matches!(raw_rx.recv().await, Some(SessionOp::Shutdown)));
            actor_retired_task.store(true, Ordering::SeqCst);
        });
        transition
            .attach_active_runtime(actor_join, scheduler_cancel, scheduler_join)
            .unwrap();

        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            transition.retire_active_runtime(),
        )
        .await
        .expect("retirement must await the old Scheduler and full-queue Actor");
        tokio::time::timeout(std::time::Duration::from_secs(1), proxy.closed())
            .await
            .expect("retirement must revoke every old generation-bound proxy");
        assert!(scheduler_retired.load(Ordering::SeqCst));
        assert!(actor_retired.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn quiesce_waits_for_final_old_generation_transcript_commit() {
        use talos_core::message::Message;

        let temp = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-final-history-quiescence")
            .unwrap();
        let session = durable.session().clone();
        session
            .append(&Message::User {
                content: "before-handoff".into(),
            })
            .unwrap();

        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let command_tx = raw_tx.clone();
        let mut transition = SessionTransition::new(raw_tx, session.clone()).unwrap();
        let actor_session = session.clone();
        let actor_join = tokio::spawn(async move {
            while let Some(operation) = raw_rx.recv().await {
                match operation {
                    SessionOp::Interrupt => actor_session
                        .append(&Message::User {
                            content: "final-old-generation-turn".into(),
                        })
                        .unwrap(),
                    SessionOp::Shutdown => break,
                    _ => {}
                }
            }
        });
        let scheduler_cancel = CancellationToken::new();
        let scheduler_token = scheduler_cancel.clone();
        let scheduler_join = tokio::spawn(async move {
            scheduler_token.cancelled().await;
        });
        transition
            .attach_active_runtime(actor_join, scheduler_cancel, scheduler_join)
            .unwrap();

        command_tx.send(SessionOp::Interrupt).await.unwrap();
        assert_eq!(transition.quiesce_same_session(&session).await.unwrap(), 1);

        let history = session.read_messages().unwrap();
        let user_contents: Vec<_> = history
            .iter()
            .filter_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            user_contents,
            vec!["before-handoff", "final-old-generation-turn"]
        );
        assert_eq!(
            PendingSubmissionStore::for_session(&session)
                .runtime_generation()
                .unwrap(),
            1
        );
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
