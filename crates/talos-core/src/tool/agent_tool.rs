use std::collections::HashSet;

use async_trait::async_trait;
use serde_json::Value;

use super::{
    ToolBackend, ToolExecutionAuthorization, ToolExecutionOutput, ToolFamily, ToolNature,
    ToolPermissionFacet, ToolProvenance, ToolResult, ToolResultProjection,
};

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

    /// Executes with concrete authorizations produced by a permission-aware
    /// composition root.
    ///
    /// Most tools do not need a path capability and retain their existing
    /// behavior. File tools override this method to validate external paths.
    async fn execute_authorized(
        &self,
        input: Value,
        _authorizations: &[ToolExecutionAuthorization],
    ) -> ToolResult {
        self.execute(input).await
    }

    /// Executes with concrete authorizations and returns an output that
    /// may carry a provider-neutral continuation artifact (ADR-051).
    ///
    /// The default implementation delegates to [`execute_authorized`]
    /// and returns the result with no continuation parts. Tools that
    /// produce one-shot provider artifacts (e.g. `read_image`) override
    /// this method.
    ///
    /// Permission wrappers MUST forward this method after obtaining the
    /// same authorizations they would use for [`execute_authorized`].
    async fn execute_authorized_with_output(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> ToolExecutionOutput {
        ToolExecutionOutput::from_result(self.execute_authorized(input, authorizations).await)
    }

    /// Executes the tool and returns an output that may carry a
    /// provider-neutral continuation artifact (ADR-051).
    ///
    /// The default implementation delegates to [`execute`] and returns
    /// no continuation parts. Permission wrappers override this to
    /// perform the same approval flow as [`execute`] and return the
    /// full [`ToolExecutionOutput`] including any continuation parts
    /// produced by the inner tool's [`execute_authorized_with_output`].
    async fn execute_with_output(&self, input: Value) -> ToolExecutionOutput {
        ToolExecutionOutput::from_result(self.execute(input).await)
    }

    /// Returns the observer-safe form of a tool input.
    ///
    /// Execution and permission evaluation always receive the original input.
    /// This projection is used only for UI events, approval presentation, and
    /// durable replay. The default preserves the complete input.
    fn project_input(&self, input: &Value) -> Value {
        input.clone()
    }

    /// Splits one execution result into model, display, and persistence views.
    ///
    /// The default keeps existing tools fully backward compatible.
    fn project_result(&self, result: &ToolResult) -> ToolResultProjection {
        ToolResultProjection::shared(result.content.clone())
    }

    /// Returns whether this tool is read-only (does not modify external state).
    ///
    /// The default implementation returns `false`. Override for tools that
    /// only read data (e.g., file readers, web fetchers).
    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        if self.is_read_only() {
            ToolNature::Read
        } else {
            ToolNature::Write
        }
    }

    /// Returns the stable presentation family for this tool.
    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    /// Returns whether this tool belongs to the always-on presentation set.
    fn is_always_on(&self) -> bool {
        false
    }

    /// Returns conditional backends supported by this tool.
    ///
    /// Tools with no conditional execution paths should rely on the default
    /// empty list.
    fn conditional_backends(&self) -> Vec<ToolBackend> {
        Vec::new()
    }

    /// Returns the backend selected by this concrete input, if any.
    ///
    /// The agent runtime checks this value against the presentation policy
    /// before permission evaluation or execution. Returning `None` means the
    /// tool is using its base path.
    fn backend_for_input(&self, _input: &Value) -> Option<String> {
        None
    }

    /// Returns a model-facing description for the disclosed backend set.
    fn description_for_backends(&self, _backends: &HashSet<String>) -> String {
        self.description().to_string()
    }

    /// Returns an input schema for the disclosed backend set.
    fn parameters_for_backends(&self, _backends: &HashSet<String>) -> Value {
        self.parameters()
    }

    /// Returns the permission facets touched by this concrete invocation.
    ///
    /// Tools that only touch one risk surface can rely on the default
    /// single-facet profile derived from [`nature`](Self::nature). Hybrid tools
    /// should override this to expose every relevant risk surface.
    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![ToolPermissionFacet::new(self.nature())]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &[]
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
