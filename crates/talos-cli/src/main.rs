//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! interactive mode for conversational agent sessions, and optional
//! stdin pipe input and CLI argument overrides.

mod approval;

use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use serde_json::Value;
use talos_agent::context::ContextLoader;
use talos_agent::Agent;
use talos_config::{Config, Provider};
use talos_core::message::{AgentEvent, Message};
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_permission::PermissionDecision;
use talos_provider::AnthropicProvider;
use talos_session::{Session, SessionManager};
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};
use talos_tui::Tui;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::approval::ApprovalPrompt;

const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

/// Permission-aware tool wrapper that checks the permission engine before
/// executing the underlying tool. In interactive mode, [`PermissionDecision::Ask`]
/// triggers a user prompt. In print mode, it defaults to deny.
struct PermissionAwareTool {
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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
    let project_name = workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("default");
    let session = session_manager
        .create_session(project_name)
        .context("failed to create session")?;

    eprintln!("Talos interactive mode (session: {})", session.id);
    eprintln!("Ctrl+C to cancel current turn, double Ctrl+C to exit.\n");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut cancel_token = CancellationToken::new();
    let mut running_task: Option<tokio::task::JoinHandle<Result<()>>> = None;
    let mut first_ctrl_c_time: Option<std::time::Instant> = None;
    let exit_token = CancellationToken::new();

    loop {
        print!("> ");
        io::stdout().flush().context("failed to flush stdout")?;

        tokio::select! {
            biased;
            _ = exit_token.cancelled() => {
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                print!("\r");
                io::stdout().flush().context("failed to flush stdout")?;
                if let Some(handle) = running_task.take() {
                    cancel_token.cancel();
                    handle.abort();
                    let _ = handle.await;
                    eprintln!("Turn cancelled.");
                    cancel_token = CancellationToken::new();
                } else {
                    let now = std::time::Instant::now();
                    if let Some(prev) = first_ctrl_c_time {
                        if now.duration_since(prev) < DOUBLE_CTRL_C_WINDOW {
                            eprintln!("Exiting.");
                            exit_token.cancel();
                            break;
                        }
                    }
                    first_ctrl_c_time = Some(now);
                    eprintln!("Press Ctrl+C again within 2 seconds to exit.");
                }
            }
            line_result = lines.next_line() => {
                let Some(input) = line_result.context("failed to read stdin")? else {
                    break;
                };
                let input = input.trim().to_string();
                first_ctrl_c_time = None;

                if input.is_empty() {
                    continue;
                }

                if let Some(handle) = running_task.take() {
                    handle.abort();
                    let _ = handle.await;
                }

                cancel_token = CancellationToken::new();
                let token = cancel_token.clone();
                let session_clone = session.clone();
                let cli_clone = cli.clone();
                let workspace_clone = workspace_root.clone();

                let handle = tokio::spawn(async move {
                    run_agent_turn(input, cli_clone, workspace_clone, session_clone, token).await
                });
                running_task = Some(handle);
            }
        }

        // Check if running task completed
        if let Some(handle) = running_task.take() {
            if handle.is_finished() {
                match handle.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => eprintln!("Error: {e}"),
                    Err(e) => {
                        if !e.is_cancelled() {
                            eprintln!("Task error: {e}");
                        }
                    }
                }
            } else {
                running_task = Some(handle);
            }
        }
    }

    // Clean shutdown: cancel any running task
    if let Some(handle) = running_task.take() {
        cancel_token.cancel();
        handle.abort();
        let _ = handle.await;
    }

    Ok(())
}

async fn run_agent_turn(
    prompt: String,
    cli: Cli,
    workspace_root: PathBuf,
    session: Session,
    cancel_token: CancellationToken,
) -> Result<()> {
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

    let prompt = if cli.no_context {
        prompt
    } else {
        let context = ContextLoader::new(workspace_root.clone()).load().map_err(|e| anyhow!("{e}"))?;
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
        inner: Arc::new(EditTool::new(workspace_root)),
        approval,
        print_mode: false,
    }));

    let agent = Agent::new(provider, registry);

    let (event_tx, mut event_rx) = broadcast::channel::<AgentEvent>(32);

    let user_msg = Message::User { content: prompt.clone() };
    session
        .append(&user_msg)
        .context("failed to log user message to session")?;

    let mut run_handle = tokio::spawn(async move { agent.run_streaming(prompt, event_tx).await });

    let mut assistant_text = String::new();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                return Ok(());
            }
            event = event_rx.recv() => {
                match event {
                    Ok(AgentEvent::TextDelta { delta }) => {
                        assistant_text.push_str(&delta);
                        print!("{delta}");
                        io::stdout().flush().context("failed to flush stdout")?;
                    }
                    Ok(AgentEvent::ToolCall { call }) => {
                        print!("\r\x1b[0K\r\n[tool: {}]\r\n", call.name);
                        io::stdout().flush().context("failed to flush stdout")?;
                        session
                            .append_event(&AgentEvent::ToolCall { call })
                            .context("failed to log tool call to session")?;
                    }
                    Ok(AgentEvent::ToolResult { result }) => {
                        print!("\r\x1b[0K[tool result: {}]\r\n", if result.is_error { "error" } else { "ok" });
                        io::stdout().flush().context("failed to flush stdout")?;
                        session
                            .append_event(&AgentEvent::ToolResult { result })
                            .context("failed to log tool result to session")?;
                    }
                    Ok(AgentEvent::TurnEnd { .. }) => {
                        if !assistant_text.is_empty() {
                            let assistant_msg = Message::Assistant {
                                content: assistant_text,
                                tool_calls: vec![],
                            };
                            session
                                .append(&assistant_msg)
                                .context("failed to log assistant message to session")?;
                        }
                        return Ok(());
                    }
                    Ok(AgentEvent::Error { message }) => {
                        print!("\r\x1b[0K\r\nError: {message}\r\n");
                        io::stdout().flush().context("failed to flush stdout")?;
                        bail!("{message}");
                    }
                    Ok(AgentEvent::TurnStart) => {}
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        print!("\r\x1b[0K\r\nWarning: dropped {n} event(s) due to slow consumer\r\n");
                        io::stdout().flush().context("failed to flush stdout")?;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        bail!("event channel closed before TurnEnd");
                    }
                }
            }
            run_result = &mut run_handle => {
                match run_result {
                    Ok(Ok(_text)) => {
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        bail!("agent error: {e}");
                    }
                    Err(e) => {
                        if e.is_cancelled() {
                            return Ok(());
                        }
                        bail!("agent task panicked: {e}");
                    }
                }
            }
        }
    }
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

fn parse_provider(s: &str) -> Result<Provider> {
    match s.to_lowercase().as_str() {
        "anthropic" => Ok(Provider::Anthropic),
        "openai" => Ok(Provider::OpenAI),
        other => Err(anyhow!("unknown provider '{other}': supported values are 'anthropic' and 'openai'")),
    }
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
