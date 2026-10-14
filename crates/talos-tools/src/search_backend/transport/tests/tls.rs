use super::*;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
};

struct Fixture {
    ports: serde_json::Value,
    child: Option<Child>,
    lines: mpsc::Receiver<String>,
    reader: Option<thread::JoinHandle<()>>,
}

impl Fixture {
    fn new(certificate: &str, mode: &str) -> Self {
        let script = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/search-tls/server.py"
        );
        let spawn = |python: &str| {
            Command::new(python)
                .args([script, "--certificate", certificate, "--proxy-mode", mode])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
        };
        let mut child = match spawn("python3") {
            Ok(child) => child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                spawn("python").expect("Python stdlib fixture prerequisite")
            }
            Err(error) => panic!("fixture spawn: {error}"),
        };
        let stdout = child.stdout.take().expect("stdout");
        let (sender, lines) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else {
                    break;
                };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let mut fixture = Self {
            ports: serde_json::Value::Null,
            child: Some(child),
            lines,
            reader: Some(reader),
        };
        let line = fixture
            .lines
            .recv_timeout(Duration::from_secs(10))
            .expect("fixture startup bound");
        fixture.ports = serde_json::from_str(&line).expect("fixture ready protocol");
        fixture
    }
    fn port(&self, key: &str) -> u16 {
        self.ports[key]
            .as_u64()
            .expect("port")
            .try_into()
            .expect("u16")
    }
    fn url(&self, host: &str, path: &str) -> Url {
        Url::parse(&format!("https://{host}:{}{path}", self.port("tls_port"))).expect("TLS URL")
    }
    fn proxy(&self) -> ToolNetworkEndpoint {
        ToolNetworkEndpoint::new(
            "http",
            "proxy-fixture.invalid",
            self.port("proxy_port"),
            "CONNECT",
            "/",
        )
        .expect("proxy identity")
    }
    fn shutdown(&mut self) -> bool {
        let Some(mut child) = self.child.take() else {
            return true;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(b"stop\n");
        }
        let limit = std::time::Instant::now() + Duration::from_secs(10);
        let success = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status.success(),
                Ok(None) if std::time::Instant::now() < limit => {
                    thread::sleep(Duration::from_millis(5))
                }
                _ => {
                    if let Err(error) = child.kill() {
                        eprintln!("fixture kill failed: {error}");
                    }
                    if let Err(error) = child.wait() {
                        eprintln!("fixture wait failed: {error}");
                    }
                    break false;
                }
            }
        };
        if let Some(reader) = self.reader.take() {
            reader.join().expect("owned stdout reader");
        }
        success
    }
    fn finish(mut self) -> serde_json::Value {
        assert!(self.shutdown(), "fixture termination");
        while let Ok(line) = self.lines.recv_timeout(Duration::from_secs(1)) {
            let value: serde_json::Value = serde_json::from_str(&line).expect("receipt protocol");
            if value.get("requests").is_some() {
                return value;
            }
        }
        panic!("terminal witness missing")
    }
    async fn event(&self, key: &str) {
        timeout_at(Instant::now() + Duration::from_secs(4), async {
            loop {
                while let Ok(line) = self.lines.try_recv() {
                    let value: serde_json::Value =
                        serde_json::from_str(&line).expect("event protocol");
                    if value[key] == true {
                        return;
                    }
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("fixture event deadline");
    }
    fn allow(&mut self) {
        self.child
            .as_mut()
            .expect("child")
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(b"allow\n")
            .expect("release gate");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn scoped(
    urls: &[(&Url, Method)],
    proxy: Option<ToolNetworkEndpoint>,
    exceptions: Vec<IpAddr>,
    redirects: u8,
) -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            urls.iter().map(|(u, m)| endpoint(u, m)).collect(),
            proxy,
            exceptions,
            redirects,
        )
        .expect("scope"),
    )
}
fn trusted(mut transport: SearchTransport) -> SearchTransport {
    transport.fixture_roots.push(
        reqwest::Certificate::from_pem(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/search-tls/root.pem"
        )))
        .expect("public test CA"),
    );
    transport
}
fn count(receipt: &serde_json::Value, key: &str) -> u64 {
    receipt[key].as_u64().expect("count")
}
fn requests(receipt: &serde_json::Value) -> &Vec<serde_json::Value> {
    receipt["requests"].as_array().expect("requests")
}

