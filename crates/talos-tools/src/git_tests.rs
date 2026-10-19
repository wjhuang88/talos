use super::*;
use talos_core::tool::{ToolNature, ToolResourceKind};

#[cfg(target_os = "linux")]
#[tokio::test]
async fn git_diff_preserves_distinct_non_utf8_path_identities() {
    use std::os::unix::ffi::OsStrExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    let first = dir.path().join(std::ffi::OsStr::from_bytes(b"file-\xfe"));
    let second = dir.path().join(std::ffi::OsStr::from_bytes(b"file-\xff"));
    std::fs::write(&first, "first-head\n").expect("fixture");
    std::fs::write(&second, "second-head\n").expect("fixture");
    run(&["add", "."]);
    run(&["commit", "-m", "initial"]);
    std::fs::write(&first, "first-index\n").expect("fixture");
    std::fs::write(&second, "second-index\n").expect("fixture");
    run(&["add", "."]);
    std::fs::write(&first, "first-worktree\n").expect("fixture");
    std::fs::write(&second, "second-worktree\n").expect("fixture");

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    for (staged, old, new) in [(true, "head", "index"), (false, "index", "worktree")] {
        let result = tool.execute(serde_json::json!({"staged": staged})).await;
        assert!(!result.is_error, "{}", result.content);
        assert!(
            result.content.contains("a/file-\\376 b/file-\\376"),
            "{}",
            result.content
        );
        assert!(
            result.content.contains("a/file-\\377 b/file-\\377"),
            "{}",
            result.content
        );
        for name in ["first", "second"] {
            assert!(
                result.content.contains(&format!("-{name}-{old}")),
                "{}",
                result.content
            );
            assert!(
                result.content.contains(&format!("+{name}-{new}")),
                "{}",
                result.content
            );
        }
        assert!(!result.content.contains('\u{fffd}'));
    }
}

#[test]
fn git_diff_display_escaping_does_not_alias_literal_backslashes() {
    assert_ne!(
        diff_path_label(b"file-\xff"),
        diff_path_label(b"file-\\377")
    );
    assert_eq!(diff_path_label(b"line\nname"), "line\\012name");
    assert_eq!(diff_path_label("中文".as_bytes()), "中文");
}

#[tokio::test]
async fn git_diff_does_not_present_one_conflict_stage_as_an_ordinary_diff() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str], succeeds: bool| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert_eq!(
            output.status.success(),
            succeeds,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"], true);
    run(&["config", "user.email", "test@test.com"], true);
    run(&["config", "user.name", "Test"], true);
    std::fs::write(dir.path().join("conflict.txt"), "base\n").expect("fixture");
    run(&["add", "."], true);
    run(&["commit", "-m", "base"], true);
    run(&["branch", "other"], true);
    std::fs::write(dir.path().join("conflict.txt"), "ours\n").expect("fixture");
    run(&["commit", "-am", "ours"], true);
    run(&["checkout", "other"], true);
    std::fs::write(dir.path().join("conflict.txt"), "theirs\n").expect("fixture");
    run(&["commit", "-am", "theirs"], true);
    run(&["checkout", "-"], true);
    run(&["merge", "other"], false);

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    for staged in [false, true] {
        let result = tool.execute(serde_json::json!({"staged": staged})).await;
        assert!(result.is_error, "{}", result.content);
        assert!(
            result
                .content
                .contains("unresolved index path conflict.txt"),
            "{}",
            result.content
        );
    }
}

#[tokio::test]
async fn git_diff_distinguishes_add_delete_and_binary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    std::fs::write(dir.path().join("text"), "added\n").expect("fixture");
    std::fs::write(dir.path().join("binary"), b"one\0").expect("fixture");
    run(&["add", "."]);
    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let added = tool.execute(serde_json::json!({"staged":true})).await;
    assert!(!added.is_error, "{}", added.content);
    assert!(
        added.content.contains("--- /dev/null\n+++ b/text"),
        "{}",
        added.content
    );
    assert!(added.content.contains("+added"));
    assert!(
        added
            .content
            .contains("Binary files /dev/null and b/binary differ")
    );
    assert!(!added.content.contains("--- ---"));
    run(&["commit", "-m", "initial"]);
    let clean = tool.execute(serde_json::json!({"staged":true})).await;
    assert_eq!(clean.content, "no changes");
    std::fs::remove_file(dir.path().join("text")).expect("remove fixture");
    let deleted = tool.execute(serde_json::json!({})).await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(
        deleted.content.contains("--- a/text\n+++ /dev/null"),
        "{}",
        deleted.content
    );
    assert!(deleted.content.contains("-added"));
    run(&["add", "-u"]);
    let staged_deleted = tool.execute(serde_json::json!({"staged":true})).await;
    assert_eq!(staged_deleted.content, deleted.content);
}

