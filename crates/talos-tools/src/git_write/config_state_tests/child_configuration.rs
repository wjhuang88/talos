#[test]
fn child_fetch_remote_binds_branch_and_single_remote_defaults() {
    let root = tempfile::Builder::new()
        .prefix("talos-config-")
        .tempdir()
        .expect("fixture");
    let path = root.path().join("config-without-dot");
    std::fs::write(&path, "").expect("source");
    let output = [
        "remote.backup.url\ngit://backup.invalid/repo",
        "remote.upstream.url\ngit://upstream.invalid/repo",
        "branch.Topic.remote\nupstream",
    ]
    .iter()
    .map(|record| format!("file:{}\0{record}\0", path.display()))
    .collect::<String>();
    let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
    assert_eq!(
        config.fetch_remote(Some("Topic")).expect("branch"),
        "upstream"
    );
    assert_eq!(config.fetch_remote(Some("topic")).expect("case"), "origin");
    assert_eq!(config.fetch_remote(None).expect("detached"), "origin");
    let output = format!(
        "file:{}\0remote.backup.url\ngit://backup.invalid/repo\0",
        path.display()
    );
    let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
    assert_eq!(config.fetch_remote(None).expect("single"), "backup");
}

#[test]
fn child_excludes_follow_child_config_and_frozen_environment() {
    let root = tempfile::tempdir().expect("fixture");
    let config = ChildConfig::capture(b"", root.path()).expect("empty config");
    let common = root.path().join("metadata");
    let home = root.path().join("home-fixture");
    let xdg = root.path().join("xdg-fixture");
    let environment = BTreeMap::from([
        (OsString::from("HOME"), home.clone().into_os_string()),
        (
            OsString::from("XDG_CONFIG_HOME"),
            xdg.clone().into_os_string(),
        ),
    ]);
    let paths = config
        .attribute_paths(root.path(), &common, &environment)
        .expect("defaults");
    assert!(paths.contains(&xdg.join("git/ignore")));
    assert!(paths.contains(&common.join("info/exclude")));
    assert!(!paths.contains(&home.join(".config/git/ignore")));
    let source = root.path().join("config");
    std::fs::write(&source, "[core]\n excludesFile = rules\n").expect("source");
    let output = format!("file:{}\0core.excludesfile\nrules\0", source.display());
    let configured = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
    let paths = configured
        .attribute_paths(root.path(), &common, &environment)
        .expect("override");
    assert!(paths.contains(&root.path().join("rules")));
    assert!(!paths.contains(&xdg.join("git/ignore")));
}

#[test]
fn child_promisor_configuration_refuses_object_inspection() {
    let root = tempfile::tempdir().expect("fixture");
    for value in [
        b"remote.origin.promisor\ntrue\0".as_slice(),
        b"remote.origin.partialclonefilter\nblob:none\0",
        b"extensions.partialclone\norigin\0",
    ] {
        let mut output =
            format!("file:{}\0", root.path().join("config").display()).into_bytes();
        output.extend_from_slice(value);
        let error = ChildConfig::capture(&output, root.path())
            .err()
            .expect("refusal");
        assert!(error.to_string().contains("promisor remote"), "{error}");
    }
}

#[test]
fn child_pull_hook_paths_preserve_common_default_and_effective_worktree_override() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("config");
    std::fs::write(&source, "").expect("source");
    let common = root.path().join("common");
    let worktree = root.path().join("effective-worktree");
    let environment = BTreeMap::new();
    let config = ChildConfig::capture(b"", root.path()).expect("empty config");
    assert_eq!(
        config
            .pull_hook_paths(&common, &worktree, &environment)
            .expect("defaults"),
        [
            common.join("hooks/reference-transaction"),
            common.join("hooks/pre-auto-gc")
        ]
    );
    let output = format!(
        "file:{}\0core.hookspath\nrelative-hooks\0",
        source.display()
    );
    let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("override");
    assert_eq!(
        config
            .pull_hook_paths(&common, &worktree, &environment)
            .expect("relative"),
        [
            worktree.join("relative-hooks/reference-transaction"),
            worktree.join("relative-hooks/pre-auto-gc")
        ]
    );
    assert!(!common.exists());
    assert!(
        !worktree.exists(),
        "path planning must not open hooks or their roots"
    );
}

