//! Dormant prepared search-candidate composition.
//!
//! This layer owns one query-independent combined network scope, freezes all selected candidates
//! under one invocation manifest, and starts exactly one shared transport budget at dispatch.
//! It does not choose a winner, hedge, register a tool, or alter production WebSearch behavior.

use std::sync::Arc;

use talos_core::tool::{
    ToolNetworkAuthority, ToolNetworkManifest, ToolNetworkScope, ToolPermissionFacet,
};
use thiserror::Error;
use tokio::time::Instant;

use super::transport::compatibility::{
    CompatibilityError,
    chain::CompatibilityChainError,
    plan::{CompatibilityPlanError, DirectCompatibilityConfiguration, PreparedCompatibilityPlan},
    searxng::{PreparedSearXng, SearXngConfiguration, SearXngError},
    shared_budget_deadline,
};
use super::transport::{TransportBudget, TransportError, TransportLimits};
use super::{
    SearchBackendError, SearchBackendId, SearchBackendResult, bing_parser::BingParseError,
    ddg_parser::DdgParseError,
};

/// Stable query-independent configuration for the dormant public-search candidate set.
pub(crate) struct SearchCompositionConfiguration {
    direct: DirectCompatibilityConfiguration,
    searxng: Option<SearXngConfiguration>,
    scope: ToolNetworkScope,
}

impl std::fmt::Debug for SearchCompositionConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchCompositionConfiguration(<redacted>)")
    }
}

impl SearchCompositionConfiguration {
    /// Builds the zero-key direct candidate plus an optional explicitly supplied SearXNG route.
    /// Credential discovery and production selection are intentionally outside this stage.
    pub(crate) fn new(searxng: Option<SearXngConfiguration>) -> Result<Self, TransportError> {
        Self::from_direct(DirectCompatibilityConfiguration::new()?, searxng)
    }

    /// Production freezes supported environment-proxy state once at tool construction. Optional
    /// providers are deliberately absent so their proxy policy cannot be accidentally widened.
    pub(crate) fn new_production() -> Result<Self, TransportError> {
        Self::from_direct(
            DirectCompatibilityConfiguration::new_with_environment_proxy()?,
            None,
        )
    }

    fn from_direct(
        direct: DirectCompatibilityConfiguration,
        searxng: Option<SearXngConfiguration>,
    ) -> Result<Self, TransportError> {
        let scope = combined_scope(
            direct.scope(),
            searxng.as_ref().map(SearXngConfiguration::scope),
        )?;
        Ok(Self {
            direct,
            searxng,
            scope,
        })
    }

    pub(crate) fn scope(&self) -> &ToolNetworkScope {
        &self.scope
    }

    /// Freezes every selected candidate against one invocation identity without starting a clock.
    pub(crate) fn prepare(
        &self,
        query: &str,
        max_results: u32,
        limits: TransportLimits,
    ) -> Result<PreparedSearchComposition, SearchCompositionPreparationError> {
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        let direct = self
            .direct
            .prepare_with_manifest(&manifest, query, max_results)?;
        let facets = direct.permission_profile().to_vec();
        let mut candidates = vec![PreparedSearchCandidate::Direct(Box::new(direct))];

        if let Some(configuration) = &self.searxng {
            let prepared = configuration.prepare_with_manifest(&manifest, query, max_results)?;
            if prepared.permission_profile() != facets.as_slice() {
                return Err(TransportError::InvalidRequest.into());
            }
            candidates.push(PreparedSearchCandidate::SearXng(Box::new(prepared)));
        }

        Ok(PreparedSearchComposition {
            manifest,
            facets,
            candidates,
            limits,
            max_results,
        })
    }
}

