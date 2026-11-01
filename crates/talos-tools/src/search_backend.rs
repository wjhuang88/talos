//! Private compatibility boundary for web-search backends.
//!
//! Typed contracts and controlled compatibility infrastructure. Production `web_search` activation
//! is assembled only through the protected router factory; the legacy `WebSearchTool` remains in
//! the crate solely as the version/deployment rollback implementation and must never be a runtime
//! fallback from a protected invocation.

#![allow(dead_code)]

use async_trait::async_trait;
use thiserror::Error;

// Controlled transport infrastructure used by the protected compatibility route.
mod transport;
// Owner for one prepared manifest, candidate set and shared dispatch budget.
mod composition;
// Owned bridge from prepared composition into the core network invocation carrier.
mod prepared_invocation;
// Prepared-only AgentTool seam used by the protected production contribution.
mod router_tool;
// Talos-owned routing core.
mod router;
// Bounded query-free health memory; no background probe.
mod health;
// Query/secret-free eligibility seam; availability never implies optional-provider consent.
mod selection;
// Production zero-key composition and activation boundary.
mod production;
pub(crate) use production::production_search_router_tool;
// Explicit opt-in bridge for cross-crate caller-boundary tests; never built by default.
#[cfg(feature = "search-integration-test")]
pub(crate) mod integration_test;
// Controlled response parsing prerequisites.
mod bing_parser;
mod ddg_parser;
mod instant_answer;

/// Backend identity retained across normalization and routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchBackendId {
    DuckDuckGo,
    Tavily,
    SearXng,
    Wikipedia,
}

/// Backend request shared by all compatibility adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchBackendRequest {
    pub(crate) query: String,
    pub(crate) max_results: u32,
}

/// One normalized backend result before model-facing formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchBackendResult {
    pub(crate) backend: SearchBackendId,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) snippet: String,
}

/// Typed failure at the compatibility boundary.
///
/// This taxonomy records only evidence the adapter can establish directly. Generic retry,
/// backoff and breaker policy remains NET-001 ownership; caller cancellation/deadline remain
/// SearchRouter control outcomes rather than guessed backend failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum SearchBackendError {
    #[error("backend is not configured")]
    NotConfigured,
    #[error("backend is unreachable")]
    Unreachable,
    #[error("backend request timed out")]
    Timeout,
    #[error("backend rate limited the request")]
    RateLimited,
    #[error("backend returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("backend authentication failed")]
    Auth,
    #[error("backend response could not be parsed")]
    Parse,
    #[error("backend returned no results")]
    NoResults,
    #[error("backend request failed: {0}")]
    RequestFailed(String),
}

/// Private adapter contract used by the compatibility composition.
#[async_trait]
pub(crate) trait SearchBackend: Send + Sync {
    fn id(&self) -> SearchBackendId;
    async fn search(
        &self,
        request: SearchBackendRequest,
    ) -> Result<Vec<SearchBackendResult>, SearchBackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_fixture_preserves_backend_identity_and_bounds_shape() {
        let request = SearchBackendRequest {
            query: "rust async".into(),
            max_results: 2,
        };
        let results = [
            SearchBackendResult {
                backend: SearchBackendId::DuckDuckGo,
                title: "Rust".into(),
                url: "https://rust-lang.org".into(),
                snippet: "A language for reliable software".into(),
            },
            SearchBackendResult {
                backend: SearchBackendId::DuckDuckGo,
                title: "Tokio".into(),
                url: "https://tokio.rs".into(),
                snippet: String::new(),
            },
        ];
        assert_eq!(request.max_results, results.len() as u32);
        assert!(
            results
                .iter()
                .all(|result| result.backend == SearchBackendId::DuckDuckGo)
        );
        assert_eq!(results[0].url, "https://rust-lang.org");
    }

    #[test]
    fn typed_failures_keep_selection_relevant_classes_distinct() {
        let classes = [
            SearchBackendError::NotConfigured,
            SearchBackendError::Unreachable,
            SearchBackendError::Timeout,
            SearchBackendError::RateLimited,
            SearchBackendError::HttpStatus(503),
            SearchBackendError::Auth,
            SearchBackendError::Parse,
            SearchBackendError::NoResults,
            SearchBackendError::RequestFailed("opaque".into()),
        ];
        for (index, left) in classes.iter().enumerate() {
            for right in classes.iter().skip(index + 1) {
                assert_ne!(left, right);
            }
        }
        assert_eq!(
            SearchBackendError::RequestFailed("opaque".into()).to_string(),
            "backend request failed: opaque"
        );
    }
}
