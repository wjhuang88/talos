//! Stored-module discovery must not traverse repository contents.

use super::*;

#[test]
fn stored_module_inventory_finds_namespaces_and_nested_modules_without_config_reads() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let parent = root.join("modules/group/HEAD");
    let nested = parent.join("modules/nested");
    std::fs::create_dir_all(&nested).expect("modules");
    for module in [&parent, &nested] {
        std::fs::write(module.join("HEAD"), "not parsed during identity discovery").expect("HEAD");
        std::fs::create_dir(module.join("config")).expect("unread config");
        std::fs::create_dir(module.join("objects")).expect("unread objects");
        #[cfg(unix)]
        std::os::unix::fs::symlink("/unadmitted", module.join("objects/redirect"))
            .expect("unread redirect");
    }
    let mut inventory =
        Inventory::stored_modules(&root, ReferenceBudget::default()).expect("inventory");
    // Rejected frontiers perform no discovery and retain no identities.
    assert!(inventory.advance(|_| false).is_err());
    assert!(inventory.visited.is_empty());
    while !inventory.complete() {
        inventory.advance(|_| true).expect("admitted names");
    }
    assert_eq!(
        inventory.stored_module_roots().collect::<Vec<_>>(),
        vec![parent.as_path(), nested.as_path()]
    );
    assert!(!inventory.visited.contains_key(&parent.join("config")));
    assert!(!inventory.visited.contains_key(&parent.join("objects")));
    inventory.revalidate().expect("unchanged");
    std::fs::create_dir(root.join("modules/added")).expect("new module namespace");
    assert!(inventory.revalidate().is_err());
}

#[cfg(unix)]
#[test]
fn stored_module_inventory_refuses_redirected_namespace() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("modules")).expect("modules");
    std::os::unix::fs::symlink("/unadmitted", root.join("modules/redirect")).expect("redirect");
    let mut inventory =
        Inventory::stored_modules(&root, ReferenceBudget::default()).expect("inventory");
    inventory.advance(|_| true).expect("names only");
    assert!(inventory.advance(|_| true).is_err());
}

#[test]
fn stored_module_inventory_shares_the_invocation_budget() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let budget = ReferenceBudget(Arc::new(Mutex::new((1, 4096))));
    let mut inventory =
        Inventory::stored_modules(&root, budget.clone()).expect("one frontier path");
    assert!(inventory.advance(|_| true).is_err());
    assert!(Inventory::stored_modules(&root, budget).is_err());
}

#[test]
fn startup_target_rejects_missing_checkout_resolution_and_shares_budget() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let budget = ReferenceBudget::default();
    let mut inventory = Inventory::stored_modules(&root, budget.clone()).expect("inventory");
    let target = root.join("missing");
    inventory.worktree_targets.insert(target.clone(), None);
    inventory
        .resolve_worktree_targets()
        .expect("checkout may be absent");
    let before = *budget.0.lock().expect("budget");
    assert!(
        inventory
            .retain_startup_worktree_target(&target)
            .expect("startup")
    );
    assert!(inventory.resolve_worktree_targets().is_err());
    let after = *budget.0.lock().expect("budget");
    assert!(after.0 < before.0 && after.1 < before.1);
    assert!(
        !inventory
            .retain_startup_worktree_target(&target)
            .expect("dedup")
    );
    assert_eq!(*budget.0.lock().expect("budget"), after);
}

#[cfg(unix)]
#[test]
fn startup_target_preserves_filesystem_order_and_link_identity() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("actual/nested")).expect("actual");
    std::fs::create_dir(root.join("actual/checkout")).expect("checkout");
    std::fs::create_dir(root.join("checkout")).expect("lexical decoy");
    std::os::unix::fs::symlink("actual/nested", root.join("link")).expect("link");
    let raw = root.join("link/../checkout");
    let mut inventory =
        Inventory::stored_modules(&root, ReferenceBudget::default()).expect("inventory");
    inventory
        .retain_startup_worktree_target(&raw)
        .expect("freeze");
    assert_eq!(inventory.resolved_startup_worktrees().count(), 0);
    inventory.resolve_worktree_targets().expect("resolve");
    assert_eq!(
        inventory.resolved_startup_worktrees().collect::<Vec<_>>(),
        vec![root.join("actual/checkout").as_path()]
    );
    std::fs::remove_file(root.join("link")).expect("replace link");
    std::os::unix::fs::symlink("actual", root.join("link")).expect("replacement");
    assert!(inventory.revalidate().is_err());
}
