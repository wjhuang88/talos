//! Dormant owned bridge from the prepared search composition into ADR-090 dispatch.
//! No AgentTool registration or production WebSearch selection changes occur here.

use std::sync::Arc;

use async_trait::async_trait;
use talos_core::tool::{
    PreparedNetworkToolInvocation, ToolExecutionOutput, ToolNetworkAuthority, ToolNetworkManifest,
    ToolPermissionFacet,
};
use tokio::sync::oneshot;
use tokio::time::Instant;

use super::SearchBackendId;
use super::composition::{
    PreparedSearchComposition, SearchCompositionConfiguration, SearchCompositionPreparationError,
};
#[cfg(feature = "search-integration-test")]
use super::integration_test::SearchRouterIntegrationProbe;
use super::router::{SearchRouterError, SearchRouterPolicy, SearchRouterSuccess, search_prepared};
use super::transport::TransportLimits;

/// One frozen router invocation prepared before permission admission.
///
/// The query and display preference are retained only for the final model-facing projection. The
/// candidate requests, manifest and result count are already frozen inside `composition` and cannot
/// be replaced during dispatch.
pub(in crate::search_backend) struct PreparedSearchRouterInvocation {
    composition: PreparedSearchComposition,
    policy: SearchRouterPolicy,
    query: String,
    include_snippets: bool,
    #[cfg(feature = "search-integration-test")]
    integration_probe: Option<Arc<SearchRouterIntegrationProbe>>,
}

impl std::fmt::Debug for PreparedSearchRouterInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedSearchRouterInvocation(<redacted>)")
    }
}

impl PreparedSearchRouterInvocation {
    pub(in crate::search_backend) fn prepare(
        configuration: &SearchCompositionConfiguration,
        query: &str,
        max_results: u32,
        include_snippets: bool,
        limits: TransportLimits,
        policy: SearchRouterPolicy,
    ) -> Result<Self, SearchCompositionPreparationError> {
        Ok(Self {
            composition: configuration.prepare(query, max_results, limits)?,
            policy,
            query: query.to_owned(),
            include_snippets,
            #[cfg(feature = "search-integration-test")]
            integration_probe: None,
        })
    }

    #[cfg(feature = "search-integration-test")]
    pub(in crate::search_backend) fn with_integration_probe(
        mut self,
        probe: Arc<SearchRouterIntegrationProbe>,
    ) -> Self {
        self.integration_probe = Some(probe);
        self
    }
}

#[async_trait]
impl PreparedNetworkToolInvocation for PreparedSearchRouterInvocation {
    fn manifest(&self) -> &ToolNetworkManifest {
        self.composition.manifest()
    }

    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        self.composition.permission_profile()
    }

    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput {
        #[cfg(feature = "search-integration-test")]
        let integration_probe = self.integration_probe.clone();
        let Self {
            composition,
            policy,
            query,
            include_snippets,
            ..
        } = *self;
        let running = composition.start(authority);
        let deadline = running.deadline();

        // Cross-crate C-V10 tests hold the real running composition here, before any candidate
        // future is polled. Caller/Session cancellation therefore proves ownership/drop propagation
        // without DNS/HTTP. Production builds do not compile this branch.
        #[cfg(feature = "search-integration-test")]
        let integration_receipt = integration_probe.map(|probe| probe.enter());
        #[cfg(feature = "search-integration-test")]
        if integration_receipt.is_some() {
            std::future::pending::<()>().await;
        }

        // PreparedNetworkToolInvocation has no separate cancellation token. The Agent owns and
        // awaits this future; dropping it drops this guard, Router and every owned candidate future.
        // Keeping the sender alive prevents the Router's closed-channel branch from firing during
        // normal execution without inventing a second cancellation authority.
        let (cancellation_guard, mut cancellation) = oneshot::channel();
        let routed = search_prepared(running, policy, &mut cancellation).await;
        drop(cancellation_guard);

        let success = match routed {
            Ok(success) => success,
            Err(error) => return ToolExecutionOutput::error(error.to_string()),
        };
        let content = match format_routed_results(&query, success, include_snippets) {
            Ok(content) => content,
            Err(error) => return ToolExecutionOutput::error(error),
        };
        if Instant::now() >= deadline {
            return ToolExecutionOutput::error(SearchRouterError::Deadline.to_string());
        }
        ToolExecutionOutput::success(content)
    }
}

