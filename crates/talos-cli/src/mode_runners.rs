//! Runtime mode runner implementations for the Talos CLI.

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rmcp::ServiceExt;
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{ConversationEngine, MessageSource, ModelPickerItem, SessionPickerItem, StreamMessage, UiOutput, UserInput};
use talos_core::message::{AgentEvent, Message};
use talos_core::session::{SessionConfig, SessionEvent, SessionOp};
use talos_core::tool::ToolRegistry;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_plugin::HookRegistry;
use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::{
    BashTool, DeleteTool, DiffTool, EditTool, GlobTool, GrepTool, LsTool, ReadTool, StatTool,
    TreeTool, WriteTool,
};
use talos_tui::Tui;
use tokio::sync::{mpsc, watch};

use crate::approval::ApprovalPrompt;
use crate::logging::init_logger;
use crate::mcp_runtime::McpSessionRuntime;
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{
    PermissionAwareTool, TuiApprovalHandler, build_mcp_tool_registry, build_print_tool_registry,
    build_tui_tool_registry, register_permission_aware_tools, register_tui_permission_aware_tools,
};
use crate::runtime_adapter;
use crate::session_setup::{
    ResumeSelection, canonical_workspace_root, resolve_prompt, resolve_session_for_workspace,
    resolve_workspace_root, workspace_display_name,
};
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};
use crate::session_transition::SessionTransition;
use crate::tui_bridge::{SessionLifecycleRequest, run_conversation_loop};
use crate::{Cli, build_hook_registry, event_loop};
use tokio::sync::Mutex;

pub(crate) async fn run_rpc_mode(cli: Cli) -> Result<()> {
    // I009-S5 begin
    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() && !cli.mock {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = if cli.mock {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let hooks = build_hook_registry(true);
    let workspace_root = PathBuf::from(".");
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let mut registry = build_print_tool_registry();
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), mcp_approval, true);
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root,
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    apply_runtime_skills(&mut agent, &runtime_skills);

    let server = talos_rpc::RpcServer::new(Arc::new(runtime_adapter::AgentRuntime(agent)));
    server.run_stdio().await
    // I009-S5 end
}

fn send_stream(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    source: MessageSource,
    text: String,
) {
    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(futures::stream::once(async move { text })),
    }));
}

