//! Inert prepared Bing-family dual-host technical plan (SEARCH-001-D3-T4 / I333).
//!
//! The production compatibility chain is still DDG-first and Bing-CN-only. This module
//! does not register another SearchBackend, construct a production Tool or dispatch HTTP.
//! All new WWW/CN end-to-end replay is test-only and never reaches a public network.

use super::{
    BodyFormat, CompatibilityError, CompatibilityProtocol, Method, PreparedTransportRequest,
    ToolNetworkEndpoint, ToolNetworkManifest, TransportError, Url, encode_form_with_limit,
    headers::CompatibilityHeaders, request_endpoint,
};
use crate::search_backend::bing_variants::BingVariant;

pub(super) struct PreparedBingDualHost {
    manifest: ToolNetworkManifest,
    www: PreparedTransportRequest,
    cn: PreparedTransportRequest,
    query: String,
    max_results: u32,
}

impl std::fmt::Debug for PreparedBingDualHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedBingDualHost(<redacted>)")
    }
}

impl PreparedBingDualHost {
    /// Query-free endpoints. Listing an endpoint does not grant Network authority.
    pub(super) fn destinations() -> Result<[ToolNetworkEndpoint; 2], TransportError> {
        Ok([
            Self::destination(BingVariant::NonCn)?,
            Self::destination(BingVariant::Cn)?,
        ])
    }

    fn destination(variant: BingVariant) -> Result<ToolNetworkEndpoint, TransportError> {
        let url = Url::parse(variant.search_page()).map_err(|_| TransportError::InvalidRequest)?;
        request_endpoint(&url, &Method::GET)
    }

