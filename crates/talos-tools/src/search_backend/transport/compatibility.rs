//! Dormant fixed request preparation; no network or approval effects.
//!
//! Protocol fields derived from rust-websearch 0.1.2, VCS b30180a0,
//! crates/websearch/src/{duckduckgo/{lite,html,instant_answer},bing}.rs.
//! MIT notice: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Talos adds fixed destinations, bounded encoding and owning invocation binding.
//! Headers/UA, retries, source selection and parser binding remain separate gates.

use super::{
    BodyFormat, Method, PreparedTransportRequest, ToolNetworkEndpoint, ToolNetworkManifest,
    TransportError, Url, encode_form_with_limit, request_endpoint,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::search_backend) enum CompatibilityProtocol {
    Lite,
    Html,
    InstantAnswer,
    Bing,
}

impl CompatibilityProtocol {
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
