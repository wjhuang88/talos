use std::fs;
use std::path::Path;

#[test]
fn transactional_bridge_sources_are_normal_rust_modules() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for removed in ["build.rs", "src/tui_bridge_impl.rs", "src/tests_impl.rs"] {
        assert!(
            !crate_root.join(removed).exists(),
            "temporary I169 source generator artifact must remain removed: {removed}"
        );
    }

    for canonical in ["src/tui_bridge.rs", "src/tests.rs"] {
        let source = fs::read_to_string(crate_root.join(canonical))
            .unwrap_or_else(|error| panic!("read canonical source {canonical}: {error}"));
        assert!(
            !source.contains("OUT_DIR") && !source.contains("include!("),
            "canonical source must not depend on generated include output: {canonical}"
        );
    }
}

#[test]
fn model_switch_marker_durability_precedes_replacement_publication() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(crate_root.join("src/model_lifecycle.rs"))
        .expect("read model lifecycle source");

    let barrier = source
        .find("persist_switch_marker_and_read_final_history(")
        .expect("durable model-switch activation barrier");
    let prepare = source
        .find("transition_guard.prepare(")
        .expect("replacement prepare boundary");
    let commit = source
        .find("transition_guard.commit(")
        .expect("replacement commit boundary");
    let publish = source
        .find("sq_tx_watch_tx.send(")
        .expect("replacement SQ publication");
    let success = source
        .find("MessageSource::System,\n                success_message")
        .expect("model-switch success publication");

    assert!(barrier < prepare);
    assert!(prepare < commit);
    assert!(commit < publish);
    assert!(publish < success);
    assert!(!source.contains("failed to persist model switch marker"));
}
