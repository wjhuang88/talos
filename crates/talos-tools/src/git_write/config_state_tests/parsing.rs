#[test]
fn include_traversal_preserves_spelling_without_relaxing_other_paths() {
    let environment = BTreeMap::new();
    for spelling in ["directory/../config", "/directory/../config"] {
        let value = Some(spelling.as_bytes().to_vec());
        assert_eq!(
            configured_include_path_value(&value, &environment)
                .expect("include spelling")
                .expect("path"),
            PathBuf::from(spelling)
        );
        assert!(configured_path_value(&value, &environment).is_err());
        let output = format!("file:/repository/config\0include.path\n{spelling}\0");
        let paths = declared_include_paths(
            output.as_bytes(),
            std::path::Path::new("/repository"),
            &environment,
        );
        if cfg!(windows) && spelling.starts_with('/') {
            assert!(paths.is_err(), "root-relative Windows include is ambiguous");
            continue;
        }
        let paths = paths.expect("declaration only");
        let expected = if spelling.starts_with('/') {
            PathBuf::from(spelling)
        } else {
            PathBuf::from("/repository").join(spelling)
        };
        assert_eq!(paths, BTreeSet::from([expected]));
    }
}

#[test]
fn declared_relative_include_retains_cancelled_ancestor_on_canonical_root() {
    let root = tempfile::tempdir().expect("fixture");
    let canonical = root.path().canonicalize().expect("canonical root");
    let origin = canonical.join("config");
    std::fs::create_dir(canonical.join("traversal")).expect("ancestor");
    let mut inputs = vec!["traversal/../target.conf"];
    if cfg!(windows) {
        inputs.push(r"traversal\..\target.conf");
    }
    for input in inputs {
        let output = format!("file:{}\0include.path\n{input}\0", origin.display());
        let paths = declared_include_paths(output.as_bytes(), &canonical, &BTreeMap::new())
            .expect("include declarations");
        assert_eq!(paths.len(), 1);
        let path = paths.iter().next().expect("include");
        let suffix = if cfg!(windows) { r"traversal\..\target.conf" } else { "traversal/../target.conf" };
        assert!(path.as_os_str().to_string_lossy().ends_with(suffix));
        assert!(path.components().any(|part| part == std::path::Component::ParentDir));
        let spellings = auxiliary_spellings(path).expect("ancestor inventory");
        assert!(spellings.contains(&canonical.join("traversal")));
    }
}

#[cfg(windows)]
#[test]
fn cancelled_windows_auxiliary_ancestor_must_be_an_existing_directory() {
    let root = tempfile::tempdir().expect("fixture");
    let canonical = root.path().canonicalize().expect("canonical root");
    std::fs::write(canonical.join("file"), b"not a directory").expect("file");
    for ancestor in ["missing", "file"] {
        let mut path = canonical.as_os_str().to_os_string();
        path.push(format!(r"\{ancestor}\..\target.conf"));
        assert!(auxiliary_spellings(std::path::Path::new(&path)).is_err());
    }
}

#[cfg(windows)]
#[test]
fn windows_auxiliary_parent_at_volume_root_stays_at_root() {
    let fixture = tempfile::tempdir().expect("fixture");
    let canonical = fixture.path().canonicalize().expect("canonical fixture");
    let root = canonical.ancestors().last().expect("volume root");
    let mut path = root.as_os_str().to_os_string();
    path.push(r"..\..");
    assert_eq!(
        windows_auxiliary_parents(std::path::Path::new(&path)).expect("root parents"),
        root
    );
}

#[cfg(windows)]
#[test]
fn auxiliary_capture_resolves_cancelled_parent_without_losing_evidence_key() {
    let root = tempfile::tempdir().expect("fixture");
    assert!(std::process::Command::new("git").arg("init").arg(root.path())
        .output().expect("repository").status.success());
    let canonical = root.path().canonicalize().expect("canonical fixture");
    std::fs::create_dir(canonical.join("traversal")).expect("ancestor");
    std::fs::write(canonical.join("target.conf"), b"retained bytes").expect("source");
    let mut spelling = canonical.as_os_str().to_os_string();
    spelling.push(r"\traversal\..\target.conf");
    let spelling = PathBuf::from(spelling);
    let repository = RepositoryState::capture(root.path()).expect("repository");
    let mut config = ConfigState::capture_host(b"", &repository, &BTreeMap::new()).expect("config");
    config.auxiliary = BTreeSet::from([spelling.clone()]);
    let captured = config.capture_auxiliary().expect("resolved capture");
    assert_eq!(captured[0].0, spelling);
    assert!(matches!(&captured[0].1.entry, snapshot::Entry::File(_, bytes) if bytes == b"retained bytes"));
    std::fs::remove_dir(canonical.join("traversal")).expect("remove ancestor");
    assert!(config.capture_auxiliary().is_err(), "missing cancelled ancestor must not be folded");
}

