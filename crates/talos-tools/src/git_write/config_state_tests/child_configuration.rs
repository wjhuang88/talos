
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
