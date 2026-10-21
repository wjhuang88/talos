//! Dormant direct-only configuration and its owned approval/execution plan.
//! No ambient proxy selection or official tool/root activation.

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
};
use crate::search_backend::transport::{SearchTransport, SystemSearchDns, TransportBudget};

/// Store once per direct-only configuration; never infer this from proxy availability.
pub(in crate::search_backend) struct DirectCompatibilityConfiguration {
    scope: ToolNetworkScope,
}

impl std::fmt::Debug for DirectCompatibilityConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DirectCompatibilityConfiguration(<redacted>)")
    }
}

impl DirectCompatibilityConfiguration {
    pub(in crate::search_backend) fn new() -> Result<Self, TransportError> {
        let endpoints = ORDER
            .iter()
            .map(|protocol| protocol.destination())
            .collect::<Result<Vec<_>, _>>()?;
        let scope = ToolNetworkScope::new_configuration(endpoints, None, vec![], 0)
            .map_err(|_| TransportError::InvalidRequest)?;
        Ok(Self { scope })
    }

    pub(in crate::search_backend) fn prepare(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedCompatibilityPlan, CompatibilityError> {
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        let chain = PreparedCompatibilityChain::prepare(&manifest, query, max_results)?;
        let facets = facets(&manifest);
        Ok(PreparedCompatibilityPlan {
            manifest,
            chain,
            facets,
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
        let transport = SearchTransport::new(self.manifest, authority, budget, dns)
            .map_err(CompatibilityPlanError::Transport)?;
        self.chain
            .execute(&transport)
            .await
            .map_err(CompatibilityPlanError::Chain)
    }
}

#[cfg(test)]
mod tests;
