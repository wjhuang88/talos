//! Unified URL fetch tool for bounded model context ingestion.

use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolContinuation, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind,
    ToolResult,
};
use thiserror::Error;

use crate::http_request::{check_ssrf_host, extract_html_text, extract_links};

const DEFAULT_MAX_BODY_BYTES: usize = 65536;
const DEFAULT_REDIRECT_LIMIT: usize = 5;
const DEFAULT_TIMEOUT_SECS: u64 = 15;

/// Errors that can occur during URL fetch execution.
#[derive(Debug, Error)]
pub enum FetchUrlError {
    /// The input does not conform to the expected schema.
    #[error("invalid fetch_url input: {0}")]
    InvalidInput(String),
    /// The HTTP request failed at the transport level.
    #[error("request failed: {0}")]
    RequestFailed(String),
}

/// Input parameters for [`FetchUrlTool`].
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchUrlInput {
    /// Target URL. Must start with `http://` or `https://`.
    pub url: String,

    /// Content extraction mode. "auto" detects HTML and extracts text,
    /// pretty-prints JSON, and returns raw text for text/* responses. "raw"
    /// returns the body as-is.
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Extract and return links from HTML pages. Default false.
    #[serde(default)]
    pub extract_links: bool,

    /// Optional timeout in seconds. Clamped to [1, 60]. Defaults to 15.
    #[serde(default)]
    #[schemars(range(min = 1, max = 60))]
    pub timeout_secs: Option<u64>,
}

fn default_mode() -> String {
    "auto".to_string()
}

/// A model-facing URL reader that converts HTTP responses into bounded context.
pub struct FetchUrlTool {
    client: reqwest::Client,
    max_body_bytes: usize,
    #[cfg(test)]
    skip_ssrf: bool,
}

impl FetchUrlTool {
    /// Creates a new [`FetchUrlTool`] with default limits.
    pub fn new() -> Self {
        Self::with_config(DEFAULT_MAX_BODY_BYTES, DEFAULT_REDIRECT_LIMIT)
    }

    /// Creates a new [`FetchUrlTool`] with explicit limits.
    pub fn with_config(max_body_bytes: usize, redirect_limit: usize) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(redirect_limit))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes,
            #[cfg(test)]
            skip_ssrf: false,
        }
    }

    /// Creates a [`FetchUrlTool`] that skips SSRF checks (test-only).
    #[cfg(test)]
    pub fn for_testing(max_body_bytes: usize, redirect_limit: usize) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(redirect_limit))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes,
            skip_ssrf: true,
        }
    }

    /// Creates a [`FetchUrlTool`] that skips SSRF and does not follow redirects (test-only).
    #[cfg(test)]
    pub fn for_testing_no_redirect(max_body_bytes: usize) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes,
            skip_ssrf: true,
        }
    }
}

impl Default for FetchUrlTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTool for FetchUrlTool {
    fn name(&self) -> &str {
        "fetch_url"
    }

