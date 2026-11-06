//! Real fixed-profile Cargo validation, including workloads beyond the old bounds.

use std::{
    fs::{self, File},
    path::Path,
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

fn validate(root: &Path, label: &str) -> (std::process::ExitStatus, serde_json::Value) {
    let stdout = root.join(format!("{label}.json"));
    let stderr = root.join(format!("{label}.stderr"));
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_talos"))
            .args(["validate", "run", "--profile", "workspace", "--json"])
            .current_dir(root)
            // Do not contend for the outer cargo test's target lock. This changes
            // only this child environment, never process-global test state.
            .env("CARGO_TARGET_DIR", root.join("target"))
            .env("CARGO_NET_OFFLINE", "true")
            .stdout(Stdio::from(File::create(&stdout).expect("stdout")))
            .stderr(Stdio::from(File::create(&stderr).expect("stderr")))
            .spawn()
            .expect("CLI"),
    );
    let deadline = Instant::now() + Duration::from_secs(180);
    let status = loop {
        if let Some(status) = child.0.try_wait().expect("status") {
            break status;
        }
        assert!(Instant::now() < deadline, "Cargo fixture did not complete");
        std::thread::sleep(Duration::from_millis(20));
    };
    let text = fs::read_to_string(stdout).expect("evidence");
    let evidence = serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!(
            "invalid evidence: {error}; stdout={text}; stderr={}",
            fs::read_to_string(stderr).expect("stderr")
        )
    });
    (status, evidence)
}

#[test]
fn real_workspace_build_and_tests_exceed_old_limits_and_preserve_failure_evidence() {
    let root = tempfile::tempdir().expect("workspace");
    fs::create_dir(root.path().join("src")).expect("src");
    fs::write(root.path().join("Cargo.toml"), "[package]\nname = \"validation-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n").expect("manifest");
    fs::write(
        root.path().join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"validation-fixture\"\nversion = \"0.1.0\"\n",
    )
    .expect("lock");
    fs::write(
        root.path().join("rust-toolchain.toml"),
        include_str!("../../../rust-toolchain.toml"),
    )
    .expect("pinned toolchain");
    fs::write(
        root.path().join("build.rs"),
        r#"fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    std::thread::sleep(std::time::Duration::from_secs(31));
    println!("cargo:warning={}", "x".repeat(24 * 1024));
}
"#,
    )
    .expect("real build script");
    let mut source = String::new();
    for index in 0..400 {
        source.push_str(&format!("#[test]\nfn fixture_{index}() {{\n    assert!(!std::path::Path::new(\"fail\").exists());\n}}\n"));
    }
    fs::write(root.path().join("src/lib.rs"), source).expect("tests");
    let lock = fs::read(root.path().join("Cargo.lock")).expect("original lock");
    let (_, evidence) = validate(root.path(), "passing");
    for id in ["fmt", "check", "test"] {
        let record = evidence["records"]
            .as_array()
            .expect("records")
            .iter()
            .find(|record| record["id"] == id)
            .expect("Cargo record");
        assert_eq!(record["status"], "passed", "{record}");
        assert_eq!(record["exit_status"], 0);
    }
    // Governance applicability is reported independently for this Cargo-only
    // project. Only real host-check results establish the workload regression.
    fs::write(root.path().join("fail"), "").expect("failure trigger");
    let (status, evidence) = validate(root.path(), "failing");
    assert!(!status.success());
    let record = evidence["records"]
        .as_array()
        .expect("records")
        .iter()
        .find(|record| record["id"] == "test")
        .expect("test record");
    assert_eq!(record["status"], "failed", "{record}");
    assert_eq!(record["exit_status"], 101);
    assert_eq!(
        fs::read(root.path().join("Cargo.lock")).expect("lock"),
        lock
    );
}