#[cfg(windows)]
#[test]
fn declared_windows_include_paths_preserve_roots_and_reject_ambiguous_forms() {
    let environment = BTreeMap::new();
    let workdir = PathBuf::from(r"C:\work");
    let source = r"C:\config";
    let root_include = format!("file:C:\\config\0include.path\ntarget.conf\0");
    assert_eq!(
        declared_include_paths(root_include.as_bytes(), &workdir, &environment)
            .expect("drive-root include"),
        BTreeSet::from([PathBuf::from(r"C:\target.conf")])
    );

    for spelling in [r"\rooted.conf", "C:drive-relative.conf"] {
        let output = format!("file:{source}\0include.path\n{spelling}\0");
        assert!(
            declared_include_paths(output.as_bytes(), &workdir, &environment).is_err(),
            "ambiguous include spelling: {spelling:?}"
        );
    }

    for spelling in [r"D:\outside\config", r"\\server\share\config"] {
        let output = format!("file:C:\\repo\\config\0include.path\n{spelling}\0");
        assert_eq!(
            declared_include_paths(output.as_bytes(), &workdir, &environment)
                .expect("absolute include"),
            BTreeSet::from([PathBuf::from(spelling)])
        );
    }
}

#[test]
fn declared_home_include_paths_reject_ambiguous_interpolation() {
    for (home, value) in [
        ("/alias/../home", "~/config"),
        ("/home", "~//absolute"),
        ("/home", "~/../outside"),
        ("relative", "~/config"),
    ] {
        let environment = BTreeMap::from([(OsString::from("HOME"), OsString::from(home))]);
        let output = format!("file:/config\0include.path\n{value}\0");
        assert!(
            declared_include_paths(
                output.as_bytes(),
                std::path::Path::new("/repository"),
                &environment
            )
            .is_err()
        );
    }
}

#[test]
fn configuration_verification_rejects_cross_key_reordering() {
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
    let path = root.path().join(".git/config");
    let origin = format!("file:{}", path.display());
    let first =
        format!("{origin}\0submodule.recurse\ntrue\0{origin}\0push.recursesubmodules\nfalse\0");
    let reordered =
        format!("{origin}\0push.recursesubmodules\nfalse\0{origin}\0submodule.recurse\ntrue\0");
    assert_eq!(
        parse_origins(first.as_bytes()).expect("first"),
        parse_origins(reordered.as_bytes()).expect("second")
    );
    let config = ConfigState::capture_host(first.as_bytes(), &repository, &BTreeMap::new())
        .expect("capture");
    config.verify_host(first.as_bytes()).expect("same ordering");
    assert!(config.verify_host(reordered.as_bytes()).is_err());
}

#[test]
fn path_interpolation_uses_only_the_frozen_environment() {
    let values = parse_host(b"core.attributesfile\n~/rules\0").expect("config");
    let root = tempfile::tempdir().expect("fixture");
    let home = root.path().join("captured-home");
    let environment = BTreeMap::from([(OsString::from("HOME"), home.clone().into_os_string())]);
    assert_eq!(
        configured_path(&values, b"core.attributesfile", &environment).expect("captured home"),
        Some(home.join("rules")),
    );
    assert!(configured_path(&values, b"core.attributesfile", &BTreeMap::new()).is_err());
}

#[test]
fn host_parser_preserves_duplicate_order_implicit_empty_and_subsection_case() {
    let values = parse_host(
        b"Core.Bare\0core.bare\n\0core.bare\nfalse\0remote.Case.URL\nline1\nline2\0",
    )
    .expect("config");
    assert_eq!(
        values[b"core.bare".as_slice()],
        vec![None, Some(vec![]), Some(b"false".to_vec())]
    );
    assert_eq!(
        values[b"remote.Case.url".as_slice()],
        vec![Some(b"line1\nline2".to_vec())]
    );
    assert!(parse_host(b"core.bare\ntrue").is_err());
}

#[test]
fn host_parser_preserves_non_utf8_configuration_bytes() {
    let values = parse_host(b"custom.value\n\xff\xfe\0").expect("raw config");
    assert_eq!(
        values[b"custom.value".as_slice()],
        vec![Some(vec![0xff, 0xfe])]
    );
}

#[test]
fn git_boolean_matches_git_disabled_spellings() {
    for value in [b"false".as_slice(), b"0", b"off", b"no", b"", b"00", b"0k"] {
        assert!(!git_boolean(Some(value)).expect("valid boolean"));
    }
    for value in [b"true".as_slice(), b"1", b"on", b"yes", b"-2", b"1k"] {
        assert!(git_boolean(Some(value)).expect("valid boolean"));
    }
    assert!(git_boolean(None).expect("implicit true"));
    assert!(git_boolean(Some(b"maybe")).is_err());
}
