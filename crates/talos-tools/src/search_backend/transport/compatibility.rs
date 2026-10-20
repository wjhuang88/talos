//! Dormant fixed requests and consuming request/response binding.
//! Preparation has no network or approval effects.
//!
//! Protocol fields derived from rust-websearch 0.1.2, VCS b30180a0,
//! crates/websearch/src/{duckduckgo/{lite,html,instant_answer},bing}.rs.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos adds fixed destinations, bounded encoding and owning invocation binding.
//! Headers/UA, retries, source selection and default activation remain gated.

use crate::search_backend::{SearchBackendResult, bing_parser, ddg_parser, instant_answer};
use thiserror::Error;

use super::{
    BodyFormat, Method, PreparedTransportRequest, SearchTransport, ToolNetworkEndpoint,
    ToolNetworkManifest, TransportError, TransportResponse, Url, encode_form_with_limit,
    request_endpoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum CompatibilityError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Ddg(#[from] ddg_parser::DdgParseError),
    #[error(transparent)]
    Bing(#[from] bing_parser::BingParseError),
}

/// Untrusted parsed data; neither a router winner nor permission to fetch results.
pub(in crate::search_backend) struct CompatibilityCandidates(Vec<SearchBackendResult>);

impl std::fmt::Debug for CompatibilityCandidates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CompatibilityCandidates(<redacted>)")
    }
}

impl CompatibilityCandidates {
    pub(in crate::search_backend) fn into_candidates(self) -> Vec<SearchBackendResult> {
        self.0
    }
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
    ) -> Result<CompatibilityCandidates, CompatibilityError> {
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
) -> Result<CompatibilityCandidates, CompatibilityError> {
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
            max_results,
        )
        .map(|data| CompatibilityCandidates(data.into_candidates()))
        .map_err(CompatibilityError::Ddg),
        CompatibilityProtocol::InstantAnswer => {
            instant_answer::parse(response.status, &response.body, query, max_results)
                .map(|data| CompatibilityCandidates(data.into_candidates()))
                .map_err(CompatibilityError::Ddg)
        }
        CompatibilityProtocol::Bing => {
            bing_parser::parse(response.status, &response.body, max_results)
                .map(|data| CompatibilityCandidates(data.into_candidates()))
                .map_err(CompatibilityError::Bing)
        }
    };
    // Always check after parsing, including an error; an expired partial winner cannot escape.
    // This fences completion, but cannot preempt synchronous parsing or suppress panic hooks.
    check_live()?;
    parsed
}

#[derive(Debug, Clone, Copy)]
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
        if !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest.into());
        }
        let request = self.prepare(manifest, query)?;
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
