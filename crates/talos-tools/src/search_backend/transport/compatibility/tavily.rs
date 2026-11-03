//! Dormant explicit Tavily adapter. Configuration grants no network authority and is never selected
//! from ambient credentials.

use super::{
    CompatibilityError, Method, PreparedTransportRequest, SearchTransport, TransportError,
    TransportResponse, Url, admission, manifest_extends_scope_without_policy_widening,
    request_endpoint,
};
use crate::search_backend::{
    SearchBackendId, SearchBackendResult,
    transport::{SystemSearchDns, TransportBudget},
};
use std::sync::Arc;
use talos_core::tool::{
    ToolNature, ToolNetworkAuthority, ToolNetworkManifest, ToolNetworkScope, ToolPermissionFacet,
    ToolResourceKind,
};
use thiserror::Error;

const TAVILY_SEARCH_URL: &str = "https://api.tavily.com/search";
const MAX_BEARER_TOKEN_BYTES: usize = 4096 - "Bearer ".len();

/// Explicit credentialed Tavily configuration. No environment lookup occurs here.
pub(in crate::search_backend) struct TavilyConfiguration {
    api_key: String,
    url: Url,
    scope: ToolNetworkScope,
}

impl std::fmt::Debug for TavilyConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TavilyConfiguration(<redacted>)")
    }
}

impl TavilyConfiguration {
    pub(in crate::search_backend) fn new(api_key: &str) -> Result<Self, TransportError> {
        if api_key.is_empty()
            || api_key.len() > MAX_BEARER_TOKEN_BYTES
            || !api_key.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(TransportError::InvalidRequest);
        }
        let url = Url::parse(TAVILY_SEARCH_URL).map_err(|_| TransportError::InvalidRequest)?;
        let endpoint = request_endpoint(&url, &Method::POST)?;
        // Credential-bearing requests deliberately refuse redirects so Authorization can never be
        // replayed to a redirected destination. Proxy support, when composed, must use the same
        // frozen global transport policy as the other selected candidates.
        let scope = ToolNetworkScope::new_configuration(vec![endpoint], None, vec![], 0)
            .map_err(|_| TransportError::InvalidRequest)?;
        Ok(Self {
            api_key: api_key.to_owned(),
            url,
            scope,
        })
    }

    pub(in crate::search_backend) fn scope(&self) -> &ToolNetworkScope {
        &self.scope
    }

    pub(in crate::search_backend) fn prepare(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedTavily, TransportError> {
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        self.prepare_with_manifest(&manifest, query, max_results)
    }

    /// Freeze one explicit Tavily request against a future combined invocation manifest.
    /// Endpoints may be a superset, but global proxy/private/redirect policy may not widen.
    pub(in crate::search_backend) fn prepare_with_manifest(
        &self,
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedTavily, TransportError> {
        if query.trim().is_empty() || query.len() > 4096 || !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest);
        }
        if !manifest_extends_scope_without_policy_widening(manifest, &self.scope) {
            return Err(TransportError::InvalidRequest);
        }

        // Preserve the legacy Tavily request shape while moving ownership under the controlled
        // prepared transport. Tavily's historical adapter requested at most ten rows even though
        // the model-facing web_search schema permits twenty.
        let body = serde_json::to_vec(&serde_json::json!({
            "query": query,
            "max_results": max_results.min(10),
            "search_depth": "basic",
            "include_answer": false,
            "include_raw_content": false,
            "include_images": false,
        }))
        .map_err(|_| TransportError::InvalidRequest)?;
        let request = PreparedTransportRequest::json_bytes(
            manifest,
            self.url.clone(),
            Method::POST,
            Some(body),
        )?
        .with_bearer_authorization(&self.api_key)?;
        let facets = facets(manifest);
        Ok(PreparedTavily {
            manifest: manifest.clone(),
            request,
            facets,
            max_results,
        })
    }
}

fn facets(manifest: &ToolNetworkManifest) -> Vec<ToolPermissionFacet> {
    let mut hosts: Vec<_> = manifest
        .scope()
        .endpoints()
        .iter()
        .map(|endpoint| endpoint.host())
        .collect();
    if let Some(proxy) = manifest.scope().proxy() {
        hosts.push(proxy.host());
    }
    hosts.sort_unstable();
    hosts.dedup();
    hosts
        .into_iter()
        .map(|host| {
            ToolPermissionFacet::with_resource(ToolNature::Network, host, ToolResourceKind::Domain)
        })
        .collect()
}

pub(in crate::search_backend) struct PreparedTavily {
    manifest: ToolNetworkManifest,
    request: PreparedTransportRequest,
    facets: Vec<ToolPermissionFacet>,
    max_results: u32,
}

impl std::fmt::Debug for PreparedTavily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedTavily(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum TavilyError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Admission(#[from] CompatibilityError),
    #[error("search_tavily_auth")]
    Auth,
    #[error("search_tavily_rate_limited")]
    RateLimited,
    #[error("search_tavily_status")]
    Status,
    #[error("search_tavily_parse")]
    Parse,
    #[error("search_tavily_limit")]
    Limit,
}

impl PreparedTavily {
    pub(in crate::search_backend) fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }

    pub(in crate::search_backend) fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }

    pub(in crate::search_backend) async fn execute(
        self,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
    ) -> Result<admission::AdmittedCompatibilityResults, TavilyError> {
        let transport =
            SearchTransport::new(self.manifest, authority, budget, Arc::new(SystemSearchDns))?;
        let response = transport.execute_prepared(self.request).await?;
        finish(response, self.max_results, || transport.budget.check_live())
    }
}

