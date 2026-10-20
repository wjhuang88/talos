use super::*;

fn capture_target(
    path: &Path,
    allow_missing: bool,
) -> Result<super::super::pull_resolution::Resolution, GitToolError> {
    ReferenceBudget::default().resolve(path, allow_missing)
}

#[test]
fn resolution_and_reference_capture_share_cumulative_allowance() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let reference = root.join("HEAD");
    std::fs::write(&reference, b"aaaa\n").expect("reference");
    let budget = ReferenceBudget::default();
    let resolution = budget.resolve(&root, false).expect("parent resolution");
    let after_resolution = *budget.0.lock().expect("budget");
    assert!(after_resolution.0 < 4096);
    assert!(after_resolution.1 < 4 * 1024 * 1024);
    resolution.revalidate().expect("retained chain");
    assert_eq!(*budget.0.lock().expect("budget"), after_resolution);
    budget.clone().capture(&reference).expect("child reference");
    assert_eq!(
        *budget.0.lock().expect("budget"),
        (after_resolution.0 - 1, after_resolution.1 - 5)
    );
    let before_failure = *budget.0.lock().expect("budget");
    assert!(
        budget
            .clone()
            .resolve(&root.join("missing"), false)
            .is_err()
    );
    let after_failure = *budget.0.lock().expect("budget");
    assert!(after_failure.0 < before_failure.0);
    assert!(after_failure.1 < before_failure.1);
    drop(resolution);
    assert_eq!(*budget.0.lock().expect("budget"), after_failure);
}

#[test]
fn alternates_share_reference_allowance_and_rebinding_does_not_recharge() {
    let temp = tempfile::tempdir().expect("fixture");
    let base = temp.path().canonicalize().expect("base");
    let budget = ReferenceBudget(Arc::new(Mutex::new((3, 1024 * 1024))));
    for name in ["parent", "child"] {
        let root = base.join(name);
        std::fs::create_dir_all(root.join("objects/info")).expect("objects");
        std::fs::write(root.join("objects/info/alternates"), b"").expect("alternate");
        if name == "parent" {
            std::fs::write(root.join("HEAD"), b"aaaa\n").expect("reference");
        }
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("frontier");
        }
        inventory.reference_budget = budget.clone();
        if name == "parent" {
            inventory
                .bind_references(&root, &root)
                .expect("first entry");
            inventory
                .bind_alternates(&root, &root)
                .expect("second entry");
            inventory
                .bind_alternates(&root, &root)
                .expect("retained entry");
            assert_eq!(
                *budget.0.lock().expect("budget"),
                (
                    0,
                    1024 * 1024 - 5 - root.join("HEAD").as_os_str().as_encoded_bytes().len()
                )
            );
        } else {
            assert!(inventory.bind_alternates(&root, &root).is_err());
        }
    }
}

#[test]
fn rejected_nonempty_alternate_consumes_shared_allowance() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("objects/info")).expect("objects");
    std::fs::write(root.join("objects/info/alternates"), b"external\n").expect("alternate");
    let budget = ReferenceBudget(Arc::new(Mutex::new((2, 1024 * 1024))));
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory.reference_budget = budget.clone();
    assert!(inventory.bind_alternates(&root, &root).is_err());
    assert_eq!(*budget.0.lock().expect("budget"), (1, 1024 * 1024 - 9));
}

