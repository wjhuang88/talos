use super::*;
mod connect;
mod headers;
mod prepared;
mod tls;
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::TcpListener,
    sync::{Mutex, atomic::AtomicBool},
    thread,
};
use talos_core::tool::{ToolNetworkAuthorizationError, ToolNetworkScope};

fn form_fields(fields: &[(&str, &str)]) -> Vec<(String, String)> {
    fields
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn form_encoding_preserves_unicode_reserved_bytes_duplicates_empty_fields_and_order() {
    let fields = form_fields(&[("q", "中 +&=~\r\n"), ("q", "*._-"), ("", ""), ("a", "")]);
    assert_eq!(
        encode_form(&fields).expect("form"),
        b"q=%E4%B8%AD+%2B%26%3D%7E%0D%0A&q=*._-&=&a="
    );
    assert!(encode_form(&[]).expect("empty form").is_empty());
    assert_eq!(
        encode_form(&vec![(String::new(), String::new()); 8192])
            .expect("maximum empty fields")
            .len(),
        16_383
    );
}

#[test]
fn bounded_generated_form_fields_round_trip_through_standard_url_decoding() {
    // Bounded generated property coverage without adding a new test dependency.
    let samples: Vec<String> = (0..=255)
        .filter_map(char::from_u32)
        .map(|c| format!("{c}中🙂 +~&=\0"))
        .collect();
    for value in samples {
        let fields = vec![(value.clone(), value), (String::new(), String::new())];
        let bytes = encode_form(&fields).expect("bounded generated input");
        assert!(bytes.len() <= 16_384);
        let mut encoded = Url::parse("https://decode.invalid/").expect("container");
        encoded.set_query(Some(std::str::from_utf8(&bytes).expect("ASCII")));
        assert_eq!(
            encoded.query_pairs().into_owned().collect::<Vec<_>>(),
            fields
        );
    }
}

#[tokio::test]
async fn real_form_post_has_fixed_type_and_exact_encoded_body_at_the_limit() {
    for (value, encoded_value) in [
        ("a".repeat(16_382), "a".repeat(16_382)),
        (
            format!("{}aa", "中".repeat(1820)),
            format!("{}aa", "%E4%B8%AD".repeat(1820)),
        ),
    ] {
        let server = Server::new(vec![ok("form")]);
        let url = server.url("/search?fixed=1");
        let t = transport(
            manifest(&[(&url, Method::POST)], loopback(), 0),
            dns(server.port),
            Duration::from_secs(3),
            1,
            1024,
        );
        let fields = form_fields(&[("q", &value)]);
        assert_eq!(
            t.execute_form(url, fields).await.expect("form POST").body,
            b"form"
        );
        let request = server.request(0);
        assert!(request.starts_with("POST /search?fixed=1 HTTP/1.1\r\n"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("content-type: application/x-www-form-urlencoded\r\n")
        );
        assert_eq!(
            request.split_once("\r\n\r\n").expect("body").1,
            format!("q={encoded_value}")
        );
        assert_eq!(encoded_value.len() + 2, 16_384);
        assert_eq!(server.count(), 1);
    }
}

#[tokio::test]
async fn form_overflow_expansion_and_field_count_are_rejected_without_dns_or_budget_use() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::POST)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(3),
        1,
        1024,
    );
    for fields in [
        form_fields(&[("q", &"a".repeat(16_383))]),
        form_fields(&[("q", &"中".repeat(1821))]),
        form_fields(&[("q", &format!("{}aaa", "中".repeat(1820)))]),
        vec![(String::new(), String::new()); 8193],
    ] {
        assert_eq!(
            t.execute_form(url.clone(), fields)
                .await
                .expect_err("oversized form"),
            TransportError::InvalidRequest
        );
    }
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn form_same_origin_redirect_retains_body_and_type_but_other_post_redirects_fail() {
    for status in [301, 302, 303, 307, 308] {
        let server = Server::new(vec![
            format!(
                "HTTP/1.1 {status} Redirect\r\nLocation: /target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            ),
            ok("form"),
        ]);
        let url = server.url("/search");
        let next = server.url("/target");
        let t = transport(
            manifest(
                &[(&url, Method::POST), (&next, Method::POST)],
                loopback(),
                1,
            ),
            dns(server.port),
            Duration::from_secs(3),
            2,
            1024,
        );
        let result = t.execute_form(url, form_fields(&[("q", "toy +&")])).await;
        if matches!(status, 307 | 308) {
            assert_eq!(result.expect("preserved POST").body, b"form");
            for i in [0, 1] {
                let request = server.request(i);
                assert!(
                    request
                        .to_ascii_lowercase()
                        .contains("content-type: application/x-www-form-urlencoded\r\n")
                );
                assert!(request.ends_with("q=toy+%2B%26"));
            }
            assert_eq!(server.count(), 2);
        } else {
            assert_eq!(
                result.expect_err("method-changing redirect"),
                TransportError::Redirect
            );
            assert_eq!(server.count(), 1);
        }
    }
}

#[tokio::test]
async fn form_cross_origin_redirect_never_forwards_body_even_when_target_is_scoped() {
    let target = Server::new(vec![ok("unexpected")]);
    let next = target.url("/target");
    let source = Server::new(vec![redirect(307, &next)]);
    let url = source.url("/search");
    let t = transport(
        manifest(
            &[(&url, Method::POST), (&next, Method::POST)],
            loopback(),
            1,
        ),
        dns(source.port),
        Duration::from_secs(3),
        2,
        1024,
    );
    assert_eq!(
        t.execute_form(url, form_fields(&[("q", "toy-secret")]))
            .await
            .expect_err("origin changed"),
        TransportError::Redirect
    );
    assert!(source.request(0).ends_with("q=toy-secret"));
    assert_eq!(target.count(), 0);
}

#[tokio::test]
async fn denied_form_scope_and_expired_budget_do_not_resolve_or_start() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(3),
        1,
        1024,
    );
    assert_eq!(
        t.execute_form(url.clone(), form_fields(&[("q", "toy")]))
            .await
            .expect_err("POST not scoped"),
        TransportError::Unauthorized
    );
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 0);
    let t = transport(
        manifest(&[(&url, Method::POST)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(3),
        1,
        1024,
    );
    // A constructed expired budget exercises the entry fence without wall-clock sleeps.
    let t = SearchTransport {
        budget: Arc::new(TransportBudget {
            deadline: Instant::now(),
            attempts: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
            max_requests: 1,
            max_bytes: 1024,
        }),
        ..t
    };
    assert_eq!(
        t.execute_form(url, form_fields(&[("q", "toy")]))
            .await
            .expect_err("expired form"),
        TransportError::Deadline
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

/// Owned local socket fixture: no upstream service, region or real credential.
pub(super) struct Server {
    port: u16,
    stop: Arc<AtomicBool>,
    pub(super) accepted: Arc<AtomicUsize>,
    pub(super) closed: Arc<AtomicUsize>,
    read_errors: Arc<Mutex<Vec<std::io::ErrorKind>>>,
    read_error_count: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<Vec<u8>>>>,
    request_times: Arc<Mutex<Vec<std::time::Instant>>>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Server {
    pub(super) fn new(responses: Vec<String>) -> Self {
        Self::new_with_delays(responses, Vec::new())
    }

    pub(super) fn new_with_delays(responses: Vec<String>, response_delays: Vec<Duration>) -> Self {
        Self::new_with_resets(responses, response_delays, Vec::new())
    }

    pub(super) fn new_with_resets(
        responses: Vec<String>,
        response_delays: Vec<Duration>,
        reset_requests: Vec<usize>,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback fixture");
        listener.set_nonblocking(true).expect("nonblocking");
        let port = listener.local_addr().expect("address").port();
        let stop = Arc::new(AtomicBool::new(false));
        let accepted = Arc::new(AtomicUsize::new(0));
        let closed = Arc::new(AtomicUsize::new(0));
        let read_errors = Arc::new(Mutex::new(Vec::with_capacity(4)));
        let read_error_count = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let request_times = Arc::new(Mutex::new(Vec::new()));
        let (s, a, c, r, times, errors, error_count) = (
            stop.clone(),
            accepted.clone(),
            closed.clone(),
            requests.clone(),
            request_times.clone(),
            read_errors.clone(),
            read_error_count.clone(),
        );
        let worker = thread::spawn(move || {
            let mut responses: VecDeque<_> = responses.into();
            let mut response_delays: VecDeque<_> = response_delays.into();
            let mut request_index = 0;
            let limit = std::time::Instant::now() + Duration::from_secs(10);
            while !s.load(Ordering::Relaxed) && std::time::Instant::now() < limit {
                let (mut socket, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("fixture accept: {e}"),
                };
                a.fetch_add(1, Ordering::Relaxed);
                socket
                    .set_read_timeout(Some(Duration::from_millis(50)))
                    .expect("read timeout");
                socket
                    .set_write_timeout(Some(Duration::from_millis(50)))
                    .expect("write timeout");
                let mut bytes = Vec::new();
                let mut buffer = [0; 1024];
                while !s.load(Ordering::Relaxed) && std::time::Instant::now() < limit {
                    match socket.read(&mut buffer) {
                        Ok(0) => {
                            c.fetch_add(1, Ordering::Relaxed);
                            break;
                        }
                        Ok(n) => bytes.extend_from_slice(&buffer[..n]),
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            ) =>
                        {
                            continue;
                        }
                        Err(error) => {
                            error_count.fetch_add(1, Ordering::Relaxed);
                            let mut errors = errors.lock().expect("read error receipt");
                            if errors.len() < 4 {
                                errors.push(error.kind());
                            }
                            break;
                        }
                    }
                    assert!(bytes.len() <= 32_768, "fixture request bound");
                    if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let size = headers
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|s| s.trim().parse::<usize>().expect("length"))
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + size {
                            break;
                        }
                    }
                }
                times
                    .lock()
                    .expect("request times")
                    .push(std::time::Instant::now());
                r.lock().expect("requests").push(bytes);
                let current_request = request_index;
                request_index += 1;
                if reset_requests.contains(&current_request) {
                    socket2::SockRef::from(&socket)
                        .set_linger(Some(Duration::ZERO))
                        .expect("abort fixture connection");
                    continue;
                }
                if let Some(response) = responses.pop_front() {
                    if let Some(delay) = response_delays.pop_front()
                        && !delay.is_zero()
                    {
                        thread::sleep(delay);
                    }
                    let _ = socket.write_all(response.as_bytes());
                } else {
                    // Hold the response open and observe peer EOF on deadline/caller drop.
                    while !s.load(Ordering::Relaxed) && std::time::Instant::now() < limit {
                        match socket.read(&mut buffer) {
                            Ok(0) => {
                                c.fetch_add(1, Ordering::Relaxed);
                                break;
                            }
                            Ok(_) => {}
                            Err(e)
                                if matches!(
                                    e.kind(),
                                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                                ) => {}
                            Err(error) => {
                                error_count.fetch_add(1, Ordering::Relaxed);
                                let mut errors = errors.lock().expect("read error receipt");
                                if errors.len() < 4 {
                                    errors.push(error.kind());
                                }
                                break;
                            }
                        }
                    }
                }
            }
            // Terminal barrier: observe connections already queued before listener drop.
            while let Ok((socket, _)) = listener.accept() {
                a.fetch_add(1, Ordering::Relaxed);
                drop(socket);
            }
        });
        Self {
            port,
            stop,
            accepted,
            closed,
            read_errors,
            read_error_count,
            requests,
            request_times,
            worker: Mutex::new(Some(worker)),
        }
    }
    pub(super) fn url(&self, path: &str) -> Url {
        Url::parse(&format!(
            "http://search-fixture.invalid:{}{path}",
            self.port
        ))
        .expect("url")
    }
    pub(super) fn count(&self) -> usize {
        self.finish();
        self.accepted.load(Ordering::Relaxed)
    }
    fn finish(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.lock().expect("worker").take() {
            worker.join().expect("fixture cleanup");
        }
    }
    pub(super) fn request(&self, index: usize) -> String {
        String::from_utf8(self.requests.lock().expect("requests")[index].clone())
            .expect("http bytes")
    }
    pub(super) fn request_time(&self, index: usize) -> std::time::Instant {
        self.request_times.lock().expect("request times")[index]
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.finish();
    }
}

fn ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
fn redirect(status: u16, to: &Url) -> String {
    format!(
        "HTTP/1.1 {status} Redirect\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
}
fn endpoint(url: &Url, method: &Method) -> ToolNetworkEndpoint {
    ToolNetworkEndpoint::new(
        url.scheme(),
        url.host_str().expect("host"),
        url.port_or_known_default().expect("port"),
        method.as_str(),
        url.path(),
    )
    .expect("endpoint")
}
fn manifest(
    urls: &[(&Url, Method)],
    exceptions: Vec<IpAddr>,
    redirects: u8,
) -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            urls.iter().map(|(u, m)| endpoint(u, m)).collect(),
            None,
            exceptions,
            redirects,
        )
        .expect("scope"),
    )
}

