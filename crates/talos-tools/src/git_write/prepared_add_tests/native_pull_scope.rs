//! Scope discovery consumes fresh approvals before reading module frontiers.

use super::*;

#[tokio::test]
async fn native_pull_scope_admits_startup_before_discovering_configuration() {
    for stored in [false, true] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        git(root.path(), &["config", "core.hooksPath", ""]);
        git(
            root.path(),
            &["remote", "add", "origin", "git://example.invalid/source"],
        );
        let module = root
            .path()
            .canonicalize()
            .expect("canonical root")
            .join(".git/modules/group/child");
        if stored {
            std::fs::create_dir_all(&module).expect("stored module");
            git(&module, &["init", "--bare"]);
            std::fs::create_dir(root.path().join(".git/startup-worktree"))
                .expect("startup worktree");
            std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
            std::fs::write(
                module.join("child-include"),
                "[remote \"origin\"]\nurl = git://example.invalid/child\n",
            )
            .expect("child include");
            std::fs::write(module.join("config"),
                "[core]\nrepositoryformatversion = 0\nworktree = ../../../startup-worktree\n[include]\npath = child-include\n",
            ).expect("startup config");
        }
        let mut plan = Box::new(
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("sender configuration");
        plan.phase = Phase::PullNativePreparation;
        let stale = proof(&plan);
        let mut denied_plan =
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("denied plan");
        denied_plan.phase = Phase::PullNativePreparation;
        let PreparedNormalStage::Complete(denied) =
            Box::new(denied_plan).execute_stage(Box::new(())).await
        else {
            panic!("missing proof continued");
        };
        assert!(denied.result.is_error);
        let mut waves = 0;
        while plan.phase == Phase::PullNativePreparation {
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("fresh scope admission");
            plan = plan
                .discover_native_pull_scope(&tokens)
                .await
                .expect("native scope");
            waves += 1;
            assert!(waves < 10, "scope discovery did not converge");
            assert!(!root.path().join(".git/FETCH_HEAD").exists());
        }
        assert!(plan.phase == Phase::PullNativeMutation);
        assert!(
            stale
                .consume_for_prepared(&plan.request(), &plan.identity)
                .is_err()
        );
        let inventory = plan.native_pull_modules.as_ref().expect("retained modules");
        assert!(inventory.complete());
        let roots = inventory.stored_module_roots().collect::<Vec<_>>();
        assert_eq!(roots.len(), usize::from(stored));
        if stored {
            // Include sender discovery and final execution in the real core
            // budget. The manual scope loop above is not delivery evidence.
            let initial =
                prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                    .expect("dispatcher fixture");
            let mut history = talos_core::tool::PreparedStageHistory::default();
            assert!(history.record_profile(&initial.identity, &initial.facets));
            let mut dispatched = Box::new(initial).discover().await.expect("sender");
            dispatched.phase = Phase::PullNativePreparation;
            let result = execute_approved_with_history(*dispatched, history).await;
            assert!(result.result.is_error);
            assert!(
                result.result.content.contains("stage limit"),
                "{:?}",
                result.result
            );
            assert!(!module.join("FETCH_HEAD").exists());
            assert_eq!(roots, vec![module.as_path()]);
            assert!(plan.native_pull_metadata.contains_key(&module));
            assert!(plan.pull_child_startup.contains_key(&module));
            assert!(plan.native_pull_configs.contains_key(&module));
            assert!(
                plan.pull_child_startup[&module]
                    .targets
                    .contains(&module.join("../../../startup-worktree"))
            );
            assert!(plan.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource.as_deref() == module.join("gitdir").to_str()
            }));
            assert!(plan.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource.as_deref() == module.join("config").to_str()
            }));
            let admitted = proof(&plan);
            let PreparedNormalStage::Complete(output) =
                plan.execute_stage(Box::new(admitted)).await
            else {
                panic!("incomplete child admission continued");
            };
            assert!(output.result.is_error);
            assert!(
                output
                    .result
                    .content
                    .contains("recursive repository configuration admission is incomplete"),
                "{:?}",
                output.result
            );
        }
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

