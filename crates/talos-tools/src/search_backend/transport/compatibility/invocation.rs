//! Dormant bridge to the existing core prepared-network trait, without root activation.

use async_trait::async_trait;
use std::sync::Arc;
use talos_core::tool::{
    PreparedNetworkToolInvocation, ToolExecutionOutput, ToolNetworkAuthority, ToolNetworkManifest,
    ToolPermissionFacet,
};

use super::{
    CompatibilityError,
    plan::{DirectCompatibilityConfiguration, PreparedCompatibilityPlan},
};
use crate::search_backend::transport::TransportLimits;

pub(in crate::search_backend) mod tool;

/// Retains one frozen plan and display options; approval waiting starts no execution clock.
pub(in crate::search_backend) struct PreparedCompatibilityInvocation {
    plan: PreparedCompatibilityPlan,
    query: String,
    include_snippets: bool,
    limits: TransportLimits,
}

impl std::fmt::Debug for PreparedCompatibilityInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedCompatibilityInvocation(<redacted>)")
    }
}

impl PreparedCompatibilityInvocation {
    pub(in crate::search_backend) fn prepare(
        configuration: &DirectCompatibilityConfiguration,
        query: &str,
        max_results: u32,
        include_snippets: bool,
        limits: TransportLimits,
    ) -> Result<Self, CompatibilityError> {
        let plan = configuration.prepare(query, max_results)?;
        Ok(Self {
            plan,
            query: query.to_owned(),
            include_snippets,
            limits,
        })
    }
}

#[async_trait]
impl PreparedNetworkToolInvocation for PreparedCompatibilityInvocation {
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
        // Exactly one clock starts at actual dispatch, not preparation/approval time.
        let budget = Arc::new(limits.start());
        let outcome = plan.execute(authority, budget.clone()).await.map(|rows| {
            crate::web_search::format_compatibility_results(
                &query,
                rows.into_results(),
                include_snippets,
            )
        });
        // Formatting/conversion must not let a late success escape the enclosing budget.
        if let Err(error) = budget.check_live() {
            return ToolExecutionOutput::error(error.to_string());
        }
        match outcome {
            Ok(content) => ToolExecutionOutput::success(content),
            Err(error) => ToolExecutionOutput::error(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests;
