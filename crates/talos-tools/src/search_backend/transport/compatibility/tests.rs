use super::*;
use crate::search_backend::transport::{SearchDns, SearchTransport, TransportBudget};
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

const ALL: [CompatibilityProtocol; 4] = [
    CompatibilityProtocol::Lite,
    CompatibilityProtocol::Html,
    CompatibilityProtocol::InstantAnswer,
    CompatibilityProtocol::Bing,
];

fn manifest(endpoints: Vec<ToolNetworkEndpoint>) -> ToolNetworkManifest {
    ToolNetworkManifest::new(
        ToolNetworkScope::new_configuration(endpoints, None, vec![], 0).expect("scope"),
    )
}

fn prepared(protocol: CompatibilityProtocol, query: &str) -> PreparedTransportRequest {
    protocol
        .prepare(
            &manifest(vec![protocol.destination().expect("endpoint")]),
            query,
        )
        .expect("prepared")
}

fn encoded(request: &PreparedTransportRequest) -> &str {
    if let Some(body) = &request.body {
        std::str::from_utf8(body).expect("ASCII")
    } else {
        request.url.query().expect("query")
    }
}

fn decoded(request: &PreparedTransportRequest) -> Vec<(String, String)> {
    let mut container = Url::parse("https://decode.invalid/").expect("container");
    container.set_query(Some(encoded(request)));
    container.query_pairs().into_owned().collect()
}

#[test]
fn fixed_destinations_come_from_actual_urls_and_methods_without_query() {
    for (protocol, url, method) in [
        (
            CompatibilityProtocol::Lite,
            "https://lite.duckduckgo.com/lite/",
            Method::POST,
        ),
        (
            CompatibilityProtocol::Html,
            "https://html.duckduckgo.com/html/",
            Method::GET,
        ),
        (
            CompatibilityProtocol::InstantAnswer,
            "https://api.duckduckgo.com/",
            Method::GET,
        ),
        (
            CompatibilityProtocol::Bing,
            "https://cn.bing.com/search",
            Method::GET,
        ),
    ] {
        let request = prepared(protocol, "toy-secret");
        assert_eq!(request.method, method);
        let endpoint = request_endpoint(&Url::parse(url).expect("url"), &method).expect("endpoint");
        assert_eq!(protocol.destination().expect("destination"), endpoint);
        assert_eq!(
            request_endpoint(&request.url, &request.method).expect("actual"),
            endpoint
        );
        assert!(!format!("{endpoint:?}").contains("toy-secret"));
        assert_eq!(request.body.is_some(), method == Method::POST);
        assert_eq!(request.url.query().is_some(), method == Method::GET);
        assert_eq!(request.url.fragment(), None);
        assert_eq!(
            request.format.content_type(),
            "application/x-www-form-urlencoded"
        );
    }
}

#[test]
fn ordered_query_and_form_bytes_match_fixed_protocol_goldens() {
    let q = "q=+%E4%B8%AD+%2B%25%26%3D%23%3F*%7E%0D%0A";
    for (protocol, suffix) in [
        (CompatibilityProtocol::Lite, "&kl=wt-wt"),
        (CompatibilityProtocol::Html, ""),
        (
            CompatibilityProtocol::InstantAnswer,
            "&format=json&no_html=1&skip_disambig=1&t=rust-agent-websearch",
        ),
        (CompatibilityProtocol::Bing, "&mkt=zh-CN"),
    ] {
        let request = prepared(protocol, " 中 +%&=#?*~\r\n");
        assert_eq!(encoded(&request), format!("{q}{suffix}"));
        assert_eq!(
            decoded(&request)[0],
            ("q".into(), " 中 +%&=#?*~\r\n".into())
        );
    }
}

