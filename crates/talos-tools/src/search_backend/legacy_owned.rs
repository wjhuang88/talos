//! Owned implementation of the old standalone WebSearchTool compatibility request path.
//!
//! The protected production SearchRouter DOES NOT call this module. It has its own
//! prepared invocation and Network authority. This code only replaces the former
//! direct `rust_websearch::search` execution of the public legacy Tool.
//!
//! Derived from rust-websearch 0.1.2 at Rust-Framework/rust-agent-framework
//! b30180a03609580e8b28dfb682284cb2ba249fba (MIT).
//! License retained in crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! Do not treat this compatibility fallback as an independent search provider.

mod probe;

use std::time::Duration;

use reqwest::{Client, Method, StatusCode, Url};
use super::{
    SearchBackendError, SearchBackendResult, bing_parser,
    compat_helpers::random_user_agent,
    ddg_parser::{self, DdgFormat},
    instant_answer,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct LegacyConfig {
    pub(crate) proxy_url: Option<String>,
}

#[cfg(test)]
impl LegacyConfig {
    pub(crate) fn with_proxy(mut self, url: &str) -> Self {
        self.proxy_url = Some(url.to_string());
        self
    }
}

fn client(config: &LegacyConfig, timeout: Duration, redirects: usize)
    -> Result<Client, SearchBackendError>
{
    let mut builder = Client::builder()
        .timeout(timeout)
        .user_agent(random_user_agent())
        .redirect(reqwest::redirect::Policy::limited(redirects));
    if let Some(url) = &config.proxy_url {
        let proxy = reqwest::Proxy::all(url)
            .map_err(|_| SearchBackendError::RequestFailed("invalid legacy proxy".into()))?;
        builder = builder.proxy(proxy);
    }
    #[cfg(test)]
    {
        // The historical upstream parity child provides its own local test CA.
        if let Some(path) = std::env::var_os("TALOS_SEARCH_PARITY_CA") {
            let pem = std::fs::read(path)
                .map_err(|_| SearchBackendError::RequestFailed("test CA read".into()))?;
            let cert = reqwest::Certificate::from_pem(&pem)
                .map_err(|_| SearchBackendError::RequestFailed("test CA parse".into()))?;
            builder = builder.add_root_certificate(cert);
        }
    }
    builder.build().map_err(|_| SearchBackendError::RequestFailed("legacy client build".into()))
}

#[derive(Clone, Copy)]
enum Protocol {
    Lite,
    Instant,
    Html,
    Bing,
}

impl Protocol {
    fn url(self) -> &'static str {
        match self {
            Self::Lite => "https://lite.duckduckgo.com/lite/",
            Self::Instant => "https://api.duckduckgo.com/",
            Self::Html => "https://html.duckduckgo.com/html/",
            Self::Bing => "https://cn.bing.com/search",
        }
    }

    fn retries(self) -> usize {
        if matches!(self, Self::Instant) { 0 } else { 1 }
    }

    fn method(self) -> Method {
        if matches!(self, Self::Lite) { Method::POST } else { Method::GET }
    }
}

fn request_url_and_body(protocol: Protocol, query: &str)
    -> Result<(Url, Option<String>), SearchBackendError>
{
    let mut url = Url::parse(protocol.url())
        .map_err(|_| SearchBackendError::RequestFailed("legacy static endpoint".into()))?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("q", query);
        match protocol {
            Protocol::Lite => { pairs.append_pair("kl", "wt-wt"); }
            Protocol::Instant => {
                pairs.append_pair("format", "json");
                pairs.append_pair("no_html", "1");
                pairs.append_pair("skip_disambig", "1");
                pairs.append_pair("t", "rust-agent-websearch");
            }
            Protocol::Bing => { pairs.append_pair("mkt", "zh-CN"); }
            Protocol::Html => {}
        }
    }
    if matches!(protocol, Protocol::Lite) {
        let body = url.query().unwrap_or_default().to_owned();
        url.set_query(None);
        Ok((url, Some(body)))
    } else {
        Ok((url, None))
    }
}

