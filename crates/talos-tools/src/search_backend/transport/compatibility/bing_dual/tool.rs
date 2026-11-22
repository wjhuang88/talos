//! Explicit, protected Bing-family opt-in. The default Auto SearchRouter is never changed.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, PreparedNetworkToolInvocation, ToolExecutionOutput, ToolFamily, ToolNature,
    ToolNetworkAuthority, ToolNetworkManifest, ToolNetworkPreparationError, ToolNetworkScope,
    ToolPermissionFacet, ToolResourceKind, ToolResult,
};

use super::super::scope_with_proxy;
use super::PreparedBingDualHost;
use crate::search_backend::transport::{
    SearchTransport, SystemSearchDns, TransportError, TransportLimits, TransportProxy,
};
use crate::web_search::{WebSearchInput, format_compatibility_results};

/// The ONLY way to select the new WWW recipient is explicit caller configuration.
pub(in crate::search_backend) struct GuardedBingDualSearchTool {
    scope: ToolNetworkScope,
    proxy: Option<Arc<TransportProxy>>,
    limits: TransportLimits,
}

impl std::fmt::Debug for GuardedBingDualSearchTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GuardedBingDualSearchTool(<redacted>)")
    }
}

impl GuardedBingDualSearchTool {
    pub(in crate::search_backend) fn new(limits: TransportLimits) -> Result<Self, TransportError> {
        let destinations = PreparedBingDualHost::destinations()?;
        // Snapshot one proxy decision for both approved hosts before any query is known.
        let proxy = TransportProxy::from_environment(&destinations)?;
        let base_scope =
            ToolNetworkScope::new_configuration(destinations.to_vec(), None, vec![], 0)
                .map_err(|_| TransportError::InvalidRequest)?;
        let scope = scope_with_proxy(&base_scope, proxy.as_ref())?;
        Ok(Self {
            scope,
            proxy,
            limits,
        })
    }

    fn prepare(
        &self,
        input: &Value,
    ) -> Result<PreparedBingDualInvocation, ToolNetworkPreparationError> {
        let parsed: WebSearchInput = serde_json::from_value(input.clone())
            .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        let plan = PreparedBingDualHost::prepare(
            &manifest,
            &parsed.query,
            parsed.max_results.clamp(1, 20),
        )
        .map_err(|_| ToolNetworkPreparationError::Invalid)?;
        let mut hosts: Vec<_> = self.scope.endpoints().iter().map(|ep| ep.host()).collect();
        if let Some(proxy) = self.scope.proxy() {
            hosts.push(proxy.host());
        }
        hosts.sort_unstable();
        hosts.dedup();
        let facets = hosts
            .into_iter()
            .map(|host| {
                ToolPermissionFacet::with_resource(
                    ToolNature::Network,
                    host,
                    ToolResourceKind::Domain,
                )
            })
            .collect();
        Ok(PreparedBingDualInvocation {
            plan,
            manifest,
            facets,
            proxy: self.proxy.clone(),
            limits: self.limits,
            query: parsed.query,
            include_snippets: parsed.include_snippets,
        })
    }
}

struct PreparedBingDualInvocation {
    plan: PreparedBingDualHost,
    manifest: ToolNetworkManifest,
    facets: Vec<ToolPermissionFacet>,
    proxy: Option<Arc<TransportProxy>>,
    limits: TransportLimits,
    query: String,
    include_snippets: bool,
}

impl std::fmt::Debug for PreparedBingDualInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedBingDualInvocation(<redacted>)")
    }
}

#[async_trait]
impl PreparedNetworkToolInvocation for PreparedBingDualInvocation {
    fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }

    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }

    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput {
        let Self {
            plan,
            manifest,
            facets: _,
            proxy,
            limits,
            query,
            include_snippets,
        } = *self;
        // The same once-only clock starts when execution begins, AFTER the approval prompt.
        let budget = Arc::new(limits.start());
        let outcome = match SearchTransport::new_with_proxy(
            manifest,
            authority,
            budget.clone(),
            Arc::new(SystemSearchDns),
            proxy,
        ) {
            Ok(transport) => plan.execute(&transport).await.map(|rows| {
                format_compatibility_results(&query, rows.into_results(), include_snippets)
            }),
            Err(error) => Err(error.into()),
        };
        if let Err(error) = budget.check_live() {
            return ToolExecutionOutput::error(error.to_string());
        }
        match outcome {
            Ok(content) => ToolExecutionOutput::success(content),
            Err(error) => ToolExecutionOutput::error(error.to_string()),
        }
    }
}

#[async_trait]
impl AgentTool for GuardedBingDualSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Explicitly selected, protected Bing WWW-first/CN-fallback search.          Requires approval for both prepared network recipients; no default activation."
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
