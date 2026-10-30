//! Dormant compatibility configuration and its owned approval/execution plan.
//! Production may freeze one environment proxy at construction; no runtime ambient selection.

use std::sync::Arc;

use talos_core::tool::{
    ToolNature, ToolNetworkAuthority, ToolNetworkManifest, ToolNetworkScope, ToolPermissionFacet,
    ToolResourceKind,
};
use thiserror::Error;

use super::{
    CompatibilityError, TransportError,
    admission::AdmittedCompatibilityResults,
    chain::{CompatibilityChainError, ORDER, PreparedCompatibilityChain},
    manifest_extends_scope_without_policy_widening,
};
use crate::search_backend::transport::{
    SearchTransport, SystemSearchDns, TransportBudget, TransportProxy,
};

/// Store once per compatibility configuration. Production freezes ambient proxy state here rather
/// than consulting environment variables while preparing or executing a query.
pub(in crate::search_backend) struct DirectCompatibilityConfiguration {
    scope: ToolNetworkScope,
    proxy: Option<Arc<TransportProxy>>,
}

impl std::fmt::Debug for DirectCompatibilityConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DirectCompatibilityConfiguration(<redacted>)")
    }
}

impl DirectCompatibilityConfiguration {
    pub(in crate::search_backend) fn new() -> Result<Self, TransportError> {
        Self::build(false)
    }

    pub(in crate::search_backend) fn new_with_environment_proxy() -> Result<Self, TransportError> {
        Self::build(true)
    }

    fn build(snapshot_environment_proxy: bool) -> Result<Self, TransportError> {
        let endpoints = ORDER
            .iter()
            .map(|protocol| protocol.destination())
            .collect::<Result<Vec<_>, _>>()?;
        let proxy = if snapshot_environment_proxy {
            TransportProxy::from_environment(&endpoints)?
        } else {
            None
        };
        let scope = ToolNetworkScope::new_configuration(
            endpoints,
            proxy.as_ref().map(|proxy| proxy.endpoint().clone()),
            vec![],
            0,
        )
        .map_err(|_| TransportError::InvalidRequest)?;
        Ok(Self { scope, proxy })
    }

    pub(in crate::search_backend) fn scope(&self) -> &ToolNetworkScope {
        &self.scope
    }

