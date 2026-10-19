//! Shared non-interactive process boundary for command tools.

#[cfg(all(windows, feature = "git"))]
pub(crate) mod windows_foreground;

#[cfg(any(windows, test))]
mod windows_command_line;

#[cfg(any(windows, test))]
mod windows_environment;

#[cfg(feature = "shell")]
use std::process::Stdio;

#[cfg(all(unix, feature = "shell"))]
use std::sync::Arc;

#[cfg(all(unix, feature = "shell"))]
use async_trait::async_trait;
#[cfg(all(unix, feature = "shell"))]
use talos_core::background_job::{
    BACKGROUND_PROCESS_EVENT_CAPACITY, BackgroundJobLauncher, BackgroundOutputChunk,
    BackgroundOutputStream, BackgroundProcessControl, BackgroundProcessEvent,
    BackgroundProcessExit, LaunchedBackgroundJob,
};
#[cfg(all(unix, feature = "shell"))]
use tokio::io::{AsyncRead, AsyncReadExt};
#[cfg(any(unix, feature = "shell"))]
use tokio::process::Command;

#[cfg(windows)]
use async_trait::async_trait;
#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use talos_core::background_job::{
    BACKGROUND_PROCESS_EVENT_CAPACITY, BackgroundJobLauncher, BackgroundOutputChunk,
    BackgroundOutputStream, BackgroundProcessControl, BackgroundProcessEvent,
    BackgroundProcessExit, LaunchedBackgroundJob,
};

/// Prevents a foreground tool child from competing with Talos for terminal input.
///
/// Pipeline callers may explicitly request a piped stdin. Every other child receives EOF. On
/// Unix, a new session also prevents programs such as `sudo` and `ssh` from bypassing stdin and
/// opening Talos's controlling terminal through `/dev/tty`.
#[cfg(feature = "shell")]
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
#[cfg(all(unix, feature = "shell"))]
pub(crate) struct UnixBackgroundLauncher {
    command: Command,
}

#[cfg(all(unix, feature = "shell"))]
impl UnixBackgroundLauncher {
    pub(crate) fn new(command: Command) -> Self {
        Self { command }
    }
}

#[cfg(all(unix, feature = "shell"))]
struct UnixProcessGroupControl {
    pgid: i32,
}

#[cfg(all(unix, feature = "shell"))]
#[async_trait]
impl BackgroundProcessControl for UnixProcessGroupControl {
    async fn terminate(&self) -> Result<(), String> {
        signal_process_group(self.pgid, libc::SIGTERM)
    }

    async fn force_terminate(&self) -> Result<(), String> {
        signal_process_group(self.pgid, libc::SIGKILL)
    }
}

#[cfg(all(unix, feature = "shell"))]
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

#[cfg(all(unix, feature = "shell"))]
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

#[cfg(all(unix, feature = "shell"))]
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

#[cfg(all(test, unix, feature = "shell"))]
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

/// Windows Job Object launcher implementing ADR-068's assigned-before-resume boundary.
#[cfg(windows)]
pub(crate) struct WindowsBackgroundLauncher {
    program: String,
    args: Vec<String>,
    cwd: std::path::PathBuf,
    env: std::collections::BTreeMap<String, String>,
}

#[cfg(windows)]
impl WindowsBackgroundLauncher {
    pub(crate) fn new(
        program: String,
        args: Vec<String>,
        cwd: std::path::PathBuf,
        env: std::collections::BTreeMap<String, String>,
    ) -> Self {
        Self {
            program,
            args,
            cwd,
            env,
        }
    }
}

#[cfg(windows)]
struct WindowsJobControl {
    job: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
unsafe impl Send for WindowsJobControl {}
#[cfg(windows)]
unsafe impl Sync for WindowsJobControl {}

#[cfg(windows)]
#[async_trait]
impl BackgroundProcessControl for WindowsJobControl {
    async fn terminate(&self) -> Result<(), String> {
        if self.job.is_null() {
            return Err("Windows Job control has no owned handle".into());
        }
        // Termination acceptance is not proof that all descendants have exited.
        let ok = unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1) };
        if ok == 0 {
            return Err(last_windows_error("TerminateJobObject"));
        }
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if self.active_processes()? == 0 {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err("Windows Job cleanup unconfirmed after 5 seconds".into());
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    async fn force_terminate(&self) -> Result<(), String> {
        self.terminate().await
    }
}

#[cfg(windows)]
impl WindowsJobControl {
    fn active_processes(&self) -> Result<u32, String> {
        // NULL queries the calling process's Job, not our owned Job. Reject it
        // explicitly so a missing handle cannot produce unrelated cleanup proof.
        if self.job.is_null() {
            return Err("Windows Job control has no owned handle".into());
        }
        use windows_sys::Win32::System::JobObjects::{
            JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JobObjectBasicAccountingInformation,
            QueryInformationJobObject,
        };
        // SAFETY: this POD output is initialized, correctly sized, and exclusively
        // borrowed. The Job handle remains owned by self for the entire query.
        let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            QueryInformationJobObject(
                self.job,
                JobObjectBasicAccountingInformation,
                std::ptr::from_mut(&mut accounting).cast(),
                std::mem::size_of_val(&accounting) as u32,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(last_windows_error("QueryInformationJobObject"))
        } else {
            Ok(accounting.ActiveProcesses)
        }
    }
}

#[cfg(windows)]
impl Drop for WindowsJobControl {
    fn drop(&mut self) {
        if !self.job.is_null() {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };
            self.job = std::ptr::null_mut();
        }
    }
}

#[cfg(windows)]
#[async_trait]
impl BackgroundJobLauncher for WindowsBackgroundLauncher {
    async fn launch(self: Box<Self>) -> Result<LaunchedBackgroundJob, String> {
        let launched = create_windows_job(&self.program, &self.args, &self.cwd, &self.env)?;
        // Background calls request no finite input.
        drop(launched.stdin);
        let control = Arc::new(WindowsJobControl { job: launched.job });
        let (tx, rx) = tokio::sync::mpsc::channel(BACKGROUND_PROCESS_EVENT_CAPACITY);
        let stdout = launched.stdout;
        let stderr = launched.stderr;
        // Raw Windows handles are pointers and are not `Send`; carry only the
        // numeric handle value across Tokio task boundaries.
        let process = launched.process as usize;
        tokio::spawn(async move {
            let stdout_task = tokio::spawn(pump_windows_output(
                stdout,
                BackgroundOutputStream::Stdout,
                tx.clone(),
            ));
            let stderr_task = tokio::spawn(pump_windows_output(
                stderr,
                BackgroundOutputStream::Stderr,
                tx.clone(),
            ));
            let wait = tokio::task::spawn_blocking(move || unsafe {
                let process = process as windows_sys::Win32::Foundation::HANDLE;
                let result = windows_sys::Win32::System::Threading::WaitForSingleObject(
                    process,
                    windows_sys::Win32::System::Threading::INFINITE,
                );
                let mut code = 1u32;
                let _ =
                    windows_sys::Win32::System::Threading::GetExitCodeProcess(process, &mut code);
                windows_sys::Win32::Foundation::CloseHandle(process);
                (result, code)
            })
            .await;
            let output = (stdout_task.await, stderr_task.await);
            let event = match (wait, output) {
                (
                    Ok((windows_sys::Win32::Foundation::WAIT_OBJECT_0, code)),
                    (Ok(Ok(())), Ok(Ok(()))),
                ) => BackgroundProcessEvent::Exited(BackgroundProcessExit {
                    code: Some(code as i32),
                    success: code == 0,
                }),
                (wait, output) => BackgroundProcessEvent::SupervisionFailed(format!(
                    "Windows background supervision failed: wait={wait:?}, output={output:?}"
                )),
            };
            let _ = tx.send(event).await;
        });
        Ok(LaunchedBackgroundJob {
            control,
            events: rx,
        })
    }
}

#[cfg(windows)]
async fn pump_windows_output(
    mut reader: tokio::fs::File,
    stream: BackgroundOutputStream,
    sender: tokio::sync::mpsc::Sender<BackgroundProcessEvent>,
) -> Result<(), String> {
    use tokio::io::AsyncReadExt;
    let mut buffer = [0u8; 4096];
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Windows background {stream:?} read failed: {e}"))?;
        if count == 0 {
            return Ok(());
        }
        sender
            .send(BackgroundProcessEvent::Output(BackgroundOutputChunk {
                stream,
                bytes: buffer[..count].to_vec(),
                captured_at: std::time::SystemTime::now(),
            }))
            .await
            .map_err(|_| "background supervisor stopped receiving output".to_owned())?;
    }
}

#[cfg(windows)]
struct WindowsLaunchHandles {
    job: windows_sys::Win32::Foundation::HANDLE,
    process: windows_sys::Win32::Foundation::HANDLE,
    stdout: tokio::fs::File,
    stderr: tokio::fs::File,
    stdin: Option<tokio::fs::File>,
}