#[test]
fn reference_allowance_is_shared_across_inventories_and_not_reset_by_rebinding() {
    let temp = tempfile::tempdir().expect("fixture");
    let base = temp.path().canonicalize().expect("base");
    let path_bytes = base
        .join("parent/HEAD")
        .as_os_str()
        .as_encoded_bytes()
        .len();
    let budget = ReferenceBudget(Arc::new(Mutex::new((10, path_bytes + 9))));
    let mut inventories = Vec::new();
    for name in ["parent", "child"] {
        let root = base.join(name);
        std::fs::create_dir(&root).expect("repository");
        std::fs::write(root.join("HEAD"), b"aaaa\n").expect("reference");
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        inventory.reference_budget = budget.clone();
        inventories.push((root, inventory));
    }
    let (root, parent) = &mut inventories[0];
    parent.bind_references(root, root).expect("first capture");
    parent
        .bind_references(root, root)
        .expect("retained evidence");
    assert_eq!(*budget.0.lock().expect("budget"), (8, 4));
    let (root, child) = &mut inventories[1];
    assert!(child.bind_references(root, root).is_err());
    assert_eq!(*budget.0.lock().expect("failed capture budget"), (7, 4));
    // Dropping an inventory does not refund cumulative discovery work.
    drop(inventories);
    let root = base.join("parent");
    let mut replacement = Inventory::new(&root, &root);
    while !replacement.complete() {
        replacement.advance(|_| true).expect("replacement frontier");
    }
    replacement.reference_budget = budget;
    assert!(replacement.bind_references(&root, &root).is_err());
}

#[test]
fn shared_reference_entry_allowance_refuses_after_exhaustion() {
    let temp = tempfile::tempdir().expect("fixture");
    let path = temp.path().join("HEAD");
    std::fs::write(&path, b"aaaa\n").expect("reference");
    let budget = ReferenceBudget(Arc::new(Mutex::new((1, 100))));
    let retained = budget.capture(&path).expect("first entry");
    assert!(budget.clone().capture(&path).is_err());
    assert_eq!(*budget.0.lock().expect("budget"), (0, 95));
    retained
        .revalidate(&path)
        .expect("existing evidence remains valid");
}

#[test]
fn directory_revalidation_compares_the_retained_set_without_rebaselining() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    for name in ["a", "b", "c"] {
        std::fs::write(root.join(name), b"contents").expect("entry");
    }
    let retained = children(&root, &ReferenceBudget::default()).expect("capture names");
    revalidate_children(&root, &retained).expect("same set");
    std::fs::remove_file(root.join("b")).expect("remove entry");
    assert!(revalidate_children(&root, &retained).is_err());
    std::fs::write(root.join("replacement"), b"contents").expect("same count, new name");
    assert!(revalidate_children(&root, &retained).is_err());
    std::fs::remove_file(root.join("replacement")).expect("remove replacement");
    std::fs::write(root.join("b"), b"contents").expect("restore name");
    revalidate_children(&root, &retained).expect("original set");
    // This checks names only; entry identities are validated independently.
    assert_eq!(
        retained,
        children(&root, &ReferenceBudget::default()).expect("unchanged baseline")
    );
}

#[test]
fn reftable_manifests_cannot_escape_admitted_backends() {
    for location in ["reftable", "worktrees/linked/reftable"] {
        for contents in [
            "../outside\n",
            "sub/file\n",
            "sub\\file\n",
            "C:outside\n",
            ".\n",
            "..\n",
            "bad\0name\n",
            "NUL.ref\n",
            "con\n",
            "COM1.ref\n",
            "lpt9.ref\n",
            "COM¹.ref\n",
            "name.\n",
            "name \n",
            "wild*.ref\n",
        ] {
            let temp = tempfile::tempdir().expect("repo");
            let root = temp.path().canonicalize().expect("root");
            let table = root.join(location);
            std::fs::create_dir_all(&table).expect("backend");
            std::fs::write(table.join("tables.list"), contents).expect("list");
            let mut inventory = Inventory::new(&root, &root);
            while !inventory.complete() {
                inventory.advance(|_| true).expect("frontier");
            }
            assert!(
                inventory.bind_references(&root, &root).is_err(),
                "{location}: {contents:?}"
            );
        }
    }
    validate_table_names(b"one.ref\ntwo.ref\n").expect("ordinary tables");
}

