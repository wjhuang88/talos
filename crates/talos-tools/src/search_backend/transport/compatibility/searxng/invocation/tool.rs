//! Dormant explicit SearXNG tool; official registration remains unchanged.

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, PreparedNetworkToolInvocation, ToolFamily, ToolNature, ToolNetworkPreparationError,
    ToolResult,
};

use super::{PreparedSearXngInvocation, SearXngConfiguration, TransportLimits};
use crate::web_search::WebSearchInput;

/// Owns one validated explicit endpoint configuration; never an ambient default.
pub(in crate::search_backend) struct GuardedSearXngSearchTool {
    configuration: SearXngConfiguration,
    limits: TransportLimits,
}

impl std::fmt::Debug for GuardedSearXngSearchTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GuardedSearXngSearchTool(<redacted>)")
    }
}

impl GuardedSearXngSearchTool {
    pub(in crate::search_backend) fn new(
        configuration: SearXngConfiguration,
        limits: TransportLimits,
    ) -> Self {
        Self {
            configuration,
            limits,
        }
    }

    fn prepare(
        &self,
        input: &Value,
    ) -> Result<PreparedSearXngInvocation, ToolNetworkPreparationError> {
        let parsed: WebSearchInput = serde_json::from_value(input.clone())
            .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        PreparedSearXngInvocation::prepare(
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
impl AgentTool for GuardedSearXngSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search through one explicitly configured SearXNG endpoint. \
         Requires approval of the complete prepared network plan."
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
