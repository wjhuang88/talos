use super::*;
use crate::search_backend::{
    SearchBackendId,
    transport::{BodyFormat, PreparedTransportRequest, SearchDns, TransportBudget, tests::Server},
};
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

fn manifest() -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            ORDER
                .iter()
                .map(|p| p.destination().expect("fixed destination"))
                .collect(),
            None,
            vec![],
            0,
        )
        .expect("scope"),
    )
}

fn prepare() -> PreparedCompatibilityChain {
    PreparedCompatibilityChain::prepare(&manifest(), " 中 +&~ ", 1).expect("chain")
}

fn response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn valid(protocol: CompatibilityProtocol) -> String {
    response(
        &String::from_utf8(super::super::execution_tests::body(protocol)).expect("fixture UTF-8"),
    )
}

fn empty(protocol: CompatibilityProtocol) -> String {
    response(if protocol == CompatibilityProtocol::InstantAnswer {
        r#"{"AbstractText":"","AbstractURL":"","AbstractSource":"","Heading":"","Answer":"","AnswerType":"","Definition":"","DefinitionSource":"","RelatedTopics":[],"Infobox":null,"Type":""}"#
    } else {
        "<html></html>"
    })
}

// Reconstruct only under cfg(test), validating every relocated endpoint and preserving
// frozen bytes/headers/protocol/query/count. Production exposes no destination override.
pub(in crate::search_backend::transport::compatibility) fn relocate(
    chain: PreparedCompatibilityChain,
    server: &Server,
) -> (PreparedCompatibilityChain, ToolNetworkManifest) {
    relocate_with_hosts(chain, server, false)
}

fn relocate_with_hosts(
    chain: PreparedCompatibilityChain,
    server: &Server,
    distinct: bool,
) -> (PreparedCompatibilityChain, ToolNetworkManifest) {
    relocate_with_ports(chain, server, distinct, None)
}

fn relocate_request(
    request: &mut PreparedTransportRequest,
    manifest: &ToolNetworkManifest,
    url: reqwest::Url,
) {
    let body = request.body.clone();
    let query = request.url.query().map(str::to_owned);
    let headers = request.headers;
    let mut relocated = PreparedTransportRequest::new(
        manifest,
        url,
        request.method.clone(),
        request.body.take(),
        BodyFormat::Form,
    )
    .expect("relocated request");
    relocated.headers = headers;
    assert_eq!(relocated.body, body);
    assert_eq!(relocated.url.query(), query.as_deref());
    assert_eq!(relocated.headers, headers);
    assert_eq!(relocated.invocation_id, manifest.invocation_id());
    *request = relocated;
}

fn relocate_with_ports(
    mut chain: PreparedCompatibilityChain,
    server: &Server,
    distinct: bool,
    ports: Option<[u16; 4]>,
) -> (PreparedCompatibilityChain, ToolNetworkManifest) {
    let urls: Vec<_> = chain
        .sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let mut url = server.url(source.request.url.path());
            if let Some(ports) = ports {
                url.set_port(Some(ports[index])).expect("fixture port");
            }
            if distinct {
                url.set_host(Some(&format!("source-{index}.invalid")))
                    .expect("synthetic host");
            }
            url.set_query(source.request.url.query());
            url
        })
        .collect();
    let scope = ToolNetworkScope::new_configuration(
        chain
            .sources
            .iter()
            .zip(&urls)
            .map(|(source, url)| {
                super::super::super::request_endpoint(url, &source.request.method)
                    .expect("endpoint")
            })
            .collect(),
        None,
        vec!["127.0.0.1".parse().expect("IP")],
        0,
    )
    .expect("loopback scope");
    let manifest = ToolNetworkManifest::new(scope);
    for (source, url) in chain.sources.iter_mut().zip(urls) {
        relocate_request(&mut source.request, &manifest, url.clone());
        if let Some(retry) = source.retry.as_mut() {
            relocate_request(retry, &manifest, url);
        }
    }
    (chain, manifest)
}

struct Effects {
    manifest: ToolNetworkManifest,
    checks: AtomicUsize,
    dns: AtomicUsize,
    allowed_checks: usize,
}
impl ToolNetworkAuthority for Effects {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        if self.checks.fetch_add(1, Ordering::Relaxed) >= self.allowed_checks {
            return Err(ToolNetworkAuthorizationError::Invalidated);
        }
        if invocation != self.manifest.invocation_id() || !self.manifest.scope().contains(endpoint)
        {
            return Err(ToolNetworkAuthorizationError::Mismatch);
        }
        Ok(())
    }
}
#[async_trait]
impl SearchDns for Effects {
    async fn resolve(&self, _: &str, port: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.dns.fetch_add(1, Ordering::Relaxed);
        Ok(vec![SocketAddr::from(([127, 0, 0, 1], port))])
    }
}
fn transport(
    manifest: ToolNetworkManifest,
    requests: usize,
    bytes: usize,
    allowed_checks: usize,
) -> (SearchTransport, Arc<Effects>, Arc<TransportBudget>) {
    let effects = Arc::new(Effects {
        manifest: manifest.clone(),
        checks: AtomicUsize::new(0),
        dns: AtomicUsize::new(0),
        allowed_checks,
    });
    let budget =
        Arc::new(TransportBudget::new(Duration::from_secs(5), requests, bytes).expect("budget"));
    (
        SearchTransport::new(manifest, effects.clone(), budget.clone(), effects.clone())
            .expect("transport"),
        effects,
        budget,
    )
}

