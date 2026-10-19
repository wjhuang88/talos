#[tokio::test]
async fn pull_child_completion_returns_hook_frontier_without_computation_only_approval() {
    let root = fixture();
    let mut plan = integration_scope_fixture(root.path()).await;
    plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
        heads: vec!["1".repeat(40)],
        fast_forward: super::super::pull_state::FastForward::Allow,
        autostash: false,
    });
    // Resume after the selected-child configuration frontier has converged;
    // no pending child fetch remains in this fixture.
    plan.pull_config_sources_discovered = true;
    plan.phase = Phase::PullChildConfiguration;
    for capability in ["git:configuration-chain", "git:pull-repository-metadata"] {
        plan.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read,
            capability,
            ToolResourceKind::Remote,
        ));
    }
    let admitted = proof(&plan);
    let stale = proof(&plan);
    let hook = plan.repository.common_dir.join("hooks/pre-merge-commit");
    let PreparedNormalStage::Continue(next) = plan.execute_stage(Box::new(admitted)).await else {
        panic!("child completion must return the hook frontier")
    };
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read && facet.resource.as_deref() == hook.to_str()
    }));
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Write
            && facet.resource.as_deref() == Some("git:pull-integration-worktree")
    }));
    assert!(!next.permission_profile().iter().any(|facet| facet.nature == ToolNature::Network));
    let PreparedNormalStage::Complete(output) = next.execute_stage(Box::new(stale)).await else {
        panic!("child proof must not authorize hook reads")
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("does not authorize this plan"));
}

#[tokio::test]
async fn pull_integration_hook_frontier_requires_a_new_plan_proof() {
    let root = fixture();
    let mut plan = integration_scope_fixture(root.path()).await;
    plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
        heads: vec!["1".repeat(40)],
        fast_forward: super::super::pull_state::FastForward::Allow,
        autostash: false,
    });
    plan.phase = Phase::PullIntegration;
    let old_proof = proof(&plan);
    let plan = plan
        .discover_pull_integration_hooks()
        .expect("hook frontier");
    assert!(plan.phase == Phase::PullIntegrationMutation);
    assert!(
        plan.pull_hook_paths
            .iter()
            .any(|path| path.ends_with("pre-merge-commit"))
    );
    assert!(
        plan.pull_hook_paths
            .iter()
            .any(|path| path.ends_with("post-merge"))
    );
    assert!(
        plan.facets.iter().any(|facet| facet.nature == ToolNature::Write
            && facet.resource.as_deref() == Some("git:pull-integration-worktree"))
    );
    assert!(
        plan.pull_hook_evidence.is_empty(),
        "discovery must not open candidates"
    );
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(old_proof)).await
    else {
        panic!("old admission continued")
    };
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("does not authorize this plan")
    );
}

#[tokio::test]
async fn pull_integration_hook_discovery_does_not_read_external_candidates() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("external hooks");
    let status = std::process::Command::new("git")
        .current_dir(root.path())
        .args(["config", "core.hooksPath"])
        .arg(outside.path())
        .status()
        .expect("config");
    assert!(status.success());
    let mut plan = integration_scope_fixture(root.path()).await;
    // A directory cannot be captured as regular hook evidence. Discovery must
    // still succeed: this candidate has not received its own Read admission.
    std::fs::create_dir(outside.path().join("pre-rebase")).expect("candidate");
    plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
        onto: "1".repeat(40),
        upstream: "2".repeat(40),
        mode: super::super::pull_state::Rebase::On,
        autostash: false,
    });
    let mut plan = plan
        .discover_pull_integration_hooks()
        .expect("text-only discovery");
    assert!(
        plan.pull_hook_paths
            .iter()
            .all(|path| path.starts_with(outside.path()))
    );
    assert!(plan.pull_hook_evidence.is_empty());
    let error = plan
        .capture_pull_integration_hooks(&[])
        .expect_err("missing Read");
    assert!(error.to_string().contains("fresh Read admission"));
    assert!(plan.pull_hook_evidence.is_empty());
    let admitted = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admitted)).await else {
        panic!("nonregular hook continued")
    };
    assert!(output.result.is_error);
    assert!(
        output.result.content.contains("regular file"),
        "{}",
        output.result.content
    );
    assert!(!root.path().join(".git/index").exists());
}

