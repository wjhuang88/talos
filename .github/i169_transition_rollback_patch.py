from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match in {path}, found {count}: {old[:100]!r}")
    path.write_text(text.replace(old, new, 1))


def replace_between(path: Path, start_marker: str, end_marker: str, replacement: str) -> None:
    text = path.read_text()
    start = text.index(start_marker)
    end = text.index(end_marker, start)
    path.write_text(text[:start] + replacement + text[end:])


# SessionManager owns index/fork recording so rollback can use the same cached index boundary.
manager = ROOT / "crates/talos-session/src/manager.rs"
replace_once(
    manager,
    '''    /// Return forks originating from the given session ID.
    pub fn get_forks(&self, session_id: &str) -> Result<Vec<ForkInfo>, IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.get_forks(session_id)
    }
''',
    '''    /// Return forks originating from the given session ID.
    pub fn get_forks(&self, session_id: &str) -> Result<Vec<ForkInfo>, IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.get_forks(session_id)
    }

    /// Record one source/child fork relationship through the manager-owned index.
    pub fn record_fork(
        &self,
        source_session_id: &Uuid,
        forked_session_id: &Uuid,
        fork_entry_id: &str,
    ) -> Result<(), IndexError> {
        let mut guard = self.get_or_create_index()?;
        let index = guard.as_mut().expect("index just created");
        index.record_fork(
            &source_session_id.to_string(),
            &forked_session_id.to_string(),
            fork_entry_id,
        )
    }
''',
)

# Publication failure restores the old logical Session and its stopped generation owner.
transition = ROOT / "crates/talos-cli/src/session_transition.rs"
replace_once(
    transition,
    '''pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The handle for the newly active session actor. Its SQ sender is the
    /// generation-binding proxy, not the raw Actor sender.
    pub new_handle: SessionHandle,
    publication_ready: CancellationToken,
}
''',
    '''pub struct CommitResult {
    /// The session that was active before the transition.
    pub old_session: Session,
    /// The stopped command target owned by the old Session before retirement.
    old_target: SessionCommandTarget,
    /// The handle for the newly active session actor. Its SQ sender is the
    /// generation-binding proxy, not the raw Actor sender.
    pub new_handle: SessionHandle,
    publication_ready: CancellationToken,
}
''',
)
replace_once(
    transition,
    '''        let mut prepared = self
            .prepared
            .take()
            .expect("prepared transition was checked before the durable fence");

        if quiesced_generation.is_none() {
''',
    '''        let mut prepared = self
            .prepared
            .take()
            .expect("prepared transition was checked before the durable fence");
        let old_target = self.active_target.clone();

        if quiesced_generation.is_none() {
''',
)
replace_once(
    transition,
    '''        Ok(CommitResult {
            old_session,
            new_handle: prepared.handle,
            publication_ready,
        })
''',
    '''        Ok(CommitResult {
            old_session,
            old_target,
            new_handle: prepared.handle,
            publication_ready,
        })
''',
)
replace_once(
    transition,
    '''        let CommitResult {
            old_session,
            new_handle,
            publication_ready,
        } = result;
''',
    '''        let CommitResult {
            old_session,
            old_target,
            new_handle,
            publication_ready,
        } = result;
''',
)
replace_once(
    transition,
    '''            Err(error) => {
                self.abort_committed_publication().await;
                Err(format!(
                    "replacement publication failed after the durable fence: {error}. The new generation is stopped; resume or retry the lifecycle operation"
                ))
            }
''',
    '''            Err(error) => {
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
''',
)

