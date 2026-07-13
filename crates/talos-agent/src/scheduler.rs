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

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};

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
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ScheduleCommand>(8);
        drop(cmd_rx); // close the receiver
        let handle = SchedulerHandle::new(cmd_tx);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let result = rt.block_on(handle.send(ScheduleCommand::Shutdown));
        assert!(result.is_err(), "send on closed channel should error");
    }
}
