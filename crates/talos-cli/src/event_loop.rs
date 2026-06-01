use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use talos_agent::context::ContextLoader;
use talos_agent::Agent;
use talos_config::Config;
use talos_core::message::{AgentEvent, Message};
use talos_core::tool::ToolRegistry;
use talos_permission::PermissionEngine;
use talos_plugin::HookRegistry;
use talos_session::{Session, SessionManager};
use talos_tools::{BashTool, EditTool, ReadTool, WriteTool};
use talos_tui::SkillInfo;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::approval::ApprovalPrompt;
use crate::{build_provider, parse_provider, Cli, PermissionAwareTool};

const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

/// User's choice when resolving an approval prompt.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalChoice {
    /// Approve this tool call once.
    ApproveOnce,
    /// Always approve this tool (add a rule).
    AlwaysApprove,
    /// Deny the tool call.
    Deny,
}

#[allow(dead_code)]
pub enum AppEvent {
    UserInput(String),
    UserInterrupt,
    AgentTextDelta(String),
    AgentToolCall(String),
    AgentToolResult(bool),
    AgentCompleted,
    AgentError(String),
    /// TUI requests approval for a tool call.
    ApprovalRequested {
        tool_name: String,
        arguments: String,
    },
    /// TUI resolved an approval prompt.
    ApprovalResolved(ApprovalChoice),
    /// Request to fork the current session from a specific entry.
    ForkSession {
        entry_id: Option<String>,
    },
    /// Fork completed with the new session ID.
    ForkCompleted {
        new_session_id: String,
        branch_id: String,
    },
    /// Toggle the skill sidebar visibility.
    ToggleSkillSidebar,
    /// Skills have been loaded or updated.
    SkillsUpdated(Vec<SkillInfo>),
}

pub enum AppState {
    WaitingForInput,
    AgentRunning {
        cancel_token: CancellationToken,
        task_handle: JoinHandle<()>,
    },
    ShuttingDown,
}

pub struct EventLoop {
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    state: AppState,
    first_ctrl_c_time: Option<Instant>,
    cli: Cli,
    workspace_root: PathBuf,
    session: Session,
    branch_id: Option<String>,
    session_manager: SessionManager,
    hook_registry: Arc<HookRegistry>,
}

impl EventLoop {
    pub fn new(
        cli: Cli,
        workspace_root: PathBuf,
        session: Session,
        session_manager: SessionManager,
        hook_registry: Arc<HookRegistry>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            event_tx,
            event_rx,
            state: AppState::WaitingForInput,
            first_ctrl_c_time: None,
            cli,
            workspace_root,
            session,
            branch_id: None,
            session_manager,
            hook_registry,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let branch_info = self
            .branch_id
            .as_ref()
            .map(|b| format!(", branch: {b}"))
            .unwrap_or_default();
        eprintln!(
            "Talos interactive mode (session: {}{branch_info})",
            self.session.id
        );
        eprintln!("Ctrl+C to cancel current turn, double Ctrl+C to exit.\n");

        self.spawn_stdin_reader();
        self.spawn_signal_handler();
        self.render();

        loop {
            let event = match self.event_rx.recv().await {
                Some(e) => e,
                None => break,
            };

            self.handle_event(event);
            self.render();

            if matches!(self.state, AppState::ShuttingDown) {
                break;
            }
        }

        self.shutdown().await;
        Ok(())
    }

