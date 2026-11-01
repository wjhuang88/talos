use super::*;
use crate::search_backend::transport::{
    BodyFormat, PreparedTransportRequest, SearchDns, TransportBudget, tests::Server,
};
use crate::web_search::WebSearchTool;
use async_trait::async_trait;
use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair};
use std::{
    collections::HashMap,
    fs, io,
    net::SocketAddr,
    process::Command,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};
use talos_core::tool::{
    ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkScope,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};

const QUERY: &str = "private parity query";
const SLOW_PRIMARY_DELAY: Duration = Duration::from_millis(4_100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    SlowPrimary,
    SameDestinationRetry,
    TerminalPrimaryFallback,
    UnreachableDuckDuckGo,
}

impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::SlowPrimary => "slow-primary",
            Self::SameDestinationRetry => "same-destination-retry",
            Self::TerminalPrimaryFallback => "terminal-primary-fallback",
            Self::UnreachableDuckDuckGo => "unreachable-duckduckgo",
        }
    }

    fn expected_queries(self) -> &'static [&'static str] {
        match self {
            Self::SlowPrimary => &["lite"],
            Self::SameDestinationRetry => &["lite", "lite"],
            Self::TerminalPrimaryFallback => &["lite", "instant"],
            Self::UnreachableDuckDuckGo => &["bing"],
        }
    }

    fn expected_prepared_queries(self) -> &'static [&'static str] {
        match self {
            Self::SameDestinationRetry => &["lite"],
            _ => self.expected_queries(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedRequest {
    host: String,
    method: String,
    target: String,
    body: Vec<u8>,
}

impl ObservedRequest {
    fn query_recipient(&self) -> Option<&'static str> {
        match (
            self.host.as_str(),
            self.method.as_str(),
            self.target.split('?').next()?,
        ) {
            ("lite.duckduckgo.com" | "source-lite.test", "POST", "/lite/") => Some("lite"),
            ("api.duckduckgo.com" | "source-instant.test", "GET", "/") => Some("instant"),
            ("html.duckduckgo.com" | "source-html.test", "GET", "/html/") => Some("html"),
            ("cn.bing.com" | "source-bing.test", "GET", "/search") => Some("bing"),
            _ => None,
        }
    }

    fn query_free_probe(&self) -> Option<(&'static str, &'static str)> {
        if self.target != "/" || !self.body.is_empty() {
            return None;
        }
        let family = match self.host.as_str() {
            "duckduckgo.com" | "probe-ddg.test" => "ddg",
            "cn.bing.com" | "probe-bing.test" => "bing",
            _ => return None,
        };
        let method = match self.method.as_str() {
            "HEAD" => "HEAD",
            "GET" => "GET",
            _ => return None,
        };
        Some((family, method))
    }
}

#[test]
fn actual_legacy_and_prepared_paths_match_query_recipient_sequences() {
    let ca = TestCa::new();
    let output = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "search_backend::transport::compatibility::chain::legacy_parity_tests::legacy_parity_child",
            "--nocapture",
        ])
        .env("TALOS_SEARCH_PARITY_CHILD", "1")
        .env("TALOS_SEARCH_PARITY_CA", &ca.root_pem)
        .env("TALOS_SEARCH_PARITY_CA_KEY", &ca.root_key)
        .env("SSL_CERT_FILE", &ca.root_pem)
        .output()
        .expect("spawn isolated parity child");
    assert!(
        output.status.success(),
        "parity child failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct TestCa {
    _directory: tempfile::TempDir,
    root_pem: std::path::PathBuf,
    root_key: std::path::PathBuf,
}

impl TestCa {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary CA directory");
        let root_key_pair = KeyPair::generate().expect("temporary test CA key");
        let root = ca_params()
            .self_signed(&root_key_pair)
            .expect("temporary test CA certificate");
        let root_pem = directory.path().join("root.pem");
        let root_key = directory.path().join("root-key.pem");
        fs::write(&root_pem, root.pem()).expect("write temporary test CA");
        fs::write(&root_key, root_key_pair.serialize_pem()).expect("write temporary test key");
        Self {
            _directory: directory,
            root_pem,
            root_key,
        }
    }
}