#[tokio::test]
async fn git_diff_reports_empty_file_addition() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    std::fs::write(dir.path().join("empty"), []).expect("empty fixture");
    run(&["add", "empty"]);
    let result = GitDiffTool::new(dir.path().to_path_buf())
        .execute(serde_json::json!({"staged": true}))
        .await;
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("new file mode 100644"),
        "{}",
        result.content
    );
    assert!(result.content.contains("diff --git a/empty b/empty"));
}

#[cfg(unix)]
#[tokio::test]
async fn git_diff_reports_mode_only_change() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    std::fs::write(dir.path().join("script"), "same\n").expect("fixture");
    run(&["add", "script"]);
    run(&["commit", "-m", "initial"]);
    std::fs::set_permissions(
        dir.path().join("script"),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("make executable");
    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let unstaged = tool.execute(serde_json::json!({})).await;
    assert!(!unstaged.is_error, "{}", unstaged.content);
    assert!(
        unstaged
            .content
            .contains("old mode 100644\nnew mode 100755"),
        "{}",
        unstaged.content
    );
    assert!(!unstaged.content.contains("--- a/script"));
    run(&["add", "script"]);
    // Turning off worktree executable-bit tracking must not hide a mode
    // change already recorded in the index.
    run(&["config", "core.fileMode", "false"]);
    let staged = tool.execute(serde_json::json!({"staged": true})).await;
    assert!(!staged.is_error, "{}", staged.content);
    assert!(
        staged.content.contains("old mode 100644\nnew mode 100755"),
        "{}",
        staged.content
    );
}

#[tokio::test]
async fn git_diff_line_limit_counts_metadata_and_exact_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    for args in [vec!["init"], vec!["add", "."]] {
        if args[0] == "add" {
            std::fs::write(dir.path().join("a"), []).expect("fixture");
            std::fs::write(dir.path().join("b"), []).expect("fixture");
        }
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let tool = GitDiffTool::new(dir.path().to_path_buf());
    for limit in [0, 1, 2, 3, 4, 5] {
        let result = tool
            .execute(serde_json::json!({"staged": true, "max_lines": limit}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        let lines: Vec<_> = result.content.lines().collect();
        if limit < 4 {
            assert_eq!(lines.len(), limit + 1, "{}", result.content);
            assert_eq!(lines[limit], format!("... (truncated at {limit} lines)"));
        } else {
            assert_eq!(lines.len(), 4, "{}", result.content);
            assert!(!result.content.contains("truncated"));
        }
    }
}

#[test]
fn git_diff_read_errors_are_not_missing_content() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(read_worktree_blob_content(dir.path(), Path::new(".")).is_err());
    assert_eq!(
        read_worktree_blob_content(dir.path(), Path::new("absent")).expect("missing"),
        None
    );
}

#[cfg(unix)]
#[test]
fn git_diff_rejects_fifo_without_waiting_for_a_writer() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fifo = dir.path().join("pipe");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("FIFO fixture")
            .success()
    );
    // Check prompt rejection with no writer; release a regressed reader below
    // before joining it so the expected failure does not strand the worker.
    let (tx, rx) = std::sync::mpsc::channel();
    let root = dir.path().to_path_buf();
    let worker = std::thread::spawn(move || {
        let result = read_worktree_blob_content(&root, Path::new("pipe"));
        let _ = tx.send(result);
    });
    let result = rx.recv_timeout(std::time::Duration::from_secs(2));
    if result.is_err() {
        // Unblock a regression so its test worker can be joined.
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo)
                .expect("release reader"),
        );
    }
    worker.join().expect("reader worker");
    let error = result
        .expect("special file rejected promptly")
        .expect_err("FIFO refused");
    assert!(error.to_string().contains("not a regular file or symlink"));
}

