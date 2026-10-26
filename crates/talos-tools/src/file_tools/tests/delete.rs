use super::*;
use serde_json::json;
use std::fs;
use talos_core::tool::ToolNature;

#[tokio::test]
async fn test_delete_file() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    fs::write(dir.path().join("temp.txt"), "content").expect("operation should succeed");

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "temp.txt" })).await;

    assert!(!result.is_error);
    assert!(result.content.contains("deleted"));
    assert!(!dir.path().join("temp.txt").exists());
}

#[tokio::test]
async fn test_delete_directory() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    fs::create_dir_all(dir.path().join("subdir/nested")).expect("operation should succeed");
    fs::write(dir.path().join("subdir/nested/file.txt"), "data").expect("operation should succeed");

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "subdir" })).await;

    assert!(!result.is_error);
    assert!(result.content.contains("deleted"));
    assert!(!dir.path().join("subdir").exists());
}

#[tokio::test]
async fn test_delete_empty_directory() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    fs::create_dir_all(dir.path().join("empty")).expect("operation should succeed");

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "empty" })).await;

    assert!(!result.is_error);
    assert!(!dir.path().join("empty").exists());
}

#[tokio::test]
async fn test_delete_not_found() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

    assert!(result.is_error);
    assert!(result.content.contains("file not found"));
}

#[tokio::test]
async fn test_delete_path_escape() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let outside = dir
        .path()
        .parent()
        .expect("operation should succeed")
        .join("outside_target.txt");
    fs::write(&outside, "secret").expect("operation should succeed");

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool
        .execute(json!({ "path": "../outside_target.txt" }))
        .await;

    assert!(result.is_error);
    assert!(result.content.contains("path escapes workspace root"));
    assert!(outside.exists());
    let _ = fs::remove_file(&outside);
}

#[tokio::test]
async fn test_delete_refuses_workspace_root() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "." })).await;

    assert!(result.is_error);
    assert!(result.content.contains("workspace root"));
    assert!(dir.path().exists());
}

#[tokio::test]
async fn test_delete_not_read_only() {
    let tool = DeleteTool::new(PathBuf::from("."));
    assert!(!tool.is_read_only());
}

fn recovery_id(content: &str) -> &str {
    content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt")
        .split(';')
        .next()
        .expect("id")
}

#[tokio::test]
async fn recoverable_delete_restores_once_without_overwriting() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("file");
    fs::write(&path, b"original\0bytes").expect("write");
    let tool = DeleteTool::new(dir.path().to_path_buf());
    let deleted = tool
        .execute(json!({"path":"file", "retain_backup":true}))
        .await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(!path.exists());
    let id = recovery_id(&deleted.content);
    fs::write(&path, "new user file").expect("recreate");
    let refused = tool.execute(json!({"path":"file", "recovery_id":id})).await;
    assert!(refused.is_error);
    assert_eq!(fs::read_to_string(&path).expect("read"), "new user file");
    fs::remove_file(&path).expect("remove");
    let restored = tool.execute(json!({"path":"file", "recovery_id":id})).await;
    assert!(!restored.is_error, "{}", restored.content);
    assert_eq!(fs::read(&path).expect("read"), b"original\0bytes");
    assert!(
        tool.execute(json!({"path":"file", "recovery_id":id}))
            .await
            .is_error
    );
}

#[tokio::test]
async fn delete_receipts_are_path_and_instance_bound() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a"), "data").expect("write");
    let tool = DeleteTool::new(dir.path().to_path_buf());
    let other = DeleteTool::new(dir.path().to_path_buf());
    let result = tool
        .execute(json!({"path":"a", "retain_backup":true}))
        .await;
    assert!(!result.is_error, "{}", result.content);
    let id = recovery_id(&result.content);
    assert!(
        tool.execute(json!({"path":"b", "recovery_id":id}))
            .await
            .is_error
    );
    assert!(
        other
            .execute(json!({"path":"a", "recovery_id":id}))
            .await
            .is_error
    );
    assert!(
        !tool
            .execute(json!({"path":"a", "recovery_id":id}))
            .await
            .is_error
    );
}

#[tokio::test]
async fn unsupported_backup_requests_do_not_delete() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir(dir.path().join("directory")).expect("mkdir");
    fs::write(dir.path().join("large"), vec![0; 2 * 1024 * 1024 + 1]).expect("write");
    let tool = DeleteTool::new(dir.path().to_path_buf());
    for name in ["directory", "large"] {
        let result = tool
            .execute(json!({"path":name, "retain_backup":true}))
            .await;
        assert!(result.is_error, "{}", result.content);
        assert!(dir.path().join(name).exists());
    }
    assert!(
        tool.execute(json!({"path":"large", "retain_backup":"yes"}))
            .await
            .is_error
    );
    assert!(dir.path().join("large").exists());
}

#[test]
fn recoverable_delete_declares_original_content_read() {
    let tool = DeleteTool::new(PathBuf::from("."));
    let ordinary = tool.permission_profile(&json!({"path":"file"}));
    let backed_up = tool.permission_profile(&json!({"path":"file", "retain_backup":true}));
    assert_eq!(ordinary.len(), 1);
    assert_eq!(backed_up.len(), 2);
    assert_eq!(backed_up[1].nature, ToolNature::Read);
}
