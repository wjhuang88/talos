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
        assert!(result.content.contains("11 bytes"));
        assert!(result.content.contains("preview:"));
        assert!(result.content.contains("hello world"));

        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_write_tool_large_preview_is_bounded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());
        let content = (0..40)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");

        let result = tool
            .execute(json!({ "path": "large.txt", "content": content }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("preview:"));
        assert!(result.content.contains("line 0"));
        assert!(result.content.contains("line 19"));
        assert!(!result.content.contains("line 20"));
        assert!(result.content.contains("preview truncated"));
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
        assert!(result.content.contains("edited edit.txt"));
        assert!(result.content.contains("diff:"));
        assert!(result.content.contains("- foo"));
        assert!(result.content.contains("+ qux"));
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

    fn snapshot_header_and_refs(content: &str) -> (String, Vec<String>) {
        let mut lines = content.lines();
        let header = lines.next().expect("snapshot header");
        let id = header
            .strip_prefix("[snapshot:")
            .and_then(|value| value.strip_suffix(']'))
            .expect("snapshot id")
            .to_string();
        let refs = lines
            .filter_map(|line| {
                line.split_once('|')
                    .map(|(reference, _)| reference.to_string())
            })
            .collect();
        (id, refs)
    }

    #[tokio::test]
    async fn snapshot_read_model_projection_is_compact_and_display_projection_is_private() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("source.rs"), "alpha\nbeta\n").unwrap();
        let (read, _, _, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());

        let result = read.execute(json!({"path": "source.rs"})).await;
        assert!(!result.is_error);
        assert!(result.content.starts_with("[snapshot:s"));
        let (snapshot_id, _) = snapshot_header_and_refs(&result.content);
        assert!(snapshot_id.bytes().all(|byte| byte.is_ascii_alphanumeric()));
        for line in result.content.lines().skip(1) {
            let (reference, _) = line.split_once('|').expect("anchored line");
            let (_, code) = reference.split_once(':').expect("line:hash");
            assert_eq!(code.len(), 2);
            assert!(code.bytes().all(|byte| byte.is_ascii_hexdigit()));
        }

        let projection = read.project_result(&result);
        assert_eq!(projection.model_content, result.content);
        assert!(!projection.display_content.contains("snapshot:"));
        assert!(projection.display_content.contains("1: alpha"));
        assert!(projection.display_content.contains("2: beta"));
        assert!(!projection.display_content.contains('|'));
        assert!(!projection.persistence_content.contains("snapshot:"));
        assert!(!projection.persistence_content.contains('|'));
    }

    #[test]
    fn snapshot_edit_schema_has_no_dangling_root_definitions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let (_, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let schema = edit.parameters();
        let serialized = schema.to_string();
        assert_eq!(
            schema.get("type").and_then(serde_json::Value::as_str),
            Some("object")
        );
        if serialized.contains("#/$defs/") {
            assert!(schema.get("$defs").is_some(), "{serialized}");
        }
    }

    #[tokio::test]
    async fn two_hex_anchor_overhead_is_bounded_against_numbered_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content = (1..=200)
            .map(|line| format!("value {line}\n"))
            .collect::<String>();
        fs::write(temp_dir.path().join("large.txt"), content).unwrap();
        let legacy = ReadTool::new(temp_dir.path().to_path_buf());
        let (snapshot, _, _, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let input = json!({"path": "large.txt", "offset": 0, "limit": 200});
        let legacy_result = legacy.execute(input.clone()).await;
        let snapshot_result = snapshot.execute(input).await;

        let overhead = snapshot_result
            .content
            .len()
            .saturating_sub(legacy_result.content.len());
        assert!(overhead <= 4 * 200 + 32, "overhead was {overhead} bytes");
        let hypothetical_32_hex_overhead = 34 * 200;
        assert!(overhead * 8 < hypothetical_32_hex_overhead * 2);
    }

    #[tokio::test]
    async fn oversized_snapshot_file_remains_readable_without_anchor_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content = format!("first\n{}", "x".repeat(2 * 1024 * 1024));
        fs::write(temp_dir.path().join("large.txt"), content).unwrap();
        let (read, _, _, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let result = read
            .execute(json!({"path": "large.txt", "offset": 0, "limit": 1}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert!(!result.content.contains("[snapshot:"));
        assert!(result.content.contains("1: first"));
    }

    #[tokio::test]
    async fn anchored_edit_commits_selected_range_and_invalidates_snapshot() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("source.rs");
        fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "source.rs"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let input = json!({
            "path": "source.rs",
            "snapshot_id": snapshot_id,
            "operations": [{
                "op": "replace_range",
                "start": refs[1],
                "end": refs[2],
                "content": "delta\nepsilon"
            }]
        });
        let projected_input = edit.project_input(&input);
        assert!(projected_input.get("snapshot_id").is_none());
        assert_eq!(projected_input["operations"][0]["start"], "2");

        let result = edit.execute(input.clone()).await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("diff:"));
        assert!(!result.content.contains("snapshot"));
        assert!(!result.content.contains("anchor"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "alpha\ndelta\nepsilon\n"
        );

        let retry = edit.execute(input).await;
        assert!(retry.is_error);
        assert!(retry.content.contains("SNAPSHOT_NOT_FOUND"));
    }

    #[tokio::test]
    async fn anchored_edit_rejects_stale_revision_without_mutation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("source.txt");
        fs::write(&path, "one\ntwo\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "source.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);
        fs::write(&path, "external\none\ntwo\n").unwrap();

        let result = edit
            .execute(json!({
                "path": "source.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[0], "content": "changed"}]
            }))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("FILE_REV_MISMATCH"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "external\none\ntwo\n");
    }

    #[tokio::test]
    async fn anchored_edit_rejects_snapshot_bound_to_another_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("one.txt"), "same\n").unwrap();
        fs::write(temp_dir.path().join("two.txt"), "same\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "one.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let result = edit
            .execute(json!({
                "path": "two.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[0], "content": "changed"}]
            }))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("SNAPSHOT_PATH_MISMATCH"));
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("two.txt")).unwrap(),
            "same\n"
        );
    }

    #[tokio::test]
    async fn anchored_edit_rejects_handle_from_another_runtime_registry() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("one.txt"), "same\n").unwrap();
        let (read, _, _, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let (_, _, foreign_edit, _) =
            super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "one.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let result = foreign_edit
            .execute(json!({
                "path": "one.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[0], "content": "changed"}]
            }))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("SNAPSHOT_NOT_FOUND"));
        assert_eq!(
            fs::read_to_string(temp_dir.path().join("one.txt")).unwrap(),
            "same\n"
        );
    }

    #[tokio::test]
    async fn anchored_edit_preserves_crlf_and_final_newline() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("windows.txt");
        fs::write(&path, b"one\r\ntwo\r\nthree\r\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "windows.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let result = edit
            .execute(json!({
                "path": "windows.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[1], "content": "second"}]
            }))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(fs::read(&path).unwrap(), b"one\r\nsecond\r\nthree\r\n");
    }

    #[tokio::test]
    async fn anchored_edit_preserves_mixed_terminators_and_missing_final_newline() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("mixed.txt");
        fs::write(&path, b"one\r\ntwo\nthree").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "mixed.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let result = edit
            .execute(json!({
                "path": "mixed.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[1], "content": "second"}]
            }))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(fs::read(&path).unwrap(), b"one\r\nsecond\nthree");
    }

    #[tokio::test]
    async fn two_digit_collision_cannot_bypass_full_revision_check() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("collision.txt");
        let mut by_code = std::collections::HashMap::<u8, String>::new();
        let (first, second) = (0..=512)
            .find_map(|value| {
                let candidate = format!("candidate-{value}");
                let code = super::snapshot::digest(candidate.as_bytes())[0];
                by_code
                    .insert(code, candidate.clone())
                    .map(|previous| (previous, candidate))
            })
            .expect("pigeonhole collision in one-byte check code");
        assert_ne!(first, second);
        assert_eq!(
            super::snapshot::digest(first.as_bytes())[0],
            super::snapshot::digest(second.as_bytes())[0]
        );
        fs::write(&path, format!("{first}\n")).unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "collision.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);
        fs::write(&path, format!("{second}\n")).unwrap();

        let result = edit
            .execute(json!({
                "path": "collision.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "replace", "target": refs[0], "content": "changed"}]
            }))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("FILE_REV_MISMATCH"));
        assert_eq!(fs::read_to_string(path).unwrap(), format!("{second}\n"));
    }

    #[tokio::test]
    async fn anchored_edit_rejects_bad_hash_and_overlapping_batch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("source.txt");
        fs::write(&path, "one\ntwo\nthree\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "source.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let bad_hash = edit
            .execute(json!({
                "path": "source.txt",
                "snapshot_id": snapshot_id,
                "operations": [{"op": "delete", "start": "1:zz"}]
            }))
            .await;
        assert!(bad_hash.is_error);
        assert!(bad_hash.content.contains("INVALID_REF"));

        let overlapping = edit
            .execute(json!({
                "path": "source.txt",
                "snapshot_id": snapshot_id,
                "operations": [
                    {"op": "replace_range", "start": refs[0], "end": refs[1], "content": "x"},
                    {"op": "delete", "start": refs[1]}
                ]
            }))
            .await;
        assert!(overlapping.is_error);
        assert!(overlapping.content.contains("INVALID_RANGE"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "one\ntwo\nthree\n");
    }

    #[tokio::test]
    async fn anchored_edit_insert_before_after_and_delete_are_one_batch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("source.txt");
        fs::write(&path, "a\nb\nc\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "source.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);

        let result = edit
            .execute(json!({
                "path": "source.txt",
                "snapshot_id": snapshot_id,
                "operations": [
                    {"op": "insert_before", "target": refs[0], "content": "zero"},
                    {"op": "insert_after", "target": refs[1], "content": "between"},
                    {"op": "delete", "start": refs[2]}
                ]
            }))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(fs::read_to_string(&path).unwrap(), "zero\na\nb\nbetween\n");
    }

    #[tokio::test]
    async fn concurrent_anchored_edits_allow_exactly_one_stale_snapshot_winner() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("source.txt");
        fs::write(&path, "original\n").unwrap();
        let (read, _, edit, _) = super::snapshot_aware_file_tools(temp_dir.path().to_path_buf());
        let read_result = read.execute(json!({"path": "source.txt"})).await;
        let (snapshot_id, refs) = snapshot_header_and_refs(&read_result.content);
        let edit = std::sync::Arc::new(edit);
        let first = {
            let edit = edit.clone();
            let snapshot_id = snapshot_id.clone();
            let target = refs[0].clone();
            tokio::spawn(async move {
                edit.execute(json!({
                    "path": "source.txt",
                    "snapshot_id": snapshot_id,
                    "operations": [{"op": "replace", "target": target, "content": "first"}]
                }))
                .await
            })
        };
        let second = {
            let edit = edit.clone();
            let target = refs[0].clone();
            tokio::spawn(async move {
                edit.execute(json!({
                    "path": "source.txt",
                    "snapshot_id": snapshot_id,
                    "operations": [{"op": "replace", "target": target, "content": "second"}]
                }))
                .await
            })
        };
        let first = first.await.expect("first task");
        let second = second.await.expect("second task");
        assert_eq!(
            [first.is_error, second.is_error]
                .into_iter()
                .filter(|value| !value)
                .count(),
            1
        );
        let content = fs::read_to_string(&path).unwrap();
        assert!(content == "first\n" || content == "second\n");
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

    #[cfg(unix)]
    #[tokio::test]
    async fn snapshot_read_rejects_symlink_that_escapes_workspace() {
        use std::os::unix::fs::symlink;

        let workspace = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("outside.txt");
        fs::write(&outside_file, "secret\n").unwrap();
        symlink(&outside_file, workspace.path().join("link.txt")).unwrap();
        let (read, _, _, _) = super::snapshot_aware_file_tools(workspace.path().to_path_buf());

        let result = read.execute(json!({"path": "link.txt"})).await;
        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
        assert_eq!(fs::read_to_string(outside_file).unwrap(), "secret\n");
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
