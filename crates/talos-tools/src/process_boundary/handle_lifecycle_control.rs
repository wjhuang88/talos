//! I302 test-only direct Win32 control; ADR-068/091 ownership and evidence limits apply.
//! Counts qualify a lifecycle interval, not resource identity, process exit or absence of leaks.

use super::{WindowsBackgroundLauncher, resolve_windows_program, windows_environment};
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, SetHandleInformation,
};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::JobObjects::{
    CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetCurrentProcess, GetProcessHandleCount,
    InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROCESS_INFORMATION, STARTUPINFOEXW, TerminateProcess,
    UpdateProcThreadAttribute,
};

pub(super) const PHASES: [&str; 4] = [
    "direct_baseline",
    "direct_pipe_cleanup",
    "direct_first_process_cleanup",
    "direct_second_process_cleanup",
];

// Only successful API outputs enter these slots. No observations are ever adopted/closed.
#[derive(Default)]
struct Owners {
    pipes: [HANDLE; 4],
    job: HANDLE,
    process: HANDLE,
    thread: HANDLE,
    storage: Vec<usize>,
    attributes_initialized: bool,
    // Stable separate allocation: native attributes borrow the elements through Delete.
    // Reborrowing Owners must not reborrow the native-borrowed element storage.
    handle_list: Vec<HANDLE>,
}

impl Owners {
    fn attributes(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }

    fn pipe(&mut self, offset: usize) -> Result<(), &'static str> {
        let mut read = std::ptr::null_mut();
        let mut write = std::ptr::null_mut();
        let security = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        // SAFETY: ADR-068/091; live initialized outputs/security. Failure outputs are
        // indeterminate and must never become owned, even if they are non-null.
        if unsafe { CreatePipe(&mut read, &mut write, &security, 0) } == 0 {
            return Err("direct CreatePipe failed");
        }
        self.pipes[offset] = read;
        self.pipes[offset + 1] = write;
        Ok(())
    }

    fn cleanup(&mut self) -> u16 {
        // SAFETY: ADR-068/091. Each slot holds only an owned successful-return handle.
        // Consume before attempting release, and attempt every remaining operation.
        // No retry after uncertain release. Nothing below allocates, panics or observes.
        unsafe {
            let mut failures = 0;
            let thread = std::mem::replace(&mut self.thread, std::ptr::null_mut());
            if !thread.is_null() && CloseHandle(thread) == 0 {
                failures |= 1;
            }
            let process = std::mem::replace(&mut self.process, std::ptr::null_mut());
            if !process.is_null() {
                if TerminateProcess(process, 1) == 0 {
                    failures |= 1 << 1;
                }
                if CloseHandle(process) == 0 {
                    failures |= 1 << 2;
                }
            }
            if std::mem::replace(&mut self.attributes_initialized, false) {
                DeleteProcThreadAttributeList(self.attributes());
            }
            let job = std::mem::replace(&mut self.job, std::ptr::null_mut());
            if !job.is_null() && CloseHandle(job) == 0 {
                failures |= 1 << 3;
            }
            for (index, slot) in self.pipes.iter_mut().enumerate() {
                let handle = std::mem::replace(slot, std::ptr::null_mut());
                if !handle.is_null() && CloseHandle(handle) == 0 {
                    failures |= 1 << (index + 4);
                }
            }
            failures
        }
    }
}

impl Drop for Owners {
    fn drop(&mut self) {
        // Fallback only for still-owned slots during unwind. Explicit cleanup reports
        // errors; a panic cannot produce a successful control receipt.
        let _ = self.cleanup();
    }
}

fn count() -> Result<u32, &'static str> {
    let mut count = 0;
    // SAFETY: ADR-091; borrowed current-process pseudohandle and live scalar output.
    if unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) } == 0 {
        Err("direct handle count failed")
    } else {
        Ok(count)
    }
}

pub(super) fn run(launcher: &WindowsBackgroundLauncher) -> Result<[u32; 4], String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_checked(launcher)))
        .unwrap_or_else(|_| Err("direct lifecycle panicked; evidence incomplete".to_owned()))
}

