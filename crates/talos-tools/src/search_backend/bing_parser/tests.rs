//! Talos-authored synthetic HTML, never acquired provider responses.
use super::*;

fn row(href: Option<&str>, title: &str, snippet: Option<&str>) -> String {
    let href = href.map(|u| format!(" href=\"{u}\"")).unwrap_or_default();
    let caption = snippet
        .map(|s| format!("<div class=b_caption><p>{s}</p></div>"))
        .unwrap_or_default();
    format!("<li class=b_algo><h2><a{href}>{title}</a></h2>{caption}</li>")
}

fn document(contents: &str) -> String {
    format!("<html><body><ol>{contents}</ol></body></html>")
}

fn results(html: &str, max: u32) -> Vec<SearchBackendResult> {
    parse(200, html.as_bytes(), max)
        .expect("synthetic response")
        .into_candidates()
}

#[test]
fn selectors_cleanup_title_fallback_and_compound_family_identity_are_preserved() {
    let html = document(
        &[
            row(
                Some("  https://one.invalid/  "),
                "<b>Rust &amp;</b><i> 中 </i>",
                Some(" <b>Snip</b> &nbsp; <i>é</i> "),
            ),
            row(Some("/relative"), "&nbsp;", None),
        ]
        .concat(),
    );
    let rows = results(&html, 20);
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (&*rows[0].title, &*rows[0].url, &*rows[0].snippet),
        ("Rust & 中", "https://one.invalid/", "Snip é")
    );
    assert_eq!(
        (&*rows[1].title, &*rows[1].url, &*rows[1].snippet),
        ("/relative", "/relative", "")
    );
    assert!(
        rows.iter()
            .all(|r| r.backend == SearchBackendId::DuckDuckGo)
    );
}

#[test]
fn only_first_link_and_first_caption_per_result_row_are_used() {
    let bad = "x".repeat(MAX_FIELD_BYTES + 1);
    let html = document(&format!(
        "<h2><a href=outside>outside row</a></h2><li class=other><h2><a href=other>other class</a></h2></li>\
        <li class=b_algo><a href=wrong>outside h2</a><h2><a href=\"\">empty first</a><a href=second>second</a></h2></li>\
        <li class=b_algo><h2><a>missing first</a><a href=second>second</a></h2></li>\
        <li class=b_algo><h2><a href=first>first</a><a href=ignored>{bad}</a></h2>\
        <div class=b_caption><p>first snippet</p><p>{bad}</p></div></li>"
    ));
    let rows = results(&html, 20);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (&*rows[0].title, &*rows[0].url, &*rows[0].snippet),
        ("first", "first", "first snippet")
    );
}

#[test]
fn url_rejection_skips_caption_but_still_checks_title() {
    let bad = "x".repeat(MAX_FIELD_BYTES + 1);
    let html = document(
        &[
            row(None, "missing", Some(&bad)),
            row(Some("   "), "empty", Some(&bad)),
            row(Some(" \tjavascript:void(0) "), "script", Some(&bad)),
            format!("<li class=b_algo><div class=b_caption><p>{bad}</p></div></li>"),
            row(Some("valid"), "good", Some("good")),
        ]
        .concat(),
    );
    assert_eq!(results(&html, 20)[0].url, "valid");
    let html = document(&row(Some(""), &bad, None));
    assert_eq!(
        parse(200, html.as_bytes(), 1).expect_err("title before skip"),
        BingParseError::Limit
    );
}

#[test]
fn trimmed_url_dedup_is_exact_without_case_fragment_or_slash_normalization() {
    let html = document(
        &[
            row(Some(" u "), "first", Some("first snippet")),
            row(Some("u"), "duplicate", Some("duplicate snippet")),
            row(Some("U"), "case", None),
            row(Some("u/"), "slash", None),
            row(Some("u#fragment"), "fragment", None),
        ]
        .concat(),
    );
    let rows = results(&html, 20);
    assert_eq!(
        rows.iter().map(|r| r.url.as_str()).collect::<Vec<_>>(),
        ["u", "U", "u/", "u#fragment"]
    );
    assert_eq!(
        (&*rows[0].title, &*rows[0].snippet),
        ("first", "first snippet")
    );
}

