//! Dormant Talos-owned SearchRouter core.
//!
//! This private module implements first-valid-success routing over explicit `SearchBackend`
//! instances and one-shot prepared candidates. It is not wired into `WebSearchTool`, the tool
//! registry, or any product default.

use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use reqwest::Url;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep, sleep_until};

use super::composition::RunningSearchComposition;
use super::health::{SearchHealthSignal, SearchHealthState};
use super::{
    SearchBackend, SearchBackendError, SearchBackendId, SearchBackendRequest, SearchBackendResult,
};

const MAX_FIELD_BYTES: usize = 16_384;

type CandidateFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<SearchBackendResult>, SearchBackendError>> + Send + 'a>>;

struct RouterCandidate<'a> {
    id: SearchBackendId,
    future: CandidateFuture<'a>,
}

struct ActiveCandidate<'a> {
    id: SearchBackendId,
    started_at: Instant,
    future: CandidateFuture<'a>,
}

/// Explicit private routing bounds. No production defaults are selected by this stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SearchRouterPolicy {
    hedge_delay: Duration,
    max_in_flight: usize,
    max_examined_results: usize,
    max_url_bytes: usize,
}

impl SearchRouterPolicy {
    pub(crate) fn new(
        hedge_delay: Duration,
        max_in_flight: usize,
        max_examined_results: usize,
        max_url_bytes: usize,
    ) -> Result<Self, SearchRouterPolicyError> {
        if max_in_flight == 0 || max_examined_results == 0 || max_url_bytes == 0 {
            return Err(SearchRouterPolicyError::ZeroBound);
        }
        Ok(Self {
            hedge_delay,
            max_in_flight,
            max_examined_results,
            max_url_bytes,
        })
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchRouterPolicyError {
    #[error("search router bounds must be non-zero")]
    ZeroBound,
    #[error("duplicate search backend: {0:?}")]
    DuplicateBackend(SearchBackendId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchRouterSuccess {
    pub(crate) backend: SearchBackendId,
    pub(crate) results: Vec<SearchBackendResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchRouterFailure {
    pub(crate) backend: SearchBackendId,
    pub(crate) kind: SearchRouterFailureKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchRouterFailureKind {
    Backend(SearchBackendError),
    NoResults,
    InvalidResults,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum SearchRouterError {
    #[error("search request is outside router bounds")]
    InvalidRequest,
    #[error("no eligible search backend")]
    NoEligible,
    #[error("all eligible search backends were exhausted")]
    Exhausted(Vec<SearchRouterFailure>),
    #[error("search router deadline expired")]
    Deadline,
    #[error("search router was cancelled")]
    Cancelled,
}

/// Private router over one explicit ordered reusable candidate set.
///
/// The lifetime is intentional: existing compatibility adapters borrow `WebSearchTool`. The
/// router owns an `Arc` to each adapter for one invocation without requiring those adapters to be
/// `'static` or spawning detached work.
pub(crate) struct SearchRouter<'a> {
    candidates: Vec<Arc<dyn SearchBackend + 'a>>,
    policy: SearchRouterPolicy,
}

impl std::fmt::Debug for SearchRouter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchRouter")
            .field(
                "candidates",
                &self
                    .candidates
                    .iter()
                    .map(|backend| backend.id())
                    .collect::<Vec<_>>(),
            )
            .field("policy", &self.policy)
            .finish()
    }
}

impl<'a> SearchRouter<'a> {
    pub(crate) fn new(
        candidates: Vec<Arc<dyn SearchBackend + 'a>>,
        policy: SearchRouterPolicy,
    ) -> Result<Self, SearchRouterPolicyError> {
        for (index, backend) in candidates.iter().enumerate() {
            if candidates[..index]
                .iter()
                .any(|existing| existing.id() == backend.id())
            {
                return Err(SearchRouterPolicyError::DuplicateBackend(backend.id()));
            }
        }
        Ok(Self { candidates, policy })
    }

    /// Routes one already-eligible reusable-backend request without health-based ordering.
    pub(crate) async fn search(
        &self,
        request: SearchBackendRequest,
        deadline: Instant,
        cancellation: &mut oneshot::Receiver<()>,
    ) -> Result<SearchRouterSuccess, SearchRouterError> {
        let requested = request.max_results as usize;
        let candidates = self.candidates_for(&request);
        route_candidates(
            self.policy,
            candidates,
            requested,
            deadline,
            cancellation,
            None,
        )
        .await
    }

    /// Routes one already-eligible reusable-backend request using explicit bounded health memory.
    pub(crate) async fn search_with_health(
        &self,
        request: SearchBackendRequest,
        deadline: Instant,
        cancellation: &mut oneshot::Receiver<()>,
        health: &mut SearchHealthState,
    ) -> Result<SearchRouterSuccess, SearchRouterError> {
        let requested = request.max_results as usize;
        let candidates = self.candidates_for(&request);
        route_candidates(
            self.policy,
            candidates,
            requested,
            deadline,
            cancellation,
            Some(health),
        )
        .await
    }

    fn candidates_for(&self, request: &SearchBackendRequest) -> Vec<RouterCandidate<'a>> {
        self.candidates
            .iter()
            .cloned()
            .map(|backend| {
                let id = backend.id();
                let candidate_request = request.clone();
                RouterCandidate {
                    id,
                    future: Box::pin(async move { backend.search(candidate_request).await }),
                }
            })
            .collect()
    }
}

/// Routes one already-running prepared composition using its frozen result bound and the exact
/// deadline of its shared transport budget. No caller can replace the query, count or deadline.
pub(crate) async fn search_prepared(
    composition: RunningSearchComposition,
    policy: SearchRouterPolicy,
    cancellation: &mut oneshot::Receiver<()>,
) -> Result<SearchRouterSuccess, SearchRouterError> {
    let requested = composition.requested_results();
    let deadline = composition.deadline();
    let candidates = composition
        .into_candidates()
        .into_iter()
        .map(|candidate| {
            let id = candidate.id();
            RouterCandidate {
                id,
                future: Box::pin(async move {
                    candidate
                        .execute()
                        .await
                        .map_err(|error| error.into_backend_error())
                }),
            }
        })
        .collect();
    route_candidates(policy, candidates, requested, deadline, cancellation, None).await
}

/// Health-aware variant of the dormant prepared path. No production caller uses this stage.
pub(crate) async fn search_prepared_with_health(
    composition: RunningSearchComposition,
    policy: SearchRouterPolicy,
    cancellation: &mut oneshot::Receiver<()>,
    health: &mut SearchHealthState,
) -> Result<SearchRouterSuccess, SearchRouterError> {
    let requested = composition.requested_results();
    let deadline = composition.deadline();
    let candidates = composition
        .into_candidates()
        .into_iter()
        .map(|candidate| {
            let id = candidate.id();
            RouterCandidate {
                id,
                future: Box::pin(async move {
                    candidate
                        .execute()
                        .await
                        .map_err(|error| error.into_backend_error())
                }),
            }
        })
        .collect();
    route_candidates(
        policy,
        candidates,
        requested,
        deadline,
        cancellation,
        Some(health),
    )
    .await
}

/// Shared first-valid-success scheduler for reusable and one-shot prepared candidates.
/// Closing the cancellation sender is cancellation; dropping this future drops all owned futures.
async fn route_candidates<'a>(
    policy: SearchRouterPolicy,
    mut candidates: Vec<RouterCandidate<'a>>,
    requested: usize,
    deadline: Instant,
    cancellation: &mut oneshot::Receiver<()>,
    mut health: Option<&mut SearchHealthState>,
) -> Result<SearchRouterSuccess, SearchRouterError> {
    if requested == 0 || requested > policy.max_examined_results {
        return Err(SearchRouterError::InvalidRequest);
    }

    if let Some(error) = control(deadline, cancellation) {
        return Err(error);
    }

    if let Some(state) = health.as_mut() {
        candidates = order_candidates_by_health(candidates, state, Instant::now());
    }

    let mut remaining = candidates.into_iter();
    let mut active: Vec<ActiveCandidate<'a>> = Vec::new();
    let mut failures = Vec::new();
    let mut launch_next = true;
    let hedge = sleep(policy.hedge_delay);
    tokio::pin!(hedge);

    loop {
        if let Some(error) = control(deadline, cancellation) {
            return Err(error);
        }

        if (launch_next || active.is_empty()) && active.len() < policy.max_in_flight {
            launch_next = false;
            if let Some(candidate) = remaining.next() {
                active.push(ActiveCandidate {
                    id: candidate.id,
                    started_at: Instant::now(),
                    future: candidate.future,
                });
                hedge.as_mut().reset(Instant::now() + policy.hedge_delay);
            } else if active.is_empty() {
                return if failures.is_empty() {
                    Err(SearchRouterError::NoEligible)
                } else {
                    Err(SearchRouterError::Exhausted(failures))
                };
            }
        }

        tokio::select! {
            biased;
            _ = &mut *cancellation => return Err(SearchRouterError::Cancelled),
            _ = sleep_until(deadline) => return Err(SearchRouterError::Deadline),
            (index, outcome) = poll_fn(|cx| {
                for (index, candidate) in active.iter_mut().enumerate() {
                    if let Poll::Ready(outcome) = candidate.future.as_mut().poll(cx) {
                        return Poll::Ready((index, outcome));
                    }
                }
                Poll::Pending
            }), if !active.is_empty() => {
                let ActiveCandidate {
                    id: backend,
                    started_at,
                    future: completed,
                } = active.remove(index);
                drop(completed);
                match outcome {
                    Err(error) => {
                        let kind = SearchRouterFailureKind::Backend(error);
                        record_health_failure(&mut health, backend, &kind, started_at);
                        failures.push(SearchRouterFailure { backend, kind });
                    }
                    Ok(results) => match admit(
                        policy,
                        backend,
                        results,
                        requested,
                        deadline,
                        cancellation,
                    ) {
                        Ok(results) => {
                            if let Some(error) = control(deadline, cancellation) {
                                return Err(error);
                            }
                            record_health_signal(
                                &mut health,
                                backend,
                                SearchHealthSignal::Success,
                                started_at,
                            );
                            return Ok(SearchRouterSuccess { backend, results });
                        }
                        Err(AdmissionError::NoResults) => {
                            let kind = SearchRouterFailureKind::NoResults;
                            record_health_failure(&mut health, backend, &kind, started_at);
                            failures.push(SearchRouterFailure { backend, kind });
                        }
                        Err(AdmissionError::InvalidResults) => {
                            let kind = SearchRouterFailureKind::InvalidResults;
                            record_health_failure(&mut health, backend, &kind, started_at);
                            failures.push(SearchRouterFailure { backend, kind });
                        }
                        Err(AdmissionError::Control(error)) => return Err(error),
                    },
                }
                if remaining.len() > 0 {
                    launch_next = true;
                }
            }
            _ = &mut hedge,
                if active.len() < policy.max_in_flight && remaining.len() > 0 =>
            {
                launch_next = true;
            }
        }
    }
}

fn order_candidates_by_health<'a>(
    candidates: Vec<RouterCandidate<'a>>,
    health: &mut SearchHealthState,
    now: Instant,
) -> Vec<RouterCandidate<'a>> {
    let mut ranked = candidates
        .into_iter()
        .enumerate()
        .map(|(position, candidate)| {
            (
                health.selection_rank(candidate.id, now),
                position,
                candidate,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(rank, position, _)| (*rank, *position));
    ranked
        .into_iter()
        .map(|(_, _, candidate)| candidate)
        .collect()
}

fn record_health_failure(
    health: &mut Option<&mut SearchHealthState>,
    backend: SearchBackendId,
    kind: &SearchRouterFailureKind,
    started_at: Instant,
) {
    record_health_signal(
        health,
        backend,
        SearchHealthSignal::from_failure(kind),
        started_at,
    );
}

fn record_health_signal(
    health: &mut Option<&mut SearchHealthState>,
    backend: SearchBackendId,
    signal: SearchHealthSignal,
    started_at: Instant,
) {
    let observed_at = Instant::now();
    if let Some(state) = health.as_mut() {
        state.record(
            backend,
            signal,
            observed_at.duration_since(started_at),
            observed_at,
        );
    }
}

fn admit(
    policy: SearchRouterPolicy,
    backend: SearchBackendId,
    results: Vec<SearchBackendResult>,
    requested: usize,
    deadline: Instant,
    cancellation: &mut oneshot::Receiver<()>,
) -> Result<Vec<SearchBackendResult>, AdmissionError> {
    if let Some(error) = control(deadline, cancellation) {
        return Err(AdmissionError::Control(error));
    }
    if results.is_empty() {
        return Err(AdmissionError::NoResults);
    }
    let result_backend = results.first().map(|result| result.backend);
    // The prepared direct candidate owns Wikipedia only as its migration-time last resort.
    let result_identity_allowed = result_backend.is_some_and(|result_backend| {
        result_backend == backend
            || (backend == SearchBackendId::DuckDuckGo
                && result_backend == SearchBackendId::Wikipedia)
    }) && results
        .iter()
        .all(|result| Some(result.backend) == result_backend);
    if results.len() > policy.max_examined_results
        || !result_identity_allowed
        || results.iter().any(|result| {
            result.title.len() > MAX_FIELD_BYTES
                || result.snippet.len() > MAX_FIELD_BYTES
                || result.url.len() > policy.max_url_bytes
        })
    {
        return Err(AdmissionError::InvalidResults);
    }

    let mut admitted = Vec::new();
    for result in results {
        if let Some(error) = control(deadline, cancellation) {
            return Err(AdmissionError::Control(error));
        }
        let valid = std::panic::catch_unwind(|| {
            Url::parse(&result.url).is_ok_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
            })
        })
        .map_err(|_| AdmissionError::InvalidResults)?;
        if let Some(error) = control(deadline, cancellation) {
            return Err(AdmissionError::Control(error));
        }
        if valid {
            admitted.push(result);
            if admitted.len() == requested {
                break;
            }
        }
    }

    if admitted.is_empty() {
        Err(AdmissionError::InvalidResults)
    } else {
        Ok(admitted)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AdmissionError {
    NoResults,
    InvalidResults,
    Control(SearchRouterError),
}

fn control(
    deadline: Instant,
    cancellation: &mut oneshot::Receiver<()>,
) -> Option<SearchRouterError> {
    if Instant::now() >= deadline {
        return Some(SearchRouterError::Deadline);
    }
    match cancellation.try_recv() {
        Ok(()) | Err(oneshot::error::TryRecvError::Closed) => Some(SearchRouterError::Cancelled),
        Err(oneshot::error::TryRecvError::Empty) => None,
    }
}

#[cfg(test)]
mod health_tests;
#[cfg(test)]
mod prepared_tests;
#[cfg(test)]
mod tests;
