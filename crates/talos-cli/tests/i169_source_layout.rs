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
        .expect("read model lifecycle source")
        .replace("\r\n", "\n");

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
    assert!(source.contains("No replacement route was published"));

    let helper_start = source
        .find("async fn persist_switch_marker_and_read_final_history(")
        .expect("activation barrier helper definition");
    let helper_end = source[helper_start..]
        .find("\nfn model_switch_markers_match(")
        .map(|offset| helper_start + offset)
        .expect("activation barrier helper end");
    let helper = &source[helper_start..helper_end];

    let fence = helper
        .find(".quiesce_same_session(session)")
        .expect("old runtime retirement");
    let tail_check = helper
        .find(".read_entries()")
        .expect("durable marker tail check");
    let marker_commit = helper
        .find(".append_with_metadata(switch_marker, marker_metadata)")
        .expect("durable marker commit");
    let replay = helper
        .find(".read_messages()")
        .expect("canonical replay after marker commit");

    assert!(fence < tail_check);
    assert!(tail_check < marker_commit);
    assert!(marker_commit < replay);
    assert!(!helper.contains("transition_guard.prepare("));
    assert!(!helper.contains("transition_guard.commit("));
    assert!(!helper.contains("sq_tx_watch_tx.send("));
}
