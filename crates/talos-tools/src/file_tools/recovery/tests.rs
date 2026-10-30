use super::*;
use crate::file_tools::EditTool;
use serde_json::json;
use talos_core::tool::AgentTool;

#[tokio::test]
async fn edit_tool_partial_failure_exposes_usable_recovery_receipt() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    let path = root.join("file.txt");
    std::fs::write(&path, "original").expect("write");
    let tool = EditTool::new(root);
    super::super::snapshot::FAIL_AFTER_RENAME.set(true);
    let result = tool
        .execute(json!({
            "path": "file.txt", "old_string": "original",
            "new_string": "edited", "retain_backup": true
        }))
        .await;
    assert!(result.is_error, "{}", result.content);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "edited");
    let id = result
        .content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt");
    let restored = tool
        .execute(json!({"path": "file.txt", "recovery_id": id}))
        .await;
    assert!(!restored.is_error, "{}", restored.content);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "original");
    let replay = tool
        .execute(json!({"path": "file.txt", "recovery_id": id}))
        .await;
    assert!(replay.is_error, "{}", replay.content);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "original");
}

#[test]
fn post_rename_edit_failure_keeps_usable_recovery_receipt() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    super::super::snapshot::FAIL_AFTER_RENAME.set(true);
    let error = store
        .replace(&path, b"original", b"edited", None)
        .expect_err("sync failure")
        .to_string();
    let id = error.split("recovery_id: ").nth(1).expect("receipt");
    assert_eq!(std::fs::read(&path).expect("read"), b"edited");
    store
        .restore(&path, id, None)
        .expect("restore after uncertain edit");
    assert_eq!(std::fs::read(&path).expect("read"), b"original");
}

#[test]
fn post_rename_restore_failure_reports_effect_and_retains_backup() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    let id = store
        .replace(&path, b"original", b"edited", None)
        .expect("edit");
    super::super::snapshot::FAIL_AFTER_RENAME.set(true);
    let error = store
        .restore(&path, &id, None)
        .expect_err("sync failure")
        .to_string();
    assert!(error.contains("restoration may already have completed"));
    assert!(error.contains(&id));
    assert_eq!(std::fs::read(&path).expect("read"), b"original");
    assert!(store.records().expect("records").contains_key(&id));
    assert!(matches!(
        store.restore(&path, &id, None),
        Err(FileToolError::FileRevisionMismatch)
    ));
}

#[test]
fn recoverable_and_ordinary_atomic_edits_share_revision_lock() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    let lock = Mutex::new(());
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            store
                .replace(&path, b"original", b"backup edit", Some(&lock))
                .map(|_| ())
        });
        let second = scope.spawn(|| {
            barrier.wait();
            atomic_replace(&path, b"ordinary edit", digest(b"original"), &lock)
        });
        assert_eq!(
            [first.join().expect("first"), second.join().expect("second")]
                .into_iter()
                .filter(Result::is_ok)
                .count(),
            1
        );
    });
}

#[test]
fn concurrent_restore_consumes_backup_exactly_once() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    let id = store
        .replace(&path, b"original", b"edited", None)
        .expect("edit");
    std::thread::scope(|scope| {
        let first = scope.spawn(|| store.restore(&path, &id, None));
        let second = scope.spawn(|| store.restore(&path, &id, None));
        let successes = [first.join().expect("first"), second.join().expect("second")]
            .into_iter()
            .filter(Result::is_ok)
            .count();
        assert_eq!(successes, 1);
    });
    assert_eq!(std::fs::read(&path).expect("read"), b"original");
}

#[test]
fn backup_restores_original_and_is_consumed_once() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    let path = root.join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    let id = store
        .replace(&path, b"original", b"edited", None)
        .expect("edit");
    assert_eq!(std::fs::read(&path).expect("read"), b"edited");
    store.restore(&path, &id, None).expect("restore");
    assert_eq!(std::fs::read(&path).expect("read"), b"original");
    assert!(store.restore(&path, &id, None).is_err());
}

#[test]
fn wrong_path_and_changed_content_do_not_consume_or_overwrite() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    let path = root.join("file.txt");
    let other = root.join("other.txt");
    std::fs::write(&path, b"original").expect("write");
    std::fs::write(&other, b"edited").expect("write");
    let store = FileRecoveryStore::default();
    let id = store
        .replace(&path, b"original", b"edited", None)
        .expect("edit");
    assert!(matches!(
        store.restore(&other, &id, None),
        Err(FileToolError::SnapshotPathMismatch)
    ));
    std::fs::write(&path, b"user changes").expect("write");
    assert!(matches!(
        store.restore(&path, &id, None),
        Err(FileToolError::FileRevisionMismatch)
    ));
    assert_eq!(std::fs::read(&path).expect("read"), b"user changes");
    assert_eq!(std::fs::read(&other).expect("read"), b"edited");
    assert!(store.records().expect("records").contains_key(&id));
}

