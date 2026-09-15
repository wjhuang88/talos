//! Browser-page record types and mock connector (T47, WEB-005 Phase 1).
//!
//! Defines `BrowserPageRecord`, `BrowserPageConnector` trait, and
//! `MockBrowserPageConnector` for safe fixture-based page reads. No cookies,
//! storage, credentials, DOM dumps, screenshots, or browser profile paths
//! are stored or exposed.

use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Read-only HTTP browser connector. It never executes scripts or persists browser state.
#[cfg(feature = "network")]
pub struct HttpBrowserPageConnector {
    client: reqwest::Client,
    max_text_bytes: usize,
    max_links: usize,
}

#[cfg(feature = "network")]
impl HttpBrowserPageConnector {
    /// Creates a connector with conservative bounded output and redirects.
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|error| format!("browser connector client: {error}"))?;
        Ok(Self {
            client,
            max_text_bytes: 32 * 1024,
            max_links: 32,
        })
    }

    /// Overrides output limits for deterministic fixtures.
    pub fn with_limits(mut self, max_text_bytes: usize, max_links: usize) -> Self {
        self.max_text_bytes = max_text_bytes;
        self.max_links = max_links;
        self
    }
}

fn extract_origin(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
        .unwrap_or_else(|| {
            let after_scheme = url.split("://").nth(1).unwrap_or(url);
            after_scheme
                .split('/')
                .next()
                .unwrap_or(after_scheme)
                .to_string()
        })
}

fn is_sensitive_query_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("api_key")
        || key == "key"
        || key == "sig"
        || key == "signature"
        || key == "auth"
}