#[tokio::test]
async fn direct_tls_preserves_original_sni_host_and_pinned_dial_address() {
    let fixture = Fixture::new("leaf.pem", "tunnel");
    let url = fixture.url("search-fixture.invalid", "/search?q=fixture-query");
    let resolver = dns(fixture.port("tls_port"));
    let t = trusted(transport(
        scoped(&[(&url, Method::GET)], None, loopback(), 0),
        resolver.clone(),
        Duration::from_secs(4),
        1,
        1024,
    ));
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect("verified TLS")
            .body,
        b"tls"
    );
    let receipt = fixture.finish();
    let sni = receipt["sni"].as_array().expect("SNI");
    assert!(!sni.is_empty() && sni.iter().all(|name| name == "search-fixture.invalid"));
    assert_eq!(count(&receipt, "tls_connections"), 1);
    assert_eq!(requests(&receipt).len(), 1);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
    assert!(
        requests(&receipt)[0]
            .as_str()
            .expect("request")
            .to_ascii_lowercase()
            .contains("host: search-fixture.invalid:")
    );
    assert_eq!(count(&receipt, "proxy_connections"), 0);
}

#[tokio::test]
async fn trusted_wrong_san_untrusted_ca_and_expired_leaf_send_no_application_input() {
    for (cert, host, trust) in [
        ("leaf.pem", "wrong-fixture.invalid", true),
        ("leaf.pem", "search-fixture.invalid", false),
        ("expired.pem", "search-fixture.invalid", true),
    ] {
        let fixture = Fixture::new(cert, "tunnel");
        let url = fixture.url(host, "/search?q=fixture-secret");
        let t = transport(
            scoped(&[(&url, Method::GET)], None, loopback(), 0),
            dns(fixture.port("tls_port")),
            Duration::from_secs(4),
            1,
            1024,
        );
        let t = if trust { trusted(t) } else { t };
        assert_eq!(
            t.execute(url, Method::GET, None)
                .await
                .expect_err("certificate rejection"),
            TransportError::Request
        );
        let receipt = fixture.finish();
        assert_eq!(count(&receipt, "tls_connections"), 1);
        assert!(requests(&receipt).is_empty());
        assert_eq!(count(&receipt, "proxy_connections"), 0);
    }
}

#[tokio::test]
async fn connect_target_ip_san_is_checked_even_with_a_trusted_fixture_ca() {
    let fixture = Fixture::new("host-only.pem", "tunnel");
    let url = fixture.url("127.0.0.1", "/search?q=fixture-secret");
    let t = trusted(transport(
        scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0),
        dns(fixture.port("proxy_port")),
        Duration::from_secs(4),
        2,
        1024,
    ));
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("IP SAN mismatch"),
        TransportError::Request
    );
    let receipt = fixture.finish();
    assert_eq!(count(&receipt, "proxy_connections"), 1);
    assert_eq!(count(&receipt, "tls_connections"), 1);
    assert!(requests(&receipt).is_empty());
}

#[tokio::test]
async fn https_downgrade_is_rejected_before_plaintext_target_receives_input() {
    let fixture = Fixture::new("leaf.pem", "tunnel");
    let url = fixture.url("search-fixture.invalid", "/downgrade");
    let next = Url::parse(&format!(
        "http://127.0.0.1:{}/target",
        fixture.port("plain_port")
    ))
    .expect("plain URL");
    let t = trusted(transport(
        scoped(
            &[(&url, Method::GET), (&next, Method::GET)],
            None,
            loopback(),
            1,
        ),
        dns(fixture.port("tls_port")),
        Duration::from_secs(4),
        2,
        1024,
    ));
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("downgrade"),
        TransportError::Redirect
    );
    let receipt = fixture.finish();
    assert_eq!(requests(&receipt).len(), 1);
    assert_eq!(count(&receipt, "plain_connections"), 0);
}

