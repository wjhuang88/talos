//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! interactive mode for conversational agent sessions, and optional
//! stdin pipe input and CLI argument overrides.

use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use talos_agent::Agent;
use talos_config::{Config, Provider};
use talos_core::message::{AgentEvent, Message};
use talos_core::tool::ToolRegistry;
use talos_provider::AnthropicProvider;
use talos_session::{Session, SessionManager};
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.print {
        return run_print_mode(cli).await;
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

    let provider = Arc::new(AnthropicProvider::new(api_key, &config.model));
    let agent = Agent::new(provider, ToolRegistry::new());

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

    let mut stdout = io::stdout().lock();
    let mut cancel_token = CancellationToken::new();
    let mut running_task: Option<tokio::task::JoinHandle<Result<()>>> = None;
    let mut first_ctrl_c_time: Option<std::time::Instant> = None;

    loop {
        stdout
            .write_all(b"> ")
            .context("failed to write prompt")?;
        stdout.flush().context("failed to flush stdout")?;

        let readline_handle = tokio::task::spawn_blocking(|| {
            let mut line = String::new();
            let result = io::stdin().read_line(&mut line);
            (line, result)
        });

        tokio::select! {
            result = readline_handle => {
                let (line, read_result) = result.context("readline task panicked")?;
                let _ = read_result.context("failed to read from stdin")?;
                let input = line.trim().to_string();

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
            _ = tokio::signal::ctrl_c() => {
                if let Some(handle) = running_task.take() {
                    cancel_token.cancel();
                    handle.abort();
                    let _ = handle.await;
                    eprintln!("\nTurn cancelled.");
                    cancel_token = CancellationToken::new();
                } else {
                    let now = std::time::Instant::now();
                    if let Some(prev) = first_ctrl_c_time {
                        if now.duration_since(prev) < DOUBLE_CTRL_C_WINDOW {
                            eprintln!("\nExiting.");
                            return Ok(());
                        }
                    }
                    first_ctrl_c_time = Some(now);
                    eprintln!("\nPress Ctrl+C again within 2 seconds to exit.");
                }
            }
        }

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
                println!();
            } else {
                running_task = Some(handle);
            }
        }
    }
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

    let provider = Arc::new(AnthropicProvider::new(api_key, &config.model));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new(workspace_root.clone())));
    registry.register(Arc::new(ReadTool::new(workspace_root.clone())));
    registry.register(Arc::new(WriteTool::new(workspace_root.clone())));
    registry.register(Arc::new(EditTool::new(workspace_root)));

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
                        eprintln!("\n[tool: {}]", call.name);
                        session
                            .append_event(&AgentEvent::ToolCall { call })
                            .context("failed to log tool call to session")?;
                    }
                    Ok(AgentEvent::ToolResult { result }) => {
                        eprintln!("[tool result: {}]", if result.is_error { "error" } else { "ok" });
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
                        eprintln!("\nError: {message}");
                        bail!("{message}");
                    }
                    Ok(AgentEvent::TurnStart) => {}
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("\nWarning: dropped {n} event(s) due to slow consumer");
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
