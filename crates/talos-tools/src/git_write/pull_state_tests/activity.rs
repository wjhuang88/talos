//! Pull activity regression coverage.

use super::*;

#[cfg(test)]
#[test]
fn pull_historical_activity_preserves_overrides_and_unresolved_pathspecs() {
    let retained =
        |config: &[u8]| BeforeFetch::capture(config, None, None, None, 40).expect("policy");
    let policy = retained(b"submodule.Child.url\n\0submodule.Child.active\nfalse\0submodule.Child.active\ntrue\0submodule.off.url\nremote\0submodule.off.active\nfalse\0submodule.bad.active\ninvalid\0");
    assert_eq!(
        policy
            .child_active("Child", "Child", std::path::Path::new("/repo"))
            .expect("last override"),
        Some(true)
    );
    assert_eq!(
        policy
            .child_active("child", "child", std::path::Path::new("/repo"))
            .expect("case sensitive"),
        Some(false)
    );
    assert_eq!(
        policy
            .child_active("off", "off", std::path::Path::new("/repo"))
            .expect("URL overridden"),
        Some(false)
    );
    assert!(
        policy
            .child_active("bad", "bad", std::path::Path::new("/repo"))
            .is_err()
    );
    let policy = retained(b"submodule.Child.url\n\0submodule.valueless.url\0");
    assert_eq!(
        policy
            .child_active("Child", "Child", std::path::Path::new("/repo"))
            .expect("empty URL present"),
        Some(true)
    );
    assert_eq!(
        policy
            .child_active("valueless", "valueless", std::path::Path::new("/repo"))
            .expect("no URL value"),
        Some(false)
    );
    let policy = retained(b"submodule.active\n:(attr:enabled)**\0submodule.Child.url\nremote\0submodule.off.active\nfalse\0");
    assert_eq!(
        policy
            .child_active("Child", "Child", std::path::Path::new("/repo"))
            .expect("pathspec before URL"),
        None
    );
    assert_eq!(
        policy
            .child_active("off", "off", std::path::Path::new("/repo"))
            .expect("override before pathspec"),
        Some(false)
    );
}

#[cfg(test)]
#[test]
fn pull_historical_activity_matches_pathspecs_without_child_reads() {
    let root = std::path::Path::new("/repo");
    for (specs, path, expected) in [
        (vec!["path/**"], "path/child", true),
        (vec!["path/**"], "other", false),
        (
            vec!["path/**", ":(exclude)path/private"],
            "path/private",
            false,
        ),
        (vec![":(exclude)path/private"], "other", true),
        (vec![":(icase)PATH/CHILD"], "path/child", true),
        (vec![":(literal)path/*"], "path/child", false),
        (vec!["path/"], "path", true),
        (vec!["/repo/path"], "path", true),
    ] {
        let mut config = Vec::new();
        for spec in &specs {
            config.extend_from_slice(b"submodule.active\n");
            config.extend_from_slice(spec.as_bytes());
            config.push(0);
        }
        let policy = BeforeFetch::capture(&config, None, None, None, 40).expect("policy");
        assert_eq!(
            policy.child_active("named", path, root).expect("match"),
            Some(expected),
            "{specs:?} {path}"
        );
    }
}

#[cfg(test)]
#[test]
fn pull_activity_matches_admitted_attribute_states() {
    let root = std::path::Path::new("/repo");
    let mut rules = gix::attrs::Search::default();
    let mut metadata = gix::attrs::search::MetadataCollection::default();
    rules.add_patterns_buffer(
        b"* enabled -disabled color=green\n",
        root.join(".gitattributes"),
        Some(root),
        &mut metadata,
        true,
    );
    for (requirement, expected) in [
        ("enabled", true),
        ("-enabled", false),
        ("-disabled", true),
        ("disabled", false),
        ("color=green", true),
        ("color=red", false),
        ("!absent", true),
        ("absent", false),
        ("enabled -disabled color=green !absent", true),
    ] {
        let config = format!("submodule.active\n:(attr:{requirement})**\0");
        let policy = BeforeFetch::capture(config.as_bytes(), None, None, None, 40).expect("policy");
        assert_eq!(
            policy
                .child_active("child", "child", root)
                .expect("unresolved"),
            None
        );
        assert_eq!(
            policy
                .child_active_with_attributes("child", "child", root, Some((&rules, &metadata)))
                .expect("admitted attributes"),
            Some(expected),
            "{requirement}"
        );
    }
}

