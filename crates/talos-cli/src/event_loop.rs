use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use anyhow::Result;
use talos_agent::permission_pipeline::ApprovalObservation;
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
    RuntimeEvent {
        generation: u64,
        event: Box<AppEvent>,
    },
    UserInput(String),
    UserInterrupt,
    ApprovalRequested(Box<TerminalApprovalRequest>),
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
    /// Fork completed with the new session ID.
    ForkCompleted {
        operation_id: uuid::Uuid,
        result: Result<Session, String>,
    },
}

pub(crate) enum AppState {
    WaitingForInput,
    AgentRunning {
        cancel_token: CancellationToken,
        task_handle: JoinHandle<()>,
    },
    ForkPending {
        operation_id: uuid::Uuid,
        cancel_token: CancellationToken,
    },
    SwitchReady {
        child: Session,
    },
    ShuttingDown,
}

pub(crate) struct EventLoop {
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,
    state: AppState,
    first_ctrl_c_time: Option<Instant>,
    session: Session,
    branch_id: Option<String>,
    session_manager: SessionManager,
    /// Clone-able sender for submitting turns to the session actor.
    sq_tx: mpsc::Sender<SessionOp>,
    approval_queue: VecDeque<TerminalApprovalRequest>,
    displayed_approval: Option<uuid::Uuid>,
    approval_rollover_barrier: bool,
    background_tools: HashMap<String, String>,
    runtime_generation: u64,
    input_started: bool,
}

#[derive(Clone)]
struct RuntimeEventSender {
    tx: mpsc::UnboundedSender<AppEvent>,
    generation: u64,
}
impl RuntimeEventSender {
    fn send(
        &self,
        event: AppEvent,
    ) -> std::result::Result<(), Box<mpsc::error::SendError<AppEvent>>> {
        self.tx
            .send(AppEvent::RuntimeEvent {
                generation: self.generation,
                event: Box::new(event),
            })
            .map_err(Box::new)
    }
}

impl EventLoop {
    pub(crate) fn new(
        session: Session,
        session_manager: SessionManager,
        handle: talos_core::session::SessionHandle,
        approval_rx: mpsc::UnboundedReceiver<TerminalApprovalRequest>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self::with_events(
            session,
            session_manager,
            handle,
            approval_rx,
            (tx, rx, 0, false),
        )
    }

    pub(crate) fn rebind(
        self,
        session: Session,
        handle: talos_core::session::SessionHandle,
        approval_rx: mpsc::UnboundedReceiver<TerminalApprovalRequest>,
    ) -> Self {
        Self::with_events(
            session,
            self.session_manager,
            handle,
            approval_rx,
            (
                self.event_tx,
                self.event_rx,
                self.runtime_generation + 1,
                self.input_started,
            ),
        )
    }

