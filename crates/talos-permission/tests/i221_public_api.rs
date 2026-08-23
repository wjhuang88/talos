use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
    PermissionMode, PermissionRequest, PermissionSessionState,
};

#[test]
fn invocation_transaction_api_is_usable_outside_the_crate() {
    let workspace = tempfile::tempdir().expect("workspace");
    let target = workspace.path().join("outside.txt");
    std::fs::write(&target, b"fixture").expect("fixture");
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        workspace.path().to_path_buf(),
    ));
    let target_text = target.display().to_string();
    let input = serde_json::json!({"path": target_text.clone()});
    let profile = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        target_text,
        ToolResourceKind::Path,
    )];
    let request = PermissionRequest::new("write", ToolProvenance::Native, &profile, &input);
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );

    assert!(matches!(
        state
            .begin_invocation(&request, &context)
            .expect("begin invocation"),
        PermissionInvocation::Ask { .. }
    ));
}