#[test]
fn reftable_files_require_frontier_admission_and_bind_content() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let table = root.join("reftable");
    std::fs::create_dir(&table).expect("backend");
    std::fs::write(table.join("tables.list"), "one.ref\n").expect("list");
    std::fs::write(table.join("one.ref"), b"table contents").expect("table");
    let mut inventory = Inventory::new(&root, &root);
    let admitted = inventory.paths();
    inventory
        .advance(|path| admitted.contains(&path.to_owned()))
        .expect("roots");
    assert!(
        inventory
            .advance(|path| admitted.contains(&path.to_owned()))
            .is_err()
    );
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory
        .bind_references(&root, &root)
        .expect("content evidence");
    inventory.revalidate().expect("unchanged");
    std::fs::write(table.join("one.ref"), b"changed content").expect("in-place edit");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn linked_reftable_list_is_reference_content() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let table = root.join("worktrees/linked/reftable");
    std::fs::create_dir_all(&table).expect("backend");
    let list = table.join("tables.list");
    std::fs::write(&list, "one.ref\n").expect("list");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory
        .bind_references(&root, &root)
        .expect("content evidence");
    std::fs::write(&list, "two.ref\n").expect("in-place list edit");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn main_worktree_config_is_bound_outside_the_current_config_chain() {
    let temp = tempfile::tempdir().expect("repo");
    let common = temp.path().canonicalize().expect("common");
    let gitdir = common.join("worktrees/current");
    std::fs::create_dir_all(&gitdir).expect("linked gitdir");
    let config = common.join("config.worktree");
    std::fs::write(&config, "[core]\n bare = true\n").expect("main config");
    let mut inventory = Inventory::new(&gitdir, &common);
    assert!(inventory.advance(|path| path != config).is_err());
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    inventory
        .bind_references(&gitdir, &common)
        .expect("evidence");
    assert!(inventory.references.iter().any(|(path, _)| path == &config));
    std::fs::write(&config, "[core]\n bare = false\n").expect("edit main config");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn pointer_discovery_uses_retained_bytes_and_defers_source_revalidation() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let registration = root.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    let pointer = registration.join("gitdir");
    std::fs::write(&pointer, "../../original/.git\n").expect("pointer");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory.bind_references(&root, &root).expect("evidence");
    // Remove the source entirely: discovery must neither reopen it nor
    // silently replace the previously admitted bytes with a fresh read.
    std::fs::remove_file(&pointer).expect("remove source");
    assert!(
        inventory
            .discover_worktree_targets(&root)
            .expect("pure parse")
    );
    assert!(
        inventory
            .worktree_targets
            .contains_key(&registration.join("../../original"))
    );
    // Parsing does not make stale evidence executable. The independent
    // source check and binding both reject the changed registration.
    assert!(inventory.revalidate().is_err());
    assert!(
        inventory
            .bind_worktree_targets(|path| Some(path.to_owned()))
            .is_err()
    );
    assert!(inventory.worktree_targets.values().all(Option::is_none));
}

#[test]
fn linked_pointer_targets_wait_for_fresh_admission() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let registration = root.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    let target = root.join("separately-admitted");
    std::fs::write(
        registration.join("gitdir"),
        "../../separately-admitted/.git \t\n",
    )
    .expect("pointer");
    std::fs::write(registration.join("commondir"), "../..\r\n").expect("common");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory
        .bind_references(&root, &root)
        .expect("pointer evidence");
    let previous = inventory.paths();
    // Target does not exist: discovery must not resolve it.
    assert!(
        inventory
            .discover_worktree_targets(&root)
            .expect("raw targets")
    );
    let raw_target = registration.join("../../separately-admitted");
    assert!(inventory.worktree_targets.contains_key(&raw_target));
    assert!(
        inventory
            .worktree_targets
            .contains_key(&registration.join("../.."))
    );
    assert!(
        inventory
            .bind_worktree_targets(|path| previous
                .contains(&path.to_owned())
                .then(|| path.to_owned()))
            .is_err()
    );
    assert!(inventory.worktree_targets.values().all(Option::is_none));
    std::fs::create_dir(&target).expect("target created independently");
    inventory
        .resolve_worktree_targets()
        .expect("resolution admitted");
    assert!(
        inventory
            .bind_worktree_targets(|_| Some(root.clone()))
            .is_err()
    );
    inventory
        .resolve_worktree_targets()
        .expect("resolution admitted");
    inventory
        .bind_worktree_targets(|path| path.canonicalize().ok())
        .expect("fresh admission");
    assert!(
        !inventory
            .discover_worktree_targets(&root)
            .expect("stable targets")
    );
    inventory.revalidate().expect("unchanged targets");
    std::fs::rename(&target, root.join("saved-target")).expect("move target");
    std::fs::create_dir(&target).expect("replacement target");
    #[cfg(any(unix, windows))]
    assert!(inventory.revalidate().is_err());
}

