//! Synthetic responses and explicit loopback scopes only; never official-host requests.
use super::*;
use crate::search_backend::{
    SearchBackendId,
    transport::{SearchDns, TransportBudget},
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
use talos_core::tool::{ToolNetworkAuthority, ToolNetworkAuthorizationError, ToolNetworkScope};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
    time::timeout,
};

const ALL: [CompatibilityProtocol; 4] = [
    CompatibilityProtocol::Lite,
    CompatibilityProtocol::Html,
    CompatibilityProtocol::InstantAnswer,
    CompatibilityProtocol::Bing,
];

fn scope(endpoint: ToolNetworkEndpoint, loopback: bool) -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(
            vec![endpoint],
            None,
            if loopback {
                vec!["127.0.0.1".parse().expect("IP")]
            } else {
                vec![]
            },
            0,
        )
        .expect("scope"),
    )
}

fn prepared(protocol: CompatibilityProtocol, query: &str, max: u32) -> PreparedCompatibilitySearch {
    protocol
        .prepare_search(
            &scope(protocol.destination().expect("endpoint"), false),
            query,
            max,
        )
        .expect("fixed factory")
}

fn body(protocol: CompatibilityProtocol) -> Vec<u8> {
    match protocol {
        CompatibilityProtocol::Lite => b"<table><tr><td><a class='result-link' href='https://first.invalid/'>First</a></td></tr><tr><td class='result-snippet'>one</td></tr><tr><td><a class='result-link' href='https://second.invalid/'>Second</a></td></tr></table>".to_vec(),
        CompatibilityProtocol::Html => b"<div><a class='result__a' href='https://first.invalid/'>First</a><a class='result__snippet'>one</a></div><div><a class='result__a' href='https://second.invalid/'>Second</a></div>".to_vec(),
        CompatibilityProtocol::Bing => b"<ol><li class='b_algo'><h2><a href='https://first.invalid/'>First</a></h2><div class='b_caption'><p>one</p></div></li><li class='b_algo'><h2><a href='https://second.invalid/'>Second</a></h2></li></ol>".to_vec(),
        CompatibilityProtocol::InstantAnswer => serde_json::to_vec(&serde_json::json!({
            "AbstractText":"", "AbstractURL":"", "AbstractSource":"", "Heading":"",
            "Answer":"one", "AnswerType":"", "Definition":"two", "DefinitionSource":"https://second.invalid/",
            "RelatedTopics":[], "Infobox":null, "Type":""
        })).expect("synthetic JSON"),
    }
}

