//! Session-scoped scheduled follow-up scheduler.
//!
//! Defines the crate-private command/event contract for the scheduler actor
//! (SF100) and the source labeling convention for injected messages.
//!
//! # Architecture
//!
//! - All types are `pub(crate)` — no public semver-bound API surface.
//! - The actor (SF101) owns a `HashMap` of active tasks and injects messages
//!   via the existing `SessionOp::Submit` queue.
//! - The `delay` tool (SF102) sends commands through [`SchedulerHandle`].
//! - No persistence, no cron, no direct tool execution — session-scoped only.
//!
//! # Permission Model
//!
//! See `docs/reference/I124-PRE-ACTIVATION-SECURITY-NOTE-2026-07-13.md`.
//! Mutation tools (`delay`, `schedule`, `cancel`) are `ToolNature::Execute`
//! (default `Ask`). `list_scheduled_tasks` is `ToolNature::Read`. Every
//! fire-time tool call receives a fresh independent permission decision.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use talos_core::session::SessionOp;
use talos_core::tool::{AgentTool, ToolFamily, ToolNature, ToolResult};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

// ── Duration bounds ─────────────────────────────────────────────────────

/// Minimum allowed delay in seconds. Zero or sub-second delays are rejected
/// because they provide no meaningful scheduling semantics and may indicate
/// a model error.
pub(crate) const MIN_DELAY_SECS: u64 = 1;

/// Maximum allowed delay in seconds (24 hours). Prevents unbounded timers
/// that would outlive any reasonable session and avoids `Duration` overflow
/// concerns when added to `Instant::now()`.
pub(crate) const MAX_DELAY_SECS: u64 = 86_400;

/// Validates a delay in seconds against the documented bounds.
///
/// Returns `Ok(())` if the delay is within `[MIN_DELAY_SECS, MAX_DELAY_SECS]`,
/// otherwise returns a descriptive error string.
pub(crate) fn validate_delay_secs(delay_secs: u64) -> Result<(), String> {
    if delay_secs < MIN_DELAY_SECS {
        return Err(format!(
            "delay_secs must be at least {MIN_DELAY_SECS}; got {delay_secs}"
        ));
    }
    if delay_secs > MAX_DELAY_SECS {
        return Err(format!(
            "delay_secs must be at most {MAX_DELAY_SECS}; got {delay_secs}"
        ));
    }
    Ok(())
}

pub(crate) const MIN_INTERVAL_SECS: u64 = 5;
pub(crate) const MAX_INTERVAL_SECS: u64 = 3_600;

pub(crate) fn validate_interval_secs(interval_secs: u64) -> Result<(), String> {
    if interval_secs < MIN_INTERVAL_SECS {
        return Err(format!(
            "interval_secs must be at least {MIN_INTERVAL_SECS}; got {interval_secs}"
        ));
    }
    if interval_secs > MAX_INTERVAL_SECS {
        return Err(format!(
            "interval_secs must be at most {MAX_INTERVAL_SECS}; got {interval_secs}"
        ));
    }
    Ok(())
}

// ── Source labeling ─────────────────────────────────────────────────────

/// Visible prefix prepended to scheduled follow-up messages.
///
/// This label ensures both the user and the model can distinguish a scheduled
/// follow-up from a user-typed message in the transcript. It is encoded in the
/// message `String` sent via `SessionOp::Submit` — no public API change is
/// required.
pub(crate) const SCHEDULED_FOLLOWUP_LABEL: &str = "[scheduled-followup]";

/// Formats a user message with the scheduled-followup source label.
///
/// The resulting string is sent via `SessionOp::Submit` and appears in the
/// transcript as a visibly labeled message.
pub(crate) fn label_scheduled_message(message: &str) -> String {
    format!("{SCHEDULED_FOLLOWUP_LABEL} {message}")
}

// ── Task IDs ────────────────────────────────────────────────────────────

static NEXT_TASK_SEQ: AtomicU64 = AtomicU64::new(1);

/// Generates a deterministic, monotonically increasing task ID.
///
/// IDs are process-scoped: they are unique within the current session/process
/// and are never persisted or reused after a restart.
pub(crate) fn next_task_id() -> String {
    format!("sched_{}", NEXT_TASK_SEQ.fetch_add(1, Ordering::Relaxed))
}

// ── Schedule kind ───────────────────────────────────────────────────────

/// The kind of scheduling for a task.
///
/// I124 implements only [`ScheduleKind::OneShot`]. Recurring schedules
/// (`Interval`) are owned by I125.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScheduleKind {
    OneShot,
    Recurring { interval: Duration },
}

impl fmt::Display for ScheduleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleKind::OneShot => write!(f, "one-shot"),
            ScheduleKind::Recurring { interval } => {
                write!(f, "recurring ({}s)", interval.as_secs())
            }
        }
    }
}

// ── Task metadata ───────────────────────────────────────────────────────

/// Snapshot of an active scheduled task.
///
/// Returned by [`ScheduleCommand::List`] for read-only inspection. Contains
/// only bounded, non-sensitive information.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ScheduledTaskInfo {
    /// Unique task identifier (e.g., `sched_1`).
    pub id: String,
    /// The message that will be injected when the task fires. Already labeled
    /// with [`SCHEDULED_FOLLOWUP_LABEL`] by the caller.
    pub message: String,
    /// The kind of schedule.
    pub kind: ScheduleKind,
    /// When the task was registered (`Instant::now()` at registration).
    pub created_at: Instant,
    /// When the task is scheduled to fire.
    pub fire_at: Instant,
}

impl ScheduledTaskInfo {
    /// Returns the remaining time until the task fires.
    ///
    /// Returns `Duration::ZERO` if the fire time has already passed.
    #[allow(dead_code)]
    pub fn remaining(&self) -> Duration {
        self.fire_at.saturating_duration_since(Instant::now())
    }
}

// ── Command/result types ────────────────────────────────────────────────

/// Result of a one-shot registration attempt.
#[derive(Debug)]
pub(crate) enum ScheduleRegistrationResult {
    /// Successfully registered.
    Registered {
        /// The assigned task ID.
        task_id: String,
    },
    /// Registration rejected due to invalid input.
    InvalidDuration {
        /// Human-readable reason.
        reason: String,
    },
}

/// Result of a cancellation attempt.
#[derive(Debug)]
pub(crate) enum CancelResult {
    /// Task was found and cancelled.
    Cancelled,
    /// Task ID not found, already fired, or already cancelled.
    NotFound,
}

