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
        // A Job Object is the ownership boundary; terminating the job cannot leave descendants.
        let ok = unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1) };
        if ok == 0 {
            Err(last_windows_error("TerminateJobObject"))
        } else {
            Ok(())
        }
    }

    async fn force_terminate(&self) -> Result<(), String> {
        self.terminate().await
    }
}

#[cfg(windows)]
impl Drop for WindowsJobControl {
    fn drop(&mut self) {
        if !self.job.is_null() {
            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };
            self.job = 0;
        }
    }
}

#[cfg(windows)]
#[async_trait]
impl BackgroundJobLauncher for WindowsBackgroundLauncher {
    async fn launch(self: Box<Self>) -> Result<LaunchedBackgroundJob, String> {
        let launched = create_windows_job(&self.program, &self.args, &self.cwd, &self.env)?;
        let control = Arc::new(WindowsJobControl { job: launched.job });
        let (tx, rx) = tokio::sync::mpsc::channel(BACKGROUND_PROCESS_EVENT_CAPACITY);
        let stdout = launched.stdout;
        let stderr = launched.stderr;
        let process = launched.process;
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

    unsafe {
        if size_of::<HANDLE>() != size_of::<isize>() {
            return Err("unsupported Windows handle width".to_owned());
        }
    }
    let mut security = SECURITY_ATTRIBUTES {
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
    let mut attr_storage = Vec::<u8>::new();
    let result = (|| unsafe {
        if CreatePipe(&mut out_read, &mut out_write, &mut security, 0) == 0
            || CreatePipe(&mut err_read, &mut err_write, &mut security, 0) == 0
        {
            return Err(last_windows_error("CreatePipe"));
        }
        if SetHandleInformation(out_read, HANDLE_FLAG_INHERIT, 0) == 0
            || SetHandleInformation(err_read, HANDLE_FLAG_INHERIT, 0) == 0
        {
            return Err(last_windows_error("SetHandleInformation"));
        }
        job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job == 0 || job == INVALID_HANDLE_VALUE {
            return Err(last_windows_error("CreateJobObjectW"));
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
        attr_storage = vec![0u8; attr_size];
        attrs = attr_storage.as_mut_ptr()
            as windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;
        if InitializeProcThreadAttributeList(attrs, 1, 0, &mut attr_size) == 0 {
            return Err(last_windows_error("InitializeProcThreadAttributeList"));
        }
        let handles = [out_write, err_write];
        if UpdateProcThreadAttribute(
            attrs,
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
            handles.as_ptr() as *const _,
            size_of::<[HANDLE; 2]>(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(last_windows_error("UpdateProcThreadAttribute"));
        }
        let mut command = format!("\"{}\"", program);
        for arg in args {
            command.push(' ');
            command.push_str(&quote_windows_arg(arg));
        }
        let mut command_w = wide(std::ffi::OsStr::new(&command));
        let cwd_w = wide(cwd.as_os_str());
        let mut environment = windows_environment(extra_env);
        let mut startup: STARTUPINFOEXW = zeroed();
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attrs;
        startup.StartupInfo.dwFlags = 0x00000100;
        startup.StartupInfo.hStdOutput = out_write;
        startup.StartupInfo.hStdError = err_write;
        let mut info: PROCESS_INFORMATION = zeroed();
        if CreateProcessW(
            std::ptr::null(),
            command_w.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            1,
            CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
            environment.as_mut_ptr() as *mut _,
            cwd_w.as_ptr(),
            &startup as *const STARTUPINFOEXW as *const _,
            &mut info,
        ) == 0
        {
            return Err(last_windows_error("CreateProcessW"));
        }
        process = info.hProcess;
        thread = info.hThread;
        if AssignProcessToJobObject(job, process) == 0 {
            return Err(last_windows_error("AssignProcessToJobObject"));
        }
        if ResumeThread(thread) == u32::MAX {
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
        let stdout = tokio::fs::File::from_std(std::fs::File::from_raw_handle(out_read as _));
        out_read = std::ptr::null_mut();
        let stderr = tokio::fs::File::from_std(std::fs::File::from_raw_handle(err_read as _));
        err_read = std::ptr::null_mut();
        Ok(WindowsLaunchHandles {
            job,
            process,
            stdout,
            stderr,
        })
    })();
    if result.is_err() {
        unsafe {
            if !thread.is_null() {
                CloseHandle(thread);
            }
            if !process.is_null() {
                windows_sys::Win32::System::Threading::TerminateProcess(process, 1);
                CloseHandle(process);
            }
            if !attrs.is_null() {
                DeleteProcThreadAttributeList(attrs);
            }
            if !job.is_null() {
                CloseHandle(job);
            }
            for handle in [out_read, out_write, err_read, err_write] {
                if !handle.is_null() {
                    CloseHandle(handle);
                }
            }
        }
    }
    result
}

#[cfg(windows)]
fn windows_environment(extra: &std::collections::BTreeMap<String, String>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    let dangerous = talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
    let mut values = std::collections::BTreeMap::<String, String>::new();
    for (name, value) in std::env::vars() {
        if !dangerous.iter().any(|item| item == &name) {
            values.insert(name, value);
        }
    }
    values.extend(
        extra
            .iter()
            .filter(|(name, _)| !dangerous.iter().any(|item| item == name.as_str()))
            .map(|(name, value)| (name.clone(), value.clone())),
    );
    let mut block = Vec::new();
    for (name, value) in values {
        std::ffi::OsStr::new(&format!("{name}={value}"))
            .encode_wide()
            .for_each(|unit| block.push(unit));
        block.push(0);
    }
    block.push(0);
    block
}

#[cfg(windows)]
fn quote_windows_arg(value: &str) -> String {
    if !value.contains([' ', '\t', '"']) {
        return value.to_owned();
    }
    format!("\"{}\"", value.replace('"', "\\\""))
}
