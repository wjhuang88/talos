#[test]
fn auxiliary_evidence_rejects_same_content_replacement_and_missing_creation() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("repository")
        .status
        .success());
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let mut config =
        ConfigState::capture_host(b"", &repository, &BTreeMap::new()).expect("config");
    let path = root.path().join("attributes");
    let retained = root.path().join("retained-attributes");
    std::fs::write(&path, b"fixed bytes").expect("source");
    config.auxiliary = BTreeSet::from([path.clone()]);
    let evidence = config.capture_auxiliary().expect("capture");
    config.revalidate_auxiliary(&evidence).expect("unchanged");
    // Keep the old inode/handle alive rather than relying on allocator timing.
    std::fs::rename(&path, &retained).expect("retain old leaf");
    std::fs::write(&path, b"fixed bytes").expect("same content replacement");
    assert!(config.revalidate_auxiliary(&evidence).is_err());

    std::fs::remove_file(&path).expect("remove replacement");
    let absent = config.capture_auxiliary().expect("capture absence");
    config.revalidate_auxiliary(&absent).expect("still absent");
    std::fs::write(&path, b"").expect("create formerly absent source");
    assert!(config.revalidate_auxiliary(&absent).is_err());
}

#[test]
fn auxiliary_evidence_cannot_be_reused_for_a_changed_source_set() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("repository")
        .status
        .success());
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let mut config =
        ConfigState::capture_host(b"", &repository, &BTreeMap::new()).expect("config");
    config.auxiliary = BTreeSet::from([root.path().join("first")]);
    let evidence = config.capture_auxiliary().expect("capture");
    config.auxiliary.insert(root.path().join("second"));
    assert!(config.revalidate_auxiliary(&evidence).is_err());
}

#[cfg(windows)]
#[test]
fn checkout_protected_paths_overlap_across_windows_prefix_spellings() {
    use std::path::Path;

    for (ordinary, verbatim) in [
        (r"C:\repo\settings", r"\\?\C:\repo\settings\rules"),
        (r"\\server\share\repo", r"\\?\UNC\server\share\repo\rules"),
        (r"C:\Users\RUNNER~1\repo", r"\\?\C:\Users\runneradmin\repo\rules"),
        (r"C:\Users\runneradmin\repo", r"\\?\C:\Users\RUNNER~1\repo\rules"),
    ] {
        assert!(potential_path_prefix(Path::new(ordinary), Path::new(verbatim)));
        let parent = Path::new(verbatim).parent().expect("parent");
        assert!(potential_path_prefix(parent, Path::new(ordinary)));
    }
    assert!(!potential_path_prefix(Path::new(r"C:\repo"), Path::new(r"\\?\D:\repo\rules")));
    assert!(!potential_path_prefix(Path::new(r"\\server\share\repo"), Path::new(r"\\?\UNC\server\other\repo\rules")));
    assert!(!potential_path_prefix(Path::new(r"C:\repo"), Path::new(r"\\?\C:\repository\rules")));
    assert!(!potential_path_prefix(Path::new(r"C:\Users\RUNNER~1\repo"), Path::new(r"\\?\C:\Users\runneradmin\other\rules")));
}

#[test]
fn global_attribute_default_uses_frozen_xdg_and_home_evidence_without_git_var() {
    // I324: older Git cannot answer GIT_ATTR_GLOBAL. The existing host
    // configuration model already names the default global attribute source
    // from frozen XDG/HOME input; this does not establish compiled SYSTEM
    // attribute defaults or grant any new filesystem-read authority.
    let root = tempfile::tempdir().expect("fixture");
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .arg(root.path())
            .output()
            .expect("repository")
            .status
            .success()
    );
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let home = root.path().join("home");
    let xdg = root.path().join("explicit-xdg");
    let mut environment = BTreeMap::new();
    environment.insert(OsString::from("HOME"), home.as_os_str().to_os_string());
    environment.insert(
        OsString::from("XDG_CONFIG_HOME"),
        xdg.as_os_str().to_os_string(),
    );

    let config =
        ConfigState::capture_host(b"", &repository, &environment).expect("empty host config");
    let xdg_attrs = xdg.join("git/attributes");
    assert!(
        config.stable_auxiliary.contains(&xdg_attrs),
        "frozen explicit XDG global attributes must remain in source evidence"
    );
    assert!(
        !config.stable_auxiliary.contains(&home.join(".config/git/attributes")),
        "HOME fallback must not replace explicit XDG selection"
    );

    // Capture follows existing approved continuation code paths; this test
    // only exercises its evidence behavior, not a new pre-admission reader.
    std::fs::create_dir_all(xdg_attrs.parent().expect("attribute parent"))
        .expect("create source parent");
    std::fs::write(&xdg_attrs, b"*.txt filter=reviewed\n").expect("attribute fixture");
    let evidence = config.capture_auxiliary().expect("fixture evidence");
    config
        .revalidate_auxiliary(&evidence)
        .expect("unchanged admitted source");
    std::fs::write(&xdg_attrs, b"*.txt filter=changed\n").expect("replace source bytes");
    assert!(
        config.revalidate_auxiliary(&evidence).is_err(),
        "a changed global attributes source must invalidate preparation"
    );

    environment.remove(std::ffi::OsStr::new("XDG_CONFIG_HOME"));
    let home_config =
        ConfigState::capture_host(b"", &repository, &environment).expect("HOME fallback config");
    assert!(
        home_config
            .stable_auxiliary
            .contains(&home.join(".config/git/attributes")),
        "HOME fallback remains a named source even when the file is missing"
    );
    assert!(
        !home_config.stable_auxiliary.contains(&xdg_attrs),
        "a changed frozen environment must produce a different source set"
    );
}
