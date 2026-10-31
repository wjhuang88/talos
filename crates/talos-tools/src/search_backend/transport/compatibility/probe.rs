//! Query-free request-time reachability gating retained from rust-websearch 0.1.2.
//!
//! The legacy dependency probes DuckDuckGo and Bing before query-bearing sends, caches the
//! result for thirty seconds, and falls back from HEAD to GET only when HEAD returns a
//! non-success/non-redirection status. Talos keeps those fixed semantics inside prepared
//! authority rather than delegating to the dependency's ambient transport.

use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant as StdInstant},
};

use tokio::time::{Instant, timeout_at};

use super::{
    Method, PreparedTransportRequest, SearchTransport, ToolNetworkEndpoint, ToolNetworkManifest,
    TransportError, Url, request_endpoint,
};

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeBackend {
    DuckDuckGo,
    Bing,
}

#[derive(Clone, Copy)]
struct ProbeEntry {
    reachable: bool,
    probed_at: StdInstant,
}

#[derive(Default)]
struct ProbeCache {
    duckduckgo: Option<ProbeEntry>,
    bing: Option<ProbeEntry>,
}

static PROBE_CACHE: OnceLock<Mutex<ProbeCache>> = OnceLock::new();

fn cache() -> &'static Mutex<ProbeCache> {
    PROBE_CACHE.get_or_init(|| Mutex::new(ProbeCache::default()))
}

fn cached(backend: ProbeBackend, now: StdInstant) -> Option<bool> {
    let cache = cache().lock().ok()?;
    let entry = match backend {
        ProbeBackend::DuckDuckGo => cache.duckduckgo,
        ProbeBackend::Bing => cache.bing,
    }?;
    (now.saturating_duration_since(entry.probed_at) < CACHE_TTL).then_some(entry.reachable)
}

fn record(backend: ProbeBackend, reachable: bool, now: StdInstant) {
    let Ok(mut cache) = cache().lock() else {
        return;
    };
    let slot = match backend {
        ProbeBackend::DuckDuckGo => &mut cache.duckduckgo,
        ProbeBackend::Bing => &mut cache.bing,
    };
    *slot = Some(ProbeEntry {
        reachable,
        probed_at: now,
    });
}

#[cfg(test)]
pub(super) fn clear_cache() {
    if let Ok(mut cache) = cache().lock() {
        *cache = ProbeCache::default();
    }
}

#[cfg(test)]
pub(super) fn seed_cache(duckduckgo: bool, bing: bool) {
    clear_cache();
    let now = StdInstant::now();
    record(ProbeBackend::DuckDuckGo, duckduckgo, now);
    record(ProbeBackend::Bing, bing, now);
}

fn root(backend: ProbeBackend) -> Result<Url, TransportError> {
    Url::parse(match backend {
        ProbeBackend::DuckDuckGo => "https://duckduckgo.com/",
        ProbeBackend::Bing => "https://cn.bing.com/",
    })
    .map_err(|_| TransportError::InvalidRequest)
}

pub(super) fn destinations() -> Result<Vec<ToolNetworkEndpoint>, TransportError> {
    let mut endpoints = Vec::with_capacity(4);
    for backend in [ProbeBackend::DuckDuckGo, ProbeBackend::Bing] {
        let url = root(backend)?;
        endpoints.push(request_endpoint(&url, &Method::HEAD)?);
        endpoints.push(request_endpoint(&url, &Method::GET)?);
    }
    Ok(endpoints)
}

struct PreparedProbe {
    backend: ProbeBackend,
    head: PreparedTransportRequest,
    get: PreparedTransportRequest,
}

pub(super) struct PreparedReachabilityProbes {
    duckduckgo: PreparedProbe,
    bing: PreparedProbe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Reachability {
    pub(super) duckduckgo: bool,
    pub(super) bing: bool,
}

impl PreparedReachabilityProbes {
    pub(super) fn prepare(manifest: &ToolNetworkManifest) -> Result<Self, TransportError> {
        let prepare = |backend| {
            let url = root(backend)?;
            Ok::<_, TransportError>(PreparedProbe {
                backend,
                head: PreparedTransportRequest::status_only(manifest, url.clone(), Method::HEAD)?,
                get: PreparedTransportRequest::status_only(manifest, url, Method::GET)?,
            })
        };
        Ok(Self {
            duckduckgo: prepare(ProbeBackend::DuckDuckGo)?,
            bing: prepare(ProbeBackend::Bing)?,
        })
    }

