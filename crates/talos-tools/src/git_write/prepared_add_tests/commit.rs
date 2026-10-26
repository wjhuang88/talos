#[cfg(unix)]
#[tokio::test]
async fn commit_target_probe_only_treats_quiet_missing_reference_as_absent() {
    let root = fixture();
    let plan = prepare(root.path());
    assert!(
        plan.inspect_commit_target("refs/heads/missing")
            .await
            .expect("missing reference")
            .is_none()
    );
    plan.command_budget.lock().expect("budget").commands = 0;
    let error = plan
        .inspect_commit_target("refs/heads/missing")
        .await
        .expect_err("exhaustion is not missing");
    assert!(
        error
            .to_string()
            .contains("cumulative command/output budget")
    );
}

#[tokio::test]
async fn commit_maintenance_recent_objects_hook_requires_execution_authorization() {
    for maintenance in [None, Some("true"), Some("false")] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("fixture Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "initial"]);
        std::fs::write(root.path().join("file"), "changed").expect("change");
        git(&["add", "file"]);
        git(&[
            "config",
            "gc.recentObjectsHook",
            "touch recent-objects-hook-ran",
        ]);
        if let Some(value) = maintenance {
            git(&["config", "maintenance.auto", value]);
        }
        let head = git(&["rev-parse", "HEAD"]);
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"prepared"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        let output = execute_approved(plan).await;
        if maintenance == Some("false") {
            assert!(!output.result.is_error, "{}", output.result.content);
            assert_ne!(git(&["rev-parse", "HEAD"]), head);
        } else {
            assert!(
                output.result.is_error && output.result.content.contains("gc.recentObjectsHook")
            );
            assert_eq!(git(&["rev-parse", "HEAD"]), head);
            assert_eq!(
                std::fs::read(root.path().join(".git/index")).expect("index"),
                index
            );
        }
        assert!(!root.path().join("recent-objects-hook-ran").exists());
    }
}

#[test]
fn retained_child_evidence_rejects_changes_after_inspection_returns() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args(["add", "file"])
        .current_dir(root.path())
        .output()
        .expect("stage");
    assert!(output.status.success());
    let paths = vec!["file".to_owned()];
    let evidence = ChildInspectionEvidence {
        attributes: Vec::new(),
        status: Vec::new(),
        sources: vec![(
            root.path().to_owned(),
            root.path().to_owned(),
            paths.clone(),
            capture_tracked_sources(root.path(), &paths).expect("sources"),
        )],
    };
    evidence.revalidate().expect("unchanged");
    std::fs::write(root.path().join("file"), "after!").expect("late edit");
    assert!(
        evidence
            .revalidate()
            .expect_err("late source change")
            .to_string()
            .contains("tracked sources")
    );
    std::fs::write(root.path().join("file"), "before").expect("restore");
}

#[test]
fn tracked_child_sources_detect_edits_without_reading_untracked_replacement_tree() {
    let root = tempfile::tempdir().expect("fixture");
    let paths = vec!["tracked".to_owned()];
    std::fs::write(root.path().join("tracked"), "first").expect("tracked");
    let before = capture_tracked_sources(root.path(), &paths).expect("before");
    std::fs::write(root.path().join("tracked"), "other").expect("same length edit");
    assert_ne!(
        before,
        capture_tracked_sources(root.path(), &paths).expect("after")
    );
    std::fs::remove_file(root.path().join("tracked")).expect("remove file");
    std::fs::create_dir_all(root.path().join("tracked/untracked/.git"))
        .expect("untracked nested repository");
    assert_eq!(
        capture_tracked_sources(root.path(), &paths).expect("directory kind only"),
        vec![snapshot::Entry::Directory(Vec::new())]
    );
}

