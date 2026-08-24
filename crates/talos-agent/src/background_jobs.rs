//! Agent/session-owned live background job supervision (ADR-060).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use talos_core::background_job::{
    BackgroundCleanupOutcome, BackgroundJobHost, BackgroundJobId, BackgroundJobLauncher,
    BackgroundJobPermit, BackgroundJobRequest, BackgroundJobState, BackgroundJobTerminalSummary,
    BackgroundOutputChunk, BackgroundOutputStream, BackgroundProcessControl,
    BackgroundProcessEvent, MAX_BACKGROUND_OUTPUT_BYTES, MAX_NONTERMINAL_BACKGROUND_JOBS,
    MAX_TERMINAL_BACKGROUND_JOBS,
};
use talos_core::session::SessionEvent;
use talos_core::tool::ToolResult;
use tokio::sync::{Notify, mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct SessionIdentity {
    id: String,
    generation: u64,
}

struct RetainedChunk {
    cursor: u64,
    #[allow(dead_code)] // Exposed to TOOL-024-C's explicit process read projection.
    stream: BackgroundOutputStream,
    bytes: Vec<u8>,
    #[allow(dead_code)] // Exposed to TOOL-024-C's explicit process read projection.
    captured_at_unix_ms: u64,
}

struct JobRecord {
    id: BackgroundJobId,
    tool_name: String,
    state: BackgroundJobState,
    chunks: VecDeque<RetainedChunk>,
    retained_bytes: usize,
    next_cursor: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    truncated: bool,
    exit_code: Option<i32>,
    started_at_unix_ms: u64,
    finished_at_unix_ms: Option<u64>,
    cleanup_outcome: BackgroundCleanupOutcome,
    cleanup_error: Option<String>,
}

impl JobRecord {
    fn starting(id: BackgroundJobId, tool_name: String, started_at_unix_ms: u64) -> Self {
        Self {
            id,
            tool_name,
            state: BackgroundJobState::Starting,
            chunks: VecDeque::new(),
            retained_bytes: 0,
            next_cursor: 0,
            stdout_bytes: 0,
            stderr_bytes: 0,
            truncated: false,
            exit_code: None,
            started_at_unix_ms,
            finished_at_unix_ms: None,
            cleanup_outcome: BackgroundCleanupOutcome::Natural,
            cleanup_error: None,
        }
    }

    fn push_output(&mut self, chunk: BackgroundOutputChunk) {
        let byte_count = chunk.bytes.len() as u64;
        match chunk.stream {
            BackgroundOutputStream::Stdout => {
                self.stdout_bytes = self.stdout_bytes.saturating_add(byte_count)
            }
            BackgroundOutputStream::Stderr => {
                self.stderr_bytes = self.stderr_bytes.saturating_add(byte_count)
            }
        }
        self.retained_bytes = self.retained_bytes.saturating_add(chunk.bytes.len());
        self.chunks.push_back(RetainedChunk {
            cursor: self.next_cursor,
            stream: chunk.stream,
            bytes: chunk.bytes,
            captured_at_unix_ms: unix_millis(chunk.captured_at),
        });
        self.next_cursor = self.next_cursor.saturating_add(1);
        while self.retained_bytes > MAX_BACKGROUND_OUTPUT_BYTES {
            let Some(chunk) = self.chunks.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(chunk.bytes.len());
            self.truncated = true;
        }
    }

    fn summary(&self) -> BackgroundJobTerminalSummary {
        BackgroundJobTerminalSummary {
            job_id: self.id.clone(),
            tool_name: self.tool_name.clone(),
            state: self.state,
            exit_code: self.exit_code,
            stdout_bytes: self.stdout_bytes,
            stderr_bytes: self.stderr_bytes,
            earliest_cursor: self
                .chunks
                .front()
                .map_or(self.next_cursor, |chunk| chunk.cursor),
            next_cursor: self.next_cursor,
            truncated: self.truncated,
            started_at_unix_ms: self.started_at_unix_ms,
            finished_at_unix_ms: self.finished_at_unix_ms.unwrap_or(self.started_at_unix_ms),
            cleanup_outcome: self.cleanup_outcome,
            cleanup_error: self.cleanup_error.clone(),
        }
    }
}

struct SupervisorState {
    closing: bool,
    reserved: usize,
    active: usize,
    jobs: HashMap<BackgroundJobId, JobRecord>,
    terminal_order: VecDeque<BackgroundJobId>,
}

struct ShutdownState {
    started: bool,
    result: Option<Result<(), String>>,
}

struct SupervisorInner {
    state: Mutex<SupervisorState>,
    identity: RwLock<SessionIdentity>,
    event_tx: mpsc::UnboundedSender<SessionEvent>,
    shutdown_token: CancellationToken,
    state_changed: Notify,
    shutdown_state: Mutex<ShutdownState>,
    shutdown_complete: Notify,
}

/// One live Agent/session background supervisor.
#[derive(Clone)]
pub(crate) struct BackgroundJobSupervisor {
    inner: Arc<SupervisorInner>,
}

impl BackgroundJobSupervisor {
    pub(crate) fn new(
        event_tx: mpsc::UnboundedSender<SessionEvent>,
        session_id: String,
        generation: u64,
    ) -> Self {
        Self {
            inner: Arc::new(SupervisorInner {
                state: Mutex::new(SupervisorState {
                    closing: false,
                    reserved: 0,
                    active: 0,
                    jobs: HashMap::new(),
                    terminal_order: VecDeque::new(),
                }),
                identity: RwLock::new(SessionIdentity {
                    id: session_id,
                    generation,
                }),
                event_tx,
                shutdown_token: CancellationToken::new(),
                state_changed: Notify::new(),
                shutdown_state: Mutex::new(ShutdownState {
                    started: false,
                    result: None,
                }),
                shutdown_complete: Notify::new(),
            }),
        }
    }

    pub(crate) fn set_identity(&self, session_id: String, generation: u64) {
        if let Ok(mut identity) = self.inner.identity.write() {
            identity.id = session_id;
            identity.generation = generation;
        }
    }

    pub(crate) fn finalizer_handle(&self) -> BackgroundJobFinalizerHandle {
        BackgroundJobFinalizerHandle {
            supervisor: self.clone(),
        }
    }

    pub(crate) fn begin_shutdown(&self) {
        let should_start = self
            .inner
            .shutdown_state
            .lock()
            .map(|mut state| {
                if state.started {
                    false
                } else {
                    state.started = true;
                    true
                }
            })
            .unwrap_or(false);
        if !should_start {
            return;
        }

        if let Ok(mut state) = self.inner.state.lock() {
            state.closing = true;
        }
        self.inner.shutdown_token.cancel();

        let supervisor = self.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let result = supervisor.wait_for_shutdown_completion().await;
                if let Ok(mut state) = supervisor.inner.shutdown_state.lock() {
                    state.result = Some(result);
                }
                supervisor.inner.shutdown_complete.notify_waiters();
            });
        } else if let Ok(mut state) = self.inner.shutdown_state.lock() {
            state.result = Some(Err(
                "background cleanup could not start without a Tokio runtime".to_owned(),
            ));
            self.inner.shutdown_complete.notify_waiters();
        }
    }

    async fn wait_for_shutdown_completion(&self) -> Result<(), String> {
        loop {
            let status = {
                match self.inner.state.lock() {
                    Ok(state) => {
                        let pending = state.reserved.saturating_add(state.active);
                        let errors = state
                            .jobs
                            .values()
                            .filter(|job| {
                                job.cleanup_outcome == BackgroundCleanupOutcome::Incomplete
                            })
                            .filter_map(|job| {
                                job.cleanup_error
                                    .as_ref()
                                    .map(|error| format!("{}: {error}", job.id))
                            })
                            .collect::<Vec<_>>();
                        Some((pending, errors))
                    }
                    Err(_) => None,
                }
            };
            match status {
                Some((0, errors)) if errors.is_empty() => return Ok(()),
                Some((0, errors)) => return Err(errors.join("; ")),
                Some(_) => self.inner.state_changed.notified().await,
                None => return Err("background supervisor state is unavailable".to_owned()),
            }
        }
    }

    pub(crate) async fn finalize(&self) -> Result<(), String> {
        self.begin_shutdown();
        loop {
            if let Some(result) = self
                .inner
                .shutdown_state
                .lock()
                .ok()
                .and_then(|state| state.result.clone())
            {
                return result;
            }
            self.inner.shutdown_complete.notified().await;
        }
    }

    fn release_reservation(&self, id: &BackgroundJobId) {
        if let Ok(mut state) = self.inner.state.lock()
            && state.jobs.remove(id).is_some()
        {
            state.reserved = state.reserved.saturating_sub(1);
        }
        self.inner.state_changed.notify_waiters();
    }

    async fn commit_launch(
        &self,
        id: BackgroundJobId,
        deadline: Instant,
        timeout_secs: u64,
        launcher: Box<dyn BackgroundJobLauncher>,
    ) -> ToolResult {
        let can_launch = {
            match self.inner.state.lock() {
                Ok(mut state) => {
                    state.reserved = state.reserved.saturating_sub(1);
                    if state.closing {
                        state.jobs.remove(&id);
                        false
                    } else {
                        state.active = state.active.saturating_add(1);
                        true
                    }
                }
                Err(_) => false,
            }
        };
        if !can_launch {
            self.inner.state_changed.notify_waiters();
            return ToolResult::error("background job admission is closed");
        }

        let launched = match launcher.launch().await {
            Ok(launched) => launched,
            Err(error) => {
                let summary = self.terminalize(
                    &id,
                    BackgroundJobState::SpawnFailed,
                    None,
                    BackgroundCleanupOutcome::Natural,
                    Some(error.clone()),
                );
                self.emit_summary(summary);
                return ToolResult::error(format!("background spawn failed ({id}): {error}"));
            }
        };

        if let Ok(mut state) = self.inner.state.lock()
            && let Some(job) = state.jobs.get_mut(&id)
        {
            job.state = BackgroundJobState::Running;
        }

        let supervisor = self.clone();
        let task_id = id.clone();
        tokio::spawn(async move {
            supervisor
                .supervise(task_id, deadline, launched.control, launched.events)
                .await;
        });

        let tool_name = self
            .inner
            .state
            .lock()
            .ok()
            .and_then(|state| state.jobs.get(&id).map(|job| job.tool_name.clone()))
            .unwrap_or_default();
        ToolResult::success(
            serde_json::json!({
                "job_id": id.as_str(),
                "state": "running",
                "tool": tool_name,
                "deadline_secs": timeout_secs,
            })
            .to_string(),
        )
    }

    async fn supervise(
        &self,
        id: BackgroundJobId,
        deadline: Instant,
        control: Arc<dyn BackgroundProcessControl>,
        mut events: mpsc::Receiver<BackgroundProcessEvent>,
    ) {
        let terminal = loop {
            tokio::select! {
                biased;
                _ = self.inner.shutdown_token.cancelled() => {
                    break self.cleanup_after_signal(
                        &id,
                        BackgroundJobState::Cancelled,
                        control.as_ref(),
                        &mut events,
                        None,
                    ).await;
                }
                _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                    break self.cleanup_after_signal(
                        &id,
                        BackgroundJobState::TimedOut,
                        control.as_ref(),
                        &mut events,
                        None,
                    ).await;
                }
                event = events.recv() => match event {
                    Some(BackgroundProcessEvent::Output(chunk)) => self.push_output(&id, chunk),
                    Some(BackgroundProcessEvent::Exited(exit)) => {
                        let state = if exit.success {
                            BackgroundJobState::Completed
                        } else {
                            BackgroundJobState::Failed
                        };
                        break (state, exit.code, BackgroundCleanupOutcome::Natural, None);
                    }
                    Some(BackgroundProcessEvent::SupervisionFailed(error)) => {
                        break self.cleanup_after_signal(
                            &id,
                            BackgroundJobState::SupervisionFailed,
                            control.as_ref(),
                            &mut events,
                            Some(error),
                        ).await;
                    }
                    None => break (
                        BackgroundJobState::SupervisionFailed,
                        None,
                        BackgroundCleanupOutcome::Incomplete,
                        Some("background process event channel closed before exit".to_owned()),
                    ),
                }
            }
        };

        let summary = self.terminalize(&id, terminal.0, terminal.1, terminal.2, terminal.3);
        self.emit_summary(summary);
    }

    async fn cleanup_after_signal(
        &self,
        id: &BackgroundJobId,
        state: BackgroundJobState,
        control: &dyn BackgroundProcessControl,
        events: &mut mpsc::Receiver<BackgroundProcessEvent>,
        mut error: Option<String>,
    ) -> (
        BackgroundJobState,
        Option<i32>,
        BackgroundCleanupOutcome,
        Option<String>,
    ) {
        if let Err(signal_error) = control.terminate().await {
            append_error(&mut error, signal_error);
        }
        let grace = tokio::time::sleep(Duration::from_secs(2));
        tokio::pin!(grace);
        loop {
            tokio::select! {
                _ = &mut grace => break,
                event = events.recv() => match event {
                    Some(BackgroundProcessEvent::Output(chunk)) => self.push_output(id, chunk),
                    Some(BackgroundProcessEvent::Exited(exit)) => {
                        return (state, exit.code, BackgroundCleanupOutcome::Terminated, error);
                    }
                    Some(BackgroundProcessEvent::SupervisionFailed(event_error)) => {
                        append_error(&mut error, event_error);
                    }
                    None => {
                        append_error(&mut error, "process event channel closed before reap".to_owned());
                        return (state, None, BackgroundCleanupOutcome::Incomplete, error);
                    }
                }
            }
        }

        if let Err(signal_error) = control.force_terminate().await {
            append_error(&mut error, signal_error);
        }
        let force_grace = tokio::time::sleep(Duration::from_secs(2));
        tokio::pin!(force_grace);
        loop {
            tokio::select! {
                _ = &mut force_grace => {
                    append_error(&mut error, "process reap exceeded cleanup bound".to_owned());
                    break (state, None, BackgroundCleanupOutcome::Incomplete, error);
                }
                event = events.recv() => match event {
                    Some(BackgroundProcessEvent::Output(chunk)) => self.push_output(id, chunk),
                    Some(BackgroundProcessEvent::Exited(exit)) => {
                        break (
                            state,
                            exit.code,
                            if error.is_some() { BackgroundCleanupOutcome::Incomplete }
                            else { BackgroundCleanupOutcome::ForceTerminated },
                            error,
                        );
                    }
                    Some(BackgroundProcessEvent::SupervisionFailed(event_error)) => append_error(&mut error, event_error),
                    None => {
                        append_error(&mut error, "process event channel closed before reap".to_owned());
                        break (state, None, BackgroundCleanupOutcome::Incomplete, error);
                    }
                }
            }
        }
    }

    fn push_output(&self, id: &BackgroundJobId, chunk: BackgroundOutputChunk) {
        if let Ok(mut state) = self.inner.state.lock()
            && let Some(job) = state.jobs.get_mut(id)
        {
            job.push_output(chunk);
        }
    }

    fn terminalize(
        &self,
        id: &BackgroundJobId,
        state_value: BackgroundJobState,
        exit_code: Option<i32>,
        cleanup_outcome: BackgroundCleanupOutcome,
        cleanup_error: Option<String>,
    ) -> Option<BackgroundJobTerminalSummary> {
        let summary = self.inner.state.lock().ok().and_then(|mut state| {
            let job = state.jobs.get_mut(id)?;
            if job.state.is_terminal() {
                return None;
            }
            job.state = state_value;
            job.exit_code = exit_code;
            job.finished_at_unix_ms = Some(unix_millis(SystemTime::now()));
            job.cleanup_outcome = cleanup_outcome;
            job.cleanup_error = cleanup_error;
            let summary = job.summary();
            state.active = state.active.saturating_sub(1);
            state.terminal_order.push_back(id.clone());
            while state.terminal_order.len() > MAX_TERMINAL_BACKGROUND_JOBS {
                if let Some(expired) = state.terminal_order.pop_front() {
                    state.jobs.remove(&expired);
                }
            }
            Some(summary)
        });
        self.inner.state_changed.notify_waiters();
        summary
    }

    fn emit_summary(&self, summary: Option<BackgroundJobTerminalSummary>) {
        let Some(summary) = summary else {
            return;
        };
        let identity = self
            .inner
            .identity
            .read()
            .ok()
            .map(|identity| identity.clone());
        if let Some(identity) = identity {
            let _ = self
                .inner
                .event_tx
                .send(SessionEvent::BackgroundJobTerminal {
                    session_id: identity.id,
                    session_generation: identity.generation,
                    summary,
                });
        }
    }
}