fn classify(error: &reqwest::Error) -> SearchBackendError {
    if error.is_timeout() { SearchBackendError::Timeout }
    else if error.is_connect() { SearchBackendError::Unreachable }
    else { SearchBackendError::RequestFailed("legacy transport failed".into()) }
}

async fn query_protocol(
    protocol: Protocol,
    query: &str,
    config: &LegacyConfig,
) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
    let (url, body) = request_url_and_body(protocol, query)?;
    for attempt in 0..=protocol.retries() {
        let http = client(config, Duration::from_secs(15), 5)?;
        let mut request = http.request(protocol.method(), url.clone());
        if let Some(body) = &body {
            request = request
                .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(body.clone());
        }
        if matches!(protocol, Protocol::Bing) {
            request = request.header(reqwest::header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8");
        }
        match request.send().await {
            Err(err) if attempt < protocol.retries() => {
                // Preserve exactly one transient-request retry in the legacy family.
                let _ = err;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(err) => return Err(classify(&err)),
            Ok(response) => {
                let status = response.status();
                let body = response.bytes().await.map_err(|_| SearchBackendError::Parse)?;
                return parse_response(protocol, status, &body, query);
            }
        }
    }
    Err(SearchBackendError::NoResults)
}

fn parse_response(
    protocol: Protocol,
    status: StatusCode,
    body: &[u8],
    query: &str,
) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
    // The old rust-websearch config used ten results, with the Tool truncating
    // to the requested count after the coordinator returned its result list.
    const LEGACY_COUNT: u32 = 10;
    let result = match protocol {
        Protocol::Lite => ddg_parser::parse(DdgFormat::Lite, status.as_u16(), body, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
        Protocol::Instant => instant_answer::parse(status.as_u16(), body, query, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
        Protocol::Html => ddg_parser::parse(DdgFormat::Html, status.as_u16(), body, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
        Protocol::Bing => bing_parser::parse(status.as_u16(), body, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
    }?;
    if result.is_empty() { Err(SearchBackendError::NoResults) } else { Ok(result) }
}

/// Historical direct Tool path only; the protected Tool never calls this.
pub(crate) async fn search(
    query: &str,
    config: &LegacyConfig,
) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
    if query.trim().is_empty() { return Err(SearchBackendError::NoResults); }
    probe::rate_limit().await;
    let (ddg, bing) = probe::reachability(config, |t, redirects| client(config, t, redirects)).await;
    if ddg {
        for p in [Protocol::Lite, Protocol::Instant, Protocol::Html] {
            if let Ok(results) = query_protocol(p, query, config).await {
                return Ok(results);
            }
        }
    }
    if bing {
        if let Ok(results) = query_protocol(Protocol::Bing, query, config).await {
            return Ok(results);
        }
    }
    Err(SearchBackendError::NoResults)
}

#[cfg(test)]
pub(crate) fn clear_probe_cache_for_parity_test() {
    probe::clear_cache();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_destinations_and_request_shape_are_stable() {
        let (lite, body) = request_url_and_body(Protocol::Lite, "中文 & rust").unwrap();
        assert_eq!(lite.as_str(), "https://lite.duckduckgo.com/lite/");
        assert_eq!(body.unwrap(), "q=%E4%B8%AD%E6%96%87+%26+rust&kl=wt-wt");
        let (instant, body) = request_url_and_body(Protocol::Instant, "rust").unwrap();
        assert!(body.is_none());
        assert!(instant.as_str().contains("t=rust-agent-websearch"));
        let (bing, body) = request_url_and_body(Protocol::Bing, "rust").unwrap();
        assert!(body.is_none());
        assert_eq!(bing.host_str(), Some("cn.bing.com"));
    }
}
