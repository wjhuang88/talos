use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use talos_core::message::Message;
use talos_core::session::{SessionEvent, SessionOp, TurnCompletionStatus};
use talos_session::{Session, SessionManager};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::event_loop::AppEvent::{
    AgentCompleted, AgentError, AgentTextDelta, AgentToolCall, AgentToolResult, UserInput,
    UserInterrupt,
};
use crate::mode_runtime::request_preview_payload;

const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

pub enum AppEvent {
    UserInput(String),
    UserInterrupt,
    AgentTextDelta(String),
    AgentToolCall(String),
    AgentToolResult(bool),
    AgentCompleted,
    AgentError(String),
    /// Request to fork the current session from a specific entry.
    ForkSession {
        entry_id: Option<String>,
    },
    /// Fork completed with the new session ID.
    ForkCompleted {
        new_session_id: String,
        branch_id: String,
    },
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
    workspace_root: PathBuf,
    session: Session,
    branch_id: Option<String>,
    session_manager: SessionManager,
    /// Clone-able sender for submitting turns to the session actor.
    sq_tx: mpsc::Sender<SessionOp>,
    /// Accumulates assistant text deltas per turn for session logging.
    assistant_text: String,
    /// Tracks whether the current turn's assistant message has already been
    /// persisted to JSONL. Prevents double-writes when both `AgentEvent::TurnEnd`
    /// and `SessionEvent::TurnCompleted::Success` arrive for the same turn.
    assistant_persisted: bool,
}

