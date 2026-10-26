use super::*;
use talos_core::tool::ToolPermissionFacet;

#[test]
fn auxiliary_resolution_is_closed_to_bound_git_sources() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"paths": ["file"]});
    let facets = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:auxiliary-path-resolution",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy().into_owned(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.join("attributes").to_string_lossy().into_owned(),
            ToolResourceKind::Path,
        ),
    ];
    let allowed = |tool: &str, facets: &[ToolPermissionFacet]| {
        compile_facets(
            &PermissionRequest::native(tool, facets, &input),
            Some(&root),
            true,
        )
        .is_ok()
    };
    for tool in [
        "git_add",
        "git_commit",
        "git_checkout",
        "git_push",
        "git_pull",
    ] {
        assert!(allowed(tool, &facets), "{tool}");
        for omitted in [1, 2] {
            let mut incomplete = facets.clone();
            incomplete.remove(omitted);
            assert!(!allowed(tool, &incomplete));
        }
    }
    for tool in ["read_file", "bash", "git_status"] {
        assert!(!allowed(tool, &facets));
    }
    for nature in [
        ToolNature::Write,
        ToolNature::Execute,
        ToolNature::Network,
        ToolNature::Internal,
    ] {
        let mut invalid = facets.clone();
        invalid[0].nature = nature;
        assert!(!allowed("git_add", &invalid));
    }
    for spelling in [
        " git:auxiliary-path-resolution",
        "git:auxiliary-path-resolution ",
        "\tgit:auxiliary-path-resolution\t",
    ] {
        for nature in [ToolNature::Read, ToolNature::Network, ToolNature::Internal] {
            let mut padded = facets.clone();
            padded[0].resource = Some(spelling.into());
            padded[0].nature = nature;
            assert!(!allowed("git_add", &padded));
        }
    }
    for index in [1, 2] {
        let mut relative = facets.clone();
        relative[index].resource = Some("relative".into());
        assert!(!allowed("git_add", &relative));
    }
    let identity = |tool: &str, facets: &[ToolPermissionFacet], input: &serde_json::Value| {
        compiled_identity(&PermissionRequest::native(tool, facets, input), Some(&root))
            .expect("compiled identity")
            .1
    };
    let original = identity("git_add", &facets, &input);
    assert_ne!(original, identity("git_commit", &facets, &input));
    assert_ne!(
        original,
        identity("git_add", &facets, &serde_json::json!({"paths": ["other"]}))
    );
    for index in [1, 2] {
        let mut changed = facets.clone();
        changed[index].resource = Some(root.join("other").to_string_lossy().into_owned());
        assert_ne!(original, identity("git_add", &changed, &input));
    }
    let mut expanded = facets.clone();
    expanded.push(ToolPermissionFacet::with_resource(
        ToolNature::Read,
        root.join("extra-source").to_string_lossy().into_owned(),
        ToolResourceKind::Path,
    ));
    assert_ne!(original, identity("git_add", &expanded, &input));
    let mut arbitrary = facets;
    arbitrary[0].resource = Some("git:arbitrary-resolution".into());
    assert!(!allowed("git_add", &arbitrary));
}

#[test]
fn pull_resolution_capability_requires_pull_metadata_and_bound_command() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote": "origin"});
    let facets = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:pull-worktree-resolution",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:pull-repository-metadata",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy().into_owned(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.join("repo/.git").to_string_lossy().into_owned(),
            ToolResourceKind::Path,
        ),
    ];
    let allowed = |tool: &str, facets: &[ToolPermissionFacet]| {
        compile_facets(
            &PermissionRequest::native(tool, facets, &input),
            Some(&root),
            true,
        )
        .is_ok()
    };
    assert!(allowed("git_pull", &facets));
    for tool in ["git_push", "git_checkout", "read_file"] {
        assert!(!allowed(tool, &facets));
    }
    for omitted in 1..facets.len() {
        let mut incomplete = facets.clone();
        incomplete.remove(omitted);
        assert!(!allowed("git_pull", &incomplete));
    }
    let mut wrong_nature = facets.clone();
    wrong_nature[0].nature = ToolNature::Write;
    assert!(!allowed("git_pull", &wrong_nature));
    let mut generic = facets;
    generic[0].resource = Some("git:arbitrary-read".into());
    assert!(!allowed("git_pull", &generic));
}

