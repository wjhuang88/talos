#[tokio::test]
async fn pull_rebase_clean_probes_preserve_index_and_touch_semantics() {
    for state in [
        "clean",
        "touch",
        "dirty",
        "staged",
        "autostash",
        "pull-autostash",
        "pull-no-autostash",
        "symbolic-policy",
        "external-orderfile",
        "binary",
        "deleted",
        "mode",
        "gitlink",
    ] {
        let root = fixture();
        let order_root = tempfile::tempdir().expect("external order fixture");
        let order_path = order_root
            .path()
            .canonicalize()
            .expect("order root")
            .join("order");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .output()
                .expect("fixture Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "base"]);
        git(&["config", "pull.rebase", "true"]);
        if state == "external-orderfile" {
            std::fs::write(&order_path, "file\n").expect("order file");
            git(&[
                "config",
                "diff.orderFile",
                order_path.to_str().expect("path"),
            ]);
        }
        if state == "symbolic-policy" {
            git(&["branch", "-m", "Topic"]);
            git(&["symbolic-ref", "refs/heads/Alias", "refs/heads/Topic"]);
            git(&["symbolic-ref", "HEAD", "refs/heads/Alias"]);
            git(&["config", "branch.Alias.rebase", "false"]);
            git(&["config", "branch.Topic.rebase", "true"]);
        }
        if state == "gitlink" {
            let head = std::process::Command::new("git")
                .current_dir(root.path())
                .args(["rev-parse", "HEAD"])
                .output()
                .expect("head");
            assert!(head.status.success());
            let oid = String::from_utf8(head.stdout).expect("oid");
            git(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{},child", oid.trim()),
            ]);
        }
        if state == "binary" {
            std::fs::write(root.path().join("file"), b"changed\0binary").expect("binary");
        } else if state == "deleted" {
            std::fs::remove_file(root.path().join("file")).expect("delete");
        } else if state == "mode" {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                git(&["config", "core.fileMode", "true"]);
                std::fs::set_permissions(
                    root.path().join("file"),
                    std::fs::Permissions::from_mode(0o755),
                )
                .expect("mode");
            }
            #[cfg(not(unix))]
            continue;
        }
        if state == "touch" {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(root.path().join("file"))
                .expect("file");
            file.set_modified(
                std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1234567890),
            )
            .expect("mtime");
        } else if matches!(
            state,
            "dirty" | "staged" | "autostash" | "pull-autostash" | "pull-no-autostash"
        ) {
            std::fs::write(root.path().join("file"), "changed\n").expect("change");
            if state == "staged" {
                git(&["add", "file"]);
            }
            if state == "autostash" {
                git(&["config", "rebase.autostash", "true"]);
            }
            if state == "pull-autostash" {
                git(&["config", "rebase.autostash", "false"]);
                git(&["config", "pull.autostash", "true"]);
            }
            if state == "pull-no-autostash" {
                git(&["config", "rebase.autostash", "true"]);
                git(&["config", "pull.autostash", "false"]);
            }
        }
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan =
            AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
                .expect("prepare");
        let mut plan = Box::new(plan).discover().await.expect("configuration");
        let mut outcome = None;
        let mut late_source_admission = false;
        for _ in 0..8 {
            let admission = proof(&plan)
                .consume_for_prepared(&plan.request(), &plan.identity)
                .expect("admission");
            match plan.discover_pull_prefetch(&admission).await {
                Ok(next) if next.phase == Phase::PullFetch => {
                    if state == "external-orderfile" {
                        assert!(
                            next.facets
                                .iter()
                                .any(|facet| facet.nature == ToolNature::Read
                                    && facet.resource.as_deref() == order_path.to_str())
                        );
                        assert!(
                            next.pull_attributes
                                .iter()
                                .any(|entry| entry.path == order_path)
                        );
                        std::fs::write(&order_path, "changed\n").expect("change order file");
                        assert!(
                            next.revalidate().is_err(),
                            "external rule change must invalidate plan"
                        );
                    }
                    if state == "symbolic-policy" {
                        assert!(
                            late_source_admission,
                            "alias policy must require fresh source admission"
                        );
                        assert!(!next.pull_attribute_paths.is_empty());
                        assert_eq!(next.pull_attributes.len(), next.pull_attribute_paths.len());
                    }
                    if matches!(state, "autostash" | "pull-autostash") {
                        let source = next.repository.workdir.join("file");
                        assert!(
                            !next.facets.iter().any(|facet| {
                                facet.nature == ToolNature::Read
                                    && facet.resource.as_deref() == source.to_str()
                            }),
                            "autostash must not request prefetch clean-source Read"
                        );
                        assert!(next.pull_sources.is_none());
                        assert!(next.pull_attributes.is_empty());
                    }
                    outcome = Some(Ok(()));
                    break;
                }
                Ok(next) => {
                    if state == "symbolic-policy" {
                        let source = next.repository.workdir.join("file");
                        if next
                            .facets
                            .iter()
                            .any(|facet| facet.resource.as_deref() == source.to_str())
                            && !admission.iter().any(|token| {
                                token.authorizes_path(
                                    "git_pull",
                                    ToolNature::Read,
                                    &next.workspace,
                                    source.to_str().expect("path"),
                                )
                            })
                        {
                            assert!(next.pull_sources.is_none());
                            assert!(next.pull_attributes.is_empty());
                            late_source_admission = true;
                        }
                    }
                    plan = next;
                }
                Err(error) => {
                    outcome = Some(Err(error));
                    break;
                }
            }
        }
        let outcome = outcome.expect("bounded preparation");
        if matches!(
            state,
            "dirty" | "staged" | "binary" | "deleted" | "mode" | "pull-no-autostash"
        ) {
            let error = outcome.expect_err("dirty source must refuse").to_string();
            assert!(
                error.contains(if state == "staged" {
                    "clean index"
                } else {
                    "clean worktree"
                }),
                "{state}: {error}"
            );
        } else {
            outcome.unwrap_or_else(|error| panic!("{state}: {error}"));
        }
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index after"),
            index,
            "{state}"
        );
        assert!(!root.path().join(".git/index.lock").exists());
        assert!(!root.path().join(".git/FETCH_HEAD").exists());
    }
}

