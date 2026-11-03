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

use super::selection::{
    SearchBackendAvailability, SearchBackendSelection, SearchBackendSelectionError,
};
use super::transport::compatibility::{
    CompatibilityError,
    chain::CompatibilityChainError,
    plan::{CompatibilityPlanError, DirectCompatibilityConfiguration, PreparedCompatibilityPlan},
    searxng::{PreparedSearXng, SearXngConfiguration, SearXngError},
    shared_budget_deadline,
    tavily::{PreparedTavily, TavilyConfiguration, TavilyError},
};
use super::transport::{TransportBudget, TransportError, TransportLimits, TransportProxy};
use super::{
    SearchBackendError, SearchBackendId, SearchBackendResult, bing_parser::BingParseError,
    ddg_parser::DdgParseError,
};

/// Stable query-independent configuration for the dormant public-search candidate set.
pub(crate) struct SearchCompositionConfiguration {
    direct: Option<DirectCompatibilityConfiguration>,
    tavily: Option<TavilyConfiguration>,
    searxng: Option<SearXngConfiguration>,
    candidate_order: Vec<SearchBackendId>,
    scope: ToolNetworkScope,
}

impl std::fmt::Debug for SearchCompositionConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchCompositionConfiguration(<redacted>)")
    }
}

impl SearchCompositionConfiguration {
    /// Builds the legacy staged direct candidate plus an optional explicitly supplied SearXNG
    /// route. New explicit override composition should use `selected` so selection policy owns the
    /// exact candidate order.
    pub(crate) fn new(searxng: Option<SearXngConfiguration>) -> Result<Self, TransportError> {
        let candidate_order = if searxng.is_some() {
            vec![SearchBackendId::DuckDuckGo, SearchBackendId::SearXng]
        } else {
            vec![SearchBackendId::DuckDuckGo]
        };
        Self::from_parts(
            Some(DirectCompatibilityConfiguration::new()?),
            None,
            searxng,
            candidate_order,
        )
    }

    /// Production freezes supported environment-proxy state exactly once against the complete
    /// selected endpoint union. Auto is still direct-only and zero-key.
    pub(crate) fn new_production() -> Result<Self, TransportError> {
        Self::new(None)?.with_environment_proxy()
    }

    /// Compose exactly the candidates admitted by explicit selection policy.
    ///
    /// Optional provider configuration establishes availability only. Any supplied provider that is
    /// not selected is discarded before scope construction, so mere availability cannot widen the
    /// manifest or activate metered/self-hosted traffic. Auto therefore remains direct-only even if
    /// both optional configurations are present.
    pub(crate) fn selected(
        selection: SearchBackendSelection,
        tavily: Option<TavilyConfiguration>,
        searxng: Option<SearXngConfiguration>,
    ) -> Result<Self, SearchCompositionConfigurationError> {
        let availability = SearchBackendAvailability::new(tavily.is_some(), searxng.is_some());
        let candidate_order = selection.resolve(availability)?;
        let direct = if candidate_order.contains(&SearchBackendId::DuckDuckGo) {
            Some(DirectCompatibilityConfiguration::new()?)
        } else {
            None
        };
        let tavily = if candidate_order.contains(&SearchBackendId::Tavily) {
            tavily
        } else {
            None
        };
        let searxng = if candidate_order.contains(&SearchBackendId::SearXng) {
            searxng
        } else {
            None
        };
        Self::from_parts(direct, tavily, searxng, candidate_order)
            .map_err(SearchCompositionConfigurationError::from)
    }

    pub(crate) fn with_environment_proxy(self) -> Result<Self, TransportError> {
        let proxy = TransportProxy::from_environment(self.scope.endpoints())?;
        self.with_proxy(proxy)
    }

    fn with_proxy(self, proxy: Option<Arc<TransportProxy>>) -> Result<Self, TransportError> {
        let direct = self
            .direct
            .map(|configuration| configuration.with_proxy(proxy.clone()))
            .transpose()?;
        let tavily = self
            .tavily
            .map(|configuration| configuration.with_proxy(proxy.clone()))
            .transpose()?;
        let searxng = self
            .searxng
            .map(|configuration| configuration.with_proxy(proxy.clone()))
            .transpose()?;
        Self::from_parts(direct, tavily, searxng, self.candidate_order)
    }