struct Effects {
    manifest: ToolNetworkManifest,
    authority: AtomicUsize,
    dns: AtomicUsize,
}
impl ToolNetworkAuthority for Effects {
    fn check(
        &self,
        invocation: uuid::Uuid,
        endpoint: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        self.authority.fetch_add(1, Ordering::Relaxed);
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
) -> (SearchTransport, Arc<Effects>, Arc<TransportBudget>) {
    let effects = Arc::new(Effects {
        manifest: manifest.clone(),
        authority: AtomicUsize::new(0),
        dns: AtomicUsize::new(0),
    });
    let budget =
        Arc::new(TransportBudget::new(Duration::from_secs(5), 1, 1_048_576).expect("budget"));
    (
        SearchTransport::new(manifest, effects.clone(), budget.clone(), effects.clone())
            .expect("transport"),
        effects,
        budget,
    )
}

struct Fixture {
    url: Url,
    task: Option<JoinHandle<Vec<u8>>>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
impl Fixture {
    async fn new(status: u16, body: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let url = Url::parse(&format!(
            "http://127.0.0.1:{}/",
            listener.local_addr().expect("address").port()
        ))
        .expect("URL");
        let task = tokio::spawn(async move {
            let (mut stream, _) = timeout(Duration::from_secs(5), listener.accept())
                .await
                .expect("accept bound")
                .expect("accept");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0; 4096];
                let n = timeout(Duration::from_secs(5), stream.read(&mut chunk))
                    .await
                    .expect("read bound")
                    .expect("read");
                assert!(n > 0, "complete request");
                request.extend_from_slice(&chunk[..n]);
                assert!(request.len() <= 32_768, "fixture request bound");
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = std::str::from_utf8(&request[..end])
                        .expect("headers")
                        .to_ascii_lowercase();
                    let length = headers
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .map(|n| n.trim().parse::<usize>().expect("length"))
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let headers = format!(
                "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).await.expect("headers");
            stream.write_all(&body).await.expect("body");
            request
        });
        Self {
            url,
            task: Some(task),
        }
    }
    async fn finish(&mut self) -> String {
        String::from_utf8(
            timeout(Duration::from_secs(5), self.task.take().expect("task"))
                .await
                .expect("join bound")
                .expect("server task"),
        )
        .expect("request UTF-8")
    }
}

// Test-only relocation. No production URL/manifest override or transport trait is added.
fn relocate(
    mut search: PreparedCompatibilitySearch,
    base: &Url,
) -> (PreparedCompatibilitySearch, ToolNetworkManifest) {
    let mut url = base.clone();
    url.set_path(search.request.url.path());
    url.set_query(search.request.url.query());
    let manifest = scope(
        request_endpoint(&url, &search.request.method).expect("fixture endpoint"),
        true,
    );
    let old = search.request;
    let request = PreparedTransportRequest::new(
        &manifest,
        url,
        old.method.clone(),
        old.body.clone(),
        old.format,
    )
    .expect("fixture preparation");
    assert_eq!(request.method, old.method);
    assert_eq!(request.body, old.body);
    assert_eq!(request.url.path(), old.url.path());
    assert_eq!(request.url.query(), old.url.query());
    assert_eq!(request.format.content_type(), old.format.content_type());
    assert_eq!(request.invocation_id, manifest.invocation_id());
    search.request = request;
    (search, manifest)
}

#[tokio::test]
async fn all_four_protocols_send_frozen_bytes_and_use_captured_parser_query_and_limit() {
    for protocol in ALL {
        for max in [1, 2] {
            let mut query = " 中 +&~ ".to_owned();
            let search = prepared(protocol, &query, max);
            let expected = search.request.body.clone();
            let expected_query = search.request.url.query().map(str::to_owned);
            let path = search.request.url.path().to_owned();
            let method = search.request.method.clone();
            query.clear();
            query.push_str("replacement");
            let mut fixture = Fixture::new(200, body(protocol)).await;
            let (search, manifest) = relocate(search, &fixture.url);
            assert_eq!(search.query, " 中 +&~ ");
            assert_eq!(search.max_results, max);
            let (transport, _, _) = transport(manifest);
            let candidates = search
                .execute(&transport)
                .await
                .expect("controlled response");
            assert_eq!(
                format!("{candidates:?}"),
                "CompatibilityCandidates(<redacted>)"
            );
            let rows = candidates.into_candidates();
            assert_eq!(rows.len(), max as usize);
            assert!(
                rows.iter()
                    .all(|r| r.backend == SearchBackendId::DuckDuckGo)
            );
            if matches!(protocol, CompatibilityProtocol::InstantAnswer) {
                assert_eq!(rows[0].title, "Answer:  中 +&~ ");
                assert_eq!(rows[0].url, "https://duckduckgo.com/?q=+%E4%B8%AD+%2B%26~+");
            } else {
                assert_eq!(rows[0].title, "First");
                assert_eq!(rows[0].url, "https://first.invalid/");
            }
            assert_eq!(rows[0].snippet, "one");
            if max == 2 {
                assert_eq!(rows[1].url, "https://second.invalid/");
            }
            let sent = fixture.finish().await;
            let target = format!(
                "{path}{}",
                expected_query.map(|q| format!("?{q}")).unwrap_or_default()
            );
            assert!(sent.starts_with(&format!("{method} {target} HTTP/1.1\r\n")));
            let (headers, body) = sent.split_once("\r\n\r\n").expect("HTTP body");
            assert_eq!(body.as_bytes(), expected.as_deref().unwrap_or_default());
            assert_eq!(
                headers
                    .lines()
                    .any(|l| l
                        .eq_ignore_ascii_case("content-type: application/x-www-form-urlencoded")),
                method == Method::POST
            );
            assert!(!sent.contains("replacement"));
        }
    }
}

#[test]
fn preparation_validates_count_then_query_and_scope_with_redacted_debug() {
    for protocol in ALL {
        let manifest = scope(protocol.destination().expect("endpoint"), false);
        for max in [0, 21, u32::MAX] {
            assert_eq!(
                protocol
                    .prepare_search(&manifest, "toy-secret", max)
                    .expect_err("max"),
                CompatibilityError::Transport(TransportError::InvalidRequest)
            );
        }
        for query in ["".to_owned(), " ".to_owned(), "toy-secret".repeat(2000)] {
            assert_eq!(
                protocol
                    .prepare_search(&manifest, &query, 1)
                    .expect_err("query"),
                CompatibilityError::Transport(TransportError::InvalidRequest)
            );
        }
        let search = protocol
            .prepare_search(&manifest, "toy-secret", 20)
            .expect("upper max");
        assert_eq!(
            format!("{search:?}"),
            "PreparedCompatibilitySearch(<redacted>)"
        );
        let wrong = scope(
            ToolNetworkEndpoint::new("https", "other.invalid", 443, "GET", "/").expect("endpoint"),
            false,
        );
        let error = protocol
            .prepare_search(&wrong, "toy-secret", 1)
            .expect_err("scope");
        assert_eq!(
            error,
            CompatibilityError::Transport(TransportError::Unauthorized)
        );
        assert!(!format!("{error:?} {error}").contains("toy-secret"));
    }
}

#[tokio::test]
async fn another_invocation_cannot_execute_or_parse_and_has_zero_network_effects() {
    for protocol in ALL {
        let first = scope(protocol.destination().expect("endpoint"), false);
        let search = protocol
            .prepare_search(&first, "toy-secret", 1)
            .expect("prepared");
        let (transport, effects, budget) =
            transport(ToolNetworkManifest::new(first.scope().clone()));
        assert_eq!(
            search
                .execute(&transport)
                .await
                .expect_err("wrong invocation"),
            CompatibilityError::Transport(TransportError::Unauthorized)
        );
        assert_eq!(effects.authority.load(Ordering::Relaxed), 0);
        assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
        assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    }
}

#[tokio::test]
async fn parser_status_and_challenge_errors_remain_closed_after_real_controlled_dispatch() {
    for (protocol, status, body, expected) in [
        (
            CompatibilityProtocol::Lite,
            500,
            b"toy-secret".as_slice(),
            CompatibilityError::Ddg(ddg_parser::DdgParseError::Status),
        ),
        (
            CompatibilityProtocol::Html,
            202,
            b"toy-secret".as_slice(),
            CompatibilityError::Ddg(ddg_parser::DdgParseError::Challenge),
        ),
        (
            CompatibilityProtocol::InstantAnswer,
            200,
            b"{toy-secret".as_slice(),
            CompatibilityError::Ddg(ddg_parser::DdgParseError::Parse),
        ),
        (
            CompatibilityProtocol::Bing,
            429,
            b"toy-secret".as_slice(),
            CompatibilityError::Bing(bing_parser::BingParseError::RateLimited),
        ),
        (
            CompatibilityProtocol::Bing,
            200,
            b"bnp_dialog toy-secret".as_slice(),
            CompatibilityError::Bing(bing_parser::BingParseError::Challenge),
        ),
    ] {
        let mut fixture = Fixture::new(status, body.to_vec()).await;
        let (search, manifest) = relocate(prepared(protocol, "toy-secret", 1), &fixture.url);
        let (transport, _, _) = transport(manifest);
        let error = search
            .execute(&transport)
            .await
            .expect_err("classified response");
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("toy-secret"));
        fixture.finish().await;
    }
}

#[test]
fn empty_responses_remain_non_winners_and_unsafe_candidates_remain_unadmitted() {
    for protocol in [
        CompatibilityProtocol::Lite,
        CompatibilityProtocol::Html,
        CompatibilityProtocol::Bing,
    ] {
        let empty = finish_response(
            protocol,
            "q",
            1,
            TransportResponse {
                status: 200,
                body: b"<html></html>".to_vec(),
            },
            || Ok(()),
        );
        assert!(matches!(
            empty,
            Err(CompatibilityError::Ddg(ddg_parser::DdgParseError::Empty)
                | CompatibilityError::Bing(bing_parser::BingParseError::Empty))
        ));
        let body = String::from_utf8(body(protocol))
            .expect("HTML")
            .replace("https://first.invalid/", "file:///untrusted");
        let rows = finish_response(
            protocol,
            "q",
            1,
            TransportResponse {
                status: 200,
                body: body.into_bytes(),
            },
            || Ok(()),
        )
        .expect("untrusted data")
        .into_candidates();
        assert_eq!(rows[0].url, "file:///untrusted");
    }
}

#[test]
fn wrong_format_responses_do_not_become_a_success_through_another_parser() {
    for protocol in ALL {
        let other = if matches!(protocol, CompatibilityProtocol::Bing) {
            CompatibilityProtocol::Html
        } else {
            CompatibilityProtocol::Bing
        };
        assert!(
            finish_response(
                protocol,
                "q",
                1,
                TransportResponse {
                    status: 200,
                    body: body(other)
                },
                || Ok(())
            )
            .is_err()
        );
    }
}

#[test]
fn whole_response_encoding_and_size_failures_never_return_partial_candidates() {
    for protocol in ALL {
        for (data, ddg, bing) in [
            (
                vec![0xff],
                ddg_parser::DdgParseError::Encoding,
                bing_parser::BingParseError::Encoding,
            ),
            (
                vec![b'x'; 1_048_577],
                ddg_parser::DdgParseError::Limit,
                bing_parser::BingParseError::Limit,
            ),
        ] {
            let result = finish_response(
                protocol,
                "q",
                1,
                TransportResponse {
                    status: 200,
                    body: data,
                },
                || Ok(()),
            );
            let expected = if matches!(protocol, CompatibilityProtocol::Bing) {
                CompatibilityError::Bing(bing)
            } else {
                CompatibilityError::Ddg(ddg)
            };
            assert_eq!(result.expect_err("whole response rejected"), expected);
        }
    }
    let mut data: serde_json::Value =
        serde_json::from_slice(&body(CompatibilityProtocol::InstantAnswer)).expect("JSON");
    data["Answer"] = serde_json::json!("");
    data["Definition"] = serde_json::json!("");
    let result = finish_response(
        CompatibilityProtocol::InstantAnswer,
        "q",
        1,
        TransportResponse {
            status: 200,
            body: serde_json::to_vec(&data).expect("JSON"),
        },
        || Ok(()),
    );
    assert_eq!(
        result.expect_err("empty IA"),
        CompatibilityError::Ddg(ddg_parser::DdgParseError::Empty)
    );
}

#[tokio::test(start_paused = true)]
async fn expired_shared_budget_fails_before_any_request_or_parser_success() {
    let protocol = CompatibilityProtocol::Html;
    let manifest = scope(protocol.destination().expect("endpoint"), false);
    let search = protocol
        .prepare_search(&manifest, "q", 1)
        .expect("prepared");
    let (transport, effects, budget) = transport(manifest);
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(
        search
            .execute(&transport)
            .await
            .expect_err("expired transport"),
        CompatibilityError::Transport(TransportError::Deadline)
    );
    assert_eq!(effects.authority.load(Ordering::Relaxed), 0);
    assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
    assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
    assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
}

#[test]
fn deadline_before_parse_prevents_parser_work_and_after_parse_dominates_success_or_error() {
    for protocol in ALL {
        let mut checks = 0;
        let result = finish_response(
            protocol,
            "q",
            1,
            TransportResponse {
                status: 200,
                body: body(protocol),
            },
            || {
                checks += 1;
                Err(TransportError::Deadline)
            },
        );
        assert_eq!(
            result.expect_err("before"),
            CompatibilityError::Transport(TransportError::Deadline)
        );
        assert_eq!(checks, 1);
        for status in [200, 500] {
            let mut checks = 0;
            let result = finish_response(
                protocol,
                "q",
                1,
                TransportResponse {
                    status,
                    body: body(protocol),
                },
                || {
                    checks += 1;
                    if checks == 1 {
                        Ok(())
                    } else {
                        Err(TransportError::Deadline)
                    }
                },
            );
            assert_eq!(
                result.expect_err("after"),
                CompatibilityError::Transport(TransportError::Deadline)
            );
            assert_eq!(checks, 2, "post check runs even after parser error");
        }
    }
}

#[tokio::test(start_paused = true)]
async fn actual_shared_budget_expiry_is_not_replaced_with_a_fresh_parse_budget() {
    for protocol in ALL {
        let search = prepared(protocol, "q", 1);
        let (transport, effects, budget) =
            transport(scope(protocol.destination().expect("endpoint"), false));
        tokio::time::advance(Duration::from_secs(5)).await;
        let result = finish_response(
            protocol,
            &search.query,
            search.max_results,
            TransportResponse {
                status: 200,
                body: body(protocol),
            },
            || transport.budget.check_live(),
        );
        assert_eq!(
            result.expect_err("expired budget"),
            CompatibilityError::Transport(TransportError::Deadline)
        );
        assert_eq!(effects.authority.load(Ordering::Relaxed), 0);
        assert_eq!(effects.dns.load(Ordering::Relaxed), 0);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
        assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    }
}
