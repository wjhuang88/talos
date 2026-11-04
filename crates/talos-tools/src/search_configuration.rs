//! Product-neutral explicit web-search configuration.
//!
//! This module carries user intent into the protected search composition without depending on
//! `talos-config`. Optional-provider availability is never selection evidence: callers must choose
//! a concrete enum variant before Tavily credentials or a SearXNG endpoint can be consumed.

use thiserror::Error;

/// Explicit product intent for the protected `web_search` contribution.
#[derive(Clone, PartialEq, Eq, Default)]
pub enum SearchToolConfiguration {
    /// Zero-key automatic search. Optional-provider configuration is ignored.
    #[default]
    Auto,
    /// Explicitly select the zero-key DuckDuckGo compatibility candidate.
    DuckDuckGo,
    /// Explicitly select Tavily using a caller-supplied credential.
    Tavily {
        /// Explicit Tavily bearer credential. Debug output is always redacted.
        api_key: String,
        /// After a selected Tavily candidate fails or yields no valid result, try Auto.
        fallback_to_auto: bool,
    },
    /// Explicitly select one caller-supplied SearXNG endpoint.
    SearXng {
        /// Explicit SearXNG base URL. Debug output is deliberately redacted.
        url: String,
        /// After a selected SearXNG candidate fails or yields no valid result, try Auto.
        fallback_to_auto: bool,
    },
}

impl std::fmt::Debug for SearchToolConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => f.write_str("SearchToolConfiguration::Auto"),
            Self::DuckDuckGo => f.write_str("SearchToolConfiguration::DuckDuckGo"),
            Self::Tavily {
                fallback_to_auto, ..
            } => f
                .debug_struct("SearchToolConfiguration::Tavily")
                .field("api_key", &"<redacted>")
                .field("fallback_to_auto", fallback_to_auto)
                .finish(),
            Self::SearXng {
                fallback_to_auto, ..
            } => f
                .debug_struct("SearchToolConfiguration::SearXng")
                .field("url", &"<redacted>")
                .field("fallback_to_auto", fallback_to_auto)
                .finish(),
        }
    }
}

/// Redacted construction failures for explicit protected-search configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SearchToolConfigurationError {
    #[error("invalid explicit Tavily search configuration")]
    InvalidTavily,
    #[error("invalid explicit SearXNG search configuration")]
    InvalidSearXng,
    #[error("search transport policy could not be frozen")]
    TransportPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero_key_auto() {
        assert_eq!(
            SearchToolConfiguration::default(),
            SearchToolConfiguration::Auto
        );
    }

    #[test]
    fn debug_redacts_explicit_provider_values() {
        let tavily_secret = "fixture-tavily-secret";
        let tavily = SearchToolConfiguration::Tavily {
            api_key: tavily_secret.into(),
            fallback_to_auto: true,
        };
        let tavily_debug = format!("{tavily:?}");
        assert!(!tavily_debug.contains(tavily_secret));
        assert!(tavily_debug.contains("fallback_to_auto"));

        let searxng_url = "https://private-search.example/internal";
        let searxng = SearchToolConfiguration::SearXng {
            url: searxng_url.into(),
            fallback_to_auto: false,
        };
        assert!(!format!("{searxng:?}").contains(searxng_url));
    }
}