#[cfg(windows)]
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WindowsLaunchFault {
    Pipe,
    Job,
    AttributeList,
    Assignment,
    Resume,
    UnsupportedHost,
}

#[cfg(windows)]
#[cfg(test)]
#[derive(Default)]
struct WindowsLaunchTestHooks<'a> {
    fault: Option<WindowsLaunchFault>,
    observer: Option<&'a mut windows_tests::LaunchObservation>,
    audit: Option<&'a mut resource_acceptance::ResourceAudit>,
}

#[cfg(windows)]
fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn last_windows_error(operation: &str) -> String {
    format!("{operation} failed: {}", std::io::Error::last_os_error())
}

#[cfg(windows)]
fn create_windows_job(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
    extra_env: &std::collections::BTreeMap<String, String>,
) -> Result<WindowsLaunchHandles, String> {
    create_windows_job_with_hooks(
        program,
        args,
        cwd,
        extra_env,
        #[cfg(test)]
        WindowsLaunchTestHooks::default(),
    )
}

#[cfg(windows)]
fn create_windows_job_with_hooks(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
    extra_env: &std::collections::BTreeMap<String, String>,
    #[cfg(test)] hooks: WindowsLaunchTestHooks<'_>,
) -> Result<WindowsLaunchHandles, String> {
    let executable = resolve_windows_program(program)?;
    let args = args
        .iter()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    let command = WindowsLaunchCommand::new(
        &executable,
        std::ffi::OsStr::new(program),
        &args,
        cwd,
        windows_environment(extra_env)?,
        false,
    )?;
    create_windows_job_native(
        command,
        #[cfg(test)]
        hooks,
    )
}

/// Fully encoded launch inputs. Native resource ownership never re-reads PATH
/// or the environment; foreground callers can supply their approved snapshot.
#[cfg(windows)]
struct WindowsLaunchCommand {
    executable: Vec<u16>,
    command: Vec<u16>,
    cwd: Vec<u16>,
    environment: Vec<u16>,
    input: WindowsInput,
}

#[cfg(windows)]
impl WindowsLaunchCommand {
    fn new(
        executable: &std::path::Path,
        arg0: &std::ffi::OsStr,
        args: &[std::ffi::OsString],
        cwd: &std::path::Path,
        environment: Vec<u16>,
        pipe_stdin: bool,
    ) -> Result<Self, String> {
        use std::os::windows::ffi::OsStrExt;
        let executable = wide(executable.as_os_str());
        let cwd = wide(cwd.as_os_str());
        let arg0 = arg0.encode_wide().collect::<Vec<_>>();
        if executable[..executable.len() - 1].contains(&0)
            || cwd[..cwd.len() - 1].contains(&0)
            || arg0.iter().any(|unit| matches!(*unit, 0 | 34))
        {
            return Err("Windows launch path contains NUL or an unsupported program quote".into());
        }
        let arguments = std::iter::once(arg0)
            .chain(args.iter().map(|arg| arg.encode_wide().collect()))
            .collect::<Vec<_>>();
        if !environment.ends_with(&[0, 0]) {
            return Err("Windows environment block is not terminated".into());
        }
        Ok(Self {
            executable,
            command: windows_command_line::encode(&arguments)?,
            cwd,
            environment,
            input: if pipe_stdin {
                WindowsInput::Pipe(WindowsInputPipe::create()?)
            } else {
                WindowsInput::null()?
            },
        })
    }
}

#[cfg(windows)]
enum WindowsInput {
    Pipe(WindowsInputPipe),
    Null(std::fs::File),
}

#[cfg(windows)]
impl WindowsInput {
    fn child(&self) -> windows_sys::Win32::Foundation::HANDLE {
        use std::os::windows::io::AsRawHandle;
        match self {
            Self::Pipe(pipe) => pipe.child.as_raw_handle() as _,
            Self::Null(file) => file.as_raw_handle() as _,
        }
    }

    fn into_writer(self) -> Option<tokio::fs::File> {
        match self {
            Self::Pipe(pipe) => {
                drop(pipe.child);
                Some(tokio::fs::File::from_std(pipe.parent))
            }
            Self::Null(_) => None,
        }
    }

    fn null() -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
        let input = std::fs::File::open(r"\\.\NUL")
            .map_err(|error| format!("open Windows stdin NUL: {error}"))?;
        // SAFETY: uniquely owned live handle; only this handle's inheritance flag
        // changes. The native launcher includes it in its explicit allowlist.
        if unsafe {
            SetHandleInformation(
                input.as_raw_handle() as _,
                HANDLE_FLAG_INHERIT,
                HANDLE_FLAG_INHERIT,
            )
        } == 0
        {
            return Err(last_windows_error("SetHandleInformation(stdin NUL)"));
        }
        Ok(Self::Null(input))
    }
}

/// Pipe endpoints are RAII-owned before entering the raw launch boundary.
#[cfg(windows)]
struct WindowsInputPipe {
    child: std::fs::File,
    parent: std::fs::File,
}

#[cfg(windows)]
impl WindowsInputPipe {
    fn create() -> Result<Self, String> {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
        use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
        let security = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        let mut read = std::ptr::null_mut();
        let mut write = std::ptr::null_mut();
        // SAFETY: ADR-068's accepted I300 extension permits the same native
        // pipe/allowlist boundary for finite foreground stdin. Each successful
        // endpoint is immediately transferred to a unique RAII file owner.
        unsafe {
            if windows_sys::Win32::System::Pipes::CreatePipe(&mut read, &mut write, &security, 0)
                == 0
            {
                return Err(last_windows_error("CreatePipe(stdin)"));
            }
            let child = std::fs::File::from_raw_handle(read as _);
            let parent = std::fs::File::from_raw_handle(write as _);
            if SetHandleInformation(write, HANDLE_FLAG_INHERIT, 0) == 0 {
                return Err(last_windows_error("SetHandleInformation(stdin)"));
            }
            Ok(Self { child, parent })
        }
    }
}

