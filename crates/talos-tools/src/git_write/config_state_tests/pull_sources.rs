#[test]
fn pull_integration_sources_reject_mutable_worktree_aliases() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .current_dir(root.path())
            .output()
            .expect("git")
            .status
            .success()
    );
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let mut config = ConfigState::capture_host(b"", &repository, &BTreeMap::new()).expect("config");
    let metadata_hook = repository.common_dir.join("hooks/post-merge");
    config
        .validate_pull_integration_sources(&repository, &[metadata_hook])
        .expect("ordinary metadata hooks");
    // Conservative overlap matching must never grant a metadata exemption
    // to a differently spelled worktree directory.
    assert!(
        config
            .validate_pull_integration_sources(
                &repository,
                &[repository.workdir.join(".GIT/hooks/post-merge")]
            )
            .is_err()
    );
    let mutable = repository.workdir.join("local-hooks/post-merge");
    let error = config
        .validate_pull_integration_sources(&repository, std::slice::from_ref(&mutable))
        .expect_err("even missing hooks can be introduced during integration");
    assert!(error.to_string().contains("may rewrite"));
    assert!(!error.to_string().contains("local-hooks"));
    config.stable_auxiliary.insert(mutable);
    assert!(
        config
            .validate_pull_integration_sources(&repository, &[])
            .is_err()
    );
    config.stable_auxiliary.clear();
    let outside = tempfile::tempdir().expect("outside");
    let external = outside.path().join("config");
    config.stable_auxiliary.insert(external.clone());
    config
        .validate_pull_integration_sources(&repository, &[])
        .expect("external immutable source");
    #[cfg(unix)]
    {
        let alias = outside.path().join("alias");
        std::os::unix::fs::symlink(&repository.workdir, &alias).expect("alias");
        config.stable_auxiliary.insert(alias.join("missing-config"));
        assert!(
            config
                .validate_pull_integration_sources(&repository, &[])
                .is_err()
        );
    }
}