#[tokio::test]
async fn legacy_parity_child() {
    if std::env::var_os("TALOS_SEARCH_PARITY_CHILD").is_none() {
        return;
    }
    let root_key = KeyPair::from_pem(
        &fs::read_to_string(
            std::env::var_os("TALOS_SEARCH_PARITY_CA_KEY").expect("test CA key path"),
        )
        .expect("read test CA key"),
    )
    .expect("parse test CA key");
    let root_params = ca_params();
    let issuer = Issuer::from_params(&root_params, &root_key);
    let mut leaf_params = CertificateParams::new(
        [
            "duckduckgo.com",
            "lite.duckduckgo.com",
            "api.duckduckgo.com",
            "html.duckduckgo.com",
            "cn.bing.com",
        ]
        .map(str::to_owned)
        .to_vec(),
    )
    .expect("test leaf SANs");
    leaf_params.is_ca = IsCa::NoCa;
    let leaf_key = KeyPair::generate().expect("test leaf key");
    let leaf = leaf_params
        .signed_by(&leaf_key, &issuer)
        .expect("test leaf certificate");
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der()));
    let provider = Arc::new(tokio_rustls::rustls::crypto::aws_lc_rs::default_provider());
    let tls = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("safe TLS versions")
        .with_no_client_auth()
        .with_single_cert(vec![leaf.der().clone()], key)
        .expect("test TLS server config");
    let tls = TlsAcceptor::from(Arc::new(tls));

    for case in [
        Case::SlowPrimary,
        Case::SameDestinationRetry,
        Case::TerminalPrimaryFallback,
        Case::UnreachableDuckDuckGo,
    ] {
        rust_websearch::clear_probe_cache();
        super::super::probe::clear_cache_for_parity_test();
        let legacy_proxy = LegacyProxy::start(case, tls.clone()).await;
        let legacy_tool = WebSearchTool::for_legacy_parity_test(&legacy_proxy.url);
        assert_eq!(
            legacy_tool.execute_legacy_parity_test(QUERY).await,
            1,
            "{} legacy result",
            case.name()
        );
        let legacy_requests = legacy_proxy.requests();
        legacy_proxy.stop().await;

        let (protected, dns_attempts) = run_prepared_case(case).await;
        let legacy_queries = query_recipients(&legacy_requests);
        let protected_queries = query_recipients(&protected);
        assert_eq!(
            legacy_queries,
            case.expected_queries(),
            "{} actual legacy query sequence",
            case.name()
        );
        assert_eq!(
            protected_queries,
            case.expected_prepared_queries(),
            "{} prepared query sequence",
            case.name()
        );
        if case == Case::SameDestinationRetry {
            assert_eq!(
                dns_attempts
                    .iter()
                    .filter(|host| host.as_str() == "source-lite.test")
                    .count(),
                2,
                "retry stays on the prepared Lite destination"
            );
        }
        assert!(
            protected_queries.len() <= legacy_queries.len(),
            "{} protected query count exceeds legacy",
            case.name()
        );
        assert!(
            legacy_queries.starts_with(&protected_queries),
            "{} protected query sequence is not a legacy prefix",
            case.name()
        );

        let legacy_probes = query_free_probes(&legacy_requests);
        let protected_probes = query_free_probes(&protected);
        assert_eq!(
            sorted_probes(legacy_probes),
            sorted_probes(protected_probes),
            "{} query-free probe set",
            case.name()
        );
        assert!(
            legacy_requests
                .iter()
                .filter(|request| request.query_free_probe().is_some())
                .all(|request| !request.target.contains('?') && request.body.is_empty()),
            "{} legacy probe disclosed query",
            case.name()
        );
    }
}

fn ca_params() -> CertificateParams {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
}

fn query_recipients(requests: &[ObservedRequest]) -> Vec<&'static str> {
    requests
        .iter()
        .filter_map(ObservedRequest::query_recipient)
        .collect()
}

fn query_free_probes(requests: &[ObservedRequest]) -> Vec<(&'static str, &'static str)> {
    requests
        .iter()
        .filter_map(ObservedRequest::query_free_probe)
        .collect()
}

