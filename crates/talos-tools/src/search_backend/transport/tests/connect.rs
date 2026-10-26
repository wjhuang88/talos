use super::*;

#[tokio::test]
async fn actual_http_and_https_connection_refusal_has_only_the_closed_error() {
    for scheme in ["http", "https"] {
        let unused = Server::new(vec![]);
        let reserved = TcpListener::bind("127.0.0.1:0").expect("reserve port");
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