#[test]
fn missing_worktree_target_requires_checkout_mode_and_retains_parent_identity() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let parent = root.join("checkout-parent");
    std::fs::create_dir(&parent).expect("parent");
    let path = parent.join("missing");
    assert!(capture_target(&path, false).is_err());
    let evidence = capture_target(&path, true).expect("missing checkout");
    assert_eq!(evidence.resolved(), path);
    std::fs::rename(&parent, root.join("saved-parent")).expect("move parent");
    std::fs::create_dir(&parent).expect("replace parent");
    #[cfg(any(unix, windows))]
    assert_ne!(evidence, capture_target(&path, true).expect("replacement"));
}

#[test]
fn missing_worktree_suffix_resolves_parent_components() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    // PathBuf::push normalizes `..` for Windows verbatim roots. Preserve the
    // raw traversal so this fixture actually exercises cancelled components.
    let mut spelling = root.as_os_str().to_owned();
    spelling.push("/missing/deep/../../checkout".replace('/', std::path::MAIN_SEPARATOR_STR));
    let path = PathBuf::from(spelling);
    assert!(
        path.components()
            .any(|part| part == std::path::Component::ParentDir)
    );
    assert!(capture_target(&path, false).is_err());
    let evidence = capture_target(&path, true).expect("forgiving target");
    assert_eq!(evidence.resolved(), root.join("checkout"));
    std::fs::write(root.join("missing"), b"not a directory").expect("replace absence");
    assert!(capture_target(&path, true).is_err());
}

#[cfg(unix)]
#[test]
fn missing_worktree_suffix_keeps_existing_symlink_parent_semantics() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("actual/nested")).expect("actual");
    std::os::unix::fs::symlink(root.join("actual/nested"), root.join("link"))
        .expect("existing link");
    let path = root.join("link/../missing/../checkout");
    let evidence = capture_target(&path, true).expect("forgiving target");
    assert_eq!(evidence.resolved(), root.join("actual/checkout"));
    let interleaved = root.join("missing/../link/../checkout");
    assert_eq!(
        capture_target(&interleaved, true)
            .expect("interleaved symlink and parent")
            .resolved(),
        root.join("actual/checkout")
    );
    std::fs::create_dir_all(root.join("elsewhere/nested")).expect("redirect target");
    std::os::unix::fs::symlink(root.join("elsewhere/nested"), root.join("actual/missing"))
        .expect("redirect absent component");
    assert_ne!(
        evidence,
        capture_target(&path, true).expect("redirected target")
    );
}

#[cfg(unix)]
#[test]
fn cancelling_missing_suffix_resolves_later_existing_symlink() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("actual")).expect("target");
    std::os::unix::fs::symlink(root.join("actual"), root.join("link")).expect("link");
    let path = root.join("missing/../link/checkout");
    let evidence = capture_target(&path, true).expect("forgiving target");
    assert_eq!(evidence.resolved(), root.join("actual/checkout"));
    std::fs::rename(root.join("actual"), root.join("saved")).expect("move target");
    std::fs::create_dir(root.join("actual")).expect("replace target");
    assert_ne!(
        evidence,
        capture_target(&path, true).expect("replacement target")
    );
}

#[cfg(unix)]
#[test]
fn missing_worktree_target_rejects_dangling_symlink_suffix() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let link = root.join("dangling");
    std::os::unix::fs::symlink(root.join("missing"), &link).expect("link");
    assert!(capture_target(&link.join("checkout"), true).is_err());
}

