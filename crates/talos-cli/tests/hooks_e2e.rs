use std::process::Command;

#[test]
fn logging_handler_emits_hook_lines_in_print_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--print", "--mock", "echo hi"])
        .env("RUST_LOG", "info")
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
