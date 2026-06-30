//! Custom prompt, append prompt, and request preview.
//!
//! Demonstrates:
//! - `custom_prompt()` to replace the default Talos identity
//! - `append_prompt()` to add domain-specific instructions
//! - `preview_request()` to inspect what would be sent to the provider
//! - Observing the preview output in the turn completion text
//!
//! Run with: `cargo run --example prompt_and_preview -p talos-runtime`

mod common;

use talos_runtime::{RuntimeBuilder, RuntimeTurnCompletionStatus, collect_until_turn_completed};

#[tokio::main]
async fn main() {
    println!("=== Talos Runtime: Prompt & Preview ===\n");

    // Step 1: Create a mock provider with request debug output.
    let provider = common::mock_provider_with_preview();

    // Step 2: Build the runtime with custom and appended prompts.
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .custom_prompt("You are CodeBot, a Rust-focused programming assistant.")
        .append_prompt("Always respond with code examples when possible.")
        .build()
        .expect("runtime builds");

    println!("Runtime built with custom identity and appended instructions.\n");

    // Step 3: Use preview_request to inspect what would be sent.
    println!("--- Previewing request ---\n");
    runtime
        .preview_request("How do I write a trait in Rust?")
        .await
        .expect("preview request succeeds");

    let status = collect_until_turn_completed(&mut runtime)
        .await
        .expect("turn completes");

    match &status {
        RuntimeTurnCompletionStatus::Success { final_text, .. } => {
            println!("\n--- Preview output ---");
            println!("{final_text}");
        }
        other => {
            eprintln!("Unexpected status: {other:?}");
        }
    }

    // Step 4: Verify the custom prompt is present and default identity is absent.
    println!("\n--- Verification ---");
    if let RuntimeTurnCompletionStatus::Success { final_text, .. } = &status {
        if final_text.contains("CodeBot") {
            println!("  ✓ Custom prompt 'CodeBot' is present in the request");
        } else {
            eprintln!("  ✗ Custom prompt 'CodeBot' is missing!");
        }

        if final_text.contains("code examples") {
            println!("  ✓ Appended prompt 'code examples' is present in the request");
        } else {
            eprintln!("  ✗ Appended prompt is missing!");
        }

        if !final_text.contains("Talos") {
            println!("  ✓ Default Talos identity is NOT present (correctly replaced)");
        } else {
            eprintln!("  ✗ Default Talos identity still present!");
        }
    }

    runtime.shutdown().await.expect("shutdown succeeds");
    println!("\nRuntime shut down cleanly.");
}