struct Authority {
    manifest: ToolNetworkManifest,
    revoked: Arc<AtomicBool>,
}
impl ToolNetworkAuthority for Authority {
    fn check(
        &self,
        id: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if self.revoked.load(Ordering::Relaxed) {
            return Err(ToolNetworkAuthorizationError::Invalidated);
        }
        if id != self.manifest.invocation_id() || !self.manifest.scope().contains(endpoint) {
            return Err(ToolNetworkAuthorizationError::Mismatch);
        }
        Ok(())
    }
}
struct Dns {
    addresses: Vec<SocketAddr>,
    calls: AtomicUsize,
    revoke: Option<Arc<AtomicBool>>,
    delay: Duration,
}
#[async_trait]
impl SearchDns for Dns {
    async fn resolve(&self, _: &str, _: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if let Some(v) = &self.revoke {
            v.store(true, Ordering::Relaxed);
        }
        tokio::time::sleep(self.delay).await;
        Ok(self.addresses.clone())
    }
}
fn dns(port: u16) -> Arc<Dns> {
    Arc::new(Dns {
        addresses: vec![SocketAddr::from(([127, 0, 0, 1], port))],
        calls: AtomicUsize::new(0),
        revoke: None,
        delay: Duration::ZERO,
    })
}
fn transport(
    manifest: ToolNetworkManifest,
    dns: Arc<dyn SearchDns>,
    duration: Duration,
    requests: usize,
    bytes: usize,
) -> SearchTransport {
    let authority = Arc::new(Authority {
        manifest: manifest.clone(),
        revoked: Arc::new(AtomicBool::new(false)),
    });
    SearchTransport::new(
        manifest,
        authority,
        Arc::new(TransportBudget::new(duration, requests, bytes).expect("budget")),
        dns,
    )
    .expect("transport")
}
fn loopback() -> Vec<IpAddr> {
    vec![IpAddr::from([127, 0, 0, 1])]
}