#[test]
fn relative_missing_worktree_backlink_binds_absence_and_ancestor() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let registration = root.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    std::fs::write(registration.join("gitdir"), "../../missing/deep/.git\n").expect("backlink");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("inventory");
    }
    inventory.bind_references(&root, &root).expect("references");
    assert!(
        inventory
            .discover_worktree_targets(&root)
            .expect("discover")
    );
    assert!(inventory.bind_worktree_targets(|_| None).is_err());
    inventory
        .resolve_worktree_targets()
        .expect("resolution admitted");
    inventory
        .bind_worktree_targets(|_| Some(root.join("missing/deep")))
        .expect("admitted missing checkout");
    inventory.revalidate().expect("stable absence");
    std::fs::create_dir(root.join("missing")).expect("create ancestor");
    assert!(inventory.revalidate().is_err());
}

#[cfg(unix)]
#[test]
fn relative_backlink_resolves_interleaved_missing_and_symlink_after_admission() {
    let temp = tempfile::tempdir().expect("repo");
    let root = temp.path().canonicalize().expect("root");
    let registration = root.join("worktrees/linked");
    std::fs::create_dir_all(&registration).expect("registration");
    std::fs::create_dir_all(root.join("actual/nested")).expect("target");
    std::os::unix::fs::symlink(root.join("actual/nested"), root.join("link"))
        .expect("existing link");
    std::fs::write(
        registration.join("gitdir"),
        "../../missing/../link/../checkout/.git\n",
    )
    .expect("backlink");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("inventory");
    }
    inventory.bind_references(&root, &root).expect("references");
    assert!(
        inventory
            .discover_worktree_targets(&root)
            .expect("discover")
    );
    assert!(inventory.bind_worktree_targets(|_| None).is_err());
    inventory
        .resolve_worktree_targets()
        .expect("resolution admitted");
    // A grant for the lexical interpretation of the original raw spelling
    // cannot authorize Git's different component-wise result.
    assert!(
        inventory
            .bind_worktree_targets(|_| Some(root.join("checkout")))
            .is_err()
    );
    inventory
        .bind_worktree_targets(|_| Some(root.join("actual/checkout")))
        .expect("exact resolved admission");
    inventory.revalidate().expect("stable target");
    std::fs::rename(root.join("actual"), root.join("saved")).expect("move target");
    std::fs::create_dir(root.join("actual")).expect("replace target");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn absolute_worktree_backlink_does_not_resolve_pruned_checkout() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let registration = root.join("worktrees/pruned");
    std::fs::create_dir_all(&registration).expect("registration");
    let missing = root.join("missing/.git");
    std::fs::write(
        registration.join("gitdir"),
        format!("{}\n", missing.display()),
    )
    .expect("pointer");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("frontier");
    }
    inventory.bind_references(&root, &root).expect("evidence");
    assert!(
        !inventory
            .discover_worktree_targets(&root)
            .expect("absolute text only")
    );
    assert!(inventory.worktree_targets.is_empty());
    inventory
        .bind_worktree_targets(|_| None)
        .expect("no target access");
}

