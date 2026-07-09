    use super::*;

    #[test]
    fn git_push_permission_profile_has_execute_and_remote_network() {
        let tool = GitPushTool::new(PathBuf::from("/workspace"));
        let profile = tool.permission_profile(&serde_json::json!({
            "remote": "upstream",
            "branch": "main"
        }));

        assert_eq!(profile.len(), 2);
        assert_eq!(profile[0].nature, ToolNature::Execute);
        assert_eq!(profile[0].resource.as_deref(), Some("git"));
        assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Command));
        assert_eq!(profile[1].nature, ToolNature::Network);
        assert_eq!(profile[1].resource.as_deref(), Some("upstream"));
        assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Remote));
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
    async fn host_git_unavailable_returns_actionable_error() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let err = crate::git::git_write::run_host_git_with_program(
            "talos-host-git-that-should-not-exist",
            tempdir.path(),
            &["status"],
        )
        .await
        .expect_err("missing git executable should fail");

        assert_eq!(
            err.to_string(),
            "git error: git not installed. Install git or use read-only tools."
        );
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

        std::fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();
        run(&["add", "test.txt"]);
        run(&["commit", "-m", "initial"]);

        std::fs::write(dir.path().join("test.txt"), "line1\nmodified\nline3\n").unwrap();

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

        std::fs::write(dir.path().join("file.txt"), "original\n").unwrap();
        run(&["add", "file.txt"]);
        run(&["commit", "-m", "initial"]);

        // Stage a change, then add an additional unstaged change so the
        // status iterator picks up the file.
        std::fs::write(dir.path().join("file.txt"), "staged\n").unwrap();
        run(&["add", "file.txt"]);
        std::fs::write(dir.path().join("file.txt"), "staged\nunstaged\n").unwrap();

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

        let all_result = tool.execute(serde_json::json!({})).await;
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

        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.path().join("docs/readme.md"), "# README\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "initial"]);

        std::fs::write(
            dir.path().join("src/main.rs"),
            "fn main() { println!(); }\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("docs/readme.md"), "# README Updated\n").unwrap();

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