#[tokio::test]
async fn pull_rebase_filter_refuses_without_running_helper() {
    let root = fixture();
    for args in [
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["config", "commit.gpgSign", "false"],
        vec!["add", "file"],
        vec!["commit", "-m", "base"],
        vec!["config", "pull.rebase", "true"],
        vec!["config", "filter.probe.clean", "touch helper-ran; cat"],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::write(root.path().join(".gitattributes"), "file filter=probe\n").expect("rules");
    let plan = AddPlan::prepare_pull(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("prepare");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let mut refused = false;
    for _ in 0..8 {
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh admission");
        match plan.discover_pull_prefetch(&admission).await {
            Ok(next) => {
                assert!(
                    next.phase != Phase::PullFetch,
                    "filtered source reached fetch"
                );
                plan = next;
            }
            Err(error) => {
                assert!(error.to_string().contains("configured filter"), "{error}");
                refused = true;
                break;
            }
        }
    }
    assert!(refused);
    assert!(!root.path().join("helper-ran").exists());
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_rebase_executes_divergent_history_with_each_backend() {
    for (backend, mode) in [
        ("merge", super::super::pull_state::Rebase::On),
        ("apply", super::super::pull_state::Rebase::On),
        ("apply", super::super::pull_state::Rebase::Merges),
    ] {
        let root = fixture();
        let git = |args: &[&str]| integration_git(root.path(), args);
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "rebase.backend", "apply"]);
        git(&["config", "--add", "rebase.backend", backend]);
        git(&["add", "file"]);
        git(&["commit", "-m", "base"]);
        let base = git(&["rev-parse", "HEAD"]);
        std::fs::write(root.path().join("remote"), "remote\n").expect("remote");
        git(&["add", "remote"]);
        git(&["commit", "-m", "remote"]);
        let target = git(&["rev-parse", "HEAD"]);
        git(&["reset", "--hard", &base]);
        std::fs::write(root.path().join("local"), "local\n").expect("local");
        git(&["add", "local"]);
        git(&["commit", "-m", "local"]);
        let original = git(&["rev-parse", "HEAD"]);
        let mut plan = integration_scope_fixture(root.path()).await;
        std::fs::write(
            root.path().join(".git/FETCH_HEAD"),
            format!("{target}\t\tfixture\n"),
        )
        .expect("FETCH_HEAD");
        plan.pull_evidence = Some(
            super::super::pull_state::PullEvidence::capture(&plan.repository, None, true)
                .expect("evidence"),
        );
        plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
            onto: target.clone(),
            upstream: target.clone(),
            mode: mode.clone(),
            autostash: false,
        });
        let plan = plan.discover_pull_integration_hooks().expect("frontier");
        for name in ["applypatch-msg", "pre-applypatch", "post-applypatch"] {
            assert_eq!(
                plan.pull_hook_paths.iter().any(|path| path.ends_with(name)),
                backend == "apply" && mode == super::super::pull_state::Rebase::On
            );
        }
        let admission = proof(&plan);
        let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("final stage")
        };
        assert!(
            !output.result.is_error,
            "{backend}: {}",
            output.result.content
        );
        assert_ne!(git(&["rev-parse", "HEAD"]), original);
        assert_eq!(git(&["rev-parse", "HEAD^"]), target);
        assert_eq!(git(&["log", "-1", "--format=%s"]), "local");
        assert_eq!(
            std::fs::read(root.path().join("local")).expect("local"),
            b"local\n"
        );
        assert_eq!(
            std::fs::read(root.path().join("remote")).expect("remote"),
            b"remote\n"
        );
        assert!(git(&["status", "--porcelain"]).is_empty());
    }
}

#[cfg(unix)]
#[tokio::test]
async fn pull_apply_backend_refuses_executable_patch_hooks_before_mutation() {
    use std::os::unix::fs::PermissionsExt;
    for name in ["applypatch-msg", "pre-applypatch", "post-applypatch"] {
        let root = fixture();
        integration_git(root.path(), &["config", "rebase.backend", "apply"]);
        let hook = root.path().join(".git/hooks").join(name);
        std::fs::write(&hook, "#!/bin/sh\ntouch patch-hook-ran\n").expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
        let mut plan = integration_scope_fixture(root.path()).await;
        plan.pull_integration = Some(super::super::pull_state::Integration::Rebase {
            onto: "1".repeat(40),
            upstream: "2".repeat(40),
            mode: super::super::pull_state::Rebase::On,
            autostash: false,
        });
        let plan = plan
            .discover_pull_integration_hooks()
            .expect("hook frontier");
        let admission = proof(&plan);
        let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("hook refusal")
        };
        assert!(output.result.is_error);
        assert!(
            output.result.content.contains("Executable Git Pull hooks"),
            "{}",
            output.result.content
        );
        assert!(!root.path().join("patch-hook-ran").exists());
        assert!(!root.path().join(".git/index").exists());
    }
}