#[test]
fn public_address_allowlist_rejects_special_and_translation_ranges() {
    for ip in [
        "0.1.2.3",
        "10.0.0.1",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.1.1",
        "172.16.0.1",
        "192.0.0.9",
        "192.0.2.1",
        "192.88.99.1",
        "192.168.1.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "240.0.0.1",
        "255.255.255.255",
        "::",
        "::1",
        "::ffff:127.0.0.1",
        "64:ff9b::7f00:1",
        "64:ff9b:1::1",
        "fc00::1",
        "fe80::1",
        "ff02::1",
        "2001::1",
        "2001:db8::1",
        "2002:0808:0808::1",
        "3fff::1",
        "3ffe::1",
    ] {
        assert!(!is_global_address(ip.parse().expect("ip")), "{ip}");
    }
    for ip in [
        "8.8.8.8",
        "1.1.1.1",
        "2606:4700:4700::1111",
        "::ffff:8.8.8.8",
    ] {
        assert!(is_global_address(ip.parse().expect("ip")), "{ip}");
    }
}

#[tokio::test]
async fn pinned_dns_reaches_real_socket_preserving_original_host_and_scoped_input() {
    let server = Server::new(vec![ok("fixture")]);
    let url = server.url("/search?q=fixture-query");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(2),
        1,
        1024,
    );
    let result = t
        .execute(url, Method::GET, None)
        .await
        .expect("real response");
    assert_eq!(
        (result.status, result.body.as_slice()),
        (200, b"fixture".as_slice())
    );
    let request = server.request(0).to_ascii_lowercase();
    assert!(request.starts_with("get /search?q=fixture-query http/1.1"));
    assert!(request.contains(&format!("host: search-fixture.invalid:{}", server.port)));
    assert!(
        !request.contains("referer:")
            && !request.contains("authorization:")
            && !request.contains("cookie:")
    );
}

