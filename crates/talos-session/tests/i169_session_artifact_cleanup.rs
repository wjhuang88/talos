use std::fs;
use std::time::Duration;

use rusqlite::Connection;
use talos_session::{
    OrphanSidecarReconciliationPolicy, SessionCleanupPolicy, SessionManager,
    remove_session_artifacts_for_transcript,
};
use tempfile::tempdir;
use uuid::Uuid;

fn artifact_paths(transcript: &std::path::Path) -> Vec<std::path::PathBuf> {
    let stem = transcript
        .file_stem()
        .expect("fixture transcript should have a file stem")
        .to_string_lossy();
    let sidecar = transcript.with_file_name(format!("{stem}.pending.sqlite"));
    vec![
        transcript.to_path_buf(),
        sidecar.clone(),
        std::path::PathBuf::from(format!("{}-wal", sidecar.display())),
        std::path::PathBuf::from(format!("{}-shm", sidecar.display())),
    ]
}

fn create_set(transcript: &std::path::Path) -> Vec<std::path::PathBuf> {
    let paths = artifact_paths(transcript);
    fs::write(&paths[0], b"transcript").expect("write transcript fixture");
    fs::write(&paths[1], b"sensitive submission and /private/image.png")
        .expect("write pending sidecar fixture");
    fs::write(&paths[2], b"wal").expect("write WAL fixture");
    fs::write(&paths[3], b"shm").expect("write SHM fixture");
    paths
}

#[test]
fn complete_artifact_boundary_removes_sidecars_before_transcript() {
    let dir = tempdir().expect("create temporary directory");
    let transcript = dir.path().join(format!("{}.tlog", Uuid::new_v4()));
    let paths = create_set(&transcript);
    let report =
        remove_session_artifacts_for_transcript(&transcript).expect("remove complete artifact set");
    assert_eq!(report.removed_artifacts, 4);
    assert_eq!(report.removed_paths.last(), Some(&transcript));
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn second_artifact_failure_retains_transcript_and_retry_completes() {
    let dir = tempdir().expect("create temporary directory");
    let transcript = dir.path().join(format!("{}.tlog", Uuid::new_v4()));
    fs::write(&transcript, b"discoverable").expect("write transcript");
    let paths = artifact_paths(&transcript);
    fs::create_dir(&paths[1]).expect("create blocked SQLite path");
    fs::write(paths[1].join("held"), b"held").expect("make directory non-empty");
    fs::write(&paths[2], b"wal").expect("write WAL");
    fs::write(&paths[3], b"shm").expect("write SHM");

    let error = remove_session_artifacts_for_transcript(&transcript)
        .expect_err("SQLite directory must block cleanup");
    let message = error.to_string();
    assert!(message.contains(&paths[1].display().to_string()));
    assert!(message.contains("retryable=true"));
    assert!(
        transcript.exists(),
        "transcript is the discoverability commit point"
    );
    assert!(!paths[2].exists());
    assert!(!paths[3].exists());

    fs::remove_file(paths[1].join("held")).expect("release blocked path");
    fs::remove_dir(&paths[1]).expect("remove blocked directory");
    let retry =
        remove_session_artifacts_for_transcript(&transcript).expect("retry complete cleanup");
    assert_eq!(retry.removed_artifacts, 1);
    assert!(!transcript.exists());
}

#[test]
fn manager_delete_is_retryable_after_sidecar_failure() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let session = manager
        .create_session("test", "/workspace")
        .expect("create session fixture");
    let paths = artifact_paths(&session.file_path);
    fs::create_dir(&paths[1]).expect("create blocked SQLite path");
    fs::write(paths[1].join("held"), b"held").expect("make directory non-empty");

    let error = manager
        .delete_session(&session.id)
        .expect_err("first delete must report exact sidecar failure");
    assert!(error.to_string().contains(&paths[1].display().to_string()));
    assert!(session.file_path.exists());

    fs::remove_file(paths[1].join("held")).expect("release blocked path");
    fs::remove_dir(&paths[1]).expect("remove blocked directory");
    manager
        .delete_session(&session.id)
        .expect("retry delete must complete");
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn retention_counts_zero_byte_session_as_removed() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let session = manager
        .create_session("zero", "/workspace")
        .expect("create zero-byte session");
    assert_eq!(fs::metadata(&session.file_path).expect("metadata").len(), 0);

    let report = manager
        .apply_cleanup(&SessionCleanupPolicy {
            workspace_root: Some("/workspace".to_string()),
            max_sessions_per_workspace: Some(0),
            ..SessionCleanupPolicy::default()
        })
        .expect("apply retention cleanup");

    assert_eq!(report.removed, 1);
    assert_eq!(report.bytes_removed, 0);
    assert!(!session.file_path.exists());
}

