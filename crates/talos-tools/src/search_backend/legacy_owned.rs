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

use std::{io::Read, time::Duration};

use super::{
    SearchBackendError, SearchBackendResult, bing_parser,
    compat_helpers::random_user_agent,
    ddg_parser::{self, DdgFormat},
    instant_answer,
};
use reqwest::{Client, Method, StatusCode, Url};

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

fn client(
    config: &LegacyConfig,
    timeout: Duration,
    redirects: usize,
) -> Result<Client, SearchBackendError> {
    // Retain Mozilla WebPKI roots ONLY for the historical standalone Tool;
    // protected SearchRouter clients keep their existing reqwest 0.13 verifier.
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    #[cfg(test)]
    let mut roots = roots;
    #[cfg(test)]
    {
        if let Some(path) = std::env::var_os("TALOS_SEARCH_PARITY_CA") {
            use rustls::pki_types::pem::PemObject;
            let pem = std::fs::read(path)
                .map_err(|_| SearchBackendError::RequestFailed("test CA read".into()))?;
            let mut count = 0;
            for cert in rustls::pki_types::CertificateDer::pem_slice_iter(&pem) {
                let der = cert
                    .map_err(|_| SearchBackendError::RequestFailed("test CA parse".into()))?;
                roots
                    .add(der)
                    .map_err(|_| SearchBackendError::RequestFailed("test CA invalid".into()))?;
                count += 1;
            }
            if count == 0 {
                return Err(SearchBackendError::RequestFailed("empty test CA".into()));
            }
        }
    }
    let tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let mut builder = Client::builder()
        .tls_backend_preconfigured(tls)
        // Prevent automatic decompression even if unrelated crates enable
        // the reqwest compression features in the unified Cargo graph.
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .timeout(timeout)
        .user_agent(random_user_agent())
        .redirect(reqwest::redirect::Policy::limited(redirects));
    if let Some(url) = &config.proxy_url {
        let proxy = reqwest::Proxy::all(url)
            .map_err(|_| SearchBackendError::RequestFailed("invalid legacy proxy".into()))?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|_| SearchBackendError::RequestFailed("legacy client build".into()))
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
        if matches!(self, Self::Lite) {
            Method::POST
        } else {
            Method::GET
        }
    }
}

fn request_url_and_body(
    protocol: Protocol,
    query: &str,
) -> Result<(Url, Option<String>), SearchBackendError> {
    let mut url = Url::parse(protocol.url())
        .map_err(|_| SearchBackendError::RequestFailed("legacy static endpoint".into()))?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("q", query);
        match protocol {
            Protocol::Lite => {
                pairs.append_pair("kl", "wt-wt");
            }
            Protocol::Instant => {
                pairs.append_pair("format", "json");
                pairs.append_pair("no_html", "1");
                pairs.append_pair("skip_disambig", "1");
                pairs.append_pair("t", "rust-agent-websearch");
            }
            Protocol::Bing => {
                pairs.append_pair("mkt", "zh-CN");
            }
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
    if error.is_timeout() {
        SearchBackendError::Timeout
    } else if error.is_connect() {
        SearchBackendError::Unreachable
    } else {
        SearchBackendError::RequestFailed("legacy transport failed".into())
    }
}

async fn query_protocol(
    protocol: Protocol,
    query: &str,
    config: &LegacyConfig,
) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
    let (url, body) = request_url_and_body(protocol, query)?;
    for attempt in 0..=protocol.retries() {
        let http = client(config, Duration::from_secs(15), 5)?;
        let mut request = http
            .request(protocol.method(), url.clone())
            // Preserve the old client's accepted formats without enabling
            // Cargo-global automatic response decompression.
            .header(reqwest::header::ACCEPT_ENCODING, "gzip, br, deflate");
        if let Some(body) = &body {
            request = request
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(body.clone());
        }
        if matches!(protocol, Protocol::Bing) {
            request = request.header(
                reqwest::header::ACCEPT_LANGUAGE,
                "zh-CN,zh;q=0.9,en;q=0.8",
            );
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
                let body = successful_body(response).await?;
                return parse_response(protocol, status, &body, query);
            }
        }
    }
    Err(SearchBackendError::NoResults)
}

