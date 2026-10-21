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
    ] {
        assert!(potential_path_prefix(Path::new(ordinary), Path::new(verbatim)));
        let parent = Path::new(verbatim).parent().expect("parent");
        assert!(potential_path_prefix(parent, Path::new(ordinary)));
    }
    assert!(!potential_path_prefix(Path::new(r"C:\repo"), Path::new(r"\\?\D:\repo\rules")));
    assert!(!potential_path_prefix(Path::new(r"\\server\share\repo"), Path::new(r"\\?\UNC\server\other\repo\rules")));
    assert!(!potential_path_prefix(Path::new(r"C:\repo"), Path::new(r"\\?\C:\repository\rules")));
}
