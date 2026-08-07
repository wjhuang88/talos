use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use thiserror::Error;

use super::AgentTool;

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

/// Stable, display-safe identity for the crate, product profile, or plugin
/// that contributed a tool.
///
/// Sources are diagnostics only: they do not grant permissions and must not
/// contain credentials, workspace paths, tool arguments, or projected input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToolContributionSource(String);

impl ToolContributionSource {
    /// Creates a stable contribution source identity.
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    /// Returns the source identity as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolContributionSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One explicitly sourced tool instance ready for product composition.
#[derive(Clone)]
pub struct ToolContribution {
    source: ToolContributionSource,
    tool: Arc<dyn AgentTool>,
}

impl ToolContribution {
    /// Creates a sourced tool contribution.
    pub fn new(source: ToolContributionSource, tool: Arc<dyn AgentTool>) -> Self {
        Self { source, tool }
    }

    /// Returns the stable source identity.
    #[must_use]
    pub fn source(&self) -> &ToolContributionSource {
        &self.source
    }

    /// Returns the contributed tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.tool.name()
    }

    /// Returns the contributed tool instance.
    #[must_use]
    pub fn tool(&self) -> &Arc<dyn AgentTool> {
        &self.tool
    }

    /// Applies an outer composition wrapper while preserving source identity.
    #[must_use]
    pub fn map_tool(mut self, wrap: impl FnOnce(Arc<dyn AgentTool>) -> Arc<dyn AgentTool>) -> Self {
        self.tool = wrap(self.tool);
        self
    }
}

/// Deterministic duplicate-name diagnostic for checked tool composition.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "duplicate tool registration '{tool_name}': existing source '{existing_source}', incoming source '{incoming_source}'"
)]
pub struct ToolRegistrationError {
    /// Duplicate tool name.
    pub tool_name: String,
    /// Source that already owns the registered name.
    pub existing_source: ToolContributionSource,
    /// Source that attempted the duplicate registration.
    pub incoming_source: ToolContributionSource,
}

const LEGACY_TOOL_REGISTRATION_SOURCE: &str = "legacy:unchecked";

/// A registry for dynamically managing agent tools.
///
/// Tools are registered under their [`AgentTool::name`] and can be retrieved,
/// listed, or have their inputs validated against their parameter schemas.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
    contribution_sources: HashMap<String, ToolContributionSource>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tool in the registry, replacing any existing tool with the
    /// same name.
    ///
    /// This historical unchecked API remains temporarily source-compatible
    /// during I158 migration. New product composition should use
    /// [`register_contribution`](Self::register_contribution).
    pub fn register(&mut self, tool: Arc<dyn AgentTool>) {
        let name = tool.name().to_owned();
        self.contribution_sources.remove(&name);
        self.tools.insert(name, tool);
    }

    /// Registers one explicitly sourced contribution without replacing an
    /// existing tool.
    ///
    /// A duplicate returns both source identities and leaves the current
    /// registry entry unchanged.
    pub fn register_contribution(
        &mut self,
        contribution: ToolContribution,
    ) -> Result<(), ToolRegistrationError> {
        let ToolContribution { source, tool } = contribution;
        let tool_name = tool.name().to_owned();

        if let Some(existing_source) = self.registered_source(&tool_name) {
            return Err(ToolRegistrationError {
                tool_name,
                existing_source,
                incoming_source: source,
            });
        }

        self.contribution_sources.insert(tool_name.clone(), source);
        self.tools.insert(tool_name, tool);
        Ok(())
    }

    /// Registers a contribution batch transactionally.
    ///
    /// The complete batch is checked against the current registry and against
    /// earlier entries in the same iteration order before any tool is inserted.
    /// On the first duplicate, the registry remains unchanged.
    pub fn register_contributions(
        &mut self,
        contributions: impl IntoIterator<Item = ToolContribution>,
    ) -> Result<(), ToolRegistrationError> {
        let contributions = contributions.into_iter().collect::<Vec<_>>();
        let mut pending_sources = HashMap::<String, ToolContributionSource>::new();

        for contribution in &contributions {
            let tool_name = contribution.name().to_owned();
            if let Some(existing_source) = self.registered_source(&tool_name) {
                return Err(ToolRegistrationError {
                    tool_name,
                    existing_source,
                    incoming_source: contribution.source().clone(),
                });
            }
            if let Some(existing_source) = pending_sources.get(&tool_name) {
                return Err(ToolRegistrationError {
                    tool_name,
                    existing_source: existing_source.clone(),
                    incoming_source: contribution.source().clone(),
                });
            }
            pending_sources.insert(tool_name, contribution.source().clone());
        }

        for ToolContribution { source, tool } in contributions {
            let tool_name = tool.name().to_owned();
            self.contribution_sources.insert(tool_name.clone(), source);
            self.tools.insert(tool_name, tool);
        }
        Ok(())
    }

    fn registered_source(&self, tool_name: &str) -> Option<ToolContributionSource> {
        self.tools.contains_key(tool_name).then(|| {
            self.contribution_sources
                .get(tool_name)
                .cloned()
                .unwrap_or_else(|| ToolContributionSource::new(LEGACY_TOOL_REGISTRATION_SOURCE))
        })
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
