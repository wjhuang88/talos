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
fn model_switch_activation_durability_precedes_replacement_publication() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(crate_root.join("src/model_lifecycle.rs"))
        .expect("read model lifecycle source")
        .replace("\r\n", "\n");

    let barrier = source
        .find("establish_model_activation_and_read_final_history(")
        .expect("durable model activation barrier");
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
    assert!(source.contains("previous_variant"));
    assert!(source.contains("SessionModelIdentity::new("));
    assert!(source.contains("No replacement route was published"));

    let helper_start = source
        .find("async fn persist_model_activation_and_read_final_history(")
        .expect("activation barrier helper definition");
    let helper_end = source[helper_start..]
        .find("\n/// Test-only compatibility harness")
        .map(|offset| helper_start + offset)
        .expect("activation barrier helper end");
    let helper = &source[helper_start..helper_end];

    let fence = helper
        .find(".quiesce_same_session(session)")
        .expect("old runtime retirement");
    let activation = helper
        .find("SessionModelActivation::new(")
        .expect("exact generation + variant-aware activation identity");
    let tail_check = helper
        .find("model_activation_tail(session)")
        .expect("machine-readable activation tail check");
    let marker_commit = helper
        .find(".append_with_metadata(&marker, marker_metadata)")
        .expect("durable activation commit");
    let replay = helper
        .find("verified_activation_history(session, &activation, &marker)")
        .expect("canonical replay after activation commit");

    assert!(fence < activation);
    assert!(activation < tail_check);
    assert!(tail_check < marker_commit);
    assert!(marker_commit < replay);
    assert!(!helper.contains("left_content == right_content"));
    assert!(!helper.contains("transition_guard.prepare("));
    assert!(!helper.contains("transition_guard.commit("));
    assert!(!helper.contains("sq_tx_watch_tx.send("));
}
