from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{path}: expected exactly one anchor, found {count}\nANCHOR:\n{old}"
        )
    file.write_text(text.replace(old, new, 1))


transition = "crates/talos-cli/src/session_transition.rs"
replace_once(
    transition,
    """pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    active_runtime: Option<ActiveRuntime>,
    prepared: Option<PreparedSession>,
}
""",
    """pub struct SessionTransition {
    active_target: SessionCommandTarget,
    active_session: Session,
    active_runtime: Option<ActiveRuntime>,
    prepared: Option<PreparedSession>,
    quiesced_generation: Option<u64>,
}
""",
)
replace_once(
    transition,
    """            active_session: session,
            active_runtime: None,
            prepared: None,
        })
""",
    """            active_session: session,
            active_runtime: None,
            prepared: None,
            quiesced_generation: None,
        })
""",
)
replace_once(
    transition,
    """    pub fn prepare(&mut self, handle: SessionHandle, session: Session) -> Result<(), String> {
""",
    """    /// Durably fences and retires the active runtime before a same-Session
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
""",
)
replace_once(
    transition,
    """    /// Commits one Actor replacement with an acknowledged fence-and-handoff.
    ///
    /// For the same logical Session, durable admission and generation advance
    /// are serialized in one SQLite transaction. The fence is permitted only
    /// when generation G owns no non-terminal custody. After the fence succeeds
    /// there are no fallible preparation steps: old proxies are revoked, the
    /// old Scheduler is cancelled and joined, reliable Actor shutdown is
    /// queued and joined, and only then is the generation-G+1 Actor spawned and
    /// published. A crash after the fence therefore leaves durable G+1 with no
    /// accepted G custody and no surviving process-local G authority.
""",
    """    /// Commits one Actor replacement with an acknowledged fence-and-handoff.
    ///
    /// For the same logical Session, durable admission and generation advance
    /// are serialized in one SQLite transaction. Callers that need canonical
    /// final history first use [`Self::quiesce_same_session`], then prepare the
    /// replacement from the post-retirement transcript. Other transitions fence
    /// and retire here. In both paths the generation-G+1 Actor is published only
    /// after generation G has lost process-local authority.
""",
)
replace_once(
    transition,
    """        let same_logical_session = prepared.session.id == self.active_session.id;
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
""",
    """        let same_logical_session = prepared.session.id == self.active_session.id;
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
""",
)
replace_once(
    transition,
    """    #[test]
    fn targeted_interrupt_generation_is_not_rewritten_by_proxy_binding() {
""",
    """    #[tokio::test]
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
""",
)

lifecycle = "crates/talos-cli/src/model_lifecycle.rs"
replace_once(
    lifecycle,
    "use talos_session::{Session, SessionManager};",
    "use talos_session::{Session, SessionError, SessionManager};",
)
replace_once(
    lifecycle,
    """    let mut history = current_session.read_messages().unwrap_or_default();
    let switch_marker = model_switch_marker(
        &previous_provider,
        &previous_model,
        &model_config.provider,
        &model_config.model,
    );
    history.push(switch_marker.clone());

    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };
""",
    """    let switch_marker = model_switch_marker(
        &previous_provider,
        &previous_model,
        &model_config.provider,
        &model_config.model,
    );
""",
)
replace_once(
    lifecycle,
    r'''    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    actor.set_persistence(
        current_session.clone(),
        crate::mode_runtime::session_metadata_for_model(
            &runtime_model_config.model,
            &runtime_model_config.provider,
        ),
    );
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition.lock().await.prepare(handle, session_for_prepare) {
        let text = format!("[Error] Failed to prepare model switch: {e}\n");
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }

    let mut transition_guard = transition.lock().await;
    match transition_guard.commit(actor, sched_pending).await {
''',
    r'''    let mut transition_guard = transition.lock().await;
    let mut history = match read_final_history_after_quiescence(
        &mut transition_guard,
        &current_session,
    )
    .await
    {
        Ok(history) => history,
        Err(FinalHistoryError::Fence(error)) => {
            let text = format!(
                "[Error] Failed to fence model switch: {error}. Previous model remains active.\n"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
        Err(FinalHistoryError::Read(error)) => {
            let text = format!(
                "[Error] Model switch fenced the old runtime but failed to read final Session history: {error}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    };
    history.push(switch_marker.clone());
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };

    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    actor.set_persistence(
        current_session.clone(),
        crate::mode_runtime::session_metadata_for_model(
            &runtime_model_config.model,
            &runtime_model_config.provider,
        ),
    );
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition_guard.prepare(handle, session_for_prepare) {
        let text = format!(
            "[Error] Failed to prepare model switch after fencing: {e}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
        );
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }

    match transition_guard.commit(actor, sched_pending).await {
''',
)
replace_once(
    lifecycle,
    r'''            let text = format!(
                "[Error] Failed to commit model switch: {e}. Previous model remains active.\n"
            );
''',
    r'''            let text = format!(
                "[Error] Failed to publish model switch after fencing: {e}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
            );
''',
)
replace_once(
    lifecycle,
    """fn model_switch_marker(
""",
    """#[derive(Debug)]
enum FinalHistoryError {
    Fence(String),
    Read(SessionError),
}

async fn read_final_history_after_quiescence(
    transition: &mut SessionTransition,
    session: &Session,
) -> Result<Vec<Message>, FinalHistoryError> {
    transition
        .quiesce_same_session(session)
        .await
        .map_err(FinalHistoryError::Fence)?;
    session.read_messages().map_err(FinalHistoryError::Read)
}

fn model_switch_marker(
""",
)
replace_once(
    lifecycle,
    """    #[test]
    fn model_switch_marker_includes_previous_and_new_identity() {
""",
    """    #[tokio::test]
    async fn model_rebuild_history_is_read_only_after_old_runtime_quiesces() {
        let temp = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-model-final-history")
            .unwrap();
        let session = durable.session().clone();
        session
            .append(&Message::User {
                content: "history-before-switch".into(),
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
                            content: "committed-during-handoff".into(),
                        })
                        .unwrap(),
                    SessionOp::Shutdown => break,
                    _ => {}
                }
            }
        });
        let scheduler_cancel = tokio_util::sync::CancellationToken::new();
        let scheduler_token = scheduler_cancel.clone();
        let scheduler_join = tokio::spawn(async move {
            scheduler_token.cancelled().await;
        });
        transition
            .attach_active_runtime(actor_join, scheduler_cancel, scheduler_join)
            .unwrap();

        command_tx.send(SessionOp::Interrupt).await.unwrap();
        let mut history = read_final_history_after_quiescence(&mut transition, &session)
            .await
            .unwrap();
        history.push(model_switch_marker(
            "old-provider",
            "old-model",
            "new-provider",
            "new-model",
        ));

        let user_contents: Vec<_> = history
            .iter()
            .filter_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            user_contents,
            vec!["history-before-switch", "committed-during-handoff"]
        );
        assert!(matches!(history.last(), Some(Message::System { .. })));
    }

    #[test]
    fn model_switch_marker_includes_previous_and_new_identity() {
""",
)