# Extend existing production-composition publication tests with old-owner restoration and child cleanup.
replace_once(
    transition,
    '''    async fn publication_failure_fixture(
        name: &str,
    ) -> (
        tempfile::TempDir,
        SessionTransition,
        CommitResult,
        Session,
        mpsc::Sender<SessionOp>,
    ) {
''',
    '''    async fn publication_failure_fixture(
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
''',
)
replace_once(
    transition,
    '''        let mut transition =
            SessionTransition::new(old_tx, old_session).expect("operation should succeed");
''',
    '''        let old_session_for_assertion = old_session.clone();
        let mut transition =
            SessionTransition::new(old_tx, old_session).expect("operation should succeed");
''',
)
replace_once(
    transition,
    '''        (temp, transition, result, new_session, raw_new_sender)
    }

    async fn assert_publication_failure_stops_new_runtime(
        transition: &SessionTransition,
        raw_new_sender: &mpsc::Sender<SessionOp>,
    ) {
        assert!(transition.active_runtime.is_none());
        tokio::time::timeout(std::time::Duration::from_secs(1), raw_new_sender.closed())
            .await
            .expect("failed publication must drop the new Actor receiver");
        assert!(raw_new_sender.is_closed());
    }
''',
    '''        (
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
''',
)

for name in ["bridge", "session-watch", "command-watch"]:
    old = f'''        let (_temp, mut transition, result, new_session, raw_new_sender) =
            publication_failure_fixture("{name}-publication-failure").await;
'''
    new = f'''        let (
            _temp,
            manager,
            mut transition,
            result,
            old_session,
            new_session,
            raw_new_sender,
        ) = publication_failure_fixture("{name}-publication-failure").await;
'''
    replace_once(transition, old, new)

replace_once(
    transition,
    '''        let (session_watch_tx, _session_watch_rx) = watch::channel(new_session.clone());
''',
    '''        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());
''',
)
replace_once(
    transition,
    '''        let (session_watch_tx, session_watch_rx) = watch::channel(new_session.clone());
        drop(session_watch_rx);
''',
    '''        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());
        drop(session_watch_rx);
''',
)
# The command-watch test has the same old text as the bridge test after the first replacement.
text = transition.read_text()
needle = '        let (session_watch_tx, _session_watch_rx) = watch::channel(new_session.clone());\n'
if text.count(needle) != 1:
    raise RuntimeError(f"expected one remaining command-watch channel, found {text.count(needle)}")
transition.write_text(text.replace(
    needle,
    '        let (session_watch_tx, session_watch_rx) = watch::channel(old_session.clone());\n',
    1,
))

old_assert = '''        assert_publication_failure_stops_new_runtime(&transition, &raw_new_sender).await;
'''
new_assert = '''        assert_publication_failure_restores_old_owner_and_cleans_child(
            &manager,
            &transition,
            &old_session,
            &new_session,
            &raw_new_sender,
        )
        .await;
        assert_eq!(session_watch_rx.borrow().id, old_session.id);
'''
text = transition.read_text()
if text.count(old_assert) != 3:
    raise RuntimeError(f"expected three publication assertions, found {text.count(old_assert)}")
transition.write_text(text.replace(old_assert, new_assert))

# Shared content-free rollback diagnostics for TUI-created Session ownership.
handlers = ROOT / "crates/talos-cli/src/session_handlers.rs"
replace_once(
    handlers,
    '''pub(crate) fn emit_session_identity_after_queue_clear(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    session_id: String,
) {
    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
        talos_conversation::SteeringQueueSnapshot {
            entries: vec![],
            total_count: 0,
            omitted_count: 0,
        },
    ));
    let _ = ui_tx.send(UiOutput::SessionIdentity { id: session_id });
}
''',
    '''pub(crate) fn emit_session_identity_after_queue_clear(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    session_id: String,
) {
    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
        talos_conversation::SteeringQueueSnapshot {
            entries: vec![],
            total_count: 0,
            omitted_count: 0,
        },
    ));
    let _ = ui_tx.send(UiOutput::SessionIdentity { id: session_id });
}

fn rollback_owned_session_message(
    session_manager: &talos_session::SessionManager,
    session: &talos_session::Session,
    operation: &str,
    primary_error: impl std::fmt::Display,
) -> String {
    let session_id = session.id;
    let transcript = session.file_path.display();
    match session_manager.rollback_session_artifacts(session) {
        Ok(report) => format!(
            "[Error] {operation}: {primary_error}. Rolled back Session {session_id} at {transcript}; removed {} filesystem artifact(s) / {} byte(s), plus binding and index/fork ownership. Previous Session remains unchanged.\\n",
            report.removed_artifacts,
            report.bytes_removed,
        ),
        Err(cleanup_error) => format!(
            "[Error] {operation}: {primary_error}. Cleanup also failed for Session {session_id} at {transcript}: {cleanup_error}. Cleanup is retryable: close open SQLite handles, retry /delete {session_id} while the transcript remains discoverable, or run talos --storage-maintenance --reconcile for a transcript-less sidecar.\\n"
        ),
    }
}
''',
)

