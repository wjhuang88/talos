//! Dormant query-independent search backend selection policy.
//!
//! Availability and selection intent are deliberately separate. A caller may establish that an
//! optional backend is available without selecting it. In particular, ambient premium credential
//! availability is never usage consent. This private seam stores no credentials, reads no
//! environment variables, defines no persisted configuration schema, and performs no I/O.

use thiserror::Error;

use super::SearchBackendId;

/// Query-independent availability facts established outside the selection policy.
///
/// The booleans describe whether the optional adapter can be constructed. They do not carry a key,
/// endpoint, query, or any other sensitive/provider-specific configuration value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SearchBackendAvailability {
    tavily: bool,
    searxng: bool,
}

impl SearchBackendAvailability {
    pub(crate) const fn new(tavily: bool, searxng: bool) -> Self {
        Self { tavily, searxng }
    }

    const fn contains(self, backend: SearchBackendId) -> bool {
        match backend {
            SearchBackendId::DuckDuckGo => true,
            SearchBackendId::Tavily => self.tavily,
            SearchBackendId::SearXng => self.searxng,
            SearchBackendId::Wikipedia => false,
        }
    }
}

/// Explicit policy for what may happen after an already-selected override fails at execution time.
///
/// This value never repairs missing or invalid override configuration. Selection must first resolve
/// successfully; only then may `Auto` appear as the next ordered candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchBackendFallback {
    /// Return the selected override failure without trying the zero-key automatic route.
    Disabled,
    /// After the selected override fails, allow exactly the normal automatic route as fallback.
    Auto,
}

/// Selection intent supplied independently from backend discovery/availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchBackendSelection {
    /// Use only the currently admitted zero-key automatic route set.
    Auto,
    /// Select one reviewed backend identity explicitly and freeze its failure fallback policy.
    Explicit {
        backend: SearchBackendId,
        fallback: SearchBackendFallback,
    },
}

impl SearchBackendSelection {
    pub(crate) const fn explicit(backend: SearchBackendId) -> Self {
        Self::Explicit {
            backend,
            fallback: SearchBackendFallback::Disabled,
        }
    }

    pub(crate) const fn explicit_with_auto_fallback(backend: SearchBackendId) -> Self {
        Self::Explicit {
            backend,
            fallback: SearchBackendFallback::Auto,
        }
    }
}

/// Typed refusal before any backend is prepared or executed.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchBackendSelectionError {
    #[error("selected search backend is unavailable: {0:?}")]
    Unavailable(SearchBackendId),
    #[error("search backend is not eligible for generic selection: {0:?}")]
    Unsupported(SearchBackendId),
}

impl SearchBackendSelection {
    /// Resolve intent into the exact ordered candidate identities permitted for this invocation.
    ///
    /// `Auto` intentionally ignores optional availability. This is the key boundary that prevents
    /// a discovered Tavily credential or SearXNG endpoint from silently opting the user into an
    /// override. Explicit fallback is added only after the requested override is proven available;
    /// therefore missing/invalid override configuration never silently degrades to Auto.
    pub(crate) fn resolve(
        self,
        availability: SearchBackendAvailability,
    ) -> Result<Vec<SearchBackendId>, SearchBackendSelectionError> {
        match self {
            Self::Auto => Ok(vec![SearchBackendId::DuckDuckGo]),
            Self::Explicit {
                backend: SearchBackendId::Wikipedia,
                ..
            } => Err(SearchBackendSelectionError::Unsupported(
                SearchBackendId::Wikipedia,
            )),
            Self::Explicit { backend, .. } if !availability.contains(backend) => {
                Err(SearchBackendSelectionError::Unavailable(backend))
            }
            Self::Explicit {
                backend: SearchBackendId::DuckDuckGo,
                ..
            } => Ok(vec![SearchBackendId::DuckDuckGo]),
            Self::Explicit {
                backend,
                fallback: SearchBackendFallback::Disabled,
            } => Ok(vec![backend]),
            Self::Explicit {
                backend,
                fallback: SearchBackendFallback::Auto,
            } => Ok(vec![backend, SearchBackendId::DuckDuckGo]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambient_optional_availability_never_changes_auto_selection() {
        let availability = SearchBackendAvailability::new(true, true);

        assert_eq!(
            SearchBackendSelection::Auto
                .resolve(availability)
                .expect("auto selection"),
            vec![SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn explicit_premium_selection_requires_available_adapter() {
        assert_eq!(
            SearchBackendSelection::explicit(SearchBackendId::Tavily)
                .resolve(SearchBackendAvailability::default()),
            Err(SearchBackendSelectionError::Unavailable(
                SearchBackendId::Tavily
            ))
        );
    }

    #[test]
    fn explicit_premium_selection_is_eligible_when_available() {
        assert_eq!(
            SearchBackendSelection::explicit(SearchBackendId::Tavily)
                .resolve(SearchBackendAvailability::new(true, false))
                .expect("explicit premium selection"),
            vec![SearchBackendId::Tavily]
        );
    }

    #[test]
    fn explicit_auto_fallback_is_ordered_after_the_selected_override() {
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Tavily)
                .resolve(SearchBackendAvailability::new(true, false))
                .expect("explicit premium fallback plan"),
            vec![SearchBackendId::Tavily, SearchBackendId::DuckDuckGo]
        );
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::SearXng)
                .resolve(SearchBackendAvailability::new(false, true))
                .expect("explicit self-hosted fallback plan"),
            vec![SearchBackendId::SearXng, SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn fallback_never_repairs_missing_explicit_configuration() {
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Tavily)
                .resolve(SearchBackendAvailability::default()),
            Err(SearchBackendSelectionError::Unavailable(
                SearchBackendId::Tavily
            ))
        );
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::SearXng)
                .resolve(SearchBackendAvailability::default()),
            Err(SearchBackendSelectionError::Unavailable(
                SearchBackendId::SearXng
            ))
        );
    }

    #[test]
    fn explicit_self_hosted_selection_requires_supplied_configuration() {
        assert_eq!(
            SearchBackendSelection::explicit(SearchBackendId::SearXng)
                .resolve(SearchBackendAvailability::default()),
            Err(SearchBackendSelectionError::Unavailable(
                SearchBackendId::SearXng
            ))
        );
        assert_eq!(
            SearchBackendSelection::explicit(SearchBackendId::SearXng)
                .resolve(SearchBackendAvailability::new(false, true))
                .expect("explicit self-hosted selection"),
            vec![SearchBackendId::SearXng]
        );
    }

    #[test]
    fn explicit_zero_key_direct_route_needs_no_optional_configuration_or_duplicate_fallback() {
        assert_eq!(
            SearchBackendSelection::explicit(SearchBackendId::DuckDuckGo)
                .resolve(SearchBackendAvailability::default())
                .expect("explicit direct selection"),
            vec![SearchBackendId::DuckDuckGo]
        );
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::DuckDuckGo)
                .resolve(SearchBackendAvailability::default())
                .expect("explicit direct selection with redundant fallback"),
            vec![SearchBackendId::DuckDuckGo]
        );
    }

    #[test]
    fn wikipedia_is_not_a_generic_search_candidate() {
        assert_eq!(
            SearchBackendSelection::explicit_with_auto_fallback(SearchBackendId::Wikipedia)
                .resolve(SearchBackendAvailability::new(true, true)),
            Err(SearchBackendSelectionError::Unsupported(
                SearchBackendId::Wikipedia
            ))
        );
    }
}
