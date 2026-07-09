//! Runtime mode runner implementations for the Talos CLI.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use rmcp::ServiceExt;
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{
    ConversationEngine, MessageSource, ModelInfo, SessionPickerItem, StreamMessage, TipKind,
    UiOutput, UserInput,
};
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

fn send_stream(ui_tx: &mpsc::UnboundedSender<UiOutput>, source: MessageSource, text: String) {
    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(futures::stream::once(async move { text })),
    }));
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
async fn handle_provider_setup(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    provider: &str,
) {
    if config.provider_authenticated(provider) {
        let data = build_model_picker_data(config);
        let _ = ui_tx.send(UiOutput::ModelPicker(data));
        return;
    }

    let _ = ui_tx.send(UiOutput::CredentialRequest(
        talos_conversation::CredentialRequestData {
            provider: provider.to_string(),
            model_id: None,
            connect_mode: false,
            default_base_url: None,
        },
    ));
}

async fn handle_connect(ui_tx: &mpsc::UnboundedSender<UiOutput>, config: &Config, provider: &str) {
    if provider.is_empty() {
        let data = build_connect_picker_data(config);
        let _ = ui_tx.send(UiOutput::ConnectPicker(data));
        return;
    }

    if config.provider_authenticated(provider) {
        send_stream(
            ui_tx,
            MessageSource::System,
            format!("[System] Provider '{provider}' is already connected.\n"),
        );
        return;
    }

    // Precedence: existing user config base_url > models.toml provider default >
    // builtin hardcoded config > None.
    let default_base_url = config
        .providers
        .get(provider)
        .and_then(|p| p.base_url.clone())
        .or_else(|| {
            talos_config::model::builtin_providers()
                .iter()
                .find(|p| p.id == provider)
                .and_then(|p| {
                    let base_url = p.api_base_url.as_deref()?;
                    Some(match p.protocol {
                        Some(talos_config::ProviderProtocol::AnthropicMessages) => {
                            let mut url = base_url.trim().trim_end_matches('/').to_string();
                            if !url.to_ascii_lowercase().ends_with("/messages") {
                                url.push_str("/messages");
                            }
                            url
                        }
                        _ => talos_config::normalize_provider_endpoint(base_url).base_url,
                    })
                })
        })
        .or_else(|| talos_config::builtin_provider_config(provider).and_then(|p| p.base_url));

    let _ = ui_tx.send(UiOutput::CredentialRequest(
        talos_conversation::CredentialRequestData {
            provider: provider.to_string(),
            model_id: None,
            connect_mode: true,
            default_base_url,
        },
    ));
}

async fn handle_connect_with_credential(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    config: &Config,
    cred: talos_conversation::CredentialResponseData,
) -> Option<Config> {
    let mut new_config = config.clone();
    new_config.set_provider_credential(&cred.provider, &cred.api_key);

    let provider_entry = new_config
        .providers
        .entry(cred.provider.clone())
        .or_default();
    if provider_entry.api_key_env.is_none() {
        provider_entry.api_key_env = match cred.provider.as_str() {
            "anthropic" => Some("ANTHROPIC_API_KEY".to_string()),
            "openai" => Some("OPENAI_API_KEY".to_string()),
            _ => Some(format!("{}_API_KEY", cred.provider.to_uppercase())),
        };
    }
    // `cred.base_url` is already resolved by the TUI credential panel to
    // either the user-typed value or the request's `default_base_url`.
    // `None` here means neither was available, so the existing (or absent)
    // `base_url` is left untouched — never overwritten with an empty value.
    if let Some(base_url) = cred.base_url.as_ref() {
        let endpoint = talos_config::normalize_provider_endpoint(base_url);
        provider_entry.protocol = endpoint.protocol;
        provider_entry.base_url = Some(endpoint.base_url);
    }

    if let Err(e) = new_config.save() {
        send_stream(
            ui_tx,
            MessageSource::Error,
            format!("[Error] Failed to save provider config: {e}\n"),
        );
        return None;
    }

    send_stream(
        ui_tx,
        MessageSource::System,
        format!(
            "[System] Provider '{}' connected. Use /model to browse its models.\n",
            cred.provider
        ),
    );
    Some(new_config)
}

