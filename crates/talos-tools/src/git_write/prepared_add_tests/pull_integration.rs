#[tokio::test]
async fn pull_integration_hook_frontier_requires_a_new_plan_proof() {
    let root = fixture();
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
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
    assert!(plan.phase == Phase::PullIntegrationHooks);
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
        !plan
            .facets
            .iter()
            .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
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
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
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
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}
