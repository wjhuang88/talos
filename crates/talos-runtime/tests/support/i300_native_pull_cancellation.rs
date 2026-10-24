//! Running native Pull interruption through the public Runtime API.
use super::*;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn interrupt_running_pull_closes_hook_descendants_and_preserves_completed_effects() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("source");
    let work = root.path().join("work");
    std::fs::create_dir(&source).expect("source");
    git(&source, &["init", "-b", "main"]);
    for (key, value) in [
        ("user.name", "Fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgSign", "false"),
        ("core.autocrlf", "false"),
    ] {
        git(&source, &["config", key, value]);
    }
    std::fs::write(source.join("file"), "before\n").expect("file");
    git(&source, &["add", "file"]);
    git(&source, &["commit", "-m", "base"]);
    git(
        root.path(),
        &[
            "clone",
            source.to_str().expect("source"),
            work.to_str().expect("work"),
        ],
    );
    std::fs::write(source.join("file"), "after\n").expect("update");
    git(&source, &["commit", "-am", "upstream"]);
    let hook = work.join(".git/hooks/post-merge");
    std::fs::write(&hook, "#!/bin/sh\n(while test ! -f release-effect; do sleep 0.01; done; printf forbidden > delayed-effect) &\nprintf observed > hook-started\nwait\n").expect("hook");
    #[cfg(unix)]
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("mode");
    git(&work, &["config", "core.hooksPath", ".git/hooks"]);
    let reviews = Arc::new(AtomicUsize::new(0));
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
            final_approvals: Arc::new(AtomicUsize::new(0)),
            allow_final: false,
            drift_repository: None,
        }))
        .permission_mode(PermissionMode::Interactive)
        .auto_assistance(true)
        .build()
        .expect("runtime");
    runtime
        .submit("update this repository from upstream")
        .await
        .expect("submit");
    tokio::time::timeout(Duration::from_secs(10), async {
        while !work.join("hook-started").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("running configured hook");
    runtime.interrupt().await.expect("interrupt");
    let terminal = tokio::time::timeout(
        Duration::from_secs(10),
        collect_until_turn_completed(&mut runtime),
    )
    .await
    .expect("bounded cancellation")
    .expect("terminal");
    assert!(
        matches!(terminal, TurnCompletionStatus::Cancelled),
        "{terminal:?}"
    );
    runtime.shutdown().await.expect("cleanup confirmed");
    std::fs::write(work.join("release-effect"), "release").expect("release");
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!work.join("delayed-effect").exists());
    assert_eq!(reviews.load(Ordering::SeqCst), 1);
    assert_eq!(
        git(&work, &["rev-parse", "HEAD"]),
        git(&source, &["rev-parse", "HEAD"])
    );
    assert_eq!(
        std::fs::read(work.join("file")).expect("retained effect"),
        b"after\n"
    );
}