#[tokio::test]
async fn actual_permission_once_authority_can_execute_and_clear_blocks_next_start() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
    use talos_permission::{
        GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
        PermissionRequest, PermissionSessionState,
    };
    let server = Server::new(vec![ok("authorized")]);
    let url = server.url("/search");
    let m = manifest(&[(&url, Method::GET)], loopback(), 0);
    let first_payload = PreparedTransportRequest::json_bytes(&m, url.clone(), Method::GET, None)
        .expect("prepare before approval");
    let second_payload = PreparedTransportRequest::json_bytes(&m, url.clone(), Method::GET, None)
        .expect("prepare before revocation");
    let resolver = dns(server.port);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Network,
        "search-fixture.invalid",
        ToolResourceKind::Domain,
    )];
    let input = serde_json::json!({"query":"fixture"});
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request =
        PermissionRequest::native("web_search", &facets, &input).with_network_manifest(&m);
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approve");
    let authority = state
        .admit_network(pending, &request, &context)
        .expect("admit");
    let t = SearchTransport::new(
        m,
        authority,
        Arc::new(TransportBudget::new(Duration::from_secs(2), 2, 1024).expect("budget")),
        resolver.clone(),
    )
    .expect("transport");
    assert_eq!(
        t.execute_prepared(first_payload)
            .await
            .expect("response")
            .status,
        200
    );
    let before = resolver.calls.load(Ordering::Relaxed);
    state.clear().expect("clear");
    assert_eq!(
        t.execute_prepared(second_payload)
            .await
            .expect_err("revoked"),
        TransportError::Unauthorized
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), before);
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn same_host_port_path_method_or_userinfo_changes_never_resolve_or_connect() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(2),
        2,
        1024,
    );
    let mut port = url.clone();
    port.set_port(Some(if server.port == 65535 {
        65534
    } else {
        server.port + 1
    }))
    .expect("port");
    for changed in [port, server.url("/other")] {
        assert_eq!(
            t.execute(changed, Method::GET, None)
                .await
                .expect_err("scope"),
            TransportError::Unauthorized
        );
    }
    assert_eq!(
        t.execute(url.clone(), Method::POST, None)
            .await
            .expect_err("method"),
        TransportError::Unauthorized
    );
    let mut userinfo = url;
    userinfo.set_username("fixture-secret").expect("userinfo");
    assert_eq!(
        t.execute(userinfo, Method::GET, None)
            .await
            .expect_err("userinfo"),
        TransportError::InvalidRequest
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn mixed_or_excess_dns_answers_and_unapproved_private_addresses_never_connect() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    for addresses in [
        vec![SocketAddr::from(([127, 0, 0, 1], server.port))],
        vec![
            SocketAddr::from(([8, 8, 8, 8], server.port)),
            SocketAddr::from(([127, 0, 0, 1], server.port)),
        ],
        vec![SocketAddr::from(([8, 8, 8, 8], server.port)); 17],
        vec![],
        vec![SocketAddr::new(
            "::ffff:127.0.0.1".parse().expect("ip"),
            server.port,
        )],
    ] {
        let resolver = Arc::new(Dns {
            addresses,
            calls: AtomicUsize::new(0),
            revoke: None,
            delay: Duration::ZERO,
        });
        let t = transport(
            manifest(&[(&url, Method::GET)], vec![], 0),
            resolver,
            Duration::from_secs(2),
            1,
            1024,
        );
        assert_eq!(
            t.execute(url.clone(), Method::GET, None)
                .await
                .expect_err("address"),
            TransportError::Address
        );
    }
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn mapped_address_exception_is_exact_not_an_ipv4_alias() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = Arc::new(Dns {
        addresses: vec![SocketAddr::new(
            "::ffff:127.0.0.1".parse().expect("mapped"),
            server.port,
        )],
        calls: AtomicUsize::new(0),
        revoke: None,
        delay: Duration::ZERO,
    });
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver,
        Duration::from_secs(2),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("exact exception"),
        TransportError::Address
    );
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn revoked_during_dns_is_rechecked_before_real_connection() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let m = manifest(&[(&url, Method::GET)], loopback(), 0);
    let revoked = Arc::new(AtomicBool::new(false));
    let resolver = Arc::new(Dns {
        addresses: vec![SocketAddr::from(([127, 0, 0, 1], server.port))],
        calls: AtomicUsize::new(0),
        revoke: Some(revoked.clone()),
        delay: Duration::ZERO,
    });
    let authority = Arc::new(Authority {
        manifest: m.clone(),
        revoked,
    });
    let t = SearchTransport::new(
        m,
        authority,
        Arc::new(TransportBudget::new(Duration::from_secs(2), 1, 1024).expect("budget")),
        resolver,
    )
    .expect("transport");
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("revoked"),
        TransportError::Unauthorized
    );
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn unapproved_redirect_target_receives_no_connection_query_or_body() {
    let target = Server::new(vec![ok("unexpected")]);
    let next = target.url("/target?q=fixture-secret");
    let source = Server::new(vec![redirect(302, &next)]);
    let url = source.url("/search?q=fixture-query");
    let resolver = dns(source.port);
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 1),
        resolver.clone(),
        Duration::from_secs(2),
        2,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("redirect"),
        TransportError::Unauthorized
    );
    assert_eq!(source.count(), 1);
    assert_eq!(target.count(), 0);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn post_cross_origin_redirect_does_not_forward_body_even_when_destination_is_scoped() {
    let target = Server::new(vec![ok("unexpected")]);
    let next = target.url("/target");
    let source = Server::new(vec![redirect(307, &next)]);
    let url = source.url("/search");
    let t = transport(
        manifest(
            &[(&url, Method::POST), (&next, Method::POST)],
            loopback(),
            1,
        ),
        dns(source.port),
        Duration::from_secs(2),
        2,
        1024,
    );
    assert_eq!(
        t.execute(
            url,
            Method::POST,
            Some(b"{\"api_key\":\"fixture-secret\"}".to_vec())
        )
        .await
        .expect_err("cross origin"),
        TransportError::Redirect
    );
    assert!(source.request(0).contains("fixture-secret"));
    assert_eq!(target.count(), 0);
}

