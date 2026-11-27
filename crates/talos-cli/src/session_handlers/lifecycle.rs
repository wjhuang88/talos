//! Session lifecycle and cleanup-recovery workflows.

use super::super::*;

#[cfg(test)]
tokio::task_local! {
    static FORK_METADATA_CUT: Arc<tokio::sync::Notify>;
    static FORK_INDEX_FAILURE: bool;
    static NEW_IDENTITY_CUT: mpsc::UnboundedSender<std::path::PathBuf>;
}

/// Publish the UI boundary between two successfully committed sessions.
///
/// The queue belongs to the retired conversation engine, so its preview must be
/// cleared before the new session identity becomes visible. Keeping this small
/// ordered helper shared by `/new`, `/resume`, and `/fork` makes the boundary
/// independently testable without constructing a full session runtime.
pub(crate) fn emit_session_identity_after_queue_clear(
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

pub(crate) fn rollback_owned_session_message(
    session_manager: &talos_session::SessionManager,
    session: &talos_session::Session,
    operation: &str,
    primary_error: impl std::fmt::Display,
) -> String {
    let session_id = session.id;
    let transcript = session.file_path.display();
    match session_manager.rollback_session_artifacts(session) {
        Ok(report) => format!(
            "[Error] {operation}: {primary_error}. Rolled back Session {session_id} at {transcript}; removed {} filesystem artifact(s) / {} byte(s), plus binding and index/fork ownership. Previous Session remains unchanged.\n",
            report.removed_artifacts, report.bytes_removed,
        ),
        Err(cleanup_error) => format!(
            "[Error] {operation}: {primary_error}. Cleanup also failed for Session {session_id} at {transcript}: {cleanup_error}. Cleanup is retryable: close open SQLite handles, retry /delete {session_id} while the transcript remains discoverable, or run talos storage maintenance --reconcile for a transcript-less sidecar.\n"
        ),
    }
}

fn fork_failure_message(
    guard: &mut crate::history_restore::StagedForkHistory,
    preserve_child: bool,
    manager: &talos_session::SessionManager,
    child: &talos_session::Session,
    operation: &str,
    error: impl std::fmt::Display,
) -> String {
    guard.retain();
    guard.release_child_ownership();
    if preserve_child {
        format!(
            "[Error] {operation}: {error}. Child {} at {} is retained; resume it or reconcile storage before retrying.\n",
            child.id,
            child.file_path.display()
        )
    } else {
        rollback_owned_session_message(manager, child, operation, error)
    }
}

fn resolve_session_delete_target<'a>(
    sessions: &'a [talos_session::SessionInfo],
    argument: &str,
) -> Option<&'a talos_session::SessionInfo> {
    if let Ok(ordinal) = argument.parse::<usize>()
        && ordinal >= 1
        && ordinal <= sessions.len()
    {
        return sessions.get(ordinal - 1);
    }

    let session_id = uuid::Uuid::parse_str(argument).ok()?;
    sessions.iter().find(|session| session.id == session_id)
}

