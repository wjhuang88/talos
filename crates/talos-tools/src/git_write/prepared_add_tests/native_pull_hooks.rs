//! Exact hook admission at the native operation boundary.

use super::*;

#[tokio::test]
async fn native_pull_hooks_require_complete_frontier_and_exact_reads() {
    for scenario in ["missing-read", "missing-frontier", "executable", "changed"] {
        let root = tempfile::tempdir().expect("fixture");
        git(root.path(), &["init", "-b", "main"]);
        let hooks = root.path().join("operation-hooks");
        std::fs::create_dir(&hooks).expect("hook directory");
        git(
            root.path(),
            &["config", "core.hooksPath", hooks.to_str().expect("path")],
        );
        git(root.path(), &["config", "commit.gpgSign", "false"]);
        let mut plan =
            prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("plan");
        let output = plan.inspect_config().await.expect("configuration fixture");
        let mut config = super::super::super::super::config_state::ConfigState::capture_host(
            &output,
            &plan.repository,
            &plan.environment,
        )
        .expect("config");
        plan.pull_hook_paths = config
            .prepare_native_pull(&plan.repository, &plan.environment, Some("main"))
            .expect("frontier");
        plan.config = Some(config);
        let candidate = hooks.join("post-merge");
        assert!(plan.pull_hook_paths.contains(&candidate));
        for path in &plan.pull_hook_paths {
            if scenario != "missing-read" || path != &candidate {
                plan.facets
                    .push(path_facet(ToolNature::Read, path, "hook fixture").expect("Read"));
            }
        }
        // A directory at an unread candidate proves the missing-read case
        // rejects admission before attempting to capture its contents/type.
        if scenario == "missing-read" {
            std::fs::create_dir(&candidate).expect("unread candidate");
        } else if scenario == "executable" {
            std::fs::write(&candidate, b"#!/bin/sh\nexit 0\n").expect("hook");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&candidate, std::fs::Permissions::from_mode(0o755))
                    .expect("executable");
            }
        } else if scenario == "missing-frontier" {
            plan.pull_hook_paths.retain(|path| path != &candidate);
        }
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("exact fixture admission");
        if scenario == "changed" {
            plan.pull_hook_evidence = plan
                .capture_native_pull_hooks(&tokens)
                .await
                .expect("absent hooks");
            std::fs::write(&candidate, b"changed after discovery\n").expect("change hook");
        }
        let error = plan
            .capture_native_pull_hooks(&tokens)
            .await
            .expect_err("unsafe frontier");
        let expected = match scenario {
            "missing-read" => "lacks exact Read admission",
            "missing-frontier" => "frontier differs",
            "executable" => "separate execution authorization",
            _ => "hook evidence changed",
        };
        assert!(error.to_string().contains(expected), "{scenario}: {error}");
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}
