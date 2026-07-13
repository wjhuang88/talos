#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn talos_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_talos"))
}

fn run_diagnostics_json(cwd: &std::path::Path) -> String {
    let output = Command::new(talos_bin())
        .args(["diagnostics", "status", "--json"])
        .current_dir(cwd)
        .output()
        .expect("talos binary should start");
    assert!(
        output.status.success(),
        "talos diagnostics status --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

fn run_diagnostics_text(cwd: &std::path::Path) -> String {
    let output = Command::new(talos_bin())
        .args(["diagnostics", "status"])
        .current_dir(cwd)
        .output()
        .expect("talos binary should start");
    assert!(
        output.status.success(),
        "talos diagnostics status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn json_output_parses_as_serde_value() {
    let dir = tempdir().unwrap();
    let json_str = run_diagnostics_json(dir.path());
    let value: serde_json::Value =
        serde_json::from_str(&json_str).expect("JSON output must parse as serde_json::Value");
    assert!(value.is_object(), "top-level should be a JSON object");
    assert!(value.get("talos_version").is_some());
    assert!(value.get("rust_toolchain").is_some());
    assert!(value.get("session_formats").is_some());
    assert!(value.get("workspace_root").is_some());
    assert!(value.get("is_git_workspace").is_some());
    assert!(value.get("workspace_trusted").is_some());
    assert!(value.get("trusted_workspace_count").is_some());
    assert!(value.get("config_exists").is_some());
    assert!(value.get("active_iterations").is_some());
    assert!(value.get("residual_gates").is_some());
}

#[test]
fn json_output_contains_no_secrets() {
    let dir = tempdir().unwrap();
    let json_str = run_diagnostics_json(dir.path());
    let lower = json_str.to_lowercase();
    assert!(!lower.contains("api_key"), "JSON must not contain api_key");
    assert!(
        !lower.contains("sk-ant"),
        "JSON must not contain API key prefix"
    );
    assert!(!lower.contains("secret"), "JSON must not contain 'secret'");
    assert!(
        !lower.contains("password"),
        "JSON must not contain 'password'"
    );
    assert!(!lower.contains("token"), "JSON must not contain 'token'");
}

#[test]
fn json_output_has_no_stale_i085_paused() {
    let dir = tempdir().unwrap();
    let json_str = run_diagnostics_json(dir.path());
    assert!(
        !json_str.contains("I085") || !json_str.contains("Paused"),
        "JSON output must not contain stale I085 Paused claim"
    );
}

#[test]
fn json_output_with_clean_iteration_source() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs").join("iterations");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::write(
        docs_dir.join("README.md"),
        "# Iterations\n\n## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I120 | Dynamic Diagnostics | **Active** (2026-07-13) | no |\n",
    )
    .unwrap();

    let json_str = run_diagnostics_json(dir.path());
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let iterations = value["active_iterations"].as_array().unwrap();
    assert!(
        iterations
            .iter()
            .any(|i| { i.as_str().map(|s| s.contains("I120")).unwrap_or(false) }),
        "should find I120 in active_iterations: {iterations:?}"
    );
}

#[test]
fn json_output_when_iteration_index_missing() {
    let dir = tempdir().unwrap();
    let json_str = run_diagnostics_json(dir.path());
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let iterations = value["active_iterations"].as_array().unwrap();
    assert!(
        iterations.iter().any(|i| {
            i.as_str()
                .map(|s| s.contains("unavailable"))
                .unwrap_or(false)
        }),
        "missing index should produce 'unavailable' diagnostic: {iterations:?}"
    );
}

#[test]
fn text_output_works() {
    let dir = tempdir().unwrap();
    let text = run_diagnostics_text(dir.path());
    assert!(
        text.contains("Talos Diagnostics Status"),
        "text output should have header"
    );
    assert!(
        text.contains("Active Iterations"),
        "text output should have iterations section"
    );
    assert!(
        text.contains("Residual Gates"),
        "text output should have gates section"
    );
}

#[test]
fn json_output_workspace_root_is_valid_json_string() {
    let dir = tempdir().unwrap();
    let json_str = run_diagnostics_json(dir.path());
    let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(
        value["workspace_root"].is_string(),
        "workspace_root must be a JSON string, not raw path"
    );
}