#[test]
fn linked_worktree_common_refs_get_a_separate_frontier() {
    let repo = tempfile::tempdir().expect("repo");
    let repo = repo.path().canonicalize().expect("canonical repo");
    let external = tempfile::tempdir().expect("external common dir");
    let external = external.path().canonicalize().expect("canonical external");
    std::fs::create_dir_all(repo.join("checkout")).expect("checkout");
    std::fs::create_dir_all(repo.join("worktrees/linked")).expect("registration");
    std::fs::create_dir_all(external.join("refs/heads")).expect("external refs");
    std::fs::write(
        repo.join("worktrees/linked/gitdir"),
        "../../checkout/.git\n",
    )
    .expect("backlink");
    std::fs::write(
        repo.join("worktrees/linked/commondir"),
        format!("{}\n", external.display()),
    )
    .expect("common pointer");
    let private_ref = external.join("refs/heads/external-only");
    std::fs::write(&private_ref, "0123456789012345678901234567890123456789\n")
        .expect("external ref");

    let mut inventory = Inventory::new(&repo, &repo);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("repo frontier");
    }
    inventory
        .bind_references(&repo, &repo)
        .expect("repo evidence");
    assert!(
        inventory
            .discover_worktree_targets(&repo)
            .expect("raw targets")
    );
    inventory
        .resolve_worktree_targets()
        .expect("resolution admitted");
    assert!(
        inventory
            .bind_worktree_targets(|path| path.canonicalize().ok())
            .expect("target binding and fresh frontier")
    );
    let external_refs = external.join("refs");
    let external_packed_refs = external.join("packed-refs");
    assert!(inventory.pending.contains(&external_refs));
    assert!(inventory.pending.contains(&external_packed_refs));
    assert!(inventory.pending.contains(&external.join("reftable")));

    // The external descendants cannot be inspected using the old frontier.
    assert!(
        inventory
            .advance(|path| !path.starts_with(&external))
            .is_err()
    );
    assert!(inventory.advance(|_| true).is_ok());
    while !inventory.complete() {
        inventory.advance(|_| true).expect("external ref frontier");
    }
    inventory
        .bind_references(&repo, &repo)
        .expect("external evidence");
    assert!(inventory.references.iter().any(|(path, evidence)| {
        path == &private_ref
            && matches!(&evidence.entry, super::super::snapshot::Entry::File(_, bytes)
                if bytes == b"0123456789012345678901234567890123456789\n")
    }));
    std::fs::write(&private_ref, "fedcba9876543210fedcba9876543210fedcba98\n")
        .expect("replace ref contents");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn linked_worktree_references_require_admission_and_retain_contents() {
    for leaf in [
        "HEAD",
        "refs/heads/private",
        "reftable/tables.list",
        "reftable/one.ref",
        "packed-refs",
        "gitdir",
        "commondir",
    ] {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let path = root.join("worktrees/linked").join(leaf);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
        std::fs::write(&path, format!("{}\n", "a".repeat(40))).expect("reference");
        let mut inventory = Inventory::new(&root, &root);
        let initial = inventory.paths();
        assert!(initial.contains(&root.join("worktrees")));
        inventory
            .advance(|path| initial.contains(&path.to_owned()))
            .expect("root admission");
        assert!(
            inventory
                .advance(|path| initial.contains(&path.to_owned()))
                .is_err()
        );
        assert!(inventory.bind_references(&root, &root).is_err());
        while !inventory.complete() {
            inventory
                .advance(|_| true)
                .expect("fresh frontier admission");
        }
        inventory
            .bind_references(&root, &root)
            .expect("bind references");
        assert!(inventory.references.iter().any(|(bound, _)| bound == &path));
        inventory
            .bind_references(&root, &root)
            .expect("unchanged retained references can be rebound");
        std::fs::write(&path, format!("{}\n", "b".repeat(40))).expect("in-place change");
        assert!(inventory.revalidate().is_err());
        assert!(inventory.bind_references(&root, &root).is_err());
    }
}

#[test]
fn graph_boundaries_bind_in_place_changes_and_absence() {
    for name in ["shallow", "info/grafts"] {
        for exists in [false, true] {
            let temp = tempfile::tempdir().expect("fixture");
            let root = temp.path().canonicalize().expect("root");
            std::fs::create_dir(root.join("info")).expect("info");
            let path = root.join(name);
            if exists {
                std::fs::write(&path, format!("{}\n", "a".repeat(40)))
                    .expect("initial graph boundary");
            }
            let mut inventory = Inventory::new(&root, &root);
            assert!(inventory.bind_references(&root, &root).is_err());
            while !inventory.complete() {
                inventory.advance(|_| true).expect("admitted frontier");
            }
            inventory.bind_references(&root, &root).expect("bind graph");
            inventory.revalidate().expect("unchanged graph");
            std::fs::write(&path, format!("{}\n", "b".repeat(40))).expect("change graph boundary");
            assert!(inventory.revalidate().is_err(), "{name}, exists={exists}");
        }
    }
}

