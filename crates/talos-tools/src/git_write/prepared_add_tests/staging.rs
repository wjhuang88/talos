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
