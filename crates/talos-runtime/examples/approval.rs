//! Approval handler and permission rules.
//!
//! Demonstrates:
//! - Implementing `ApprovalHandler` to make runtime approval decisions
//! - Composing permission rules with an approval handler
//! - Auto-approving read tools, denying write tools
//! - Observing how `Ask` decisions flow through the handler
//!
//! Run with: `cargo run --example approval -p talos-runtime`

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::approval::ApprovalChoice;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use talos_permission::{PermissionDecision, PermissionRule};
use talos_runtime::{
    ApprovalHandler, RuntimeBuilder, RuntimeTurnCompletionStatus, collect_until_turn_completed,
};

/// A write tool that would modify a file (requires approval).
struct WriteFileTool;

#[async_trait]
impl AgentTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Writes content to a file"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let path = input.get("path").and_then(Value::as_str).unwrap_or("?");
        ToolResult::success(format!("Written to {path}"))
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Write
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}

/// An approval handler that denies write tools.
struct AutoApprovalHandler;

#[async_trait]
impl ApprovalHandler for AutoApprovalHandler {
    async fn request_approval(
        &self,
        tool_name: &str,
        arguments: &Value,
        _summary_fields: &[String],
    ) -> ApprovalChoice {
        println!("  [Approval] {tool_name}({arguments}) → denied (write tool)");
        ApprovalChoice::Deny
    }
}

#[tokio::main]
async fn main() {
    println!("=== Talos Runtime: Approval Handler ===\n");

    // Step 1: Create a mock provider that tries to call the write tool.
    let provider = Arc::new(
        common::MockProvider::new()
            .with_tool_call(
                "write_file",
                serde_json::json!({
                    "path": "output.txt",
                    "content": "Hello, world!"
                }),
            )
            .with_response("I attempted to write to output.txt."),
    );

    // Step 2: Build the runtime with the write tool and an approval handler.
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(".")
        .tool(Arc::new(WriteFileTool))
        .approval_handler(Arc::new(AutoApprovalHandler))
        .build()
        .expect("runtime builds");

    println!("Runtime built with write_file tool and approval handler.\n");

    // Step 3: Submit a message.
    // The mock provider emits a tool call for write_file.
    // The permission engine returns Ask (write tools require approval).
    // The approval handler denies it.
    runtime
        .submit("Write hello to output.txt")
        .await
        .expect("submit succeeds");

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

    // Step 4: Rebuild with a permission rule that allows writes to "output.txt".
    println!("\n--- Rebuilding with allow rule for output.txt ---\n");

    let provider2 = Arc::new(
        common::MockProvider::new()
            .with_tool_call(
                "write_file",
                serde_json::json!({
                    "path": "output.txt",
                    "content": "Allowed this time!"
                }),
            )
            .with_response("File written successfully."),
    );

    let mut runtime2 = RuntimeBuilder::new()
        .provider(provider2)
        .workspace_root(".")
        .tool(Arc::new(WriteFileTool))
        .permission_rule(PermissionRule::new_nature(
            ToolNature::Write,
            Some("output.txt".to_string()),
            Some(talos_permission::ResourceKind::Path),
            PermissionDecision::Allow,
        ))
        .approval_handler(Arc::new(AutoApprovalHandler))
        .build()
        .expect("runtime builds");

    runtime2
        .submit("Write to output.txt again")
        .await
        .expect("submit succeeds");

    let status2 = collect_until_turn_completed(&mut runtime2)
        .await
        .expect("turn completes");

    match &status2 {
        RuntimeTurnCompletionStatus::Success { final_text, .. } => {
            println!("\nFinal response: {final_text}");
        }
        other => {
            eprintln!("Unexpected status: {other:?}");
        }
    }

    runtime2.shutdown().await.expect("shutdown succeeds");
    println!("\nRuntime shut down cleanly.");
}