#[cfg(test)]
#[test]
fn pull_activity_attribute_precedence_and_exclusions() {
    let root = std::path::Path::new("/repo");
    let mut rules = gix::attrs::Search::default();
    let mut metadata = gix::attrs::search::MetadataCollection::default();
    for (source, base, bytes, macros) in [
        ("/system", None, b"* color=system\n".as_slice(), true),
        ("/global", None, b"* color=global\n", true),
        ("/repo/.gitattributes", Some(root), b"* color=root\n", true),
        (
            "/repo/nested/.gitattributes",
            Some(root),
            b"* color=nested\nchild -enabled\nchild enabled\n",
            false,
        ),
        (
            "/repo/.git/info/attributes",
            None,
            b"private !color\n",
            true,
        ),
    ] {
        rules.add_patterns_buffer(bytes, source.into(), base, &mut metadata, macros);
    }
    for (path, spec, expected) in [
        ("child", ":(attr:color=root)**", true),
        ("nested/child", ":(attr:color=nested enabled)**", true),
        ("nested/child", ":(attr:color=root)**", false),
        ("other/child", ":(attr:color=root)**", true),
        ("private", ":(attr:!color)**", true),
        ("private", ":(attr:color=root)**", false),
        ("nested/child", ":(exclude,attr:enabled)**", false),
        ("child", ":(exclude,attr:enabled)**", true),
    ] {
        let config = format!("submodule.active\n{spec}\0");
        let policy = BeforeFetch::capture(config.as_bytes(), None, None, None, 40).expect("policy");
        assert_eq!(
            policy
                .child_active_with_attributes("child", path, root, Some((&rules, &metadata)))
                .expect("attribute precedence"),
            Some(expected),
            "{path}: {spec}"
        );
    }
}

#[cfg(test)]
#[test]
fn pull_activity_attributes_agree_with_native_git() {
    let temp = tempfile::tempdir().expect("repository");
    let root = temp.path();
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", root)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .args(args)
            .output()
            .expect("Git process");
        assert!(output.status.success(), "{:?}", output.stderr);
        output.stdout
    };
    git(&["init", "-q"]);
    std::fs::create_dir(root.join("nested")).expect("directory");
    let mut rules = gix::attrs::Search::default();
    let mut metadata = gix::attrs::search::MetadataCollection::default();
    for (name, bytes, macros) in [
        (
            ".gitattributes",
            b"* selected color=root\n".as_slice(),
            true,
        ),
        (
            "nested/.gitattributes",
            b"* -selected color=nested\nunknown !selected\n",
            false,
        ),
    ] {
        std::fs::write(root.join(name), bytes).expect("attributes");
        rules.add_patterns_buffer(bytes, root.join(name), Some(root), &mut metadata, macros);
    }
    for (path, attr, value, requirement) in [
        ("child", "selected", "set", "selected"),
        ("nested/child", "selected", "unset", "-selected"),
        ("nested/unknown", "selected", "unspecified", "!selected"),
        ("nested/child", "color", "nested", "color=nested"),
        ("child", "absent", "unspecified", "!absent"),
    ] {
        assert_eq!(
            git(&["check-attr", "-z", attr, "--", path]),
            format!("{path}\0{attr}\0{value}\0").as_bytes()
        );
        let config = format!("submodule.active\n:(attr:{requirement})**\0");
        let policy = BeforeFetch::capture(config.as_bytes(), None, None, None, 40).expect("policy");
        assert_eq!(
            policy
                .child_active_with_attributes("child", path, root, Some((&rules, &metadata)))
                .expect("match"),
            Some(true)
        );
    }
    git(&["add", "--", ".gitattributes", "nested/.gitattributes"]);
    std::fs::remove_file(root.join(".gitattributes")).expect("remove worktree rules");
    std::fs::remove_file(root.join("nested/.gitattributes")).expect("remove nested rules");
    assert_eq!(
        git(&["check-attr", "-z", "color", "--", "nested/child"]),
        b"nested/child\0color\0nested\0",
        "ordinary check-attr falls back to index when worktree rules are absent"
    );
}

#[cfg(test)]
#[test]
fn pull_activity_empty_rule_set_is_unspecified() {
    let root = std::path::Path::new("/repo");
    let rules = gix::attrs::Search::default();
    let metadata = gix::attrs::search::MetadataCollection::default();
    let policy = BeforeFetch::capture(
        b"submodule.active\n:(attr:!enabled)**\0",
        None,
        None,
        None,
        40,
    )
    .expect("policy");
    assert_eq!(
        policy
            .child_active_with_attributes("child", "child", root, Some((&rules, &metadata)))
            .expect("missing file is valid evidence"),
        Some(true)
    );
}

