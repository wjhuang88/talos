#[test]
fn checkout_reload_hook_inventory_preserves_source_cache_and_defers_reads() {
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
    let environment = BTreeMap::new();
    let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
    let records = vec![(
        PathBuf::from("target.conf"),
        super::super::target_config::Record {
            key: b"core.hookspath".to_vec(),
            value: Some(b"target-hooks".to_vec()),
        },
    )];
    let paths = config
        .checkout_reload_hook_paths(&records, &repository, &environment)
        .expect("inventory");
    assert_eq!(
        paths,
        vec![
            repository.workdir.join("target-hooks/post-checkout"),
            repository
                .workdir
                .join("target-hooks/reference-transaction")
        ]
    );
    assert!(!repository.workdir.join("target-hooks").exists());
    let absent =
        ConfigState::capture_checkout_reload_hooks(&paths).expect("admitted missing paths");
    assert!(
        absent
            .iter()
            .all(|(_, evidence)| matches!(evidence.entry, snapshot::Entry::Missing))
    );
    std::fs::create_dir(repository.workdir.join("target-hooks")).expect("directory");
    std::fs::write(&paths[0], b"fixture").expect("hook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&paths[0], std::fs::Permissions::from_mode(0o700))
            .expect("mode");
    }
    assert!(ConfigState::capture_checkout_reload_hooks(&paths).is_err());
    config.values.insert(
        b"core.hookspath".to_vec(),
        vec![Some(b"source-hooks".to_vec())],
    );
    let cached = config
        .checkout_reload_hook_paths(&records, &repository, &environment)
        .expect("cached source");
    assert_eq!(
        cached[0],
        repository.workdir.join("source-hooks/post-checkout")
    );
    // Inventory generation never probes even an executable target candidate.
    assert_eq!(
        paths,
        ConfigState::capture_host(b"", &repository, &environment)
            .expect("config")
            .checkout_reload_hook_paths(&records, &repository, &environment)
            .expect("pure inventory")
    );
}

#[test]
fn checkout_reload_refuses_target_promisor_and_uses_final_comparison_value() {
    let records = |entries: &[(&str, Option<&str>)]| {
        entries
            .iter()
            .map(|(key, value)| {
                (
                    PathBuf::from("target.conf"),
                    super::super::target_config::Record {
                        key: key.as_bytes().to_vec(),
                        value: value.map(|value| value.as_bytes().to_vec()),
                    },
                )
            })
            .collect::<Vec<_>>()
    };
    for entry in [
        ("remote.target.promisor", Some("true")),
        ("remote.target.promisor", None),
        ("remote.target.promisor", Some("invalid")),
        ("remote.target.partialclonefilter", Some("blob:none")),
        ("extensions.partialclone", Some("target")),
    ] {
        assert!(ConfigState::validate_checkout_reload(&records(&[entry])).is_err());
    }
    assert!(
        ConfigState::validate_checkout_reload(&records(&[
            ("remote.target.promisor", Some("false")),
            ("status.comparebranches", None),
            ("status.comparebranches", Some("@{upstream}")),
        ]))
        .is_ok()
    );
    assert!(
        ConfigState::validate_checkout_reload(&records(&[
            ("status.comparebranches", Some("@{upstream}")),
            ("status.comparebranches", None),
        ]))
        .is_err()
    );
}

#[test]
fn checkout_hooks_are_deferred_and_recursive_effects_are_explicit() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .arg(root.path())
            .output()
            .expect("git")
            .status
            .success()
    );
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let environment = BTreeMap::new();
    let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
    let hook = repository.common_dir.join("hooks/post-checkout");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").expect("hook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .expect("executable");
    }
    config
        .prepare_checkout(&repository, &environment, false)
        .expect("discovery does not inspect hook");
    assert!(config.auxiliary.contains(&hook));
    assert!(
        config
            .capture_auxiliary()
            .expect_err("active hook")
            .to_string()
            .contains("post-checkout")
    );
    let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
    config
        .values
        .insert(b"submodule.recurse".to_vec(), vec![Some(b"true".to_vec())]);
    assert!(
        config
            .prepare_checkout(&repository, &environment, false)
            .expect_err("child write scope")
            .to_string()
            .contains("child worktree")
    );
    config
        .values
        .insert(b"submodule.recurse".to_vec(), vec![Some(b"false".to_vec())]);
    config
        .prepare_checkout(&repository, &environment, false)
        .expect("nonrecursive gitlinks allowed");
    config.values.insert(
        b"includeif.onbranch:target.path".to_vec(),
        vec![Some(b"target-config".to_vec())],
    );
    assert!(
        config
            .prepare_checkout(&repository, &environment, false)
            .expect_err("conditional reload")
            .to_string()
            .contains("target-branch discovery")
    );
}