/// Commands sent to the [`SchedulerActor`](crate::scheduler) via a bounded
/// mpsc channel.
///
/// The actor owns the receiver; tools and the CLI hold a
/// [`SchedulerHandle`] (the sender clone) to issue commands.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ScheduleCommand {
    /// Register a one-shot delayed follow-up.
    RegisterOneShot {
        /// Caller-supplied task ID (from [`next_task_id`]). If `None`, the
        /// actor generates one.
        id: Option<String>,
        /// The user message to inject when the delay expires.
        message: String,
        /// The delay duration, already validated by [`validate_delay_secs`].
        delay: Duration,
        /// One-shot response channel for the registration result.
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    },
    /// Register a recurring follow-up that fires at a bounded interval.
    RegisterRecurring {
        id: Option<String>,
        message: String,
        interval: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    },
    /// Cancel a scheduled task by ID.
    Cancel {
        /// The task ID to cancel.
        id: String,
        /// One-shot response channel for the cancellation result.
        response_tx: oneshot::Sender<CancelResult>,
    },
    /// Request a snapshot of all active scheduled tasks.
    List {
        /// One-shot response channel for the task list.
        response_tx: oneshot::Sender<Vec<ScheduledTaskInfo>>,
    },
    /// Shut down the scheduler actor and cancel all pending tasks.
    ///
    /// Sent when the session is closing or the cancellation token fires.
    Shutdown,
}

// ── Scheduler handle ────────────────────────────────────────────────────

/// Handle for sending commands to the scheduler actor.
///
/// This is a cloneable, `Send + Sync` handle. Multiple tools can share the
/// same handle by cloning it. The handle holds only a bounded
/// `mpsc::Sender` — when the actor shuts down, `send()` returns an error
/// that the caller must handle gracefully (no panic).
///
/// The struct and its field are `pub(crate)` — only `talos-agent` internal
/// code can construct or inspect a handle. External callers receive the
/// delay tool as `Arc<dyn AgentTool>` via [`create_delay_tool_and_scheduler`].
#[derive(Clone)]
pub(crate) struct SchedulerHandle {
    cmd_tx: mpsc::Sender<ScheduleCommand>,
}

impl SchedulerHandle {
    /// Creates a new handle wrapping the given command sender.
    ///
    /// This is `pub(crate)` — only code within `talos-agent` can construct a
    /// handle. The CLI composition root calls
    /// [`spawn_scheduler_actor`](crate::scheduler) (SF101) which returns a
    /// handle.
    pub(crate) fn new(cmd_tx: mpsc::Sender<ScheduleCommand>) -> Self {
        Self { cmd_tx }
    }

    /// Sends a command to the scheduler actor.
    ///
    /// Returns `Err` if the actor has shut down. Callers must handle this
    /// gracefully — typically by returning a bounded error result to the
    /// model, never by panicking.
    pub(crate) async fn send(
        &self,
        command: ScheduleCommand,
    ) -> Result<(), mpsc::error::SendError<ScheduleCommand>> {
        self.cmd_tx.send(command).await
    }
}

impl fmt::Debug for SchedulerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchedulerHandle")
            .field("cmd_tx", &"mpsc::Sender<ScheduleCommand>")
            .finish()
    }
}

// ── Actor ───────────────────────────────────────────────────────────────

/// Internal bookkeeping for a live scheduled task.
struct ActiveTask {
    info: ScheduledTaskInfo,
    handle: JoinHandle<()>,
}

/// Single-owner scheduler actor for session-scoped delayed follow-ups.
///
/// Owns all scheduling state for the current process/session. No persistence,
/// no cross-session visibility. The actor receives commands via `cmd_rx`,
/// fires one-shot tasks by sending `SessionOp::Submit` through `sq_tx`, and
/// shuts down when the `CancellationToken` fires or `Shutdown` is received.
pub(crate) struct SchedulerActor {
    cmd_rx: mpsc::Receiver<ScheduleCommand>,
    sq_tx: mpsc::Sender<SessionOp>,
    cancel_token: CancellationToken,
    tasks: HashMap<String, ActiveTask>,
    fired_tx: mpsc::UnboundedSender<String>,
    fired_rx: mpsc::UnboundedReceiver<String>,
}

impl SchedulerActor {
    pub(crate) fn new(
        cmd_rx: mpsc::Receiver<ScheduleCommand>,
        sq_tx: mpsc::Sender<SessionOp>,
        cancel_token: CancellationToken,
    ) -> Self {
        let (fired_tx, fired_rx) = mpsc::unbounded_channel();
        Self {
            cmd_rx,
            sq_tx,
            cancel_token,
            tasks: HashMap::new(),
            fired_tx,
            fired_rx,
        }
    }

    /// Runs the actor event loop until shutdown.
    ///
    /// Returns when the `CancellationToken` fires, `Shutdown` is received,
    /// or the command channel closes. All remaining tasks are aborted on
    /// exit — no fire can occur after the actor stops.
    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                biased;

