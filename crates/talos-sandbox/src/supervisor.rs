//! Owned Unix command lifecycle (ADR-082). No wait path reaps before the last
//! process-group signal. Deliberate process-group escape is outside this boundary.

use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::{Notify, oneshot};

use crate::SandboxError;

const POLL: Duration = Duration::from_millis(10);
const CLEANUP_LIMIT: Duration = Duration::from_secs(2);

#[derive(Default)]
struct State {
    pending: usize,
    failure: Option<String>,
}

#[derive(Default)]
pub(crate) struct Cleanup {
    state: Mutex<State>,
    changed: Notify,
}

impl Cleanup {
    fn register(self: &Arc<Self>) -> Receipt {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).pending += 1;
        Receipt {
            owner: self.clone(),
            finished: false,
        }
    }

    pub(crate) async fn wait(&self, timeout: Duration) -> Result<(), SandboxError> {
        tokio::time::timeout(timeout, async {
            loop {
                let changed = self.changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                {
                    let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                    if state.pending == 0 {
                        return match &state.failure {
                            Some(error) => Err(failed(error)),
                            None => Ok(()),
                        };
                    }
                }
                changed.await;
            }
        })
        .await
        .map_err(|_| failed("sandbox cleanup receipt deadline exceeded"))?
    }
}

struct Receipt {
    owner: Arc<Cleanup>,
    finished: bool,
}

impl Receipt {
    fn finish(&mut self, error: Option<String>) {
        let mut state = self.owner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.pending -= 1;
        if state.failure.is_none() {
            state.failure = error;
        }
        self.finished = true;
        drop(state);
        self.owner.changed.notify_waiters();
    }
}

impl Drop for Receipt {
    fn drop(&mut self) {
        if !self.finished {
            self.finish(Some(
                "sandbox supervisor stopped without confirmed cleanup".into(),
            ));
        }
    }
}

