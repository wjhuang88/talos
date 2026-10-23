//! Real prepared-stage convergence for contained native discovery.

use super::*;

#[tokio::test]
async fn native_pull_external_startup_worktree_requires_fresh_read() {
    let root = tempfile::tempdir().expect("fixture");
    let external = tempfile::tempdir().expect("external worktree");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/parent"],
    );
    let module = root.path().join(".git/modules/child");
    std::fs::create_dir_all(&module).expect("module");
    git(&module, &["init", "--bare", "-b", "main"]);
    git(&module, &["config", "core.bare", "false"]);
    git(
        &module,
        &[
            "config",
            "core.worktree",
            external.path().to_str().expect("path"),
        ],
    );
    git(&module, &["config", "core.hooksPath", ""]);
    git(
        &module,
        &["remote", "add", "origin", "git://example.invalid/child"],
    );
    let mut plan =
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan");
    plan.phase = Phase::PullNativePreparation;
    let admitted = proof(&plan);
    let next = match Box::new(plan).execute_stage(Box::new(admitted)).await {
        PreparedNormalStage::Continue(next) => next,
        PreparedNormalStage::Complete(output) => panic!("discovery stopped: {:?}", output.result),
    };
    let target = external.path().canonicalize().expect("actual target");
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read && facet.resource.as_deref() == target.to_str()
    }));
    assert!(
        !next
            .permission_profile()
            .iter()
            .any(|facet| { matches!(facet.nature, ToolNature::Write | ToolNature::Network) })
    );
    assert!(!module.join("FETCH_HEAD").exists());
}

#[tokio::test]
async fn native_pull_contained_modules_use_one_discovery_profile() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/parent"],
    );
    let mut modules = Vec::new();
    for name in ["group/child", "group/child/modules/nested"] {
        let module = root.path().join(".git/modules").join(name);
        std::fs::create_dir_all(&module).expect("module");
        git(&module, &["init", "--bare", "-b", "main"]);
        git(&module, &["config", "core.hooksPath", ""]);
        git(
            &module,
            &["remote", "add", "origin", "git://example.invalid/child"],
        );
        modules.push(module);
    }
    // An initialized child's startup directory is part of the displayed
    // parent worktree scope after bounded pointer resolution.
    let child_worktree = root.path().join("child");
    std::fs::create_dir(&child_worktree).expect("child worktree");
    git(&modules[0], &["config", "core.bare", "false"]);
    git(
        &modules[0],
        &[
            "config",
            "core.worktree",
            child_worktree.to_str().expect("path"),
        ],
    );
    let mut plan =
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan");
    // Native routing remains gated; enter the real native dispatcher directly.
    plan.phase = Phase::PullNativePreparation;
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    assert!(
        !plan
            .facets
            .iter()
            .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
    );
    let stale = proof(&plan);
    let admitted = proof(&plan);
    let next = match Box::new(plan).execute_stage(Box::new(admitted)).await {
        PreparedNormalStage::Continue(next) => next,
        PreparedNormalStage::Complete(output) => panic!("discovery stopped: {:?}", output.result),
    };
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Write),
        "contained discovery required another approval: {:?}",
        next.permission_profile()
    );
    assert!(history.record_profile(
        next.plan_identity().expect("final identity"),
        next.permission_profile()
    ));
    for module in &modules {
        assert!(!module.join("FETCH_HEAD").exists());
        assert!(
            next.permission_profile()
                .iter()
                .any(|facet| facet.nature == ToolNature::Write
                    && facet.resource.as_deref()
                        == module.canonicalize().expect("module").to_str())
        );
    }
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Write
            && facet.resource.as_deref()
                == child_worktree
                    .canonicalize()
                    .expect("child target")
                    .to_str()
    }));
    let PreparedNormalStage::Complete(denied) = next.execute_stage(Box::new(stale)).await else {
        panic!("stale discovery proof continued");
    };
    assert!(denied.result.is_error);
    assert!(
        denied
            .result
            .content
            .contains("does not authorize this plan")
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn native_pull_external_parent_hooks_publish_read_frontier_before_capture() {
    let root = tempfile::tempdir().expect("fixture");
    let external = tempfile::tempdir().expect("external hooks");
    git(root.path(), &["init", "-b", "main"]);
    let hooks = external.path().canonicalize().expect("hook root");
    git(
        root.path(),
        &["config", "core.hooksPath", hooks.to_str().expect("path")],
    );
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/parent"],
    );
    // An unread nonregular candidate must not be captured before the external
    // frontier receives fresh exact admission.
    std::fs::create_dir(hooks.join("post-merge")).expect("unread candidate");
    let mut plan =
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan");
    plan.phase = Phase::PullNativePreparation;
    let admitted = proof(&plan);
    let tokens = admitted
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("initial proof");
    let plan = Box::new(plan)
        .discover_native_pull_scope(&tokens)
        .await
        .expect("external frontier");
    assert!(plan.phase == Phase::PullNativePreparation);
    assert!(
        !plan
            .facets
            .iter()
            .any(|facet| matches!(facet.nature, ToolNature::Write | ToolNature::Network))
    );
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == hooks.join("post-merge").to_str())
    );
    assert!(
        plan.auxiliary_paths
            .as_ref()
            .expect("resolver")
            .leaf_evidence(&hooks.join("post-merge"))
            .is_none()
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}
