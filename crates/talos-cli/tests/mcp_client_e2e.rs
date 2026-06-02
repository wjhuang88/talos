use std::path::PathBuf;
use std::process::Command;

#[test]
fn mcp_client_e2e_routes_tool_call_through_fixture_server() {
    let fixture_bin = build_fixture_binary();

    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args([
            "--print",
            "--mock",
            "--mcp-server-fixture",
            fixture_bin.to_string_lossy().as_ref(),
            "call fixture echo",
        ])
        .env("RUST_LOG", "info")
        .output()
        .expect("run talos binary with MCP fixture");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("event")
            && stderr.contains("OnToolCallProposed")
            && stderr.contains("turn_id"),
        "stderr missing hook events: {stderr}"
    );

    let proposed_count = stderr.matches("OnToolCallProposed").count();
    assert!(
        proposed_count >= 2,
        "expected at least two tool proposals (provider + MCP adapter), got {proposed_count}. stderr: {stderr}"
    );
}

fn build_fixture_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_src = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("echo_mcp_server.rs");
    let output = std::env::temp_dir().join("talos_echo_mcp_server_e2e_bin");

    let status = Command::new("rustc")
        .arg("--edition=2024")
        .arg("-o")
        .arg(&output)
        .arg(fixture_src)
        .status()
        .expect("spawn rustc for fixture build");
    assert!(status.success(), "fixture rustc compile failed");

    output
}
