use std::{fs, process::Command};

#[test]
fn logging_handler_emits_hook_lines_in_print_mode() {
    let temp = tempfile::TempDir::new().expect("create isolated HOME");
    let home = temp.path().join("home");
    let talos_dir = home.join(".talos");
    fs::create_dir_all(&talos_dir).expect("create isolated ~/.talos");
    fs::write(talos_dir.join("config.toml"), "").expect("write isolated config");

    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--print", "--mock", "echo hi"])
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("XDG_CONFIG_HOME", "")
        .env("RUST_LOG", "debug")
        .output()
        .expect("run talos binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LoggingHandler"), "stderr: {stderr}");
    assert!(stderr.contains("TurnStart"), "stderr: {stderr}");
    assert!(stderr.contains("TurnComplete"), "stderr: {stderr}");
}