fn sorted_probes(
    mut probes: Vec<(&'static str, &'static str)>,
) -> Vec<(&'static str, &'static str)> {
    probes.sort_unstable();
    probes
}

async fn run_prepared_case(case: Case) -> (Vec<ObservedRequest>, Vec<String>) {
    let fixtures = PreparedFixtures::new(case);
    let original = fixed_manifest();
    let chain = PreparedCompatibilityChain::prepare(&original, QUERY, 1).expect("prepared chain");
    let (chain, manifest, probe_urls, dns_ports) = relocate_for_case(chain, &fixtures);
    let probes = super::super::probe::PreparedReachabilityProbes::relocate_for_parity_test(
        &manifest,
        probe_urls[0].clone(),
        probe_urls[1].clone(),
    )
    .expect("prepared query-free probes");
    let effects = Arc::new(TestEffects {
        manifest: manifest.clone(),
        dns_ports,
        case,
        dns_attempts: Mutex::new(Vec::new()),
        network_failed: std::sync::atomic::AtomicBool::new(false),
    });
    let transport = SearchTransport::new(
        manifest,
        effects.clone(),
        Arc::new(
            TransportBudget::new(Duration::from_secs(30), 16, 1_048_576).expect("parity budget"),
        ),
        effects.clone(),
    )
    .expect("prepared transport");
    let reachability = probes
        .execute(&transport)
        .await
        .expect("reachability probes");
    if let Err(error) = chain
        .execute_with_reachability(&transport, reachability)
        .await
    {
        panic!(
            "prepared search result failed: {error:?}; fixture requests: {:?}",
            fixtures.requests()
        );
    }
    let dns_attempts = effects
        .dns_attempts
        .lock()
        .expect("DNS attempt log")
        .clone();
    (fixtures.requests(), dns_attempts)
}

fn fixed_manifest() -> ToolNetworkManifest {
    let mut endpoints: Vec<_> = ORDER
        .iter()
        .map(|protocol| protocol.destination().expect("fixed query endpoint"))
        .collect();
    endpoints.extend(super::super::probe::destinations().expect("fixed probe endpoints"));
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(endpoints, None, vec![], 0).expect("fixed scope"),
    )
}

fn relocate_for_case(
    mut chain: PreparedCompatibilityChain,
    fixtures: &PreparedFixtures,
) -> (
    PreparedCompatibilityChain,
    ToolNetworkManifest,
    [reqwest::Url; 2],
    HashMap<String, u16>,
) {
    let query_urls: Vec<_> = chain
        .sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let mut url = fixtures.query[index].url(source.request.url.path());
            url.set_host(Some(&format!(
                "source-{}.test",
                ["lite", "instant", "html", "bing"][index]
            )))
            .expect("query fixture host");
            url.set_query(source.request.url.query());
            url
        })
        .collect();
    let probe_urls = std::array::from_fn(|index| {
        let mut url = fixtures.probes[index].url("/");
        url.set_host(Some(if index == 0 {
            "probe-ddg.test"
        } else {
            "probe-bing.test"
        }))
        .expect("probe fixture host");
        url
    });
    let mut endpoints = Vec::new();
    let mut dns_ports = HashMap::new();
    for (source, url) in chain.sources.iter().zip(&query_urls) {
        endpoints.push(
            super::super::super::request_endpoint(url, &source.request.method)
                .expect("query endpoint"),
        );
        dns_ports.insert(
            url.host_str().expect("query fixture hostname").to_owned(),
            url.port().expect("query fixture port"),
        );
    }
    for url in &probe_urls {
        endpoints.push(
            super::super::super::request_endpoint(url, &reqwest::Method::HEAD)
                .expect("HEAD probe endpoint"),
        );
        endpoints.push(
            super::super::super::request_endpoint(url, &reqwest::Method::GET)
                .expect("GET probe endpoint"),
        );
        dns_ports.insert(
            url.host_str().expect("probe fixture hostname").to_owned(),
            url.port().expect("probe fixture port"),
        );
    }
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            endpoints,
            None,
            vec!["127.0.0.1".parse().expect("loopback")],
            0,
        )
        .expect("loopback scope"),
    );
    for (source, url) in chain.sources.iter_mut().zip(query_urls) {
        relocate_request(&mut source.request, &manifest, url.clone());
        if let Some(retry) = source.retry.as_mut() {
            relocate_request(retry, &manifest, url);
        }
    }
    (chain, manifest, probe_urls, dns_ports)
}

