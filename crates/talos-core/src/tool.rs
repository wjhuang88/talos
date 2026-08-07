//! Agent tool abstraction layer.
//!
//! This module defines the [`AgentTool`] trait for implementing pluggable tools,
//! a [`ToolRegistry`] for dynamic tool registration and lookup, and associated
//! types for tool execution results and errors.

mod agent_tool;
mod authorization;
mod protocol;
mod registry;
mod result_presentation;

pub use self::{agent_tool::*, authorization::*, protocol::*, registry::*, result_presentation::*};

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
