use super::*;
use talos_core::tool::AgentTool;

mod delete;

mod file_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_read_tool_read_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("line1"));
        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
    }

    #[tokio::test]
    async fn test_read_tool_line_range() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "line1\nline2\nline3\nline4\n",
        )
        .unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "start_line": 2, "end_line": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 1, "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_zero() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 1 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: a"));
        assert!(!result.content.contains("2: b"));
    }

    #[tokio::test]
    async fn test_read_tool_limit_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: line1"));
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("more lines"));
        assert!(result.content.contains("offset=2"));
    }

    #[tokio::test]
    async fn test_read_tool_pagination_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=10).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("7 more lines"));
        assert!(result.content.contains("offset=3"));
    }

    #[tokio::test]
    async fn test_read_tool_no_truncation_no_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 100 }))
            .await;

        assert!(!result.is_error);
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_takes_precedence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "test.txt",
                "start_line": 1,
                "end_line": 2,
                "offset": 2,
                "limit": 2
            }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("3: line3"));
        assert!(result.content.contains("4: line4"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("2: line2"));
    }

    #[tokio::test]
    async fn test_read_tool_backward_compat_no_params() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a"));
        assert!(result.content.contains("b"));
        assert!(result.content.contains("c"));
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "../outside.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_read_tool_binary_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("binary.bin"), &[0u8, 1, 2, 3]).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "binary.bin" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("binary"));
    }

    #[tokio::test]
    async fn test_read_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[tokio::test]
    async fn test_write_tool_new_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({ "path": "new.txt", "content": "hello world" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("wrote"));

        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_write_tool_refuses_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("existing.txt");
        fs::write(&file, "old content").unwrap();

        let tool = WriteTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "existing.txt", "content": "new content" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("already exists"));
        assert!(result.content.contains("edit tool"));
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "old content");
    }

    #[tokio::test]
    async fn test_write_tool_create_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "a/b/c/deep.txt",
                "content": "deep content"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(temp_dir.path().join("a/b/c/deep.txt")).unwrap();
        assert_eq!(content, "deep content");
    }

    #[tokio::test]
    async fn test_edit_tool_replace_first_occurrence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "foo bar foo baz").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "foo",
                "new_string": "qux"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "qux bar foo baz");
    }

    #[tokio::test]
    async fn test_edit_tool_no_match() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "hello world").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "notfound",
                "new_string": "replacement"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("string not found"));
    }

    #[tokio::test]
    async fn test_edit_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "../outside.txt",
                "old_string": "secret",
                "new_string": "exposed"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_edit_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "nonexistent.txt",
                "old_string": "foo",
                "new_string": "bar"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[test]
    fn test_resolve_workspace_path_within_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("subdir/file.txt");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "content").unwrap();

        let resolved = resolve_workspace_path(temp_dir.path(), "subdir/file.txt").unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_workspace_path_escape_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = resolve_workspace_path(temp_dir.path(), "../outside.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileToolError::PathEscape(_)));
    }
}

#[cfg(test)]
#[allow(warnings)]
mod ls_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;\n").unwrap();
        fs::write(dir.path().join(".hidden"), "secret\n").unwrap();
        dir
    }

    #[tokio::test]
    async fn test_ls_flat() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("Cargo.toml"));
        assert!(result.content.contains("src"));
        assert!(!result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_show_hidden() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "all": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_recursive() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "recursive": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("src/mod.rs"));
    }

    #[tokio::test]
    async fn test_ls_specific_dir() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "src" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_dir_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let src_line = result.content.lines().find(|l| l.contains("src")).unwrap();
        assert!(src_line.ends_with('/'));
        assert!(!src_line.contains(' '));
    }

    #[tokio::test]
    async fn test_ls_file_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let toml_line = result
            .content
            .lines()
            .find(|l| l.contains("Cargo.toml"))
            .unwrap();
        assert!(!toml_line.ends_with('/'));
    }

    #[tokio::test]
    async fn test_ls_file_shows_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap();
        assert!(line.ends_with(" 11"));
    }

    #[tokio::test]
    async fn test_ls_not_found() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent" })).await;

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_ls_single_file() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "main.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_long_format() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let txt_line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap_or_else(|| panic!("no test.txt line in: {}", result.content));
        assert!(txt_line.starts_with('-'));
        assert!(txt_line.contains("rw"));
    }

    #[tokio::test]
    async fn test_ls_long_format_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let src_line = result
            .content
            .lines()
            .find(|l| l.contains("src"))
            .unwrap_or_else(|| panic!("no src line in: {}", result.content));
        assert!(src_line.starts_with('d'));
    }

    #[tokio::test]
    async fn test_ls_long_shows_permissions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "content").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap();
        let perms_field = line.split_whitespace().nth(0).unwrap_or("");
        assert!(perms_field.starts_with('-'));
        assert!(perms_field.len() == 10);
    }

    #[test]
    fn test_ls_tool_is_read_only() {
        let tool = LsTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}