fn finish(
    response: TransportResponse,
    max_results: u32,
    mut live: impl FnMut() -> Result<(), TransportError>,
) -> Result<admission::AdmittedCompatibilityResults, TavilyError> {
    live()?;
    let parsed = parse(response);
    live()?;
    let admitted = admission::admit(SearchBackendId::Tavily, parsed?, max_results, &mut live);
    live()?;
    Ok(admitted?)
}

fn parse(response: TransportResponse) -> Result<Vec<SearchBackendResult>, TavilyError> {
    match response.status {
        401 => return Err(TavilyError::Auth),
        429 => return Err(TavilyError::RateLimited),
        status if !(200..300).contains(&status) => return Err(TavilyError::Status),
        _ => {}
    }
    if response.body.len() > 1_048_576 {
        return Err(TavilyError::Limit);
    }
    std::panic::catch_unwind(|| {
        let value: serde_json::Value =
            serde_json::from_slice(&response.body).map_err(|_| TavilyError::Parse)?;
        let rows = value
            .get("results")
            .and_then(serde_json::Value::as_array)
            .ok_or(TavilyError::Parse)?;
        if rows.len() > admission::MAX_CANDIDATES as usize {
            return Err(TavilyError::Limit);
        }
        Ok(rows
            .iter()
            .map(|row| SearchBackendResult {
                backend: SearchBackendId::Tavily,
                title: row
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Untitled")
                    .to_owned(),
                url: row
                    .get("url")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
                snippet: row
                    .get("content")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
            })
            .collect())
    })
    .map_err(|_| TavilyError::Parse)?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_is_explicit_fixed_scope_and_debug_redacted() {
        let secret = "toy-tavily-secret";
        let configuration = TavilyConfiguration::new(secret).expect("configuration");
        assert!(!format!("{configuration:?}").contains(secret));
        assert_eq!(configuration.scope().redirect_limit(), 0);
        let endpoints = configuration.scope().endpoints();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].scheme(), "https");
        assert_eq!(endpoints[0].host(), "api.tavily.com");
        assert_eq!(endpoints[0].port(), 443);
        assert_eq!(endpoints[0].method(), "POST");
        assert_eq!(endpoints[0].path(), "/search");
    }

    #[test]
    fn configuration_rejects_non_explicit_or_unsafe_credentials() {
        for invalid in [
            String::new(),
            "contains space".to_owned(),
            "line\nfeed".to_owned(),
            "中".to_owned(),
            "a".repeat(MAX_BEARER_TOKEN_BYTES + 1),
        ] {
            let error = TavilyConfiguration::new(&invalid).expect_err("invalid credential");
            assert_eq!(error, TransportError::InvalidRequest);
            if !invalid.is_empty() {
                assert!(!format!("{error:?} {error}").contains(&invalid));
            }
        }
    }

    #[test]
    fn preparation_preserves_private_query_and_secret_redaction() {
        let secret = "toy-tavily-secret";
        let query = "private query";
        let configuration = TavilyConfiguration::new(secret).expect("configuration");
        let prepared = configuration.prepare(query, 3).expect("prepared");
        let debug = format!("{prepared:?}");
        assert!(!debug.contains(secret));
        assert!(!debug.contains(query));
        assert_eq!(prepared.permission_profile().len(), 1);
        assert_eq!(prepared.manifest().scope(), configuration.scope());
    }

    #[test]
    fn parser_classifies_status_without_response_body_disclosure() {
        for (status, expected) in [
            (401, TavilyError::Auth),
            (429, TavilyError::RateLimited),
            (503, TavilyError::Status),
        ] {
            let secret_body = b"body-must-not-escape".to_vec();
            let error = parse(TransportResponse {
                status,
                body: secret_body,
            })
            .expect_err("status failure");
            assert_eq!(error, expected);
            assert!(!format!("{error:?} {error}").contains("body-must-not-escape"));
        }
    }

    #[test]
    fn parser_preserves_tavily_identity_and_bounds_candidate_array() {
        let body = serde_json::to_vec(&serde_json::json!({
            "results": [
                {
                    "title": "Rust",
                    "url": "https://rust-lang.org/",
                    "content": "Reliable software"
                },
                {
                    "title": "Tokio",
                    "url": "https://tokio.rs/",
                    "content": "Async runtime"
                }
            ]
        }))
        .expect("fixture");
        let rows = parse(TransportResponse { status: 200, body }).expect("parsed");
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|row| row.backend == SearchBackendId::Tavily)
        );

        let oversized = serde_json::to_vec(&serde_json::json!({
            "results": (0..=admission::MAX_CANDIDATES)
                .map(|index| serde_json::json!({
                    "title": format!("row-{index}"),
                    "url": format!("https://example.com/{index}"),
                    "content": "fixture"
                }))
                .collect::<Vec<_>>()
        }))
        .expect("oversized fixture");
        assert_eq!(
            parse(TransportResponse {
                status: 200,
                body: oversized,
            }),
            Err(TavilyError::Limit)
        );
    }
}
