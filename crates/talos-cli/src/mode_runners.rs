//! Runtime mode runner implementations for the Talos CLI.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rmcp::ServiceExt;
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{ConversationEngine, MessageSource, ModelInfo, UiOutput, UserInput};
use talos_core::message::{AgentEvent, Message};
use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};
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
use crate::mode_runtime::{
    apply_mcp_fixture_config, maybe_set_memory_provider, request_preview_payload,
    session_metadata_for_model, set_todo_prompt_provider,
};
pub(crate) use crate::mode_runtime::{apply_session_model_to_config, context_files_for_agent};
use crate::model_lifecycle::{
    RebuildSessionParams, build_model_picker_data, provider_setup_target_model,
    rebuild_session_for_model,
};
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{
    PermissionAwareTool, TuiApprovalHandler, build_mcp_tool_registry, build_print_tool_registry,
    build_tui_tool_registry, register_permission_aware_tools, register_tui_permission_aware_tools,
};
use crate::runtime_adapter;
use crate::session_setup::{
    ResumeSelection, canonical_workspace_root, resolve_session_for_workspace,
    resolve_workspace_root, workspace_display_name, workspace_path_display,
};
use crate::session_transition::SessionTransition;
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};
use crate::todo_view;
use crate::tui_bridge::{ConversationLoopIo, SessionLifecycleRequest, run_conversation_loop};
use crate::{Cli, build_hook_registry, event_loop};
use tokio::sync::Mutex;

pub(crate) use crate::mode_inline::run_inline_mode;
pub(crate) use crate::mode_print::run_print_mode;

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
    let runtime_skills = discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
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
    maybe_set_memory_provider(&mut agent, &config);

    let server = talos_rpc::RpcServer::new(Arc::new(runtime_adapter::AgentRuntime(agent)));
    server.run_stdio().await
    // I009-S5 end
}

pub(crate) fn resolve_model_info(config: &Config) -> ModelInfo {
    let builtins = talos_config::model::builtin_models();
    let meta =
        talos_config::model::find_model_by_provider(&builtins, &config.provider, &config.model);

    let (context_limit, _) = config.resolve_model_limits();

    let pricing = meta.and_then(|m| m.pricing.as_ref());
    let input_price = pricing.and_then(|p| p.input_per_1m);
    let output_price = pricing.and_then(|p| p.output_per_1m);

    ModelInfo {
        model_name: config.model.clone(),
        provider: config.provider.clone(),
        context_limit: Some(context_limit),
        input_price_per_million: input_price,
        output_price_per_million: output_price,
    }
}