// Reject non-success status at header time; never buffer a stalled or large
// 403/429/503 body before falling back to the next fixed search destination.
async fn successful_body(
    response: reqwest::Response,
) -> Result<Vec<u8>, SearchBackendError> {
    let status = response.status();
    if !status.is_success() {
        return Err(SearchBackendError::HttpStatus(status.as_u16()));
    }
    let encoding = response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let wire = response.bytes().await.map_err(|_| SearchBackendError::Parse)?;
    decode_legacy_content(&encoding, &wire)
}

// Decode ONLY this historical Tool's response bytes. Never enable reqwest's
// shared gzip/br/deflate feature flags, which would change SearchRouter.
fn decode_legacy_content(encoding: &str, body: &[u8]) -> Result<Vec<u8>, SearchBackendError> {
    const MAX_DECOMPRESSED_BYTES: u64 = 8 * 1024 * 1024;
    let reader: Box<dyn Read + '_> = match encoding {
        "gzip" => Box::new(flate2::read::GzDecoder::new(body)),
        "deflate" => Box::new(flate2::read::ZlibDecoder::new(body)),
        "br" => Box::new(brotli::Decompressor::new(body, 4096)),
        // The protected parser will reject an unsupported encoding; never
        // accidentally reinterpret it as known compressed content.
        _ => return Ok(body.to_vec()),
    };
    let mut decoded = Vec::new();
    reader
        .take(MAX_DECOMPRESSED_BYTES + 1)
        .read_to_end(&mut decoded)
        .map_err(|_| SearchBackendError::Parse)?;
    if decoded.len() as u64 > MAX_DECOMPRESSED_BYTES {
        return Err(SearchBackendError::Parse);
    }
    Ok(decoded)
}