#[cfg(windows)]
fn create_windows_job_native(
    mut command: WindowsLaunchCommand,
    #[cfg(test)] mut hooks: WindowsLaunchTestHooks<'_>,
) -> Result<WindowsLaunchHandles, String> {
    use std::mem::{size_of, zeroed};
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::{
        CloseHandle, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, SetHandleInformation,
    };
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW,
        DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT,
        InitializeProcThreadAttributeList, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION,
        ResumeThread, STARTUPINFOEXW, UpdateProcThreadAttribute,
    };

    #[cfg(test)]
    if hooks.fault == Some(WindowsLaunchFault::UnsupportedHost) {
        return Err(
            "background_process_tree_unsupported: nested Windows Job Object host".to_owned(),
        );
    }

    if size_of::<HANDLE>() != size_of::<isize>() {
        return Err("unsupported Windows handle width".to_owned());
    }
    let security = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: std::ptr::null_mut(),
        bInheritHandle: 1,
    };
    let mut out_read: HANDLE = std::ptr::null_mut();
    let mut out_write: HANDLE = std::ptr::null_mut();
    let mut err_read: HANDLE = std::ptr::null_mut();
    let mut err_write: HANDLE = std::ptr::null_mut();
    let mut job: HANDLE = std::ptr::null_mut();
    let mut process: HANDLE = std::ptr::null_mut();
    let mut thread: HANDLE = std::ptr::null_mut();
    let mut attrs: windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST =
        std::ptr::null_mut();
    let mut attr_storage = Vec::<usize>::new();
    #[cfg(test)]
    let mut attrs_initialized = false;
    // UpdateProcThreadAttribute borrows this storage until Delete, including
    // failure cleanup outside the launch closure in production and tests.
    let mut inherited_handle_list = Vec::with_capacity(3);
    #[cfg(test)]
    macro_rules! acquired {
        ($role:ident, $value:expr) => {
            if let Some(audit) = hooks.audit.as_deref_mut() {
                audit.acquired(resource_acceptance::Role::$role, $value as usize);
            }
        };
    }
    // Test-only fixed observers cannot propagate errors/panics past live raw owners.
    #[cfg(test)]
    macro_rules! observe {
        ($phase:literal) => {
            if let Some(observer) = hooks.observer.as_deref_mut() {
                observer.record(
                    $phase,
                    [
                        out_read, out_write, err_read, err_write, job, process, thread,
                    ],
                );
            }
        };
    }
    #[cfg(test)]
    macro_rules! checked_close {
        ($role:ident, $handle:expr) => {
            if let Some(audit) = hooks.audit.as_deref_mut() {
                audit.close(resource_acceptance::Role::$role, $handle as usize, || {
                    CloseHandle($handle)
                })
            } else {
                CloseHandle($handle)
            }
        };
    }
    // Keep raw owners in this scope so an unwind takes the same cleanup path
    // as an ordinary launch failure. Attribute backing storage also stays live.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        #[cfg(test)]
        observe!("before_pipe");
        if CreatePipe(&mut out_read, &mut out_write, &security, 0) == 0 {
            return Err(last_windows_error("CreatePipe"));
        }
        #[cfg(test)]
        {
            acquired!(OutRead, out_read);
            acquired!(OutWrite, out_write);
            observe!("pipe_created");
        }
        #[cfg(test)]
        if hooks.fault == Some(WindowsLaunchFault::Pipe) {
            return Err("injected pipe setup failure".to_owned());
        }
        if CreatePipe(&mut err_read, &mut err_write, &security, 0) == 0 {
            return Err(last_windows_error("CreatePipe"));
        }
        #[cfg(test)]
        {
            acquired!(ErrRead, err_read);
            acquired!(ErrWrite, err_write);
        }
        if SetHandleInformation(out_read, HANDLE_FLAG_INHERIT, 0) == 0
            || SetHandleInformation(err_read, HANDLE_FLAG_INHERIT, 0) == 0
        {
            return Err(last_windows_error("SetHandleInformation"));
        }
        job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() || job == INVALID_HANDLE_VALUE {
            return Err(last_windows_error("CreateJobObjectW"));
        }
        #[cfg(test)]
        acquired!(Job, job);
        #[cfg(test)]
        if hooks.fault == Some(WindowsLaunchFault::Job) {
            return Err("injected Job Object setup failure".to_owned());
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &mut limits as *mut _ as *mut _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(last_windows_error("SetInformationJobObject"));
        }
        let mut attr_size = 0usize;
        let _ = InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut attr_size);
        let attr_words = attr_size.div_ceil(size_of::<usize>());
        attr_storage = vec![0usize; attr_words];
        let pending_attrs = attr_storage.as_mut_ptr()
            as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;
        if InitializeProcThreadAttributeList(pending_attrs, 1, 0, &mut attr_size) == 0 {
            return Err(last_windows_error("InitializeProcThreadAttributeList"));
        }
        // Publish only initialized lists to the shared failure cleanup.
        attrs = pending_attrs;
        #[cfg(test)]
        {
            attrs_initialized = true;
            acquired!(Attributes, attrs);
        }
        #[cfg(test)]
        if hooks.fault == Some(WindowsLaunchFault::AttributeList) {
            return Err("injected attribute-list setup failure".to_owned());
        }
        inherited_handle_list.extend([out_write, err_write, command.input.child()]);
        let handles = &inherited_handle_list;
        if UpdateProcThreadAttribute(
            attrs,
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            handles.as_ptr() as *const _,
            handles.len() * size_of::<HANDLE>(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(last_windows_error("UpdateProcThreadAttribute"));
        }
        let mut startup: STARTUPINFOEXW = zeroed();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attrs;
        startup.StartupInfo.dwFlags = 0x00000100;
        startup.StartupInfo.hStdOutput = out_write;
        startup.StartupInfo.hStdError = err_write;
        startup.StartupInfo.hStdInput = command.input.child();
        let mut info: PROCESS_INFORMATION = zeroed();
        #[cfg(test)]
        observe!("before_process");
        if CreateProcessW(
            command.executable.as_mut_ptr(),
            command.command.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            1,
            CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            command.environment.as_mut_ptr() as *mut _,
            command.cwd.as_ptr(),
            &startup as *const STARTUPINFOEXW as *const _,
            &mut info,
        ) == 0
        {
            return Err(last_windows_error("CreateProcessW"));
        }
        process = info.hProcess;
        thread = info.hThread;
        #[cfg(test)]
        {
            acquired!(Process, process);
            acquired!(Thread, thread);
            observe!("process_created");
        }
        #[cfg(test)]
        let assignment_failed = hooks.fault == Some(WindowsLaunchFault::Assignment)
            || AssignProcessToJobObject(job, process) == 0;
        #[cfg(not(test))]
        let assignment_failed = AssignProcessToJobObject(job, process) == 0;
        if assignment_failed {
            #[cfg(test)]
            if hooks.fault == Some(WindowsLaunchFault::Assignment) {
                return Err("injected AssignProcessToJobObject failure".to_owned());
            }
            return Err(format!(
                "background_process_tree_unsupported: {}",
                last_windows_error("AssignProcessToJobObject")
            ));
        }
        #[cfg(test)]
        let resume_failed =
            hooks.fault == Some(WindowsLaunchFault::Resume) || ResumeThread(thread) == u32::MAX;
        #[cfg(not(test))]
        let resume_failed = ResumeThread(thread) == u32::MAX;
        if resume_failed {
            #[cfg(test)]
            if hooks.fault == Some(WindowsLaunchFault::Resume) {
                return Err("injected ResumeThread failure".to_owned());
            }
            return Err(last_windows_error("ResumeThread"));
        }
        CloseHandle(thread);
        thread = std::ptr::null_mut();
        DeleteProcThreadAttributeList(attrs);
        attrs = std::ptr::null_mut();
        CloseHandle(out_write);
        out_write = std::ptr::null_mut();
        CloseHandle(err_write);
        err_write = std::ptr::null_mut();
        let stdout = std::fs::File::from_raw_handle(out_read as _);
        out_read = std::ptr::null_mut();
        let stdout = tokio::fs::File::from_std(stdout);
        let stderr = std::fs::File::from_raw_handle(err_read as _);
        err_read = std::ptr::null_mut();
        let stderr = tokio::fs::File::from_std(stderr);
        Ok(WindowsLaunchHandles {
            job,
            process,
            stdout,
            stderr,
            stdin: command.input.into_writer(),
        })
    }))
    .unwrap_or_else(|_| Err("Windows native launch panicked; cleanup required".to_owned()));
    if result.is_err() {
        #[cfg(test)]
        observe!("before_cleanup");
        #[cfg(test)]
        let mut cleanup_failures = 0u16;
        unsafe {
            if !thread.is_null() {
                #[cfg(test)]
                let _closed = checked_close!(Thread, thread);
                #[cfg(not(test))]
                let _closed = CloseHandle(thread);
                #[cfg(test)]
                if _closed == 0 {
                    cleanup_failures |= 1;
                }
            }
            if !process.is_null() {
                #[cfg(test)]
                let _terminated = if let Some(audit) = hooks.audit.as_deref_mut() {
                    audit.terminate(
                        process as usize,
                        || windows_sys::Win32::System::Threading::TerminateProcess(process, 1),
                        |millis| {
                            windows_sys::Win32::System::Threading::WaitForSingleObject(
                                process, millis,
                            )
                        },
                    )
                } else {
                    windows_sys::Win32::System::Threading::TerminateProcess(process, 1)
                };
                #[cfg(not(test))]
                let _terminated =
                    windows_sys::Win32::System::Threading::TerminateProcess(process, 1);
                #[cfg(test)]
                let _closed = checked_close!(Process, process);
                #[cfg(not(test))]
                let _closed = CloseHandle(process);
                #[cfg(test)]
                {
                    if _terminated == 0 {
                        cleanup_failures |= 1 << 1;
                    }
                    if _closed == 0 {
                        cleanup_failures |= 1 << 2;
                    }
                }
            }
            if !attrs.is_null() {
                #[cfg(test)]
                if let Some(audit) = hooks.audit.as_deref_mut() {
                    audit.delete(attrs as usize, attrs_initialized, || {
                        DeleteProcThreadAttributeList(attrs)
                    });
                } else {
                    DeleteProcThreadAttributeList(attrs);
                }
                #[cfg(not(test))]
                DeleteProcThreadAttributeList(attrs);
            }
            if !job.is_null() {
                #[cfg(test)]
                let _closed = checked_close!(Job, job);
                #[cfg(not(test))]
                let _closed = CloseHandle(job);
                #[cfg(test)]
                if _closed == 0 {
                    cleanup_failures |= 1 << 3;
                }
            }
            #[cfg(test)]
            let mut pipe_bit = 1u16 << 4;
            for handle in [out_read, out_write, err_read, err_write] {
                if !handle.is_null() {
                    #[cfg(test)]
                    let _closed = if let Some(audit) = hooks.audit.as_deref_mut() {
                        audit.close_pipe(handle as usize, || CloseHandle(handle))
                    } else {
                        CloseHandle(handle)
                    };
                    #[cfg(not(test))]
                    let _closed = CloseHandle(handle);
                    #[cfg(test)]
                    if _closed == 0 {
                        cleanup_failures |= pipe_bit;
                    }
                }
                #[cfg(test)]
                {
                    pipe_bit <<= 1;
                }
            }
        }
        // Report only after every cleanup operation has been attempted. Test failures
        // must not short-circuit release of a later handle or termination of the child.
        #[cfg(test)]
        observe!("after_cleanup");
        #[cfg(test)]
        assert_eq!(cleanup_failures, 0, "Windows launch cleanup failure mask");
    }
    result
}

