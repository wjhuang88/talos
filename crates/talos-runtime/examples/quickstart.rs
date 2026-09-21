//! Quickstart: minimal Talos runtime embedding.
//!
//! Demonstrates:
//! - Creating a `RuntimeBuilder` and injecting a mock provider
//! - Submitting a user message
//! - Streaming events and printing text deltas
//! - Graceful shutdown
//!
//! Run with: `cargo run --locked --example quickstart -p talos-runtime`

use async_trait::async_trait;
use std::sync::Arc;
use talos_runtime::{
    AgentEvent, LanguageModel, Message, ProviderResult, Receiver, RuntimeBuilder, SessionEvent,
    StopReason, TurnEventPayload, Usage,
};

struct LocalProvider;

#[async_trait]
impl LanguageModel for LocalProvider {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        let (tx, rx) = tokio::sync::mpsc::channel(3);
        tokio::spawn(async move {
            for event in [
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Hello from a local custom provider!".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                },
            ] {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
    }
}

#[tokio::main]
async fn main() -> talos_runtime::RuntimeResult<()> {
    println!("=== Talos Runtime Quickstart ===\n");

    // Step 1: Create a mock provider that streams a response.
    let provider = Arc::new(LocalProvider);

    // Step 2: Build the runtime with the provider.
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .build()?;

    println!("Runtime built successfully.\n");

    // Step 3: Submit a user message to start a turn.
    runtime.submit("Say hello!").await?;

    println!("Message submitted. Streaming events:\n");

    // Step 4: Collect events until the turn completes.
    while let Some(event) = runtime.next_event().await {
        match &event {
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Progress { event },
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
                payload: TurnEventPayload::Started,
                ..
            } => {
                println!("  [turn] {turn_id} started");
            }
            SessionEvent::TurnEvent {
                turn_id,
                payload: TurnEventPayload::Completed { status },
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
    runtime.shutdown().await?;
    println!("Runtime shut down cleanly.");
    Ok(())
}
