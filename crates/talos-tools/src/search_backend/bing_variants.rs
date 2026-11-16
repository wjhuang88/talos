//! Offline-only Bing endpoint-variant selection contract (SEARCH-001-D3-T3a / I329).
//!
//! This module is deliberately NOT called by the production SearchRouter or compatibility
//! transport. An endpoint name here is neither prepared Network authority nor permission to
//! issue an HTTP request. Admission of a new query destination is a separate T3b gate.
//! In particular, switching hosts must never be used to evade throttling or access controls.

/// Both endpoints belong to one Bing source family, not two SearchBackend identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BingVariant {
    /// Preferred candidate for a future separately authorized Bing route.
    NonCn,
    /// Existing, currently prepared compatibility destination.
    Cn,
}

impl BingVariant {
    /// A query-free identifier only; this does not construct or authorize a request.
    pub(super) const fn search_page(self) -> &'static str {
        match self {
            Self::NonCn => "https://www.bing.com/search",
            Self::Cn => "https://cn.bing.com/search",
        }
    }
}

/// Strictly typed source-local availability evidence.
///
/// An ordinary HTTP error response is intentionally *not* in this category. A status alone
/// cannot distinguish a server outage from an anti-automation or rate-control response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BingAvailabilityFailure {
    Dns,
    ConnectionEstablishment,
    ConnectionRefused,
    SourceLocalTimeout,
}

/// Query-free summary of one synthetic endpoint attempt.
///
/// Only `Unavailable` can make a fallback eligible; other failures or empty results must
/// terminate this family attempt, even if the underlying transport has a broader retry policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BingAttempt {
    OrganicResults,
    ValidEmpty,
    Unavailable(BingAvailabilityFailure),
    HttpStatus(u16),
    RateLimited,
    Challenge,
    AccessDenied,
    UnsupportedRedirect,
    InvalidResponse,
    ParserDrift,
    EncodingError,
    PermissionDenied,
    Cancelled,
    DeadlineExceeded,
    RequestBudgetExhausted,
}

/// An offline statement of five distinct gates; it does not confer actual Network authority.
///
/// A future caller must independently prove that the alternate *prepared* request and its
/// per-hop permission, overall deadline and physical send budget are already established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct BingFallbackGuards {
    pub(super) cn_prepared: bool,
    pub(super) network_permitted: bool,
    pub(super) proxy_route_prepared: bool,
    pub(super) invocation_live: bool,
    pub(super) send_budget_remaining: bool,
}

