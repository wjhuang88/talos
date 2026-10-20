#[tokio::test]
async fn pull_interactive_rebase_scope_does_not_authorize_sequence_editor() {
    let root = fixture();
    let mut plan = integration_scope_fixture(root.path()).await;
    plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
        onto: "1".repeat(40),
        upstream: "2".repeat(40),
        mode: super::super::pull_state::Rebase::Interactive,
        autostash: false,
    });
    let before = std::fs::read(root.path().join(".git/HEAD")).expect("HEAD");
    let error = match plan.discover_pull_integration_hooks() {
        Ok(_) => panic!("worktree scope cannot grant editor execution"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("sequence editor execution authorization"));
    assert_eq!(std::fs::read(root.path().join(".git/HEAD")).expect("HEAD"), before);
}

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
    isolate_integration_fixture_filters(root);
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

fn isolate_integration_fixture_filters(root: &Path) {
    // These integration fixtures exercise repository-local configuration,
    // not optional host filter installations (e.g. a global Git LFS driver).
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["config", "--global", "--name-only", "--get-regexp", "^filter\\."])
        .output().expect("global filter names");
    assert!(output.status.success() || output.status.code() == Some(1));
    for key in String::from_utf8(output.stdout).expect("filter names").lines() {
        if [".clean", ".smudge", ".process"].iter().any(|suffix| key.ends_with(suffix))
            && !std::process::Command::new("git").current_dir(root)
                .args(["config", "--local", "--get", key]).output()
                .expect("local filter").status.success()
        {
            integration_git(root, &["config", key, ""]);
        }
    }
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
    assert!(output.result.content.contains("Pull integration command failed"), "{}", output.result.content);
    assert_eq!(std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("after"), before);
    assert!(!root.path().join(".git/index").exists());
    assert!(!root.path().join(".git/MERGE_HEAD").exists());
}

#[tokio::test]
async fn pull_integration_executes_admitted_fast_forward_with_owned_runner() {
    let root = fixture();
    let git = |args: &[&str]| integration_git(root.path(), args);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    let original = git(&["rev-parse", "HEAD"]);
    std::fs::write(root.path().join("file"), "fetched content").expect("target content");
    git(&["commit", "-am", "target"]);
    let target = git(&["rev-parse", "HEAD"]);
    git(&["reset", "--hard", &original]);
    let mut plan = integration_scope_fixture(root.path()).await;
    std::fs::write(root.path().join(".git/FETCH_HEAD"), format!("{target}\t\tfixture\n")).expect("fetched target");
    plan.pull_evidence = Some(super::super::pull_state::PullEvidence::capture(
        &plan.repository, None, true,
    ).expect("fresh target evidence"));
    plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
        heads: vec![target.clone()],
        fast_forward: super::super::pull_state::FastForward::Only,
        autostash: false,
    });
    let plan = plan.discover_pull_integration_hooks().expect("integration frontier");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await else {
        panic!("final integration stage");
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["rev-parse", "HEAD"]), target);
    assert_eq!(std::fs::read_to_string(root.path().join("file")).expect("integrated file"), "fetched content");
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

#[tokio::test]
async fn pull_rebase_instruction_format_requires_separate_todo_authorization() {
    for mode in [
        super::super::pull_state::Rebase::On,
        super::super::pull_state::Rebase::Merges,
    ] {
        let root = fixture();
        assert!(std::process::Command::new("git")
            .current_dir(root.path())
            .args(["config", "rebase.instructionFormat", "%s%nexec echo private-value > format-helper-ran"])
            .output().expect("configure format").status.success());
        let mut plan = integration_scope_fixture(root.path()).await;
        plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
            onto: "1".repeat(40), upstream: "2".repeat(40), mode, autostash: false,
        });
        let before = std::fs::read(root.path().join(".git/HEAD")).expect("HEAD");
        let error = match plan.discover_pull_integration_hooks() {
            Ok(_) => panic!("format-created exec cannot inherit worktree scope"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("executable-todo effect authorization"), "{error}");
        assert!(!error.contains("private-value"));
        assert!(!root.path().join("format-helper-ran").exists());
        assert_eq!(std::fs::read(root.path().join(".git/HEAD")).expect("HEAD"), before);
    }
}

#[tokio::test]
async fn pull_instruction_format_is_dormant_for_merge_and_empty_for_rebase() {
    for (format, rebase) in [("%s%nexec echo probe", false), ("", true)] {
        let root = fixture();
        assert!(std::process::Command::new("git")
            .current_dir(root.path())
            .args(["config", "rebase.instructionFormat", format])
            .output().expect("configure format").status.success());
        let mut plan = integration_scope_fixture(root.path()).await;
        if rebase {
            plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
                onto: "1".repeat(40), upstream: "2".repeat(40),
                mode: super::super::pull_state::Rebase::On, autostash: false,
            });
        }
        plan.discover_pull_integration_hooks().expect("inactive custom format");
    }
}

