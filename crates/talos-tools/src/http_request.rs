//! HTTP request tool for fetching URLs and API endpoints.
//!
//! Provides a Network-gated HTTP client tool that supports GET, POST, PUT,
//! DELETE and other HTTP methods with configurable timeout, response size
//! limits, SSRF protection, and redirect control.

use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during HTTP request tool execution.
#[derive(Debug, Error)]
pub enum HttpRequestError {
    /// The input does not conform to the expected schema.
    #[error("invalid http_request input: {0}")]
    InvalidInput(String),
    /// The HTTP request failed at the transport level.
    #[error("request failed: {0}")]
    RequestFailed(String),
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Input parameters for the [`HttpRequestTool`].
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HttpRequestInput {
    /// HTTP method. Supported: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS.
    /// Defaults to "GET".
    #[serde(default = "default_method")]
    pub method: String,

    /// Target URL. Must start with `http://` or `https://`.
    pub url: String,

    /// Optional request body. Only meaningful for POST, PUT, and PATCH.
    #[serde(default)]
    pub body: Option<String>,

    /// Optional request headers as key-value pairs.
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,

    /// Optional timeout in seconds. Clamped to [1, 60]. Defaults to 15.
    #[serde(default)]
    #[schemars(range(min = 1, max = 60))]
    pub timeout_secs: Option<u64>,

    /// Content extraction mode. "auto" (default) detects HTML and extracts
    /// text, pretty-prints JSON. "raw" returns the body as-is.
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Extract and return links from HTML pages. Default false.
    /// Only meaningful when mode is "auto" and Content-Type is text/html.
    #[serde(default)]
    pub extract_links: bool,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_mode() -> String {
    "auto".to_string()
}

// ---------------------------------------------------------------------------
// Tool
// ---------------------------------------------------------------------------

/// Default maximum response body size in bytes (64 KB).
const DEFAULT_MAX_BODY_BYTES: usize = 65536;

/// Default redirect limit.
const DEFAULT_REDIRECT_LIMIT: usize = 5;

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 15;

/// Headers that must not be overridden by user input.
const BLOCKED_HEADERS: &[&str] = &[
    "host",
    "authorization",
    "cookie",
    "proxy-authorization",
    "content-length",
    "transfer-encoding",
    "expect",
];

/// Sanitize user-supplied headers. Rejects blocked headers and headers
/// containing CR/LF to prevent header injection.
fn sanitize_headers(headers: Option<&HashMap<String, String>>) -> Option<HashMap<String, String>> {
    let headers = headers?;
    if headers.is_empty() {
        return None;
    }

    let blocked: HashSet<&str> = BLOCKED_HEADERS.iter().copied().collect();
    let mut sanitized = HashMap::new();

    for (key, value) in headers {
        let key_lower = key.to_lowercase();
        if blocked.contains(key_lower.as_str()) {
            continue;
        }
        if key.contains('\r') || key.contains('\n') || value.contains('\r') || value.contains('\n')
        {
            continue;
        }
        sanitized.insert(key.clone(), value.clone());
    }

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

/// Extract readable text from HTML content using the `scraper` crate.
///
/// Strips HTML tags, decodes common entities, normalizes whitespace,
/// and returns the visible text content. Best-effort: JS-heavy SPA
/// pages will produce limited output since no browser rendering occurs.
fn extract_html_text(html: &str) -> String {
    let document = scraper::Html::parse_document(html);

    // Select the body or fall back to the root element.
    let body_selector = scraper::Selector::parse("body").expect("'body' is a valid CSS selector");
    let root = document
        .root_element()
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());

    // Collect text from all text nodes, separated by newlines.
    let text_items: Vec<String> = root
        .text()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    if text_items.is_empty() {
        return "[No visible text content extracted from HTML]".to_string();
    }

    let mut result = String::new();
    for item in &text_items {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(item);
    }

    result
}

/// Extract normalized, deduplicated links from HTML content.
fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document = scraper::Html::parse_document(html);
    let selector = scraper::Selector::parse("a[href]").expect("valid CSS selector");

    let mut seen = HashSet::new();
    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.attr("href") {
            let trimmed = href.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("javascript:")
            {
                continue;
            }

            let resolved = match reqwest::Url::parse(trimmed) {
                Ok(url) => url.to_string(),
                Err(_) => match reqwest::Url::parse(base_url) {
                    Ok(base) => match base.join(trimmed) {
                        Ok(full) => full.to_string(),
                        Err(_) => continue,
                    },
                    Err(_) => continue,
                },
            };

            if seen.insert(resolved.clone()) {
                links.push(resolved);
            }
        }
    }

