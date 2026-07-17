//! Runtime mode runner implementations for the Talos CLI.

#[path = "session_handlers.rs"]
mod session_handlers;
pub(crate) use session_handlers::*;
#[path = "mode_interactive.rs"]
mod mode_interactive;
pub(crate) use mode_interactive::run_interactive_mode;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rmcp::ServiceExt;
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{
    ContentOutput, ConversationEngine, MessageSource, ModelInfo, SessionPickerItem, TipKind,
    UiOutput, UserInput,
};
use talos_core::message::Message;
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnEventPayload,
};
use talos_core::tool::{ToolPresentationPolicy, ToolRegistry};
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_plugin::HookRegistry;
use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::{BashTool, DiffTool, GlobTool, GrepTool, LsTool, StatTool, TreeTool};
use talos_tui::Tui;
use tokio::sync::{mpsc, watch};

use crate::approval::ApprovalPrompt;
use crate::logging::init_logger;
use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    apply_mcp_fixture_config, maybe_set_memory_provider, session_metadata_for_model,
    set_todo_prompt_provider,
};
pub(crate) use crate::mode_runtime::{apply_session_model_to_config, context_files_for_agent};
use crate::model_lifecycle::{
    RebuildSessionParams, build_model_picker_data, provider_setup_target_model,
    rebuild_session_for_model,
};
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{
    PermissionAwareTool, TuiApprovalHandler, build_mcp_tool_registry, build_print_tool_registry,
    build_tui_tool_registry, register_explicit_permission_aware_plugins,
    register_explicit_tui_plugins, register_permission_aware_tools,
    register_tui_permission_aware_tools,
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
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_print_tool_registry(sched_tools);
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(
        &mut registry,
        mcp_runtime.tools(),
        mcp_approval.clone(),
        true,
    );
    let loaded_plugin_packages = register_explicit_permission_aware_plugins(
        &mut registry,
        &cli.plugin_packages,
        mcp_approval,
        true,
    )
    .map_err(anyhow::Error::msg)?;
    let runtime_skills = discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.clone(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    if !loaded_plugin_packages.is_empty() {
        let mut policy = ToolPresentationPolicy::runtime_default();
        for capability in loaded_plugin_packages
            .iter()
            .flat_map(|package| package.capabilities.iter())
        {
            policy = policy.disclose_tool(capability.clone());
        }
        agent.set_tool_presentation_policy(policy);
    }
    apply_runtime_skills(&mut agent, &runtime_skills);
    maybe_set_memory_provider(&mut agent, &config);

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::headless_deny(),
        workspace_root,
        initial_history: vec![],
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    tokio::spawn(async move { actor.run().await });
    let server = talos_rpc::RpcServer::new(Arc::new(runtime_adapter::AgentRuntime::new(handle)));
    server.run_stdio().await
    // I009-S5 end
}

fn send_stream(ui_tx: &mpsc::UnboundedSender<UiOutput>, source: MessageSource, text: String) {
    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block { source, text }));
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
    let talos_root = crate::storage::resolve_talos_root();

    if cli.trust {
        let trust_store = talos_permission::WorkspaceTrustStore::new(&talos_root);
        if talos_permission::is_git_workspace(&workspace_root) {
            trust_store
                .grant_trust(&workspace_root)
                .context("failed to write trust file")?;
            eprintln!("Workspace trusted: {}", workspace_root.display());
        } else {
            eprintln!("--trust requires a Git workspace");
        }
    }

    let approval_handler = Arc::new(TuiApprovalHandler::new_with_trust(
        ui_output_tx.clone(),
        workspace_root.to_path_buf(),
        &talos_root,
    ));

    let hooks = build_hook_registry(true);
    let provider = build_provider(&config, &api_key, cli.mock);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_tui_tool_registry(
        approval_handler.clone(),
        workspace_root.to_path_buf(),
        session.id,
        sched_tools,
    );
    register_tui_permission_aware_tools(
        &mut registry,
        mcp_runtime.tools(),
        approval_handler.clone(),
    );
    let loaded_plugin_packages =
        register_explicit_tui_plugins(&mut registry, &cli.plugin_packages, approval_handler)
            .map_err(anyhow::Error::msg)?;

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks.clone(),
    );
    agent.set_tool_protocol(config.tool_protocol());
    if !loaded_plugin_packages.is_empty() {
        let mut policy = ToolPresentationPolicy::runtime_default();
        for capability in loaded_plugin_packages
            .iter()
            .flat_map(|package| package.capabilities.iter())
        {
            policy = policy.disclose_tool(capability.clone());
        }
        agent.set_tool_presentation_policy(policy);
    }
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
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        session.clone(),
        session_metadata_for_model(&config.model, &config.provider),
    );
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

    let (bridge_tx, bridge_rx) = mpsc::unbounded_channel::<SessionEvent>();
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
                if let SessionEvent::TurnEvent {
                    payload:
                        TurnEventPayload::Completed {
                            status:
                                talos_core::session::TurnCompletionStatus::Success {
                                    final_text: _,
                                    new_messages: _,
                                },
                        },
                    ..
                } = &session_event
                {
                    if let Err(e) = session_manager_for_persist.update_index(&owning_session) {
                        eprintln!("Warning: failed to update session index: {e}");
                    }
                    // Keep on-disk session logs bounded after persistence. The session owns
                    // the write lock, so archival cannot race an append from this bridge.
                    if owning_session
                        .read_entries()
                        .map(|entries| entries.len() > 200)
                        .unwrap_or(false)
                        && let Err(e) = owning_session.compact_archived(50)
                    {
                        eprintln!("Warning: failed to archive session: {e}");
                    }
                }
                let _ = bridge_tx.send(session_event);
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
    let hook_decls: Vec<(String, String, bool)> = config
        .hooks
        .declarations
        .iter()
        .map(|d| (d.name.clone(), d.event.clone(), d.enabled))
        .collect();
    let loaded_plugin_diagnostics = loaded_plugin_packages
        .iter()
        .map(|package| talos_conversation::LoadedPluginDiagnostic {
            name: package.name.clone(),
            version: package.version.clone(),
            carrier: package.carrier.clone(),
            capabilities: package.capabilities.clone(),
        })
        .collect::<Vec<_>>();
    let engine = ConversationEngine::new(config.model.clone(), config.provider.clone())
        .with_skills(skill_diagnostics)
        .with_mcp_servers(mcp_runtime.diagnostics().to_vec())
        .with_hook_declarations(hook_decls.clone())
        .with_loaded_plugins(loaded_plugin_diagnostics.clone())
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
        let ext_snapshot = talos_conversation::build_extension_snapshot_with_plugins(
            mcp_runtime.diagnostics(),
            &hook_decls,
            &[],
            &loaded_plugin_diagnostics,
        );
        let extensions = serde_json::to_value(&ext_snapshot).unwrap_or(serde_json::Value::Null);
        let snapshot = crate::dashboard_helpers::build_dashboard_snapshot(
            &config,
            &session_manager,
            &workspace_root_str,
            extensions,
        );
        let server = talos_dashboard::DashboardServer::with_loopback_only(
            snapshot,
            config.dashboard.loopback_only,
        );
        let token = server.token().to_string();
        match server.serve().await {
            Ok((addr, _)) => {
                let url = format!("http://{addr}/");
                if config.dashboard.loopback_only {
                    tracing::info!(dashboard_url = %url, "dashboard started (loopback-only)");
                    let _ = ui_output_tx_for_dashboard.send(dashboard_available_tip(&url, true));
                } else {
                    tracing::info!(dashboard_url = %url, dashboard_token = %token, "dashboard started");
                    let _ = ui_output_tx_for_dashboard.send(dashboard_available_tip(&url, false));
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "dashboard failed to start");
                let _ = ui_output_tx_for_dashboard.send(dashboard_failure_tip(&e.to_string()));
            }
        }
    }

    tui.run().await?;
    Ok(())
}

pub(crate) fn dashboard_available_tip(url: &str, loopback_only: bool) -> UiOutput {
    let detail = if loopback_only {
        "loopback-only"
    } else {
        "token required, see terminal output"
    };
    UiOutput::Tip {
        text: format!("Dashboard ready: {url} ({detail})"),
        kind: TipKind::Info,
    }
}

pub(crate) fn dashboard_failure_tip(message: &str) -> UiOutput {
    UiOutput::Tip {
        text: format!("Dashboard failed to start: {message}"),
        kind: TipKind::Error,
    }
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
#[cfg(test)]
#[path = "mode_runners_tests.rs"]
mod tests;
