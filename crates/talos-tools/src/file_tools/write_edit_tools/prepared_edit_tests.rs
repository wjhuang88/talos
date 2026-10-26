use super::*;
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
    PermissionMode, PermissionSessionState,
};

pub(super) fn admit(
    root: &Path,
    operation: &dyn PreparedNormalToolInvocation,
    input: &Value,
) -> Box<dyn std::any::Any + Send> {
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(root.to_owned()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = PermissionRequest::native("edit", operation.permission_profile(), input);
    let pending = match state
        .begin_invocation(&request, &context)
        .expect("evaluation")
    {
        PermissionInvocation::Allow(pending) => *pending,
        PermissionInvocation::Ask { once, .. } => state
            .approve_once(*once, &request, &context)
            .expect("approval"),
        PermissionInvocation::Deny(_) => panic!("unexpected denial"),
    };
    Box::new(
        state
            .try_admit_prepared_invocation(
                pending,
                &request,
                &context,
                operation.plan_identity().expect("plan"),
            )
            .expect("admitted"),
    )
}

#[tokio::test]
async fn captured_edit_executes_and_restores_through_owning_tool() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("greeting.txt");
    std::fs::write(&path, "hi hi").expect("original");
    let tool = EditTool::new(root.path().to_owned());
    let input = serde_json::json!({"path":"greeting.txt","old_string":"hi","new_string":"hello","retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    assert!(operation.prepared_file_mutation_evidence().is_none());
    let read_identity = operation.plan_identity().expect("read plan").clone();
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("capture stage");
    };
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), "hi hi");
    let evidence = operation
        .prepared_file_mutation_evidence()
        .expect("captured evidence");
    assert!(!evidence.belongs_to(&read_identity));
    assert_eq!(evidence.description().before, "hi hi");
    assert_eq!(evidence.description().after.as_deref(), Some("hello hi"));
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("mutation stage");
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(std::fs::read_to_string(&path).expect("edited"), "hello hi");
    let id = output
        .result
        .content
        .lines()
        .find_map(|line| line.strip_prefix("recovery_id: "))
        .expect("receipt");
    let restored = tool
        .execute(serde_json::json!({"path":"greeting.txt","recovery_id":id}))
        .await;
    assert!(!restored.is_error, "{}", restored.content);
    assert_eq!(std::fs::read_to_string(&path).expect("restored"), "hi hi");
}

#[tokio::test]
async fn oversized_result_is_refused_before_continuation_or_write() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("greeting.txt");
    std::fs::write(&path, "hi").expect("original");
    let tool = EditTool::new(root.path().to_owned());
    let input = serde_json::json!({"path":"greeting.txt","old_string":"hi",
        "new_string":"x".repeat(talos_core::tool::MAX_FILE_MUTATION_REVIEW_BYTES + 1),
        "retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("oversized result cannot be offered for review");
    };
    assert!(output.result.is_error);
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), "hi");
}

#[tokio::test]
async fn capture_requires_admission_and_changed_contents_are_never_overwritten() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("greeting.txt");
    std::fs::write(&path, "hi").expect("original");
    let tool = EditTool::new(root.path().to_owned());
    let input = serde_json::json!({"path":"greeting.txt","old_string":"hi","new_string":"hello","retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let PreparedNormalStage::Complete(output) = operation.execute_stage(Box::new(())).await else {
        panic!("must refuse");
    };
    assert!(output.result.is_error);
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("capture");
    };
    std::fs::write(&path, "someone else's update").expect("concurrent update");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("mutation");
    };
    assert!(output.result.is_error);
    assert_eq!(
        std::fs::read_to_string(&path).expect("preserved"),
        "someone else's update"
    );
}
