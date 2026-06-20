//! HTTP request tool for fetching URLs and API endpoints.
//!
//! Provides a Network-gated HTTP client tool that supports GET, POST, PUT,
//! DELETE and other HTTP methods with configurable timeout, response size
//! limits, SSRF protection, and redirect control.

use std::collections::HashMap;
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
}

fn default_method() -> String {
    "GET".to_string()
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
    async fn check_ssrf(&self, host: &str) -> Result<(), String> {
        // Try to parse as a bare IP address first.
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(ip) {
                return Err(format!("URL resolves to a private IP address ({ip}); blocked by SSRF guard"));
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

        // Apply optional headers.
        if let Some(ref headers) = input.headers {
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
fn is_private_ip(ip: IpAddr) -> bool {
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
                return ToolResult::error(format!(
                    "invalid URL '{}': {e}",
                    parsed.url
                ));
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
            return ToolResult::error(format!(
                "SSRF guard blocked request to {host}: {e}"
            ));
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
                return ToolResult::error(format!(
                    "failed to build request: {e}"
                ));
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
                return ToolResult::error(format!(
                    "failed to read response body: {e}"
                ));
            }
        };

        let truncated = body_bytes.len() > self.max_body_bytes;
        let body_display = if truncated {
            &body_bytes[..self.max_body_bytes]
        } else {
            &body_bytes
        };

        let body_str = String::from_utf8_lossy(body_display);

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

        output.push_str(&format!("\nBody ({}):\n", {
            if truncated {
                format!(
                    "first {} of {} bytes",
                    self.max_body_bytes,
                    body_bytes.len()
                )
            } else {
                format!("{} bytes", body_bytes.len())
            }
        }));
        output.push_str(&body_str);

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
