//! Production-library preparation and exact admission, including Windows Job execution.
#![cfg(all(any(unix, windows), feature = "git"))]

use talos_core::tool::{AgentTool, PreparedNormalStage, PreparedStageHistory, ToolProvenance};
use talos_permission::{
    GrantScope, PermissionContext, PermissionEngine, PermissionRequest, PermissionSessionState,
};
use talos_tools::git::GitAddTool;

fn git(root: &std::path::Path, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .expect("fixture Git");
    assert!(output.status.success(), "{output:?}");
    output.stdout
}

#[tokio::test]
async fn production_prepared_add_requires_proof_and_stages_exact_path() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.autocrlf", "false"]);
    std::fs::write(root.path().join("selected.txt"), "selected\n").expect("selected");
    std::fs::write(root.path().join("other.txt"), "other\n").expect("other");
    let input = serde_json::json!({"paths":["selected.txt"]});
    let tool = GitAddTool::new(root.path().to_owned());
    let before = std::fs::read(root.path().join(".git/index")).ok();
    let operation = tool
        .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
        .expect("production preparation")
        .expect("prepared operation");
    let output = operation.execute(Box::new(())).await;
    assert!(output.result.is_error, "forged authority must fail");
    assert_eq!(std::fs::read(root.path().join(".git/index")).ok(), before);
    let mut operation = tool
        .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
        .expect("fresh production preparation")
        .expect("prepared operation");
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        root.path().to_owned(),
    ));
    let context = PermissionContext::compatibility();
    let mut history = PreparedStageHistory::default();
    loop {
        let plan = operation.plan_identity().expect("identity");
        assert!(
            history.record_profile(plan, operation.permission_profile()),
            "bounded distinct stages"
        );
        let request = PermissionRequest::native("git_add", operation.permission_profile(), &input);
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("approve");
        let proof = state
            .try_admit_prepared_invocation(pending, &request, &context, plan)
            .expect("exact admission");
        match operation.execute_stage(Box::new(proof)).await {
            PreparedNormalStage::Continue(next) => operation = next,
            PreparedNormalStage::Complete(output) => {
                assert!(!output.result.is_error, "{}", output.result.content);
                break;
            }
        }
    }
    assert_eq!(
        git(root.path(), &["diff", "--cached", "--name-only"]),
        b"selected.txt\n"
    );
    assert_eq!(git(root.path(), &["show", ":selected.txt"]), b"selected\n");
    assert_eq!(
        std::fs::read(root.path().join("other.txt")).expect("other"),
        b"other\n"
    );
}