#[tokio::test]
async fn child_status_filter_is_refused_without_executing_helper() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    std::fs::write(child.join("tracked"), "first\n").expect("tracked");
    git(&child, &["add", "tracked"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    let parent_attributes = root.path().join("parent-attributes");
    std::fs::write(&parent_attributes, "").expect("parent attributes");
    git(
        root.path(),
        &[
            "config",
            "core.attributesFile",
            parent_attributes.to_str().expect("path"),
        ],
    );
    let plan = discover_sender_approved(Box::new(
        AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"parent"}),
            ToolProvenance::Native,
        )
        .expect("plan"),
    ))
    .await
    .expect("child attribute discovery");
    let child_global = plan
        .inspect_attribute_path_at(&child, "GIT_ATTR_GLOBAL")
        .await
        .expect("child global");
    assert_ne!(child_global, parent_attributes);
    assert!(plan.child_attributes.contains(&child_global));
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == child_global.to_str())
    );
    std::fs::write(child.join(".gitattributes"), "tracked filter=probe\n").expect("attributes");
    git(
        &child,
        &["config", "filter.probe.clean", "touch helper-ran; cat"],
    );
    std::fs::write(child.join("tracked"), "other\n").expect("same length edit");
    for all in [false, true] {
        let input = serde_json::json!({"message":"parent", "all":all});
        let result = execute_approved(
            AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("plan"),
        )
        .await;
        assert!(
            result.result.is_error
                && result
                    .result
                    .content
                    .contains("Submodule status uses a configured filter"),
            "{}",
            result.result.content
        );
        assert!(!child.join("helper-ran").exists());
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    }
}

#[tokio::test]
async fn child_invalid_index_is_read_only_after_continuation_admission() {
    let root = fixture();
    let child = root.path().join("module");
    let external = tempfile::tempdir().expect("external metadata");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(
        &child,
        &[
            "init",
            "--separate-git-dir",
            external
                .path()
                .join("metadata")
                .to_str()
                .expect("metadata path"),
        ],
    );
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    std::fs::write(
        external.path().join("metadata/index"),
        "invalid child index",
    )
    .expect("invalid index");
    let input = serde_json::json!({"message":"parent", "all":true});
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native)
        .expect("prepare without reading child index");
    let next = discover_sender_approved(Box::new(plan))
        .await
        .expect("discovery must not read child index");
    assert!(next.permission_profile().iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref()
                == external
                    .path()
                    .join("metadata")
                    .canonicalize()
                    .expect("canonical metadata")
                    .to_str()
    }));
    drop(next);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    let result = execute_approved(
        AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("second plan"),
    )
    .await;
    assert!(result.result.is_error);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    assert_eq!(
        std::fs::read(external.path().join("metadata/index")).expect("preserved index"),
        b"invalid child index"
    );
}

#[tokio::test]
async fn child_config_discovery_detects_included_fsmonitor_before_status() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    let outside = tempfile::tempdir().expect("external config");
    let included = outside.path().join("config");
    std::fs::write(&included, "[core]\n fsmonitor = touch helper-ran\n").expect("helper config");
    git(
        &child,
        &["config", "include.path", included.to_str().expect("path")],
    );
    let input = serde_json::json!({"message":"parent", "all":true});
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native)
        .expect("no child config before admission");
    let Err(error) = discover_sender_approved(Box::new(plan)).await else {
        panic!("child fsmonitor allowed")
    };
    assert!(error.to_string().contains("submodule fsmonitor"));
    assert!(!child.join("helper-ran").exists());
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    std::fs::write(&included, "[core]\n fsmonitor = false\n").expect("disabled helper");
    let plan = AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native).expect("plan");
    let next = discover_sender_approved(Box::new(plan))
        .await
        .expect("disabled fsmonitor should discover");
    assert!(
        next.permission_profile()
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == included.to_str())
    );
    drop(next); // Declining continuation has no mutation effects.
    assert!(!child.join("helper-ran").exists());
}

