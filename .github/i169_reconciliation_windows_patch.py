from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match in {path}, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1))


pending = ROOT / "crates/talos-session/src/pending_submission.rs"
replace_once(
    pending,
    "use std::path::{Path, PathBuf};\n",
    "use std::fs::OpenOptions;\nuse std::path::{Path, PathBuf};\n",
)
replace_once(
    pending,
    '''    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
''',
    '''    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !self.session_file.exists() {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.session_file.as_ref())
            {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        let connection = Connection::open(self.path.as_ref())?;
''',
)

artifacts = ROOT / "crates/talos-session/src/artifacts.rs"
replace_once(
    artifacts,
    "/// Default maximum number of recognized sidecar entries inspected in one pass.\n",
    "/// Default maximum number of filesystem directory entries inspected in one pass.\n",
)
replace_once(
    artifacts,
    "    /// Maximum recognized sidecar directory entries inspected in this pass.\n",
    "    /// Maximum filesystem directory entries inspected in this pass.\n",
)
replace_once(
    artifacts,
    "    /// Number of recognized sidecar directory entries inspected.\n",
    "    /// Number of filesystem directory entries inspected, including unrelated names.\n",
)
old_scan = '''    'workspaces: for workspace_entry in fs::read_dir(&canonical_root)? {
        let workspace_entry = workspace_entry?;
        let metadata = fs::symlink_metadata(workspace_entry.path())?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let workspace = workspace_entry.path().canonicalize()?;
        if !workspace.starts_with(&canonical_root) {
            continue;
        }
        for entry in fs::read_dir(&workspace)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(id) = parse_sidecar_session_id(name) else {
                continue;
            };
            if report.scanned_entries >= limit {
                report.bounded = true;
                break 'workspaces;
            }
            report.scanned_entries = report.scanned_entries.saturating_add(1);
            let key = (workspace.clone(), id);
'''
new_scan = '''    'workspaces: for workspace_entry in fs::read_dir(&canonical_root)? {
        if report.scanned_entries >= limit {
            report.bounded = true;
            break;
        }
        report.scanned_entries = report.scanned_entries.saturating_add(1);
        let workspace_entry = workspace_entry?;
        let metadata = fs::symlink_metadata(workspace_entry.path())?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let workspace = workspace_entry.path().canonicalize()?;
        if !workspace.starts_with(&canonical_root) {
            continue;
        }
        for entry in fs::read_dir(&workspace)? {
            if report.scanned_entries >= limit {
                report.bounded = true;
                break 'workspaces;
            }
            report.scanned_entries = report.scanned_entries.saturating_add(1);
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(id) = parse_sidecar_session_id(name) else {
                continue;
            };
            let key = (workspace.clone(), id);
'''
replace_once(artifacts, old_scan, new_scan)

fetch_url = ROOT / "crates/talos-tools/src/fetch_url.rs"
replace_once(
    fetch_url,
    "    const TEST_SERVER_TIMEOUT: StdDuration = StdDuration::from_secs(5);\n",
    "    const TEST_SERVER_TIMEOUT: StdDuration = StdDuration::from_secs(30);\n",
)
replace_once(
    fetch_url,
    '''            "HTTP/1.1 302 Found\\r\\n\\
             Location: http://127.0.0.1:{port}/final\\r\\n\\
             Content-Type: text/html\\r\\n\\
             Content-Length: 0\\r\\n\\r\\n"
''',
    '''            "HTTP/1.1 302 Found\\r\\n\\
             Location: http://127.0.0.1:{port}/final\\r\\n\\
             Content-Type: text/html\\r\\n\\
             Content-Length: 0\\r\\n\\
             Connection: close\\r\\n\\r\\n"
''',
)

artifact_tests = ROOT / "crates/talos-session/tests/i169_session_artifact_cleanup.rs"
text = artifact_tests.read_text()
text += r'''

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
            "provider",
            "model",
            None,
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
            "provider",
            "model",
            None,
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
            "provider",
            "model",
            None,
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

    connection.execute_batch("ROLLBACK;").expect("release SQLite lock");
    drop(connection);
    manager
        .delete_session(&session.id)
        .expect("retry succeeds after closing SQLite handle");
    assert!(!session.file_path.exists());
    assert!(!store.path().exists());
}
'''
artifact_tests.write_text(text)
