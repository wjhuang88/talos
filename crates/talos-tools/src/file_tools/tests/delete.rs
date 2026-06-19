use super::*;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_delete_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("temp.txt"), "content").unwrap();

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "temp.txt" })).await;

    assert!(!result.is_error);
    assert!(result.content.contains("deleted"));
    assert!(!dir.path().join("temp.txt").exists());
}

#[tokio::test]
async fn test_delete_directory() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("subdir/nested")).unwrap();
    fs::write(dir.path().join("subdir/nested/file.txt"), "data").unwrap();

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "subdir" })).await;

    assert!(!result.is_error);
    assert!(result.content.contains("deleted"));
    assert!(!dir.path().join("subdir").exists());
}

#[tokio::test]
async fn test_delete_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("empty")).unwrap();

    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "empty" })).await;

    assert!(!result.is_error);
    assert!(!dir.path().join("empty").exists());
}

#[tokio::test]
async fn test_delete_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let tool = DeleteTool::new(dir.path().to_path_buf());
    let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

    assert!(result.is_error);
    assert!(result.content.contains("file not found"));
}

#[tokio::test]
async fn test_delete_path_escape() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().parent().unwrap().join("outside_target.txt");
    fs::write(&outside, "secret").unwrap();

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
    let dir = tempfile::tempdir().unwrap();
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
