//! Controlled HTTP transport for the staged search compatibility migration.
//! Production proxy support is an immutable environment snapshot carried through prepared authority.

pub(super) mod compatibility;

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
    #[error("search_transport_connection_refused")]
    ConnectionRefused,
    #[error("search_transport_network_failed")]
    Network,
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

// Only the actual send/body-read boundary uses this classification. Construction errors remain
// Request. Legacy rust-websearch retries reqwest Timeout/Network once for Lite/HTML/Bing, so the
// controlled transport preserves only genuinely transient transport failures as typed Network
// outcomes. TLS/certificate, CONNECT protocol, HTTP protocol and untyped body-framing failures
// remain terminal Request.
fn classify_send_error(error: reqwest::Error) -> TransportError {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let timeout = error.is_timeout();
        let mut current: Option<&(dyn std::error::Error + 'static)> = Some(&error);
        // Bound Talos's traversal independently from reqwest's own classification helpers.
        for _ in 0..8 {
            let Some(cause) = current else { break };
            if let Some(io) = cause.downcast_ref::<std::io::Error>() {
                match io.kind() {
                    std::io::ErrorKind::ConnectionRefused => {
                        return TransportError::ConnectionRefused;
                    }
                    std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::NotConnected
                    | std::io::ErrorKind::TimedOut => return TransportError::Network,
                    _ => {}
                }
            }
            current = cause.source();
        }
        if timeout {
            TransportError::Network
        } else {
            TransportError::Request
        }
    }))
    .unwrap_or(TransportError::Request)
}

/// One execution budget shared by all future router candidates and redirect hops.
pub(super) struct TransportBudget {
    deadline: Instant,
    attempts: AtomicUsize,
    bytes: AtomicUsize,
    max_requests: usize,
    max_bytes: usize,
}

/// Validated execution policy, without a running clock or permission authority.
#[derive(Debug, Clone, Copy)]
pub(super) struct TransportLimits {
    duration: Duration,
    max_requests: usize,
    max_bytes: usize,
}

impl TransportLimits {
    pub(super) fn new(
        duration: Duration,
        max_requests: usize,
        max_bytes: usize,
    ) -> Result<Self, TransportError> {
        let standard = duration <= Duration::from_secs(120) && (1..=16).contains(&max_requests);
        // One exact enlarged envelope is reserved for production legacy-compatibility parity:
        // four query-free probes, seven fixed-family query attempts and one Wikipedia fallback,
        // with CONNECT + application-send accounting under HTTPS proxying.
        let compatibility_parity =
            duration == Duration::from_secs(135) && max_requests == 24;
        if duration.is_zero()
            || (!standard && !compatibility_parity)
            || !(1..=1_048_576).contains(&max_bytes)
        {
            return Err(TransportError::InvalidRequest);
        }
        Ok(Self {
            duration,
            max_requests,
            max_bytes,
        })
    }

    pub(super) fn start(self) -> TransportBudget {
        TransportBudget {
            deadline: Instant::now() + self.duration,
            attempts: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
            max_requests: self.max_requests,
            max_bytes: self.max_bytes,
        }
    }
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
        Ok(TransportLimits::new(duration, max_requests, max_bytes)?.start())
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

/// Immutable environment-proxy snapshot for the fixed HTTPS compatibility family.
///
/// The URL may contain credentials and is therefore never exposed through Debug or the network
/// manifest. The manifest receives only the sanitized proxy endpoint. `proxied_hosts` freezes the
/// NO_PROXY decision at tool construction so later environment mutation cannot change routing.
pub(super) struct TransportProxy {
    endpoint: ToolNetworkEndpoint,
    url: Url,
    proxied_hosts: Vec<String>,
}

impl std::fmt::Debug for TransportProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TransportProxy(<redacted>)")
    }
}