    fn spawn_stdin_reader(&self) {
        let tx = self.event_tx.clone();
        std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut line = String::new();
            loop {
                line.clear();
                match stdin.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let input = line.trim().to_string();
                        if tx.send(AppEvent::UserInput(input)).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    fn spawn_signal_handler(&self) {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::signal::ctrl_c().await.ok();
                if tx.send(AppEvent::UserInterrupt).is_err() {
                    break;
                }
            }
        });
    }

    fn handle_event(&mut self, event: AppEvent) {
        match (&mut self.state, event) {
            (AppState::WaitingForInput, AppEvent::UserInput(input)) => {
                if input.is_empty() {
                    return;
                }
                self.first_ctrl_c_time = None;

                if let Some(rest) = input.strip_prefix("/fork") {
                    self.handle_fork_command(rest.trim());
                    return;
                }

                self.start_agent_turn(input);
            }

            (AppState::WaitingForInput, AppEvent::UserInterrupt) => {
                print!("\r");
                io::stdout().flush().ok();
                let now = Instant::now();
                if let Some(prev) = self.first_ctrl_c_time {
                    if now.duration_since(prev) < DOUBLE_CTRL_C_WINDOW {
                        eprintln!("Exiting.");
                        self.state = AppState::ShuttingDown;
                        return;
                    }
                }
                self.first_ctrl_c_time = Some(now);
                eprintln!("Press Ctrl+C again within 2 seconds to exit.");
            }

            (
                AppState::AgentRunning {
                    cancel_token,
                    task_handle,
                },
                AppEvent::UserInterrupt,
            ) => {
                print!("\r");
                io::stdout().flush().ok();
                cancel_token.cancel();
                task_handle.abort();
                eprintln!("Turn cancelled.");
                self.state = AppState::WaitingForInput;
                self.first_ctrl_c_time = None;
            }

            (AppState::AgentRunning { .. }, AppEvent::AgentTextDelta(delta)) => {
                print!("{delta}");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AppEvent::AgentToolCall(name)) => {
                print!("\r\x1b[0K\r\n[tool: {name}]\r\n");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AppEvent::AgentToolResult(is_error)) => {
                let status = if is_error { "error" } else { "ok" };
                print!("[tool result: {status}]\r\n");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AppEvent::AgentCompleted) => {
                println!();
                self.state = AppState::WaitingForInput;
            }

            (AppState::AgentRunning { .. }, AppEvent::AgentError(msg)) => {
                eprintln!("Error: {msg}");
                self.state = AppState::WaitingForInput;
            }

            (AppState::WaitingForInput, AppEvent::ForkCompleted { new_session_id, branch_id }) => {
                self.branch_id = Some(branch_id.clone());
                // Reload the fork from disk so subsequent turns append to the new file.
                // Without this, `self.session` still points at the source session's id/path
                // and tool calls would be logged into the source JSONL, not the fork.
                if let Ok(new_uuid) = uuid::Uuid::parse_str(&new_session_id) {
                    match self.session_manager.get_session(&new_uuid) {
                        Ok(forked) => {
                            self.session = forked;
                            eprintln!(
                                "Fork completed. New session: {new_session_id}, branch: {branch_id}"
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "Fork completed but could not reload fork session: {e}. \
                                 Falling back to in-memory id; subsequent turns may write to the \
                                 source session."
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "Fork completed. New session: {new_session_id}, branch: {branch_id} \
                         (invalid uuid; not reloading)"
                    );
                }
            }

            (AppState::WaitingForInput, AppEvent::ForkSession { entry_id }) => {
                self.handle_fork_session(entry_id);
            }

            (_, AppEvent::ToggleSkillSidebar) => {
                // Forwarded to TUI layer; CLI event loop acknowledges the event.
            }

            (_, AppEvent::SkillsUpdated(skills)) => {
                eprintln!("Skills updated: {} loaded", skills.len());
            }

            _ => {}
        }
    }

    fn start_agent_turn(&mut self, input: String) {
        let cancel_token = CancellationToken::new();
        let token = cancel_token.clone();
        let session = self.session.clone();
        let cli = self.cli.clone();
        let workspace = self.workspace_root.clone();
        let event_tx = self.event_tx.clone();
        let session_manager = self.session_manager.clone();
        let hook_registry = self.hook_registry.clone();

        let task_handle = tokio::spawn(async move {
            let result = run_agent_turn_inner(
                input,
                cli,
                workspace,
                session,
                session_manager,
                token,
                event_tx.clone(),
                hook_registry,
            )
            .await;
            if let Err(e) = result {
                let _ = event_tx.send(AppEvent::AgentError(e.to_string()));
            }
        });

        self.state = AppState::AgentRunning {
            cancel_token,
            task_handle,
        };
    }

    fn render(&self) {
        if matches!(self.state, AppState::WaitingForInput) {
            print!("> ");
            io::stdout().flush().ok();
        }
    }

    fn handle_fork_command(&mut self, entry_id: &str) {
        let entry_id = if entry_id.is_empty() {
            None
        } else {
            Some(entry_id.to_string())
        };

        let _ = self.event_tx.send(AppEvent::ForkSession { entry_id });
    }

    fn handle_fork_session(&mut self, entry_id: Option<String>) {
        use talos_session::SessionIndex;

        let session = self.session.clone();
        let _workspace = self.workspace_root.clone();
        let event_tx = self.event_tx.clone();

        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<(String, String)> {
                let entries = session.read_entries()?;
                if entries.is_empty() {
                    anyhow::bail!("cannot fork an empty session");
                }

                let fork_from_id = match &entry_id {
                    Some(id) => {
                        if entries.iter().any(|e| e.id == *id) {
                            id.clone()
                        } else {
                            anyhow::bail!("entry not found: {id}");
                        }
                    }
                    None => entries.last().expect("entries checked non-empty above").id.clone(),
                };

                let mut forked = session.clone();
                let branch_id = forked.fork(&fork_from_id)?;

                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let sessions_dir = std::path::PathBuf::from(home).join(".talos").join("sessions");
                let project_dir = sessions_dir.join(&forked.project);
                std::fs::create_dir_all(&project_dir)?;

                let new_id = uuid::Uuid::new_v4();
                let new_file_path = project_dir.join(format!("{new_id}.jsonl"));

                let entries_to_copy = if let Some(branch) = forked.get_branch(&branch_id) {
                    branch.entries.clone()
                } else {
                    entries
                };

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&new_file_path)?;

                for entry in &entries_to_copy {
                    let line = serde_json::to_string(entry)?;
                    std::io::Write::write_all(&mut file, line.as_bytes())?;
                    std::io::Write::write_all(&mut file, b"\n")?;
                }

                if let Ok(mut index) = SessionIndex::new(&sessions_dir.join("index.db")) {
                    let _ = index.init_schema();
                    let _ = index.record_fork(&session.id.to_string(), &new_id.to_string(), &fork_from_id);
                    // Re-stamp `forked` with the new identity BEFORE indexing so the
                    // SQLite FTS5 index points at the fork's id/file_path/branch_id,
                    // not the source's. Without this, search and list_recent would
                    // surface the source under the fork's UUID.
                    forked.with_fork_identity(new_id, new_file_path.clone(), branch_id.clone());
                    let _ = index.index_session(&forked);
                }

                Ok((new_id.to_string(), branch_id))
            })();

            match result {
                Ok((new_session_id, branch_id)) => {
                    let _ = event_tx.send(AppEvent::ForkCompleted { new_session_id, branch_id });
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::AgentError(format!("Fork failed: {e}")));
                }
            }
        });
    }

    async fn shutdown(&mut self) {
        if let AppState::AgentRunning {
            cancel_token,
            task_handle,
        } = std::mem::replace(&mut self.state, AppState::ShuttingDown)
        {
            cancel_token.cancel();
            task_handle.abort();
            let _ = task_handle.await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_agent_turn_inner(
    prompt: String,
    cli: Cli,
    workspace_root: PathBuf,
    session: Session,
    session_manager: SessionManager,
    cancel_token: CancellationToken,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    hook_registry: Arc<HookRegistry>,
) -> Result<()> {
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
        String::new()
    } else {
        config.api_key().map_err(|e| anyhow::anyhow!("{e}"))?
    };

    let prompt = if cli.no_context {
        prompt
    } else {
        let context = ContextLoader::new(workspace_root.clone())
            .load()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if context.is_empty() {
            prompt
        } else {
            format!("{context}\n\n{prompt}")
        }
    };

    let provider = build_provider(&config, &api_key, cli.mock);

    let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

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
        approval,
        print_mode: false,
    }));

    let agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(PermissionEngine::new())),
        None,
        workspace_root.clone(),
        hook_registry,
    );

    let (agent_event_tx, mut agent_event_rx) = broadcast::channel::<AgentEvent>(32);

    let user_msg = Message::User {
        content: prompt.clone(),
    };
    session
        .append(&user_msg)
        .context("failed to log user message to session")?;

    let mut run_handle =
        tokio::spawn(async move { agent.run_streaming(prompt, agent_event_tx).await });

    let mut assistant_text = String::new();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                return Ok(());
            }
            event = agent_event_rx.recv() => {
                match event {
                    Ok(AgentEvent::TextDelta { delta }) => {
                        assistant_text.push_str(&delta);
                        let _ = event_tx.send(AppEvent::AgentTextDelta(delta));
                    }
                    Ok(AgentEvent::ToolCall { call, .. }) => {
                        let _ = event_tx.send(AppEvent::AgentToolCall(call.name.clone()));
                        session
                            .append_event(&AgentEvent::ToolCall {
                                call,
                                provenance: Default::default(),
                            })
                            .context("failed to log tool call to session")?;
                    }
                    Ok(AgentEvent::ToolResult { result }) => {
                        let is_error = result.is_error;
                        let _ = event_tx.send(AppEvent::AgentToolResult(is_error));
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
                        if let Err(e) = session_manager.update_index(&session) {
                            eprintln!("Warning: failed to refresh session index: {e}");
                        }
                        let _ = event_tx.send(AppEvent::AgentCompleted);
                        return Ok(());
                    }
                    Ok(AgentEvent::Error { message }) => {
                        let _ = event_tx.send(AppEvent::AgentError(message.clone()));
                        bail!("{message}");
                    }
                    Ok(AgentEvent::TurnStart) => {}
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Warning: dropped {n} event(s) due to slow consumer");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        bail!("event channel closed before TurnEnd");
                    }
                }
            }
            run_result = &mut run_handle => {
                match run_result {
                    Ok(Ok(_)) => {
                        if let Err(e) = session_manager.update_index(&session) {
                            eprintln!("Warning: failed to refresh session index: {e}");
                        }
                        let _ = event_tx.send(AppEvent::AgentCompleted);
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
