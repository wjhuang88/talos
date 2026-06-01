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
mod event_loop;
mod evolution_runtime;

/// Nord theme ANSI color constants for terminal output.
///
/// Reference: https://www.nordtheme.com/docs/colors-and-palettes
mod colors {
    /// Reset all formatting.
    pub const RESET: &str = "\x1b[0m";
    /// Bold text.
    pub const BOLD: &str = "\x1b[1m";

    // Polar Night
    /// nord3 — comments, timestamps (RGB: 76, 86, 106).
    pub const NORD3: &str = "\x1b[38;2;76;86;106m";

    // Frost
    /// nord8 — primary accent, session IDs (RGB: 136, 192, 208).
    pub const NORD8: &str = "\x1b[38;2;136;192;208m";

    // Aurora
    /// nord13 — warning, snippet highlights (RGB: 235, 203, 139).
    pub const NORD13: &str = "\x1b[38;2;235;203;139m";
    /// nord14 — success, project names (RGB: 163, 190, 140).
    pub const NORD14: &str = "\x1b[38;2;163;190;140m";
}

use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use clap::ValueEnum;
use rmcp::ServiceExt;
use serde_json::Value;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::Agent;
use talos_config::{Config, Provider};
#[cfg(debug_assertions)]
use talos_config::McpServerConfig;
use talos_core::message::AgentEvent;
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_evolution::store::KnowledgeStore;
use talos_mcp::client::McpClientManager;
use talos_mcp::server::{McpPermissionGate, TalosMcpHandler};
use talos_permission::{PermissionDecision, PermissionRule};
use talos_plugin::{HookRegistry, LoggingHandler};
use talos_provider::AnthropicProvider;
use talos_provider::openai::OpenAIProvider;
use talos_rpc::RpcServer;
use talos_session::{IndexError, SessionManager};
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};
use talos_tui::Tui;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::approval::ApprovalPrompt;
use crate::evolution_runtime::EvolutionRuntime;

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

/// Permission-aware tool wrapper that checks the permission engine before
/// executing the underlying tool. In interactive mode, [`PermissionDecision::Ask`]
/// triggers a user prompt. In print mode, it defaults to deny.
pub(crate) struct PermissionAwareTool {
    inner: Arc<dyn AgentTool>,
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
}

#[async_trait]
impl AgentTool for PermissionAwareTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let tool_name = self.inner.name().to_owned();
        let decision = {
            let mut approval = self.approval.lock().expect("approval lock poisoned");
            let engine_decision = approval.engine().evaluate(&tool_name, &input);

            match engine_decision {
                PermissionDecision::Allow => PermissionDecision::Allow,
                PermissionDecision::Deny(reason) => PermissionDecision::Deny(reason),
                PermissionDecision::Ask => {
                    if self.print_mode {
                        PermissionDecision::Deny(
                            "Print mode: interactive approval unavailable".to_string(),
                        )
                    } else {
                        match approval.prompt(&tool_name, &input) {
                            Ok(decision) => decision,
                            Err(e) => PermissionDecision::Deny(format!("Approval error: {e}")),
                        }
                    }
                }
            }
        };

        match decision {
            PermissionDecision::Allow => self.inner.execute(input).await,
            PermissionDecision::Deny(reason) => {
                ToolResult::error(format!("Permission denied: {reason}"))
            }
            PermissionDecision::Ask => {
                unreachable!("Ask decision should have been resolved by prompt or print-mode default")
            }
        }
    }

    fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }
}

#[derive(Parser, Clone)]
#[command(name = "talos", version, about = "Next-generation agent runtime")]
struct Cli {
    #[arg(help = "The prompt to send to the agent.")]
    prompt: Option<String>,

    #[arg(short, long, help = "Print mode: stream the response to stdout and exit.")]
    print: bool,

    #[arg(short, long, help = "Override the model name (e.g., `claude-sonnet-4-20250514`).")]
    model: Option<String>,

    #[arg(long, help = "Override the provider (`anthropic` or `openai`).")]
    provider: Option<String>,

