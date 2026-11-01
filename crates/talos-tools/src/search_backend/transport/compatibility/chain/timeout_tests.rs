use super::*;
use crate::search_backend::{
    SearchBackendId,
    transport::{
        BodyFormat, PreparedTransportRequest, SearchDns, SearchTransport, TransportBudget,
        tests::Server,
    },
};
use async_trait::async_trait;
use reqwest::Url;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use talos_core::tool::{
    ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkScope,
};

fn manifest() -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            ORDER
                .iter()
                .map(|protocol| protocol.destination().expect("fixed destination"))
                .collect(),
            None,
            vec![],
            0,
        )
        .expect("scope"),
    )
}

fn valid(protocol: CompatibilityProtocol) -> String {
    let body =
        String::from_utf8(super::super::execution_tests::body(protocol)).expect("fixture UTF-8");
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn relocate_request(
    request: &mut PreparedTransportRequest,
    manifest: &ToolNetworkManifest,
    url: Url,
) {
    let headers = request.headers;
    let mut relocated = PreparedTransportRequest::new(
        manifest,
        url,
        request.method.clone(),
        request.body.take(),
        BodyFormat::Form,
    )
    .expect("relocated request");
    relocated.headers = headers;
    *request = relocated;
}

fn relocate_with(
    mut chain: PreparedCompatibilityChain,
    url_for: impl Fn(&str) -> Url,
) -> (PreparedCompatibilityChain, ToolNetworkManifest) {
    let urls: Vec<_> = chain
        .sources
        .iter()
        .map(|source| {
            let mut url = url_for(source.request.url.path());
            url.set_query(source.request.url.query());
            url
        })
        .collect();
    let scope = ToolNetworkScope::new_configuration(
        chain
            .sources
            .iter()
            .zip(&urls)
            .map(|(source, url)| {
                super::super::super::request_endpoint(url, &source.request.method)
                    .expect("endpoint")
            })
            .collect(),
        None,
        vec!["127.0.0.1".parse().expect("loopback")],
        0,
    )
    .expect("loopback scope");
    let manifest = ToolNetworkManifest::new(scope);
    for (source, url) in chain.sources.iter_mut().zip(urls) {
        relocate_request(&mut source.request, &manifest, url.clone());
        if let Some(retry) = source.retry.as_mut() {
            relocate_request(retry, &manifest, url);
        }
    }
    (chain, manifest)
}

fn relocate(
    chain: PreparedCompatibilityChain,
    server: &Server,
) -> (PreparedCompatibilityChain, ToolNetworkManifest) {
    relocate_with(chain, |path| server.url(path))
}

struct AllowAuthority(ToolNetworkManifest);

impl ToolNetworkAuthority for AllowAuthority {
    fn check(
        &self,
        invocation_id: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if invocation_id == self.0.invocation_id() && self.0.scope().contains(endpoint) {
            Ok(())
        } else {
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }
}

struct StallDns {
    calls: AtomicUsize,
    stalls: usize,
}

#[async_trait]
impl SearchDns for StallDns {
    async fn resolve(&self, _host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        let index = self.calls.fetch_add(1, Ordering::Relaxed);
        if index < self.stalls {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

struct LoopbackDns(AtomicUsize);

#[async_trait]
impl SearchDns for LoopbackDns {
    async fn resolve(&self, _host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

struct DelayedServer {
    port: u16,
    accepted: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl DelayedServer {
    fn new(first_delay: Duration, responses: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback fixture");
        listener.set_nonblocking(true).expect("nonblocking");
        let port = listener.local_addr().expect("address").port();
        let accepted = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let accepted_worker = accepted.clone();
        let stop_worker = stop.clone();
        let worker = thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !stop_worker.load(Ordering::Relaxed) && std::time::Instant::now() < deadline {
                let (mut socket, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("fixture accept: {error}"),
                };
                let index = accepted_worker.fetch_add(1, Ordering::Relaxed);
                socket
                    .set_read_timeout(Some(Duration::from_millis(50)))
                    .expect("read timeout");
                let mut request = Vec::new();
                let mut buffer = [0; 1024];
                loop {
                    match socket.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(size) => request.extend_from_slice(&buffer[..size]),
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) => {}
                        Err(error) => panic!("fixture read: {error}"),
                    }
                    if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&request[..end]);
                        let body = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|value| value.trim().parse::<usize>().expect("length"))
                            })
                            .unwrap_or(0);
                        if request.len() >= end + 4 + body {
                            break;
                        }
                    }
                }
                if index == 0 {
                    thread::sleep(first_delay);
                }
                if let Some(response) = responses.get(index) {
                    let _ = socket.write_all(response.as_bytes());
                }
            }
        });
        Self {
            port,
            accepted,
            stop,
            worker: Some(worker),
        }
    }

    fn url(&self, path: &str) -> Url {
        Url::parse(&format!(
            "http://search-fixture.invalid:{}{path}",
            self.port
        ))
        .expect("fixture url")
    }

    fn count(&self) -> usize {
        self.accepted.load(Ordering::Relaxed)
    }
}

