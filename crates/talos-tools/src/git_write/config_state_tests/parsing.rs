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
        )
        .expect("declaration only");
        let expected = if spelling.starts_with('/') {
            PathBuf::from(spelling)
        } else {
            PathBuf::from("/repository").join(spelling)
        };
        assert_eq!(paths, BTreeSet::from([expected]));
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