impl TransportProxy {
    /// Snapshot the same environment family used by reqwest/hyper-util for HTTPS destinations.
    /// Uppercase names win because hyper-util's matcher checks them first. A CGI environment does
    /// not consume proxy variables. Invalid/unsupported selected proxy configuration fails closed
    /// instead of being silently bypassed by the protected route.
    pub(super) fn from_environment(
        endpoints: &[ToolNetworkEndpoint],
    ) -> Result<Option<Arc<Self>>, TransportError> {
        if std::env::var_os("REQUEST_METHOD").is_some() {
            return Ok(None);
        }
        let all = first_environment_value(&["ALL_PROXY", "all_proxy"]);
        let https = first_environment_value(&["HTTPS_PROXY", "https_proxy"]);
        let no = first_environment_value(&["NO_PROXY", "no_proxy"]);

        let selected = if !https.is_empty() { &https } else { &all };
        if selected.is_empty() {
            return Ok(None);
        }
        let url = parse_proxy_url(selected).ok_or(TransportError::UnsupportedProxy)?;
        Self::from_url(url, &no, endpoints).map(|proxy| proxy.map(Arc::new))
    }

    #[cfg(test)]
    pub(super) fn from_snapshot(
        proxy: &str,
        no_proxy: &str,
        endpoints: &[ToolNetworkEndpoint],
    ) -> Result<Option<Arc<Self>>, TransportError> {
        let url = parse_proxy_url(proxy).ok_or(TransportError::UnsupportedProxy)?;
        Self::from_url(url, no_proxy, endpoints).map(|proxy| proxy.map(Arc::new))
    }

    fn from_url(
        mut url: Url,
        no_proxy: &str,
        endpoints: &[ToolNetworkEndpoint],
    ) -> Result<Option<Self>, TransportError> {
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || endpoints.iter().any(|endpoint| {
                endpoint.scheme() != "https"
                    || !matches!(endpoint.method(), "GET" | "HEAD" | "POST")
            })
        {
            return Err(TransportError::UnsupportedProxy);
        }

        // hyper-util normalizes an environment proxy to its authority and root path.
        url.set_path("/");
        url.set_query(None);
        url.set_fragment(None);
        let host = url.host_str().ok_or(TransportError::UnsupportedProxy)?;
        let endpoint_host = if host.contains(':') {
            format!("[{host}]")
        } else {
            host.to_owned()
        };
        let port = url
            .port_or_known_default()
            .ok_or(TransportError::UnsupportedProxy)?;
        let endpoint = ToolNetworkEndpoint::new(url.scheme(), &endpoint_host, port, "CONNECT", "/")
            .map_err(|_| TransportError::UnsupportedProxy)?;

        let mut proxied_hosts = endpoints
            .iter()
            .filter(|endpoint| !no_proxy_matches(no_proxy, endpoint.host()))
            .map(|endpoint| endpoint.host().to_ascii_lowercase())
            .collect::<Vec<_>>();
        proxied_hosts.sort_unstable();
        proxied_hosts.dedup();
        if proxied_hosts.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            endpoint,
            url,
            proxied_hosts,
        }))
    }

    pub(super) fn endpoint(&self) -> &ToolNetworkEndpoint {
        &self.endpoint
    }

    fn applies_to(&self, url: &Url) -> bool {
        url.host_str().is_some_and(|host| {
            self.proxied_hosts
                .binary_search_by(|candidate| candidate.as_str().cmp(&host.to_ascii_lowercase()))
                .is_ok()
        })
    }

    fn reqwest_proxy(&self) -> Result<reqwest::Proxy, TransportError> {
        reqwest::Proxy::https(self.url.clone()).map_err(|_| TransportError::UnsupportedProxy)
    }
}

fn first_environment_value(names: &[&str]) -> String {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            return value;
        }
    }
    String::new()
}

fn parse_proxy_url(value: &str) -> Option<Url> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let normalized = if value.contains("://") {
        value.to_owned()
    } else {
        format!("http://{value}")
    };
    let url = Url::parse(&normalized).ok()?;
    matches!(url.scheme(), "http" | "https").then_some(url)
}