#[tokio::test]
async fn scoped_get_and_same_origin_post_redirects_preserve_method_and_have_bounded_hops() {
    for method in [Method::GET, Method::POST] {
        let server=Server::new(vec!["HTTP/1.1 307 Redirect\r\nLocation: /next\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),ok("next")]);
        let (url, next) = (server.url("/search"), server.url("/next"));
        let t = transport(
            manifest(
                &[(&url, method.clone()), (&next, method.clone())],
                loopback(),
                1,
            ),
            dns(server.port),
            Duration::from_secs(2),
            2,
            1024,
        );
        let body = (method == Method::POST).then(|| b"fixture-body".to_vec());
        assert_eq!(
            t.execute(url, method.clone(), body)
                .await
                .expect("redirect response")
                .body,
            b"next"
        );
        assert!(server.request(1).starts_with(&format!("{method} /next ")));
        assert!(!server.request(1).to_ascii_lowercase().contains("referer:"));
        if method == Method::POST {
            assert!(server.request(1).ends_with("fixture-body"));
        }
    }
}

#[tokio::test]
async fn request_and_response_budgets_are_shared_and_not_reset_per_call() {
    let server = Server::new(vec![ok("123"), ok("456")]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(2),
        2,
        5,
    );
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect("first")
            .body,
        b"123"
    );
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect_err("total bytes"),
        TransportError::ResponseLimit
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("total starts"),
        TransportError::RequestLimit
    );
    assert_eq!(server.count(), 2);
}

