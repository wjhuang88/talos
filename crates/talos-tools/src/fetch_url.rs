//! Content-fetch tool: extracts readable text and links from web pages.
//!
//! The `fetch_url` tool takes a URL, fetches it via HTTP, detects the
//! content type, and returns token-efficient content to the model. For
//! HTML pages, it extracts visible text and collects high-value links.
//! Reuses the same SSRF guard and size limits as `http_request`.

use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolNature, ToolResult};
use thiserror::Error;

use crate::http_request;

const DEFAULT_MAX_BODY_BYTES: usize = 65536;
const DEFAULT_TIMEOUT_SECS: u64 = 15;
const MAX_LINKS_RETURNED: usize = 20;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum FetchUrlError {
    #[error("invalid fetch_url input: {0}")]
    InvalidInput(String),
    #[error("fetch failed: {0}")]
    FetchFailed(String),
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchUrlInput {
    /// The URL to fetch.
    pub url: String,

    /// Mode: "auto" (default) extracts content and links. "raw" returns as-is.
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Maximum number of links to return. Default 10, max 20.
    #[serde(default = "default_max_links")]
    #[schemars(range(min = 1, max = 20))]
    pub max_links: u32,
}

fn default_mode() -> String {
    "auto".to_string()
}

fn default_max_links() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Tool
// ---------------------------------------------------------------------------

pub struct FetchUrlTool {
    client: reqwest::Client,
    max_body_bytes: usize,
}

impl FetchUrlTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("reqwest Client builder should never fail with default config");

        Self {
            client,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }

    /// Extract links from HTML content. Returns deduplicated, normalized URLs.
    fn extract_links(html: &str, base_url: &str) -> Vec<String> {
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse("a[href]").expect("valid CSS selector");

        let mut seen = HashSet::new();
        let mut links = Vec::new();

        for element in document.select(&selector) {
            if let Some(href) = element.attr("href") {
                let trimmed = href.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("javascript:") {
                    continue;
                }

                let resolved = match reqwest::Url::parse(trimmed) {
                    Ok(url) => url.to_string(),
                    Err(_) => {
                        match reqwest::Url::parse(base_url) {
                            Ok(base) => {
                                match base.join(trimmed) {
                                    Ok(full) => full.to_string(),
                                    Err(_) => continue,
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                };

                if seen.insert(resolved.clone()) {
                    links.push(resolved);
                }
            }
        }

        links
    }

    /// Extract visible text from HTML, without tags.
    fn extract_text(html: &str) -> String {
        let document = scraper::Html::parse_document(html);
        let body_selector = scraper::Selector::parse("body").expect("valid CSS selector");
        let root = document
            .root_element()
            .select(&body_selector)
            .next()
            .unwrap_or_else(|| document.root_element());

        let text_items: Vec<String> = root
            .text()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        if text_items.is_empty() {
            return "[No visible text content]".to_string();
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
        "Fetch a URL and return extracted content. For HTML pages, extracts readable text and \
         collects links. For JSON APIs, returns pretty-printed JSON. For other content types, \
         returns the raw response. SSRF-protected. Requires network permission."
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

    fn summary_fields(&self) -> &'static [&'static str] {
        &["url"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: FetchUrlInput = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(FetchUrlError::InvalidInput(e.to_string()).to_string());
            }
        };

        // Validate URL.
        let parsed_url = match reqwest::Url::parse(&parsed.url) {
            Ok(url) => url,
            Err(e) => return ToolResult::error(format!("invalid URL: {e}")),
        };

        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return ToolResult::error(format!("unsupported URL scheme '{scheme}'"));
        }

        // SSRF guard (reuse http_request's logic).
        let host = match parsed_url.host_str() {
            Some(h) => h.to_string(),
            None => return ToolResult::error("URL has no host".to_string()),
        };

        if let Err(e) = check_ssrf(&host).await {
            return ToolResult::error(format!("SSRF guard blocked: {e}"));
        }

        // Fetch.
        let response = match self.client.get(parsed.url.as_str()).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return ToolResult::error(FetchUrlError::FetchFailed(e.to_string()).to_string());
            }
        };

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

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

        let is_raw = parsed.mode == "raw";
        let size_label = if truncated {
            format!("first {} of {} bytes", self.max_body_bytes, body_bytes.len())
        } else {
            format!("{} bytes", body_bytes.len())
        };

        let mut output = format!("URL: {}\nStatus: {}\n", parsed.url, status);

        if is_raw {
            output.push_str(&format!("Content ({size_label}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.contains("text/html") {
            let html_str = String::from_utf8_lossy(body_display);
            let text = Self::extract_text(&html_str);
            let links = Self::extract_links(&html_str, &parsed.url);

            output.push_str(&format!("Content ({size_label}, text/html):\n\n"));
            output.push_str(&text);
            output.push('\n');

            if !links.is_empty() {
                let count = links.len();
                let show = links.len().min(parsed.max_links as usize).min(MAX_LINKS_RETURNED);
                output.push_str(&format!("\n── Links ({count} total, showing {show}) ──\n"));
                for link in links.iter().take(show) {
                    output.push_str(&format!("  {link}\n"));
                }
                if links.len() > show {
                    output.push_str(&format!("  … and {} more\n", links.len() - show));
                }
            }
        } else if content_type.contains("application/json") {
            output.push_str(&format!("Content ({size_label}, application/json):\n"));
            match serde_json::from_slice::<Value>(body_display) {
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
            output.push_str(&format!("Content ({size_label}, {content_type}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        } else if content_type.is_empty() || content_type.contains("octet-stream") {
            output.push_str(&format!(
                "Content: binary/unknown ({size_label})\n\
                 Content-Type: {content_type}\n\
                 [Use mode: \"raw\" to view raw bytes, or save_url to download]\n"
            ));
        } else {
            output.push_str(&format!("Content ({size_label}, {content_type}):\n"));
            output.push_str(&String::from_utf8_lossy(body_display));
        }

        if truncated {
            output.push_str(&format!(
                "\n\n[Response truncated at {} bytes. Total: {} bytes]",
                self.max_body_bytes,
                body_bytes.len()
            ));
        }

        ToolResult::success(output)
    }
}

// ---------------------------------------------------------------------------
// SSRF guard (same logic as http_request)
// ---------------------------------------------------------------------------

async fn check_ssrf(host: &str) -> Result<(), String> {
    use std::net::IpAddr;

    if let Ok(ip) = host.parse::<IpAddr>() {
        if http_request::is_private_ip(ip) {
            return Err(format!("URL resolves to private IP ({ip})"));
        }
        return Ok(());
    }

    let addr = format!("{host}:0");
    let addrs = tokio::net::lookup_host(&addr)
        .await
        .map_err(|e| format!("DNS resolution failed: {e}"))?;

    for sock_addr in addrs {
        if http_request::is_private_ip(sock_addr.ip()) {
            return Err(format!("{host} resolves to private IP ({})", sock_addr.ip()));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        let tool = FetchUrlTool::new();
        assert_eq!(tool.name(), "fetch_url");
    }

    #[test]
    fn test_tool_nature_is_network() {
        let tool = FetchUrlTool::new();
        assert!(matches!(tool.nature(), ToolNature::Network));
    }

    #[test]
    fn test_tool_summary_fields() {
        let tool = FetchUrlTool::new();
        assert_eq!(tool.summary_fields(), &["url"]);
    }

    #[test]
    fn test_tool_emits_parameters_schema() {
        let tool = FetchUrlTool::new();
        let schema = tool.parameters();
        assert!(schema.is_object());
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_default_mode() {
        assert_eq!(default_mode(), "auto");
    }

    #[test]
    fn test_default_max_links() {
        assert_eq!(default_max_links(), 10);
    }

    #[test]
    fn test_deserialize_minimal_input() {
        let json = r#"{"url": "https://example.com"}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.url, "https://example.com");
        assert_eq!(input.mode, "auto");
        assert_eq!(input.max_links, 10);
    }

    #[test]
    fn test_deserialize_full_input() {
        let json = r#"{"url": "https://example.com", "mode": "raw", "max_links": 5}"#;
        let input: FetchUrlInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.mode, "raw");
        assert_eq!(input.max_links, 5);
    }

    #[test]
    fn test_extract_links_html() {
        let html = "\
            <html><body>\
            <a href=\"https://example.com/page1\">Page 1</a>\
            <a href=\"/relative\">Relative</a>\
            <a href=\"#section\">Anchor</a>\
            <a href=\"javascript:void(0)\">JS</a>\
            <a href=\"https://example.com/page1\">Dup</a>\
            </body></html>";
        let links = FetchUrlTool::extract_links(html, "https://example.com");
        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/relative".to_string()));
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = "\
            <html><body>\
            <h1>Title</h1>\
            <p>Some <strong>bold</strong> text.</p>\
            </body></html>";
        let text = FetchUrlTool::extract_text(html);
        assert!(text.contains("Title"));
        assert!(text.contains("bold"));
        assert!(text.contains("text"));
    }

    #[test]
    fn test_extract_text_empty_html() {
        let text = FetchUrlTool::extract_text("<html></html>");
        assert_eq!(text, "[No visible text content]");
    }
}