#[cfg(unix)]
#[test]
fn git_diff_reads_symlink_entry_not_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("target"), "not link content").expect("fixture");
    std::os::unix::fs::symlink("target", dir.path().join("link")).expect("symlink");
    assert_eq!(
        read_worktree_blob_content(dir.path(), Path::new("link")).expect("link"),
        Some(b"target".to_vec())
    );
    std::fs::remove_file(dir.path().join("target")).expect("remove fixture");
    assert_eq!(
        read_worktree_blob_content(dir.path(), Path::new("link")).expect("broken link"),
        Some(b"target".to_vec())
    );
}

#[cfg(unix)]
#[tokio::test]
async fn git_diff_does_not_follow_replaced_ancestor_symlink() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("repository");
    let outside = tempfile::tempdir().expect("outside");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("host Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.name", "Test"]);
    run(&["config", "user.email", "test@example.invalid"]);
    std::fs::create_dir(root.path().join("dir")).expect("directory");
    std::fs::write(root.path().join("dir/file"), "tracked\n").expect("tracked");
    run(&["add", "dir/file"]);
    run(&["commit", "-m", "base"]);
    std::fs::write(outside.path().join("file"), "outside-private-content\n").expect("outside");
    std::fs::remove_dir_all(root.path().join("dir")).expect("remove directory");
    let deleted = GitDiffTool::new(root.path().to_path_buf())
        .execute(serde_json::json!({}))
        .await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(deleted.content.contains("deleted file mode"));
    assert!(deleted.content.contains("-tracked"));
    symlink(outside.path(), root.path().join("dir")).expect("replace with symlink");

    let result = GitDiffTool::new(root.path().to_path_buf())
        .execute(serde_json::json!({}))
        .await;
    assert!(
        !result.content.contains("outside-private-content"),
        "{}",
        result.content
    );
    assert!(read_worktree_blob_content(root.path(), Path::new("dir/file")).is_err());
    assert!(result.is_error, "{}", result.content);
    assert!(result.content.contains("ancestor"));
}

#[tokio::test]
async fn git_diff_literal_path_boundary_and_untracked_exclusion() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required for Git integration tests");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    for name in ["foo", "foobar"] {
        std::fs::write(dir.path().join(name), "before\n").expect("write fixture");
    }
    run(&["add", "foo", "foobar"]);
    run(&["commit", "-m", "initial"]);
    for name in ["foo", "foobar"] {
        std::fs::write(dir.path().join(name), "after\n").expect("modify fixture");
    }
    std::fs::write(dir.path().join("untracked"), "untracked sentinel\n").expect("write fixture");
    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let all = tool.execute(serde_json::json!({})).await;
    assert!(!all.is_error, "{}", all.content);
    assert!(all.content.contains("a/foo"));
    assert!(all.content.contains("a/foobar"));
    assert!(!all.content.contains("untracked"), "{}", all.content);
    let filtered = tool.execute(serde_json::json!({"path": "foo"})).await;
    assert!(!filtered.is_error, "{}", filtered.content);
    assert!(filtered.content.contains("a/foo"), "{}", filtered.content);
    assert!(!filtered.content.contains("foobar"), "{}", filtered.content);
}

#[tokio::test]
async fn git_diff_ref_comparison_requires_two_resolved_refs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required for Git integration tests");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    std::fs::write(dir.path().join("file"), "first\n").expect("write fixture");
    run(&["add", "file"]);
    run(&["commit", "-m", "first"]);
    std::fs::write(dir.path().join("file"), "second\n").expect("write fixture");
    run(&["add", "file"]);
    run(&["commit", "-m", "second"]);

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let valid = tool
        .execute(serde_json::json!({"base_ref": "HEAD~1", "head_ref": "HEAD"}))
        .await;
    assert!(!valid.is_error, "{}", valid.content);
    assert!(valid.content.contains("+second"), "{}", valid.content);

    // The read-only bridge must not run repository-selected converters, and
    // caller paths are literal, not magic Git pathspecs.
    std::fs::write(dir.path().join(".gitattributes"), "file diff=i300fixture\n")
        .expect("attributes fixture");
    run(&[
        "config",
        "diff.i300fixture.textconv",
        "i300-nonexistent-converter",
    ]);
    run(&["config", "color.ui", "always"]);
    let raw = tool
        .execute(serde_json::json!({"base_ref": "HEAD~1", "head_ref": "HEAD", "path": "file"}))
        .await;
    assert!(!raw.is_error, "{}", raw.content);
    assert!(raw.content.contains("+second"), "{}", raw.content);
    assert!(
        !raw.content.contains('\u{1b}'),
        "color config must not alter patch output"
    );
    let literal = tool
        .execute(serde_json::json!({"base_ref": "HEAD~1", "head_ref": "HEAD", "path": ":(glob)*"}))
        .await;
    assert!(!literal.is_error, "{}", literal.content);
    assert!(
        literal.content.is_empty(),
        "pathspec magic must not expand: {}",
        literal.content
    );

    let missing = tool.execute(serde_json::json!({"base_ref": "HEAD"})).await;
    assert!(missing.is_error, "{}", missing.content);
    assert!(missing.content.contains("must be provided together"));

    let option = tool
        .execute(serde_json::json!({"base_ref": "--output=outside", "head_ref": "HEAD"}))
        .await;
    assert!(option.is_error, "{}", option.content);
    assert!(!dir.path().join("outside").exists());
}