    pub(in crate::search_backend) fn prepare(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedCompatibilityPlan, CompatibilityError> {
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        self.prepare_with_manifest(&manifest, query, max_results)
    }

    /// Freeze this candidate against an invocation manifest owned by a future combined plan.
    /// Endpoints may be a superset, but global proxy/private/redirect policy may not widen.
    pub(in crate::search_backend) fn prepare_with_manifest(
        &self,
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedCompatibilityPlan, CompatibilityError> {
        if !manifest_extends_scope_without_policy_widening(manifest, &self.scope) {
            return Err(TransportError::InvalidRequest.into());
        }
        let chain = PreparedCompatibilityChain::prepare(manifest, query, max_results)?;
        let facets = facets(manifest);
        Ok(PreparedCompatibilityPlan {
            manifest: manifest.clone(),
            chain,
            facets,
            proxy: self.proxy.clone(),
            #[cfg(test)]
            dns: Arc::new(SystemSearchDns),
        })
    }
}

fn facets(manifest: &ToolNetworkManifest) -> Vec<ToolPermissionFacet> {
    let mut hosts: Vec<_> = manifest
        .scope()
        .endpoints()
        .iter()
        .map(|endpoint| endpoint.host())
        .collect();
    if let Some(proxy) = manifest.scope().proxy() {
        hosts.push(proxy.host());
    }
    hosts.sort_unstable();
    hosts.dedup();
    hosts
        .into_iter()
        .map(|host| {
            ToolPermissionFacet::with_resource(ToolNature::Network, host, ToolResourceKind::Domain)
        })
        .collect()
}

/// One consuming object retains the manifest shown for approval and all frozen requests.
/// Neither this object nor a manifest clone grants permission.
pub(in crate::search_backend) struct PreparedCompatibilityPlan {
    manifest: ToolNetworkManifest,
    chain: PreparedCompatibilityChain,
    facets: Vec<ToolPermissionFacet>,
    proxy: Option<Arc<TransportProxy>>,
    #[cfg(test)]
    dns: Arc<dyn crate::search_backend::transport::SearchDns>,
}

impl std::fmt::Debug for PreparedCompatibilityPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedCompatibilityPlan(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum CompatibilityPlanError {
    #[error("search_compatibility_plan_transport")]
    Transport(TransportError),
    #[error("search_compatibility_plan_chain")]
    Chain(CompatibilityChainError),
}

impl PreparedCompatibilityPlan {
    pub(in crate::search_backend) fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }

    pub(in crate::search_backend) fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }

    pub(in crate::search_backend) async fn execute(
        self,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityPlanError> {
        #[cfg(not(test))]
        let dns = Arc::new(SystemSearchDns);
        #[cfg(test)]
        let dns = self.dns;
        let transport =
            SearchTransport::new_with_proxy(self.manifest, authority, budget, dns, self.proxy)
                .map_err(CompatibilityPlanError::Transport)?;
        self.chain
            .execute(&transport)
            .await
            .map_err(CompatibilityPlanError::Chain)
    }
}

#[cfg(test)]
pub(super) mod tests;

#[cfg(test)]
mod shared_manifest_tests {
    use super::*;
    use crate::search_backend::transport::compatibility::searxng::SearXngConfiguration;
    use std::net::IpAddr;
    use talos_core::tool::ToolNetworkEndpoint;

    fn endpoints(
        direct: &DirectCompatibilityConfiguration,
        searxng: &SearXngConfiguration,
    ) -> Vec<ToolNetworkEndpoint> {
        let mut endpoints = direct.scope().endpoints().to_vec();
        endpoints.extend_from_slice(searxng.scope().endpoints());
        endpoints
    }

    fn shared_manifest(
        direct: &DirectCompatibilityConfiguration,
        searxng: &SearXngConfiguration,
    ) -> ToolNetworkManifest {
        ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(endpoints(direct, searxng), None, vec![], 0)
                .expect("shared scope"),
        )
    }

    #[test]
    fn public_candidates_can_freeze_against_one_shared_invocation_manifest() {
        let direct = DirectCompatibilityConfiguration::new().expect("direct");
        let searxng = SearXngConfiguration::new("https://search.example/prefix/").expect("searxng");
        let manifest = shared_manifest(&direct, &searxng);

        let direct_plan = direct
            .prepare_with_manifest(&manifest, "private query", 2)
            .expect("direct plan");
        let searxng_plan = searxng
            .prepare_with_manifest(&manifest, "private query", 2)
            .expect("searxng plan");

        assert_eq!(direct_plan.manifest(), &manifest);
        assert_eq!(searxng_plan.manifest(), &manifest);
        assert_eq!(
            direct_plan.manifest().invocation_id(),
            searxng_plan.manifest().invocation_id()
        );
        assert_eq!(
            direct_plan.permission_profile(),
            searxng_plan.permission_profile()
        );
        assert_eq!(manifest.scope().endpoints().len(), 5);
        assert_eq!(direct_plan.permission_profile().len(), 5);
        assert!(!format!("{manifest:?}").contains("private query"));
    }

    #[test]
    fn shared_manifest_cannot_widen_global_network_policy() {
        let direct = DirectCompatibilityConfiguration::new().expect("direct");
        let searxng = SearXngConfiguration::new("https://search.example/").expect("searxng");
        let proxy =
            ToolNetworkEndpoint::new("http", "proxy.example", 8080, "CONNECT", "/").expect("proxy");
        let private: IpAddr = "127.0.0.1".parse().expect("private ip");
        for (proxy, private_addresses, redirect_limit) in [
            (Some(proxy), vec![], 0),
            (None, vec![private], 0),
            (None, vec![], 1),
        ] {
            let manifest = ToolNetworkManifest::new(
                ToolNetworkScope::new_configuration(
                    endpoints(&direct, &searxng),
                    proxy,
                    private_addresses,
                    redirect_limit,
                )
                .expect("scope"),
            );
            assert!(matches!(
                direct.prepare_with_manifest(&manifest, "q", 1),
                Err(CompatibilityError::Transport(
                    TransportError::InvalidRequest
                ))
            ));
            assert!(matches!(
                searxng.prepare_with_manifest(&manifest, "q", 1),
                Err(TransportError::InvalidRequest)
            ));
        }
    }
}
