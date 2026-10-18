//! Talos-authored synthetic HTML; no acquired provider response or live query.
use super::*;

const FORMATS: [DdgFormat; 2] = [DdgFormat::Lite, DdgFormat::Html];

fn link(format: DdgFormat, href: Option<&str>, title: &str) -> String {
    let class = match format {
        DdgFormat::Lite => "result-link",
        DdgFormat::Html => "result__a",
    };
    let href = href.map(|h| format!(" href=\"{h}\"")).unwrap_or_default();
    format!("<a class=\"{class}\"{href}>{title}</a>")
}

fn snippet(format: DdgFormat, value: &str) -> String {
    match format {
        DdgFormat::Lite => format!("<tr><td class=\"result-snippet\">{value}</td></tr>"),
        DdgFormat::Html => format!("<a class=\"result__snippet\">{value}</a>"),
    }
}

fn row(format: DdgFormat, href: Option<&str>, title: &str, summary: &str) -> String {
    let a = link(format, href, title);
    match format {
        DdgFormat::Lite => format!("<tr><td>{a}</td></tr>{}", snippet(format, summary)),
        DdgFormat::Html => format!("<div>{a}{}</div>", snippet(format, summary)),
    }
}

fn document(format: DdgFormat, contents: &str) -> String {
    match format {
        DdgFormat::Lite => format!("<html><body><table>{contents}</table></body></html>"),
        DdgFormat::Html => format!("<html><body><div>{contents}</div></body></html>"),
    }
}

fn results(format: DdgFormat, html: &str, max: u32) -> Vec<SearchBackendResult> {
    parse(format, 200, html.as_bytes(), max)
        .expect("synthetic parse")
        .into_candidates()
}

#[test]
fn selectors_nested_text_entities_and_empty_title_follow_compatibility_rules() {
    for format in FORMATS {
        let html = document(
            format,
            &[
                row(
                    format,
                    Some("https://one.invalid/"),
                    " <b>Rust &amp;</b><i> 中 </i>",
                    "<b>Snip</b> &nbsp; <i>two</i>",
                ),
                row(
                    format,
                    Some("//two.invalid/"),
                    "&nbsp;",
                    "&lt;em&gt;X&lt;/em&gt;",
                ),
            ]
            .concat(),
        );
        let rows = results(format, &html, 20);
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|r| r.backend == SearchBackendId::DuckDuckGo)
        );
        assert_eq!(
            (&*rows[0].title, &*rows[0].snippet),
            ("Rust & 中", "Snip two")
        );
        assert_eq!(
            (&*rows[1].title, &*rows[1].url, &*rows[1].snippet),
            ("https://two.invalid/", "https://two.invalid/", "X")
        );
    }
}