#[tokio::test]
async fn git_diff_ref_comparison_rejects_oversized_host_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("host git required for Git integration tests");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);
    std::fs::write(dir.path().join("large"), "first\n").expect("fixture");
    run(&["add", "large"]);
    run(&["commit", "-m", "first"]);
    std::fs::write(dir.path().join("large"), "x".repeat(300_000)).expect("fixture");
    run(&["add", "large"]);
    run(&["commit", "-m", "second"]);

    let result = GitDiffTool::new(dir.path().to_path_buf())
        .execute(serde_json::json!({"base_ref": "HEAD~1", "head_ref": "HEAD"}))
        .await;
    assert!(result.is_error, "{}", result.content);
    assert!(
        result.content.contains("output exceeds 256 KiB"),
        "{}",
        result.content
    );
}

#[tokio::test]
async fn git_push_refuses_legacy_execution_without_prepared_admission() {
    let tool = GitPushTool::new(PathBuf::from("/workspace"));
    let result = tool
        .execute(serde_json::json!({"remote":"upstream","branch":"main"}))
        .await;
    assert!(result.is_error && result.content.contains("prepared invocation"));
}

#[test]
fn git_pull_permission_profile_has_execute_remote_and_workspace_write() {
    let tool = GitPullTool::new(PathBuf::from("/workspace"));
    let profile = tool.permission_profile(&serde_json::json!({}));

    assert_eq!(profile.len(), 3);
    assert_eq!(profile[0].nature, ToolNature::Execute);
    assert_eq!(profile[0].resource.as_deref(), Some("git"));
    assert_eq!(profile[1].nature, ToolNature::Network);
    assert_eq!(profile[1].resource.as_deref(), Some("origin"));
    assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Remote));
    assert_eq!(profile[2].nature, ToolNature::Write);
    assert_eq!(profile[2].resource.as_deref(), Some("/workspace"));
    assert_eq!(profile[2].resource_kind, Some(ToolResourceKind::Path));
}

#[tokio::test]
async fn git_pull_refuses_legacy_execution_until_prepared_effects_exist() {
    let tool = GitPullTool::new(PathBuf::from("/workspace"));
    let result = tool.execute(serde_json::json!({})).await;
    assert!(result.is_error);
    assert!(result.content.contains("prepared invocation"));
    assert!(result.content.contains("fetch, merge/rebase"));
}

#[tokio::test]
async fn host_git_unavailable_returns_actionable_error() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let err = crate::git::git_write::run_host_git_with_program(
        "talos-host-git-that-should-not-exist",
        tempdir.path(),
        &["status"],
    )
    .await
    .expect_err("missing git executable should fail");

    let message = err.to_string();
    assert!(message.contains("read-only tools"), "{message}");
    #[cfg(unix)]
    {
        assert!(
            message.contains("verify Git is installed and executable"),
            "{message}"
        );
        assert!(message.contains("No such file or directory"), "{message}");
    }
    #[cfg(not(unix))]
    assert!(message.contains("git not installed"), "{message}");
}

