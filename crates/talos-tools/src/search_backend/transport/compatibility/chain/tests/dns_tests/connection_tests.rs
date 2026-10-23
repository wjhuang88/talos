use super::*;

fn reserve(mut ports: std::ops::Range<u16>) -> std::net::TcpListener {
    // Keep refused destinations separate from each other and from the default
    // ephemeral ports used by parallel loopback servers and outgoing connects.
    // The listener is released immediately before the real connection attempt.
    ports
        .find_map(|port| std::net::TcpListener::bind(("127.0.0.1", port)).ok())
        .expect("reserve unused non-ephemeral refused port")
}

#[tokio::test]
async fn refused_connection_advances_to_next_admitted_http_source_without_replay() {
    let server = Server::new(vec![valid(ORDER[1])]);
    let server_port = server
        .url("/")
        .port_or_known_default()
        .expect("server port");
    let reserved = reserve(16_384..16_896);
    let port = reserved.local_addr().expect("address").port();
    let (chain, transport, resolver, budget) = setup_with_ports(
        &server,
        vec![],
        4,
        AfterFailure::Continue,
        Some([port, server_port, server_port, server_port]),
    );
    let ua = chain.sources[1]
        .request
        .headers
        .expect("frozen headers")
        .user_agent();
    drop(reserved);
    let rows = chain
        .execute(&transport)
        .await
        .expect("next admitted source")
        .into_results();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].url.starts_with("https://duckduckgo.com/"));
    assert_eq!(hosts(&resolver), ["source-0.invalid", "source-1.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(server.count(), 1);
    let wire = server.request(0);
    assert!(wire.starts_with("GET /?q="));
    assert!(wire.contains("%E4%B8%AD+%2B%26%7E+"));
    assert!(
        wire.lines()
            .any(|line| line.eq_ignore_ascii_case(&format!("user-agent: {ua}")))
    );
}

// Policy-only injection at the existing resolver error seam. Real send-boundary
// classification and next-source execution remain separate socket tests.
struct RefusingDns(Arc<Resolver>);

#[async_trait]
impl SearchDns for RefusingDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.0.resolve(host, port).await?;
        Err(TransportError::ConnectionRefused)
    }
}

#[tokio::test]
async fn injected_refusal_errors_exhaust_four_frozen_sources_without_reset() {
    let unused = Server::new(vec![]);
    let (chain, mut transport, resolver, budget) =
        setup(&unused, vec![], 4, AfterFailure::Continue);
    transport.dns = Arc::new(RefusingDns(resolver.clone()));
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("finite exhaustion"),
        CompatibilityChainError::Exhausted(
            [CompatibilityError::Transport(TransportError::ConnectionRefused); 4]
        )
    );
    assert_eq!(
        hosts(&resolver),
        [
            "source-0.invalid",
            "source-1.invalid",
            "source-2.invalid",
            "source-3.invalid"
        ]
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 4);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(unused.count(), 0);
}

#[tokio::test]
async fn refused_connection_consumes_budget_before_the_next_source_can_resolve() {
    let server = Server::new(vec![valid(ORDER[1])]);
    let server_port = server
        .url("/")
        .port_or_known_default()
        .expect("server port");
    let reserved = reserve(16_896..17_408);
    let port = reserved.local_addr().expect("address").port();
    let (chain, transport, resolver, budget) = setup_with_ports(
        &server,
        vec![],
        1,
        AfterFailure::Continue,
        Some([port, server_port, server_port, server_port]),
    );
    drop(reserved);
    assert_eq!(
        chain.execute(&transport).await.expect_err("spent budget"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[1],
            error: CompatibilityError::Transport(TransportError::RequestLimit),
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn truncated_response_body_remains_terminal_request_failure() {
    let server = Server::new(vec![
        "HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nx".into(),
        valid(ORDER[1]),
    ]);
    let (chain, transport, resolver, budget) = setup(&server, vec![], 4, AfterFailure::Continue);
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("body read is not refusal"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[0],
            error: CompatibilityError::Transport(TransportError::Request),
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
