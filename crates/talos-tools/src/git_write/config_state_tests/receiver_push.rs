#[test]
fn receiver_effect_inventory_defers_reads_and_rejects_unbound_commands() {
    let root = tempfile::tempdir().expect("fixture");
    let base = root.path().canonicalize().expect("canonical");
    let mut config = ChildConfig::capture(b"", &base).expect("empty configuration");
    config.values.insert(
        b"core.hookspath".to_vec(),
        vec![Some(b"relative-hooks".to_vec())],
    );
    config.values.insert(
        b"receive.fsck.skiplist".to_vec(),
        vec![Some(b"relative-skiplist".to_vec())],
    );
    let effects = config
        .receiver_effects(&base, &base, &BTreeMap::new())
        .expect("inventory");
    assert!(
        effects
            .hooks
            .contains(&base.join("relative-hooks/pre-receive"))
    );
    assert!(effects.paths.contains(&base.join("relative-skiplist")));
    assert!(!base.join("relative-hooks").exists());
    assert!(!base.join("relative-skiplist").exists());
    effects.capture().expect("missing auxiliaries");
    for (key, value) in [
        ("hook.check.command", "command"),
        ("core.alternateRefsCommand", "command"),
        ("gc.recentObjectsHook", "command"),
        ("gc.repackFilterTo", "/outside"),
        ("receive.denyCurrentBranch", "updateInstead"),
    ] {
        let key = key.to_ascii_lowercase().into_bytes();
        config
            .values
            .insert(key.clone(), vec![Some(value.as_bytes().to_vec())]);
        assert!(
            config
                .receiver_effects(&base, &base, &BTreeMap::new())
                .is_err(),
            "{key:?}"
        );
        config.values.remove(&key);
    }
    config
        .values
        .insert(b"receive.autogc".to_vec(), vec![Some(b"false".to_vec())]);
    config.values.insert(
        b"gc.recentobjectshook".to_vec(),
        vec![Some(b"inactive".to_vec())],
    );
    config.values.insert(
        b"gc.repackfilterto".to_vec(),
        vec![Some(b"/inactive".to_vec())],
    );
    config
        .receiver_effects(&base, &base, &BTreeMap::new())
        .expect("disabled maintenance");
}

#[cfg(unix)]
#[test]
fn receiver_auxiliary_capture_refuses_executable_hooks_and_object_alternates() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().expect("fixture");
    let base = root.path().canonicalize().expect("canonical");
    let config = ChildConfig::capture(b"", &base).expect("config");
    let effects = config
        .receiver_effects(&base, &base, &BTreeMap::new())
        .expect("inventory");
    std::fs::create_dir(base.join("hooks")).expect("hooks");
    let hook = base.join("hooks/pre-receive");
    std::fs::write(&hook, "#!/bin/sh\nexit 0\n").expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
        .expect("executable");
    assert!(
        effects
            .capture()
            .expect_err("refuse hook")
            .to_string()
            .contains("Executable receiver hook")
    );
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o644))
        .expect("inactive hook");
    effects.capture().expect("inactive hook allowed");
    std::fs::create_dir_all(base.join("objects/info")).expect("objects");
    std::fs::write(
        base.join("objects/info/alternates"),
        "/unopened-object-store\n",
    )
    .expect("alternates");
    assert!(
        effects
            .capture()
            .expect_err("refuse alternates")
            .to_string()
            .contains("object alternates")
    );
}

#[test]
fn receiver_capture_rejects_relative_origin_before_snapshot() {
    let root = tempfile::tempdir().expect("fixture");
    let error = ChildConfig::capture_receiver(
        b"file:missing-relative\0core.bare\nfalse\0",
        root.path(),
    )
    .err()
    .expect("relative origin refusal");
    assert!(error.to_string().contains("origin must be absolute"));
}

#[test]
fn push_recursion_uses_cross_key_declaration_order() {
    for (records, expected) in [
        (
            "file:a\0submodule.recurse\ntrue\0file:b\0push.recursesubmodules\nfalse\0",
            false,
        ),
        (
            "file:a\0push.recursesubmodules\nfalse\0file:b\0submodule.recurse\ntrue\0",
            true,
        ),
        (
            "file:a\0push.recursesubmodules\non-demand\0file:b\0submodule.recurse\nfalse\0",
            false,
        ),
        (
            "file:a\0submodule.recurse\nfalse\0file:b\0push.recursesubmodules\ncheck\0",
            true,
        ),
    ] {
        assert_eq!(
            push_recursion_enabled(records.as_bytes()).expect("config"),
            expected
        );
    }
    assert!(push_recursion_enabled(b"file:a\0submodule.recurse\ninvalid\0").is_err());
    // Git aborts on an invalid early value even when a later key disables
    // recursion. Do not approve a plan by considering only the last value.
    for value in ["true", "yes", "1", "ON", "invalid"] {
        let output = format!(
            "file:a\0push.recursesubmodules\n{value}\0file:b\0submodule.recurse\nfalse\0"
        );
        assert!(
            push_recursion_enabled(output.as_bytes()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn push_helper_checks_preserve_disabled_settings_and_defer_hook_reads() {
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
    let environment = BTreeMap::new();
    for (key, value, refused) in [
        ("push.gpgsign", "false", false),
        ("push.gpgsign", "if-asked", true),
        ("push.recursesubmodules", "false", false),
        ("push.recursesubmodules", "on-demand", true),
        ("submodule.recurse", "true", true),
        ("submodule.recurse", "false", false),
        ("push.negotiate", "true", true),
        ("push.negotiate", "false", false),
        ("remote.origin.receivepack", "custom-receive", true),
        ("remote.origin.vcs", "custom", true),
        ("core.gitproxy", "custom-proxy", true),
    ] {
        let output = format!(
            "file:{}\0{key}\n{value}\0",
            repository.common_dir.join("config").display()
        );
        let mut config =
            ConfigState::capture_host(output.as_bytes(), &repository, &environment)
                .expect("config");
        assert_eq!(
            config
                .prepare_push(&repository, &environment, "origin",)
                .is_err(),
            refused,
            "{key}"
        );
        if !refused {
            assert!(
                config
                    .auxiliary
                    .contains(&repository.common_dir.join("hooks/pre-push"))
            );
        }
    }
}
