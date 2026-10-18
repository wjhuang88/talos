use super::*;

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
