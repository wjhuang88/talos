#[test]
fn pull_attribute_evidence_detects_creation_and_replacement() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().canonicalize().expect("root").join("attributes");
    let budget = || snapshot::Budget {
        entries: 32,
        bytes: 4096,
    };
    let missing = PullAttribute::capture(&path, &mut budget()).expect("missing");
    missing.revalidate(&mut budget()).expect("unchanged");
    std::fs::write(&path, "* -filter\n").expect("create");
    assert!(missing.revalidate(&mut budget()).is_err());
    let present = PullAttribute::capture(&path, &mut budget()).expect("present");
    std::fs::write(&path, "* filter=external\n").expect("change");
    assert!(present.revalidate(&mut budget()).is_err());
}

#[cfg(unix)]
#[test]
fn pull_attribute_evidence_refuses_parent_and_leaf_links() {
    let root = tempfile::tempdir().expect("fixture");
    let root = root.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("rules")).expect("directory");
    let path = root.join("rules/attributes");
    std::fs::write(&path, "* -filter\n").expect("rules");
    let budget = || snapshot::Budget {
        entries: 32,
        bytes: 4096,
    };
    let evidence = PullAttribute::capture(&path, &mut budget()).expect("capture");
    std::fs::rename(root.join("rules"), root.join("moved")).expect("move");
    std::os::unix::fs::symlink(root.join("moved"), root.join("rules")).expect("parent link");
    assert!(evidence.revalidate(&mut budget()).is_err());
    assert!(PullAttribute::capture(&path, &mut budget()).is_err());
    let leaf = root.join("leaf");
    std::os::unix::fs::symlink(root.join("moved/attributes"), &leaf).expect("leaf link");
    assert!(PullAttribute::capture(&leaf, &mut budget()).is_err());
}

#[test]
fn pull_source_evidence_detects_content_and_missing_path_changes() {
    let root = tempfile::tempdir().expect("fixture");
    std::fs::write(root.path().join("file"), "before").expect("source");
    let evidence =
        PullSources::capture(root.path(), vec!["file".into(), "missing".into()]).expect("capture");
    evidence.revalidate(root.path()).expect("unchanged");
    std::fs::write(root.path().join("file"), "after").expect("change");
    assert!(evidence.revalidate(root.path()).is_err());
    std::fs::write(root.path().join("file"), "before").expect("restore");
    std::fs::write(root.path().join("missing"), "created").expect("create");
    assert!(evidence.revalidate(root.path()).is_err());
}

#[cfg(unix)]
#[test]
fn pull_source_evidence_refuses_redirected_parent() {
    let root = tempfile::tempdir().expect("fixture");
    let outside = tempfile::tempdir().expect("outside");
    std::fs::create_dir(root.path().join("dir")).expect("directory");
    std::fs::write(root.path().join("dir/file"), "same").expect("source");
    std::fs::write(outside.path().join("file"), "same").expect("outside source");
    let evidence = PullSources::capture(root.path(), vec!["dir/file".into()]).expect("capture");
    std::fs::rename(root.path().join("dir"), root.path().join("old")).expect("move");
    std::os::unix::fs::symlink(outside.path(), root.path().join("dir")).expect("redirect");
    assert!(evidence.revalidate(root.path()).is_err());
    assert!(PullSources::capture(root.path(), vec!["dir/file".into()]).is_err());
}

#[test]
fn pull_sparse_tree_listing_rejects_malformed_and_oversize_records() {
    let prefix = b"dir/";
    let good = b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tfile\0";
    assert_eq!(
        pull_sparse_names(prefix, good).expect("good"),
        b"dir/file\0"
    );
    for bad in [
        b"100644 blob zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\tfile\0".as_slice(),
        b"100644 blob 0000000000000000000000000000000000000000\tfile\0",
        b"100644 tree aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tdir\0",
        b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t../file\0",
        b"100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\tfile",
    ] {
        assert!(pull_sparse_names(prefix, bad).is_err());
    }
    let large = vec![b'x'; 16 * 1024 * 1024 + 1];
    assert!(pull_sparse_names(prefix, &large).is_err());
}

#[test]
fn pull_tracked_inventory_only_constructs_literal_paths() {
    let root = tempfile::tempdir().expect("fixture");
    let names = pull_tracked_read_paths(root.path(), b"missing/leaf\0-leading\0").expect("names");
    assert!(names.contains(&root.path().join("missing/leaf")));
    assert!(!root.path().join("missing").exists());
    for bad in [
        b"../outside\0".as_slice(),
        b"/absolute\0",
        b"a//b\0",
        b"a/./b\0",
        b"unterminated",
    ] {
        assert!(pull_tracked_read_paths(root.path(), bad).is_err());
    }
}

