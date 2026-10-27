//! Dormant explicit SearXNG adapter. Configuration grants no network authority.

pub(in crate::search_backend) mod invocation;
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

pub(in crate::search_backend) struct SearXngConfiguration {
    url: Url,
    scope: ToolNetworkScope,
}
impl std::fmt::Debug for SearXngConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearXngConfiguration(<redacted>)")
    }
}
impl SearXngConfiguration {
    pub(in crate::search_backend) fn new(base: &str) -> Result<Self, TransportError> {
        if base.len() > 8192 || base.bytes().any(|b| b.is_ascii_control()) || base.trim() != base {
            return Err(TransportError::InvalidRequest);
        }
        let mut url = std::panic::catch_unwind(|| Url::parse(base))
            .map_err(|_| TransportError::InvalidRequest)?
            .map_err(|_| TransportError::InvalidRequest)?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(TransportError::InvalidRequest);
        }
        let path = format!("{}/search", url.path().trim_end_matches('/'));
        url.set_path(&path);
        let endpoint = request_endpoint(&url, &Method::GET)?;
        let scope = ToolNetworkScope::new_configuration(vec![endpoint], None, vec![], 0)
            .map_err(|_| TransportError::InvalidRequest)?;
        Ok(Self { url, scope })
    }

    pub(in crate::search_backend) fn scope(&self) -> &ToolNetworkScope {
        &self.scope
    }

    pub(in crate::search_backend) fn prepare(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedSearXng, TransportError> {
        let manifest = ToolNetworkManifest::new(self.scope.clone());
        self.prepare_with_manifest(&manifest, query, max_results)
    }

    /// Freeze this candidate against an invocation manifest owned by a future combined plan.
    /// Endpoints may be a superset, but global proxy/private/redirect policy may not widen.
    pub(in crate::search_backend) fn prepare_with_manifest(
        &self,
        manifest: &ToolNetworkManifest,
        query: &str,
        max_results: u32,
    ) -> Result<PreparedSearXng, TransportError> {
        if query.trim().is_empty() || query.len() > 4096 || !(1..=20).contains(&max_results) {
            return Err(TransportError::InvalidRequest);
        }
        if !manifest_extends_scope_without_policy_widening(manifest, &self.scope) {
            return Err(TransportError::InvalidRequest);
        }
        // Match the existing adapter's byte encoding, including %20 rather than '+'.
        let mut encoded = String::from("format=json&q=");
        for byte in query.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char)
                }
                _ => {
                    use std::fmt::Write;
                    write!(&mut encoded, "%{byte:02X}")
                        .map_err(|_| TransportError::InvalidRequest)?;
                }
            }
            if encoded.len() > 4096 {
                return Err(TransportError::InvalidRequest);
            }
        }
        let mut url = self.url.clone();
        url.set_query(Some(&encoded));
        let request = PreparedTransportRequest::json_bytes(manifest, url, Method::GET, None)?;
        let facets = facets(manifest);
        Ok(PreparedSearXng {
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
    hosts.sort_unstable();
    hosts.dedup();
    hosts
        .into_iter()
        .map(|host| {
            ToolPermissionFacet::with_resource(ToolNature::Network, host, ToolResourceKind::Domain)
        })
        .collect()
}

pub(in crate::search_backend) struct PreparedSearXng {
    manifest: ToolNetworkManifest,
    request: PreparedTransportRequest,
    facets: Vec<ToolPermissionFacet>,
    max_results: u32,
}
impl std::fmt::Debug for PreparedSearXng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedSearXng(<redacted>)")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(in crate::search_backend) enum SearXngError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Admission(#[from] CompatibilityError),
    #[error("search_searxng_status")]
    Status,
    #[error("search_searxng_parse")]
    Parse,
    #[error("search_searxng_limit")]
    Limit,
}
impl PreparedSearXng {
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
    ) -> Result<admission::AdmittedCompatibilityResults, SearXngError> {
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
) -> Result<admission::AdmittedCompatibilityResults, SearXngError> {
    live()?;
    let parsed = parse(response);
    live()?;
    let admitted = admission::admit(SearchBackendId::SearXng, parsed?, max_results, &mut live);
    live()?;
    Ok(admitted?)
}
fn parse(response: TransportResponse) -> Result<Vec<SearchBackendResult>, SearXngError> {
    if !(200..300).contains(&response.status) {
        return Err(SearXngError::Status);
    }
    if response.body.len() > 1_048_576 {
        return Err(SearXngError::Limit);
    }
    std::panic::catch_unwind(|| {
        let value: serde_json::Value =
            serde_json::from_slice(&response.body).map_err(|_| SearXngError::Parse)?;
        let rows = value
            .get("results")
            .and_then(serde_json::Value::as_array)
            .ok_or(SearXngError::Parse)?;
        // Deliberately stricter than legacy take(max_results): reject oversized arrays.
        if rows.len() > admission::MAX_CANDIDATES as usize {
            return Err(SearXngError::Limit);
        }
        Ok(rows
            .iter()
            .map(|row| SearchBackendResult {
                backend: SearchBackendId::SearXng,
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
                    .or_else(|| row.get("snippet").and_then(serde_json::Value::as_str))
                    .unwrap_or("")
                    .to_owned(),
            })
            .collect())
    })
    .map_err(|_| SearXngError::Parse)?
}

#[cfg(test)]
mod tests;
