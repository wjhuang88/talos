//! Disposable offline wire experiment for SEARCH-001-D / I294.
//! Inputs are independently invented from Purili's published API schema, not
//! captured search responses. This code exists only in an integration-test
//! binary: no SearchBackend, HTTP request, provider registration or routing.
//! Integer widths, strictness, filtering and bounds are research choices.
#![cfg(feature = "network")]

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};

const SYNTHETIC: &str = include_str!("fixtures/i294_purili_synthetic.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireResponse {
    results: Vec<WireHit>,
    total: i64,
    time_ms: String,
    has_next: bool,
    total_pages: i64,
    page: i64,
    #[serde(default)]
    correction: Option<Value>,
    #[serde(default)]
    query_analysis: Option<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireHit {
    #[serde(rename = "id")]
    _id: String,
    title: String,
    url: String,
    #[serde(rename = "displayUrl")]
    _display_url: String,
    description: String,
    #[serde(default, rename = "favicon")]
    _favicon: Option<String>,
    source: String,
}

#[derive(Deserialize)]
struct ErrorEnvelope {
    #[serde(rename = "error")]
    _error: String,
}

#[derive(Debug, PartialEq)]
struct ObservedHit {
    url: String,
    title: String,
    description: String,
    origin: String,
}

#[derive(Debug, PartialEq)]
enum YieldKind {
    Empty,
    InvalidOnly,
    Usable,
}

#[derive(Debug, PartialEq)]
struct Observation {
    wire_count: usize,
    advertised_total: i64,
    invalid_urls: usize,
    page: i64,
    total_pages: i64,
    has_next: bool,
    time_ms: String,
    correction_present: bool,
    query_analysis_present: bool,
    kind: YieldKind,
    hits: Vec<ObservedHit>,
}

#[derive(Debug, PartialEq)]
enum HttpClass {
    Client(u16),
    Server(u16),
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Search(Observation),
    HttpError(HttpClass),
}

#[derive(Debug, PartialEq)]
enum DecodeFailure {
    InvalidLimit,
    BodyTooLarge,
    InvalidWire,
    UnexpectedStatus(u16),
}

// Bounds an already received body. It does not prove transport buffering,
// redirect policy, deadline propagation, cancellation or endpoint safety.
fn observe_http(
    status: u16,
    body: &[u8],
    max_body_bytes: usize,
    max_results: usize,
) -> Result<Outcome, DecodeFailure> {
    if max_results == 0 {
        return Err(DecodeFailure::InvalidLimit);
    }
    if body.len() > max_body_bytes {
        return Err(DecodeFailure::BodyTooLarge);
    }

    if status == 200 {
        return observe_success(body, max_results).map(Outcome::Search);
    }

    if (400..=599).contains(&status) {
        serde_json::from_slice::<ErrorEnvelope>(body).map_err(|_| DecodeFailure::InvalidWire)?;
        return Ok(Outcome::HttpError(if status < 500 {
            HttpClass::Client(status)
        } else {
            HttpClass::Server(status)
        }));
    }

    Err(DecodeFailure::UnexpectedStatus(status))
}

fn observe_success(body: &[u8], max_results: usize) -> Result<Observation, DecodeFailure> {
    let wire: WireResponse =
        serde_json::from_slice(body).map_err(|_| DecodeFailure::InvalidWire)?;
    let wire_count = wire.results.len();
    let mut invalid_urls = 0;
    let mut hits = Vec::new();

    for hit in wire.results {
        let valid_url = Url::parse(&hit.url).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        });
        if !valid_url {
            invalid_urls += 1;
        } else if hits.len() < max_results {
            hits.push(ObservedHit {
                url: hit.url,
                title: hit.title,
                description: hit.description,
                origin: hit.source,
            });
        }
    }

    let kind = if wire_count == 0 {
        YieldKind::Empty
    } else if hits.is_empty() {
        YieldKind::InvalidOnly
    } else {
        YieldKind::Usable
    };

    Ok(Observation {
        wire_count,
        advertised_total: wire.total,
        invalid_urls,
        page: wire.page,
        total_pages: wire.total_pages,
        has_next: wire.has_next,
        time_ms: wire.time_ms,
        correction_present: wire.correction.is_some(),
        query_analysis_present: wire.query_analysis.is_some(),
        kind,
        hits,
    })
}