new_function = r'''#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_new(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    runtime_builder: &TuiRuntimeBuilder,
    session_manager: &talos_session::SessionManager,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
) {
    let mut transition = transition.lock().await;

    let workspace_root_str = canonical_workspace_root(runtime_builder.workspace_root());
    let new_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(session) => session,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to create new session: {error}\n"),
            );
            return;
        }
    };

    if let Err(error) = crate::mode_runtime::ensure_session_runtime_identity(config, &new_session) {
        let text = rollback_owned_session_message(
            session_manager,
            &new_session,
            "Failed to initialize new Session runtime identity",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let built_runtime = match runtime_builder.build(config, &new_session, vec![]).await {
        Ok(runtime) => runtime,
        Err(error) => {
            let text = rollback_owned_session_message(
                session_manager,
                &new_session,
                "Failed to construct new Session runtime",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    let handle = built_runtime.handle;
    let actor = built_runtime.actor;
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        transition.rollback();
        let text = rollback_owned_session_message(
            session_manager,
            &new_session,
            "Failed to retain new Session MCP runtime",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let new_session_for_watch = new_session.clone();
    if let Err(error) = transition.prepare(handle, new_session) {
        transition.rollback();
        let text = rollback_owned_session_message(
            session_manager,
            &new_session_for_watch,
            "Failed to prepare new Session",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor, sched_pending).await {
        Ok(result) => match transition
            .publish_commit(
                result,
                new_session_for_watch.clone(),
                session_watch_tx,
                sq_tx_watch_tx,
                bridge_rx_update_tx,
            )
            .await
        {
            Ok(_) => {
                emit_session_identity_after_queue_clear(
                    ui_tx,
                    new_session_for_watch.id.to_string(),
                );
                send_stream(
                    ui_tx,
                    MessageSource::System,
                    "[System] New session started. Previous session preserved.\n".to_string(),
                );
            }
            Err(error) => {
                let text = rollback_owned_session_message(
                    session_manager,
                    &new_session_for_watch,
                    "Failed to publish new Session runtime",
                    error,
                );
                send_stream(ui_tx, MessageSource::Error, text);
            }
        },
        Err(error) => {
            transition.rollback();
            let text = rollback_owned_session_message(
                session_manager,
                &new_session_for_watch,
                "Failed to commit new Session; old Session remains active",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

'''
replace_between(
    handlers,
    "#[allow(clippy::too_many_arguments)]\npub(crate) async fn handle_session_new(",
    "/// Handle `/resume` — list candidates or resume a specific session.\n",
    new_function,
)

