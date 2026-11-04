//! Real CLI cancellation evidence; no live provider or real Cargo build.
#![cfg(unix)]

use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn interrupt_preserves_json_stops_later_checks_and_cleans_descendants() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
    let bin = root.path().join("bin");
    std::fs::create_dir(&bin).expect("bin");
    let cargo = bin.join("cargo");
    std::fs::write(
        &cargo,
        "#!/bin/sh\necho \"$1\" >> calls\ntouch ready\n(sleep 1; touch escaped) &\nwait\n",
    )
    .expect("fixture");
    std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).expect("executable");
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_talos"))
            .args(["validate", "run", "--profile", "workspace", "--json"])
            .current_dir(root.path())
            .env("PATH", std::env::join_paths(paths).expect("path"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("CLI"),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    while !root.path().join("ready").exists() {
        assert!(
            child.0.try_wait().expect("status").is_none(),
            "CLI exited before fixture"
        );
        assert!(Instant::now() < deadline, "fixture did not start");
        std::thread::sleep(Duration::from_millis(10));
    }
    // Test-only host utility sends SIGINT to this retained, unreaped child PID.
    assert!(
        Command::new("/bin/kill")
            .args(["-INT", &child.0.id().to_string()])
            .status()
            .expect("signal utility")
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(8);
    let status = loop {
        if let Some(status) = child.0.try_wait().expect("status") {
            break status;
        }
        assert!(Instant::now() < deadline, "cancellation did not complete");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(!status.success());
    let evidence: serde_json::Value =
        serde_json::from_reader(child.0.stdout.take().expect("stdout")).expect("JSON evidence");
    let records = evidence["records"].as_array().expect("records");
    assert_eq!(records[0]["status"], "failed");
    assert!(
        records[0]["stderr_summary"]
            .as_str()
            .expect("reason")
            .contains("cleanup confirmed")
    );
    assert!(
        records[1..]
            .iter()
            .all(|record| record["status"] == "not_started")
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("calls")).expect("calls"),
        "fmt\n"
    );
    std::thread::sleep(Duration::from_millis(1100));
    assert!(!root.path().join("escaped").exists());
}
