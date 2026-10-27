#![cfg(feature = "shared-composition")]

use super::super::*;
use talos_core::tool::{ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt};

struct Owner;

#[async_trait]
impl ToolCleanupReceipt for Owner {
    async fn wait(&self, _: std::time::Duration) -> Result<(), ToolCleanupError> {
        Ok(())
    }
}

impl ToolCleanupOwner for Owner {
    fn seal(&self) {}
}

struct CallerRead(Arc<Owner>);

#[async_trait]
impl AgentTool for CallerRead {
    fn name(&self) -> &str {
        "read"
    }
    fn description(&self) -> &str {
        "caller read"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type":"object"})
    }
    fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupOwner>> {
        Some(self.0.clone())
    }
    async fn execute(&self, _: Value) -> ToolResult {
        ToolResult::success("caller")
    }
}

#[test]
fn shared_registration_preserves_trust_only_for_the_winning_instance() {
    for caller_last in [false, true] {
        let owner = Arc::new(Owner);
        let caller = Arc::new(CallerRead(owner.clone()));
        let builder = if caller_last {
            RuntimeBuilder::new().shared_tools().tool(caller)
        } else {
            RuntimeBuilder::new().tool(caller).shared_tools()
        };
        let (registry, owners) = register_runtime_tools(builder.tools).expect("registration");
        assert_eq!(registry.is_builtin_instance("read"), !caller_last);
        assert_eq!(
            registry.get("read").expect("read").description() == "caller read",
            caller_last
        );
        let owner: Arc<dyn ToolCleanupOwner> = owner;
        assert!(owners.iter().any(|retained| Arc::ptr_eq(retained, &owner)));
    }
}

#[test]
fn repeated_shared_registration_keeps_builtin_winners() {
    let builder = RuntimeBuilder::new().shared_tools().shared_tools();
    let (registry, _) = register_runtime_tools(builder.tools).expect("registration");
    assert!(registry.is_builtin_instance("read"));
    assert!(registry.is_builtin_instance("exec"));
}

#[test]
fn session_todo_factory_preserves_identity_without_promoting_caller_instances() {
    let directory = tempfile::tempdir().expect("sessions");
    let session_id = uuid::Uuid::new_v4();
    let caller =
        talos_session::todo_tool_contributions_for_sessions_dir(directory.path(), session_id)
            .remove(0)
            .tool()
            .clone();
    let name = caller.name().to_owned();
    for caller_last in [false, true] {
        let builder = if caller_last {
            RuntimeBuilder::new()
                .session_todo_tools(directory.path(), session_id)
                .tool(caller.clone())
        } else {
            RuntimeBuilder::new()
                .tool(caller.clone())
                .session_todo_tools(directory.path(), session_id)
        };
        let (registry, _) = register_runtime_tools(builder.tools).expect("registration");
        assert_eq!(registry.is_builtin_instance(&name), !caller_last);
        assert!(registry.is_builtin_instance("todo_query"));
        assert!(registry.is_builtin_instance("todo_update"));
    }
}

#[test]
fn overridden_caller_owners_are_retained_and_deduplicated() {
    let first = Arc::new(Owner);
    let second = Arc::new(Owner);
    let builder = RuntimeBuilder::new()
        .tool(Arc::new(CallerRead(first.clone())))
        .tool(Arc::new(CallerRead(second.clone())))
        .tool(Arc::new(CallerRead(first.clone())));
    let (_, owners) = register_runtime_tools(builder.tools).expect("registration");
    assert_eq!(owners.len(), 2);
    let first: Arc<dyn ToolCleanupOwner> = first;
    let second: Arc<dyn ToolCleanupOwner> = second;
    assert!(owners.iter().any(|owner| Arc::ptr_eq(owner, &first)));
    assert!(owners.iter().any(|owner| Arc::ptr_eq(owner, &second)));
}