#[test]
fn external_gitlink_marker_is_planned_without_reading_destination() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("external metadata");
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("checkout");
    let marker = child.join(".git");
    std::fs::write(&marker, format!("gitdir: {}\n", outside.path().display())).expect("marker");
    // Not a valid repository: preparation must still succeed, because the
    // destination's contents have no authorization yet.
    std::fs::write(outside.path().join("config"), "invalid config").expect("external config");
    let output = std::process::Command::new("git")
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ])
        .current_dir(root.path())
        .output()
        .expect("index");
    assert!(output.status.success());
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"parent"}),
        ToolProvenance::Native,
    )
    .expect("deferred external metadata");
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == outside.path().to_str())
    );
    std::fs::write(outside.path().join("config"), "different invalid config")
        .expect("external change");
    plan.revalidate()
        .expect("external content not read before admission");
    std::fs::write(&marker, "gitdir: other\n").expect("retarget marker");
    assert!(
        plan.revalidate().is_err(),
        "marker retarget must invalidate preparation"
    );
}

#[test]
fn commit_gitlink_preparation_defers_child_head_and_worktree_reads() {
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir(&child).expect("module");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    git(
        &child,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "first",
        ],
    );
    git(root.path(), &["add", "module", "file"]);
    let (paths, before) = tracked_commit_sources(root.path()).expect("sources");
    assert_eq!(paths, ["file"]);
    assert_eq!(before.len(), 1);
    assert!(before[0].metadata_path.is_some());
    std::fs::create_dir(child.join("untracked")).expect("untracked");
    std::fs::create_dir(child.join("untracked/.git")).expect("nested untracked marker");
    assert_eq!(
        before,
        tracked_commit_sources(root.path())
            .expect("unchanged metadata")
            .1
    );
    git(
        &child,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "second",
        ],
    );
    let after = tracked_commit_sources(root.path()).expect("new HEAD").1;
    assert_eq!(before[0].index_oid, after[0].index_oid);
    assert_eq!(
        before, after,
        "preparation must defer child HEAD reads until admission"
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"parent", "all":true}),
        ToolProvenance::Native,
    )
    .expect("plan with gitlink");
    assert_eq!(plan.gitlinks.len(), 1);
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref()
                == child
                    .join(".git")
                    .canonicalize()
                    .expect("canonical child metadata")
                    .to_str()
    }));
}

#[test]
fn commit_gitlink_preserves_missing_and_uninitialized_checkout_states() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ])
        .current_dir(root.path())
        .output()
        .expect("gitlink");
    assert!(output.status.success());
    let missing = tracked_commit_sources(root.path()).expect("missing").1;
    assert_eq!(missing[0].entry, snapshot::Entry::Missing);
    std::fs::create_dir(root.path().join("module")).expect("uninitialized");
    let empty = tracked_commit_sources(root.path()).expect("empty").1;
    assert!(matches!(empty[0].entry, snapshot::Entry::Directory(_)));
    assert!(empty[0].metadata_path.is_none());
    assert_ne!(missing, empty);
}

#[tokio::test]
async fn commit_all_deletes_replaced_tracked_file_and_refuses_nested_repository() {
    for nested in [false, true] {
        let root = fixture();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                // Finish fixture maintenance before retaining object-layout
                // evidence, matching the supervised production invocation.
                .args(["-c", "maintenance.autoDetach=false", "-c", "gc.autoDetach=false"])
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).expect("utf8")
        };
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["add", "file"]);
        git(&["commit", "-m", "initial"]);
        std::fs::remove_file(root.path().join("file")).expect("remove tracked file");
        std::fs::create_dir(root.path().join("file")).expect("replacement directory");
        if nested {
            git(&["init", "file"]);
        }
        std::fs::write(root.path().join("file/untracked"), "not a commit source")
            .expect("nested untracked");
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"delete replaced file", "all":true}),
            ToolProvenance::Native,
        )
        .expect("tracked-only plan");
        let head = git(&["rev-parse", "HEAD"]);
        let output = execute_approved(plan).await;
        if nested {
            assert!(
                output.result.is_error && output.result.content.contains("nested repository"),
                "{}",
                output.result.content
            );
            assert_eq!(git(&["rev-parse", "HEAD"]), head);
        } else {
            assert!(!output.result.is_error, "{}", output.result.content);
            assert!(git(&["ls-tree", "--name-only", "HEAD"]).is_empty());
        }
        assert_eq!(
            std::fs::read_to_string(root.path().join("file/untracked")).expect("preserved"),
            "not a commit source"
        );
    }
}