    /// Both request objects must be frozen under the SAME invocation manifest,
    /// before user approval/dispatch. A failed second-host validation cannot
    /// return a partially usable WWW plan or consume randomized headers.
    pub(super) fn prepare(
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<Self, CompatibilityError> {
        Self::prepare_with(
            manifest,
            query,
            max_results,
            crate::search_backend::compat_helpers::random_user_agent,
        )
    }

    fn prepare_with(
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
        select: impl FnOnce() -> &'static str + std::panic::UnwindSafe,
    ) -> Result<Self, CompatibilityError> {
        if !(1..=20).contains(&max_results)
            || query.len() > 16_384
            || query.trim().is_empty()
            || manifest.scope().redirect_limit() != 0
        {
            return Err(TransportError::InvalidRequest.into());
        }
        // The exact two destinations must both have been declared before
        // preparing either request; no fallback-time authority expansion.
        if !Self::destinations()?
            .iter()
            .all(|endpoint| manifest.scope().endpoints().contains(endpoint))
        {
            return Err(TransportError::Unauthorized.into());
        }
        let mut www = Self::prepare_variant(manifest, query, BingVariant::NonCn)?;
        let mut cn = Self::prepare_variant(manifest, query, BingVariant::Cn)?;
        let snapshot = CompatibilityHeaders::capture(CompatibilityProtocol::Bing, select)?;
        www.headers = Some(snapshot);
        cn.headers = Some(snapshot);
        Ok(Self {
            manifest: manifest.clone(),
            www,
            cn,
            query: query.to_owned(),
            max_results,
        })
    }

    fn prepare_variant(
        manifest: &ToolNetworkManifest,
        query: &str,
        variant: BingVariant,
    ) -> Result<PreparedTransportRequest, TransportError> {
        let mut url =
            Url::parse(variant.search_page()).map_err(|_| TransportError::InvalidRequest)?;
        // Fixed market strings are endpoint-variant query formatting, not
        // geographical inference or additional host/proxy selection.
        let market = match variant {
            BingVariant::NonCn => "en-US",
            BingVariant::Cn => "zh-CN",
        };
        let encoded = encode_form_with_limit(
            &[
                ("q".to_owned(), query.to_owned()),
                ("mkt".to_owned(), market.to_owned()),
            ],
            4096,
        )?;
        url.set_query(Some(
            std::str::from_utf8(&encoded).map_err(|_| TransportError::InvalidRequest)?,
        ));
        PreparedTransportRequest::new(manifest, url, Method::GET, None, BodyFormat::Form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_backend::{
        bing_variants::{
            BingAttempt, BingAvailabilityFailure, BingFallbackGuards, BingNext, next_variant,
        },
        transport::{TransportBudget, TransportResponse},
    };
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    use talos_core::tool::{ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkScope};

    struct FixtureAuthority {
        manifest: ToolNetworkManifest,
        denied_host: Option<&'static str>,
        checks: AtomicUsize,
    }

    impl ToolNetworkAuthority for FixtureAuthority {
        fn check(
            &self,
            invocation: uuid::Uuid,
            endpoint: &ToolNetworkEndpoint,
        ) -> Result<(), ToolNetworkAuthorizationError> {
            self.checks.fetch_add(1, Ordering::Relaxed);
            if invocation != self.manifest.invocation_id()
                || !self.manifest.scope().contains(endpoint)
                || self.denied_host == Some(endpoint.host())
            {
                Err(ToolNetworkAuthorizationError::Mismatch)
            } else {
                Ok(())
            }
        }
    }

    enum SyntheticOutcome {
        Response(u16, Vec<u8>),
        Transport(TransportError),
        SourceLocalTimeout,
    }

    const ALLOWED: BingFallbackGuards = BingFallbackGuards {
        cn_prepared: true,
        network_permitted: true,
        proxy_route_prepared: true,
        invocation_live: true,
        send_budget_remaining: true,
    };

    fn fixture_scope() -> ToolNetworkManifest {
        let endpoints = PreparedBingDualHost::destinations().expect("fixed origins");
        ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(endpoints.to_vec(), None, vec![], 0)
                .expect("bounded dual-host scope"),
        )
    }

    fn fixture_authority(manifest: &ToolNetworkManifest) -> FixtureAuthority {
        FixtureAuthority {
            manifest: manifest.clone(),
            denied_host: None,
            checks: AtomicUsize::new(0),
        }
    }

    fn fixture_budget() -> TransportBudget {
        TransportBudget::new(Duration::from_secs(5), 4, 1_048_576).expect("bounded mock budget")
    }

    fn organic() -> Vec<u8> {
        b"<ol><li class='b_algo'><h2><a href='https://result.invalid/one'>One</a></h2><div class='b_caption'><p>One snippet</p></div></li></ol>".to_vec()
    }

    fn frozen() -> (ToolNetworkManifest, PreparedBingDualHost) {
        let manifest = fixture_scope();
        let prepared = PreparedBingDualHost::prepare_with(
            &manifest,
            "user query & privacy",
            2,
            || "Synthetic Bing/1.0",
        )
        .expect("both requests prevalidated");
        (manifest, prepared)
    }

    impl PreparedBingDualHost {
        /// Offline replay only. A real HTTP send is intentionally impossible here.
        /// Simulated sends still check frozen invocation authority and the SAME budget.
        fn replay<F>(
            &self,
            authority: &dyn ToolNetworkAuthority,
            budget: &TransportBudget,
            guards: BingFallbackGuards,
            mut send: F,
        ) -> Result<super::super::admission::AdmittedCompatibilityResults, CompatibilityError>
        where
            F: FnMut(BingVariant, &PreparedTransportRequest) -> SyntheticOutcome,
        {
            let (first, observed) =
                self.simulate(BingVariant::NonCn, &self.www, authority, budget, &mut send);
            if first.is_ok() {
                return first;
            }
            if next_variant(BingVariant::NonCn, observed, guards) != BingNext::TryCn {
                return first;
            }
            self.simulate(BingVariant::Cn, &self.cn, authority, budget, &mut send)
                .0
        }

        fn simulate<F>(
            &self,
            variant: BingVariant,
            request: &PreparedTransportRequest,
            authority: &dyn ToolNetworkAuthority,
            budget: &TransportBudget,
            send: &mut F,
        ) -> (
            Result<super::super::admission::AdmittedCompatibilityResults, CompatibilityError>,
            BingAttempt,
        )
        where
            F: FnMut(BingVariant, &PreparedTransportRequest) -> SyntheticOutcome,
        {
            let check = || -> Result<(), CompatibilityError> {
                budget.check_live()?;
                if request.invocation_id != self.manifest.invocation_id() {
                    return Err(TransportError::Unauthorized.into());
                }
                let endpoint = request_endpoint(&request.url, &request.method)?;
                authority
                    .check(self.manifest.invocation_id(), &endpoint)
                    .map_err(|_| TransportError::Unauthorized)?;
                budget.check_live()?;
                Ok(())
            };
            if let Err(error) = check() {
                return (Err(error), BingAttempt::PermissionDenied);
            }
            // Mirror the protected transport's accounting before an outbound attempt:
            // even a failed DNS lookup consumes one physical request unit. Proxy
            // CONNECT accounting belongs to the real transport, not this mock.
            if let Err(error) = TransportBudget::reserve(
                &budget.attempts,
                1,
                budget.max_requests,
                TransportError::RequestLimit,
            ) {
                return (Err(error.into()), BingAttempt::RequestBudgetExhausted);
            }
            let outcome = send(variant, request);
            if let Err(error) = budget.check_live() {
                return (Err(error.into()), BingAttempt::DeadlineExceeded);
            }
            match outcome {
                SyntheticOutcome::Transport(TransportError::Dns) => (
                    Err(TransportError::Dns.into()),
                    BingAttempt::Unavailable(BingAvailabilityFailure::Dns),
                ),
                SyntheticOutcome::Transport(TransportError::ConnectionRefused) => (
                    Err(TransportError::ConnectionRefused.into()),
                    BingAttempt::Unavailable(BingAvailabilityFailure::ConnectionRefused),
                ),
                SyntheticOutcome::Transport(error) => {
                    (Err(error.into()), BingAttempt::InvalidResponse)
                }
                SyntheticOutcome::SourceLocalTimeout => (
                    Err(TransportError::Network.into()),
                    BingAttempt::Unavailable(BingAvailabilityFailure::SourceLocalTimeout),
                ),
                SyntheticOutcome::Response(status, body) => {
                    // A received body must charge the same shared response budget
                    // before parsing; an over-budget response cannot become a winner
                    // or spuriously authorize switching to CN.
                    if let Err(error) = TransportBudget::reserve(
                        &budget.bytes,
                        body.len(),
                        budget.max_bytes,
                        TransportError::ResponseLimit,
                    ) {
                        return (Err(error.into()), BingAttempt::InvalidResponse);
                    }
                    let response = TransportResponse { status, body };
                    let result = super::super::finish_response(
                        CompatibilityProtocol::Bing,
                        &self.query,
                        self.max_results,
                        response,
                        || budget.check_live(),
                    );
                    let observed = match &result {
                        Ok(_) => BingAttempt::OrganicResults,
                        Err(_) if status == 403 => BingAttempt::AccessDenied,
                        Err(_) if status == 429 => BingAttempt::RateLimited,
                        Err(_) if (300..400).contains(&status) => BingAttempt::UnsupportedRedirect,
                        Err(_) if status != 200 => BingAttempt::HttpStatus(status),
                        Err(CompatibilityError::Bing(
                            crate::search_backend::bing_parser::BingParseError::Empty,
                        )) => BingAttempt::ValidEmpty,
                        Err(CompatibilityError::Bing(
                            crate::search_backend::bing_parser::BingParseError::Challenge,
                        )) => BingAttempt::Challenge,
                        Err(_) => BingAttempt::InvalidResponse,
                    };
                    (result, observed)
                }
            }
        }
    }

    #[test]
    fn both_origins_and_bytes_are_frozen_before_dispatch_with_redacted_debug() {
        let (manifest, plan) = frozen();
        assert_eq!(format!("{plan:?}"), "PreparedBingDualHost(<redacted>)");
        assert_eq!(plan.www.invocation_id, manifest.invocation_id());
        assert_eq!(plan.cn.invocation_id, manifest.invocation_id());
        assert_eq!(plan.www.url.host_str(), Some("www.bing.com"));
        assert_eq!(plan.cn.url.host_str(), Some("cn.bing.com"));
        assert_eq!(plan.www.url.path(), "/search");
        assert_eq!(plan.cn.url.path(), "/search");
        assert!(
            plan.www
                .url
                .query()
                .unwrap_or_default()
                .contains("q=user+query+%26+privacy")
        );
        assert!(
            plan.www
                .url
                .query()
                .unwrap_or_default()
                .contains("mkt=en-US")
        );
        assert!(
            plan.cn
                .url
                .query()
                .unwrap_or_default()
                .contains("mkt=zh-CN")
        );
        assert_eq!(
            plan.www.headers.expect("header").user_agent(),
            "Synthetic Bing/1.0"
        );
        assert_eq!(
            plan.cn.headers.expect("header").user_agent(),
            "Synthetic Bing/1.0"
        );
        assert_eq!(manifest.scope().redirect_limit(), 0);
        assert_eq!(manifest.scope().endpoints().len(), 2);
        assert!(!format!("{plan:?}").contains("user query"));
    }

    #[test]
    fn missing_cn_permission_denies_plan_before_header_supplier_runs() {
        let full = fixture_scope();
        let partial = ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(
                vec![PreparedBingDualHost::destinations().expect("fixed")[0].clone()],
                None,
                vec![],
                0,
            )
            .expect("scope"),
        );
        let calls = AtomicUsize::new(0);
        let outcome = PreparedBingDualHost::prepare_with(&partial, "sensitive", 1, || {
            calls.fetch_add(1, Ordering::Relaxed);
            "Synthetic Bing/1.0"
        });
        assert!(matches!(
            outcome,
            Err(CompatibilityError::Transport(TransportError::Unauthorized))
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert_eq!(full.scope().endpoints().len(), 2);
    }

    #[test]
    fn invalid_query_and_nonzero_redirect_manifest_fail_closed() {
        let manifest = fixture_scope();
        for query in ["", "   "] {
            assert!(PreparedBingDualHost::prepare(&manifest, query, 2).is_err());
        }
        assert!(PreparedBingDualHost::prepare(&manifest, &"x".repeat(16_385), 2).is_err());
        assert!(PreparedBingDualHost::prepare(&manifest, "valid", 0).is_err());
        let redirecting = ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(
                PreparedBingDualHost::destinations().unwrap().to_vec(),
                None,
                vec![],
                1,
            )
            .expect("scope"),
        );
        assert!(matches!(
            PreparedBingDualHost::prepare(&redirecting, "private query", 2),
            Err(CompatibilityError::Transport(
                TransportError::InvalidRequest
            ))
        ));
    }

    #[test]
    fn organic_www_wins_without_second_host_or_new_source_identity() {
        let (manifest, plan) = frozen();
        let authority = fixture_authority(&manifest);
        let mut calls = Vec::new();
        let result = plan
            .replay(&authority, &fixture_budget(), ALLOWED, |variant, _| {
                calls.push(variant);
                SyntheticOutcome::Response(200, organic())
            })
            .expect("organic winner")
            .into_results();
        assert_eq!(calls, vec![BingVariant::NonCn]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].url, "https://result.invalid/one");
        assert_eq!(
            result[0].backend,
            crate::search_backend::SearchBackendId::DuckDuckGo
        );
        assert_eq!(authority.checks.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn typed_dns_refusal_and_source_local_timeout_only_may_try_cn() {
        for mode in 0..3 {
            let (manifest, plan) = frozen();
            let authority = fixture_authority(&manifest);
            let mut calls = Vec::new();
            let result = plan
                .replay(&authority, &fixture_budget(), ALLOWED, |variant, _| {
                    calls.push(variant);
                    if variant == BingVariant::NonCn {
                        match mode {
                            0 => SyntheticOutcome::Transport(TransportError::Dns),
                            1 => SyntheticOutcome::Transport(TransportError::ConnectionRefused),
                            _ => SyntheticOutcome::SourceLocalTimeout,
                        }
                    } else {
                        SyntheticOutcome::Response(200, organic())
                    }
                })
                .expect("permitted fallback");
            assert_eq!(result.into_results().len(), 1);
            assert_eq!(calls, vec![BingVariant::NonCn, BingVariant::Cn]);
            assert_eq!(authority.checks.load(Ordering::Relaxed), 2);
        }
    }

    #[test]
    fn all_http_failures_empty_challenge_and_unknown_redirect_stop_host_rotation() {
        let outcomes = [
            SyntheticOutcome::Response(403, b"denied".to_vec()),
            SyntheticOutcome::Response(429, b"limited".to_vec()),
            SyntheticOutcome::Response(500, b"error".to_vec()),
            SyntheticOutcome::Response(502, b"error".to_vec()),
            SyntheticOutcome::Response(503, b"error".to_vec()),
            SyntheticOutcome::Response(200, b"<html>no organic search results</html>".to_vec()),
            SyntheticOutcome::Response(200, b"g-recaptcha".to_vec()),
            SyntheticOutcome::Response(302, Vec::new()),
        ];
        for first in outcomes {
            let (manifest, plan) = frozen();
            let authority = fixture_authority(&manifest);
            let mut calls = 0;
            assert!(
                plan.replay(&authority, &fixture_budget(), ALLOWED, |_, _| {
                    calls += 1;
                    if calls == 1 {
                        first_response_clone(&first)
                    } else {
                        SyntheticOutcome::Response(200, organic())
                    }
                })
                .is_err()
            );
            assert_eq!(calls, 1);
        }
    }

    fn first_response_clone(outcome: &SyntheticOutcome) -> SyntheticOutcome {
        match outcome {
            SyntheticOutcome::Response(status, body) => {
                SyntheticOutcome::Response(*status, body.clone())
            }
            SyntheticOutcome::Transport(error) => SyntheticOutcome::Transport(*error),
            SyntheticOutcome::SourceLocalTimeout => SyntheticOutcome::SourceLocalTimeout,
        }
    }

    #[test]
    fn mock_replay_charges_one_shared_send_and_response_budget() {
        let (manifest, plan) = frozen();
        let authority = fixture_authority(&manifest);
        let request_limit =
            TransportBudget::new(Duration::from_secs(5), 1, 1_048_576).expect("one request");
        let mut sends = 0;
        let outcome = plan.replay(&authority, &request_limit, ALLOWED, |_, _| {
            sends += 1;
            SyntheticOutcome::Transport(TransportError::Dns)
        });
        assert!(matches!(
            outcome,
            Err(CompatibilityError::Transport(TransportError::RequestLimit))
        ));
        assert_eq!(sends, 1, "no CN callback after the shared send limit");
        assert_eq!(request_limit.attempts.load(Ordering::Relaxed), 1);

        let (manifest, plan) = frozen();
        let authority = fixture_authority(&manifest);
        let response_limit =
            TransportBudget::new(Duration::from_secs(5), 4, 1).expect("one response byte");
        let mut sends = 0;
        let outcome = plan.replay(&authority, &response_limit, ALLOWED, |_, _| {
            sends += 1;
            SyntheticOutcome::Response(200, organic())
        });
        assert!(matches!(
            outcome,
            Err(CompatibilityError::Transport(TransportError::ResponseLimit))
        ));
        assert_eq!(sends, 1, "over-budget response must never rotate hosts");
        assert_eq!(response_limit.attempts.load(Ordering::Relaxed), 1);
        assert_eq!(response_limit.bytes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn ambiguous_transport_network_and_denials_do_not_trigger_fallback() {
        for error in [
            TransportError::Network,
            TransportError::Request,
            TransportError::Redirect,
            TransportError::Unauthorized,
            TransportError::Deadline,
        ] {
            let (manifest, plan) = frozen();
            let authority = fixture_authority(&manifest);
            let mut attempts = 0;
            assert!(
                plan.replay(&authority, &fixture_budget(), ALLOWED, |_, _| {
                    attempts += 1;
                    SyntheticOutcome::Transport(error)
                })
                .is_err()
            );
            assert_eq!(attempts, 1, "ambiguous failure must not rotate hosts");
        }
    }

    #[test]
    fn fallback_refused_without_guards_or_cn_authority_or_live_budget() {
        let (manifest, plan) = frozen();
        let authority = fixture_authority(&manifest);
        let denied = BingFallbackGuards {
            send_budget_remaining: false,
            ..ALLOWED
        };
        let mut calls = 0;
        assert!(
            plan.replay(&authority, &fixture_budget(), denied, |_, _| {
                calls += 1;
                SyntheticOutcome::Transport(TransportError::Dns)
            })
            .is_err()
        );
        assert_eq!(calls, 1);

        let authority = FixtureAuthority {
            manifest: manifest.clone(),
            denied_host: Some("cn.bing.com"),
            checks: AtomicUsize::new(0),
        };
        let mut calls = 0;
        assert!(
            plan.replay(&authority, &fixture_budget(), ALLOWED, |_, _| {
                calls += 1;
                SyntheticOutcome::Transport(TransportError::Dns)
            })
            .is_err()
        );
        assert_eq!(calls, 1, "CN must not send on permission denial");

        let expired = TransportBudget::new(Duration::from_nanos(1), 4, 1_048_576).expect("budget");
        std::thread::sleep(Duration::from_millis(1));
        let mut calls = 0;
        assert!(
            plan.replay(&fixture_authority(&manifest), &expired, ALLOWED, |_, _| {
                calls += 1;
                SyntheticOutcome::Response(200, organic())
            })
            .is_err()
        );
        assert_eq!(calls, 0, "expired invocation must not send");
    }
}