    fn description(&self) -> &str {
        "Fetch a URL for model context. Uses static HTTP with SSRF protection, detects common \
         content types, extracts readable HTML text, pretty-prints JSON, and can include bounded \
         HTML links. Use http_request only when custom methods, headers, bodies, or low-level HTTP \
         inspection are needed."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(FetchUrlInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Network
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        if let Some(url) = input.get("url").and_then(Value::as_str)
            && let Ok(parsed) = reqwest::Url::parse(url)
            && let Some(host) = parsed.host_str()
        {
            return vec![ToolPermissionFacet::with_resource(
                ToolNature::Network,
                host.to_lowercase(),
                ToolResourceKind::Domain,
            )];
        }
        vec![ToolPermissionFacet::new(ToolNature::Network)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["url", "mode"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: FetchUrlInput = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(FetchUrlError::InvalidInput(e.to_string()).to_string());
            }
        };

        if parsed.mode != "auto" && parsed.mode != "raw" {
            return ToolResult::error(
                FetchUrlError::InvalidInput("mode must be either \"auto\" or \"raw\"".to_string())
                    .to_string(),
            );
        }

        let parsed_url = match reqwest::Url::parse(&parsed.url) {
            Ok(url) => url,
            Err(e) => return ToolResult::error(format!("invalid URL '{}': {e}", parsed.url)),
        };

        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return ToolResult::error(format!(
                "unsupported URL scheme '{scheme}'. Only http and https are supported"
            ));
        }

        let host = match parsed_url.host_str() {
            Some(host) => host.to_string(),
            None => return ToolResult::error(format!("URL has no host: {}", parsed.url)),
        };

        #[cfg(not(test))]
        let ssrf_blocked = check_ssrf_host(&host).await.err();
        #[cfg(test)]
        let ssrf_blocked = if self.skip_ssrf {
            None
        } else {
            check_ssrf_host(&host).await.err()
        };
        if let Some(e) = ssrf_blocked {
            return ToolResult::error(format!("SSRF guard blocked request to {host}: {e}"));
        }

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

        let response = match client.get(parsed.url.as_str()).send().await {
            Ok(response) => response,
            Err(e) => {
                return ToolResult::error(FetchUrlError::RequestFailed(e.to_string()).to_string());
            }
        };

        let status = response.status();
        let response_headers = response.headers().clone();
        let final_url = response.url().clone();
        let body_bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return ToolResult::error(format!("failed to read response body: {e}")),
        };

        let truncated = body_bytes.len() > self.max_body_bytes;
        let body_display = if truncated {
            &body_bytes[..self.max_body_bytes]
        } else {
            &body_bytes
        };
        let content_type = response_headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        let size_label = if truncated {
            format!(
                "first {} of {} bytes",
                self.max_body_bytes,
                body_bytes.len()
            )
        } else {
            format!("{} bytes", body_bytes.len())
        };

        let redirected = final_url.as_str() != parsed.url;

        let mut output = String::new();
        output.push_str(&format!("URL: {}\n", parsed.url));
        output.push_str(&format!("Status: {status}\n"));
        output.push_str(&format!("Content-Type: {content_type}\n"));

        if redirected {
            output.push_str(&format!("[redirected: {} → {}]\n", parsed.url, final_url));
        }
        if !content_type.is_empty() {
            output.push_str(&format!("[content-type: {content_type}]\n"));
        }

