use std::fs;
use std::path::Path;

#[test]
fn steering_batching_has_no_delimiter_authoritative_api() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let engine = fs::read_to_string(crate_root.join("src/engine.rs"))
        .expect("read conversation Engine source");
    let tests = fs::read_to_string(crate_root.join("src/engine_tests.rs"))
        .expect("read conversation Engine tests");

    assert!(
        !engine.contains("drain_steering_queue_batched"),
        "delimiter-authoritative batch API must remain removed"
    );
    assert!(
        !engine.contains(".join(\"\\n\\n\")"),
        "steering items must not be flattened through a delimiter join"
    );
    assert!(
        !tests.contains("drain_steering_queue_batched"),
        "tests must exercise structured transactions or legacy single-item FIFO drain"
    );
    assert!(
        engine.contains("prepare_steering_submission")
            && engine.contains("commit_prepared_steering")
            && engine.contains("rollback_prepared_steering"),
        "structured prepare/commit/rollback must remain the transactional authority"
    );
}
