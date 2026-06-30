//! Custom tool injection.
//!
//! Demonstrates:
//! - Defining a read-only custom tool implementing `AgentTool`
//! - Registering the tool with `RuntimeBuilder::tool()`
//! - Configuring a mock provider to emit a tool call
//! - Observing the tool execution in the event stream
//!
//! Run with: `cargo run --example custom_tool -p talos-runtime`

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use talos_runtime::{RuntimeBuilder, RuntimeTurnCompletionStatus, collect_until_turn_completed};

/// A simple read-only tool that returns a greeting for a given name.
struct GreetTool;

#[async_trait]
impl AgentTool for GreetTool {
    fn name(&self) -> &str {
        "greet"
    }

    fn description(&self) -> &str {
        "Returns a greeting for the given name"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The name to greet" }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let name = input.get("name").and_then(Value::as_str).unwrap_or("world");
        ToolResult::success(format!("Hello, {name}!"))
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["name"]
    }
}

#[tokio::main]
async fn main() {
    println!("=== Talos Runtime: Custom Tool ===\n");

    // Step 1: Create a mock provider that emits a tool call for "greet".
    let provider = Arc::new(
        common::MockProvider::new()
            .with_tool_call("greet", serde_json::json!({"name": "Developer"}))
            .with_response("I greeted the user using the greet tool."),
    );

    // Step 2: Build the runtime with the custom tool registered.
    // Read-only tools are allowed by default (no approval needed).
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .tool(Arc::new(GreetTool))
        .build()
        .expect("runtime builds");

    println!("Runtime built with custom 'greet' tool.\n");

    // Step 3: Submit a message that triggers the tool call.
    runtime
        .submit("Greet the Developer")
        .await
        .expect("submit succeeds");

    // Step 4: Collect events and observe the tool call lifecycle.
    let status = collect_until_turn_completed(&mut runtime)
        .await
        .expect("turn completes");

    match &status {
        RuntimeTurnCompletionStatus::Success { final_text, .. } => {
            println!("\nFinal response: {final_text}");
        }
        other => {
            eprintln!("Unexpected status: {other:?}");
        }
    }

    // Step 5: Shutdown.
    runtime.shutdown().await.expect("shutdown succeeds");
    println!("Runtime shut down cleanly.");
}