#[tokio::test]
async fn pull_integration_conflict_retains_user_state_and_reports_partial_effects() {
    let root = fixture();
    let git = |args: &[&str]| integration_git(root.path(), args);
    // The admitted merge needs its own committer identity; command-local
    // overrides used to construct history do not reach the prepared runner.
    git(&["config", "user.useConfigOnly", "true"]);
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    let base = git(&["rev-parse", "HEAD"]);
    std::fs::write(root.path().join("file"), "remote\n").expect("remote");
    git(&["commit", "-am", "remote"]);
    let target = git(&["rev-parse", "HEAD"]);
    git(&["reset", "--hard", &base]);
    std::fs::write(root.path().join("file"), "local\n").expect("local");
    git(&["commit", "-am", "local"]);
    let original = git(&["rev-parse", "HEAD"]);
    let mut plan = integration_scope_fixture(root.path()).await;
    std::fs::write(root.path().join(".git/FETCH_HEAD"), format!("{target}\t\tfixture\n")).expect("FETCH_HEAD");
    plan.pull_evidence = Some(super::super::pull_state::PullEvidence::capture(
        &plan.repository, None, true,
    ).expect("fresh evidence"));
    plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
        heads: vec![target.clone()],
        fast_forward: super::super::pull_state::FastForward::Allow,
        autostash: false,
    });
    let plan = plan.discover_pull_integration_hooks().expect("integration frontier");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await else {
        panic!("final integration stage");
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("conflicts or partial effects may remain"));
    assert_eq!(git(&["rev-parse", "HEAD"]), original);
    assert!(root.path().join(".git/MERGE_HEAD").exists(), "{}", output.result.content);
    assert_eq!(git(&["rev-parse", "MERGE_HEAD"]), target);
    assert!(!git(&["ls-files", "--unmerged"]).is_empty());
    assert!(std::fs::read_to_string(root.path().join("file")).expect("conflict").contains("<<<<<<<"));
}

fn integration_git(root: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid"])
        .args(args).output().expect("Git fixture");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("Git output").trim().to_owned()
}

#[tokio::test]
async fn pull_integration_attribute_programs_cannot_inherit_worktree_authority() {
    for driver in [false, true] {
        let root = fixture();
        let git = |args: &[&str]| integration_git(root.path(), args);
        if driver {
            std::fs::write(root.path().join(".gitattributes"), "file merge=probe\n")
                .expect("base attributes");
        }
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        let base = git(&["rev-parse", "HEAD"]);
        std::fs::write(root.path().join("file"), "remote\n").expect("remote");
        if !driver {
            std::fs::write(root.path().join(".gitattributes"), "file filter=probe\n")
                .expect("target-only attributes");
        }
        git(&["add", "."]);
        git(&["commit", "-m", "remote"]);
        let target = git(&["rev-parse", "HEAD"]);
        git(&["reset", "--hard", &base]);
        std::fs::write(root.path().join("file"), "local\n").expect("local");
        git(&["commit", "-am", "local"]);
        let local = git(&["rev-parse", "HEAD"]);
        let key = if driver { "merge.probe.driver" } else { "filter.probe.smudge" };
        let program = if driver {
            "printf executed > helper-ran"
        } else {
            "cat; printf executed > helper-ran"
        };
        git(&["config", key, program]);
        // Establish that native integration really selects this program;
        // target-only filter attributes are absent from the current worktree.
        let native = std::process::Command::new("git")
            .current_dir(root.path())
            .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                "merge", "--no-edit", &target])
            .output().expect("native merge");
        assert!(root.path().join("helper-ran").exists(),
            "driver={driver}: {}", String::from_utf8_lossy(&native.stderr));
        git(&["config", "--unset", key]);
        git(&["reset", "--hard", &local]);
        std::fs::remove_file(root.path().join("helper-ran")).expect("remove native sentinel");
        git(&["config", key, program]);
        let mut plan = integration_scope_fixture(root.path()).await;
        plan.pull_integration = Some(super::super::pull_state::Integration::Merge {
            heads: vec![target],
            fast_forward: super::super::pull_state::FastForward::Allow,
            autostash: false,
        });
        let error = match plan.discover_pull_integration_hooks() {
            Ok(_) => panic!("attribute program inherited worktree authority"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains(if driver { "custom merge driver" } else { "configured filter" }), "{error}");
        assert!(!error.contains(program));
        assert!(!root.path().join("helper-ran").exists());
        assert_eq!(git(&["rev-parse", "HEAD"]), local);
        assert!(!root.path().join(".git/MERGE_HEAD").exists());
    }
}