#[test]
fn all_payload_validation_precedes_any_draw_including_late_get_bounds_and_scope() {
    let draws = AtomicUsize::new(0);
    let select = || {
        draws.fetch_add(1, Ordering::Relaxed);
        "synthetic UA"
    };
    for (query, max, scope) in [
        ("q".to_owned(), 0, manifest()),
        (" ".to_owned(), 1, manifest()),
        ("x".repeat(4096), 1, manifest()), // Lite form fits; later GET does not.
        (
            "q".to_owned(),
            1,
            ToolNetworkManifest::new(
                ToolNetworkScope::new_configuration(
                    ORDER[..3]
                        .iter()
                        .map(|p| p.destination().expect("destination"))
                        .collect(),
                    None,
                    vec![],
                    0,
                )
                .expect("scope without Bing"),
            ),
        ),
    ] {
        assert!(PreparedCompatibilityChain::prepare_with(&scope, &query, max, select).is_err());
        assert_eq!(draws.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn frozen_chain_owns_four_headers_query_count_and_three_prepared_retries() {
    let draws = AtomicUsize::new(0);
    let mut query = "toy-secret".to_owned();
    let chain = PreparedCompatibilityChain::prepare_with(&manifest(), &query, 2, || {
        ["ua-lite", "ua-instant", "ua-html", "ua-bing"][draws.fetch_add(1, Ordering::Relaxed)]
    })
    .expect("four draws");
    query.clear();
    assert_eq!(
        format!("{chain:?}"),
        "PreparedCompatibilityChain(<redacted>)"
    );
    assert_eq!(draws.load(Ordering::Relaxed), 4);
    for (index, source) in chain.sources.iter().enumerate() {
        assert_eq!(source.protocol, ORDER[index]);
        assert_eq!(source.query, "toy-secret");
        assert_eq!(source.max_results, 2);
        let headers = source.request.headers.expect("headers");
        assert_eq!(
            headers.user_agent(),
            ["ua-lite", "ua-instant", "ua-html", "ua-bing"][index]
        );
        assert_eq!(source.retry.is_some(), matches!(index, 0 | 2 | 3));
        if let Some(retry) = &source.retry {
            assert_eq!(retry.headers, Some(headers));
            assert_eq!(retry.url, source.request.url);
            assert_eq!(retry.body, source.request.body);
        }
    }
    for panic in [false, true] {
        let draws = AtomicUsize::new(0);
        let error = PreparedCompatibilityChain::prepare_with(&manifest(), "q", 1, || {
            if draws.fetch_add(1, Ordering::Relaxed) == 1 {
                assert!(!panic, "synthetic helper failure");
                "invalid\r\nheader"
            } else {
                "synthetic UA"
            }
        })
        .expect_err("capture stops at second draw");
        assert_eq!(
            error,
            CompatibilityError::Transport(TransportError::InvalidRequest)
        );
        assert_eq!(draws.load(Ordering::Relaxed), 2);
    }
}

#[test]
fn continuation_table_allows_only_typed_availability_and_semantic_failures() {
    for error in [
        TransportError::Dns,
        TransportError::ConnectionRefused,
        TransportError::Network,
    ] {
        assert!(may_continue(CompatibilityError::Transport(error)));
    }
    for error in [
        DdgParseError::Empty,
        DdgParseError::Status,
        DdgParseError::Challenge,
    ] {
        assert!(may_continue(CompatibilityError::Ddg(error)));
    }
    for error in [
        BingParseError::Empty,
        BingParseError::Status,
        BingParseError::RateLimited,
        BingParseError::Challenge,
    ] {
        assert!(may_continue(CompatibilityError::Bing(error)));
    }
    for error in [AdmissionError::Empty, AdmissionError::Invalid] {
        assert!(may_continue(CompatibilityError::Admission(error)));
    }
    for error in [
        TransportError::InvalidRequest,
        TransportError::Unauthorized,
        TransportError::UnsupportedProxy,
        TransportError::Address,
        TransportError::Redirect,
        TransportError::RequestLimit,
        TransportError::ResponseLimit,
        TransportError::ResponseEncoding,
        TransportError::Deadline,
        TransportError::Request,
    ] {
        assert!(!may_continue(CompatibilityError::Transport(error)));
    }
    for error in [
        DdgParseError::Limit,
        DdgParseError::Encoding,
        DdgParseError::Parse,
    ] {
        assert!(!may_continue(CompatibilityError::Ddg(error)));
    }
    for error in [
        BingParseError::Limit,
        BingParseError::Encoding,
        BingParseError::Parse,
    ] {
        assert!(!may_continue(CompatibilityError::Bing(error)));
    }
    for error in [
        AdmissionError::Limit,
        AdmissionError::Identity,
        AdmissionError::Parse,
    ] {
        assert!(!may_continue(CompatibilityError::Admission(error)));
    }
}

mod dns_tests;

#[tokio::test]
async fn each_winner_position_preserves_frozen_wire_order_and_starts_no_later_source() {
    for winner in 0..4 {
        let server = Server::new(
            (0..=winner)
                .map(|i| {
                    if i == winner {
                        valid(ORDER[i])
                    } else {
                        empty(ORDER[i])
                    }
                })
                .collect(),
        );
        let chain = prepare();
        let uas: Vec<_> = chain
            .sources
            .iter()
            .map(|s| s.request.headers.expect("headers").user_agent())
            .collect();
        let (chain, manifest) = relocate(chain, &server);
        let (transport, effects, budget) = transport(manifest, 4, 1_048_576, usize::MAX);
        let rows = chain
            .execute(&transport)
            .await
            .expect("first admitted source")
            .into_results();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].backend, SearchBackendId::DuckDuckGo);
        assert_eq!(server.count(), winner + 1);
        assert_eq!(effects.dns.load(Ordering::Relaxed), winner + 1);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), winner + 1);
        for index in 0..=winner {
            let sent = server.request(index);
            let method = if index == 0 { "POST" } else { "GET" };
            let path = ORDER[index]
                .destination()
                .expect("endpoint")
                .path()
                .to_owned();
            assert!(sent.starts_with(&format!(
                "{method} {path}{}",
                if index == 0 { " HTTP/1.1" } else { "?" }
            )));
            assert!(
                sent.lines()
                    .any(|line| line.eq_ignore_ascii_case(&format!("user-agent: {}", uas[index])))
            );
            assert_eq!(
                sent.to_ascii_lowercase().contains("accept-language:"),
                index == 3
            );
            assert!(sent.contains("%E4%B8%AD+%2B%26%7E+"));
            for name in [
                "authorization:",
                "proxy-authorization:",
                "cookie:",
                "referer:",
            ] {
                assert!(!sent.to_ascii_lowercase().contains(name));
            }
        }
    }
}