fn combined_scope(
    direct: &ToolNetworkScope,
    searxng: Option<&ToolNetworkScope>,
) -> Result<ToolNetworkScope, TransportError> {
    let mut endpoints = direct.endpoints().to_vec();
    if let Some(searxng) = searxng {
        if direct.proxy() != searxng.proxy()
            || direct.private_addresses() != searxng.private_addresses()
            || direct.redirect_limit() != searxng.redirect_limit()
        {
            return Err(TransportError::InvalidRequest);
        }
        endpoints.extend_from_slice(searxng.endpoints());
    }
    ToolNetworkScope::new_configuration(
        endpoints,
        direct.proxy().cloned(),
        direct.private_addresses().to_vec(),
        direct.redirect_limit(),
    )
    .map_err(|_| TransportError::InvalidRequest)
}

#[derive(Debug, Error)]
pub(crate) enum SearchCompositionPreparationError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Compatibility(#[from] CompatibilityError),
}

/// One pre-approval object containing the complete manifest/profile and all frozen candidates.
pub(crate) struct PreparedSearchComposition {
    manifest: ToolNetworkManifest,
    facets: Vec<ToolPermissionFacet>,
    candidates: Vec<PreparedSearchCandidate>,
    limits: TransportLimits,
    max_results: u32,
}

impl std::fmt::Debug for PreparedSearchComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedSearchComposition(<redacted>)")
    }
}

impl PreparedSearchComposition {
    pub(crate) fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }

    pub(crate) fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }

    /// Starts the sole execution clock and distributes the same authority/budget to every
    /// candidate. No detached task is spawned; candidate futures remain caller-owned.
    pub(crate) fn start(
        self,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> RunningSearchComposition {
        let budget = Arc::new(self.limits.start());
        let candidates = self
            .candidates
            .into_iter()
            .map(|candidate| RunningSearchCandidate {
                candidate,
                authority: authority.clone(),
                budget: budget.clone(),
            })
            .collect();
        RunningSearchComposition {
            budget,
            candidates,
            max_results: self.max_results,
        }
    }
}

pub(crate) struct RunningSearchComposition {
    budget: Arc<TransportBudget>,
    candidates: Vec<RunningSearchCandidate>,
    max_results: u32,
}

impl std::fmt::Debug for RunningSearchComposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunningSearchComposition")
            .field(
                "candidates",
                &self
                    .candidates
                    .iter()
                    .map(RunningSearchCandidate::id)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl RunningSearchComposition {
    pub(crate) fn candidates(&self) -> &[RunningSearchCandidate] {
        &self.candidates
    }

    /// Exact monotonic deadline owned by the already-running shared transport budget.
    pub(crate) fn deadline(&self) -> Instant {
        shared_budget_deadline(&self.budget)
    }

    /// Frozen result count shared by every candidate; Router receives no replacement request.
    pub(crate) fn requested_results(&self) -> usize {
        self.max_results as usize
    }

    pub(crate) fn into_candidates(self) -> Vec<RunningSearchCandidate> {
        self.candidates
    }

    #[cfg(test)]
    fn budget(&self) -> &Arc<TransportBudget> {
        &self.budget
    }
}

enum PreparedSearchCandidate {
    Direct(Box<PreparedCompatibilityPlan>),
    SearXng(Box<PreparedSearXng>),
}

impl PreparedSearchCandidate {
    fn id(&self) -> SearchBackendId {
        match self {
            Self::Direct(_) => SearchBackendId::DuckDuckGo,
            Self::SearXng(_) => SearchBackendId::SearXng,
        }
    }
}

pub(crate) struct RunningSearchCandidate {
    candidate: PreparedSearchCandidate,
    authority: Arc<dyn ToolNetworkAuthority>,
    budget: Arc<TransportBudget>,
}

impl std::fmt::Debug for RunningSearchCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunningSearchCandidate")
            .field("backend", &self.id())
            .finish_non_exhaustive()
    }
}

impl RunningSearchCandidate {
    pub(crate) fn id(&self) -> SearchBackendId {
        self.candidate.id()
    }

    pub(crate) async fn execute(
        self,
    ) -> Result<Vec<SearchBackendResult>, SearchCandidateExecutionError> {
        match self.candidate {
            PreparedSearchCandidate::Direct(plan) => (*plan)
                .execute(self.authority, self.budget)
                .await
                .map(|rows| rows.into_results())
                .map_err(SearchCandidateExecutionError::Compatibility),
            PreparedSearchCandidate::SearXng(plan) => (*plan)
                .execute(self.authority, self.budget)
                .await
                .map(|rows| rows.into_results())
                .map_err(SearchCandidateExecutionError::SearXng),
        }
    }

