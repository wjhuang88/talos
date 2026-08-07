use std::fs;
use std::path::Path;

#[test]
fn custody_helpers_stay_private_behind_the_session_actor() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let actor = fs::read_to_string(crate_root.join("src/session.rs"))
        .expect("read AppServerSession actor module");
    let custody = fs::read_to_string(crate_root.join("src/session/custody.rs"))
        .expect("read Session custody helper module");

    assert!(actor.contains("mod custody;"));
    assert!(!actor.contains("pub mod custody;"));
    assert!(!actor.contains("pub(crate) mod custody;"));

    for helper in [
        "reconcile_running_submissions",
        "restore_pending_submissions",
        "release_in_memory_pending_on_shutdown",
        "finish_structured_turn",
        "accept_durable_submission",
        "reconcile_submission",
        "accept_submission",
        "cancel_paused_submission",
        "pause_before_start",
    ] {
        assert!(custody.contains(&format!("fn {helper}(")));
        assert!(!actor.contains(&format!("fn {helper}(")));
    }

    assert!(actor.contains("pub struct AppServerSession"));
    assert!(actor.contains("pub async fn run(&mut self)"));
    assert!(actor.contains("async fn start_submission("));
    assert!(actor.contains("fn commit_turn_record("));
    assert!(!custody.contains("struct AppServerSession"));
    assert!(!custody.contains("async fn run("));
    assert!(!custody.contains("async fn start_submission("));
}
