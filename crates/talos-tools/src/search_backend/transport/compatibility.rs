//! Dormant fixed requests and consuming request/response binding.
//! Preparation has no network or approval effects.
//!
//! Protocol fields derived from rust-websearch 0.1.2, VCS b30180a0,
//! crates/websearch/src/{duckduckgo/{lite,html,instant_answer},bing}.rs.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos adds fixed destinations, bounded encoding and owning invocation binding.
//! Fixed UA/language snapshots remain dormant; retries/source/default activation gated.

mod admission;
pub(in crate::search_backend) mod chain;
pub(super) mod headers;
pub(in crate::search_backend) mod invocation;
pub(in crate::search_backend) mod plan;
pub(in crate::search_backend) mod searxng;

use crate::search_backend::{SearchBackendId, bing_parser, ddg_parser, instant_answer};
use talos_core::tool::ToolNetworkScope;
use thiserror::Error;
use tokio::time::Instant;

use admission::{AdmissionError, AdmittedCompatibilityResults};

use super::{
    BodyFormat, Method, PreparedTransportRequest, SearchTransport, ToolNetworkEndpoint,
    ToolNetworkManifest, TransportBudget, TransportError, TransportResponse, Url,
    encode_form_with_limit, request_endpoint,
};

/// Exposes the exact running budget deadline to the dormant router integration.
/// The budget remains transport-owned; callers cannot replace or extend this clock.
pub(in crate::search_backend) fn shared_budget_deadline(budget: &TransportBudget) -> Instant {
    budget.deadline
}

