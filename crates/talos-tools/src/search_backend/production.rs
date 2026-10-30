//! Production composition for the protected Talos-owned `web_search` route.
//!
//! This module selects only the zero-key compatibility family. Optional providers remain outside
//! Auto selection and no ambient provider credential discovery occurs here. Supported HTTPS proxy
//! environment state is frozen once into the prepared configuration, including the NO_PROXY route
//! decision. The manifest exposes the sanitized proxy destination while credentials remain private.
//! Invalid or unsupported selected proxy configuration fails closed before query preparation.
//! Rollback is a version/deployment revert under ADR-090; this composition never falls back at
//! runtime to the legacy opaque `WebSearchTool` execution path.

use std::sync::Arc;
use std::time::Duration;

use talos_core::tool::AgentTool;

use super::composition::SearchCompositionConfiguration;
use super::router::SearchRouterPolicy;
use super::router_tool::GuardedSearchRouterTool;
use super::transport::{TransportError, TransportLimits};

// Legacy compatibility applies a 15-second timeout per request. Lite, HTML and Bing retry one
// Timeout/Network failure after 500ms; Instant Answer does not retry. That is at most seven
// query-bearing attempts across the four fixed destinations and about 106.5 seconds of request +
// retry-delay envelope, so 120 seconds bounds the invocation without cutting off a legacy-success
// path. A proxied HTTPS attempt reserves CONNECT + application send, hence fourteen physical units.
// This does not increase the maximum four query-bearing destinations. Response bytes retain the
// controlled transport's hard maximum. Auto currently has exactly one candidate, so hedge timing
// is unobservable; max_in_flight=1 makes that fact explicit.
const PRODUCTION_SEARCH_BUDGET: Duration = Duration::from_secs(120);
const PRODUCTION_SEARCH_MAX_REQUESTS: usize = 14;
const PRODUCTION_SEARCH_MAX_BYTES: usize = 1_048_576;
const PRODUCTION_SEARCH_MAX_EXAMINED_RESULTS: usize = 20;
const PRODUCTION_SEARCH_MAX_URL_BYTES: usize = 8192;

fn production_search_router_tool_with_configuration(
    configuration: SearchCompositionConfiguration,
    preparation_unavailable: bool,
) -> Arc<dyn AgentTool> {
    let limits = TransportLimits::new(
        PRODUCTION_SEARCH_BUDGET,
        PRODUCTION_SEARCH_MAX_REQUESTS,
        PRODUCTION_SEARCH_MAX_BYTES,
    )
    .expect("static protected search transport bounds must be valid");
    let policy = SearchRouterPolicy::new(
        Duration::ZERO,
        1,
        PRODUCTION_SEARCH_MAX_EXAMINED_RESULTS,
        PRODUCTION_SEARCH_MAX_URL_BYTES,
    )
    .expect("static protected search router bounds must be valid");
    let tool = GuardedSearchRouterTool::new(configuration, limits, policy);
    let tool = if preparation_unavailable {
        tool.with_preparation_unavailable()
    } else {
        tool
    };

    Arc::new(tool)
}

/// Builds the single production `web_search` implementation registered by Talos products.
///
/// Static compatibility destinations are repository-owned invariants. Proxy configuration is the
/// only ambient input and is snapshotted here, before any query exists. A supported snapshot is
/// carried through the prepared plan and controlled transport. An unsupported selected proxy never
/// causes a direct fallback: the tool stays registered but preparation closes as `Unavailable`.
pub(crate) fn production_search_router_tool() -> Arc<dyn AgentTool> {
    match SearchCompositionConfiguration::new_production() {
        Ok(configuration) => production_search_router_tool_with_configuration(configuration, false),
        Err(TransportError::UnsupportedProxy) => {
            let direct = SearchCompositionConfiguration::new(None)
                .expect("static protected search compatibility configuration must be valid");
            production_search_router_tool_with_configuration(direct, true)
        }
        Err(error) => panic!("invalid static protected search configuration: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use talos_core::tool::{AgentTool, ToolFamily, ToolNature, ToolNetworkPreparationError};

    use super::*;
    use crate::WebSearchTool;

    fn direct_production_tool() -> Arc<dyn AgentTool> {
        production_search_router_tool_with_configuration(
            SearchCompositionConfiguration::new(None).expect("direct configuration"),
            false,
        )
    }

    #[test]
    fn production_router_preserves_rollback_contract_and_requires_prepared_authority() {
        let protected = direct_production_tool();
        let legacy = WebSearchTool::new();
        let legacy: &dyn AgentTool = &legacy;

        assert_eq!(protected.name(), legacy.name());
        assert_eq!(protected.parameters(), legacy.parameters());
        assert_eq!(protected.nature(), ToolNature::Network);
        assert_eq!(protected.nature(), legacy.nature());
        assert_eq!(protected.family(), ToolFamily::Network);
        assert_eq!(protected.family(), legacy.family());
        assert_eq!(protected.summary_fields(), legacy.summary_fields());
        assert_eq!(protected.is_read_only(), legacy.is_read_only());
        assert!(protected.requires_network_invocation());
        assert!(!legacy.requires_network_invocation());

        let prepared = protected
            .prepare_network_invocation(&serde_json::json!({
                "query": "contract only",
                "max_results": 2,
                "include_snippets": true
            }))
            .expect("production preparation")
            .expect("protected invocation");
        assert_eq!(prepared.manifest().scope().endpoints().len(), 4);
        assert_eq!(prepared.permission_profile().len(), 4);
    }

    #[test]
    fn unsupported_proxy_configuration_stays_fail_closed() {
        let protected = production_search_router_tool_with_configuration(
            SearchCompositionConfiguration::new(None).expect("direct configuration"),
            true,
        );
        assert!(protected.requires_network_invocation());
        assert!(matches!(
            protected.prepare_network_invocation(&serde_json::json!({
                "query": "must not bypass configured proxy",
                "max_results": 2,
                "include_snippets": true
            })),
            Err(ToolNetworkPreparationError::Unavailable)
        ));
    }

    #[tokio::test]
    async fn production_router_legacy_execution_surface_stays_fail_closed() {
        let protected = direct_production_tool();
        let result = protected
            .execute(serde_json::json!({
                "query": "must not bypass prepared authority",
                "max_results": 1,
                "include_snippets": false
            }))
            .await;

        assert!(result.is_error);
        assert_eq!(result.content, "search_prepared_invocation_required");
    }
}