#[test]
fn closed_git_discovery_scope_cannot_become_generic_remote_read() {
    let input = serde_json::json!({"paths": ["file"]});
    let root = std::env::current_dir().expect("workspace");
    let command = root.join("git").to_string_lossy().into_owned();
    let mut facets = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:configuration-chain",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(ToolNature::Execute, command, ToolResourceKind::Command),
    ];
    for tool in [
        "git_add",
        "git_commit",
        "git_checkout",
        "git_push",
        "git_pull",
    ] {
        assert!(
            compile_facets(
                &PermissionRequest::native(tool, &facets, &input),
                Some(&root),
                true
            )
            .is_ok()
        );
    }
    assert!(
        compile_facets(
            &PermissionRequest::native("read_file", &facets, &input),
            Some(&root),
            true
        )
        .is_err()
    );
    assert!(
        compile_facets(
            &PermissionRequest::native(
                "git_commit",
                &facets,
                &serde_json::json!({"message":"commit"})
            ),
            Some(&root),
            true
        )
        .is_ok()
    );
    assert!(
        compile_facets(
            &PermissionRequest::native("git_push", &facets, &input),
            Some(&root),
            true
        )
        .is_ok()
    );
    assert!(
        compile_facets(
            &PermissionRequest::native("git_fetch", &facets, &input),
            Some(&root),
            true
        )
        .is_err()
    );
    assert!(
        compile_facets(
            &PermissionRequest::native("git_add", &facets[..1], &input),
            Some(&root),
            true
        )
        .is_err()
    );
    facets[0].resource = Some("arbitrary:file".into());
    assert!(
        compile_facets(
            &PermissionRequest::native("git_add", &facets, &input),
            Some(&root),
            true
        )
        .is_err()
    );
}

#[test]
fn receiver_reference_tree_requires_push_and_explicit_root() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote": "receiver"});
    let mut facets = vec![ToolPermissionFacet::with_resource(
        ToolNature::Read,
        "git:receiver-reference-tree",
        ToolResourceKind::Remote,
    )];
    let allowed = |facets: &[ToolPermissionFacet], tool| {
        compile_facets(
            &PermissionRequest::native(tool, facets, &input),
            Some(&root),
            true,
        )
        .is_ok()
    };
    assert!(!allowed(&facets, "git_push"));
    facets.push(ToolPermissionFacet::with_resource(
        ToolNature::Read,
        root.join("receiver/refs").to_string_lossy().into_owned(),
        ToolResourceKind::Path,
    ));
    assert!(allowed(&facets, "git_push"));
    assert!(!allowed(&facets, "git_add"));
    assert!(!allowed(&facets, "read_file"));
    facets[1].resource = Some(root.join("receiver/other").to_string_lossy().into_owned());
    assert!(!allowed(&facets, "git_push"));
}

#[test]
fn receiver_metadata_requires_push_command_and_matching_path_nature() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote":"receiver"});
    for nature in [ToolNature::Read, ToolNature::Write] {
        let mut facets = vec![ToolPermissionFacet::with_resource(
            nature,
            "git:receiver-metadata",
            ToolResourceKind::Remote,
        )];
        let allowed = |facets: &[ToolPermissionFacet], tool| {
            compile_facets(
                &PermissionRequest::native(tool, facets, &input),
                Some(&root),
                true,
            )
            .is_ok()
        };
        assert!(!allowed(&facets, "git_push"));
        facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy().into_owned(),
            ToolResourceKind::Command,
        ));
        assert!(!allowed(&facets, "git_push"));
        facets.push(ToolPermissionFacet::with_resource(
            nature,
            root.join("receiver").to_string_lossy().into_owned(),
            ToolResourceKind::Path,
        ));
        assert!(allowed(&facets, "git_push"));
        for tool in ["git_add", "git_commit", "git_pull", "write_file"] {
            assert!(!allowed(&facets, tool));
        }
        facets[2].nature = if nature == ToolNature::Read {
            ToolNature::Write
        } else {
            ToolNature::Read
        };
        assert!(!allowed(&facets, "git_push"));
    }
}

#[test]
fn accepted_pull_metadata_capability_requires_bound_pull_roots() {
    let root = std::env::current_dir().expect("workspace");
    let input = serde_json::json!({"remote":"pull"});
    let facets = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:pull-repository-metadata",
            ToolResourceKind::Remote,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            root.join("git").to_string_lossy().into_owned(),
            ToolResourceKind::Command,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Read,
            root.join("repo/.git").to_string_lossy().into_owned(),
            ToolResourceKind::Path,
        ),
    ];
    assert!(
        compile_facets(
            &PermissionRequest::native("git_pull", &facets, &input),
            Some(&root),
            true,
        )
        .is_ok()
    );
}
