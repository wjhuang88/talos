use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
#[cfg(test)]
use talos_agent::permission_pipeline::PermissionBinding;
use talos_core::background_job::BackgroundJobTerminalSummary;
use talos_core::session::{SessionEvent, SessionOp, TurnCompletionStatus, TurnEventPayload};
use talos_session::{Session, SessionManager};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::approval::{ApprovalPrompt, TerminalApprovalRequest};
use crate::background_projection::{format_background_result, format_background_terminal};
use crate::event_loop::AppEvent::{
    AgentCompleted, AgentError, AgentTextDelta, AgentToolCall, AgentToolResult, ApprovalRequested,
    BackgroundJobTerminal, UserInput, UserInterrupt,
};
use crate::mode_runtime::request_preview_payload;

const DOUBLE_CTRL_C_WINDOW: Duration = Duration::from_secs(2);

pub(crate) enum AppEvent {
    UserInput(String),
    UserInterrupt,
    ApprovalRequested(TerminalApprovalRequest),
    AgentTextDelta(String),
    AgentToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    AgentToolResult {
        tool_use_id: String,
        is_error: bool,
        content: String,
    },
    BackgroundJobTerminal(BackgroundJobTerminalSummary),
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

pub(crate) enum AppState {
    WaitingForInput,
    AgentRunning {
        cancel_token: CancellationToken,
        task_handle: JoinHandle<()>,
    },
    ShuttingDown,
}

pub(crate) struct EventLoop {
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
    approval_queue: VecDeque<TerminalApprovalRequest>,
    displayed_approval: Option<uuid::Uuid>,
    approval_rollover_barrier: bool,
    background_tools: HashMap<String, String>,
}

impl EventLoop {
    pub(crate) fn new(
        workspace_root: PathBuf,
        session: Session,
        session_manager: SessionManager,
        handle: talos_core::session::SessionHandle,
        mut approval_rx: mpsc::UnboundedReceiver<TerminalApprovalRequest>,
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
                    SessionEvent::TurnEvent {
                        payload:
                            TurnEventPayload::Progress {
                                event: talos_core::message::AgentEvent::TextDelta { delta },
                            },
                        ..
                    } => {
                        let _ = event_tx_forward.send(AgentTextDelta(delta));
                    }
                    SessionEvent::TurnEvent {
                        payload:
                            TurnEventPayload::Progress {
                                event: talos_core::message::AgentEvent::ToolCall { call, .. },
                            },
                        ..
                    } => {
                        let _ = event_tx_forward.send(AgentToolCall {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: call.input.clone(),
                        });
                    }
                    SessionEvent::TurnEvent {
                        payload:
                            TurnEventPayload::Progress {
                                event: talos_core::message::AgentEvent::ToolResult { result },
                            },
                        ..
                    } => {
                        let _ = event_tx_forward.send(AgentToolResult {
                            tool_use_id: result.tool_use_id.clone(),
                            is_error: result.is_error,
                            content: result.content.clone(),
                        });
                    }
                    SessionEvent::TurnEvent {
                        payload: TurnEventPayload::Completed { status },
                        ..
                    } => match status {
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
                    SessionEvent::BackgroundJobTerminal { summary, .. } => {
                        let _ = event_tx_forward.send(BackgroundJobTerminal(summary));
                    }
                    SessionEvent::Error { message } => {
                        let _ = event_tx_forward.send(AgentError(message));
                    }
                    _ => {}
                }
            }
        });
        let event_tx_approval = event_tx.clone();
        tokio::spawn(async move {
            while let Some(request) = approval_rx.recv().await {
                if event_tx_approval.send(ApprovalRequested(request)).is_err() {
                    break;
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
            approval_queue: VecDeque::new(),
            displayed_approval: None,
            approval_rollover_barrier: false,
            background_tools: HashMap::new(),
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
        let event = match event {
            ApprovalRequested(request) => {
                if matches!(self.state, AppState::AgentRunning { .. }) {
                    self.approval_queue.push_back(request);
                    self.render_active_approval();
                } else {
                    let _ = request.response.send(talos_core::ApprovalChoice::Deny);
                }
                return;
            }
            UserInput(input)
                if matches!(self.state, AppState::AgentRunning { .. })
                    && (self.displayed_approval.is_some() || !self.approval_queue.is_empty()) =>
            {
                match resolve_displayed_approval(
                    &mut self.approval_queue,
                    &mut self.displayed_approval,
                    &mut self.approval_rollover_barrier,
                    &input,
                ) {
                    ApprovalInputOutcome::Resolved => {}
                    ApprovalInputOutcome::Invalid => {
                        eprintln!("Invalid input. Please enter y, a, or n.");
                        self.displayed_approval = None;
                    }
                    ApprovalInputOutcome::Stale => {
                        eprintln!("Approval request expired; review the current request.");
                    }
                }
                self.render_active_approval();
                return;
            }
            event => {
                self.render_active_approval();
                event
            }
        };
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
                self.deny_pending_approvals();
                self.background_tools.clear();
                eprintln!("Turn cancelled.");
                self.state = AppState::WaitingForInput;
                self.first_ctrl_c_time = None;
            }

            (AppState::AgentRunning { .. }, AgentTextDelta(delta)) => {
                print!("{delta}");
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, AgentToolCall { id, name, input }) => {
                track_background_tool(&mut self.background_tools, id, &name, &input);
                print!("\r\x1b[0K\r\n[tool: {name}]\r\n");
                io::stdout().flush().ok();
            }

            (
                AppState::AgentRunning { .. },
                AgentToolResult {
                    tool_use_id,
                    is_error,
                    content,
                },
            ) => {
                let status = if is_error { "error" } else { "ok" };
                print!("[tool result: {status}]\r\n");
                if let Some(tool_name) = self.background_tools.remove(&tool_use_id)
                    && let Some(summary) = format_background_result(&tool_name, &content)
                {
                    eprintln!("{summary}");
                }
                io::stdout().flush().ok();
            }

            (AppState::AgentRunning { .. }, BackgroundJobTerminal(summary))
            | (AppState::WaitingForInput, BackgroundJobTerminal(summary)) => {
                eprintln!("{}", format_background_terminal(&summary));
            }

            (AppState::AgentRunning { .. }, AgentCompleted) => {
                self.deny_pending_approvals();
                self.background_tools.clear();
                println!();
                if let Err(e) = self.session_manager.update_index(&self.session) {
                    eprintln!("Warning: failed to refresh session index: {e}");
                }
                self.state = AppState::WaitingForInput;
            }

            (AppState::AgentRunning { .. }, AgentError(msg)) => {
                self.deny_pending_approvals();
                self.background_tools.clear();
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

    fn deny_pending_approvals(&mut self) {
        self.displayed_approval = None;
        self.approval_rollover_barrier = false;
        while let Some(request) = self.approval_queue.pop_front() {
            let _ = request.response.send(talos_core::ApprovalChoice::Deny);
        }
    }

    fn render_active_approval(&mut self) {
        if select_active_approval(
            &mut self.approval_queue,
            &mut self.displayed_approval,
            &mut self.approval_rollover_barrier,
        ) {
            return;
        }
        let Some(request) = self.approval_queue.front() else {
            return;
        };
        if ApprovalPrompt::render_choice_prompt(
            &request.request.tool_name,
            &request.request.arguments,
            &request.request.preview,
        )
        .is_err()
            && let Some(request) = self.approval_queue.pop_front()
        {
            self.displayed_approval = None;
            let _ = request.response.send(talos_core::ApprovalChoice::Deny);
        }
    }

    fn start_agent_turn(&mut self, input: String) {
        self.background_tools.clear();
        let cancel_token = CancellationToken::new();
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
                let ext = session.file_extension();
                let new_file_path = project_dir.join(format!("{new_id}.{ext}"));

                let entries_to_copy = if let Some(branch) = forked.get_branch(&branch_id) {
                    branch.entries.clone()
                } else {
                    entries
                };

                if ext == "tlog" {
                    let store = talos_session::CompactTextSessionStore;
                    use talos_session::SessionStore;
                    for entry in &entries_to_copy {
                        store.append_entry(&new_file_path, entry)?;
                    }
                } else {
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

fn is_background_tool_call(name: &str, input: &serde_json::Value) -> bool {
    name == "process"
        || matches!(name, "bash" | "exec")
            && input.get("background").and_then(serde_json::Value::as_bool) == Some(true)
}

fn track_background_tool(
    tools: &mut HashMap<String, String>,
    id: String,
    name: &str,
    input: &serde_json::Value,
) {
    if is_background_tool_call(name, input) {
        tools.insert(id, name.to_owned());
    }
}

#[cfg(test)]
mod background_pairing_tests {
    use super::*;

    #[test]
    fn interleaved_calls_remain_paired_by_tool_use_id() {
        let mut tools = HashMap::new();
        track_background_tool(
            &mut tools,
            "background-1".to_owned(),
            "bash",
            &serde_json::json!({"background": true}),
        );
        track_background_tool(
            &mut tools,
            "foreground-1".to_owned(),
            "read_file",
            &serde_json::json!({"path": "README.md"}),
        );
        track_background_tool(
            &mut tools,
            "process-1".to_owned(),
            "process",
            &serde_json::json!({"action": "read", "job_id": "job_1"}),
        );

        assert!(tools.remove("foreground-1").is_none());
        assert_eq!(tools.remove("background-1").as_deref(), Some("bash"));
        assert_eq!(tools.remove("process-1").as_deref(), Some("process"));
        assert!(tools.is_empty());
    }

    #[test]
    fn foreground_shell_calls_are_not_tracked() {
        assert!(!is_background_tool_call(
            "bash",
            &serde_json::json!({"command": "printf done"}),
        ));
        assert!(!is_background_tool_call(
            "exec",
            &serde_json::json!({"argv": ["printf", "done"], "background": false}),
        ));
    }
}

enum ApprovalInputOutcome {
    Resolved,
    Invalid,
    Stale,
}

fn resolve_displayed_approval(
    queue: &mut VecDeque<TerminalApprovalRequest>,
    displayed: &mut Option<uuid::Uuid>,
    rollover_barrier: &mut bool,
    input: &str,
) -> ApprovalInputOutcome {
    if *rollover_barrier {
        *rollover_barrier = false;
        return ApprovalInputOutcome::Stale;
    }
    let Some(displayed_id) = *displayed else {
        return ApprovalInputOutcome::Stale;
    };
    let current_is_displayed = queue
        .front()
        .is_some_and(|request| request.id == displayed_id && !request.response.is_closed());
    if !current_is_displayed {
        *displayed = None;
        return ApprovalInputOutcome::Stale;
    }
    let choice = match input.trim() {
        "y" | "Y" => talos_core::ApprovalChoice::ApproveOnce,
        "a" | "A" => talos_core::ApprovalChoice::AlwaysApprove,
        "n" | "N" => talos_core::ApprovalChoice::Deny,
        _ => return ApprovalInputOutcome::Invalid,
    };
    let request = queue.pop_front().expect("displayed request is present");
    *displayed = None;
    let _ = request.response.send(choice);
    ApprovalInputOutcome::Resolved
}

fn select_active_approval(
    queue: &mut VecDeque<TerminalApprovalRequest>,
    displayed: &mut Option<uuid::Uuid>,
    rollover_barrier: &mut bool,
) -> bool {
    let displayed_is_current = displayed.is_some_and(|id| {
        queue
            .front()
            .is_some_and(|request| request.id == id && !request.response.is_closed())
    });
    if displayed_is_current {
        return true;
    }
    let replaced_displayed = displayed.take().is_some();
    if replaced_displayed {
        *rollover_barrier = true;
    }
    queue.retain(|request| !request.response.is_closed());
    if let Some(request) = queue.front() {
        *displayed = Some(request.id);
    }
    false
}

#[cfg(test)]
mod approval_queue_tests {
    use super::*;
    use talos_agent::permission_pipeline::PermissionApprovalRequest;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
    use talos_permission::{
        InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
        PermissionMode, PermissionRequest, PermissionSessionState,
    };

    fn queued_request() -> (
        TerminalApprovalRequest,
        tokio::sync::oneshot::Receiver<talos_core::ApprovalChoice>,
    ) {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"fixture").expect("fixture");
        let target_text = target.display().to_string();
        let input = serde_json::json!({"path": target_text.clone()});
        let profile = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            target_text,
            ToolResourceKind::Path,
        )];
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let request = PermissionRequest::new("write", ToolProvenance::Native, &profile, &input);
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let PermissionInvocation::Ask { session, .. } = state
            .begin_invocation(&request, &context)
            .expect("approval proposal")
        else {
            panic!("write should require approval")
        };
        let (response, response_rx) = tokio::sync::oneshot::channel();
        (
            TerminalApprovalRequest {
                id: uuid::Uuid::new_v4(),
                request: PermissionApprovalRequest {
                    tool_name: "write".to_owned(),
                    provenance: ToolProvenance::Native,
                    arguments: input,
                    summary_fields: vec!["path".to_owned()],
                    preview: session.preview().clone(),
                    binding: PermissionBinding {
                        session_id: state.session_id().expect("session id").stable_id(),
                        revisions: state
                            .state_snapshot()
                            .expect("snapshot")
                            .revisions
                            .as_array(),
                        mode: context.mode(),
                        interaction: context.interaction(),
                    },
                },
                response,
            },
            response_rx,
        )
    }

    #[test]
    fn input_for_expired_displayed_request_never_approves_next_request() {
        let (expired, expired_rx) = queued_request();
        let expired_id = expired.id;
        drop(expired_rx);
        let (current, mut current_rx) = queued_request();
        let current_id = current.id;
        let mut queue = VecDeque::from([expired, current]);
        let mut displayed = Some(expired_id);
        let mut rollover_barrier = false;

        assert!(!select_active_approval(
            &mut queue,
            &mut displayed,
            &mut rollover_barrier,
        ));
        assert_eq!(displayed, Some(current_id));

        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "y"),
            ApprovalInputOutcome::Stale
        ));
        assert!(matches!(
            current_rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));

        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "y"),
            ApprovalInputOutcome::Resolved
        ));
        assert_eq!(
            current_rx.try_recv().expect("current request response"),
            talos_core::ApprovalChoice::ApproveOnce
        );
    }

    #[test]
    fn rollover_barrier_survives_an_empty_queue_before_next_request() {
        let (expired, expired_rx) = queued_request();
        let expired_id = expired.id;
        drop(expired_rx);
        let mut queue = VecDeque::from([expired]);
        let mut displayed = Some(expired_id);
        let mut rollover_barrier = false;

        assert!(!select_active_approval(
            &mut queue,
            &mut displayed,
            &mut rollover_barrier,
        ));
        assert!(queue.is_empty());
        assert!(displayed.is_none());
        assert!(rollover_barrier);

        let (current, mut current_rx) = queued_request();
        queue.push_back(current);
        assert!(!select_active_approval(
            &mut queue,
            &mut displayed,
            &mut rollover_barrier,
        ));
        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "y"),
            ApprovalInputOutcome::Stale
        ));
        assert!(matches!(
            current_rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "y"),
            ApprovalInputOutcome::Resolved
        ));
        assert_eq!(
            current_rx.try_recv().expect("current request response"),
            talos_core::ApprovalChoice::ApproveOnce
        );
    }
}