#[tokio::test]
async fn git_diff_produces_unified_diff_content() {
    if std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipping: host git not available");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("git command")
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);

    std::fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n")
        .expect("operation should succeed");
    run(&["add", "test.txt"]);
    run(&["commit", "-m", "initial"]);

    std::fs::write(dir.path().join("test.txt"), "line1\nmodified\nline3\n")
        .expect("operation should succeed");

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let result = tool.execute(serde_json::json!({})).await;

    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("diff --git a/test.txt b/test.txt"),
        "expected diff --git header, got: {}",
        result.content
    );
    assert!(
        result.content.contains("--- a/test.txt"),
        "expected --- header, got: {}",
        result.content
    );
    assert!(
        result.content.contains("+++ b/test.txt"),
        "expected +++ header, got: {}",
        result.content
    );
    assert!(
        result.content.contains("-line2"),
        "expected removed line, got: {}",
        result.content
    );
    assert!(
        result.content.contains("+modified"),
        "expected added line, got: {}",
        result.content
    );
}

#[tokio::test]
async fn git_diff_staged_shows_head_vs_index() {
    if std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipping: host git not available");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("git command")
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);

    std::fs::write(dir.path().join("file.txt"), "original\n").expect("operation should succeed");
    run(&["add", "file.txt"]);
    run(&["commit", "-m", "initial"]);

    // A staged-only change must be visible without an additional worktree edit.
    std::fs::write(dir.path().join("file.txt"), "staged\n").expect("operation should succeed");
    run(&["add", "file.txt"]);

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let staged_result = tool.execute(serde_json::json!({"staged": true})).await;
    assert!(!staged_result.is_error, "{}", staged_result.content);
    assert!(
        staged_result.content.contains("-original"),
        "expected -original in staged diff, got: {}",
        staged_result.content
    );
    assert!(
        staged_result.content.contains("+staged"),
        "expected +staged in staged diff, got: {}",
        staged_result.content
    );
    assert!(
        !staged_result.content.contains("+unstaged"),
        "staged diff must exclude unstaged changes, got: {}",
        staged_result.content
    );

    let clean_worktree = tool.execute(serde_json::json!({})).await;
    assert!(!clean_worktree.is_error, "{}", clean_worktree.content);
    assert_eq!(clean_worktree.content, "no changes");

    std::fs::write(dir.path().join("file.txt"), "unstaged\n").expect("operation should succeed");
    let all_result = tool.execute(serde_json::json!({})).await;
    assert!(!all_result.is_error, "{}", all_result.content);
    assert!(
        all_result.content.contains("-staged"),
        "unstaged diff must use index as baseline: {}",
        all_result.content
    );
    assert!(
        !all_result.content.contains("-original"),
        "unstaged diff must not compare against HEAD: {}",
        all_result.content
    );
    assert!(
        all_result.content.contains("+unstaged"),
        "default diff should include unstaged changes, got: {}",
        all_result.content
    );
}

#[tokio::test]
async fn git_diff_path_filter_excludes_non_matching() {
    if std::process::Command::new("git")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("skipping: host git not available");
        return;
    }

    let dir = tempfile::tempdir().expect("tempdir");
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("git command")
    };
    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test"]);

    std::fs::create_dir_all(dir.path().join("src")).expect("operation should succeed");
    std::fs::create_dir_all(dir.path().join("docs")).expect("operation should succeed");
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n")
        .expect("operation should succeed");
    std::fs::write(dir.path().join("docs/readme.md"), "# README\n")
        .expect("operation should succeed");
    run(&["add", "."]);
    run(&["commit", "-m", "initial"]);

    std::fs::write(
        dir.path().join("src/main.rs"),
        "fn main() { println!(); }\n",
    )
    .expect("operation should succeed");
    std::fs::write(dir.path().join("docs/readme.md"), "# README Updated\n")
        .expect("operation should succeed");

    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let result = tool.execute(serde_json::json!({"path": "src/"})).await;

    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("src/main.rs"),
        "expected src/main.rs in filtered diff, got: {}",
        result.content
    );
    assert!(
        !result.content.contains("docs/readme.md"),
        "docs/readme.md should be filtered out, got: {}",
        result.content
    );
}

