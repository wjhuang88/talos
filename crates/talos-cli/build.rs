use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn replace_once(source: &mut String, from: &str, to: &str, label: &str) {
    assert!(
        source.contains(from),
        "I169 bridge probe could not find {label}"
    );
    *source = source.replacen(from, to, 1);
}

fn normalized_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .replace("\r\n", "\n")
}

fn main() {
    println!("cargo:rerun-if-changed=src/tui_bridge_impl.rs");
    println!("cargo:rerun-if-changed=src/tests_impl.rs");

    let mut bridge_source = normalized_source(Path::new("src/tui_bridge_impl.rs"));

    replace_once(
        &mut bridge_source,
        "//! Bridge between the conversation engine and the TUI.\n//!\n//! Contains the conversation loop that mediates between agent events,\n//! user input, and UI output channels.\n",
        "// Bridge between the conversation engine and the TUI.\n//\n// Contains the conversation loop that mediates between agent events,\n// user input, and UI output channels.\n",
        "included-module documentation header",
    );
    replace_once(
        &mut bridge_source,
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    let state_was_legacy_cancelling =\n        matches!(&state, BridgeTurnState::LegacyCancelling { .. });\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "legacy cancellation state snapshot",
    );
    replace_once(
        &mut bridge_source,
        "            let cancelling = matches!(state, BridgeTurnState::LegacyCancelling { .. });",
        "            let cancelling = state_was_legacy_cancelling;",
        "legacy cancellation state use",
    );
    replace_once(
        &mut bridge_source,
        "                    cancel_requested,\n                    last_reconcile,\n                    reconcile_attempts,",
        "                    cancel_requested: _,\n                    last_reconcile: _,\n                    reconcile_attempts: _,",
        "rejected submission unused fields",
    );

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(output_dir.join("tui_bridge_impl.rs"), bridge_source)
        .expect("write generated I169 bridge implementation");

    let tests_source = normalized_source(Path::new("src/tests_impl.rs"));
    fs::write(output_dir.join("tests_impl.rs"), tests_source)
        .expect("write generated I169 CLI tests");
}