#[test]
fn skipped_and_duplicate_links_still_consume_their_original_snippet_positions() {
    for format in FORMATS {
        let html = document(
            format,
            &[
                row(format, None, "no href", "s0"),
                row(format, Some("https:"), "invalid", "s1"),
                row(format, Some("https://a.invalid/"), "first", "s2"),
                row(format, Some("//a.invalid/"), "duplicate", "s3"),
                row(format, Some(""), "empty", "s4"),
                row(format, Some("https://b.invalid/"), "second", "s5"),
            ]
            .concat(),
        );
        let rows = results(format, &html, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!((&*rows[0].title, &*rows[0].snippet), ("first", "s2"));
        assert_eq!((&*rows[1].title, &*rows[1].snippet), ("second", "s5"));
    }
}

#[test]
fn missing_and_excess_snippets_preserve_positional_defaults() {
    for format in FORMATS {
        // Use valid rows, with only one of the two snippet elements present.
        let first = row(format, Some("https://a.invalid/"), "first", "only");
        let second = row(format, Some("https://b.invalid/"), "second", "unused")
            .replace(&snippet(format, "unused"), "");
        let rows = results(format, &document(format, &(first + &second)), 2);
        assert_eq!((&*rows[0].snippet, &*rows[1].snippet), ("only", ""));
        let extra = document(
            format,
            &(row(format, Some("https://a.invalid/"), "one", "kept")
                + &snippet(format, &"x".repeat(MAX_FIELD_BYTES + 1))),
        );
        assert_eq!(results(format, &extra, 1)[0].snippet, "kept");
    }
}

#[test]
fn existing_url_helper_quirks_are_candidates_not_url_admission() {
    let cases = [
        ("//a.invalid/", "https://a.invalid/"),
        (
            "/l/?uddg=https%3A%2F%2Fa.invalid%2Fq+space&rut=abc",
            "https://a.invalid/q space&rut=abc",
        ),
        ("/l/?uddg=%ZZ", "/l/?uddg=%ZZ"),
        ("/l/?uddg=%FF", "/l/?uddg=%FF"),
        ("/l/?uddg=中%41", "A"),
        ("javascript:alert(1)", "javascript:alert(1)"),
        ("/relative", "/relative"),
        (
            "https://a.invalid/?x=1&amp;amp;y=2",
            "https://a.invalid/?x=1&y=2",
        ),
    ];
    for format in FORMATS {
        for (raw, expected) in cases {
            let html = document(format, &row(format, Some(raw), "title", "summary"));
            assert_eq!(results(format, &html, 1)[0].url, expected);
        }
    }
}

#[test]
fn captcha_status_and_body_markers_keep_lite_html_distinctions() {
    for format in FORMATS {
        let html = document(format, &row(format, Some("https://a.invalid/"), "ok", "ok"));
        for marker in ["challenge-form", "g-recaptcha"] {
            let html = format!("{html}<!--{marker}-->");
            assert_eq!(
                parse(format, 200, html.as_bytes(), 1).expect_err("challenge"),
                DdgParseError::Challenge
            );
            assert_eq!(
                parse(format, 403, html.as_bytes(), 1).expect_err("status before marker"),
                DdgParseError::Status
            );
        }
        let cf = format!("{html}<!--cf-challenge-->");
        match format {
            DdgFormat::Lite => assert_eq!(
                parse(format, 200, cf.as_bytes(), 1).expect_err("lite marker"),
                DdgParseError::Challenge
            ),
            DdgFormat::Html => assert_eq!(results(format, &cf, 1).len(), 1),
        }
        let expected = match format {
            DdgFormat::Lite => DdgParseError::Status,
            DdgFormat::Html => DdgParseError::Challenge,
        };
        assert_eq!(
            parse(format, 202, html.as_bytes(), 1).expect_err("202 classification"),
            expected
        );
        assert_eq!(
            results(format, &(html + "<!--CHALLENGE-FORM-->"), 1).len(),
            1
        );
    }
}

#[test]
fn empty_and_malformed_synthetic_documents_have_explicit_outcomes() {
    for format in FORMATS {
        for body in [
            "",
            "<html>no results</html>",
            "<a href='https://a.invalid/'>wrong selector</a>",
            "<script src='https://never.invalid/'></script>",
        ] {
            assert_eq!(
                parse(format, 200, body.as_bytes(), 1).expect_err("empty"),
                DdgParseError::Empty
            );
        }
        let class = match format {
            DdgFormat::Lite => "result-link",
            DdgFormat::Html => "result__a",
        };
        let malformed = format!("<a class={class} href='https://a.invalid/'>x<b> nested");
        assert_eq!(results(format, &malformed, 1)[0].title, "x nested");
    }
}

#[test]
fn input_byte_and_result_limits_are_checked_before_dom_work() {
    for format in FORMATS {
        let mut html = document(format, &row(format, Some("https://a.invalid/"), "ok", "ok"));
        for limit in [0, 21, u32::MAX] {
            assert_eq!(
                parse(format, 200, html.as_bytes(), limit).expect_err("result bound"),
                DdgParseError::Limit
            );
        }
        html.push_str(&" ".repeat(MAX_BODY_BYTES - html.len()));
        assert_eq!(results(format, &html, 1).len(), 1);
        html.push(' ');
        assert_eq!(
            parse(format, 200, html.as_bytes(), 1).expect_err("body bound"),
            DdgParseError::Limit
        );
    }
}

#[test]
fn exact_and_overflow_fields_fail_whole_response_without_partial_success() {
    for format in FORMATS {
        let exact = "x".repeat(MAX_FIELD_BYTES);
        let overflow = exact.clone() + "x";
        for field in 0..3 {
            let make = |value: &str| {
                row(
                    format,
                    Some(if field == 0 {
                        value
                    } else {
                        "https://a.invalid/"
                    }),
                    if field == 1 { value } else { "title" },
                    if field == 2 { value } else { "summary" },
                )
            };
            assert_eq!(
                results(format, &document(format, &make(&exact)), 1).len(),
                1
            );
            let html = document(
                format,
                &(row(format, Some("https://first.invalid/"), "first", "first") + &make(&overflow)),
            );
            assert_eq!(
                parse(format, 200, html.as_bytes(), 2).expect_err("no partial response"),
                DdgParseError::Limit
            );
            assert_eq!(results(format, &html, 1).len(), 1); // Upstream stops after max accepted results.
        }
    }
}

#[test]
fn inserted_text_separators_and_utf8_byte_lengths_are_bounded() {
    for format in FORMATS {
        let title = format!("<b>{}</b><i>{}</i>", "a".repeat(8192), "b".repeat(8191));
        let html = document(format, &row(format, Some("x"), &title, ""));
        assert_eq!(results(format, &html, 1)[0].title.len(), MAX_FIELD_BYTES);
        let bad_title = title.replace("</i>", "b</i>");
        let html = document(format, &row(format, Some("x"), &bad_title, ""));
        assert_eq!(
            parse(format, 200, html.as_bytes(), 1).expect_err("joined bytes"),
            DdgParseError::Limit
        );
        let title = "中".repeat(5461) + "a";
        let html = document(format, &row(format, Some("x"), &title, ""));
        assert_eq!(results(format, &html, 1)[0].title, title);
        let html = document(format, &row(format, Some("x"), &(title + "a"), ""));
        assert_eq!(
            parse(format, 200, html.as_bytes(), 1).expect_err("UTF-8 byte bound"),
            DdgParseError::Limit
        );
    }
}

#[test]
fn helper_expansion_is_bounded_after_url_resolution() {
    for format in FORMATS {
        let raw = "//".to_owned() + &"a".repeat(MAX_FIELD_BYTES - 8);
        let html = document(format, &row(format, Some(&raw), "", ""));
        let rows = results(format, &html, 1);
        assert_eq!(rows[0].url.len(), MAX_FIELD_BYTES);
        assert_eq!(rows[0].title, rows[0].url);
        let raw = raw + "a";
        assert!(raw.len() < MAX_FIELD_BYTES);
        let html = document(format, &row(format, Some(&raw), "", ""));
        assert_eq!(
            parse(format, 200, html.as_bytes(), 1).expect_err("resolved bound"),
            DdgParseError::Limit
        );
    }
}

#[test]
fn encoding_errors_and_debug_are_content_free() {
    for format in FORMATS {
        for body in [
            b"toy-secret\xff".as_slice(),
            b"\xef\xbb\xbftoy-secret".as_slice(),
        ] {
            let error = parse(format, 200, body, 1).expect_err("encoding");
            assert_eq!(error, DdgParseError::Encoding);
            assert!(!format!("{error:?} {error}").contains("toy-secret"));
        }
        let html = document(
            format,
            &row(
                format,
                Some("https://toy-secret.invalid/"),
                "toy-secret",
                "toy-secret",
            ),
        );
        assert_eq!(
            format!("{:?}", parse(format, 200, html.as_bytes(), 1)),
            "Ok(DdgCandidates(<redacted>))"
        );
        let error = parse(format, 500, html.as_bytes(), 1).expect_err("status");
        assert_eq!(error, DdgParseError::Status);
        assert!(!format!("{error:?} {error}").contains("toy-secret"));
    }
}

#[test]
fn bounded_generated_unicode_results_preserve_order_and_requested_count() {
    for format in FORMATS {
        for max in 1..=20 {
            let html = document(
                format,
                &(0..25)
                    .map(|i| {
                        row(
                            format,
                            Some(&format!("https://fixture.invalid/{i}")),
                            &format!("中🙂 {i}"),
                            &format!("é &amp; {i}"),
                        )
                    })
                    .collect::<String>(),
            );
            let rows = results(format, &html, max);
            assert_eq!(rows.len(), max as usize);
            for (i, item) in rows.iter().enumerate() {
                assert_eq!(item.url, format!("https://fixture.invalid/{i}"));
                assert_eq!(item.title, format!("中🙂 {i}"));
                assert_eq!(item.snippet, format!("é & {i}"));
            }
        }
    }
}
