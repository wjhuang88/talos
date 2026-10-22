#[test]
fn child_startup_layout_matches_native_selection_rules() {
    for case in [
        "stale-private",
        "linked-suppressed",
        "private-override",
        "bare",
        "no-version",
    ] {
        let root = tempfile::tempdir().expect("fixture");
        let worktree = if cfg!(windows) {
            root.path().to_owned()
        } else {
            root.path().canonicalize().expect("canonical")
        };
        assert!(
            std::process::Command::new("git")
                .current_dir(&worktree)
                .args(["init"])
                .output()
                .expect("init")
                .status
                .success()
        );
        let gitdir = worktree.join(".git");
        let selected = worktree.join("selected");
        std::fs::create_dir(&selected).expect("selected worktree");
        let has_common = case == "linked-suppressed";
        if has_common {
            // Presence matters even if commondir resolves back to gitdir.
            std::fs::write(gitdir.join("commondir"), ".\n").expect("commondir");
        }
        let version = if case == "no-version" {
            ""
        } else {
            "repositoryFormatVersion=0\n"
        };
        let bare = case == "bare";
        let raw = if matches!(
            case,
            "linked-suppressed" | "private-override" | "bare" | "no-version"
        ) {
            worktree.join("unopened-missing")
        } else {
            selected.clone()
        };
        let extension = if case == "private-override" {
            "[extensions]\nworktreeConfig=true\n"
        } else {
            ""
        };
        std::fs::write(
            gitdir.join("config"),
            format!("[core]\n{version}bare={bare}\n{extension}"),
        )
        .expect("config");
        let set_worktree = |file: &std::path::Path, target: &std::path::Path| {
            let output = std::process::Command::new("git")
                .args(["config", "--file"])
                .arg(file)
                .arg("core.worktree")
                .arg(target)
                .output()
                .expect("configure worktree");
            assert!(output.status.success(), "{output:?}");
        };
        set_worktree(&gitdir.join("config"), &raw);
        let private_target = if case == "private-override" {
            selected.clone()
        } else {
            worktree.join("ignored-missing")
        };
        set_worktree(&gitdir.join("config.worktree"), &private_target);
        let layout =
            super::ChildStartupLayout::capture(&gitdir, &gitdir, has_common).expect("layout");
        let expected = if matches!(case, "stale-private" | "private-override") {
            BTreeSet::from([selected.clone()])
        } else {
            BTreeSet::new()
        };
        assert_eq!(layout.targets, expected, "{case}");
        let output = std::process::Command::new("git")
            .current_dir(&worktree)
            .arg("--git-dir")
            .arg(&gitdir)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .expect("native startup");
        if bare {
            assert!(!output.status.success(), "bare has no worktree");
        } else {
            assert!(
                output.status.success(),
                "{case}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                PathBuf::from(String::from_utf8(output.stdout).expect("UTF8").trim())
                    .canonicalize()
                    .expect("native worktree identity"),
                expected.iter().next().unwrap_or(&worktree)
                    .canonicalize()
                    .expect("expected worktree identity"),
                "{case}"
            );
        }
    }
}

#[test]
fn child_startup_layout_retains_raw_target_hidden_by_include() {
    let root = tempfile::tempdir().expect("fixture");
    let gitdir = root.path().join("metadata");
    std::fs::create_dir(&gitdir).expect("metadata");
    std::fs::write(gitdir.join("config"),
        "[core]\nrepositoryFormatVersion=0\nworktree=../unapproved\n[include]\npath=override.conf\n").expect("config");
    std::fs::write(
        gitdir.join("override.conf"),
        "[core]\nworktree=../approved\n",
    )
    .expect("include");
    let layout = super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("layout");
    assert_eq!(
        layout.targets,
        BTreeSet::from([gitdir.join("../unapproved")])
    );
    assert!(!root.path().join("unapproved").exists());
    layout.revalidate().expect("unchanged");
    std::fs::write(
        gitdir.join("config.worktree"),
        "[core]\nworktree=../another\n",
    )
    .expect("late config");
    layout
        .revalidate()
        .expect("disabled private config is ignored");
    let layout =
        super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("new layout");
    assert!(!layout.targets.contains(&gitdir.join("../another")));
    std::fs::write(gitdir.join("config"),
        "[core]\nrepositoryFormatVersion=0\nworktree=../unapproved\n[extensions]\nworktreeConfig=true\n").expect("enabled config");
    assert!(layout.revalidate().is_err());
    let enabled =
        super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("enabled layout");
    assert_eq!(enabled.targets, BTreeSet::from([gitdir.join("../another")]));
}

#[test]
fn native_pull_startup_sources_obey_local_and_shared_limits() {
    let root = tempfile::tempdir().expect("fixture");
    let shared = super::super::pull_inventory::ReferenceBudget::default();
    let mut bytes = vec![b' '; 256 * 1024 - 1];
    bytes[0] = b'#';
    for index in 0..17 {
        let dir = root.path().join(index.to_string());
        std::fs::create_dir(&dir).expect("child");
        std::fs::write(dir.join("config"), &bytes).expect("source");
        let result = ChildStartupLayout::capture_native_pull(&dir, &dir, false, &shared.clone());
        assert_eq!(result.is_ok(), index < 16, "child {index}");
    }
    let dir = root.path().join("oversized");
    std::fs::create_dir(&dir).expect("child");
    bytes.resize(256 * 1024 + 1, b' ');
    std::fs::write(dir.join("config"), bytes).expect("oversized source");
    assert!(ChildStartupLayout::capture_native_pull(
        &dir, &dir, false, &super::super::pull_inventory::ReferenceBudget::default(),
    ).is_err());
}