        if parsed.mode == "raw" {
            output.push_str(&format!("\nBody ({size_label}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.contains("text/html") {
            output.push_str(&format!("\nContent ({size_label}, text/html):\n"));
            let html = String::from_utf8_lossy(body_display);
            let extracted_text = extract_html_text(&html);
            output.push_str(&extracted_text);

            let html_body_len = html.len();
            let text_len = extracted_text.chars().count();
            if text_len < 100 && html_body_len > 2000 {
                output.push_str(
                    "\n\n[Sparse HTML: page may be client-rendered (JavaScript/SPA). \
                     Use http_request with appropriate headers or a browser-based tool for full content.]",
                );
            }

            if parsed.extract_links {
                let links = extract_links(&html, &parsed.url);
                if !links.is_empty() {
                    let count = links.len();
                    let shown = count.min(20);
                    output.push_str(&format!("\n\nLinks ({count} total, showing {shown}):\n"));
                    for link in links.iter().take(shown) {
                        output.push_str(&format!("- {link}\n"));
                    }
                    if count > shown {
                        output.push_str(&format!("- ... and {} more\n", count - shown));
                    }
                }
            }
        } else if content_type.contains("application/json") {
            output.push_str(&format!("\nContent ({size_label}, application/json):\n"));
            match serde_json::from_slice::<serde_json::Value>(body_display) {
                Ok(value) => output.push_str(
                    &serde_json::to_string_pretty(&value)
                        .unwrap_or_else(|_| String::from_utf8_lossy(body_display).to_string()),
                ),
                Err(_) => output.push_str(&String::from_utf8_lossy(body_display)),
            }
        } else if content_type.starts_with("text/") {
            output.push_str(&format!("\nContent ({size_label}, {content_type}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.is_empty() || content_type.contains("octet-stream") {
            output.push_str(&format!("\nContent: binary/unknown ({size_label})\n"));
            output.push_str(
                "[Binary or unrecognized content - use save_url to persist bytes or http_request \
                 for low-level inspection]\n",
            );
        } else {
            output.push_str(&format!("\nContent ({size_label}, {content_type}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        }

        if truncated {
            output.push_str(&format!(
                "\n\n[Response truncated at {} bytes. Total size: {} bytes]",
                self.max_body_bytes,
                body_bytes.len()
            ));
        }

        if status.is_redirection() || content_type.contains("text/html") && output.len() < 200 {
            output.push_str("\n\n[Need custom headers, method, body, or lower-level HTTP inspection? Retry with http_request.]");
            return ToolResult::success(output).with_continuation(ToolContinuation::disclose_tool(
                "http_request",
                "advanced_http_required",
            ));
        }

        ToolResult::success(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn fetch_url_tool_metadata() {
        let tool = FetchUrlTool::new();
        assert_eq!(tool.name(), "fetch_url");
        assert_eq!(tool.family(), ToolFamily::Network);
        assert_eq!(tool.summary_fields(), &["url", "mode"]);
    }

    #[test]
    fn permission_profile_uses_url_host() {
        let tool = FetchUrlTool::new();
        let profile = tool.permission_profile(&serde_json::json!({
            "url": "https://Example.com/path"
        }));
        assert_eq!(
            profile,
            vec![ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "example.com",
                ToolResourceKind::Domain
            )]
        );
    }

    #[test]
    fn extract_html_text_returns_empty_for_script_only() {
        let html = r#"<html><head><script>var x=1;</script></head><body></body></html>"#;
        let text = extract_html_text(html);
        assert!(text.len() < 100 || text.contains("No visible text"));
    }

    #[test]
    fn extract_html_text_extracts_body_text() {
        let html = r#"<html><body><h1>Hello</h1><p>World</p></body></html>"#;
        let text = extract_html_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn extract_html_text_large_html_minimal_text() {
        let mut html = String::from("<html><head>");
        for i in 0..200 {
            html.push_str(&format!("<script>var x{i}={i};</script>\n"));
        }
        html.push_str("</head><body><div id=\"app\"></div></body></html>");
        assert!(html.len() > 2000);
        let text = extract_html_text(&html);
        assert!(text.chars().count() < 100);
    }

    fn serve_http(listener: &TcpListener, response: &str) {
        let (mut stream, _) = listener.accept().expect("accept failed");
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap();
        stream.write_all(response.as_bytes()).expect("write failed");
        stream.flush().expect("flush failed");
    }

    fn serve_two_responses(listener: &TcpListener, response1: &str, response2: &str) {
        let (mut stream, _) = listener.accept().expect("accept failed");
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap();
        stream
            .write_all(response1.as_bytes())
            .expect("write failed");
        stream.flush().expect("flush failed");
        drop(stream);

        let (mut stream, _) = listener.accept().expect("accept failed");
        let _ = stream.read(&mut buf).unwrap();
        stream
            .write_all(response2.as_bytes())
            .expect("write failed");
        stream.flush().expect("flush failed");
    }

    #[tokio::test]
    async fn redirect_diagnostics_shows_final_url() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let redirect_response = format!(
            "HTTP/1.1 302 Found\r\n\
             Location: http://127.0.0.1:{port}/final\r\n\
             Content-Type: text/html\r\n\
             Content-Length: 0\r\n\r\n"
        );
        let final_response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: 27\r\n\r\n\
             <html><body>Hello World</body></html>"
        );

        let server = thread::spawn(move || {
            serve_two_responses(&listener, &redirect_response, &final_response);
        });

        let tool = FetchUrlTool::for_testing(65536, 5);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/start")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(
            result.content.contains("[redirected:"),
            "expected redirect diagnostic, got: {}",
            result.content
        );
        assert!(
            result.content.contains("→"),
            "expected redirect arrow, got: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn content_type_summary_in_output() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: 27\r\n\r\n\
             <html><body>Hello World</body></html>"
        );

        let server = thread::spawn(move || {
            serve_http(&listener, &response);
        });

        let tool = FetchUrlTool::for_testing(65536, 0);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(
            result
                .content
                .contains("[content-type: text/html; charset=utf-8]"),
            "expected content-type summary, got: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn sparse_html_hint_for_spa_content() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let mut html_body = String::from("<html><head>");
        for i in 0..200 {
            html_body.push_str(&format!("<script>var x{i}={i};</script>\n"));
        }
        html_body.push_str("</head><body><div id=\"app\"></div></body></html>");
        let body_len = html_body.len();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {body_len}\r\n\r\n\
             {html_body}"
        );

        let server = thread::spawn(move || {
            serve_http(&listener, &response);
        });

        let tool = FetchUrlTool::for_testing(65536, 0);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(
            result.content.contains("[Sparse HTML:"),
            "expected sparse HTML hint, got: {}",
            result.content
        );
        assert!(
            result.content.contains("client-rendered"),
            "expected client-rendered mention, got: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn continuation_emitted_for_redirect_status() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let redirect_response = format!(
            "HTTP/1.1 302 Found\r\n\
             Location: http://127.0.0.1:{port}/final\r\n\
             Content-Type: text/html\r\n\
             Content-Length: 0\r\n\r\n"
        );

        let server = thread::spawn(move || {
            serve_http(&listener, &redirect_response);
        });

        let tool = FetchUrlTool::for_testing_no_redirect(65536);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/start")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(result.content.contains("302"));
        assert_eq!(result.continuations.len(), 1);
        let cont = &result.continuations[0];
        assert_eq!(cont.tool, "http_request");
        assert_eq!(cont.reason, "advanced_http_required");
        assert!(cont.is_tool_disclosure());
    }

    #[tokio::test]
    async fn continuation_emitted_for_sparse_html() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let mut html_body = String::from("<html><head>");
        for i in 0..200 {
            html_body.push_str(&format!("<script>var x{i}={i};</script>\n"));
        }
        html_body.push_str("</head><body><div id=\"app\"></div></body></html>");
        let body_len = html_body.len();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {body_len}\r\n\r\n\
             {html_body}"
        );

        let server = thread::spawn(move || {
            serve_http(&listener, &response);
        });

        let tool = FetchUrlTool::for_testing(65536, 0);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(
            result.content.contains("[Sparse HTML:"),
            "expected sparse HTML hint, got: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn no_continuation_for_normal_html() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let port = listener.local_addr().unwrap().port();

        let body = "<html><body><h1>Welcome to Our Website</h1>\
                    <p>This is a normal page with enough text content to exceed the \
                    two hundred character threshold that triggers the continuation \
                    mechanism. We need sufficient body text here to ensure the output \
                    string grows beyond the limit. Additional paragraph with more \
                    details about the page content and its purpose for testing.</p>\
                    </body></html>";
        let body_len = body.len();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {body_len}\r\n\r\n\
             {body}"
        );

        let server = thread::spawn(move || {
            serve_http(&listener, &response);
        });

        let tool = FetchUrlTool::for_testing(65536, 0);
        let result = tool
            .execute(serde_json::json!({
                "url": format!("http://127.0.0.1:{port}/")
            }))
            .await;

        server.join().unwrap();

        assert!(!result.is_error);
        assert!(
            result.continuations.is_empty(),
            "expected no continuation for normal HTML, got: {:?}",
            result.continuations
        );
    }
}
