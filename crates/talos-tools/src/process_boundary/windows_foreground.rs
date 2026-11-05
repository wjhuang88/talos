//! Foreground ownership over the shared assigned-before-resume Job launcher.

use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use talos_core::tool::{ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Notify, oneshot};

use super::{BackgroundProcessControl, WindowsJobControl, WindowsLaunchCommand};

#[derive(Default)]
struct State {
    pending: usize,
    sealed: bool,
    failed: bool,
}

#[derive(Default)]
struct Scope {
    state: Mutex<State>,
    changed: Notify,
    parent: Option<Arc<Scope>>,
}

impl Scope {
    fn register(self: &Arc<Self>) -> Result<Reservation, String> {
        let parent = self.parent.as_ref().map(Self::register).transpose()?;
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.sealed {
            if let Some(mut parent) = parent {
                parent.finish(false);
            }
            return Err("Windows command admission is sealed".into());
        }
        state.pending += 1;
        Ok(Reservation {
            scope: self.clone(),
            parent: parent.map(Box::new),
            finished: false,
        })
    }

    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError> {
        tokio::time::timeout(timeout, async {
            loop {
                let changed = self.changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                {
                    let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                    if state.pending == 0 {
                        return if state.failed {
                            Err(ToolCleanupError::Unconfirmed)
                        } else {
                            Ok(())
                        };
                    }
                }
                changed.await;
            }
        })
        .await
        .map_err(|_| ToolCleanupError::Timeout)?
    }
}

struct Reservation {
    scope: Arc<Scope>,
    parent: Option<Box<Reservation>>,
    finished: bool,
}

