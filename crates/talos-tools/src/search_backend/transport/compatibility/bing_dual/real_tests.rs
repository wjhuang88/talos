//! Deterministic real socket / fake DNS execution conformance; no public Bing HTTP requests.

use super::*;
use crate::search_backend::transport::{SearchDns, TransportBudget, tests::Server};
use async_trait::async_trait;
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use talos_core::tool::{
    ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkEndpoint, ToolNetworkScope,
};

struct LocalDns {
    fail_www: bool,
    calls: AtomicUsize,
}

#[async_trait]
impl SearchDns for LocalDns {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail_www && host == "www-fixture.invalid" {
            return Err(TransportError::Dns);
        }
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}

struct LocalAuthority {
    manifest: ToolNetworkManifest,
    deny_cn: bool,
}

impl ToolNetworkAuthority for LocalAuthority {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if invocation != self.manifest.invocation_id()
            || !self.manifest.scope().contains(endpoint)
            || (self.deny_cn && endpoint.host() == "cn-fixture.invalid")
        {
            Err(ToolNetworkAuthorizationError::Mismatch)
        } else {
            Ok(())
        }
    }
}

fn http(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn organic() -> &'static str {
    "<ol><li class='b_algo'><h2><a href='https://result.invalid/one'>One</a></h2><div class='b_caption'><p>One snippet</p></div></li></ol>"
}

/// Relocation of the already frozen plan is restricted to cfg(test), reauthorizing
/// exactly two loopback socket hosts. The production constructor never offers it.
fn local_plan(server: &Server) -> (ToolNetworkManifest, PreparedBingDualHost) {
    let fixed = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            PreparedBingDualHost::destinations().unwrap().to_vec(),
            None,
            vec![],
            0,
        )
        .unwrap(),
    );
    let mut plan = PreparedBingDualHost::prepare_with(
        &fixed,
        "private query & fixture",
        2,
        || "Local-Fixture/1",
    )
    .expect("both production requests frozen");
    let mut www_url = server.url("/search");
    www_url.set_host(Some("www-fixture.invalid")).expect("host");
    www_url.set_query(plan.www.url.query());
    let mut cn_url = server.url("/search");
    cn_url.set_host(Some("cn-fixture.invalid")).expect("host");
    cn_url.set_query(plan.cn.url.query());
    let manifest = ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            [www_url.clone(), cn_url.clone()]
                .iter()
                .map(|u| request_endpoint(u, &Method::GET).expect("endpoint"))
                .collect(),
            None,
            vec!["127.0.0.1".parse().expect("loopback")],
            0,
        )
        .expect("only two local targets"),
    );
    for (request, url) in [(&mut plan.www, www_url), (&mut plan.cn, cn_url)] {
        let headers = request.headers;
        let mut relocated =
            PreparedTransportRequest::new(&manifest, url, Method::GET, None, BodyFormat::Form)
                .expect("test-local frozen payload");
        relocated.headers = headers;
        assert_eq!(relocated.url.query(), request.url.query());
        *request = relocated;
    }
    plan.manifest = manifest.clone();
    (manifest, plan)
}

fn transport(
    manifest: ToolNetworkManifest,
    fail_www: bool,
    deny_cn: bool,
    max_requests: usize,
    max_bytes: usize,
) -> (SearchTransport, Arc<LocalDns>) {
    let dns = Arc::new(LocalDns {
        fail_www,
        calls: AtomicUsize::new(0),
    });
    let t = SearchTransport::new(
        manifest.clone(),
        Arc::new(LocalAuthority { manifest, deny_cn }),
        Arc::new(
            TransportBudget::new(Duration::from_secs(5), max_requests, max_bytes)
                .expect("bounded transport"),
        ),
        dns.clone(),
    )
    .expect("local protected HTTP transport");
    (t, dns)
}

#[tokio::test]
async fn real_www_first_wins_without_cn_connection_and_preserves_frozen_query() {
    let server = Server::new(vec![http(200, organic())]);
    let (manifest, plan) = local_plan(&server);
    let (t, dns) = transport(manifest, false, false, 2, 4096);
    let rows = plan
        .execute(&t)
        .await
        .expect("real WWW result")
        .into_results();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].url, "https://result.invalid/one");
    assert_eq!(server.count(), 1);
    assert_eq!(dns.calls.load(Ordering::Relaxed), 1);
    assert!(
        server
            .request(0)
            .to_ascii_lowercase()
            .contains("host: www-fixture.invalid")
    );
    assert!(server.request(0).contains("q=private+query+%26+fixture"));
}

