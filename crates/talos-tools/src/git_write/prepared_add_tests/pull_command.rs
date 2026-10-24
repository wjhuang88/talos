use super::native_pull::git;
use super::*;

fn repositories() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let local = root.path().join("local");
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    std::fs::write(source.join("file"), "base\n").expect("file");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "base"]);
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("source"),
            local.to_str().expect("local"),
        ],
    );
    git(&local, &["config", "pull.rebase", "false"]);
    git(&local, &["config", "user.useConfigOnly", "true"]);
    git(&local, &["config", "user.name", "Native Pull fixture"]);
    git(&local, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::write(source.join("file"), "remote\n").expect("file");
    git(&source, &["commit", "-am", "remote"]);
    (root, source, local)
}

#[tokio::test]
async fn native_command_discovery_then_one_mutation_preserves_configured_hook() {
    let (_root, source, local) = repositories();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hook = local.join(".git/hooks/post-merge");
        std::fs::write(&hook, "#!/bin/sh\nprintf executed > command-hook-ran\n").expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
        git(&local, &["config", "core.hooksPath", ".git/hooks"]);
    }
    let plan =
        prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native).expect("plan");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let admission = proof(&plan);
    let PreparedNormalStage::Continue(plan) =
        Box::new(plan).execute_stage(Box::new(admission)).await
    else {
        panic!("configuration discovery must return command plan")
    };
    assert!(!local.join("command-hook-ran").exists());
    assert_eq!(
        std::fs::read_to_string(local.join("file")).expect("before"),
        "base\n"
    );
    let profile = plan.permission_profile();
    assert!(
        profile
            .iter()
            .any(|facet| facet.resource.as_deref() == Some("git:pull-command"))
    );
    assert!(history.record_profile(plan.plan_identity().expect("identity"), profile));
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(local.clone()));
    let input = serde_json::json!({});
    let request = PermissionRequest::native("git_pull", profile, &input);
    let context = PermissionContext::compatibility();
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approve");
    let admission = state
        .try_admit_prepared_invocation(
            pending,
            &request,
            &context,
            plan.plan_identity().expect("identity"),
        )
        .expect("admit");
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("one final native command must complete")
    };
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(
        git(&local, &["rev-parse", "HEAD"]),
        git(&source, &["rev-parse", "HEAD"])
    );
    #[cfg(unix)]
    assert_eq!(
        std::fs::read_to_string(local.join("command-hook-ran")).expect("hook ran"),
        "executed"
    );
}