async fn integration_scope_fixture(root: &Path) -> Box<AddPlan> {
    std::fs::write(
        root.join(".git/FETCH_HEAD"),
        format!("{}\t\thttps://private.invalid/DO_NOT_DISCLOSE\n", "1".repeat(40)),
    ).expect("FETCH_HEAD");
    let plan = AddPlan::prepare_pull(root, &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
        heads: vec!["1".repeat(40)],
        fast_forward: super::super::pull_state::FastForward::Allow,
        autostash: false,
    });
    plan.pull_evidence = Some(super::super::pull_state::PullEvidence::capture(
        &plan.repository, None, true,
    ).expect("retained evidence"));
    plan
}

#[tokio::test]
async fn pull_integration_scope_binds_new_proof_and_hides_fetch_descriptions() {
    let root = fixture();
    let plan = integration_scope_fixture(root.path()).await;
    let stale = proof(&plan);
    let plan = plan.prepare_pull_integration_scope().expect("scope");
    assert!(plan.phase == Phase::PullIntegrationMutation);
    assert!(stale.consume_for_prepared(&plan.request(), &plan.identity).is_err());
    for nature in [ToolNature::Read, ToolNature::Write] {
        let scope = plan.facets.iter().find(|facet| facet.nature == nature
            && facet.resource.as_deref() == Some("git:pull-integration-worktree"))
            .expect("scope");
        let description = scope.description.as_deref().expect("description");
        assert!(description.contains(&"1".repeat(40)));
        assert!(description.contains("FETCH_HEAD sha256="));
        assert!(!description.contains("DO_NOT_DISCLOSE"));
        assert!(!description.contains("private.invalid"));
    }
    assert!(plan.validate_pull_integration_scope(&[]).is_err());
    let admission = proof(&plan);
    let tokens = admission.consume_for_prepared(&plan.request(), &plan.identity).expect("proof");
    plan.validate_pull_integration_scope(&tokens).expect("bound roots");
    std::fs::write(root.path().join(".git/FETCH_HEAD"), b"changed").expect("drift");
    assert!(plan.validate_pull_integration_scope(&tokens).is_err());
}

#[tokio::test]
async fn pull_integration_scope_refuses_missing_capability_even_with_root_tokens() {
    let root = fixture();
    let mut plan = integration_scope_fixture(root.path()).await
        .prepare_pull_integration_scope().expect("scope");
    plan.facets.retain(|facet| !(facet.nature == ToolNature::Write
        && facet.resource.as_deref() == Some("git:pull-integration-worktree")));
    let admission = proof(&plan);
    let tokens = admission.consume_for_prepared(&plan.request(), &plan.identity).expect("proof");
    assert!(plan.validate_pull_integration_scope(&tokens).expect_err("missing scope")
        .to_string().contains("capability missing"));
}

#[tokio::test]
async fn pull_integration_scope_runs_through_admitted_continuations_without_mutation() {
    let root = fixture();
    let plan = integration_scope_fixture(root.path()).await
        .discover_pull_integration_hooks().expect("hook frontier");
    let before = std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("before");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await else {
        panic!("hook reads and integration scope must consume one stage, not another continuation")
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("Pull integration scope admitted"), "{}", output.result.content);
    assert_eq!(std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("after"), before);
    assert!(!root.path().join(".git/index").exists());
    assert!(!root.path().join(".git/MERGE_HEAD").exists());
}

#[tokio::test]
async fn pull_integration_scope_does_not_replace_exact_hook_read_admission() {
    let root = fixture();
    let mut plan = integration_scope_fixture(root.path())
        .await
        .discover_pull_integration_hooks()
        .expect("combined frontier");
    let hooks: Vec<_> = plan
        .pull_hook_paths
        .iter()
        .map(|path| path.to_str().expect("path").to_owned())
        .collect();
    plan.facets.retain(|facet| {
        !(facet.nature == ToolNature::Read
            && facet
                .resource
                .as_ref()
                .is_some_and(|resource| hooks.contains(resource)))
    });
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("worktree scope must not grant implicit hook reads")
    };
    assert!(output.result.is_error);
    assert!(
        output
            .result
            .content
            .contains("hook frontier lacks fresh Read admission"),
        "{}",
        output.result.content
    );
    assert!(!root.path().join(".git/index").exists());
}