#[cfg(windows)]
fn windows_environment(
    extra: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<u16>, String> {
    let dangerous = talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
    let entries = std::env::vars_os().chain(extra.iter().map(|(name, value)| {
        (
            std::ffi::OsString::from(name),
            std::ffi::OsString::from(value),
        )
    }));
    windows_environment::snapshot(entries.filter(|(name, _)| {
        !dangerous.iter().any(|item| {
            name.to_str()
                .is_some_and(|name| item.eq_ignore_ascii_case(name))
        })
    }))
}

#[cfg(windows)]
fn resolve_windows_program(program: &str) -> Result<std::path::PathBuf, String> {
    let candidate = std::path::PathBuf::from(program);
    if candidate.is_absolute() || candidate.components().count() > 1 {
        return candidate
            .is_file()
            .then_some(candidate)
            .ok_or_else(|| format!("Windows executable does not exist: {program}"));
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    for directory in std::env::split_paths(&path) {
        let direct = directory.join(program);
        if direct.is_file() {
            return Ok(direct);
        }
        if direct.extension().is_none() {
            let exe = direct.with_extension("exe");
            if exe.is_file() {
                return Ok(exe);
            }
        }
    }
    Err(format!(
        "Windows executable was not found on PATH: {program}"
    ))
}

#[cfg(test)]
mod native_inventory;

#[cfg(all(test, windows))]
mod resource_acceptance;

#[cfg(all(test, windows))]
mod windows_launch_command_tests;

#[cfg(all(test, windows))]
mod handle_snapshot;

#[cfg(all(test, windows))]
mod handle_lifecycle_control;

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use talos_core::background_job::{BackgroundJobLauncher, BackgroundProcessEvent};

    const RETURNED_ORDER: [&str; 7] = [
        "stdout_read",
        "stdout_write",
        "stderr_read",
        "stderr_write",
        "job",
        "process",
        "thread",
    ];

    #[derive(serde::Serialize)]
    struct CallSample {
        phase: &'static str,
        before_snapshot: u32,
        after_snapshot: u32,
        // Values returned to this launcher, NOT still-live ownership or object IDs.
        returned: [usize; 7],
        #[serde(skip)]
        handles: handle_snapshot::Handles,
    }

    pub(super) struct LaunchObservation {
        phases: [&'static str; 4],
        samples: [Option<CallSample>; 4],
        next: usize,
        error: Option<String>,
    }

    impl LaunchObservation {
        fn new(mode: u8) -> Self {
            Self {
                phases: if mode == 5 {
                    [
                        "before_pipe",
                        "pipe_created",
                        "before_cleanup",
                        "after_cleanup",
                    ]
                } else {
                    [
                        "before_process",
                        "process_created",
                        "before_cleanup",
                        "after_cleanup",
                    ]
                },
                samples: std::array::from_fn(|_| None),
                next: 0,
                error: None,
            }
        }

        pub(super) fn record(
            &mut self,
            phase: &'static str,
            returned: [windows_sys::Win32::Foundation::HANDLE; 7],
        ) {
            if self.error.is_some() || !self.phases.contains(&phase) {
                return;
            }
            // Cover count, capture, numeric copy AND storage. Never unwind past the
            // launcher's live raw resources or return early from its cleanup.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if self.next >= 4 || self.phases[self.next] != phase {
                    return Err("call checkpoint order/cap".to_owned());
                }
                let before_snapshot = current_process_handle_count();
                let handles = handle_snapshot::capture()?;
                let after_snapshot = current_process_handle_count();
                self.samples[self.next] = Some(CallSample {
                    phase,
                    before_snapshot,
                    after_snapshot,
                    returned: returned.map(|handle| handle as usize),
                    handles,
                });
                self.next += 1;
                Ok(())
            }))
            .unwrap_or_else(|_| Err("call observer panicked; evidence incomplete".to_owned()));
            if let Err(error) = result {
                self.error = Some(error);
            }
        }

        fn finish(self) -> Result<Vec<u8>, String> {
            if let Some(error) = self.error {
                return Err(error);
            }
            let samples = self
                .samples
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or("missing call checkpoint; evidence incomplete")?;
            let calls = serde_json::to_value(&samples).map_err(|_| "call receipt encoding")?;
            let phases = samples
                .into_iter()
                .map(|sample| (sample.phase, sample.handles))
                .collect::<Vec<_>>();
            let mut receipt: serde_json::Value =
                serde_json::from_slice(&handle_snapshot::differences(&phases)?)
                    .map_err(|_| "call differences JSON")?;
            receipt["calls"] = calls;
            receipt["returned_order"] = serde_json::json!(RETURNED_ORDER);
            let output = serde_json::to_vec(&receipt).map_err(|_| "call receipt encoding")?;
            if output.len() > handle_snapshot::OUTPUT_CAP - 4096 {
                return Err("call receipt byte cap".to_owned());
            }
            Ok(output)
        }
    }

    fn powershell(script: &str) -> WindowsBackgroundLauncher {
        WindowsBackgroundLauncher::new(
            std::env::var("WINDIR").map_or_else(
                |_| "powershell.exe".to_owned(),
                |windir| format!(r#"{windir}\System32\WindowsPowerShell\v1.0\powershell.exe"#),
            ),
            vec![
                "-NoLogo".to_owned(),
                "-NoProfile".to_owned(),
                "-NonInteractive".to_owned(),
                "-Command".to_owned(),
                script.to_owned(),
            ],
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            BTreeMap::new(),
        )
    }

    #[test]
    fn windows_environment_filters_dangerous_names_case_insensitively() {
        let dangerous = talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
        let Some(name) = dangerous.first() else {
            return;
        };
        let mut extra = BTreeMap::new();
        extra.insert(name.to_ascii_lowercase(), "must-not-appear".to_owned());
        let block =
            String::from_utf16_lossy(&windows_environment(&extra).expect("valid environment"));
        assert!(!block.contains("must-not-appear"));
    }

    #[tokio::test]
    async fn windows_job_captures_output_and_reaps_process() {
        let launched = Box::new(powershell(
            "Write-Output 'stdout-ok'; [Console]::Error.WriteLine('stderr-ok')",
        ))
        .launch()
        .await
        .expect("Windows Job Object launcher succeeds");
        let mut events = launched.events;
        let mut output = Vec::new();
        let exit = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = events.recv().await {
                match event {
                    BackgroundProcessEvent::Output(chunk) => output.extend(chunk.bytes),
                    BackgroundProcessEvent::Exited(exit) => return Some(exit),
                    BackgroundProcessEvent::SupervisionFailed(error) => panic!("{error}"),
                }
            }
            None
        })
        .await
        .expect("Windows process must reach a terminal event")
        .expect("Windows process must emit an exit event");

        assert!(exit.success);
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("stdout-ok"));
        assert!(output.contains("stderr-ok"));
    }

    #[tokio::test]
    async fn windows_job_preserves_descendant_output_after_leader_exit() {
        // The child waits for the leader itself, so its delayed output cannot
        // accidentally precede leader exit on a slow CI runner.
        let script = r#"$childCommand = 'Wait-Process -Id ' + $PID + ' -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 500; Write-Output descendant-after-leader-exit'; $child = Start-Process powershell.exe -ArgumentList @('-NoLogo', '-NoProfile', '-NonInteractive', '-Command', ('"' + $childCommand + '"')) -NoNewWindow -PassThru; exit 0"#;
        let launched = Box::new(powershell(script))
            .launch()
            .await
            .expect("Windows Job Object launcher succeeds");
        let control = launched.control;
        let mut events = launched.events;
        let mut output = Vec::new();
        let terminal = tokio::time::timeout(Duration::from_secs(20), async {
            while let Some(event) = events.recv().await {
                match event {
                    BackgroundProcessEvent::Output(chunk) => output.extend(chunk.bytes),
                    terminal => return Some(terminal),
                }
            }
            None
        })
        .await;

        // Clean up before asserting even if the terminal event never arrived.
        // Dropping the retained control also closes the kill-on-close Job.
        let cleanup = control.force_terminate().await;
        drop(control);
        if terminal.is_err() {
            let _ = tokio::time::timeout(Duration::from_secs(10), async {
                while let Some(event) = events.recv().await {
                    if !matches!(event, BackgroundProcessEvent::Output(_)) {
                        break;
                    }
                }
            })
            .await;
        }
        assert!(cleanup.is_ok(), "Job cleanup failed: {cleanup:?}");
        let terminal = terminal
            .expect("descendant output must reach EOF within the test deadline")
            .expect("launcher must emit a terminal event");
        assert!(
            matches!(terminal, BackgroundProcessEvent::Exited(ref exit) if exit.success),
            "unexpected terminal event: {terminal:?}"
        );
        assert!(
            String::from_utf8_lossy(&output).contains("descendant-after-leader-exit"),
            "leader exit must not truncate delayed descendant output"
        );
    }

    #[tokio::test]
    async fn windows_child_runs_only_after_job_assignment_and_resume() {
        let marker =
            std::env::temp_dir().join(format!("talos-i226-assigned-marker-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let script = format!(
            "Set-Content -LiteralPath '{}' -Value assigned",
            marker.display()
        );
        let launched = Box::new(powershell(&script))
            .launch()
            .await
            .expect("assigned Windows Job Object launcher succeeds");
        let mut events = launched.events;
        while let Some(event) = events.recv().await {
            if matches!(event, BackgroundProcessEvent::Exited(_)) {
                break;
            }
        }
        assert!(marker.exists(), "child marker must be written after resume");
        let _ = std::fs::remove_file(marker);
    }

    #[tokio::test]
    async fn windows_job_termination_reaches_terminal_event() {
        let launched = Box::new(powershell("Start-Sleep -Seconds 30"))
            .launch()
            .await
            .expect("Windows Job Object launcher succeeds");
        let control = launched.control;
        let mut events = launched.events;
        tokio::time::sleep(Duration::from_millis(100)).await;
        control
            .terminate()
            .await
            .expect("Job Object termination succeeds");

        let exit = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = events.recv().await {
                if let BackgroundProcessEvent::Exited(exit) = event {
                    return Some(exit);
                }
            }
            None
        })
        .await
        .expect("terminated Windows process must be reaped")
        .expect("terminated Windows process must emit an exit event");
        assert!(!exit.success);
    }

    #[tokio::test]
    async fn windows_job_termination_reaps_a_powershell_grandchild() {
        let script = "$child = Start-Process powershell.exe -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 30' -PassThru; Write-Output ('grandchild-pid=' + $child.Id); Start-Sleep -Seconds 30";
        let launched = Box::new(powershell(script))
            .launch()
            .await
            .expect("Windows Job Object launcher succeeds");
        let control = launched.control;
        let mut events = launched.events;
        let mut output = Vec::new();
        let grandchild_pid = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = events.recv().await {
                if let BackgroundProcessEvent::Output(chunk) = event {
                    output.extend(chunk.bytes);
                    let text = String::from_utf8_lossy(&output);
                    if let Some(pid) = text
                        .lines()
                        .find_map(|line| line.strip_prefix("grandchild-pid=")?.trim().parse().ok())
                    {
                        return Some(pid);
                    }
                }
            }
            None
        })
        .await
        .expect("grandchild marker must arrive")
        .expect("grandchild marker must contain a PID");

        control
            .terminate()
            .await
            .expect("Job Object termination succeeds");
        tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = events.recv().await {
                if matches!(event, BackgroundProcessEvent::Exited(_)) {
                    break;
                }
            }
        })
        .await
        .expect("leader must be reaped after Job termination");

        for _ in 0..20 {
            if !windows_process_exists(grandchild_pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("Job termination left grandchild process {grandchild_pid} alive");
    }

    async fn windows_handle_probe(launcher: WindowsBackgroundLauncher) -> bool {
        let launched = Box::new(launcher)
            .launch()
            .await
            .expect("Windows Job Object launcher succeeds");
        let mut events = launched.events;
        let mut output = Vec::new();
        while let Some(event) = events.recv().await {
            match event {
                BackgroundProcessEvent::Output(chunk) => output.extend(chunk.bytes),
                BackgroundProcessEvent::Exited(exit) => {
                    assert!(exit.success, "handle probe must exit successfully");
                    break;
                }
                BackgroundProcessEvent::SupervisionFailed(error) => panic!("{error}"),
            }
        }
        String::from_utf8_lossy(&output)
            .lines()
            .any(|line| line.trim() == "1")
    }

    #[tokio::test]
    async fn windows_concurrent_spawn_excludes_unrelated_inheritable_handle() {
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
        use windows_sys::Win32::System::Threading::CreateEventW;

        let security = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        let unrelated: HANDLE = unsafe { CreateEventW(&security, 0, 0, std::ptr::null()) };
        assert!(
            !unrelated.is_null(),
            "unrelated inheritable handle must exist"
        );

        let script = r#"Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class TalosHandleProbe { [DllImport("kernel32.dll")] public static extern bool GetHandleInformation(IntPtr handle, out uint flags); }'; $flags = 0; if ([TalosHandleProbe]::GetHandleInformation([IntPtr]([long]$args[0]), [ref]$flags)) { '1' } else { '0' }"#;
        let handle_arg = (unrelated as usize).to_string();
        let mut left_launcher = powershell(script);
        left_launcher.args.push(handle_arg.clone());
        let mut right_launcher = powershell(script);
        right_launcher.args.push(handle_arg);
        let (left, right) = tokio::join!(
            windows_handle_probe(left_launcher),
            windows_handle_probe(right_launcher),
        );
        unsafe { CloseHandle(unrelated) };

        assert!(
            !left && !right,
            "unrelated inheritable handle must not be observable in either child"
        );
    }

    #[test]
    fn windows_unsupported_job_host_is_rejected_fail_closed() {
        let launcher = powershell("Write-Output should-not-run");
        let error = match create_windows_job_with_hooks(
            &launcher.program,
            &launcher.args,
            &launcher.cwd,
            &launcher.env,
            WindowsLaunchTestHooks {
                fault: Some(WindowsLaunchFault::UnsupportedHost),
                ..Default::default()
            },
        ) {
            Ok(_) => panic!("unsupported Job host must fail closed"),
            Err(error) => error,
        };
        assert!(error.contains("background_process_tree_unsupported"));
    }

    #[tokio::test]
    async fn windows_job_rejects_invalid_working_directory_without_spawning() {
        let launcher = WindowsBackgroundLauncher::new(
            std::env::var("WINDIR").map_or_else(
                |_| "powershell.exe".to_owned(),
                |windir| format!(r#"{windir}\System32\WindowsPowerShell\v1.0\powershell.exe"#),
            ),
            vec!["-NoLogo".to_owned()],
            PathBuf::from(r"C:\talos\path-that-does-not-exist"),
            BTreeMap::new(),
        );
        let error = match Box::new(launcher).launch().await {
            Ok(_) => panic!("invalid working directory must fail closed"),
            Err(error) => error,
        };
        assert!(error.contains("CreateProcessW"));
    }

    #[tokio::test]
    async fn windows_assignment_failure_cleans_suspended_process_without_running_marker() {
        let marker = std::env::temp_dir().join(format!(
            "talos-i226-assignment-marker-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&marker);
        let script = format!(
            "Set-Content -LiteralPath '{}' -Value marker",
            marker.display()
        );
        let launcher = powershell(&script);
        let error = match create_windows_job_with_hooks(
            &launcher.program,
            &launcher.args,
            &launcher.cwd,
            &launcher.env,
            WindowsLaunchTestHooks {
                fault: Some(WindowsLaunchFault::Assignment),
                ..Default::default()
            },
        ) {
            Ok(_) => panic!("injected assignment failure must fail closed"),
            Err(error) => error,
        };
        assert!(error.contains("AssignProcessToJobObject"));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!marker.exists(), "suspended child ran before assignment");
    }

    #[tokio::test]
    async fn windows_resume_failure_cleans_assigned_process_and_handles() {
        let launcher = powershell("Start-Sleep -Seconds 30");
        let error = match create_windows_job_with_hooks(
            &launcher.program,
            &launcher.args,
            &launcher.cwd,
            &launcher.env,
            WindowsLaunchTestHooks {
                fault: Some(WindowsLaunchFault::Resume),
                ..Default::default()
            },
        ) {
            Ok(_) => panic!("injected resume failure must fail closed"),
            Err(error) => error,
        };
        assert!(error.contains("ResumeThread"));
    }

    #[tokio::test]
    async fn windows_partial_setup_failures_release_every_parent_handle() {
        const TEST: &str = "process_boundary::windows_tests::windows_partial_setup_failures_release_every_parent_handle";
        const CHILD_RECEIPT: &str = "TALOS_WINDOWS_HANDLE_COUNT_RECEIPT";
        const MODE: &str = "TALOS_WINDOWS_HANDLE_SNAPSHOT_MODE";
        let Some(receipt) = std::env::var_os(CHILD_RECEIPT) else {
            // GetProcessHandleCount covers the whole process. Other concurrently running
            // tests must not change this fixture's baseline or final sample.
            let mut outcomes = Vec::new();
            let mut controls = Vec::new();
            // Cold, observer, native cleanup fixtures and stage controls: owned processes. Retain
            // historical failures; attempt all workers before asserting any outcome.
            for mode in [0u8, 1, 2, 3, 4, 5, 6, 7, 8] {
                let directory = tempfile::Builder::new()
                    .prefix("talos-handle-count-")
                    .tempdir()
                    .expect("owned receipt directory");
                let receipt = directory.path().join("receipt.json");
                let mut environment = BTreeMap::new();
                environment.insert(
                    CHILD_RECEIPT.to_owned(),
                    receipt.to_str().expect("receipt path").to_owned(),
                );
                // Explicit zero prevents an inherited observer marker warming the cold child.
                environment.insert(MODE.to_owned(), mode.to_string());
                // The intentional caught-panic fixture must not inherit an unbounded
                // backtrace setting; no process-global environment mutation is used.
                environment.insert("RUST_BACKTRACE".to_owned(), "0".to_owned());
                // The outer Job also owns descendants created before the inner fault
                // reaches assignment. Its last control drop kills the whole tree.
                let launcher = WindowsBackgroundLauncher::new(
                    std::env::current_exe()
                        .expect("test executable")
                        .into_os_string()
                        .into_string()
                        .expect("test executable path"),
                    ["--exact", TEST, "--test-threads=1", "--nocapture"]
                        .map(str::to_owned)
                        .to_vec(),
                    std::env::current_dir().expect("test working directory"),
                    environment,
                );
                let launched = match Box::new(launcher).launch().await {
                    Ok(launched) => launched,
                    Err(error) => {
                        outcomes.push((mode, Err(format!("launch failed: {error}"))));
                        continue;
                    }
                };
                let control = launched.control;
                let mut events = launched.events;
                let mut output = Vec::new();
                let mut overflow = false;
                let result = tokio::time::timeout(Duration::from_secs(60), async {
                    while let Some(event) = events.recv().await {
                        match event {
                            BackgroundProcessEvent::Output(chunk) => {
                                let remaining = 16_384 - output.len();
                                overflow |= chunk.bytes.len() > remaining;
                                // Retention is bounded; keep draining after the cap.
                                output.extend_from_slice(
                                    &chunk.bytes[..chunk.bytes.len().min(remaining)],
                                );
                            }
                            BackgroundProcessEvent::Exited(exit) => return Ok(exit.success),
                            BackgroundProcessEvent::SupervisionFailed(error) => return Err(error),
                        }
                    }
                    Err("handle child ended without Exited".to_owned())
                })
                .await;
                if !matches!(&result, Ok(Ok(true))) || overflow {
                    let cleanup = control.force_terminate().await;
                    let terminal = matches!(&result, Ok(Ok(_)))
                        || tokio::time::timeout(Duration::from_secs(10), async {
                            while let Some(event) = events.recv().await {
                                if matches!(event, BackgroundProcessEvent::Exited(_)) {
                                    return true;
                                }
                            }
                            false
                        })
                        .await
                        .unwrap_or(false);
                    eprintln!("handle fixture cleanup={cleanup:?} terminal={terminal}");
                }
                eprintln!("{}", String::from_utf8_lossy(&output));
                let validation = if matches!(result, Ok(Ok(true))) && !overflow {
                    validate_handle_receipt(&receipt, mode)
                } else {
                    Err(format!("exit={result:?}; overflow={overflow}"))
                };
                outcomes.push((mode, validation));
                controls.push(control); // retain Jobs through every worker/receipt check
            }
            eprintln!("handle worker outcomes={outcomes:?}");
            assert_eq!(outcomes.len(), 9);
            for (mode, result) in outcomes {
                assert!(result.is_ok(), "handle worker mode={mode}: {result:?}");
            }
            drop(controls);
            return;
        };
        assert_eq!(
            std::env::args().skip(1).collect::<Vec<_>>(),
            ["--exact", TEST, "--test-threads=1", "--nocapture"],
            "child marker requires the exact isolated invocation"
        );
        let receipt = PathBuf::from(receipt);
        assert_eq!(
            receipt.file_name(),
            Some(std::ffi::OsStr::new("receipt.json"))
        );
        let directory = receipt
            .parent()
            .expect("receipt directory")
            .canonicalize()
            .expect("owned directory");
        assert!(
            directory
                .file_name()
                .expect("directory name")
                .to_string_lossy()
                .starts_with("talos-handle-count-")
        );
        assert_eq!(
            directory.parent(),
            Some(
                std::env::temp_dir()
                    .canonicalize()
                    .expect("temp root")
                    .as_path()
            )
        );
        let mode = match std::env::var_os(MODE) {
            None => 0,
            Some(value) if value == "0" => 0,
            Some(value) if value == "1" => 1,
            Some(value) if value == "2" => 2,
            Some(value) if value == "3" => 3,
            Some(value) if value == "4" => 4,
            Some(value) if value == "5" => 5,
            Some(value) if value == "6" => 6,
            Some(value) if value == "7" => 7,
            Some(value) if value == "8" => 8,
            Some(_) => panic!("invalid owned handle worker mode"),
        };
        if mode == 2 {
            handle_snapshot::native_fixtures(); // exactly four captures in its own Job
            std::fs::write(
                receipt,
                serde_json::to_vec(&serde_json::json!({
                    "child_pid":std::process::id(),"snapshots":4,"native_checks":true
                }))
                .expect("native fixture JSON"),
            )
            .expect("native fixture receipt");
            return;
        }
        let launcher = powershell("Start-Sleep -Seconds 30");
        if mode == 7 {
            let counts = handle_lifecycle_control::run(&launcher)
                .expect("complete direct Win32 lifecycle control");
            // Exercise this control's own partial setup cleanup after measurement.
            // A nonexistent cwd reaches real CreateProcessW failure with live pipes,
            // Job and initialized attributes. Its error is accepted only if cleanup
            // completed without an error mask; no failing output handles are adopted.
            let mut failure_launcher = launcher;
            failure_launcher.cwd = directory.join("missing-direct-control-cwd");
            assert!(!failure_launcher.cwd.exists());
            assert_eq!(
                handle_lifecycle_control::run(&failure_launcher)
                    .err()
                    .as_deref(),
                Some("direct CreateProcessW failed"),
                "must reach real process creation failure and finish checked cleanup"
            );
            // Serialization and receipt I/O follow every sampled lifecycle/cleanup.
            let output = serde_json::to_vec(&serde_json::json!({
                "child_pid":std::process::id(),"direct_cycles":2,"snapshots":0,
                "native_failure_check":true,
                "phases":handle_lifecycle_control::PHASES,"counts":counts,
            }))
            .expect("direct lifecycle JSON");
            assert!(output.len() <= 1024);
            eprintln!(
                "DIRECT_WIN32_LIFECYCLE {}",
                String::from_utf8_lossy(&output)
            );
            std::fs::write(receipt, output).expect("direct lifecycle receipt");
            return;
        }
        if matches!(mode, 5 | 6) {
            let mut observer = LaunchObservation::new(mode);
            let fault = if mode == 5 {
                WindowsLaunchFault::Pipe
            } else {
                WindowsLaunchFault::Assignment
            };
            let result = create_windows_job_with_hooks(
                &launcher.program,
                &launcher.args,
                &launcher.cwd,
                &launcher.env,
                WindowsLaunchTestHooks {
                    fault: Some(fault),
                    observer: Some(&mut observer),
                    ..Default::default()
                },
            );
            assert_eq!(
                result.err().as_deref(),
                Some(if mode == 5 {
                    "injected pipe setup failure"
                } else {
                    "injected AssignProcessToJobObject failure"
                })
            );
            // Explicit receipt assertions/output follow original cleanup. A caught
            // observer panic may already have invoked the ordinary panic hook.
            let output = observer
                .finish()
                .expect("complete bounded call-boundary receipt");
            eprintln!(
                "PSS_CALL_BOUNDARY mode={mode} {}",
                String::from_utf8_lossy(&output)
            );
            std::fs::write(receipt, output).expect("call-boundary receipt");
            return;
        }
        if matches!(mode, 3 | 4) {
            let mut phases = Vec::new();
            let mut observations = Vec::new();
            record_handle_stage("control_baseline", &mut phases, &mut observations);
            if mode == 3 {
                assert_handle_stage_failure(
                    &launcher,
                    &launcher.program,
                    Some(WindowsLaunchFault::Pipe),
                    "injected pipe setup failure",
                );
                record_handle_stage("first_pipe_cleanup", &mut phases, &mut observations);
                // Fail after initializing the attribute list so this stage measures
                // cleanup of acquired native resources, not early input rejection.
                assert_handle_stage_failure(
                    &launcher,
                    &launcher.program,
                    Some(WindowsLaunchFault::AttributeList),
                    "injected attribute-list setup failure",
                );
                record_handle_stage("attribute_list_cleanup", &mut phases, &mut observations);
            } else {
                let resolved =
                    resolve_windows_program(&launcher.program).expect("known fixture program");
                drop(resolved);
                record_handle_stage("program_resolved", &mut phases, &mut observations);
                // Exercise the existing launcher environment builder (which reads/copies
                // the worker's environment), then immediately discard its output. No extra
                // observational copy, inspection, log or persistence of those contents.
                drop(windows_environment(&launcher.env));
                record_handle_stage("environment_built", &mut phases, &mut observations);
            }
            assert_handle_stage_failure(
                &launcher,
                &launcher.program,
                Some(WindowsLaunchFault::Assignment),
                "injected AssignProcessToJobObject failure",
            );
            record_handle_stage("suspended_process_cleanup", &mut phases, &mut observations);
            let output =
                handle_snapshot::differences(&phases).expect("bounded complete stage differences");
            eprintln!("PSS_STAGE_COUNTS mode={mode} observations={observations:?}");
            eprintln!(
                "PSS_STAGE_TYPES mode={mode} {}",
                String::from_utf8_lossy(&output)
            );
            std::fs::write(receipt, output).expect("stage-control receipt");
            return;
        }
        if mode == 8 {
            let summary = resource_acceptance::negative_controls(&launcher);
            let output = serde_json::to_vec(&serde_json::json!({
                "child_pid":std::process::id(), "negative_controls":summary,
            }))
            .expect("negative receipt encoding");
            eprintln!(
                "WINDOWS_RESOURCE_NEGATIVES {}",
                String::from_utf8_lossy(&output)
            );
            std::fs::write(receipt, output).expect("negative receipt write");
            return;
        }
        let mut phases = Vec::new();
        let mut observer_counts = Vec::new();
        if mode == 1 {
            for phase in ["baseline", "observer_control"] {
                let before = current_process_handle_count();
                phases.push((
                    phase,
                    handle_snapshot::capture().expect("checked self snapshot"),
                ));
                observer_counts.push((phase, before, current_process_handle_count()));
            }
        }
        let before = current_process_handle_count();
        // Fixed storage and read-only counts keep diagnostics out of the sampled work.
        // Index 0 is the baseline; each group of eight follows the fault order below.
        let mut counts = [0u32; 41];
        counts[0] = before;
        let mut attempts = 0;
        let mut known = resource_acceptance::Summary::default();
        for fault in [
            WindowsLaunchFault::Pipe,
            WindowsLaunchFault::Job,
            WindowsLaunchFault::AttributeList,
            WindowsLaunchFault::Assignment,
            WindowsLaunchFault::Resume,
        ] {
            for _ in 0..8 {
                let mut audit = resource_acceptance::ResourceAudit::new(fault, attempts + 1, None);
                let result = create_windows_job_with_hooks(
                    &launcher.program,
                    &launcher.args,
                    &launcher.cwd,
                    &launcher.env,
                    WindowsLaunchTestHooks {
                        fault: Some(fault),
                        audit: Some(&mut audit),
                        ..Default::default()
                    },
                );
                assert!(result.is_err(), "{fault:?} must fail closed");
                let expected = match fault {
                    WindowsLaunchFault::Pipe => "injected pipe setup failure",
                    WindowsLaunchFault::Job => "injected Job Object setup failure",
                    WindowsLaunchFault::AttributeList => "injected attribute-list setup failure",
                    WindowsLaunchFault::Assignment => "injected AssignProcessToJobObject failure",
                    WindowsLaunchFault::Resume => "injected ResumeThread failure",
                    WindowsLaunchFault::UnsupportedHost => panic!("not a partial-setup fault"),
                };
                assert_eq!(
                    result.err().expect("injected failure"),
                    expected,
                    "must reach the requested fault, not fail earlier"
                );
                known.add(audit.finish().expect("actual known-resource cleanup"));
                attempts += 1;
                counts[attempts] = current_process_handle_count();
                if mode == 1 && attempts == 1 {
                    let before = current_process_handle_count();
                    phases.push((
                        "first_pipe",
                        handle_snapshot::capture().expect("checked self snapshot"),
                    ));
                    observer_counts.push(("first_pipe", before, current_process_handle_count()));
                }
            }
        }
        let after = current_process_handle_count();
        // Preserve the independent final sample and emit evidence before a failure.
        // This is a diagnostic trace, not a warmup, threshold change or success receipt.
        eprintln!(
            "WINDOWS_HANDLE_COUNT_TRACE fault_order=Pipe,Job,AttributeList,Assignment,Resume attempts_per_fault=8 counts={counts:?} before={before} after={after}"
        );
        if mode == 1 {
            phases.push((
                "final_40",
                handle_snapshot::capture().expect("checked self snapshot"),
            ));
            observer_counts.push(("final_40", after, current_process_handle_count()));
            assert_eq!(attempts, 40);
            let output =
                handle_snapshot::differences(&phases).expect("bounded complete differences");
            eprintln!("PSS_OBSERVER_COUNTS {observer_counts:?}");
            eprintln!("PSS_HANDLE_TYPES {}", String::from_utf8_lossy(&output));
            std::fs::write(receipt, output).expect("snapshot receipt");
            return; // observer metadata cannot substitute for the cold worker resource proof
        }
        // Effective TEST-004/ADR091: raw totals remain diagnostics, not ownership proof.
        known.require_all40();
        // Receipt I/O and diagnostics are deliberately outside the sampled interval.
        assert_eq!(attempts, 40);
        std::fs::write(
            receipt,
            serde_json::to_vec(&serde_json::json!({
                "child_pid": std::process::id(), "attempts": attempts,
                "before": before, "after": after, "known_resources": known,
            }))
            .expect("receipt encoding"),
        )
        .expect("receipt write");
        eprintln!("WINDOWS_HANDLE_COUNT_RECEIPT attempts={attempts} before={before} after={after}");
        eprintln!(
            "WINDOWS_KNOWN_RESOURCE_RECEIPT {}",
            serde_json::to_string(&known).expect("known-resource receipt")
        );
    }

    fn validate_handle_receipt(path: &std::path::Path, mode: u8) -> Result<(), String> {
        let cap = if matches!(mode, 0 | 7 | 8) {
            1024
        } else {
            handle_snapshot::OUTPUT_CAP
        };
        if std::fs::metadata(path)
            .map_err(|_| "missing receipt")?
            .len()
            > cap as u64
        {
            return Err("receipt byte cap".to_owned());
        }
        let bytes = std::fs::read(path).map_err(|_| "receipt read failed")?;
        if bytes.len() > cap {
            return Err("receipt byte cap".to_owned());
        }
        let data: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "invalid receipt JSON")?;
        let pid = data["child_pid"]
            .as_u64()
            .and_then(|pid| u32::try_from(pid).ok())
            .ok_or("invalid child PID")?;
        if pid == 0 || pid == std::process::id() {
            return Err("receipt not another process".to_owned());
        }
        if mode == 8 {
            if data["negative_controls"] != resource_acceptance::negative_receipt() {
                return Err("negative resource controls incomplete".to_owned());
            }
        } else if mode == 7 {
            if data["direct_cycles"].as_u64() != Some(2)
                || data["snapshots"].as_u64() != Some(0)
                || data["native_failure_check"] != true
                || data["phases"] != serde_json::json!(handle_lifecycle_control::PHASES)
                || data["counts"].as_array().is_none_or(|counts| {
                    counts.len() != 4
                        || counts
                            .iter()
                            .any(|n| n.as_u64().and_then(|n| u32::try_from(n).ok()).is_none())
                })
            {
                return Err("invalid direct lifecycle receipt".to_owned());
            }
            // Count growth is diagnostic; mode0 independently proves known-resource cleanup.
        } else if mode == 0 {
            if data["attempts"].as_u64() != Some(40) {
                return Err("wrong fault count".to_owned());
            }
            let before = data["before"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("invalid baseline")?;
            let after = data["after"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("invalid final count")?;
            let _diagnostic_counts = (before, after);
            if data["known_resources"] != resource_acceptance::all40_receipt() {
                return Err("known resource proof incomplete".to_owned());
            }
        } else {
            if data["snapshots"].as_u64() != Some(4) {
                return Err("wrong snapshot count".to_owned());
            }
            if mode != 2
                && data["changes"]
                    .as_array()
                    .is_none_or(|changes| changes.len() > 32)
            {
                return Err("missing/oversized type differences".to_owned());
            }
            if mode != 2 {
                let expected = match mode {
                    1 => ["baseline", "observer_control", "first_pipe", "final_40"],
                    3 => [
                        "control_baseline",
                        "first_pipe_cleanup",
                        "attribute_list_cleanup",
                        "suspended_process_cleanup",
                    ],
                    4 => [
                        "control_baseline",
                        "program_resolved",
                        "environment_built",
                        "suspended_process_cleanup",
                    ],
                    5 => [
                        "before_pipe",
                        "pipe_created",
                        "before_cleanup",
                        "after_cleanup",
                    ],
                    6 => [
                        "before_process",
                        "process_created",
                        "before_cleanup",
                        "after_cleanup",
                    ],
                    _ => return Err("invalid observer mode".to_owned()),
                };
                let phases = data["phase_sizes"]
                    .as_array()
                    .ok_or("missing observer phases")?;
                if phases.len() != expected.len()
                    || phases.iter().zip(expected).any(|(entry, phase)| {
                        entry[0].as_str() != Some(phase)
                            || entry[1].as_u64().is_none_or(|size| size > 1024)
                    })
                {
                    return Err("observer phase order/entry cap invalid".to_owned());
                }
                if matches!(mode, 5 | 6) {
                    if data["returned_order"] != serde_json::json!(RETURNED_ORDER) {
                        return Err("invalid returned-handle role order".to_owned());
                    }
                    let calls = data["calls"].as_array().ok_or("missing call checkpoints")?;
                    if calls.len() != 4
                        || calls.iter().zip(expected).any(|(entry, phase)| {
                            entry["phase"].as_str() != Some(phase)
                                || ["before_snapshot", "after_snapshot"]
                                    .into_iter()
                                    .any(|key| {
                                        entry[key]
                                            .as_u64()
                                            .and_then(|n| u32::try_from(n).ok())
                                            .is_none()
                                    })
                                || entry["returned"].as_array().is_none_or(|values| {
                                    values.len() != 7 || values.iter().any(|v| v.as_u64().is_none())
                                })
                        })
                    {
                        return Err("invalid call checkpoint receipt".to_owned());
                    }
                }
            }
            if mode == 2 && data["native_checks"] != true {
                return Err("native fixtures missing".to_owned());
            }
        }
        Ok(())
    }

    fn record_handle_stage(
        phase: &'static str,
        phases: &mut Vec<(&'static str, handle_snapshot::Handles)>,
        observations: &mut Vec<(&'static str, u32, u32)>,
    ) {
        let before = current_process_handle_count();
        phases.push((
            phase,
            handle_snapshot::capture().expect("checked self stage snapshot"),
        ));
        observations.push((phase, before, current_process_handle_count()));
    }

    fn assert_handle_stage_failure(
        launcher: &WindowsBackgroundLauncher,
        program: &str,
        fault: Option<WindowsLaunchFault>,
        expected: &str,
    ) {
        let result = create_windows_job_with_hooks(
            program,
            &launcher.args,
            &launcher.cwd,
            &launcher.env,
            WindowsLaunchTestHooks {
                fault,
                ..Default::default()
            },
        );
        assert_eq!(
            result.err().as_deref(),
            Some(expected),
            "must reach the intended stage"
        );
    }

    #[test]
    fn direct_lifecycle_receipt_preserves_growth_as_evidence() {
        let directory = tempfile::tempdir().expect("fixture directory");
        let path = directory.path().join("receipt.json");
        let mut data = serde_json::json!({
            "child_pid":std::process::id().wrapping_add(1).max(1),
            "direct_cycles":2,"snapshots":0,"native_failure_check":true,
            "phases":handle_lifecycle_control::PHASES,"counts":[73,74,76,1000],
        });
        let write = |data: &serde_json::Value| {
            std::fs::write(&path, serde_json::to_vec(data).expect("fixture JSON")).expect("write");
        };
        write(&data);
        assert!(validate_handle_receipt(&path, 7).is_ok());
        assert!(validate_handle_receipt(&path, 0).is_err());
        for (field, value) in [
            ("direct_cycles", serde_json::json!(1)),
            ("snapshots", serde_json::json!(4)),
            ("native_failure_check", serde_json::json!(false)),
            ("phases", serde_json::json!(vec!["direct_baseline"; 4])),
            ("counts", serde_json::json!([73, 74, 76])),
            ("counts", serde_json::json!([73, 74, 76, u64::MAX])),
            ("counts", serde_json::json!([73, 74, 76, "unknown"])),
        ] {
            let original = data[field].clone();
            data[field] = value;
            write(&data);
            assert!(validate_handle_receipt(&path, 7).is_err(), "{field}");
            data[field] = original;
        }
    }

    #[test]
    fn call_receipt_requires_complete_matching_checkpoints() {
        let directory = tempfile::tempdir().expect("fixture directory");
        let path = directory.path().join("receipt.json");
        let phases = [
            "before_pipe",
            "pipe_created",
            "before_cleanup",
            "after_cleanup",
        ];
        let mut data = serde_json::json!({"child_pid":std::process::id().wrapping_add(1).max(1),
            "snapshots":4,"changes":[],"returned_order":RETURNED_ORDER,
            "phase_sizes":phases.map(|phase| (phase,73)),
            "calls":phases.map(|phase| serde_json::json!({"phase":phase,
                "before_snapshot":73,"after_snapshot":73,"returned":vec![0u64;7]}))});
        std::fs::write(&path, serde_json::to_vec(&data).expect("fixture JSON")).expect("write");
        assert!(validate_handle_receipt(&path, 5).is_ok());
        assert!(validate_handle_receipt(&path, 6).is_err());
        data["calls"][2]["returned"] = serde_json::json!(vec![0u64; 6]);
        std::fs::write(&path, serde_json::to_vec(&data).expect("fixture JSON")).expect("write");
        assert!(validate_handle_receipt(&path, 5).is_err());
        let mut observer = LaunchObservation::new(5);
        // Wrong first phase rejects before any native capture; failure stays contained.
        observer.record("pipe_created", [std::ptr::null_mut(); 7]);
        assert_eq!(
            observer.finish().err().as_deref(),
            Some("call checkpoint order/cap")
        );
        assert!(LaunchObservation::new(6).finish().is_err());
    }

    #[test]
    fn handle_stage_receipt_requires_exact_phase_order() {
        let directory = tempfile::tempdir().expect("fixture directory");
        let path = directory.path().join("receipt.json");
        let pid = std::process::id().wrapping_add(1).max(1);
        let mut data = serde_json::json!({"child_pid":pid,"snapshots":4,"changes":[],
            "phase_sizes":[["control_baseline",73],["first_pipe_cleanup",74],
                ["attribute_list_cleanup",74],["suspended_process_cleanup",76]]});
        std::fs::write(&path, serde_json::to_vec(&data).expect("fixture JSON"))
            .expect("fixture write");
        assert!(validate_handle_receipt(&path, 3).is_ok());
        assert!(
            validate_handle_receipt(&path, 4).is_err(),
            "another worker's phases are not valid evidence"
        );
        data["phase_sizes"][2][0] = serde_json::json!("control_baseline");
        std::fs::write(&path, serde_json::to_vec(&data).expect("fixture JSON"))
            .expect("fixture write");
        assert!(validate_handle_receipt(&path, 3).is_err());
    }

    #[test]
    fn known_resource_receipts_require_actual_complete_summary_and_negatives() {
        let directory = tempfile::tempdir().expect("receipt directory");
        let path = directory.path().join("receipt.json");
        let mut data = serde_json::json!({"child_pid":1,"attempts":40,"before":73,"after":76,
            "known_resources":resource_acceptance::all40_receipt()});
        std::fs::write(&path, data.to_string()).expect("receipt");
        assert!(validate_handle_receipt(&path, 0).is_ok());
        for field in [
            "attempts",
            "resources",
            "closes",
            "deletes",
            "terminations",
            "exits",
            "generation_sum",
        ] {
            let original = data["known_resources"][field].take();
            std::fs::write(&path, data.to_string()).expect("incomplete receipt");
            assert!(validate_handle_receipt(&path, 0).is_err(), "{field}");
            data["known_resources"][field] = original;
        }
        data = serde_json::json!({"child_pid":1,"negative_controls":resource_acceptance::negative_receipt()});
        std::fs::write(&path, data.to_string()).expect("negative receipt");
        assert!(validate_handle_receipt(&path, 8).is_ok());
        data["negative_controls"]["recovered"] = 3.into();
        std::fs::write(&path, data.to_string()).expect("incomplete recovery");
        assert!(validate_handle_receipt(&path, 8).is_err());
    }

    #[test]
    fn handle_receipt_failures_are_results_not_early_panics() {
        let directory = tempfile::tempdir().expect("fixture directory");
        let path = directory.path().join("receipt.json");
        assert!(validate_handle_receipt(&path, 0).is_err());
        std::fs::write(&path, b"bad JSON").expect("fixture write");
        assert!(validate_handle_receipt(&path, 0).is_err());
        std::fs::write(&path, vec![b'x'; 1025]).expect("fixture write");
        assert!(validate_handle_receipt(&path, 0).is_err());
        std::fs::write(
            &path,
            br#"{"child_pid":1,"attempts":40,"before":73,"after":76}"#,
        )
        .expect("fixture write");
        assert!(validate_handle_receipt(&path, 0).is_err());
    }

    fn windows_process_exists(pid: u32) -> bool {
        std::process::Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!(
                    "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
                ),
            ])
            .status()
            .is_ok_and(|status| status.success())
    }

    fn current_process_handle_count() -> u32 {
        let mut count = 0;
        let ok = unsafe {
            windows_sys::Win32::System::Threading::GetProcessHandleCount(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut count,
            )
        };
        assert_ne!(ok, 0, "GetProcessHandleCount must succeed");
        count
    }
}