#[test]
fn replacement_symbolic_target_waits_for_full_reference_frontier() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("refs/replace")).expect("replacement directory");
    std::fs::create_dir_all(root.join("refs/heads/a/b")).expect("target directory");
    std::fs::write(
        root.join("refs/replace/object"),
        b"ref: refs/heads/a/b/target\n",
    )
    .expect("symbolic replacement");
    std::fs::write(root.join("refs/heads/a/b/target"), b"replacement\n").expect("target");
    let mut inventory = Inventory::new(&root, &root);
    for _ in 0..3 {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    assert!(inventory.subtree_complete(&root.join("refs/replace")));
    assert!(!inventory.subtree_complete(&root.join("refs")));
    let old_paths = inventory.paths();
    inventory.advance(|_| true).expect("target name discovery");
    assert!(
        inventory
            .advance(|path| old_paths.iter().any(|old| old == path))
            .is_err()
    );
    assert!(!inventory.subtree_complete(&root.join("refs")));
    inventory.advance(|_| true).expect("target admission");
    assert!(inventory.subtree_complete(&root.join("refs")));
}

#[test]
fn replacement_refs_wait_for_descendants_and_absence_is_bound() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
    std::fs::write(root.join("refs/replace/object"), b"replacement").expect("ref");
    let mut inventory = Inventory::new(&root, &root);
    inventory.advance(|_| true).expect("roots");
    assert!(!inventory.subtree_complete(&root.join("refs/replace")));
    inventory.advance(|_| true).expect("replace directory");
    assert!(!inventory.subtree_complete(&root.join("refs/replace")));
    inventory.advance(|_| true).expect("reference");
    assert!(inventory.subtree_complete(&root.join("refs/replace")));
    assert!(inventory.subtree_complete(&root.join("refs/missing")));
    std::fs::create_dir(root.join("refs/missing")).expect("new subtree");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn child_symbolic_reference_cannot_escape_retained_namespace() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
    std::fs::write(root.join("refs/replace/object"), b"ref: FETCH_HEAD\n").expect("pseudo target");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    assert!(inventory.bind_references(&root, &root).is_err());
}

#[test]
fn child_metadata_waits_for_its_own_admission_and_changes_refuse() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("objects")).expect("objects");
    std::fs::create_dir(root.join("objects/aa")).expect("fanout");
    std::fs::write(root.join("objects/aa/object"), b"object").expect("object");
    let mut inventory = Inventory::new(&root, &root);
    assert!(!inventory.subtree_complete(&root.join("objects")));
    let initial = inventory.paths();
    inventory
        .advance(|path| initial.iter().any(|allowed| allowed == path))
        .expect("first frontier");
    assert!(inventory.paths().contains(&root.join("objects/aa")));
    assert!(!inventory.subtree_complete(&root.join("objects")));
    assert!(
        inventory
            .advance(|path| initial.iter().any(|allowed| allowed == path))
            .is_err()
    );
    inventory.advance(|_| true).expect("second frontier");
    assert!(!inventory.subtree_complete(&root.join("objects")));
    inventory.advance(|_| true).expect("files");
    assert!(inventory.subtree_complete(&root.join("objects")));
    assert!(inventory.complete());
    std::fs::write(root.join("shallow"), b"new").expect("new leaf");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn child_alternates_wait_for_inventory_and_bind_in_place_changes() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("objects/info")).expect("objects");
    let path = root.join("objects/info/alternates");
    std::fs::write(&path, b"").expect("empty");
    let mut inventory = Inventory::new(&root, &root);
    assert!(inventory.bind_alternates(&root, &root).is_err());
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    inventory
        .bind_alternates(&root, &root)
        .expect("empty alternates");
    std::fs::write(&path, b"/unadmitted/object/store\n").expect("redirect");
    assert!(inventory.revalidate().is_err());
    let mut changed = Inventory::new(&root, &root);
    while !changed.complete() {
        changed.advance(|_| true).expect("admitted frontier");
    }
    assert!(changed.bind_alternates(&root, &root).is_err());
}