    #[arg(long, help = "Launch terminal UI instead of readline loop.")]
    tui: bool,

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

    #[arg(long, default_value = "20", help = "Maximum results for --search or --list.")]
    limit: usize,

    #[arg(long, help = "Override the default system prompt entirely.")]
    system_prompt: Option<String>,

    #[arg(long, help = "Append additional instructions to the system prompt.")]
    append_system_prompt: Option<String>,

    #[arg(long, help = "Use mock LLM provider for testing (no API key required).")]
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

    init_tracing();

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

    if !io::stdin().is_terminal() {
        return run_print_mode(cli).await;
    }

    run_interactive_mode(cli).await
}

fn run_learned_mode(_cli: Cli) -> Result<()> {
    let db_path = dirs::home_dir()
        .context("failed to find home directory")?
        .join(".talos")
        .join("index.db");

    if !db_path.exists() {
        println!("No evolution data found. Run talos with an agent to start learning.");
        return Ok(());
    }

    let store = KnowledgeStore::open(db_path.to_str().unwrap_or_default())
        .context("failed to open knowledge store")?;

    let patterns = store.get_all_patterns().context("failed to get patterns")?;

    if patterns.is_empty() {
        println!("No patterns learned yet. Use the agent and provide feedback to start learning.");
        return Ok(());
    }

    println!("=== Learned Patterns ===\n");

    for (i, pattern) in patterns.iter().enumerate() {
        let status = if pattern.active { "active" } else { "inactive" };
        println!(
            "{}. [{}] {} (confidence: {:.0}%, evidence: {}, status: {})",
            i + 1,
            pattern.category,
            pattern.description,
            pattern.confidence * 100.0,
            pattern.evidence_count,
            status
        );
        println!("   Instruction: {}", pattern.instruction);
        println!();
    }

    Ok(())
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
        String::new()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let hooks = build_hook_registry();
    let agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        build_print_tool_registry(),
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        PathBuf::from("."),
        hooks,
    );

    let server = RpcServer::new(Arc::new(agent));
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
        String::new()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let prompt = resolve_prompt(cli.prompt)?;

    let hooks = build_hook_registry();
    let mut registry = build_print_tool_registry();
    let mut permission_engine = talos_permission::PermissionEngine::new();
    // I009-S3 begin
    #[cfg(debug_assertions)]
    if let Some(path) = cli.mcp_server_fixture.clone() {
        config.mcp.servers = vec![McpServerConfig {
            name: "fixture".to_string(),
            transport: "stdio".to_string(),
            command: path.to_string_lossy().to_string(),
            args: Vec::new(),
            env: std::collections::HashMap::from([
                ("ECHO_PREFIX".to_string(), "fixture".to_string()),
            ]),
            cwd: std::env::current_dir().ok(),
        }];
    }

    #[cfg(debug_assertions)]
    let fixture_mode = cli.mcp_server_fixture.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;

    let mcp_manager = McpClientManager::start(&config.mcp, hooks.clone()).await?;
    for startup_failure in mcp_manager.startup_failures() {
        eprintln!(
            "Warning: MCP server '{}' failed to start: {}",
            startup_failure.server, startup_failure.error
        );
    }
    for tool in mcp_manager.discover_tools().await {
        permission_engine.add_rule(PermissionRule::new(
            tool.name(),
            None,
            PermissionDecision::Allow,
        ));
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
        Some(Arc::new(permission_engine)),
        None,
        PathBuf::from("."),
        hooks,
    );

    if !cli.no_context {
        let workspace_root = std::env::current_dir().context("failed to determine working directory")?;
        let context = ContextLoader::new(workspace_root).load().map_err(|e| anyhow!("{e}"))?;
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

    let mut evolution = match EvolutionRuntime::open_default(None) {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("Warning: evolution disabled: {e}");
            None
        }
    };

    let mut append_parts: Vec<String> = Vec::new();
    if let Some(ref append_prompt) = cli.append_system_prompt {
        append_parts.push(append_prompt.clone());
    }
    if let Some(runtime) = evolution.as_ref() {
        let context = runtime.evolution_context();
        if !context.is_empty() {
            append_parts.push(context);
        }
    }
    if !append_parts.is_empty() {
        agent.set_append_prompt(append_parts.join("\n\n"));
    }

    if let Some(runtime) = evolution.as_mut() {
        runtime.start_turn();
        runtime.observe_user_input(&prompt);
    }

    let (event_tx, mut event_rx) = broadcast::channel::<AgentEvent>(32);

    let _run_handle = tokio::spawn(async move { agent.run_streaming(prompt, event_tx).await });

    let mut stdout = io::stdout().lock();
    loop {
        match event_rx.recv().await {
            Ok(AgentEvent::TextDelta { delta }) => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
            Ok(AgentEvent::TurnEnd { .. }) => {
                println!();
                if let Some(runtime) = evolution.as_mut() {
                    if let Err(e) = runtime.ingest() {
                        eprintln!("Warning: evolution ingest failed: {e}");
                    }
                }
                return Ok(());
            }
            Ok(AgentEvent::Error { message }) => {
                eprintln!("Error: {message}");
                if let Some(runtime) = evolution.as_mut() {
                    runtime.observe_event(&AgentEvent::Error {
                        message: message.clone(),
                    });
                    let _ = runtime.ingest();
                }
                std::process::exit(1);
            }
            Ok(AgentEvent::TurnStart | AgentEvent::ToolCall { .. } | AgentEvent::ToolResult { .. }) => {}
            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("Warning: dropped {n} event(s) due to slow consumer");
            }
            Err(broadcast::error::RecvError::Closed) => {
                bail!("event channel closed before TurnEnd");
            }
        }
    }
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
        String::new()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let hooks = build_hook_registry();

    let (event_tx, event_rx) = broadcast::channel::<AgentEvent>(32);

    let mut tui = Tui::new().context("failed to initialize TUI")?;

    // Channel for TUI to send user messages back for agent processing
    let (user_msg_tx, mut user_msg_rx) = mpsc::unbounded_channel::<String>();
    tui.set_message_tx(user_msg_tx);

    // Keep the broadcast channel alive so the TUI doesn't exit when an agent task completes
    let _event_tx_alive = event_tx.clone();

    // Spawn a task to handle user messages from TUI and spawn agent tasks
    let config_clone = config.clone();
    let api_key_clone = api_key.clone();
    tokio::spawn(async move {
        while let Some(input) = user_msg_rx.recv().await {
            let provider = build_provider(&config_clone, &api_key_clone, cli.mock);
            let registry = build_print_tool_registry();
            let agent = Agent::with_security_and_hooks(
                provider,
                registry,
                Some(Arc::new(talos_permission::PermissionEngine::new())),
                None,
                PathBuf::from("."),
                hooks.clone(),
            );
            let event_tx = event_tx.clone();
            tokio::spawn(async move {
                let _ = agent.run_streaming(input, event_tx).await;
            });
        }
    });

    // Run TUI in the main task (blocking until user exits)
    tui.run(event_rx).await
}