#[tokio::test]
async fn git_diff_staged_gitlink_uses_oid_without_loading_submodule_object() {
    let dir = tempfile::tempdir().expect("workspace");
    let run = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("Git fixture");
        assert!(output.status.success(), "{output:?}");
    };
    run(&["init"]);
    run(&["config", "user.name", "Test"]);
    run(&["config", "user.email", "test@example.invalid"]);
    let first = "1111111111111111111111111111111111111111";
    let second = "2222222222222222222222222222222222222222";
    run(&[
        "update-index",
        "--add",
        "--cacheinfo",
        "160000",
        first,
        "module",
    ]);
    let tool = GitDiffTool::new(dir.path().to_path_buf());
    let added = tool.execute(serde_json::json!({"staged":true})).await;
    assert!(!added.is_error, "{}", added.content);
    assert!(
        added.content.contains("new file mode 160000"),
        "{}",
        added.content
    );
    assert!(
        added
            .content
            .contains(&format!("+Subproject commit {first}")),
        "{}",
        added.content
    );
    run(&["commit", "-m", "gitlink"]);
    run(&["update-index", "--cacheinfo", "160000", second, "module"]);
    let changed = tool.execute(serde_json::json!({"staged":true})).await;
    assert!(!changed.is_error, "{}", changed.content);
    assert!(
        changed
            .content
            .contains(&format!("-Subproject commit {first}")),
        "{}",
        changed.content
    );
    assert!(
        changed
            .content
            .contains(&format!("+Subproject commit {second}")),
        "{}",
        changed.content
    );
    run(&["update-index", "--force-remove", "module"]);
    let deleted = tool.execute(serde_json::json!({"staged": true})).await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(
        deleted.content.contains("deleted file mode 160000"),
        "{}",
        deleted.content
    );
    assert!(
        deleted
            .content
            .contains(&format!("-Subproject commit {first}")),
        "{}",
        deleted.content
    );
}

#[cfg(unix)]
#[tokio::test]
async fn git_diff_ignores_worktree_executable_bit_when_filemode_is_false() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("workspace");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("Git fixture");
        assert!(output.status.success(), "{output:?}");
        output.stdout
    };
    git(&["init"]);
    git(&["config", "core.fileMode", "false"]);
    std::fs::write(dir.path().join("file"), "before\n").expect("fixture");
    git(&["add", "file"]);
    std::fs::write(dir.path().join("file"), "after\n").expect("fixture");
    std::fs::set_permissions(
        dir.path().join("file"),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("permissions");
    let expected =
        String::from_utf8(git(&["diff", "--no-ext-diff", "--no-textconv"])).expect("patch");
    assert!(!expected.contains("old mode"));
    let result = GitDiffTool::new(dir.path().to_path_buf())
        .execute(serde_json::json!({}))
        .await;
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("-before\n+after"),
        "{}",
        result.content
    );
    assert!(!result.content.contains("old mode"), "{}", result.content);
    assert!(!result.content.contains("new mode"), "{}", result.content);
}

#[cfg(unix)]
#[tokio::test]
async fn git_diff_uses_owner_execute_bit_like_git() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("workspace");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .output()
            .expect("Git fixture");
        assert!(out.status.success(), "{out:?}");
        out.stdout
    };
    git(&["init"]);
    git(&["config", "core.fileMode", "true"]);
    let path = dir.path().join("file");
    std::fs::write(&path, "before\n").expect("fixture");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("permissions");
    git(&["add", "file"]);
    std::fs::write(&path, "after\n").expect("fixture");
    for mode in [0o654, 0o645] {
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
            .expect("permissions");
        let host =
            String::from_utf8(git(&["diff", "--no-ext-diff", "--no-textconv"])).expect("patch");
        assert!(!host.contains("old mode"), "{host}");
        let result = GitDiffTool::new(dir.path().to_path_buf())
            .execute(serde_json::json!({}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("+after"), "{}", result.content);
        assert!(!result.content.contains("old mode"), "{}", result.content);
    }
}

