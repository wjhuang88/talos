//! Fail-closed source inventory: any change to these native boundary files needs
//! a refreshed baseline AND independent protected source review. This is a change
//! detector, not semantic proof of resource ownership or alias absence.
const SOURCES: [&str; 5] = [
    include_str!("../process_boundary.rs"),
    include_str!("resource_acceptance.rs"),
    include_str!("handle_lifecycle_control.rs"),
    include_str!("handle_snapshot.rs"),
    include_str!("windows_foreground.rs"),
];
const EXPECTED: [u64; 5] = [
    0xc080cbefcd2d8c27,
    0xaaf4ee820de4ea11,
    0x77b368f79911fbee,
    0xa95c60b55dcdf0fc,
    0x644841c7dba4298d,
];
fn fingerprint(source: &str) -> u64 {
    source
        .replace("\r\n", "\n")
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        })
}
fn validate(sources: [&str; 5]) -> bool {
    sources
        .into_iter()
        .zip(EXPECTED)
        .all(|(source, expected)| fingerprint(source) == expected)
}
#[test]
fn reviewed_native_sites_and_instrumentation_are_unchanged() {
    assert!(
        validate(SOURCES),
        "native inventory changed: review every acquisition/transfer/release/unsafe site before updating baseline"
    );
}
#[test]
fn inventory_rejects_uncovered_calls_and_removed_or_moved_observation() {
    for (index, original) in SOURCES.into_iter().enumerate() {
        let added = format!("{original}\nunsafe {{ alias::CreatePipe(a,b,c,0); }}");
        let mut changed = SOURCES;
        changed[index] = &added;
        assert!(
            !validate(changed),
            "uncovered acquisition must invalidate inventory"
        );
    }
    for mutation in [
        SOURCES[0].replacen("acquired!(OutRead, out_read);", "", 1),
        SOURCES[0].replacen(
            "acquired!(OutRead, out_read);",
            "acquired!(OutWrite, out_write);",
            1,
        ),
        SOURCES[0].replacen("checked_close!(Thread, thread)", "CloseHandle(thread)", 1),
        SOURCES[1].replacen(
            "let omit = self.negative == Some(Negative::Attributes);",
            "let omit = false;",
            1,
        ),
    ] {
        assert_ne!(mutation, SOURCES[0]);
        let mut changed = SOURCES;
        let index = if mutation.contains("pub(super) struct ResourceAudit") {
            1
        } else {
            0
        };
        changed[index] = &mutation;
        assert!(
            !validate(changed),
            "missing/misplaced instrumentation or operation-path hook must invalidate inventory"
        );
    }
}

#[test]
fn checkout_crlf_preserves_reviewed_native_inventory() {
    let lf = SOURCES.map(|source| source.replace("\r\n", "\n"));
    let crlf = lf.each_ref().map(|source| source.replace("\n", "\r\n"));
    for checkout in [&lf, &crlf] {
        assert!(validate(std::array::from_fn(
            |index| checkout[index].as_str()
        )));
    }
}