#[tokio::test]
async fn child_filter_command_uses_effective_root_without_discovery_prefix() {
    let root = fixture();
    let discovery = root.path().join("nested");
    std::fs::create_dir(&discovery).expect("discovery");
    assert!(
        std::process::Command::new("git")
            .current_dir(&discovery)
            .arg("init")
            .output()
            .expect("child init")
            .status
            .success()
    );
    std::fs::write(
        root.path().join(".gitattributes"),
        "tracked filter=root-filter\n",
    )
    .expect("root attributes");
    std::fs::write(discovery.join(".gitattributes"), "tracked -filter\n")
        .expect("discovery attributes");
    let plan = prepare(root.path());
    let command = plan
        .child_filter_command(&discovery.join(".git"), root.path(), &["tracked".into()])
        .expect("bound directory arguments");
    let output = plan
        .run_command(command, std::time::Duration::from_secs(10), 4096)
        .await
        .expect("real child filter probe");
    assert_eq!(output, b"tracked\0filter\0root-filter\0");
}

#[tokio::test]
async fn child_bound_config_discovery_does_not_resolve_configured_worktree() {
    let root = fixture();
    let child = root.path().join("child");
    assert!(
        std::process::Command::new("git")
            .current_dir(root.path())
            .args(["init", "child"])
            .output()
            .expect("init")
            .status
            .success()
    );
    let missing = root.path().join("unapproved-missing-worktree");
    assert!(
        std::process::Command::new("git")
            .current_dir(&child)
            .args(["config", "core.worktree"])
            .arg(&missing)
            .output()
            .expect("config")
            .status
            .success()
    );
    let plan = AddPlan::prepare_checkout(
        root.path(),
        &serde_json::json!({"branch":"HEAD"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = plan
        .inspect_child_bootstrap_config(&child.join(".git"), &child)
        .await
        .expect("bound probe");
    assert!(
        output
            .windows(missing.as_os_str().len())
            .any(|value| { value == missing.to_string_lossy().as_bytes() })
    );
    assert!(!missing.exists());
}

#[tokio::test]
async fn real_commit_runs_configured_automatic_maintenance() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["config", "maintenance.auto", "true"]);
    git(&["config", "maintenance.gc.enabled", "false"]);
    git(&["config", "maintenance.loose-objects.enabled", "true"]);
    git(&["config", "maintenance.loose-objects.auto", "1"]);
    git(&["add", "file"]);
    let pack_dir = root.path().join(".git/objects/pack");
    assert_eq!(
        std::fs::read_dir(&pack_dir)
            .expect("pack directory")
            .count(),
        0
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"maintenance"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert!(
        std::fs::read_dir(&pack_dir)
            .expect("packs")
            .any(|entry| entry
                .expect("entry")
                .path()
                .extension()
                .is_some_and(|extension| extension == "pack")),
        "automatic loose-object maintenance must produce a pack before completion"
    );
}

#[tokio::test]
async fn real_commit_preserves_index_semantics_and_all_excludes_untracked() {
    let root = fixture();
    let git = |args: &[&str]| {
        let result = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("git");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).expect("utf8")
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    std::fs::write(root.path().join("file"), "unstaged").expect("edit");
    std::fs::write(root.path().join("untracked"), "private").expect("untracked");
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"index only"}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["show", "HEAD:file"]), "before");
    assert_eq!(
        std::fs::read_to_string(root.path().join("file")).expect("worktree"),
        "unstaged"
    );
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"tracked", "all":true}),
        ToolProvenance::Native,
    )
    .expect("all plan");
    let output = execute_approved(plan).await;
    assert!(!output.result.is_error, "{}", output.result.content);
    assert_eq!(git(&["show", "HEAD:file"]), "unstaged");
    assert_eq!(git(&["ls-tree", "--name-only", "HEAD"]), "file\n");
    let head = git(&["rev-parse", "HEAD"]);
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"no changes"}),
        ToolProvenance::Native,
    )
    .expect("no-op plan");
    assert!(execute_approved(plan).await.result.is_error);
    assert_eq!(git(&["rev-parse", "HEAD"]), head);
}