fn relocate_request(
    request: &mut PreparedTransportRequest,
    manifest: &ToolNetworkManifest,
    url: reqwest::Url,
) {
    let headers = request.headers;
    let body = request.body.clone();
    let mut relocated = PreparedTransportRequest::new(
        manifest,
        url,
        request.method.clone(),
        request.body.take(),
        BodyFormat::Form,
    )
    .expect("relocated prepared request");
    relocated.headers = headers;
    assert_eq!(relocated.body, body);
    *request = relocated;
}

struct TestEffects {
    manifest: ToolNetworkManifest,
    dns_ports: HashMap<String, u16>,
    case: Case,
    dns_attempts: Mutex<Vec<String>>,
    network_failed: std::sync::atomic::AtomicBool,
}

impl ToolNetworkAuthority for TestEffects {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if invocation == self.manifest.invocation_id() && self.manifest.scope().contains(endpoint) {
            Ok(())
        } else {
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }
}

#[async_trait]
impl SearchDns for TestEffects {
    async fn resolve(&self, host: &str, _: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.dns_attempts
            .lock()
            .expect("DNS attempt log")
            .push(host.to_owned());
        if self.case == Case::SameDestinationRetry
            && host == "source-lite.test"
            && !self.network_failed.swap(true, Ordering::Relaxed)
        {
            return Err(TransportError::Network);
        }
        let port = self.dns_ports.get(host).ok_or(TransportError::Dns)?;
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], *port))])
    }
}

struct PreparedFixtures {
    query: [Server; 4],
    probes: [Server; 2],
}

impl PreparedFixtures {
    fn new(case: Case) -> Self {
        let lite_result = html_result();
        let (lite, delay) = match case {
            Case::SlowPrimary => (
                vec![wire_response(200, lite_result.as_bytes())],
                vec![SLOW_PRIMARY_DELAY],
            ),
            Case::SameDestinationRetry => {
                (vec![wire_response(200, lite_result.as_bytes())], Vec::new())
            }
            Case::TerminalPrimaryFallback => (vec![wire_response(403, &[])], Vec::new()),
            Case::UnreachableDuckDuckGo => (vec![wire_response(503, &[])], Vec::new()),
        };
        let instant = if case == Case::TerminalPrimaryFallback {
            vec![wire_response(200, instant_result().as_bytes())]
        } else {
            vec![wire_response(503, &[])]
        };
        let bing = if case == Case::UnreachableDuckDuckGo {
            vec![wire_response(200, bing_result().as_bytes())]
        } else {
            vec![wire_response(503, &[])]
        };
        let query = [
            Server::new_with_delays(lite, delay),
            Server::new(instant),
            Server::new(vec![wire_response(503, &[])]),
            Server::new(bing),
        ];
        let ddg_probe = if case == Case::UnreachableDuckDuckGo {
            vec![wire_response(503, &[]), wire_response(503, &[])]
        } else {
            vec![wire_response(200, &[])]
        };
        let probes = [
            Server::new(ddg_probe),
            Server::new(vec![wire_response(200, &[])]),
        ];
        Self { query, probes }
    }

    fn requests(&self) -> Vec<ObservedRequest> {
        let mut requests = Vec::new();
        for server in self.query.iter().chain(&self.probes) {
            let count = server.count();
            for index in 0..count {
                requests.push((
                    server.request_time(index),
                    parse_observed_request(&server.request(index)),
                ));
            }
        }
        requests.sort_by_key(|(time, _)| *time);
        requests.into_iter().map(|(_, request)| request).collect()
    }
}

fn html_result() -> String {
    r#"<html><body><a class="result-link" href="https://example.test/">Title</a><td class="result-snippet">Snippet</td></body></html>"#.to_owned()
}

