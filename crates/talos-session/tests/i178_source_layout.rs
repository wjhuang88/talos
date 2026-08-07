use std::path::PathBuf;

use talos_session::{PendingSubmissionError, PendingSubmissionRecord, PendingSubmissionStore};

#[test]
fn pending_submission_persistence_helpers_stay_private_behind_store_facade() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/pending_submission");
    let facade = std::fs::read_to_string(root.with_extension("rs")).expect("read facade source");
    let codec = std::fs::read_to_string(root.join("codec.rs")).expect("read codec source");
    let storage = std::fs::read_to_string(root.join("storage.rs")).expect("read storage source");

    assert!(facade.contains("pub struct PendingSubmissionStore"));
    assert!(facade.contains("impl PendingSubmissionStore"));
    assert!(facade.contains("mod codec;"));
    assert!(facade.contains("mod storage;"));
    assert!(!facade.contains("pub mod codec"));
    assert!(!facade.contains("pub mod storage"));
    assert!(codec.contains("fn decode_state"));
    assert!(codec.contains("fn tuple_to_record"));
    assert!(storage.contains("fn ensure_schema"));
    assert!(storage.contains("fn load_runtime_state"));
    assert!(storage.contains("fn prune_tombstones"));
}

#[test]
fn pending_submission_public_api_paths_remain_available() {
    fn assert_store(_: Option<PendingSubmissionStore>) {}
    fn assert_record(_: Option<PendingSubmissionRecord>) {}
    fn assert_error(_: Option<PendingSubmissionError>) {}

    assert_store(None);
    assert_record(None);
    assert_error(None);
}