impl Drop for DelayedServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.join().expect("fixture cleanup");
        }
    }
}

#[tokio::test]
async fn stalled_primary_retries_before_reachable_fallback() {
    // Both Lite attempts time out locally, then the already-approved Instant Answer fallback runs.
    let server = Server::new(vec![valid(CompatibilityProtocol::InstantAnswer)]);
    let chain = PreparedCompatibilityChain::prepare(&manifest(), "fixture query", 1)
        .expect("prepared chain");
    let (chain, manifest) = relocate(chain, &server);
    let dns = Arc::new(StallDns {
        calls: AtomicUsize::new(0),
        stalls: 2,
    });
    let transport = SearchTransport::new(
        manifest.clone(),
        Arc::new(AllowAuthority(manifest)),
        Arc::new(
            TransportBudget::new(Duration::from_secs(2), 4, 1_048_576).expect("shared budget"),
        ),
        dns.clone(),
    )
    .expect("transport");

    let rows = chain
        .execute_with_policy(
            &transport,
            Duration::from_millis(200),
            Duration::from_millis(10),
        )
        .await
        .expect("reachable fallback")
        .into_results();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].backend, SearchBackendId::DuckDuckGo);
    assert_eq!(dns.calls.load(Ordering::Relaxed), 3);
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn source_timeout_retry_can_recover_without_later_query_disclosure() {
    let server = Server::new(vec![valid(CompatibilityProtocol::Lite)]);
    let chain = PreparedCompatibilityChain::prepare(&manifest(), "fixture retry query", 1)
        .expect("prepared chain");
    let (chain, manifest) = relocate(chain, &server);
    let dns = Arc::new(StallDns {
        calls: AtomicUsize::new(0),
        stalls: 1,
    });
    let transport = SearchTransport::new(
        manifest.clone(),
        Arc::new(AllowAuthority(manifest)),
        Arc::new(
            TransportBudget::new(Duration::from_secs(1), 3, 1_048_576).expect("shared budget"),
        ),
        dns.clone(),
    )
    .expect("transport");

    let rows = chain
        .execute_with_policy(
            &transport,
            Duration::from_millis(100),
            Duration::from_millis(10),
        )
        .await
        .expect("same-source retry")
        .into_results();

    assert_eq!(rows.len(), 1);
    assert_eq!(dns.calls.load(Ordering::Relaxed), 2);
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn slow_successful_primary_does_not_disclose_query_to_later_source() {
    // The first request is fully received before its response is delayed. Its response arrives
    // after one quarter of the enclosing budget (the old equal-share cutoff) but before the
    // configured per-source window. A correct protected chain therefore returns the primary result
    // without opening a second query-bearing connection.
    let server = DelayedServer::new(
        Duration::from_millis(350),
        vec![
            valid(CompatibilityProtocol::Lite),
            valid(CompatibilityProtocol::InstantAnswer),
        ],
    );
    let chain = PreparedCompatibilityChain::prepare(&manifest(), "fixture disclosure query", 1)
        .expect("prepared chain");
    let (chain, manifest) = relocate_with(chain, |path| server.url(path));
    let dns = Arc::new(LoopbackDns(AtomicUsize::new(0)));
    let transport = SearchTransport::new(
        manifest.clone(),
        Arc::new(AllowAuthority(manifest)),
        Arc::new(
            TransportBudget::new(Duration::from_secs(1), 4, 1_048_576).expect("shared budget"),
        ),
        dns.clone(),
    )
    .expect("transport");

    let rows = chain
        .execute_with_source_timeout(&transport, Duration::from_millis(500))
        .await
        .expect("slow primary success")
        .into_results();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].backend, SearchBackendId::DuckDuckGo);
    assert_eq!(dns.0.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