#[tokio::test]
async fn invalid_only_source_cannot_win_and_exhaustion_retains_four_observed_failures() {
    let invalid = response(
        &String::from_utf8(super::super::execution_tests::body(
            CompatibilityProtocol::Lite,
        ))
        .expect("body")
        .replace("https://first.invalid/", "file:///first")
        .replace("https://second.invalid/", "file:///second"),
    );
    let server = Server::new(vec![invalid, valid(CompatibilityProtocol::InstantAnswer)]);
    let (chain, manifest) = relocate(prepare(), &server);
    let (transport, _, _) = transport(manifest, 4, 1_048_576, usize::MAX);
    assert!(
        chain
            .execute(&transport)
            .await
            .expect("later valid source")
            .into_results()[0]
            .url
            .starts_with("https://duckduckgo.com/")
    );
    assert_eq!(server.count(), 2);
    let server = Server::new(ORDER.iter().map(|p| empty(*p)).collect());
    let (chain, manifest) = relocate(prepare(), &server);
    let (transport, _, _) = self::transport(manifest, 4, 1_048_576, usize::MAX);
    let error = chain.execute(&transport).await.expect_err("all four empty");
    assert_eq!(
        error,
        CompatibilityChainError::Exhausted([
            CompatibilityError::Ddg(DdgParseError::Empty),
            CompatibilityError::Ddg(DdgParseError::Empty),
            CompatibilityError::Ddg(DdgParseError::Empty),
            CompatibilityError::Bing(BingParseError::Empty),
        ])
    );
    assert_eq!(error.to_string(), "search_compatibility_chain_exhausted");
    assert!(!format!("{error:?}").contains("toy-secret"));
    assert_eq!(server.count(), 4);
}