fork_function = r'''/// Handle `/fork` — clone the active session's durable history into a child session.
///
/// Copies the source transcript bytes to a fresh UUID, establishes inherited
/// runtime identity and fork/index ownership, then publishes one replacement.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_fork(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    runtime_builder: &TuiRuntimeBuilder,
    session_manager: &talos_session::SessionManager,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
) {
    let mut transition = transition.lock().await;

    let source_session = session_watch_rx.borrow().clone();
    if let Err(error) = crate::mode_runtime::reconcile_session_runtime_state(&source_session) {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!("[Error] Cannot fork stopped Session runtime: {error}\n"),
        );
        return;
    }

    let source_bytes = match source_session.snapshot_bytes() {
        Ok(bytes) => bytes,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to read source Session file: {error}\n"),
            );
            return;
        }
    };
    let fork_history = match source_session.read_messages() {
        Ok(history) => history,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to read source Session history: {error}\n"),
            );
            return;
        }
    };
    let fork_entry_id = match source_session.read_entries() {
        Ok(entries) => match entries.last() {
            Some(entry) => entry.id.clone(),
            None => {
                send_stream(
                    ui_tx,
                    MessageSource::Error,
                    "[Error] Cannot fork an empty Session.\n".to_string(),
                );
                return;
            }
        },
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to read source Session entries: {error}\n"),
            );
            return;
        }
    };

    let workspace_root_str = canonical_workspace_root(runtime_builder.workspace_root());
    let child_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(session) => session,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to create child Session: {error}\n"),
            );
            return;
        }
    };
    let child_id = child_session.id;
    let child_path = child_session.file_path.clone();

    if let Some(parent) = child_path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        let text = rollback_owned_session_message(
            session_manager,
            &child_session,
            "Failed to create child Session directory",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }
    if let Err(error) = std::fs::write(&child_path, &source_bytes) {
        let text = rollback_owned_session_message(
            session_manager,
            &child_session,
            "Failed to clone source Session history",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let inherited_identity = match talos_session::PendingSubmissionStore::for_session(
        &source_session,
    )
    .runtime_state()
    {
        Ok(Some(state))
            if state.status == talos_session::SessionRuntimeActivationStatus::Committed =>
        {
            state.activation.target
        }
        Ok(Some(state)) => {
            let text = rollback_owned_session_message(
                session_manager,
                &child_session,
                "Source Session activation is not committed",
                format_args!("activation {}", state.activation.activation_id),
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
        Ok(None) => talos_session::SessionRuntimeIdentity::new(
            &config.provider,
            &config.model,
            config.variant.as_deref(),
        ),
        Err(error) => {
            let text = rollback_owned_session_message(
                session_manager,
                &child_session,
                "Failed to read source Session runtime identity",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    if let Err(error) = talos_session::PendingSubmissionStore::for_session(&child_session)
        .initialize_runtime_identity(inherited_identity)
    {
        let text = rollback_owned_session_message(
            session_manager,
            &child_session,
            "Failed to initialize fork runtime identity",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let index_result = session_manager
        .update_index(&source_session)
        .and_then(|()| session_manager.update_index(&child_session))
        .and_then(|()| {
            session_manager.record_fork(&source_session.id, &child_session.id, &fork_entry_id)
        });
    if let Err(error) = index_result {
        let text = rollback_owned_session_message(
            session_manager,
            &child_session,
            "Failed to publish child Session index/fork ownership",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let mut fork_config = config.clone();
    apply_session_model_to_config(&mut fork_config, &child_session);
    let built_runtime = match runtime_builder
        .build(&fork_config, &child_session, fork_history)
        .await
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let text = rollback_owned_session_message(
                session_manager,
                &child_session,
                "Failed to construct fork runtime",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    let handle = built_runtime.handle;
    let actor = built_runtime.actor;
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        transition.rollback();
        let text = rollback_owned_session_message(
            session_manager,
            &child_session,
            "Failed to retain fork MCP runtime",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let child_session_for_watch = child_session.clone();
    if let Err(error) = transition.prepare(handle, child_session) {
        transition.rollback();
        let text = rollback_owned_session_message(
            session_manager,
            &child_session_for_watch,
            "Failed to prepare fork",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor, sched_pending).await {
        Ok(result) => match transition
            .publish_commit(
                result,
                child_session_for_watch.clone(),
                session_watch_tx,
                sq_tx_watch_tx,
                bridge_rx_update_tx,
            )
            .await
        {
            Ok(old_session) => {
                emit_session_identity_after_queue_clear(
                    ui_tx,
                    child_session_for_watch.id.to_string(),
                );
                send_stream(
                    ui_tx,
                    MessageSource::System,
                    format!(
                        "[System] Forked Session {child_id} (source: {}).\n",
                        old_session.id
                    ),
                );
            }
            Err(error) => {
                let text = rollback_owned_session_message(
                    session_manager,
                    &child_session_for_watch,
                    "Failed to publish fork runtime",
                    error,
                );
                send_stream(ui_tx, MessageSource::Error, text);
            }
        },
        Err(error) => {
            transition.rollback();
            let text = rollback_owned_session_message(
                session_manager,
                &child_session_for_watch,
                "Failed to commit fork; old Session remains active",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

'''
replace_between(
    handlers,
    "/// Handle `/fork` — clone the active session's durable history into a child session.\n",
    "#[cfg(test)]\nmod tests {\n",
    fork_function,
)