#[test]
fn child_fetch_refuses_external_effects_and_honors_disabling_values() {
    let root = tempfile::tempdir().expect("fixture");
    std::fs::write(root.path().join("config"), "").expect("source");
    let capture = |records: &[(&str, &str)]| {
        let mut output = String::new();
        for (key, value) in records {
            output.push_str(&format!(
                "file:{}\0{key}\n{value}\0",
                root.path().join("config").display()
            ));
        }
        ChildConfig::capture(output.as_bytes(), root.path()).expect("capture")
    };
    for records in [
        vec![("fetch.bundleuri", "https://example.com/bundle")],
        vec![("core.gitproxy", "proxy-program")],
        vec![("gc.recentobjectshook", "hook-program")],
        vec![("gc.repackfilterto", "/external/objects")],
        vec![("maintenance.auto", "invalid")],
    ] {
        assert!(capture(&records).prepare_pull_transport("origin").is_err());
    }
    for records in [
        vec![
            ("fetch.bundleuri", "https://example.com/bundle"),
            ("fetch.bundleuri", ""),
        ],
        vec![("gc.repackfilterto", "")],
        vec![
            ("maintenance.auto", "false"),
            ("gc.recentobjectshook", "hook-program"),
            ("gc.repackfilterto", "/external/objects"),
        ],
    ] {
        capture(&records)
            .prepare_pull_transport("origin")
            .expect("disabled effect");
    }
}

#[test]
fn child_configuration_rejects_new_origins_before_source_capture() {
    let root = tempfile::tempdir().expect("fixture");
    let source = root.path().join("config");
    std::fs::write(&source, "[custom]\n value = same\n").expect("source");
    let output = format!("file:{}\0custom.value\nsame\0", source.display());
    let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
    config.verify_host(output.as_bytes()).expect("unchanged");
    // The newly reported origin is deliberately not a readable regular
    // configuration file. Output comparison must reject it without capture.
    let changed = format!("file:{}\0custom.value\nsame\0", root.path().display());
    assert!(
        config
            .verify_host(changed.as_bytes())
            .expect_err("new origin")
            .to_string()
            .contains("changed during discovery")
    );
    config.revalidate().expect("original evidence retained");
}

#[test]
fn child_fetch_destination_preserves_first_url_and_rewrite_order() {
    let root = tempfile::tempdir().expect("fixture");
    std::fs::write(root.path().join("config"), "").expect("source");
    let output = |records: &[&str]| {
        records
            .iter()
            .map(|record| format!("file:{}\0{record}\0", root.path().join("config").display()))
            .collect::<String>()
    };
    let first = output(&[
        "remote.origin.url\nalias:repo",
        "remote.origin.url\ngit://ignored.invalid/repo",
        "url.git://first.invalid/.insteadof\nalias:",
        "url.git://second.invalid/.insteadof\nalias:",
    ]);
    let config = ChildConfig::capture(first.as_bytes(), root.path()).expect("config");
    assert_eq!(
        config.fetch_destinations("origin").expect("fetch"),
        ["git://first.invalid/repo"]
    );
    let reordered = output(&[
        "remote.origin.url\nalias:repo",
        "remote.origin.url\ngit://ignored.invalid/repo",
        "url.git://second.invalid/.insteadof\nalias:",
        "url.git://first.invalid/.insteadof\nalias:",
    ]);
    assert!(config.verify_host(reordered.as_bytes()).is_err());
}

#[cfg(unix)]
#[test]
fn child_configuration_rejects_equal_content_ancestor_retarget() {
    let root = tempfile::tempdir().expect("fixture");
    for directory in ["first", "second"] {
        std::fs::create_dir(root.path().join(directory)).expect("directory");
        std::fs::write(
            root.path().join(directory).join("config"),
            "[core]\n fsmonitor = false\n",
        )
        .expect("config");
    }
    let link = root.path().join("current");
    std::os::unix::fs::symlink(root.path().join("first"), &link).expect("link");
    let path = link.join("config");
    let output = format!("file:{}\0core.fsmonitor\nfalse\0", path.display());
    let config = super::ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
    config.revalidate().expect("unchanged");
    std::fs::remove_file(&link).expect("remove fixture link");
    std::os::unix::fs::symlink(root.path().join("second"), &link).expect("retarget");
    assert!(
        config
            .revalidate()
            .expect_err("identity changed")
            .to_string()
            .contains("path identity")
    );
}