async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let workspace_root = std::env::current_dir().context("failed to determine working directory")?;

    let session_manager = SessionManager::new().context("failed to initialize session manager")?;

    let session = if let Some(ref source_session_id) = cli.fork {
        fork_session(&session_manager, source_session_id)?
    } else if let Some(ref session_id) = cli.session {
        session_manager
            .resume_session(session_id)
            .with_context(|| format!("failed to resume session {session_id}"))?
    } else if cli.r#continue {
        let sessions = session_manager
            .list_sessions()
            .context("failed to list sessions")?;
        if sessions.is_empty() {
            let project_name = workspace_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("default");
            session_manager
                .create_session(project_name)
                .context("failed to create session")?
        } else {
            let most_recent = sessions
                .iter()
                .max_by_key(|s| s.timestamp)
                .context("no sessions found")?;
            session_manager
                .get_session(&most_recent.id)
                .with_context(|| format!("failed to resume session {}", most_recent.id))?
        }
    } else if cli.resume {
        let sessions = session_manager
            .list_sessions()
            .context("failed to list sessions")?;
        if sessions.is_empty() {
            println!("No existing sessions. Creating a new one.");
            let project_name = workspace_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("default");
            session_manager
                .create_session(project_name)
                .context("failed to create session")?
        } else {
            println!("{}Available sessions:{}\n", colors::BOLD, colors::RESET);
            for (idx, s) in sessions.iter().enumerate() {
                let ts = s.timestamp.format("%Y-%m-%d %H:%M");
                println!(
                    "  {}. {}{}{} ({}{}{}) {}{} messages | {}",
                    idx + 1,
                    colors::NORD8,
                    s.id,
                    colors::RESET,
                    colors::NORD14,
                    s.project,
                    colors::RESET,
                    colors::NORD3,
                    s.message_count,
                    ts,
                );
            }
            print!("\nSelect a session (1-{}): ", sessions.len());
            io::stdout().flush().context("failed to flush stdout")?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("failed to read input")?;
            let choice: usize = input
                .trim()
                .parse()
                .context("invalid selection")?;
            if choice < 1 || choice > sessions.len() {
                bail!("selection out of range");
            }
            let selected = &sessions[choice - 1];
            session_manager
                .get_session(&selected.id)
                .with_context(|| format!("failed to resume session {}", selected.id))?
        }
    } else {
        let project_name = workspace_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        session_manager
            .create_session(project_name)
            .context("failed to create session")?
    };

    let event_loop = event_loop::EventLoop::new(
        cli,
        workspace_root,
        session,
        session_manager,
        build_hook_registry(),
    );
    event_loop.run().await
}