#[tokio::test]
async fn commit_plan_binds_tracked_sources_and_rejects_add_proof() {
    let root = fixture();
    let output = std::process::Command::new("git")
        .args(["add", "file"])
        .current_dir(root.path())
        .output()
        .expect("stage");
    assert!(output.status.success());
    std::fs::write(root.path().join("untracked"), "not selected").expect("untracked");
    let commit = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"first"}),
        ToolProvenance::Native,
    )
    .expect("commit plan");
    assert_eq!(commit.paths, ["file"]);
    assert_eq!(commit.request().tool_name(), "git_commit");
    assert!(commit.facets.iter().any(|facet| {
        facet
            .description
            .as_deref()
            .is_some_and(|description| description.contains("maintenance.autoDetach=false"))
    }));
    let add = prepare(root.path());
    let result = Box::new(commit).execute_stage(Box::new(proof(&add))).await;
    let PreparedNormalStage::Complete(result) = result else {
        panic!("exchanged proof accepted")
    };
    assert!(result.result.is_error);
    let tool = super::super::GitCommitTool::new(root.path().to_owned());
    assert!(
        tool.execute(serde_json::json!({"message":"first"}))
            .await
            .is_error
    );
}

#[tokio::test]
async fn commit_inspects_parent_filters_before_mutation_in_both_modes() {
    for all in [false, true] {
        let root = fixture();
        assert!(
            std::process::Command::new("git")
                .args(["add", "file"])
                .current_dir(root.path())
                .status()
                .expect("stage")
                .success()
        );
        std::fs::write(root.path().join(".gitattributes"), "file filter=probe\n")
            .expect("attributes");
        assert!(
            std::process::Command::new("git")
                .args(["config", "filter.probe.clean", "touch helper-ran; cat"])
                .current_dir(root.path())
                .status()
                .expect("filter")
                .success()
        );
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let output = execute_approved(
            AddPlan::prepare_commit(
                root.path(),
                &serde_json::json!({"message":"filtered", "all":all}),
                ToolProvenance::Native,
            )
            .expect("plan"),
        )
        .await;
        assert!(
            output.result.is_error && output.result.content.contains("configured filter"),
            "{}",
            output.result.content
        );
        assert!(!root.path().join("helper-ran").exists());
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
        assert_eq!(
            index,
            std::fs::read(root.path().join(".git/index")).expect("preserved index")
        );
    }
}

#[tokio::test]
async fn rerere_refuses_active_records_but_preserves_inert_and_disabled_configurations() {
    for (enabled, cache, records, refused) in [
        (Some("true"), false, false, false),
        (Some("true"), true, true, true),
        (Some("false"), true, true, false),
        (None, true, true, true),
        (None, false, true, false),
    ] {
        let root = fixture();
        if let Some(value) = enabled {
            assert!(
                std::process::Command::new("git")
                    .args(["config", "rerere.enabled", value])
                    .current_dir(root.path())
                    .status()
                    .expect("config")
                    .success()
            );
        }
        if cache {
            std::fs::create_dir(root.path().join(".git/rr-cache")).expect("cache");
        }
        if records {
            std::fs::write(root.path().join(".git/MERGE_RR"), "active record").expect("record");
        }
        let plan = AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"first"}),
            ToolProvenance::Native,
        )
        .expect("plan");
        match discover_sender_approved(Box::new(plan)).await {
            Ok(_) => assert!(!refused),
            Err(error) => {
                assert!(refused, "{error}");
                assert!(error.to_string().contains("rerere replay"));
            }
        }
        assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
    }
}

