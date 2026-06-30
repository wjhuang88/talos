//! Shared helpers for talos-runtime examples.

pub use talos_provider::mock::MockProvider;

use talos_core::message::AgentEvent;
use talos_core::provider::LanguageModel;
use talos_core::session::SessionEvent;
use talos_runtime::{RuntimeBuilder, RuntimeHandle, RuntimeTurnCompletionStatus};

pub fn mock_provider(response: &str) -> std::sync::Arc<dyn LanguageModel> {
    std::sync::Arc::new(MockProvider::new().with_response(response))
}

pub fn mock_provider_with_preview() -> std::sync::Arc<dyn LanguageModel> {
    std::sync::Arc::new(MockProvider::new().with_request_debug_builder(|messages| {
        serde_json::to_string_pretty(messages).expect("messages should serialize")
    }))
}

pub async fn build_minimal_runtime(provider: std::sync::Arc<dyn LanguageModel>) -> RuntimeHandle {
    RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .build()
        .expect("runtime builds with a provider")
}

pub async fn run_turn(runtime: &mut RuntimeHandle, message: &str) -> RuntimeTurnCompletionStatus {
    println!("--- Submitting: {message} ---");
    runtime.submit(message).await.expect("submit succeeds");

    let mut final_status = RuntimeTurnCompletionStatus::Cancelled;

    while let Some(event) = runtime.next_event().await {
        print_event(&event);
        if let SessionEvent::TurnCompleted { status, .. } = event {
            final_status = status;
            break;
        }
    }

    final_status
}

pub fn print_event(event: &SessionEvent) {
    match event {
        SessionEvent::AgentEvent { event } => match event {
            AgentEvent::TurnStart => println!("  [TurnStart]"),
            AgentEvent::TextDelta { delta } => println!("  [TextDelta] {delta}"),
            AgentEvent::ToolCallStarted { name } => println!("  [ToolCallStarted] {name}"),
            AgentEvent::ToolCall { call, .. } => {
                println!("  [ToolCall] {}({})", call.name, call.input)
            }
            AgentEvent::ToolResult { result } => {
                let tag = if result.is_error { "ERROR" } else { "OK" };
                println!("  [ToolResult] [{tag}] {}", result.content);
            }
            AgentEvent::TurnEnd { stop_reason, usage } => {
                println!(
                    "  [TurnEnd] reason={:?} in={} out={}",
                    stop_reason, usage.input_tokens, usage.output_tokens
                );
            }
            AgentEvent::Error { message } => println!("  [Error] {message}"),
            _ => println!("  [AgentEvent] (other)"),
        },
        SessionEvent::TurnStarted { turn_id } => println!("  [TurnStarted] id={turn_id}"),
        SessionEvent::TurnCompleted { turn_id, status } => {
            println!("  [TurnCompleted] id={turn_id} status={status:?}")
        }
        SessionEvent::ApprovalRequired {
            tool_name,
            arguments,
            call_id,
        } => println!("  [ApprovalRequired] {tool_name}({arguments}) call={call_id}"),
        SessionEvent::Error { message } => println!("  [RuntimeError] {message}"),
        _ => println!("  [SessionEvent] (other)"),
    }
}

pub fn final_text(status: &RuntimeTurnCompletionStatus) -> Option<&str> {
    match status {
        RuntimeTurnCompletionStatus::Success { final_text, .. } => Some(final_text),
        _ => None,
    }
}