    pub(super) async fn execute(
        self,
        transport: &SearchTransport,
    ) -> Result<Reachability, TransportError> {
        let now = StdInstant::now();
        let ddg_cached = cached(ProbeBackend::DuckDuckGo, now);
        let bing_cached = cached(ProbeBackend::Bing, now);

        let ddg = use_cached_or_probe(ddg_cached, self.duckduckgo, transport);
        let bing = use_cached_or_probe(bing_cached, self.bing, transport);
        let (ddg, bing) = tokio::join!(ddg, bing);
        let ddg = ddg?;
        let bing = bing?;

        let recorded_at = StdInstant::now();
        if ddg_cached.is_none() {
            record(ProbeBackend::DuckDuckGo, ddg, recorded_at);
        }
        if bing_cached.is_none() {
            record(ProbeBackend::Bing, bing, recorded_at);
        }
        Ok(Reachability {
            duckduckgo: ddg,
            bing,
        })
    }
}

async fn use_cached_or_probe(
    cached: Option<bool>,
    probe: PreparedProbe,
    transport: &SearchTransport,
) -> Result<bool, TransportError> {
    if let Some(reachable) = cached {
        return Ok(reachable);
    }
    probe_one(probe, transport).await
}

async fn probe_one(
    probe: PreparedProbe,
    transport: &SearchTransport,
) -> Result<bool, TransportError> {
    debug_assert!(matches!(
        probe.backend,
        ProbeBackend::DuckDuckGo | ProbeBackend::Bing
    ));
    match probe_status(probe.head, transport).await? {
        Some(status) if is_reachable_status(status) => Ok(true),
        Some(_) => Ok(probe_status(probe.get, transport)
            .await?
            .is_some_and(is_reachable_status)),
        None => Ok(false),
    }
}

fn is_reachable_status(status: u16) -> bool {
    (200..400).contains(&status)
}

async fn probe_status(
    request: PreparedTransportRequest,
    transport: &SearchTransport,
) -> Result<Option<u16>, TransportError> {
    transport.budget.check_live()?;
    let local_deadline = std::cmp::min(
        Instant::now() + PROBE_TIMEOUT,
        transport.budget.deadline,
    );
    match timeout_at(local_deadline, transport.execute_prepared(request)).await {
        Ok(Ok(response)) => Ok(Some(response.status)),
        Ok(Err(error)) => classify_probe_error(error),
        Err(_) => match transport.budget.check_live() {
            Ok(()) => Ok(None),
            Err(error) => Err(error),
        },
    }
}

fn classify_probe_error(error: TransportError) -> Result<Option<u16>, TransportError> {
    match error {
        // The legacy probe treats request/network failures as Unreachable and therefore sends no
        // query to that backend family. Preserve that disclosure gate.
        TransportError::Dns
        | TransportError::ConnectionRefused
        | TransportError::Network
        | TransportError::Request => Ok(None),
        // Authority, containment and shared-budget failures remain terminal; a query-free probe
        // must never weaken the protected invocation boundary.
        TransportError::InvalidRequest
        | TransportError::Unauthorized
        | TransportError::UnsupportedProxy
        | TransportError::Address
        | TransportError::Redirect
        | TransportError::RequestLimit
        | TransportError::ResponseLimit
        | TransportError::ResponseEncoding
        | TransportError::Deadline => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_probe_scope_is_query_free_and_matches_legacy_roots() {
        let endpoints = destinations().expect("fixed endpoints");
        assert_eq!(endpoints.len(), 4);
        assert_eq!(endpoints[0].host(), "duckduckgo.com");
        assert_eq!(endpoints[0].method(), "HEAD");
        assert_eq!(endpoints[1].host(), "duckduckgo.com");
        assert_eq!(endpoints[1].method(), "GET");
        assert_eq!(endpoints[2].host(), "cn.bing.com");
        assert_eq!(endpoints[2].method(), "HEAD");
        assert_eq!(endpoints[3].host(), "cn.bing.com");
        assert_eq!(endpoints[3].method(), "GET");
        assert!(endpoints.iter().all(|endpoint| endpoint.path() == "/"));
    }

    #[test]
    fn only_success_and_redirection_are_reachable() {
        for status in [200, 204, 301, 302, 307, 308, 399] {
            assert!(is_reachable_status(status), "{status}");
        }
        for status in [199, 400, 404, 500, 599] {
            assert!(!is_reachable_status(status), "{status}");
        }
    }

    #[test]
    fn deterministic_cache_seed_is_query_free_and_bounded_to_both_legacy_families() {
        seed_cache(false, true);
        let now = StdInstant::now();
        assert_eq!(cached(ProbeBackend::DuckDuckGo, now), Some(false));
        assert_eq!(cached(ProbeBackend::Bing, now), Some(true));
        clear_cache();
        assert_eq!(cached(ProbeBackend::DuckDuckGo, now), None);
        assert_eq!(cached(ProbeBackend::Bing, now), None);
    }
}
