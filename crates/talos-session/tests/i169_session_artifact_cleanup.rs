use std::fs;
use talos_session::{SessionManager, remove_session_artifacts_for_transcript};
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
        remove_session_artifacts_for_transcript(&transcript).expect("remove complete artifact set")
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
