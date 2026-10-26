//! Agent tool abstraction layer.
//!
//! This module defines the [`AgentTool`] trait for implementing pluggable tools,
//! a [`ToolRegistry`] for dynamic tool registration and lookup, and associated
//! types for tool execution results and errors.

mod agent_tool;
mod authorization;
mod browser_arguments;
mod cleanup;
mod network;
mod prepared;
mod prepared_command;
mod prepared_file_mutation;
mod protocol;
mod registry;
mod result_presentation;
mod todo_presentation;

pub use self::{agent_tool::*, authorization::*, protocol::*, registry::*, result_presentation::*};
pub use browser_arguments::{BrowserRawArguments, BrowserRawArgumentsError};
pub use cleanup::{ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt};
pub use network::*;
pub use prepared::{
    AuthorizedToolInvocation, PREPARED_INVOCATION_TIMEOUT, PreparedExecutionOutput,
    PreparedFailureCode, PreparedInvocationError, PreparedNormalStage,
    PreparedNormalToolInvocation, PreparedPlanIdentity, PreparedStageHistory,
    PreparedToolInvocation,
};
pub use prepared_command::{
    PreparedCommandDescription, PreparedCommandEffect, PreparedCommandEvidence,
    PreparedCommandTargets,
};
pub use prepared_file_mutation::{
    MAX_FILE_MUTATION_REVIEW_BYTES, PreparedFileMutationDescription, PreparedFileMutationEvidence,
};
pub use todo_presentation::{
    TodoDisplaySpan, TodoDisplayStatus, TodoPresentation, VisibleTodoPresentation,
    validate_todo_display_spans,
};

/// Helper macro to generate a JSON Schema value from a type that implements
/// `schemars::JsonSchema`.
#[macro_export]
macro_rules! tool_parameters {
    ($type:ty) => {{
        let schema = schemars::schema_for!($type);
        serde_json::to_value(schema).unwrap_or(serde_json::Value::Object(Default::default()))
    }};
}

#[cfg(test)]
mod tests;