pub(crate) async fn run_tui_mode(cli: Cli) -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("\n\n--- TALOS PANIC ---");
        eprintln!(
            "Location: {}",
            info.location().map(|l| l.to_string()).unwrap_or_default()
        );
        eprintln!("Message: {}\n", info);
        original_hook(info);
    }));

    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

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
        ResumeSelection::Latest,
        false,
    )?;
    apply_session_model_to_config(&mut config, &session);

    let needs_model_setup = config.model.is_empty() && !cli.mock;
    let needs_api_key = !cli.mock && !needs_model_setup && config.api_key().is_err();

    if (needs_model_setup || needs_api_key) && !cli.mock && cli.no_init {
        bail!(
            "no model configured and --no-init was given. Set 'model' in ~/.talos/config.toml, pass --model, or remove --no-init to run the setup wizard."
        );
    }

    let mock_for_startup = cli.mock || needs_model_setup || needs_api_key;
    let api_key = if mock_for_startup {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

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
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        session.id,
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
    let runtime_skills = discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
    apply_runtime_skills(&mut agent, &runtime_skills);
    let runtime_skills = Arc::new(Mutex::new(runtime_skills));
    maybe_set_memory_provider(&mut agent, &config);
    set_todo_prompt_provider(&mut agent, &session_manager, &session);

    agent.set_context_files(context_files_for_agent(
        &config,
        &workspace_root,
        !cli.no_context,
    )?);

    let initial_history = session.read_messages().unwrap_or_default();
    let visible_history = initial_history.clone();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let sq_tx_signal = handle.sq_tx.clone();
    tokio::spawn(async move { actor.run().await });

    let transition = Arc::new(Mutex::new(SessionTransition::new(
        handle.sq_tx.clone(),
        session.clone(),
    )));
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            let _ = sq_tx_signal.try_send(SessionOp::Interrupt);
        }
    });

    // Shared state for persistence continuity across session switches.
    // watch channels let the bridge forwarder and user persister read the
    // CURRENTLY active session and sq_tx without owning a stale clone.
    let (session_watch_tx, session_watch_rx) = tokio::sync::watch::channel(session.clone());
    let (sq_tx_watch_tx, sq_tx_watch_rx) = tokio::sync::watch::channel(handle.sq_tx.clone());
    let (model_info_tx, model_info_rx) = tokio::sync::watch::channel(resolve_model_info(&config));
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
    let model_info_tx_for_handler = model_info_tx.clone();
    let model_context_limit = config.resolve_model_limits().0;
    let ui_tx_for_wizard = ui_tx_for_handler.clone();
    tokio::spawn(async move {
        let mut config_for_handler = config_for_handler;
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
                    if let Some(new_config) = handle_session_resume(
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
                    .await
                    {
                        let _ = model_info_tx_for_handler.send(resolve_model_info(&new_config));
                        config_for_handler = new_config;
                    }
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
                SessionLifecycleRequest::Todo(req) => {
                    todo_view::handle_todo_command(
                        &ui_tx_for_handler,
                        &session_manager_for_handler,
                        &session_watch_rx_for_handler,
                        req,
                    );
                }
                SessionLifecycleRequest::ModelSwitch(req) => {
                    if let Some(new_config) = handle_session_model(
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
                        &session_manager_for_handler,
                        req.model_id,
                        cli.mock,
                    )
                    .await
                    {
                        let _ = model_info_tx_for_handler.send(resolve_model_info(&new_config));
                        config_for_handler = new_config;
                    }
                }
                SessionLifecycleRequest::ModelSwitchWithCredential(resp) => {
                    if let Some(new_config) = handle_session_model_with_credential(
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
                        &session_manager_for_handler,
                        resp,
                        cli.mock,
                    )
                    .await
                    {
                        let _ = model_info_tx_for_handler.send(resolve_model_info(&new_config));
                        config_for_handler = new_config;
                    }
                }
                SessionLifecycleRequest::ProviderSetup(provider) => {
                    handle_provider_setup(&ui_tx_for_handler, &config_for_handler, &provider).await;
                }
                SessionLifecycleRequest::ConnectRequest { provider } => {
                    handle_connect(&ui_tx_for_handler, &config_for_handler, &provider).await;
                }
                SessionLifecycleRequest::ConnectWithCredential(resp) => {
                    if let Some(new_config) = handle_connect_with_credential(
                        &ui_tx_for_handler,
                        &config_for_handler,
                        resp,
                    )
                    .await
                    {
                        config_for_handler = new_config;
                    }
                }
            }
        }
    });

    let (bridge_tx, bridge_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let session_manager_for_persist = session_manager.clone();
    let mut bridge_forwarder = handle.eq_rx;
    let model_info_rx_for_bridge = model_info_rx.clone();
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
                    SessionEvent::AgentEvent { event } => {
                        if !matches!(event, AgentEvent::ThinkingDelta { .. }) {
                            let _ = owning_session.append_event(&event);
                        }
                        let _ = bridge_tx.send(event);
                    }
                    SessionEvent::TurnCompleted {
                        status:
                            talos_core::session::TurnCompletionStatus::Success {
                                final_text: _,
                                new_messages,
                            },
                        ..
                    } => {
                        for msg in &new_messages {
                            if matches!(msg, Message::User { .. }) {
                                continue;
                            }
                            let info = model_info_rx_for_bridge.borrow().clone();
                            let metadata =
                                session_metadata_for_model(&info.model_name, &info.provider);
                            if let Err(e) = owning_session.append_with_metadata(msg, metadata) {
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
            // Old actor's event stream exhausted — switch persistence to the new session.
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
    let model_info_rx_for_user = model_info_rx.clone();
    tokio::spawn(async move {
        while let Some(msg) = user_msg_rx.recv().await {
            let user_msg = Message::User {
                content: msg.clone(),
            };
            let session = session_watch_rx_for_user.borrow().clone();
            let info = model_info_rx_for_user.borrow().clone();
            let metadata = session_metadata_for_model(&info.model_name, &info.provider);
            if let Err(e) = session.append_with_metadata(&user_msg, metadata) {
                eprintln!("Warning: failed to persist user message: {e}");
            }
            if let Err(e) = session_manager_for_user_persist.update_index(&session) {
                eprintln!("Warning: failed to update session index: {e}");
            }
            let sq_tx = sq_tx_watch_rx_for_user.borrow().clone();
            let _ = sq_tx
                .send(match request_preview_payload(&msg) {
                    Some(message) => SessionOp::PreviewRequest { message },
                    None => SessionOp::Submit { message: msg },
                })
                .await;
        }
    });

    let mut tui = Tui::new().context("failed to initialize TUI")?;
    tui.hydrate_history(&visible_history);
    if !visible_history.is_empty() {
        send_stream(
            &ui_output_tx,
            MessageSource::System,
            format!("[System] Continued session {}.\n", session.id),
        );
    }

    let (user_input_tx, user_input_rx) = mpsc::unbounded_channel::<UserInput>();

    tui.set_ui_output_rx(ui_output_rx);
    tui.set_user_input_tx(user_input_tx.clone());
    tui.set_model_name(config.model.clone());
    tui.set_provider(config.provider.clone());
    tui.set_workspace_path(workspace_path_display(&workspace_root));
    tui.set_session_id(session.id.to_string());

    let skill_diagnostics = runtime_skills.lock().await.diagnostics();
    let engine = ConversationEngine::new(config.model.clone(), config.provider.clone())
        .with_skills(skill_diagnostics)
        .with_mcp_servers(mcp_runtime.diagnostics().to_vec())
        .with_workspace_root(workspace_root.clone());
    let session_tx_for_wizard = session_tx.clone();
    let sq_tx_watch_for_loop = sq_tx_watch_rx.clone();
    let ui_output_tx_for_dashboard = ui_output_tx.clone();
    tokio::spawn(async move {
        run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx: bridge_rx,
                user_rx: user_input_rx,
                ui_tx: ui_output_tx,
                submit_tx: user_msg_tx,
                sq_tx_watch: sq_tx_watch_for_loop,
                model_info_watch: model_info_rx,
                session_tx,
                runtime_skills,
            },
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

    if config.dashboard.enabled {
        let snapshot = crate::dashboard_helpers::build_dashboard_snapshot(&config, &session_manager, &workspace_root_str);
        let server = talos_dashboard::DashboardServer::with_loopback_only(
            snapshot,
            config.dashboard.loopback_only,
        );
        let token = server.token().to_string();
        match server.serve().await {
            Ok((addr, _)) => {
                let url = format!("http://{addr}/");
                if config.dashboard.loopback_only {
                    eprintln!("Dashboard: {url} (loopback-only, no token)");
                    send_stream(
                        &ui_output_tx_for_dashboard,
                        MessageSource::System,
                        format!(
                            "[System] Dashboard available at {url} (loopback-only, no token).\n"
                        ),
                    );
                } else {
                    eprintln!("Dashboard: {url} (token: {token})");
                    send_stream(
                        &ui_output_tx_for_dashboard,
                        MessageSource::System,
                        format!(
                            "[System] Dashboard available at {url} with bearer token {token}.\n"
                        ),
                    );
                }
            }
            Err(e) => {
                eprintln!("Dashboard: failed to start: {e}");
                send_stream(
                    &ui_output_tx_for_dashboard,
                    MessageSource::Error,
                    format!("[Error] Dashboard failed to start: {e}\n"),
                );
            }
        }
    }

    tui.run().await?;
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
        bail!(
            "no model configured. Set 'model' in ~/.talos/config.toml, pass --model, or run `talos` in TUI mode for the setup wizard."
        );
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
    registry.register(Arc::new(GitBranchListTool::new(
        workspace_root.to_path_buf(),
    )));
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
    let runtime_skills = discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
    apply_runtime_skills(&mut agent, &runtime_skills);
    maybe_set_memory_provider(&mut agent, &config);
    set_todo_prompt_provider(&mut agent, &session_manager, &session);

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
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    let event_loop = event_loop::EventLoop::new(workspace_root, session, session_manager, handle);
    event_loop.run().await
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

