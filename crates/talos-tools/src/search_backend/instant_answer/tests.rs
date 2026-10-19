//! Synthetic JSON only; no acquired provider response or live query.
use super::*;
use serde_json::{Value, json};

fn response() -> Value {
    json!({"AbstractText":"", "AbstractURL":"", "AbstractSource":"", "Heading":"",
        "Answer":"", "AnswerType":"", "Definition":"", "DefinitionSource":"",
        "RelatedTopics":[], "Infobox":null, "Type":""})
}

fn topic(text: &str, url: &str) -> Value {
    json!({"Text":text, "FirstURL":url})
}

fn results(data: &Value, query: &str, max: u32) -> Vec<SearchBackendResult> {
    parse(
        200,
        &serde_json::to_vec(data).expect("fixture JSON"),
        query,
        max,
    )
    .expect("fixture response")
    .into_candidates()
}

fn error(data: &Value, query: &str, max: u32) -> DdgParseError {
    parse(
        200,
        &serde_json::to_vec(data).expect("fixture JSON"),
        query,
        max,
    )
    .expect_err("fixture rejection")
}

#[test]
fn top_level_order_titles_sources_and_cleaning_match_upstream() {
    let mut data = response();
    data["AbstractText"] = json!("<b>abstract</b>&amp;");
    data["Heading"] = json!("<b>raw heading</b>");
    data["AbstractURL"] = json!("https://abstract.invalid/");
    data["Answer"] = json!("<code>answer</code>");
    data["Definition"] = json!("<i>definition</i>");
    data["DefinitionSource"] = json!("source label, not URL");
    let rows = results(&data, "a b", 20);
    assert_eq!(rows.len(), 3);
    assert_eq!(
        (&*rows[0].title, &*rows[0].url, &*rows[0].snippet),
        (
            "<b>raw heading</b>",
            "https://abstract.invalid/",
            "abstract&"
        )
    );
    assert_eq!(
        (&*rows[1].title, &*rows[1].url, &*rows[1].snippet),
        ("Answer: a b", "https://duckduckgo.com/?q=a+b", "answer")
    );
    assert_eq!(
        (&*rows[2].title, &*rows[2].url, &*rows[2].snippet),
        ("Definition: a b", "source label, not URL", "definition")
    );
    assert!(
        rows.iter()
            .all(|r| r.backend == SearchBackendId::DuckDuckGo)
    );
}

#[test]
fn each_top_level_addition_honors_max_and_calc_is_case_sensitive() {
    let mut data = response();
    for key in ["AbstractText", "Answer", "Definition"] {
        data[key] = json!(key);
    }
    for max in 1..=3 {
        assert_eq!(results(&data, "q", max).len(), max as usize);
    }
    data["AnswerType"] = json!("calc");
    let rows = results(&data, "q", 3);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].title, "Definition: q");
    data["AnswerType"] = json!("Calc");
    assert_eq!(results(&data, "q", 3).len(), 3);
}

#[test]
fn limited_cleaning_preserves_unknown_tags_case_and_whitespace() {
    let mut data = response();
    data["AbstractText"] =
        json!(" <b>B</b><i>I</i><code>C</code><pre>P</pre><br><br/><br />  <B>X</B><a>Y</a>&amp; ");
    assert_eq!(
        results(&data, "", 1)[0].snippet,
        " BICP     <B>X</B><a>Y</a>& "
    );
    data["AbstractText"] = json!("&lt;b&gt;X&lt;/b&gt;");
    assert_eq!(results(&data, "", 1)[0].snippet, "<b>X</b>");
}

#[test]
fn fallback_encoding_preserves_utf8_space_plus_star_tilde_and_control_bytes() {
    let mut data = response();
    data["AbstractText"] = json!("abstract");
    let query = "A z-_.~*+/%中🙂\0\n";
    let row = results(&data, query, 1).remove(0);
    assert_eq!(row.title, query);
    assert_eq!(
        row.url,
        "https://duckduckgo.com/?q=A+z-_.~%2A%2B%2F%25%E4%B8%AD%F0%9F%99%82%00%0A"
    );
    assert_eq!(results(&data, "", 1)[0].url, "https://duckduckgo.com/?q=");
}