#[tokio::test]
async fn git_diff_unstaged_submodule_head_and_dirty_content_match_git() {
    let root = tempfile::tempdir().expect("workspace");
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("Git fixture");
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).expect("text")
    };
    git(root.path(), &["init"]);
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("child");
    git(&child, &["init"]);
    git(&child, &["config", "user.name", "Test"]);
    git(&child, &["config", "user.email", "test@example.invalid"]);
    std::fs::write(child.join("file"), "first\n").expect("fixture");
    git(&child, &["add", "file"]);
    git(&child, &["commit", "-m", "first"]);
    let first = git(&child, &["rev-parse", "HEAD"]).trim().to_owned();
    std::fs::write(
        root.path().join(".gitmodules"),
        "[submodule \"module\"]\n path = module\n url = ./module\n",
    )
    .expect("modules");
    git(root.path(), &["add", ".gitmodules", "module"]);
    let tool = GitDiffTool::new(root.path().to_path_buf());
    let clean = tool.execute(serde_json::json!({})).await;
    assert!(!clean.is_error, "{}", clean.content);
    assert_eq!(clean.content, "no changes");
    std::fs::write(child.join("untracked"), "new\n").expect("fixture");
    let untracked_host = git(root.path(), &["diff"]);
    let untracked = tool.execute(serde_json::json!({})).await;
    assert!(!untracked.is_error, "{}", untracked.content);
    assert_eq!(
        untracked.content.contains("-dirty"),
        untracked_host.contains("-dirty")
    );
    git(
        root.path(),
        &["config", "submodule.module.ignore", "untracked"],
    );
    let ignored = tool.execute(serde_json::json!({})).await;
    assert!(!ignored.is_error, "{}", ignored.content);
    assert_eq!(ignored.content, "no changes");
    std::fs::remove_file(child.join("untracked")).expect("remove fixture");
    git(root.path(), &["config", "submodule.module.ignore", "none"]);
    for commit in [false, true] {
        std::fs::write(child.join("file"), "second\n").expect("fixture");
        if commit {
            git(&child, &["add", "file"]);
            git(&child, &["commit", "-m", "second"]);
        }
        if !commit {
            git(root.path(), &["config", "submodule.module.ignore", "dirty"]);
            assert!(git(root.path(), &["diff"]).is_empty());
            let ignored = tool.execute(serde_json::json!({})).await;
            assert!(!ignored.is_error, "{}", ignored.content);
            assert_eq!(ignored.content, "no changes");
            git(root.path(), &["config", "submodule.module.ignore", "none"]);
            git(root.path(), &["config", "diff.ignoreSubmodules", "all"]);
            let override_host = git(root.path(), &["diff"]);
            let override_result = tool.execute(serde_json::json!({})).await;
            assert!(!override_result.is_error, "{}", override_result.content);
            assert!(override_host.contains("-dirty"));
            assert!(
                override_result.content.contains("-dirty"),
                "{}",
                override_result.content
            );
            git(root.path(), &["config", "--unset", "diff.ignoreSubmodules"]);
            git(&child, &["add", "file"]);
            let staged_child = tool.execute(serde_json::json!({})).await;
            assert!(!staged_child.is_error, "{}", staged_child.content);
            assert!(
                staged_child.content.contains("-dirty"),
                "{}",
                staged_child.content
            );
        }
        let head = git(&child, &["rev-parse", "HEAD"]).trim().to_owned();
        let suffix = if commit { "" } else { "-dirty" };
        let expected = format!("+Subproject commit {head}{suffix}");
        let host = git(root.path(), &["diff", "--no-ext-diff", "--no-textconv"]);
        assert!(host.contains(&expected), "{host}");
        let result = tool.execute(serde_json::json!({})).await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains(&expected), "{}", result.content);
        assert!(
            result
                .content
                .contains(&format!("-Subproject commit {first}")),
            "{}",
            result.content
        );
    }
    // A leftover .gitmodules section must not turn a removed index gitlink
    // into an unstaged addition.
    git(root.path(), &["rm", "--cached", "-f", "module"]);
    assert!(git(root.path(), &["diff"]).is_empty());
    let removed = tool.execute(serde_json::json!({})).await;
    assert!(!removed.is_error, "{}", removed.content);
    assert_eq!(removed.content, "no changes");
}

#[tokio::test]
async fn git_diff_missing_and_uninitialized_submodule_match_git() {
    let root = tempfile::tempdir().expect("workspace");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).expect("text")
    };
    git(&["init"]);
    let oid = "1111111111111111111111111111111111111111";
    std::fs::write(
        root.path().join(".gitmodules"),
        "[submodule \"module\"]\n path = module\n url = ./module\n",
    )
    .expect("modules");
    git(&["add", ".gitmodules"]);
    git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        "160000",
        oid,
        "module",
    ]);
    let tool = GitDiffTool::new(root.path().to_path_buf());
    for empty_directory in [false, true] {
        if empty_directory {
            std::fs::create_dir(root.path().join("module")).expect("empty checkout");
        }
        let host = git(&["diff", "--no-ext-diff", "--no-textconv"]);
        let result = tool.execute(serde_json::json!({})).await;
        assert!(!result.is_error, "{}", result.content);
        if host.is_empty() {
            assert_eq!(result.content, "no changes");
        } else {
            assert!(host.contains("deleted file mode 160000"), "{host}");
            assert!(
                result.content.contains("deleted file mode 160000"),
                "{}",
                result.content
            );
            assert!(
                result
                    .content
                    .contains(&format!("-Subproject commit {oid}")),
                "{}",
                result.content
            );
        }
    }
}

