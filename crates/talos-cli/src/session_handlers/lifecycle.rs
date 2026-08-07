//! Session lifecycle and cleanup-recovery workflows.

use super::super::*;

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

    let target_session = match &session_id {
        Some(id) => {
            // Try parsing as ordinal (1-based) first, then fall back to UUID.
            if let Ok(n) = id.parse::<usize>() {
                let sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Failed to list sessions: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return None;
                    }
                };
                if sessions.is_empty() {
                    let text = "[System] No sessions found for this workspace.\n".to_string();
                    send_stream(ui_tx, MessageSource::System, text);
                    return None;
                }
                let mut sessions = sessions;
                sessions
                    .sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
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
                // Fall back to treating it as a UUID (backward compat).
                match session_manager.resume_session(id) {
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
            let sessions = match session_manager.list_workspace_sessions(&workspace_root_str) {
                Ok(s) => s,
                Err(e) => {
                    let text = format!("[Error] Failed to list sessions: {e}\n");
                    send_stream(ui_tx, MessageSource::Error, text);
                    return None;
                }
            };

            if sessions.is_empty() {
                let text = "[System] No sessions found for this workspace.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return None;
            }

            let mut sessions = sessions;
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

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

    if let Err(error) = crate::mode_runtime::reconcile_session_runtime_state(&target_session) {
        let text = format!("[Error] Cannot resume stopped Session runtime: {error}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }
    let mut resume_config = config.clone();
    apply_session_model_to_config(&mut resume_config, &target_session);
    if let Err(error) =
        crate::mode_runtime::ensure_session_runtime_identity(&resume_config, &target_session)
    {
        let text =
            format!("[Error] Failed to establish resumed Session runtime identity: {error}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }
    let resume_history = match target_session.read_messages() {
        Ok(h) => h,
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
    let actor = built_runtime.actor;
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

    let inherited_identity =
        match talos_session::PendingSubmissionStore::for_session(&source_session).runtime_state() {
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