#[tokio::test]
async fn native_pull_scope_rejects_startup_configuration_drift() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/source"],
    );
    let module = root
        .path()
        .canonicalize()
        .expect("root")
        .join(".git/modules/child");
    std::fs::create_dir_all(&module).expect("module");
    git(&module, &["init", "--bare"]);
    std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    let config = module.join("config");
    std::fs::write(&config, "[core]\nrepositoryformatversion = 0\n").expect("config");
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan"),
    )
    .discover()
    .await
    .expect("sender");
    plan.phase = Phase::PullNativePreparation;
    for _ in 0..8 {
        if plan.phase != Phase::PullNativePreparation {
            break;
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_native_pull_scope(&tokens)
            .await
            .expect("scope");
    }
    assert!(plan.pull_child_startup.contains_key(&module));
    std::fs::write(
        &config,
        "[core]\nrepositoryformatversion = 0\nworktree = elsewhere\n",
    )
    .expect("drift");
    assert!(plan.revalidate().is_err());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn native_pull_scope_retains_child_pointer_identity_without_opening_targets() {
    let root = tempfile::tempdir().expect("fixture");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/source"],
    );
    let module = root
        .path()
        .canonicalize()
        .expect("root")
        .join(".git/modules/child");
    std::fs::create_dir_all(&module).expect("module");
    git(&module, &["init", "--bare"]);
    std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    let unopened = root.path().join("absent-common");
    std::fs::write(module.join("commondir"), unopened.to_str().expect("path")).expect("pointer");
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan"),
    )
    .discover()
    .await
    .expect("sender");
    plan.phase = Phase::PullNativePreparation;
    for _ in 0..8 {
        if plan.native_pull_metadata.contains_key(&module) {
            break;
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_native_pull_scope(&tokens)
            .await
            .expect("scope");
    }
    assert!(plan.phase == Phase::PullNativePreparation);
    assert_eq!(plan.native_pull_metadata[&module].common_path, unopened);
    assert!(!unopened.exists());
    std::fs::write(module.join("commondir"), "other-common").expect("changed pointer");
    assert!(plan.revalidate().is_err());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn native_pull_scope_resolves_and_then_admits_external_common_config() {
    for deny_config in [true, false] {
        external_common_scope(deny_config).await;
    }
}

async fn external_common_scope(deny_config: bool) {
    let root = tempfile::tempdir().expect("fixture");
    let common = tempfile::tempdir().expect("external common");
    git(root.path(), &["init", "-b", "main"]);
    git(root.path(), &["config", "core.hooksPath", ""]);
    git(
        root.path(),
        &["remote", "add", "origin", "git://example.invalid/source"],
    );
    let module = root
        .path()
        .canonicalize()
        .expect("root")
        .join(".git/modules/child");
    std::fs::create_dir_all(&module).expect("module");
    git(&module, &["init", "--bare"]);
    std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    #[cfg(unix)]
    let declared = {
        std::os::unix::fs::symlink(
            common.path().parent().expect("parent"),
            root.path().join("alias"),
        )
        .expect("common alias");
        root.path()
            .join("alias")
            .join(common.path().file_name().expect("name"))
    };
    #[cfg(not(unix))]
    let declared = common.path().to_owned();
    std::fs::write(
        module.join("commondir"),
        declared.to_str().expect("common path"),
    )
    .expect("commondir");
    git(common.path(), &["init", "--bare"]);
    std::fs::write(
        common.path().join("config"),
        "[core]\nrepositoryformatversion = 0\n",
    )
    .expect("common config");
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("plan"),
    )
    .discover()
    .await
    .expect("sender");
    plan.phase = Phase::PullNativePreparation;
    let resolved_config = common
        .path()
        .canonicalize()
        .expect("canonical common")
        .join("config");
    let mut fresh_target_frontier = false;
    for _ in 0..10 {
        if plan.phase != Phase::PullNativePreparation {
            break;
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        plan = plan
            .discover_native_pull_scope(&tokens)
            .await
            .expect("scope");
        if !fresh_target_frontier
            && plan.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource.as_deref() == resolved_config.to_str()
            })
        {
            assert!(!plan.pull_child_startup.contains_key(&module));
            fresh_target_frontier = true;
            if deny_config {
                plan.facets.retain(|facet| {
                    facet.nature != ToolNature::Read
                        || facet.resource.as_deref() != resolved_config.to_str()
                });
                let tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("profile without target content grant");
                let error = match plan.discover_native_pull_scope(&tokens).await {
                    Ok(_) => panic!("unadmitted common config was read"),
                    Err(error) => error,
                };
                assert!(error.to_string().contains("separate Read admission"));
                assert!(!root.path().join(".git/FETCH_HEAD").exists());
                return;
            }
        }
    }
    assert!(plan.phase == Phase::PullNativeMutation);
    assert!(
        plan.native_pull_modules
            .as_ref()
            .expect("modules")
            .complete()
    );
    assert!(plan.native_pull_metadata.contains_key(&module));
    assert!(plan.pull_child_startup.contains_key(&module));
    assert!(fresh_target_frontier);
    let common = common.path().canonicalize().expect("canonical common");
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref() == common.join("config").to_str()
    }));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
    std::fs::write(
        common.join("config"),
        "[core]\nrepositoryformatversion = 1\n",
    )
    .expect("configuration drift");
    assert!(plan.revalidate().is_err());
}

