#[tokio::test]
async fn pull_postfetch_rebuilds_evidence_before_selecting_integration() {
    for source in [
        "worktree",
        "index",
        "head",
        "absent",
        "linked-tamper",
        "linked-budget",
    ] {
        let root = fixture();
        let current = b"[submodule \"current\"]\npath = old\npath = current/child\nfetchRecurseSubmodules = true\nfetchRecurseSubmodules = on-demand\n";
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["-c", "maintenance.autoDetach=false", "-c", "gc.autoDetach=false"])
                .args(args)
                .output()
                .expect("Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        if source == "head" {
            std::fs::write(root.path().join(".gitmodules"), current).expect("HEAD modules");
            git(&["add", ".gitmodules"]);
        }
        git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        git(&["config", "remote.origin.url", "git://example.invalid/repo"]);
        std::fs::write(root.path().join("diff-order"), "missing/*\n").expect("order file");
        git(&["config", "diff.orderFile", "diff-order"]);
        // A dangling commit is already stored but is not reachable from any
        // pre-fetch ref. Fetching it must include it in child-graph discovery
        // while excluding its already-reachable parent.
        let child_oid = git(&["rev-parse", "HEAD"]);
        let child_oid = std::str::from_utf8(&child_oid).expect("child OID").trim();
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{child_oid},missing/child"),
        ]);
        std::fs::write(
            root.path().join(".gitmodules"),
            "[submodule \"child\"]\npath = missing/child\nurl = git://example.invalid/child\n",
        )
        .expect("module declaration");
        git(&["add", ".gitmodules"]);
        let tree_oid = git(&["write-tree"]);
        let tree_oid = std::str::from_utf8(&tree_oid).expect("tree OID").trim();
        let target = git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit-tree",
            tree_oid,
            "-p",
            "HEAD",
            "-m",
            "new fetched history",
        ]);
        git(&["reset", "--mixed", "HEAD"]);
        std::fs::remove_file(root.path().join(".gitmodules")).expect("remove fixture config");
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{child_oid},current/child"),
        ]);
        match source {
            "worktree" => {
                std::fs::write(root.path().join(".gitmodules"), current).expect("worktree modules")
            }
            "index" => {
                std::fs::write(root.path().join(".gitmodules"), current).expect("index modules");
                git(&["add", ".gitmodules"]);
                std::fs::remove_file(root.path().join(".gitmodules"))
                    .expect("missing worktree modules");
            }
            "head" => {
                git(&["rm", "--cached", ".gitmodules"]);
            }
            _ => {}
        }
        let target = std::str::from_utf8(&target).expect("object ID").trim();
        git(&["config", "submodule.child.active", "true"]);
        let mut plan = Box::new(
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("Pull"),
        );
        let mut history = talos_core::tool::PreparedStageHistory::default();
        let mut recorded_stages = 0;
        assert!(history.record_profile(&plan.identity, &plan.facets));
        recorded_stages += 1;
        plan = plan.discover().await.expect("configuration");
        for _ in 0..8 {
            if plan.phase == Phase::PullFetch {
                break;
            }
            assert!(history.record_profile(&plan.identity, &plan.facets));
            recorded_stages += 1;
            let admission = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("proof");
            plan = plan
                .discover_pull_prefetch(&admission)
                .await
                .expect("prefetch");
        }
        assert!(plan.phase == Phase::PullFetch);
        assert!(history.record_profile(&plan.identity, &plan.facets));
        recorded_stages += 1;
        assert!(
            plan.pull_attributes
                .iter()
                .any(|attribute| attribute.path == plan.repository.workdir.join("diff-order"))
        );

        // Exercise the post-fetch boundary with actual Git metadata mutation,
        // independently of a network daemon or successful transport fixture.
        git(&["fetch", "--no-recurse-submodules", ".", target]);
        assert!(
            plan.revalidate().is_err(),
            "old FETCH_HEAD evidence must expire"
        );
        plan.facets
            .retain(|facet| matches!(facet.nature, ToolNature::Read | ToolNature::Execute));
        plan.pull_inventory = None;
        plan.identity = PreparedPlanIdentity::new();
        plan.phase = Phase::PullPostFetch;
        let external = tempfile::tempdir().expect("postfetch external common");
        let pointer = root.path().join(".git/worktrees/linked/commondir");
        if source.starts_with("linked-") {
            std::fs::create_dir_all(pointer.parent().expect("registration")).expect("registration");
            std::fs::write(
                &pointer,
                format!(
                    "{}\n",
                    external.path().canonicalize().expect("external").display()
                ),
            )
            .expect("external common pointer");
        }
        if source == "linked-budget" {
            let deep = external
                .path()
                .canonicalize()
                .expect("canonical external root")
                .join("refs/heads/a/b/c/d/e/f/g/h/i/j/topic");
            std::fs::create_dir_all(deep.parent().expect("parents")).expect("deep refs");
            std::fs::write(&deep, format!("{target}\n")).expect("external ref");
            let fetched =
                std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("fetch effects");
            while history.record_profile(&plan.identity, &plan.facets) {
                recorded_stages += 1;
                let tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("postfetch frontier admission");
                plan = plan
                    .discover_pull_postfetch(&tokens)
                    .await
                    .unwrap_or_else(|error| {
                        panic!("{source} frontier at stage {recorded_stages}: {error}")
                    });
                assert!(plan.phase == Phase::PullPostFetch);
            }
            assert_eq!(recorded_stages, 8);
            assert!(
                !plan
                    .pull_inventory
                    .as_ref()
                    .expect("retained frontier")
                    .paths()
                    .contains(&deep)
            );
            assert_eq!(
                std::fs::read(root.path().join(".git/FETCH_HEAD")).expect("retained fetch"),
                fetched
            );
            continue;
        }
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh proof");
        assert!(!admission.is_empty());
        let next = plan
            .discover_pull_postfetch(&admission)
            .await
            .expect("postfetch");
        if source == "linked-tamper" {
            assert!(next.phase == Phase::PullPostFetch);
            assert!(next.pull_inventory.is_some());
            let tokens = proof(&next)
                .consume_for_prepared(&next.request(), &next.identity)
                .expect("target admission");
            std::fs::write(&pointer, "../..\n").expect("change retained pointer");
            assert!(next.discover_pull_postfetch(&tokens).await.is_err());
            assert!(root.path().join(".git/FETCH_HEAD").exists());
            continue;
        }
        assert!(
            next.phase == Phase::PullChildIdentity,
            "source={source}, activity={:?}",
            next.pull_changed_children
                .values()
                .map(|child| &child.activity)
                .collect::<Vec<_>>()
        );
        assert!(next.pull_activity_globals.is_empty());
        for path in [
            next.repository.workdir.join("missing/child"),
            next.repository.workdir.join("missing/child/.git"),
        ] {
            assert!(next.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }));
            assert!(next.child_read(&path, &admission).is_err());
        }
        assert!(next.child_locations.is_empty());
        assert_eq!(next.pull_new_commits, [target]);
        assert!(next.pull_child_trees.contains_key(target));
        assert_eq!(
            next.pull_index_children
                .get("current/child")
                .map(String::as_str),
            Some(child_oid)
        );
        assert!(!root.path().join("current").exists());
        let child_tree = &next.pull_child_trees[target];
        assert_eq!(
            child_tree.gitlinks.get("missing/child").map(String::as_str),
            Some(child_oid)
        );
        assert_eq!(child_tree.changed_gitlinks, child_tree.gitlinks);
        let changed = &next.pull_changed_children["child"];
        assert_eq!(changed.commit, target);
        assert_eq!(changed.path, "missing/child");
        assert!(changed.objects.contains(child_oid));
        assert_eq!(
            changed.effective_recursion,
            Some(super::super::pull_children::Recursion::OnDemand)
        );
        assert!(child_tree.modules_blob.is_some());
        assert_eq!(
            child_tree
                .module_names
                .get("missing/child")
                .map(String::as_str),
            Some("child")
        );
        assert!(!root.path().join("missing").exists());
        let modules = next
            .pull_current_modules
            .as_ref()
            .expect("current declarations");
        if source == "absent" {
            assert!(modules.module_names.is_empty());
        } else {
            assert_eq!(modules.module_names["current/child"], "current", "{source}");
            assert_eq!(
                modules.module_recursion["current"],
                super::super::pull_children::Recursion::OnDemand
            );
        }
        assert_eq!(
            root.path().join(".gitmodules").exists(),
            source == "worktree"
        );
        match next.pull_integration.as_ref().expect("integration") {
            super::super::pull_state::Integration::Merge { heads, .. } => {
                let fetched = git(&["rev-parse", "FETCH_HEAD^{commit}"]);
                assert_eq!(heads, &[String::from_utf8(fetched).expect("OID").trim().to_owned()]);
            }
            other => panic!("expected merge plan, got {other:?}"),
        }
        next.revalidate().expect("fresh evidence");
        std::fs::write(root.path().join(".gitmodules"), b"changed after admission")
            .expect("tamper modules");
        assert!(
            next.revalidate().is_err(),
            "{source} module evidence must expire"
        );
        if source == "worktree" {
            std::fs::write(root.path().join(".gitmodules"), current).expect("restore modules");
        } else {
            std::fs::remove_file(root.path().join(".gitmodules")).expect("restore absence");
        }
        next.revalidate().expect("restored module evidence");
        std::fs::write(root.path().join(".git/FETCH_HEAD"), "tampered").expect("tamper");
        assert!(next.revalidate().is_err());
    }
}
