//! Dormant bounded query-free health state for SearchRouter selection.
//!
//! This private module stores only backend identity, content-free coarse outcomes, latency and
//! monotonic observation/expiry times. It is not wired into production routing or background
//! probing; callers must provide explicit capacity and TTL bounds.

use std::collections::VecDeque;
use std::time::Duration;

use thiserror::Error;
use tokio::time::Instant;

use super::router::SearchRouterFailureKind;
use super::{SearchBackendError, SearchBackendId};

/// Content-free health signal suitable for bounded routing observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchHealthSignal {
    Success,
    NoResults,
    InvalidResults,
    NotConfigured,
    Unreachable,
    Timeout,
    RateLimited,
    HttpStatus,
    Auth,
    Parse,
    RequestFailed,
}

impl SearchHealthSignal {
    /// Endpoint-availability interpretation without turning query-specific absence into outage.
    pub(crate) fn availability_effect(self) -> SearchHealthAvailability {
        match self {
            Self::Success => SearchHealthAvailability::Healthy,
            Self::NoResults => SearchHealthAvailability::Neutral,
            Self::NotConfigured | Self::Auth => SearchHealthAvailability::Unavailable,
            Self::InvalidResults
            | Self::Unreachable
            | Self::Timeout
            | Self::RateLimited
            | Self::HttpStatus
            | Self::Parse
            | Self::RequestFailed => SearchHealthAvailability::Degraded,
        }
    }