// Upstream reqwest text() accepts malformed HTML UTF-8 and strips a leading BOM.
// Keep strict JSON bytes unchanged. This adaptation belongs ONLY to the legacy
// standalone Tool; protected SearchRouter parser policy is unmodified.
fn legacy_html_bytes(body: &[u8]) -> Vec<u8> {
    String::from_utf8_lossy(body)
        .trim_start_matches('\u{feff}')
        .as_bytes()
        .to_vec()
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
    let html_body;
    let decoded_body = if matches!(protocol, Protocol::Instant) {
        body
    } else {
        html_body = legacy_html_bytes(body);
        &html_body
    };
    let result = match protocol {
        Protocol::Lite => {
            ddg_parser::parse(DdgFormat::Lite, status.as_u16(), decoded_body, LEGACY_COUNT)
                .map(|x| x.into_candidates())
                .map_err(|_| SearchBackendError::Parse)
        }
        Protocol::Instant => instant_answer::parse(status.as_u16(), body, query, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
        Protocol::Html => {
            ddg_parser::parse(DdgFormat::Html, status.as_u16(), decoded_body, LEGACY_COUNT)
                .map(|x| x.into_candidates())
                .map_err(|_| SearchBackendError::Parse)
        }
        Protocol::Bing => bing_parser::parse(status.as_u16(), decoded_body, LEGACY_COUNT)
            .map(|x| x.into_candidates())
            .map_err(|_| SearchBackendError::Parse),
    }?;
    if result.is_empty() {
        Err(SearchBackendError::NoResults)
    } else {
        Ok(result)
    }
}

/// Historical direct Tool path only; the protected Tool never calls this.
pub(crate) async fn search(
    query: &str,
    config: &LegacyConfig,
) -> Result<Vec<SearchBackendResult>, SearchBackendError> {
    if query.trim().is_empty() {
        return Err(SearchBackendError::NoResults);
    }
    probe::rate_limit().await;
    let (ddg, bing) =
        probe::reachability(config, |t, redirects| client(config, t, redirects)).await;
    if ddg {
        for p in [Protocol::Lite, Protocol::Instant, Protocol::Html] {
            if let Ok(results) = query_protocol(p, query, config).await {
                return Ok(results);
            }
        }
    }
    if bing && let Ok(results) = query_protocol(Protocol::Bing, query, config).await {
        return Ok(results);
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

    fn spawn_http_fixture(
        status: &'static str,
        encoding: Option<&'static str>,
        body: &'static [u8],
        stall_body: bool,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("loopback fixture");
        let url = format!("http://{}", listener.local_addr().expect("fixture address"));
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("fixture accept");
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .expect("fixture timeout");
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request);
            let content_encoding = encoding
                .map(|value| format!("Content-Encoding: {value}\r\n"))
                .unwrap_or_default();
            let headers = format!(
                "HTTP/1.1 {status}\r\n{content_encoding}Content-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream
                .write_all(headers.as_bytes())
                .expect("fixture headers");
            if stall_body {
                // A non-200 response must be rejected before this body is read.
                std::thread::sleep(Duration::from_millis(500));
            } else {
                stream.write_all(body).expect("fixture body");
            }
        });
        (url, server)
    }

    #[tokio::test]
    async fn non_success_status_rejects_before_stalled_body() {
        let (url, server) =
            spawn_http_fixture("503 Service Unavailable", None, b"never sent", true);
        let http =
            client(&LegacyConfig::default(), Duration::from_secs(3), 0).expect("legacy client");
        let response = http.get(url).send().await.expect("fixture response");
        let result = tokio::time::timeout(Duration::from_millis(250), successful_body(response))
            .await
            .expect("error response must not wait for body");
        assert!(matches!(result, Err(SearchBackendError::HttpStatus(503))));
        server.join().expect("fixture thread");
    }

    #[tokio::test]
    async fn legacy_client_decompresses_upstream_content_encodings() {
        const GZIP: &[u8] = &[
            0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xcb, 0x49, 0x4d, 0x4f,
            0x4c, 0xae, 0x54, 0x48, 0xce, 0xcf, 0x2d, 0x28, 0x4a, 0x2d, 0x2e, 0x4e, 0x4d, 0x51,
            0x48, 0xcb, 0xac, 0x28, 0x29, 0x2d, 0x4a, 0x05, 0x00, 0x13, 0x70, 0xfb, 0xdd, 0x19,
            0x00, 0x00, 0x00,
        ];
        const BROTLI: &[u8] = &[
            0x0b, 0x0c, 0x80, 0x6c, 0x65, 0x67, 0x61, 0x63, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70,
            0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x20, 0x66, 0x69, 0x78, 0x74, 0x75, 0x72, 0x65,
            0x03,
        ];
        const DEFLATE: &[u8] = &[
            0x78, 0x9c, 0xcb, 0x49, 0x4d, 0x4f, 0x4c, 0xae, 0x54, 0x48, 0xce, 0xcf, 0x2d, 0x28,
            0x4a, 0x2d, 0x2e, 0x4e, 0x4d, 0x51, 0x48, 0xcb, 0xac, 0x28, 0x29, 0x2d, 0x4a, 0x05,
            0x00, 0x7f, 0x94, 0x09, 0xf2,
        ];
        for (encoding, body) in [("gzip", GZIP), ("br", BROTLI), ("deflate", DEFLATE)] {
            let (url, server) = spawn_http_fixture("200 OK", Some(encoding), body, false);
            let http =
                client(&LegacyConfig::default(), Duration::from_secs(3), 0).expect("legacy client");
            let response = http.get(url).send().await.expect("compressed response");
            let decoded = successful_body(response)
                .await
                .expect("decompress content encoding");
            assert_eq!(decoded, b"legacy compressed fixture", "{encoding}");
            server.join().expect("fixture thread");
        }
    }

    #[test]
    fn html_compatibility_decodes_malformed_utf8_and_bom_without_changing_json() {
        let raw = b"\xef\xbb\xbf<p>caf\xe9</p>";
        let normalized = legacy_html_bytes(raw);
        assert_eq!(String::from_utf8(normalized).unwrap(), "<p>caf\u{fffd}</p>");
        assert_eq!(raw[0..3], [0xef, 0xbb, 0xbf]);
        // A real Lite result must remain usable after the standalone adapter
        // converts the BOM and malformed UTF-8. The protected parser itself
        // continues to reject the original wire bytes.
        let lite = b"\xef\xbb\xbf<a class=\"result-link\" href=\"https://example.org/story\">caf\xe9</a><td class=\"result-snippet\">ok</td>";
        assert!(ddg_parser::parse(DdgFormat::Lite, 200, lite, 10).is_err());
        let rows = parse_response(Protocol::Lite, StatusCode::OK, lite, "test")
            .expect("legacy HTML decode before protected parser");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].url, "https://example.org/story");
        assert_eq!(rows[0].title, "caf\u{fffd}");
        // Unlike the HTML compatibility adapter, Instant Answer must reject
        // malformed JSON UTF-8 rather than silently replacing it.
        assert!(matches!(
            parse_response(
                Protocol::Instant,
                StatusCode::OK,
                b"{\"Abstract\":\"\xff\"}",
                "test",
            ),
            Err(SearchBackendError::Parse)
        ));
    }
}