#[test]
fn duplicate_caption_errors_fail_whole_response_but_max_stops_before_later_rows() {
    let first = row(Some("u"), "first", Some("first"));
    for field in 0..2 {
        let bad = "x".repeat(MAX_FIELD_BYTES + 1);
        let html = document(
            &(first.clone()
                + &row(
                    Some(" u "),
                    if field == 0 { &bad } else { "duplicate" },
                    Some(if field == 1 { &bad } else { "duplicate" }),
                )),
        );
        assert_eq!(
            parse(200, html.as_bytes(), 2).expect_err("duplicate checked before dedup"),
            BingParseError::Limit
        );
        assert_eq!(results(&html, 1)[0].title, "first");
    }
}

#[test]
fn raw_candidate_urls_are_not_repaired_resolved_or_admitted() {
    let urls = [
        "https:",
        "Javascript:alert(1)",
        "ftp://host.invalid/",
        "/relative",
        "/l/?uddg=https%3A%2F%2Fa.invalid",
        "//host.invalid/",
        "https://a.invalid/?x=1&amp;amp;y=2",
    ];
    let html = document(
        &urls
            .iter()
            .map(|u| row(Some(u), "", None))
            .collect::<String>(),
    );
    let rows = results(&html, 20);
    let expected = [
        "https:",
        "Javascript:alert(1)",
        "ftp://host.invalid/",
        "/relative",
        "/l/?uddg=https%3A%2F%2Fa.invalid",
        "//host.invalid/",
        "https://a.invalid/?x=1&amp;y=2",
    ];
    assert_eq!(
        rows.iter().map(|r| r.url.as_str()).collect::<Vec<_>>(),
        expected
    );
    assert!(rows.iter().all(|r| r.title == r.url));
}

#[test]
fn status_and_raw_challenge_markers_keep_upstream_classification() {
    let valid = document(&row(Some("u"), "ok", None));
    for marker in [
        "g-recaptcha",
        "cf-challenge",
        "verifyContainer",
        "bnp_dialog",
    ] {
        for body in [
            format!("{valid}<!--{marker}-->"),
            format!("{valid}<script>'{marker}'</script>"),
            format!("{valid}<p>{marker}</p>"),
        ] {
            assert_eq!(
                parse(200, body.as_bytes(), 1).expect_err("raw marker"),
                BingParseError::Challenge
            );
            for status in [429, 503] {
                assert_eq!(
                    parse(status, body.as_bytes(), 1).expect_err("rate status first"),
                    BingParseError::RateLimited
                );
            }
            for status in [202, 301, 403, 500] {
                assert_eq!(
                    parse(status, body.as_bytes(), 1).expect_err("status first"),
                    BingParseError::Status
                );
            }
        }
        assert_eq!(
            results(&(valid.clone() + &marker.to_uppercase()), 1).len(),
            1
        );
    }
    assert_eq!(results(&(valid + "<!--challenge-form-->"), 1).len(), 1); // DDG-only marker.
}

#[test]
fn empty_decoy_and_malformed_synthetic_documents_have_closed_outcomes() {
    for body in [
        "",
        "<p>no result</p>",
        "<script src='https://never.invalid/'></script>",
        "<li class=b_algo><a href=u>wrong selector</a></li>",
    ] {
        assert_eq!(
            parse(200, body.as_bytes(), 1).expect_err("empty"),
            BingParseError::Empty
        );
    }
    let malformed = "<li class=b_algo><h2><a href=u>Hello<b>world";
    assert_eq!(results(malformed, 1)[0].title, "Hello world");
}

#[test]
fn input_and_requested_count_bounds_are_checked_before_dom_work() {
    let mut html = document(&row(Some("u"), "ok", None));
    for max in [0, 21, u32::MAX] {
        assert_eq!(
            parse(200, html.as_bytes(), max).expect_err("count"),
            BingParseError::Limit
        );
    }
    html.push_str(&" ".repeat(MAX_BODY_BYTES - html.len()));
    assert_eq!(results(&html, 1).len(), 1);
    html.push(' ');
    for status in [200, 503] {
        assert_eq!(
            parse(status, html.as_bytes(), 1).expect_err("body before status"),
            BingParseError::Limit
        );
    }
}

