use super::*;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};
use talos_core::tool::{ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkScope};

fn endpoint(host: &str, path: &str) -> ToolNetworkEndpoint {
    ToolNetworkEndpoint::new("https", host, 443, "GET", path).expect("endpoint")
}

#[test]
fn snapshot_is_sanitized_and_freezes_no_proxy_per_destination() {
    let endpoints = vec![
        endpoint("lite.duckduckgo.com", "/lite/"),
        endpoint("cn.bing.com", "/search"),
    ];
    let proxy = TransportProxy::from_snapshot(
        "http://fixture-user:fixture-secret@proxy.example:8080",
        "duckduckgo.com",
        &endpoints,
    )
    .expect("snapshot")
    .expect("proxy retained for Bing");

    assert_eq!(proxy.endpoint().scheme(), "http");
    assert_eq!(proxy.endpoint().host(), "proxy.example");
    assert_eq!(proxy.endpoint().port(), 8080);
    assert!(!format!("{proxy:?}").contains("fixture-secret"));
    assert!(!proxy.applies_to(&Url::parse("https://lite.duckduckgo.com/lite/").expect("url")));
    assert!(proxy.applies_to(&Url::parse("https://cn.bing.com/search").expect("url")));

    assert!(
        TransportProxy::from_snapshot("http://proxy.example:8080", "*", &endpoints)
            .expect("snapshot")
            .is_none()
    );
    assert!(matches!(
        TransportProxy::from_snapshot("socks5://proxy.example:1080", "", &endpoints),
        Err(TransportError::UnsupportedProxy)
    ));
}

struct Authority(ToolNetworkManifest);

impl ToolNetworkAuthority for Authority {
    fn check(
        &self,
        invocation_id: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if invocation_id == self.0.invocation_id()
            && (self.0.scope().endpoints().contains(endpoint)
                || self.0.scope().proxy() == Some(endpoint))
        {
            Ok(())
        } else {
            Err(ToolNetworkAuthorizationError::Mismatch)
        }
    }
}

struct LoopbackDns {
    calls: AtomicUsize,
}

#[async_trait]
impl SearchDns for LoopbackDns {
    async fn resolve(&self, _host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

struct Fixture {
    ports: serde_json::Value,
    child: Option<Child>,
    lines: mpsc::Receiver<String>,
    reader: Option<thread::JoinHandle<()>>,
}

impl Fixture {
    fn new() -> Self {
        let script = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/search-tls/server.py"
        );
        let spawn = |python: &str| {
            Command::new(python)
                .args([
                    script,
                    "--certificate",
                    "leaf.pem",
                    "--proxy-mode",
                    "tunnel",
                ])
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
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let ports = serde_json::from_str(
            &lines
                .recv_timeout(Duration::from_secs(10))
                .expect("fixture startup bound"),
        )
        .expect("fixture ready protocol");
        Self {
            ports,
            child: Some(child),
            lines,
            reader: Some(reader),
        }
    }

    fn port(&self, key: &str) -> u16 {
        self.ports[key]
            .as_u64()
            .expect("port")
            .try_into()
            .expect("u16")
    }

    fn finish(mut self) -> serde_json::Value {
        let mut child = self.child.take().expect("child");
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b"stop\n").expect("stop fixture");
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    assert!(status.success(), "fixture status");
                    break;
                }
                Ok(None) if std::time::Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(5));
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("fixture termination");
                }
            }
        }
        if let Some(reader) = self.reader.take() {
            reader.join().expect("fixture reader");
        }
        while let Ok(line) = self.lines.recv_timeout(Duration::from_secs(1)) {
            let value: serde_json::Value = serde_json::from_str(&line).expect("receipt protocol");
            if value.get("requests").is_some() {
                return value;
            }
        }
        panic!("terminal witness missing")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"stop\n");
            }
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[tokio::test]
async fn frozen_hostname_proxy_preserves_working_connect_path_without_direct_target_dns() {
    let fixture = Fixture::new();
    let tls_port = fixture.port("tls_port");
    let proxy_port = fixture.port("proxy_port");
    let url = Url::parse(&format!(
        "https://search-fixture.invalid:{tls_port}/search?q=fixture-query"
    ))
    .expect("target url");
    let target = request_endpoint(&url, &Method::GET).expect("target endpoint");
    let proxy = TransportProxy::from_snapshot(
        &format!("http://proxy-fixture.invalid:{proxy_port}"),
        "",
        std::slice::from_ref(&target),
    )
    .expect("proxy snapshot")
    .expect("proxy applies");
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            vec![target],
            Some(proxy.endpoint().clone()),
            vec![],
            0,
        )
        .expect("scope"),
    );
    let dns = Arc::new(LoopbackDns {
        calls: AtomicUsize::new(0),
    });
    let authority = Arc::new(Authority(manifest.clone()));
    let mut transport = SearchTransport::new_with_proxy(
        manifest,
        authority,
        Arc::new(TransportBudget::new(Duration::from_secs(4), 2, 1024).expect("budget")),
        dns.clone(),
        Some(proxy),
    )
    .expect("controlled proxy transport");
    transport.fixture_roots.push(
        reqwest::Certificate::from_pem(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/search-tls/root.pem"
        )))
        .expect("fixture root"),
    );

    assert_eq!(
        transport
            .execute(url, Method::GET, None)
            .await
            .expect("proxied search")
            .body,
        b"tls"
    );
    let receipt = fixture.finish();
    let connect = receipt["connect"][0].as_str().expect("CONNECT receipt");
    assert!(connect.starts_with(&format!(
        "CONNECT search-fixture.invalid:{tls_port} HTTP/1.1"
    )));
    assert!(!connect.contains("fixture-query"));
    assert_eq!(receipt["proxy_connections"].as_u64(), Some(1));
    assert_eq!(receipt["tls_connections"].as_u64(), Some(1));
    assert_eq!(dns.calls.load(Ordering::Relaxed), 1);
    assert!(
        receipt["sni"]
            .as_array()
            .expect("SNI")
            .iter()
            .all(|value| value == "search-fixture.invalid")
    );
    assert!(
        receipt["requests"][0]
            .as_str()
            .expect("application request")
            .contains("fixture-query")
    );
}
