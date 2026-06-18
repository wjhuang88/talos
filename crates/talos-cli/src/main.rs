//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! interactive mode for conversational agent sessions, and optional
//! stdin pipe input and CLI argument overrides.
//!
//! # Session Commands
//!
//! - `--search <query>`: Full-text search across indexed session messages
//! - `--list`: List recent sessions from the SQLite index
//! - `--resume`: Interactive session selection from recent sessions
//! - `--continue`: Resume the most recent session automatically
//! - `--session <id>`: Resume a specific session by UUID

mod approval;
mod colors;
mod event_loop;
mod logging;
mod provider_setup;
mod registry;
mod runtime_adapter;
mod session_setup;
mod tui_bridge;

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use clap::ValueEnum;
use rmcp::ServiceExt;
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_conversation::{ConversationEngine, UiOutput, UserInput};
use talos_core::message::AgentEvent;
use talos_core::session::{SessionConfig, SessionEvent, SessionOp};
use talos_core::tool::ToolRegistry;
use talos_mcp::client::McpClientManager;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::PermissionRule;
use talos_plugin::{HookRegistry, LoggingHandler};
use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::{
    BashTool, DeleteTool, DiffTool, EditTool, GlobTool, GrepTool, LsTool, ReadTool, StatTool,
    TreeTool, WriteTool,
};
use talos_tui::Tui;
use tokio::sync::mpsc;

use crate::approval::ApprovalPrompt;
use crate::logging::init_logger;
use crate::provider_setup::{build_provider, config_to_mcp_client_config, parse_provider};
use crate::registry::{
    PermissionAwareTool, TuiApprovalHandler, build_mcp_tool_registry, build_print_tool_registry,
    build_tui_tool_registry,
};
use crate::session_setup::{
    ResumeSelection, canonical_workspace_root, resolve_prompt, resolve_session_for_workspace,
    resolve_workspace_root, run_learned_mode, run_list_mode, run_search_mode,
    workspace_display_name,
};
use crate::tui_bridge::run_conversation_loop;

/// Runtime mode selection.
#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    /// Print mode.
    Print,
    /// TUI mode.
    Tui,
    /// Interactive mode.
    Interactive,
    /// MCP server placeholder.
    McpServer,
    /// JSON-RPC placeholder.
    Rpc,
}

#[derive(Parser, Clone)]
#[command(name = "talos", version, about = "Next-generation agent runtime")]
struct Cli {
    #[arg(help = "The prompt to send to the agent.")]
    prompt: Option<String>,