    pub(crate) fn from_failure(kind: &SearchRouterFailureKind) -> Self {
        match kind {
            SearchRouterFailureKind::NoResults => Self::NoResults,
            SearchRouterFailureKind::InvalidResults => Self::InvalidResults,
            SearchRouterFailureKind::Backend(error) => match error {
                SearchBackendError::NotConfigured => Self::NotConfigured,
                SearchBackendError::Unreachable => Self::Unreachable,
                SearchBackendError::Timeout => Self::Timeout,
                SearchBackendError::RateLimited => Self::RateLimited,
                SearchBackendError::HttpStatus(_) => Self::HttpStatus,
                SearchBackendError::Auth => Self::Auth,
                SearchBackendError::Parse => Self::Parse,
                SearchBackendError::NoResults => Self::NoResults,
                SearchBackendError::RequestFailed(_) => Self::RequestFailed,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchHealthAvailability {
    Healthy,
    Neutral,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SearchHealthObservation {
    pub(crate) signal: SearchHealthSignal,
    pub(crate) latency: Duration,
    pub(crate) observed_at: Instant,
    pub(crate) expires_at: Instant,
}

impl SearchHealthObservation {
    fn is_expired(self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchHealthPolicyError {
    #[error("search health capacity must be non-zero")]
    ZeroCapacity,
    #[error("search health ttl must be non-zero")]
    ZeroTtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SearchHealthEntry {
    backend: SearchBackendId,
    observation: SearchHealthObservation,
}

/// Explicitly bounded invocation-external health memory.
///
/// No query, result URL, response body, credential or raw error string can be stored by this type.
/// Recording an existing backend refreshes it as the newest observation. Recording a new backend
/// at capacity evicts the oldest retained observation. Expired evidence is removed lazily on use.
#[derive(Debug)]
pub(crate) struct SearchHealthState {
    capacity: usize,
    ttl: Duration,
    entries: VecDeque<SearchHealthEntry>,
}

impl SearchHealthState {
    pub(crate) fn new(capacity: usize, ttl: Duration) -> Result<Self, SearchHealthPolicyError> {
        if capacity == 0 {
            return Err(SearchHealthPolicyError::ZeroCapacity);
        }
        if ttl.is_zero() {
            return Err(SearchHealthPolicyError::ZeroTtl);
        }
        Ok(Self {
            capacity,
            ttl,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    pub(crate) fn record(
        &mut self,
        backend: SearchBackendId,
        signal: SearchHealthSignal,
        latency: Duration,
        observed_at: Instant,
    ) {
        self.purge_expired(observed_at);
        if let Some(index) = self.entries.iter().position(|entry| entry.backend == backend) {
            self.entries.remove(index);
        }
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(SearchHealthEntry {
            backend,
            observation: SearchHealthObservation {
                signal,
                latency,
                observed_at,
                expires_at: observed_at + self.ttl,
            },
        });
    }

    pub(crate) fn observation(
        &mut self,
        backend: SearchBackendId,
        now: Instant,
    ) -> Option<SearchHealthObservation> {
        self.purge_expired(now);
        self.entries
            .iter()
            .find(|entry| entry.backend == backend)
            .map(|entry| entry.observation)
    }

    pub(crate) fn len(&mut self, now: Instant) -> usize {
        self.purge_expired(now);
        self.entries.len()
    }

    pub(crate) fn is_empty(&mut self, now: Instant) -> bool {
        self.purge_expired(now);
        self.entries.is_empty()
    }

    fn purge_expired(&mut self, now: Instant) {
        self.entries
            .retain(|entry| !entry.observation.is_expired(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_rejects_unbounded_zero_values() {
        assert_eq!(
            SearchHealthState::new(0, Duration::from_secs(1)).unwrap_err(),
            SearchHealthPolicyError::ZeroCapacity
        );
        assert_eq!(
            SearchHealthState::new(1, Duration::ZERO).unwrap_err(),
            SearchHealthPolicyError::ZeroTtl
        );
    }

    #[test]
    fn state_evicts_oldest_observation_at_explicit_capacity() {
        let now = Instant::now();
        let mut state = SearchHealthState::new(2, Duration::from_secs(60)).expect("health state");
        state.record(
            SearchBackendId::DuckDuckGo,
            SearchHealthSignal::Success,
            Duration::from_millis(10),
            now,
        );
        state.record(
            SearchBackendId::SearXng,
            SearchHealthSignal::Timeout,
            Duration::from_millis(20),
            now + Duration::from_millis(1),
        );
        state.record(
            SearchBackendId::Wikipedia,
            SearchHealthSignal::Parse,
            Duration::from_millis(30),
            now + Duration::from_millis(2),
        );

        assert_eq!(state.len(now + Duration::from_millis(2)), 2);
        assert!(
            state
                .observation(SearchBackendId::DuckDuckGo, now + Duration::from_millis(2))
                .is_none()
        );
        assert!(
            state
                .observation(SearchBackendId::SearXng, now + Duration::from_millis(2))
                .is_some()
        );
        assert!(
            state
                .observation(SearchBackendId::Wikipedia, now + Duration::from_millis(2))
                .is_some()
        );
    }

    #[test]
    fn refresh_replaces_existing_backend_without_growing_state() {
        let now = Instant::now();
        let mut state = SearchHealthState::new(2, Duration::from_secs(60)).expect("health state");
        state.record(
            SearchBackendId::DuckDuckGo,
            SearchHealthSignal::Timeout,
            Duration::from_millis(50),
            now,
        );
        state.record(
            SearchBackendId::DuckDuckGo,
            SearchHealthSignal::Success,
            Duration::from_millis(5),
            now + Duration::from_secs(1),
        );

        assert_eq!(state.len(now + Duration::from_secs(1)), 1);
        let observation = state
            .observation(SearchBackendId::DuckDuckGo, now + Duration::from_secs(1))
            .expect("observation");
        assert_eq!(observation.signal, SearchHealthSignal::Success);
        assert_eq!(observation.latency, Duration::from_millis(5));
        assert_eq!(observation.observed_at, now + Duration::from_secs(1));
    }

    #[test]
    fn expired_evidence_returns_to_unknown_state() {
        let now = Instant::now();
        let mut state = SearchHealthState::new(2, Duration::from_secs(5)).expect("health state");
        state.record(
            SearchBackendId::DuckDuckGo,
            SearchHealthSignal::Unreachable,
            Duration::from_millis(50),
            now,
        );

        assert!(
            state
                .observation(SearchBackendId::DuckDuckGo, now + Duration::from_secs(4))
                .is_some()
        );
        assert!(
            state
                .observation(SearchBackendId::DuckDuckGo, now + Duration::from_secs(5))
                .is_none()
        );
        assert!(state.is_empty(now + Duration::from_secs(5)));
    }

    #[test]
    fn no_results_remains_neutral_instead_of_endpoint_outage() {
        assert_eq!(
            SearchHealthSignal::NoResults.availability_effect(),
            SearchHealthAvailability::Neutral
        );
        assert_eq!(
            SearchHealthSignal::Unreachable.availability_effect(),
            SearchHealthAvailability::Degraded
        );
    }

    #[test]
    fn failure_normalization_drops_raw_error_and_status_details() {
        let secret = "opaque private backend detail";
        let opaque = SearchRouterFailureKind::Backend(SearchBackendError::RequestFailed(
            secret.to_owned(),
        ));
        let status = SearchRouterFailureKind::Backend(SearchBackendError::HttpStatus(503));

        let opaque_signal = SearchHealthSignal::from_failure(&opaque);
        let status_signal = SearchHealthSignal::from_failure(&status);
        assert_eq!(opaque_signal, SearchHealthSignal::RequestFailed);
        assert_eq!(status_signal, SearchHealthSignal::HttpStatus);
        assert!(!format!("{opaque_signal:?}").contains(secret));
        assert!(!format!("{status_signal:?}").contains("503"));
    }
}