#[test]
fn retention_counts_complete_artifact_bytes() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let session = manager
        .create_session("retention", "/workspace")
        .expect("create session fixture");
    let paths = create_set(&session.file_path);
    let expected_bytes = paths
        .iter()
        .map(|path| fs::metadata(path).expect("read fixture metadata").len())
        .sum::<u64>();

    let report = manager
        .apply_cleanup(&SessionCleanupPolicy {
            workspace_root: Some("/workspace".to_string()),
            max_sessions_per_workspace: Some(0),
            ..SessionCleanupPolicy::default()
        })
        .expect("apply retention cleanup");

    assert_eq!(report.removed, 1);
    assert_eq!(report.bytes_removed, expected_bytes);
    assert!(paths.iter().all(|path| !path.exists()));
}

fn orphan_policy(protected_session_ids: Vec<Uuid>) -> OrphanSidecarReconciliationPolicy {
    OrphanSidecarReconciliationPolicy {
        protected_session_ids,
        max_entries: 128,
        minimum_age: Duration::ZERO,
    }
}

#[test]
fn production_root_scan_reconciles_only_valid_safe_orphans() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);

    let sqlite_only = Uuid::new_v4();
    let sqlite_path = workspace.join(format!("{sqlite_only}.pending.sqlite"));
    Connection::open(&sqlite_path).expect("create valid orphan SQLite");

    let wal_only = Uuid::new_v4();
    let wal_path = workspace.join(format!("{wal_only}.pending.sqlite-wal"));
    fs::write(&wal_path, b"wal").expect("write WAL-only orphan");

    let shm_only = Uuid::new_v4();
    let shm_path = workspace.join(format!("{shm_only}.pending.sqlite-shm"));
    fs::write(&shm_path, b"shm").expect("write SHM-only orphan");

    let full = Uuid::new_v4();
    let full_sqlite = workspace.join(format!("{full}.pending.sqlite"));
    Connection::open(&full_sqlite).expect("create full orphan SQLite");
    let full_wal = std::path::PathBuf::from(format!("{}-wal", full_sqlite.display()));
    let full_shm = std::path::PathBuf::from(format!("{}-shm", full_sqlite.display()));
    fs::write(&full_wal, b"wal").expect("write full WAL");
    fs::write(&full_shm, b"shm").expect("write full SHM");

    let with_transcript = Uuid::new_v4();
    let transcript = workspace.join(format!("{with_transcript}.tlog"));
    fs::write(&transcript, b"live transcript").expect("write transcript");
    let transcript_sidecar = workspace.join(format!("{with_transcript}.pending.sqlite"));
    Connection::open(&transcript_sidecar).expect("create transcript sidecar");

    let protected = Uuid::new_v4();
    let protected_sidecar = workspace.join(format!("{protected}.pending.sqlite"));
    Connection::open(&protected_sidecar).expect("create protected sidecar");

    let invalid = workspace.join("not-a-uuid.pending.sqlite");
    fs::write(&invalid, b"invalid").expect("write invalid name");
    let similar = workspace.join(format!("{}.pending.sqlite.backup", Uuid::new_v4()));
    fs::write(&similar, b"similar").expect("write similar suffix");

    let report = manager
        .reconcile_orphan_sidecars(&orphan_policy(vec![protected]))
        .expect("run bounded orphan reconciliation");
    assert_eq!(report.removed_sets, 4);
    assert!((4..=6).contains(&report.removed_artifacts));
    assert!(report.failures.is_empty());
    assert!(!report.bounded);
    assert!(!sqlite_path.exists());
    assert!(!wal_path.exists());
    assert!(!shm_path.exists());
    assert!(!full_sqlite.exists());
    assert!(!full_wal.exists());
    assert!(!full_shm.exists());
    assert!(transcript.exists());
    assert!(transcript_sidecar.exists());
    assert!(protected_sidecar.exists());
    assert!(invalid.exists());
    assert!(similar.exists());

    let second = manager
        .reconcile_orphan_sidecars(&orphan_policy(vec![protected]))
        .expect("second reconciliation is idempotent");
    assert_eq!(second.removed_sets, 0);
    assert_eq!(second.removed_artifacts, 0);
}

