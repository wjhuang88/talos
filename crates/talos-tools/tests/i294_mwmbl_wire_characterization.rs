//! Disposable offline wire experiment for SEARCH-001-D / I294.
//! Inputs are independently invented, not captured search responses. This parser
//! lives only in the integration-test binary: no SearchBackend, requests or wiring.
//! Integer widths, filtering and limits are research choices, not production policy.
#![cfg(feature = "network")]

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};

const SYNTHETIC: &str = include_str!("fixtures/i294_mwmbl_synthetic.json");

#[derive(Deserialize)]
struct WireResponse {
    #[serde(rename = "query")]
    _query: String,
    number_of_results: i64,
    results: Vec<WireHit>,
    monthly_usage: Option<i64>,
    monthly_limit: Option<i64>,
}

#[derive(Deserialize)]
struct WireHit {
    url: String,
    title: String,
    content: String,
    engine: String,
    #[serde(rename = "title_highlights")]
    _title_highlights: Vec<String>,
    #[serde(rename = "content_highlights")]
    _content_highlights: Vec<String>,
    #[serde(rename = "score")]
    _score: f64,
}

#[derive(Debug, PartialEq)]
struct ObservedHit {
    url: String,
    title: String,
    content: String,
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
    count_matches: bool,
    invalid_urls: usize,
    usage: (Option<i64>, Option<i64>),
    kind: YieldKind,
    hits: Vec<ObservedHit>,
}

#[derive(Debug, PartialEq)]
enum DecodeFailure {
    InvalidLimit,
    BodyTooLarge,
    InvalidWire,
}

// Bounds an already received body, not transport buffering/time/cancellation.
fn observe(
    body: &[u8],
    max_body_bytes: usize,
    max_results: usize,
) -> Result<Observation, DecodeFailure> {
    if max_results == 0 {
        return Err(DecodeFailure::InvalidLimit);
    }
    if body.len() > max_body_bytes {
        return Err(DecodeFailure::BodyTooLarge);
    }
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
                content: hit.content,
                origin: hit.engine,
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
        count_matches: usize::try_from(wire.number_of_results).ok() == Some(wire_count),
        invalid_urls,
        usage: (wire.monthly_usage, wire.monthly_limit),
        kind,
        hits,
    })
}

fn fixture() -> Value {
    serde_json::from_str(SYNTHETIC).expect("authored fixture is JSON")
}
fn inspect(value: &Value, max_results: usize) -> Result<Observation, DecodeFailure> {
    let body = serde_json::to_vec(value).expect("fixture value serializes");
    observe(&body, body.len(), max_results)
}

#[test]
fn mixed_origins_and_unknown_label_survive_without_invented_backend_identity() {
    let observation = inspect(&fixture(), 10).expect("synthetic wire shape");
    assert_eq!(observation.kind, YieldKind::Usable);
    assert_eq!(observation.wire_count, 6);
    assert!(observation.count_matches);
    assert_eq!(observation.usage, (None, None));
    assert_eq!(
        observation
            .hits
            .iter()
            .map(|hit| hit.origin.as_str())
            .collect::<Vec<_>>(),
        [
            "mwmbl",
            "wikipedia",
            "google",
            "user",
            "eusp",
            "future-engine"
        ]
    );
    // Labels grant no native-route credit or proof of a live upstream query.
}

#[test]
fn wikipedia_only_and_unknown_only_are_preserved_as_wire_origins() {
    for index in [1, 5] {
        let mut value = fixture();
        value["results"] = json!([value["results"][index].clone()]);
        value["number_of_results"] = json!(1);
        let observation = inspect(&value, 1).expect("single-origin fixture");
        assert_eq!(observation.hits[0].origin, value["results"][0]["engine"]);
        assert_ne!(observation.hits[0].origin, "mwmbl");
    }
}

#[test]
fn plain_unicode_text_and_wire_order_survive_highlights_and_scores() {
    let mut value = fixture();
    value["results"][0]["score"] = json!(0.01);
    value["results"][1]["score"] = json!(999.0);
    value["results"][0]["title_highlights"] = json!(["replacement"]);
    let observation = inspect(&value, 2).expect("plain-text fixture");
    assert_eq!(observation.hits[0].title, "独立编造 Ω <b>literal</b>");
    assert_eq!(
        observation.hits[0].content,
        "synthetic snippet — 非真实搜索结果"
    );
    assert_eq!(observation.hits[0].origin, "mwmbl");
    assert_eq!(observation.hits[1].origin, "wikipedia");
}