#[test]
fn child_absent_alternates_are_bound_by_directory_listing() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("objects/info")).expect("objects");
    let mut inventory = Inventory::new(&root, &root);
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    inventory
        .bind_alternates(&root, &root)
        .expect("absent alternates");
    assert!(inventory.alternates.is_empty());
    std::fs::write(
        root.join("objects/info/http-alternates"),
        b"http://unadmitted.invalid\n",
    )
    .expect("new redirect");
    assert!(inventory.revalidate().is_err());
}

#[test]
fn child_reference_contents_bind_in_place_symbolic_target_changes() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
    std::fs::create_dir_all(root.join("refs/heads")).expect("heads");
    let target = root.join("refs/heads/target");
    std::fs::write(&target, format!("{}\n", "a".repeat(40))).expect("target");
    std::fs::write(
        root.join("refs/replace/object"),
        b"ref: refs/heads/target\n",
    )
    .expect("symbolic replacement");
    let mut inventory = Inventory::new(&root, &root);
    assert!(inventory.bind_references(&root, &root).is_err());
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted frontier");
    }
    inventory
        .bind_references(&root, &root)
        .expect("reference bytes");
    std::fs::write(&target, format!("{}\n", "b".repeat(40))).expect("in-place change");
    assert!(inventory.revalidate().is_err());
}

#[cfg(unix)]
#[test]
fn symlink_child_is_not_traversed_during_parent_discovery() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("objects")).expect("objects");
    std::os::unix::fs::symlink("/unapproved", root.join("objects/escape")).expect("link");
    let mut inventory = Inventory::new(&root, &root);
    inventory.advance(|_| true).expect("names only");
    assert!(inventory.advance(|_| true).is_err());
}

#[cfg(unix)]
#[test]
fn discovered_directory_replacement_refuses_even_with_closed_scope() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let metadata = root.join("metadata");
    let external = root.join("external");
    std::fs::create_dir(&metadata).expect("metadata");
    std::fs::create_dir_all(metadata.join("refs/heads")).expect("refs");
    std::fs::create_dir(&external).expect("external");
    std::fs::write(external.join("outside"), b"unapproved reference").expect("outside");
    let mut inventory = Inventory::new(&metadata, &metadata);
    inventory.advance(|_| true).expect("discover refs");
    inventory.advance(|_| true).expect("discover heads");
    std::fs::rename(metadata.join("refs/heads"), metadata.join("saved-heads"))
        .expect("move discovered directory");
    std::os::unix::fs::symlink(&external, metadata.join("refs/heads"))
        .expect("redirect discovered directory");
    assert!(inventory.revalidate().is_err());
    assert!(inventory.advance(|_| true).is_err());
    assert!(
        !inventory
            .paths()
            .contains(&metadata.join("refs/heads/outside"))
    );
}

#[cfg(unix)]
#[test]
fn redirected_metadata_parent_refuses_before_initial_enumeration() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let external = root.join("external");
    std::fs::create_dir_all(external.join("refs")).expect("external refs");
    std::fs::write(external.join("refs/outside"), b"unapproved reference").expect("outside");
    let redirected = root.join("metadata");
    std::os::unix::fs::symlink(&external, &redirected).expect("redirect root");
    let mut inventory = Inventory::new(&redirected, &redirected);
    assert!(inventory.advance(|_| true).is_err());
    assert!(!inventory.paths().contains(&redirected.join("refs/outside")));
}

#[test]
fn added_directory_entry_invalidates_discovered_inventory() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("objects")).expect("objects");
    let mut inventory = Inventory::new(&root, &root);
    inventory.advance(|_| true).expect("initial inventory");
    assert!(inventory.complete());
    std::fs::write(root.join("objects/unadmitted"), b"new object").expect("new entry");
    assert!(inventory.revalidate().is_err());
    assert!(inventory.advance(|_| true).is_err());
}
