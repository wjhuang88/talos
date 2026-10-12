//! Disposable offline C result-admission experiment; production never calls it.
//! Already-normalized synthetic hits, actual URL parsing, and an injected clock.
//! Limits and terminal ordering are fixture choices, not accepted shipping policy.
//! No HTTP, DNS, permission admission, router wiring or parser preemption evidence.
#![cfg(feature = "network")]

use reqwest::Url;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Hit {
    title: String,
    url: String,
    snippet: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Yield {
    backend: &'static str,
    hits: Vec<Hit>,
}

#[derive(Debug, PartialEq, Eq)]
enum Failure {
    InvalidLimits,
    InputTooLarge,
    NoResults,
    InvalidResults,
    Deadline,
}

// Bounds work on an already-materialized response, NOT transport/body allocation.
// URL acceptance deliberately uses WHATWG parsing, not strict raw URL syntax.
fn admit(
    backend: &'static str,
    hits: Vec<Hit>,
    max_results: usize,
    deadline: Instant,
    mut now: impl FnMut() -> Instant,
) -> Result<Yield, Failure> {
    if now() >= deadline {
        return Err(Failure::Deadline);
    }
    if !(1..=3).contains(&max_results) {
        return Err(Failure::InvalidLimits);
    }
    if hits.len() > 8 {
        return Err(Failure::InputTooLarge);
    }
    if hits.is_empty() {
        return Err(Failure::NoResults);
    }
    let mut accepted = Vec::new();
    for hit in hits {
        if now() >= deadline {
            return Err(Failure::Deadline);
        }
        let valid = hit.url.len() <= 256
            && Url::parse(&hit.url).is_ok_and(|url| {
                matches!(url.scheme(), "http" | "https")
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
            });
        // A parse that crosses the deadline cannot authorize a winner.
        // This cannot interrupt synchronous parsing while it is running.
        if now() >= deadline {
            return Err(Failure::Deadline);
        }
        if valid {
            accepted.push(hit);
            if accepted.len() == max_results {
                break;
            }
        }
    }
    if accepted.is_empty() {
        Err(Failure::InvalidResults)
    } else {
        Ok(Yield {
            backend,
            hits: accepted,
        })
    }
}

fn hit(url: &str) -> Hit {
    Hit {
        title: "合成标题 <b>inert</b>".into(),
        url: url.into(),
        snippet: "independently invented fixture".into(),
    }
}

fn observe(hits: Vec<Hit>, max_results: usize) -> Result<Yield, Failure> {
    let start = Instant::now();
    admit(
        "synthetic-a",
        hits,
        max_results,
        start + Duration::from_secs(1),
        || start,
    )
}

#[test]
fn empty_and_invalid_only_are_distinct_non_winners() {
    assert_eq!(observe(Vec::new(), 1), Err(Failure::NoResults));
    assert_eq!(observe(vec![hit("")], 1), Err(Failure::InvalidResults));
}

#[test]
fn url_parser_rejects_relative_unsupported_malformed_and_credentials() {
    for url in [
        "",
        "/relative",
        "//example.invalid/path",
        "not a URL",
        "mailto:person@example.invalid",
        "file:///tmp/fixture",
        "javascript:void(0)",
        "https://[invalid",
        "https://",
        "https://user@example.invalid/",
        "https://:password@example.invalid/",
        "https://user:password@example.invalid/",
    ] {
        assert_eq!(
            observe(vec![hit(url)], 1),
            Err(Failure::InvalidResults),
            "{url}"
        );
    }
}

#[test]
fn mixed_response_filters_before_truncation_preserving_identity_order_and_fields() {
    let first = hit("http://example.invalid/first");
    let second = hit("https://example.invalid/第二?q=fixture#fragment");
    let observed = observe(
        vec![
            hit("file:///invalid"),
            first.clone(),
            hit(""),
            second.clone(),
            hit("https://example.invalid/third"),
        ],
        2,
    )
    .expect("usable mixed response");
    assert_eq!(
        observed,
        Yield {
            backend: "synthetic-a",
            hits: vec![first, second]
        }
    );
}

#[test]
fn parsed_url_acceptance_is_not_raw_syntax_or_fetch_safety() {
    // WHATWG parsing repairs these strings; returning the original display text
    // is intentional here, and must not be promoted to a safe-to-fetch claim.
    for raw in [
        "https:example.invalid/path",
        "https:///example.invalid/path",
        "https://example.invalid/\npath",
        "https://127.0.0.1/private",
        "http://[::1]/",
    ] {
        let observed = observe(vec![hit(raw)], 1).expect("parser-admissible fixture");
        assert_eq!(observed.hits[0].url, raw);
    }
}

#[test]
fn fixture_limits_bound_input_scan_url_bytes_and_output() {
    assert_eq!(
        observe(vec![hit("https://example.invalid/")], 0),
        Err(Failure::InvalidLimits)
    );
    assert_eq!(
        observe(vec![hit("https://example.invalid/")], 4),
        Err(Failure::InvalidLimits)
    );
    assert_eq!(observe(vec![hit(""); 9], 1), Err(Failure::InputTooLarge));
    assert_eq!(observe(vec![hit(""); 8], 1), Err(Failure::InvalidResults));
    assert_eq!(
        observe(vec![hit("https://example.invalid/"); 8], 3)
            .expect("fixture maximum output")
            .hits
            .len(),
        3
    );
    let prefix = "https://example.invalid/";
    let boundary = format!("{prefix}{}", "x".repeat(256 - prefix.len()));
    assert!(observe(vec![hit(&boundary)], 1).is_ok());
    assert_eq!(
        observe(vec![hit(&format!("{boundary}x"))], 1),
        Err(Failure::InvalidResults)
    );
    let start = Instant::now();
    let mut checks = 0;
    let output = admit(
        "synthetic-b",
        vec![hit(""), hit("https://example.invalid/"), hit("")],
        1,
        start + Duration::from_secs(1),
        || {
            checks += 1;
            start
        },
    )
    .expect("bounded output");
    assert_eq!(output.backend, "synthetic-b");
    assert_eq!(output.hits.len(), 1);
    assert_eq!(
        checks, 5,
        "initial check plus pre/post checks for two scanned hits"
    );
}

#[test]
fn expired_budget_prevents_admission_even_for_an_empty_response() {
    let start = Instant::now();
    assert_eq!(
        admit("synthetic-a", Vec::new(), 1, start, || start),
        Err(Failure::Deadline)
    );
}

#[test]
fn deadline_crossed_during_last_parse_prevents_success() {
    let start = Instant::now();
    let mut ticks = [0, 0, 10].into_iter();
    let observed = admit(
        "synthetic-a",
        vec![hit("https://example.invalid/")],
        1,
        start + Duration::from_millis(10),
        || start + Duration::from_millis(ticks.next().expect("bounded checks")),
    );
    assert_eq!(observed, Err(Failure::Deadline));
    assert_eq!(ticks.next(), None);
}

#[test]
fn budget_does_not_reset_between_entries_or_return_a_partial_winner_on_expiry() {
    let start = Instant::now();
    let mut ticks = [0, 0, 1, 10].into_iter();
    let observed = admit(
        "synthetic-a",
        vec![hit("https://example.invalid/"), hit("")],
        2,
        start + Duration::from_millis(10),
        || start + Duration::from_millis(ticks.next().expect("bounded checks")),
    );
    assert_eq!(observed, Err(Failure::Deadline));
    assert_eq!(ticks.next(), None);
}
