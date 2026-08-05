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
use talos_session::{PendingSubmissionStore, Session, SessionRuntimeActivation};

use crate::mcp_runtime::McpSessionRuntime;
use tokio::sync::{mpsc, watch};
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
    /// The stopped command target owned by the old Session before retirement.
    old_target: SessionCommandTarget,
    /// The handle for the newly active session actor. Its SQ sender is the
    /// generation-binding proxy, not the raw Actor sender.
    pub new_handle: SessionHandle,
    publication_ready: CancellationToken,
}

struct ActiveRuntime {
    actor_join: JoinHandle<()>,
    scheduler_cancel: CancellationToken,
    scheduler_join: JoinHandle<()>,
    publication_abort: CancellationToken,
    _mcp_runtime: Option<McpSessionRuntime>,
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
    prepared_mcp_runtime: Option<McpSessionRuntime>,
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
            prepared_mcp_runtime: None,
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
            publication_abort: CancellationToken::new(),
            _mcp_runtime: None,
        });
        Ok(())
    }

    /// Keeps the initial MCP client manager alive for the lifetime of the active runtime.
    pub fn attach_active_mcp_runtime(&mut self, runtime: McpSessionRuntime) -> Result<(), String> {
        let active = self
            .active_runtime
            .as_mut()
            .ok_or_else(|| "attach the active Actor runtime before MCP ownership".to_string())?;
        active._mcp_runtime = Some(runtime);
        Ok(())
    }

    /// Installs MCP ownership for the next prepared replacement.
    pub fn prepare_mcp_runtime(&mut self, runtime: McpSessionRuntime) -> Result<(), String> {
        if self.prepared_mcp_runtime.is_some() {
            return Err("an MCP runtime is already prepared".to_string());
        }
        self.prepared_mcp_runtime = Some(runtime);
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
    /// Durably stages an exact model activation while fencing the old generation.
    ///
    /// The sidecar update and generation fence share the same SQLite immediate
    /// transaction. The returned generation is not runnable until the caller
    /// durably appends the matching transcript marker and commits the activation.
    pub async fn quiesce_same_session_for_activation(
        &mut self,
        session: &Session,
        activation: &SessionRuntimeActivation,
    ) -> Result<u64, String> {
        if session.id != self.active_session.id {
            return Err(
                "activation quiesce requires the currently active logical Session".to_string(),
            );
        }
        if self.prepared.is_some() {
            return Err(
                "cannot quiesce while a session transition is already prepared".to_string(),
            );
        }
        if let Some(generation) = self.quiesced_generation {
            if generation == activation.generation {
                return Ok(generation);
            }
            return Err(format!(
                "Session is already quiesced at generation {generation}, not activation generation {}",
                activation.generation
            ));
        }

        let next_generation = PendingSubmissionStore::for_session(session)
            .stage_runtime_activation(self.active_generation(), activation)
            .map_err(|error| format!("failed to stage Session runtime activation: {error}"))?;
        self.retire_active_runtime().await;
        self.quiesced_generation = Some(next_generation);
        Ok(next_generation)
    }

    #[cfg(test)]
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
        let old_target = self.active_target.clone();

        if quiesced_generation.is_none() {
            self.retire_active_runtime().await;
        }
        self.quiesced_generation = None;

        actor.set_generation(next_generation);
        let publication_ready = CancellationToken::new();
        let publication_abort = CancellationToken::new();
        let actor_ready = publication_ready.clone();
        let actor_abort = publication_abort.clone();
        let actor_join = tokio::spawn(async move {
            tokio::select! {
                _ = actor_ready.cancelled() => actor.run().await,
                _ = actor_abort.cancelled() => {}
            }
        });

        let new_target = SessionCommandTarget::new(prepared.handle.sq_tx.clone(), next_generation);
        prepared.handle.sq_tx = new_target.bind_sender();
        let scheduler_cancel = CancellationToken::new();
        let scheduler_ready = publication_ready.clone();
        let scheduler_abort = publication_abort.clone();
        let scheduler_sender = prepared.handle.sq_tx.clone();
        let scheduler_cancel_for_task = scheduler_cancel.clone();
        let scheduler_join = tokio::spawn(async move {
            tokio::select! {
                _ = scheduler_ready.cancelled() => {
                    let task = pending_scheduler.spawn(
                        scheduler_sender,
                        next_generation,
                        scheduler_cancel_for_task,
                    );
                    if let Err(error) = task.await {
                        tracing::warn!(%error, "Session Scheduler task did not join cleanly");
                    }
                }
                _ = scheduler_abort.cancelled() => {}
            }
        });
        self.active_runtime = Some(ActiveRuntime {
            actor_join,
            scheduler_cancel,
            scheduler_join,
            publication_abort,
            _mcp_runtime: self.prepared_mcp_runtime.take(),
        });
        self.active_target = new_target;
        let old_session = std::mem::replace(&mut self.active_session, prepared.session);

        Ok(CommitResult {
            old_session,
            old_target,
            new_handle: prepared.handle,
            publication_ready,
        })
    }

    /// Publishes all required replacement routes before allowing the new
    /// Actor or Scheduler to execute. Any failed ownership handoff leaves the
    /// new generation durably fenced but stopped and returns an actionable
    /// error; ordinary success must not be emitted by the caller.
    pub async fn publish_commit(
        &mut self,
        result: CommitResult,
        session: Session,
        session_watch_tx: &watch::Sender<Session>,
        sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
        bridge_rx_update_tx: &mpsc::UnboundedSender<(
            Session,
            mpsc::UnboundedReceiver<talos_core::session::SessionEvent>,
        )>,
    ) -> Result<Session, String> {
        let CommitResult {
            old_session,
            old_target,
            new_handle,
            publication_ready,
        } = result;
        let SessionHandle { sq_tx, eq_rx } = new_handle;

        let publication = (|| {
            bridge_rx_update_tx
                .send((session.clone(), eq_rx))
                .map_err(|_| "Bridge event route receiver is unavailable".to_string())?;
            session_watch_tx
                .send(session)
                .map_err(|_| "Session watch receiver is unavailable".to_string())?;
            sq_tx_watch_tx
                .send(sq_tx)
                .map_err(|_| "command-route watch receiver is unavailable".to_string())?;
            Ok::<(), String>(())
        })();

        match publication {
            Ok(()) => {
                publication_ready.cancel();
                Ok(old_session)
            }
            Err(error) => {
                let failed_session = self.active_session.clone();
                self.abort_committed_publication().await;
                self.active_target = old_target;
                self.active_session = old_session.clone();
                session_watch_tx.send_replace(old_session.clone());
                Err(format!(
                    "replacement publication failed after the durable fence: {error}. Failed Session {} is stopped; logical ownership was restored to Session {} at generation {}. Resume or retry the lifecycle operation",
                    failed_session.id,
                    old_session.id,
                    self.active_generation(),
                ))
            }
        }
    }

    async fn abort_committed_publication(&mut self) {
        self.active_target.cancel_routes();
        if let Some(runtime) = self.active_runtime.take() {
            runtime.publication_abort.cancel();
            runtime.scheduler_cancel.cancel();
            if let Err(error) = runtime.scheduler_join.await {
                tracing::warn!(%error, "aborted Session Scheduler task did not join cleanly");
            }
            if let Err(error) = runtime.actor_join.await {
                tracing::warn!(%error, "aborted Session Actor task did not join cleanly");
            }
        }
    }

    async fn retire_active_runtime(&mut self) {
        self.active_target.cancel_routes();
        if let Some(runtime) = self.active_runtime.take() {
            runtime.publication_abort.cancel();
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
        self.prepared_mcp_runtime = None;
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
        watch_tx.send(sender_one).expect("operation should succeed");
        watch_tx.send(sender_two).expect("operation should succeed");

        watch_rx.changed().await.expect("operation should succeed");
        let current = watch_rx.borrow().clone();
        assert_eq!(authoritative_generation_for_sender(&current), Some(2));
    }

    #[test]
    fn transition_rehydrates_the_durable_session_generation() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-transition-generation")
            .expect("operation should succeed");
        let store = PendingSubmissionStore::for_session(durable.session());
        assert_eq!(
            store
                .advance_runtime_generation(0)
                .expect("operation should succeed"),
            1
        );
        let (sender, _receiver) = mpsc::channel(1);

        let transition = SessionTransition::new(sender, durable.session().clone())
            .expect("operation should succeed");
        assert_eq!(transition.active_generation(), 1);
    }

    async fn publication_failure_fixture(
        name: &str,
    ) -> (
        tempfile::TempDir,
        talos_session::SessionManager,
        SessionTransition,
        CommitResult,
        Session,
        Session,
        mpsc::Sender<SessionOp>,
    ) {
        use std::sync::Arc;
        use talos_agent::{Agent, create_scheduler_tools};
        use talos_core::session::{RuntimePolicy, SessionConfig};
        use talos_core::tool::ToolRegistry;
        use talos_provider::mock::MockProvider;

        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let old_session = manager
            .create_session(&format!("{name}-old"), "")
            .expect("operation should succeed");
        let new_session = manager
            .create_session(&format!("{name}-new"), "")
            .expect("operation should succeed");
        let (old_tx, mut old_rx) = mpsc::channel(4);
        tokio::spawn(async move {
            while let Some(operation) = old_rx.recv().await {
                if matches!(operation, SessionOp::Shutdown) {
                    break;
                }
            }
        });
        let old_session_for_assertion = old_session.clone();
        let mut transition =
            SessionTransition::new(old_tx, old_session).expect("operation should succeed");

        let agent = Agent::with_security(
            Arc::new(MockProvider::new().with_response("unused")),
            ToolRegistry::new(),
            None,
            None,
            temp.path().to_path_buf(),
        );
        let (handle, actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: temp.path().to_path_buf(),
                initial_history: Vec::new(),
                model_context_limit: 32_768,
            },
        );
        let raw_new_sender = handle.sq_tx.clone();
        let (_, pending_scheduler) = create_scheduler_tools();
        transition
            .prepare(handle, new_session.clone())
            .expect("operation should succeed");
        let result = transition
            .commit(actor, pending_scheduler)
            .await
            .expect("operation should succeed");
        (
            temp,
            manager,
            transition,
            result,
            old_session_for_assertion,
            new_session,
            raw_new_sender,
        )
    }

    async fn assert_publication_failure_restores_old_owner_and_cleans_child(
        manager: &talos_session::SessionManager,
        transition: &SessionTransition,
        old_session: &Session,
        failed_session: &Session,
        raw_new_sender: &mpsc::Sender<SessionOp>,
    ) {
        assert!(transition.active_runtime.is_none());
        assert_eq!(transition.active_session.id, old_session.id);
        assert_eq!(
            transition.active_generation(),
            PendingSubmissionStore::for_session(old_session)
                .runtime_generation()
                .expect("load restored generation")
        );
        tokio::time::timeout(std::time::Duration::from_secs(1), raw_new_sender.closed())
            .await
            .expect("failed publication must drop the new Actor receiver");
        assert!(raw_new_sender.is_closed());
        manager
            .rollback_session_artifacts(failed_session)
            .expect("failed child cleanup must succeed after publication abort");
        assert!(old_session.file_path.exists());
        assert!(!failed_session.file_path.exists());
        let sqlite = failed_session
            .file_path
            .with_file_name(format!("{}.pending.sqlite", failed_session.id));
        assert!(!sqlite.exists());
        assert!(!std::path::PathBuf::from(format!("{}-wal", sqlite.display())).exists());
        assert!(!std::path::PathBuf::from(format!("{}-shm", sqlite.display())).exists());
    }

    #[tokio::test]
    async fn bridge_publication_failure_stops_replacement_without_success() {
        let (_temp, manager, mut transition, result, old_session, new_session, raw_new_sender) =
            publication_failure_fixture("bridge-publication-failure").await;
        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());
        let (placeholder_tx, _placeholder_rx) = mpsc::channel(1);
        let (sq_watch_tx, _sq_watch_rx) = watch::channel(placeholder_tx);
        let (bridge_tx, bridge_rx) = mpsc::unbounded_channel();
        drop(bridge_rx);

        let error = transition
            .publish_commit(
                result,
                new_session.clone(),
                &session_watch_tx,
                &sq_watch_tx,
                &bridge_tx,
            )
            .await
            .expect_err("operation should fail");
        assert!(error.contains("Bridge event route receiver is unavailable"));
        assert!(error.contains("logical ownership was restored"));
        assert!(error.contains(&old_session.id.to_string()));
        assert!(error.contains(&new_session.id.to_string()));
        assert_publication_failure_restores_old_owner_and_cleans_child(
            &manager,
            &transition,
            &old_session,
            &new_session,
            &raw_new_sender,
        )
        .await;
        assert_eq!(session_watch_rx.borrow().id, old_session.id);
    }

    #[tokio::test]
    async fn session_watch_publication_failure_stops_replacement_without_success() {
        let (_temp, manager, mut transition, result, old_session, new_session, raw_new_sender) =
            publication_failure_fixture("session-watch-publication-failure").await;
        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());
        drop(session_watch_rx);
        let (placeholder_tx, _placeholder_rx) = mpsc::channel(1);
        let (sq_watch_tx, _sq_watch_rx) = watch::channel(placeholder_tx);
        let (bridge_tx, _bridge_rx) = mpsc::unbounded_channel();

        let error = transition
            .publish_commit(
                result,
                new_session.clone(),
                &session_watch_tx,
                &sq_watch_tx,
                &bridge_tx,
            )
            .await
            .expect_err("operation should fail");
        assert!(error.contains("Session watch receiver is unavailable"));
        assert!(error.contains("logical ownership was restored"));
        assert!(error.contains(&old_session.id.to_string()));
        assert!(error.contains(&new_session.id.to_string()));
        assert_publication_failure_restores_old_owner_and_cleans_child(
            &manager,
            &transition,
            &old_session,
            &new_session,
            &raw_new_sender,
        )
        .await;
    }

    #[tokio::test]
    async fn command_watch_publication_failure_stops_replacement_without_success() {
        let (_temp, manager, mut transition, result, old_session, new_session, raw_new_sender) =
            publication_failure_fixture("command-watch-publication-failure").await;
        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());
        let (placeholder_tx, placeholder_rx) = mpsc::channel(1);
        let (sq_watch_tx, sq_watch_rx) = watch::channel(placeholder_tx);
        drop(sq_watch_rx);
        drop(placeholder_rx);
        let (bridge_tx, _bridge_rx) = mpsc::unbounded_channel();

        let error = transition
            .publish_commit(
                result,
                new_session.clone(),
                &session_watch_tx,
                &sq_watch_tx,
                &bridge_tx,
            )
            .await
            .expect_err("operation should fail");
        assert!(error.contains("command-route watch receiver is unavailable"));
        assert!(error.contains("logical ownership was restored"));
        assert!(error.contains(&old_session.id.to_string()));
        assert!(error.contains(&new_session.id.to_string()));
        assert_publication_failure_restores_old_owner_and_cleans_child(
            &manager,
            &transition,
            &old_session,
            &new_session,
            &raw_new_sender,
        )
        .await;
        assert_eq!(session_watch_rx.borrow().id, old_session.id);
    }

    #[tokio::test]
    async fn runtime_retirement_waits_through_full_queue_for_scheduler_and_actor() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-awaited-runtime-retirement")
            .expect("operation should succeed");
        let (raw_tx, mut raw_rx) = mpsc::channel(1);
        raw_tx
            .send(SessionOp::Interrupt)
            .await
            .expect("operation should succeed");
        let mut transition = SessionTransition::new(raw_tx, durable.session().clone())
            .expect("operation should succeed");
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
            .expect("operation should succeed");

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

        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = talos_session::SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-final-history-quiescence")
            .expect("operation should succeed");
        let session = durable.session().clone();
        session
            .append(&Message::User {
                content: "before-handoff".into(),
            })
            .expect("operation should succeed");

        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let command_tx = raw_tx.clone();
        let mut transition =
            SessionTransition::new(raw_tx, session.clone()).expect("operation should succeed");
        let actor_session = session.clone();
        let actor_join = tokio::spawn(async move {
            while let Some(operation) = raw_rx.recv().await {
                match operation {
                    SessionOp::Interrupt => actor_session
                        .append(&Message::User {
                            content: "final-old-generation-turn".into(),
                        })
                        .expect("operation should succeed"),
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
            .expect("operation should succeed");

        command_tx
            .send(SessionOp::Interrupt)
            .await
            .expect("operation should succeed");
        assert_eq!(
            transition
                .quiesce_same_session(&session)
                .await
                .expect("operation should succeed"),
            1
        );

        let history = session.read_messages().expect("operation should succeed");
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
                .expect("operation should succeed"),
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