/// A shared manifest may add candidate endpoints, but it must not widen global transport policy.
/// Private-address exceptions, proxy selection and redirect allowance apply to the whole manifest,
/// not to one endpoint, so mixing different values would grant one candidate another's authority.
pub(super) fn manifest_extends_scope_without_policy_widening(
    manifest: &ToolNetworkManifest,
    required: &ToolNetworkScope,
) -> bool {
    let supplied = manifest.scope();
    required
        .endpoints()
        .iter()
        .all(|endpoint| supplied.endpoints().contains(endpoint))
        && supplied.proxy() == required.proxy()
        && supplied.private_addresses() == required.private_addresses()
        && supplied.redirect_limit() == required.redirect_limit()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum CompatibilityError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Ddg(#[from] ddg_parser::DdgParseError),
    #[error(transparent)]
    Bing(#[from] bing_parser::BingParseError),
    #[error(transparent)]
    Admission(#[from] AdmissionError),
}

/// Retain this same object across future admission. No Clone or replacement inputs.
pub(in crate::search_backend) struct PreparedCompatibilitySearch {
    request: PreparedTransportRequest,
    protocol: CompatibilityProtocol,
    query: String,
    max_results: u32,
}

impl std::fmt::Debug for PreparedCompatibilitySearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedCompatibilitySearch(<redacted>)")
    }
}

impl PreparedCompatibilitySearch {
    pub(in crate::search_backend) async fn execute(
        self,
        transport: &SearchTransport,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityError> {
        let response = transport.execute_prepared(self.request).await?;
        finish_response(
            self.protocol,
            &self.query,
            self.max_results,
            response,
            || transport.budget.check_live(),
        )
    }
}

// The only production caller supplies the existing shared execution-budget checker.
// A private injected check permits deterministic boundary tests, not a clock override API.
fn finish_response(
    protocol: CompatibilityProtocol,
    query: &str,
    max_results: u32,
    response: TransportResponse,
    mut check_live: impl FnMut() -> Result<(), TransportError>,
) -> Result<AdmittedCompatibilityResults, CompatibilityError> {
    check_live()?;
    let parsed = match protocol {
        CompatibilityProtocol::Lite | CompatibilityProtocol::Html => ddg_parser::parse(
            if matches!(protocol, CompatibilityProtocol::Lite) {
                ddg_parser::DdgFormat::Lite
            } else {
                ddg_parser::DdgFormat::Html
            },
            response.status,
            &response.body,
            admission::MAX_CANDIDATES,
        )
        .map(|data| data.into_candidates())
        .map_err(CompatibilityError::Ddg),
        CompatibilityProtocol::InstantAnswer => instant_answer::parse(
            response.status,
            &response.body,
            query,
            admission::MAX_CANDIDATES,
        )
        .map(|data| data.into_candidates())
        .map_err(CompatibilityError::Ddg),
        CompatibilityProtocol::Bing => {
            bing_parser::parse(response.status, &response.body, admission::MAX_CANDIDATES)
                .map(|data| data.into_candidates())
                .map_err(CompatibilityError::Bing)
        }
    };
    // Always check after parsing, including an error; an expired partial winner cannot escape.
    // This fences completion, but cannot preempt synchronous parsing or suppress panic hooks.
    check_live()?;
    let expected_backend = match protocol {
        CompatibilityProtocol::Lite
        | CompatibilityProtocol::Html
        | CompatibilityProtocol::InstantAnswer
        | CompatibilityProtocol::Bing => SearchBackendId::DuckDuckGo,
    };
    let admitted = admission::admit(expected_backend, parsed?, max_results, &mut check_live);
    // An admission error completed after expiry cannot replace the enclosing deadline.
    check_live()?;
    admitted
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::search_backend) enum CompatibilityProtocol {
    Lite,
    Html,
    InstantAnswer,
    Bing,
}

impl CompatibilityProtocol {
    /// Locally freeze request and parser inputs together, without granting authority.
    pub(in crate::search_backend) fn prepare_search(
        self,
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedCompatibilitySearch, CompatibilityError> {
        self.prepare_search_with(
            manifest,
            query,
            max_results,
            rust_websearch::anti_detection::random_user_agent,
        )
    }

    // Private supplier permits deterministic validation-order tests, not a caller override.
    fn prepare_search_with(
        self,
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
        select: impl FnOnce() -> &'static str + std::panic::UnwindSafe,
    ) -> Result<PreparedCompatibilitySearch, CompatibilityError> {
        if !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest.into());
        }
        let request = self.prepare_with(manifest, query, select)?;
        Ok(PreparedCompatibilitySearch {
            request,
            protocol: self,
            query: query.to_owned(),
            max_results,
        })
    }

    fn target(self) -> Result<(Url, Method), TransportError> {
        let (url, method) = match self {
            Self::Lite => ("https://lite.duckduckgo.com/lite/", Method::POST),
            Self::Html => ("https://html.duckduckgo.com/html/", Method::GET),
            Self::InstantAnswer => ("https://api.duckduckgo.com/", Method::GET),
            Self::Bing => ("https://cn.bing.com/search", Method::GET),
        };
        Ok((
            Url::parse(url).map_err(|_| TransportError::InvalidRequest)?,
            method,
        ))
    }

    /// Query-free canonical destination for planning a trusted configuration scope.
    /// This does not grant authority or perform DNS, probes or HTTP.
    pub(in crate::search_backend) fn destination(
        self,
    ) -> Result<ToolNetworkEndpoint, TransportError> {
        let (url, method) = self.target()?;
        request_endpoint(&url, &method)
    }

    /// Freeze original nonblank query bytes into one invocation's owned request.
    /// No approval seal: callers must retain this same object across future admission.
    pub(in crate::search_backend) fn prepare(
        self,
        manifest: &ToolNetworkManifest,
        query: &str,
    ) -> Result<PreparedTransportRequest, TransportError> {
        self.prepare_with(
            manifest,
            query,
            rust_websearch::anti_detection::random_user_agent,
        )
    }

    fn prepare_with(
        self,
        manifest: &ToolNetworkManifest,
        query: &str,
        select: impl FnOnce() -> &'static str + std::panic::UnwindSafe,
    ) -> Result<PreparedTransportRequest, TransportError> {
        let mut request = self.prepare_payload(manifest, query)?;
        request.headers = Some(headers::CompatibilityHeaders::capture(self, select)?);
        Ok(request)
    }

    // Chain preparation validates every payload before selecting any header snapshots.
    fn prepare_payload(
        self,
        manifest: &ToolNetworkManifest,
        query: &str,
    ) -> Result<PreparedTransportRequest, TransportError> {
        if query.len() > 16_384 || query.trim().is_empty() {
            return Err(TransportError::InvalidRequest);
        }
        let mut fields = vec![("q".to_owned(), query.to_owned())];
        let fixed: &[(&str, &str)] = match self {
            Self::Lite => &[("kl", "wt-wt")],
            Self::Html => &[],
            Self::InstantAnswer => &[
                ("format", "json"),
                ("no_html", "1"),
                ("skip_disambig", "1"),
                ("t", "rust-agent-websearch"),
            ],
            Self::Bing => &[("mkt", "zh-CN")],
        };
        fields.extend(
            fixed
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        );
        let (mut url, method) = self.target()?;
        let limit = if method == Method::POST { 16_384 } else { 4096 };
        let encoded = encode_form_with_limit(&fields, limit)?;
        let body = if method == Method::POST {
            Some(encoded)
        } else {
            url.set_query(Some(
                std::str::from_utf8(&encoded).map_err(|_| TransportError::InvalidRequest)?,
            ));
            None
        };
        PreparedTransportRequest::new(manifest, url, method, body, BodyFormat::Form)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod execution_tests;
