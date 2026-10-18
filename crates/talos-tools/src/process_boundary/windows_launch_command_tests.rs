use super::*;

#[tokio::test]
async fn no_input_child_observes_eof() {
    let launched = Box::new(WindowsBackgroundLauncher::new(
        "powershell.exe".into(),
        vec![
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            "[Console]::Write([Console]::OpenStandardInput().ReadByte())".into(),
        ],
        std::env::current_dir().expect("current directory"),
        std::collections::BTreeMap::new(),
    ))
    .launch()
    .await
    .expect("launch");
    let mut events = launched.events;
    let control = launched.control;
    let result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let mut output = Vec::new();
        while let Some(event) = events.recv().await {
            match event {
                BackgroundProcessEvent::Output(chunk) => output.extend(chunk.bytes),
                BackgroundProcessEvent::Exited(exit) => return Ok((exit, output)),
                BackgroundProcessEvent::SupervisionFailed(error) => return Err(error),
            }
        }
        Err("missing terminal event".into())
    })
    .await;
    control.terminate().await.expect("cleanup before assertion");
    let (exit, output) = result.expect("bounded EOF read").expect("terminal event");
    assert!(exit.success);
    assert_eq!(output, b"-1");
}

#[test]
fn no_input_launch_owns_inheritable_nul_with_immediate_eof() {
    use std::io::Read;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{GetHandleInformation, HANDLE_FLAG_INHERIT};
    let mut command = WindowsLaunchCommand::new(
        std::path::Path::new(r"C:\bound\git.exe"),
        std::ffi::OsStr::new("git.exe"),
        &[],
        std::path::Path::new(r"C:\work"),
        vec![0, 0],
        false,
    )
    .expect("no-input command");
    let WindowsInput::Null(input) = &mut command.input else {
        panic!("explicit EOF handle required");
    };
    let mut flags = 0;
    // SAFETY: borrowed live handle and valid writable flags output.
    assert_ne!(
        unsafe { GetHandleInformation(input.as_raw_handle() as _, &mut flags) },
        0
    );
    assert_ne!(flags & HANDLE_FLAG_INHERIT, 0);
    assert_eq!(input.read(&mut [0; 1]).expect("NUL read"), 0);
}

#[tokio::test]
async fn job_cleanup_confirms_live_process_exit_before_returning() {
    let launched = create_windows_job(
        "powershell.exe",
        &[
            "-NoLogo".into(),
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            "Start-Sleep -Seconds 60".into(),
        ],
        &std::env::current_dir().expect("current directory"),
        &std::collections::BTreeMap::new(),
    )
    .expect("launch owned process");
    // SAFETY: native launch transfers its sole process-handle ownership here.
    // File is used only as a CloseHandle owner, never for process I/O.
    use std::os::windows::io::FromRawHandle;
    let _process = unsafe { std::fs::File::from_raw_handle(launched.process as _) };
    let control = WindowsJobControl { job: launched.job };
    assert!(control.active_processes().expect("live Job query") > 0);
    control.terminate().await.expect("confirmed process exit");
    assert_eq!(control.active_processes().expect("empty Job query"), 0);
}

#[tokio::test]
async fn job_cleanup_confirms_empty_job_and_rejects_invalid_handle() {
    use windows_sys::Win32::System::JobObjects::CreateJobObjectW;
    // SAFETY: default security attributes and no name; ownership transfers below.
    let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    assert!(!job.is_null(), "{}", last_windows_error("CreateJobObjectW"));
    let control = WindowsJobControl { job };
    assert_eq!(control.active_processes().expect("query empty Job"), 0);
    control
        .terminate()
        .await
        .expect("confirmed empty Job cleanup");

    let invalid = WindowsJobControl {
        job: std::ptr::null_mut(),
    };
    assert!(invalid.active_processes().is_err());
    assert!(invalid.terminate().await.is_err());
}

#[test]
fn finite_input_pipe_delivers_bytes_then_eof_and_inherits_only_reader() {
    use std::io::{Read, Write};
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{GetHandleInformation, HANDLE_FLAG_INHERIT};
    let WindowsInputPipe {
        mut child,
        mut parent,
    } = WindowsInputPipe::create().expect("pipe");
    let mut child_flags = 0;
    let mut parent_flags = 0;
    // SAFETY: borrowed live File handles, valid writable flag outputs.
    unsafe {
        assert_ne!(
            GetHandleInformation(child.as_raw_handle() as _, &mut child_flags),
            0
        );
        assert_ne!(
            GetHandleInformation(parent.as_raw_handle() as _, &mut parent_flags),
            0
        );
    }
    assert_ne!(child_flags & HANDLE_FLAG_INHERIT, 0);
    assert_eq!(parent_flags & HANDLE_FLAG_INHERIT, 0);
    parent.write_all(b"finite\0input").expect("write");
    drop(parent);
    let mut received = Vec::new();
    child
        .read_to_end(&mut received)
        .expect("EOF after parent closes");
    assert_eq!(received, b"finite\0input");
}

#[test]
fn native_launch_inputs_preserve_raw_arguments_and_frozen_environment() {
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::OsStringExt;
    let environment =
        windows_environment::snapshot([(OsString::from("FROZEN"), OsString::from_wide(&[0xd800]))])
            .expect("snapshot");
    let command = WindowsLaunchCommand::new(
        std::path::Path::new(r"C:\bound\git.exe"),
        OsStr::new(r"C:\bound\git.exe"),
        &[OsString::from_wide(&[0xdc00])],
        std::path::Path::new(r"C:\work"),
        environment.clone(),
        false,
    )
    .expect("raw command");
    assert_eq!(command.environment, environment);
    assert!(command.command.contains(&0xdc00));
    assert!(
        WindowsLaunchCommand::new(
            std::path::Path::new(r"C:\bound\git.exe"),
            OsStr::new("bad\0program"),
            &[],
            std::path::Path::new(r"C:\work"),
            vec![0, 0],
            false,
        )
        .is_err()
    );
}