# Add diagnostic tests to the existing handlers test module.
replace_once(
    handlers,
    '''mod tests {
    use super::*;

    #[test]
    fn committed_session_boundary_clears_preview_before_identity() {
''',
    '''mod tests {
    use super::*;

    #[test]
    fn rollback_diagnostic_preserves_primary_and_cleanup_failures() {
        let dir = tempfile::tempdir().expect("temporary rollback test directory");
        let manager = talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let child = manager
            .defer_create_session("talos", "/workspace")
            .expect("defer child Session");
        std::fs::create_dir_all(
            child
                .file_path
                .parent()
                .expect("child transcript has a parent"),
        )
        .expect("create child directory");
        std::fs::write(&child.file_path, b"non-sensitive transcript fixture")
            .expect("write child transcript");
        let sqlite = child
            .file_path
            .with_file_name(format!("{}.pending.sqlite", child.id));
        std::fs::create_dir(&sqlite).expect("create blocked SQLite target");
        std::fs::write(sqlite.join("held"), b"held").expect("make target non-empty");

        let message = rollback_owned_session_message(
            &manager,
            &child,
            "Failed to construct fork runtime",
            "provider build failed",
        );
        assert!(message.contains("provider build failed"));
        assert!(message.contains("Cleanup also failed"));
        assert!(message.contains(&child.id.to_string()));
        assert!(message.contains(&sqlite.display().to_string()));
        assert!(message.contains("retryable"));
        assert!(child.file_path.exists());
        assert!(!message.contains("non-sensitive transcript fixture"));

        std::fs::remove_file(sqlite.join("held")).expect("release blocked target");
        std::fs::remove_dir(&sqlite).expect("remove blocked target");
        manager
            .rollback_session_artifacts(&child)
            .expect("retry cleanup succeeds");
    }

    #[test]
    fn rollback_diagnostic_reports_complete_success_without_content() {
        let dir = tempfile::tempdir().expect("temporary rollback test directory");
        let manager = talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let child = manager
            .create_session("talos", "/workspace")
            .expect("create child Session");
        talos_session::PendingSubmissionStore::for_session(&child)
            .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                "provider",
                "model",
                None,
            ))
            .expect("initialize child identity");

        let message = rollback_owned_session_message(
            &manager,
            &child,
            "Failed to commit fork",
            "durable fence failed",
        );
        assert!(message.contains("durable fence failed"));
        assert!(message.contains("Rolled back Session"));
        assert!(message.contains(&child.id.to_string()));
        assert!(!child.file_path.exists());
        assert!(!message.contains("submission"));
    }

    #[test]
    fn committed_session_boundary_clears_preview_before_identity() {
''',
)

