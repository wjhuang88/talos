//! Live-session background job contracts.
//!
//! The Agent/session owns admission and supervision. Tools own only validated
//! platform launch and process-tree control primitives.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::tool::ToolResult;

/// Maximum number of non-terminal jobs owned by one live session.
pub const MAX_NONTERMINAL_BACKGROUND_JOBS: usize = 8;
/// Maximum number of terminal job summaries retained by one live session.
pub const MAX_TERMINAL_BACKGROUND_JOBS: usize = 32;
/// Maximum combined stdout/stderr bytes retained for one job.
pub const MAX_BACKGROUND_OUTPUT_BYTES: usize = 64 * 1024;
/// Maximum number of process events buffered between a launcher and supervisor.
pub const BACKGROUND_PROCESS_EVENT_CAPACITY: usize = 64;

/// Opaque identifier scoped to one live Agent/session supervisor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct BackgroundJobId(String);

impl BackgroundJobId {
    /// Creates an identifier from a supervisor-generated opaque value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the opaque display value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BackgroundJobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Monotonic state of a live-session background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobState {
    /// Capacity is reserved and platform launch is in progress.
    Starting,
    /// The process was launched and is being supervised.
    Running,
    /// The process exited successfully.
    Completed,
    /// The process exited unsuccessfully.
    Failed,
    /// The absolute job deadline elapsed.
    TimedOut,
    /// Explicit cancellation or session shutdown won the terminal race.
    Cancelled,
    /// The process could not be launched.
    SpawnFailed,
    /// Output or process supervision failed.
    SupervisionFailed,
}

impl BackgroundJobState {
    /// Returns whether no later state transition is permitted.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Starting | Self::Running)
    }
}

/// Origin stream for one ordered output chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundOutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One bounded output fragment emitted by a platform launcher.
#[derive(Debug)]
pub struct BackgroundOutputChunk {
    /// Origin stream.
    pub stream: BackgroundOutputStream,
    /// Exact bytes read from the pipe.
    pub bytes: Vec<u8>,
    /// Capture timestamp.
    pub captured_at: SystemTime,
}

/// Platform-neutral process exit observation after the leader is reaped.
#[derive(Debug, Clone, Copy)]
pub struct BackgroundProcessExit {
    /// Platform exit code when available.
    pub code: Option<i32>,
    /// Whether the platform status represents success.
    pub success: bool,
}

/// Ordered launcher-to-supervisor event.
#[derive(Debug)]
pub enum BackgroundProcessEvent {
    /// Captured stdout or stderr bytes.
    Output(BackgroundOutputChunk),
    /// The process leader was reaped and pipe readers completed.
    Exited(BackgroundProcessExit),
    /// A pipe reader or leader wait failed.
    SupervisionFailed(String),
}

/// Checked platform termination primitive for one owned process tree/group.
#[async_trait]
pub trait BackgroundProcessControl: Send + Sync {
    /// Requests graceful group/tree termination.
    async fn terminate(&self) -> Result<(), String>;
    /// Requests forceful group/tree termination.
    async fn force_terminate(&self) -> Result<(), String>;
}

/// Result of a successful platform launch.
pub struct LaunchedBackgroundJob {
    /// Checked process-tree control.
    pub control: Arc<dyn BackgroundProcessControl>,
    /// Bounded, globally ordered output/exit event stream.
    pub events: mpsc::Receiver<BackgroundProcessEvent>,
}

/// Tool-owned platform launcher invoked only after permission admission.
#[async_trait]
pub trait BackgroundJobLauncher: Send {
    /// Launches one supported process shape.
    async fn launch(self: Box<Self>) -> Result<LaunchedBackgroundJob, String>;
}

/// Semantic execution mode selected before permission evaluation.
pub enum ToolExecutionAdmission {
    /// Preserve the existing synchronous tool path.
    Foreground,
    /// Reserve and start one supervised background job.
    Background(BackgroundJobRequest),
}

/// Semantically admitted background start request.
pub struct BackgroundJobRequest {
    /// Stable tool identity (`bash` or `exec` in TOOL-024-B).
    pub tool_name: String,
    /// Absolute lifetime, clamped by the tool to 1-600 seconds.
    pub timeout: Duration,
    /// Exact permission resource required to control this job.
    pub background_resource: String,
}

/// One non-cloneable capacity reservation created before permission evaluation.
#[async_trait]
pub trait BackgroundJobPermit: Send {
    /// Commits the reserved start after permission succeeds.
    async fn launch(self: Box<Self>, launcher: Box<dyn BackgroundJobLauncher>) -> ToolResult;
}

/// Agent/session-owned host for background job admission and supervision.
#[async_trait]
pub trait BackgroundJobHost: Send + Sync {
    /// Reserves capacity and fixes the absolute deadline before permission evaluation.
    async fn reserve(
        &self,
        request: BackgroundJobRequest,
    ) -> Result<Box<dyn BackgroundJobPermit>, String>;
}

/// Observable cleanup result for one terminal job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundCleanupOutcome {
    /// The process exited without a cleanup signal.
    Natural,
    /// Graceful group termination completed.
    Terminated,
    /// Forceful group termination was required.
    ForceTerminated,
    /// Cleanup or reap failed and manual attention may be required.
    Incomplete,
}

/// UI-neutral terminal summary emitted once by the owning session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BackgroundJobTerminalSummary {
    /// Opaque live-session job identifier.
    pub job_id: BackgroundJobId,
    /// Tool that launched the job.
    pub tool_name: String,
    /// Winning terminal state.
    pub state: BackgroundJobState,
    /// Exit code when available.
    pub exit_code: Option<i32>,
    /// Total stdout bytes observed before terminalization.
    pub stdout_bytes: u64,
    /// Total stderr bytes observed before terminalization.
    pub stderr_bytes: u64,
    /// Earliest retained output cursor.
    pub earliest_cursor: u64,
    /// Cursor assigned to the next output chunk.
    pub next_cursor: u64,
    /// Whether output eviction occurred.
    pub truncated: bool,
    /// Milliseconds since the Unix epoch when admission was reserved.
    pub started_at_unix_ms: u64,
    /// Milliseconds since the Unix epoch when the job terminalized.
    pub finished_at_unix_ms: u64,
    /// Process cleanup result.
    pub cleanup_outcome: BackgroundCleanupOutcome,
    /// Display-safe cleanup or supervision failure.
    pub cleanup_error: Option<String>,
}
