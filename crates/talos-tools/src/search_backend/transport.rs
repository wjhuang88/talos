//! Controlled direct HTTP transport for the staged search compatibility migration.
//! No default tool wiring, proxy support, protocol parser or provider admission.

use std::{
    net::{IpAddr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use reqwest::{Client, Method, Url};
use talos_core::tool::{ToolNetworkAuthority, ToolNetworkEndpoint, ToolNetworkManifest};
use thiserror::Error;
use tokio::time::{Instant, timeout_at};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(super) enum TransportError {
    #[error("search_transport_invalid_request")]
    InvalidRequest,
    #[error("search_transport_unauthorized")]
    Unauthorized,
    #[error("search_transport_unsupported_proxy")]
    UnsupportedProxy,
    #[error("search_transport_dns_failed")]
    Dns,
    #[error("search_transport_address_denied")]
    Address,
    #[error("search_transport_redirect_denied")]
    Redirect,
    #[error("search_transport_request_limit")]
    RequestLimit,
    #[error("search_transport_response_limit")]
    ResponseLimit,
    #[error("search_transport_response_encoding")]
    ResponseEncoding,
    #[error("search_transport_deadline")]
    Deadline,
    #[error("search_transport_request_failed")]
    Request,
}

/// One execution budget shared by all future router candidates and redirect hops.
pub(super) struct TransportBudget {
    deadline: Instant,
    attempts: AtomicUsize,
    bytes: AtomicUsize,
    max_requests: usize,
    max_bytes: usize,
}

impl TransportBudget {
    fn check_live(&self) -> Result<(), TransportError> {
        if Instant::now() >= self.deadline {
            Err(TransportError::Deadline)
        } else {
            Ok(())
        }
    }
    pub(super) fn new(
        duration: Duration,
        max_requests: usize,
        max_bytes: usize,
    ) -> Result<Self, TransportError> {
        if duration.is_zero()
            || duration > Duration::from_secs(60)
            || !(1..=8).contains(&max_requests)
            || !(1..=1_048_576).contains(&max_bytes)
        {
            return Err(TransportError::InvalidRequest);
        }
        Ok(Self {
            deadline: Instant::now() + duration,
            attempts: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
            max_requests,
            max_bytes,
        })
    }

    fn reserve(
        counter: &AtomicUsize,
        amount: usize,
        limit: usize,
        error: TransportError,
    ) -> Result<(), TransportError> {
        let mut current = counter.load(Ordering::Relaxed);
        loop {
            let next = current
                .checked_add(amount)
                .filter(|next| *next <= limit)
                .ok_or(error)?;
            match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return Ok(()),
                Err(observed) => current = observed,
            }
        }
    }
}

#[async_trait]
pub(super) trait SearchDns: Send + Sync {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError>;
}

pub(super) struct SystemSearchDns;

#[async_trait]
impl SearchDns for SystemSearchDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        // Keep the seventeenth result so over-limit answers are rejected, not truncated safe.
        Ok(tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| TransportError::Dns)?
            .take(17)
            .collect())
    }
}

pub(super) struct TransportResponse {
    pub(super) status: u16,
    pub(super) body: Vec<u8>,
}

impl std::fmt::Debug for TransportResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TransportResponse(<redacted>)")
    }
}

pub(super) struct SearchTransport {
    manifest: ToolNetworkManifest,
    authority: Arc<dyn ToolNetworkAuthority>,
    budget: Arc<TransportBudget>,
    dns: Arc<dyn SearchDns>,
}

impl SearchTransport {
    pub(super) fn new(
        manifest: ToolNetworkManifest,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
        dns: Arc<dyn SearchDns>,
    ) -> Result<Self, TransportError> {
        if manifest.scope().proxy().is_some() {
            // No silent direct fallback for a plan that explicitly requires a proxy.
            return Err(TransportError::UnsupportedProxy);
        }
        Ok(Self {
            manifest,
            authority,
            budget,
            dns,
        })
    }

    fn check(&self, url: &Url, method: &Method) -> Result<ToolNetworkEndpoint, TransportError> {
        self.budget.check_live()?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.as_str().len() > 8192
            || url.query().is_some_and(|q| q.len() > 4096)
        {
            return Err(TransportError::InvalidRequest);
        }
        let endpoint = ToolNetworkEndpoint::new(
            url.scheme(),
            url.host_str().ok_or(TransportError::InvalidRequest)?,
            url.port_or_known_default()
                .ok_or(TransportError::InvalidRequest)?,
            method.as_str(),
            url.path(),
        )
        .map_err(|_| TransportError::InvalidRequest)?;
        if !self.manifest.scope().endpoints().contains(&endpoint) {
            return Err(TransportError::Unauthorized);
        }
        self.authority
            .check(self.manifest.invocation_id(), &endpoint)
            .map_err(|_| TransportError::Unauthorized)?;
        Ok(endpoint)
    }

