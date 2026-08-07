use std::fs;
use std::path::Path;

#[test]
fn legacy_projection_stays_private_behind_the_tui_bridge_facade() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade =
        fs::read_to_string(crate_root.join("src/tui_bridge.rs")).expect("read tui bridge facade");
    let projection = fs::read_to_string(crate_root.join("src/tui_bridge/legacy_projection.rs"))
        .expect("read legacy projection module");

    assert!(facade.contains("mod legacy_projection;"));
    assert!(facade.contains("legacy_projection::handle_legacy_turn_event("));
    assert!(!facade.contains("fn handle_structured_legacy_projection("));

    assert!(projection.contains("pub(super) fn handle_legacy_turn_event("));
    assert!(projection.contains("fn handle_structured_legacy_projection("));
    assert!(!projection.contains("run_conversation_loop"));
    assert!(!projection.contains("SessionLifecycleRequest"));
}
