use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn rpc_mode_system_version_roundtrip() {
    let fixture = fs::read_to_string("tests/fixtures/rpc_hello.json").expect("read fixture");

    let mut child = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args(["--mode", "rpc", "--mock"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn talos rpc mode");

    {
        let stdin = child.stdin.as_mut().expect("stdin handle");
        stdin
            .write_all(fixture.as_bytes())
            .expect("write fixture to stdin");
    }

    let output = child.wait_with_output().expect("wait output");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().expect("rpc response line");
    let response: serde_json::Value = serde_json::from_str(line).expect("parse response json");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["version"], "0.1.0");
    assert_eq!(response["result"]["protocol"], 1);
}
