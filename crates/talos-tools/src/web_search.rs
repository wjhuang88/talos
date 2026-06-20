//! Web search tool using multiple backends with race + fallback strategy.
//!
//! Searches DuckDuckGo by default (no API key required), with optional
//! Tavily and SearXNG backends for enhanced results. Falls back to
//! Wikipedia OpenSearch when all other backends fail.

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

/// Errors that can occur during web search tool execution.
#[derive(Debug, Error)]
pub enum WebSearchError {
    /// The input does not conform to the expected schema.
    #[error("invalid web_search input: {0}")]
    InvalidInput(String),
    /// All search backends failed.
    #[error("all search backends failed: {0}")]
    AllBackendsFailed(String),
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Input parameters for the [`WebSearchTool`].
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebSearchInput {
    /// Search query string.
    pub query: String,

    /// Maximum number of results to return. Default 10, max 20.
    #[serde(default = "default_max_results")]
    #[schemars(range(min = 1, max = 20))]
    pub max_results: u32,

    /// Whether to include text snippets in results. Default true.
    #[serde(default = "default_include_snippets")]
    pub include_snippets: bool,
}

fn default_max_results() -> u32 {
    10
}

fn default_include_snippets() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Search result types
// ---------------------------------------------------------------------------

/// A single search result.
#[derive(Debug, Clone)]
struct WebResult {
    title: String,
    url: String,
    snippet: String,
}

/// Backend that produced the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultSource {
    DuckDuckGo,
    Tavily,
    SearXNG,
    Wikipedia,
}

// ---------------------------------------------------------------------------
// Tool
// ---------------------------------------------------------------------------

/// A tool that performs web searches using multiple backends.
///
/// **Default (zero config)**: DuckDuckGo via `rust-websearch` crate.
/// Falls back to Wikipedia OpenSearch if DuckDuckGo fails.
///
/// **Optional backends** (configured via environment variables):
/// - **Tavily**: AI-optimized search. Set `TAVILY_API_KEY` env var.
/// - **SearXNG**: Self-hosted metasearch. Set `SEARXNG_URL` env var.
///
/// All available backends are queried in parallel (race). The first
/// successful response wins.
pub struct WebSearchTool {
    duckduckgo_config: rust_websearch::SearchConfig,
    tavily_api_key: Option<String>,
    searxng_url: Option<String>,
}

impl WebSearchTool {
    /// Create a new [`WebSearchTool`].
    ///
    /// Reads `TAVILY_API_KEY` and `SEARXNG_URL` from the environment
    /// at construction time. These are optional — the tool works with
    /// zero configuration (DuckDuckGo only).
    pub fn new() -> Self {
        let tavily_api_key = std::env::var("TAVILY_API_KEY").ok();
        let searxng_url = std::env::var("SEARXNG_URL").ok();

        Self {
            duckduckgo_config: rust_websearch::SearchConfig::default(),
            tavily_api_key,
            searxng_url,
        }
    }

    /// Search DuckDuckGo via rust-websearch crate.
    async fn search_duckduckgo(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<WebResult>, String> {
        let results = rust_websearch::search(query, &self.duckduckgo_config)
            .await
            .map_err(|e| format!("DuckDuckGo search failed: {e}"))?;

        let converted: Vec<WebResult> = results
            .results
            .into_iter()
            .take(max_results as usize)
            .map(|r| WebResult {
                title: r.title,
                url: r.url,
                snippet: r.snippet,
            })
            .collect();

        if converted.is_empty() {
            return Err("DuckDuckGo returned no results".to_string());
        }

        Ok(converted)
    }

    /// Search Tavily AI-optimized search API.
    async fn search_tavily(
        &self,
        query: &str,
        max_results: u32,
        _include_snippets: bool,
    ) -> Result<Vec<WebResult>, String> {
        let api_key = self
            .tavily_api_key
            .as_ref()
            .ok_or_else(|| "TAVILY_API_KEY not set".to_string())?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;

        let response = client
            .post("https://api.tavily.com/search")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&serde_json::json!({
                "query": query,
                "max_results": max_results.min(10),
                "search_depth": "basic",
                "include_answer": false,
                "include_raw_content": false,
                "include_images": false,
            }))
            .send()
            .await
            .map_err(|e| format!("Tavily request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Tavily returned status {}", response.status()));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("failed to parse Tavily response: {e}"))?;

