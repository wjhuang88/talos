//! Private compatibility boundary for web-search backends.
//!
//! This module deliberately contains only typed contracts. The existing
//! `WebSearchTool` remains the compatibility composition until SEARCH-001-C
//! introduces Talos-owned routing behavior.

use async_trait::async_trait;
use thiserror::Error;

/// Backend identity retained across normalization and routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchBackendId {
    /// DuckDuckGo through the rust-websearch compatibility adapter.
    DuckDuckGo,
    /// Explicitly configured Tavily adapter.
    Tavily,
    /// Explicitly configured SearXNG adapter.
    SearXng,
    /// Wikipedia compatibility fallback.
    Wikipedia,
}

/// Backend request shared by all compatibility adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchBackendRequest {
    /// Query text passed to the backend.
    pub(crate) query: String,
    /// Maximum number of normalized results.
    pub(crate) max_results: u32,
}

/// One normalized backend result before model-facing formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchBackendResult {
    /// Backend that produced this result.
    pub(crate) backend: SearchBackendId,
    /// Result title.
    pub(crate) title: String,
    /// Canonical result URL.
    pub(crate) url: String,
    /// Optional backend snippet.
    pub(crate) snippet: String,
}

/// Typed failure at the compatibility boundary.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum SearchBackendError {
    /// Backend was not enabled by explicit configuration.
    #[error("backend is not configured")]
    NotConfigured,
    /// Backend response could not be parsed into the normalized shape.
    #[error("invalid backend response: {0}")]
    InvalidResponse(String),
    /// Transport or backend failure retained for diagnostics.
    #[error("backend request failed: {0}")]
    RequestFailed(String),
}

/// Private adapter contract used by the compatibility composition.
#[async_trait]
pub(crate) trait SearchBackend: Send + Sync {
    /// Identify the adapter without exposing platform Provider concepts.
    fn id(&self) -> SearchBackendId;

    /// Execute a bounded request and return normalized results.
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
        let results = vec![
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
    fn typed_failures_distinguish_configuration_and_response() {
        assert_ne!(
            SearchBackendError::NotConfigured,
            SearchBackendError::InvalidResponse("".into())
        );
        assert_eq!(
            SearchBackendError::RequestFailed("timeout".into()).to_string(),
            "backend request failed: timeout"
        );
    }
}