#[cfg(unix)]
#[test]
fn child_configuration_rejects_identical_file_replacement() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().join("config");
    std::fs::write(&path, "[core]\n fsmonitor = false\n").expect("config");
    let output = format!("file:{}\0core.fsmonitor\nfalse\0", path.display());
    let config = super::ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
    let replacement = root.path().join("replacement");
    std::fs::copy(&path, &replacement).expect("identical bytes");
    std::fs::rename(replacement, &path).expect("replace");
    assert!(
        config
            .revalidate()
            .expect_err("identity changed")
            .to_string()
            .contains("path identity")
    );
}

#[test]
fn native_pull_child_configs_share_budget_across_cloned_continuations() {
    let root = tempfile::tempdir().expect("fixture");
    let budget = super::super::pull_inventory::ReferenceBudget::default();
    let continuation = budget.clone();
    for name in ["first", "second"] {
        std::fs::write(root.path().join(name), vec![b' '; 2 * 1024 * 1024 + 1])
            .expect("config source");
    }
    let output = |name: &str| {
        format!(
            "file:{}\0remote.origin.url\ngit://example.invalid/repo\0",
            root.path().join(name).display()
        )
    };
    ChildConfig::capture_native_pull(output("first").as_bytes(), root.path(), &budget)
        .expect("first child fits");
    assert!(
        ChildConfig::capture_native_pull(
            output("second").as_bytes(),
            root.path(),
            &continuation,
        )
        .is_err()
    );
    assert!(
        ChildConfig::capture_native_pull(output("first").as_bytes(), root.path(), &budget)
            .is_err()
    );
}

#[test]
fn native_pull_retained_host_output_shares_the_source_budget() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().join("config");
    std::fs::write(&path, "").expect("empty source");
    // The source itself is empty: exhaustion must come from retained output,
    // including its ordered bytes, parsed values and origin representations.
    let output = format!(
        "file:{}\0remote.origin.url\n{}\0",
        path.display(),
        "x".repeat(512 * 1024),
    );
    let budget = super::super::pull_inventory::ReferenceBudget::default();
    let continuation = budget.clone();
    for _ in 0..2 {
        ChildConfig::capture_native_pull(output.as_bytes(), root.path(), &budget)
            .expect("retained output fits");
    }
    assert!(
        ChildConfig::capture_native_pull(output.as_bytes(), root.path(), &continuation).is_err()
    );
    // Oversized output is rejected before parsing or trying to open its origin.
    let oversized = vec![b'x'; 2 * 1024 * 1024];
    assert!(
        ChildConfig::capture_native_pull(
            &oversized,
            root.path(),
            &super::super::pull_inventory::ReferenceBudget::default(),
        ).is_err()
    );
}

#[cfg(unix)]
#[test]
fn bounded_configuration_source_does_not_admit_redirected_ancestors() {
    let root = tempfile::tempdir().expect("fixture");
    let root = root.path().canonicalize().expect("root");
    let actual = root.join("actual");
    std::fs::create_dir(&actual).expect("directory");
    std::fs::write(actual.join("config"), "").expect("source");
    let linked = root.join("linked");
    std::os::unix::fs::symlink(&actual, &linked).expect("directory link");
    for (source, admitted) in [(actual.join("config"), true), (linked.join("config"), false)] {
        let output = format!("file:{}\0remote.origin.url\ngit://example.invalid/repo\0", source.display());
        let config = ChildConfig::capture(output.as_bytes(), &root).expect("chain evidence");
        assert_eq!(config.source_is_unredirected(&source), admitted);
        assert!(!config.source_is_unredirected(&root.join("uncaptured")));
    }
}