#[test]
fn required_fields_even_unused_keep_missing_null_and_type_errors() {
    for key in [
        "AbstractText",
        "AbstractURL",
        "AbstractSource",
        "Heading",
        "Answer",
        "AnswerType",
        "Definition",
        "DefinitionSource",
        "Type",
        "RelatedTopics",
    ] {
        for replacement in [None, Some(Value::Null), Some(json!(3))] {
            let mut data = response();
            data["AbstractText"] = json!("valid");
            if let Some(value) = replacement {
                data[key] = value;
            } else {
                data.as_object_mut().expect("object").remove(key);
            }
            assert_eq!(error(&data, "q", 1), DdgParseError::Parse, "{key}");
        }
    }
}

#[test]
fn optional_and_default_fields_preserve_upstream_schema() {
    let mut data = response();
    data["RelatedTopics"] = json!([topic("topic", "relative")]);
    for infobox in [None, Some(Value::Null), Some(json!({}))] {
        if let Some(value) = infobox {
            data["Infobox"] = value;
        } else {
            data.as_object_mut().expect("object").remove("Infobox");
        }
        assert_eq!(results(&data, "q", 1)[0].url, "relative");
    }
    for value in [Value::Null, json!({"URL":""})] {
        data["RelatedTopics"][0]["Icon"] = value;
        data["RelatedTopics"][0]["Result"] = Value::Null;
        assert_eq!(results(&data, "q", 1)[0].snippet, "topic");
    }
    data["RelatedTopics"][0]["Topics"] = Value::Null;
    assert_eq!(error(&data, "q", 1), DdgParseError::Parse);
    data["RelatedTopics"][0]["Topics"] = json!([]);
    for bad in [json!({}), json!({"URL":null}), json!({"URL":4})] {
        data["RelatedTopics"][0]["Icon"] = bad;
        assert_eq!(error(&data, "q", 1), DdgParseError::Parse);
    }
}

#[test]
fn schema_is_validated_before_selection_and_unknown_fields_remain_ignored() {
    let mut data = response();
    data["AbstractText"] = json!("enough for max1");
    data["Unknown"] = json!({"arbitrary":[true, 3]});
    assert_eq!(results(&data, "q", 1).len(), 1);
    for bad in [
        json!({"Name":"group", "Topics":[]}),
        json!({"Text":"x"}),
        json!({"Text":"x", "FirstURL":"u", "Icon":{}}),
    ] {
        data["RelatedTopics"] = json!([bad]);
        assert_eq!(error(&data, "q", 1), DdgParseError::Parse);
    }
    data["RelatedTopics"] = json!([]);
    for bad in [
        json!({"content":[{"label":"x", "value":"y"}]}),
        json!({"content":[{"label":"x", "value":"y", "data_type":null}]}),
        json!({"meta":[{"label":"x"}]}),
        json!({"content":null}),
        json!({"meta":null}),
    ] {
        data["Infobox"] = bad;
        assert_eq!(error(&data, "q", 1), DdgParseError::Parse);
    }
}

#[test]
fn direct_typed_json_rejects_duplicate_known_fields_and_trailing_data() {
    let data = serde_json::to_string(&response()).expect("JSON");
    for extra in [
        r#""AbstractText":"second""#,
        r#""AbstractSource":"second""#,
        r#""RelatedTopics":[]"#,
    ] {
        let duplicate = format!("{{{extra},{}", &data[1..]);
        assert_eq!(
            parse(200, duplicate.as_bytes(), "q", 1).expect_err("duplicate"),
            DdgParseError::Parse
        );
    }
    let trailing = data + "{}";
    assert_eq!(
        parse(200, trailing.as_bytes(), "q", 1).expect_err("trailing"),
        DdgParseError::Parse
    );
}