fn format_routed_results(
    query: &str,
    success: SearchRouterSuccess,
    include_snippets: bool,
) -> Result<String, &'static str> {
    match success.backend {
        SearchBackendId::DuckDuckGo => Ok(crate::web_search::format_compatibility_results(
            query,
            success.results,
            include_snippets,
        )),
        SearchBackendId::SearXng => Ok(crate::web_search::format_searxng_results(
            query,
            success.results,
            include_snippets,
        )),
        SearchBackendId::Tavily | SearchBackendId::Wikipedia => {
            Err("search_router_unexpected_prepared_backend")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use talos_core::tool::{ToolNetworkAuthorizationError, ToolNetworkEndpoint};
    use uuid::Uuid;

    use super::*;
    use crate::search_backend::SearchBackendResult;

    #[derive(Default)]
    struct RejectAuthority {
        checks: AtomicUsize,
    }

    impl ToolNetworkAuthority for RejectAuthority {
        fn check(
            &self,
            _invocation_id: Uuid,
            _destination: &ToolNetworkEndpoint,
        ) -> Result<(), ToolNetworkAuthorizationError> {
            self.checks.fetch_add(1, Ordering::SeqCst);
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }

    fn limits() -> TransportLimits {
        TransportLimits::new(Duration::from_secs(5), 5, 1_048_576).expect("limits")
    }

    fn policy() -> SearchRouterPolicy {
        SearchRouterPolicy::new(Duration::from_millis(50), 1, 20, 8192).expect("policy")
    }

    #[test]
    fn preparation_forwards_one_manifest_and_profile_without_debug_query() {
        let configuration = SearchCompositionConfiguration::new(None).expect("configuration");
        let prepared = PreparedSearchRouterInvocation::prepare(
            &configuration,
            "private bridge query",
            2,
            true,
            limits(),
            policy(),
        )
        .expect("prepared");

        assert_eq!(prepared.manifest().scope(), configuration.scope());
        assert_eq!(prepared.permission_profile().len(), 4);
        assert!(!format!("{prepared:?}").contains("private bridge query"));
    }

    #[tokio::test(start_paused = true)]
    async fn rejected_authority_exhausts_without_legacy_or_network_fallback() {
        let configuration = SearchCompositionConfiguration::new(None).expect("configuration");
        let prepared = PreparedSearchRouterInvocation::prepare(
            &configuration,
            "private bridge query",
            1,
            true,
            limits(),
            policy(),
        )
        .expect("prepared");
        let authority = Arc::new(RejectAuthority::default());

        let output = Box::new(prepared).execute(authority.clone()).await;

        assert!(output.result.is_error);
        assert!(
            output
                .result
                .content
                .contains("all eligible search backends were exhausted")
        );
        assert!(!output.result.content.contains("private bridge query"));
        assert_eq!(authority.checks.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn routed_display_uses_only_admitted_prepared_backend_formatters() {
        let rows = vec![SearchBackendResult {
            backend: SearchBackendId::DuckDuckGo,
            title: "Rust".into(),
            url: "https://www.rust-lang.org/".into(),
            snippet: "language".into(),
        }];
        let expected = crate::web_search::format_compatibility_results("rust", rows.clone(), true);
        let actual = format_routed_results(
            "rust",
            SearchRouterSuccess {
                backend: SearchBackendId::DuckDuckGo,
                results: rows,
            },
            true,
        )
        .expect("display");
        assert_eq!(actual, expected);

        for backend in [SearchBackendId::Tavily, SearchBackendId::Wikipedia] {
            assert_eq!(
                format_routed_results(
                    "rust",
                    SearchRouterSuccess {
                        backend,
                        results: Vec::<SearchBackendResult>::new(),
                    },
                    true,
                ),
                Err("search_router_unexpected_prepared_backend")
            );
        }
    }

    #[test]
    fn invalid_preparation_stays_closed_before_dispatch() {
        let configuration = SearchCompositionConfiguration::new(None).expect("configuration");
        let error = PreparedSearchRouterInvocation::prepare(
            &configuration,
            "   ",
            1,
            true,
            limits(),
            policy(),
        )
        .expect_err("blank query");
        assert!(matches!(
            error,
            SearchCompositionPreparationError::Transport(_)
                | SearchCompositionPreparationError::Compatibility(_)
        ));
    }
}