fn build_hook_registry() -> Arc<HookRegistry> {
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(LoggingHandler::new()));
    Arc::new(registry)
}

fn build_print_tool_registry() -> ToolRegistry {
    let approval = Arc::new(Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::new(),
    )));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(BashTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ReadTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(WriteTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(EditTool::new(PathBuf::from("."))),
        approval,
        print_mode: true,
    }));
    registry
}

/// A lightweight health/status tool for MCP mode.
struct StatusTool;

#[async_trait]
impl AgentTool for StatusTool {
    fn name(&self) -> &str {
        "status"
    }

    fn description(&self) -> &str {
        "Return Talos MCP server status"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("talos mcp server alive")
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

fn build_mcp_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ReadTool::new(PathBuf::from("."))));
    registry.register(Arc::new(WriteTool::new(PathBuf::from("."))));
    registry.register(Arc::new(EditTool::new(PathBuf::from("."))));
    registry.register(Arc::new(StatusTool));
    registry
}

// I009-S4 begin
async fn run_mcp_server() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .try_init();

    let tool_registry = Arc::new(build_mcp_tool_registry());
    let permission_engine = Arc::new(talos_permission::PermissionEngine::new());
    let hook_registry = build_hook_registry();
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

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn resolve_prompt(cli_prompt: Option<String>) -> Result<String> {
    if let Some(prompt) = cli_prompt {
        return Ok(prompt);
    }

    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read from stdin")?;
        let trimmed = buffer.trim().to_string();
        if trimmed.is_empty() {
            return Err(anyhow!("stdin is empty"));
        }
        return Ok(trimmed);
    }

    Err(anyhow!(
        "no prompt provided. Usage: talos \"your prompt\" -p, or echo \"prompt\" | talos -p"
    ))
}

pub(crate) fn parse_provider(s: &str) -> Result<Provider> {
    match s.to_lowercase().as_str() {
        "anthropic" => Ok(Provider::Anthropic),
        "openai" => Ok(Provider::OpenAI),
        other => Err(anyhow!("unknown provider '{other}': supported values are 'anthropic' and 'openai'")),
    }
}