fn fixture() -> Value {
    serde_json::from_str(SYNTHETIC).expect("authored fixture is JSON")
}

fn inspect(value: &Value, max_results: usize) -> Result<Observation, DecodeFailure> {
    let body = serde_json::to_vec(value).expect("fixture value serializes");
    match observe_http(200, &body, body.len(), max_results)? {
        Outcome::Search(observation) => Ok(observation),
        Outcome::HttpError(_) => unreachable!("status is 200"),
    }
}

#[test]
fn crawled_and_unknown_sources_survive_without_invented_backend_identity() {
    let observation = inspect(&fixture(), 10).expect("synthetic wire shape");
    assert_eq!(observation.kind, YieldKind::Usable);
    assert_eq!(observation.wire_count, 4);
    assert_eq!(observation.advertised_total, 174);
    assert_eq!(observation.page, 1);
    assert_eq!(observation.total_pages, 18);
    assert!(observation.has_next);
    assert_eq!(observation.time_ms, "0.09");
    assert_eq!(
        observation
            .hits
            .iter()
            .map(|hit| hit.origin.as_str())
            .collect::<Vec<_>>(),
        ["crawled", "future-source", "crawled", "crawled"]
    );
    // A wire source label is provenance evidence, not admission or failure-domain proof.
}

#[test]
fn plain_unicode_text_and_wire_order_are_preserved() {
    let observation = inspect(&fixture(), 2).expect("plain-text fixture");
    assert_eq!(observation.hits[0].title, "独立编造 Ω <b>literal</b>");
    assert_eq!(
        observation.hits[0].description,
        "synthetic snippet — 非真实搜索结果"
    );
    assert_eq!(observation.hits[0].origin, "crawled");
    assert_eq!(observation.hits[1].origin, "future-source");
}

#[test]
fn empty_invalid_only_and_malformed_success_are_distinct() {
    let mut empty = fixture();
    empty["results"] = json!([]);
    empty["total"] = json!(0);
    empty["hasNext"] = json!(false);
    empty["totalPages"] = json!(0);
    assert_eq!(
        inspect(&empty, 3).expect("empty shape").kind,
        YieldKind::Empty
    );

    let mut invalid = fixture();
    for hit in invalid["results"].as_array_mut().expect("fixture array") {
        hit["url"] = json!("relative/path");
    }
    let observation = inspect(&invalid, 3).expect("valid shape, invalid URLs");
    assert_eq!(observation.kind, YieldKind::InvalidOnly);
    assert_eq!(observation.invalid_urls, 4);

    assert_eq!(
        observe_http(200, b"<html>challenge</html>", 100, 3),
        Err(DecodeFailure::InvalidWire)
    );
    assert_eq!(
        observe_http(200, b"{", 100, 3),
        Err(DecodeFailure::InvalidWire)
    );
}

#[test]
fn unsafe_or_relative_urls_do_not_consume_valid_result_slots() {
    for url in [
        "relative/path",
        "//example.invalid/path",
        "mailto:a@example.invalid",
        "javascript:alert(1)",
        "file:///tmp/fixture",
        "ftp://example.invalid/file",
        "https://user@example.invalid/path",
        "https://user:pass@example.invalid/path",
        "https://",
        "http://[broken",
    ] {
        let mut value = fixture();
        value["results"][0]["url"] = json!(url);
        let observation = inspect(&value, 2).expect("URL rejection fixture");
        assert_eq!(observation.invalid_urls, 1, "{url}");
        assert_eq!(observation.hits.len(), 2, "{url}");
        assert_eq!(observation.hits[0].origin, "future-source", "{url}");
    }
    // No result is fetched; endpoint/redirect/SSRF policy remains a separate boundary.
}

