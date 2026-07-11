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

#[test]
fn rpc_mode_agent_run_uses_session_runtime_and_returns_final_text() {
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
            .write_all(
                b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"agent.run\",\"params\":{\"prompt\":\"hello\",\"stream\":true}}\n",
            )
            .expect("write agent.run request");
    }

    let output = child.wait_with_output().expect("wait output");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let response = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|value| value["id"] == 2)
        .unwrap_or_else(|| panic!("agent.run response missing from: {stdout}"));
    assert_eq!(response["id"], 2);
    assert!(
        response["result"]["result"]
            .as_str()
            .is_some_and(|text| text.contains("mock LLM")),
        "response: {response}"
    );
}
