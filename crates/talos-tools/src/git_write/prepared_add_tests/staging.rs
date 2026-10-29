#[tokio::test]
async fn retained_add_refuses_changed_invocation_before_staging() {
    for changed in ["paths", "environment", "cwd", "program", "missing"] {
        let root = fixture();
        std::fs::write(root.path().join("other"), "must remain unstaged\n")
            .expect("other file");
        let mut plan = AddPlan::prepare(
            root.path(),
            &serde_json::json!({"paths": ["file"]}),
            ToolProvenance::Native,
        )
        .expect("plan");
        match changed {
            "paths" => plan.paths = vec!["other".into()],
            "environment" => {
                plan.environment.insert("GIT_CONFIG_NOSYSTEM".into(), "changed".into());
            }
            "cwd" => plan.repository.workdir = root.path().join("other-directory"),
            "program" => plan.program = root.path().join("replacement-git"),
            "missing" => plan.native_add_command = None,
            _ => unreachable!(),
        }
        let output = execute_approved(plan).await;
        assert!(output.result.is_error, "{changed}: {}", output.result.content);
        assert!(output.result.content.contains("Native Add command changed"),
            "{changed}: {}", output.result.content);
        assert_eq!(integration_git(root.path(), &["diff", "--cached", "--name-only"]), "");
    }
}

// Original selective-staging acceptance: preserve another path's staged blob
// even when its working copy has additional, deliberately unstaged changes.
#[tokio::test]
async fn prepared_add_preserves_unrelated_staged_and_unstaged_content() {
    let root = fixture();
    let git = |args: &[&str]| integration_git(root.path(), args);
    std::fs::write(root.path().join("second"), "base\n").expect("second base");
    std::fs::write(root.path().join("other"), "base\n").expect("base");
    git(&["add", "file", "second", "other"]);
    git(&["commit", "-m", "base"]);
    std::fs::write(root.path().join("other"), "already staged\n").expect("staged");
    git(&["add", "other"]);
    let retained_entry = git(&["ls-files", "--stage", "--", "other"]);
    std::fs::write(root.path().join("other"), "keep unstaged\n").expect("unstaged");
    std::fs::write(root.path().join("file"), "selected\n").expect("selected");
    std::fs::write(root.path().join("second"), "second selected\n").expect("second");

    let plan = AddPlan::prepare(
        root.path(),
        &serde_json::json!({"paths": ["file", "second"]}),
        ToolProvenance::Native,
    ).expect("two-path plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["show", ":file"]), "selected");
    assert_eq!(git(&["show", ":second"]), "second selected");
    assert_eq!(git(&["show", ":other"]), "already staged");
    assert_eq!(git(&["ls-files", "--stage", "--", "other"]), retained_entry);
    assert_eq!(
        std::fs::read_to_string(root.path().join("other")).expect("working copy"),
        "keep unstaged\n"
    );
    assert_eq!(git(&["diff", "--name-only"]), "other");
    assert_eq!(git(&["diff", "--cached", "--name-only"]), "file\nother\nsecond");
}
