//! Agent tool abstraction layer.
//!
//! This module defines the [`AgentTool`] trait for implementing pluggable tools,
//! a [`ToolRegistry`] for dynamic tool registration and lookup, and associated
//! types for tool execution results and errors.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// Errors that can occur during tool registration, lookup, or execution.
#[derive(Debug, Error)]
pub enum ToolError {
    /// The requested tool is not registered in the registry.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// The input provided to a tool does not match its expected parameters.
    #[error("invalid input for tool: {0}")]
    InvalidInput(String),

    /// An error occurred during tool execution.
    #[error("tool execution error: {0}")]
    ExecutionError(String),
}

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Provenance of a registered tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolProvenance {
    /// A native tool registered within the main process.
    #[default]
    Native,
    /// A tool provided by a remote MCP server.
    McpRemote { server: String },
}

/// The result of executing a tool.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// The output content produced by the tool.
    pub content: String,
    /// Whether the execution resulted in an error.
    pub is_error: bool,
}

impl ToolResult {
    /// Creates a successful tool result with the given content.
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }

    /// Creates an error tool result with the given error message.
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

/// A pluggable agent tool that can be registered and invoked dynamically.
///
/// Implementors must provide a name, description, parameter schema, and
/// execution logic. The trait is object-safe and can be used as
/// `dyn AgentTool` behind an `Arc`.
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Returns the unique name of this tool.
    fn name(&self) -> &str;

    /// Returns a human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// Returns the JSON Schema describing the expected input parameters.
    ///
    /// The default implementation uses `schemars` to generate a schema from
    /// the associated `Parameters` type. Override this method to provide a
    /// custom schema.
    fn parameters(&self) -> Value;

    /// Executes the tool with the given input and returns a result.
    ///
    /// The `input` is expected to conform to the schema returned by
    /// [`parameters`](Self::parameters).
    async fn execute(&self, input: Value) -> ToolResult;

    /// Returns whether this tool is read-only (does not modify external state).
    ///
    /// The default implementation returns `false`. Override for tools that
    /// only read data (e.g., file readers, web fetchers).
    fn is_read_only(&self) -> bool {
        false
    }

    /// Returns the provenance of this tool.
    ///
    /// The default implementation returns [`ToolProvenance::Native`].
    /// Override for tools that live in another process or behind a
    /// network boundary (e.g., MCP remote tools) so consumers can
    /// render an origin marker in the UI.
    fn provenance(&self) -> ToolProvenance {
        ToolProvenance::Native
    }
}