#[async_trait]
impl BackgroundJobHost for BackgroundJobSupervisor {
    async fn reserve(
        &self,
        request: BackgroundJobRequest,
    ) -> Result<Box<dyn BackgroundJobPermit>, String> {
        let id = BackgroundJobId::new(format!("job_{}", uuid::Uuid::new_v4()));
        let deadline = Instant::now() + request.timeout;
        let timeout_secs = request.timeout.as_secs();
        let started_at_unix_ms = unix_millis(SystemTime::now());
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| "background supervisor state is unavailable".to_owned())?;
        if state.closing {
            return Err("background job admission is closed".to_owned());
        }
        if state.reserved.saturating_add(state.active) >= MAX_NONTERMINAL_BACKGROUND_JOBS {
            return Err("background job capacity exhausted".to_owned());
        }
        state.reserved = state.reserved.saturating_add(1);
        state.jobs.insert(
            id.clone(),
            JobRecord::starting(id.clone(), request.tool_name.clone(), started_at_unix_ms),
        );
        drop(state);
        Ok(Box::new(BackgroundStartPermit {
            supervisor: self.clone(),
            id: Some(id),
            deadline,
            timeout_secs,
            committed: false,
        }))
    }
}

struct BackgroundStartPermit {
    supervisor: BackgroundJobSupervisor,
    id: Option<BackgroundJobId>,
    deadline: Instant,
    timeout_secs: u64,
    committed: bool,
}