pub(crate) async fn run_print_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() && !cli.mock {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = if cli.mock {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let workspace_root = resolve_workspace_root(&cli)?;
    apply_mcp_fixture_config(&mut config, &cli);
    let prompt = resolve_prompt(cli.prompt)?;

    let hooks = build_hook_registry(true);
    let mut registry = build_print_tool_registry();

    #[cfg(debug_assertions)]
    let fixture_mode = cli.mcp_server_fixture.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
    let request_preview_mode = prompt.trim_start().starts_with("/mock-request");

    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), mcp_approval, true);

    let provider = if fixture_mode && cli.mock && !request_preview_mode {
        use talos_provider::mock::MockProvider;
        Arc::new(
            MockProvider::new()
                .with_tool_call("mcp:fixture:echo", serde_json::json!({ "text": "ping" }))
                .with_response("fixture tool call complete"),
        ) as Arc<dyn talos_core::provider::LanguageModel>
    } else {
        build_provider(&config, &api_key, cli.mock)
    };
    // I009-S3 end

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(
            talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
        )),
        None,
        workspace_root.to_path_buf(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    apply_runtime_skills(&mut agent, &runtime_skills);

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.to_path_buf())
            .load()
            .map_err(|e| anyhow!("{e}"))?;
        if !context.is_empty() {
            agent.set_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: context,
            }]);
        }
    }

    if let Some(ref system_prompt) = cli.system_prompt {
        agent.set_custom_prompt(system_prompt.clone());
    }

    if let Some(ref append_prompt) = cli.append_system_prompt {
        agent.set_append_prompt(append_prompt.clone());
    }

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        print_mode: true,
        workspace_root: workspace_root.to_path_buf(),
        initial_history: vec![],
        model_context_limit,
    };
    let (mut handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    handle
        .sq_tx
        .send(SessionOp::Submit { message: prompt })
        .await
        .context("failed to submit message to session")?;

    let mut stdout = io::stdout().lock();
    while let Some(event) = handle.eq_rx.recv().await {
        match event {
            SessionEvent::AgentEvent(AgentEvent::TextDelta { delta }) => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
            SessionEvent::AgentEvent(AgentEvent::TurnEnd { .. }) => {
                println!();
                return Ok(());
            }
            SessionEvent::AgentEvent(AgentEvent::Error { message }) => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
            SessionEvent::TurnCompleted { status, .. } => match status {
                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    println!();
                    return Ok(());
                }
                talos_core::session::TurnCompletionStatus::Cancelled => {
                    return Ok(());
                }
                talos_core::session::TurnCompletionStatus::Error { message } => {
                    eprintln!("Error: {message}");
                    std::process::exit(1);
                }
            },
            SessionEvent::Error { message } => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
            SessionEvent::AgentEvent(_) => {}
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_session_delete(
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
                    let text = format!("[Error] Invalid selection '{arg}'. Use /delete to pick a session.\n");
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

#[allow(clippy::too_many_arguments)]
async fn handle_session_model(
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
    model_id: String,
    mock: bool,
) {
    if model_id.is_empty() {
        let catalog = talos_config::model::builtin_models();
        let items: Vec<ModelPickerItem> = catalog
            .iter()
            .map(|m| {
                let provider_authed = config.provider_authenticated(&m.provider);
                let pricing_str = m.pricing.as_ref().map(|p| {
                    let input = p.input_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                    let output = p.output_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                    if input.is_empty() && output.is_empty() {
                        String::new()
                    } else {
                        format!("{input}/{output}")
                    }
                });
                let ctx_str = m.context_limit
                    .map(|c| format!("{}K", c / 1000))
                    .unwrap_or_else(|| "?".to_string());
                ModelPickerItem {
                    command: "/model".to_string(),
                    model_id: m.id.clone(),
                    provider: m.provider.clone(),
                    label: format!("{}   {}   {}", m.id, m.provider, ctx_str),
                    context_limit: m.context_limit,
                    pricing: pricing_str,
                    authenticated: provider_authed,
                }
            })
            .collect();
        let _ = ui_tx.send(UiOutput::ModelPicker(items));
        return;
    }

    let mut model_config = config.clone();
    if let Err(e) = model_config.set_active_model(&model_id) {
        let text = format!("[Error] Unknown model '{model_id}': {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let provider_name = model_config.provider.clone();
    if !model_config.provider_authenticated(&provider_name) {
        let _ = ui_tx.send(UiOutput::CredentialRequest(
            talos_conversation::CredentialRequestData {
                provider: provider_name,
                model_id: model_id.clone(),
            },
        ));
        return;
    }

    let api_key = match model_config.api_key() {
        Ok(k) => k,
        Err(e) => {
            let text = format!("[Error] Failed to resolve API key for {provider_name}: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let model_context_limit = model_config.resolve_model_limits().0;

    let current_session = session_watch_rx.borrow().clone();
    let history = current_session.read_messages().unwrap_or_default();

    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };

    let provider = build_provider(&model_config, &api_key, mock);
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
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(model_config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }

    let (handle, actor) = AppServerSession::new(agent, session_config);
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition.lock().await.prepare(handle, session_for_prepare) {
        let text = format!("[Error] Failed to prepare model switch: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let mut transition_guard = transition.lock().await;
    match transition_guard.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(current_session.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((result.old_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!("[Error] Bridge forwarder unavailable; model switch events will not be persisted or displayed.");
            }
            let text = format!("[System] Switched to model {model_id}.\n");
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition_guard.rollback();
            let text = format!("[Error] Failed to commit model switch: {e}. Previous model remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_session_model_with_credential(
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
    cred: talos_conversation::CredentialResponseData,
    mock: bool,
) {
    let mut model_config = config.clone();
    model_config.set_provider_credential(&cred.provider, &cred.api_key);
    if let Err(e) = model_config.save() {
        let text = format!("[Error] Failed to persist credentials: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }
    if let Err(e) = model_config.set_active_model(&cred.model_id) {
        let text = format!("[Error] Unknown model '{}': {e}\n", cred.model_id);
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let api_key = cred.api_key.clone();
    let model_context_limit = model_config.resolve_model_limits().0;
    let current_session = session_watch_rx.borrow().clone();
    let history = current_session.read_messages().unwrap_or_default();

    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history: history,
        model_context_limit,
    };

    let provider = build_provider(&model_config, &api_key, mock);
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
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
    register_tui_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval_handler);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(model_config.tool_protocol());
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }

    let (handle, actor) = AppServerSession::new(agent, session_config);
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition.lock().await.prepare(handle, session_for_prepare) {
        let text = format!("[Error] Failed to prepare model switch: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    let mut transition_guard = transition.lock().await;
    match transition_guard.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(current_session.clone());
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((result.old_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!("[Error] Bridge forwarder unavailable; model switch events will not be persisted or displayed.");
            }
            let text = format!(
                "[System] Credentials saved. Switched to model {}.\n",
                cred.model_id
            );
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition_guard.rollback();
            let text = format!("[Error] Failed to commit model switch: {e}. Previous model remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

pub(crate) async fn run_tui_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    let needs_model_setup = config.model.is_empty() && !cli.mock;
    let needs_api_key = !cli.mock && !needs_model_setup && config.api_key().is_err();

    if (needs_model_setup || needs_api_key) && !cli.mock && cli.no_init {
        bail!("no model configured and --no-init was given. Set 'model' in ~/.talos/config.toml, pass --model, or remove --no-init to run the setup wizard.");
    }

    let mock_for_startup = cli.mock || needs_model_setup || needs_api_key;
    let api_key = if mock_for_startup {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let workspace_root = resolve_workspace_root(&cli)?;

    let (ui_output_tx, ui_output_rx) = mpsc::unbounded_channel::<UiOutput>();
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_output_tx.clone(),
        workspace_root.to_path_buf(),
    ));

    let hooks = build_hook_registry(true);
    let provider = build_provider(&config, &api_key, cli.mock);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
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
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    apply_runtime_skills(&mut agent, &runtime_skills);

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.to_path_buf())
            .load()
            .map_err(|e| anyhow!("{e}"))?;
        if !context.is_empty() {
            agent.set_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: context,
            }]);
        }
    }

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Latest,
        false,
    )?;

    let initial_history = session.read_messages().unwrap_or_default();
    let visible_history = initial_history.clone();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let sq_tx_signal = handle.sq_tx.clone();
    tokio::spawn(async move { actor.run().await });

    let transition = Arc::new(Mutex::new(SessionTransition::new(handle.sq_tx.clone(), session.clone())));
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            let _ = sq_tx_signal.try_send(SessionOp::Interrupt);
        }
    });

    // Shared state for persistence continuity across session switches.
    // watch channels let the bridge forwarder and user persister read the
    // CURRENTLY active session and sq_tx without owning a stale clone.
    let (session_watch_tx, session_watch_rx) =
        tokio::sync::watch::channel(session.clone());
    let (sq_tx_watch_tx, sq_tx_watch_rx) =
        tokio::sync::watch::channel(handle.sq_tx.clone());
    // Dedicated channel for handing off the new eq_rx + old_session to the
    // bridge forwarder after a session switch. The bridge must keep persisting
    // any in-flight events for the previous session until that actor's eq_rx
    // is exhausted (SESSION-002-D).
    let (bridge_rx_update_tx, mut bridge_rx_update_rx) = mpsc::unbounded_channel::<(
        talos_session::Session,
        tokio::sync::mpsc::UnboundedReceiver<SessionEvent>,
    )>();

    // Session lifecycle handler: processes /new, /resume, /fork requests.
    // After commit, updates the shared watch channels and bridge_rx_update
    // so persistence follows the new session.
    let (session_tx, mut session_rx) = mpsc::unbounded_channel::<SessionLifecycleRequest>();
    let transition_for_handler = transition.clone();
    let ui_tx_for_handler = ui_output_tx.clone();
    let config_for_handler = config.clone();
    let api_key_for_handler = api_key.clone();
    let hooks_for_handler = hooks.clone();
    let workspace_root_for_handler = workspace_root.to_path_buf();
    let session_manager_for_handler = session_manager.clone();
    let mcp_config_for_handler = config.mcp.clone();
    let session_watch_tx_for_handler = session_watch_tx.clone();
    let sq_tx_watch_tx_for_handler = sq_tx_watch_tx.clone();
    let bridge_rx_update_tx_for_handler = bridge_rx_update_tx.clone();
    let session_watch_rx_for_handler = session_watch_rx.clone();
    let model_context_limit = config.resolve_model_limits().0;
    let ui_tx_for_wizard = ui_tx_for_handler.clone();
    tokio::spawn(async move {
        while let Some(req) = session_rx.recv().await {
            match req {
                SessionLifecycleRequest::New(_) => {
                    handle_session_new(
                        &transition_for_handler,
                        &ui_tx_for_handler,
                        &config_for_handler,
                        &api_key_for_handler,
                        &hooks_for_handler,
                        &workspace_root_for_handler,
                        &session_manager_for_handler,
                        &mcp_config_for_handler,
                        &session_watch_tx_for_handler,
                        &sq_tx_watch_tx_for_handler,
                        &bridge_rx_update_tx_for_handler,
                        model_context_limit,
                        cli.mock,
                    )
                    .await;
                }
                SessionLifecycleRequest::Resume(req) => {
                    handle_session_resume(
                        &transition_for_handler,
                        &ui_tx_for_handler,
                        &config_for_handler,
                        &api_key_for_handler,
                        &hooks_for_handler,
                        &workspace_root_for_handler,
                        &session_manager_for_handler,
                        &mcp_config_for_handler,
                        &session_watch_tx_for_handler,
                        &sq_tx_watch_tx_for_handler,
                        &bridge_rx_update_tx_for_handler,
                        model_context_limit,
                        req.session_id,
                        cli.mock,
                    )
                    .await;
                }
                SessionLifecycleRequest::Fork(_) => {
                    handle_session_fork(
                        &transition_for_handler,
                        &ui_tx_for_handler,
                        &config_for_handler,
                        &api_key_for_handler,
                        &hooks_for_handler,
                        &workspace_root_for_handler,
                        &session_manager_for_handler,
                        &mcp_config_for_handler,
                        &session_watch_tx_for_handler,
                        &sq_tx_watch_tx_for_handler,
                        &bridge_rx_update_tx_for_handler,
                        model_context_limit,
                        &session_watch_rx_for_handler,
                        cli.mock,
                    )
                    .await;
                }
                SessionLifecycleRequest::Delete(req) => {
                    handle_session_delete(
                        &ui_tx_for_handler,
                        &workspace_root_for_handler,
                        &session_manager_for_handler,
                        &session_watch_rx_for_handler,
                        req.selection,
                    )
                    .await;
                }
                SessionLifecycleRequest::ModelSwitch(req) => {
                    handle_session_model(
                        &transition_for_handler,
                        &ui_tx_for_handler,
                        &config_for_handler,
                        &hooks_for_handler,
                        &workspace_root_for_handler,
                        &mcp_config_for_handler,
                        &session_watch_tx_for_handler,
                        &sq_tx_watch_tx_for_handler,
                        &bridge_rx_update_tx_for_handler,
                        &session_watch_rx_for_handler,
                        req.model_id,
                        cli.mock,
                    )
                    .await;
                }
                SessionLifecycleRequest::ModelSwitchWithCredential(resp) => {
                    handle_session_model_with_credential(
                        &transition_for_handler,
                        &ui_tx_for_handler,
                        &config_for_handler,
                        &hooks_for_handler,
                        &workspace_root_for_handler,
                        &mcp_config_for_handler,
                        &session_watch_tx_for_handler,
                        &sq_tx_watch_tx_for_handler,
                        &bridge_rx_update_tx_for_handler,
                        &session_watch_rx_for_handler,
                        resp,
                        cli.mock,
                    )
                    .await;
                }
            }
        }
    });

    let (bridge_tx, bridge_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let session_manager_for_persist = session_manager.clone();
    let mut bridge_forwarder = handle.eq_rx;
    // Cached snapshot of the session that owns the *current* eq_rx stream.
    // Until the inner `while let` exhausts, every event arriving on
    // `bridge_forwarder` belongs to this session, even if the watch channel
    // has already been updated to the next session. This is the SESSION-002-D
    // ordering invariant: in-flight events for the old actor must persist to
    // the old actor's session.
    let mut owning_session: talos_session::Session = session.clone();
    tokio::spawn(async move {
        loop {
            while let Some(session_event) = bridge_forwarder.recv().await {
                match session_event {
                    SessionEvent::AgentEvent(ref agent_event) => {
                        let _ = owning_session.append_event(agent_event);
                        let _ = bridge_tx.send(agent_event.clone());
                    }
                    SessionEvent::TurnCompleted {
                        status: talos_core::session::TurnCompletionStatus::Success { final_text: _, new_messages },
                        ..
                    } => {
                        for msg in &new_messages {
                            if matches!(msg, Message::User { .. }) {
                                continue;
                            }
                            if let Err(e) = owning_session.append(msg) {
                                eprintln!("Warning: failed to persist message: {e}");
                            }
                        }
                        if let Err(e) = session_manager_for_persist.update_index(&owning_session) {
                            eprintln!("Warning: failed to update session index: {e}");
                        }
                    }
                    SessionEvent::TurnCompleted {
                        status: talos_core::session::TurnCompletionStatus::Error { message },
                        ..
                    } => {
                        let _ = bridge_tx.send(AgentEvent::Error { message });
                    }
                    SessionEvent::Error { message } => {
                        let _ = bridge_tx.send(AgentEvent::Error { message });
                    }
                    _ => {}
                }
            }
            // Old actor's event stream exhausted — switch to the new session.
            match bridge_rx_update_rx.recv().await {
                Some((old_session, new_rx)) => {
                    owning_session = old_session;
                    bridge_forwarder = new_rx;
                }
                None => break,
            }
        }
    });

    let session_manager_for_user_persist = session_manager.clone();
    let (user_msg_tx, mut user_msg_rx) = mpsc::unbounded_channel::<String>();
    let session_watch_rx_for_user = session_watch_rx.clone();
    let sq_tx_watch_rx_for_user = sq_tx_watch_rx.clone();
    tokio::spawn(async move {
        while let Some(msg) = user_msg_rx.recv().await {
            let user_msg = Message::User { content: msg.clone() };
            let session = session_watch_rx_for_user.borrow().clone();
            if let Err(e) = session.append(&user_msg) {
                eprintln!("Warning: failed to persist user message: {e}");
            }
            if let Err(e) = session_manager_for_user_persist.update_index(&session) {
                eprintln!("Warning: failed to update session index: {e}");
            }
            let sq_tx = sq_tx_watch_rx_for_user.borrow().clone();
            let _ = sq_tx.send(SessionOp::Submit { message: msg }).await;
        }
    });

    let mut tui = Tui::new().context("failed to initialize TUI")?;
    tui.hydrate_history(&visible_history);

    let (user_input_tx, user_input_rx) = mpsc::unbounded_channel::<UserInput>();

    tui.set_ui_output_rx(ui_output_rx);
    tui.set_user_input_tx(user_input_tx.clone());
    tui.set_model_name(config.model.clone());

    let engine = ConversationEngine::new(config.model.clone())
        .with_skills(runtime_skills.diagnostics())
        .with_mcp_servers(mcp_runtime.diagnostics().to_vec());
    let session_tx_for_wizard = session_tx.clone();
    let sq_tx_watch_for_loop = sq_tx_watch_rx.clone();
    tokio::spawn(async move {
        run_conversation_loop(
            engine,
            bridge_rx,
            user_input_rx,
            ui_output_tx,
            user_msg_tx,
            sq_tx_watch_for_loop,
            session_tx,
        )
        .await;
    });

    if needs_model_setup || needs_api_key {
        let _ = session_tx_for_wizard.send(SessionLifecycleRequest::ModelSwitch(
            talos_conversation::ModelSwitchRequest {
                model_id: String::new(),
                provider_needs_credential: false,
            },
        ));
        if needs_api_key {
            send_stream(
                &ui_tx_for_wizard,
                MessageSource::System,
                format!(
                    "[System] Model '{}' is configured but the API key is missing. Select a model to configure credentials.\n",
                    config.model
                ),
            );
        }
    }

    tui.run().await?;
    Ok(())
}

pub(crate) async fn run_inline_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() && !cli.mock {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = if cli.mock {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let workspace_root = resolve_workspace_root(&cli)?;
    let hooks = build_hook_registry(true);
    let provider = build_provider(&config, &api_key, cli.mock);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let mut registry = build_print_tool_registry();
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), mcp_approval, true);

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    apply_runtime_skills(&mut agent, &runtime_skills);

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.to_path_buf())
            .load()
            .map_err(|e| anyhow!("{e}"))?;
        if !context.is_empty() {
            agent.set_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: context,
            }]);
        }
    }

    if let Some(ref prompt) = cli.system_prompt {
        agent.set_custom_prompt(prompt.clone());
    }
    if let Some(ref append) = cli.append_system_prompt {
        agent.set_append_prompt(append.clone());
    }

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Disabled,
        false,
    )?;

    let initial_history = session.read_messages().unwrap_or_default();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        print_mode: true,
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    let sq_tx = handle.sq_tx.clone();
    let mut eq_rx = handle.eq_rx;

    let stdin = io::stdin();

    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            let _ = sq_tx.try_send(SessionOp::Interrupt);
        }
    });

    println!("Talos inline mode. Type /quit to exit.");
    println!();

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => bail!("stdin error: {e}"),
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "/quit" || input == "/exit" {
            break;
        }

        let _ = handle
            .sq_tx
            .send(SessionOp::Submit {
                message: input.to_string(),
            })
            .await;

        let user_msg = talos_core::message::Message::User {
            content: input.to_string(),
        };
        if let Err(e) = session.append(&user_msg) {
            eprintln!("Warning: failed to persist user message: {e}");
        }

        let mut turn_done = false;
        while let Some(event) = eq_rx.recv().await {
            match event {
                SessionEvent::AgentEvent(agent_event) => match agent_event {
                    AgentEvent::TextDelta { delta } => {
                        print!("{delta}");
                        let _ = io::stdout().flush();
                    }
                    AgentEvent::TurnEnd { .. } => {
                        println!();
                        turn_done = true;
                        break;
                    }
                    AgentEvent::Error { message } => {
                        eprintln!("\nError: {message}");
                        turn_done = true;
                        break;
                    }
                    _ => {}
                },
                SessionEvent::TurnCompleted { status, .. } => {
                    match status {
                        talos_core::session::TurnCompletionStatus::Success { final_text: _, new_messages } => {
                            for msg in &new_messages {
                        if matches!(msg, talos_core::message::Message::User { .. }) {
                                    continue;
                                }
                                if let Err(e) = session.append(msg) {
                                    eprintln!("Warning: failed to persist message: {e}");
                                }
                            }
                            if let Err(e) = session_manager.update_index(&session) {
                                eprintln!("Warning: failed to update session index: {e}");
                            }
                        }
                        talos_core::session::TurnCompletionStatus::Cancelled => {
                            println!("\n(turn cancelled)");
                        }
                        talos_core::session::TurnCompletionStatus::Error { message } => {
                            eprintln!("\nError: {message}");
                        }
                    }
                    turn_done = true;
                    break;
                }
                SessionEvent::Error { message } => {
                    eprintln!("\nError: {message}");
                    turn_done = true;
                    break;
                }
                _ => {}
            }
        }

        if !turn_done {
            break;
        }
    }

    let _ = handle.sq_tx.send(SessionOp::Shutdown).await;
    Ok(())
}