/// Builds [`talos_conversation::ConnectPickerData`] for the `/connect` picker.
///
/// Uses the compiled-in `models.toml` data (`[[providers]]` for display name,
/// API base URL, docs URL; `[[models]]` for model counts per provider).
fn build_connect_picker_data(config: &Config) -> talos_conversation::ConnectPickerData {
    use std::collections::BTreeMap;
    use talos_conversation::{ConnectPickerData, ConnectPickerItem};

    let all = talos_config::model::builtin_models();
    let mut model_counts: BTreeMap<String, usize> = BTreeMap::new();
    for m in &all {
        *model_counts.entry(m.provider.clone()).or_default() += 1;
    }

    let providers: BTreeMap<String, talos_config::model::BuiltinProvider> =
        talos_config::model::builtin_providers()
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

    let mut connected = Vec::new();
    let mut available = Vec::new();

    for (provider_id, count) in model_counts {
        let has_credential = config.provider_authenticated(&provider_id);
        let (name, api_base_url, doc_url) = providers
            .get(&provider_id)
            .map(|p| (p.name.clone(), p.api_base_url.clone(), p.doc_url.clone()))
            .unwrap_or_else(|| (provider_id.clone(), None, None));
        let item = ConnectPickerItem {
            provider: provider_id.clone(),
            name,
            model_count: count,
            api_base_url,
            has_credential,
            doc_url,
        };
        if has_credential {
            connected.push(item);
        } else {
            available.push(item);
        }
    }

    ConnectPickerData {
        connected,
        available,
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
                    let _ = ui_output_tx_for_dashboard.send(UiOutput::Tip {
                        text: format!("Dashboard: {url} (loopback-only)"),
                        kind: TipKind::Info,
                    });
                } else {
                    eprintln!("Dashboard: {url} (token: {token})");
                    let _ = ui_output_tx_for_dashboard.send(UiOutput::Tip {
                        text: format!("Dashboard: {url} (token: {token})"),
                        kind: TipKind::Info,
                    });
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


#[cfg(test)]
mod connect_tests {
    use super::*;
    use crate::test_support::HOME_ENV_MUTEX;
    use talos_config::ProviderConfig;

    /// Runs `f` with `HOME` redirected to a fresh temp directory, restoring
    /// the original `HOME` afterward. Must be called under `HOME_ENV_MUTEX`
    /// (shared crate-wide — see `crate::test_support` for why a private
    /// per-module mutex is not sufficient).
    async fn with_isolated_home<F, Fut, T>(f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let dir = tempfile::tempdir().expect("tempdir");
        let original = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", dir.path()) };
        let result = f().await;
        match original {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        result
    }

    #[test]
    fn build_connect_picker_data_none_falls_back_without_blocking() {
        let config = Config::default();
        let data = build_connect_picker_data(&config);
        assert!(data.connected.len() + data.available.len() > 0);
    }

    #[tokio::test]
    async fn handle_connect_with_credential_writes_new_provider_api_key_and_base_url() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let config = Config::default();
            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-secret".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://api.groq.com/openai/v1".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("new provider connect must succeed");

        let groq = new_config
            .providers
            .get("groq")
            .expect("groq entry created");
        assert_eq!(groq.api_key.as_deref(), Some("gsk-secret"));
        assert_eq!(groq.api_key_env.as_deref(), Some("GROQ_API_KEY"));
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_preserves_unrelated_provider_fields() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let mut config = Config::default();
            config.providers.insert(
                "groq".to_string(),
                ProviderConfig {
                    base_url: Some("https://old.groq.example/v1".to_string()),
                    models: std::collections::HashMap::from([(
                        "llama-3".to_string(),
                        talos_config::ModelConfig {
                            context_limit: Some(999_000),
                            output_limit: None,
                            reasoning: None,
                        },
                    )]),
                    ..Default::default()
                },
            );
            // An unrelated provider that must remain completely untouched.
            config.providers.insert(
                "anthropic".to_string(),
                ProviderConfig {
                    api_key: Some("sk-ant-untouched".to_string()),
                    ..Default::default()
                },
            );

            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-updated".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: None, // user left it blank; resolved default was also None
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("existing provider reconnect must succeed");

        let groq = new_config.providers.get("groq").expect("groq entry exists");
        assert_eq!(groq.api_key.as_deref(), Some("gsk-updated"));
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://old.groq.example/v1"),
            "existing base_url must be preserved when cred.base_url is None"
        );
        assert_eq!(
            groq.models.get("llama-3").and_then(|m| m.context_limit),
            Some(999_000),
            "existing model overrides must be preserved"
        );

        let anthropic = new_config
            .providers
            .get("anthropic")
            .expect("anthropic entry exists");
        assert_eq!(
            anthropic.api_key.as_deref(),
            Some("sk-ant-untouched"),
            "unrelated provider must not be touched by a groq connect"
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_updates_base_url_when_provided() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let mut config = Config::default();
            config.providers.insert(
                "groq".to_string(),
                ProviderConfig {
                    base_url: Some("https://old.groq.example/v1".to_string()),
                    ..Default::default()
                },
            );

            let cred = talos_conversation::CredentialResponseData {
                provider: "groq".to_string(),
                api_key: "gsk-updated".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://new.groq.example/v1".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("update must succeed");

        let groq = new_config.providers.get("groq").unwrap();
        assert_eq!(
            groq.base_url.as_deref(),
            Some("https://new.groq.example/v1"),
            "explicit user input must update the base_url"
        );
    }

    #[tokio::test]
    async fn handle_connect_default_base_url_falls_back_to_builtin_provider_config() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();

        // groq has an api_base_url in the models.toml [[providers]] section
        // (from BUILD_MODELS=1) AND a hardcoded base_url in builtin_provider_config.
        // Both return "https://api.groq.com/openai/v1".
        handle_connect(&tx, &config, "groq").await;
        drop(tx);

        let mut default_base_url = None;
        while let Some(output) = rx.recv().await {
            if let UiOutput::CredentialRequest(req) = output {
                default_base_url = req.default_base_url;
            }
        }

        assert_eq!(
            default_base_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
    }

    #[tokio::test]
    async fn handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let config = Config::default();

        handle_connect(&tx, &config, "minimax-coding-plan").await;
        drop(tx);

        let mut default_base_url = None;
        while let Some(output) = rx.recv().await {
            if let UiOutput::CredentialRequest(req) = output {
                default_base_url = req.default_base_url;
            }
        }

        assert_eq!(
            default_base_url.as_deref(),
            Some("https://api.minimax.io/anthropic/v1/messages")
        );
    }

    #[tokio::test]
    async fn handle_connect_with_credential_sets_anthropic_protocol_for_minimax_endpoint() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let new_config = with_isolated_home(|| async {
            let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
            let config = Config::default();
            let cred = talos_conversation::CredentialResponseData {
                provider: "minimax-coding-plan".to_string(),
                api_key: "minimax-secret".to_string(),
                model_id: None,
                connect_mode: true,
                base_url: Some("https://api.minimax.io/anthropic/v1/messages".to_string()),
            };

            let result = handle_connect_with_credential(&tx, &config, cred).await;
            drop(tx);
            while rx.recv().await.is_some() {}
            result
        })
        .await
        .expect("minimax coding plan connect must succeed");

        let minimax = new_config
            .providers
            .get("minimax-coding-plan")
            .expect("minimax coding plan entry created");
        assert_eq!(
            minimax.protocol,
            talos_config::ProviderProtocol::AnthropicMessages
        );
        assert_eq!(
            minimax.base_url.as_deref(),
            Some("https://api.minimax.io/anthropic/v1/messages")
        );
    }

    #[tokio::test]
    async fn handle_connect_already_authenticated_does_not_request_credential() {
        let (tx, mut rx) = mpsc::unbounded_channel::<UiOutput>();
        let mut config = Config::default();
        config.providers.insert(
            "groq".to_string(),
            ProviderConfig {
                api_key: Some("gsk-existing".to_string()),
                ..Default::default()
            },
        );

        handle_connect(&tx, &config, "groq").await;
        drop(tx);

        let mut saw_credential_request = false;
        while let Some(output) = rx.recv().await {
            if matches!(output, UiOutput::CredentialRequest(_)) {
                saw_credential_request = true;
            }
        }
        assert!(
            !saw_credential_request,
            "already-connected provider must not prompt for credentials again"
        );
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;

    #[test]
    fn parse_dashboard_board_section_extracts_items() {
        let board = "# Board

## Now

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T57 Tool sweep | Active | [x](x.md) | Tests |

## Blocked / Paused

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T58 Dashboard review | Blocked | [x](x.md) | Security |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| T61 Rehearsal | Planned | [x](x.md) | Evidence |
";

        assert_eq!(
            crate::dashboard_helpers::parse_dashboard_board_section(board, "Blocked / Paused"),
            vec![("T58 Dashboard review".to_string(), "Blocked".to_string())]
        );
        assert_eq!(
            crate::dashboard_helpers::parse_dashboard_board_section(board, "Next"),
            vec![("T61 Rehearsal".to_string(), "Planned".to_string())]
        );
    }
}