#[tokio::test]
async fn git_diff_submodule_replaced_by_file_preserves_content() {
    let root = tempfile::tempdir().expect("workspace");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).expect("text")
    };
    git(&["init"]);
    std::fs::write(
        root.path().join(".gitmodules"),
        "[submodule \"module\"]\n path = module\n url = ./module\n",
    )
    .expect("modules");
    git(&["add", ".gitmodules"]);
    git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        "160000",
        "1111111111111111111111111111111111111111",
        "module",
    ]);
    std::fs::write(root.path().join("module"), "replacement\n").expect("file replacement");
    let tool = GitDiffTool::new(root.path().to_path_buf());
    let host = git(&["diff", "--no-ext-diff", "--no-textconv"]);
    assert!(host.contains("+replacement"), "{host}");
    let result = tool.execute(serde_json::json!({})).await;
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("+replacement"),
        "{}",
        result.content
    );
    assert!(
        result
            .content
            .contains("old mode 160000\nnew mode 100644\n"),
        "{}",
        result.content
    );
    git(&["add", "module"]);
    std::fs::write(root.path().join("module"), "edited replacement\n").expect("edit");
    let result = tool.execute(serde_json::json!({})).await;
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("-replacement\n+edited replacement"),
        "{}",
        result.content
    );
    assert!(
        !result.content.contains("Subproject commit"),
        "{}",
        result.content
    );
}

#[tokio::test]
async fn git_diff_retains_index_executable_mode_without_filemode_tracking() {
    let root = tempfile::tempdir().expect("workspace");
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).expect("text")
    };
    git(&["init"]);
    git(&["config", "core.fileMode", "false"]);
    std::fs::write(root.path().join("script"), "before\n").expect("file");
    git(&["add", "script"]);
    git(&["update-index", "--chmod=+x", "script"]);
    std::fs::write(root.path().join("script"), "after\n").expect("edit");
    let host = git(&["diff", "--no-ext-diff", "--no-textconv"]);
    assert!(!host.contains("old mode"), "{host}");
    let result = GitDiffTool::new(root.path().to_path_buf())
        .execute(serde_json::json!({}))
        .await;
    assert!(!result.is_error, "{}", result.content);
    assert!(
        result.content.contains("-before\n+after"),
        "{}",
        result.content
    );
    assert!(!result.content.contains("old mode"), "{}", result.content);
}

#[cfg(unix)]
#[tokio::test]
async fn git_diff_symlink_targets_preserve_index_and_worktree_separation() {
    let root = tempfile::tempdir().expect("fixture");
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).expect("patch")
    };
    git(&["init"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "user.email", "test@example.invalid"]);
    let link = root.path().join("link");
    std::os::unix::fs::symlink("original-target", &link).expect("initial link");
    git(&["add", "link"]);
    git(&["commit", "-m", "initial"]);
    std::fs::remove_file(&link).expect("remove initial");
    std::os::unix::fs::symlink("staged-target", &link).expect("staged link");
    git(&["add", "link"]);
    std::fs::remove_file(&link).expect("remove staged");
    std::os::unix::fs::symlink("worktree-target", &link).expect("worktree link");
    std::fs::write(root.path().join("worktree-target"), "private target body\n")
        .expect("target content");
    let tool = GitDiffTool::new(root.path().to_path_buf());
    for (staged, old, new, excluded) in [
        (true, "original-target", "staged-target", "worktree-target"),
        (false, "staged-target", "worktree-target", "original-target"),
    ] {
        let args = if staged {
            vec!["diff", "--cached"]
        } else {
            vec!["diff"]
        };
        let host = git(&args);
        let result = tool.execute(serde_json::json!({"staged": staged})).await;
        assert!(!result.is_error, "{}", result.content);
        for patch in [&host, &result.content] {
            assert!(patch.contains(&format!("-{old}\n")), "{patch}");
            assert!(patch.contains(&format!("+{new}\n")), "{patch}");
            assert!(patch.contains("\\ No newline at end of file"), "{patch}");
            assert!(!patch.contains(excluded), "{patch}");
            assert!(!patch.contains("private target body"), "{patch}");
        }
    }
}
