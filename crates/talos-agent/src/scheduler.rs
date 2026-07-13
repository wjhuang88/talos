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
    /// Fires exactly once after the specified delay.
    OneShot,
}

impl fmt::Display for ScheduleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScheduleKind::OneShot => write!(f, "one-shot"),
        }
    }
}

// ── Task metadata ───────────────────────────────────────────────────────

/// Snapshot of an active scheduled task.
///
/// Returned by [`ScheduleCommand::List`] for read-only inspection. Contains
/// only bounded, non-sensitive information.
#[derive(Debug, Clone)]
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
/// The struct is `pub` so it can appear in public function signatures (e.g.,
/// tool factory functions), but its field is private and construction is
/// `pub(crate)` — only `talos-agent` internal code can create one.
#[derive(Clone)]
pub struct SchedulerHandle {
    /// Bounded command channel sender. When the actor is shut down, this
    /// sender returns a `SendError`, which callers must handle gracefully.
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

/// Creates a scheduler handle and a pending actor for two-phase composition.
///
/// Composition roots must create the scheduler BEFORE the session (to register
/// the delay tool) but can only spawn the actor AFTER the session provides
/// `sq_tx`. This function splits those two steps.
///
/// Typical usage:
/// ```
/// # use talos_agent::scheduler::create_scheduler;
/// # let mut registry = talos_core::tool::ToolRegistry::new();
/// let (handle, pending) = create_scheduler();
/// registry.register(std::sync::Arc::new(
///     talos_agent::scheduler::DelayTool::new(handle),
/// ));
/// // ... create agent + session ...
/// // pending.spawn(session_handle.sq_tx.clone(), cancel_token);
/// ```
pub fn create_scheduler() -> (SchedulerHandle, PendingSchedulerActor) {
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let handle = SchedulerHandle::new(cmd_tx);
    let pending = PendingSchedulerActor { cmd_rx };
    (handle, pending)
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
pub struct DelayTool {
    handle: SchedulerHandle,
}

impl DelayTool {
    /// Creates a new delay tool linked to the scheduler actor.
    pub fn new(handle: SchedulerHandle) -> Self {
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

    // ── DelayTool unit tests (SF103) ────────────────────────────────────

    #[test]
    fn delay_tool_nature_is_execute() {
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);
        assert_eq!(tool.nature(), talos_core::tool::ToolNature::Execute);
    }

    #[tokio::test(start_paused = true)]
    async fn delay_tool_executes_and_returns_task_id() {
        let (sq_tx, _sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();
        let (handle, pending) = create_scheduler();
        let tool = DelayTool::new(handle);
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
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        let (handle, _pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        let (handle, pending) = create_scheduler();
        let tool = DelayTool::new(handle);

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
        use talos_core::tool::AgentTool;

        let (sq_tx, mut sq_rx) = mpsc::channel(512);
        let cancel_token = CancellationToken::new();

        let (handle, pending) = create_scheduler();
        let tool = DelayTool::new(handle);
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

        let (handle, pending) = create_scheduler();
        let tool = DelayTool::new(handle);
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
}
