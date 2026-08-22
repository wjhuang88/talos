use talos_core::tool::{
    ToolAuthorizationScope, ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind,
};
use talos_permission::{
    GrantScope, GrantSource, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionEngine, PermissionMode, PermissionRequest, PermissionSessionState,
};

#[test]
fn scoped_grant_api_is_usable_outside_the_crate() {
    let workspace = tempfile::tempdir().expect("tempdir");
    let target = workspace.path().join("sdk-output.txt");
    std::fs::write(&target, b"fixture").expect("write fixture");
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        workspace.path().to_path_buf(),
    ));
    let input = serde_json::json!({"path": target});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        target.display().to_string(),
        ToolResourceKind::Path,
    )];
    let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );

    assert_eq!(
        state
            .evaluate(&request, &context)
            .expect("evaluation")
            .decision(),
        PermissionDecision::Ask
    );
    let proposal = state
        .propose(&request, &context, GrantScope::Session)
        .expect("proposal");
    assert_eq!(proposal.scope(), GrantScope::Session);
    assert_eq!(proposal.preview().facets().len(), 1);
    state
        .approve_session(proposal, &request, &context, GrantSource::SdkHostApproval)
        .expect("Session approval");

    let pending = state
        .prepare_authorized(&request, &context)
        .expect("preparation")
        .expect("matching Session grant");
    let authorizations = state.admit(pending, &request, &context).expect("admission");
    assert!(
        authorizations
            .iter()
            .all(|authorization| authorization.scope() == ToolAuthorizationScope::Session)
    );
}