#[cfg(test)]
#[test]
fn pull_child_recursion_precedence_and_candidate_validation() {
    use super::super::pull_children::Recursion;
    let retained =
        |config: &[u8]| BeforeFetch::capture(config, None, None, None, 40).expect("policy");
    for (global, expected) in [
        ("false", Recursion::Off),
        ("true", Recursion::On),
        ("on-demand", Recursion::OnDemand),
    ] {
        let config = format!(
            "fetch.recursesubmodules\n{global}\0submodule.Child.fetchrecursesubmodules\ninvalid\0"
        );
        assert_eq!(
            retained(config.as_bytes())
                .child_recursion("Child", Some(Recursion::Off))
                .expect("global override"),
            expected
        );
    }
    let policy = retained(b"submodule.Child.fetchrecursesubmodules\nfalse\0submodule.Child.fetchrecursesubmodules\non-demand\0submodule.child.fetchrecursesubmodules\ntrue\0submodule.bad.fetchrecursesubmodules\ninvalid\0");
    assert_eq!(
        policy
            .child_recursion("Child", Some(Recursion::Off))
            .expect("last local"),
        Recursion::OnDemand
    );
    assert_eq!(
        policy
            .child_recursion("child", Some(Recursion::Off))
            .expect("case sensitive"),
        Recursion::On
    );
    assert!(policy.child_recursion("bad", None).is_err());
    for (historical, expected) in [
        (None, Recursion::OnDemand),
        (Some(Recursion::Invalid), Recursion::OnDemand),
        (Some(Recursion::Off), Recursion::Off),
        (Some(Recursion::On), Recursion::On),
    ] {
        assert_eq!(
            policy
                .child_recursion("absent", historical)
                .expect("historical fallback"),
            expected
        );
    }
    assert_eq!(
        retained(b"submodule.Child.fetchrecursesubmodules\0")
            .child_recursion("Child", None)
            .expect("implicit"),
        Recursion::On
    );
    assert_eq!(
        retained(b"submodule.Child.fetchrecursesubmodules\n\0")
            .child_recursion("Child", None)
            .expect("empty"),
        Recursion::Off
    );
}

#[cfg(test)]
#[test]
fn pull_fetch_recursion_preserves_modes_and_declaration_order() {
    for (config, expected) in [
        (b"".as_slice(), FetchRecursion::Default),
        (
            b"submodule.Child.fetchrecursesubmodules\ntrue\0",
            FetchRecursion::Default,
        ),
        (
            b"fetch.recursesubmodules\non-demand\0",
            FetchRecursion::OnDemand,
        ),
        (b"fetch.recursesubmodules\0", FetchRecursion::On),
        (b"fetch.recursesubmodules\n\0", FetchRecursion::Off),
        (
            b"submodule.recurse\ntrue\0fetch.recursesubmodules\nfalse\0",
            FetchRecursion::Off,
        ),
        (
            b"fetch.recursesubmodules\nfalse\0submodule.recurse\ntrue\0",
            FetchRecursion::On,
        ),
        (
            b"submodule.recurse\nfalse\0fetch.recursesubmodules\non-demand\0",
            FetchRecursion::OnDemand,
        ),
        (
            b"fetch.recursesubmodules\non-demand\0submodule.recurse\nfalse\0",
            FetchRecursion::Off,
        ),
        (
            b"fetch.recursesubmodules\ntrue\0fetch.recursesubmodules\nfalse\0",
            FetchRecursion::Off,
        ),
    ] {
        let retained = BeforeFetch::capture(config, None, None, None, 40).expect("retained policy");
        assert_eq!(retained.fetch_recursion, expected, "{config:?}");
    }
    // An invalid earlier declaration is fatal even if a later value overrides
    // it, because native fetch validates configuration before parsing argv.
    for config in [
        b"fetch.recursesubmodules\ninvalid\0fetch.recursesubmodules\nfalse\0".as_slice(),
        b"submodule.recurse\non-demand\0fetch.recursesubmodules\nfalse\0",
        b"fetch.recursesubmodules\nON-DEMAND\0",
        b"fetch.recursesubmodules\nfalse",
    ] {
        assert!(
            BeforeFetch::capture(config, None, None, None, 40).is_err(),
            "{config:?}"
        );
    }
}
