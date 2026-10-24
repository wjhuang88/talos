use super::encode;

#[test]
fn empty_and_nonempty_blocks_have_double_nul_terminators() {
    assert_eq!(encode([]).expect("empty"), vec![0, 0]);
    assert_eq!(
        encode([(vec![65], vec![])]).expect("empty value"),
        vec![65, 61, 0, 0]
    );
}

#[test]
fn preserves_raw_utf16_and_drive_current_directory() {
    assert_eq!(
        encode([(vec![65], vec![0xd800, 0xdc00, 0xdfff])]).expect("raw units"),
        vec![65, 61, 0xd800, 0xdc00, 0xdfff, 0, 0]
    );
    assert!(encode([(vec![61, 67, 58], vec![67, 58, 92])]).is_ok());
}

#[test]
fn invalid_entries_fail_without_returning_a_partial_block() {
    for name in [vec![], vec![0], vec![65, 61], vec![61], vec![61, 49, 58]] {
        assert!(encode([(name, vec![66])]).is_err());
    }
    assert!(encode([(vec![65], vec![66, 0, 67])]).is_err());
}

#[cfg(windows)]
#[test]
fn explicit_snapshot_overrides_case_equivalent_keys_and_preserves_nonunicode() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let block = super::snapshot([
        (OsString::from("Path"), OsString::from("old")),
        (OsString::from("PATH"), OsString::from_wide(&[0xd800])),
    ])
    .expect("explicit environment");
    let entries = block
        .split(|unit| *unit == 0)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].last(), Some(&0xd800));
    assert!(super::snapshot([(OsString::new(), OsString::from("invalid"))]).is_err());
    assert!(super::snapshot([(OsString::from("a=b"), OsString::from("invalid"))]).is_err());
}