#[tokio::test]
async fn real_dns_unavailability_rotates_only_to_prepared_cn_once() {
    let server = Server::new(vec![http(200, organic())]);
    let (manifest, plan) = local_plan(&server);
    let (t, dns) = transport(manifest, true, false, 2, 4096);
    let rows = plan
        .execute(&t)
        .await
        .expect("real CN fallback")
        .into_results();
    assert_eq!(rows.len(), 1);
    assert_eq!(server.count(), 1);
    assert_eq!(dns.calls.load(Ordering::Relaxed), 2);
    assert!(
        server
            .request(0)
            .to_ascii_lowercase()
            .contains("host: cn-fixture.invalid")
    );
    assert!(server.request(0).contains("mkt=zh-CN"));
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn real_http_denials_all_5xx_and_valid_empty_never_rotate_host() {
    for (status, body) in [
        (403, "denied"),
        (429, "rate limited"),
        (500, "server"),
        (502, "bad gateway"),
        (503, "unavailable"),
        (200, "<html>No organic results</html>"),
        (302, "redirect"),
    ] {
        let server = Server::new(vec![http(status, body)]);
        let (manifest, plan) = local_plan(&server);
        let (t, dns) = transport(manifest, false, false, 2, 4096);
        assert!(
            plan.execute(&t).await.is_err(),
            "status {status} must be terminal"
        );
        assert_eq!(server.count(), 1, "no CN send on status {status}");
        assert_eq!(dns.calls.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test]
async fn real_cn_denial_and_request_limit_block_after_first_dns_failure() {
    for (deny_cn, allowed_requests) in [(true, 2), (false, 1)] {
        let server = Server::new(vec![http(200, organic())]);
        let (manifest, plan) = local_plan(&server);
        let (t, dns) = transport(manifest, true, deny_cn, allowed_requests, 4096);
        assert!(plan.execute(&t).await.is_err());
        assert_eq!(server.count(), 0, "CN never reaches socket");
        assert_eq!(dns.calls.load(Ordering::Relaxed), 1);
        assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test]
async fn real_source_local_timeout_uses_same_budget_and_reaches_cn_only_when_live() {
    let server = Server::new(vec![http(200, organic())]);
    let (manifest, plan) = local_plan(&server);
    let (t, dns) = transport(manifest, true, false, 2, 4096);
    // A caller-cancelled/expired invocation must never start or reset a new host budget.
    // The dedicated I333 synthetic tests cover source-local expiry without sleeping 15s.
    let mut t = t;
    Arc::get_mut(&mut t.budget)
        .expect("unique test budget")
        .deadline = Instant::now() - Duration::from_millis(1);
    assert!(
        plan.execute_with_source_timeout(&t, Duration::from_millis(1))
            .await
            .is_err()
    );
    assert_eq!(server.count(), 0);
    assert_eq!(dns.calls.load(Ordering::Relaxed), 0);
    assert_eq!(t.budget.attempts.load(Ordering::Relaxed), 0);
}

struct SharedProxyDnsFailure {
    calls: AtomicUsize,
}

#[async_trait]
impl SearchDns for SharedProxyDnsFailure {
    async fn resolve(&self, _: &str, _: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(TransportError::Dns)
    }
}

#[tokio::test]
async fn proxy_dns_failure_is_shared_infrastructure_and_must_not_rotate_bing_host() {
    use crate::search_backend::transport::TransportProxy;

    let endpoints = PreparedBingDualHost::destinations().expect("two frozen hosts");
    let proxy = TransportProxy::from_snapshot(
        "http://proxy-fixture.invalid:3128",
        "",
        &endpoints,
    )
    .expect("proxy policy")
    .expect("both hosts use proxy");
    let scope = ToolNetworkScope::new_configuration(
        endpoints.to_vec(),
        Some(proxy.endpoint().clone()),
        vec![],
        0,
    )
    .expect("both hosts plus proxy approved before query");
    let manifest = ToolNetworkManifest::new(scope);
    let plan = PreparedBingDualHost::prepare_with(
        &manifest,
        "proxy secret query",
        2,
        || "Local-Fixture/1",
    )
    .expect("both query-bearing requests frozen");
    let dns = Arc::new(SharedProxyDnsFailure {
        calls: AtomicUsize::new(0),
    });
    let budget =
        Arc::new(TransportBudget::new(Duration::from_secs(5), 4, 4096).expect("two proxy sends"));
    let t = SearchTransport::new_with_proxy(
        manifest.clone(),
        Arc::new(LocalAuthority {
            manifest,
            deny_cn: false,
        }),
        budget.clone(),
        dns.clone(),
        Some(proxy),
    )
    .expect("prepared proxy transport");
    assert_eq!(
        plan.execute(&t)
            .await
            .expect_err("proxy DNS cannot authorize CN rotation"),
        CompatibilityError::Transport(TransportError::Dns),
    );
    assert_eq!(dns.calls.load(Ordering::Relaxed), 1);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 2);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
}