discovery = "crates/talos-cli/src/provider_discovery.rs"
replace_once(
    discovery,
    """// Discovery is best-effort during provider registration. A dedicated
// connect timeout prevents an unreachable endpoint from consuming the
// entire request budget before a TCP connection exists.
#[cfg(not(test))]
const DISCOVERY_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(test)]
const DISCOVERY_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

#[cfg(not(test))]
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);
// Several unit tests intentionally exercise unreachable loopback/example
// endpoints while serializing process-global HOME changes. Keep those
// failures tightly bounded so one network fixture cannot stall every test
// waiting on the shared HOME mutex.
#[cfg(test)]
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);
""",
    """#[cfg(test)]
const DISCOVERY_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

#[cfg(not(test))]
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);
// Unit tests intentionally exercise unreachable endpoints while serializing
// process-global HOME changes. Test-only bounds prevent a network fixture
// from stalling unrelated tests without changing production timeout policy.
#[cfg(test)]
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);
""",
)
replace_once(
    discovery,
    """    let client = reqwest::Client::builder()
        .connect_timeout(DISCOVERY_CONNECT_TIMEOUT)
        .timeout(DISCOVERY_TIMEOUT)
        .build()
        .map_err(|e| DiscoveryError::Network(e.to_string()))?;
""",
    """    let client_builder = reqwest::Client::builder().timeout(DISCOVERY_TIMEOUT);
    #[cfg(test)]
    let client_builder = client_builder.connect_timeout(DISCOVERY_CONNECT_TIMEOUT);
    let client = client_builder
        .build()
        .map_err(|e| DiscoveryError::Network(e.to_string()))?;
""",
)

story = "docs/backlog/active/TUI-044-transactional-batched-steering-turn.md"
replace_once(
    story,
    """- Race and reconstruction evidence covers concurrent admission versus fencing, full Actor queues, old-Scheduler cancellation, Actor receiver closure, durable generation 1+ reopen, stale-command rejection, journal state, receipt generation, and Provider call counts.
""",
    """- Race and reconstruction evidence covers concurrent admission versus fencing, full Actor queues, old-Scheduler cancellation, Actor receiver closure, durable generation 1+ reopen, stale-command rejection, journal state, receipt generation, and Provider call counts.
- Same-Session model/provider replacement completes external preparation first, durably fences admission, retires the old Scheduler/Actor, and only then reads canonical final transcript history and constructs the replacement Actor. A final old user or Scheduler Turn cannot disappear between snapshot and fence.
- Provider-discovery stabilization is test-only; production discovery retains its pre-I169 30-second request policy and no production timeout exception is claimed by I169.
""",
)

iteration = Path("docs/iterations/I169-batched-steering-turn.md")
text = iteration.read_text()
note = """## Final-history handoff remediation (2026-08-04)

- Model/provider replacement performs fallible Provider, MCP, tool, skill and context preparation before the irreversible generation fence.
- It then advances durable generation, revokes old routes, joins the old Scheduler and Actor, reads canonical final transcript history, appends the switch marker, and constructs/publishes the replacement Actor.
- Focused race evidence queues a final old-generation transcript commit during retirement and proves replacement history observes it before the switch marker.
- Provider-discovery connection bounding remains test-only; production timeout behavior is outside I169 and unchanged.
"""
if "## Final-history handoff remediation (2026-08-04)" not in text:
    iteration.write_text(text.rstrip() + "\n\n" + note + "\n")
