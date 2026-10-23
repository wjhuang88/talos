//! Dormant SearXNG bridge; no tool registration or configuration discovery.
use super::{
    PreparedSearXng, SearXngConfiguration, SearXngError, admission::AdmittedCompatibilityResults,
};
use crate::search_backend::transport::{TransportError, TransportLimits};
use async_trait::async_trait;
use std::sync::Arc;
use talos_core::tool::{
    PreparedNetworkToolInvocation, ToolExecutionOutput, ToolNetworkAuthority, ToolNetworkManifest,
    ToolPermissionFacet,
};

pub(in crate::search_backend) mod tool;

pub(in crate::search_backend) struct PreparedSearXngInvocation {
    plan: PreparedSearXng,
    query: String,
    include_snippets: bool,
    limits: TransportLimits,
}
impl std::fmt::Debug for PreparedSearXngInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedSearXngInvocation(<redacted>)")
    }
}
impl PreparedSearXngInvocation {
    pub(in crate::search_backend) fn prepare(
        configuration: &SearXngConfiguration,
        query: &str,
        max_results: u32,
        include_snippets: bool,
        limits: TransportLimits,
    ) -> Result<Self, TransportError> {
        Ok(Self {
            plan: configuration.prepare(query, max_results)?,
            query: query.to_owned(),
            include_snippets,
            limits,
        })
    }
}
#[async_trait]
impl PreparedNetworkToolInvocation for PreparedSearXngInvocation {
    fn manifest(&self) -> &ToolNetworkManifest {
        self.plan.manifest()
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        self.plan.permission_profile()
    }
    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput {
        let Self {
            plan,
            query,
            include_snippets,
            limits,
        } = *self;
        // Preparation/approval waiting never consumes this sole execution budget.
        let budget = Arc::new(limits.start());
        let outcome = plan.execute(authority, budget.clone()).await;
        finish_output(outcome, &query, include_snippets, || budget.check_live())
    }
}
// The production checker always uses that same execution budget. Private injection permits
// deterministic completion-fence tests, not a caller-supplied clock or formatting interface.
fn finish_output(
    outcome: Result<AdmittedCompatibilityResults, SearXngError>,
    query: &str,
    include_snippets: bool,
    check_live: impl FnOnce() -> Result<(), TransportError>,
) -> ToolExecutionOutput {
    let formatted = outcome.map(|rows| {
        crate::web_search::format_searxng_results(query, rows.into_results(), include_snippets)
    });
    // Fence both formatted success and error completion; expiration takes precedence.
    if let Err(error) = check_live() {
        return ToolExecutionOutput::error(error.to_string());
    }
    match formatted {
        Ok(content) => ToolExecutionOutput::success(content),
        Err(error) => ToolExecutionOutput::error(error.to_string()),
    }
}
#[cfg(test)]
mod tests;