#[test]
fn empty_wire_invalid_urls_and_malformed_wire_are_different_outcomes() {
    let mut empty = fixture();
    empty["results"] = json!([]);
    empty["number_of_results"] = json!(0);
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
    assert_eq!(observation.invalid_urls, 6);
    assert_eq!(
        observe(b"<html>challenge</html>", 100, 3),
        Err(DecodeFailure::InvalidWire)
    );
    assert_eq!(observe(b"{", 100, 3), Err(DecodeFailure::InvalidWire));
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
        assert_eq!(observation.hits[0].origin, "wikipedia", "{url}");
    }
    // No result is fetched; endpoint/redirect/SSRF policy is a separate boundary.
}

#[test]
fn required_fields_and_wrong_types_are_detected_at_both_levels() {
    for field in ["query", "number_of_results", "results"] {
        let mut value = fixture();
        value.as_object_mut().expect("fixture object").remove(field);
        assert_eq!(
            inspect(&value, 3),
            Err(DecodeFailure::InvalidWire),
            "{field}"
        );
    }
    for field in [
        "url",
        "title",
        "content",
        "engine",
        "title_highlights",
        "content_highlights",
        "score",
    ] {
        for replacement in [None, Some(Value::Null), Some(json!({"wrong": true}))] {
            let mut value = fixture();
            let hit = value["results"][0].as_object_mut().expect("fixture hit");
            if let Some(replacement) = replacement {
                hit.insert(field.into(), replacement);
            } else {
                hit.remove(field);
            }
            assert_eq!(
                inspect(&value, 3),
                Err(DecodeFailure::InvalidWire),
                "{field}"
            );
        }
    }
    for (field, replacement) in [
        ("query", json!(7)),
        ("number_of_results", json!("6")),
        ("results", json!({})),
    ] {
        let mut value = fixture();
        value[field] = replacement;
        assert_eq!(
            inspect(&value, 3),
            Err(DecodeFailure::InvalidWire),
            "{field}"
        );
    }
}

#[test]
fn advertised_count_never_controls_allocation_or_result_limit() {
    for count in [-1, 0, 999_999_999] {
        let mut value = fixture();
        value["number_of_results"] = json!(count);
        let observation = inspect(&value, 2).expect("integer count shape");
        assert!(!observation.count_matches);
        assert_eq!(observation.wire_count, 6);
        assert_eq!(observation.hits.len(), 2);
    }
}

#[test]
fn usage_is_optional_metadata_and_extra_fields_do_not_change_results() {
    let mut value = fixture();
    value
        .as_object_mut()
        .expect("fixture object")
        .remove("monthly_usage");
    value
        .as_object_mut()
        .expect("fixture object")
        .remove("monthly_limit");
    value["future_field"] = json!(true);
    value["results"][0]["future_field"] = json!([1, 2]);
    assert_eq!(
        inspect(&value, 2).expect("optional usage").usage,
        (None, None)
    );
    value["monthly_usage"] = json!(4);
    value["monthly_limit"] = json!(20);
    let observation = inspect(&value, 2).expect("integer usage");
    assert_eq!(observation.usage, (Some(4), Some(20)));
    assert_eq!(observation.hits.len(), 2);
    value["monthly_usage"] = json!("4");
    assert_eq!(inspect(&value, 2), Err(DecodeFailure::InvalidWire));
}

#[test]
fn body_and_output_bounds_are_explicit_experiment_parameters() {
    let body = SYNTHETIC.as_bytes();
    assert!(observe(body, body.len(), 1).is_ok());
    assert_eq!(
        observe(body, body.len() - 1, 1),
        Err(DecodeFailure::BodyTooLarge)
    );
    assert_eq!(
        observe(body, body.len(), 0),
        Err(DecodeFailure::InvalidLimit)
    );
    assert_eq!(
        observe(body, body.len(), 1)
            .expect("bounded output")
            .hits
            .len(),
        1
    );
    assert_eq!(observe(&[0xff], 1, 1), Err(DecodeFailure::InvalidWire));
}