pub(crate) fn build_provider(config: &Config, api_key: &str, mock: bool) -> Arc<dyn talos_core::provider::LanguageModel> {
    if mock {
        use talos_provider::mock::MockProvider;
        return Arc::new(MockProvider::new()
            .with_response("I'm a mock LLM. I can help you with testing and development without making real API calls."));
    }
    match config.provider {
        Provider::Anthropic => Arc::new(AnthropicProvider::new(api_key, &config.model)),
        Provider::OpenAI => Arc::new(OpenAIProvider::new(api_key, &config.model)),
    }
}

fn run_search_mode(cli: Cli) -> Result<()> {
    let query = cli.search.as_ref().expect("search query required");
    let manager = SessionManager::new().context("failed to initialize session manager")?;

    let results = manager.search(query, cli.limit).map_err(|e| match e {
        IndexError::SqliteError(e) => anyhow!("search error: {e}\nHint: run a session first to build the index."),
        IndexError::IoError(e) => anyhow!("I/O error: {e}"),
        IndexError::InvalidUuid(e) => anyhow!("invalid UUID: {e}"),
    })?;

    if results.is_empty() {
        println!("No results found for '{query}'.");
        return Ok(());
    }

    println!(
        "{}Found {} result(s) for '{}':{}\n",
        colors::BOLD,
        results.len(),
        query,
        colors::RESET
    );

    for (i, result) in results.iter().enumerate() {
        let ts = result.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let snippet = highlight_snippet(&result.snippet);
        println!(
            "{:>3}. {}{}{} {}{}{} {}{}{}\n     {}\n",
            i + 1,
            colors::NORD3,
            ts,
            colors::RESET,
            colors::NORD8,
            result.session_id,
            colors::RESET,
            colors::NORD14,
            result.project,
            colors::RESET,
            snippet,
        );
    }

    Ok(())
}

/// Format a search snippet with Nord theme highlighting for matched terms.
///
/// Replaces FTS5 `<b>...</b>` markers with ANSI color codes.
fn highlight_snippet(snippet: &str) -> String {
    snippet
        .replace("<b>", &format!("{}{}BOLD{}{}", colors::NORD13, colors::BOLD, colors::RESET, colors::NORD13))
        .replace("</b>", colors::RESET)
}

fn run_list_mode(cli: Cli) -> Result<()> {
    let manager = SessionManager::new().context("failed to initialize session manager")?;

    let sessions = manager.list_recent(cli.limit).map_err(|e| match e {
        IndexError::SqliteError(e) => anyhow!("list error: {e}\nHint: run `talos --search <term>` first to build the index."),
        IndexError::IoError(e) => anyhow!("I/O error: {e}"),
        IndexError::InvalidUuid(e) => anyhow!("invalid UUID: {e}"),
    })?;

    if sessions.is_empty() {
        println!("No indexed sessions found. Run `talos --search <term>` to build the index.");
        return Ok(());
    }

    println!(
        "{}Recent sessions ({}):{}\n",
        colors::BOLD,
        sessions.len(),
        colors::RESET
    );

    for (i, session) in sessions.iter().enumerate() {
        let ts = session.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        println!(
            "{:>3}. {}{}{} | {}{}{} | {} messages | {}{}{}",
            i + 1,
            colors::NORD8,
            session.id,
            colors::RESET,
            colors::NORD14,
            session.project,
            colors::RESET,
            session.message_count,
            colors::NORD3,
            ts,
            colors::RESET,
        );
    }

    Ok(())
}