# CLI --fork uses the same manager-owned rollback and combines primary + cleanup diagnostics.
setup = ROOT / "crates/talos-cli/src/session_setup.rs"
cli_fork = r'''fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<Session> {
    let source = manager
        .resume_session(source_session_id)
        .with_context(|| format!("failed to load source Session {source_session_id}"))?;

    let entries = source
        .read_entries()
        .context("failed to read source entries")?;
    if entries.is_empty() {
        bail!("cannot fork an empty Session");
    }
    let fork_entry_id = entries
        .last()
        .expect("entries checked non-empty above")
        .id
        .clone();
    let source_bytes = source
        .snapshot_bytes()
        .context("failed to snapshot source Session")?;

    let new_id = Uuid::new_v4();
    let project_path = source
        .file_path
        .parent()
        .context("source Session file has no parent directory")?
        .to_path_buf();
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;
    let new_file_path = project_path.join(format!("{new_id}.{}", source.file_extension()));
    let mut child = Session::new(
        new_id,
        source.project.clone(),
        source.workspace_root.clone(),
        new_file_path.clone(),
    );

    let fork_result = (|| -> Result<()> {
        std::fs::write(&new_file_path, &source_bytes)
            .context("failed to clone source Session bytes")?;
        child
            .fork(&fork_entry_id)
            .context("failed to create fork branch")?;

        match talos_session::PendingSubmissionStore::for_session(&source)
            .runtime_state()
            .context("failed to read source Session runtime identity")?
        {
            Some(state)
                if state.status == talos_session::SessionRuntimeActivationStatus::Committed =>
            {
                talos_session::PendingSubmissionStore::for_session(&child)
                    .initialize_runtime_identity(state.activation.target)
                    .context("failed to initialize fork runtime identity")?;
            }
            Some(state) => {
                bail!(
                    "source Session activation {} is not committed",
                    state.activation.activation_id
                );
            }
            None => {}
        }

        manager
            .update_index(&source)
            .context("failed to index source Session")?;
        manager
            .update_index(&child)
            .context("failed to index forked Session")?;
        manager
            .record_fork(&source.id, &child.id, &fork_entry_id)
            .context("failed to record fork relationship")?;
        Ok(())
    })();

    match fork_result {
        Ok(()) => {
            eprintln!(
                "Forked Session {source_session_id} -> {new_id} (from entry {fork_entry_id})"
            );
            Ok(child)
        }
        Err(primary_error) => match manager.rollback_session_artifacts(&child) {
            Ok(report) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; rollback removed {} filesystem artifact(s) / {} byte(s), plus binding and index/fork ownership",
                new_file_path.display(),
                report.removed_artifacts,
                report.bytes_removed,
            )),
            Err(cleanup_error) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; cleanup also failed: {cleanup_error}; cleanup is retryable after closing open SQLite handles via --delete or --storage-maintenance --reconcile",
                new_file_path.display(),
            )),
        },
    }
}

'''
replace_between(
    setup,
    "fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<Session> {\n",
    "#[cfg(test)]\nmod fork_tests {\n",
    cli_fork,
)

# Artifact test proves rollback deletes child index + fork relation while retaining source.
artifact_tests = ROOT / "crates/talos-session/tests/i169_session_artifact_cleanup.rs"
text = artifact_tests.read_text()
text += r'''

#[test]
fn rollback_removes_child_index_and_fork_relation_but_preserves_source() {
    use talos_core::message::Message;

    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().join("sessions"));
    let source = manager
        .create_session("source", "/workspace")
        .expect("create source Session");
    source
        .append(&Message::User {
            content: "source-entry".to_string(),
        })
        .expect("append source entry");
    let fork_entry_id = source
        .read_entries()
        .expect("read source entries")
        .last()
        .expect("source entry exists")
        .id
        .clone();
    let child = manager
        .create_session("child", "/workspace")
        .expect("create child Session");
    child
        .append(&Message::User {
            content: "child-entry".to_string(),
        })
        .expect("append child entry");
    talos_session::PendingSubmissionStore::for_session(&child)
        .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
            "provider",
            "model",
            None,
        ))
        .expect("initialize child identity");
    manager.update_index(&source).expect("index source");
    manager.update_index(&child).expect("index child");
    manager
        .record_fork(&source.id, &child.id, &fork_entry_id)
        .expect("record fork relation");
    assert_eq!(
        manager
            .get_forks(&source.id.to_string())
            .expect("read source forks")
            .len(),
        1
    );

    manager
        .rollback_session_artifacts(&child)
        .expect("rollback child artifact ownership");

    assert!(source.file_path.exists());
    assert!(!child.file_path.exists());
    assert!(
        manager
            .get_forks(&source.id.to_string())
            .expect("read source forks after rollback")
            .is_empty()
    );
    assert!(manager.get_session(&child.id).is_err());
    assert!(manager.get_session(&source.id).is_ok());
}
'''
artifact_tests.write_text(text)