#[tokio::test]
async fn chunked_body_overflow_stops_without_returning_partial_success() {
    let server=Server::new(vec!["HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\n1234\r\n4\r\n5678\r\n0\r\n\r\n".into()]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(2),
        1,
        5,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("overflow"),
        TransportError::ResponseLimit
    );
}

#[tokio::test]
async fn enclosing_deadline_covers_dns_and_response_wait_and_drops_socket() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = Arc::new(Dns {
        addresses: vec![SocketAddr::from(([127, 0, 0, 1], server.port))],
        calls: AtomicUsize::new(0),
        revoke: None,
        delay: Duration::from_millis(100),
    });
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver,
        Duration::from_millis(20),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect_err("dns timeout"),
        TransportError::Deadline
    );
    assert_eq!(server.count(), 0);
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_millis(150),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("response timeout"),
        TransportError::Deadline
    );
    timeout_at(Instant::now() + Duration::from_secs(2), async {
        while server.closed.load(Ordering::Relaxed) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap_or_else(|error| {
        let (shapes, shapes_truncated) = {
            let requests = server.requests.lock().expect("request receipt");
            let shapes = requests.iter().take(4)
                .map(|bytes| (bytes.len(), bytes.windows(4).any(|s| s == b"\r\n\r\n")))
                .collect::<Vec<_>>();
            (shapes, requests.len() > 4)
        };
        let errors = server.read_errors.lock().expect("read error receipt").clone();
        let errors_unrepresented = server.read_error_count.load(Ordering::Relaxed) > errors.len();
        panic!("owned connection closes: {error}; accepted={} committed_request_shapes={shapes:?} shapes_truncated={shapes_truncated} eof_reads={} read_errors={errors:?} errors_unrepresented={errors_unrepresented}",
            server.accepted.load(Ordering::Relaxed), server.closed.load(Ordering::Relaxed));
    });
}

#[tokio::test]
async fn caller_abort_drops_owned_request_and_observed_peer_connection() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(5),
        1,
        1024,
    );
    let task = tokio::spawn(async move { t.execute(url, Method::GET, None).await });
    timeout_at(Instant::now() + Duration::from_secs(2), async {
        while server.requests.lock().expect("requests").is_empty() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("request receipt");
    task.abort();
    assert!(task.await.expect_err("aborted").is_cancelled());
    timeout_at(Instant::now() + Duration::from_secs(2), async {
        while server.closed.load(Ordering::Relaxed) == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("peer EOF receipt");
}

#[test]
fn explicit_proxy_and_invalid_bounds_are_closed_errors_not_direct_fallback() {
    let url = Url::parse("https://example.com/search").expect("url");
    let proxy =
        ToolNetworkEndpoint::new("http", "proxy.example", 8080, "CONNECT", "/").expect("proxy");
    let m = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            vec![endpoint(&url, &Method::GET)],
            Some(proxy),
            vec![],
            0,
        )
        .expect("scope"),
    );
    let authority = Arc::new(Authority {
        manifest: m.clone(),
        revoked: Arc::new(AtomicBool::new(false)),
    });
    assert!(matches!(
        SearchTransport::new(
            m,
            authority,
            Arc::new(TransportBudget::new(Duration::from_secs(2), 1, 1024).expect("budget")),
            dns(80)
        ),
        Err(TransportError::UnsupportedProxy)
    ));
    for (duration, requests, bytes) in [
        (Duration::ZERO, 1, 1),
        (Duration::from_secs(121), 1, 1),
        (Duration::from_secs(1), 17, 1),
        (Duration::from_secs(1), 1, 1_048_577),
    ] {
        assert!(TransportBudget::new(duration, requests, bytes).is_err());
    }
    assert_eq!(
        format!(
            "{:?}",
            TransportResponse {
                status: 200,
                body: b"fixture-secret".to_vec()
            }
        ),
        "TransportResponse(<redacted>)"
    );
}

#[tokio::test]
async fn ambient_proxy_environment_is_not_consulted_in_a_separate_process() {
    const CHILD: &str = "TALOS_SEARCH_TRANSPORT_CHILD";
    if let Ok(port) = std::env::var(CHILD) {
        let port = port.parse::<u16>().expect("fixture port");
        let url = Url::parse(&format!("http://search-fixture.invalid:{port}/search")).expect("url");
        let t = transport(
            manifest(&[(&url, Method::GET)], loopback(), 0),
            dns(port),
            Duration::from_secs(2),
            1,
            1024,
        );
        assert_eq!(
            t.execute(url, Method::GET, None)
                .await
                .expect("direct connection")
                .body,
            b"direct"
        );
        return;
    }
    let target = Server::new(vec![ok("direct")]);
    let proxy = Server::new(vec![ok("proxy")]);
    let mut child=std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact","search_backend::transport::tests::ambient_proxy_environment_is_not_consulted_in_a_separate_process"])
        .env(CHILD,target.port.to_string()).env("HTTP_PROXY",format!("http://127.0.0.1:{}",proxy.port)).env("http_proxy",format!("http://127.0.0.1:{}",proxy.port))
        .env("HTTPS_PROXY",format!("http://127.0.0.1:{}",proxy.port)).env("ALL_PROXY",format!("http://127.0.0.1:{}",proxy.port))
        .env("NO_PROXY","").env("no_proxy","").spawn().expect("child test");
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let result = loop {
        if let Some(status) = child.try_wait().expect("child wait") {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().expect("owned child kill");
            child.wait().expect("owned child cleanup");
            panic!("proxy fixture child timeout");
        }
        thread::sleep(Duration::from_millis(5));
    };
    assert!(result.success());
    assert_eq!(target.count(), 1);
    assert_eq!(proxy.count(), 0);
}

#[tokio::test]
async fn system_dns_and_literal_ip_paths_reach_only_explicit_local_addresses() {
    let server = Server::new(vec![ok("dns"), ok("literal")]);
    let mut url = server.url("/search");
    url.set_host(Some("localhost")).expect("localhost");
    let exceptions = vec![
        IpAddr::from([127, 0, 0, 1]),
        "::1".parse().expect("loopback"),
    ];
    let t = transport(
        manifest(&[(&url, Method::GET)], exceptions, 0),
        Arc::new(SystemSearchDns),
        Duration::from_secs(2),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect("system DNS")
            .body,
        b"dns"
    );
    let mut url = server.url("/search");
    url.set_host(Some("127.0.0.1")).expect("IP");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(2),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect("literal")
            .body,
        b"literal"
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
}

struct PortDns(AtomicUsize);
#[async_trait]
impl SearchDns for PortDns {
    async fn resolve(&self, _: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

#[tokio::test]
async fn explicitly_scoped_cross_origin_get_checks_and_pins_the_new_hop() {
    let target = Server::new(vec![ok("target")]);
    let next = target.url("/target?q=next");
    let source = Server::new(vec![redirect(302, &next)]);
    let url = source.url("/search?q=initial");
    let resolver = Arc::new(PortDns(AtomicUsize::new(0)));
    let t = transport(
        manifest(&[(&url, Method::GET), (&next, Method::GET)], loopback(), 1),
        resolver.clone(),
        Duration::from_secs(2),
        2,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect("scoped redirect")
            .body,
        b"target"
    );
    assert_eq!(resolver.0.load(Ordering::Relaxed), 2);
    assert_eq!(target.count(), 1);
    let request = target.request(0).to_ascii_lowercase();
    assert!(request.contains(&format!("host: search-fixture.invalid:{}", target.port)));
    assert!(
        !request.contains("referer:")
            && !request.contains("cookie:")
            && !request.contains("authorization:")
    );
}

#[tokio::test]
async fn redirect_hop_limit_and_post_method_changes_stop_before_next_connection() {
    for (method, limit, status, expected) in [
        (Method::GET, 0, 302, 1),
        (Method::GET, 1, 302, 2),
        (Method::POST, 1, 303, 1),
    ] {
        let response = format!(
            "HTTP/1.1 {status} Redirect\r\nLocation: /search\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let server = Server::new(vec![response.clone(), response]);
        let url = server.url("/search");
        let t = transport(
            manifest(&[(&url, method.clone())], loopback(), limit),
            dns(server.port),
            Duration::from_secs(2),
            8,
            1024,
        );
        assert_eq!(
            t.execute(url, method, None)
                .await
                .expect_err("bounded redirect"),
            TransportError::Redirect
        );
        assert_eq!(server.count(), expected);
    }
}

#[tokio::test]
async fn request_query_body_and_method_bounds_fail_before_dns_or_connect() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::GET), (&url, Method::POST)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(2),
        2,
        1024,
    );
    let mut large = url.clone();
    large.set_query(Some(&"x".repeat(4097)));
    for (u, m, b) in [
        (large, Method::GET, None),
        (url.clone(), Method::POST, Some(vec![0; 16_385])),
        (url.clone(), Method::GET, Some(vec![0])),
        (url, Method::PUT, None),
    ] {
        assert_eq!(
            t.execute(u, m, b).await.expect_err("input bound"),
            TransportError::InvalidRequest
        );
    }
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn concurrent_candidates_share_atomic_request_limit() {
    let server = Server::new(vec![ok("one")]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(2),
        1,
        1024,
    );
    let (a, b) = tokio::join!(
        t.execute(url.clone(), Method::GET, None),
        t.execute(url, Method::GET, None)
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert_eq!(a.err().or(b.err()), Some(TransportError::RequestLimit));
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn failed_response_is_not_automatically_retried_by_http_client() {
    let server = Server::new(vec![String::new()]);
    let url = server.url("/search");
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        dns(server.port),
        Duration::from_secs(2),
        8,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("closed response"),
        TransportError::Request
    );
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn body_read_timeout_remains_retryable_network() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept");
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = socket.read(&mut buffer).expect("request");
            assert_ne!(count, 0, "request headers");
            request.extend_from_slice(&buffer[..count]);
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nx")
            .expect("partial body");
        thread::sleep(Duration::from_millis(300));
    });
    let client = Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .expect("client");
    let mut response = client
        .get(format!("http://{address}/"))
        .send()
        .await
        .expect("response headers");
    let error = loop {
        match response.chunk().await {
            Ok(Some(_)) => {}
            Ok(None) => panic!("fixture body should time out"),
            Err(error) => break error,
        }
    };

    assert!(error.is_timeout());
    assert_eq!(classify_body_read_error(error), TransportError::Network);
    server.join().expect("server");
}

#[tokio::test]
async fn response_header_limits_and_unsupported_encoding_stop_before_body_use() {
    for (headers, error) in [
        (
            format!("X-Fixture: {}\r\n", "x".repeat(16_385)),
            TransportError::ResponseLimit,
        ),
        (
            (0..65)
                .map(|n| format!("X-Fixture-{n}: x\r\n"))
                .collect::<String>(),
            TransportError::ResponseLimit,
        ),
        (
            "Content-Encoding: gzip\r\n".into(),
            TransportError::ResponseEncoding,
        ),
    ] {
        let server = Server::new(vec![format!(
            "HTTP/1.1 200 OK\r\n{headers}Content-Length: 3\r\nConnection: close\r\n\r\nabc"
        )]);
        let url = server.url("/search");
        let t = transport(
            manifest(&[(&url, Method::GET)], loopback(), 0),
            dns(server.port),
            Duration::from_secs(2),
            1,
            1024,
        );
        assert_eq!(
            t.execute(url, Method::GET, None)
                .await
                .expect_err("metadata bound"),
            error
        );
    }
}

#[tokio::test(start_paused = true)]
async fn expired_shared_budget_never_resolves_or_starts_a_connection() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = dns(server.port);
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(1),
        1,
        1024,
    );
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("expired"),
        TransportError::Deadline
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

