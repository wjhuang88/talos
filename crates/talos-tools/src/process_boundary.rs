//! Shared non-interactive process boundary for command tools.

use std::process::Stdio;

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
