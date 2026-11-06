use super::*;

// Cooperating native refusal checks share a low-port lease. Do not use the
// ephemeral range used by parallel loopback servers/outgoing connections.
// This is not an OS reservation after listener drop: arbitrary external binders
// remain outside the fixture's control, and unexpected native errors must fail.
static REFUSAL_PORT_LEASE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn reserve_refusal_port() -> TcpListener {
    (16_384..16_896)
        .find_map(|port| TcpListener::bind(("127.0.0.1", port)).ok())
        .expect("reserve unused non-ephemeral native refusal port")
}

#[tokio::test]
async fn actual_http_and_https_connection_refusal_has_only_the_closed_error() {
    let _lease = REFUSAL_PORT_LEASE.lock().await;
    for scheme in ["http", "https"] {
        let unused = Server::new(vec![]);
        let reserved = reserve_refusal_port();
        let port = reserved.local_addr().expect("address").port();
        let mut url = unused.url("/private-query?q=synthetic");
        url.set_scheme(scheme).expect("scheme");
        url.set_port(Some(port)).expect("port");
        let resolver = dns(port);
        let transport = transport(
            manifest(&[(&url, Method::GET)], loopback(), 0),
            resolver.clone(),
            Duration::from_secs(3),
            1,
            1024,
        );
        // Another process could reuse this port. Any unexpected actual error fails this
        // assertion; it is never accepted as an equivalent outcome or silently skipped.
        drop(reserved);
        let error = transport
            .execute(url, Method::GET, None)
            .await
            .expect_err("refused socket");
        assert_eq!(error, TransportError::ConnectionRefused);
        assert_eq!(error.to_string(), "search_transport_connection_refused");
        assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
        assert_eq!(transport.budget.attempts.load(Ordering::Relaxed), 1);
        assert_eq!(transport.budget.bytes.load(Ordering::Relaxed), 0);
        assert_eq!(unused.count(), 0);
    }
}

#[tokio::test]
async fn released_listener_port_can_be_rebound_and_accept_instead_of_refuse() {
    let _lease = REFUSAL_PORT_LEASE.lock().await;
    // Deterministic counterexample to the old policy-fixture assumption: bind
    // the competitor after release, before connecting. No timing lottery/retry.
    let reserved = reserve_refusal_port();
    let address = reserved.local_addr().expect("address");
    drop(reserved);
    let replacement = TcpListener::bind(address).expect("competitor rebinds released port");
    let connected = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .expect("old supposedly refused destination accepts a connection");
    assert_eq!(connected.peer_addr().expect("peer"), address);
    drop(connected);
    drop(replacement);
}
