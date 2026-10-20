use super::*;

fn hit(url: &str) -> SearchBackendResult {
    SearchBackendResult {
        backend: SearchBackendId::DuckDuckGo,
        title: "合成标题 <b>inert</b>".into(),
        url: url.into(),
        snippet: "synthetic display text".into(),
    }
}

fn observe(
    candidates: Vec<SearchBackendResult>,
    max: u32,
) -> Result<AdmittedCompatibilityResults, CompatibilityError> {
    admit(SearchBackendId::DuckDuckGo, candidates, max, || Ok(()))
}

#[test]
fn empty_and_invalid_only_responses_are_distinct_non_winners() {
    assert_eq!(
        observe(vec![], 1).expect_err("empty"),
        CompatibilityError::Admission(AdmissionError::Empty)
    );
    for url in [
        "",
        "/relative",
        "//example.invalid/path",
        "not a URL",
        "https://",
        "https://[invalid",
        "file:///tmp/secret",
        "javascript:void(0)",
        "mailto:person@example.invalid",
        "https://user@example.invalid/",
        "https://:password@example.invalid/",
        "https://user:password@example.invalid/",
    ] {
        let error = observe(vec![hit(url)], 1).expect_err("invalid URL");
        assert_eq!(
            error,
            CompatibilityError::Admission(AdmissionError::Invalid)
        );
        assert!(!format!("{error:?} {error}").contains("password"));
    }
}

#[test]
fn mixed_response_filters_before_count_and_preserves_identity_order_and_fields() {
    let first = hit("http://example.invalid/first");
    let second = hit("https://example.invalid/第二?q=fixture#fragment");
    let admitted = observe(
        vec![
            hit("file:///invalid"),
            first.clone(),
            hit(""),
            second.clone(),
            hit("https://last.invalid/"),
        ],
        2,
    )
    .expect("mixed response");
    assert_eq!(
        format!("{admitted:?}"),
        "AdmittedCompatibilityResults(<redacted>)"
    );
    assert_eq!(admitted.into_results(), vec![first, second]);
}

#[test]
fn whatwg_display_acceptance_does_not_claim_raw_syntax_or_fetch_safety() {
    for raw in [
        "https:example.invalid/path",
        "https:///example.invalid/path",
        "https://example.invalid/\npath",
        "https://127.0.0.1/private",
        "http://[::1]/",
    ] {
        assert_eq!(
            observe(vec![hit(raw)], 1)
                .expect("display URL")
                .into_results()[0],
            hit(raw)
        );
    }
}

#[test]
fn count_and_candidate_bounds_are_checked_before_selection() {
    for max in [0, 21, u32::MAX] {
        assert_eq!(
            observe(vec![hit("https://example.invalid/")], max).expect_err("count"),
            CompatibilityError::Transport(TransportError::InvalidRequest)
        );
    }
    assert_eq!(
        observe(vec![hit("https://example.invalid/"); 20], 20)
            .expect("bound")
            .into_results()
            .len(),
        20
    );
    assert_eq!(
        observe(vec![hit("https://example.invalid/"); 21], 1).expect_err("input bound"),
        CompatibilityError::Admission(AdmissionError::Limit)
    );
}

#[test]
fn trailing_identity_and_field_violations_reject_the_whole_response() {
    let first = hit("https://first.invalid/");
    let mut other = first.clone();
    other.backend = SearchBackendId::Tavily;
    assert_eq!(
        observe(vec![first.clone(), other], 1).expect_err("trailing identity"),
        CompatibilityError::Admission(AdmissionError::Identity)
    );
    let mut boundary = hit(&format!(
        "https://example.invalid/{}",
        "x".repeat(8192 - "https://example.invalid/".len())
    ));
    boundary.title = "x".repeat(16_384);
    boundary.snippet = "x".repeat(16_384);
    assert_eq!(
        observe(vec![boundary.clone()], 1)
            .expect("exact bounds")
            .into_results(),
        vec![boundary.clone()]
    );
    for field in [0, 1, 2] {
        let mut oversized = boundary.clone();
        match field {
            0 => oversized.title.push('x'),
            1 => oversized.snippet.push('x'),
            _ => oversized.url.push('x'),
        }
        assert_eq!(
            observe(vec![first.clone(), oversized], 1).expect_err("trailing field bound"),
            CompatibilityError::Admission(AdmissionError::Limit)
        );
    }
}

#[test]
fn shared_deadline_rejects_expired_work_and_a_partial_winner() {
    assert_eq!(
        admit(SearchBackendId::DuckDuckGo, vec![], 0, || Err(
            TransportError::Deadline
        ))
        .expect_err("deadline dominates validation"),
        CompatibilityError::Transport(TransportError::Deadline)
    );
    // Entry + whole-response checks, then pre/post first URL; expiry before the second
    // candidate must discard the already accepted first result.
    let mut checks = 0;
    let result = admit(
        SearchBackendId::DuckDuckGo,
        vec![
            hit("https://first.invalid/"),
            hit("https://second.invalid/"),
        ],
        2,
        || {
            checks += 1;
            if checks == 8 {
                Err(TransportError::Deadline)
            } else {
                Ok(())
            }
        },
    );
    assert_eq!(checks, 8);
    assert_eq!(
        result.expect_err("partial result rejected"),
        CompatibilityError::Transport(TransportError::Deadline)
    );
}
