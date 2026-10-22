//! Scope discovery consumes fresh approvals before reading module frontiers.

use super::*;

#[tokio::test]
async fn native_pull_scope_discovers_stored_modules_without_expanding_configuration() {
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
            std::fs::write(module.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
            // Startup parsing must not expand includes. This directory would
            // make include-expanded discovery fail if attempted at this stage.
            std::fs::create_dir(module.join("unread-include")).expect("unread include");
            std::fs::write(module.join("config"),
                "[core]\nrepositoryformatversion = 0\nworktree = ../../../absent-worktree\n[include]\npath = unread-include\n",
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
            assert!(waves < 8, "scope discovery did not converge");
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
            assert_eq!(roots, vec![module.as_path()]);
            assert!(plan.native_pull_metadata.contains_key(&module));
            assert!(plan.pull_child_startup.contains_key(&module));
            assert!(
                plan.pull_child_startup[&module]
                    .targets
                    .contains(&module.join("../../../absent-worktree"))
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
    assert!(plan.phase == Phase::PullNativeMutation);
    assert_eq!(plan.native_pull_metadata[&module].common_path, unopened);
    assert!(!unopened.exists());
    std::fs::write(module.join("commondir"), "other-common").expect("changed pointer");
    assert!(plan.revalidate().is_err());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}
