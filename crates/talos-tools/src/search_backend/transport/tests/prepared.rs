use super::*;

struct CountAuthority {
    calls: AtomicUsize,
}
impl ToolNetworkAuthority for CountAuthority {
    fn check(
        &self,
        _: uuid::Uuid,
        _: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[tokio::test]
async fn local_preparation_and_cross_invocation_rejection_have_no_execution_effects() {
    let server = Server::new(vec![]);
    let url = server.url("/search?q=toy-secret");
    let first = manifest(&[(&url, Method::POST)], loopback(), 0);
    let second = ToolNetworkManifest::new(first.scope().clone());
    assert_eq!(first.scope(), second.scope());
    assert_ne!(first.invocation_id(), second.invocation_id());
    let resolver = dns(server.port);
    let authority = Arc::new(CountAuthority {
        calls: AtomicUsize::new(0),
    });
    let budget = Arc::new(TransportBudget::new(Duration::from_secs(2), 1, 1024).expect("budget"));
    let t = SearchTransport::new(second, authority.clone(), budget.clone(), resolver.clone())
        .expect("transport");
    let request = PreparedTransportRequest::form(&first, url, form_fields(&[("q", "toy-secret")]))
        .expect("local preparation");
    assert_eq!(
        format!("{request:?}"),
        "PreparedTransportRequest(<redacted>)"
    );
    assert_eq!(authority.calls.load(Ordering::Relaxed), 0);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(
        t.execute_prepared(request)
            .await
            .expect_err("wrong invocation"),
        TransportError::Unauthorized
    );
    assert_eq!(authority.calls.load(Ordering::Relaxed), 0);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}

#[test]
fn prepared_constructors_validate_url_scope_method_and_body_without_execution() {
    let url = Url::parse("https://example.com/search?q=toy-secret").expect("url");
    let m = manifest(&[(&url, Method::GET), (&url, Method::POST)], vec![], 0);
    let mut path = url.clone();
    path.set_path("/other");
    let mut port = url.clone();
    port.set_port(Some(8443)).expect("port");
    let mut userinfo = url.clone();
    userinfo.set_username("toy-secret").expect("userinfo");
    let mut fragment = url.clone();
    fragment.set_fragment(Some("toy-secret"));
    let mut query = url.clone();
    query.set_query(Some(&"q".repeat(4097)));
    let mut long_path = url.clone();
    long_path.set_path(&format!("/{}", "a".repeat(2048)));
    for (url, method, body, expected) in [
        (path, Method::GET, None, TransportError::Unauthorized),
        (port, Method::GET, None, TransportError::Unauthorized),
        (
            url.clone(),
            Method::HEAD,
            None,
            TransportError::Unauthorized,
        ),
        (userinfo, Method::GET, None, TransportError::InvalidRequest),
        (fragment, Method::GET, None, TransportError::InvalidRequest),
        (query, Method::GET, None, TransportError::InvalidRequest),
        (long_path, Method::GET, None, TransportError::InvalidRequest),
        (
            url.clone(),
            Method::PUT,
            None,
            TransportError::InvalidRequest,
        ),
        (
            url.clone(),
            Method::POST,
            Some(vec![0; 16_385]),
            TransportError::InvalidRequest,
        ),
        (
            url.clone(),
            Method::GET,
            Some(vec![0]),
            TransportError::InvalidRequest,
        ),
    ] {
        let error = PreparedTransportRequest::json_bytes(&m, url, method, body)
            .expect_err("invalid preparation");
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("toy-secret"));
    }
    assert_eq!(
        PreparedTransportRequest::form(&m, url.clone(), form_fields(&[("q", &"a".repeat(16_383))]))
            .expect_err("encoded form limit"),
        TransportError::InvalidRequest
    );
    PreparedTransportRequest::json_bytes(&m, url.clone(), Method::POST, Some(vec![0; 16_384]))
        .expect("exact byte bound");
    PreparedTransportRequest::form(&m, url, form_fields(&[("q", &"a".repeat(16_382))]))
        .expect("exact encoded bound");
}

#[tokio::test]
async fn prepared_payloads_own_url_and_bytes_independently_of_source_mutation() {
    for form in [false, true] {
        let server = Server::new(vec![ok("owned")]);
        let mut url = server.url("/search?q=original");
        let m = manifest(&[(&url, Method::POST)], loopback(), 0);
        let mut body = b"{\"q\":\"original\"}".to_vec();
        let mut fields = form_fields(&[("q", "original +&")]);
        let request = if form {
            PreparedTransportRequest::form(&m, url.clone(), fields.clone())
        } else {
            PreparedTransportRequest::json_bytes(&m, url.clone(), Method::POST, Some(body.clone()))
        }
        .expect("prepare owned payload");
        url.set_query(Some("q=replacement"));
        body.fill(b'x');
        fields[0].1 = "replacement".into();
        let t = transport(m, dns(server.port), Duration::from_secs(3), 1, 1024);
        assert_eq!(
            t.execute_prepared(request).await.expect("response").body,
            b"owned"
        );
        let sent = server.request(0);
        assert!(sent.starts_with("POST /search?q=original HTTP/1.1\r\n"));
        assert!(!sent.contains("replacement"));
        let (content_type, expected) = if form {
            ("application/x-www-form-urlencoded", "q=original+%2B%26")
        } else {
            ("application/json", "{\"q\":\"original\"}")
        };
        assert!(
            sent.to_ascii_lowercase()
                .contains(&format!("content-type: {content_type}\r\n"))
        );
        assert_eq!(sent.split_once("\r\n\r\n").expect("body").1, expected);
    }
}

#[tokio::test]
async fn same_scope_queries_run_concurrently_with_separate_invocation_authorities() {
    let server = Server::new(vec![ok("result"), ok("result")]);
    let first_url = server.url("/search?q=first");
    let second_url = server.url("/search?q=second");
    let first = manifest(&[(&first_url, Method::POST)], loopback(), 0);
    let second = ToolNetworkManifest::new(first.scope().clone());
    let a = PreparedTransportRequest::json_bytes(
        &first,
        first_url.clone(),
        Method::POST,
        Some(b"first-body".to_vec()),
    )
    .expect("a");
    let b = PreparedTransportRequest::form(
        &second,
        second_url.clone(),
        form_fields(&[("q", "second-body")]),
    )
    .expect("b");
    let wrong_a = PreparedTransportRequest::json_bytes(&first, first_url, Method::POST, None)
        .expect("wrong a");
    let wrong_b = PreparedTransportRequest::form(&second, second_url, vec![]).expect("wrong b");
    let resolver = dns(server.port);
    let ta = transport(first, resolver.clone(), Duration::from_secs(3), 1, 1024);
    let tb = transport(second, resolver.clone(), Duration::from_secs(3), 1, 1024);
    assert_eq!(
        ta.execute_prepared(wrong_b)
            .await
            .expect_err("wrong a authority"),
        TransportError::Unauthorized
    );
    assert_eq!(
        tb.execute_prepared(wrong_a)
            .await
            .expect_err("wrong b authority"),
        TransportError::Unauthorized
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    let (ra, rb) = tokio::join!(ta.execute_prepared(a), tb.execute_prepared(b));
    assert_eq!(ra.expect("a response").status, 200);
    assert_eq!(rb.expect("b response").status, 200);
    let mut sent = [server.request(0), server.request(1)];
    sent.sort();
    assert!(sent[0].starts_with("POST /search?q=first HTTP/1.1\r\n"));
    assert_eq!(
        sent[0].split_once("\r\n\r\n").expect("body a").1,
        "first-body"
    );
    assert!(sent[1].starts_with("POST /search?q=second HTTP/1.1\r\n"));
    assert_eq!(
        sent[1].split_once("\r\n\r\n").expect("body b").1,
        "q=second-body"
    );
}

#[tokio::test(start_paused = true)]
async fn expired_budget_rejects_prepared_payload_before_authority_dns_or_reservation() {
    let server = Server::new(vec![]);
    let url = server.url("/search");
    let m = manifest(&[(&url, Method::GET)], loopback(), 0);
    let request =
        PreparedTransportRequest::json_bytes(&m, url, Method::GET, None).expect("prepared");
    let authority = Arc::new(CountAuthority {
        calls: AtomicUsize::new(0),
    });
    let resolver = dns(server.port);
    let budget = Arc::new(TransportBudget::new(Duration::from_secs(1), 1, 1024).expect("budget"));
    let t = SearchTransport::new(m, authority.clone(), budget.clone(), resolver.clone())
        .expect("transport");
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        t.execute_prepared(request).await.expect_err("expired"),
        TransportError::Deadline
    );
    assert_eq!(authority.calls.load(Ordering::Relaxed), 0);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(server.count(), 0);
}
