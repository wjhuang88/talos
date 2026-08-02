use std::env;
use std::fs;
use std::path::PathBuf;

fn replace_once(source: &mut String, from: &str, to: &str, label: &str) {
    assert!(
        source.contains(from),
        "I169 bridge probe could not find {label}"
    );
    *source = source.replacen(from, to, 1);
}

fn main() {
    println!("cargo:rerun-if-changed=src/tui_bridge_impl.rs");

    let mut source = fs::read_to_string("src/tui_bridge_impl.rs")
        .expect("read I169 bridge implementation source");

    replace_once(
        &mut source,
        "//! Bridge between the conversation engine and the TUI.\n//!\n//! Contains the conversation loop that mediates between agent events,\n//! user input, and UI output channels.\n",
        "// Bridge between the conversation engine and the TUI.\n//\n// Contains the conversation loop that mediates between agent events,\n// user input, and UI output channels.\n",
        "included-module documentation header",
    );
    replace_once(
        &mut source,
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);\n    let state_was_legacy_cancelling =\n        matches!(&state, BridgeTurnState::LegacyCancelling { .. });\n    match state {\n        BridgeTurnState::Idle\n            if sequence == 0 && matches!(payload, TurnEventPayload::Started) =>",
        "legacy cancellation state snapshot",
    );
    replace_once(
        &mut source,
        "            let cancelling = matches!(state, BridgeTurnState::LegacyCancelling { .. });",
        "            let cancelling = state_was_legacy_cancelling;",
        "legacy cancellation state use",
    );
    replace_once(
        &mut source,
        "                    cancel_requested,\n                    last_reconcile,\n                    reconcile_attempts,",
        "                    cancel_requested: _,\n                    last_reconcile: _,\n                    reconcile_attempts: _,",
        "rejected submission unused fields",
    );

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("tui_bridge_impl.rs");
    fs::write(output, source).expect("write generated I169 bridge implementation");
}
