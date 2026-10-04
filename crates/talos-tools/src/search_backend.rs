//! Private compatibility boundary for web-search backends.
//!
//! This module deliberately contains only typed contracts. The existing
//! `WebSearchTool` remains the compatibility composition until SEARCH-001-C
//! introduces Talos-owned routing behavior.
//!
//! The seam is intentionally staged before adapter wiring; the allowlist keeps this
//! intermediate contract from becoming a production behavior change.

#![allow(dead_code)]

use async_trait::async_trait;
use thiserror::Error;

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
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum SearchBackendError {
    #[error("backend is not configured")]
    NotConfigured,
    #[error("invalid backend response: {0}")]
    InvalidResponse(String),
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
