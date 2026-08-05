from pathlib import Path
import re

manager = Path('crates/talos-session/src/manager.rs')
text = manager.read_text()
marker = 'const KNOWN_EXTENSIONS: &[&str] = &["jsonl", "tlog"];\n'
helper = '''

/// Remove the complete artifact set owned by one Session transcript path.
pub fn remove_session_artifacts_for_transcript(
    transcript_path: &Path,
) -> Result<u64, SessionError> {
    let stem = transcript_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| SessionError::ParseError(format!(
            "invalid session transcript path: {}",
            transcript_path.display()
        )))?;
    let sidecar = transcript_path.with_file_name(format!("{stem}.pending.sqlite"));
    let mut removed_bytes = 0_u64;
    for path in [
        transcript_path.to_path_buf(),
        sidecar.clone(),
        PathBuf::from(format!("{}-wal", sidecar.display())),
        PathBuf::from(format!("{}-shm", sidecar.display())),
    ] {
        match fs::metadata(&path) {
            Ok(metadata) => {
                fs::remove_file(&path)?;
                removed_bytes = removed_bytes.saturating_add(metadata.len());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(SessionError::IoError(error)),
        }
    }
    Ok(removed_bytes)
}
'''
if 'pub fn remove_session_artifacts_for_transcript' not in text:
    assert marker in text
    text = text.replace(marker, marker + helper, 1)
old = '''        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
'''
assert old in text
text = text.replace(old, '        remove_session_artifacts_for_transcript(&file_path)?;\n', 1)
old = '''            if candidate.file_path.exists() {
                fs::remove_file(&candidate.file_path)?;
                report.removed += 1;
                report.bytes_removed += candidate.size_bytes;
            }
'''
assert old in text
text = text.replace(old, '''            let removed_bytes = remove_session_artifacts_for_transcript(&candidate.file_path)?;
            if removed_bytes > 0 {
                report.removed += 1;
                report.bytes_removed = report.bytes_removed.saturating_add(removed_bytes);
            }
''', 1)
manager.write_text(text)

session_lib = Path('crates/talos-session/src/lib.rs')
text = session_lib.read_text()
old = '''pub use manager::{
    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,
};'''
new = '''pub use manager::{
    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,
    remove_session_artifacts_for_transcript,
};'''
assert old in text
session_lib.write_text(text.replace(old, new, 1))

handlers = Path('crates/talos-cli/src/session_handlers.rs')
text = handlers.read_text()
pattern = re.compile(r'std::fs::remove_file\(&([A-Za-z_][A-Za-z0-9_]*)\.file_path\)')
text, count = pattern.subn(r'talos_session::remove_session_artifacts_for_transcript(&\1.file_path)', text)
assert count >= 2, count
handlers.write_text(text)

agent = Path('crates/talos-agent/src/lib.rs')
text = agent.read_text()
old = 'self.bash_compression_enabled && observed.call.name == "bash"'
assert old in text
text = text.replace(old, 'self.bash_compression_enabled\n                                && matches!(observed.call.name.as_str(), "bash" | "powershell")', 1)
agent.write_text(text)

Path('crates/talos-session/tests/i169_session_artifact_cleanup.rs').write_text(r'''use std::fs;
use talos_session::{remove_session_artifacts_for_transcript, SessionManager};
use tempfile::tempdir;
use uuid::Uuid;

fn create_set(transcript: &std::path::Path) -> Vec<std::path::PathBuf> {
    let stem = transcript
        .file_stem()
        .expect("fixture transcript should have a file stem")
        .to_string_lossy();
    let sidecar = transcript.with_file_name(format!("{stem}.pending.sqlite"));
    fs::write(transcript, b"transcript").expect("write transcript fixture");
    fs::write(&sidecar, b"sensitive submission and /private/image.png")
        .expect("write pending sidecar fixture");
    let wal = std::path::PathBuf::from(format!("{}-wal", sidecar.display()));
    let shm = std::path::PathBuf::from(format!("{}-shm", sidecar.display()));
    fs::write(&wal, b"wal").expect("write WAL fixture");
    fs::write(&shm, b"shm").expect("write SHM fixture");
    vec![transcript.to_path_buf(), sidecar, wal, shm]
}

#[test]
fn complete_artifact_boundary_removes_all_files() {
    let dir = tempdir().expect("create temporary directory");
    let transcript = dir.path().join(format!("{}.tlog", Uuid::new_v4()));
    let paths = create_set(&transcript);
    assert!(
        remove_session_artifacts_for_transcript(&transcript)
            .expect("remove complete artifact set")
            > 0
    );
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn manager_delete_removes_pending_payload() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let session = manager
        .create_session("test", "/workspace")
        .expect("create session fixture");
    let paths = create_set(&session.file_path);
    manager
        .delete_session(&session.id)
        .expect("delete complete session artifact set");
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn failure_rollback_is_idempotent() {
    let dir = tempdir().expect("create temporary directory");
    let transcript = dir.path().join(format!("{}.tlog", Uuid::new_v4()));
    let paths = create_set(&transcript);
    remove_session_artifacts_for_transcript(&transcript)
        .expect("remove complete artifact set first time");
    assert_eq!(
        remove_session_artifacts_for_transcript(&transcript)
            .expect("repeat complete artifact cleanup"),
        0
    );
    assert!(paths.iter().all(|path| !path.exists()));
}
''')
