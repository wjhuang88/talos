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
            .prepare_pull(&repository, &BTreeMap::new())
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
            config.prepare_pull(&repository, &BTreeMap::new()).is_err(),
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
                .prepare_pull(&repository, &BTreeMap::new())
                .is_err(),
            "{records}"
        );
    }
    for records in [
        "fetch.bundleURI\n\0",
        "fetch.bundleURI\n/outside.bundle\0fetch.bundleURI\n\0",
    ] {
        capture(records)
            .prepare_pull(&repository, &BTreeMap::new())
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