impl EventLoop {
    pub fn new(
        workspace_root: PathBuf,
        session: Session,
        session_manager: SessionManager,
        handle: talos_core::session::SessionHandle,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let sq_tx = handle.sq_tx;

        // Spawn a single long-lived forwarding task that translates
        // SessionEvent → AppEvent for the lifetime of the EventLoop.
        let mut eq_rx = handle.eq_rx;
        let event_tx_forward = event_tx.clone();
        tokio::spawn(async move {
            while let Some(session_event) = eq_rx.recv().await {
                match session_event {
                    SessionEvent::AgentEvent {
                        event: talos_core::message::AgentEvent::TextDelta { delta },
                    } => {
                        let _ = event_tx_forward.send(AgentTextDelta(delta));
                    }
                    SessionEvent::AgentEvent {
                        event: talos_core::message::AgentEvent::ToolCall { call, .. },
                    } => {
                        let _ = event_tx_forward.send(AgentToolCall(call.name.clone()));
                    }
                    SessionEvent::AgentEvent {
                        event: talos_core::message::AgentEvent::ToolResult { result },
                    } => {
                        let _ = event_tx_forward.send(AgentToolResult(result.is_error));
                    }
                    SessionEvent::AgentEvent {
                        event: talos_core::message::AgentEvent::TurnEnd { .. },
                    } => {
                        let _ = event_tx_forward.send(AgentCompleted);
                    }
                    SessionEvent::AgentEvent {
                        event: talos_core::message::AgentEvent::Error { message },
                    } => {
                        let _ = event_tx_forward.send(AgentError(message));
                    }
                    SessionEvent::TurnCompleted { status, .. } => match status {
                        TurnCompletionStatus::Success { .. } => {
                            let _ = event_tx_forward.send(AgentCompleted);
                        }
                        TurnCompletionStatus::Cancelled => {
                            // Turn was cancelled; L1 transitions back to WaitingForInput
                            // via the existing cancel_token flow.
                        }
                        TurnCompletionStatus::Error { message } => {
                            let _ = event_tx_forward.send(AgentError(message));
                        }
                    },
                    SessionEvent::Error { message } => {
                        let _ = event_tx_forward.send(AgentError(message));
                    }
                    _ => {}
                }
            }
        });

        Self {
            event_tx,
            event_rx,
            state: AppState::WaitingForInput,
            first_ctrl_c_time: None,
            workspace_root,
            session,
            branch_id: None,
            session_manager,
            sq_tx,
            assistant_text: String::new(),
            assistant_persisted: false,
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
                        if tx.send(UserInput(input)).is_err() {
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
                if tx.send(UserInterrupt).is_err() {
                    break;
                }
            }
        });
    }

    fn handle_event(&mut self, event: AppEvent) {
        match (&mut self.state, event) {
            (AppState::WaitingForInput, UserInput(input)) => {
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

            (AppState::WaitingForInput, UserInterrupt) => {
                print!("\r");
                io::stdout().flush().ok();
                let now = Instant::now();
                if let Some(prev) = self.first_ctrl_c_time
                    && now.duration_since(prev) < DOUBLE_CTRL_C_WINDOW
                {
                    eprintln!("Exiting.");
                    self.state = AppState::ShuttingDown;
                    return;
                }
                self.first_ctrl_c_time = Some(now);
                eprintln!("Press Ctrl+C again within 2 seconds to exit.");
            }

            (
                AppState::AgentRunning {
                    cancel_token,
                    task_handle,
                },
                UserInterrupt,
            ) => {
                print!("\r");
                io::stdout().flush().ok();
                cancel_token.cancel();
                // Send interrupt to the session actor.
                let sq_tx = self.sq_tx.clone();
                tokio::spawn(async move {
                    let _ = sq_tx.send(SessionOp::Interrupt).await;
                });
                // Abort the dummy task handle (no-op, but keeps the enum contract).
                task_handle.abort();
                eprintln!("Turn cancelled.");
                self.state = AppState::WaitingForInput;
                self.first_ctrl_c_time = None;
            }

            (AppState::AgentRunning { .. }, AgentTextDelta(delta)) => {
                print!("{delta}");
                io::stdout().flush().ok();
                // Accumulate for session logging.
                self.assistant_text.push_str(&delta);
            }

            (AppState::AgentRunning { .. }, AgentToolCall(name)) => {
                print!("\r\x1b[0K\r\n[tool: {name}]\r\n");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AgentToolResult(is_error)) => {
                let status = if is_error { "error" } else { "ok" };
                print!("[tool result: {status}]\r\n");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AgentCompleted) => {
                println!();
                if !self.assistant_persisted {
                    if !self.assistant_text.is_empty() {
                        let assistant_msg = Message::Assistant {
                            content: std::mem::take(&mut self.assistant_text),
                            tool_calls: vec![],
                            reasoning: None,
                        };
                        if let Err(e) = self.session.append(&assistant_msg) {
                            eprintln!("Warning: failed to log assistant message: {e}");
                        }
                    }
                    if let Err(e) = self.session_manager.update_index(&self.session) {
                        eprintln!("Warning: failed to refresh session index: {e}");
                    }
                    self.assistant_persisted = true;
                }
                self.state = AppState::WaitingForInput;
            }

            (AppState::AgentRunning { .. }, AgentError(msg)) => {
                eprintln!("Error: {msg}");
                self.state = AppState::WaitingForInput;
            }

            (
                AppState::WaitingForInput,
                AppEvent::ForkCompleted {
                    new_session_id,
                    branch_id,
                },
            ) => {
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

            _ => {}
        }
    }

    fn start_agent_turn(&mut self, input: String) {
        let cancel_token = CancellationToken::new();
        self.assistant_text.clear();
        self.assistant_persisted = false;

        // Log user message to session.
        let user_msg = Message::User {
            content: input.clone(),
        };
        if let Err(e) = self.session.append(&user_msg) {
            eprintln!("Warning: failed to log user message: {e}");
        }

        // Submit through session.
        let sq_tx = self.sq_tx.clone();
        let task_handle = tokio::spawn(async move {
            let _ = sq_tx
                .send(match request_preview_payload(&input) {
                    Some(message) => SessionOp::PreviewRequest { message },
                    None => SessionOp::Submit { message: input },
                })
                .await;
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
        let sessions_dir = self.session_manager.sessions_dir().to_path_buf();

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
                    None => entries
                        .last()
                        .expect("entries checked non-empty above")
                        .id
                        .clone(),
                };

                let mut forked = session.clone();
                let branch_id = forked.fork(&fork_from_id)?;

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
                    let _ = index.record_fork(
                        &session.id.to_string(),
                        &new_id.to_string(),
                        &fork_from_id,
                    );
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
                    let _ = event_tx.send(AppEvent::ForkCompleted {
                        new_session_id,
                        branch_id,
                    });
                }
                Err(e) => {
                    let _ = event_tx.send(AgentError(format!("Fork failed: {e}")));
                }
            }
        });
    }

    async fn shutdown(&mut self) {
        // Send shutdown to the session actor.
        let _ = self.sq_tx.send(SessionOp::Shutdown).await;

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
