//! Session lifecycle handlers for the Talos TUI.
//!
//! Extracted from `mode_runners.rs` to reduce file size and improve
//! maintainability. All functions are behavior-preserving.

use std::sync::Arc;

use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{MessageSource, SessionPickerItem, StreamMessage, UiOutput};
use talos_core::message::Message;
use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};
use talos_plugin::HookRegistry;
use tokio::sync::{mpsc, watch, Mutex};

use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    apply_session_model_to_config, context_files_for_agent, maybe_set_memory_provider,
    set_todo_prompt_provider,
};
use crate::model_lifecycle::{
    RebuildSessionParams, build_model_picker_data, provider_setup_target_model,
    rebuild_session_for_model,
};
use crate::provider_setup::build_provider;
use crate::registry::{build_tui_tool_registry, register_tui_permission_aware_tools, TuiApprovalHandler};
use crate::session_setup::canonical_workspace_root;
use crate::session_transition::SessionTransition;
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};


pub(crate) fn send_stream(ui_tx: &mpsc::UnboundedSender<UiOutput>, source: MessageSource, text: String) {
    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(futures::stream::once(async move { text })),
    }));
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

            let target = match arg.parse::<usize>() {
                Ok(n) if n >= 1 && n <= sessions.len() => &sessions[n - 1],
                _ => {
                    let text = format!(
                        "[Error] Invalid selection '{arg}'. Use /delete to pick a session.\n"
                    );
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
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


/// Handle `/new` — create a fresh session and transition to it.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_new(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    model_context_limit: u32,
    mock: bool,
) {
    let mut transition = transition.lock().await;

    let session_manager = session_manager.clone();
    let workspace_root_str = canonical_workspace_root(workspace_root);
    let new_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(s) => s,
        Err(e) => {
            let text = format!("[Error] Failed to create new session: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let new_history: Vec<Message> = vec![];
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: new_history,
        model_context_limit,
    };

    let provider = build_provider(config, api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    mcp_runtime.report_startup_failures();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        new_session.id,
    );
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root, config.skills.discover_shared) {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, config);
    set_todo_prompt_provider(&mut agent, &session_manager, &new_session);

    let (handle, actor) = AppServerSession::new(agent, session_config);

    // Clone for watch channel update after commit (new_session is moved into prepare).
    let new_session_for_watch = new_session.clone();
    if let Err(e) = transition.prepare(handle, new_session) {
        let _ = std::fs::remove_file(&new_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare new session: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(new_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((new_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; new session events will not be persisted or displayed."
                );
            }
            let _ = ui_tx.send(UiOutput::SessionIdentity {
                id: new_session_for_watch.id.to_string(),
            });
            let text = "[System] New session started. Previous session preserved.\n".to_string();
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&new_session_for_watch.file_path);
            let text =
                format!("[Error] Failed to commit new session: {e}. Old session remains active.\n");
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
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    _model_context_limit: u32,
    session_id: Option<String>,
    mock: bool,
) -> Option<Config> {
    let mut transition = transition.lock().await;

    let workspace_root_str = canonical_workspace_root(workspace_root);

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

    let mut resume_config = config.clone();
    apply_session_model_to_config(&mut resume_config, &target_session);
    let resume_api_key = match resume_config.api_key() {
        Ok(key) => key,
        Err(e) if mock => {
            tracing::warn!("failed to resolve resumed session api key in mock mode: {e}");
            api_key.to_string()
        }
        Err(e) => {
            let text = format!(
                "[Error] Failed to resolve API key for resumed session model '{}': {e}\n",
                resume_config.model
            );
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    let resume_model_context_limit = resume_config.resolve_model_limits().0;

    let resume_history = match target_session.read_messages() {
        Ok(h) => h,
        Err(e) => {
            let text = format!("[Error] Failed to read session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

    let resume_history_for_hydrate = resume_history.clone();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: resume_history,
        model_context_limit: resume_model_context_limit,
    };

    let provider = build_provider(&resume_config, &resume_api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };
    mcp_runtime.report_startup_failures();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        target_session.id,
    );
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(resume_config.tool_protocol());
    if let Ok(skills) =
        discover_runtime_skills(workspace_root, resume_config.skills.discover_shared)
    {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, &resume_config);
    set_todo_prompt_provider(&mut agent, session_manager, &target_session);
    match context_files_for_agent(&resume_config, workspace_root, true) {
        Ok(files) => agent.set_context_files(files),
        Err(e) => {
            let text = format!("[Error] Failed to load context files: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    }

    let (handle, actor) = AppServerSession::new(agent, session_config);

    // Clone for watch channel update after commit (target_session is moved into prepare).
    let target_session_for_watch = target_session.clone();
    if let Err(e) = transition.prepare(handle, target_session) {
        let _ = std::fs::remove_file(&target_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare resume: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(target_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((target_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; resumed session events will not be persisted or displayed."
                );
            }
            let _ = ui_tx.send(UiOutput::HydrateHistory(resume_history_for_hydrate));
            let _ = ui_tx.send(UiOutput::SessionIdentity {
                id: target_session_for_watch.id.to_string(),
            });
            let text = format!(
                "[System] Resumed session {}.\n",
                target_session_for_watch.id
            );
            send_stream(ui_tx, MessageSource::System, text);
            Some(resume_config)
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&target_session_for_watch.file_path);
            let text =
                format!("[Error] Failed to commit resume: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
            None
        }
    }
}


/// Handle `/fork` — clone the active session's durable history into a child session.
///
/// Copies the source JSONL file to a new path with a fresh UUID, creates a new
/// [`talos_session::Session`], and swaps the agent context. The source session
/// remains byte-for-byte unchanged.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_fork(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    api_key: &str,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    session_manager: &talos_session::SessionManager,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    model_context_limit: u32,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    mock: bool,
) {
    let mut transition = transition.lock().await;

    let source_session = session_watch_rx.borrow().clone();

    let source_bytes = match source_session.snapshot_bytes() {
        Ok(b) => b,
        Err(e) => {
            let text = format!("[Error] Failed to read source session file: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let fork_history = match source_session.read_messages() {
        Ok(h) => h,
        Err(e) => {
            let text = format!("[Error] Failed to read source session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let workspace_root_str = canonical_workspace_root(workspace_root);
    let child_session = match session_manager.defer_create_session("talos", &workspace_root_str) {
        Ok(s) => s,
        Err(e) => {
            let text = format!("[Error] Failed to create child session: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    let child_id = child_session.id;

    let child_path = child_session.file_path.clone();
    if let Some(parent) = child_path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        let text = format!("[Error] Failed to create child session directory: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    if let Err(e) = std::fs::write(&child_path, &source_bytes) {
        let text = format!("[Error] Failed to clone session history: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: fork_history,
        model_context_limit,
    };

    let provider = build_provider(config, api_key, mock);
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_tx.clone(),
        workspace_root.to_path_buf(),
    ));
    let mcp_runtime = match McpSessionRuntime::start(mcp_config, hooks.clone()).await {
        Ok(r) => r,
        Err(e) => {
            let text = format!("[Error] Failed to start MCP runtime: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };
    mcp_runtime.report_startup_failures();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        child_session.id,
    );
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root, config.skills.discover_shared) {
        apply_runtime_skills(&mut agent, &skills);
    }
    maybe_set_memory_provider(&mut agent, config);
    set_todo_prompt_provider(&mut agent, session_manager, &child_session);

    let (handle, actor) = AppServerSession::new(agent, session_config);

    // Clone for watch channel update after commit (child_session is moved into prepare).
    let child_session_for_watch = child_session.clone();
    if let Err(e) = transition.prepare(handle, child_session) {
        let _ = std::fs::remove_file(&child_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare fork: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(child_session_for_watch.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((child_session_for_watch.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!(
                    "[Error] Bridge forwarder unavailable; forked session events will not be persisted or displayed."
                );
            }
            let _ = ui_tx.send(UiOutput::SessionIdentity {
                id: child_session_for_watch.id.to_string(),
            });
            let text = format!(
                "[System] Forked session {child_id} (source: {}).\n",
                result.old_session.id
            );
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&child_session_for_watch.file_path);
            let text = format!("[Error] Failed to commit fork: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}


#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    session_manager: &talos_session::SessionManager,
    model_id: String,
    mock: bool,
) -> Option<Config> {
    if model_id.is_empty() {
        let data = build_model_picker_data(config);
        let _ = ui_tx.send(UiOutput::ModelPicker(data));
        return None;
    }

    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let mut model_config = config.clone();
    if let Err(e) = model_config.set_active_model(&model_id) {
        let text = format!("[Error] Unknown model '{model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    let provider_name = model_config.provider.clone();

    // No-op if the selected model+provider is already active.
    if config.model == model_id && config.provider == provider_name {
        return None;
    }

    if !model_config.provider_authenticated(&provider_name) {
        let _ = ui_tx.send(UiOutput::CredentialRequest(
            talos_conversation::CredentialRequestData {
                provider: provider_name,
                model_id: Some(model_id.clone()),
                connect_mode: false,
                default_base_url: None,
            },
        ));
        return None;
    }

    let api_key = match model_config.api_key() {
        Ok(k) => k,
        Err(e) => {
            let text = format!("[Error] Failed to resolve API key for {provider_name}: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return None;
        }
    };

    if rebuild_session_for_model(RebuildSessionParams {
        transition,
        ui_tx,
        model_config: &model_config,
        hooks,
        workspace_root,
        mcp_config,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        session_manager,
        api_key,
        previous_model,
        previous_provider,
        model_id: model_id.clone(),
        provider_for_status: provider_name,
        success_message: format!("[System] Switched to model {model_id}.\n"),
        mock,
    })
    .await
    {
        if let Err(e) = model_config.save() {
            let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
        Some(model_config)
    } else {
        None
    }
}


#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model_with_credential(
    transition: &Arc<Mutex<SessionTransition>>,
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    hooks: &Arc<HookRegistry>,
    workspace_root: &std::path::Path,
    mcp_config: &talos_config::McpConfig,
    session_watch_tx: &watch::Sender<talos_session::Session>,
    sq_tx_watch_tx: &watch::Sender<mpsc::Sender<SessionOp>>,
    bridge_rx_update_tx: &mpsc::UnboundedSender<(
        talos_session::Session,
        mpsc::UnboundedReceiver<SessionEvent>,
    )>,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    session_manager: &talos_session::SessionManager,
    cred: talos_conversation::CredentialResponseData,
    mock: bool,
) -> Option<Config> {
    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let mut model_config = config.clone();
    model_config.set_provider_credential(&cred.provider, &cred.api_key);
    if let Err(e) = model_config.save() {
        let text = format!("[Error] Failed to persist credentials: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    let model_id = match &cred.model_id {
        Some(id) => id.clone(),
        None => match provider_setup_target_model(&model_config, &cred.provider) {
            Some(id) => id,
            None => {
                let text = format!(
                    "[Error] Credentials saved, but no models are configured for provider '{}'.\n",
                    cred.provider
                );
                send_stream(ui_tx, MessageSource::Error, text);
                return None;
            }
        },
    };

    if let Err(e) = model_config.set_active_model(&model_id) {
        let text = format!("[Error] Unknown model '{model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return None;
    }

    let api_key = cred.api_key.clone();
    let provider_for_status = model_config.provider.clone();

    if rebuild_session_for_model(RebuildSessionParams {
        transition,
        ui_tx,
        model_config: &model_config,
        hooks,
        workspace_root,
        mcp_config,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        session_manager,
        api_key,
        previous_model,
        previous_provider,
        model_id: model_id.clone(),
        provider_for_status,
        success_message: format!("[System] Credentials saved. Switched to model {model_id}.\n"),
        mock,
    })
    .await
    {
        if let Err(e) = model_config.save() {
            let text = format!("[Error] Model switched, but failed to persist config: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
        Some(model_config)
    } else {
        None
    }
}