fn instant_result() -> String {
    r#"{"AbstractText":"Answer","AbstractURL":"https://example.test/","AbstractSource":"Example","Heading":"Answer","Answer":"","AnswerType":"","Definition":"","DefinitionSource":"","RelatedTopics":[],"Infobox":null,"Type":"A"}"#.to_owned()
}

fn bing_result() -> String {
    r#"<html><body><ol id="b_results"><li class="b_algo"><h2><a href="https://example.test/">Title</a></h2><div class="b_caption"><p class="b_lineclamp2">Snippet</p></div></li></ol></body></html>"#.to_owned()
}

fn wire_response(status: u16, body: &[u8]) -> String {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        503 => "Unavailable",
        _ => "Fixture",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    response.push_str(std::str::from_utf8(body).expect("fixture response UTF-8"));
    response
}

fn parse_observed_request(raw: &str) -> ObservedRequest {
    let (head, body) = raw.split_once("\r\n\r\n").expect("captured request");
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next().expect("request line").split_whitespace();
    let method = request_line.next().expect("method").to_owned();
    let target = request_line.next().expect("target").to_owned();
    let host = lines
        .filter_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("host:")
                .map(str::to_owned)
        })
        .next()
        .expect("Host header")
        .trim()
        .split(':')
        .next()
        .expect("hostname")
        .to_owned();
    ObservedRequest {
        host,
        method,
        target,
        body: body.as_bytes().to_vec(),
    }
}

#[derive(Clone)]
enum ResponseAction {
    Close,
    Reply(Duration, Vec<u8>),
}

fn html_success(delay: Duration) -> ResponseAction {
    let body = br#"<html><body><a class="result-link" href="https://example.test/">Title</a><td class="result-snippet">Snippet</td></body></html>"#;
    ResponseAction::Reply(delay, status_response(200, body))
}

fn instant_success() -> ResponseAction {
    let body = br#"{"AbstractText":"Answer","AbstractURL":"https://example.test/","AbstractSource":"Example","Heading":"Answer","Answer":"","AnswerType":"","Definition":"","DefinitionSource":"","RelatedTopics":[],"Infobox":null,"Type":"A"}"#;
    ResponseAction::Reply(Duration::ZERO, status_response(200, body))
}

fn bing_success() -> ResponseAction {
    let body = br#"<html><body><ol id="b_results"><li class="b_algo"><h2><a href="https://example.test/">Title</a></h2><div class="b_caption"><p class="b_lineclamp2">Snippet</p></div></li></ol></body></html>"#;
    ResponseAction::Reply(Duration::ZERO, status_response(200, body))
}

fn status_response(status: u16, body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        403 => "Forbidden",
        503 => "Unavailable",
        _ => "Fixture",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut bytes = response.into_bytes();
    bytes.extend_from_slice(body);
    bytes
}

async fn read_until_headers(stream: &mut impl AsyncReadExtCompat) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    while !data.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        stream.read_exact(&mut byte).await?;
        data.push(byte[0]);
        if data.len() > 32_768 {
            return Err(io::Error::other("fixture header bound"));
        }
    }
    Ok(data)
}

trait AsyncReadExtCompat: tokio::io::AsyncRead + Unpin {}
impl<T: tokio::io::AsyncRead + Unpin> AsyncReadExtCompat for T {}

async fn parse_request(
    stream: &mut impl AsyncReadExtCompat,
    head: &[u8],
) -> io::Result<ObservedRequest> {
    let text = std::str::from_utf8(head).map_err(io::Error::other)?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let method = first.next().unwrap_or_default().to_owned();
    let target = first.next().unwrap_or_default().to_owned();
    let mut host = String::new();
    let mut content_length = 0usize;
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("host:") {
            host = value
                .trim()
                .split(':')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
        }
        if let Some(value) = lower.strip_prefix("content-length:") {
            content_length = value.trim().parse().map_err(io::Error::other)?;
        }
    }
    if content_length > 16_384 {
        return Err(io::Error::other("fixture body bound"));
    }
    let mut body = vec![0; content_length];
    stream.read_exact(&mut body).await?;
    Ok(ObservedRequest {
        host,
        method,
        target,
        body,
    })
}

