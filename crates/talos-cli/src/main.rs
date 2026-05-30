//! Talos CLI — primary command-line interface.
//!
//! Supports print mode (`-p`) for streaming LLM responses to stdout,
//! interactive mode for conversational agent sessions, and optional
//! stdin pipe input and CLI argument overrides.

use std::io::{self, IsTerminal, Read};
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
use tokio::io::{AsyncBufReadExt, BufReader};
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

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    let mut cancel_token = CancellationToken::new();
    let mut running_task: Option<tokio::task::JoinHandle<Result<()>>> = None;
    let mut first_ctrl_c_time: Option<std::time::Instant> = None;

    loop {
        print!("> ");
        io::stdout().flush().context("failed to flush stdout")?;

        tokio::select! {
            biased;
            // Ctrl+C takes priority
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
                            eprintln!("Exiting.");
                            return Ok(());
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

    Ok(())
}

/// Normalize C0 control characters to their Ctrl+letter equivalent.
/// Terminals in raw mode may send raw C0 codes (e.g., \x03 for Ctrl+C).
fn normalize_key(code: KeyCode, modifiers: KeyModifiers) -> (KeyCode, KeyModifiers) {
    if modifiers.is_empty() {
        if let KeyCode::Char(ch) = code {
            if let Some(ctrl) = c0_to_ctrl(ch) {
                return (KeyCode::Char(ctrl), KeyModifiers::CONTROL);
            }
        }
    }
    (code, modifiers)
}

fn c0_to_ctrl(ch: char) -> Option<char> {
    match u32::from(ch) {
        0x01..=0x1a => char::from_u32(0x60 + u32::from(ch)), // 0x01→'a', 0x03→'c'
        _ => None,
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

    // ── c0_to_ctrl ──────────────────────────────────────────────

    #[test]
    fn c0_to_ctrl_maps_a_to_z() {
        assert_eq!(c0_to_ctrl('\x01'), Some('a'));
        assert_eq!(c0_to_ctrl('\x03'), Some('c'));
        assert_eq!(c0_to_ctrl('\x1a'), Some('z'));
    }

    #[test]
    fn c0_to_ctrl_returns_none_for_printable() {
        assert_eq!(c0_to_ctrl('a'), None);
        assert_eq!(c0_to_ctrl(' '), None);
        assert_eq!(c0_to_ctrl('\x7f'), None);
    }

    #[test]
    fn c0_to_ctrl_maps_newline_to_ctrl_j() {
        assert_eq!(c0_to_ctrl('\n'), Some('j'));
    }

    // ── normalize_key ───────────────────────────────────────────

    #[test]
    fn normalize_key_passes_through_with_modifiers() {
        assert_eq!(
            normalize_key(KeyCode::Char('c'), KeyModifiers::CONTROL),
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
        );
    }

    #[test]
    fn normalize_key_passes_through_non_char() {
        assert_eq!(
            normalize_key(KeyCode::Enter, KeyModifiers::NONE),
            (KeyCode::Enter, KeyModifiers::NONE)
        );
    }

    #[test]
    fn normalize_key_converts_c0_to_ctrl() {
        assert_eq!(
            normalize_key(KeyCode::Char('\x03'), KeyModifiers::NONE),
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
        );
        assert_eq!(
            normalize_key(KeyCode::Char('\x01'), KeyModifiers::NONE),
            (KeyCode::Char('a'), KeyModifiers::CONTROL)
        );
    }

    #[test]
    fn normalize_key_leaves_printable_unchanged() {
        assert_eq!(
            normalize_key(KeyCode::Char('x'), KeyModifiers::NONE),
            (KeyCode::Char('x'), KeyModifiers::NONE)
        );
    }

    // ── InteractiveState input handling ─────────────────────────

    fn make_state() -> InteractiveState {
        InteractiveState {
            session: make_test_session(),
            cli: Cli { prompt: None, print: false, model: None, provider: None },
            workspace_root: PathBuf::from("/tmp"),
            cancel_token: CancellationToken::new(),
            running_task: None,
            first_ctrl_c_time: None,
            input_buffer: String::new(),
        }
    }

    fn make_test_session() -> Session {
        let dir = tempfile::tempdir().unwrap();
        let mgr = SessionManager::with_dir(dir.path().to_path_buf());
        mgr.create_session("test").unwrap()
    }

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(crossterm::event::KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    #[test]
    fn typing_appends_to_buffer() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('h'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('i'), KeyModifiers::NONE)))).unwrap();
        assert_eq!(s.input_buffer, "hi");
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('a'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('b'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Backspace, KeyModifiers::NONE)))).unwrap();
        assert_eq!(s.input_buffer, "a");
    }

    #[test]
    fn backspace_on_empty_does_not_panic() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Backspace, KeyModifiers::NONE)))).unwrap();
        assert_eq!(s.input_buffer, "");
    }

    #[test]
    fn ctrl_c_clears_buffer() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('h'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('i'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        assert_eq!(s.input_buffer, "");
    }

    #[test]
    fn ctrl_c_single_press_sets_timer() {
        let mut s = make_state();
        let action = s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        assert!(matches!(action, EventAction::Continue));
        assert!(s.first_ctrl_c_time.is_some());
    }

    #[test]
    fn ctrl_c_double_press_exits() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        // Second press within 2 seconds
        let action = s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        assert!(matches!(action, EventAction::Exit));
    }

    #[test]
    fn ctrl_c_double_press_after_timeout_resets() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        // Simulate timeout by setting first_ctrl_c_time to the past
        s.first_ctrl_c_time = Some(std::time::Instant::now() - Duration::from_secs(3));
        let action = s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        assert!(matches!(action, EventAction::Continue));
        // Timer should be refreshed
        assert!(s.first_ctrl_c_time.is_some());
    }

    #[tokio::test]
    async fn enter_submits_non_empty_input() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('h'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('i'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Enter, KeyModifiers::NONE)))).unwrap();
        assert_eq!(s.input_buffer, "");
        assert!(s.running_task.is_some());
    }

    #[tokio::test]
    async fn enter_on_empty_does_not_spawn_task() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Enter, KeyModifiers::NONE)))).unwrap();
        assert!(s.running_task.is_none());
    }

    #[tokio::test]
    async fn enter_clears_ctrl_c_timer() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap();
        assert!(s.first_ctrl_c_time.is_some());
        s.handle_event(Some(Ok(key_event(KeyCode::Char('h'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Enter, KeyModifiers::NONE)))).unwrap();
        assert!(s.first_ctrl_c_time.is_none());
    }

    #[test]
    fn arrow_keys_ignored() {
        let mut s = make_state();
        s.handle_event(Some(Ok(key_event(KeyCode::Char('a'), KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Left, KeyModifiers::NONE)))).unwrap();
        s.handle_event(Some(Ok(key_event(KeyCode::Right, KeyModifiers::NONE)))).unwrap();
        assert_eq!(s.input_buffer, "a");
    }

    #[test]
    fn ctrl_d_not_treated_as_printable() {
        let mut s = make_state();
        // Ctrl+D normalizes to Ctrl+d, which has CONTROL modifier
        let (code, modifiers) = normalize_key(KeyCode::Char('\x04'), KeyModifiers::NONE);
        assert_eq!(code, KeyCode::Char('d'));
        assert_eq!(modifiers, KeyModifiers::CONTROL);
        // Should NOT match (KeyCode::Char('d'), KeyModifiers::NONE)
        s.handle_event(Some(Ok(key_event(code, modifiers)))).unwrap();
        assert_eq!(s.input_buffer, "");
    }

    #[test]
    fn non_key_event_ignored() {
        let mut s = make_state();
        s.handle_event(Some(Ok(Event::Resize(80, 24)))).unwrap();
        assert_eq!(s.input_buffer, "");
    }

    #[test]
    fn release_event_ignored() {
        let mut s = make_state();
        s.handle_event(Some(Ok(Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::empty(),
        })))).unwrap();
        assert_eq!(s.input_buffer, "");
    }
}
