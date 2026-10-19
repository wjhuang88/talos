#[tokio::test]
async fn pull_reconciliation_returns_fresh_approval_before_final_merge() {
    for scenario in [
        "approve",
        "deny",
        "fetch-head",
        "config",
        "head",
        "metadata-attribute",
    ] {
        let root = fixture();
        let git = |args: &[&str]| integration_git(root.path(), args);
        git(&["add", "file"]);
        git(&["commit", "-m", "base"]);
        let original = git(&["rev-parse", "HEAD"]);
        std::fs::write(root.path().join("file"), "remote content").expect("content");
        std::fs::write(root.path().join("new-file"), "new tracked content").expect("new source");
        git(&["add", "."]);
        git(&["commit", "-m", "target"]);
        let current = git(&["rev-parse", "HEAD"]);
        git(&["reset", "--hard", &original]);
        let before = super::super::pull_state::BeforeFetch::probe(
            b"pull.rebase\nfalse\0",
            None,
            40,
            |args| {
                let output = std::process::Command::new("git")
                    .current_dir(root.path())
                    .args(args)
                    .output()
                    .expect("probe");
                async move { Ok(output) }
            },
        )
        .await
        .expect("before fetch");
        // Match fetch --update-head-ok: HEAD moves while index/worktree stay old.
        git(&["update-ref", "HEAD", &current]);
        let mut plan = integration_scope_fixture(root.path()).await;
        std::fs::write(
            root.path().join(".git/FETCH_HEAD"),
            format!("{current}\t\tfixture\n"),
        )
        .expect("fetched");
        plan.repository = RepositoryState::capture(root.path()).expect("repository");
        plan.pull_evidence = Some(
            super::super::pull_state::PullEvidence::capture(&plan.repository, None, true)
                .expect("evidence"),
        );
        let metadata_attribute = plan.repository.common_dir.join("info/attributes");
        plan.pull_attributes.push(
            PullAttribute::capture(
                &metadata_attribute,
                &mut snapshot::Budget {
                    entries: 1,
                    bytes: 4096,
                },
            )
            .expect("metadata attribute evidence"),
        );
        plan.pull_before = Some(before);
        plan.pull_remote = Some("origin".into());
        plan.pull_sources = Some(
            PullSources::capture(&plan.repository.workdir, vec!["file".into()])
                .expect("old sources"),
        );
        plan.pull_integration = Some(super::super::pull_state::Integration::Reconcile {
            original,
            current: current.clone(),
        });
        let plan = plan
            .discover_pull_integration_hooks()
            .expect("reconcile scope");
        if matches!(
            scenario,
            "fetch-head" | "config" | "head" | "metadata-attribute"
        ) {
            // Exercise the post-command observation boundary directly: mutations
            // here must not be accepted as effects of read-tree.
            plan.execute_pull_integration()
                .await
                .expect("reconciliation");
            let expected = match scenario {
                "fetch-head" => {
                    std::fs::write(root.path().join(".git/FETCH_HEAD"), b"changed\n")
                        .expect("replace fetched evidence");
                    "FETCH_HEAD changed"
                }
                "metadata-attribute" => {
                    std::fs::write(
                        root.path().join(".git/info/attributes"),
                        b"* filter=unexpected\n",
                    )
                    .expect("replace metadata attribute");
                    "attribute changed"
                }
                "config" => {
                    git(&["config", "pull.ff", "only"]);
                    "configuration"
                }
                "head" => {
                    git(&["update-ref", "HEAD", &git(&["rev-parse", "HEAD^"])]);
                    "HEAD changed"
                }
                _ => unreachable!(),
            };
            let error = match plan.refresh_reconciled_pull().await {
                Ok(_) => panic!("{scenario} drift was rebaselined"),
                Err(error) => error.to_string(),
            };
            assert!(error.contains(expected), "{scenario}: {error}");
            continue;
        }
        let stale = proof(&plan);
        let admission = proof(&plan);
        let PreparedNormalStage::Continue(next) = plan.execute_stage(Box::new(admission)).await
        else {
            panic!("successful reconciliation must return a new approval stage");
        };
        assert_eq!(
            std::fs::read(root.path().join("file")).expect("reconciled"),
            b"remote content"
        );
        assert_eq!(
            std::fs::read(root.path().join("new-file")).expect("new source"),
            b"new tracked content"
        );
        if scenario == "deny" {
            let PreparedNormalStage::Complete(output) = next.execute_stage(Box::new(stale)).await
            else {
                panic!("stale proof continued")
            };
            assert!(output.result.is_error);
            assert!(output.result.content.contains("does not authorize"));
            continue;
        }
        let mut next = next;
        let mut stages = 0;
        loop {
            stages += 1;
            assert!(stages <= 2, "no repeated fetch or reconciliation");
            let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
                root.path().into(),
            ));
            let context = PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            );
            let input = serde_json::json!({});
            let request = PermissionRequest::native("git_pull", next.permission_profile(), &input);
            let proposal = state
                .propose(&request, &context, GrantScope::Once)
                .expect("proposal");
            let pending = state
                .approve_once(proposal, &request, &context)
                .expect("approval");
            let admission = state
                .try_admit_prepared_invocation(
                    pending,
                    &request,
                    &context,
                    next.plan_identity().expect("identity"),
                )
                .expect("admission");
            match next.execute_stage(Box::new(admission)).await {
                PreparedNormalStage::Continue(plan) => next = plan,
                PreparedNormalStage::Complete(output) => {
                    assert!(!output.result.is_error, "{}", output.result.content);
                    assert_eq!(git(&["rev-parse", "HEAD"]), current);
                    break;
                }
            }
        }
    }
}
