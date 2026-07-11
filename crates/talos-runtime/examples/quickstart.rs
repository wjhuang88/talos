//! Quickstart: minimal Talos runtime embedding.
//!
//! Demonstrates:
//! - Creating a `RuntimeBuilder` and injecting a mock provider
//! - Submitting a user message
//! - Streaming events and printing text deltas
//! - Graceful shutdown
//!
//! Run with: `cargo run --example quickstart -p talos-runtime`

mod common;

use talos_core::message::AgentEvent;
use talos_core::session::SessionEvent;
use talos_runtime::RuntimeBuilder;

#[tokio::main]
async fn main() {
    println!("=== Talos Runtime Quickstart ===\n");

    // Step 1: Create a mock provider that streams a response.
    let provider =
        common::mock_provider("Hello from the Talos runtime! I am a mock LLM running locally.");

    // Step 2: Build the runtime with the provider.
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .build()
        .expect("runtime builds with a provider");

    println!("Runtime built successfully.\n");

    // Step 3: Submit a user message to start a turn.
    runtime.submit("Say hello!").await.expect("submit succeeds");

    println!("Message submitted. Streaming events:\n");

    // Step 4: Collect events until the turn completes.
    while let Some(event) = runtime.next_event().await {
        match &event {
            SessionEvent::TurnEvent {
                payload: talos_core::session::TurnEventPayload::Progress { event },
                ..
            } => match event {
                AgentEvent::TurnStart => println!("  ▶ Turn started"),
                AgentEvent::TextDelta { delta } => print!("  {delta}"),
                AgentEvent::TurnEnd { stop_reason, .. } => {
                    println!("\n  ■ Turn ended: {stop_reason:?}");
                }
                _ => {}
            },
            SessionEvent::TurnEvent {
                turn_id,
                payload: talos_core::session::TurnEventPayload::Started,
                ..
            } => {
                println!("  [turn] {turn_id} started");
            }
            SessionEvent::TurnEvent {
                turn_id,
                payload: talos_core::session::TurnEventPayload::Completed { status },
                ..
            } => {
                println!("  [turn] {turn_id} completed: {status:?}");
                break;
            }
            SessionEvent::Error { message } => {
                eprintln!("  [error] {message}");
                break;
            }
            _ => {}
        }
    }

    // Step 5: Graceful shutdown.
    println!("\nShutting down runtime...");
    runtime.shutdown().await.expect("shutdown succeeds");
    println!("Runtime shut down cleanly.");
}
