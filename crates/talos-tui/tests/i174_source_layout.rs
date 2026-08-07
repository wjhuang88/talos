use std::fs;
use std::path::Path;

#[test]
fn app_responsibilities_stay_private_behind_the_tui_facade() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = fs::read_to_string(crate_root.join("src/app.rs")).expect("read app facade");
    let input = fs::read_to_string(crate_root.join("src/app/input.rs")).expect("read app input");
    let output = fs::read_to_string(crate_root.join("src/app/output.rs")).expect("read app output");
    let frame = fs::read_to_string(crate_root.join("src/app/frame.rs")).expect("read app frame");

    for module in ["frame", "input", "output"] {
        assert!(facade.contains(&format!("mod {module};")));
        assert!(!facade.contains(&format!("pub mod {module};")));
        assert!(!facade.contains(&format!("pub(crate) mod {module};")));
    }

    assert!(facade.contains("pub struct Tui"));
    assert!(facade.contains("pub async fn run"));
    assert!(facade.contains("tokio::select!"));
    assert!(input.contains("fn handle_input_event"));
    assert!(output.contains("fn handle_ui_output"));
    assert!(output.contains("fn next_stream_chunk"));
    assert!(frame.contains("fn draw_frame"));
    assert!(frame.contains("fn anchor_frame_history_start"));

    for responsibility in [&input, &output, &frame] {
        assert!(!responsibility.contains("pub async fn run"));
        assert!(!responsibility.contains("tokio::select!"));
    }
}

#[test]
fn existing_tui_root_path_still_compiles() {
    let _ = std::any::TypeId::of::<talos_tui::Tui>();
}