/// A registry for dynamically managing agent tools.
///
/// Tools are registered under their [`AgentTool::name`] and can be retrieved,
/// listed, or have their inputs validated against their parameter schemas.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tool in the registry, replacing any existing tool with the
    /// same name.
    pub fn register(&mut self, tool: Arc<dyn AgentTool>) {
        self.tools.insert(tool.name().to_owned(), tool);
    }

    /// Retrieves a tool by name, or `None` if not registered.
    pub fn get(&self, name: &str) -> Option<&dyn AgentTool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Returns a list of all registered tools.
    pub fn list(&self) -> Vec<&dyn AgentTool> {
        self.tools.values().map(|t| t.as_ref()).collect()
    }

    /// Validates that the given input conforms to the tool's parameter schema.
    ///
    /// Returns `Ok(())` if the tool exists and the input is an object, or
    /// `Err(ToolError)` if the tool is not found or the input is invalid.
    ///
    /// This performs a basic structural check (input must be a JSON object).
    /// Full JSON Schema validation can be added later via the `jsonschema` crate.
    pub fn validate_input(&self, name: &str, input: &Value) -> Result<(), ToolError> {
        let tool = self
            .get(name)
            .ok_or_else(|| ToolError::ToolNotFound(name.to_owned()))?;

        let params = tool.parameters();

        // Basic validation: input must be an object
        if !input.is_object() {
            return Err(ToolError::InvalidInput(format!(
                "expected object for tool '{name}', got {}",
                input_type_name(input)
            )));
        }

        // Check required fields if the schema specifies them
        if let Some(schema_obj) = params.as_object()
            && let Some(Value::Array(required)) = schema_obj.get("required")
            && let Some(input_obj) = input.as_object()
        {
            for req in required {
                if let Some(req_key) = req.as_str()
                    && !input_obj.contains_key(req_key)
                {
                    return Err(ToolError::InvalidInput(format!(
                        "missing required field '{req_key}' for tool '{name}'"
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Returns a human-readable type name for a JSON value.
fn input_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

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
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// Mock tool for testing.
    struct MockTool {
        tool_name: String,
        tool_description: String,
        read_only: bool,
    }

    impl MockTool {
        fn new(name: &str, description: &str) -> Self {
            Self {
                tool_name: name.to_owned(),
                tool_description: description.to_owned(),
                read_only: true,
            }
        }
    }

    #[async_trait]
    impl AgentTool for MockTool {
        fn name(&self) -> &str {
            &self.tool_name
        }

        fn description(&self) -> &str {
            &self.tool_description
        }

        fn parameters(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "A message to echo"
                    }
                },
                "required": ["message"]
            })
        }

        async fn execute(&self, input: Value) -> ToolResult {
            if let Some(msg) = input.get("message").and_then(Value::as_str) {
                ToolResult::success(format!("echo: {msg}"))
            } else {
                ToolResult::error("missing 'message' field".to_owned())
            }
        }

        fn is_read_only(&self) -> bool {
            self.read_only
        }
    }

    /// Mock tool with typed parameters for schema generation testing.
    #[derive(JsonSchema, Deserialize)]
    #[allow(dead_code)]
    struct GreetParams {
        /// The name to greet.
        name: String,
        /// Whether to use formal greeting.
        #[serde(default)]
        formal: bool,
    }

    #[allow(dead_code)]
    struct TypedMockTool;

    #[async_trait]
    impl AgentTool for TypedMockTool {
        fn name(&self) -> &str {
            "greet"
        }

        fn description(&self) -> &str {
            "Greet someone by name"
        }

        fn parameters(&self) -> Value {
            tool_parameters!(GreetParams)
        }

        async fn execute(&self, input: Value) -> ToolResult {
            let name = input.get("name").and_then(Value::as_str).unwrap_or("World");
            ToolResult::success(format!("Hello, {name}!"))
        }
    }

    #[test]
    fn test_register_and_get_tool() {
        let mut registry = ToolRegistry::new();
        let tool = Arc::new(MockTool::new("echo", "Echoes a message"));
        registry.register(tool);

        let retrieved = registry.get("echo");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "echo");
    }

    #[test]
    fn test_tool_not_found() {
        let registry = ToolRegistry::new();
        assert!(registry.get("nonexistent").is_none());

        let result = registry.validate_input("nonexistent", &serde_json::json!({}));
        assert!(matches!(result, Err(ToolError::ToolNotFound(_))));
    }

    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));
        registry.register(Arc::new(MockTool::new("reverse", "Reverses a string")));

        let tools = registry.list();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_validate_input_valid() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

        let input = serde_json::json!({ "message": "hello" });
        assert!(registry.validate_input("echo", &input).is_ok());
    }

    #[test]
    fn test_validate_input_missing_required() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

        let input = serde_json::json!({});
        let result = registry.validate_input("echo", &input);
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing required field 'message'")
        );
    }

    #[test]
    fn test_validate_input_not_object() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

        let input = serde_json::json!("not an object");
        let result = registry.validate_input("echo", &input);
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_tool_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

        let tool = registry.get("echo").unwrap();
        let result = tool
            .execute(serde_json::json!({ "message": "hello" }))
            .await;
        assert!(!result.is_error);
        assert_eq!(result.content, "echo: hello");
    }

    #[tokio::test]
    async fn test_tool_execute_error() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

        let tool = registry.get("echo").unwrap();
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_error);
    }

    #[test]
    fn test_tool_is_read_only() {
        let tool = MockTool::new("echo", "Echoes a message");
        assert!(tool.is_read_only());
    }

    #[test]
    fn test_tool_parameters_macro() {
        let schema = tool_parameters!(GreetParams);
        assert!(schema.is_object());
        let obj = schema.as_object().unwrap();
        assert!(obj.contains_key("properties"));
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Original")));
        registry.register(Arc::new(MockTool::new("echo", "Replacement")));

        let tool = registry.get("echo").unwrap();
        assert_eq!(tool.description(), "Replacement");
    }

    #[test]
    fn test_tool_result_helpers() {
        let success = ToolResult::success("ok");
        assert!(!success.is_error);
        assert_eq!(success.content, "ok");

        let error = ToolResult::error("failed");
        assert!(error.is_error);
        assert_eq!(error.content, "failed");
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProtocol {
    #[default]
    Native,
    TalosStrict,
    Compat,
}

impl ToolProtocol {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "native" => Some(ToolProtocol::Native),
            "talos-strict" | "talos_xml_json_strict" => Some(ToolProtocol::TalosStrict),
            "compat" | "compatibility" => Some(ToolProtocol::Compat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolProtocolConfig {
    pub protocol: ToolProtocol,
    pub strict_prompt: bool,
    pub stream_filter: bool,
    pub schema_validate: bool,
}

impl ToolProtocolConfig {
    pub fn for_protocol(protocol: ToolProtocol) -> Self {
        match protocol {
            ToolProtocol::Native => ToolProtocolConfig {
                protocol,
                strict_prompt: false,
                stream_filter: false,
                schema_validate: false,
            },
            ToolProtocol::TalosStrict => ToolProtocolConfig {
                protocol,
                strict_prompt: true,
                stream_filter: true,
                schema_validate: true,
            },
            ToolProtocol::Compat => ToolProtocolConfig {
                protocol,
                strict_prompt: false,
                stream_filter: true,
                schema_validate: false,
            },
        }
    }
}