struct CancelOnDrop(Option<oneshot::Sender<()>>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

fn failed(message: impl Into<String>) -> SandboxError {
    SandboxError::ExecutionFailed(message.into())
}

/// Only this owner may reap or signal the leader. A std Child has no implicit
/// Tokio orphan-reaper, including during asynchronous pipe setup failures.
struct OwnedChild {
    child: Child,
    pgid: libc::pid_t,
    may_signal: bool,
}

impl OwnedChild {
    fn observe(&mut self) -> io::Result<bool> {
        // SAFETY: zero is a valid initialization for the output-only siginfo_t.
        // waitid receives a valid writable pointer and a positive owned child ID.
        // WNOWAIT deliberately retains the zombie and therefore its PID identity.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        loop {
            // SAFETY: ADR-082 permits this non-reaping observation. No other
            // wait/try_wait path runs while may_signal is true.
            let result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    self.pgid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if result == 0 {
                // SAFETY: waitid initialized info; si_pid is valid for SIGCHLD.
                return Ok(unsafe { info.si_pid() } != 0);
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if error.raw_os_error() == Some(libc::ECHILD) {
                self.may_signal = false;
            }
            return Err(error);
        }
    }

    fn terminate_group(&mut self) -> io::Result<()> {
        self.terminate_with(|pgid| {
            loop {
                // SAFETY: terminate_with has verified sole ownership of the positive
                // PID established as group leader by setsid (ADR-082). No reaping
                // occurs before this final, fixed SIGKILL operation.
                let result = unsafe { libc::kill(-pgid, libc::SIGKILL) };
                if result == 0 {
                    return Ok(());
                }
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.raw_os_error() == Some(libc::ESRCH) {
                    return Ok(());
                }
                return Err(error);
            }
        })
        .or_else(|error| {
            #[cfg(target_os = "macos")]
            {
                if error.raw_os_error() == Some(libc::EPERM)
                    && self.darwin_group_is_only_owned_leader()?
                {
                    return Ok(());
                }
            }
            Err(error)
        })
    }

    #[cfg(target_os = "macos")]
    fn darwin_group_is_only_owned_leader(&mut self) -> io::Result<bool> {
        if !self.may_signal || !self.observe()? {
            return Ok(false);
        }
        const PROC_PGRP_ONLY: u32 = 2;
        let mut pids = [0 as libc::pid_t; 2];
        // SAFETY: pids is a valid writable two-entry pid_t buffer. The query is
        // restricted to this already-owned positive PGID and reads no process
        // metadata. A one-entry result is complete because the capacity is two.
        let bytes = unsafe {
            libc::proc_listpids(
                PROC_PGRP_ONLY,
                self.pgid as u32,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as libc::c_int,
            )
        };
        let one = std::mem::size_of::<libc::pid_t>() as libc::c_int;
        if bytes != one || pids[0] != self.pgid {
            return Ok(false);
        }
        // Re-check ownership after enumeration and before allowing reap. The
        // leader must remain an unreaped waitid(WNOWAIT) child.
        self.observe()
    }

    fn terminate_with(
        &mut self,
        signal: impl FnOnce(libc::pid_t) -> io::Result<()>,
    ) -> io::Result<()> {
        if !self.may_signal {
            return Err(io::Error::other(
                "sandbox child ownership lost; refusing stale group signal",
            ));
        }
        // Revalidate ownership immediately before the final signal. Exited
        // children remain unreaped, so their group ID cannot be recycled.
        self.observe()?;
        signal(self.pgid)
    }

    async fn reap(&mut self) -> io::Result<std::process::ExitStatus> {
        loop {
            if self.observe()? {
                // Final signal has already happened. Disable signals BEFORE
                // the sole reaping call; Drop cannot target a recycled ID.
                self.may_signal = false;
                return self.child.wait();
            }
            tokio::time::sleep(POLL).await;
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.may_signal {
            // Emergency runtime teardown is not successful cleanup (Receipt
            // reports failure). Still attempt termination before any reaping.
            let _ = self.terminate_group();
            self.may_signal = false;
            let _ = self.child.try_wait();
        }
    }
}

pub(crate) async fn execute(
    mut command: Command,
    cleanup: Arc<Cleanup>,
    deadline: Duration,
    resources: impl Send + 'static,
) -> Result<Output, SandboxError> {
    let started = tokio::time::Instant::now();
    let mut receipt = cleanup.register();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: the post-fork hook invokes only the async-signal-safe setsid and
    // errno observation. No allocation, locks, environment mutation or unwind.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    let mut child = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| command.spawn()))
    {
        Ok(Ok(child)) => child,
        Ok(Err(error)) => {
            receipt.finish(None);
            return Err(failed(format!(
                "sandbox spawn/session setup failed: {error}"
            )));
        }
        Err(_) => {
            receipt.finish(Some(
                "sandbox process spawn panicked; cleanup unconfirmed".into(),
            ));
            return Err(failed("sandbox process spawn panicked"));
        }
    };
    let pgid = match libc::pid_t::try_from(child.id()) {
        Ok(id) if id > 0 => id,
        _ => {
            // OS child IDs fit pid_t; retain an honest error if this invariant
            // ever changes rather than construct a negative/zero group target.
            let _ = child.kill();
            let _ = child.try_wait();
            receipt.finish(Some(
                "sandbox returned invalid process ID; cleanup unconfirmed".into(),
            ));
            return Err(failed("sandbox returned invalid process ID"));
        }
    };
    let owner = OwnedChild {
        child,
        pgid,
        may_signal: true,
    };
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let cancel = CancelOnDrop(Some(cancel_tx));
    let task = tokio::spawn(async move {
        let _resources = resources;
        let result = supervise(owner, cancel_rx, deadline.saturating_sub(started.elapsed())).await;
        receipt.finish(result.1);
        result.0
    });
    let result = task
        .await
        .map_err(|error| failed(format!("sandbox supervisor failed: {error}")))?;
    drop(cancel);
    result
}

async fn supervise(
    mut owner: OwnedChild,
    cancel: oneshot::Receiver<()>,
    deadline: Duration,
) -> (Result<Output, SandboxError>, Option<String>) {
    let stdout = owner
        .child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing stdout"))
        .and_then(tokio::process::ChildStdout::from_std);
    let stderr = owner
        .child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing stderr"))
        .and_then(tokio::process::ChildStderr::from_std);
    let (stdout, stderr) = match (stdout, stderr) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        (stdout, stderr) => {
            let detail = format!(
                "sandbox pipe setup failed: stdout={:?}, stderr={:?}",
                stdout.err(),
                stderr.err()
            );
            let cleanup = match owner.terminate_group() {
                Err(error) => Some(format!("{detail}; group cleanup: {error}")),
                Ok(()) => match tokio::time::timeout(CLEANUP_LIMIT, owner.reap()).await {
                    Ok(Ok(_)) => None,
                    other => Some(format!("{detail}; reap cleanup: {other:?}")),
                },
            };
            return (Err(failed(detail)), cleanup);
        }
    };
    supervise_io(owner, cancel, deadline, stdout, stderr).await
}

async fn supervise_io(
    mut owner: OwnedChild,
    mut cancel: oneshot::Receiver<()>,
    deadline: Duration,
    mut stdout: impl AsyncRead + Unpin,
    mut stderr: impl AsyncRead + Unpin,
) -> (Result<Output, SandboxError>, Option<String>) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut out_eof = false;
    let mut err_eof = false;
    let mut out_buf = [0u8; 8192];
    let mut err_buf = [0u8; 8192];
    let deadline = tokio::time::sleep(deadline);
    tokio::pin!(deadline);
    let reason = loop {
        match owner.observe() {
            Ok(true) if out_eof && err_eof => break None,
            Ok(_) => {}
            Err(error) => break Some(format!("sandbox exit observation failed: {error}")),
        }
        tokio::select! {
            _ = &mut cancel => break Some("sandbox execution cancelled".into()),
            _ = &mut deadline => break Some("sandbox execution timed out".into()),
            read = stdout.read(&mut out_buf), if !out_eof => match read {
                Ok(0) => out_eof = true,
                Ok(n) => out.extend_from_slice(&out_buf[..n]),
                Err(error) => break Some(format!("sandbox stdout read failed: {error}")),
            },
            read = stderr.read(&mut err_buf), if !err_eof => match read {
                Ok(0) => err_eof = true,
                Ok(n) => err.extend_from_slice(&err_buf[..n]),
                Err(error) => break Some(format!("sandbox stderr read failed: {error}")),
            },
            _ = tokio::time::sleep(POLL) => {},
        }
    };
    if let Err(error) = owner.terminate_group() {
        let detail = format!("sandbox process-group cleanup failed: {error}");
        return (Err(failed(&detail)), Some(detail));
    }
    let cleanup = tokio::time::timeout(CLEANUP_LIMIT, async {
        let (status, stdout, stderr) = tokio::join!(
            owner.reap(),
            stdout.read_to_end(&mut out),
            stderr.read_to_end(&mut err),
        );
        Ok::<_, io::Error>((status?, stdout?, stderr?))
    })
    .await;
    match cleanup {
        Ok(Ok((status, _, _))) => {
            let result = match reason {
                Some(reason) => Err(failed(reason)),
                None => Ok(Output {
                    status,
                    stdout: out,
                    stderr: err,
                }),
            };
            (result, None)
        }
        other => {
            let detail = format!("sandbox cleanup unconfirmed: {other:?}");
            (Err(failed(&detail)), Some(detail))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell(script: &str, directory: &std::path::Path) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]).current_dir(directory);
        command
    }

    fn test_owner(mut command: Command) -> OwnedChild {
        // The safe std API creates the isolated group required by these
        // lifecycle tests; production additionally establishes a new session.
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = command.spawn().expect("owned child");
        let pgid = libc::pid_t::try_from(child.id()).expect("pid");
        OwnedChild {
            child,
            pgid,
            may_signal: true,
        }
    }

    async fn ready(path: &std::path::Path) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while !path.exists() {
                tokio::time::sleep(POLL).await;
            }
        })
        .await
        .expect("child readiness handshake");
    }

    #[tokio::test]
    async fn leader_exit_preserves_delayed_descendant_stdout_and_stderr() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let result = execute(
            shell(
                "(sleep 0.1; printf descendant-out; printf descendant-err >&2) & exit 7",
                directory.path(),
            ),
            cleanup.clone(),
            Duration::from_secs(2),
            (),
        )
        .await
        .expect("complete descendant output");
        assert_eq!(result.status.code(), Some(7));
        assert_eq!(result.stdout, b"descendant-out");
        assert_eq!(result.stderr, b"descendant-err");
        cleanup.wait(Duration::from_secs(1)).await.expect("receipt");
    }

    #[tokio::test]
    async fn timeout_cleans_up_descendant_holding_pipes_after_leader_exit() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let started = tokio::time::Instant::now();
        let result = execute(
            shell(
                "(sleep 1; printf late > forbidden) & exit 0",
                directory.path(),
            ),
            cleanup.clone(),
            Duration::from_millis(100),
            (),
        )
        .await
        .expect_err("deadline applies after leader exit");
        assert!(result.to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
        cleanup.wait(Duration::from_secs(1)).await.expect("receipt");
        tokio::time::sleep(Duration::from_millis(1050)).await;
        assert!(!directory.path().join("forbidden").exists());
    }

    #[tokio::test]
    async fn caller_drop_cleans_ready_shell_and_grandchild_before_write() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let command = shell(
            "sh -c 'printf ready > ready; sleep 1; printf late > forbidden' & wait",
            directory.path(),
        );
        let running = tokio::spawn(execute(
            command,
            cleanup.clone(),
            Duration::from_secs(5),
            (),
        ));
        ready(&directory.path().join("ready")).await;
        assert!(
            cleanup.wait(Duration::from_millis(20)).await.is_err(),
            "active command is not cleaned"
        );
        running.abort();
        assert!(running.await.expect_err("cancelled caller").is_cancelled());
        cleanup
            .wait(Duration::from_secs(2))
            .await
            .expect("cancel cleanup receipt");
        tokio::time::sleep(Duration::from_millis(1050)).await;
        assert!(!directory.path().join("forbidden").exists());
    }

    #[tokio::test]
    async fn cancellation_after_observed_leader_exit_still_cleans_descendant() {
        let directory = tempfile::tempdir().expect("workspace");
        let command = shell(
            "(printf ready > ready; sleep 1; printf late > forbidden) & exit 0",
            directory.path(),
        );
        let mut owner = test_owner(command);
        ready(&directory.path().join("ready")).await;
        tokio::time::timeout(Duration::from_secs(2), async {
            while !owner.observe().expect("non-reaping observation") {
                tokio::time::sleep(POLL).await;
            }
        })
        .await
        .expect("leader exit proven before cancellation");
        let (sender, receiver) = oneshot::channel();
        sender.send(()).expect("cancellation");
        let (result, cleanup_error) = supervise(owner, receiver, Duration::from_secs(5)).await;
        assert!(
            result
                .expect_err("cancelled")
                .to_string()
                .contains("cancelled")
        );
        assert!(cleanup_error.is_none(), "{cleanup_error:?}");
        tokio::time::sleep(Duration::from_millis(1050)).await;
        assert!(!directory.path().join("forbidden").exists());
    }

    #[tokio::test]
    async fn closed_pipes_do_not_mean_running_leader_completed() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let result = execute(
            shell(
                "exec 1>&- 2>&-; sleep 1; printf late > forbidden",
                directory.path(),
            ),
            cleanup.clone(),
            Duration::from_millis(100),
            (),
        )
        .await
        .expect_err("leader still running");
        assert!(result.to_string().contains("timed out"));
        cleanup.wait(Duration::from_secs(1)).await.expect("receipt");
        assert!(!directory.path().join("forbidden").exists());
    }

    #[tokio::test]
    async fn ordinary_completion_terminates_silent_remaining_group_members() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        execute(
            shell(
                "(exec 1>&- 2>&-; sleep 1; printf late > forbidden) & exit 0",
                directory.path(),
            ),
            cleanup.clone(),
            Duration::from_secs(2),
            (),
        )
        .await
        .expect("leader completed");
        cleanup.wait(Duration::from_secs(1)).await.expect("receipt");
        tokio::time::sleep(Duration::from_millis(1050)).await;
        assert!(!directory.path().join("forbidden").exists());
    }

    #[tokio::test]
    async fn continuous_output_does_not_reset_deadline() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let started = tokio::time::Instant::now();
        let result = execute(
            shell("while :; do printf x; printf y >&2; done", directory.path()),
            cleanup.clone(),
            Duration::from_millis(100),
            (),
        )
        .await
        .expect_err("bounded output loop");
        assert!(result.to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
        cleanup.wait(Duration::from_secs(1)).await.expect("receipt");
    }

    #[tokio::test]
    async fn spawn_failure_has_no_outstanding_cleanup() {
        let cleanup = Arc::new(Cleanup::default());
        let result = execute(
            Command::new("/no-such-talos-sandbox-command"),
            cleanup.clone(),
            Duration::from_secs(1),
            (),
        )
        .await;
        assert!(
            result
                .expect_err("spawn error")
                .to_string()
                .contains("spawn/session setup")
        );
        cleanup
            .wait(Duration::from_secs(1))
            .await
            .expect("no spawned process");
    }

    #[tokio::test]
    async fn session_establishment_failure_does_not_execute_command() {
        let directory = tempfile::tempdir().expect("workspace");
        let cleanup = Arc::new(Cleanup::default());
        let mut command = shell("printf forbidden > forbidden", directory.path());
        // SAFETY: ADR-082's same async-signal-safe setsid boundary. Establishing
        // the session once makes production's second setsid fail deterministically
        // (an existing process-group leader cannot create another session).
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    Err(io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        let error = execute(command, cleanup.clone(), Duration::from_secs(1), ())
            .await
            .expect_err("session setup refused");
        assert!(error.to_string().contains("spawn/session setup"));
        assert!(!directory.path().join("forbidden").exists());
        cleanup
            .wait(Duration::from_secs(1))
            .await
            .expect("failed spawn reaped by std");
    }

    #[tokio::test]
    async fn abandoned_receipt_is_sticky_failure() {
        let cleanup = Arc::new(Cleanup::default());
        drop(cleanup.register());
        assert!(cleanup.wait(Duration::from_secs(1)).await.is_err());
        cleanup.register().finish(None);
        assert!(cleanup.wait(Duration::from_secs(1)).await.is_err());
    }

    #[test]
    fn lost_child_ownership_permanently_disables_group_signals() {
        let child = Command::new("true").spawn().expect("spawn");
        let pgid = libc::pid_t::try_from(child.id()).expect("pid");
        let mut owner = OwnedChild {
            child,
            pgid,
            may_signal: true,
        };
        owner.child.wait().expect("simulate an external reaper");
        assert_eq!(
            owner.observe().expect_err("ECHILD").raw_os_error(),
            Some(libc::ECHILD)
        );
        assert!(!owner.may_signal);
        assert!(owner.terminate_group().is_err());
    }

    #[tokio::test]
    async fn signal_failure_is_not_reaped_or_reported_as_success() {
        let directory = tempfile::tempdir().expect("workspace");
        let mut owner = test_owner(shell("sleep 10", directory.path()));
        let error = owner
            .terminate_with(|_| Err(io::Error::from_raw_os_error(libc::EPERM)))
            .expect_err("signal failure");
        assert_eq!(error.raw_os_error(), Some(libc::EPERM));
        assert!(
            owner.may_signal,
            "identity must remain pinned for cleanup retry"
        );
        owner.terminate_group().expect("real final signal");
        tokio::time::timeout(Duration::from_secs(2), owner.reap())
            .await
            .expect("bounded reap")
            .expect("reaped");
        assert!(!owner.may_signal);
        assert!(
            owner.terminate_group().is_err(),
            "no late signal after reap"
        );
    }

    #[tokio::test]
    async fn reader_error_cannot_turn_into_success_or_clean_receipt() {
        struct BrokenReader;
        impl AsyncRead for BrokenReader {
            fn poll_read(
                self: std::pin::Pin<&mut Self>,
                _: &mut std::task::Context<'_>,
                _: &mut tokio::io::ReadBuf<'_>,
            ) -> std::task::Poll<io::Result<()>> {
                std::task::Poll::Ready(Err(io::Error::other("injected read failure")))
            }
        }
        let directory = tempfile::tempdir().expect("workspace");
        let owner = test_owner(shell("sleep 10", directory.path()));
        let (_sender, receiver) = oneshot::channel();
        let (result, cleanup_error) = supervise_io(
            owner,
            receiver,
            Duration::from_secs(2),
            BrokenReader,
            tokio::io::empty(),
        )
        .await;
        assert!(
            result
                .expect_err("read failure")
                .to_string()
                .contains("injected read failure")
        );
        assert!(
            cleanup_error.is_some(),
            "failed output cleanup cannot claim EOF"
        );
    }
}