    #[cfg(test)]
    fn budget(&self) -> &Arc<TransportBudget> {
        &self.budget
    }
}

#[derive(Debug, Error)]
pub(crate) enum SearchCandidateExecutionError {
    #[error(transparent)]
    Compatibility(CompatibilityPlanError),
    #[error(transparent)]
    SearXng(SearXngError),
}

impl SearchCandidateExecutionError {
    /// Collapse controlled transport/parser details into the existing private backend taxonomy.
    /// Router deadline/cancellation still win as enclosing control outcomes.
    pub(crate) fn into_backend_error(self) -> SearchBackendError {
        match self {
            Self::Compatibility(error) => compatibility_plan_backend_error(error),
            Self::SearXng(error) => searxng_backend_error(error),
        }
    }
}

fn transport_backend_error(error: TransportError) -> SearchBackendError {
    match error {
        TransportError::Dns | TransportError::ConnectionRefused | TransportError::Network => {
            SearchBackendError::Unreachable
        }
        TransportError::Deadline => SearchBackendError::Timeout,
        TransportError::ResponseEncoding => SearchBackendError::Parse,
        TransportError::InvalidRequest
        | TransportError::Unauthorized
        | TransportError::UnsupportedProxy
        | TransportError::Address
        | TransportError::Redirect
        | TransportError::RequestLimit
        | TransportError::ResponseLimit
        | TransportError::Request => {
            SearchBackendError::RequestFailed("controlled search transport failed".into())
        }
    }
}

fn compatibility_backend_error(error: CompatibilityError) -> SearchBackendError {
    match error {
        CompatibilityError::Transport(error) => transport_backend_error(error),
        CompatibilityError::Ddg(error) => match error {
            DdgParseError::Empty => SearchBackendError::NoResults,
            DdgParseError::Encoding | DdgParseError::Limit | DdgParseError::Parse => {
                SearchBackendError::Parse
            }
            DdgParseError::Status | DdgParseError::Challenge => SearchBackendError::RequestFailed(
                "duckduckgo compatibility source unavailable".into(),
            ),
        },
        CompatibilityError::Bing(error) => match error {
            BingParseError::Empty => SearchBackendError::NoResults,
            BingParseError::RateLimited => SearchBackendError::RateLimited,
            BingParseError::Encoding | BingParseError::Limit | BingParseError::Parse => {
                SearchBackendError::Parse
            }
            BingParseError::Status | BingParseError::Challenge => {
                SearchBackendError::RequestFailed("bing compatibility source unavailable".into())
            }
        },
        // Admission failures are already inside the controlled candidate boundary. Preserve a
        // conservative semantic failure rather than guessing a backend availability condition.
        CompatibilityError::Admission(_) => SearchBackendError::Parse,
    }
}

fn compatibility_plan_backend_error(error: CompatibilityPlanError) -> SearchBackendError {
    match error {
        CompatibilityPlanError::Transport(error) => transport_backend_error(error),
        CompatibilityPlanError::Chain(CompatibilityChainError::Terminal { error, .. }) => {
            compatibility_backend_error(error)
        }
        CompatibilityPlanError::Chain(CompatibilityChainError::Exhausted(errors)) => {
            let mapped = errors.map(compatibility_backend_error);
            let first = mapped[0].clone();
            if mapped.iter().all(|error| error == &first) {
                first
            } else {
                SearchBackendError::RequestFailed("compatibility sources exhausted".into())
            }
        }
    }
}