#[tokio::test]
async fn native_command_refuses_configuration_drift_before_fetch() {
    let (_root, _source, local) = repositories();
    let plan =
        prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native).expect("plan");
    let plan = Box::new(plan)
        .discover_pull_command()
        .await
        .expect("discovery");
    let admission = proof(&plan);
    git(&local, &["config", "pull.rebase", "true"]);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("drift must refuse")
    };
    assert!(output.result.is_error);
    assert!(!local.join(".git/FETCH_HEAD").exists());
    assert_eq!(
        std::fs::read_to_string(local.join("file")).expect("unchanged"),
        "base\n"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn native_command_cancellation_closes_configured_hook_descendants() {
    use std::os::unix::fs::PermissionsExt;
    let (_root, _source, local) = repositories();
    let hook = local.join(".git/hooks/post-merge");
    std::fs::write(&hook, "#!/bin/sh\n(while test ! -f release-effect; do sleep 0.01; done; printf forbidden > delayed-effect) &\nprintf observed > hook-started\nwait\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    git(&local, &["config", "core.hooksPath", ".git/hooks"]);
    let plan =
        prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native).expect("plan");
    let plan = Box::new(plan)
        .discover_pull_command()
        .await
        .expect("discovery");
    let receipt = plan.cleanup_receipt().expect("cleanup receipt");
    let admission = proof(&plan);
    let task = tokio::spawn(async move {
        let _ = plan.execute_stage(Box::new(admission)).await;
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !local.join("hook-started").exists() {
            assert!(!task.is_finished(), "command must reach configured hook");
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("hook started");
    task.abort();
    assert!(task.await.expect_err("cancelled").is_cancelled());
    receipt
        .wait(std::time::Duration::from_secs(3))
        .await
        .expect("owned descendants closed");
    std::fs::write(local.join("release-effect"), "release\n").expect("release");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(!local.join("delayed-effect").exists());
    // Cancellation retains the completed integration preceding the hook.
    assert_eq!(
        std::fs::read_to_string(local.join("file")).expect("file"),
        "remote\n"
    );
}

#[tokio::test]
async fn native_command_conflict_preserves_partial_effects_without_repair() {
    let (_root, source, local) = repositories();
    std::fs::write(local.join("file"), "local\n").expect("local change");
    git(&local, &["commit", "-am", "local"]);
    let before = git(&local, &["rev-parse", "HEAD"]);
    let plan =
        prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native).expect("plan");
    let plan = Box::new(plan)
        .discover_pull_command()
        .await
        .expect("discovery");
    let admission = proof(&plan);
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
    else {
        panic!("native command must return its failure without a repair stage")
    };
    assert!(output.result.is_error);
    assert!(output.result.content.contains("partial effects may remain"));
    assert_eq!(git(&local, &["rev-parse", "HEAD"]), before);
    assert_eq!(
        git(&local, &["rev-parse", "FETCH_HEAD"]),
        git(&source, &["rev-parse", "HEAD"])
    );
    assert!(
        local.join(".git/MERGE_HEAD").is_file(),
        "native Pull must reach merge conflict: {}",
        output.result.content
    );
    assert!(!git(&local, &["ls-files", "--unmerged"]).is_empty());
    assert!(
        std::fs::read_to_string(local.join("file"))
            .expect("conflict")
            .contains("<<<<<<<")
    );
}

#[tokio::test]
async fn native_command_preview_withholds_config_credentials_and_helper_payloads() {
    let (_root, _source, local) = repositories();
    git(
        &local,
        &[
            "config",
            "remote.origin.url",
            "https://private-user:private-password@example.invalid:8443/private-path?token=private-token",
        ],
    );
    git(
        &local,
        &["config", "credential.helper", "!echo private-helper-secret"],
    );
    git(
        &local,
        &[
            "config",
            "filter.private-filter.clean",
            "echo private-filter-secret",
        ],
    );
    let plan =
        prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native).expect("plan");
    let plan = Box::new(plan)
        .discover_pull_command()
        .await
        .expect("discovery");
    let context = PermissionContext::compatibility();
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(local.clone()));
    let proposal = state
        .propose(&plan.request(), &context, GrantScope::Once)
        .expect("proposal");
    let notes = proposal.preview().notes().join("\n");
    for secret in [
        "private-user",
        "private-password",
        "private-path",
        "private-token",
        "private-helper-secret",
        "private-filter-secret",
        "private-filter",
    ] {
        assert!(!notes.contains(secret), "preview disclosed {secret}");
    }
    assert!(notes.contains("example.invalid:8443"));
    assert!(notes.contains("credentials"));
    assert!(notes.contains("filters"));
    assert!(!local.join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn native_command_refuses_frozen_invocation_substitution_before_fetch() {
    for changed in ["argv", "environment", "cwd", "program"] {
        let (_root, source, local) = repositories();
        let plan = prepare_pull_fixture(&local, &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan");
        let mut plan = Box::new(plan)
            .discover_pull_command()
            .await
            .expect("discovery");
        let admission = proof(&plan);
        let mut environment = plan.environment.clone();
        if changed == "environment" {
            environment.insert("GIT_CONFIG_COUNT".into(), "0".into());
            environment.insert("TALOS_FROZEN_TEST".into(), "changed".into());
        }
        let program = if changed == "program" {
            source.join("unapproved-git")
        } else {
            plan.program.clone()
        };
        let cwd = if changed == "cwd" {
            &source
        } else {
            &plan.repository.workdir
        };
        let arguments = if changed == "argv" {
            vec!["pull", "--", "unapproved-remote"]
        } else {
            vec![
                "-c",
                "maintenance.autoDetach=false",
                "-c",
                "gc.autoDetach=false",
                "--no-pager",
                "pull",
            ]
        };
        plan.native_pull_command = Some(
            command::FrozenCommand::new(&program, cwd, arguments, &environment)
                .expect("substituted invocation"),
        );
        let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("changed invocation must refuse without another stage")
        };
        assert!(output.result.is_error, "{changed}");
        assert!(
            output
                .result
                .content
                .contains("command changed after preparation"),
            "{changed}: {}",
            output.result.content
        );
        assert!(!local.join(".git/FETCH_HEAD").exists(), "{changed}");
        assert_eq!(
            std::fs::read_to_string(local.join("file")).expect("file"),
            "base\n"
        );
    }
}