struct AdvancingDns {
    calls: AtomicUsize,
    returned: AtomicBool,
}
#[async_trait]
impl SearchDns for AdvancingDns {
    async fn resolve(&self, _: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        tokio::time::advance(Duration::from_secs(1)).await;
        self.returned.store(true, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

#[tokio::test(start_paused = true)]
async fn dns_ready_after_advancing_past_deadline_cannot_start_a_request() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = Arc::new(AdvancingDns {
        calls: AtomicUsize::new(0),
        returned: AtomicBool::new(false),
    });
    let t = transport(
        manifest(&[(&url, Method::GET)], loopback(), 0),
        resolver.clone(),
        Duration::from_secs(1),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("late ready DNS"),
        TransportError::Deadline
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
    assert!(
        resolver.returned.load(Ordering::Relaxed),
        "DNS returned Ready before fence"
    );
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn failed_dns_answer_consumes_shared_hop_budget_before_another_lookup() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let resolver = Arc::new(Dns {
        addresses: vec![],
        calls: AtomicUsize::new(0),
        revoke: None,
        delay: Duration::ZERO,
    });
    let t = transport(
        manifest(&[(&url, Method::GET)], vec![], 0),
        resolver.clone(),
        Duration::from_secs(2),
        1,
        1024,
    );
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect_err("empty DNS"),
        TransportError::Address
    );
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("attempt budget"),
        TransportError::RequestLimit
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 0);
}
