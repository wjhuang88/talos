from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"expected exactly one match in {path}, got {text.count(old)}")
    file.write_text(text.replace(old, new, 1))


replace_once(
    "crates/talos-agent/src/lib.rs",
    "const DOOM_LOOP_THRESHOLD: u32 = 3;\n/// Shared admission contract",
    "const DOOM_LOOP_THRESHOLD: u32 = 3;\n\nfn should_compress_shell_output(tool_name: &str) -> bool {\n    matches!(tool_name, \"bash\" | \"powershell\")\n}\n\n/// Shared admission contract",
)
replace_once(
    "crates/talos-agent/src/lib.rs",
    "self.bash_compression_enabled\n                        && matches!(observed.call.name.as_str(), \"bash\" | \"powershell\")",
    "self.bash_compression_enabled\n                        && should_compress_shell_output(&observed.call.name)",
)
agent = Path("crates/talos-agent/src/lib.rs")
text = agent.read_text()
marker = "mod i169_shell_compression_regression"
if marker not in text:
    text += '''\n\n#[cfg(test)]\nmod i169_shell_compression_regression {\n    use super::should_compress_shell_output;\n\n    #[test]\n    fn production_shell_compression_predicate_covers_bash_and_powershell_only() {\n        assert!(should_compress_shell_output("bash"));\n        assert!(should_compress_shell_output("powershell"));\n        assert!(!should_compress_shell_output("read"));\n        assert!(!should_compress_shell_output("fetch_url"));\n    }\n}\n'''
    agent.write_text(text)

replace_once(
    "crates/talos-session/src/error.rs",
    "use thiserror::Error;\nuse uuid::Uuid;",
    "use std::path::PathBuf;\n\nuse thiserror::Error;\nuse uuid::Uuid;",
)
replace_once(
    "crates/talos-session/src/error.rs",
    "    IoError(#[from] std::io::Error),\n\n    /// A line",
    "    IoError(#[from] std::io::Error),\n\n    /// Removing one artifact from a Session-owned artifact set failed.\n    #[error(\"failed to remove session artifact {path}: {source}\")]\n    ArtifactCleanup {\n        /// Exact artifact path whose removal failed.\n        path: PathBuf,\n        /// Underlying filesystem failure.\n        #[source]\n        source: std::io::Error,\n    },\n\n    /// A line",
)
replace_once(
    "crates/talos-session/src/manager.rs",
    "                fs::remove_file(&path)?;\n                removed_bytes = removed_bytes.saturating_add(metadata.len());",
    "                fs::remove_file(&path).map_err(|source| SessionError::ArtifactCleanup {\n                    path: path.clone(),\n                    source,\n                })?;\n                removed_bytes = removed_bytes.saturating_add(metadata.len());",
)
replace_once(
    "crates/talos-session/src/manager.rs",
    "            Err(error) => return Err(SessionError::IoError(error)),",
    "            Err(source) => {\n                return Err(SessionError::ArtifactCleanup {\n                    path: path.clone(),\n                    source,\n                });\n            }",
)

test = Path("crates/talos-session/tests/i169_session_artifact_cleanup.rs")
text = test.read_text()
if "orphan_sidecars_are_removed_without_a_transcript" not in text:
    text += '''\n\n#[test]\nfn orphan_sidecars_are_removed_without_a_transcript() {\n    let dir = tempdir().expect("create temporary directory");\n    let transcript = dir.path().join("orphan.jsonl");\n    let sidecar = transcript.with_file_name("orphan.pending.sqlite");\n    let wal = std::path::PathBuf::from(format!("{}-wal", sidecar.display()));\n    let shm = std::path::PathBuf::from(format!("{}-shm", sidecar.display()));\n    fs::write(&sidecar, b"pending").expect("write orphan pending database");\n    fs::write(&wal, b"wal").expect("write orphan WAL");\n    fs::write(&shm, b"shm").expect("write orphan SHM");\n\n    let removed = remove_session_artifacts_for_transcript(&transcript)\n        .expect("remove orphan Session sidecars");\n\n    assert_eq!(removed, 13);\n    assert!(!transcript.exists());\n    assert!(!sidecar.exists());\n    assert!(!wal.exists());\n    assert!(!shm.exists());\n}\n\n#[test]\nfn cleanup_failure_identifies_the_exact_artifact_path() {\n    let dir = tempdir().expect("create temporary directory");\n    let transcript = dir.path().join("blocked.jsonl");\n    fs::create_dir(&transcript).expect("create non-removable transcript fixture");\n\n    let error = remove_session_artifacts_for_transcript(&transcript)\n        .expect_err("directory fixture must not be reported as successfully removed");\n    let message = error.to_string();\n\n    assert!(message.contains("failed to remove session artifact"));\n    assert!(message.contains(&transcript.display().to_string()));\n}\n'''
    test.write_text(text)