    #[arg(
        short,
        long,
        help = "Print mode: stream the response to stdout and exit."
    )]
    print: bool,

    #[arg(
        short,
        long,
        help = "Override the model name (e.g., `claude-sonnet-4-20250514`)."
    )]
    model: Option<String>,

    #[arg(long, help = "Override the provider (`anthropic` or `openai`).")]
    provider: Option<String>,

    #[arg(long, help = "Launch terminal UI instead of readline loop.")]
    tui: bool,

    #[arg(
        long,
        conflicts_with_all = ["tui", "repl", "print"],
        help = "Inline terminal mode: Codex-like UX, no alt-screen, preserves scrollback."
    )]
    inline: bool,

    #[arg(
        long,
        conflicts_with = "tui",
        help = "Force the readline interactive REPL (default is TUI on a TTY)."
    )]
    repl: bool,

    #[arg(long, help = "Skip loading workspace context.")]
    no_context: bool,

    #[arg(short = 'c', long, help = "Resume the most recent session.")]
    r#continue: bool,

    #[arg(short = 'r', long, help = "List sessions and prompt for selection.")]
    resume: bool,

    #[arg(long, help = "Resume a specific session by ID.")]
    session: Option<String>,

    #[arg(long, help = "Fork from a specific session ID, creating a new branch.")]
    fork: Option<String>,

    #[arg(long, help = "Search session messages with full-text search.")]
    search: Option<String>,

    #[arg(long, help = "List recent sessions from the index.")]
    list: bool,

    #[arg(
        long,
        default_value = "20",
        help = "Maximum results for --search or --list."
    )]
    limit: usize,

    #[arg(long, help = "Override the default system prompt entirely.")]
    system_prompt: Option<String>,

    #[arg(long, help = "Append additional instructions to the system prompt.")]
    append_system_prompt: Option<String>,

    #[arg(
        short = 'w',
        long,
        value_name = "PATH",
        help = "Set the workspace root directory (default: current working directory)."
    )]
    workspace: Option<String>,

    #[arg(
        long,
        help = "Use mock LLM provider for testing (no API key required)."
    )]
    mock: bool,

    #[arg(long, help = "Display learned patterns from the evolution engine.")]
    learned: bool,

    #[arg(long, value_enum, help = "Explicit runtime mode.")]
    mode: Option<Mode>,

    // I009-S3 begin
    #[cfg(debug_assertions)]
    #[arg(
        long,
        value_name = "PATH",
        help = "Use local fixture MCP server binary (tests/dev only)."
    )]
    mcp_server_fixture: Option<PathBuf>,
    // I009-S3 end
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if matches!(cli.mode, Some(Mode::McpServer)) {
        return run_mcp_server().await;
    }

    let terminal_ui = cli.tui
        || (!cli.print
            && cli.search.is_none()
            && !cli.list
            && !cli.learned
            && !matches!(cli.mode, Some(Mode::Rpc))
            && io::stdin().is_terminal());
    let config_for_logging = Config::load().ok();
    init_logger(
        config_for_logging.as_ref().map(|config| &config.log),
        terminal_ui,
    );

    if cli.search.is_some() {
        return run_search_mode(cli);
    }

    if cli.list {
        return run_list_mode(cli);
    }

    if cli.learned {
        return run_learned_mode(cli);
    }

    if matches!(cli.mode, Some(Mode::Rpc)) {
        return run_rpc_mode(cli).await;
    }

    if cli.print {
        return run_print_mode(cli).await;
    }

    if cli.tui {
        return run_tui_mode(cli).await;
    }

    if cli.inline {
        return run_inline_mode(cli).await;
    }

    if cli.repl {
        return run_interactive_mode(cli).await;
    }

    if !io::stdin().is_terminal() {
        return run_print_mode(cli).await;
    }

    run_tui_mode(cli).await
}

async fn run_rpc_mode(cli: Cli) -> Result<()> {
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
    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        build_print_tool_registry(),
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        PathBuf::from("."),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());

    let server = talos_rpc::RpcServer::new(Arc::new(runtime_adapter::AgentRuntime(agent)));
    server.run_stdio().await
    // I009-S5 end
}

async fn run_print_mode(cli: Cli) -> Result<()> {
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
    let prompt = resolve_prompt(cli.prompt)?;

    let hooks = build_hook_registry(true);
    let mut registry = build_print_tool_registry();
    let mut permission_engine =
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.clone());
    // I009-S3 begin
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

    #[cfg(debug_assertions)]
    let fixture_mode = cli.mcp_server_fixture.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;

    let mcp_manager =
        McpClientManager::start(&config_to_mcp_client_config(&config.mcp), hooks.clone()).await?;
    for startup_failure in mcp_manager.startup_failures() {
        eprintln!(
            "Warning: MCP server '{}' failed to start: {}",
            startup_failure.server, startup_failure.error
        );
    }
    for tool in mcp_manager.discover_tools().await {
        if tool.is_read_only() {
            permission_engine.add_rule(PermissionRule::new(
                tool.name(),
                None,
                talos_permission::PermissionDecision::Allow,
            ));
        }
        registry.register(tool);
    }
    // I009-S3 end

    let provider = if fixture_mode && cli.mock {
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
            talos_permission::PermissionEngine::with_workspace_root(workspace_root.clone()),
        )),
        None,
        workspace_root.clone(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.clone())
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

    let session_config = SessionConfig {
        print_mode: true,
        workspace_root: workspace_root.clone(),
        initial_history: vec![],
        model_context_limit: 128_000,
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
    bail!("session event channel closed unexpectedly");
}

