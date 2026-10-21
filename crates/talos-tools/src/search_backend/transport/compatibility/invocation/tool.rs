//! Private explicit prepared-only tool seam; official registration remains unchanged.

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, PreparedNetworkToolInvocation, ToolFamily, ToolNature, ToolNetworkPreparationError,
    ToolResult,
};

use super::{DirectCompatibilityConfiguration, PreparedCompatibilityInvocation, TransportLimits};
use crate::web_search::WebSearchInput;

/// Immutable direct-only instance with explicit caller-selected limits, never an ambient default.
pub(in crate::search_backend) struct GuardedCompatibilitySearchTool {
    configuration: DirectCompatibilityConfiguration,
    limits: TransportLimits,
}

impl std::fmt::Debug for GuardedCompatibilitySearchTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GuardedCompatibilitySearchTool(<redacted>)")
    }
}

impl GuardedCompatibilitySearchTool {
    pub(in crate::search_backend) fn new(
        limits: TransportLimits,
    ) -> Result<Self, ToolNetworkPreparationError> {
        Ok(Self {
            configuration: DirectCompatibilityConfiguration::new()
                .map_err(|_| ToolNetworkPreparationError::Unavailable)?,
            limits,
        })
    }

    fn prepare(
        &self,
        input: &Value,
    ) -> Result<PreparedCompatibilityInvocation, ToolNetworkPreparationError> {
        let parsed: WebSearchInput = serde_json::from_value(input.clone())
            .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        PreparedCompatibilityInvocation::prepare(
            &self.configuration,
            &parsed.query,
            parsed.max_results.clamp(1, 20),
            parsed.include_snippets,
            self.limits,
        )
        .map_err(|_| ToolNetworkPreparationError::Invalid)
    }
}

#[async_trait]
impl AgentTool for GuardedCompatibilitySearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search through an explicitly configured direct compatibility route. \
         The prepared plan may send the query to DuckDuckGo Lite, Instant Answer, HTML, \
         and Bing in that order. Requires approval of the complete network plan."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(WebSearchInput)
    }

    fn requires_network_invocation(&self) -> bool {
        true
    }

    fn prepare_network_invocation(
        &self,
        input: &Value,
    ) -> Result<Option<Box<dyn PreparedNetworkToolInvocation>>, ToolNetworkPreparationError> {
        Ok(Some(Box::new(self.prepare(input)?)))
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Network
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["query"]
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::error("search_prepared_invocation_required")
    }
}

#[cfg(test)]
mod tests;