#[async_trait]
impl BackgroundJobPermit for BackgroundStartPermit {
    async fn launch(mut self: Box<Self>, launcher: Box<dyn BackgroundJobLauncher>) -> ToolResult {
        let Some(id) = self.id.take() else {
            return ToolResult::error("background reservation is unavailable");
        };
        self.committed = true;
        self.supervisor
            .commit_launch(id, self.deadline, self.timeout_secs, launcher)
            .await
    }
}

impl Drop for BackgroundStartPermit {
    fn drop(&mut self) {
        if !self.committed
            && let Some(id) = self.id.take()
        {
            self.supervisor.release_reservation(&id);
        }
    }
}

/// Cloneable shutdown handle consumed by the Runtime finalizer registry.
#[doc(hidden)]
#[derive(Clone)]
pub struct BackgroundJobFinalizerHandle {
    supervisor: BackgroundJobSupervisor,
}

impl BackgroundJobFinalizerHandle {
    /// Starts or joins the idempotent background cleanup driver.
    pub async fn finalize(&self) -> Result<(), String> {
        self.supervisor.finalize().await
    }
}

fn append_error(target: &mut Option<String>, error: String) {
    match target {
        Some(existing) => {
            existing.push_str("; ");
            existing.push_str(&error);
        }
        None => *target = Some(error),
    }
}

fn unix_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_chunk_fields_remain_available_for_process_tool_follow_up() {
        let chunk = RetainedChunk {
            cursor: 3,
            stream: BackgroundOutputStream::Stderr,
            bytes: b"err".to_vec(),
            captured_at_unix_ms: 7,
        };
        assert_eq!(chunk.cursor, 3);
        assert_eq!(chunk.stream, BackgroundOutputStream::Stderr);
        assert_eq!(chunk.captured_at_unix_ms, 7);
    }
}