/// Fork an existing session, creating a new session file with entries up to the
/// fork point. Records the fork relationship in the SQLite index.
fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<talos_session::Session> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use talos_session::Session;

    let source = manager
        .resume_session(source_session_id)
        .with_context(|| format!("failed to load source session {source_session_id}"))?;

    let entries = source.read_entries().context("failed to read source entries")?;
    if entries.is_empty() {
        bail!("cannot fork an empty session");
    }

    let fork_entry_id = entries.last().expect("entries checked non-empty above").id.clone();

    let new_id = Uuid::new_v4();
    let project_dir = manager
        .list_sessions()
        .context("failed to list sessions")?
        .iter()
        .find(|s| s.id.to_string() == source_session_id)
        .map(|s| s.project.clone())
        .unwrap_or_else(|| "default".to_string());

    let home = std::env::var("HOME").map_err(|e| anyhow!("{e}"))?;
    let sessions_dir = PathBuf::from(home).join(".talos").join("sessions");
    let project_path = sessions_dir.join(&project_dir);
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;

    let new_file_path = project_path.join(format!("{new_id}.jsonl"));

    let mut new_session = Session::new(new_id, project_dir.clone(), new_file_path.clone());

    for entry in &entries {
        let line = serde_json::to_string(entry).map_err(|e| anyhow!("serialize error: {e}"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_file_path)
            .context("failed to create fork file")?;
        writeln!(file, "{line}").context("failed to write fork entry")?;
    }

    new_session.fork(&fork_entry_id).context("failed to create fork branch")?;

    if let Ok(mut index) = talos_session::SessionIndex::new(
        &sessions_dir.join("index.db"),
    ) {
        let _ = index.init_schema();
        let _ = index.record_fork(source_session_id, &new_id.to_string(), &fork_entry_id);
        let _ = index.index_session(&new_session);
    }

    eprintln!("Forked session {source_session_id} -> {new_id} (from entry {fork_entry_id})");

    Ok(new_session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_provider_anthropic() {
        assert!(matches!(parse_provider("anthropic"), Ok(Provider::Anthropic)));
        assert!(matches!(parse_provider("Anthropic"), Ok(Provider::Anthropic)));
        assert!(matches!(parse_provider("ANTHROPIC"), Ok(Provider::Anthropic)));
    }

    #[test]
    fn parse_provider_openai() {
        assert!(matches!(parse_provider("openai"), Ok(Provider::OpenAI)));
        assert!(matches!(parse_provider("OpenAI"), Ok(Provider::OpenAI)));
    }

    #[test]
    fn parse_provider_unknown() {
        assert!(parse_provider("unknown").is_err());
        assert!(parse_provider("").is_err());
    }

    // === Snippet Highlighting Tests ===

    #[test]
    fn highlight_snippet_replaces_b_tags() {
        let input = "This is a <b>matched</b> term in the snippet.";
        let output = highlight_snippet(input);
        assert!(output.contains(colors::NORD13));
        assert!(output.contains(colors::BOLD));
        assert!(!output.contains("<b>"));
        assert!(!output.contains("</b>"));
    }

    #[test]
    fn highlight_snippet_multiple_matches() {
        let input = "<b>first</b> and <b>second</b> match";
        let output = highlight_snippet(input);
        // Each match produces 2 NORD13 sequences (before and after BOLD/RESET)
        let nord13_count = output.matches(colors::NORD13).count();
        assert_eq!(nord13_count, 4, "Should have 4 NORD13 sequences (2 per match)");
    }

    #[test]
    fn highlight_snippet_no_tags_passthrough() {
        let input = "No matches in this snippet.";
        let output = highlight_snippet(input);
        assert_eq!(output, input);
    }

    #[test]
    fn highlight_snippet_empty_string() {
        let output = highlight_snippet("");
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
        let invalid_ids = vec![
            "not-a-uuid",
            "550e8400-e29b-41d4-a716",
            "",
        ];
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
        // All color constants should start with the ANSI escape sequence
        for color in [
            colors::NORD3,
            colors::NORD8,
            colors::NORD13,
            colors::NORD14,
        ] {
            assert!(
                color.starts_with("\x1b["),
                "Color constant should start with ANSI escape: {color:?}"
            );
        }
        assert!(colors::RESET.starts_with("\x1b["));
        assert!(colors::BOLD.starts_with("\x1b["));
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

        // Search on empty index may return empty results or error if DB not initialized
        let results = manager.search("nonexistent", 10);
        // Either empty results or an error is acceptable for uninitialized index
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