#[test]
fn expiry_and_capacity_fail_without_mutation() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    let path = root.join("file.txt");
    std::fs::write(&path, b"original").expect("write");
    let store = FileRecoveryStore::default();
    let id = store
        .replace(&path, b"original", b"edited", None)
        .expect("edit");
    store
        .records()
        .expect("records")
        .get_mut(&id)
        .expect("backup")
        .created = Instant::now() - TTL;
    assert!(store.restore(&path, &id, None).is_err());
    assert_eq!(std::fs::read(&path).expect("read"), b"edited");
    let oversized = vec![0; MAX_FILE_BYTES + 1];
    assert!(store.replace(&path, &oversized, b"new", None).is_err());
    for _ in 0..MAX_RECORDS {
        store
            .replace(&path, b"edited", b"edited", None)
            .expect("backup");
    }
    assert!(store.replace(&path, b"edited", b"new", None).is_err());
    assert_eq!(std::fs::read(&path).expect("read"), b"edited");
}

#[tokio::test]
async fn edit_tool_returns_a_real_recovery_receipt_and_restores() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    std::fs::write(root.join("file.txt"), "before").expect("write");
    let tool = EditTool::new(root.clone());
    let result = tool.execute(json!({"path":"file.txt","old_string":"before","new_string":"after","retain_backup":true})).await;
    assert!(!result.is_error, "{}", result.content);
    let id = result
        .content
        .lines()
        .find_map(|line| line.strip_prefix("recovery_id: "))
        .expect("receipt");
    let restored = tool
        .execute(json!({"path":"file.txt","recovery_id":id}))
        .await;
    assert!(!restored.is_error, "{}", restored.content);
    assert_eq!(
        std::fs::read_to_string(root.join("file.txt")).expect("read"),
        "before"
    );
    assert!(
        tool.execute(json!({"path":"file.txt","recovery_id":id}))
            .await
            .is_error
    );
    let validator = jsonschema::validator_for(&tool.parameters()).expect("schema");
    assert!(validator.is_valid(&json!({"path":"file.txt","recovery_id":id})));
    assert!(validator.is_valid(
        &json!({"path":"file.txt","old_string":"a","new_string":"b","retain_backup":true})
    ));
}

#[tokio::test]
async fn recovery_preserves_path_permission_and_rejects_mixed_modes() {
    let dir = tempfile::tempdir().expect("dir");
    let external = tempfile::tempdir().expect("external");
    let path = external.path().join("file.txt");
    std::fs::write(&path, "untouched").expect("write");
    let tool = EditTool::new(dir.path().canonicalize().expect("root"));
    assert!(
        tool.execute(json!({"path":path,"recovery_id":"missing"}))
            .await
            .is_error
    );
    assert_eq!(std::fs::read_to_string(path).expect("read"), "untouched");
    assert!(
        tool.execute(
            json!({"path":"file.txt","recovery_id":"missing","old_string":"a","new_string":"b"})
        )
        .await
        .is_error
    );
}

#[test]
fn total_byte_budget_keeps_existing_backups_and_refuses_new_mutation() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file.txt");
    let content = vec![b'a'; MAX_FILE_BYTES];
    std::fs::write(&path, &content).expect("write");
    let store = FileRecoveryStore::default();
    for _ in 0..MAX_TOTAL_BYTES / MAX_FILE_BYTES {
        store
            .replace(&path, &content, &content, None)
            .expect("backup");
    }
    assert!(store.replace(&path, &content, b"changed", None).is_err());
    assert_eq!(std::fs::read(&path).expect("read"), content);
    assert_eq!(
        store.records().expect("records").len(),
        MAX_TOTAL_BYTES / MAX_FILE_BYTES
    );
}