fn sanitize_url_for_record(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return url.split('?').next().unwrap_or(url).to_string();
    };

    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);

    let pairs: Vec<(String, String)> = parsed
        .query_pairs()
        .map(|(key, value)| {
            let value = if is_sensitive_query_key(&key) {
                "***".to_string()
            } else {
                value.into_owned()
            };
            (key.into_owned(), value)
        })
        .collect();

    parsed.set_query(None);
    if !pairs.is_empty() {
        let mut query = parsed.query_pairs_mut();
        for (key, value) in pairs {
            query.append_pair(&key, &value);
        }
    }

    parsed.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPageLink {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPageRecord {
    pub record_id: String,
    pub url: String,
    pub final_url: String,
    pub origin: String,
    pub title: String,
    pub visible_text_excerpt: String,
    pub selected_links: Vec<BrowserPageLink>,
    pub connector_kind: String,
    pub access_mode: String,
}

impl BrowserPageRecord {
    pub fn new_mock(url: &str, title: &str, text: &str) -> Self {
        let sanitized_url = sanitize_url_for_record(url);
        let origin = extract_origin(&sanitized_url);
        Self {
            record_id: Uuid::new_v4().to_string(),
            url: sanitized_url.clone(),
            final_url: sanitized_url,
            origin,
            title: title.to_string(),
            visible_text_excerpt: text.to_string(),
            selected_links: Vec::new(),
            connector_kind: "mock".to_string(),
            access_mode: "current_tab".to_string(),
        }
    }

    pub fn with_links(mut self, links: Vec<BrowserPageLink>) -> Self {
        self.selected_links = links
            .into_iter()
            .map(|link| BrowserPageLink {
                text: link.text,
                url: sanitize_url_for_record(&link.url),
            })
            .collect();
        self
    }
}

#[async_trait]
pub trait BrowserPageConnector: Send + Sync {
    async fn read_page(&self, url: &str) -> Result<BrowserPageRecord, String>;
}

#[cfg(feature = "network")]
#[async_trait]
impl BrowserPageConnector for HttpBrowserPageConnector {
    async fn read_page(&self, url: &str) -> Result<BrowserPageRecord, String> {
        let parsed = Url::parse(url).map_err(|error| format!("invalid browser URL: {error}"))?;
        match parsed.scheme() {
            "http" | "https" => {}
            scheme => return Err(format!("unsupported browser URL scheme: {scheme}")),
        }
        let host = parsed
            .host_str()
            .ok_or_else(|| "browser URL has no host".to_string())?;
        crate::http_request::check_ssrf_host(host).await?;
        let response = self
            .client
            .get(parsed)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!("browser request returned {}", response.status()));
        }
        let final_url = response.url().to_string();
        let final_base = response.url().clone();
        let max_response_bytes = (self.max_text_bytes as u64).saturating_mul(8);
        let Some(content_length) = response.content_length() else {
            return Err("browser response has no bounded content length".to_string());
        };
        if content_length > max_response_bytes {
            return Err("browser response exceeds bounded size".to_string());
        }
        let body = response.text().await.map_err(|error| error.to_string())?;
        let document = scraper::Html::parse_document(&body);
        let title_selector =
            scraper::Selector::parse("title").map_err(|error| error.to_string())?;
        let mut title = document
            .select(&title_selector)
            .next()
            .map(|node| node.text().collect::<String>())
            .unwrap_or_default();
        if title.len() > 1024 {
            title.truncate(title.floor_char_boundary(1024));
        }
        let text_selector = scraper::Selector::parse("body").map_err(|error| error.to_string())?;
        let mut visible = document
            .select(&text_selector)
            .next()
            .map(|node| node.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        visible.truncate(visible.floor_char_boundary(self.max_text_bytes.min(visible.len())));
        let link_selector =
            scraper::Selector::parse("a[href]").map_err(|error| error.to_string())?;
        let mut links = Vec::new();
        for node in document.select(&link_selector).take(self.max_links) {
            if let Some(href) = node.value().attr("href")
                && let Ok(link_url) = Url::parse(href).or_else(|_| final_base.join(href))
                && matches!(link_url.scheme(), "http" | "https")
            {
                let mut text = node.text().collect::<String>();
                text.truncate(text.floor_char_boundary(512));
                links.push(BrowserPageLink {
                    text,
                    url: sanitize_url_for_record(link_url.as_str()),
                });
            }
        }
        let mut record =
            BrowserPageRecord::new_mock(&final_url, &title, &visible).with_links(links);
        record.url = sanitize_url_for_record(url);
        record.final_url = sanitize_url_for_record(&final_url);
        record.origin = extract_origin(&record.final_url);
        record.connector_kind = "http-read-only".to_string();
        Ok(record)
    }
}

pub struct MockBrowserPageConnector {
    records: std::collections::HashMap<String, BrowserPageRecord>,
}

impl MockBrowserPageConnector {
    pub fn new() -> Self {
        Self {
            records: std::collections::HashMap::new(),
        }
    }

    pub fn with_record(mut self, url: &str, record: BrowserPageRecord) -> Self {
        self.records.insert(url.to_string(), record);
        self
    }
}

