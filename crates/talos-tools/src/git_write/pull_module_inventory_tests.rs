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