// Fixed compatibility targets are hostnames, so the IP/CIDR NO_PROXY cases cannot match them.
// This mirrors hyper-util's domain matcher: exact and subdomain matching are case-insensitive,
// a leading dot is equivalent, and '*' is the sole wildcard.
fn no_proxy_matches(no_proxy: &str, host: &str) -> bool {
    let host = host.trim_matches(['[', ']']);
    no_proxy.split(',').map(str::trim).any(|entry| {
        if entry.is_empty() {
            return false;
        }
        if entry == "*" {
            return true;
        }
        let entry = entry.trim_start_matches('.');
        entry.eq_ignore_ascii_case(host)
            || (host.len() > entry.len()
                && host
                    .get(host.len() - entry.len()..)
                    .is_some_and(|tail| tail.eq_ignore_ascii_case(entry))
                && host.as_bytes().get(host.len() - entry.len() - 1) == Some(&b'.'))
    })
}

pub(super) struct SearchTransport {
    manifest: ToolNetworkManifest,
    authority: Arc<dyn ToolNetworkAuthority>,
    budget: Arc<TransportBudget>,
    dns: Arc<dyn SearchDns>,
    proxy: Option<Arc<TransportProxy>>,
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
    encode_form_with_limit(fields, 16_384)
}

fn encode_form_with_limit(
    fields: &[(String, String)],
    limit: usize,
) -> Result<Vec<u8>, TransportError> {
    // Check exact serializer expansion before allocating the encoded body. Every pair
    // costs '=' plus separators; even empty fields therefore have a bounded count.
    if fields.len() > 8192 {
        return Err(TransportError::InvalidRequest);
    }
    let mut length: usize = 0;
    for (index, (name, value)) in fields.iter().enumerate() {
        length = length
            .checked_add(if index == 0 { 1 } else { 2 })
            .filter(|n| *n <= limit)
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
                    .filter(|n| *n <= limit)
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

/// Immutable payload for one invocation. Preparation is local and grants no authority.
/// Not Clone: execution consumes this object, but the manifest is not globally single-use.
pub(super) struct PreparedTransportRequest {
    invocation_id: uuid::Uuid,
    url: Url,
    method: Method,
    body: Option<Vec<u8>>,
    format: BodyFormat,
    headers: Option<compatibility::headers::CompatibilityHeaders>,
    status_only: bool,
}

impl std::fmt::Debug for PreparedTransportRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedTransportRequest(<redacted>)")
    }
}

fn request_endpoint(url: &Url, method: &Method) -> Result<ToolNetworkEndpoint, TransportError> {
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.as_str().len() > 8192
        || url.query().is_some_and(|q| q.len() > 4096)
    {
        return Err(TransportError::InvalidRequest);
    }
    ToolNetworkEndpoint::new(
        url.scheme(),
        url.host_str().ok_or(TransportError::InvalidRequest)?,
        url.port_or_known_default()
            .ok_or(TransportError::InvalidRequest)?,
        method.as_str(),
        url.path(),
    )
    .map_err(|_| TransportError::InvalidRequest)
}

impl PreparedTransportRequest {
    /// Own already-encoded bytes with the fixed JSON content type; does not parse JSON.
    pub(super) fn json_bytes(
        manifest: &ToolNetworkManifest,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<Self, TransportError> {
        Self::new(manifest, url, method, body, BodyFormat::Json)
    }

    pub(super) fn form(
        manifest: &ToolNetworkManifest,
        url: Url,
        fields: Vec<(String, String)>,
    ) -> Result<Self, TransportError> {
        Self::new(
            manifest,
            url,
            Method::POST,
            Some(encode_form(&fields)?),
            BodyFormat::Form,
        )
    }

    pub(super) fn status_only(
        manifest: &ToolNetworkManifest,
        url: Url,
        method: Method,
    ) -> Result<Self, TransportError> {
        if !matches!(method, Method::GET | Method::HEAD) {
            return Err(TransportError::InvalidRequest);
        }
        let mut request = Self::new(manifest, url, method, None, BodyFormat::Json)?;
        request.status_only = true;
        Ok(request)
    }

    fn new(
        manifest: &ToolNetworkManifest,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
        format: BodyFormat,
    ) -> Result<Self, TransportError> {
        if !matches!(method, Method::GET | Method::HEAD | Method::POST)
            || body.as_ref().is_some_and(|b| b.len() > 16_384)
            || (method != Method::POST && body.is_some())
        {
            return Err(TransportError::InvalidRequest);
        }
        let endpoint = request_endpoint(&url, &method)?;
        if !manifest.scope().endpoints().contains(&endpoint) {
            return Err(TransportError::Unauthorized);
        }
        Ok(Self {
            invocation_id: manifest.invocation_id(),
            url,
            method,
            body,
            format,
            headers: None,
            status_only: false,
        })
    }
}

impl SearchTransport {
    pub(super) fn new(
        manifest: ToolNetworkManifest,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
        dns: Arc<dyn SearchDns>,
    ) -> Result<Self, TransportError> {
        Self::new_with_proxy(manifest, authority, budget, dns, None)
    }