#[test]
fn required_fields_and_wrong_types_are_detected_at_both_levels() {
    for field in ["results", "total", "timeMs", "hasNext", "totalPages", "page"] {
        let mut value = fixture();
        value.as_object_mut().expect("fixture object").remove(field);
        assert_eq!(inspect(&value, 3), Err(DecodeFailure::InvalidWire), "{field}");
    }

    for field in ["id", "title", "url", "displayUrl", "description", "source"] {
        for replacement in [None, Some(Value::Null), Some(json!({"wrong": true}))] {
            let mut value = fixture();
            let hit = value["results"][0].as_object_mut().expect("fixture hit");
            if let Some(replacement) = replacement {
                hit.insert(field.into(), replacement);
            } else {
                hit.remove(field);
            }
            assert_eq!(inspect(&value, 3), Err(DecodeFailure::InvalidWire), "{field}");
        }
    }

    for (field, replacement) in [
        ("results", json!({})),
        ("total", json!("174")),
        ("timeMs", json!(0.09)),
        ("hasNext", json!(1)),
        ("totalPages", json!("18")),
        ("page", json!("1")),
    ] {
        let mut value = fixture();
        value[field] = replacement;
        assert_eq!(inspect(&value, 3), Err(DecodeFailure::InvalidWire), "{field}");
    }
}

#[test]
fn optional_metadata_and_unknown_fields_do_not_change_results() {
    let mut value = fixture();
    value["correction"] = json!({
        "type": "showing",
        "original": "invented",
        "corrected": "invented corrected"
    });
    value["queryAnalysis"] = json!({"future": true});
    value["futureField"] = json!([1, 2, 3]);
    value["results"][0]["futureField"] = json!(true);
    let observation = inspect(&value, 2).expect("optional metadata");
    assert!(observation.correction_present);
    assert!(observation.query_analysis_present);
    assert_eq!(observation.hits.len(), 2);

    value
        .as_object_mut()
        .expect("fixture object")
        .remove("correction");
    value
        .as_object_mut()
        .expect("fixture object")
        .remove("queryAnalysis");
    let observation = inspect(&value, 2).expect("missing optional metadata");
    assert!(!observation.correction_present);
    assert!(!observation.query_analysis_present);
}

#[test]
fn advertised_counts_never_control_allocation_or_output_limit() {
    for total in [-1, 0, 999_999_999] {
        let mut value = fixture();
        value["total"] = json!(total);
        value["totalPages"] = json!(999_999_999);
        let observation = inspect(&value, 2).expect("integer pagination metadata");
        assert_eq!(observation.advertised_total, total);
        assert_eq!(observation.wire_count, 4);
        assert_eq!(observation.hits.len(), 2);
    }
}

#[test]
fn documented_http_errors_are_distinct_from_empty_success() {
    let error = br#"{"error":"invented diagnostic"}"#;
    assert_eq!(
        observe_http(400, error, error.len(), 3),
        Ok(Outcome::HttpError(HttpClass::Client(400)))
    );
    assert_eq!(
        observe_http(503, error, error.len(), 3),
        Ok(Outcome::HttpError(HttpClass::Server(503)))
    );
    assert_eq!(
        observe_http(302, error, error.len(), 3),
        Err(DecodeFailure::UnexpectedStatus(302))
    );
    assert_eq!(
        observe_http(503, b"<html>down</html>", 100, 3),
        Err(DecodeFailure::InvalidWire)
    );

    let mut empty = fixture();
    empty["results"] = json!([]);
    let body = serde_json::to_vec(&empty).expect("empty fixture serializes");
    let Outcome::Search(observation) =
        observe_http(200, &body, body.len(), 3).expect("empty success")
    else {
        panic!("200 search response should not be classified as HTTP error");
    };
    assert_eq!(observation.kind, YieldKind::Empty);
}

#[test]
fn body_and_output_bounds_are_explicit_experiment_parameters() {
    let body = SYNTHETIC.as_bytes();
    assert!(observe_http(200, body, body.len(), 1).is_ok());
    assert_eq!(
        observe_http(200, body, body.len() - 1, 1),
        Err(DecodeFailure::BodyTooLarge)
    );
    assert_eq!(
        observe_http(200, body, body.len(), 0),
        Err(DecodeFailure::InvalidLimit)
    );
    let Outcome::Search(observation) =
        observe_http(200, body, body.len(), 1).expect("bounded output")
    else {
        panic!("fixture should be a search response");
    };
    assert_eq!(observation.hits.len(), 1);
    assert_eq!(
        observe_http(200, &[0xff], 1, 1),
        Err(DecodeFailure::InvalidWire)
    );
}