#[tokio::test]
async fn native_pull_scope_startup_requires_resolution_then_exact_target_admission() {
    for missing in [false, true] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        git(root.path(), &["config", "core.hooksPath", ""]);
        git(
            root.path(),
            &["remote", "add", "origin", "git://example.invalid/source"],
        );
        let root_path = root.path().canonicalize().expect("root");
        let module = root_path.join(".git/modules/child");
        let target = root_path.join("checkout");
        std::fs::create_dir_all(&module).expect("module");
        git(&module, &["init", "--bare"]);
        if !missing {
            std::fs::create_dir(&target).expect("checkout");
        }
        std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        std::fs::write(
            module.join("config"),
            "[core]\nrepositoryformatversion = 0\nworktree = ../../../checkout\n",
        )
        .expect("config");
        let mut plan = Box::new(
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("sender");
        plan.phase = Phase::PullNativePreparation;
        let mut saw_resolution = false;
        for _ in 0..10 {
            if plan
                .facets
                .iter()
                .any(|facet| facet.resource.as_deref() == Some("git:pull-worktree-resolution"))
            {
                saw_resolution = true;
                assert!(
                    !plan
                        .facets
                        .iter()
                        .any(|facet| facet.resource.as_deref() == target.to_str())
                );
                break;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("proof");
            plan = plan
                .discover_native_pull_scope(&tokens)
                .await
                .expect("discovery");
        }
        assert!(saw_resolution);
        assert_eq!(
            plan.native_pull_modules
                .as_ref()
                .expect("modules")
                .resolved_startup_worktrees()
                .count(),
            0
        );
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("resolution proof");
        let next = plan.discover_native_pull_scope(&tokens).await;
        if missing {
            assert!(next.is_err(), "missing startup target must refuse");
        } else {
            let mut plan = next.expect("resolved frontier");
            assert!(plan.phase == Phase::PullNativePreparation);
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == target.to_str())
            );
            plan.facets
                .retain(|facet| facet.resource.as_deref() != target.to_str());
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("without target grant");
            let error = match plan.discover_native_pull_scope(&tokens).await {
                Ok(_) => panic!("unadmitted startup target advanced"),
                Err(error) => error,
            };
            assert!(error.to_string().contains("separate Read admission"));
        }
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}