impl Reservation {
    fn finish(&mut self, failed: bool) {
        if let Some(parent) = &mut self.parent {
            parent.finish(failed);
        }
        let mut state = self.scope.state.lock().unwrap_or_else(|e| e.into_inner());
        state.pending -= 1;
        state.failed |= failed;
        self.finished = true;
        drop(state);
        self.scope.changed.notify_waiters();
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if !self.finished {
            self.finish(true);
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct OwnedCommandExecutor {
    scope: Arc<Scope>,
}

impl OwnedCommandExecutor {
    pub(crate) fn operation(&self) -> OwnedCommandOperation {
        OwnedCommandOperation {
            scope: Arc::new(Scope {
                parent: Some(self.scope.clone()),
                ..Scope::default()
            }),
        }
    }
}

#[async_trait]
impl ToolCleanupReceipt for OwnedCommandExecutor {
    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError> {
        self.scope.wait(timeout).await
    }
}

impl ToolCleanupOwner for OwnedCommandExecutor {
    fn seal(&self) {
        self.scope
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .sealed = true;
    }
}

#[derive(Clone)]
pub(crate) struct OwnedCommandOperation {
    scope: Arc<Scope>,
}

#[derive(Clone)]
#[cfg(any(feature = "git", feature = "project-management", test))]
pub(crate) struct CleanupReceipt {
    scope: Arc<Scope>,
}

#[async_trait]
#[cfg(any(feature = "git", feature = "project-management", test))]
impl ToolCleanupReceipt for CleanupReceipt {
    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError> {
        self.scope.wait(timeout).await
    }
}

struct CancelOnDrop(Option<oneshot::Sender<()>>);

#[derive(Default)]
struct CapturedStream {
    bytes: Vec<u8>,
    total: usize,
}
#[derive(Default)]
struct ToolCapture {
    stdout: Mutex<CapturedStream>,
    stderr: Mutex<CapturedStream>,
    timed_out: std::sync::atomic::AtomicBool,
}
#[cfg(feature = "shell")]
pub(crate) struct ToolCommandOutput {
    pub(crate) output: Output,
    pub(crate) stdout_total: usize,
    pub(crate) stderr_total: usize,
    pub(crate) timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandStartState {
    NotStarted,
    Started,
    Unknown,
}

#[derive(Debug)]
pub(crate) struct CommandFailure {
    pub(crate) start_state: CommandStartState,
    pub(crate) message: String,
}

impl From<String> for CommandFailure {
    fn from(message: String) -> Self {
        Self {
            start_state: CommandStartState::NotStarted,
            message,
        }
    }
}

impl From<&str> for CommandFailure {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl OwnedCommandOperation {
    /// Adapt shell-tool argv and inherited environment to the existing owned launcher.
    #[cfg(feature = "shell")]
    pub(crate) async fn execute_tool_command(
        &self,
        program: &str,
        args: &[String],
        cwd: &std::path::Path,
        environment: &std::collections::BTreeMap<String, String>,
        input: Option<Vec<u8>>,
        deadline: Duration,
        limit: usize,
        memory: crate::process_memory::ProcessMemoryPolicy,
    ) -> Result<ToolCommandOutput, CommandFailure> {
        let executable = super::resolve_windows_program(program)?;
        let mut command = Command::new(executable);
        command.args(args).current_dir(cwd).env_clear();
        // Materialize inheritance before entering the owned native boundary.
        // The shared environment encoder still rejects dangerous names.
        let dangerous = talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
        for (key, value) in std::env::vars_os() {
            if !dangerous
                .iter()
                .any(|name| key.to_string_lossy().eq_ignore_ascii_case(name))
            {
                command.env(key, value);
            }
        }
        command.envs(environment);
        let capture = Arc::new(ToolCapture::default());
        let output = self
            .execute_inner(
                command,
                deadline,
                limit,
                input,
                Some(memory.bytes()),
                Some(capture.clone()),
            )
            .await?;
        let stdout_total = capture
            .stdout
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .total;
        let stderr_total = capture
            .stderr
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .total;
        Ok(ToolCommandOutput {
            output,
            stdout_total,
            stderr_total,
            timed_out: capture.timed_out.load(std::sync::atomic::Ordering::SeqCst),
        })
    }

    /// Keep blocking preparation registered through cancellation and shutdown.
    /// Dropping the caller requests cancellation but does not abandon ownership
    /// of a kernel call that has not returned yet. The callback must not launch.
    #[cfg(feature = "project-management")]
    pub(crate) async fn run_preparation<T: Send + 'static>(
        &self,
        cancellation: tokio_util::sync::CancellationToken,
        prepare: impl FnOnce(tokio_util::sync::CancellationToken) -> T + Send + 'static,
    ) -> Result<T, String> {
        let mut reservation = self.scope.register()?;
        let guard = cancellation.clone().drop_guard();
        let task = tokio::task::spawn_blocking(move || {
            let result = prepare(cancellation);
            reservation.finish(false);
            result
        });
        let result = task
            .await
            .map_err(|_| "Windows preparation worker failed; cleanup unconfirmed".to_owned());
        drop(guard);
        result
    }

    #[cfg(any(feature = "git", feature = "project-management", test))]
    pub(crate) fn cleanup_receipt(&self) -> CleanupReceipt {
        CleanupReceipt {
            scope: self.scope.clone(),
        }
    }

    #[cfg(any(feature = "git", test))]
    pub(crate) async fn execute(
        &self,
        command: Command,
        deadline: Duration,
        limit: usize,
    ) -> Result<Output, String> {
        self.execute_inner(command, deadline, limit, None, None, None)
            .await
            .map_err(|failure| failure.message)
    }

    #[cfg(any(feature = "git", feature = "project-management", test))]
    pub(crate) async fn execute_with_start_evidence(
        &self,
        command: Command,
        deadline: Duration,
        limit: usize,
    ) -> Result<Output, CommandFailure> {
        self.execute_inner(command, deadline, limit, None, None, None)
            .await
    }

    #[cfg(any(feature = "git", test))]
    pub(crate) async fn execute_with_input(
        &self,
        command: Command,
        deadline: Duration,
        limit: usize,
        input: Vec<u8>,
    ) -> Result<Output, String> {
        self.execute_inner(command, deadline, limit, Some(input), None, None)
            .await
            .map_err(|failure| failure.message)
    }

    /// Only explicit environment entries are transferred. Git callers construct
    /// these commands with env_clear; inheritance is never reconstructed here.
    async fn execute_inner(
        &self,
        command: Command,
        deadline: Duration,
        limit: usize,
        input: Option<Vec<u8>>,
        memory_bytes: Option<u64>,
        capture: Option<Arc<ToolCapture>>,
    ) -> Result<Output, CommandFailure> {
        // Encoding and supervisor scheduling consume the caller's budget too.
        let deadline = tokio::time::Instant::now() + deadline;
        let cwd = command
            .get_current_dir()
            .ok_or("Windows foreground cwd is not bound")?;
        let executable = std::path::Path::new(command.get_program());
        if !cwd.is_absolute() || !executable.is_absolute() {
            return Err("Windows foreground executable and cwd must be absolute".into());
        }
        let environment = super::windows_environment::snapshot(
            command
                .get_envs()
                .filter_map(|(key, value)| value.map(|value| (key.to_owned(), value.to_owned()))),
        )?;
        let mut command = WindowsLaunchCommand::new(
            executable,
            command.get_program(),
            &command
                .get_args()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
            cwd,
            environment,
            input.is_some(),
        )?;
        command.memory_bytes = memory_bytes;
        // Register before handing any launch authority to the detached supervisor.
        let mut reservation = self.scope.register()?;
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let guard = CancelOnDrop(Some(cancel_tx));
        let (result_tx, result_rx) = oneshot::channel();
        tokio::spawn(async move {
            // Native launch currently reports errors without distinguishing
            // pre-creation failure from partial launch. Never infer zero startup
            // from its diagnostic string; cleanup stays independently checked.
            let mut start_state = CommandStartState::Unknown;
            let (result, cleanup_failed) = supervise(
                command,
                input,
                deadline,
                limit,
                cancel_rx,
                &mut start_state,
                capture,
            )
            .await;
            reservation.finish(cleanup_failed);
            let _ = result_tx.send(result.map_err(|message| CommandFailure {
                start_state,
                message,
            }));
        });
        let result = result_rx.await.map_err(|_| CommandFailure {
            start_state: CommandStartState::Unknown,
            message: "Windows foreground supervisor stopped".to_owned(),
        })?;
        drop(guard);
        result
    }
}

async fn read(
    mut file: tokio::fs::File,
    remaining: Arc<std::sync::atomic::AtomicUsize>,
    capture: Option<(Arc<ToolCapture>, bool, usize)>,
) -> Result<Vec<u8>, String> {
    use std::sync::atomic::Ordering;
    let mut output = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let count = file
            .read(&mut buffer)
            .await
            .map_err(|error| format!("Windows output read failed: {error}"))?;
        if count == 0 {
            return Ok(output);
        }
        if let Some((capture, is_stdout, limit)) = &capture {
            let stream = if *is_stdout {
                &capture.stdout
            } else {
                &capture.stderr
            };
            let mut stream = stream.lock().unwrap_or_else(|e| e.into_inner());
            stream.total = stream.total.saturating_add(count);
            let retained = count.min(limit.saturating_sub(stream.bytes.len()));
            stream.bytes.extend_from_slice(&buffer[..retained]);
            continue;
        }
        remaining
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(count)
            })
            .map_err(|_| "Windows output limit exceeded".to_owned())?;
        output.extend_from_slice(&buffer[..count]);
    }
}

async fn supervise(
    command: WindowsLaunchCommand,
    input: Option<Vec<u8>>,
    deadline: tokio::time::Instant,
    limit: usize,
    mut cancel: oneshot::Receiver<()>,
    start_state: &mut CommandStartState,
    capture: Option<Arc<ToolCapture>>,
) -> (Result<Output, String>, bool) {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::os::windows::process::ExitStatusExt;
    use windows_sys::Win32::Foundation::{WAIT_FAILED, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};

    let (control, process, stdout, stderr, stdin) = {
        let launched = match super::create_windows_job_native_checked(
            command,
            || {
                if tokio::time::Instant::now() >= deadline
                    || !matches!(cancel.try_recv(), Err(oneshot::error::TryRecvError::Empty))
                {
                    Err("Windows foreground launch cancelled or deadline exceeded".into())
                } else {
                    Ok(())
                }
            },
            #[cfg(test)]
            super::WindowsLaunchTestHooks::default(),
        ) {
            Ok(launched) => launched,
            Err(error) => return (Err(error), true),
        };
        *start_state = CommandStartState::Started;
        let control = WindowsJobControl { job: launched.job };
        // SAFETY: native launch transfers unique ownership of the process handle.
        let process = unsafe { OwnedHandle::from_raw_handle(launched.process as _) };
        (
            control,
            process,
            launched.stdout,
            launched.stderr,
            launched.stdin,
        )
    };
    let remaining = Arc::new(std::sync::atomic::AtomicUsize::new(limit));
    let mut stdout = tokio::spawn(read(
        stdout,
        remaining.clone(),
        capture.clone().map(|c| (c, true, limit)),
    ));
    let mut stderr = tokio::spawn(read(
        stderr,
        remaining,
        capture.clone().map(|c| (c, false, limit)),
    ));
    let mut writer = tokio::spawn(async move {
        match (stdin, input) {
            (Some(mut writer), Some(input)) => {
                writer
                    .write_all(&input)
                    .await
                    .map_err(|error| format!("Windows stdin delivery failed: {error}"))?;
                writer
                    .flush()
                    .await
                    .map_err(|error| format!("Windows stdin flush failed: {error}"))?;
                drop(writer);
                Ok(())
            }
            (None, None) => Ok(()),
            _ => Err("Windows stdin ownership mismatch".into()),
        }
    });
    let (mut stdout_joined, mut stderr_joined, mut writer_joined) = (None, None, None);
    let collect = async {
        let exit = async {
            loop {
                // SAFETY: owned live process handle, nonblocking observation.
                let status = unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) };
                if status == WAIT_FAILED {
                    return Err(super::last_windows_error("WaitForSingleObject"));
                }
                if status == WAIT_OBJECT_0 {
                    let mut code = 0;
                    // SAFETY: live process handle and writable exit code output.
                    if unsafe { GetExitCodeProcess(process.as_raw_handle() as _, &mut code) } == 0 {
                        return Err(super::last_windows_error("GetExitCodeProcess"));
                    }
                    // Descendants may have closed stdio; leader exit alone is not cleanup.
                    control.terminate().await?;
                    return Ok(std::process::ExitStatus::from_raw(code));
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };
        let (stdout, stderr, (), status) = tokio::try_join!(
            join(&mut stdout, &mut stdout_joined),
            join(&mut stderr, &mut stderr_joined),
            join(&mut writer, &mut writer_joined),
            exit,
        )?;
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    };
    let mut result: Result<Output, String> = tokio::select! {
        _ = &mut cancel => Err("Windows foreground execution cancelled".into()),
        result = tokio::time::timeout_at(deadline, collect) => match result {
            Ok(result) => result,
            Err(_) => if let Some(capture) = &capture {
                capture.timed_out.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(Output { status: std::process::ExitStatus::from_raw(1), stdout: Vec::new(), stderr: Vec::new() })
            } else { Err("Windows foreground deadline exceeded".into()) },
        },
    };
    // Join all workers after Job termination; dropping File futures alone does
    // not cancel the blocking IO owned by Tokio's filesystem adapter.
    let cleanup_error = control.terminate().await.err();
    let cleanup_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let stdout_closed = finish_io(&mut stdout, stdout_joined, cleanup_deadline).await;
    let stderr_closed = finish_io(&mut stderr, stderr_joined, cleanup_deadline).await;
    let writer_closed = finish_io(&mut writer, writer_joined, cleanup_deadline).await;
    if let (Some(capture), Ok(output)) = (&capture, &mut result) {
        output.stdout = std::mem::take(
            &mut capture
                .stdout
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .bytes,
        );
        output.stderr = std::mem::take(
            &mut capture
                .stderr
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .bytes,
        );
    }
    if cleanup_error.is_some() || !stdout_closed || !stderr_closed || !writer_closed {
        // Preserve the failed boundary and execution outcome: an unconfirmed
        // pipe worker and an unconfirmed Job need different investigations.
        let execution = match &result {
            Ok(output) => format!("exit={}", output.status),
            Err(error) => error.clone(),
        };
        (
            Err(format!(
                "Windows foreground cleanup unconfirmed: job={cleanup_error:?}, stdout_closed={stdout_closed}, stderr_closed={stderr_closed}, stdin_closed={writer_closed}; execution={execution}"
            )),
            true,
        )
    } else {
        (result, false)
    }
}

async fn join<T>(
    task: &mut tokio::task::JoinHandle<Result<T, String>>,
    joined: &mut Option<bool>,
) -> Result<T, String> {
    let result = task.await;
    *joined = Some(result.is_ok());
    result.map_err(|_| "Windows IO worker stopped unexpectedly".to_owned())?
}

async fn finish_io<T>(
    task: &mut tokio::task::JoinHandle<Result<T, String>>,
    joined: Option<bool>,
    deadline: tokio::time::Instant,
) -> bool {
    if let Some(confirmed) = joined {
        return confirmed;
    }
    match tokio::time::timeout_at(deadline, &mut *task).await {
        Ok(Ok(_)) => true,
        _ => {
            task.abort();
            false
        }
    }
}

#[cfg(test)]
#[path = "windows_foreground_tests.rs"]
mod tests;