#[test]
fn field_limits_reject_whole_response_including_raw_href_before_trim() {
    for field in 0..3 {
        let make = |value: &str| {
            row(
                Some(if field == 0 { value } else { "u" }),
                if field == 1 { value } else { "title" },
                Some(if field == 2 { value } else { "snippet" }),
            )
        };
        assert_eq!(
            results(&document(&make(&"x".repeat(MAX_FIELD_BYTES))), 1).len(),
            1
        );
        let html = document(
            &(row(Some("first"), "first", None) + &make(&"x".repeat(MAX_FIELD_BYTES + 1))),
        );
        assert_eq!(
            parse(200, html.as_bytes(), 2).expect_err("whole response"),
            BingParseError::Limit
        );
        assert_eq!(results(&html, 1).len(), 1);
    }
    let exact = " ".repeat(MAX_FIELD_BYTES - 1) + "u";
    assert_eq!(
        results(&document(&row(Some(&exact), "", None)), 1)[0].url,
        "u"
    );
    let html = document(&row(Some(&(exact + " ")), "", None));
    assert_eq!(
        parse(200, html.as_bytes(), 1).expect_err("raw before trim"),
        BingParseError::Limit
    );
    let html = document(&row(Some("u"), &" ".repeat(MAX_FIELD_BYTES + 1), None));
    assert_eq!(
        parse(200, html.as_bytes(), 1).expect_err("joined before cleanup"),
        BingParseError::Limit
    );
}

#[test]
fn joined_text_separators_and_utf8_bytes_count_toward_field_limits() {
    for snippet_field in [false, true] {
        let exact = format!("<b>{}</b><i>{}</i>", "a".repeat(8192), "b".repeat(8191));
        let make = |s: &str| {
            document(&row(
                Some("u"),
                if snippet_field { "title" } else { s },
                Some(if snippet_field { s } else { "snippet" }),
            ))
        };
        assert_eq!(results(&make(&exact), 1).len(), 1);
        assert_eq!(
            parse(200, make(&(exact + "b")).as_bytes(), 1).expect_err("separator count"),
            BingParseError::Limit
        );
        let unicode = "中".repeat(5461) + "a";
        assert_eq!(results(&make(&unicode), 1).len(), 1);
        assert_eq!(
            parse(200, make(&(unicode + "a")).as_bytes(), 1).expect_err("UTF-8 bytes"),
            BingParseError::Limit
        );
    }
    let html = document(&row(Some("u"), "a<b></b>b", None));
    assert_eq!(results(&html, 1)[0].title, "a b");
}

#[test]
fn dom_and_existing_cleanup_entity_order_remain_characterized() {
    let html = document(&row(
        Some("u"),
        "&lt;em&gt;X&lt;/em&gt;",
        Some("&amp;lt;em&amp;gt;Y&amp;lt;/em&amp;gt; &amp;#46;"),
    ));
    let rows = results(&html, 1);
    assert_eq!(rows[0].title, "X");
    assert_eq!(rows[0].snippet, "<em>Y</em> &#46;");
}

#[test]
fn bounded_generated_unicode_results_preserve_order_for_every_requested_count() {
    let html = document(
        &(0..25)
            .map(|i| {
                row(
                    Some(&format!("url{i}")),
                    &format!("中🙂 {i}"),
                    Some(&format!("é {i}")),
                )
            })
            .collect::<String>(),
    );
    for max in 1..=20 {
        let rows = results(&html, max);
        assert_eq!(rows.len(), max as usize);
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(r.title, format!("中🙂 {i}"));
            assert_eq!(r.url, format!("url{i}"));
            assert_eq!(r.snippet, format!("é {i}"));
        }
    }
}

#[test]
fn encoding_and_debug_errors_are_content_free() {
    for body in [
        b"toy-secret\xff".as_slice(),
        b"\xef\xbb\xbftoy-secret".as_slice(),
    ] {
        assert_eq!(
            parse(200, body, 1).expect_err("encoding"),
            BingParseError::Encoding
        );
        assert_eq!(
            parse(503, body, 1).expect_err("status first"),
            BingParseError::RateLimited
        );
    }
    let html = document(&row(
        Some("https://toy-secret.invalid/"),
        "toy-secret",
        Some("toy-secret"),
    ));
    assert_eq!(
        format!("{:?}", parse(200, html.as_bytes(), 1)),
        "Ok(BingCandidates(<redacted>))"
    );
    for e in [
        BingParseError::Limit,
        BingParseError::Encoding,
        BingParseError::Status,
        BingParseError::RateLimited,
        BingParseError::Challenge,
        BingParseError::Parse,
        BingParseError::Empty,
    ] {
        assert!(e.to_string().starts_with("search_bing_response_"));
        assert!(!format!("{e:?} {e}").contains("toy-secret"));
    }
}
