//! Dormant prepared-only AgentTool seam for the Talos-owned SearchRouter.
//! Official WebSearch registration and production selection remain unchanged.

#[cfg(feature = "search-integration-test")]
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, PreparedNetworkToolInvocation, ToolFamily, ToolNature, ToolNetworkPreparationError,
    ToolResult,
};

use super::composition::SearchCompositionConfiguration;
#[cfg(feature = "search-integration-test")]
use super::integration_test::SearchRouterIntegrationProbe;
use super::prepared_invocation::PreparedSearchRouterInvocation;
use super::router::SearchRouterPolicy;
use super::transport::TransportLimits;
use crate::web_search::WebSearchInput;

/// Owns one explicitly selected routed composition and its bounded execution policies.
///
/// The constructor accepts an already-built composition so provider discovery/selection stays
/// outside this seam. No environment variables, credentials, or production defaults are read here.
pub(in crate::search_backend) struct GuardedSearchRouterTool {
    configuration: SearchCompositionConfiguration,
    limits: TransportLimits,
    policy: SearchRouterPolicy,
    preparation_available: bool,
    #[cfg(feature = "search-integration-test")]
    integration_probe: Option<Arc<SearchRouterIntegrationProbe>>,
}

impl std::fmt::Debug for GuardedSearchRouterTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GuardedSearchRouterTool(<redacted>)")
    }
}

impl GuardedSearchRouterTool {
    pub(in crate::search_backend) fn new(
        configuration: SearchCompositionConfiguration,
        limits: TransportLimits,
        policy: SearchRouterPolicy,
    ) -> Self {
        Self {
            configuration,
            limits,
            policy,
            preparation_available: true,
            #[cfg(feature = "search-integration-test")]
            integration_probe: None,
        }
    }

    /// Marks a locally observed production configuration as unsupported before any invocation can
    /// be prepared. The reason/value is deliberately not retained in this tool or exposed to the
    /// model; callers receive only the closed preparation error.
    pub(in crate::search_backend) fn with_preparation_unavailable(mut self) -> Self {
        self.preparation_available = false;
        self
    }

    #[cfg(feature = "search-integration-test")]
    pub(in crate::search_backend) fn with_integration_probe(
        mut self,
        probe: Arc<SearchRouterIntegrationProbe>,
    ) -> Self {
        self.integration_probe = Some(probe);
        self
    }

    fn prepare(
        &self,
        input: &Value,
    ) -> Result<PreparedSearchRouterInvocation, ToolNetworkPreparationError> {
        if !self.preparation_available {
            return Err(ToolNetworkPreparationError::Unavailable);
        }
        let parsed: WebSearchInput = serde_json::from_value(input.clone())
            .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        let prepared = PreparedSearchRouterInvocation::prepare(
            &self.configuration,
            &parsed.query,
            parsed.max_results.clamp(1, 20),
            parsed.include_snippets,
            self.limits,
            self.policy,
        )
        .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        #[cfg(feature = "search-integration-test")]
        let prepared = match &self.integration_probe {
            Some(probe) => prepared.with_integration_probe(Arc::clone(probe)),
            None => prepared,
        };
        Ok(prepared)
    }
}

#[async_trait]
impl AgentTool for GuardedSearchRouterTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search through an explicitly configured Talos-owned routed candidate set. \
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
mod tests {
    use std::time::Duration;

    use talos_core::tool::{ToolAuthorizationScope, ToolExecutionAuthorization};

    use super::*;

    fn tool() -> GuardedSearchRouterTool {
        GuardedSearchRouterTool::new(
            SearchCompositionConfiguration::new(None).expect("configuration"),
            TransportLimits::new(Duration::from_secs(5), 5, 1_048_576).expect("limits"),
            SearchRouterPolicy::new(Duration::from_millis(50), 1, 20, 8192).expect("policy"),
        )
    }

    fn input(query: &str) -> Value {
        serde_json::json!({
            "query": query,
            "max_results": 2,
            "include_snippets": true
        })
    }

    #[test]
    fn seam_preserves_web_search_contract_without_registration_or_ambient_selection() {
        let tool = tool();
        let object: &dyn AgentTool = &tool;
        assert_eq!(object.name(), "web_search");
        assert_eq!(object.nature(), ToolNature::Network);
        assert_eq!(object.family(), ToolFamily::Network);
        assert_eq!(object.summary_fields(), &["query"]);
        assert!(!object.is_read_only());
        assert!(object.requires_network_invocation());
        assert!(!object.requires_original_invocation());
        assert_eq!(
            object.parameters(),
            talos_core::tool_parameters!(WebSearchInput)
        );
        assert!(object.permission_profile(&input("q"))[0].resource.is_none());
        assert_eq!(format!("{tool:?}"), "GuardedSearchRouterTool(<redacted>)");
    }

    #[test]
    fn each_preparation_gets_fresh_invocation_identity_over_one_stable_scope() {
        let tool = tool();
        let first = tool
            .prepare_network_invocation(&input("first private query"))
            .expect("prepare")
            .expect("prepared");
        let second = tool
            .prepare_network_invocation(&input("second private query"))
            .expect("prepare")
            .expect("prepared");

        assert_eq!(first.manifest().scope(), second.manifest().scope());
        assert_ne!(
            first.manifest().invocation_id(),
            second.manifest().invocation_id()
        );
        assert_eq!(first.manifest().scope().endpoints().len(), 9);
        assert_eq!(first.permission_profile(), second.permission_profile());
        assert_eq!(first.permission_profile().len(), 6);
        assert!(!format!("{:?}", first.manifest()).contains("first private query"));
    }

    #[test]
    fn malformed_and_blank_inputs_fail_before_network_admission() {
        let tool = tool();
        for invalid in [serde_json::json!({"query": 7}), input("   ")] {
            assert!(matches!(
                tool.prepare_network_invocation(&invalid),
                Err(ToolNetworkPreparationError::Invalid)
            ));
        }
    }

    #[test]
    fn unavailable_configuration_fails_before_plan_creation() {
        let tool = tool().with_preparation_unavailable();
        assert!(matches!(
            tool.prepare_network_invocation(&input("must stay local")),
            Err(ToolNetworkPreparationError::Unavailable)
        ));
    }

    #[tokio::test]
    async fn every_legacy_execution_surface_stays_fail_closed() {
        let tool = tool();
        let object: &dyn AgentTool = &tool;
        let input = input("rust");
        let workspace = tempfile::tempdir().expect("workspace");
        let paths = [ToolExecutionAuthorization::for_path(
            "web_search",
            ToolNature::Network,
            workspace.path(),
            ".",
            ToolAuthorizationScope::Once,
        )
        .expect("path authorization")];

        for result in [
            object.execute(input.clone()).await,
            object.execute_authorized(input.clone(), &paths).await,
        ] {
            assert!(result.is_error);
            assert_eq!(result.content, "search_prepared_invocation_required");
        }
        for output in [
            object.execute_with_output(input.clone()).await,
            object.execute_authorized_with_output(input, &paths).await,
        ] {
            assert!(output.result.is_error);
            assert_eq!(output.result.content, "search_prepared_invocation_required");
            assert!(output.next_provider_parts.is_empty());
        }
    }
}