        let results_array = body["results"].as_array().ok_or_else(|| {
            format!(
                "unexpected Tavily response format: {}",
                serde_json::to_string_pretty(&body).unwrap_or_default()
            )
        })?;

        let mut results = Vec::new();
        for item in results_array.iter().take(max_results as usize) {
            results.push(WebResult {
                title: item["title"].as_str().unwrap_or("Untitled").to_string(),
                url: item["url"].as_str().unwrap_or("").to_string(),
                snippet: item["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            });
        }

        if results.is_empty() {
            return Err("Tavily returned no results".to_string());
        }

        Ok(results)
    }

    /// Search a self-hosted SearXNG instance.
    async fn search_searxng(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<WebResult>, String> {
        let searxng_url = self
            .searxng_url
            .as_ref()
            .ok_or_else(|| "SEARXNG_URL not configured".to_string())?;

        let base = searxng_url.trim_end_matches('/');
        let search_url = format!("{base}/search?format=json&q={}", urlencoding(query));

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;

        let response = client
            .get(&search_url)
            .send()
            .await
            .map_err(|e| format!("SearXNG request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("SearXNG returned status {}", response.status()));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("failed to parse SearXNG response: {e}"))?;

        let results_array = body["results"]
            .as_array()
            .ok_or_else(|| "SearXNG response missing results array".to_string())?;

        let mut results = Vec::new();
        for item in results_array.iter().take(max_results as usize) {
            results.push(WebResult {
                title: item["title"].as_str().unwrap_or("Untitled").to_string(),
                url: item["url"].as_str().unwrap_or("").to_string(),
                snippet: item["content"]
                    .as_str()
                    .or(item["snippet"].as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }

        if results.is_empty() {
            return Err("SearXNG returned no results".to_string());
        }

        Ok(results)
    }

    /// Fallback: Wikipedia OpenSearch.
    async fn search_wikipedia(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<WebResult>, String> {
        let encoded = urlencoding(query);
        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=opensearch&search={encoded}&limit={max_results}&namespace=0&format=json"
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Wikipedia request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Wikipedia returned status {}", response.status()));
        }

        // Wikipedia OpenSearch returns: [query, [titles], [descriptions], [urls]]
        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("failed to parse Wikipedia response: {e}"))?;

        let titles = body[1].as_array();
        let descriptions = body[2].as_array();
        let urls = body[3].as_array();

        let (Some(titles), Some(descriptions), Some(urls)) = (titles, descriptions, urls) else {
            return Err("unexpected Wikipedia response format".to_string());
        };

        let count = titles.len().min(descriptions.len()).min(urls.len());
        if count == 0 {
            return Err("Wikipedia returned no results".to_string());
        }

        let mut results = Vec::new();
        for i in 0..count.min(max_results as usize) {
            results.push(WebResult {
                title: titles[i].as_str().unwrap_or("Untitled").to_string(),
                url: urls[i].as_str().unwrap_or("").to_string(),
                snippet: descriptions[i].as_str().unwrap_or("").to_string(),
            });
        }

        Ok(results)
    }

    /// Run all available backends in parallel (race), with fallback chain.
    async fn execute_search(
        &self,
        query: &str,
        max_results: u32,
        include_snippets: bool,
    ) -> (Vec<WebResult>, ResultSource) {
        // Try DuckDuckGo (always available) and optional backends in parallel.
        let ddg_fut = self.search_duckduckgo(query, max_results);
        let tavily_fut = async {
            if self.tavily_api_key.is_some() {
                self.search_tavily(query, max_results, include_snippets)
                    .await
            } else {
                Err("Tavily not configured".to_string())
            }
        };
        let searxng_fut = async {
            if self.searxng_url.is_some() {
                self.search_searxng(query, max_results).await
            } else {
                Err("SearXNG not configured".to_string())
            }
        };

        // Race: first successful response wins.
        let race_result = tokio::select! {
            res = ddg_fut => res.map(|r| (r, ResultSource::DuckDuckGo)),
            res = tavily_fut => res.map(|r| (r, ResultSource::Tavily)),
            res = searxng_fut => res.map(|r| (r, ResultSource::SearXNG)),
        };

        match race_result {
            Ok((results, source)) => (results, source),
            Err(_) => {
                // All primary backends failed. Try Wikipedia as last resort.
                match self.search_wikipedia(query, max_results).await {
                    Ok(results) => (results, ResultSource::Wikipedia),
                    Err(_) => (vec![], ResultSource::Wikipedia),
                }
            }
        }
    }

    /// Format results for model consumption.
    fn format_results(
        &self,
        query: &str,
        results: &[WebResult],
        source: ResultSource,
        include_snippets: bool,
    ) -> String {
        let source_label = match source {
            ResultSource::DuckDuckGo => "DuckDuckGo",
            ResultSource::Tavily => "Tavily",
            ResultSource::SearXNG => "SearXNG",
            ResultSource::Wikipedia => "Wikipedia (fallback)",
        };

        let mut output = format!(
            "Searched: \"{query}\"\nSource: {source_label}\nResults: {}\n\n",
            results.len()
        );

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, result.title));
            output.push_str(&format!("   {}\n", result.url));
            if include_snippets && !result.snippet.is_empty() {
                // Truncate long snippets at UTF-8 character boundary.
                let snippet = if result.snippet.len() > 300 {
                    let boundary = result
                        .snippet
                        .char_indices()
                        .take_while(|(i, _)| *i < 300)
                        .map(|(i, c)| i + c.len_utf8())
                        .last()
                        .unwrap_or(300);
                    format!("{}…", &result.snippet[..boundary])
                } else {
                    result.snippet.clone()
                };
                output.push_str(&format!("   {snippet}\n"));
            }
            output.push('\n');
        }

        if results.is_empty() {
            output.push_str("No results found.\n");
        }

        output
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple URL encoding for query parameters.
fn urlencoding(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

// ---------------------------------------------------------------------------
// AgentTool implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl AgentTool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for current information, documentation, or answers. \
         Uses DuckDuckGo by default (no API key needed). \
         Optionally uses Tavily (TAVILY_API_KEY env) or SearXNG (SEARXNG_URL env) for enhanced results. \
         Falls back to Wikipedia when other backends fail. Requires network permission."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(WebSearchInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["query"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: WebSearchInput = match serde_json::from_value(input) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::error(format!(
                    "{}",
                    WebSearchError::InvalidInput(e.to_string())
                ));
            }
        };

        if parsed.query.trim().is_empty() {
            return ToolResult::error(
                WebSearchError::InvalidInput("query must not be empty".to_string()).to_string(),
            );
        }

        let max_results = parsed.max_results.clamp(1, 20);

        let (results, source) = self
            .execute_search(&parsed.query, max_results, parsed.include_snippets)
            .await;

        if results.is_empty() {
            return ToolResult::error(
                WebSearchError::AllBackendsFailed(format!(
                    "no results found for query \"{}\"",
                    parsed.query
                ))
                .to_string(),
            );
        }

        let output = self.format_results(&parsed.query, &results, source, parsed.include_snippets);
        ToolResult::success(output)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "web_search");
    }

