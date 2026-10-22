//! Child include-chain discovery retains evidence across fresh source admission.

use super::*;

#[tokio::test]
async fn native_pull_child_destinations_and_final_configuration_checks_are_bound() {
    for scenario in [
        "unchanged",
        "include-drift",
        "missing-source",
        "missing-chain",
        "missing-destination",
        "missing-startup",
        "local",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        git(root.path(), &["config", "core.hooksPath", ""]);
        git(
            root.path(),
            &["remote", "add", "origin", "git://example.invalid/parent"],
        );
        let module = root
            .path()
            .canonicalize()
            .expect("root")
            .join(".git/modules/child");
        std::fs::create_dir_all(&module).expect("module");
        git(&module, &["init", "--bare", "-b", "main"]);
        let startup = root
            .path()
            .canonicalize()
            .expect("root")
            .join("child-worktree");
        std::fs::create_dir(&startup).expect("startup worktree");
        git(&module, &["config", "core.bare", "false"]);
        git(
            &module,
            &["config", "core.worktree", startup.to_str().expect("path")],
        );
        git(&module, &["config", "include.path", "included-config"]);
        let source = module.join("included-config");
        let destination = if scenario == "local" {
            "../source.git"
        } else {
            "alias:child"
        };
        std::fs::write(&source, format!(
            "[branch \"main\"]\nremote = upstream\n[remote \"origin\"]\nurl = git://example.invalid/decoy\n[remote \"upstream\"]\nurl = {destination}\n[url \"git://example.invalid/rewritten/\"]\ninsteadOf = alias:\n",
        )).expect("include");
        let mut plan = Box::new(
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("sender");
        plan.phase = Phase::PullNativePreparation;
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
        }
        assert!(plan.phase == Phase::PullNativeMutation);
        let network = "git://example.invalid/rewritten/child";
        assert!(
            !plan
                .facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Network
                    && facet.resource.as_deref() == Some("git://example.invalid/decoy"))
        );
        if scenario == "local" {
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Read
                        && facet.resource.as_deref() == module.join("../source.git").to_str())
            );
        } else {
            assert!(
                plan.facets
                    .iter()
                    .any(|facet| facet.nature == ToolNature::Network
                        && facet.resource.as_deref() == Some(network))
            );
        }
        match scenario {
            "include-drift" => std::fs::write(
                &source,
                "[remote \"upstream\"]\nurl = git://example.invalid/changed\n",
            )
            .expect("drift"),
            "missing-source" => plan
                .facets
                .retain(|facet| facet.resource.as_deref() != source.to_str()),
            "missing-chain" => plan
                .facets
                .retain(|facet| facet.resource.as_deref() != Some("git:configuration-chain")),
            "missing-destination" => plan
                .facets
                .retain(|facet| facet.resource.as_deref() != Some(network)),
            "missing-startup" => plan
                .facets
                .retain(|facet| facet.resource.as_deref() != startup.to_str()),
            _ => {}
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        if matches!(scenario, "unchanged" | "local") {
            plan.validate_native_pull_configurations(&tokens)
                .await
                .expect("unchanged child scope");
        } else {
            let error = plan
                .execute_native_pull(&tokens)
                .await
                .expect_err("unapproved or changed configuration")
                .to_string();
            let expected = match scenario {
                "include-drift" => "configuration changed",
                "missing-source" => "separate Read admission",
                "missing-chain" => "discovery authority missing",
                "missing-destination" => "destination differs",
                "missing-startup" => "separate Read admission",
                _ => unreachable!(),
            };
            assert!(error.contains(expected), "{scenario}: {error}");
        }
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
        assert!(!module.join("FETCH_HEAD").exists());
    }
}

#[tokio::test]
async fn native_pull_child_configuration_requires_source_admission_and_rejects_drift() {
    for deny_source in [true, false] {
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
        let source = module.join("included-config");
        std::fs::write(
            &source,
            "[remote \"origin\"]\nurl = git://example.invalid/child\n",
        )
        .expect("include source");
        git(&module, &["config", "include.path", "included-config"]);
        let mut plan = Box::new(
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan"),
        )
        .discover()
        .await
        .expect("sender");
        plan.phase = Phase::PullNativePreparation;
        let mut source_frontier = false;
        let mut checked_chain_frontier = false;
        for _ in 0..10 {
            if !checked_chain_frontier
                && plan
                    .facets
                    .iter()
                    .any(|facet| facet.resource.as_deref() == module.join("config").to_str())
            {
                plan.facets
                    .retain(|facet| facet.resource.as_deref() != Some("git:configuration-chain"));
                // If the newly published capability were used immediately,
                // this malformed include would make the host probe fail.
                std::fs::write(&source, "[invalid configuration").expect("malformed include");
                let tokens = proof(&plan)
                    .consume_for_prepared(&plan.request(), &plan.identity)
                    .expect("proof without configuration discovery");
                plan = plan
                    .discover_native_pull_scope(&tokens)
                    .await
                    .expect("new chain frontier");
                assert!(plan.native_pull_configs.is_empty());
                std::fs::write(
                    &source,
                    "[remote \"origin\"]\nurl = git://example.invalid/child\n",
                )
                .expect("restore include");
                checked_chain_frontier = true;
            }
            let tokens = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("fresh proof");
            plan = plan
                .discover_native_pull_scope(&tokens)
                .await
                .expect("discovery");
            if plan.native_pull_configs.contains_key(&module) {
                source_frontier = true;
                break;
            }
            assert!(!root.path().join(".git/FETCH_HEAD").exists());
        }
        assert!(source_frontier, "child configuration was not discovered");
        assert!(checked_chain_frontier);
        assert!(plan.phase == Phase::PullNativePreparation);
        assert!(plan.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read && facet.resource.as_deref() == source.to_str()
        }));
        if deny_source {
            plan.facets.retain(|facet| {
                facet.nature != ToolNature::Read || facet.resource.as_deref() != source.to_str()
            });
            // A malformed source must not be reopened under a missing grant.
            std::fs::write(&source, "[invalid configuration").expect("malformed include");
        } else {
            std::fs::write(
                &source,
                "[remote \"origin\"]\nurl = git://example.invalid/changed\n",
            )
            .expect("include drift");
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("proof");
        let error = match plan.discover_native_pull_scope(&tokens).await {
            Ok(_) => panic!("unapproved or changed include continued"),
            Err(error) => error.to_string(),
        };
        if deny_source {
            assert!(error.contains("separate Read admission"), "{error}");
        } else {
            assert!(
                error.contains("configuration") && error.contains("changed"),
                "{error}"
            );
        }
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}