#[tokio::test]
async fn pull_config_admission_cannot_authorize_graph_probes() {
    let root = fixture();
    let input = serde_json::json!({});
    let tool = super::super::GitPullTool::new(root.path().to_path_buf());
    let prepared = tool
        .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
        .expect("prepare")
        .expect("prepared Pull");
    #[cfg(unix)]
    assert!(prepared.cleanup_receipt().is_some());
    let PreparedNormalStage::Complete(denied) = prepared.execute_stage(Box::new(())).await else {
        panic!("missing admission continued")
    };
    assert!(denied.result.is_error);

    let plan = prepare_pull_fixture(root.path(), &input, ToolProvenance::Native).expect("Pull");
    let old_proof = proof(&plan);
    let plan = Box::new(plan).discover().await.expect("configuration");
    let PreparedNormalStage::Complete(output) = plan.execute_stage(Box::new(old_proof)).await
    else {
        panic!("configuration proof authorized graph inspection")
    };
    assert!(output.result.is_error && output.result.content.contains("does not authorize"));

    let plan = prepare_pull_fixture(root.path(), &input, ToolProvenance::Native).expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("config");
    assert!(plan.pull_evidence.is_none());
    assert!(plan.facets.iter().any(|facet| {
        facet.nature == ToolNature::Read
            && facet.resource.as_deref() == plan.repository.common_dir.join("info/grafts").to_str()
    }));
    for _ in 0..5 {
        assert!(history.record_profile(&plan.identity, &plan.facets));
        let admitted = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh exact inventory admission");
        plan = plan
            .discover_pull_prefetch(&admitted)
            .await
            .expect("inventory/prefetch");
        if plan.phase == Phase::PullFetch {
            break;
        }
        if plan
            .pull_graph_paths
            .as_ref()
            .is_some_and(|paths| paths.contains(&plan.repository.common_dir.join("info/grafts")))
        {
            assert!(plan.pull_evidence.is_none());
            assert!(plan.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource.as_deref()
                        == plan.repository.common_dir.join("info/grafts").to_str()
            }));
        }
        assert!(plan.pull_before.is_none(), "no probes during inventory");
    }
    assert!(plan.phase == Phase::PullFetch);
    assert!(history.record_profile(&plan.identity, &plan.facets));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_sparse_index_requires_object_discovery_before_source_capture() {
    let root = fixture();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .expect("Git fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    std::fs::create_dir(root.path().join("kept")).expect("kept");
    std::fs::write(root.path().join("kept/child"), "child").expect("child");
    std::fs::create_dir(root.path().join("excluded")).expect("excluded");
    std::fs::write(root.path().join("excluded/child"), "outside cone").expect("excluded child");
    // More than 16 KiB of ls-tree output; identical blobs keep the object
    // inventory small so this exercises listing capacity, not object count.
    for number in 0..300 {
        std::fs::write(
            root.path().join(format!("excluded/entry-{number:04}")),
            "outside cone",
        )
        .expect("large sparse tree entry");
    }
    git(&["add", "."]);
    git(&["commit", "-m", "child"]);
    git(&["sparse-checkout", "init", "--cone"]);
    git(&["sparse-checkout", "set", "--sparse-index", "kept"]);
    git(&["config", "index.sparse", "true"]);
    git(&["config", "pull.rebase", "true"]);
    git(&["branch", "-m", "Topic"]);
    git(&["config", "remote.origin.url", "/missing-fixture-remote"]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    git(&["config", "branch.Topic.remote", "origin"]);
    git(&["config", "branch.Topic.merge", "refs/heads/Topic"]);
    git(&["update-ref", "refs/remotes/origin/Topic", "HEAD"]);
    let index = super::super::pull_state::decode_index(&root.path().join(".git/index"), 40)
        .expect("sparse index")
        .0;
    assert!(
        index
            .entries()
            .iter()
            .any(|entry| entry.mode.bits() == 0o040000)
    );
    let plan = prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let original_index = std::fs::read(root.path().join(".git/index")).expect("index bytes");
    let mut saw_descendant_continuation = false;
    let mut stages = 1;
    let mut stage_trace = Vec::new();
    for _ in 0..8 {
        assert!(history.record_profile(&plan.identity, &plan.facets));
        stages += 1;
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("admission");
        match plan.discover_pull_prefetch(&admission).await {
            Ok(next) => {
                stage_trace.push((
                    stages,
                    next.pull_graph_paths.is_some(),
                    next.pull_inventory
                        .as_ref()
                        .is_some_and(|inventory| inventory.complete()),
                    next.facets.len(),
                ));
                let descendant = next.repository.workdir.join("excluded/child");
                if next
                    .facets
                    .iter()
                    .any(|facet| facet.resource.as_deref() == descendant.to_str())
                    && !admission.iter().any(|token| {
                        token.authorizes_path(
                            "git_pull",
                            ToolNature::Read,
                            &next.workspace,
                            descendant.to_str().expect("path"),
                        )
                    })
                {
                    assert!(next.pull_sources.is_none());
                    assert!(next.phase == Phase::PullPreparation);
                    saw_descendant_continuation = true;
                }
                if next.phase == Phase::PullFetch {
                    assert!(
                        stages <= 6,
                        "sparse preparation must reserve fetch and integration: {stages}; trace {stage_trace:?}"
                    );
                    assert!(saw_descendant_continuation);
                    assert!(next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == Some("git:pull-repository-metadata")
                    }));
                    assert!(next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == next.repository.git_dir.to_str()
                    }));
                    assert!(!next.facets.iter().any(|facet| {
                        facet.nature == ToolNature::Write
                            && facet.resource.as_deref() == next.repository.workdir.to_str()
                    }));
                    assert!(history.record_profile(&next.identity, &next.facets));
                    assert!(
                        history.record_profile(&PreparedPlanIdentity::new(), &next.facets),
                        "integration must fit actual shared history after fetch reservation"
                    );
                    assert_eq!(
                        std::fs::read(root.path().join(".git/index")).expect("index after"),
                        original_index
                    );
                    assert!(next.pull_graph_paths.as_ref().is_some_and(|paths| {
                        paths.iter().any(|path| path.ends_with("excluded/child"))
                    }));
                    return;
                }
                plan = next
            }
            Err(error) => {
                panic!("sparse expansion failed: {error}");
            }
        }
    }
    panic!("sparse index did not reach prepared fetch");
}

