use super::*;
use talos_core::tool::ToolPermissionFacet;

#[test]
fn integration_scope_requires_pull_command_and_matching_absolute_root() {
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
    for nature in [ToolNature::Read, ToolNature::Write] {
        let facets = vec![
            ToolPermissionFacet::with_resource(
                nature,
                "git:pull-integration-worktree",
                ToolResourceKind::Remote,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                root.join("git").to_string_lossy().into_owned(),
                ToolResourceKind::Command,
            ),
            ToolPermissionFacet::with_resource(
                nature,
                root.join("repo").to_string_lossy().into_owned(),
                ToolResourceKind::Path,
            ),
        ];
        assert!(allowed("git_pull", &facets));
        for tool in ["git_push", "git_checkout", "git_commit", "write_file"] {
            assert!(!allowed(tool, &facets));
        }
        for omitted in [1, 2] {
            let mut incomplete = facets.clone();
            incomplete.remove(omitted);
            assert!(!allowed("git_pull", &incomplete));
        }
        for index in [1, 2] {
            let mut relative = facets.clone();
            relative[index].resource = Some("relative".into());
            assert!(!allowed("git_pull", &relative));
        }
        let mut mismatch = facets.clone();
        mismatch[2].nature = if nature == ToolNature::Read {
            ToolNature::Write
        } else {
            ToolNature::Read
        };
        assert!(!allowed("git_pull", &mismatch));
        let mut arbitrary = facets.clone();
        arbitrary[0].resource = Some("git:arbitrary-worktree".into());
        assert!(!allowed("git_pull", &arbitrary));
        let mut execute = facets;
        execute[0].nature = ToolNature::Execute;
        assert!(!allowed("git_pull", &execute));
    }
}