#[test]
fn query_cannot_inject_fixed_fields_destination_or_fragment() {
    let query = "&kl=evil&mkt=evil&format=xml#https://evil.invalid/?q=x";
    for protocol in ALL {
        let request = prepared(protocol, query);
        let pairs = decoded(&request);
        assert_eq!(pairs[0], ("q".into(), query.into()));
        assert_eq!(pairs.iter().filter(|(k, _)| k == "q").count(), 1);
        assert_eq!(
            pairs.len(),
            match protocol {
                CompatibilityProtocol::Lite | CompatibilityProtocol::Bing => 2,
                CompatibilityProtocol::Html => 1,
                CompatibilityProtocol::InstantAnswer => 5,
            }
        );
        assert_ne!(request.url.host_str(), Some("evil.invalid"));
        assert_eq!(request.url.fragment(), None);
    }
}

#[test]
fn blank_queries_are_rejected_without_trimming_nonblank_input() {
    for protocol in ALL {
        let m = manifest(vec![protocol.destination().expect("endpoint")]);
        for query in ["", " ", "\r\n\t", "\u{2003}\u{3000}"] {
            assert_eq!(
                protocol.prepare(&m, query).expect_err("blank"),
                TransportError::InvalidRequest
            );
        }
        let original = " \ttoy-secret中\r\n ";
        assert_eq!(
            decoded(&protocol.prepare(&m, original).expect("nonblank"))[0].1,
            original
        );
        assert_eq!(
            protocol
                .prepare(&m, &"x".repeat(16_385))
                .expect_err("raw bound"),
            TransportError::InvalidRequest
        );
    }
}

#[test]
fn exact_ascii_limits_include_every_fixed_field_and_separator() {
    for protocol in ALL {
        let m = manifest(vec![protocol.destination().expect("endpoint")]);
        let overhead = encoded(&prepared(protocol, "x")).len() - 1;
        let limit = if matches!(protocol, CompatibilityProtocol::Lite) {
            16_384
        } else {
            4096
        };
        let query = "a".repeat(limit - overhead);
        assert_eq!(
            encoded(&protocol.prepare(&m, &query).expect("exact bound")).len(),
            limit
        );
        assert_eq!(
            protocol.prepare(&m, &(query + "a")).expect_err("one over"),
            TransportError::InvalidRequest
        );
    }
}

#[test]
fn exact_unicode_limits_count_encoded_utf8_bytes_instead_of_characters() {
    for protocol in ALL {
        let m = manifest(vec![protocol.destination().expect("endpoint")]);
        let overhead = encoded(&prepared(protocol, "x")).len() - 1;
        let limit = if matches!(protocol, CompatibilityProtocol::Lite) {
            16_384
        } else {
            4096
        };
        let available = limit - overhead;
        let query = format!(
            "{}{}",
            "中".repeat(available / 9),
            "a".repeat(available % 9)
        );
        assert_eq!(
            encoded(&protocol.prepare(&m, &query).expect("exact UTF-8 bound")).len(),
            limit
        );
        assert_eq!(
            protocol
                .prepare(&m, &(query + "a"))
                .expect_err("one encoded byte over"),
            TransportError::InvalidRequest
        );
    }
}

#[test]
fn generated_unicode_control_and_reserved_queries_round_trip_for_all_protocols() {
    for value in (0..=255)
        .filter_map(char::from_u32)
        .map(|c| format!("{c}中🙂 +%&=#?*~\0"))
    {
        for protocol in ALL {
            assert_eq!(
                decoded(&prepared(protocol, &value))[0],
                ("q".into(), value.clone())
            );
        }
    }
}

#[test]
fn wrong_scheme_host_port_method_or_path_never_matches_scope() {
    for protocol in ALL {
        let (url, method) = protocol.target().expect("target");
        let host = url.host_str().expect("host");
        for (scheme, host, port, method, path) in [
            ("http", host, 443, method.as_str(), url.path()),
            ("https", "evil.invalid", 443, method.as_str(), url.path()),
            ("https", host, 8443, method.as_str(), url.path()),
            ("https", host, 443, "HEAD", url.path()),
            ("https", host, 443, method.as_str(), "/unapproved"),
        ] {
            let endpoint =
                ToolNetworkEndpoint::new(scheme, host, port, method, path).expect("endpoint");
            assert_eq!(
                protocol
                    .prepare(&manifest(vec![endpoint]), "toy-secret")
                    .expect_err("scope"),
                TransportError::Unauthorized
            );
        }
    }
}