#[tokio::test]
async fn malformed_instant_answer_schema_is_terminal_not_an_empty_result() {
    let server = Server::new(vec![empty(ORDER[0]), response("{}"), valid(ORDER[2])]);
    let (chain, manifest) = relocate(prepare(), &server);
    let (transport, effects, _) = transport(manifest, 4, 1_048_576, usize::MAX);
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("schema failure is terminal"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[1],
            error: CompatibilityError::Ddg(DdgParseError::Parse),
        }
    );
    assert_eq!(server.count(), 2);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn terminal_response_failures_never_advance_to_another_source() {
    let oversized_url = format!("https://oversized.invalid/{}", "x".repeat(8192));
    for (wire, expected) in [
        ("HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(), CompatibilityError::Transport(TransportError::ResponseEncoding)),
        (response("\u{feff}<html></html>"), CompatibilityError::Ddg(DdgParseError::Encoding)),
        (response(&String::from_utf8(super::super::execution_tests::body(CompatibilityProtocol::Lite)).expect("body").replace("https://first.invalid/", &oversized_url)), CompatibilityError::Admission(AdmissionError::Limit)),
    ] {
        let server = Server::new(vec![wire, valid(CompatibilityProtocol::InstantAnswer)]);
        let (chain, manifest) = relocate(prepare(), &server);
        let (transport, effects, _) = transport(manifest, 4, 1_048_576, usize::MAX);
        assert_eq!(chain.execute(&transport).await.expect_err("terminal"), CompatibilityChainError::Terminal { protocol: ORDER[0], error: expected });
        assert_eq!(server.count(), 1);
        assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test]
async fn shared_request_and_byte_limits_are_not_reset_between_sources() {
    for (requests, bytes, expected, sends) in [
        (1, 1_048_576, TransportError::RequestLimit, 1),
        (4, "<html></html>".len(), TransportError::ResponseLimit, 2),
    ] {
        let server = Server::new(ORDER.iter().map(|p| empty(*p)).collect());
        let (chain, manifest) = relocate(prepare(), &server);
        let (transport, effects, budget) = transport(manifest, requests, bytes, usize::MAX);
        assert_eq!(
            chain
                .execute(&transport)
                .await
                .expect_err("cumulative budget"),
            CompatibilityChainError::Terminal {
                protocol: ORDER[1],
                error: CompatibilityError::Transport(expected)
            }
        );
        assert_eq!(server.count(), sends);
        assert_eq!(effects.dns.load(Ordering::Relaxed), sends);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), sends);
        assert_eq!(budget.bytes.load(Ordering::Relaxed), "<html></html>".len());
    }
}

#[tokio::test]
async fn later_authority_rejection_and_wrong_invocation_stop_before_further_dns_or_send() {
    let server = Server::new(ORDER.iter().map(|p| empty(*p)).collect());
    let (chain, manifest) = relocate(prepare(), &server);
    let (transport, effects, _) = transport(manifest.clone(), 4, 1_048_576, 2);
    assert_eq!(
        chain
            .execute(&transport)
            .await
            .expect_err("later live checker denial"),
        CompatibilityChainError::Terminal {
            protocol: ORDER[1],
            error: CompatibilityError::Transport(TransportError::Unauthorized)
        }
    );
    assert_eq!(server.count(), 1);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    let server = Server::new(vec![]);
    let (chain, manifest) = relocate(prepare(), &server);
    let other = ToolNetworkManifest::new(manifest.scope().clone());
    let (transport, effects, budget) = self::transport(other, 4, 1_048_576, usize::MAX);
    assert!(matches!(
        chain.execute(&transport).await,
        Err(CompatibilityChainError::Terminal {
            error: CompatibilityError::Transport(TransportError::Unauthorized),
            ..
        })
    ));
    assert_eq!(server.count(), 0);
    assert_eq!(effects.checks.load(Ordering::Relaxed), 0);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn caller_abort_or_actual_shared_deadline_disposes_owned_request_without_later_source() {
    for abort in [true, false] {
        let server = Server::new(vec![]);
        let (chain, manifest) = relocate(prepare(), &server);
        let (transport, effects, _) = transport(manifest, 4, 1_048_576, usize::MAX);
        let task = tokio::spawn(async move { chain.execute(&transport).await });
        tokio::time::timeout(Duration::from_secs(3), async {
            while server.accepted.load(Ordering::Relaxed) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("actual accepted connection");
        if abort {
            task.abort();
            assert!(task.await.expect_err("aborted").is_cancelled());
        } else {
            // Freeze only after the real connection starts, avoiding auto-advance during setup.
            tokio::time::pause();
            tokio::time::advance(Duration::from_secs(5)).await;
            let result = task.await;
            tokio::time::resume();
            assert_eq!(
                result.expect("joined").expect_err("deadline"),
                CompatibilityChainError::Terminal {
                    protocol: ORDER[0],
                    error: CompatibilityError::Transport(TransportError::Deadline)
                }
            );
        }
        tokio::time::timeout(Duration::from_secs(3), async {
            while server.closed.load(Ordering::Relaxed) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("observed peer EOF");
        assert_eq!(server.count(), 1);
        assert_eq!(effects.dns.load(Ordering::Relaxed), 1);
    }
}
