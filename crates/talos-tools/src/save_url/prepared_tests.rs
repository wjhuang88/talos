use super::*;
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
    PermissionMode, PermissionSessionState,
};

fn admit(
    root: &std::path::Path,
    operation: &dyn PreparedNormalToolInvocation,
    input: &Value,
) -> Box<dyn std::any::Any + Send> {
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(root.to_owned()));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = PermissionRequest::native("save_url", operation.permission_profile(), input);
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
            .expect("admission"),
    )
}

#[tokio::test]
async fn download_only_captures_then_exact_write_can_be_restored() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("file");
    std::fs::write(&path, "original").expect("original");
    let (tool, server) = super::super::recovery_tests::tool_with_download(b"downloaded").await;
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture","destination":path,"retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    assert!(
        !operation
            .permission_profile()
            .iter()
            .any(|f| f.nature == ToolNature::Write)
    );
    let old_plan = operation.plan_identity().expect("plan").clone();
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("download stage");
    };
    server.await.expect("server");
    assert_eq!(std::fs::read(&path).expect("unchanged"), b"original");
    let evidence = operation
        .prepared_file_mutation_evidence()
        .expect("complete images");
    assert!(!evidence.belongs_to(&old_plan));
    assert_eq!(evidence.description().before, "original");
    assert_eq!(evidence.description().after.as_deref(), Some("downloaded"));
    assert!(
        !operation
            .permission_profile()
            .iter()
            .any(|f| f.nature == ToolNature::Network)
    );
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("write stage");
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(std::fs::read(&path).expect("downloaded"), b"downloaded");
    let id = output
        .result
        .content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt");
    let restored = tool
        .execute(serde_json::json!({"url":"ignored","destination":path,"recovery_id":id}))
        .await;
    assert!(!restored.is_error, "{}", restored.content);
    assert_eq!(std::fs::read(&path).expect("restored"), b"original");
}

#[tokio::test]
async fn changed_destination_is_not_written_after_download() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("file");
    std::fs::write(&path, "original").expect("original");
    let (tool, server) = super::super::recovery_tests::tool_with_download(b"downloaded").await;
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture","destination":path,"retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("download");
    };
    server.await.expect("server");
    std::fs::write(&path, "user change").expect("concurrent edit");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("write");
    };
    assert!(output.result.is_error);
    assert_eq!(std::fs::read(&path).expect("preserved"), b"user change");
}

#[tokio::test]
async fn binary_images_keep_human_route_and_can_still_be_written() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("file");
    std::fs::write(&path, b"\xfforiginal").expect("original");
    let (tool, server) = super::super::recovery_tests::tool_with_download(b"\xffdownloaded").await;
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture","destination":path,"retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("download");
    };
    server.await.expect("server");
    assert!(operation.prepared_file_mutation_evidence().is_none());
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("write");
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(std::fs::read(&path).expect("binary"), b"\xffdownloaded");
}

#[tokio::test]
async fn arbitrary_or_other_plan_admission_has_no_effects() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("file");
    std::fs::write(&path, "original").expect("original");
    let tool = SaveUrlTool::new();
    let input = serde_json::json!({"url":"http://93.184.216.34/must-not-request","destination":path,"retain_backup":true});
    let other = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("other plan");
    for proof in [
        Box::new(()) as Box<dyn std::any::Any + Send>,
        admit(root.path(), other.as_ref(), &input),
    ] {
        let operation = prepare(&tool, &input, ToolProvenance::Native)
            .expect("prepare")
            .expect("operation");
        let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
            panic!("refusal");
        };
        assert!(output.result.is_error);
        assert!(output.result.content.contains("admission"));
        assert_eq!(std::fs::read(&path).expect("unchanged"), b"original");
    }
}

#[tokio::test]
async fn relative_destination_uses_admitted_workspace_not_process_cwd() {
    let root = tempfile::tempdir().expect("workspace");
    let name = format!("save-{}.txt", uuid::Uuid::new_v4());
    let path = root.path().join(&name);
    let ambient = std::env::current_dir().expect("cwd").join(&name);
    assert!(!ambient.exists());
    std::fs::write(&path, "approved original").expect("original");
    let (tool, server) = super::super::recovery_tests::tool_with_download(b"downloaded").await;
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture","destination":name,"retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Continue(operation) = operation.execute_stage(proof).await else {
        panic!("download");
    };
    server.await.expect("server");
    assert_eq!(
        operation
            .prepared_file_mutation_evidence()
            .expect("evidence")
            .description()
            .before,
        "approved original"
    );
    let proof = admit(root.path(), operation.as_ref(), &input);
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("write");
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(std::fs::read(&path).expect("approved file"), b"downloaded");
    assert!(!ambient.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn retargeted_input_link_cannot_redirect_capture_or_later_write() {
    let root = tempfile::tempdir().expect("workspace");
    let first = root.path().join("first");
    let second = root.path().join("second");
    let link = root.path().join("destination");
    std::fs::write(&first, "approved").expect("first");
    std::fs::write(&second, "unapproved").expect("second");
    std::os::unix::fs::symlink(&first, &link).expect("link");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("proxy");
    let tool = SaveUrlTool {
        client: reqwest::Client::builder()
            .no_proxy()
            .proxy(
                reqwest::Proxy::all(format!(
                    "http://{}",
                    listener.local_addr().expect("address")
                ))
                .expect("proxy"),
            )
            .build()
            .expect("client"),
        max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        recovery: Default::default(),
    };
    let input = serde_json::json!({"url":"http://93.184.216.34/fixture","destination":link,"retain_backup":true});
    let operation = prepare(&tool, &input, ToolProvenance::Native)
        .expect("prepare")
        .expect("operation");
    let proof = admit(root.path(), operation.as_ref(), &input);
    std::fs::remove_file(&link).expect("unlink");
    std::os::unix::fs::symlink(&second, &link).expect("retarget");
    let PreparedNormalStage::Complete(output) = operation.execute_stage(proof).await else {
        panic!("retargeted admission must fail before download");
    };
    assert!(output.result.is_error);
    assert!(
        output.result.content.contains("admission"),
        "{}",
        output.result.content
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&first).expect("first preserved"), b"approved");
    assert_eq!(
        std::fs::read(&second).expect("second preserved"),
        b"unapproved"
    );
}
