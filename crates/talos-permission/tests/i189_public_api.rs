use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionDecision, PermissionEngine, PermissionMode,
    PermissionOutcome, PermissionRequest, PermissionRule,
};

#[test]
fn structured_permission_api_is_usable_outside_the_crate() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Ask,
    ));
    let facets = [ToolPermissionFacet::new(ToolNature::Write)];
    let input = serde_json::json!({"path": "output.txt"});
    let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );

    let report = engine.evaluate_request(&request, &context);

    assert_eq!(report.outcome(), PermissionOutcome::Ask);
    assert_eq!(report.decision(), PermissionDecision::Ask);
    assert_eq!(report.facets().len(), 1);
    assert_eq!(engine.rules().len(), 1);
    assert!(!engine.is_trusted_workspace());
}