async fn run_tui_mode(cli: Cli) -> Result<()> {
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

    let (ui_output_tx, ui_output_rx) = mpsc::unbounded_channel::<UiOutput>();
    let approval_handler = Arc::new(TuiApprovalHandler::new(
        ui_output_tx.clone(),
        workspace_root.clone(),
    ));

    let hooks = build_hook_registry(true);
    let provider = build_provider(&config, &api_key, cli.mock);
    let registry = build_tui_tool_registry(approval_handler, workspace_root.clone());

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.clone(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.clone())
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

    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.clone(),
        initial_history,
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    let sq_tx_signal = handle.sq_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            let _ = sq_tx_signal.try_send(SessionOp::Interrupt);
        }
    });

    let (bridge_tx, bridge_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let session_for_persist = session.clone();
    let session_manager_for_persist = session_manager.clone();
    let mut bridge_forwarder = handle.eq_rx;
    tokio::spawn(async move {
        let mut pending_tool_results: Vec<talos_core::message::MessageToolResult> = Vec::new();
        while let Some(session_event) = bridge_forwarder.recv().await {
            match session_event {
                SessionEvent::AgentEvent(ref agent_event) => {
                    if let AgentEvent::ToolResult { result } = agent_event {
                        pending_tool_results.push(result.clone());
                    }
                    let _ = bridge_tx.send(agent_event.clone());
                }
                SessionEvent::TurnCompleted {
                    status: talos_core::session::TurnCompletionStatus::Success { final_text },
                    ..
                } if !final_text.is_empty() => {
                    let assistant_msg = talos_core::message::Message::Assistant {
                        content: final_text,
                        tool_calls: vec![],
                    };
                    if let Err(e) = session_for_persist.append(&assistant_msg) {
                        eprintln!("Warning: failed to persist assistant message: {e}");
                    }
                    for result in pending_tool_results.drain(..) {
                        let tool_msg = talos_core::message::Message::Tool { result };
                        if let Err(e) = session_for_persist.append(&tool_msg) {
                            eprintln!("Warning: failed to persist tool result: {e}");
                        }
                    }
                    if let Err(e) = session_manager_for_persist.update_index(&session_for_persist) {
                        eprintln!("Warning: failed to update session index: {e}");
                    }
                }
                SessionEvent::TurnCompleted {
                    status: talos_core::session::TurnCompletionStatus::Success { .. },
                    ..
                } => {
                    pending_tool_results.clear();
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
    });

    let session_for_user_persist = session.clone();
    let session_manager_for_user_persist = session_manager.clone();
    let (user_msg_tx, mut user_msg_rx) = mpsc::unbounded_channel::<String>();
    let sq_tx_inner = handle.sq_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = user_msg_rx.recv().await {
            let user_msg = talos_core::message::Message::User {
                content: msg.clone(),
            };
            if let Err(e) = session_for_user_persist.append(&user_msg) {
                eprintln!("Warning: failed to persist user message: {e}");
            }
            if let Err(e) = session_manager_for_user_persist.update_index(&session_for_user_persist)
            {
                eprintln!("Warning: failed to update session index: {e}");
            }
            let _ = sq_tx_inner.send(SessionOp::Submit { message: msg }).await;
        }
    });

    let mut tui = Tui::new().context("failed to initialize TUI")?;
    tui.hydrate_history(&visible_history);

    let (user_input_tx, user_input_rx) = mpsc::unbounded_channel::<UserInput>();

    tui.set_ui_output_rx(ui_output_rx);
    tui.set_user_input_tx(user_input_tx);
    tui.set_model_name(config.model.clone());

    let engine = ConversationEngine::new(config.model.clone());
    let interrupt_tx = handle.sq_tx.clone();
    tokio::spawn(async move {
        run_conversation_loop(
            engine,
            bridge_rx,
            user_input_rx,
            ui_output_tx,
            user_msg_tx,
            interrupt_tx,
        )
        .await;
    });

    tui.run().await?;
    Ok(())
}

async fn run_inline_mode(cli: Cli) -> Result<()> {
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
    let registry = build_print_tool_registry();

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.clone(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.clone())
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

    let session_config = SessionConfig {
        print_mode: true,
        workspace_root: workspace_root.clone(),
        initial_history,
        model_context_limit: 128_000,
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
                        talos_core::session::TurnCompletionStatus::Success { final_text } => {
                            if !final_text.is_empty() {
                                let assistant_msg = talos_core::message::Message::Assistant {
                                    content: final_text,
                                    tool_calls: vec![],
                                };
                                if let Err(e) = session.append(&assistant_msg) {
                                    eprintln!("Warning: failed to persist assistant message: {e}");
                                }
                                if let Err(e) = session_manager.update_index(&session) {
                                    eprintln!("Warning: failed to update session index: {e}");
                                }
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

async fn run_interactive_mode(cli: Cli) -> Result<()> {
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
        bail!("no model configured");
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
        inner: Arc::new(BashTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ReadTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(WriteTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(EditTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GrepTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GlobTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(LsTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DeleteTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DiffTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(StatTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(GitStatusTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitDiffTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitLogTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitShowTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitBranchListTool::new(workspace_root.clone())));
    registry.register(Arc::new(TreeTool::new(workspace_root.clone())));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitAddTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPushTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPullTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(workspace_root.clone())),
        approval: approval.clone(),
        print_mode: false,
    }));

    let hooks = build_hook_registry(true);

    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.clone(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());

    if !cli.no_context {
        let context = ContextLoader::new(workspace_root.clone())
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

    let session_config = SessionConfig {
        print_mode: false,
        workspace_root: workspace_root.clone(),
        initial_history,
        model_context_limit: 128_000,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    let event_loop = event_loop::EventLoop::new(workspace_root, session, session_manager, handle);
    event_loop.run().await
}

fn build_hook_registry(include_evolution: bool) -> Arc<HookRegistry> {
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(LoggingHandler::new()));
    if include_evolution {
        match talos_evolution::EvolutionHookHandler::open_default(
            talos_evolution::EvolutionConfig::default(),
            None,
        ) {
            Ok(Some(handler)) => registry.register(Arc::new(handler)),
            Ok(None) => {}
            Err(e) => eprintln!("Warning: evolution disabled: {e}"),
        }
    }
    Arc::new(registry)
}

// I009-S4 begin
async fn run_mcp_server() -> Result<()> {
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

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn parse_provider_anthropic() {
        assert_eq!(parse_provider("anthropic").unwrap(), "anthropic");
        assert_eq!(parse_provider("Anthropic").unwrap(), "anthropic");
        assert_eq!(parse_provider("ANTHROPIC").unwrap(), "anthropic");
    }

    #[test]
    fn parse_provider_openai() {
        assert_eq!(parse_provider("openai").unwrap(), "openai");
        assert_eq!(parse_provider("OpenAI").unwrap(), "openai");
    }

    #[test]
    fn parse_provider_custom_name() {
        assert_eq!(parse_provider("DashScope").unwrap(), "dashscope");
        assert!(parse_provider("").is_err());
    }

    // === Snippet Highlighting Tests ===

    #[test]
    fn highlight_snippet_replaces_b_tags() {
        let input = "This is a <b>matched</b> term in the snippet.";
        let output = registry::highlight_snippet(input);
        assert!(output.contains(colors::NORD13));
        assert!(output.contains(colors::BOLD));
        assert!(!output.contains("BOLD"));
        assert!(!output.contains("<b>"));
        assert!(!output.contains("</b>"));
    }

    #[test]
    fn highlight_snippet_multiple_matches() {
        let input = "<b>first</b> and <b>second</b> match";
        let output = registry::highlight_snippet(input);
        let nord13_count = output.matches(colors::NORD13).count();
        assert_eq!(
            nord13_count, 4,
            "Should have 4 NORD13 sequences (2 per match)"
        );
    }

    #[test]
    fn highlight_snippet_no_tags_passthrough() {
        let input = "No matches in this snippet.";
        let output = registry::highlight_snippet(input);
        assert_eq!(output, input);
    }

    #[test]
    fn highlight_snippet_empty_string() {
        let output = registry::highlight_snippet("");
        assert_eq!(output, "");
    }

    // === Session ID Parsing Tests ===

    #[test]
    fn session_id_valid_uuid_parses() {
        let valid_id = "550e8400-e29b-41d4-a716-446655440000";
        let result = uuid::Uuid::parse_str(valid_id);
        assert!(result.is_ok());
    }

    #[test]
    fn session_id_invalid_uuid_fails() {
        let invalid_ids = vec!["not-a-uuid", "550e8400-e29b-41d4-a716", ""];
        for invalid_id in invalid_ids {
            let result = uuid::Uuid::parse_str(invalid_id);
            assert!(result.is_err(), "Should fail to parse: {invalid_id}");
        }
    }

    // === Color Constant Tests ===

    #[test]
    fn color_constants_are_non_empty() {
        assert!(!colors::RESET.is_empty());
        assert!(!colors::BOLD.is_empty());
        assert!(!colors::NORD3.is_empty());
        assert!(!colors::NORD8.is_empty());
        assert!(!colors::NORD13.is_empty());
        assert!(!colors::NORD14.is_empty());
    }

    #[test]
    fn color_constants_contain_ansi_escape() {
        for color in [colors::NORD3, colors::NORD8, colors::NORD13, colors::NORD14] {
            assert!(
                color.starts_with("\x1b["),
                "Color constant should start with ANSI escape: {color:?}"
            );
        }
        assert!(colors::RESET.starts_with("\x1b["));
        assert!(colors::BOLD.starts_with("\x1b["));
    }

    #[tokio::test]
    async fn conversation_loop_displays_drained_queued_input() {
        let engine = ConversationEngine::new("test-model".to_string());
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (submit_tx, mut submit_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            agent_rx,
            user_rx,
            ui_tx,
            submit_tx,
            interrupt_tx,
        ));

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        user_tx
            .send(UserInput::Message("queued follow-up".to_string()))
            .unwrap();
        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                usage: Default::default(),
            })
            .unwrap();

        let mut saw_queued_user_stream = false;
        let mut saw_queue_drained_status = false;
        for _ in 0..8 {
            let Some(output) = ui_rx.recv().await else {
                break;
            };
            match output {
                UiOutput::Stream(msg) if msg.source == talos_conversation::MessageSource::User => {
                    saw_queued_user_stream = true;
                }
                UiOutput::Status(status) if status.is_processing && status.steering_count == 0 => {
                    saw_queue_drained_status = true;
                }
                _ => {}
            }
            if saw_queued_user_stream && saw_queue_drained_status {
                break;
            }
        }

        assert!(saw_queued_user_stream);
        assert!(saw_queue_drained_status);
        assert!(matches!(
            submit_rx.try_recv(),
            Ok(message) if message == "queued follow-up"
        ));

        drop(agent_tx);
        drop(user_tx);
        loop_handle.await.unwrap();
    }

    // === Error Handling Tests ===

    #[test]
    fn session_manager_resume_invalid_id() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let result = manager.resume_session("not-a-valid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn session_manager_resume_nonexistent_session() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let result = manager.resume_session(&valid_uuid);
        assert!(result.is_err());
        match result.unwrap_err() {
            talos_session::SessionError::SessionNotFound(_) => {}
            other => panic!("expected SessionNotFound, got {other:?}"),
        }
    }

    #[test]
    fn session_manager_search_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let results = manager.search("nonexistent", 10);
        if let Ok(r) = results {
            assert!(r.is_empty());
        }
    }

    #[test]
    fn session_manager_list_recent_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let results = manager.list_recent(10);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }
}
