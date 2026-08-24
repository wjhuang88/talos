//! Shared non-interactive process boundary for command tools.

use std::process::Stdio;

#[cfg(unix)]
use std::sync::Arc;

#[cfg(unix)]
use async_trait::async_trait;
#[cfg(unix)]
use talos_core::background_job::{
    BACKGROUND_PROCESS_EVENT_CAPACITY, BackgroundJobLauncher, BackgroundOutputChunk,
    BackgroundOutputStream, BackgroundProcessControl, BackgroundProcessEvent,
    BackgroundProcessExit, LaunchedBackgroundJob,
};
#[cfg(unix)]
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

/// Prevents a foreground tool child from competing with Talos for terminal input.
///
/// Pipeline callers may explicitly request a piped stdin. Every other child receives EOF. On
/// Unix, a new session also prevents programs such as `sudo` and `ssh` from bypassing stdin and
/// opening Talos's controlling terminal through `/dev/tty`.
pub(crate) fn isolate_terminal_input(cmd: &mut Command, pipe_stdin: bool) {
    if pipe_stdin {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    #[cfg(unix)]
    {
        // SAFETY: the closure runs after fork and before exec. `setsid(2)` and errno inspection
        // are async-signal-safe, and the closure performs no allocation, locking or formatting.
        // ADR-007 explicitly authorizes this terminal-containment site.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
}

/// Applies the same resource hardening used by the foreground shell boundary.
#[cfg(unix)]
pub(crate) fn apply_process_hardening(cmd: &mut Command) {
    let c_names: Vec<std::ffi::CString> =
        talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names()
            .into_iter()
            .map(|name| std::ffi::CString::new(name).expect("valid environment variable name"))
            .collect();

    // SAFETY: this closure runs post-fork/pre-exec and performs only the ADR-007-authorized,
    // async-signal-safe environment and rlimit operations. It allocates nothing and cannot panic.
    unsafe {
        cmd.pre_exec(move || {
            for name in &c_names {
                libc::unsetenv(name.as_ptr());
            }
            let limits = [
                (libc::RLIMIT_CORE, 0_u64),
                (libc::RLIMIT_CPU, 300_u64),
                (libc::RLIMIT_AS, 2_u64 * 1024 * 1024 * 1024),
            ];
            for (resource, limit) in limits {
                let rlim = libc::rlimit {
                    rlim_cur: limit,
                    rlim_max: limit,
                };
                // Some Unix targets reject one of these advisory limits (for example
                // RLIMIT_AS on macOS); preserve the existing best-effort foreground behavior.
                let _ = libc::setrlimit(resource, &rlim as *const _);
            }
            Ok(())
        });
    }
}

/// Unix launcher for one already validated shell or direct-exec command.
#[cfg(unix)]
pub(crate) struct UnixBackgroundLauncher {
    command: Command,
}

#[cfg(unix)]
impl UnixBackgroundLauncher {
    pub(crate) fn new(command: Command) -> Self {
        Self { command }
    }
}

#[cfg(unix)]
struct UnixProcessGroupControl {
    pgid: i32,
}

#[cfg(unix)]
#[async_trait]
impl BackgroundProcessControl for UnixProcessGroupControl {
    async fn terminate(&self) -> Result<(), String> {
        signal_process_group(self.pgid, libc::SIGTERM)
    }

    async fn force_terminate(&self) -> Result<(), String> {
        signal_process_group(self.pgid, libc::SIGKILL)
    }
}

#[cfg(unix)]
fn signal_process_group(pgid: i32, signal: i32) -> Result<(), String> {
    if pgid <= 0 {
        return Err("refusing to signal a non-positive process group id".to_owned());
    }
    // SAFETY: ADR-060 authorizes only checked negative-PGID SIGTERM/SIGKILL calls. `pgid` is
    // validated positive before negation; no arbitrary signal number reaches this function.
    let result = unsafe { libc::kill(-pgid, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(format!("failed to signal process group {pgid}: {error}"))
    }
}

#[cfg(unix)]
#[async_trait]
impl BackgroundJobLauncher for UnixBackgroundLauncher {
    async fn launch(mut self: Box<Self>) -> Result<LaunchedBackgroundJob, String> {
        self.command.stdout(Stdio::piped()).stderr(Stdio::piped());
        isolate_terminal_input(&mut self.command, false);
        apply_process_hardening(&mut self.command);
        let mut child = self
            .command
            .spawn()
            .map_err(|error| format!("failed to spawn background process: {error}"))?;
        let pid = child
            .id()
            .ok_or_else(|| "background process has no platform id".to_owned())?;
        let pgid = i32::try_from(pid)
            .ok()
            .filter(|pgid| *pgid > 0)
            .ok_or_else(|| "background process group id is invalid".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "background stdout pipe is unavailable".to_owned())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "background stderr pipe is unavailable".to_owned())?;
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(BACKGROUND_PROCESS_EVENT_CAPACITY);
        let stdout_task = tokio::spawn(pump_output(
            stdout,
            BackgroundOutputStream::Stdout,
            event_tx.clone(),
        ));
        let stderr_task = tokio::spawn(pump_output(
            stderr,
            BackgroundOutputStream::Stderr,
            event_tx.clone(),
        ));
        tokio::spawn(async move {
            let wait = child.wait().await;
            let stdout_result = stdout_task.await;
            let stderr_result = stderr_task.await;
            let event = match (wait, stdout_result, stderr_result) {
                (Ok(status), Ok(Ok(())), Ok(Ok(()))) => {
                    BackgroundProcessEvent::Exited(BackgroundProcessExit {
                        code: status.code(),
                        success: status.success(),
                    })
                }
                (wait, stdout, stderr) => BackgroundProcessEvent::SupervisionFailed(format!(
                    "background reap/read failed: wait={wait:?}, stdout={stdout:?}, stderr={stderr:?}"
                )),
            };
            let _ = event_tx.send(event).await;
        });

        Ok(LaunchedBackgroundJob {
            control: Arc::new(UnixProcessGroupControl { pgid }),
            events: event_rx,
        })
    }
}

#[cfg(unix)]
async fn pump_output<R>(
    mut reader: R,
    stream: BackgroundOutputStream,
    sender: tokio::sync::mpsc::Sender<BackgroundProcessEvent>,
) -> Result<(), String>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0_u8; 4096];
    loop {
        let read = reader
            .read(&mut buffer)
            .await
            .map_err(|error| format!("background {stream:?} read failed: {error}"))?;
        if read == 0 {
            return Ok(());
        }
        sender
            .send(BackgroundProcessEvent::Output(BackgroundOutputChunk {
                stream,
                bytes: buffer[..read].to_vec(),
                captured_at: std::time::SystemTime::now(),
            }))
            .await
            .map_err(|_| "background supervisor stopped receiving output".to_owned())?;
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use talos_core::background_job::{BackgroundJobLauncher, BackgroundProcessEvent};

    #[tokio::test]
    async fn unix_launcher_captures_bounded_ordered_output_and_reaps_leader() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf out; printf err >&2"]);
        let launched = Box::new(UnixBackgroundLauncher::new(command))
            .launch()
            .await
            .expect("launcher succeeds");
        let mut events = launched.events;
        let mut output = Vec::new();
        let mut exited = false;
        while let Some(event) = events.recv().await {
            match event {
                BackgroundProcessEvent::Output(chunk) => output.push(chunk),
                BackgroundProcessEvent::Exited(exit) => {
                    assert!(exit.success);
                    exited = true;
                    break;
                }
                BackgroundProcessEvent::SupervisionFailed(error) => panic!("{error}"),
            }
        }
        assert!(exited);
        let combined = output
            .iter()
            .flat_map(|chunk| chunk.bytes.iter().copied())
            .collect::<Vec<_>>();
        assert!(combined == b"outerr" || combined == b"errout");
    }

    #[tokio::test]
    async fn unix_launcher_treats_esrch_as_already_exited() {
        let command = Command::new("true");
        let launched = Box::new(UnixBackgroundLauncher::new(command))
            .launch()
            .await
            .expect("launcher succeeds");
        let control = launched.control;
        let _ = launched.events;
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        assert!(control.terminate().await.is_ok());
    }
}