#[tokio::test]
async fn explicit_connect_uses_literal_target_ip_san_and_pinned_proxy_peer() {
    let fixture = Fixture::new("leaf.pem", "tunnel");
    let url = fixture.url("127.0.0.1", "/search?q=fixture-query");
    let resolver = dns(fixture.port("proxy_port"));
    let t = trusted(transport(
        scoped(
            &[(&url, Method::POST)],
            Some(fixture.proxy()),
            loopback(),
            0,
        ),
        resolver.clone(),
        Duration::from_secs(4),
        2,
        1024,
    ));
    let result = t
        .execute(
            url,
            Method::POST,
            Some(b"{\"key\":\"fixture-secret\"}".to_vec()),
        )
        .await;
    let port = fixture.port("tls_port");
    let receipt = fixture.finish();
    assert_eq!(result.expect("CONNECT TLS").body, b"tls");
    assert_eq!(count(&receipt, "proxy_connections"), 1);
    assert_eq!(requests(&receipt).len(), 1);
    assert!(
        receipt["sni"]
            .as_array()
            .expect("SNI")
            .iter()
            .all(serde_json::Value::is_null)
    ); // IP identity has no hostname SNI.
    let connect = receipt["connect"][0].as_str().expect("CONNECT");
    assert!(connect.starts_with(&format!("CONNECT 127.0.0.1:{port} HTTP/1.1")));
    assert!(!connect.contains("fixture-query") && !connect.contains("fixture-secret"));
    assert!(
        requests(&receipt)[0]
            .as_str()
            .expect("TLS request")
            .contains("fixture-secret")
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn real_once_scope_includes_proxy_and_clear_prevents_another_compound_hop() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
    use talos_permission::{
        GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
        PermissionRequest, PermissionSessionState,
    };
    let fixture = Fixture::new("leaf.pem", "tunnel");
    let url = fixture.url("127.0.0.1", "/search");
    let m = scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0);
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let facets = ["127.0.0.1", "proxy-fixture.invalid"].map(|host| {
        ToolPermissionFacet::with_resource(ToolNature::Network, host, ToolResourceKind::Domain)
    });
    let input = serde_json::json!({"query":"fixture"});
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request =
        PermissionRequest::native("web_search", &facets, &input).with_network_manifest(&m);
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("whole scope proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approve");
    let authority = state
        .admit_network(pending, &request, &context)
        .expect("whole-scope live authority");
    let t = trusted(
        SearchTransport::new(
            m,
            authority,
            Arc::new(TransportBudget::new(Duration::from_secs(4), 4, 1024).expect("budget")),
            dns(fixture.port("proxy_port")),
        )
        .expect("transport"),
    );
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect("first")
            .status,
        200
    );
    state.clear().expect("clear");
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("revoked"),
        TransportError::Unauthorized
    );
    let receipt = fixture.finish();
    assert_eq!(count(&receipt, "proxy_connections"), 1);
    assert_eq!(requests(&receipt).len(), 1);
}

struct Deny {
    manifest: ToolNetworkManifest,
    endpoint: ToolNetworkEndpoint,
}
impl ToolNetworkAuthority for Deny {
    fn check(
        &self,
        id: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if id != self.manifest.invocation_id() || endpoint == &self.endpoint {
            Err(ToolNetworkAuthorizationError::Invalidated)
        } else {
            Ok(())
        }
    }
}

#[tokio::test]
async fn proxy_or_target_denial_blocks_dns_connect_and_application_request() {
    for deny_proxy in [true, false] {
        let fixture = Fixture::new("leaf.pem", "tunnel");
        let url = fixture.url("127.0.0.1", "/search");
        let proxy = fixture.proxy();
        let m = scoped(&[(&url, Method::GET)], Some(proxy.clone()), loopback(), 0);
        let authority = Arc::new(Deny {
            manifest: m.clone(),
            endpoint: if deny_proxy {
                proxy
            } else {
                endpoint(&url, &Method::GET)
            },
        });
        let resolver = dns(fixture.port("proxy_port"));
        let t = SearchTransport::new(
            m,
            authority,
            Arc::new(TransportBudget::new(Duration::from_secs(4), 2, 1024).expect("budget")),
            resolver.clone(),
        )
        .expect("transport");
        assert_eq!(
            t.execute(url, Method::GET, None).await.expect_err("denied"),
            TransportError::Unauthorized
        );
        let receipt = fixture.finish();
        assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
        assert_eq!(count(&receipt, "proxy_connections"), 0);
        assert!(requests(&receipt).is_empty());
    }
}

