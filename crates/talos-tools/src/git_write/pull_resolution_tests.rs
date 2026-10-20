use super::*;

#[cfg(windows)]
#[test]
fn verbatim_disk_resolution_records_the_complete_root() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("verbatim root");
    assert!(matches!(
        root.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), std::path::Prefix::VerbatimDisk(_))
    ));
    let evidence = Resolution::capture(&root, false, &mut Budget::default())
        .expect("resolve verbatim disk path");
    assert_eq!(evidence.resolved(), root);
    let first = &evidence.records.first().expect("root record").path;
    assert!(matches!(
        first.components().next_back(),
        Some(Component::RootDir)
    ));
    evidence.revalidate().expect("unchanged identities");
}

#[test]
fn component_queue_reservation_precedes_metadata_discovery() {
    let temp = tempfile::tempdir().expect("fixture");
    let path = temp.path().join("missing/../checkout");
    let mut budget = Budget {
        entries: 4096,
        bytes: 1,
    };
    assert!(Resolution::capture(&path, true, &mut budget).is_err());
    // No component metadata record was charged before the queue was refused.
    assert_eq!((budget.entries, budget.bytes), (4096, 1));
}

#[cfg(unix)]
#[test]
fn link_expansion_refuses_when_only_its_queue_exceeds_remaining_bytes() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let link = root.join("link");
    std::os::unix::fs::symlink(".", &link).expect("link to parent");
    let mut complete = Budget::default();
    let initial_bytes = complete.bytes;
    let resolved = Resolution::capture(&link, false, &mut complete).expect("full allowance");
    assert_eq!(resolved.resolved(), root);
    // The final expansion is `.` and needs queue text but no new record.
    // Removing one byte must refuse, without resetting discovery already done.
    let mut limited = Budget {
        entries: 4096,
        bytes: initial_bytes - complete.bytes - 1,
    };
    assert!(Resolution::capture(&link, false, &mut limited).is_err());
    assert_eq!(limited.entries, complete.entries);
    assert_eq!(limited.bytes, 0);
}

#[test]
fn rejects_files_even_when_cancelled() {
    let temp = tempfile::tempdir().expect("fixture");
    std::fs::write(temp.path().join("file"), b"data").expect("file");
    let mut budget = Budget::default();
    assert!(Resolution::capture(&temp.path().join("file/.."), true, &mut budget).is_err());
}

#[test]
fn missing_components_and_cancelled_ancestors_remain_bound() {
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    let mut budget = Budget::default();
    let evidence = Resolution::capture(&root.join("absent/../checkout"), true, &mut budget)
        .expect("missing checkout");
    assert_eq!(evidence.resolved(), root.join("checkout"));
    evidence.revalidate().expect("unchanged");
    std::fs::create_dir(root.join("absent")).expect("replace absence");
    assert!(evidence.revalidate().is_err());
    assert!(Resolution::capture(&root.join("missing"), false, &mut budget).is_err());
}

#[test]
fn budget_is_cumulative_and_failed_charge_is_atomic() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut budget = Budget::default();
    let evidence = Resolution::capture(temp.path(), false, &mut budget).expect("first");
    let remaining = (budget.entries, budget.bytes);
    evidence.revalidate().expect("replay");
    assert_eq!((budget.entries, budget.bytes), remaining);
    Resolution::capture(temp.path(), false, &mut budget).expect("second");
    assert!(budget.entries < remaining.0);
    let remaining = (budget.entries, budget.bytes);
    assert!(budget.charge(remaining.0 + 1, 0).is_err());
    assert_eq!((budget.entries, budget.bytes), remaining);
    assert!(budget.charge(0, remaining.1 + 1).is_err());
    assert_eq!((budget.entries, budget.bytes), remaining);
    budget.entries = 0;
    assert!(Resolution::capture(temp.path(), false, &mut budget).is_err());
}

#[cfg(unix)]
#[test]
fn link_expansion_precedes_parent_and_detects_redirection() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir_all(root.join("target/deep")).expect("target");
    symlink("target/deep", root.join("link")).expect("link");
    let evidence =
        Resolution::capture(&root.join("link/../checkout"), true, &mut Budget::default())
            .expect("resolve");
    assert_eq!(evidence.resolved(), root.join("target/checkout"));
    std::fs::remove_file(root.join("link")).expect("unlink");
    symlink("target", root.join("link")).expect("redirect");
    assert!(evidence.revalidate().is_err());
    symlink("missing", root.join("dangling")).expect("dangling");
    assert!(Resolution::capture(&root.join("dangling"), true, &mut Budget::default(),).is_err());
}

#[cfg(unix)]
#[test]
fn accepts_64_link_hops_and_refuses_65_or_loop() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().expect("fixture");
    let root = temp.path().canonicalize().expect("root");
    std::fs::create_dir(root.join("end")).expect("end");
    for index in (0..65).rev() {
        let target = if index == 64 {
            "end".into()
        } else {
            format!("link{}", index + 1)
        };
        symlink(target, root.join(format!("link{index}"))).expect("link");
    }
    Resolution::capture(&root.join("link1"), false, &mut Budget::default()).expect("64 hops");
    assert!(Resolution::capture(&root.join("link0"), false, &mut Budget::default(),).is_err());
    symlink("loop", root.join("loop")).expect("loop");
    assert!(Resolution::capture(&root.join("loop"), false, &mut Budget::default(),).is_err());
}