    fn with_events(
        session: Session,
        session_manager: SessionManager,
        handle: talos_core::session::SessionHandle,
        mut approval_rx: mpsc::UnboundedReceiver<TerminalApprovalRequest>,
        events: (
            mpsc::UnboundedSender<AppEvent>,
            mpsc::UnboundedReceiver<AppEvent>,
            u64,
            bool,
        ),
    ) -> Self {
        let (event_tx, event_rx, runtime_generation, input_started) = events;
        let sq_tx = handle.sq_tx;

        // Spawn a single long-lived forwarding task that translates
        // SessionEvent → AppEvent for the lifetime of the EventLoop.
        let mut eq_rx = handle.eq_rx;
        let event_tx_forward = RuntimeEventSender {
            tx: event_tx.clone(),
            generation: runtime_generation,
        };
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
                                event: talos_core::message::AgentEvent::ToolResult { result, .. },
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
                    SessionEvent::SubmissionRejected { reason, .. } => {
                        let message = if reason == talos_core::submission::SubmissionRejectionReason::ContextBudgetExceeded {
                            "Request not started: context budget exceeded after checkpoint preparation. Session history retained.".to_owned()
                        } else {
                            format!("Request not started: {reason:?}.")
                        };
                        let _ = event_tx_forward.send(AgentError(message));
                    }
                    _ => {}
                }
            }
        });
        let event_tx_approval = RuntimeEventSender {
            tx: event_tx.clone(),
            generation: runtime_generation,
        };
        tokio::spawn(async move {
            while let Some(request) = approval_rx.recv().await {
                if event_tx_approval
                    .send(ApprovalRequested(Box::new(request)))
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            event_tx,
            event_rx,
            state: AppState::WaitingForInput,
            first_ctrl_c_time: None,
            session,
            branch_id: None,
            session_manager,
            sq_tx,
            approval_queue: VecDeque::new(),
            displayed_approval: None,
            approval_rollover_barrier: false,
            background_tools: HashMap::new(),
            runtime_generation,
            input_started,
        }
    }

    pub async fn run(&mut self) -> Result<Option<Session>> {
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

        if !self.input_started {
            self.spawn_stdin_reader();
            self.spawn_signal_handler();
            self.input_started = true;
        }
        self.render();

        loop {
            let event = match self.event_rx.recv().await {
                Some(e) => e,
                None => break,
            };

            self.handle_event(event);
            self.render();

            if matches!(
                self.state,
                AppState::ShuttingDown | AppState::SwitchReady { .. }
            ) {
                break;
            }
        }

        let next = match &self.state {
            AppState::SwitchReady { child } => Some(child.clone()),
            _ => None,
        };
        self.shutdown().await;
        Ok(next)
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
            AppEvent::RuntimeEvent { generation, event }
                if generation == self.runtime_generation =>
            {
                *event
            }
            AppEvent::RuntimeEvent { .. } => return,
            event => event,
        };
        let event = match event {
            ApprovalRequested(request) => {
                if matches!(self.state, AppState::AgentRunning { .. }) {
                    self.approval_queue.push_back(*request);
                    self.render_active_approval();
                } else {
                    let _ = request.response.send(ApprovalObservation::unavailable());
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
                self.deny_pending_approvals(true);
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
                self.deny_pending_approvals(false);
                self.background_tools.clear();
                println!();
                if let Err(e) = self.session_manager.update_index(&self.session) {
                    eprintln!("Warning: failed to refresh session index: {e}");
                }
                self.state = AppState::WaitingForInput;
            }

            (AppState::AgentRunning { .. }, AgentError(msg)) => {
                self.deny_pending_approvals(false);
                self.background_tools.clear();
                eprintln!("Error: {msg}");
                self.state = AppState::WaitingForInput;
            }

            (
                AppState::ForkPending {
                    operation_id,
                    cancel_token,
                },
                AppEvent::ForkCompleted {
                    operation_id: completed,
                    result,
                },
            ) if *operation_id == completed => {
                let cancelled = cancel_token.is_cancelled();
                self.state = AppState::WaitingForInput;
                match result {
                    Ok(child) if cancelled => {
                        eprintln!(
                            "Fork cancelled after publication; child {} at {} retained. Parent remains active.",
                            child.id,
                            child.file_path.display()
                        );
                    }
                    Ok(child) => {
                        eprintln!("Fork published: {}; switching runtime.", child.id);
                        self.state = AppState::SwitchReady { child };
                    }
                    Err(error) => eprintln!("Fork failed: {error}"),
                }
            }
            (AppState::ForkPending { cancel_token, .. }, UserInterrupt) => {
                cancel_token.cancel();
                eprintln!("Cancelling fork; waiting for publication worker to finish.");
            }
            (AppState::ForkPending { .. }, UserInput(_)) => {
                eprintln!("Fork in progress; wait for completion or press Ctrl+C to cancel.");
            }

            _ => {}
        }
    }

    fn deny_pending_approvals(&mut self, user_cancelled: bool) {
        self.displayed_approval = None;
        self.approval_rollover_barrier = false;
        while let Some(request) = self.approval_queue.pop_front() {
            let observation = if user_cancelled {
                ApprovalObservation::user_cancelled()
            } else {
                ApprovalObservation::unavailable()
            };
            let _ = request.response.send(observation);
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
            let _ = request.response.send(ApprovalObservation::unavailable());
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

        self.handle_fork_session(entry_id);
    }

    fn handle_fork_session(&mut self, entry_id: Option<String>) {
        if !matches!(self.state, AppState::WaitingForInput) {
            return;
        }
        let operation_id = uuid::Uuid::new_v4();
        let cancel_token = CancellationToken::new();
        self.state = AppState::ForkPending {
            operation_id,
            cancel_token: cancel_token.clone(),
        };
        let session = self.session.clone();
        let manager = self.session_manager.clone();
        let event_tx = self.event_tx.clone();

        let sq_tx = self.sq_tx.clone();
        let runtime = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<talos_session::Session> {
            let snapshot = if crate::history_restore::requires_fork_receipt(&session)? {
                None
            } else {
                Some(runtime.block_on(async {
                    tokio::select! {
                        biased;
                        _ = cancel_token.cancelled() => Err(anyhow::anyhow!("fork cancelled")),
                        result = tokio::time::timeout(std::time::Duration::from_secs(35), async {
                            let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();
                            sq_tx.send(SessionOp::CaptureOrdinaryFork {
                                through_entry: entry_id.clone(), receipt_tx: Some(receipt_tx),
                            }).await.map_err(|_| anyhow::anyhow!("source Actor unavailable"))?;
                            receipt_rx.recv().await.ok_or_else(|| anyhow::anyhow!("source Actor dropped capture"))?
                                .map_err(|error| anyhow::anyhow!(error))
                        }) => result.map_err(|_| anyhow::anyhow!("source Actor capture timed out"))?,
                    }
                })?)
            };
            if cancel_token.is_cancelled() { anyhow::bail!("fork cancelled"); }
            crate::session_setup::fork_session_from_snapshot(
                &manager,
                &session,
                entry_id.as_deref(),
                snapshot,
                |child| {
                    runtime.block_on(async {
                        tokio::select! {
                            biased;
                            _ = cancel_token.cancelled() => Err(anyhow::anyhow!("fork cancelled")),
                            result = tokio::time::timeout(std::time::Duration::from_secs(35), async {
                            let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();
                            sq_tx
                                .send(SessionOp::ForkHistory {
                                    child_id: child.id.to_string(),
                                    child_branch: child.current_branch.clone(),
                                    through_entry: entry_id.clone(),
                                    receipt_tx: Some(receipt_tx),
                                })
                                .await
                                .map_err(|_| anyhow::anyhow!("source Actor unavailable"))?;
                            receipt_rx
                                .recv()
                                .await
                                .ok_or_else(|| {
                                    anyhow::anyhow!("source Actor dropped fork receipt")
                                })?
                                .map_err(|error| anyhow::anyhow!("{error:?}"))
                        }) => result.map_err(|_| anyhow::anyhow!("source Actor fork timed out"))?,
                        }
                    })
                },
            )
            })().map_err(|error| format!("{error:#}"));
            let _ = event_tx.send(AppEvent::ForkCompleted {
                operation_id,
                result,
            });
        });
    }

    async fn shutdown(&mut self) {
        if let AppState::ForkPending { cancel_token, .. } = &self.state {
            cancel_token.cancel();
            while let Some(event) = self.event_rx.recv().await {
                self.handle_event(event);
                if !matches!(self.state, AppState::ForkPending { .. }) {
                    break;
                }
            }
        }
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

    #[tokio::test]
    async fn repl_prestart_budget_rejection_returns_to_input_without_a_turn_completion() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let session = manager.create_session("rejection", "").expect("session");
        let original = std::fs::read(&session.file_path).expect("original transcript");
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (eq_tx, eq_rx) = mpsc::unbounded_channel();
        let (_approval_tx, approval_rx) = mpsc::unbounded_channel();
        let mut repl = EventLoop::new(
            session.clone(),
            manager,
            talos_core::session::SessionHandle { sq_tx, eq_rx },
            approval_rx,
        );
        repl.handle_event(UserInput("synthetic rejected request".into()));
        assert!(matches!(sq_rx.recv().await, Some(SessionOp::Submit { .. })));
        eq_tx
            .send(SessionEvent::SubmissionRejected {
                session_id: session.id.to_string(),
                submission_id: "private-submission-id".into(),
                sender_generation: 0,
                reason: talos_core::submission::SubmissionRejectionReason::ContextBudgetExceeded,
            })
            .expect("reject before turn start");
        let event = tokio::time::timeout(Duration::from_secs(1), repl.event_rx.recv())
            .await
            .expect("prestart rejection must reach REPL")
            .expect("event");
        let AppEvent::RuntimeEvent { event: inner, .. } = &event else {
            panic!("generation fence")
        };
        let AgentError(message) = inner.as_ref() else {
            panic!("visible rejection")
        };
        assert!(message.contains("context budget"));
        assert!(!message.contains("private-submission-id"));
        repl.handle_event(event);
        assert!(matches!(repl.state, AppState::WaitingForInput));
        assert_eq!(
            std::fs::read(&session.file_path).expect("retained transcript"),
            original
        );
        repl.handle_event(UserInput("next request".into()));
        assert!(
            matches!(sq_rx.recv().await, Some(SessionOp::Submit { message }) if message == "next request")
        );
        repl.shutdown().await;
    }

    #[tokio::test]
    async fn repl_archived_fork_uses_actor_owned_source() {
        assert_repl_fork_uses_actor_owned_source(true).await;
    }

    #[tokio::test]
    async fn repl_ordinary_fork_uses_actor_snapshot_and_child_runtime() {
        assert_repl_fork_uses_actor_owned_source(false).await;
    }

    async fn assert_repl_fork_uses_actor_owned_source(archived: bool) {
        use talos_core::message::Message;
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let workspace = crate::session_setup::canonical_workspace_root(root.path());
        let parent = manager.create_session("repl", &workspace).expect("parent");
        for i in 0..16 {
            parent
                .append(&Message::User {
                    content: format!("repl fact {i}"),
                })
                .expect("append");
        }
        let selected = parent.read_entries().expect("entries")[7].id.clone();
        if archived {
            parent.compact_archived(0).expect("keep0");
        }
        let original = std::fs::read(&parent.file_path).expect("bytes");
        let (ui_tx, _ui_rx) = mpsc::unbounded_channel();
        let builder = crate::tui_runtime_builder::TuiRuntimeBuilder::new(
            crate::build_hook_registry(true),
            root.path().to_path_buf(),
            manager.clone(),
            std::sync::Arc::new(crate::registry::TuiApprovalHandler::new(
                ui_tx,
                root.path().to_path_buf(),
            )),
            Vec::new(),
            false,
            true,
        );
        let built = builder
            .build(&talos_config::Config::default(), &parent, vec![])
            .await
            .expect("runtime");
        let mut actor = built.actor;
        let actor_join = tokio::spawn(async move { actor.run().await });
        let (_approval_tx, approval_rx) = mpsc::unbounded_channel();
        let mut event_loop =
            EventLoop::new(parent.clone(), manager.clone(), built.handle, approval_rx);
        event_loop.handle_event(UserInput(format!("/fork {selected}")));
        assert!(matches!(event_loop.state, AppState::ForkPending { .. }));
        event_loop.handle_event(UserInput("must not enter parent during fork".into()));
        let child_id = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                let event = event_loop.event_rx.recv().await.expect("event");
                event_loop.handle_event(event);
                if let AppState::SwitchReady { child } = &event_loop.state {
                    break child.id.to_string();
                }
                assert!(
                    matches!(event_loop.state, AppState::ForkPending { .. }),
                    "fork failed"
                );
            }
        })
        .await
        .expect("fork completes");
        let child = manager.resume_session(&child_id).expect("child");
        let restored = crate::history_restore::read_session_history(&child)
            .await
            .expect("restore");
        assert_eq!(restored.len(), 8);
        assert!(
            matches!(restored.last(), Some(Message::User {content}) if content == "repl fact 7")
        );
        assert_eq!(
            manager
                .get_forks(&parent.id.to_string())
                .expect("lineage")
                .len(),
            1
        );
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
        event_loop.shutdown().await;
        actor_join.await.expect("parent actor joined");
        let built = builder
            .build(&talos_config::Config::default(), &child, restored)
            .await
            .expect("child runtime");
        let mut actor = built.actor;
        let actor_join = tokio::spawn(async move { actor.run().await });
        let (_approval_tx, approval_rx) = mpsc::unbounded_channel();
        let mut event_loop = event_loop.rebind(child.clone(), built.handle, approval_rx);
        event_loop.handle_event(UserInput("post-fork child turn".into()));
        event_loop.handle_event(AppEvent::RuntimeEvent {
            generation: 0,
            event: Box::new(AgentCompleted),
        });
        assert!(
            matches!(event_loop.state, AppState::AgentRunning { .. }),
            "old runtime cannot finish child turn"
        );
        tokio::time::timeout(Duration::from_secs(10), async {
            while matches!(event_loop.state, AppState::AgentRunning { .. }) {
                let event = event_loop.event_rx.recv().await.expect("turn event");
                event_loop.handle_event(event);
            }
        })
        .await
        .expect("child turn completes");
        let child_history = crate::history_restore::read_session_history(&child)
            .await
            .expect("child history");
        assert!(
            child_history.iter().any(
                |m| matches!(m, Message::User { content } if content == "post-fork child turn")
            )
        );
        assert_eq!(
            std::fs::read(&parent.file_path).expect("parent bytes"),
            original
        );
        event_loop.shutdown().await;
        actor_join.await.expect("child actor joined");
    }

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
        "1" => talos_core::ApprovalChoice::ApproveOnce,
        "2" => talos_core::ApprovalChoice::AlwaysApprove,
        "3" => talos_core::ApprovalChoice::Deny,
        _ => return ApprovalInputOutcome::Invalid,
    };
    let request = queue.pop_front().expect("displayed request is present");
    *displayed = None;
    let _ = request
        .response
        .send(ApprovalObservation::observed_human(choice));
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
        tokio::sync::oneshot::Receiver<ApprovalObservation>,
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

    #[tokio::test]
    async fn actual_terminal_teardown_distinguishes_interrupt_from_completion_and_error() {
        for (event, expected) in [
            (UserInterrupt, ApprovalObservation::user_cancelled()),
            (AgentCompleted, ApprovalObservation::unavailable()),
            (
                AgentError("fixture failure".into()),
                ApprovalObservation::unavailable(),
            ),
        ] {
            let root = tempfile::tempdir().expect("isolated storage");
            let manager = SessionManager::with_dir(root.path().join("sessions"));
            let session = manager
                .create_session("fixture", &root.path().to_string_lossy())
                .expect("fixture session");
            let (event_tx, event_rx) = mpsc::unbounded_channel();
            let (sq_tx, mut sq_rx) = mpsc::channel(8);
            let (request, mut response) = queued_request();
            let displayed = request.id;
            let cancel = CancellationToken::new();
            let mut driver = EventLoop {
                event_tx,
                event_rx,
                state: AppState::AgentRunning {
                    cancel_token: cancel.clone(),
                    task_handle: tokio::spawn(async {}),
                },
                first_ctrl_c_time: None,
                session,
                branch_id: None,
                session_manager: manager,
                sq_tx,
                approval_queue: VecDeque::from([request]),
                displayed_approval: Some(displayed),
                approval_rollover_barrier: true,
                background_tools: HashMap::new(),
                runtime_generation: 1,
                input_started: false,
            };
            driver.handle_event(event);
            assert_eq!(
                response.try_recv().expect("actual teardown response"),
                expected
            );
            assert!(driver.approval_queue.is_empty());
            assert!(driver.displayed_approval.is_none());
            assert!(!driver.approval_rollover_barrier);
            assert!(matches!(driver.state, AppState::WaitingForInput));
            if expected == ApprovalObservation::user_cancelled() {
                assert!(cancel.is_cancelled());
                assert!(matches!(sq_rx.recv().await, Some(SessionOp::Interrupt)));
            } else {
                assert!(!cancel.is_cancelled());
                assert!(sq_rx.try_recv().is_err());
            }
        }
    }

    #[test]
    fn actual_terminal_choice_is_attributed_to_the_displayed_request() {
        use talos_agent::permission_pipeline::ApprovalAttribution;
        for (input, choice) in [
            ("1", talos_core::ApprovalChoice::ApproveOnce),
            ("2", talos_core::ApprovalChoice::AlwaysApprove),
            ("3", talos_core::ApprovalChoice::Deny),
        ] {
            let (request, mut response) = queued_request();
            let mut displayed = Some(request.id);
            let mut queue = VecDeque::from([request]);
            let mut barrier = false;
            assert!(matches!(
                resolve_displayed_approval(&mut queue, &mut displayed, &mut barrier, input),
                ApprovalInputOutcome::Resolved
            ));
            let observed = response.try_recv().expect("real input response");
            assert_eq!(observed.attribution(), ApprovalAttribution::ObservedHuman);
            assert_eq!(observed.into_choice(), choice);
            assert!(queue.is_empty());
        }
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
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "1"),
            ApprovalInputOutcome::Stale
        ));
        assert!(matches!(
            current_rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));

        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "1"),
            ApprovalInputOutcome::Resolved
        ));
        assert_eq!(
            current_rx
                .try_recv()
                .expect("current request response")
                .into_choice(),
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
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "1"),
            ApprovalInputOutcome::Stale
        ));
        assert!(matches!(
            current_rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        assert!(matches!(
            resolve_displayed_approval(&mut queue, &mut displayed, &mut rollover_barrier, "1"),
            ApprovalInputOutcome::Resolved
        ));
        assert_eq!(
            current_rx
                .try_recv()
                .expect("current request response")
                .into_choice(),
            talos_core::ApprovalChoice::ApproveOnce
        );
    }
}