#[tokio::test]
async fn anchored_edit_backup_restores_original_and_invalidates_old_snapshot() {
    let dir = tempfile::tempdir().expect("dir");
    let root = dir.path().canonicalize().expect("root");
    std::fs::write(root.join("file.txt"), "before\nafter\n").expect("write");
    let (read, _, edit, _) = crate::file_tools::snapshot_aware_file_tools(root.clone());
    let read_result = read.execute(json!({"path":"file.txt"})).await;
    let mut lines = read_result.content.lines();
    let snapshot = lines
        .next()
        .expect("snapshot")
        .strip_prefix("[snapshot:")
        .and_then(|s| s.strip_suffix(']'))
        .expect("id");
    let target = lines
        .next()
        .expect("line")
        .split_once('|')
        .expect("reference")
        .0;
    let input = json!({"path":"file.txt","snapshot_id":snapshot,"retain_backup":true,"operations":[{"op":"replace","target":target,"content":"changed"}]});
    let result = edit.execute(input.clone()).await;
    assert!(!result.is_error, "{}", result.content);
    let id = result
        .content
        .lines()
        .find_map(|line| line.strip_prefix("recovery_id: "))
        .expect("receipt");
    assert!(
        !edit
            .execute(json!({"path":"file.txt","recovery_id":id}))
            .await
            .is_error
    );
    assert_eq!(
        std::fs::read_to_string(root.join("file.txt")).expect("read"),
        "before\nafter\n"
    );
    assert!(edit.execute(input).await.is_error);
}

#[cfg(unix)]
#[test]
fn backup_open_rejects_leaf_links_and_special_files() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target");
    std::fs::write(&target, "private").expect("write");
    let link = dir.path().join("link");
    symlink(&target, &link).expect("symlink");
    assert!(super::open_backup_file(&link).is_err());
    let socket = dir.path().join("socket");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).expect("bind");
    assert!(super::open_backup_file(&socket).is_err());
    let fifo = dir.path().join("fifo");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("Unix mkfifo fixture")
            .success()
    );
    // Without O_NONBLOCK this open would wait indefinitely for a writer.
    assert!(super::open_backup_file(&fifo).is_err());
}

#[tokio::test]
async fn recoverable_edit_rejects_oversize_before_mutation() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("large.txt");
    let bytes = vec![b'a'; MAX_FILE_BYTES + 1];
    std::fs::write(&path, &bytes).expect("write");
    let tool = EditTool::new(dir.path().to_path_buf());
    let result = tool
        .execute(
            json!({"path":"large.txt", "old_string":"a", "new_string":"b", "retain_backup":true}),
        )
        .await;
    assert!(result.is_error);
    assert!(result.content.contains("2 MiB"), "{}", result.content);
    assert_eq!(std::fs::read(path).expect("read"), bytes);
}

#[tokio::test]
async fn external_recoverable_edit_requires_read_and_write() {
    use talos_core::tool::{ToolAuthorizationScope, ToolExecutionAuthorization, ToolNature};
    let workspace = tempfile::tempdir().expect("workspace");
    let external = tempfile::tempdir().expect("external");
    let path = external.path().join("file.txt");
    std::fs::write(&path, "before").expect("write");
    let path = path.to_string_lossy().into_owned();
    let write = ToolExecutionAuthorization::for_path(
        "edit",
        ToolNature::Write,
        workspace.path(),
        &path,
        ToolAuthorizationScope::Once,
    )
    .expect("write grant");
    let read = ToolExecutionAuthorization::for_path(
        "edit",
        ToolNature::Read,
        workspace.path(),
        &path,
        ToolAuthorizationScope::Once,
    )
    .expect("read grant");
    let tool = EditTool::new(workspace.path().to_path_buf());
    let input =
        json!({"path":path,"old_string":"before","new_string":"after","retain_backup":true});
    assert_eq!(tool.permission_profile(&input).len(), 2);
    let refused = tool
        .execute_authorized(input.clone(), std::slice::from_ref(&write))
        .await;
    assert!(refused.is_error);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "before");
    let result = tool.execute_authorized(input, &[write, read]).await;
    assert!(!result.is_error, "{}", result.content);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "after");
}

#[test]
fn oversized_replacement_is_rejected_before_backup_or_write() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file");
    std::fs::write(&path, b"before").expect("write");
    let store = FileRecoveryStore::default();
    assert!(
        store
            .replace(&path, b"before", &vec![b'x'; MAX_FILE_BYTES + 1], None)
            .is_err()
    );
    assert!(store.records().expect("records").is_empty());
    assert_eq!(std::fs::read(path).expect("read"), b"before");
}

#[test]
fn atomic_revision_check_rejects_grown_file_without_mutation() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().canonicalize().expect("root").join("file");
    let bytes = vec![b'x'; MAX_FILE_BYTES + 1];
    std::fs::write(&path, &bytes).expect("write");
    let result = atomic_replace(&path, b"new", digest(b"before"), &Mutex::new(()));
    assert!(result.is_err());
    assert_eq!(std::fs::read(path).expect("read"), bytes);
}
