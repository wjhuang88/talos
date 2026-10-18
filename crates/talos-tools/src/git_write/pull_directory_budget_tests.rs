use super::*;

#[test]
fn borrowed_queries_preserve_allowance_but_owned_capture_cannot_retry_exhaustion() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let budget = ReferenceBudget(Arc::new(Mutex::new((21, 1024 * 1024))));
    let inventory =
        Inventory::with_reference_budget(&root, &root, budget.clone()).expect("frontier");
    let before = *budget.0.lock().expect("allowance");
    assert_eq!(before.0, 1);
    assert!(inventory.iter_paths().count() > 1);
    assert_eq!(inventory.iter_exact_paths(&root, &root).count(), 0);
    assert_eq!(*budget.0.lock().expect("allowance"), before);
    assert!(inventory.capture_paths().is_err());
    let exhausted = *budget.0.lock().expect("allowance");
    assert_eq!(exhausted.0, 0);
    assert!(exhausted.1 < before.1);
    assert!(inventory.capture_paths().is_err());
    assert_eq!(*budget.0.lock().expect("allowance"), exhausted);
}

#[test]
fn retained_directory_revalidation_does_not_restart_discovery() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("refs")).expect("refs");
    std::fs::write(root.join("refs/main"), b"oid").expect("reference");
    std::fs::write(root.join("HEAD"), b"aaaa\n").expect("head");
    let budget = ReferenceBudget::default();
    let mut inventory =
        Inventory::with_reference_budget(&root, &root, budget.clone()).expect("frontier");
    while !inventory.complete() {
        inventory.advance(|_| true).expect("discovery");
    }
    let retained = *budget.0.lock().expect("allowance");
    inventory.revalidate().expect("first comparison");
    inventory.revalidate().expect("second comparison");
    assert_eq!(*budget.0.lock().expect("allowance"), retained);
    assert!(retained.0 < 4096);
    assert!(retained.1 < 4 * 1024 * 1024);
    inventory.bind_references(&root, &root).expect("bodies");
    // HEAD and refs/main use the exact ledger already consumed by directory
    // discovery, rather than a fresh body allowance.
    assert_eq!(
        *budget.0.lock().expect("allowance"),
        (retained.0 - 2, retained.1 - 8)
    );
}

#[test]
fn initial_frontiers_share_allowance_and_failed_construction_does_not_refund() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let budget = ReferenceBudget(Arc::new(Mutex::new((39, 1024 * 1024))));
    let parent =
        Inventory::with_reference_budget(&root, &root, budget.clone()).expect("first frontier");
    let after_parent = *budget.0.lock().expect("allowance");
    assert_eq!(after_parent.0, 19);
    assert!(Inventory::with_reference_budget(&root, &root, budget.clone()).is_err());
    let after_failure = *budget.0.lock().expect("allowance");
    assert_eq!(after_failure.0, 0);
    assert!(after_failure.1 < after_parent.1);
    drop(parent);
    assert_eq!(*budget.0.lock().expect("allowance"), after_failure);
    assert!(Inventory::with_reference_budget(&root, &root, budget.clone()).is_err());
    assert_eq!(*budget.0.lock().expect("allowance"), after_failure);
}

#[test]
fn directory_names_consume_shared_bytes_before_joining() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::write(root.join("reference"), b"oid").expect("reference");
    let required = root.as_os_str().as_encoded_bytes().len() + 1 + "reference".len();
    let budget = ReferenceBudget(Arc::new(Mutex::new((2, required))));
    let retained = children(&root, &budget).expect("first listing");
    assert_eq!(*budget.0.lock().expect("allowance"), (1, 0));
    assert!(children(&root, &budget).is_err());
    assert_eq!(*budget.0.lock().expect("allowance"), (0, 0));
    drop(retained);
    assert_eq!(*budget.0.lock().expect("allowance"), (0, 0));
}

#[test]
fn failed_directory_advance_preserves_frontier_and_cannot_reset_allowance() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("refs")).expect("refs");
    for index in 0..24 {
        std::fs::write(root.join("refs").join(format!("branch-{index}")), b"oid")
            .expect("reference");
    }
    let budget = ReferenceBudget(Arc::new(Mutex::new((90, 1024 * 1024))));
    let mut inventory =
        Inventory::with_reference_budget(&root, &root, budget.clone()).expect("initial frontier");
    loop {
        let frontier = inventory.pending.clone();
        match inventory.advance(|_| true) {
            Ok(()) => assert!(!inventory.complete(), "fixture must exceed allowance"),
            Err(_) => {
                assert_eq!(inventory.pending, frontier);
                break;
            }
        }
    }
    assert_eq!(budget.0.lock().expect("allowance").0, 0);
    let exhausted = *budget.0.lock().expect("allowance");
    assert!(inventory.advance(|_| true).is_err());
    assert_eq!(*budget.0.lock().expect("allowance"), exhausted);
    drop(inventory);
    assert!(Inventory::with_reference_budget(&root, &root, budget.clone()).is_err());
    assert_eq!(*budget.0.lock().expect("allowance"), exhausted);
}