pub(crate) async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let workspace_root = resolve_workspace_root(&cli)?;

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Prompt,
        true,
    )?;

    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() && !cli.mock {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml, pass --model, or run `talos` in TUI mode for the setup wizard.");
    }

    let api_key = if cli.mock {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::new(),
    )));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(BashTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ReadTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(WriteTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(EditTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GrepTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GlobTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(LsTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DeleteTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DiffTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(StatTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(GitStatusTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitDiffTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitLogTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitShowTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitBranchListTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(TreeTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitAddTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPushTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPullTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));

    let hooks = build_hook_registry(true);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval.clone(), false);

    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    apply_runtime_skills(&mut agent, &runtime_skills);

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.to_path_buf())
            .load()
            .map_err(|e| anyhow!("{e}"))?;
        if !context.is_empty() {
            agent.set_context_files(vec![ContextFile {
                path: "AGENTS.md".into(),
                content: context,
            }]);
        }
    }

    if let Some(ref system_prompt) = cli.system_prompt {
        agent.set_custom_prompt(system_prompt.clone());
    }

    if let Some(ref append_prompt) = cli.append_system_prompt {
        agent.set_append_prompt(append_prompt.clone());
    }

    let initial_history = session.read_messages().unwrap_or_default();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    let event_loop = event_loop::EventLoop::new(workspace_root, session, session_manager, handle);
    event_loop.run().await
}

fn apply_mcp_fixture_config(config: &mut Config, cli: &Cli) {
    #[cfg(debug_assertions)]
    if let Some(path) = cli.mcp_server_fixture.clone() {
        config.mcp.servers = vec![talos_config::McpServerConfig {
            name: "fixture".to_string(),
            transport: "stdio".to_string(),
            command: path.to_string_lossy().to_string(),
            args: Vec::new(),
            env: std::collections::HashMap::from([(
                "ECHO_PREFIX".to_string(),
                "fixture".to_string(),
            )]),
            cwd: std::env::current_dir().ok(),
        }];
    }

    #[cfg(not(debug_assertions))]
    let _ = (config, cli);
}
pub(crate) async fn run_mcp_server() -> Result<()> {
    let config_for_logging = Config::load().ok();
    init_logger(config_for_logging.as_ref().map(|config| &config.log), false);

    let tool_registry = Arc::new(build_mcp_tool_registry());
    let permission_engine = Arc::new(talos_permission::PermissionEngine::new());
    let hook_registry = build_hook_registry(false);
    let permission_gate = Arc::new(McpPermissionGate::new(permission_engine, hook_registry));
    let handler = TalosMcpHandler::new(tool_registry, permission_gate);

    let running = handler
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| anyhow!("failed to start mcp server: {e}"))?;
    let _ = running
        .waiting()
        .await
        .map_err(|e| anyhow!("mcp server join error: {e}"))?;
    Ok(())
}
// I009-S4 end