    fn from_parts(
        direct: Option<DirectCompatibilityConfiguration>,
        tavily: Option<TavilyConfiguration>,
        searxng: Option<SearXngConfiguration>,
        candidate_order: Vec<SearchBackendId>,
    ) -> Result<Self, TransportError> {
        let mut scopes = Vec::with_capacity(candidate_order.len());
        for backend in &candidate_order {
            let scope = match backend {
                SearchBackendId::DuckDuckGo => {
                    direct.as_ref().map(DirectCompatibilityConfiguration::scope)
                }
                SearchBackendId::Tavily => tavily.as_ref().map(TavilyConfiguration::scope),
                SearchBackendId::SearXng => searxng.as_ref().map(SearXngConfiguration::scope),
                SearchBackendId::Wikipedia => None,
            }
            .ok_or(TransportError::InvalidRequest)?;
            scopes.push(scope);
        }
        let scope = combined_scope(&scopes)?;
        Ok(Self {
            direct,
            tavily,
            searxng,
            candidate_order,
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
        let mut facets = None;
        let mut candidates = Vec::with_capacity(self.candidate_order.len());

        for backend in &self.candidate_order {
            match backend {
                SearchBackendId::DuckDuckGo => {
                    let configuration =
                        self.direct.as_ref().ok_or(TransportError::InvalidRequest)?;
                    let prepared =
                        configuration.prepare_with_manifest(&manifest, query, max_results)?;
                    freeze_facets(&mut facets, prepared.permission_profile())?;
                    candidates.push(PreparedSearchCandidate::Direct(Box::new(prepared)));
                }
                SearchBackendId::Tavily => {
                    let configuration =
                        self.tavily.as_ref().ok_or(TransportError::InvalidRequest)?;
                    let prepared =
                        configuration.prepare_with_manifest(&manifest, query, max_results)?;
                    freeze_facets(&mut facets, prepared.permission_profile())?;
                    candidates.push(PreparedSearchCandidate::Tavily(Box::new(prepared)));
                }
                SearchBackendId::SearXng => {
                    let configuration = self
                        .searxng
                        .as_ref()
                        .ok_or(TransportError::InvalidRequest)?;
                    let prepared =
                        configuration.prepare_with_manifest(&manifest, query, max_results)?;
                    freeze_facets(&mut facets, prepared.permission_profile())?;
                    candidates.push(PreparedSearchCandidate::SearXng(Box::new(prepared)));
                }
                SearchBackendId::Wikipedia => return Err(TransportError::InvalidRequest.into()),
            }
        }

        Ok(PreparedSearchComposition {
            manifest,
            facets: facets.ok_or(TransportError::InvalidRequest)?,
            candidates,
            limits,
            max_results,
        })
    }
}

fn freeze_facets(
    frozen: &mut Option<Vec<ToolPermissionFacet>>,
    candidate: &[ToolPermissionFacet],
) -> Result<(), TransportError> {
    match frozen {
        Some(existing) if existing.as_slice() != candidate => Err(TransportError::InvalidRequest),
        Some(_) => Ok(()),
        None => {
            *frozen = Some(candidate.to_vec());
            Ok(())
        }
    }
}

fn combined_scope(scopes: &[&ToolNetworkScope]) -> Result<ToolNetworkScope, TransportError> {
    let first = *scopes.first().ok_or(TransportError::InvalidRequest)?;
    let mut endpoints = Vec::new();
    for scope in scopes {
        if first.proxy() != scope.proxy()
            || first.private_addresses() != scope.private_addresses()
            || first.redirect_limit() != scope.redirect_limit()
        {
            return Err(TransportError::InvalidRequest);
        }
        endpoints.extend_from_slice(scope.endpoints());
    }
    ToolNetworkScope::new_configuration(
        endpoints,
        first.proxy().cloned(),
        first.private_addresses().to_vec(),
        first.redirect_limit(),
    )
    .map_err(|_| TransportError::InvalidRequest)
}

#[derive(Debug, Error)]
pub(crate) enum SearchCompositionConfigurationError {
    #[error(transparent)]
    Selection(#[from] SearchBackendSelectionError),
    #[error(transparent)]
    Transport(#[from] TransportError),
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
    Tavily(Box<PreparedTavily>),
    SearXng(Box<PreparedSearXng>),
}

impl PreparedSearchCandidate {
    fn id(&self) -> SearchBackendId {
        match self {
            Self::Direct(_) => SearchBackendId::DuckDuckGo,
            Self::Tavily(_) => SearchBackendId::Tavily,
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
            PreparedSearchCandidate::Tavily(plan) => (*plan)
                .execute(self.authority, self.budget)
                .await
                .map(|rows| rows.into_results())
                .map_err(SearchCandidateExecutionError::Tavily),
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
    Tavily(TavilyError),
    #[error(transparent)]
    SearXng(SearXngError),
}

impl SearchCandidateExecutionError {
    /// Collapse controlled transport/parser details into the existing private backend taxonomy.
    /// Router deadline/cancellation still win as enclosing control outcomes.
    pub(crate) fn into_backend_error(self) -> SearchBackendError {
        match self {
            Self::Compatibility(error) => compatibility_plan_backend_error(error),
            Self::Tavily(error) => tavily_backend_error(error),
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
        CompatibilityError::Unreachable => SearchBackendError::Unreachable,
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
        CompatibilityPlanError::Wikipedia(error) => compatibility_backend_error(error),
    }
}

fn tavily_backend_error(error: TavilyError) -> SearchBackendError {
    match error {
        TavilyError::Transport(error) => transport_backend_error(error),
        TavilyError::Admission(error) => compatibility_backend_error(error),
        TavilyError::Auth => SearchBackendError::Auth,
        TavilyError::RateLimited => SearchBackendError::RateLimited,
        TavilyError::Status => SearchBackendError::RequestFailed("tavily status failure".into()),
        TavilyError::Parse | TavilyError::Limit => SearchBackendError::Parse,
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

    fn running_candidates(configuration: SearchCompositionConfiguration) -> Vec<SearchBackendId> {
        configuration
            .prepare("private query", 2, limits())
            .expect("prepared")
            .start(Arc::new(RejectAuthority))
            .candidates()
            .iter()
            .map(RunningSearchCandidate::id)
            .collect()
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
        assert_eq!(first.permission_profile().len(), 7);
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
        assert_eq!(prepared.manifest().scope().endpoints().len(), 9);
        let running = prepared.start(Arc::new(RejectAuthority));
        assert_eq!(running.candidates().len(), 1);
        assert_eq!(running.candidates()[0].id(), SearchBackendId::DuckDuckGo);
    }

    #[test]
    fn auto_discards_optional_availability_before_scope_construction() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::Auto,
            Some(TavilyConfiguration::new("fixture-tavily-key").expect("tavily")),
            Some(SearXngConfiguration::new("https://search.example/").expect("searxng")),
        )
        .expect("auto config");
        let prepared = configuration
            .prepare("private query", 2, limits())
            .expect("prepared");

        assert_eq!(prepared.manifest().scope().endpoints().len(), 9);
        let running = prepared.start(Arc::new(RejectAuthority));
        assert_eq!(running.candidates().len(), 1);
        assert_eq!(running.candidates()[0].id(), SearchBackendId::DuckDuckGo);
    }

    #[test]
    fn explicit_tavily_without_fallback_excludes_direct() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::explicit(SearchBackendId::Tavily),
            Some(TavilyConfiguration::new("fixture-tavily-key").expect("tavily")),
            None,
        )
        .expect("explicit config");
        assert_eq!(
            running_candidates(configuration),
            vec![SearchBackendId::Tavily]
        );
    }

    #[test]
    fn explicit_tavily_auto_fallback_is_override_then_direct() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Tavily),
            Some(TavilyConfiguration::new("fixture-tavily-key").expect("tavily")),
            None,
        )
        .expect("explicit config");
        assert_eq!(
            running_candidates(configuration),
            vec![SearchBackendId::Tavily, SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn explicit_searxng_without_fallback_excludes_direct() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::explicit(SearchBackendId::SearXng),
            None,
            Some(SearXngConfiguration::new("https://search.example/").expect("searxng")),
        )
        .expect("explicit config");
        assert_eq!(
            running_candidates(configuration),
            vec![SearchBackendId::SearXng]
        );
    }

    #[test]
    fn explicit_searxng_auto_fallback_is_override_then_direct() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::SearXng),
            None,
            Some(SearXngConfiguration::new("https://search.example/").expect("searxng")),
        )
        .expect("explicit config");
        assert_eq!(
            running_candidates(configuration),
            vec![SearchBackendId::SearXng, SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn explicit_direct_auto_fallback_does_not_duplicate_candidate() {
        let configuration = SearchCompositionConfiguration::selected(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::DuckDuckGo),
            None,
            None,
        )
        .expect("explicit direct config");
        assert_eq!(
            running_candidates(configuration),
            vec![SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn selected_candidates_share_one_proxy_snapshot_in_scope_and_facets() {
        for (selection, tavily, searxng, expected_candidates) in [
            (
                SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Tavily),
                Some(TavilyConfiguration::new("fixture-tavily-key").expect("tavily")),
                None,
                vec![SearchBackendId::Tavily, SearchBackendId::DuckDuckGo],
            ),
            (
                SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::SearXng),
                None,
                Some(SearXngConfiguration::new("https://search.example/").expect("searxng")),
                vec![SearchBackendId::SearXng, SearchBackendId::DuckDuckGo],
            ),
        ] {
            let configuration =
                SearchCompositionConfiguration::selected(selection, tavily, searxng)
                    .expect("selected config");
            let secret = "proxy-fixture-secret";
            let proxy = TransportProxy::from_snapshot(
                &format!("http://fixture-user:{secret}@proxy.example:8080"),
                "",
                configuration.scope().endpoints(),
            )
            .expect("proxy snapshot")
            .expect("proxy applies");
            let proxy_endpoint = proxy.endpoint().clone();
            let configuration = configuration
                .with_proxy(Some(proxy))
                .expect("shared proxy config");
            assert_eq!(configuration.scope().proxy(), Some(&proxy_endpoint));
            assert!(!format!("{configuration:?}").contains(secret));

            let prepared = configuration
                .prepare("private query", 2, limits())
                .expect("prepared with shared proxy");
            assert_eq!(prepared.manifest().scope().proxy(), Some(&proxy_endpoint));
            assert!(
                prepared
                    .permission_profile()
                    .iter()
                    .any(|facet| facet.resource.as_deref() == Some("proxy.example"))
            );
            assert!(!format!("{prepared:?}").contains(secret));
            let running = prepared.start(Arc::new(RejectAuthority));
            assert_eq!(
                running
                    .candidates()
                    .iter()
                    .map(RunningSearchCandidate::id)
                    .collect::<Vec<_>>(),
                expected_candidates
            );
        }
    }

    #[test]
    fn explicit_missing_configuration_fails_before_scope_construction() {
        assert!(matches!(
            SearchCompositionConfiguration::selected(
                SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Tavily),
                None,
                None,
            ),
            Err(SearchCompositionConfigurationError::Selection(
                SearchBackendSelectionError::Unavailable(SearchBackendId::Tavily)
            ))
        ));
    }

    #[test]
    fn execution_errors_map_without_guessing_router_control() {
        assert_eq!(
            SearchCandidateExecutionError::Tavily(TavilyError::Auth).into_backend_error(),
            SearchBackendError::Auth
        );
        assert_eq!(
            SearchCandidateExecutionError::Tavily(TavilyError::RateLimited).into_backend_error(),
            SearchBackendError::RateLimited
        );
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
