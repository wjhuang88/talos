use std::fs;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

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

#[test]
fn inline_binary_skill_activation_reaches_request_preview() {
    let workspace = tempfile::tempdir().expect("create temporary workspace");
    let skill_dir = workspace.path().join(".talos/skills/review");
    fs::create_dir_all(&skill_dir).expect("create skill directory");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: review
description: Review code safely
triggers:
  - review
---
# Review Skill

I058_BINARY_SKILL_BODY_REACHED_PROVIDER_CONTEXT
"#,
    )
    .expect("write skill fixture");

    let mut child = Command::new(env!("CARGO_BIN_EXE_talos"))
        .args([
            "--mock",
            "--inline",
            "--workspace",
            workspace.path().to_str().expect("UTF-8 workspace path"),
            "--no-init",
        ])
        .env("HOME", workspace.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn talos binary");

    {
        let stdin = child.stdin.as_mut().expect("child stdin");
        stdin
            .write_all(b"/skills activate review\n/mock-request verify activated skill\n/quit\n")
            .expect("write scripted input");
    }

    let output = child.wait_with_output().expect("run talos binary");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Skill activated: review"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("Request preview (no API call made)"));
    assert!(
        stdout.contains("I058_BINARY_SKILL_BODY_REACHED_PROVIDER_CONTEXT"),
        "stdout: {stdout}"
    );
}