#[test]
fn commit_refuses_unmerged_index_before_discovery() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("UTF8")
    };
    let oid = git(&["hash-object", "-w", "file"]);
    use std::io::Write;
    let mut child = std::process::Command::new("git")
        .args(["update-index", "--index-info"])
        .current_dir(root.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("index fixture");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(format!("100644 {} 2\tfile\n", oid.trim()).as_bytes())
        .expect("unmerged entry");
    assert!(child.wait().expect("index").success());
    let error = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"conflicted"}),
        ToolProvenance::Native,
    )
    .err()
    .expect("unmerged refused");
    assert!(error.to_string().contains("unmerged"));
}

#[tokio::test]
async fn commit_discovery_defers_hooks_and_ignores_inactive_template_and_tag_signing() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["config", "commit.template", "/unavailable/unused-template"]);
    git(&["config", "tag.gpgSign", "true"]);
    git(&["config", "commit.gpgSign", "false"]);
    let plan = AddPlan::prepare_commit(
        root.path(),
        &serde_json::json!({"message":"first"}),
        ToolProvenance::Native,
    )
    .expect("commit");
    let next = discover_sender_approved(Box::new(plan))
        .await
        .expect("ordinary config discovery refused");
    for name in [
        "pre-commit",
        "prepare-commit-msg",
        "commit-msg",
        "post-commit",
        "reference-transaction",
        "pre-auto-gc",
    ] {
        assert!(next.permission_profile().iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet
                    .resource
                    .as_deref()
                    .is_some_and(|resource| resource.ends_with(name))
        }));
    }
    assert!(
        !next
            .permission_profile()
            .iter()
            .any(|facet| facet.resource.as_deref() == Some("/unavailable/unused-template"))
    );
    drop(next);
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn dynamic_child_attribute_target_requires_fresh_read_and_retains_identity() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    let child = root.path().join("module");
    std::fs::create_dir_all(child.join("nested")).expect("child");
    let git = |cwd: &Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&child, &["init"]);
    std::fs::write(child.join("nested/tracked"), "data").expect("tracked");
    git(&child, &["add", "nested/tracked"]);
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            "1111111111111111111111111111111111111111",
            "module",
        ],
    );
    let outside = tempfile::tempdir().expect("external attributes");
    let target = outside
        .path()
        .canonicalize()
        .expect("external root")
        .join("attributes");
    std::fs::write(&target, "* -filter\n").expect("attributes");
    let spelling = child.join("nested/.gitattributes");
    symlink(&target, &spelling).expect("external attribute link");
    let plan = discover_sender_approved(Box::new(
        AddPlan::prepare_commit(
            root.path(),
            &serde_json::json!({"message":"parent"}),
            ToolProvenance::Native,
        )
        .expect("plan"),
    ))
    .await
    .expect("sender discovery");
    let old = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("index admission");
    let mut plan = plan
        .discover_child_attribute_targets(&old)
        .expect("index-selected paths");
    assert!(
        plan.facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Read
                && facet.resource.as_deref() == target.to_str())
    );
    let auxiliary = plan.auxiliary_paths.as_mut().expect("resolver");
    assert!(auxiliary.leaf_evidence(&spelling).is_none());
    assert!(
        auxiliary.capture_admitted(&old, &[]).is_err(),
        "old frontier cannot read external attribute body"
    );
    let fresh = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("target admission");
    let auxiliary = plan.auxiliary_paths.as_mut().expect("resolver");
    auxiliary
        .capture_admitted(&fresh, &[])
        .expect("approved bodies");
    auxiliary.revalidate(&fresh).expect("unchanged");
    std::fs::rename(&target, outside.path().join("retained")).expect("retain inode");
    std::fs::write(&target, "* -filter\n").expect("same content replacement");
    assert!(auxiliary.revalidate(&fresh).is_err());
    assert!(!root.path().join(".git/COMMIT_EDITMSG").exists());
}