#[test]
fn live_sqlite_owner_is_skipped_until_lock_is_released() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);
    let id = Uuid::new_v4();
    let sqlite = workspace.join(format!("{id}.pending.sqlite"));
    let connection = Connection::open(&sqlite).expect("create SQLite");
    connection
        .execute_batch("CREATE TABLE held(value INTEGER); BEGIN EXCLUSIVE;")
        .expect("hold exclusive ownership");

    let first = manager
        .reconcile_orphan_sidecars(&orphan_policy(Vec::new()))
        .expect("busy reconciliation does not crash");
    assert_eq!(first.removed_sets, 0);
    assert!(first.skipped_sets >= 1);
    assert!(sqlite.exists());

    connection.execute_batch("ROLLBACK;").expect("release lock");
    drop(connection);
    let second = manager
        .reconcile_orphan_sidecars(&orphan_policy(Vec::new()))
        .expect("released orphan is removed");
    assert_eq!(second.removed_sets, 1);
    assert!(!sqlite.exists());
}

#[test]
fn orphan_scan_is_bounded() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);
    for _ in 0..2 {
        fs::write(
            workspace.join(format!("{}.pending.sqlite-wal", Uuid::new_v4())),
            b"wal",
        )
        .expect("write bounded candidate");
    }
    let report = manager
        .reconcile_orphan_sidecars(&OrphanSidecarReconciliationPolicy {
            protected_session_ids: Vec::new(),
            max_entries: 1,
            minimum_age: Duration::ZERO,
        })
        .expect("bounded reconciliation");
    assert_eq!(report.scanned_entries, 1);
    assert!(report.bounded);
}

#[cfg(unix)]
#[test]
fn orphan_scan_never_follows_sidecar_symlinks() {
    use std::os::unix::fs::symlink;

    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);
    let outside = dir.path().join("outside.sqlite");
    fs::write(&outside, b"outside").expect("write outside target");
    let link = workspace.join(format!("{}.pending.sqlite", Uuid::new_v4()));
    symlink(&outside, &link).expect("create sidecar symlink");

    let report = manager
        .reconcile_orphan_sidecars(&orphan_policy(Vec::new()))
        .expect("symlink reconciliation");
    assert_eq!(report.removed_sets, 0);
    assert!(link.exists());
    assert!(outside.exists());
}

#[test]
fn rollback_removes_child_index_and_fork_relation_but_preserves_source() {
    use talos_core::message::Message;

    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().join("sessions"));
    let source = manager
        .create_session("source", "/workspace")
        .expect("create source Session");
    source
        .append(&Message::User {
            content: "source-entry".to_string(),
        })
        .expect("append source entry");
    let fork_entry_id = source
        .read_entries()
        .expect("read source entries")
        .last()
        .expect("source entry exists")
        .id
        .clone();
    let child = manager
        .create_session("child", "/workspace")
        .expect("create child Session");
    child
        .append(&Message::User {
            content: "child-entry".to_string(),
        })
        .expect("append child entry");
    talos_session::PendingSubmissionStore::for_session(&child)
        .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
            "provider", "model", None,
        ))
        .expect("initialize child identity");
    manager.update_index(&source).expect("index source");
    manager.update_index(&child).expect("index child");
    manager
        .record_fork(&source.id, &child.id, &fork_entry_id)
        .expect("record fork relation");
    assert_eq!(
        manager
            .get_forks(&source.id.to_string())
            .expect("read source forks")
            .len(),
        1
    );

    manager
        .rollback_session_artifacts(&child)
        .expect("rollback child artifact ownership");

    assert!(source.file_path.exists());
    assert!(!child.file_path.exists());
    assert!(
        manager
            .get_forks(&source.id.to_string())
            .expect("read source forks after rollback")
            .is_empty()
    );
    assert!(manager.get_session(&child.id).is_err());
    assert!(manager.get_session(&source.id).is_ok());
}