/// Handle `/new` — create a fresh session and transition to it.
#[allow(clippy::too_many_arguments)]
async fn handle_session_new(
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
        print_mode: false,
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
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
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
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }

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
            let _ = session_watch_tx.send(new_session_for_watch);
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((result.old_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!("[Error] Bridge forwarder unavailable; new session events will not be persisted or displayed.");
            }
            let text = "[System] New session started. Previous session preserved.\n".to_string();
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&new_session_for_watch.file_path);
            let text = format!("[Error] Failed to commit new session: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

/// Handle `/resume` — list candidates or resume a specific session.
#[allow(clippy::too_many_arguments)]
async fn handle_session_resume(
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
    session_id: Option<String>,
    mock: bool,
) {
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
                        return;
                    }
                };
                if sessions.is_empty() {
                    let text = "[System] No sessions found for this workspace.\n".to_string();
                    send_stream(ui_tx, MessageSource::System, text);
                    return;
                }
                let mut sessions = sessions;
                sessions.sort_by(|a, b| {
                    b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id))
                });
                if n == 0 || n > sessions.len() {
                    let text = format!("[Error] Invalid session number {n}. Valid range: 1-{}.\n", sessions.len());
                    send_stream(ui_tx, MessageSource::Error, text);
                    return;
                }
                let selected = &sessions[n - 1];
                let selected_id = selected.id.to_string();
                match session_manager.resume_session(&selected_id) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return;
                    }
                }
            } else {
                // Fall back to treating it as a UUID (backward compat).
                match session_manager.resume_session(id) {
                    Ok(s) => s,
                    Err(e) => {
                        let text = format!("[Error] Session '{id}' not found or invalid: {e}\n");
                        send_stream(ui_tx, MessageSource::Error, text);
                        return;
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
                    return;
                }
            };

            if sessions.is_empty() {
                let text = "[System] No sessions found for this workspace.\n".to_string();
                send_stream(ui_tx, MessageSource::System, text);
                return;
            }

            let mut sessions = sessions;
            sessions.sort_by(|a, b| {
                b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id))
            });

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
            return;
        }
    };

    let resume_history = match target_session.read_messages() {
        Ok(h) => h,
        Err(e) => {
            let text = format!("[Error] Failed to read session history: {e}\n");
            send_stream(ui_tx, MessageSource::Error, text);
            return;
        }
    };

    let resume_history_for_hydrate = resume_history.clone();
    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.to_path_buf(),
        initial_history: resume_history,
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
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
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
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }

    let (handle, actor) = AppServerSession::new(agent, session_config);

    // Clone for watch channel update after commit (target_session is moved into prepare).
    let target_session_for_watch = target_session.clone();
    if let Err(e) = transition.prepare(handle, target_session) {
        let _ = std::fs::remove_file(&target_session_for_watch.file_path);
        let text = format!("[Error] Failed to prepare resume: {e}\n");
        send_stream(ui_tx, MessageSource::Error, text);
        return;
    }

    match transition.commit(actor) {
        Ok(result) => {
            let _ = session_watch_tx.send(target_session_for_watch);
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((result.old_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!("[Error] Bridge forwarder unavailable; resumed session events will not be persisted or displayed.");
            }
            let _ = ui_tx.send(UiOutput::HydrateHistory(resume_history_for_hydrate));
            let text = format!("[System] Resumed session {}.\n", session_id.unwrap_or_default());
            send_stream(ui_tx, MessageSource::System, text);
        }
        Err(e) => {
            transition.rollback();
            let _ = std::fs::remove_file(&target_session_for_watch.file_path);
            let text = format!("[Error] Failed to commit resume: {e}. Old session remains active.\n");
            send_stream(ui_tx, MessageSource::Error, text);
        }
    }
}

/// Handle `/fork` — clone the active session's durable history into a child session.
///
/// Copies the source JSONL file to a new path with a fresh UUID, creates a new
/// [`talos_session::Session`], and swaps the agent context. The source session
/// remains byte-for-byte unchanged.
#[allow(clippy::too_many_arguments)]
async fn handle_session_fork(
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
        && let Err(e) = std::fs::create_dir_all(parent) {
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
        print_mode: false,
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
    let mut registry = build_tui_tool_registry(approval_handler.clone(), workspace_root.to_path_buf());
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
    if let Ok(skills) = discover_runtime_skills(workspace_root) {
        apply_runtime_skills(&mut agent, &skills);
    }

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
            let _ = session_watch_tx.send(child_session_for_watch);
            let _ = sq_tx_watch_tx.send(result.new_handle.sq_tx.clone());
            if bridge_rx_update_tx
                .send((result.old_session.clone(), result.new_handle.eq_rx))
                .is_err()
            {
                eprintln!("[Error] Bridge forwarder unavailable; forked session events will not be persisted or displayed.");
            }
            let text = format!("[System] Forked session {child_id} (source: {}).\n", result.old_session.id);
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