#[test]
fn topic_depth_first_order_result_preference_and_raw_sentence_split_are_preserved() {
    let mut data = response();
    let mut parent = topic("ignored", "parent");
    parent["Result"] = json!("Parent! summary");
    parent["Topics"] = json!([topic("Child。 summary", "child")]);
    let mut suppressed = topic("must not use Text", "suppressed");
    suppressed["Result"] = json!("");
    suppressed["Topics"] = json!([topic("! empty title", "empty-title")]);
    data["RelatedTopics"] = json!([
        parent,
        suppressed,
        topic("<a href=\"https://host.invalid/\">Link</a> tail", "anchor"),
        topic("A&#46; B", "entity")
    ]);
    let rows = results(&data, "q", 20);
    assert_eq!(
        rows.iter().map(|r| r.url.as_str()).collect::<Vec<_>>(),
        ["parent", "child", "empty-title", "anchor", "entity"]
    );
    assert_eq!(
        rows.iter().map(|r| r.title.as_str()).collect::<Vec<_>>(),
        ["Parent", "Child", "", "<a href=\"https://host", "A&#46; B"]
    );
}

#[test]
fn infobox_meta_precedes_content_without_cleanup_or_url_repair() {
    let mut data = response();
    data["Infobox"] = json!({"meta":[{"label":" <b>label</b> ", "value":"&amp;"}],
        "content":[{"label":"content", "value":"<i>raw</i>", "data_type":"string"}]});
    let rows = results(&data, "q", 20);
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (&*rows[0].title, &*rows[0].snippet),
        (" <b>label</b> ", "&amp;")
    );
    assert_eq!(
        (&*rows[1].title, &*rows[1].snippet),
        ("content", "<i>raw</i>")
    );
    assert!(rows.iter().all(|r| r.url.is_empty()));
}

#[test]
fn duplicates_unsafe_candidates_and_whitespace_keep_legacy_behavior() {
    let mut data = response();
    data["AbstractText"] = json!(" ");
    data["AbstractURL"] = json!("javascript:alert(1)");
    data["RelatedTopics"] = json!([
        topic(" ", "javascript:alert(1)"),
        topic("relative", "/path")
    ]);
    let rows = results(&data, "q", 20);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].url, rows[1].url);
    assert_eq!(rows[1].snippet, " ");
    assert_eq!(rows[2].url, "/path");
}

fn nested(depth: usize) -> Value {
    let mut node = topic("leaf", "leaf");
    for _ in 1..depth {
        let mut parent = topic("", "");
        parent["Topics"] = json!([node]);
        node = parent;
    }
    node
}

#[test]
fn topic_depth_default_json_recursion_and_wide_empty_topics_are_bounded() {
    let mut data = response();
    data["RelatedTopics"] = json!([nested(MAX_TOPIC_DEPTH)]);
    assert_eq!(results(&data, "q", 1)[0].title, "leaf");
    data["RelatedTopics"] = json!([nested(MAX_TOPIC_DEPTH + 1)]);
    assert_eq!(error(&data, "q", 1), DdgParseError::Limit);
    data["AbstractText"] = json!("max already met");
    assert_eq!(results(&data, "q", 1).len(), 1); // Depth guard applies to visited topics.
    data["RelatedTopics"] = json!([nested(70)]);
    assert_eq!(error(&data, "q", 1), DdgParseError::Parse); // Schema is always parsed in full.
    data["AbstractText"] = json!("");
    data["RelatedTopics"] = json!(vec![topic("", ""); 5000]);
    assert_eq!(error(&data, "q", 20), DdgParseError::Empty);
}

#[test]
fn input_query_and_count_limits_have_exact_and_overflow_cases() {
    let mut data = response();
    data["AbstractText"] = json!("valid");
    data["AbstractURL"] = json!("provided");
    let mut raw = serde_json::to_vec(&data).expect("JSON");
    raw.resize(MAX_BODY_BYTES, b' ');
    assert!(parse(200, &raw, "q", 1).is_ok());
    raw.push(b' ');
    assert_eq!(
        parse(200, &raw, "q", 1).expect_err("input"),
        DdgParseError::Limit
    );
    assert_eq!(
        results(&data, &"q".repeat(MAX_FIELD_BYTES), 1)[0]
            .title
            .len(),
        MAX_FIELD_BYTES
    );
    assert_eq!(
        error(&data, &"q".repeat(MAX_FIELD_BYTES + 1), 1),
        DdgParseError::Limit
    );
    for max in [0, 21, u32::MAX] {
        assert_eq!(error(&data, "q", max), DdgParseError::Limit);
    }
}

