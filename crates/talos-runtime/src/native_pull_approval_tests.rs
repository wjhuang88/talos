use super::*;
use talos_core::tool::{ToolNature, ToolResourceKind};

struct LegacyHost;

#[async_trait]
impl ApprovalHandler for LegacyHost {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("argument-only approval must not receive command authority")
    }
}

#[tokio::test]
async fn legacy_host_cannot_approve_native_pull_without_its_scope_notes() {
    let root = tempfile::tempdir().expect("workspace");
    let facets = [
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.path().to_string_lossy(),
            ToolResourceKind::Path,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.path().join("git").to_string_lossy(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "git:pull-command",
            ToolResourceKind::Remote,
        )
        .with_description("native Pull and transitive effects; not per-path confinement"),
    ];
    let input = serde_json::json!({});
    let request = PermissionRequest::native("git_pull", &facets, &input);
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        root.path().to_path_buf(),
    ));
    let proposal = state
        .propose(
            &request,
            &PermissionContext::compatibility(),
            GrantScope::Once,
        )
        .expect("proposal");
    assert!(!proposal.preview().notes().is_empty());
    assert_eq!(
        LegacyHost
            .request_scoped_approval("git_pull", &input, &[], proposal.preview())
            .await,
        ApprovalChoice::Deny
    );
}