    #[test]
    fn test_tool_is_not_read_only() {
        let tool = WebSearchTool::new();
        assert!(!tool.is_read_only());
    }

    #[test]
    fn test_tool_nature_is_network() {
        let tool = WebSearchTool::new();
        assert!(matches!(tool.nature(), ToolNature::Network));
    }

    #[test]
    fn test_tool_summary_fields() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.summary_fields(), &["query"]);
    }

    #[test]
    fn test_tool_has_description() {
        let tool = WebSearchTool::new();
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_tool_emits_parameters_schema() {
        let tool = WebSearchTool::new();
        let schema = tool.parameters();
        assert!(
            schema.is_object(),
            "parameters should be a JSON Schema object"
        );
        let props = schema.get("properties");
        assert!(
            props.is_some(),
            "schema should have properties for query, max_results, etc."
        );
    }

    #[test]
    fn test_default_max_results() {
        assert_eq!(default_max_results(), 10);
    }

    #[test]
    fn test_default_include_snippets() {
        assert!(default_include_snippets());
    }

    // Input deserialization tests.

    #[test]
    fn test_deserialize_minimal_input() {
        let json = r#"{"query": "rust async tokio"}"#;
        let input: WebSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "rust async tokio");
        assert_eq!(input.max_results, 10);
        assert!(input.include_snippets);
    }

    #[test]
    fn test_deserialize_full_input() {
        let json = r#"{
            "query": "rust axum middleware",
            "max_results": 5,
            "include_snippets": false
        }"#;
        let input: WebSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "rust axum middleware");
        assert_eq!(input.max_results, 5);
        assert!(!input.include_snippets);
    }

    #[test]
    fn test_deserialize_missing_query_fails() {
        let json = r#"{"max_results": 5}"#;
        let result: Result<WebSearchInput, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // URL encoding tests.

    #[test]
    fn test_urlencoding_basic() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
    }

    #[test]
    fn test_urlencoding_special_chars() {
        let encoded = urlencoding("rust & go?");
        assert!(encoded.contains("%26")); // &
        assert!(encoded.contains("%3F")); // ?
    }

    #[test]
    fn test_urlencoding_chinese() {
        let encoded = urlencoding("你好");
        assert!(encoded.contains('%'));
        assert!(encoded.len() > 6); // each CJK char = %XX%XX%XX
    }

    // Formatting tests.

    #[test]
    fn test_format_results_with_snippets() {
        let tool = WebSearchTool::new();
        let results = vec![WebResult {
            title: "Rust Programming Language".to_string(),
            url: "https://rust-lang.org".to_string(),
            snippet: "A language empowering everyone to build reliable and efficient software."
                .to_string(),
        }];
        let output = tool.format_results("rust", &results, ResultSource::DuckDuckGo, true);
        assert!(output.contains("Source: DuckDuckGo"));
        assert!(output.contains("rust-lang.org"));
        assert!(output.contains("reliable and efficient"));
    }

    #[test]
    fn test_format_results_without_snippets() {
        let tool = WebSearchTool::new();
        let results = vec![WebResult {
            title: "Rust".to_string(),
            url: "https://rust-lang.org".to_string(),
            snippet: "should not appear".to_string(),
        }];
        let output = tool.format_results("rust", &results, ResultSource::DuckDuckGo, false);
        assert!(!output.contains("should not appear"));
    }

    #[test]
    fn test_format_results_wikipedia_fallback() {
        let tool = WebSearchTool::new();
        let results = vec![WebResult {
            title: "Rust (fungus)".to_string(),
            url: "https://en.wikipedia.org/wiki/Rust_(fungus)".to_string(),
            snippet: "Rusts are fungal plant pathogens".to_string(),
        }];
        let output = tool.format_results("rust", &results, ResultSource::Wikipedia, true);
        assert!(output.contains("Wikipedia (fallback)"));
    }

    #[test]
    fn test_format_results_empty() {
        let tool = WebSearchTool::new();
        let output = tool.format_results("xyzzy", &[], ResultSource::DuckDuckGo, true);
        assert!(output.contains("No results found"));
    }
}