impl BingFallbackGuards {
    const fn all_satisfied(self) -> bool {
        self.cn_prepared
            && self.network_permitted
            && self.proxy_route_prepared
            && self.invocation_live
            && self.send_budget_remaining
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BingNext {
    /// Organic results already won; do not disclose the query to another host.
    Complete,
    /// A specifically prepared and permitted CN request would be considered next.
    TryCn,
    /// No second host may be attempted for this observed outcome.
    Stop,
}

/// Non-CN is always the initial Bing-family candidate; no GeoIP or startup probe.
pub(super) const fn first_variant() -> BingVariant {
    BingVariant::NonCn
}

/// Pure, deterministic decision. It performs no I/O, URL resolution, health check or retry.
///
/// It is *not* wired into the current CN-only production chain. In particular, a failure on
/// CN cannot start a new attempt, and a missing permission or deadline never gains a fallback.
pub(super) const fn next_variant(
    attempted: BingVariant,
    outcome: BingAttempt,
    guards: BingFallbackGuards,
) -> BingNext {
    match outcome {
        BingAttempt::OrganicResults => BingNext::Complete,
        BingAttempt::Unavailable(_) => {
            if matches!(attempted, BingVariant::NonCn) && guards.all_satisfied() {
                BingNext::TryCn
            } else {
                BingNext::Stop
            }
        }
        // All HTTP statuses (including 500/502/503), valid-empty responses, challenges,
        // access refusals, malformed bodies and caller-side termination remain terminal.
        _ => BingNext::Stop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOWED: BingFallbackGuards = BingFallbackGuards {
        cn_prepared: true,
        network_permitted: true,
        proxy_route_prepared: true,
        invocation_live: true,
        send_budget_remaining: true,
    };

    #[test]
    fn bing_non_cn_precedes_cn_without_a_third_bare_cn_alias() {
        assert_eq!(first_variant(), BingVariant::NonCn);
        assert_eq!(
            BingVariant::NonCn.search_page(),
            "https://www.bing.com/search"
        );
        assert_eq!(
            BingVariant::Cn.search_page(),
            "https://cn.bing.com/search"
        );
        assert_ne!(
            BingVariant::NonCn.search_page(),
            BingVariant::Cn.search_page()
        );
    }

    #[test]
    fn organic_winner_does_not_disclose_query_to_the_other_bing_host() {
        for variant in [BingVariant::NonCn, BingVariant::Cn] {
            assert_eq!(
                next_variant(variant, BingAttempt::OrganicResults, ALLOWED),
                BingNext::Complete
            );
        }
    }

    #[test]
    fn only_typed_source_local_unavailability_allows_non_cn_to_cn() {
        for failure in [
            BingAvailabilityFailure::Dns,
            BingAvailabilityFailure::ConnectionEstablishment,
            BingAvailabilityFailure::ConnectionRefused,
            BingAvailabilityFailure::SourceLocalTimeout,
        ] {
            assert_eq!(
                next_variant(
                    BingVariant::NonCn,
                    BingAttempt::Unavailable(failure),
                    ALLOWED
                ),
                BingNext::TryCn
            );
            // Exactly one secondary endpoint, no ping-pong or unbounded retry.
            assert_eq!(
                next_variant(BingVariant::Cn, BingAttempt::Unavailable(failure), ALLOWED),
                BingNext::Stop
            );
        }
    }

    #[test]
    fn fallback_needs_prepared_request_permission_proxy_live_clock_and_budget() {
        let denied = [
            BingFallbackGuards {
                cn_prepared: false,
                ..ALLOWED
            },
            BingFallbackGuards {
                network_permitted: false,
                ..ALLOWED
            },
            BingFallbackGuards {
                proxy_route_prepared: false,
                ..ALLOWED
            },
            BingFallbackGuards {
                invocation_live: false,
                ..ALLOWED
            },
            BingFallbackGuards {
                send_budget_remaining: false,
                ..ALLOWED
            },
            BingFallbackGuards::default(),
        ];
        for guards in denied {
            assert_eq!(
                next_variant(
                    BingVariant::NonCn,
                    BingAttempt::Unavailable(BingAvailabilityFailure::Dns),
                    guards,
                ),
                BingNext::Stop
            );
        }
    }

    #[test]
    fn all_http_responses_are_not_host_unavailability_evidence() {
        // Explicitly reject all server-error classes, not merely throttling-like 503.
        for status in [
            200, 302, 400, 401, 403, 404, 408, 429, 500, 502, 503, 504,
        ] {
            assert_eq!(
                next_variant(
                    BingVariant::NonCn,
                    BingAttempt::HttpStatus(status),
                    ALLOWED,
                ),
                BingNext::Stop,
                "status {status} must not switch hosts"
            );
        }
        assert_eq!(
            next_variant(BingVariant::NonCn, BingAttempt::ValidEmpty, ALLOWED),
            BingNext::Stop,
            "valid HTTP 200 with no organic results is not source unavailability"
        );
    }

    #[test]
    fn challenges_redirects_parsing_and_permission_denials_fail_closed() {
        for attempt in [
            BingAttempt::RateLimited,
            BingAttempt::Challenge,
            BingAttempt::AccessDenied,
            BingAttempt::UnsupportedRedirect,
            BingAttempt::InvalidResponse,
            BingAttempt::ParserDrift,
            BingAttempt::EncodingError,
            BingAttempt::PermissionDenied,
            BingAttempt::Cancelled,
            BingAttempt::DeadlineExceeded,
            BingAttempt::RequestBudgetExhausted,
        ] {
            assert_eq!(
                next_variant(BingVariant::NonCn, attempt, ALLOWED),
                BingNext::Stop,
                "{attempt:?} must not switch hosts"
            );
        }
    }

    #[test]
    fn no_implicit_permission_or_query_state_is_in_the_policy() {
        // Defaults fail closed; there is no query field, HTTP client, network call or clock.
        assert_eq!(
            next_variant(
                first_variant(),
                BingAttempt::Unavailable(BingAvailabilityFailure::SourceLocalTimeout),
                BingFallbackGuards::default(),
            ),
            BingNext::Stop
        );
    }
}