#[tokio::test]
async fn pull_graph_read_requires_closed_capability_after_configuration() {
    let root = fixture();
    let plan = prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    plan.facets
        .retain(|facet| facet.resource.as_deref() != Some("git:pull-repository-metadata"));
    let admission = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("exact-path admission without closed capability");
    let Err(error) = plan.discover_pull_prefetch(&admission).await else {
        panic!("missing metadata capability continued");
    };
    assert!(
        error
            .to_string()
            .contains("dedicated metadata Read admission")
    );
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
}

#[tokio::test]
async fn pull_unapproved_resolution_does_not_inspect_missing_external_target() {
    let root = fixture();
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    let common = plan.repository.common_dir.clone();
    let registration = common.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    let missing = root.path().join("missing-external-common");
    std::fs::write(
        registration.join("commondir"),
        format!("{}\n", missing.display()),
    )
    .expect("pointer");
    let mut inventory = super::super::pull_inventory::Inventory::new(&common, &common);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("approved metadata");
    }
    inventory.bind_references(&common, &common).expect("source evidence");
    for _ in 0..2 {
        assert!(AddPlan::advance_pull_worktree_targets(
            &mut inventory,
            &common,
            &mut plan.facets,
            &plan.workspace,
            &[],
            false,
        ).expect("unapproved capability must defer without inspecting missing target"));
        assert!(inventory.exact_paths(&common, &common).is_empty());
    }
    assert!(AddPlan::advance_pull_worktree_targets(
        &mut inventory,
        &common,
        &mut plan.facets,
        &plan.workspace,
        &[],
        true,
    ).is_err(), "approved resolution detects the missing common directory");
}

