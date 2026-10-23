#[test]
fn pull_metadata_uses_shared_budget_including_missing_pointers() {
    let directory = tempfile::tempdir().expect("fixture");
    let root = directory.path().canonicalize().expect("canonical");
    std::fs::write(root.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
    let budget = super::super::pull_inventory::ReferenceBudget::default();
    // Two captures of each of the three pointers per metadata snapshot,
    // including missing commondir/gitdir, exhaust the same 4096 entries.
    for _ in 0..682 {
        super::Metadata::capture_with_budget(&root, &root, Some(&budget))
            .expect("shared allowance");
    }
    assert!(super::Metadata::capture_with_budget(&root, &root, Some(&budget)).is_err());
    assert!(super::Metadata::capture_with_budget(&root, &root, Some(&budget)).is_err());
    super::Metadata::capture(&root, &root).expect("ordinary receiver remains independent");
}

#[test]
fn pull_metadata_preserves_pointer_size_limit() {
    let directory = tempfile::tempdir().expect("fixture");
    let root = directory.path().canonicalize().expect("canonical");
    std::fs::write(root.join("HEAD"), vec![b'x'; 16 * 1024 + 1]).expect("oversize HEAD");
    let budget = super::super::pull_inventory::ReferenceBudget::default();
    assert!(super::Metadata::capture_with_budget(&root, &root, Some(&budget)).is_err());
}

#[test]
fn pull_pointer_bytes_share_budget_across_sources() {
    let directory = tempfile::tempdir().expect("fixture");
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    for path in [&first, &second] {
        std::fs::write(path, vec![b'x'; 16 * 1024]).expect("pointer");
    }
    let budget = super::super::pull_inventory::ReferenceBudget::default();
    for _ in 0..128 {
        super::Pointer::capture_with_budget(&first, Some(&budget)).expect("first");
        super::Pointer::capture_with_budget(&second, Some(&budget)).expect("second");
    }
    assert!(super::Pointer::capture_with_budget(&first, Some(&budget)).is_err());
    assert!(super::Pointer::capture_with_budget(&second, Some(&budget)).is_err());
}
