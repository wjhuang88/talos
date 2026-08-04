from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_exact(path: str, old: str, new: str, expected: int = 1) -> None:
    text = read(path)
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} matches, found {count}")
    write(path, text.replace(old, new))


transition = "crates/talos-cli/src/session_transition.rs"
replace_exact(
    transition,
    "use talos_agent::session::AppServerSession;\n",
    "use talos_agent::{PendingSchedulerActor, session::AppServerSession};\n",
)
replace_exact(
    transition,
    '''    /// Revokes every generation-bound proxy immediately and reliably queues a
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
''',
    '''    fn cancel_routes(&self) {
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
''',
)
replace_exact(
    transition,
    "struct PreparedSession {\n",
    '''struct ActiveRuntime {
    actor_join: JoinHandle<()>,
    scheduler_cancel: CancellationToken,
    scheduler_join: JoinHandle<()>,
}

struct PreparedSession {
''',
)
replace_exact(
    transition,
    '''pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    prepared: Option<PreparedSession>,
}
''',
    '''pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    active_runtime: Option<ActiveRuntime>,
    prepared: Option<PreparedSession>,
}
''',
)
replace_exact(
    transition,
    '''            active_target: SessionCommandTarget::new(sq_tx, generation),
            active_session: session,
            prepared: None,
''',
    '''            active_target: SessionCommandTarget::new(sq_tx, generation),
            active_session: session,
            active_runtime: None,
            prepared: None,
''',
)
replace_exact(
    transition,
    '''    pub fn active_generation(&self) -> u64 {
        self.active_target.generation
    }

    pub fn prepare''',
    '''    pub fn active_generation(&self) -> u64 {
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

    pub fn prepare''',
)
text = read(transition)
start = text.index("    /// Commits one Actor replacement behind an atomic durable generation fence.")
end = text.index("    pub fn rollback(&mut self)", start)
replacement = '''    /// Commits one Actor replacement with an acknowledged fence-and-handoff.
    ///
    /// For the same logical Session, durable admission and generation advance
    /// are serialized in one SQLite transaction. The fence is permitted only
    /// when generation G owns no non-terminal custody. After the fence succeeds
    /// there are no fallible preparation steps: old proxies are revoked, the
    /// old Scheduler is cancelled and joined, reliable Actor shutdown is
    /// queued and joined, and only then is the generation-G+1 Actor spawned and
    /// published. A crash after the fence therefore leaves durable G+1 with no
    /// accepted G custody and no surviving process-local G authority.
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
            .expect("prepared transition was checked before the durable fence");

        self.retire_active_runtime().await;

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
            generation: next_generation,
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

'''
write(transition, text[:start] + replacement + text[end:])
text = read(transition)
start = text.index(
    "    #[tokio::test]\n    async fn retirement_revokes_proxy_and_waits_through_a_full_raw_queue()"
)
end = text.index(
    "    #[test]\n    fn targeted_interrupt_generation_is_not_rewritten_by_proxy_binding()",
    start,
)
replacement = '''    #[tokio::test]
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

'''
write(transition, text[:start] + replacement + text[end:])

mode = "crates/talos-cli/src/mode_runners.rs"
replace_exact(
    mode,
    '''    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    actor.set_persistence(
        session.clone(),
        session_metadata_for_model(&config.model, &config.provider),
    );
    let transition_owner = SessionTransition::new(handle.sq_tx.clone(), session.clone())
        .map_err(anyhow::Error::msg)?;
    let session_generation = transition_owner.active_generation();
    actor.set_generation(session_generation);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        session_generation,
        tokio_util::sync::CancellationToken::new(),
    );
    let sq_tx_signal = handle.sq_tx.clone();
    tokio::spawn(async move { actor.run().await });

    let transition = Arc::new(Mutex::new(transition_owner));
''',
    '''    let (mut handle, mut actor) = AppServerSession::new(agent, session_config);
    actor.set_persistence(
        session.clone(),
        session_metadata_for_model(&config.model, &config.provider),
    );
    let mut transition_owner = SessionTransition::new(handle.sq_tx.clone(), session.clone())
        .map_err(anyhow::Error::msg)?;
    let session_generation = transition_owner.active_generation();
    actor.set_generation(session_generation);
    handle.sq_tx = transition_owner.bind_active_sender();
    let sq_tx_signal = handle.sq_tx.clone();
    let actor_join = tokio::spawn(async move { actor.run().await });
    let scheduler_cancel = tokio_util::sync::CancellationToken::new();
    let scheduler_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        session_generation,
        scheduler_cancel.clone(),
    );
    transition_owner
        .attach_active_runtime(actor_join, scheduler_cancel, scheduler_join)
        .map_err(anyhow::Error::msg)?;

    let transition = Arc::new(Mutex::new(transition_owner));
''',
)

handlers = "crates/talos-cli/src/session_handlers.rs"
replace_exact(
    handlers,
    '''    match transition.commit(actor) {
        Ok(result) => {
            let _sched_join = sched_pending.spawn(
                result.new_handle.sq_tx.clone(),
                result.generation,
                tokio_util::sync::CancellationToken::new(),
            );
''',
    '''    match transition.commit(actor, sched_pending).await {
        Ok(result) => {
''',
    expected=3,
)

lifecycle = "crates/talos-cli/src/model_lifecycle.rs"
replace_exact(
    lifecycle,
    '''    match transition_guard.commit(actor) {
        Ok(result) => {
            let _sched_join = sched_pending.spawn(
                result.new_handle.sq_tx.clone(),
                result.generation,
                tokio_util::sync::CancellationToken::new(),
            );
''',
    '''    match transition_guard.commit(actor, sched_pending).await {
        Ok(result) => {
''',
)

session_tests = "crates/talos-agent/src/session/tests.rs"
replace_exact(
    session_tests,
    "    actor.set_generation(",
    "    set_authoritative_generation(&mut actor, ",
    expected=6,
)
replace_exact(
    session_tests,
    '''#[test]
fn structured_submission_rejects_unbounded_image_metadata() {
''',
    '''fn set_authoritative_generation(actor: &mut AppServerSession, generation: u64) {
    let store = actor.pending_store.clone();
    let current = store.runtime_generation().unwrap();
    for expected in current..generation {
        assert_eq!(
            store.advance_runtime_generation(expected).unwrap(),
            expected + 1
        );
    }
    actor.set_generation(generation);
}

#[test]
fn structured_submission_rejects_unbounded_image_metadata() {
''',
)
