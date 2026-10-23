use super::*;
use talos_core::tool::ToolPermissionFacet;

#[test]
fn bounded_pull_discovery_is_closed_and_requires_read_root_and_command() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote": "origin"});
    let allowed = |tool: &str, facets: &[ToolPermissionFacet]| {
        compile_facets(
            &PermissionRequest::native(tool, facets, &input),
            Some(&root),
            true,
        )
        .is_ok()
    };
    let facets = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:pull-bounded-discovery",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy().into_owned(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.join("repo").to_string_lossy().into_owned(),
            ToolResourceKind::Path,
        ),
    ];
    assert!(allowed("git_pull", &facets));
    for tool in ["git_push", "git_checkout", "git_commit", "read_file"] {
        assert!(!allowed(tool, &facets));
    }
    for omitted in [1, 2] {
        let mut candidate = facets.clone();
        candidate.remove(omitted);
        assert!(!allowed("git_pull", &candidate));
    }
    for index in [1, 2] {
        let mut candidate = facets.clone();
        candidate[index].resource = Some("relative".into());
        assert!(!allowed("git_pull", &candidate));
    }
    for nature in [
        ToolNature::Write,
        ToolNature::Execute,
        ToolNature::Network,
        ToolNature::Internal,
    ] {
        let mut candidate = facets.clone();
        candidate[0].nature = nature;
        assert!(!allowed("git_pull", &candidate));
    }
    for kind in [
        ToolResourceKind::Path,
        ToolResourceKind::Command,
        ToolResourceKind::Domain,
    ] {
        let mut candidate = facets.clone();
        candidate[0].resource_kind = Some(kind);
        assert!(!allowed("git_pull", &candidate));
    }
    let mut candidate = facets.clone();
    candidate[2].nature = ToolNature::Write;
    assert!(!allowed("git_pull", &candidate));
    candidate = facets;
    candidate[0].resource = Some(" git:pull-bounded-discovery ".into());
    assert!(!allowed("git_pull", &candidate));
}