fn run_checked(launcher: &WindowsBackgroundLauncher) -> Result<[u32; 4], String> {
    // Prepare the same fixture inputs before acquiring resources or taking the baseline.
    let wide = |value: &std::ffi::OsStr| value.encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let mut program = wide(resolve_windows_program(&launcher.program)?.as_os_str());
    let arguments = std::iter::once(launcher.program.as_str())
        .chain(launcher.args.iter().map(String::as_str))
        .map(|argument| {
            std::ffi::OsStr::new(argument)
                .encode_wide()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let command = super::windows_command_line::encode(&arguments)?;
    // CreateProcessW can modify its command buffer; prepare both independent copies now.
    let commands = [command.clone(), command];
    let cwd = wide(launcher.cwd.as_os_str());
    let mut environment = windows_environment(&launcher.env)?;
    let mut counts = [0; 4];
    counts[0] = count()?;
    {
        let mut owners = Owners::default();
        let result = owners.pipe(0);
        let cleanup = owners.cleanup();
        if cleanup != 0 {
            return Err(format!(
                "direct pipe setup={result:?}; cleanup failure mask={cleanup}"
            ));
        }
        result?;
    }
    counts[1] = count()?;
    for (index, mut command) in commands.into_iter().enumerate() {
        let mut owners = Owners::default();
        owners.handle_list.resize(2, std::ptr::null_mut());
        // SAFETY: ADR-068/091; same binding/flags as the launcher, owned buffers and
        // success-only handle transfer. Outer Job owns this worker and its descendants.
        // The control child stays suspended: no assignment or resume is attempted.
        let result = (|| unsafe {
            owners.pipe(0)?;
            owners.pipe(2)?;
            if SetHandleInformation(owners.pipes[0], HANDLE_FLAG_INHERIT, 0) == 0
                || SetHandleInformation(owners.pipes[2], HANDLE_FLAG_INHERIT, 0) == 0
            {
                return Err("direct SetHandleInformation failed");
            }
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() || job == INVALID_HANDLE_VALUE {
                return Err("direct CreateJobObjectW failed");
            }
            owners.job = job;
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) == 0
            {
                return Err("direct SetInformationJobObject failed");
            }
            let mut size = 0usize;
            let _ = InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut size);
            if size == 0 || size > 65_536 {
                return Err("direct attribute-list size invalid");
            }
            owners.storage = vec![0; size.div_ceil(size_of::<usize>())];
            if InitializeProcThreadAttributeList(owners.attributes(), 1, 0, &mut size) == 0 {
                return Err("direct InitializeProcThreadAttributeList failed");
            }
            owners.attributes_initialized = true;
            owners
                .handle_list
                .copy_from_slice(&[owners.pipes[1], owners.pipes[3]]);
            let handle_list = owners.handle_list.as_ptr();
            if UpdateProcThreadAttribute(
                owners.attributes(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handle_list.cast(),
                size_of::<[HANDLE; 2]>(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 0
            {
                return Err("direct UpdateProcThreadAttribute failed");
            }
            let mut startup: STARTUPINFOEXW = zeroed();
            startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
            startup.StartupInfo.dwFlags = 0x00000100;
            startup.StartupInfo.hStdOutput = owners.pipes[1];
            startup.StartupInfo.hStdError = owners.pipes[3];
            startup.lpAttributeList = owners.attributes();
            let mut info: PROCESS_INFORMATION = zeroed();
            if CreateProcessW(
                program.as_mut_ptr(),
                command.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                environment.as_mut_ptr().cast(),
                cwd.as_ptr(),
                (&startup as *const STARTUPINFOEXW).cast(),
                &mut info,
            ) == 0
            {
                return Err("direct CreateProcessW failed");
            }
            owners.process = info.hProcess;
            owners.thread = info.hThread;
            Ok(())
        })();
        let cleanup = owners.cleanup();
        if cleanup != 0 {
            return Err(format!(
                "direct process setup={result:?}; cleanup failure mask={cleanup}"
            ));
        }
        result?;
        drop(owners);
        counts[index + 2] = count()?;
    }
    Ok(counts)
}