fn searxng_backend_error(error: SearXngError) -> SearchBackendError {
    match error {
        SearXngError::Transport(error) => transport_backend_error(error),
        SearXngError::Admission(error) => compatibility_backend_error(error),
        SearXngError::Status => SearchBackendError::RequestFailed("searxng status failure".into()),
        SearXngError::Parse | SearXngError::Limit => SearchBackendError::Parse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use talos_core::tool::{ToolNetworkAuthorizationError, ToolNetworkEndpoint};
    use uuid::Uuid;

    struct RejectAuthority;

    impl ToolNetworkAuthority for RejectAuthority {
        fn check(
            &self,
            _invocation_id: Uuid,
            _destination: &ToolNetworkEndpoint,
        ) -> Result<(), ToolNetworkAuthorizationError> {
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }

    fn limits() -> TransportLimits {
        TransportLimits::new(Duration::from_secs(5), 5, 1_048_576).expect("limits")
    }

    #[test]
    fn combined_configuration_identity_is_stable_across_queries() {
        let searxng = SearXngConfiguration::new("https://search.example/prefix/").expect("searxng");
        let configuration = SearchCompositionConfiguration::new(Some(searxng)).expect("config");
        let first = configuration
            .prepare("first private query", 2, limits())
            .expect("first");
        let second = configuration
            .prepare("second private query", 2, limits())
            .expect("second");

        assert_eq!(first.manifest().scope(), configuration.scope());
        assert_eq!(second.manifest().scope(), configuration.scope());
        assert_eq!(
            first.manifest().scope().configuration_id(),
            second.manifest().scope().configuration_id()
        );
        assert_ne!(
            first.manifest().invocation_id(),
            second.manifest().invocation_id()
        );
        assert_eq!(first.permission_profile(), second.permission_profile());
        assert_eq!(first.permission_profile().len(), 5);
        assert!(!format!("{first:?}").contains("first private query"));
    }

    #[test]
    fn dispatch_shares_one_budget_deadline_and_requested_count() {
        let searxng = SearXngConfiguration::new("https://search.example/").expect("searxng");
        let configuration = SearchCompositionConfiguration::new(Some(searxng)).expect("config");
        let prepared = configuration
            .prepare("private query", 2, limits())
            .expect("prepared");
        let manifest_id = prepared.manifest().invocation_id();
        let running = prepared.start(Arc::new(RejectAuthority));

        assert_eq!(running.candidates().len(), 2);
        assert_eq!(running.candidates()[0].id(), SearchBackendId::DuckDuckGo);
        assert_eq!(running.candidates()[1].id(), SearchBackendId::SearXng);
        assert_eq!(running.requested_results(), 2);
        assert_eq!(running.deadline(), shared_budget_deadline(running.budget()));
        for candidate in running.candidates() {
            assert!(Arc::ptr_eq(running.budget(), candidate.budget()));
        }
        assert!(!format!("{running:?}").contains(&manifest_id.to_string()));
    }

    #[test]
    fn direct_only_configuration_keeps_one_candidate_and_combined_scope() {
        let configuration = SearchCompositionConfiguration::new(None).expect("config");
        let prepared = configuration
            .prepare("private query", 2, limits())
            .expect("prepared");
        assert_eq!(prepared.manifest().scope().endpoints().len(), 4);
        let running = prepared.start(Arc::new(RejectAuthority));
        assert_eq!(running.candidates().len(), 1);
        assert_eq!(running.candidates()[0].id(), SearchBackendId::DuckDuckGo);
    }

    #[test]
    fn execution_errors_map_without_guessing_router_control() {
        assert_eq!(
            SearchCandidateExecutionError::SearXng(SearXngError::Transport(
                TransportError::Deadline
            ))
            .into_backend_error(),
            SearchBackendError::Timeout
        );
        assert_eq!(
            SearchCandidateExecutionError::SearXng(SearXngError::Transport(TransportError::Dns))
                .into_backend_error(),
            SearchBackendError::Unreachable
        );
        assert_eq!(
            SearchCandidateExecutionError::SearXng(SearXngError::Transport(
                TransportError::Network
            ))
            .into_backend_error(),
            SearchBackendError::Unreachable
        );
        assert_eq!(
            SearchCandidateExecutionError::SearXng(SearXngError::Parse).into_backend_error(),
            SearchBackendError::Parse
        );
    }
}
