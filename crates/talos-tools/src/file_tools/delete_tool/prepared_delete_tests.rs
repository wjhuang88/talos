use super::*;
use std::path::Path;
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
    PermissionMode, PermissionSessionState,
};

fn admit(
    root: &Path,
    operation: &dyn PreparedNormalToolInvocation,
    input: &Value,
) -> Box<dyn std::any::Any + Send> {
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(root.to_owned()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = PermissionRequest::native("delete", operation.permission_profile(), input);
    let pending = match state
        .begin_invocation(&request, &context)
        .expect("evaluation")
    {
        PermissionInvocation::Allow(pending) => *pending,
        PermissionInvocation::Ask { once, .. } => state
            .approve_once(*once, &request, &context)
            .expect("approval"),
        PermissionInvocation::Deny(_) => panic!("denial"),
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
async fn captured_delete_preserves_changes_and_restores_successful_deletion() {
    for scenario in ["text", "binary", "nul", "changed", "missing_proof"] {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("file");
        let original: &[u8] = if scenario == "binary" {
            &[255, 0, 1]
        } else if scenario == "nul" {
            b"header\0payload"
        } else {
            b"original"
        };
        std::fs::write(&path, original).expect("write");
        let tool = DeleteTool::new(root.path().to_owned());
        let input = serde_json::json!({"path":"file","retain_backup":true});
        let operation = prepare(&tool, &input, ToolProvenance::Native)
            .expect("prepare")
            .expect("plan");
        assert!(operation.prepared_file_mutation_evidence().is_none());
        let initial_identity = operation.plan_identity().expect("identity").clone();
        let proof = admit(root.path(), operation.as_ref(), &input);
        let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
            panic!("capture")
        };
        assert_eq!(std::fs::read(&path).expect("unchanged"), original);
        if matches!(scenario, "binary" | "nul") {
            assert!(operation.prepared_file_mutation_evidence().is_none());
        } else {
            let evidence = operation
                .prepared_file_mutation_evidence()
                .expect("text evidence");
            assert!(!evidence.belongs_to(&initial_identity));
            assert_eq!(evidence.description().before, "original");
            assert!(evidence.description().after.is_none());
        }
        if scenario == "changed" {
            std::fs::write(&path, b"concurrent update").expect("change");
        }
        let proof = if scenario == "missing_proof" {
            Box::new(()) as Box<dyn std::any::Any + Send>
        } else {
            admit(root.path(), operation.as_ref(), &input)
        };
        let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
            panic!("deletion")
        };
        if scenario == "changed" || scenario == "missing_proof" {
            assert!(output.result.is_error);
            assert_eq!(
                std::fs::read(&path).expect("preserved"),
                if scenario == "changed" {
                    b"concurrent update".as_slice()
                } else {
                    original
                }
            );
        } else {
            assert!(!output.result.is_error, "{}", output.result.content);
            assert!(!path.exists());
            let id = output
                .result
                .content
                .split("recovery_id: ")
                .nth(1)
                .expect("receipt")
                .split(';')
                .next()
                .expect("id");
            let restored = tool
                .execute(serde_json::json!({"path":"file","recovery_id":id}))
                .await;
            assert!(!restored.is_error, "{}", restored.content);
            assert_eq!(std::fs::read(&path).expect("restored"), original);
        }
    }
}