    pub(super) async fn execute(
        &self,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<TransportResponse, TransportError> {
        self.budget.check_live()?;
        if !matches!(method, Method::GET | Method::HEAD | Method::POST)
            || body.as_ref().is_some_and(|b| b.len() > 16_384)
            || (method != Method::POST && body.is_some())
        {
            return Err(TransportError::InvalidRequest);
        }
        let result = timeout_at(self.budget.deadline, self.execute_inner(url, method, body))
            .await
            .map_err(|_| TransportError::Deadline)?;
        self.budget.check_live()?;
        result
    }

    async fn execute_inner(
        &self,
        mut url: Url,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<TransportResponse, TransportError> {
        let mut redirects = 0;
        loop {
            let endpoint = self.check(&url, &method)?; // Denied hosts never reach DNS.
            // Bound every hop attempt, including failed DNS or a later denied send.
            TransportBudget::reserve(
                &self.budget.attempts,
                1,
                self.budget.max_requests,
                TransportError::RequestLimit,
            )?;
            let host = endpoint
                .host()
                .trim_start_matches('[')
                .trim_end_matches(']');
            let addresses = match host.parse::<IpAddr>() {
                Ok(ip) => vec![SocketAddr::new(ip, endpoint.port())],
                Err(_) => self.dns.resolve(host, endpoint.port()).await?,
            };
            self.budget.check_live()?;
            if addresses.is_empty()
                || addresses.len() > 16
                || addresses.iter().any(|a| {
                    a.port() != endpoint.port()
                        || (!is_global_address(a.ip())
                            && !self.manifest.scope().private_addresses().contains(&a.ip()))
                })
            {
                return Err(TransportError::Address);
            }
            // A fresh direct HTTP/1 client for each hop cannot reuse another authority's
            // connection, coalesce origins, follow redirects or consult ambient proxies.
            // Override the actual dial set while preserving hostname/Host/SNI/TLS validation.
            let client = Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .referer(false)
                .http1_only()
                .pool_max_idle_per_host(0)
                .no_gzip()
                .no_brotli()
                .no_deflate()
                .no_zstd()
                .resolve_to_addrs(host, &addresses)
                .build()
                .map_err(|_| TransportError::Request)?;
            let mut request = client
                .request(method.clone(), url.clone())
                .header(reqwest::header::ACCEPT_ENCODING, "identity");
            if let Some(body) = body.as_ref() {
                request = request
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body.clone());
            }
            let request = request
                .build()
                .map_err(|_| TransportError::InvalidRequest)?;
            // Last synchronous fence, after DNS and client construction, before logical send.
            self.check(request.url(), request.method())?;
            let mut response = client
                .execute(request)
                .await
                .map_err(|_| TransportError::Request)?;
            self.budget.check_live()?;
            // Hyper's HTTP/1 parser has its own finite buffer limit; this tighter
            // bound applies after parsing, before redirect interpretation or body reads.
            if response.headers().len() > 64
                || response
                    .headers()
                    .iter()
                    .map(|(name, value)| name.as_str().len() + value.as_bytes().len())
                    .sum::<usize>()
                    > 16_384
            {
                return Err(TransportError::ResponseLimit);
            }
            if response
                .headers()
                .get_all(reqwest::header::CONTENT_ENCODING)
                .iter()
                .any(|value| value.as_bytes() != b"identity")
            {
                return Err(TransportError::ResponseEncoding);
            }
            if response.status().is_redirection() {
                if redirects >= self.manifest.scope().redirect_limit() {
                    return Err(TransportError::Redirect);
                }
                let status = response.status().as_u16();
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|h| h.to_str().ok())
                    .filter(|s| s.len() <= 8192)
                    .ok_or(TransportError::Redirect)?;
                let next = url.join(location).map_err(|_| TransportError::Redirect)?;
                if !matches!(status, 301 | 302 | 303 | 307 | 308)
                    || (url.scheme() == "https" && next.scheme() != "https")
                    || (method == Method::POST
                        && (!matches!(status, 307 | 308) || next.origin() != url.origin()))
                {
                    return Err(TransportError::Redirect);
                }
                self.check(&next, &method)?; // No DNS or bytes to a denied next hop.
                redirects += 1;
                url = next;
                continue;
            }
            let status = response.status().as_u16();
            let mut result = Vec::new();
            loop {
                self.budget.check_live()?;
                let chunk = response
                    .chunk()
                    .await
                    .map_err(|_| TransportError::Request)?;
                self.budget.check_live()?;
                let Some(chunk) = chunk else {
                    break;
                };
                TransportBudget::reserve(
                    &self.budget.bytes,
                    chunk.len(),
                    self.budget.max_bytes,
                    TransportError::ResponseLimit,
                )?;
                result.extend_from_slice(&chunk);
            }
            self.budget.check_live()?;
            return Ok(TransportResponse {
                status,
                body: result,
            });
        }
    }
}

/// Conservative public-unicast allowlist; not generic HTTP tool policy. Exact explicit
/// exceptions remain exact IpAddr matches (mapped IPv6 is not an IPv4 exception alias).
fn is_global_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192
                    && (b == 168 || (b == 0 && matches!(c, 0 | 2)) || (b == 88 && c == 99)))
                || (a == 198 && (matches!(b, 18 | 19) || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return is_global_address(IpAddr::V4(v4));
            }
            let s = ip.segments();
            (s[0] & 0xe000) == 0x2000 // excludes local, multicast and translation prefixes
                && s[0] != 0x2002 // 6to4 embeds another dial destination
                && s[0] != 0x3ffe // retired 6bone testing allocation
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
                && !(s[0] == 0x3fff && (s[1] & 0xf000) == 0)
        }
    }
}

#[cfg(test)]
mod tests;
