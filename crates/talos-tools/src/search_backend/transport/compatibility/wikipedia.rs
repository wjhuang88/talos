//! Prepared migration-time Wikipedia OpenSearch fallback retained from legacy `WebSearchTool`.
//!
//! Wikipedia remains a compatibility fallback rather than a generic/native search provider. The
//! fixed endpoint is declared before permission, the original query is frozen into one GET request,
//! execution uses the same controlled transport, and no runtime call returns to `WebSearchTool`.

use std::time::Duration;

use serde_json::Value;
use tokio::time::{Instant, timeout_at};

use crate::search_backend::{SearchBackendId, SearchBackendResult};

use super::{
    CompatibilityError, Method, PreparedTransportRequest, SearchTransport, ToolNetworkEndpoint,
    ToolNetworkManifest, TransportError, Url,
    admission::{self, AdmissionError, AdmittedCompatibilityResults},
    request_endpoint,
};

const LEGACY_WIKIPEDIA_TIMEOUT: Duration = Duration::from_secs(10);

fn base_url() -> Result<Url, TransportError> {
    Url::parse("https://en.wikipedia.org/w/api.php").map_err(|_| TransportError::InvalidRequest)
}

pub(super) fn destination() -> Result<ToolNetworkEndpoint, TransportError> {
    request_endpoint(&base_url()?, &Method::GET)
}

pub(super) struct PreparedWikipediaSearch {
    request: PreparedTransportRequest,
    max_results: u32,
}

impl std::fmt::Debug for PreparedWikipediaSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedWikipediaSearch(<redacted>)")
    }
}

impl PreparedWikipediaSearch {
    pub(super) fn prepare(
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<Self, CompatibilityError> {
        if query.len() > 16_384 || query.trim().is_empty() || !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest.into());
        }
        let mut url = base_url()?;
        url.query_pairs_mut()
            .append_pair("action", "opensearch")
            .append_pair("search", query)
            .append_pair("limit", &max_results.to_string())
            .append_pair("namespace", "0")
            .append_pair("format", "json");
        let request = PreparedTransportRequest::json_bytes(manifest, url, Method::GET, None)?;
        Ok(Self {
            request,
            max_results,
        })
    }

    pub(super) async fn execute(
        self,
        transport: &SearchTransport,
    ) -> Result<AdmittedCompatibilityResults, CompatibilityError> {
        transport.budget.check_live()?;
        let source_deadline = std::cmp::min(
            Instant::now() + LEGACY_WIKIPEDIA_TIMEOUT,
            transport.budget.deadline,
        );
        let response = match timeout_at(source_deadline, transport.execute_prepared(self.request)).await
        {
            Ok(result) => result?,
            Err(_) => match transport.budget.check_live() {
                Ok(()) => return Err(CompatibilityError::Unreachable),
                Err(error) => return Err(error.into()),
            },
        };
        transport.budget.check_live()?;
        if !(200..300).contains(&response.status) {
            return Err(CompatibilityError::Unreachable);
        }
        let candidates = parse(&response.body, self.max_results)?;
        transport.budget.check_live()?;
        admission::admit(
            SearchBackendId::Wikipedia,
            candidates,
            self.max_results,
            || transport.budget.check_live(),
        )
    }
}

fn parse(body: &[u8], max_results: u32) -> Result<Vec<SearchBackendResult>, CompatibilityError> {
    let body: Value = serde_json::from_slice(body).map_err(|_| AdmissionError::Parse)?;
    let titles = body.get(1).and_then(Value::as_array);
    let descriptions = body.get(2).and_then(Value::as_array);
    let urls = body.get(3).and_then(Value::as_array);
    let (Some(titles), Some(descriptions), Some(urls)) = (titles, descriptions, urls) else {
        return Err(AdmissionError::Parse.into());
    };
    let count = titles
        .len()
        .min(descriptions.len())
        .min(urls.len())
        .min(max_results as usize);
    if count == 0 {
        return Err(AdmissionError::Empty.into());
    }
    Ok((0..count)
        .map(|index| SearchBackendResult {
            backend: SearchBackendId::Wikipedia,
            title: titles[index]
                .as_str()
                .unwrap_or("Untitled")
                .to_owned(),
            url: urls[index].as_str().unwrap_or("").to_owned(),
            snippet: descriptions[index].as_str().unwrap_or("").to_owned(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{ToolNetworkManifest, ToolNetworkScope};

    #[test]
    fn endpoint_is_fixed_and_query_free() {
        let endpoint = destination().expect("fixed endpoint");
        assert_eq!(endpoint.host(), "en.wikipedia.org");
        assert_eq!(endpoint.method(), "GET");
        assert_eq!(endpoint.path(), "/w/api.php");
    }

    #[test]
    fn preparation_freezes_only_the_legacy_query_shape() {
        let endpoint = destination().expect("destination");
        let manifest = ToolNetworkManifest::new(
            ToolNetworkScope::new_configuration(vec![endpoint], None, vec![], 0).expect("scope"),
        );
        let prepared = PreparedWikipediaSearch::prepare(&manifest, "rust async", 2)
            .expect("prepared wikipedia");
        assert_eq!(prepared.max_results, 2);
        assert_eq!(prepared.request.url.host_str(), Some("en.wikipedia.org"));
        assert_eq!(prepared.request.url.path(), "/w/api.php");
        let query = prepared.request.url.query().expect("query");
        assert!(query.contains("action=opensearch"));
        assert!(query.contains("search=rust+async"));
        assert!(query.contains("limit=2"));
        assert!(query.contains("namespace=0"));
        assert!(query.contains("format=json"));
    }

    #[test]
    fn parser_preserves_legacy_shape_before_shared_admission() {
        let rows = parse(
            br#"["q",["Rust","Tokio"],["Lang","Runtime"],["https://rust-lang.org","https://tokio.rs"]]"#,
            2,
        )
        .expect("rows");
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.backend == SearchBackendId::Wikipedia));
        assert_eq!(rows[0].title, "Rust");
        assert_eq!(rows[1].url, "https://tokio.rs");
    }
}
