//! Controlled HTTP transport for the staged search compatibility migration.
//! No default wiring; proxy support is limited to HTTP CONNECT / HTTPS IPv4 literals.

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
    #[cfg(test)]
    fixture_roots: Vec<reqwest::Certificate>,
}

#[derive(Clone, Copy)]
enum BodyFormat {
    Json,
    Form,
}

impl BodyFormat {
    fn content_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Form => "application/x-www-form-urlencoded",
        }
    }
}

fn encode_form(fields: &[(String, String)]) -> Result<Vec<u8>, TransportError> {
    // Check exact serializer expansion before allocating the encoded body. Every pair
    // costs '=' plus separators; even empty fields therefore have a bounded count.
    if fields.len() > 8192 {
        return Err(TransportError::InvalidRequest);
    }
    let mut length: usize = 0;
    for (index, (name, value)) in fields.iter().enumerate() {
        length = length
            .checked_add(if index == 0 { 1 } else { 2 })
            .filter(|n| *n <= 16_384)
            .ok_or(TransportError::InvalidRequest)?;
        for field in [name, value] {
            if field.len() > 16_384 {
                return Err(TransportError::InvalidRequest);
            }
            for byte in field.bytes() {
                let size = if byte.is_ascii_alphanumeric()
                    || matches!(byte, b'-' | b'_' | b'.' | b'*' | b' ')
                {
                    1
                } else {
                    3
                };
                length = length
                    .checked_add(size)
                    .filter(|n| *n <= 16_384)
                    .ok_or(TransportError::InvalidRequest)?;
            }
        }
    }
    // This URL is solely an in-memory serializer container, never a destination.
    let mut encoded =
        Url::parse("https://form.invalid/").map_err(|_| TransportError::InvalidRequest)?;
    encoded.query_pairs_mut().extend_pairs(
        fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str())),
    );
    Ok(encoded.query().unwrap_or_default().as_bytes().to_vec())
}

impl SearchTransport {
    pub(super) fn new(
        manifest: ToolNetworkManifest,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
        dns: Arc<dyn SearchDns>,
    ) -> Result<Self, TransportError> {
        if let Some(proxy) = manifest.scope().proxy()
            && (proxy.scheme() != "http"
                || proxy.method() != "CONNECT"
                || proxy.path() != "/"
                || manifest.scope().endpoints().iter().any(|endpoint| {
                    endpoint.scheme() != "https"
                        || endpoint
                            .host()
                            .trim_matches(['[', ']'])
                            .parse::<std::net::Ipv4Addr>()
                            .is_err()
                        || !matches!(endpoint.method(), "GET" | "HEAD" | "POST")
                }))
        {
            // Hostname CONNECT delegates target DNS to the proxy; it cannot be locally pinned.
            return Err(TransportError::UnsupportedProxy);
        }
        Ok(Self {
            manifest,
            authority,
            budget,
            dns,
            #[cfg(test)]
            fixture_roots: Vec::new(),
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
        if let Some(proxy) = self.manifest.scope().proxy() {
            self.authority
                .check(self.manifest.invocation_id(), proxy)
                .map_err(|_| TransportError::Unauthorized)?;
        }
        self.budget.check_live()?;
        Ok(endpoint)
    }

    pub(super) async fn execute(
        &self,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<TransportResponse, TransportError> {
        self.execute_formatted(url, method, body, BodyFormat::Json)
            .await
    }

    pub(super) async fn execute_form(
        &self,
        url: Url,
        fields: Vec<(String, String)>,
    ) -> Result<TransportResponse, TransportError> {
        self.budget.check_live()?;
        let body = encode_form(&fields)?;
        self.execute_formatted(url, Method::POST, Some(body), BodyFormat::Form)
            .await
    }

    async fn execute_formatted(
        &self,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
        format: BodyFormat,
    ) -> Result<TransportResponse, TransportError> {
        self.budget.check_live()?;
        if !matches!(method, Method::GET | Method::HEAD | Method::POST)
            || body.as_ref().is_some_and(|b| b.len() > 16_384)
            || (method != Method::POST && body.is_some())
        {
            return Err(TransportError::InvalidRequest);
        }
        let result = timeout_at(
            self.budget.deadline,
            self.execute_inner(url, method, body, format),
        )
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
        format: BodyFormat,
    ) -> Result<TransportResponse, TransportError> {
        let mut redirects = 0;
        loop {
            let endpoint = self.check(&url, &method)?; // Denied hosts never reach DNS.
            // Bound every hop attempt, including failed DNS or a later denied send.
            TransportBudget::reserve(
                &self.budget.attempts,
                if self.manifest.scope().proxy().is_some() {
                    2
                } else {
                    1
                },
                self.budget.max_requests,
                TransportError::RequestLimit,
            )?;
            let mut host = endpoint
                .host()
                .trim_start_matches('[')
                .trim_end_matches(']');
            let mut port = endpoint.port();
            if let Some(proxy) = self.manifest.scope().proxy() {
                let target = host
                    .parse::<IpAddr>()
                    .map_err(|_| TransportError::UnsupportedProxy)?;
                self.check_addresses(&[SocketAddr::new(target, port)], port)?;
                host = proxy.host().trim_matches(['[', ']']);
                port = proxy.port();
            }
            let addresses = match host.parse::<IpAddr>() {
                Ok(ip) => vec![SocketAddr::new(ip, port)],
                Err(_) => self.dns.resolve(host, port).await?,
            };
            self.budget.check_live()?;
            self.check_addresses(&addresses, port)?;
            // A fresh HTTP/1 client cannot reuse another authority's connection, coalesce
            // origins, follow redirects or consult ambient proxies. Pin the direct target
            // or explicit proxy peer; TLS still verifies the original target URL identity.
            let builder = Client::builder()
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
                .resolve_to_addrs(host, &addresses);
            let builder = if let Some(proxy) = self.manifest.scope().proxy() {
                let proxy_url = format!("http://{}:{}/", proxy.host(), proxy.port());
                builder.proxy(
                    reqwest::Proxy::https(proxy_url)
                        .map_err(|_| TransportError::UnsupportedProxy)?,
                )
            } else {
                builder
            };
            #[cfg(test)]
            let builder = builder.tls_certs_merge(self.fixture_roots.clone());
            let client = builder.build().map_err(|_| TransportError::Request)?;
            let mut request = client
                .request(method.clone(), url.clone())
                .header(reqwest::header::ACCEPT_ENCODING, "identity");
            if let Some(body) = body.as_ref() {
                request = request
                    .header(reqwest::header::CONTENT_TYPE, format.content_type())
                    .body(body.clone());
            }
            let request = request
                .build()
                .map_err(|_| TransportError::InvalidRequest)?;
            // Compound CONNECT + TLS + HTTPS starts at this one synchronous fence. Reqwest
            // exposes no check between its internal physical steps; later revocation stops
            // later hops, and cannot undo the already-started compound hop.
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

    fn check_addresses(&self, addresses: &[SocketAddr], port: u16) -> Result<(), TransportError> {
        if addresses.is_empty()
            || addresses.len() > 16
            || addresses.iter().any(|address| {
                address.port() != port
                    || (!is_global_address(address.ip())
                        && !self
                            .manifest
                            .scope()
                            .private_addresses()
                            .contains(&address.ip()))
            })
        {
            Err(TransportError::Address)
        } else {
            Ok(())
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
