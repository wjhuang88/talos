//! Dormant Talos-owned SearchRouter core.
//!
//! This private module implements first-valid-success routing over explicit `SearchBackend`
//! instances. It is not wired into `WebSearchTool`, the tool registry, or any product default.

use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use reqwest::Url;
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep, sleep_until};

use super::{
    SearchBackend, SearchBackendError, SearchBackendId, SearchBackendRequest, SearchBackendResult,
};

type CandidateFuture =
    Pin<Box<dyn Future<Output = Result<Vec<SearchBackendResult>, SearchBackendError>> + Send>>;

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

/// Private router over one explicit ordered candidate set.
pub(crate) struct SearchRouter {
    candidates: Vec<Arc<dyn SearchBackend>>,
    policy: SearchRouterPolicy,
}

impl std::fmt::Debug for SearchRouter {
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

impl SearchRouter {
    pub(crate) fn new(
        candidates: Vec<Arc<dyn SearchBackend>>,
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

    /// Routes one already-eligible request without spawning detached work.
    ///
    /// The enclosing deadline is shared by all candidates and admission work. Closing the
    /// cancellation sender is treated as cancellation. Dropping this future drops all owned
    /// candidate futures; transport-specific spawned work remains a separate integration gate.
    pub(crate) async fn search(
        &self,
        request: SearchBackendRequest,
        deadline: Instant,
        cancellation: &mut oneshot::Receiver<()>,
    ) -> Result<SearchRouterSuccess, SearchRouterError> {
        let requested = request.max_results as usize;
        if requested == 0 || requested > self.policy.max_examined_results {
            return Err(SearchRouterError::InvalidRequest);
        }

        if let Some(error) = control(deadline, cancellation) {
            return Err(error);
        }

        let mut remaining = self.candidates.iter().cloned();
        let mut active: Vec<(SearchBackendId, CandidateFuture)> = Vec::new();
        let mut failures = Vec::new();
        let mut launch_next = true;
        let hedge = sleep(self.policy.hedge_delay);
        tokio::pin!(hedge);

        loop {
            if let Some(error) = control(deadline, cancellation) {
                return Err(error);
            }

            if (launch_next || active.is_empty()) && active.len() < self.policy.max_in_flight {
                launch_next = false;
                if let Some(backend) = remaining.next() {
                    let backend_id = backend.id();
                    let candidate_request = request.clone();
                    active.push((
                        backend_id,
                        Box::pin(async move { backend.search(candidate_request).await }),
                    ));
                    hedge
                        .as_mut()
                        .reset(Instant::now() + self.policy.hedge_delay);
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
                    for (index, (_, future)) in active.iter_mut().enumerate() {
                        if let Poll::Ready(outcome) = future.as_mut().poll(cx) {
                            return Poll::Ready((index, outcome));
                        }
                    }
                    Poll::Pending
                }), if !active.is_empty() => {
                    let (backend, completed) = active.remove(index);
                    drop(completed);
                    match outcome {
                        Err(error) => failures.push(SearchRouterFailure {
                            backend,
                            kind: SearchRouterFailureKind::Backend(error),
                        }),
                        Ok(results) => match self.admit(
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
                                return Ok(SearchRouterSuccess { backend, results });
                            }
                            Err(AdmissionError::NoResults) => failures.push(SearchRouterFailure {
                                backend,
                                kind: SearchRouterFailureKind::NoResults,
                            }),
                            Err(AdmissionError::InvalidResults) => failures.push(SearchRouterFailure {
                                backend,
                                kind: SearchRouterFailureKind::InvalidResults,
                            }),
                            Err(AdmissionError::Control(error)) => return Err(error),
                        },
                    }
                    if !remaining.as_slice().is_empty() {
                        launch_next = true;
                    }
                }
                _ = &mut hedge,
                    if active.len() < self.policy.max_in_flight && !remaining.as_slice().is_empty() =>
                {
                    launch_next = true;
                }
            }
        }
    }

    fn admit(
        &self,
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
        if results.len() > self.policy.max_examined_results
            || results.iter().any(|result| result.backend != backend)
        {
            return Err(AdmissionError::InvalidResults);
        }

        let mut admitted = Vec::new();
        for result in results {
            if let Some(error) = control(deadline, cancellation) {
                return Err(AdmissionError::Control(error));
            }
            let valid = result.url.len() <= self.policy.max_url_bytes
                && Url::parse(&result.url).is_ok_and(|url| {
                    matches!(url.scheme(), "http" | "https")
                        && url.host_str().is_some()
                        && url.username().is_empty()
                        && url.password().is_none()
                });
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
mod tests;
