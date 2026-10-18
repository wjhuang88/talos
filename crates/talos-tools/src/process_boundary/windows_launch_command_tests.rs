use super::*;

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
        )
        .is_err()
    );
}