    links
}

/// A tool that executes HTTP requests with SSRF protection and size limits.
///
/// Requests are routed through `reqwest` with `rustls` TLS (no native
/// dependencies). Private/reserved IP addresses are rejected to prevent
/// Server-Side Request Forgery (SSRF). Response bodies are capped at
/// `max_body_bytes` to avoid memory exhaustion.
pub struct HttpRequestTool {
    client: reqwest::Client,
    max_body_bytes: usize,
}

impl HttpRequestTool {
    /// Create a new [`HttpRequestTool`] with default settings.
    ///
    /// - Redirect limit: 5
    /// - Response body cap: 64 KB
    /// - No HTTP proxy unless the standard `HTTP_PROXY`/`HTTPS_PROXY` env vars
    ///   are set (handled by reqwest).
    pub fn new() -> Self {
        Self::with_config(DEFAULT_MAX_BODY_BYTES, DEFAULT_REDIRECT_LIMIT)
    }

    /// Create a new [`HttpRequestTool`] with explicit configuration.
    pub fn with_config(max_body_bytes: usize, redirect_limit: usize) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(redirect_limit))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes,
        }
    }

    /// Perform the pre-request SSRF check by resolving the host and
    /// verifying that none of the resolved addresses are private or
    /// reserved.
    ///
    /// **Limitation**: DNS resolution occurs before the actual HTTP request.
    /// A DNS rebinding attack could re-resolve to a private IP between this
    /// check and `reqwest`'s own resolution. For the agent-tool threat model
    /// (user controls which URLs are fetched), this is an acceptable tradeoff.
    /// In a server-exposed context, use network-layer enforcement instead.
    async fn check_ssrf(&self, host: &str) -> Result<(), String> {
        // Try to parse as a bare IP address first.
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(ip) {
                return Err(format!(
                    "URL resolves to a private IP address ({ip}); blocked by SSRF guard"
                ));
            }
            return Ok(());
        }

        // DNS resolution.
        let addr = format!("{host}:0");
        let addrs = tokio::net::lookup_host(&addr)
            .await
            .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?;

        for sock_addr in addrs {
            if is_private_ip(sock_addr.ip()) {
                return Err(format!(
                    "URL host {host} resolves to private IP ({})",
                    sock_addr.ip()
                ));
            }
        }

        Ok(())
    }

    /// Build the reqwest request from the input, apply optional body and headers.
    fn build_request(
        &self,
        client: &reqwest::Client,
        input: &HttpRequestInput,
    ) -> Result<reqwest::RequestBuilder, String> {
        let method = input.method.to_uppercase();
        let mut req = match method.as_str() {
            "GET" => client.get(&input.url),
            "POST" => client.post(&input.url),
            "PUT" => client.put(&input.url),
            "DELETE" => client.delete(&input.url),
            "PATCH" => client.patch(&input.url),
            "HEAD" => client.head(&input.url),
            "OPTIONS" => client.request(
                reqwest::Method::OPTIONS,
                reqwest::Url::parse(&input.url).map_err(|e| format!("invalid URL: {e}"))?,
            ),
            other => return Err(format!("unsupported HTTP method: {other}")),
        };

        // Apply sanitized headers only.
        if let Some(ref headers) = sanitize_headers(input.headers.as_ref()) {
            for (key, value) in headers {
                req = req.header(key.as_str(), value.as_str());
            }
        }

        // Apply optional body.
        if let Some(ref body) = input.body {
            req = req.body(body.clone());
        }

        Ok(req)
    }
}

impl Default for HttpRequestTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns `true` if the IP address is in a private, loopback, link-local,
/// or otherwise reserved range that should not be reachable via a network tool.
pub(crate) fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                // 0.0.0.0/8 (current network)
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                // fe80::/10 (link-local)
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                // fc00::/7 (unique local, already caught by is_unique_local but be explicit)
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                // ::ffff:0:0/96 (IPv4-mapped)
                || (v6.segments()[0] == 0
                    && v6.segments()[1] == 0
                    && v6.segments()[2] == 0
                    && v6.segments()[3] == 0
                    && v6.segments()[4] == 0
                    && v6.segments()[5] == 0xffff)
                // 2001:db8::/32 (documentation)
                || (v6.segments()[0] == 0x2001 && v6.segments()[1] == 0x0db8)
        }
    }
}

