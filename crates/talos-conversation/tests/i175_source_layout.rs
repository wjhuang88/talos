use std::fs;
use std::path::Path;

#[test]
fn engine_responsibilities_stay_private_behind_the_facade() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = fs::read_to_string(crate_root.join("src/engine.rs")).expect("read engine facade");
    let commands = fs::read_to_string(crate_root.join("src/engine/commands.rs"))
        .expect("read engine commands");
    let projection = fs::read_to_string(crate_root.join("src/engine/projection.rs"))
        .expect("read engine projection");

    for module in ["commands", "projection"] {
        assert!(facade.contains(&format!("mod {module};")));
        assert!(!facade.contains(&format!("pub mod {module};")));
        assert!(!facade.contains(&format!("pub(crate) mod {module};")));
    }

    assert!(facade.contains("pub struct ConversationEngine"));
    assert!(facade.contains("pub fn handle_agent_event"));
    assert!(facade.contains("pub fn enqueue_structured_steering"));
    assert!(facade.contains("pub fn commit_prepared_steering"));
    assert!(commands.contains("pub fn handle_slash_command"));
    assert!(commands.contains("fn handle_todo_command"));
    assert!(projection.contains("pub fn transcript_plain_text"));
    assert!(projection.contains("pub fn build_extension_snapshot"));

    for responsibility in [&commands, &projection] {
        assert!(!responsibility.contains("pub fn handle_agent_event"));
        assert!(!responsibility.contains("pub fn enqueue_structured_steering"));
        assert!(!responsibility.contains("pub fn commit_prepared_steering"));
    }
}

#[test]
fn conversation_public_paths_still_compile() {
    let _ = std::any::TypeId::of::<talos_conversation::ConversationEngine>();

    let _ = talos_conversation::build_extension_snapshot(&[], &[], &[]);
    let _ = talos_conversation::build_extension_snapshot_with_plugins(&[], &[], &[], &[]);
}