fn retain_resumable_sessions(
    sessions: &mut Vec<talos_session::SessionInfo>,
    active_id: uuid::Uuid,
) {
    sessions.retain(|session| session.id != active_id && session.message_count > 0);
    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_delete(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    selection: Option<String>,
) {
    let workspace_root_str = canonical_workspace_root(workspace_root);
    let active_id = session_watch_rx.borrow().id;

    match &selection {
        None => {
            let mut sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
            };
            if sessions.is_empty() {
                let text = "[System] No sessions found for this workspace.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return;
            }
            sessions.retain(|s| s.id != active_id);
            if sessions.is_empty() {
                let text = "[System] No other sessions in this workspace to delete. The active session cannot be deleted.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return;
            }
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            let items: Vec<SessionPickerItem> = sessions
                .iter()
                .enumerate()
                .map(|(i, s)| SessionPickerItem {
                    command: "/delete".to_string(),
                    ordinal: i + 1,
                    timestamp: s.timestamp.to_string(),
                    message_count: s.message_count,
                    preview: if s.last_message_preview.is_empty() {
                        "(empty)".to_string()
                    } else {
                        s.last_message_preview.clone()
                    },
                })
                .collect();

            let _ = ui_tx.send(UiOutput::SessionPicker(items));
        }
        Some(arg) => {
            let mut sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
            };
            sessions.retain(|s| s.id != active_id);
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            let Some(target) = resolve_session_delete_target(&sessions, arg) else {
                let text = format!(
                    "[Error] Invalid selection '{arg}'. Use /delete to pick a session or /delete <session-uuid>.\n"
                );
                send_stream(ui_tx, MessageSource::Error, text);
                return;
            };

            let target_id = target.id;
            match session_manager.delete_session(&target_id) {
                Ok(()) => {
                    let text = format!("[System] Deleted session {target_id}.\n");
                    send_stream(ui_tx, MessageSource::System, text);
                }
                Err(e) => {
                    let text = format!("[Error] Failed to delete session {target_id}: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
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

    let runtime_lease = match talos_session::SessionRuntimeLease::acquire(&new_session.file_path) {
        Ok(lease) => lease,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!(
                    "[Error] Cannot acquire new Session runtime ownership: {error}; artifacts retained.\n"
                ),
            );
            return;
        }
    };

    if let Err(error) = crate::mode_runtime::ensure_session_runtime_identity(config, &new_session) {
        drop(runtime_lease);
        let text = rollback_owned_session_message(
            session_manager,
            &new_session,
            "Failed to initialize new Session runtime identity",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    #[cfg(test)]
    if let Ok(reached) = NEW_IDENTITY_CUT.try_with(Clone::clone) {
        reached
            .send(new_session.file_path.clone())
            .expect("cut receiver");
        std::future::pending::<()>().await;
    }

    let built_runtime = match runtime_builder.build(config, &new_session, vec![]).await {
        Ok(runtime) => runtime,
        Err(error) => {
            drop(runtime_lease);
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
    let mut actor = built_runtime.actor;
    if let Err(error) = actor.adopt_runtime_lease(runtime_lease) {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!(
                "[Error] Cannot transfer new Session runtime ownership: {error}; artifacts retained.\n"
            ),
        );
        return;
    }
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        transition.rollback();
        drop(actor);
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
        drop(actor);
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

/// Handle `/resume` — list candidates or resume a specific session.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_resume(
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
    session_id: Option<String>,
) -> Option<Config> {
    let mut transition = transition.lock().await;

    let workspace_root_str = canonical_workspace_root(runtime_builder.workspace_root());

    let mut sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
        Ok(sessions) => sessions,
        Err(error) => {
            let text = format!("[Error] Failed to list sessions: {error}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    retain_resumable_sessions(&mut sessions, transition.active_session().id);

    let target_session = match &session_id {
        Some(id) => {
            // Try parsing as ordinal (1-based) first, then fall back to UUID.
            if let Ok(n) = id.parse::<usize>() {
                if sessions.is_empty() {
                    let text =
                        "[System] No other non-empty sessions to resume in this workspace.\n"
                            .to_string();
                    send_stream(ui_tx, MessageSource::System, text);
                    return None;
                }
                if n == 0 || n > sessions.len() {
                    let text = format!(
                        "[Error] Invalid session number {n}. Valid range: 1-{}.\n",
                        sessions.len()
                    );
                    send_stream(ui_tx, MessageSource::Error, text);
                    return None;
                }
                let selected = &sessions[n - 1];
                let selected_id = selected.id.to_string();
                match session_manager.resume_session(&selected_id) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                }
            } else {
                // Fall back to treating it as a UUID (backward compat), while preserving the
                // same current/empty filtering used by the picker and ordinal path.
                let selected_id = uuid::Uuid::parse_str(id)
                    .ok()
                    .filter(|candidate| sessions.iter().any(|session| session.id == *candidate));
                let Some(selected_id) = selected_id else {
                    let text = format!(
                        "[Error] Session '{id}' is not a resumable non-empty Session in this workspace.\n"
                    );
                    send_stream(ui_tx, MessageSource::Error, text);
                    return None;
                };
                match session_manager.resume_session(&selected_id.to_string()) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                }
            }
        }
        None => {
            if sessions.is_empty() {
                let text = "[System] No other non-empty sessions to resume in this workspace.\n"
                    .to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return None;
            }

            let items: Vec<SessionPickerItem> = sessions
                .iter()
                .enumerate()
                .map(|(i, s)| SessionPickerItem {
                    command: "/resume".to_string(),
                    ordinal: i + 1,
                    timestamp: s.timestamp.to_string(),
                    message_count: s.message_count,
                    preview: if s.last_message_preview.is_empty() {
                        "(empty)".to_string()
                    } else {
                        s.last_message_preview.clone()
                    },
                })
                .collect();

            let _ = ui_tx.send(UiOutput::SessionPicker(items));
            return None;
        }
    };

    let runtime_lease = match talos_session::SessionRuntimeLease::acquire(&target_session.file_path)
    {
        Ok(lease) => lease,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Cannot acquire resumed Session runtime ownership: {error}\n"),
            );
            return None;
        }
    };
    if let Err(error) = crate::mode_runtime::reconcile_session_runtime_state(&target_session) {
        let text = format!("[Error] Cannot resume stopped Session runtime: {error}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }
    let mut resume_config = config.clone();

    if let Err(error) =
        apply_session_model_to_config(&mut resume_config, &target_session).and_then(|()| {
            crate::mode_runtime::ensure_session_runtime_identity(&resume_config, &target_session)
        })
    {
        let text =
            format!("[Error] Failed to establish resumed Session runtime identity: {error}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }
    let resume_history = match crate::history_restore::read_session_snapshot(&target_session).await
    {
        Ok((history, _)) => history,
        Err(e) => {
            let text = format!("[Error] Failed to read session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

    let resume_history_for_hydrate = resume_history.clone();
    let built_runtime = match runtime_builder
        .build(&resume_config, &target_session, resume_history)
        .await
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let text = format!("[Error] Failed to construct resumed Session runtime: {error}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    let handle = built_runtime.handle;
    let mut actor = built_runtime.actor;
    if let Err(error) = actor.adopt_runtime_lease(runtime_lease) {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!("[Error] Cannot transfer resumed Session runtime ownership: {error}\n"),
        );
        return None;
    }
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        let text = format!("[Error] Failed to retain resumed MCP runtime: {error}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    // Clone for watch channel update after commit (target_session is moved into prepare).
    let target_session_for_watch = target_session.clone();
    if let Err(e) = transition.prepare(handle, target_session) {
        transition.rollback();
        let text = format!("[Error] Failed to prepare resume: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    match transition.commit(actor, sched_pending).await {
        Ok(result) => match transition
            .publish_commit(
                result,
                target_session_for_watch.clone(),
                session_watch_tx,
                sq_tx_watch_tx,
                bridge_rx_update_tx,
            )
            .await
        {
            Ok(_) => {
                let _ = ui_tx.send(UiOutput::HydrateHistory(resume_history_for_hydrate));
                emit_session_identity_after_queue_clear(
                    ui_tx,
                    target_session_for_watch.id.to_string(),
                );
                let text = format!(
                    "[System] Resumed session {}.\n",
                    target_session_for_watch.id
                );
                send_stream(ui_tx, MessageSource::System, text);
                Some(resume_config)
            }
            Err(error) => {
                let text = format!("[Error] {error}\n");
                send_stream(ui_tx, MessageSource::Error, text);
                None
            }
        },
        Err(e) => {
            transition.rollback();
            let text =
                format!("[Error] Failed to commit resume: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
            None
        }
    }
}

/// Handle `/fork` — clone the active session's durable history into a child session.
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
    let receipt_required = match crate::history_restore::requires_fork_receipt(&source_session) {
        Ok(required) => required,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Fork source unavailable: {error}\n"),
            );
            return;
        }
    };
    // Receipt publication is irreversible: hold child execution ownership from
    // before publication through metadata preparation and Actor handoff.
    let child_runtime_lease = if receipt_required {
        match talos_session::SessionRuntimeLease::acquire(&child_path) {
            Ok(lease) => Some(lease),
            Err(error) => {
                send_stream(
                    ui_tx,
                    MessageSource::Error,
                    format!(
                        "[Error] Cannot acquire fork child runtime ownership: {error}; artifacts retained.\n"
                    ),
                );
                return;
            }
        }
    } else {
        None
    };
    let mut captured_identity = None;
    let (mut staged_history, fork_history, fork_entry_id) = if receipt_required {
        let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();
        if transition
            .bind_active_sender()
            .send(SessionOp::ForkHistory {
                child_id: child_id.to_string(),
                child_branch: child_session.current_branch.clone(),
                through_entry: None,
                receipt_tx: Some(receipt_tx),
            })
            .await
            .is_err()
        {
            send_stream(
                ui_tx,
                MessageSource::Error,
                "[Error] Source runtime unavailable for fork.\n".into(),
            );
            return;
        }
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(35), receipt_rx.recv()).await;
        let tail = match result {
            Ok(Some(Ok(tail))) => tail,
            _ => {
                send_stream(
                    ui_tx,
                    MessageSource::Error,
                    format!(
                        "[Error] Fork publication did not complete for child {child_id} at {}; retained artifacts require reconciliation.\n",
                        child_path.display()
                    ),
                );
                return;
            }
        };
        let history = match crate::history_restore::read_session_history_owned(
            &child_session,
            child_runtime_lease
                .as_ref()
                .expect("receipt child ownership"),
        )
        .await
        {
            Ok(history) => history,
            Err(error) => {
                send_stream(
                    ui_tx,
                    MessageSource::Error,
                    format!(
                        "[Error] Published child {child_id} history unavailable: {error}; artifacts retained.\n"
                    ),
                );
                return;
            }
        };
        (
            crate::history_restore::StagedForkHistory::published_receipt(child_path.clone()),
            history,
            tail,
        )
    } else {
        let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();
        if transition
            .bind_active_sender()
            .send(SessionOp::CaptureOrdinaryFork {
                through_entry: None,
                receipt_tx: Some(receipt_tx),
            })
            .await
            .is_err()
        {
            send_stream(
                ui_tx,
                MessageSource::Error,
                "[Error] Source runtime unavailable for capture.\n".into(),
            );
            return;
        }
        let snapshot =
            match tokio::time::timeout(std::time::Duration::from_secs(35), receipt_rx.recv()).await
            {
                Ok(Some(Ok(snapshot))) => snapshot,
                result => {
                    let reason = match result {
                        Ok(Some(Err(error))) => error,
                        _ => "source capture timed out or disconnected".into(),
                    };
                    send_stream(
                        ui_tx,
                        MessageSource::Error,
                        format!("[Error] Cannot capture fork history: {reason}\n"),
                    );
                    return;
                }
            };
        let mut staged_history =
            match crate::history_restore::stage_captured_fork_history(&child_session, &snapshot) {
                Ok(guard) => guard,
                Err(error) => {
                    send_stream(
                        ui_tx,
                        MessageSource::Error,
                        format!("[Error] Failed to install captured fork history: {error}\n"),
                    );
                    return;
                }
            };
        staged_history.own_child(session_manager.clone(), child_session.clone());
        let fork_history = match crate::history_restore::read_session_history(&child_session).await
        {
            Ok(history) => history,
            Err(error) => {
                send_stream(
                    ui_tx,
                    MessageSource::Error,
                    format!("[Error] Failed to read captured child history: {error}\n"),
                );
                return;
            }
        };
        captured_identity = snapshot.runtime_identity;
        (staged_history, fork_history, snapshot.tail_id)
    };

    if receipt_required {
        if let Err(error) = child_session.restore_history_fork_runtime(
            child_runtime_lease
                .as_ref()
                .expect("receipt child ownership"),
            std::time::Instant::now() + std::time::Duration::from_secs(30),
        ) {
            let text = fork_failure_message(
                &mut staged_history,
                true,
                session_manager,
                &child_session,
                "Failed to restore captured fork runtime identity",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    } else {
        let inherited_identity = match captured_identity {
            Some((provider, model, variant)) => {
                talos_session::SessionRuntimeIdentity::new(&provider, &model, variant.as_deref())
            }
            None => talos_session::SessionRuntimeIdentity::new(
                &config.provider,
                &config.model,
                config.variant.as_deref(),
            ),
        };
        if let Err(error) = talos_session::PendingSubmissionStore::for_session(&child_session)
            .initialize_runtime_identity(inherited_identity)
        {
            let text = fork_failure_message(
                &mut staged_history,
                receipt_required,
                session_manager,
                &child_session,
                "Failed to initialize fork runtime identity",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    }

    let index_result = session_manager
        .update_index(&source_session)
        .and_then(|()| session_manager.update_index(&child_session))
        .and_then(|()| {
            session_manager.record_fork(&source_session.id, &child_session.id, &fork_entry_id)
        });
    #[cfg(test)]
    let index_result = if FORK_INDEX_FAILURE.try_with(|fail| *fail).unwrap_or(false) {
        Err(talos_session::IndexError::IoError(std::io::Error::other(
            "injected index publication failure",
        )))
    } else {
        index_result
    };
    if let Err(error) = index_result {
        let text = fork_failure_message(
            &mut staged_history,
            receipt_required,
            session_manager,
            &child_session,
            "Failed to publish child Session index/fork ownership",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let mut fork_config = config.clone();

    // Receipt absence means no inherited identity, not an unrecorded activation.
    // Persist the effective host selection before constructing the child runtime.
    if let Err(error) =
        apply_session_model_to_config(&mut fork_config, &child_session).and_then(|()| {
            crate::mode_runtime::ensure_session_runtime_identity(&fork_config, &child_session)
        })
    {
        let text = fork_failure_message(
            &mut staged_history,
            receipt_required,
            session_manager,
            &child_session,
            "Failed to persist selected fork runtime identity",
            error,
        );
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }
    #[cfg(test)]
    if let Ok(reached) = FORK_METADATA_CUT.try_with(Clone::clone) {
        reached.notify_one();
        std::future::pending::<()>().await;
    }
    let built_runtime = match runtime_builder
        .build(&fork_config, &child_session, fork_history)
        .await
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let text = fork_failure_message(
                &mut staged_history,
                receipt_required,
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
    let mut actor = built_runtime.actor;
    let child_runtime_lease = child_runtime_lease.or_else(|| staged_history.take_child_ownership());
    // Once the Actor receives ownership, cancellation must preserve the child,
    // regardless of local destruction order or transition progress.
    staged_history.retain();
    if let Some(lease) = child_runtime_lease
        && let Err(error) = actor.adopt_runtime_lease(lease)
    {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!(
                "[Error] Cannot transfer fork child runtime ownership: {error}; artifacts retained.\n"
            ),
        );
        return;
    }
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        transition.rollback();
        drop(actor);
        let text = fork_failure_message(
            &mut staged_history,
            receipt_required,
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
        drop(actor);
        let text = fork_failure_message(
            &mut staged_history,
            receipt_required,
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
                staged_history.retain();
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
                let text = fork_failure_message(
                    &mut staged_history,
                    true,
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
            let text = fork_failure_message(
                &mut staged_history,
                true,
                session_manager,
                &child_session_for_watch,
                "Failed to commit fork",
                error,
            );
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::Message;

    // Lifecycle cancellation tests isolate transition retirement; production capture
    // admission and contention are separately exercised by real Actor tests.
    async fn respond_to_ordinary_capture(
        source: &talos_session::Session,
        rx: &mut mpsc::Receiver<SessionOp>,
    ) {
        let Some(SessionOp::CaptureOrdinaryFork {
            through_entry,
            receipt_tx: Some(tx),
        }) = rx.recv().await
        else {
            panic!("expected ordinary capture");
        };
        let lease = talos_session::SessionRuntimeLease::acquire(&source.file_path)
            .expect("source ownership");
        let snapshot = source
            .capture_ordinary_fork(
                &lease,
                through_entry.as_deref(),
                std::time::Instant::now() + std::time::Duration::from_secs(5),
                || false,
            )
            .expect("capture");
        tx.send(Ok(talos_core::session::OrdinaryForkCapture {
            bytes: snapshot.bytes,
            tail_id: snapshot.tail.id,
            runtime_identity: snapshot
                .runtime_identity
                .map(|identity| (identity.provider, identity.model, identity.variant)),
        }))
        .expect("receipt");
    }

    #[tokio::test]
    async fn new_session_identity_preparation_excludes_competitors_until_cancelled() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let parent = manager
            .create_session("parent", &canonical_workspace_root(root.path()))
            .expect("parent");
        let (raw_tx, _raw_rx) = mpsc::channel(4);
        let transition = Arc::new(Mutex::new(
            SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner"),
        ));
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (route_tx, route_rx) = watch::channel(raw_tx);
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let (cut_tx, mut cut_rx) = mpsc::unbounded_channel();
        let child_path;
        {
            let operation = NEW_IDENTITY_CUT.scope(
                cut_tx,
                handle_session_new(
                    &transition,
                    &ui_tx,
                    &config,
                    &builder,
                    &manager,
                    &session_tx,
                    &route_tx,
                    &bridge_tx,
                ),
            );
            tokio::pin!(operation);
            child_path = tokio::select! {
                path = cut_rx.recv() => path.expect("identity prepared"),
                _ = &mut operation => panic!("unexpected completion"),
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => panic!("cut timeout"),
            };
            assert!(talos_session::SessionRuntimeLease::acquire(&child_path).is_err());
        }
        talos_session::SessionRuntimeLease::acquire(&child_path)
            .expect("cancel releases ownership");
        assert_eq!(transition.lock().await.active_session().id, parent.id);
        assert!(!session_rx.has_changed().expect("session route"));
        assert!(!route_rx.has_changed().expect("command route"));
        assert!(bridge_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn fork_handler_cancel_after_metadata_preserves_parent_and_removes_child() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let route = owner.bind_active_sender();
        let transition = Arc::new(Mutex::new(owner));
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (route_tx, route_rx) = watch::channel(raw_tx);
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let capture_source = parent.clone();
        let capture_responder = tokio::spawn(async move {
            respond_to_ordinary_capture(&capture_source, &mut raw_rx).await;
            raw_rx
        });

        let reached = Arc::new(tokio::sync::Notify::new());
        {
            let operation = FORK_METADATA_CUT.scope(
                reached.clone(),
                handle_session_fork(
                    &transition,
                    &ui_tx,
                    &config,
                    &builder,
                    &manager,
                    &session_tx,
                    &route_tx,
                    &bridge_tx,
                    &session_rx,
                ),
            );
            tokio::pin!(operation);
            tokio::select! {
                () = &mut operation => panic!("fork returned before cancellation cut"),
                () = reached.notified() => {},
                () = tokio::time::sleep(std::time::Duration::from_secs(10)) => panic!("cut deadline"),
            }
            assert_eq!(
                manager
                    .get_forks(&parent.id.to_string())
                    .expect("staged lineage")
                    .len(),
                1
            );
        }
        assert!(
            manager
                .get_forks(&parent.id.to_string())
                .expect("clean lineage")
                .is_empty()
        );
        let sessions = manager.list_sessions().expect("sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, parent.id);
        assert_eq!(transition.lock().await.active_session().id, parent.id);
        assert!(!session_rx.has_changed().expect("session watch"));
        assert!(!route_rx.has_changed().expect("route watch"));
        assert!(bridge_rx.try_recv().is_err());
        let mut raw_rx = capture_responder.await.expect("capture responder");
        route
            .send(SessionOp::Submit {
                message: "parent remains active".into(),
            })
            .await
            .expect("admission");
        assert!(
            matches!(tokio::time::timeout(std::time::Duration::from_secs(2), raw_rx.recv()).await.expect("delivery"),
            Some(SessionOp::Submit { message }) if message == "parent remains active")
        );
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
    }

    #[tokio::test]
    async fn fork_handler_success_publishes_retained_child_with_original_history() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let built = builder
            .build(&config, &parent, vec![])
            .await
            .expect("parent runtime");
        let raw_tx = built.handle.sq_tx.clone();
        let mut actor = built.actor;
        let mut owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let actor_join = tokio::spawn(async move { actor.run().await });
        let cancel = tokio_util::sync::CancellationToken::new();
        let stopped = cancel.clone();
        let scheduler_join = tokio::spawn(async move { stopped.cancelled().await });
        owner
            .attach_active_runtime(actor_join, cancel, scheduler_join)
            .expect("attach");
        let transition = Arc::new(Mutex::new(owner));
        let (route_tx, route_rx) = watch::channel(raw_tx);

        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            handle_session_fork(
                &transition,
                &ui_tx,
                &config,
                &builder,
                &manager,
                &session_tx,
                &route_tx,
                &bridge_tx,
                &session_rx,
            ),
        )
        .await
        .expect("fork completes");
        let child = session_rx.borrow().clone();
        assert_ne!(child.id, parent.id);
        assert_eq!(transition.lock().await.active_session().id, child.id);
        assert_eq!(
            std::fs::read(&child.file_path).expect("retained child"),
            original
        );
        assert_eq!(
            child.read_messages().expect("child history"),
            parent.read_messages().expect("parent history")
        );
        assert_eq!(
            manager
                .get_forks(&parent.id.to_string())
                .expect("lineage")
                .len(),
            1
        );
        assert!(session_rx.has_changed().expect("session watch"));
        assert!(route_rx.has_changed().expect("command watch"));
        assert_eq!(bridge_rx.try_recv().expect("bridge").0.id, child.id);
        transition.lock().await.shutdown().await;
        assert!(
            child.file_path.exists(),
            "successful child is retained after shutdown"
        );
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
    }

    #[tokio::test]
    async fn archived_fork_handler_uses_real_actor_and_preserves_all_history() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        for i in 0..15 {
            parent
                .append(&Message::User {
                    content: format!("archived fact {i}"),
                })
                .expect("append");
        }
        parent.compact_archived(0).expect("keep0");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let built = builder
            .build(&config, &parent, vec![])
            .await
            .expect("parent runtime");
        let raw_tx = built.handle.sq_tx.clone();
        let mut actor = built.actor;
        let mut owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let actor_join = tokio::spawn(async move { actor.run().await });
        let cancel = tokio_util::sync::CancellationToken::new();
        let stopped = cancel.clone();
        let scheduler_join = tokio::spawn(async move { stopped.cancelled().await });
        owner
            .attach_active_runtime(actor_join, cancel, scheduler_join)
            .expect("attach");
        let transition = Arc::new(Mutex::new(owner));
        let (route_tx, route_rx) = watch::channel(raw_tx);

        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            handle_session_fork(
                &transition,
                &ui_tx,
                &config,
                &builder,
                &manager,
                &session_tx,
                &route_tx,
                &bridge_tx,
                &session_rx,
            ),
        )
        .await
        .expect("fork completes");
        let child = session_rx.borrow().clone();
        assert_ne!(child.id, parent.id);
        assert_eq!(transition.lock().await.active_session().id, child.id);
        let restored = crate::history_restore::read_session_history(&child)
            .await
            .expect("child restore");
        assert_eq!(restored.len(), 16);
        assert!(restored.iter().any(
            |message| matches!(message, Message::User { content } if content == "archived fact 0")
        ));
        assert_eq!(
            manager
                .get_forks(&parent.id.to_string())
                .expect("lineage")
                .len(),
            1
        );
        assert!(session_rx.has_changed().expect("session watch"));
        assert!(route_rx.has_changed().expect("command watch"));
        assert_eq!(bridge_rx.try_recv().expect("bridge").0.id, child.id);
        transition.lock().await.shutdown().await;
        assert!(
            child.file_path.exists(),
            "successful child is retained after shutdown"
        );
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
    }

    #[tokio::test]
    async fn archived_fork_cancel_after_metadata_retains_published_child() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        for i in 0..15 {
            parent
                .append(&Message::User {
                    content: format!("archived fact {i}"),
                })
                .expect("append");
        }
        parent.compact_archived(0).expect("keep0");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let built = builder
            .build(&config, &parent, vec![])
            .await
            .expect("parent runtime");
        let raw_tx = built.handle.sq_tx.clone();
        let mut actor = built.actor;
        let mut owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let actor_join = tokio::spawn(async move { actor.run().await });
        let cancel = tokio_util::sync::CancellationToken::new();
        let stopped = cancel.clone();
        let scheduler_join = tokio::spawn(async move { stopped.cancelled().await });
        owner
            .attach_active_runtime(actor_join, cancel, scheduler_join)
            .expect("attach");
        let transition = Arc::new(Mutex::new(owner));
        let (route_tx, route_rx) = watch::channel(raw_tx);

        let reached = Arc::new(tokio::sync::Notify::new());
        {
            let operation = FORK_METADATA_CUT.scope(
                reached.clone(),
                handle_session_fork(
                    &transition,
                    &ui_tx,
                    &config,
                    &builder,
                    &manager,
                    &session_tx,
                    &route_tx,
                    &bridge_tx,
                    &session_rx,
                ),
            );
            tokio::pin!(operation);
            tokio::select! {
                _ = reached.notified() => {},
                _ = &mut operation => panic!("must pause after metadata"),
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => panic!("deadline"),
            }
            let published_child = std::fs::read_dir(parent.file_path.parent().expect("parent dir"))
                .expect("inventory")
                .map(|entry| entry.expect("entry").path())
                .find(|path| {
                    path != &parent.file_path
                        && path.extension().is_some_and(|ext| ext == "tlog")
                        && path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .is_some_and(|stem| uuid::Uuid::parse_str(stem).is_ok())
                })
                .expect("published child");
            assert!(
                talos_session::SessionRuntimeLease::acquire(&published_child).is_err(),
                "published child stays fenced during metadata-to-Actor preparation"
            );
        }
        assert_eq!(session_rx.borrow().id, parent.id);
        assert_eq!(transition.lock().await.active_session().id, parent.id);
        let children: Vec<_> = std::fs::read_dir(parent.file_path.parent().expect("directory"))
            .expect("inventory")
            .map(|entry| entry.expect("entry").path())
            .filter(|path| {
                path != &parent.file_path
                    && path.extension().is_some_and(|ext| ext == "tlog")
                    && path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
            })
            .collect();
        assert_eq!(children.len(), 1, "committed history survives cancellation");
        let id = uuid::Uuid::parse_str(
            children[0]
                .file_stem()
                .expect("stem")
                .to_str()
                .expect("text"),
        )
        .expect("id");
        let child = manager.get_session(&id).expect("retained child");
        let identity = talos_session::PendingSubmissionStore::for_session(&child)
            .runtime_state()
            .expect("child runtime metadata")
            .expect("selected identity persisted");
        assert_eq!(
            identity.status,
            talos_session::SessionRuntimeActivationStatus::Committed
        );
        assert_eq!(
            identity.activation.target,
            talos_session::SessionRuntimeIdentity::new(
                &config.provider,
                &config.model,
                config.variant.as_deref()
            )
        );

        drop(
            talos_session::SessionRuntimeLease::acquire(&child.file_path)
                .expect("cancelled preparation releases child ownership"),
        );
        assert_eq!(
            crate::history_restore::read_session_history(&child)
                .await
                .expect("history")
                .len(),
            16
        );
        assert!(!route_rx.has_changed().expect("old route"));
        assert!(bridge_rx.try_recv().is_err());
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
        transition.lock().await.shutdown().await;
    }

    #[tokio::test]
    async fn archived_fork_metadata_error_retains_published_child() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        for i in 0..15 {
            parent
                .append(&Message::User {
                    content: format!("archived fact {i}"),
                })
                .expect("append");
        }
        parent.compact_archived(0).expect("keep0");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        let built = builder
            .build(&config, &parent, vec![])
            .await
            .expect("parent runtime");
        let raw_tx = built.handle.sq_tx.clone();
        let mut actor = built.actor;
        let mut owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let actor_join = tokio::spawn(async move { actor.run().await });
        let cancel = tokio_util::sync::CancellationToken::new();
        let stopped = cancel.clone();
        let scheduler_join = tokio::spawn(async move { stopped.cancelled().await });
        owner
            .attach_active_runtime(actor_join, cancel, scheduler_join)
            .expect("attach");
        let transition = Arc::new(Mutex::new(owner));
        let (route_tx, route_rx) = watch::channel(raw_tx);

        FORK_INDEX_FAILURE
            .scope(
                true,
                handle_session_fork(
                    &transition,
                    &ui_tx,
                    &config,
                    &builder,
                    &manager,
                    &session_tx,
                    &route_tx,
                    &bridge_tx,
                    &session_rx,
                ),
            )
            .await;
        assert_eq!(session_rx.borrow().id, parent.id);
        assert_eq!(transition.lock().await.active_session().id, parent.id);
        let children: Vec<_> = std::fs::read_dir(parent.file_path.parent().expect("directory"))
            .expect("inventory")
            .map(|entry| entry.expect("entry").path())
            .filter(|path| {
                path != &parent.file_path
                    && path.extension().is_some_and(|ext| ext == "tlog")
                    && path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
            })
            .collect();
        assert_eq!(children.len(), 1, "committed history survives cancellation");
        let id = uuid::Uuid::parse_str(
            children[0]
                .file_stem()
                .expect("stem")
                .to_str()
                .expect("text"),
        )
        .expect("id");
        let child = manager.get_session(&id).expect("retained child");
        assert_eq!(
            crate::history_restore::read_session_history(&child)
                .await
                .expect("history")
                .len(),
            16
        );
        assert!(!route_rx.has_changed().expect("old route"));
        assert!(bridge_rx.try_recv().is_err());
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
        transition.lock().await.shutdown().await;
    }

    #[tokio::test]
    async fn fork_handler_cancel_during_commit_retains_resumable_child_and_finishes_retirement() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let parent = manager
            .create_session("parent", &workspace)
            .expect("parent");
        parent
            .append(&Message::User {
                content: "original constraint".into(),
            })
            .expect("history");
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let mut owner = SessionTransition::new(raw_tx.clone(), parent.clone()).expect("owner");
        let route = owner.bind_active_sender();
        let stopped = tokio_util::sync::CancellationToken::new();
        let stop = stopped.clone();
        let release = Arc::new(tokio::sync::Notify::new());
        let scheduler_release = release.clone();
        let scheduler = tokio::spawn(async move {
            stop.cancelled().await;
            scheduler_release.notified().await;
        });
        let capture_source = parent.clone();
        let actor = tokio::spawn(async move {
            respond_to_ordinary_capture(&capture_source, &mut raw_rx).await;
            assert!(matches!(raw_rx.recv().await, Some(SessionOp::Shutdown)));
        });
        owner
            .attach_active_runtime(actor, stopped.clone(), scheduler)
            .expect("runtime");
        let transition = Arc::new(Mutex::new(owner));
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(parent.clone());
        let (route_tx, route_rx) = watch::channel(raw_tx);
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let config = Config::default();
        {
            let operation = handle_session_fork(
                &transition,
                &ui_tx,
                &config,
                &builder,
                &manager,
                &session_tx,
                &route_tx,
                &bridge_tx,
                &session_rx,
            );
            tokio::pin!(operation);
            tokio::select! {
                () = &mut operation => panic!("fork returned before cancellation cut"),
                () = stopped.cancelled() => {},
                () = tokio::time::sleep(std::time::Duration::from_secs(10)) => panic!("cut deadline"),
            }
            assert_eq!(
                manager
                    .get_forks(&parent.id.to_string())
                    .expect("staged lineage")
                    .len(),
                1
            );
        }
        assert_eq!(
            manager
                .get_forks(&parent.id.to_string())
                .expect("retained lineage")
                .len(),
            1
        );
        let sessions = manager.list_sessions().expect("sessions");
        assert_eq!(sessions.len(), 2);
        let child_id = sessions
            .iter()
            .find(|info| info.id != parent.id)
            .expect("retained child")
            .id;
        let child = manager.get_session(&child_id).expect("discoverable child");
        let lease = talos_session::SessionRuntimeLease::acquire(&child.file_path)
            .expect("cancelled Actor releases child ownership");
        let history = crate::history_restore::read_session_history_owned(&child, &lease)
            .await
            .expect("child can resume");
        assert_eq!(history.len(), 1);
        assert_eq!(
            std::fs::read(&child.file_path).expect("child bytes"),
            original
        );
        drop(lease);
        assert_eq!(transition.lock().await.active_session().id, parent.id);
        assert!(!session_rx.has_changed().expect("session watch"));
        assert!(!route_rx.has_changed().expect("route watch"));
        assert!(bridge_rx.try_recv().is_err());
        release.notify_one();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            transition.lock().await.shutdown().await;
            route.closed().await;
        })
        .await
        .expect("retirement completes after cancellation");
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
    }

    #[tokio::test]
    async fn resume_corrupt_history_keeps_original_session_and_command_route() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let workspace = canonical_workspace_root(root.path());
        let original = manager
            .create_session("original", &workspace)
            .expect("original");
        let target = manager
            .create_session("target", &workspace)
            .expect("target");
        target
            .append(&Message::User {
                content: "valid inventory entry".into(),
            })
            .expect("nonempty target");
        let bytes = std::fs::read(&target.file_path).expect("history");
        let end = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .expect("header");
        // Legacy inventory skips TALOS headers; strict restoration validates the
        // version before retiring the active runtime.
        let mut corrupt = b"TALOS\t99\t1000\n".to_vec();
        corrupt.extend_from_slice(&bytes[end + 1..]);
        std::fs::write(&target.file_path, corrupt).expect("invalid header version");
        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let owner = SessionTransition::new(raw_tx.clone(), original.clone()).expect("owner");
        let route = owner.bind_active_sender();
        let generation = owner.active_generation();
        let transition = Arc::new(Mutex::new(owner));
        let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
        let (session_tx, session_rx) = watch::channel(original.clone());
        let (route_tx, route_rx) = watch::channel(raw_tx);
        let (bridge_tx, mut bridge_rx) = mpsc::unbounded_channel();
        let builder = TuiRuntimeBuilder::new(
            build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            Arc::new(TuiApprovalHandler::new(
                ui_tx.clone(),
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            handle_session_resume(
                &transition,
                &ui_tx,
                &Config::default(),
                &builder,
                &manager,
                &session_tx,
                &route_tx,
                &bridge_tx,
                Some(target.id.to_string()),
            ),
        )
        .await
        .expect("history refusal must not retire the original runtime");
        assert!(result.is_none());
        assert_eq!(transition.lock().await.active_session().id, original.id);
        assert_eq!(transition.lock().await.active_generation(), generation);
        assert_eq!(session_rx.borrow().id, original.id);
        assert!(!session_rx.has_changed().expect("session watch"));
        assert!(!route_rx.has_changed().expect("route watch"));
        assert!(bridge_rx.try_recv().is_err());
        let mut outputs = Vec::new();
        while let Ok(output) = ui_rx.try_recv() {
            if let UiOutput::Content(ContentOutput::Block { text, .. }) = output {
                outputs.push(text);
            }
        }
        assert!(
            outputs
                .iter()
                .any(|output| output.contains("Failed to read session history")),
            "{outputs:?}"
        );
        route
            .send(SessionOp::Submit {
                message: "original still works".into(),
            })
            .await
            .expect("admission");
        let operation = tokio::time::timeout(std::time::Duration::from_secs(2), raw_rx.recv())
            .await
            .expect("delivery deadline")
            .expect("old route");
        assert!(
            matches!(operation, SessionOp::Submit { message } if message == "original still works")
        );
    }

    fn info(id: uuid::Uuid, message_count: usize, age_minutes: u64) -> talos_session::SessionInfo {
        talos_session::SessionInfo {
            id,
            project: "test".to_string(),
            workspace_root: "/test".to_string(),
            last_message_preview: String::new(),
            timestamp: (std::time::SystemTime::now()
                - std::time::Duration::from_secs(age_minutes.saturating_mul(60)))
            .into(),
            message_count,
        }
    }

    #[test]
    fn resume_candidates_exclude_active_and_empty_sessions_then_sort_newest_first() {
        let active = uuid::Uuid::new_v4();
        let empty = uuid::Uuid::new_v4();
        let older = uuid::Uuid::new_v4();
        let newer = uuid::Uuid::new_v4();
        let mut sessions = vec![
            info(active, 3, 0),
            info(empty, 0, 1),
            info(older, 2, 3),
            info(newer, 1, 2),
        ];

        retain_resumable_sessions(&mut sessions, active);

        assert_eq!(
            sessions
                .iter()
                .map(|session| session.id)
                .collect::<Vec<_>>(),
            vec![newer, older]
        );
    }
}
