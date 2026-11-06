use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn injected_refusal_retries_before_real_next_source_despite_occupied_destination() {
    // Each contender owns a real listener at source zero; execute independent
    // admitted chains concurrently rather than repeatedly hoping for a free port.
    let mut contenders = tokio::task::JoinSet::new();
    for _ in 0..8 {
        contenders.spawn(assert_refusal_policy_with_occupied_destination());
    }
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(result) = contenders.join_next().await {
            result.expect("concurrent refusal policy assertion");
        }
    })
    .await
    .expect("bounded concurrent fixture execution");
}

async fn assert_refusal_policy_with_occupied_destination() {
    let server = Server::new(vec![valid(ORDER[1])]);
    let server_port = server
        .url("/")
        .port_or_known_default()
        .expect("server port");
    // Deliberately occupy the formerly refused destination. Removing the injected
    // error would send the query to this wrong source and violate the assertions.
    let occupied = Server::new(vec![valid(ORDER[0])]);
    let port = occupied
        .url("/")
        .port_or_known_default()
        .expect("occupied port");
    let (chain, mut transport, resolver, budget) = setup_with_ports(
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
    transport.dns = Arc::new(SelectiveRefusingDns(resolver.clone()));
    let rows = chain
        .execute(&transport)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "next admitted source: {error:?}; resolved={:?}; attempts={}; refused_port={port}; server_port={server_port}",
                hosts(&resolver),
                budget.attempts.load(Ordering::Relaxed),
            )
        })
        .into_results();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].url.starts_with("https://duckduckgo.com/"));
    assert_eq!(
        hosts(&resolver),
        ["source-0.invalid", "source-0.invalid", "source-1.invalid"]
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 3);
    assert_eq!(server.count(), 1);
    assert_eq!(occupied.count(), 0);
    assert_eq!(occupied.accepted.load(Ordering::Relaxed), 0);
    let wire = server.request(0);
    assert!(wire.starts_with("GET /?q="));
    assert!(wire.contains("%E4%B8%AD+%2B%26%7E+"));
    assert!(
        wire.lines()
            .any(|line| line.eq_ignore_ascii_case(&format!("user-agent: {ua}")))
    );
}

// Preserve resolver observations, but make only source zero unavailable. Later
// sources still execute real HTTP through the ordinary transport and authority.
struct SelectiveRefusingDns(Arc<Resolver>);

#[async_trait]
impl SearchDns for SelectiveRefusingDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        let addresses = self.0.resolve(host, port).await?;
        if host == "source-0.invalid" {
            Err(TransportError::ConnectionRefused)
        } else {
            Ok(addresses)
        }
    }
}

// Policy-only injection at the existing resolver error seam. Real send-boundary
// classification remains in transport::tests::connect; next-source execution
// above still uses real HTTP.
struct RefusingDns(Arc<Resolver>);

#[async_trait]
impl SearchDns for RefusingDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.0.resolve(host, port).await?;
        Err(TransportError::ConnectionRefused)
    }
}

#[tokio::test]
async fn injected_refusal_errors_exhaust_legacy_seven_attempts_without_reset() {
    let unused = Server::new(vec![]);
    let (chain, mut transport, resolver, budget) =
        setup(&unused, vec![], 7, AfterFailure::Continue);
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
            "source-0.invalid",
            "source-1.invalid",
            "source-2.invalid",
            "source-2.invalid",
            "source-3.invalid",
            "source-3.invalid"
        ]
    );
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 7);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(unused.count(), 0);
}

#[tokio::test]
async fn injected_refusal_consumes_budget_before_retry_despite_occupied_destination() {
    let server = Server::new(vec![valid(ORDER[1])]);
    let server_port = server
        .url("/")
        .port_or_known_default()
        .expect("server port");
    let occupied = Server::new(vec![valid(ORDER[0])]);
    let port = occupied
        .url("/")
        .port_or_known_default()
        .expect("occupied port");
    let (chain, mut transport, resolver, budget) = setup_with_ports(
        &server,
        vec![],
        1,
        AfterFailure::Continue,
        Some([port, server_port, server_port, server_port]),
    );
    transport.dns = Arc::new(SelectiveRefusingDns(resolver.clone()));
    assert_eq!(
        chain.execute(&transport).await.expect_err("spent budget"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[0],
            error: CompatibilityError::Transport(TransportError::RequestLimit),
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 0);
    assert_eq!(occupied.count(), 0);
    assert_eq!(occupied.accepted.load(Ordering::Relaxed), 0);
}

struct NetworkOnceDns {
    inner: Arc<Resolver>,
    failed: std::sync::atomic::AtomicBool,
}

#[async_trait]
impl SearchDns for NetworkOnceDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        let addresses = self.inner.resolve(host, port).await?;
        if !self.failed.swap(true, std::sync::atomic::Ordering::Relaxed) {
            return Err(TransportError::Network);
        }
        Ok(addresses)
    }
}

#[tokio::test]
async fn injected_network_failure_retries_same_source_before_later_disclosure() {
    let server = Server::new(vec![valid(ORDER[0])]);
    let (chain, mut transport, resolver, budget) =
        setup(&server, vec![], 4, AfterFailure::Continue);
    transport.dns = Arc::new(NetworkOnceDns {
        inner: resolver.clone(),
        failed: std::sync::atomic::AtomicBool::new(false),
    });
    let rows = chain
        .execute(&transport)
        .await
        .expect("typed network failure retries the admitted source")
        .into_results();
    assert_eq!(rows.len(), 1);
    assert_eq!(hosts(&resolver), ["source-0.invalid", "source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn truncated_response_body_is_terminal_and_never_retried_or_disclosed_later() {
    let server = Server::new(vec![
        "HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nx".into(),
        valid(ORDER[0]),
    ]);
    let (chain, transport, resolver, budget) = setup(&server, vec![], 4, AfterFailure::Continue);
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("malformed body framing must be terminal"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[0],
            error: CompatibilityError::Transport(TransportError::Request),
        }
    );
    assert_eq!(hosts(&resolver), ["source-0.invalid"]);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(server.count(), 1);
}