    pub(super) fn new_with_proxy(
        manifest: ToolNetworkManifest,
        authority: Arc<dyn ToolNetworkAuthority>,
        budget: Arc<TransportBudget>,
        dns: Arc<dyn SearchDns>,
        proxy: Option<Arc<TransportProxy>>,
    ) -> Result<Self, TransportError> {
        match (manifest.scope().proxy(), proxy.as_deref()) {
            (Some(declared), Some(snapshot)) => {
                if declared != snapshot.endpoint()
                    || manifest.scope().endpoints().iter().any(|endpoint| {
                        endpoint.scheme() != "https"
                            || !matches!(endpoint.method(), "GET" | "HEAD" | "POST")
                    })
                {
                    return Err(TransportError::UnsupportedProxy);
                }
            }
            (Some(proxy), None)
                if proxy.scheme() != "http"
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
                    }) =>
            {
                // The legacy explicit-scope subset retains its stricter literal target contract.
                return Err(TransportError::UnsupportedProxy);
            }
            (None, Some(_)) => return Err(TransportError::InvalidRequest),
            _ => {}
        }
        Ok(Self {
            manifest,
            authority,
            budget,
            dns,
            proxy,
            #[cfg(test)]
            fixture_roots: Vec::new(),
        })
    }

    fn uses_proxy(&self, url: &Url) -> bool {
        match (&self.proxy, self.manifest.scope().proxy()) {
            (Some(snapshot), Some(_)) => snapshot.applies_to(url),
            (None, Some(_)) => true,
            _ => false,
        }
    }

    fn check(&self, url: &Url, method: &Method) -> Result<ToolNetworkEndpoint, TransportError> {
        self.budget.check_live()?;
        let endpoint = request_endpoint(url, method)?;
        if !self.manifest.scope().endpoints().contains(&endpoint) {
            return Err(TransportError::Unauthorized);
        }
        self.authority
            .check(self.manifest.invocation_id(), &endpoint)
            .map_err(|_| TransportError::Unauthorized)?;
        if self.uses_proxy(url)
            && let Some(proxy) = self.manifest.scope().proxy()
        {
            self.authority
                .check(self.manifest.invocation_id(), proxy)
                .map_err(|_| TransportError::Unauthorized)?;
        }
        self.budget.check_live()?;
        Ok(endpoint)
    }

    /// The future prepared tool invocation must hold this same object across approval.
    /// Invocation identity checks do not prove that a caller constructed it before approval.
    pub(super) async fn execute_prepared(
        &self,
        request: PreparedTransportRequest,
    ) -> Result<TransportResponse, TransportError> {
        if request.invocation_id != self.manifest.invocation_id() {
            return Err(TransportError::Unauthorized);
        }
        self.budget.check_live()?;
        let result = timeout_at(
            self.budget.deadline,
            self.execute_inner(
                request.url,
                request.method,
                request.body,
                request.format,
                request.headers,
                request.status_only,
            ),
        )
        .await
        .map_err(|_| TransportError::Deadline)?;
        self.budget.check_live()?;
        result
    }

    // Legacy raw fixture adapters cannot be called from production adapter code.
    #[cfg(test)]
    async fn execute(
        &self,
        url: Url,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<TransportResponse, TransportError> {
        self.budget.check_live()?;
        let request = PreparedTransportRequest::json_bytes(&self.manifest, url, method, body)?;
        self.execute_prepared(request).await
    }

    #[cfg(test)]
    async fn execute_form(
        &self,
        url: Url,
        fields: Vec<(String, String)>,
    ) -> Result<TransportResponse, TransportError> {
        self.budget.check_live()?;
        let request = PreparedTransportRequest::form(&self.manifest, url, fields)?;
        self.execute_prepared(request).await
    }

    async fn execute_inner(
        &self,
        mut url: Url,
        method: Method,
        body: Option<Vec<u8>>,
        format: BodyFormat,
        headers: Option<compatibility::headers::CompatibilityHeaders>,
        status_only: bool,
    ) -> Result<TransportResponse, TransportError> {
        let mut redirects = 0;
        loop {
            let endpoint = self.check(&url, &method)?; // Denied hosts never reach DNS.
            let uses_proxy = self.uses_proxy(&url);
            // Bound every hop attempt, including failed DNS or a later denied send. A proxied
            // HTTPS request reserves one CONNECT plus one application send.
            TransportBudget::reserve(
                &self.budget.attempts,
                if uses_proxy { 2 } else { 1 },
                self.budget.max_requests,
                TransportError::RequestLimit,
            )?;
            let target_host = endpoint
                .host()
                .trim_start_matches('[')
                .trim_end_matches(']');
            let target_port = endpoint.port();
            let (host, port) = if uses_proxy {
                let proxy = self
                    .manifest
                    .scope()
                    .proxy()
                    .ok_or(TransportError::UnsupportedProxy)?;
                if self.proxy.is_none() {
                    let target = target_host
                        .parse::<IpAddr>()
                        .map_err(|_| TransportError::UnsupportedProxy)?;
                    self.check_addresses(&[SocketAddr::new(target, target_port)], target_port)?;
                }
                (proxy.host().trim_matches(['[', ']']), proxy.port())
            } else {
                (target_host, target_port)
            };
            let addresses = match host.parse::<IpAddr>() {
                Ok(ip) => vec![SocketAddr::new(ip, port)],
                Err(_) => self.dns.resolve(host, port).await?,
            };
            self.budget.check_live()?;
            if uses_proxy && self.proxy.is_some() {
                self.check_proxy_addresses(&addresses, port)?;
            } else {
                self.check_addresses(&addresses, port)?;
            }
            // A fresh HTTP/1 client cannot reuse another authority's connection, coalesce
            // origins, follow redirects or consult ambient proxies. Direct requests pin the
            // target. Proxied requests pin the frozen proxy peer; the trusted configured proxy
            // owns remote hostname resolution exactly as the legacy environment-proxy path did.
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
            let builder = if uses_proxy {
                let proxy = if let Some(snapshot) = &self.proxy {
                    snapshot.reqwest_proxy()?
                } else {
                    let endpoint = self
                        .manifest
                        .scope()
                        .proxy()
                        .ok_or(TransportError::UnsupportedProxy)?;
                    let proxy_url = format!("http://{}:{}/", endpoint.host(), endpoint.port());
                    reqwest::Proxy::https(proxy_url)
                        .map_err(|_| TransportError::UnsupportedProxy)?
                };
                builder.proxy(proxy)
            } else {
                builder
            };
            #[cfg(test)]
            let builder = builder.tls_certs_merge(self.fixture_roots.clone());
            let client = builder.build().map_err(|_| TransportError::Request)?;
            let mut request = client
                .request(method.clone(), url.clone())
                .header(reqwest::header::ACCEPT_ENCODING, "identity");
            if let Some(headers) = headers {
                request = headers.apply(request);
            }
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
            let mut response = client.execute(request).await.map_err(classify_send_error)?;
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
            let status = response.status().as_u16();
            if status_only {
                return Ok(TransportResponse {
                    status,
                    body: Vec::new(),
                });
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
            let mut result = Vec::new();
            loop {
                self.budget.check_live()?;
                let chunk = response.chunk().await.map_err(classify_send_error)?;
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

    fn check_proxy_addresses(
        &self,
        addresses: &[SocketAddr],
        port: u16,
    ) -> Result<(), TransportError> {
        // The proxy endpoint is explicit trusted configuration, so loopback/private peers are
        // allowed just as they were for the legacy environment proxy. Keep only finite answer and
        // exact-port bounds; the proxy hostname itself is still pinned for this connection.
        if addresses.is_empty()
            || addresses.len() > 16
            || addresses.iter().any(|address| address.port() != port)
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
mod proxy_snapshot_tests;
#[cfg(test)]
mod tests;