#[test]
fn prepared_query_is_owned_and_scope_reuse_does_not_bind_query_or_invocation() {
    for protocol in ALL {
        let first = manifest(vec![protocol.destination().expect("endpoint")]);
        let second = ToolNetworkManifest::new(first.scope().clone());
        let mut query = "toy-secret original".to_owned();
        let request = protocol.prepare(&first, &query).expect("first");
        query.clear();
        query.push_str("replacement");
        let other = protocol
            .prepare(&first, &query)
            .expect("same invocation can prepare again");
        let fresh = protocol
            .prepare(&second, &query)
            .expect("same scope fresh invocation");
        assert_eq!(decoded(&request)[0].1, "toy-secret original");
        assert_eq!(decoded(&other)[0].1, "replacement");
        assert_eq!(first.scope(), second.scope());
        assert_eq!(request.invocation_id, other.invocation_id);
        assert_ne!(request.invocation_id, fresh.invocation_id);
    }
}

#[test]
fn request_and_failure_debug_are_content_free_for_every_protocol() {
    for protocol in ALL {
        let request = prepared(protocol, "toy-secret");
        assert_eq!(
            format!("{request:?}"),
            "PreparedTransportRequest(<redacted>)"
        );
        let wrong = manifest(vec![
            ToolNetworkEndpoint::new("https", "evil.invalid", 443, "GET", "/").expect("endpoint"),
        ]);
        for error in [
            protocol
                .prepare(&wrong, "toy-secret")
                .expect_err("unauthorized"),
            protocol
                .prepare(&wrong, &"toy-secret".repeat(2000))
                .expect_err("too long"),
        ] {
            assert!(!format!("{protocol:?} {error:?} {error}").contains("toy-secret"));
        }
    }
}

struct Effects(AtomicUsize);
impl ToolNetworkAuthority for Effects {
    fn check(
        &self,
        _: uuid::Uuid,
        _: &ToolNetworkEndpoint,
    ) -> Result<(), ToolNetworkAuthorizationError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
#[async_trait]
impl SearchDns for Effects {
    async fn resolve(&self, _: &str, _: u16) -> Result<Vec<SocketAddr>, TransportError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err(TransportError::Dns)
    }
}

#[tokio::test]
async fn preparation_and_cross_invocation_failure_have_zero_execution_effects() {
    for protocol in ALL {
        let first = manifest(vec![protocol.destination().expect("endpoint")]);
        let second = ToolNetworkManifest::new(first.scope().clone());
        let authority = Arc::new(Effects(AtomicUsize::new(0)));
        let dns = Arc::new(Effects(AtomicUsize::new(0)));
        let budget =
            Arc::new(TransportBudget::new(Duration::from_secs(2), 1, 1024).expect("budget"));
        let transport =
            SearchTransport::new(second, authority.clone(), budget.clone(), dns.clone())
                .expect("transport");
        let request = protocol.prepare(&first, "toy-secret").expect("prepared");
        assert_eq!(
            protocol.prepare(&first, " ").expect_err("blank"),
            TransportError::InvalidRequest
        );
        for before_execution in [true, false] {
            if !before_execution {
                assert_eq!(
                    transport
                        .execute_prepared(
                            protocol
                                .prepare(&first, "second")
                                .expect("another owned request")
                        )
                        .await
                        .expect_err("wrong invocation"),
                    TransportError::Unauthorized
                );
            }
            assert_eq!(authority.0.load(Ordering::Relaxed), 0);
            assert_eq!(dns.0.load(Ordering::Relaxed), 0);
            assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
            assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
        }
        assert_eq!(
            transport
                .execute_prepared(request)
                .await
                .expect_err("wrong invocation"),
            TransportError::Unauthorized
        );
        assert_eq!(authority.0.load(Ordering::Relaxed), 0);
        assert_eq!(dns.0.load(Ordering::Relaxed), 0);
        assert_eq!(budget.attempts.load(Ordering::Relaxed), 0);
        assert_eq!(budget.bytes.load(Ordering::Relaxed), 0);
    }
}