#[tokio::test]
async fn pull_parent_linked_targets_require_fresh_exact_admission() {
    let root = fixture();
    let mut plan = Box::new(
        prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
            .expect("Pull"),
    )
    .discover()
    .await
    .expect("configuration");
    let common = plan.repository.common_dir.clone();
    let registration = common.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    std::fs::write(registration.join("gitdir"), "/absent-checkout/.git\n")
        .expect("absolute backlink remains text");
    let external = tempfile::tempdir().expect("external common");
    let external = external.path().canonicalize().expect("canonical external");
    std::fs::create_dir_all(external.join("refs/heads")).expect("external refs");
    let reference = external.join("refs/heads/topic");
    std::fs::write(&reference, format!("{}\n", "1".repeat(40))).expect("reference");
    std::fs::write(
        registration.join("commondir"),
        format!("{}\n", external.display()),
    )
    .expect("common pointer");
    let mut inventory = super::super::pull_inventory::Inventory::new(&common, &common);
    let old = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("original grants");
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &old)
            .expect("targets")
    );
    assert!(AddPlan::pull_resolution_admitted(&plan.facets));
    assert!(inventory.exact_paths(&common, &common).is_empty());
    assert!(!inventory.paths().contains(&reference));
    // Only the newly approved dedicated capability permits chain inspection.
    let resolution = proof(&plan)
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("resolution capability");
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &resolution)
            .expect("resolved target plan")
    );
    assert!(inventory.exact_paths(&common, &common).contains(&external));
    // Capability approval does not grant the newly resolved target's contents.
    assert!(
        !plan
            .advance_pull_parent_inventory(&mut inventory, &resolution)
            .expect("target still lacks exact admission")
    );
    assert!(!inventory.paths().contains(&reference));
    for _ in 0..8 {
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh frontier grants");
        if plan
            .advance_pull_parent_inventory(&mut inventory, &tokens)
            .expect("advance")
        {
            break;
        }
    }
    assert!(inventory.complete());
    assert!(inventory.paths().contains(&reference));
    inventory.revalidate().expect("bound contents");
    std::fs::write(&reference, format!("{}\n", "2".repeat(40))).expect("change contents");
    assert!(inventory.revalidate().is_err());
}

#[tokio::test]
async fn pull_tracking_inventory_uses_shared_stage_budget() {
    let root = fixture();
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
    git(&["symbolic-ref", "HEAD", "refs/heads/Topic"]);
    git(&["config", "user.name", "Fixture"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "commit.gpgSign", "false"]);
    git(&["add", "file"]);
    git(&["commit", "-m", "base"]);
    git(&["config", "remote.origin.url", "/missing-fixture-remote"]);
    git(&[
        "config",
        "remote.origin.fetch",
        "+refs/heads/*:refs/remotes/origin/*",
    ]);
    git(&["config", "branch.Topic.remote", "origin"]);
    git(&["config", "branch.Topic.merge", "refs/heads/Topic"]);
    git(&["config", "pull.rebase", "true"]);
    git(&["update-ref", "refs/remotes/origin/Topic", "HEAD"]);
    // Namespace depth must not consume prepared-runner stages now that
    // graph inspection uses the dedicated metadata Read capability.
    git(&["update-ref", "refs/heads/a/b/c/d/e/f/g/h/i/j/deep", "HEAD"]);
    let absent_tracking_log = root.path().join(".git/logs/refs/remotes/origin/Topic");
    std::fs::remove_file(&absent_tracking_log).expect("absent tracking reflog");
    let plan = prepare_pull_fixture(root.path(), &serde_json::json!({}), ToolProvenance::Native)
        .expect("Pull");
    let mut history = talos_core::tool::PreparedStageHistory::default();
    assert!(history.record_profile(&plan.identity, &plan.facets));
    let mut plan = Box::new(plan).discover().await.expect("configuration");
    let mut stages = 1;
    while plan.phase != Phase::PullFetch {
        assert!(
            history.record_profile(&plan.identity, &plan.facets),
            "inventory exhausted stage budget"
        );
        stages += 1;
        let admission = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("fresh admission");
        plan = plan
            .discover_pull_prefetch(&admission)
            .await
            .expect("inventory");
    }
    assert!(history.record_profile(&plan.identity, &plan.facets));
    assert!(
        stages <= 3,
        "post-fetch discovery, reconciliation and integration stages must remain available"
    );
    for path in [
        plan.repository.workdir.join(".gitattributes"),
        plan.repository.common_dir.join("info/attributes"),
    ] {
        assert!(
            plan.facets
                .iter()
                .any(|facet| facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Path)
                    && facet.resource.as_deref() == path.to_str())
        );
    }
    assert!(history.record_profile(&PreparedPlanIdentity::new(), &plan.facets));
    assert!(!root.path().join(".git/FETCH_HEAD").exists());
    let reference = plan.repository.common_dir.join("refs/heads/Topic");
    let original_reference = std::fs::read(&reference).expect("reference");
    std::fs::write(&reference, format!("{}\n", "1".repeat(40))).expect("change reference in place");
    assert!(
        plan.revalidate().is_err(),
        "reference contents must remain bound"
    );
    std::fs::write(&reference, original_reference).expect("restore reference");
    plan.revalidate().expect("restored reference evidence");
    std::fs::write(&absent_tracking_log, b"new evidence").expect("create reflog");
    assert!(
        plan.revalidate().is_err(),
        "absent reflog must remain bound"
    );
}
