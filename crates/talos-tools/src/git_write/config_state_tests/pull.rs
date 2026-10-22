#[test]
fn pull_helper_checks_follow_maintenance_and_selected_remote() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(root.path())
            .output()
            .expect("git")
            .status
            .success()
    );
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let capture = |records: &str| {
        let source = repository.common_dir.join("config");
        let output = records
            .split_terminator('\0')
            .map(|record| format!("file:{}\0{record}\0", source.display()))
            .collect::<String>();
        ConfigState::capture_host(output.as_bytes(), &repository, &BTreeMap::new())
            .expect("config")
    };

    for records in [
        "maintenance.auto\nfalse\0gc.recentObjectsHook\nhook\0gc.repackFilterTo\n/out\0",
        "maintenance.auto\n0\0gc.recentObjectsHook\nhook\0gc.repackFilterTo\n\0",
    ] {
        let mut config = capture(records);
        config
            .prepare_native_pull(&repository, &BTreeMap::new(), None)
            .expect("disabled maintenance keeps external settings inactive");
    }
    for records in [
        "maintenance.auto\ntrue\0gc.recentObjectsHook\nhook\0",
        "maintenance.auto\nyes\0gc.repackFilterTo\n/out\0",
        "gc.recentObjectsHook\nhook\0",
        "gc.repackFilterTo\n/out\0",
    ] {
        let mut config = capture(records);
        assert!(
            config.prepare_native_pull(&repository, &BTreeMap::new(), None).is_err(),
            "{records}"
        );
    }

    let config = capture("remote.origin.uploadpack\ncustom-upload\0");
    assert!(config.prepare_pull_transport("origin").is_err());
    assert!(config.prepare_pull_transport("other").is_ok());
    let config = capture("remote.origin.vcs\ncustom-vcs\0");
    assert!(config.prepare_pull_transport("origin").is_err());
    assert!(config.prepare_pull_transport("other").is_ok());

    for records in [
        "fetch.bundleURI\n/outside.bundle\0",
        "fetch.bundleURI\nhttps://example.invalid/bundles\0",
        "fetch.bundleURI\0",
        "fetch.bundleURI\n\0fetch.bundleURI\n/outside.bundle\0",
    ] {
        assert!(
            capture(records)
                .prepare_native_pull(&repository, &BTreeMap::new(), None)
                .is_err(),
            "{records}"
        );
    }
    for records in [
        "fetch.bundleURI\n\0",
        "fetch.bundleURI\n/outside.bundle\0fetch.bundleURI\n\0",
    ] {
        capture(records)
            .prepare_native_pull(&repository, &BTreeMap::new(), None)
            .expect("empty final bundle URI disables retrieval");
    }
}

#[test]
fn pull_diff_order_file_resolves_relative_paths_under_workdir() {
    let root = tempfile::tempdir().expect("fixture");
    let workdir = root.path().canonicalize().expect("workdir");
    let mut state = ConfigState {
        values: Values::new(),
        ordered_output: Vec::new(),
        origins: BTreeSet::new(),
        files: Vec::new(),
        identities: Vec::new(),
        auxiliary: BTreeSet::new(),
        stable_auxiliary: BTreeSet::new(),
        hooks: Vec::new(),
    };
    for (value, expected) in [
        (b"rules/order".to_vec(), workdir.join("rules/order")),
        (
            workdir
                .join("absolute-order")
                .to_string_lossy()
                .as_bytes()
                .to_vec(),
            workdir.join("absolute-order"),
        ),
    ] {
        state
            .values
            .insert(b"diff.orderfile".to_vec(), vec![Some(value)]);
        assert_eq!(
            state
                .pull_diff_order_file(&workdir, &BTreeMap::new())
                .expect("path"),
            Some(expected)
        );
    }
    state.values.insert(
        b"diff.orderfile".to_vec(),
        vec![Some(b"rules\0order".to_vec())],
    );
    assert!(
        state
            .pull_diff_order_file(&workdir, &BTreeMap::new())
            .is_err()
    );
}

#[test]
fn native_pull_discovers_policy_hooks_before_fetch_without_reading_candidates() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(std::process::Command::new("git").args(["init"]).current_dir(root.path())
        .output().expect("git").status.success());
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let capture = |records: &str| {
        let source = repository.common_dir.join("config");
        let output = records.split_terminator('\0')
            .map(|record| format!("file:{}\0{record}\0", source.display()))
            .collect::<String>();
        ConfigState::capture_host(output.as_bytes(), &repository, &BTreeMap::new()).expect("config")
    };
    // A directory at a hook filename would fail a content read. Discovery only
    // publishes paths; even these invalid candidates must await Read admission.
    let hooks = repository.workdir.join("candidate-hooks");
    std::fs::create_dir_all(hooks.join("pre-rebase")).expect("candidate");
    let records = "core.hookspath\ncandidate-hooks\0pull.rebase\nfalse\0branch.Topic.rebase\ntrue\0rebase.backend\napply\0";
    let paths = capture(records).prepare_native_pull(&repository, &BTreeMap::new(), Some("Topic")).expect("rebase frontier");
    for name in ["pre-rebase", "applypatch-msg", "pre-applypatch", "post-applypatch", "reference-transaction"] {
        assert!(paths.contains(&hooks.join(name)), "{name}");
    }
    assert!(!paths.contains(&hooks.join("pre-merge-commit")));
    let paths = capture(records).prepare_native_pull(&repository, &BTreeMap::new(), Some("topic")).expect("merge frontier");
    assert!(paths.contains(&hooks.join("pre-merge-commit")));
    assert!(!paths.contains(&hooks.join("pre-rebase")));
    assert!(!repository.git_dir.join("FETCH_HEAD").exists());
}

#[test]
fn native_pull_rejects_unapproved_program_effects_before_fetch() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(std::process::Command::new("git").args(["init"]).current_dir(root.path())
        .output().expect("git").status.success());
    let repository = RepositoryState::capture(root.path()).expect("repository");
    for records in [
        "pull.rebase\ninteractive\0",
        "pull.rebase\ntrue\0rebase.instructionformat\n%nexec command\0",
        "filter.fixture.process\ncommand\0",
        "merge.fixture.driver\ncommand\0",
        "commit.gpgsign\ntrue\0",
    ] {
        let source = repository.common_dir.join("config");
        let output = records.split_terminator('\0')
            .map(|record| format!("file:{}\0{record}\0", source.display()))
            .collect::<String>();
        let mut config = ConfigState::capture_host(output.as_bytes(), &repository, &BTreeMap::new()).expect("config");
        assert!(config.prepare_native_pull(&repository, &BTreeMap::new(), None).is_err(), "{records}");
        assert!(!repository.git_dir.join("FETCH_HEAD").exists());
    }
}
