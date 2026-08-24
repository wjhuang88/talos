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

/// Model-visible action for a live session-owned background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessAction {
    Read,
    Status,
    List,
    Cancel,
}

#[derive(Clone)]
struct SessionIdentity {
    id: String,
    generation: u64,
}

struct RetainedChunk {
    /// Monotonic byte offset within this job's combined output stream.
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
    control: Option<Arc<dyn BackgroundProcessControl>>,
    cancel_token: Option<CancellationToken>,
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
            control: None,
            // Cancellation begins at reservation time so a queued launch can be
            // cancelled before the platform launcher is invoked.
            cancel_token: Some(CancellationToken::new()),
        }
    }

    /// Appends one output chunk and advances the byte cursor by its exact size.
    fn push_output(&mut self, chunk: BackgroundOutputChunk) {
        if chunk.bytes.is_empty() {
            return;
        }
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
        self.next_cursor = self.next_cursor.saturating_add(byte_count);
        while self.retained_bytes > MAX_BACKGROUND_OUTPUT_BYTES {
            let Some(chunk) = self.chunks.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(chunk.bytes.len());
            self.truncated = true;
        }
    }

    fn is_terminal(&self) -> bool {
        self.state.is_terminal()
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
    shutdown_requested: bool,
    closing: bool,
    reserved: usize,
    active: usize,
    launching: usize,
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
                    shutdown_requested: false,
                    closing: false,
                    reserved: 0,
                    active: 0,
                    launching: 0,
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
            state.shutdown_requested = true;
        }
        self.inner.shutdown_token.cancel();

        let supervisor = self.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                supervisor.close_admission_after_launches().await;
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

    async fn close_admission_after_launches(&self) {
        loop {
            // Register the notification before observing state so a final launcher completion
            // cannot slip between the check and subscription.
            let notified = self.inner.state_changed.notified();
            let closed = self
                .inner
                .state
                .lock()
                .map(|mut state| {
                    if state.launching == 0 {
                        state.closing = true;
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(true);
            if closed {
                self.inner.state_changed.notify_waiters();
                return;
            }
            notified.await;
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
                    if state.closing || state.shutdown_requested {
                        state.jobs.remove(&id);
                        false
                    } else {
                        state.active = state.active.saturating_add(1);
                        state.launching = state.launching.saturating_add(1);
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
                if let Ok(mut state) = self.inner.state.lock() {
                    state.launching = state.launching.saturating_sub(1);
                }
                self.inner.state_changed.notify_waiters();
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

        if let Ok(mut state) = self.inner.state.lock() {
            state.launching = state.launching.saturating_sub(1);
        }
        self.inner.state_changed.notify_waiters();

        let cancel_token = self
            .inner
            .state
            .lock()
            .ok()
            .and_then(|state| state.jobs.get(&id).and_then(|job| job.cancel_token.clone()))
            .unwrap_or_default();
        if cancel_token.is_cancelled() {
            if let Ok(mut state) = self.inner.state.lock() {
                state.launching = state.launching.saturating_sub(1);
            }
            self.inner.state_changed.notify_waiters();
            let summary = self.terminalize(
                &id,
                BackgroundJobState::Cancelled,
                None,
                BackgroundCleanupOutcome::Natural,
                None,
            );
            self.emit_summary(summary);
            return ToolResult::success(
                serde_json::json!({"job_id": id.as_str(), "state": "cancelled"}).to_string(),
            );
        }
        if let Ok(mut state) = self.inner.state.lock()
            && let Some(job) = state.jobs.get_mut(&id)
        {
            job.state = BackgroundJobState::Running;
            job.control = Some(launched.control.clone());
            job.cancel_token = Some(cancel_token.clone());
        }

        let supervisor = self.clone();
        let task_id = id.clone();
        tokio::spawn(async move {
            supervisor
                .supervise(
                    task_id,
                    deadline,
                    launched.control,
                    cancel_token,
                    launched.events,
                )
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
        cancel_token: CancellationToken,
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
                _ = cancel_token.cancelled() => {
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
        self.inner.state_changed.notify_waiters();
    }

    pub(crate) async fn process_action(
        &self,
        action: ProcessAction,
        job_id: Option<&str>,
        cursor: Option<u64>,
        max_bytes: Option<usize>,
        wait_ms: Option<u64>,
    ) -> ToolResult {
        const MAX_READ_BYTES: usize = 64 * 1024;
        const MAX_WAIT_MS: u64 = 5_000;
        let max_bytes = max_bytes.unwrap_or(16 * 1024).clamp(1, MAX_READ_BYTES);
        let wait_ms = wait_ms.unwrap_or(0).min(MAX_WAIT_MS);
        match action {
            ProcessAction::List => self.process_list(max_bytes),
            ProcessAction::Status => job_id.map_or_else(
                || ToolResult::error("process status requires job_id"),
                |id| self.process_status(id),
            ),
            ProcessAction::Read => match job_id {
                Some(id) => {
                    self.process_read(id, cursor.unwrap_or(0), max_bytes, wait_ms)
                        .await
                }
                None => ToolResult::error("process read requires job_id"),
            },
            ProcessAction::Cancel => match job_id {
                Some(id) => self.process_cancel(id, wait_ms.max(5_000)).await,
                None => ToolResult::error("process cancel requires job_id"),
            },
        }
    }

    fn process_list(&self, max_bytes: usize) -> ToolResult {
        let mut jobs = self.inner.state.lock().ok().map_or_else(Vec::new, |state| {
            state.jobs.values().map(job_status_json).collect::<Vec<_>>()
        });
        jobs.sort_by(|a, b| {
            a.get("job_id")
                .and_then(serde_json::Value::as_str)
                .cmp(&b.get("job_id").and_then(serde_json::Value::as_str))
        });
        let mut truncated = false;
        while serde_json::to_vec(&jobs).is_ok_and(|bytes| bytes.len() > max_bytes) {
            truncated = true;
            if jobs.pop().is_none() {
                break;
            }
        }
        ToolResult::success(serde_json::json!({"jobs": jobs, "truncated": truncated}).to_string())
    }

    fn process_status(&self, job_id: &str) -> ToolResult {
        let Some(job) = self.inner.state.lock().ok().and_then(|state| {
            state
                .jobs
                .get(&BackgroundJobId::new(job_id))
                .map(job_status_json)
        }) else {
            return ToolResult::error("unknown background job");
        };
        ToolResult::success(job.to_string())
    }

    async fn process_read(
        &self,
        job_id: &str,
        cursor: u64,
        max_bytes: usize,
        wait_ms: u64,
    ) -> ToolResult {
        let deadline = tokio::time::Instant::now() + Duration::from_millis(wait_ms);
        loop {
            let notified = self.inner.state_changed.notified();
            match self.read_snapshot(job_id, cursor, max_bytes) {
                Ok((value, has_data, terminal)) if has_data || terminal || wait_ms == 0 => {
                    return ToolResult::success(value.to_string());
                }
                Err(error) => return ToolResult::error(error),
                Ok(_) => {}
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return self
                    .read_snapshot(job_id, cursor, max_bytes)
                    .map(|(value, _, _)| ToolResult::success(value.to_string()))
                    .unwrap_or_else(ToolResult::error);
            }
        }
    }

    fn read_snapshot(
        &self,
        job_id: &str,
        cursor: u64,
        max_bytes: usize,
    ) -> Result<(serde_json::Value, bool, bool), String> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| "background supervisor state is unavailable".to_owned())?;
        let Some(job) = state.jobs.get(&BackgroundJobId::new(job_id)) else {
            return Err("unknown background job".to_owned());
        };
        let earliest = job
            .chunks
            .front()
            .map_or(job.next_cursor, |chunk| chunk.cursor);
        let dropped_before = (cursor < earliest).then_some(earliest);
        let mut used = 0usize;
        let mut events = Vec::new();
        let mut read_cursor = cursor;
        let mut partial = false;
        for chunk in &job.chunks {
            if chunk.cursor.saturating_add(chunk.bytes.len() as u64) <= read_cursor {
                continue;
            }
            if used >= max_bytes {
                break;
            }
            let remaining = max_bytes - used;
            let offset = read_cursor.saturating_sub(chunk.cursor) as usize;
            let bytes = &chunk.bytes[offset..];
            let bytes = &bytes[..bytes.len().min(remaining)];
            let event_cursor = read_cursor;
            used = used.saturating_add(bytes.len());
            events.push(serde_json::json!({
                "seq": chunk.cursor,
                "cursor": event_cursor,
                "stream": chunk.stream,
                "text": String::from_utf8_lossy(bytes),
                "captured_at_unix_ms": chunk.captured_at_unix_ms,
            }));
            read_cursor = read_cursor.saturating_add(bytes.len() as u64);
            if bytes.len() < chunk.bytes.len() {
                partial = offset.saturating_add(bytes.len()) < chunk.bytes.len();
            }
            if partial {
                break;
            }
        }
        let next_cursor = read_cursor;
        let terminal = job.is_terminal();
        let value = serde_json::json!({
            "job_id": job.id,
            "state": job.state,
            "cursor": cursor,
            "next_cursor": next_cursor,
            "events": events,
            "partial": partial,
            "dropped_before": dropped_before,
            "exit_code": job.exit_code,
            "eof": terminal,
            "truncated": job.truncated,
        });
        let has_data = value
            .get("events")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|events| !events.is_empty());
        Ok((value, has_data, terminal))
    }

    async fn process_cancel(&self, job_id: &str, wait_ms: u64) -> ToolResult {
        let (token, starting) = self
            .inner
            .state
            .lock()
            .ok()
            .and_then(|state| {
                state.jobs.get(&BackgroundJobId::new(job_id)).map(|job| {
                    (
                        job.cancel_token.clone(),
                        job.state == BackgroundJobState::Starting,
                    )
                })
            })
            .unwrap_or((None, false));
        let Some(token) = token else {
            return self.process_status(job_id);
        };
        token.cancel();
        if starting {
            let mut result = self.process_status(job_id);
            if !result.is_error
                && let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&result.content)
            {
                value["cancellation_requested"] = serde_json::Value::Bool(true);
                result.content = value.to_string();
            }
            return result;
        }
        let deadline = tokio::time::Instant::now() + Duration::from_millis(wait_ms.min(5_000));
        loop {
            let notified = self.inner.state_changed.notified();
            if let Some(status) = self.inner.state.lock().ok().and_then(|state| {
                state
                    .jobs
                    .get(&BackgroundJobId::new(job_id))
                    .map(job_status_json)
            }) && status
                .get("state")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|state| state != "running" && state != "starting")
            {
                return ToolResult::success(status.to_string());
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return self.process_status(job_id);
            }
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
        if state.closing || state.shutdown_requested {
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

fn job_status_json(job: &JobRecord) -> serde_json::Value {
    serde_json::json!({
        "job_id": job.id,
        "tool": job.tool_name,
        "state": job.state,
        "exit_code": job.exit_code,
        "stdout_bytes": job.stdout_bytes,
        "stderr_bytes": job.stderr_bytes,
        "earliest_cursor": job.chunks.front().map_or(job.next_cursor, |chunk| chunk.cursor),
        "next_cursor": job.next_cursor,
        "truncated": job.truncated,
        "started_at_unix_ms": job.started_at_unix_ms,
        "finished_at_unix_ms": job.finished_at_unix_ms,
        "cleanup_outcome": job.cleanup_outcome,
    })
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

    #[tokio::test]
    async fn shutdown_fence_waits_for_in_flight_launch_before_closing_admission() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        {
            let mut state = supervisor.inner.state.lock().unwrap();
            state.shutdown_requested = true;
            state.launching = 1;
        }

        let waiter = {
            let supervisor = supervisor.clone();
            tokio::spawn(async move { supervisor.close_admission_after_launches().await })
        };
        tokio::task::yield_now().await;
        assert!(!supervisor.inner.state.lock().unwrap().closing);

        supervisor.inner.state.lock().unwrap().launching = 0;
        supervisor.inner.state_changed.notify_waiters();
        waiter.await.unwrap();
        assert!(supervisor.inner.state.lock().unwrap().closing);
    }

    #[tokio::test]
    async fn process_read_advances_cursor_without_repeating_chunks() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let id = BackgroundJobId::new("job_test");
        let mut job = JobRecord::starting(id.clone(), "bash".to_owned(), 1);
        job.state = BackgroundJobState::Running;
        job.push_output(BackgroundOutputChunk {
            stream: BackgroundOutputStream::Stdout,
            bytes: b"one".to_vec(),
            captured_at: SystemTime::UNIX_EPOCH,
        });
        job.push_output(BackgroundOutputChunk {
            stream: BackgroundOutputStream::Stderr,
            bytes: b"two".to_vec(),
            captured_at: SystemTime::UNIX_EPOCH,
        });
        supervisor.inner.state.lock().unwrap().jobs.insert(id, job);

        let first = supervisor
            .process_action(
                ProcessAction::Read,
                Some("job_test"),
                Some(0),
                Some(3),
                None,
            )
            .await;
        let first: serde_json::Value = serde_json::from_str(&first.content).unwrap();
        assert_eq!(first["events"][0]["text"], "one");
        assert_eq!(first["next_cursor"], 3);

        let second = supervisor
            .process_action(
                ProcessAction::Read,
                Some("job_test"),
                Some(3),
                Some(3),
                None,
            )
            .await;
        let second: serde_json::Value = serde_json::from_str(&second.content).unwrap();
        assert_eq!(second["events"][0]["text"], "two");
        assert_eq!(second["next_cursor"], 6);
    }

    #[tokio::test]
    async fn process_read_resumes_inside_a_large_chunk_without_repeating_bytes() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let id = BackgroundJobId::new("job_large");
        let mut job = JobRecord::starting(id.clone(), "bash".to_owned(), 1);
        job.state = BackgroundJobState::Running;
        job.push_output(BackgroundOutputChunk {
            stream: BackgroundOutputStream::Stdout,
            bytes: b"abcdef".to_vec(),
            captured_at: SystemTime::UNIX_EPOCH,
        });
        supervisor.inner.state.lock().unwrap().jobs.insert(id, job);

        let first = supervisor
            .process_action(
                ProcessAction::Read,
                Some("job_large"),
                Some(0),
                Some(2),
                None,
            )
            .await;
        let first: serde_json::Value = serde_json::from_str(&first.content).unwrap();
        assert_eq!(first["events"][0]["text"], "ab");
        assert_eq!(first["next_cursor"], 2);
        assert_eq!(first["partial"], true);

        let second = supervisor
            .process_action(
                ProcessAction::Read,
                Some("job_large"),
                first["next_cursor"].as_u64(),
                Some(2),
                None,
            )
            .await;
        let second: serde_json::Value = serde_json::from_str(&second.content).unwrap();
        assert_eq!(second["events"][0]["text"], "cd");
        assert_eq!(second["next_cursor"], 4);
    }

    #[tokio::test]
    async fn process_cancel_marks_starting_job_without_launching_it() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let id = BackgroundJobId::new("job_starting");
        let job = JobRecord::starting(id.clone(), "bash".to_owned(), 1);
        supervisor.inner.state.lock().unwrap().jobs.insert(id, job);

        let result = supervisor
            .process_action(
                ProcessAction::Cancel,
                Some("job_starting"),
                None,
                None,
                None,
            )
            .await;
        let result: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(result["state"], "starting");
        assert_eq!(result["cancellation_requested"], true);
        assert!(
            supervisor
                .inner
                .state
                .lock()
                .unwrap()
                .jobs
                .get(&BackgroundJobId::new("job_starting"))
                .unwrap()
                .cancel_token
                .as_ref()
                .unwrap()
                .is_cancelled()
        );
    }

    #[tokio::test]
    async fn process_unknown_job_does_not_reveal_foreign_state() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let result = supervisor
            .process_action(ProcessAction::Status, Some("job_foreign"), None, None, None)
            .await;
        assert!(result.is_error);
        assert_eq!(result.content, "unknown background job");
    }
}