struct LegacyProxy {
    url: String,
    requests: Arc<Mutex<Vec<ObservedRequest>>>,
    task: JoinHandle<()>,
}

impl LegacyProxy {
    async fn start(case: Case, tls: TlsAcceptor) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("legacy proxy listener");
        let port = listener.local_addr().expect("proxy address").port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let query_counts = Arc::new(Mutex::new(HashMap::new()));
        let recorded = requests.clone();
        let counts = query_counts.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let tls = tls.clone();
                let recorded = recorded.clone();
                let counts = counts.clone();
                tokio::spawn(async move {
                    let _ = handle_legacy_connect(stream, tls, case, recorded, counts).await;
                });
            }
        });
        Self {
            url: format!("http://127.0.0.1:{port}"),
            requests,
            task,
        }
    }

    fn requests(&self) -> Vec<ObservedRequest> {
        self.requests.lock().expect("legacy request log").clone()
    }

    async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }
}

async fn handle_legacy_connect(
    mut stream: TcpStream,
    tls: TlsAcceptor,
    case: Case,
    requests: Arc<Mutex<Vec<ObservedRequest>>>,
    query_counts: Arc<Mutex<HashMap<&'static str, usize>>>,
) -> io::Result<()> {
    let connect = read_until_headers(&mut stream).await?;
    let line = std::str::from_utf8(&connect)
        .map_err(io::Error::other)?
        .lines()
        .next()
        .unwrap_or_default();
    let authority = line
        .strip_prefix("CONNECT ")
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_default();
    let host = authority
        .strip_suffix(":443")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        host.as_str(),
        "duckduckgo.com"
            | "lite.duckduckgo.com"
            | "api.duckduckgo.com"
            | "html.duckduckgo.com"
            | "cn.bing.com"
    ) {
        stream
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
            .await?;
        return Ok(());
    }
    stream
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await?;
    let mut tls_stream = tls.accept(stream).await.map_err(io::Error::other)?;
    let request = read_until_headers(&mut tls_stream).await?;
    let parsed = parse_request(&mut tls_stream, &request).await?;
    let recipient = parsed.query_recipient();
    if let Some(recipient) = recipient {
        let mut counts = query_counts.lock().expect("legacy query counts");
        *counts.entry(recipient).or_default() += 1;
    }
    requests
        .lock()
        .expect("legacy request log")
        .push(parsed.clone());
    let attempt = recipient
        .and_then(|recipient| {
            query_counts
                .lock()
                .expect("legacy counts")
                .get(recipient)
                .copied()
        })
        .unwrap_or(0);
    match legacy_response(case, &parsed, recipient, attempt) {
        ResponseAction::Close => Ok(()),
        ResponseAction::Reply(delay, response) => {
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            tls_stream.write_all(&response).await?;
            Ok(())
        }
    }
}

fn legacy_response(
    case: Case,
    request: &ObservedRequest,
    recipient: Option<&'static str>,
    attempt: usize,
) -> ResponseAction {
    if let Some((family, _)) = request.query_free_probe() {
        let reachable = !(case == Case::UnreachableDuckDuckGo && family == "ddg");
        return ResponseAction::Reply(
            Duration::ZERO,
            status_response(if reachable { 200 } else { 503 }, &[]),
        );
    }
    match (case, recipient, attempt) {
        (Case::SlowPrimary, Some("lite"), _) => html_success(SLOW_PRIMARY_DELAY),
        (Case::SameDestinationRetry, Some("lite"), 1) => ResponseAction::Close,
        (Case::SameDestinationRetry, Some("lite"), _) => html_success(Duration::ZERO),
        (Case::TerminalPrimaryFallback, Some("lite"), _) => {
            ResponseAction::Reply(Duration::ZERO, status_response(403, &[]))
        }
        (Case::TerminalPrimaryFallback, Some("instant"), _) => instant_success(),
        (Case::UnreachableDuckDuckGo, Some("bing"), _) => bing_success(),
        _ => ResponseAction::Reply(Duration::ZERO, status_response(503, &[])),
    }
}
