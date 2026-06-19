use std::fs;
use std::process::Command;

#[test]
fn request_preview_contains_workspace_skill_metadata() {
    let workspace = tempfile::tempdir().expect("create temporary workspace");
    let skill_dir = workspace.path().join(".talos/skills/i033-e2e");
    fs::create_dir_all(&skill_dir).expect("create skill directory");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: i033-e2e
description: Runtime skill activation end-to-end fixture
triggers:
  - i033-e2e
---
# I033 E2E

This temporary fixture verifies runtime discovery.
"#,
    )
    .expect("write skill fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args([
            "--mock",
            "--print",
            "--workspace",
            workspace.path().to_str().expect("UTF-8 workspace path"),
            "--",
            "/mock-request verify runtime skill discovery",
        ])
        .env("HOME", workspace.path())
        .output()
        .expect("run talos binary");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Request preview (no API call made)"));
    assert!(stdout.contains("i033-e2e"), "stdout: {stdout}");
    assert!(
        stdout.contains("Runtime skill activation end-to-end fixture"),
        "stdout: {stdout}"
    );
}
