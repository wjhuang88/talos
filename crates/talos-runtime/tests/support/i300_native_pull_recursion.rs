//! Real recursive Pull through Runtime, Agent, Auto and supervised Git.
use super::*;

#[tokio::test]
async fn runtime_auto_pull_updates_initialized_submodule() {
    let root = tempfile::tempdir().expect("fixture");
    let child = root.path().join("child");
    let source = root.path().join("source");
    let work = root.path().join("work");
    let baseline = root.path().join("native-baseline");
    for path in [&child, &source] {
        std::fs::create_dir(path).expect("repository");
        git(path, &["init", "-b", "main"]);
        for (key, value) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgSign", "false"),
            ("core.autocrlf", "false"),
        ] {
            git(path, &["config", key, value]);
        }
    }
    std::fs::write(child.join("file"), "before\n").expect("child file");
    git(&child, &["add", "file"]);
    git(&child, &["commit", "-m", "base"]);
    git(
        &source,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            child.to_str().expect("child path"),
            "nested",
        ],
    );
    git(&source, &["commit", "-am", "parent base"]);
    git(
        root.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "clone",
            "--recurse-submodules",
            source.to_str().expect("source path"),
            work.to_str().expect("work path"),
        ],
    );
    git(&work, &["config", "submodule.recurse", "true"]);
    git(&work, &["config", "protocol.file.allow", "always"]);
    git(&work.join("nested"), &["config", "core.autocrlf", "false"]);
    git(
        root.path(),
        &[
            "-c",
            "protocol.file.allow=always",
            "clone",
            "--recurse-submodules",
            source.to_str().expect("source path"),
            baseline.to_str().expect("baseline path"),
        ],
    );
    git(&baseline, &["config", "submodule.recurse", "true"]);
    git(&baseline, &["config", "protocol.file.allow", "always"]);
    git(
        &baseline.join("nested"),
        &["config", "core.autocrlf", "false"],
    );
    for checkout in [&work, &baseline] {
        git(checkout, &["config", "core.autocrlf", "false"]);
        assert_eq!(
            std::fs::read(checkout.join("nested/file")).expect("initial child bytes"),
            b"before\n"
        );
        assert!(git(&checkout.join("nested"), &["status", "--porcelain"]).is_empty());
    }
    std::fs::write(child.join("file"), "after\n").expect("update");
    git(&child, &["commit", "-am", "child update"]);
    let expected_child = git(&child, &["rev-parse", "HEAD"]);
    git(&source.join("nested"), &["fetch"]);
    git(&source.join("nested"), &["checkout", "origin/main"]);
    git(&source, &["commit", "-am", "parent update"]);
    // Keep native Git failures distinguishable from the Runtime execution path.
    // The fixture helper reports captured stderr on failure.
    git(&baseline, &["pull"]);
    assert_eq!(
        git(&baseline.join("nested"), &["rev-parse", "HEAD"]),
        expected_child
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hook = work.join(".git/modules/nested/hooks/post-checkout");
        std::fs::write(&hook, "#!/bin/sh\nprintf observed > helper-marker\n").expect("hook");
        std::fs::set_permissions(hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    }
    let reviews = Arc::new(AtomicUsize::new(0));
    let approvals = Arc::new(AtomicUsize::new(0));
    let mut runtime = RuntimeBuilder::new()
        .workspace_root(&work)
        .provider(Arc::new(Model {
            turns: AtomicUsize::new(0),
            reviews: reviews.clone(),
            allowed: true,
            assessor_failure: false,
        }))
        .tool(Arc::new(talos_tools::git::GitPullTool::new(work.clone())))
        .approval_handler(Arc::new(Human {
            final_approvals: approvals.clone(),
            allow_final: false,
            drift_repository: None,
        }))
        .permission_mode(PermissionMode::Interactive)
        .auto_assistance(true)
        .build()
        .expect("runtime");
    runtime
        .submit("update this repository and its submodules from upstream")
        .await
        .expect("submit");
    let terminal = tokio::time::timeout(
        Duration::from_secs(30),
        collect_until_turn_completed(&mut runtime),
    )
    .await
    .expect("deadline")
    .expect("terminal");
    assert!(matches!(terminal, TurnCompletionStatus::Success { .. }));
    assert_eq!(reviews.load(Ordering::SeqCst), 1);
    assert_eq!(approvals.load(Ordering::SeqCst), 0);
    assert_eq!(
        git(&work, &["rev-parse", "HEAD"]),
        git(&source, &["rev-parse", "HEAD"])
    );
    assert_eq!(
        git(&work.join("nested"), &["rev-parse", "HEAD"]),
        expected_child
    );
    assert_eq!(
        std::fs::read(work.join("nested/file")).expect("child contents"),
        b"after\n"
    );
    #[cfg(unix)]
    assert_eq!(
        std::fs::read(work.join("nested/helper-marker")).expect("helper effect"),
        b"observed"
    );
    runtime.shutdown().await.expect("shutdown");
}
