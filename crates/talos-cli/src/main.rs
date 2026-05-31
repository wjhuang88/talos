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
use serde_json::Value;
use talos_agent::context::ContextLoader;
use talos_agent::Agent;
use talos_config::{Config, Provider};
use talos_core::message::AgentEvent;
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_permission::PermissionDecision;
use talos_provider::AnthropicProvider;
use talos_session::{IndexError, SessionManager};
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};
use talos_tui::Tui;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::approval::ApprovalPrompt;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.search.is_some() {
        return run_search_mode(cli);
    }

    if cli.list {
        return run_list_mode(cli);
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

async fn run_print_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = config.api_key().map_err(|e| anyhow!("{e}"))?;

    let prompt = resolve_prompt(cli.prompt)?;
    let prompt = if cli.no_context {
        prompt
    } else {
        let workspace_root = std::env::current_dir().context("failed to determine working directory")?;
        let context = ContextLoader::new(workspace_root).load().map_err(|e| anyhow!("{e}"))?;
        if context.is_empty() {
            prompt
        } else {
            format!("{context}\n\n{prompt}")
        }
    };

    let provider = Arc::new(AnthropicProvider::new(api_key, &config.model));

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

    let agent = Agent::new(provider, registry);

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
                return Ok(());
            }
            Ok(AgentEvent::Error { message }) => {
                eprintln!("Error: {message}");
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

    if config.model.is_empty() {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = config.api_key().map_err(|e| anyhow!("{e}"))?;

    let provider = Arc::new(AnthropicProvider::new(api_key, &config.model));

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

    let agent = Agent::new(provider, registry);

    let (event_tx, event_rx) = broadcast::channel::<AgentEvent>(32);

    let mut tui = Tui::new().context("failed to initialize TUI")?;

    let run_handle = tokio::spawn(async move { agent.run_streaming("Hello".to_string(), event_tx).await });

    let tui_result = tui.run(event_rx).await;

    run_handle.abort();

    tui_result
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

    let event_loop = event_loop::EventLoop::new(cli, workspace_root, session);
    event_loop.run().await
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
        IndexError::SqliteError(e) => anyhow!("list error: {e}"),
        IndexError::IoError(e) => anyhow!("I/O error: {e}"),
        IndexError::InvalidUuid(e) => anyhow!("invalid UUID: {e}"),
    })?;

    if sessions.is_empty() {
        println!("No indexed sessions found. Run sessions to build the index.");
        return Ok(());
    }

    println!("Recent sessions ({}):\n", sessions.len());

    for (i, session) in sessions.iter().enumerate() {
        let ts = session.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        println!(
            "{:>3}. {} | {} | {} messages | {}",
            i + 1,
            session.id,
            session.project,
            session.message_count,
            ts,
        );
    }

    Ok(())
}

/// Fork an existing session, creating a new session file with entries up to the
/// fork point. Records the fork relationship in the SQLite index.
fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<talos_session::Session> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use talos_session::{Session, SessionEntry};

    let source = manager
        .resume_session(source_session_id)
        .with_context(|| format!("failed to load source session {source_session_id}"))?;

    let entries = source.read_entries().context("failed to read source entries")?;
    if entries.is_empty() {
        bail!("cannot fork an empty session");
    }

    let fork_entry_id = entries.last().map(|e| e.id.clone()).unwrap();

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
}