#[tokio::test]
async fn proxy_dns_revocation_mixed_answers_and_target_address_guard_fail_before_connect() {
    for case in 0..3 {
        let fixture = Fixture::new("leaf.pem", "tunnel");
        let url = fixture.url(
            if case == 2 {
                "198.51.100.1"
            } else {
                "127.0.0.1"
            },
            "/search",
        );
        let m = scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0);
        let revoked = Arc::new(AtomicBool::new(false));
        let mut addresses = vec![SocketAddr::from((
            [127, 0, 0, 1],
            fixture.port("proxy_port"),
        ))];
        if case == 1 {
            addresses.push(SocketAddr::from((
                [10, 0, 0, 1],
                fixture.port("proxy_port"),
            )));
        }
        let resolver = Arc::new(Dns {
            addresses,
            calls: AtomicUsize::new(0),
            revoke: (case == 0).then(|| revoked.clone()),
            delay: Duration::ZERO,
        });
        let authority = Arc::new(Authority {
            manifest: m.clone(),
            revoked,
        });
        let t = SearchTransport::new(
            m,
            authority,
            Arc::new(TransportBudget::new(Duration::from_secs(4), 2, 1024).expect("budget")),
            resolver.clone(),
        )
        .expect("transport");
        assert_eq!(
            t.execute(url, Method::GET, None)
                .await
                .expect_err("address/revoke"),
            if case == 0 {
                TransportError::Unauthorized
            } else {
                TransportError::Address
            }
        );
        let receipt = fixture.finish();
        assert_eq!(count(&receipt, "proxy_connections"), 0);
        assert_eq!(count(&receipt, "tls_connections"), 0);
        assert_eq!(
            resolver.calls.load(Ordering::Relaxed),
            usize::from(case != 2)
        );
    }
}

#[tokio::test]
async fn unsupported_hostname_http_or_https_proxy_has_no_direct_fallback() {
    for case in 0..6 {
        let fixture = Fixture::new("leaf.pem", "tunnel");
        let mut url = fixture.url(
            if case == 5 {
                "[::1]"
            } else if case == 0 {
                "search-fixture.invalid"
            } else {
                "127.0.0.1"
            },
            "/search",
        );
        if case == 1 {
            url.set_scheme("http").expect("scheme");
        }
        let proxy = if matches!(case, 2..=4) {
            ToolNetworkEndpoint::new(
                if case == 2 { "https" } else { "http" },
                "proxy-fixture.invalid",
                fixture.port("proxy_port"),
                if case == 4 { "GET" } else { "CONNECT" },
                if case == 3 { "/unsupported" } else { "/" },
            )
            .expect("proxy")
        } else {
            fixture.proxy()
        };
        let m = scoped(&[(&url, Method::GET)], Some(proxy), loopback(), 0);
        let authority = Arc::new(Authority {
            manifest: m.clone(),
            revoked: Arc::new(AtomicBool::new(false)),
        });
        assert!(matches!(
            SearchTransport::new(
                m,
                authority,
                Arc::new(TransportBudget::new(Duration::from_secs(4), 2, 1024).expect("budget")),
                dns(fixture.port("proxy_port"))
            ),
            Err(TransportError::UnsupportedProxy)
        ));
        let receipt = fixture.finish();
        assert_eq!(count(&receipt, "proxy_connections"), 0);
        assert_eq!(count(&receipt, "tls_connections"), 0);
    }
}