                _ = self.cancel_token.cancelled() => break,

                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        Some(ScheduleCommand::RegisterOneShot {
                            id,
                            message,
                            delay,
                            response_tx,
                        }) => {
                            self.handle_register_one_shot(id, message, delay, response_tx);
                        }
                        Some(ScheduleCommand::RegisterRecurring {
                            id,
                            message,
                            interval,
                            response_tx,
                        }) => {
                            self.handle_register_recurring(
                                id,
                                message,
                                interval,
                                response_tx,
                            );
                        }
                        Some(ScheduleCommand::Cancel { id, response_tx }) => {
                            self.handle_cancel(id, response_tx);
                        }
                        Some(ScheduleCommand::List { response_tx }) => {
                            self.handle_list(response_tx);
                        }
                        Some(ScheduleCommand::Shutdown) => break,
                        None => break,
                    }
                }

                Some(task_id) = self.fired_rx.recv() => {
                    self.tasks.remove(&task_id);
                }
            }
        }

        for (_, task) in self.tasks.drain() {
            task.handle.abort();
        }
    }

    fn handle_register_one_shot(
        &mut self,
        id: Option<String>,
        message: String,
        delay: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    ) {
        if let Err(reason) = validate_delay_secs(delay.as_secs()) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidDuration { reason });
            return;
        }

        let task_id = id.unwrap_or_else(next_task_id);
        let now = Instant::now();
        let fire_at = now + delay;
        let labeled_message = label_scheduled_message(&message);

        let sq_tx = self.sq_tx.clone();
        let fired_tx = self.fired_tx.clone();
        let task_id_for_fire = task_id.clone();
        let labeled_for_fire = labeled_message.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            if sq_tx
                .send(SessionOp::Submit {
                    message: labeled_for_fire,
                })
                .await
                .is_err()
            {
                tracing::debug!(
                    task_id = %task_id_for_fire,
                    "scheduled follow-up fire: session queue closed"
                );
            }
            let _ = fired_tx.send(task_id_for_fire);
        });

        self.tasks.insert(
            task_id.clone(),
            ActiveTask {
                info: ScheduledTaskInfo {
                    id: task_id.clone(),
                    message: labeled_message,
                    kind: ScheduleKind::OneShot,
                    created_at: now,
                    fire_at,
                },
                handle,
            },
        );

        let _ = response_tx.send(ScheduleRegistrationResult::Registered { task_id });
    }

    fn handle_register_recurring(
        &mut self,
        id: Option<String>,
        message: String,
        interval: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    ) {
        if let Err(reason) = validate_interval_secs(interval.as_secs()) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidDuration { reason });
            return;
        }

        let task_id = id.unwrap_or_else(next_task_id);
        let now = Instant::now();
        let labeled_message = label_scheduled_message(&message);

        let sq_tx = self.sq_tx.clone();
        let task_id_for_fire = task_id.clone();
        let labeled_for_fire = labeled_message.clone();

        let handle = tokio::spawn(async move {
            let mut timer =
                tokio::time::interval_at(tokio::time::Instant::now() + interval, interval);
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                timer.tick().await;

                if sq_tx
                    .send(SessionOp::Submit {
                        message: labeled_for_fire.clone(),
                    })
                    .await
                    .is_err()
                {
                    tracing::debug!(
                        task_id = %task_id_for_fire,
                        "recurring follow-up: session queue closed"
                    );
                    break;
                }
            }
        });

        self.tasks.insert(
            task_id.clone(),
            ActiveTask {
                info: ScheduledTaskInfo {
                    id: task_id.clone(),
                    message: labeled_message,
                    kind: ScheduleKind::Recurring { interval },
                    created_at: now,
                    fire_at: now + interval,
                },
                handle,
            },
        );

        let _ = response_tx.send(ScheduleRegistrationResult::Registered { task_id });
    }

    fn handle_cancel(&mut self, id: String, response_tx: oneshot::Sender<CancelResult>) {
        if let Some(task) = self.tasks.remove(&id) {
            task.handle.abort();
            let _ = response_tx.send(CancelResult::Cancelled);
        } else {
            let _ = response_tx.send(CancelResult::NotFound);
        }
    }

    fn handle_list(&self, response_tx: oneshot::Sender<Vec<ScheduledTaskInfo>>) {
        let snapshot: Vec<ScheduledTaskInfo> =
            self.tasks.values().map(|t| t.info.clone()).collect();
        let _ = response_tx.send(snapshot);
    }
}

/// Spawns the scheduler actor and returns a handle for sending commands.
///
/// The actor owns all scheduling state. The returned `SchedulerHandle` can
/// be cloned to share among tools. The `JoinHandle` completes when the actor
/// shuts down.
///
/// The `sq_tx` should be the same sender used by the session actor, so
/// scheduled fires inject into the same ordered queue as user messages.
/// The `cancel_token` should be linked to session shutdown.
#[allow(dead_code)]
pub(crate) fn spawn_scheduler_actor(
    sq_tx: mpsc::Sender<SessionOp>,
    cancel_token: CancellationToken,
) -> (SchedulerHandle, JoinHandle<()>) {
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let handle = SchedulerHandle::new(cmd_tx);
    let actor = SchedulerActor::new(cmd_rx, sq_tx, cancel_token);
    let join = tokio::spawn(async move { actor.run().await });
    (handle, join)
}

// ── Two-phase composition infrastructure ─────────────────────────────────

/// Creates the delay tool and a pending scheduler actor for compatibility with
/// the I124 public API.
///
/// New composition roots should use [`create_scheduler_tools`] to receive both
/// scheduler tools. This entry point remains available so existing consumers
/// are not broken by the additive I125 API.
pub fn create_delay_tool_and_scheduler() -> (Arc<dyn AgentTool>, PendingSchedulerActor) {
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let handle = SchedulerHandle::new(cmd_tx);
    let tool: Arc<dyn AgentTool> = Arc::new(DelayTool::new(handle));
    let pending = PendingSchedulerActor { cmd_rx };
    (tool, pending)
}

/// Creates scheduler tools (delay + schedule) and a pending scheduler actor
/// for two-phase composition.
///
/// Returns the tools as `Vec<Arc<dyn AgentTool>>` (ready for the caller to
/// wrap in permission wrappers) and a [`PendingSchedulerActor`] that must be
/// spawned after the session provides `sq_tx`.
///
/// This is the additive I125 composition entry point. The I124-compatible
/// [`create_delay_tool_and_scheduler`] entry point remains available. All
/// internal scheduler types are `pub(crate)`. See ADR-041 for the API boundary.
pub fn create_scheduler_tools() -> (Vec<Arc<dyn AgentTool>>, PendingSchedulerActor) {
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let handle = SchedulerHandle::new(cmd_tx);
    let tools: Vec<Arc<dyn AgentTool>> = vec![
        Arc::new(DelayTool::new(handle.clone())),
        Arc::new(ScheduleTool::new(handle)),
    ];
    let pending = PendingSchedulerActor { cmd_rx };
    (tools, pending)
}

/// Holds the scheduler command receiver until `sq_tx` is available.
///
/// After the session is created, call [`spawn`](Self::spawn) to start the
/// actor. The actor will abort all tasks when the `CancellationToken` fires.
pub struct PendingSchedulerActor {
    cmd_rx: mpsc::Receiver<ScheduleCommand>,
}

impl PendingSchedulerActor {
    /// Spawns the scheduler actor, linking it to the session queue and
    /// shutdown token.
    ///
    /// The returned `JoinHandle` completes when the actor shuts down (via
    /// `Shutdown` command, `CancellationToken`, or command channel closure).
    pub fn spawn(
        self,
        sq_tx: mpsc::Sender<SessionOp>,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let actor = SchedulerActor::new(self.cmd_rx, sq_tx, cancel_token);
        tokio::spawn(async move { actor.run().await })
    }
}

impl fmt::Debug for PendingSchedulerActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingSchedulerActor")
            .field("cmd_rx", &"mpsc::Receiver<ScheduleCommand>")
            .finish()
    }
}

// ── Delay tool (SF102) ───────────────────────────────────────────────────

/// Built-in tool for scheduling a one-shot delayed follow-up message.
///
/// Permission: `ToolNature::Execute` (default `Ask`). The registration
/// approval authorizes only this scheduling operation — any tool call in the
/// follow-up turn receives its own fresh permission decision.
///
/// Session-scoped: the scheduled task dies when the process exits and is
/// never persisted.
pub(crate) struct DelayTool {
    handle: SchedulerHandle,
}

impl DelayTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for DelayTool {
    fn name(&self) -> &str {
        "delay"
    }

