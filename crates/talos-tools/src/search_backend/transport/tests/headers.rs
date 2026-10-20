//! Fixed compatibility profiles only; relocation is confined to synthetic loopback tests.
use super::*;
use crate::search_backend::transport::compatibility::CompatibilityProtocol;

fn loopback_url(server: &Server, path: &str) -> Url {
    let mut url = server.url(path);
    url.set_host(Some("127.0.0.1"))
        .expect("literal fixture peer");
    url
}

fn relocate(
    protocol: CompatibilityProtocol,
    mut url: Url,
    fixture_manifest: &ToolNetworkManifest,
) -> PreparedTransportRequest {
    let official = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            vec![protocol.destination().expect("fixed endpoint")],
            None,
            vec![],
            0,
        )
        .expect("fixed scope"),
    );
    let fixed = protocol
        .prepare(&official, "toy-query")
        .expect("fixed preparation");
    assert_eq!(url.path(), fixed.url.path());
    url.set_query(fixed.url.query());
    let mut request = PreparedTransportRequest::new(
        fixture_manifest,
        url,
        fixed.method.clone(),
        fixed.body.clone(),
        fixed.format,
    )
    .expect("fixture preparation");
    request.headers = fixed.headers;
    assert_eq!(request.method, fixed.method);
    assert_eq!(request.body, fixed.body);
    assert_eq!(request.url.query(), fixed.url.query());
    assert_eq!(request.format.content_type(), fixed.format.content_type());
    assert_eq!(request.headers, fixed.headers);
    request
}

fn header_values<'a>(request: &'a str, name: &str) -> Vec<&'a str> {
    request
        .split_once("\r\n\r\n")
        .expect("HTTP request")
        .0
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim())
        .collect()
}

fn assert_profile(request: &str, user_agent: &str, bing: bool) {
    assert_eq!(header_values(request, "user-agent"), vec![user_agent]);
    assert_eq!(
        header_values(request, "accept-language"),
        if bing {
            vec!["zh-CN,zh;q=0.9,en;q=0.8"]
        } else {
            vec![]
        }
    );
    assert_eq!(header_values(request, "accept-encoding"), vec!["identity"]);
    for name in ["authorization", "proxy-authorization", "cookie", "referer"] {
        assert!(header_values(request, name).is_empty());
    }
}

#[tokio::test]
async fn generic_json_and_form_requests_do_not_acquire_compatibility_headers() {
    for form in [false, true] {
        let server = Server::new(vec![ok("generic")]);
        let url = loopback_url(&server, "/search");
        let m = manifest(&[(&url, Method::POST)], loopback(), 0);
        let request = if form {
            PreparedTransportRequest::form(&m, url, form_fields(&[("q", "toy")]))
        } else {
            PreparedTransportRequest::json_bytes(&m, url, Method::POST, Some(b"{}".to_vec()))
        }
        .expect("generic request");
        assert!(request.headers.is_none());
        let t = transport(m, dns(server.port), Duration::from_secs(3), 1, 1024);
        assert_eq!(
            t.execute_prepared(request).await.expect("response").body,
            b"generic"
        );
        let wire = server.request(0);
        assert!(header_values(&wire, "user-agent").is_empty());
        assert!(header_values(&wire, "accept-language").is_empty());
        assert_eq!(header_values(&wire, "accept-encoding"), vec!["identity"]);
        assert_eq!(
            header_values(&wire, "content-type"),
            vec![if form {
                "application/x-www-form-urlencoded"
            } else {
                "application/json"
            }]
        );
    }
}

#[tokio::test]
async fn scoped_get_redirects_reuse_the_same_snapshot_on_every_origin() {
    for protocol in [CompatibilityProtocol::Html, CompatibilityProtocol::Bing] {
        for cross_origin in [false, true] {
            let other = Server::new(vec![ok("done")]);
            let other_url = loopback_url(&other, "/next");
            let source = Server::new(if cross_origin {
                vec![redirect(307, &other_url)]
            } else {
                vec!["HTTP/1.1 307 Redirect\r\nLocation: /next\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), ok("done")]
            });
            let url = loopback_url(&source, protocol.destination().expect("endpoint").path());
            let next = if cross_origin {
                other_url
            } else {
                loopback_url(&source, "/next")
            };
            let m = manifest(&[(&url, Method::GET), (&next, Method::GET)], loopback(), 1);
            let request = relocate(protocol, url, &m);
            let ua = request.headers.expect("frozen headers").user_agent();
            let t = transport(m, dns(source.port), Duration::from_secs(3), 2, 1024);
            assert_eq!(
                t.execute_prepared(request).await.expect("redirect").body,
                b"done"
            );
            let bing = matches!(protocol, CompatibilityProtocol::Bing);
            assert_profile(&source.request(0), ua, bing);
            if cross_origin {
                assert_profile(&other.request(0), ua, bing);
                assert_eq!(source.count(), 1);
                assert_eq!(other.count(), 1);
            } else {
                assert_profile(&source.request(1), ua, bing);
                assert_eq!(source.count(), 2);
                assert_eq!(other.count(), 0);
            }
            assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 2);
        }
    }
}

#[tokio::test]
async fn frozen_headers_do_not_authorize_an_unscoped_redirect_target() {
    let target = Server::new(vec![ok("unexpected")]);
    let source = Server::new(vec![redirect(302, &loopback_url(&target, "/next"))]);
    let protocol = CompatibilityProtocol::Bing;
    let url = loopback_url(&source, protocol.destination().expect("endpoint").path());
    let m = manifest(&[(&url, Method::GET)], loopback(), 1);
    let request = relocate(protocol, url, &m);
    let ua = request.headers.expect("frozen headers").user_agent();
    let resolver = dns(source.port);
    let t = transport(m, resolver.clone(), Duration::from_secs(3), 2, 1024);
    assert_eq!(
        t.execute_prepared(request)
            .await
            .expect_err("unscoped target"),
        TransportError::Unauthorized
    );
    assert_profile(&source.request(0), ua, true);
    assert_eq!(source.count(), 1);
    assert_eq!(target.count(), 0);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0); // Literal peer, no DNS.
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 1);
}