#[cfg(test)]
mod archive_fork_tests {
    use super::*;
    use talos_core::message::Message;

    #[tokio::test]
    async fn repl_pending_fork_cancels_receipt_and_ignores_stale_completion() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let parent = manager.create_session("cancel", "").expect("parent");
        parent
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        parent.compact_archived(0).expect("archive");
        let (sq_tx, mut sq_rx) = mpsc::channel(4);
        let (_eq_tx, eq_rx) = mpsc::unbounded_channel();
        let (_approval_tx, approval_rx) = mpsc::unbounded_channel();
        let mut repl = EventLoop::new(
            parent.clone(),
            manager.clone(),
            talos_core::session::SessionHandle { sq_tx, eq_rx },
            approval_rx,
        );
        repl.handle_event(UserInput("/fork".into()));
        let receipt = match tokio::time::timeout(Duration::from_secs(5), sq_rx.recv())
            .await
            .expect("request")
            .expect("op")
        {
            SessionOp::ForkHistory {
                receipt_tx: Some(tx),
                ..
            } => tx,
            _ => panic!("expected fork"),
        };
        repl.handle_event(UserInput("must not submit".into()));
        assert!(sq_rx.try_recv().is_err());
        repl.handle_event(AppEvent::ForkCompleted {
            operation_id: uuid::Uuid::new_v4(),
            result: Ok(parent.clone()),
        });
        assert!(matches!(repl.state, AppState::ForkPending { .. }));
        repl.handle_event(UserInterrupt);
        assert!(
            matches!(&repl.state, AppState::ForkPending { cancel_token, .. } if cancel_token.is_cancelled())
        );
        let result = tokio::time::timeout(Duration::from_secs(5), repl.event_rx.recv())
            .await
            .expect("cancel finishes")
            .expect("completion");
        assert!(
            matches!(&result, AppEvent::ForkCompleted { result: Err(message), .. } if message.contains("fork cancelled"))
        );
        repl.handle_event(result);
        assert!(matches!(repl.state, AppState::WaitingForInput));
        assert!(
            receipt.is_closed(),
            "Actor observes cancellation through closed receipt"
        );
        assert_eq!(repl.session.id, parent.id);
        assert_eq!(manager.list_sessions().expect("sessions").len(), 1);
    }

    #[tokio::test]
    async fn repl_fork_preserves_archived_source_when_actor_unavailable() {
        for explicit_entry in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let manager = SessionManager::with_dir(root.path().join("sessions"));
            let source = manager.create_session("archive-repl", "").expect("source");
            for index in 0..205 {
                source
                    .append(&Message::User {
                        content: format!("original fact {index}"),
                    })
                    .expect("append");
            }
            source.compact_archived(50).expect("archive");
            let before = source.snapshot_bytes().expect("before");
            let tail = source
                .read_entries()
                .expect("entries")
                .last()
                .expect("tail")
                .id
                .clone();
            let (event_tx, event_rx) = mpsc::unbounded_channel();
            let (sq_tx, sq_rx) = mpsc::channel(1);
            drop(sq_rx);
            let mut repl = EventLoop {
                event_tx,
                event_rx,
                state: AppState::WaitingForInput,
                first_ctrl_c_time: None,
                session: source.clone(),
                branch_id: None,
                session_manager: manager.clone(),
                sq_tx,
                approval_queue: VecDeque::new(),
                displayed_approval: None,
                approval_rollover_barrier: false,
                background_tools: HashMap::new(),
                runtime_generation: 0,
                input_started: false,
            };
            let command = if explicit_entry {
                format!("/fork {tail}")
            } else {
                "/fork".into()
            };
            repl.handle_event(AppEvent::UserInput(command));
            assert!(matches!(repl.state, AppState::ForkPending { .. }));
            let result = tokio::time::timeout(Duration::from_secs(10), repl.event_rx.recv())
                .await
                .expect("fork worker deadline")
                .expect("fork worker event");
            match &result {
                AppEvent::ForkCompleted {
                    result: Err(message),
                    ..
                } => assert!(message.contains("source Actor unavailable")),
                _ => panic!("unexpected fork worker event"),
            }
            repl.handle_event(result);
            assert!(matches!(repl.state, AppState::WaitingForInput));
            assert_eq!(manager.list_sessions().expect("sessions").len(), 1);
            assert!(
                manager
                    .get_forks(&source.id.to_string())
                    .expect("forks")
                    .is_empty()
            );
            assert_eq!(source.snapshot_bytes().expect("preserved head"), before);
            assert_eq!(
                source
                    .inspect_archive_recovery(Instant::now() + Duration::from_secs(5), || false)
                    .expect("all originals recoverable")
                    .entries()
                    .len(),
                205
            );
        }
    }
}