#[tokio::test]
async fn failed_or_oversized_connect_response_never_reaches_tls_target() {
    for mode in ["deny", "oversize"] {
        let fixture = Fixture::new("leaf.pem", mode);
        let url = fixture.url("127.0.0.1", "/search");
        let t = trusted(transport(
            scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0),
            dns(fixture.port("proxy_port")),
            Duration::from_secs(4),
            2,
            1024,
        ));
        assert_eq!(
            t.execute(url, Method::GET, None)
                .await
                .expect_err("CONNECT failure"),
            TransportError::Request
        );
        let receipt = fixture.finish();
        assert_eq!(count(&receipt, "proxy_connections"), 1);
        assert_eq!(count(&receipt, "tls_connections"), 0);
    }
}

#[tokio::test]
async fn proxy_budget_counts_compound_hop_and_deadline_covers_connect_wait() {
    let fixture = Fixture::new("leaf.pem", "hold");
    let url = fixture.url("127.0.0.1", "/search");
    let resolver = dns(fixture.port("proxy_port"));
    let t = trusted(transport(
        scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0),
        resolver.clone(),
        Duration::from_secs(4),
        1,
        1024,
    ));
    assert_eq!(
        t.execute(url.clone(), Method::GET, None)
            .await
            .expect_err("two reservations"),
        TransportError::RequestLimit
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    let t = trusted(transport(
        scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0),
        resolver,
        Duration::from_millis(300),
        2,
        1024,
    ));
    assert_eq!(
        t.execute(url, Method::GET, None)
            .await
            .expect_err("CONNECT deadline"),
        TransportError::Deadline
    );
    let receipt = fixture.finish();
    assert_eq!(count(&receipt, "proxy_connections"), 1);
    assert_eq!(count(&receipt, "tls_connections"), 0);
}

#[tokio::test]
async fn compound_start_revocation_allows_started_work_but_blocks_the_next_redirect_hop() {
    for redirect in [false, true] {
        let mut fixture = Fixture::new("leaf.pem", "gate");
        let url = fixture.url("127.0.0.1", if redirect { "/redirect" } else { "/search" });
        let next = fixture.url("127.0.0.1", "/search");
        let m = scoped(
            &[(&url, Method::GET), (&next, Method::GET)],
            Some(fixture.proxy()),
            loopback(),
            1,
        );
        let revoked = Arc::new(AtomicBool::new(false));
        let authority = Arc::new(Authority {
            manifest: m.clone(),
            revoked: revoked.clone(),
        });
        let t = trusted(
            SearchTransport::new(
                m,
                authority,
                Arc::new(TransportBudget::new(Duration::from_secs(5), 4, 1024).expect("budget")),
                dns(fixture.port("proxy_port")),
            )
            .expect("transport"),
        );
        let task = tokio::spawn(async move { t.execute(url, Method::GET, None).await });
        fixture.event("connect_seen").await;
        revoked.store(true, Ordering::Relaxed);
        fixture.allow();
        let result = task.await.expect("owned operation");
        if redirect {
            assert_eq!(result.expect_err("later hop"), TransportError::Unauthorized);
        } else {
            assert_eq!(result.expect("already started compound hop").body, b"tls");
        }
        let receipt = fixture.finish();
        assert_eq!(count(&receipt, "proxy_connections"), 1);
        assert_eq!(requests(&receipt).len(), 1);
    }
}

#[tokio::test]
async fn caller_abort_closes_waiting_connect_before_any_tls_application_start() {
    let fixture = Fixture::new("leaf.pem", "hold");
    let url = fixture.url("127.0.0.1", "/search");
    let t = trusted(transport(
        scoped(&[(&url, Method::GET)], Some(fixture.proxy()), loopback(), 0),
        dns(fixture.port("proxy_port")),
        Duration::from_secs(5),
        2,
        1024,
    ));
    let task = tokio::spawn(async move { t.execute(url, Method::GET, None).await });
    fixture.event("connect_seen").await;
    task.abort();
    assert!(task.await.expect_err("aborted").is_cancelled());
    fixture.event("proxy_closed").await;
    let receipt = fixture.finish();
    assert_eq!(count(&receipt, "proxy_connections"), 1);
    assert_eq!(count(&receipt, "tls_connections"), 0);
    assert!(requests(&receipt).is_empty());
}