#[test]
fn pending_sidecar_creation_first_establishes_transcript_ownership() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().join("sessions"));
    let session = manager
        .defer_create_session("deferred", "/workspace")
        .expect("defer Session without a transcript");
    assert!(!session.file_path.exists());

    let store = talos_session::PendingSubmissionStore::for_session(&session);
    store
        .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
            "provider", "model", None,
        ))
        .expect("initialize runtime identity");

    assert!(session.file_path.exists());
    assert!(store.path().exists());
    let report = manager
        .reconcile_orphan_sidecars(&OrphanSidecarReconciliationPolicy {
            protected_session_ids: Vec::new(),
            max_entries: 64,
            minimum_age: Duration::ZERO,
        })
        .expect("reconciliation must preserve transcript-owned sidecar");
    assert_eq!(report.removed_sets, 0);
    assert!(store.path().exists());

    manager
        .rollback_session_artifacts(&session)
        .expect("rollback complete deferred ownership");
    assert!(!session.file_path.exists());
    assert!(!store.path().exists());
}

#[test]
fn jsonl_transcript_prevents_orphan_sidecar_deletion() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);
    let id = Uuid::new_v4();
    let jsonl = workspace.join(format!("{id}.jsonl"));
    let sqlite = workspace.join(format!("{id}.pending.sqlite"));
    fs::write(&jsonl, b"{}\n").expect("write JSONL transcript");
    Connection::open(&sqlite).expect("create sidecar");

    let report = manager
        .reconcile_orphan_sidecars(&OrphanSidecarReconciliationPolicy {
            protected_session_ids: Vec::new(),
            max_entries: 64,
            minimum_age: Duration::ZERO,
        })
        .expect("reconcile JSONL-owned sidecar");
    assert_eq!(report.removed_sets, 0);
    assert!(jsonl.exists());
    assert!(sqlite.exists());
}

#[test]
fn orphan_scan_budget_counts_unrelated_directory_entries() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions);
    for index in 0..32 {
        fs::write(workspace.join(format!("unrelated-{index}")), b"noise")
            .expect("write unrelated directory entry");
    }

    let report = manager
        .reconcile_orphan_sidecars(&OrphanSidecarReconciliationPolicy {
            protected_session_ids: Vec::new(),
            max_entries: 3,
            minimum_age: Duration::ZERO,
        })
        .expect("bounded reconciliation");
    assert_eq!(report.scanned_entries, 3);
    assert!(report.bounded);
    assert_eq!(report.removed_sets, 0);
}

#[cfg(unix)]
#[test]
fn unix_sidecar_permission_failure_keeps_transcript_and_retry_succeeds() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let manager = SessionManager::with_dir(sessions.clone());
    let session = manager
        .create_session("permission", "/workspace")
        .expect("create Session");
    let store = talos_session::PendingSubmissionStore::for_session(&session);
    store
        .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
            "provider", "model", None,
        ))
        .expect("create sidecar");
    let parent = session.file_path.parent().expect("Session parent");
    let original = fs::metadata(parent).expect("parent metadata").permissions();
    let mut blocked = original.clone();
    blocked.set_mode(0o500);
    fs::set_permissions(parent, blocked).expect("block directory deletion");

    let first = manager.delete_session(&session.id);
    fs::set_permissions(parent, original).expect("restore directory permissions");
    let error = first.expect_err("permission boundary must reject cleanup");
    assert!(error.to_string().contains("retryable=true"));
    assert!(session.file_path.exists());

    manager
        .delete_session(&session.id)
        .expect("retry succeeds after restoring permissions");
    assert!(!session.file_path.exists());
    assert!(!store.path().exists());
}

#[cfg(windows)]
#[test]
fn windows_open_sqlite_handle_keeps_transcript_and_retry_succeeds() {
    let dir = tempdir().expect("create temporary directory");
    let manager = SessionManager::with_dir(dir.path().join("sessions"));
    let session = manager
        .create_session("windows-handle", "/workspace")
        .expect("create Session");
    let store = talos_session::PendingSubmissionStore::for_session(&session);
    store
        .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
            "provider", "model", None,
        ))
        .expect("create sidecar");
    let connection = Connection::open(store.path()).expect("open live SQLite handle");
    connection
        .execute_batch("BEGIN EXCLUSIVE;")
        .expect("hold exclusive SQLite ownership");

    let error = manager
        .delete_session(&session.id)
        .expect_err("Windows open handle must block artifact cleanup");
    assert!(error.to_string().contains("retryable=true"));
    assert!(session.file_path.exists());

    connection
        .execute_batch("ROLLBACK;")
        .expect("release SQLite lock");
    drop(connection);
    manager
        .delete_session(&session.id)
        .expect("retry succeeds after closing SQLite handle");
    assert!(!session.file_path.exists());
    assert!(!store.path().exists());
}