    fn description(&self) -> &str {
        "Schedule a one-shot delayed follow-up message that will be injected \
         into the session after the specified delay. The task is \
         session-scoped: it dies when the process exits and is never \
         persisted. Minimum delay: 1 second. Maximum delay: 86400 seconds \
         (24 hours)."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The follow-up message to inject after the delay."
                },
                "delay_secs": {
                    "type": "integer",
                    "description": "Delay in seconds before the message is injected.",
                    "minimum": MIN_DELAY_SECS,
                    "maximum": MAX_DELAY_SECS
                }
            },
            "required": ["message", "delay_secs"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        let message = match input.get("message").and_then(|v| v.as_str()) {
            Some(msg) if !msg.is_empty() => msg.to_string(),
            _ => return ToolResult::error("missing or empty 'message' field"),
        };

        let delay_secs = match input.get("delay_secs").and_then(|v| v.as_u64()) {
            Some(secs) => secs,
            None => {
                return ToolResult::error(
                    "missing or invalid 'delay_secs' field (expected a positive integer)",
                );
            }
        };

        if let Err(reason) = validate_delay_secs(delay_secs) {
            return ToolResult::error(reason);
        }

        let (response_tx, response_rx) = oneshot::channel();
        let command = ScheduleCommand::RegisterOneShot {
            id: None,
            message,
            delay: Duration::from_secs(delay_secs),
            response_tx,
        };

        if self.handle.send(command).await.is_err() {
            return ToolResult::error("scheduler is not available (session may be shutting down)");
        }

        match response_rx.await {
            Ok(ScheduleRegistrationResult::Registered { task_id }) => ToolResult::success(format!(
                "Scheduled follow-up registered.\nTask ID: {task_id}\nDelay: {delay_secs} second(s)\n\nThe message will be injected into the session after the delay as a visibly labeled scheduled follow-up."
            )),
            Ok(ScheduleRegistrationResult::InvalidDuration { reason }) => ToolResult::error(reason),
            Err(_) => ToolResult::error("scheduler dropped the request"),
        }
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

pub(crate) struct ScheduleTool {
    handle: SchedulerHandle,
}

impl ScheduleTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for ScheduleTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Schedule a recurring follow-up message that fires at a bounded \
         interval. The message is injected into the session at each interval \
         until cancelled. Session-scoped: dies when the process exits. \
         Minimum interval: 5 seconds. Maximum interval: 3600 seconds (1 hour). \
         Missed ticks are delayed, not burst-caught-up."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The follow-up message to inject at each interval."
                },
                "interval_secs": {
                    "type": "integer",
                    "description": "Interval in seconds between fires.",
                    "minimum": MIN_INTERVAL_SECS,
                    "maximum": MAX_INTERVAL_SECS
                }
            },
            "required": ["message", "interval_secs"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        let message = match input.get("message").and_then(|v| v.as_str()) {
            Some(msg) if !msg.is_empty() => msg.to_string(),
            _ => return ToolResult::error("missing or empty 'message' field"),
        };

        let interval_secs = match input.get("interval_secs").and_then(|v| v.as_u64()) {
            Some(secs) => secs,
            None => {
                return ToolResult::error(
                    "missing or invalid 'interval_secs' field (expected a positive integer)",
                );
            }
        };

        if let Err(reason) = validate_interval_secs(interval_secs) {
            return ToolResult::error(reason);
        }

        let (response_tx, response_rx) = oneshot::channel();
        let command = ScheduleCommand::RegisterRecurring {
            id: None,
            message,
            interval: Duration::from_secs(interval_secs),
            response_tx,
        };

        if self.handle.send(command).await.is_err() {
            return ToolResult::error("scheduler is not available (session may be shutting down)");
        }

        match response_rx.await {
            Ok(ScheduleRegistrationResult::Registered { task_id }) => ToolResult::success(format!(
                "Recurring follow-up registered.\nTask ID: {task_id}\nInterval: {interval_secs} second(s)\n\nThe message will be injected into the session at each interval as a visibly labeled scheduled follow-up."
            )),
            Ok(ScheduleRegistrationResult::InvalidDuration { reason }) => ToolResult::error(reason),
            Err(_) => ToolResult::error("scheduler dropped the request"),
        }
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_delay_secs_accepts_valid_range() {
        assert!(validate_delay_secs(MIN_DELAY_SECS).is_ok());
        assert!(validate_delay_secs(60).is_ok());
        assert!(validate_delay_secs(MAX_DELAY_SECS).is_ok());
    }

    #[test]
    fn test_validate_delay_secs_rejects_zero() {
        let result = validate_delay_secs(0);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("at least"),
            "error should mention minimum"
        );
    }

    #[test]
    fn test_validate_delay_secs_rejects_excessive() {
        let result = validate_delay_secs(MAX_DELAY_SECS + 1);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("at most"),
            "error should mention maximum"
        );
    }

    #[test]
    fn test_label_scheduled_message_prepends_label() {
        let labeled = label_scheduled_message("check the build status");
        assert!(labeled.starts_with(SCHEDULED_FOLLOWUP_LABEL));
        assert!(labeled.contains("check the build status"));
        assert!(
            labeled == "[scheduled-followup] check the build status",
            "exact format must be stable for transcript parsing"
        );
    }

    #[test]
    fn test_label_scheduled_message_preserves_empty_message() {
        let labeled = label_scheduled_message("");
        assert_eq!(labeled, "[scheduled-followup] ");
    }

    #[test]
    fn test_next_task_id_is_monotonic() {
        let id1 = next_task_id();
        let id2 = next_task_id();
        let id3 = next_task_id();
        assert!(id1 != id2, "IDs must be unique");
        assert!(id2 != id3, "IDs must be unique");
        assert!(id1 != id3, "IDs must be unique");
        assert!(id1.starts_with("sched_"), "ID should have sched_ prefix");
    }

    #[test]
    fn test_schedule_kind_display() {
        assert_eq!(ScheduleKind::OneShot.to_string(), "one-shot");
    }

    #[test]
    fn test_scheduled_task_info_remaining() {
        let now = Instant::now();
        let info = ScheduledTaskInfo {
            id: "sched_test".to_string(),
            message: "test message".to_string(),
            kind: ScheduleKind::OneShot,
            created_at: now,
            fire_at: now + Duration::from_secs(60),
        };
        let remaining = info.remaining();
        // remaining should be close to 60 seconds (within a small tolerance)
        assert!(remaining <= Duration::from_secs(60));
        assert!(remaining > Duration::from_secs(58));
    }

    #[test]
    fn test_scheduled_task_info_remaining_past_fire() {
        let now = Instant::now();
        let info = ScheduledTaskInfo {
            id: "sched_past".to_string(),
            message: "past message".to_string(),
            kind: ScheduleKind::OneShot,
            created_at: now - Duration::from_secs(120),
            fire_at: now - Duration::from_secs(60),
        };
        assert_eq!(info.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_scheduler_handle_send_on_closed_channel() {
        let (cmd_tx, cmd_rx) = mpsc::channel::<ScheduleCommand>(8);
        drop(cmd_rx);
        let handle = SchedulerHandle::new(cmd_tx);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let result = rt.block_on(handle.send(ScheduleCommand::Shutdown));
        assert!(result.is_err(), "send on closed channel should error");
    }

    // ── Actor behavior tests (paused time) ──────────────────────────────

    async fn yield_times(n: usize) {
        for _ in 0..n {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn actor_one_shot_fires_and_injects_labeled_message() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "check the build".to_string(),
                delay: Duration::from_secs(1),
                response_tx: resp_tx,
            })
            .await
            .unwrap();

        let task_id = match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::Registered { task_id } => task_id,
            other => panic!("expected Registered, got {other:?}"),
        };
        assert!(task_id.starts_with("sched_"));

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;

        let op = sq_rx
            .try_recv()
            .expect("message should have been injected after delay");
        match op {
            SessionOp::Submit { message } => {
                assert!(
                    message.starts_with(SCHEDULED_FOLLOWUP_LABEL),
                    "injected message must carry the source label"
                );
                assert!(message.contains("check the build"));
            }
            other => panic!("expected Submit, got {other:?}"),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn actor_cancelled_one_shot_does_not_fire() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "should not fire".to_string(),
                delay: Duration::from_secs(1),
                response_tx: resp_tx,
            })
            .await
            .unwrap();

        let task_id = match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::Registered { task_id } => task_id,
            _ => panic!("expected Registered"),
        };

        let (cancel_tx, cancel_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::Cancel {
                id: task_id,
                response_tx: cancel_tx,
            })
            .await
            .unwrap();
        assert!(matches!(cancel_rx.await.unwrap(), CancelResult::Cancelled));

        tokio::time::advance(Duration::from_secs(5)).await;
        yield_times(10).await;

        assert!(
            sq_rx.try_recv().is_err(),
            "cancelled task must not inject a message"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_shutdown_aborts_all_tasks() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        for msg in &["task-a", "task-b", "task-c"] {
            let (resp_tx, resp_rx) = oneshot::channel();
            handle
                .send(ScheduleCommand::RegisterOneShot {
                    id: None,
                    message: msg.to_string(),
                    delay: Duration::from_secs(5),
                    response_tx: resp_tx,
                })
                .await
                .unwrap();
            assert!(matches!(
                resp_rx.await.unwrap(),
                ScheduleRegistrationResult::Registered { .. }
            ));
        }

        handle.send(ScheduleCommand::Shutdown).await.unwrap();
        yield_times(5).await;

        tokio::time::advance(Duration::from_secs(10)).await;
        yield_times(10).await;

        assert!(
            sq_rx.try_recv().is_err(),
            "no task should fire after shutdown"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_cancel_token_aborts_all_tasks() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token.clone());

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "should not fire".to_string(),
                delay: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(matches!(
            resp_rx.await.unwrap(),
            ScheduleRegistrationResult::Registered { .. }
        ));

        cancel_token.cancel();
        yield_times(5).await;

        tokio::time::advance(Duration::from_secs(10)).await;
        yield_times(10).await;

        assert!(
            sq_rx.try_recv().is_err(),
            "no task should fire after cancellation token fires"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_rejects_invalid_duration() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "bad delay".to_string(),
                delay: Duration::from_secs(0),
                response_tx: resp_tx,
            })
            .await
            .unwrap();

        match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::InvalidDuration { reason } => {
                assert!(reason.contains("at least"));
            }
            _ => panic!("expected InvalidDuration"),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn actor_list_returns_active_tasks() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        for _ in 0..3 {
            let (resp_tx, resp_rx) = oneshot::channel();
            handle
                .send(ScheduleCommand::RegisterOneShot {
                    id: None,
                    message: "pending".to_string(),
                    delay: Duration::from_secs(60),
                    response_tx: resp_tx,
                })
                .await
                .unwrap();
            assert!(matches!(
                resp_rx.await.unwrap(),
                ScheduleRegistrationResult::Registered { .. }
            ));
        }

        let (list_tx, list_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::List {
                response_tx: list_tx,
            })
            .await
            .unwrap();

        let snapshot = list_rx.await.unwrap();
        assert_eq!(snapshot.len(), 3);
        for info in &snapshot {
            assert!(matches!(info.kind, ScheduleKind::OneShot));
            assert!(info.message.starts_with(SCHEDULED_FOLLOWUP_LABEL));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn actor_fired_task_removed_from_list() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "fires soon".to_string(),
                delay: Duration::from_secs(1),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(resp_rx.await.is_ok());

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;

        let (list_tx, list_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::List {
                response_tx: list_tx,
            })
            .await
            .unwrap();

        let snapshot = list_rx.await.unwrap();
        assert!(snapshot.is_empty(), "fired task must be removed from list");
    }

    #[tokio::test(start_paused = true)]
    async fn actor_cancel_unknown_returns_not_found() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (cancel_tx, cancel_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::Cancel {
                id: "nonexistent".to_string(),
                response_tx: cancel_tx,
            })
            .await
            .unwrap();

        assert!(matches!(cancel_rx.await.unwrap(), CancelResult::NotFound));
    }

    #[tokio::test(start_paused = true)]
    async fn actor_closed_session_queue_no_panic() {
        let (sq_tx, sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        drop(sq_rx);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message: "queue closed".to_string(),
                delay: Duration::from_secs(1),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(resp_rx.await.is_ok());

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;

        let (list_tx, list_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::List {
                response_tx: list_tx,
            })
            .await
            .unwrap();

        let snapshot = list_rx.await.unwrap();
        assert!(
            snapshot.is_empty(),
            "fired task must be cleaned up even when queue is closed"
        );
    }

    // ── Recurring behavior tests (SF110) ─────────────────────────────────

    #[test]
    fn validate_interval_secs_accepts_valid_range() {
        assert!(validate_interval_secs(MIN_INTERVAL_SECS).is_ok());
        assert!(validate_interval_secs(30).is_ok());
        assert!(validate_interval_secs(MAX_INTERVAL_SECS).is_ok());
    }

    #[test]
    fn validate_interval_secs_rejects_below_minimum() {
        let result = validate_interval_secs(MIN_INTERVAL_SECS - 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least"));
    }

    #[test]
    fn validate_interval_secs_rejects_above_maximum() {
        let result = validate_interval_secs(MAX_INTERVAL_SECS + 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at most"));
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_no_immediate_first_tick() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "tick".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(matches!(
            resp_rx.await.unwrap(),
            ScheduleRegistrationResult::Registered { .. }
        ));

        yield_times(10).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "no immediate first tick — recurring must wait one interval"
        );

        tokio::time::advance(Duration::from_secs(6)).await;
        yield_times(10).await;
        assert!(
            sq_rx.try_recv().is_ok(),
            "first fire after one interval period"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_fires_at_cadence() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "cadence".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(resp_rx.await.is_ok());

        // First fire at t=5
        tokio::time::advance(Duration::from_secs(6)).await;
        yield_times(10).await;
        let op = sq_rx.try_recv();
        assert!(op.is_ok(), "first fire at ~t=5");
        match op.unwrap() {
            SessionOp::Submit { message } => assert!(message.starts_with(SCHEDULED_FOLLOWUP_LABEL)),
            _ => panic!("expected Submit"),
        }

        // Second fire at t=10
        tokio::time::advance(Duration::from_secs(5)).await;
        yield_times(10).await;
        assert!(sq_rx.try_recv().is_ok(), "second fire at ~t=10");

        // Third fire at t=15
        tokio::time::advance(Duration::from_secs(5)).await;
        yield_times(10).await;
        assert!(sq_rx.try_recv().is_ok(), "third fire at ~t=15");
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_cancelled_stops_firing() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "cancel-me".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        let task_id = match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::Registered { task_id } => task_id,
            _ => panic!("expected Registered"),
        };

        // Cancel before first fire
        let (cancel_tx, cancel_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::Cancel {
                id: task_id,
                response_tx: cancel_tx,
            })
            .await
            .unwrap();
        assert!(matches!(cancel_rx.await.unwrap(), CancelResult::Cancelled));

        // Advance well past multiple intervals
        tokio::time::advance(Duration::from_secs(30)).await;
        yield_times(15).await;

        assert!(
            sq_rx.try_recv().is_err(),
            "cancelled recurring task must not fire"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_missed_tick_delay_not_burst() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "delay-test".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(resp_rx.await.is_ok());

        // Advance 21 seconds past 4 interval boundaries (t=5,10,15,20).
        // With MissedTickBehavior::Delay, only ONE fire occurs — the
        // delayed tick reschedules to now+interval rather than catching up.
        // With Burst, all 4 missed ticks would fire.
        tokio::time::advance(Duration::from_secs(21)).await;
        yield_times(20).await;

        let mut count = 0;
        while sq_rx.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(
            count, 1,
            "Delay produces exactly 1 fire after missing 4 intervals; \
             Burst would produce 4 catch-up fires"
        );

        // Next fire rescheduled to t=21+5=26. Advance to t=25 — no fire yet.
        tokio::time::advance(Duration::from_secs(4)).await;
        yield_times(10).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "no fire before the rescheduled interval boundary (t=26)"
        );

        // Advance to t=27 — fire at t=26 now available
        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        assert!(
            sq_rx.try_recv().is_ok(),
            "exactly one fire at the rescheduled boundary (t=26)"
        );
        assert!(
            sq_rx.try_recv().is_err(),
            "no additional fire — next at t=31"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_cancel_race_no_duplicate() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "race".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        let task_id = match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::Registered { task_id } => task_id,
            _ => panic!("expected Registered"),
        };

        // Make the first timer tick ready without yielding to its task, then
        // enqueue Cancel. The timer task and actor command are now competing at
        // the same paused-time boundary.
        tokio::time::advance(Duration::from_secs(5)).await;
        let (cancel_tx, cancel_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::Cancel {
                id: task_id,
                response_tx: cancel_tx,
            })
            .await
            .unwrap();
        assert!(matches!(cancel_rx.await.unwrap(), CancelResult::Cancelled));
        yield_times(10).await;

        // Depending on which ready task wins, the boundary tick may have
        // entered the queue before cancellation. It must never be duplicated.
        let mut boundary_count = 0;
        while sq_rx.try_recv().is_ok() {
            boundary_count += 1;
        }
        assert!(
            boundary_count <= 1,
            "the competing boundary may enqueue at most one turn, got {boundary_count}"
        );

        tokio::time::advance(Duration::from_secs(30)).await;
        yield_times(15).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "no recurring turn may be enqueued after cancellation is confirmed"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_shutdown_no_duplicate() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "shutdown-test".to_string(),
                interval: Duration::from_secs(5),
                response_tx: resp_tx,
            })
            .await
            .unwrap();
        assert!(resp_rx.await.is_ok());

        // Make the timer and Shutdown command compete at the first boundary.
        tokio::time::advance(Duration::from_secs(5)).await;
        handle.send(ScheduleCommand::Shutdown).await.unwrap();
        join.await.unwrap();
        yield_times(5).await;

        let mut boundary_count = 0;
        while sq_rx.try_recv().is_ok() {
            boundary_count += 1;
        }
        assert!(
            boundary_count <= 1,
            "the competing shutdown boundary may enqueue at most one turn, got {boundary_count}"
        );

        tokio::time::advance(Duration::from_secs(30)).await;
        yield_times(15).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "no recurring turn may be enqueued after scheduler shutdown completes"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn actor_recurring_rejects_invalid_interval() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, _join) = spawn_scheduler_actor(sq_tx, cancel_token);

        let (resp_tx, resp_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message: "bad".to_string(),
                interval: Duration::from_secs(1), // below MIN_INTERVAL_SECS=5
                response_tx: resp_tx,
            })
            .await
            .unwrap();

        match resp_rx.await.unwrap() {
            ScheduleRegistrationResult::InvalidDuration { reason } => {
                assert!(reason.contains("at least"));
            }
            _ => panic!("expected InvalidDuration"),
        }
    }

    // ── DelayTool unit tests (SF103) ────────────────────────────────────

    #[test]
    fn delay_tool_nature_is_execute() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        assert_eq!(tool.nature(), talos_core::tool::ToolNature::Execute);
    }

    #[test]
    fn legacy_delay_factory_remains_compatible() {
        let (tool, _pending) = create_delay_tool_and_scheduler();

        assert_eq!(tool.name(), "delay");
        assert_eq!(tool.nature(), talos_core::tool::ToolNature::Execute);
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_executes_and_returns_task_id() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (tools, pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let _join = pending.spawn(sq_tx, cancel_token);

        let result = tool
            .execute(serde_json::json!({
                "message": "follow up on the deploy",
                "delay_secs": 60
            }))
            .await;

        assert!(!result.is_error, "valid input should succeed");
        assert!(
            result.content.contains("Task ID:"),
            "result should contain the task ID: {}",
            result.content
        );
        assert!(
            result.content.contains("sched_"),
            "task ID should use sched_ prefix: {}",
            result.content
        );
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_rejects_missing_message() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let result = tool
            .execute(serde_json::json!({
                "delay_secs": 10
            }))
            .await;

        assert!(result.is_error, "missing message should error");
        assert!(result.content.contains("message"));
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_rejects_missing_delay_secs() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let result = tool
            .execute(serde_json::json!({
                "message": "test"
            }))
            .await;

        assert!(result.is_error, "missing delay_secs should error");
        assert!(result.content.contains("delay_secs"));
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_rejects_empty_message() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let result = tool
            .execute(serde_json::json!({
                "message": "",
                "delay_secs": 10
            }))
            .await;

        assert!(result.is_error, "empty message should error");
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_rejects_zero_delay() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let result = tool
            .execute(serde_json::json!({
                "message": "test",
                "delay_secs": 0
            }))
            .await;

        assert!(result.is_error, "zero delay should error");
        assert!(result.content.contains("at least"));
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_rejects_excessive_delay() {
        let (tools, _pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let result = tool
            .execute(serde_json::json!({
                "message": "test",
                "delay_secs": MAX_DELAY_SECS + 1
            }))
            .await;

        assert!(result.is_error, "excessive delay should error");
        assert!(result.content.contains("at most"));
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_error_when_scheduler_unavailable() {
        let (tools, pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        // Drop the pending actor without spawning — the command channel
        // has no receiver, so send() will fail.
        drop(pending);

        // Give the runtime a turn to ensure the channel is fully closed.
        tokio::task::yield_now().await;

        let result = tool
            .execute(serde_json::json!({
                "message": "test",
                "delay_secs": 10
            }))
            .await;

        assert!(
            result.is_error,
            "unavailable scheduler should return error, not panic"
        );
    }

    // ── End-to-end integration test (SF103 fixture-provider proof) ──────

    #[tokio::test(start_paused = true)]
    async fn delay_tool_end_to_end_fires_and_injects_labeled_message() {
        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();

        let (tools, pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let _join = pending.spawn(sq_tx, cancel_token);

        // Step 1: Execute the delay tool as the model would.
        let result = tool
            .execute(serde_json::json!({
                "message": "check on the build status",
                "delay_secs": 1
            }))
            .await;

        assert!(
            !result.is_error,
            "delay tool should succeed with valid input"
        );
        assert!(result.content.contains("Task ID:"));

        // Verify no message has been injected yet (before the delay).
        yield_times(5).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "no message should be injected before the delay expires"
        );

        // Step 2: Advance time past the delay.
        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;

        // Step 3: Verify exactly one labeled message was injected.
        let op = sq_rx
            .try_recv()
            .expect("one labeled message should be injected after the delay");
        match op {
            SessionOp::Submit { message } => {
                assert!(
                    message.starts_with(SCHEDULED_FOLLOWUP_LABEL),
                    "injected message must carry the scheduled-followup source label"
                );
                assert!(
                    message.contains("check on the build status"),
                    "injected message must contain the original text"
                );
            }
            other => panic!("expected SessionOp::Submit, got {other:?}"),
        }

        // Step 4: Verify no second injection (one-shot fires exactly once).
        yield_times(10).await;
        assert!(
            sq_rx.try_recv().is_err(),
            "one-shot must fire exactly once — no duplicate injection"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_end_to_end_permission_is_fresh_per_call() {
        // This test proves that the delay tool's registration approval
        // does not pre-approve any subsequent tool call. The injected
        // message is just a String — any tool call derived from it will
        // go through the normal permission pipeline.
        //
        // We verify this by confirming the injected SessionOp is a plain
        // Submit with a String message, carrying no permission grant,
        // no tool call, and no pre-approval metadata.

        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();

        let (tools, pending) = create_scheduler_tools();
        let tool = tools[0].clone();

        let _join = pending.spawn(sq_tx, cancel_token);

        tool.execute(serde_json::json!({
            "message": "verify permissions",
            "delay_secs": 1
        }))
        .await;

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;

        let op = sq_rx.try_recv().expect("message should be injected");
        match op {
            SessionOp::Submit { message } => {
                // The injected op is a plain String message.
                // It carries no permission state, no tool call,
                // and no pre-approval. The session actor will treat it
                // identically to a user-typed message — any tool call
                // in the resulting turn gets a fresh permission decision.
                assert!(message.starts_with(SCHEDULED_FOLLOWUP_LABEL));
            }
            _ => panic!("expected SessionOp::Submit"),
        }
    }

    // ── Fixture-provider test through real Agent/session path (SF103 re-review) ─

    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering as AtomicOrdering;

    use talos_core::message::{AgentEvent, Message, StopReason, Usage};
    use talos_core::provider::{LanguageModel, ProviderResult};
    use talos_core::session::{RuntimePolicy, SessionConfig};
    use talos_core::tool::{ToolPermissionFacet, ToolRegistry, ToolResourceKind};
    use talos_permission::PermissionEngine;
    use talos_plugin::HookRegistry;

    use crate::Agent;
    use crate::session::AppServerSession;

    /// Minimal mock model that returns pre-configured event vectors, one per
    /// `stream()` call. Pattern replicated from `session/tests.rs`.
    struct MockLanguageModel {
        responses: Arc<Mutex<VecDeque<Vec<AgentEvent>>>>,
        observed_requests: Arc<Mutex<Vec<Vec<Message>>>>,
    }

    impl MockLanguageModel {
        fn new(responses: Vec<Vec<AgentEvent>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
                observed_requests: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl LanguageModel for MockLanguageModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            self.observed_requests
                .lock()
                .unwrap()
                .push(messages.to_vec());
            let (tx, rx) = mpsc::channel(64);
            let events = {
                self.responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .unwrap_or_default()
            };
            tokio::spawn(async move {
                for event in events {
                    let _ = tx.send(event).await;
                }
            });
            Ok(rx)
        }
    }

    /// Test tool with a resource-tagged permission facet so the engine can
    /// deny it specifically while allowing the delay tool.
    struct TrackingTool {
        executed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl AgentTool for TrackingTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Test tool"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({"type": "object", "properties": {}})
        }
        async fn execute(&self, _input: serde_json::Value) -> ToolResult {
            self.executed.store(true, AtomicOrdering::SeqCst);
            ToolResult::success("executed")
        }
        fn nature(&self) -> ToolNature {
            ToolNature::Execute
        }
        fn permission_profile(&self, _input: &serde_json::Value) -> Vec<ToolPermissionFacet> {
            vec![ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "test:echo".to_string(),
                ToolResourceKind::Remote,
            )]
        }
    }

    /// Proves: (1) delay fires through the real Agent/session path, (2) the
    /// follow-up tool call receives an independent Deny decision — not
    /// inherited from the delay tool's execution.
    ///
    /// The delay tool has Execute with no resource → engine returns Ask →
    /// falls through (no wrapper in this agent-level test). The echo tool has
    /// Execute with resource "test:echo" → engine returns Deny → blocked.
    /// If the delay's effective Allow were inherited, echo would execute.
    /// Echo NOT executing proves a fresh, independent Deny.
    #[tokio::test(start_paused = true)]
    async fn fixture_provider_delay_fires_and_follow_up_gets_fresh_deny() {
        let echo_executed = Arc::new(AtomicBool::new(false));

        let (tools, sched_pending) = create_scheduler_tools();
        let delay_tool = tools[0].clone();

        let mut registry = ToolRegistry::new();
        registry.register(delay_tool);
        registry.register(Arc::new(TrackingTool {
            executed: echo_executed.clone(),
        }));

        let model = MockLanguageModel::new(vec![
            // Turn 1a: delay tool call
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "call_1".into(),
                        name: "delay".into(),
                        input: serde_json::json!({"message": "check", "delay_secs": 1}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ],
            // Turn 1b: text after delay result
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Scheduled.".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ],
            // Turn 2a: echo tool call (from scheduled follow-up)
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "call_2".into(),
                        name: "echo".into(),
                        input: serde_json::json!({}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ],
            // Turn 2b: text after echo result (denied)
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Echo was denied.".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ],
        ]);
        let remaining_responses = model.responses.clone();
        let observed_requests = model.observed_requests.clone();

        // Deny "test:echo" resource; delay has no resource so default Ask applies
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": {"Deny": "echo denied by fresh permission decision"},
                    "nature": "Execute",
                    "resource": "test:echo",
                    "resource_kind": "remote"
                }]
            }))
            .unwrap();

        let hooks = Arc::new(HookRegistry::new());
        let agent = Agent::with_security_and_hooks(
            Arc::new(model),
            registry,
            Some(Arc::new(engine)),
            None,
            PathBuf::from("/tmp"),
            hooks,
        );

        let config = SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);
        let _sched_join = sched_pending.spawn(handle.sq_tx.clone(), CancellationToken::new());

        let sq_tx = handle.sq_tx.clone();
        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "schedule a check".into(),
            })
            .await
            .unwrap();

        // Let turn 1 process
        yield_times(30).await;

        // Advance time past the 1-second delay
        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(30).await;

        assert_eq!(
            remaining_responses.lock().unwrap().len(),
            0,
            "all four provider responses must be consumed, proving the scheduled turn completed"
        );
        let scheduled_message_observed = observed_requests.lock().unwrap().iter().any(|request| {
            request.iter().any(|message| {
                matches!(
                    message,
                    Message::User { content }
                        if content.starts_with(SCHEDULED_FOLLOWUP_LABEL)
                )
            })
        });
        assert!(
            scheduled_message_observed,
            "a provider request must contain the labeled scheduled follow-up message"
        );

        // The scheduled turn ran, but the echo tool must NOT be executed: it
        // received an independent Deny decision rather than inherited approval.
        assert!(
            !echo_executed.load(AtomicOrdering::SeqCst),
            "echo must NOT execute — it received a fresh Deny decision in \
             the follow-up turn, independent from the delay tool's execution"
        );

        let _ = sq_tx.send(SessionOp::Shutdown).await;
        let _ = actor_task.await;
    }

    /// Proves: (1) recurring fire reaches the provider through the full
    /// Agent/session pipeline, (2) the follow-up turn's tool call receives
    /// an independent Deny decision — schedule approval is not reused.
    #[tokio::test(start_paused = true)]
    async fn fixture_provider_recurring_fires_and_follow_up_gets_fresh_deny() {
        let echo_executed = Arc::new(AtomicBool::new(false));

        let (tools, sched_pending) = create_scheduler_tools();
        let schedule_tool = tools[1].clone();

        let mut registry = ToolRegistry::new();
        registry.register(schedule_tool);
        registry.register(Arc::new(TrackingTool {
            executed: echo_executed.clone(),
        }));

        let model = MockLanguageModel::new(vec![
            // Turn 1a: schedule tool call
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "call_1".into(),
                        name: "schedule".into(),
                        input: serde_json::json!({"message": "check", "interval_secs": 5}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ],
            // Turn 1b: text after schedule result
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Recurring check scheduled.".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ],
            // Turn 2a: echo tool call (from recurring fire)
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: talos_core::message::ToolCall {
                        id: "call_2".into(),
                        name: "echo".into(),
                        input: serde_json::json!({}),
                    },
                    provenance: Default::default(),
                    summary_fields: vec![],
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: Usage::default(),
                },
            ],
            // Turn 2b: text after echo denied
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Echo was denied.".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ],
        ]);
        let observed_requests = model.observed_requests.clone();

        // Deny "test:echo"; schedule has no resource so default Ask applies
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": {"Deny": "echo denied by fresh recurring permission"},
                    "nature": "Execute",
                    "resource": "test:echo",
                    "resource_kind": "remote"
                }]
            }))
            .unwrap();

        let hooks = Arc::new(HookRegistry::new());
        let agent = Agent::with_security_and_hooks(
            Arc::new(model),
            registry,
            Some(Arc::new(engine)),
            None,
            PathBuf::from("/tmp"),
            hooks,
        );

        let config = SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: "/tmp".into(),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);
        let _sched_join = sched_pending.spawn(handle.sq_tx.clone(), CancellationToken::new());

        let sq_tx = handle.sq_tx.clone();
        let actor_task = tokio::spawn(async move { actor.run().await });

        sq_tx
            .send(SessionOp::Submit {
                message: "set up recurring check".into(),
            })
            .await
            .unwrap();

        yield_times(30).await;

        // Advance past the first 5-second interval
        tokio::time::advance(Duration::from_secs(6)).await;
        yield_times(30).await;

        // (1) Recurring fire reached the provider
        let recurring_fire_reached_provider =
            observed_requests.lock().unwrap().iter().any(|request| {
                request.iter().any(|message| {
                    matches!(
                        message,
                        Message::User { content }
                            if content.starts_with(SCHEDULED_FOLLOWUP_LABEL)
                    )
                })
            });
        assert!(
            recurring_fire_reached_provider,
            "recurring fire must reach the provider through the full session pipeline"
        );

        // (2) The follow-up echo tool must NOT execute — it received an
        // independent Deny, proving schedule approval was not reused.
        assert!(
            !echo_executed.load(AtomicOrdering::SeqCst),
            "echo must NOT execute — it received a fresh independent Deny in \
             the recurring follow-up turn, not inherited from schedule approval"
        );

        let _ = sq_tx.send(SessionOp::Shutdown).await;
        let _ = actor_task.await;
    }
}