#[test]
fn selected_field_limits_fail_whole_response_and_max_skips_later_fields() {
    for field in ["Heading", "AbstractURL", "AbstractText"] {
        let mut data = response();
        data["AbstractText"] = json!("valid");
        data[field] = json!("x".repeat(MAX_FIELD_BYTES));
        assert_eq!(results(&data, "q", 1).len(), 1);
        data[field] = json!("x".repeat(MAX_FIELD_BYTES + 1));
        assert_eq!(error(&data, "q", 1), DdgParseError::Limit);
    }
    let mut data = response();
    data["AbstractText"] = json!("valid first");
    data["RelatedTopics"] = json!([topic(&"x".repeat(MAX_FIELD_BYTES + 1), "second")]);
    assert_eq!(error(&data, "q", 2), DdgParseError::Limit);
    assert_eq!(results(&data, "q", 1).len(), 1);
    data["AbstractSource"] = json!("x".repeat(MAX_FIELD_BYTES + 1)); // Unused string has schema/body bounds only.
    assert_eq!(results(&data, "q", 1).len(), 1);
}

#[test]
fn fallback_url_expansion_and_title_prefix_count_toward_limits() {
    let mut data = response();
    data["AbstractText"] = json!("valid");
    let prefix = "https://duckduckgo.com/?q=".len();
    let query = "a".repeat(MAX_FIELD_BYTES - prefix);
    assert_eq!(results(&data, &query, 1)[0].url.len(), MAX_FIELD_BYTES);
    assert_eq!(error(&data, &(query + "a"), 1), DdgParseError::Limit);
    let query = "中".repeat((MAX_FIELD_BYTES - prefix) / 9);
    assert!(results(&data, &query, 1)[0].url.len() <= MAX_FIELD_BYTES);
    assert_eq!(error(&data, &(query + "中"), 1), DdgParseError::Limit);
    data["AbstractText"] = json!("");
    data["Definition"] = json!("valid");
    data["DefinitionSource"] = json!("provided");
    let query = "a".repeat(MAX_FIELD_BYTES - "Definition: ".len());
    assert_eq!(results(&data, &query, 1)[0].title.len(), MAX_FIELD_BYTES);
    assert_eq!(error(&data, &(query + "a"), 1), DdgParseError::Limit);
}

#[test]
fn encoding_status_parse_errors_and_debug_are_content_free() {
    for body in [
        b"toy-secret\xff".as_slice(),
        b"\xef\xbb\xbftoy-secret".as_slice(),
    ] {
        assert_eq!(
            parse(200, body, "q", 1).expect_err("encoding"),
            DdgParseError::Encoding
        );
    }
    for body in [b"toy-secret".as_slice(), b"[]", br#"{"Answer":"\uD800"}"#] {
        let e = parse(200, body, "q", 1).expect_err("parse");
        assert_eq!(e, DdgParseError::Parse);
        assert!(!format!("{e:?} {e}").contains("toy-secret"));
    }
    for status in [202, 403, 429, 500] {
        assert_eq!(
            parse(status, b"toy-secret", "q", 1).expect_err("status"),
            DdgParseError::Status
        );
    }
    assert_eq!(error(&response(), "q", 1), DdgParseError::Empty);
    let mut data = response();
    data["AbstractText"] = json!("toy-secret");
    let raw = serde_json::to_vec(&data).expect("JSON");
    assert_eq!(
        format!("{:?}", parse(200, &raw, "toy-secret", 1)),
        "Ok(DdgCandidates(<redacted>))"
    );
}

#[test]
fn generated_unicode_candidates_preserve_order_for_all_requested_counts() {
    let mut data = response();
    data["RelatedTopics"] = json!(
        (0..25)
            .map(|i| topic(&format!("中🙂 {i}。 more"), &format!("url{i}")))
            .collect::<Vec<_>>()
    );
    for max in 1..=20 {
        let rows = results(&data, "q", max);
        assert_eq!(rows.len(), max as usize);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.title, format!("中🙂 {i}"));
            assert_eq!(row.url, format!("url{i}"));
        }
    }
}