impl Default for MockBrowserPageConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserPageConnector for MockBrowserPageConnector {
    async fn read_page(&self, url: &str) -> Result<BrowserPageRecord, String> {
        self.records
            .get(url)
            .cloned()
            .ok_or_else(|| format!("no mock record for URL: {url}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_excludes_cookies() {
        let record = BrowserPageRecord::new_mock("https://example.com", "Test", "text");
        let json = serde_json::to_string(&record).expect("operation should succeed");
        assert!(!json.contains("cookie"));
        assert!(!json.contains("Cookie"));
    }

    #[test]
    fn record_excludes_storage() {
        let record = BrowserPageRecord::new_mock("https://example.com", "Test", "text");
        let json = serde_json::to_string(&record).expect("operation should succeed");
        assert!(!json.contains("localStorage"));
        assert!(!json.contains("sessionStorage"));
        assert!(!json.contains("storage"));
    }

    #[test]
    fn record_excludes_credentials() {
        let record = BrowserPageRecord::new_mock(
            "https://user:pass@example.com/path?token=abc&api_key=sk-test&ok=1",
            "Test",
            "text",
        );
        let json = serde_json::to_string(&record).expect("operation should succeed");
        assert!(!json.contains("password"));
        for url in [&record.url, &record.final_url, &record.origin] {
            assert!(!url.contains("abc"));
            assert!(!url.contains("sk-test"));
            assert!(!url.contains("user:pass"));
        }
        assert!(!json.contains("secret"));
        assert!(json.contains("token=***"));
        assert!(json.contains("api_key=***"));
        assert!(json.contains("ok=1"));
    }

    #[test]
    fn record_excludes_dom_and_screenshots() {
        let record = BrowserPageRecord::new_mock("https://example.com", "Test", "text");
        let json = serde_json::to_string(&record).expect("operation should succeed");
        assert!(!json.contains("dom"));
        assert!(!json.contains("screenshot"));
        assert!(!json.contains("profile_path"));
    }

    #[test]
    fn record_stores_approved_fields() {
        let record = BrowserPageRecord::new_mock(
            "https://example.com/dashboard",
            "Dashboard",
            "Welcome to your dashboard",
        )
        .with_links(vec![BrowserPageLink {
            text: "Settings".to_string(),
            url: "https://example.com/settings".to_string(),
        }]);

        assert_eq!(record.url, "https://example.com/dashboard");
        assert_eq!(record.origin, "example.com");
        assert_eq!(record.title, "Dashboard");
        assert!(record.visible_text_excerpt.contains("Welcome"));
        assert_eq!(record.selected_links.len(), 1);
        assert_eq!(record.connector_kind, "mock");
    }

    #[test]
    fn record_sanitizes_selected_link_urls() {
        let record =
            BrowserPageRecord::new_mock("https://example.com", "Test", "text").with_links(vec![
                BrowserPageLink {
                    text: "Account".to_string(),
                    url: "https://user:pass@example.com/account?token=abc&ok=1".to_string(),
                },
            ]);

        assert_eq!(record.selected_links.len(), 1);
        let sanitized_url = &record.selected_links[0].url;
        assert!(!sanitized_url.contains("user:pass"));
        assert!(!sanitized_url.contains("abc"));
        assert!(sanitized_url.contains("token=***"));
        assert!(sanitized_url.contains("ok=1"));
    }

    #[tokio::test]
    async fn mock_connector_returns_canned_record() {
        let record = BrowserPageRecord::new_mock("https://example.com", "Test", "Hello");
        let connector = MockBrowserPageConnector::new().with_record("https://example.com", record);
        let result = connector
            .read_page("https://example.com")
            .await
            .expect("operation should succeed");
        assert_eq!(result.title, "Test");
        assert!(result.visible_text_excerpt.contains("Hello"));
    }

    #[tokio::test]
    async fn mock_connector_errors_on_unknown_url() {
        let connector = MockBrowserPageConnector::new();
        let result = connector.read_page("https://unknown.com").await;
        assert!(result.is_err());
    }

    #[test]
    fn record_serializes_to_clean_json() {
        let record = BrowserPageRecord::new_mock("https://example.com", "Test", "text");
        let json: serde_json::Value = serde_json::from_str(
            &serde_json::to_string(&record).expect("operation should succeed"),
        )
        .expect("operation should succeed");
        let keys: Vec<&str> = json
            .as_object()
            .expect("operation should succeed")
            .keys()
            .map(|k| k.as_str())
            .collect();
        let approved = [
            "record_id",
            "url",
            "final_url",
            "origin",
            "title",
            "visible_text_excerpt",
            "selected_links",
            "connector_kind",
            "access_mode",
        ];
        for key in &keys {
            assert!(
                approved.contains(key),
                "unexpected field '{key}' in BrowserPageRecord JSON"
            );
        }
    }
}