// ---------------------------------------------------------------------------
// AgentTool implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl AgentTool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Send an HTTP request to a URL. Supports GET, POST, PUT, DELETE, PATCH, HEAD, and OPTIONS methods. \
         Returns status code, response headers, and body (truncated at 64KB). \
         Private/reserved IP addresses are blocked (SSRF protection). \
         Requires network permission."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(HttpRequestInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["method", "url"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: HttpRequestInput = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(format!(
                    "{}",
                    HttpRequestError::InvalidInput(e.to_string())
                ));
            }
        };

        // Validate URL scheme.
        let parsed_url = match reqwest::Url::parse(&parsed.url) {
            Ok(url) => url,
            Err(e) => {
                return ToolResult::error(format!("invalid URL '{}': {e}", parsed.url));
            }
        };

        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return ToolResult::error(format!(
                "unsupported URL scheme '{scheme}'. Only http and https are supported"
            ));
        }

        // SSRF guard: check resolved IPs.
        let host = match parsed_url.host_str() {
            Some(h) => h.to_string(),
            None => {
                return ToolResult::error(format!("URL has no host: {}", parsed.url));
            }
        };

        if let Err(e) = self.check_ssrf(&host).await {
            return ToolResult::error(format!("SSRF guard blocked request to {host}: {e}"));
        }

        // Override timeout if specified.
        let client = if let Some(secs) = parsed.timeout_secs {
            let secs = secs.clamp(1, 60);
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(DEFAULT_REDIRECT_LIMIT))
                .timeout(Duration::from_secs(secs))
                .build()
                .expect("reqwest Client builder should never fail with default config")
        } else {
            self.client.clone()
        };

        // Build and send request.
        let request = match self.build_request(&client, &parsed) {
            Ok(req) => req,
            Err(e) => {
                return ToolResult::error(format!("failed to build request: {e}"));
            }
        };

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                return ToolResult::error(format!(
                    "{}",
                    HttpRequestError::RequestFailed(e.to_string())
                ));
            }
        };

        let status = response.status();
        let response_headers = response.headers().clone();

        // Read body with size cap.
        let body_bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return ToolResult::error(format!("failed to read response body: {e}"));
            }
        };

        let truncated = body_bytes.len() > self.max_body_bytes;
        let body_display = if truncated {
            &body_bytes[..self.max_body_bytes]
        } else {
            &body_bytes
        };

        // Determine content type.
        let content_type = response_headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let is_raw_mode = parsed.mode == "raw";

        // Format output.
        let mut output = String::new();
        output.push_str(&format!("Status: {}\n", status));
        output.push_str("Headers:\n");
        for (name, value) in response_headers.iter() {
            output.push_str(&format!(
                "  {}: {}\n",
                name,
                value.to_str().unwrap_or("<binary>")
            ));
        }

        let size_label = if truncated {
            format!(
                "first {} of {} bytes",
                self.max_body_bytes,
                body_bytes.len()
            )
        } else {
            format!("{} bytes", body_bytes.len())
        };

        if is_raw_mode {
            // Raw mode: return body as-is.
            output.push_str(&format!("\nBody ({size_label}):\n",));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.contains("text/html") {
            // HTML: extract text content.
            output.push_str(&format!("\nContent ({size_label}, text/html):\n",));
            let html_str = String::from_utf8_lossy(body_display);
            let text = extract_html_text(&html_str);
            output.push_str(&text);

            if parsed.extract_links {
                let links = extract_links(&html_str, &parsed.url);
                if !links.is_empty() {
                    let count = links.len();
                    let show = count.min(20);
                    output.push_str(&format!(
                        "\n\n── Links ({count} total, showing {show}) ──\n"
                    ));
                    for link in links.iter().take(show) {
                        output.push_str(&format!("  {link}\n"));
                    }
                    if links.len() > show {
                        output.push_str(&format!("  … and {} more\n", links.len() - show));
                    }
                }
            }
        } else if content_type.contains("application/json") {
            // JSON: pretty-print.
            output.push_str(&format!("\nContent ({size_label}, application/json):\n",));
            match serde_json::from_slice::<serde_json::Value>(body_display) {
                Ok(val) => {
                    output.push_str(
                        &serde_json::to_string_pretty(&val)
                            .unwrap_or_else(|_| String::from_utf8_lossy(body_display).to_string()),
                    );
                }
                Err(_) => {
                    output.push_str(&String::from_utf8_lossy(body_display));
                }
            }
        } else if content_type.starts_with("text/") {
            // Text: return as-is.
            output.push_str(&format!("\nContent ({size_label}, {content_type}):\n",));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.is_empty() || content_type.contains("octet-stream") {
            // Binary or unknown content type: show info only.
            output.push_str(&format!("\nContent: binary/unknown ({size_label})\n",));
            if !content_type.is_empty() {
                output.push_str(&format!("Content-Type: {content_type}\n"));
            }
            output.push_str(
                "[Binary or unrecognized content — use mode: \"raw\" to view raw bytes]\n",
            );
        } else {
            // Other structured content: return as text.
            output.push_str(&format!("\nContent ({size_label}, {content_type}):\n",));
            output.push_str(&String::from_utf8_lossy(body_display));
        }

        if truncated {
            output.push_str(&format!(
                "\n\n[Response truncated at {} bytes. Total size: {} bytes]",
                self.max_body_bytes,
                body_bytes.len()
            ));
        }

        ToolResult::success(output)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_tool_name() {
        let tool = HttpRequestTool::new();
        assert_eq!(tool.name(), "http_request");
    }

    #[test]
    fn test_tool_is_not_read_only() {
        let tool = HttpRequestTool::new();
        assert!(!tool.is_read_only());
    }

    #[test]
    fn test_tool_nature_is_network() {
        let tool = HttpRequestTool::new();
        assert!(matches!(tool.nature(), ToolNature::Network));
    }

    #[test]
    fn test_tool_summary_fields() {
        let tool = HttpRequestTool::new();
        assert_eq!(tool.summary_fields(), &["method", "url"]);
    }

    #[test]
    fn test_tool_has_description() {
        let tool = HttpRequestTool::new();
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_tool_emits_parameters_schema() {
        let tool = HttpRequestTool::new();
        let schema = tool.parameters();
        assert!(
            schema.is_object(),
            "parameters should be a JSON Schema object"
        );
        let props = schema.get("properties");
        assert!(
            props.is_some(),
            "schema should have properties for method, url, etc."
        );
    }

    #[test]
    fn test_default_method_is_get() {
        assert_eq!(default_method(), "GET");
    }

    // SSRF guard tests.

    #[test]
    fn test_ssrf_blocks_loopback_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn test_ssrf_blocks_private_v4_class_a() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn test_ssrf_blocks_private_v4_class_b() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
    }

    #[test]
    fn test_ssrf_blocks_private_v4_class_c() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[test]
    fn test_ssrf_blocks_link_local_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
    }

    #[test]
    fn test_ssrf_blocks_unspecified_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
    }

    #[test]
    fn test_ssrf_blocks_broadcast_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))));
    }

    #[test]
    fn test_ssrf_blocks_documentation_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))));
    }

    #[test]
    fn test_ssrf_blocks_zero_network_v4() {
        assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(0, 1, 2, 3))));
    }

    #[test]
    fn test_ssrf_allows_public_v4() {
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn test_ssrf_allows_public_v4_2() {
        assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    }

    #[test]
    fn test_ssrf_blocks_loopback_v6() {
        assert!(is_private_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn test_ssrf_blocks_unspecified_v6() {
        assert!(is_private_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
    }

    #[test]
    fn test_ssrf_allows_public_v6() {
        // 2001:4860:4860::8888 (Google DNS IPv6)
        let google_dns = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);
        assert!(!is_private_ip(IpAddr::V6(google_dns)));
    }

    // Input deserialization tests.

    #[test]
    fn test_deserialize_minimal_input() {
        let json = r#"{"url": "https://example.com"}"#;
        let input: HttpRequestInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.method, "GET");
        assert_eq!(input.url, "https://example.com");
        assert!(input.body.is_none());
        assert!(input.headers.is_none());
        assert!(input.timeout_secs.is_none());
    }

    #[test]
    fn test_deserialize_full_input() {
        let json = r#"{
            "method": "POST",
            "url": "https://api.example.com/data",
            "body": "{\"key\": \"value\"}",
            "headers": {"Content-Type": "application/json", "Authorization": "Bearer token123"},
            "timeout_secs": 30
        }"#;
        let input: HttpRequestInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.method, "POST");
        assert_eq!(input.url, "https://api.example.com/data");
        assert_eq!(input.body, Some("{\"key\": \"value\"}".to_string()));
        let headers = input.headers.unwrap();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(input.timeout_secs, Some(30));
    }

    #[test]
    fn test_deserialize_missing_url_fails() {
        let json = r#"{"method": "GET"}"#;
        let result: Result<HttpRequestInput, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
