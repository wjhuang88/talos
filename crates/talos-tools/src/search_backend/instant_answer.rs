//! Dormant DDG Instant Answer response parsing; all outputs remain untrusted.
//!
//! Derived from rust-websearch 0.1.2 src/duckduckgo/instant_answer.rs at
//! b30180a03609580e8b28dfb682284cb2ba249fba (crates/websearch).
//! Copyright (c) 2026 Rust-Framework Contributors. Full MIT notice:
//! ../../LICENSES/rust-websearch-MIT.txt
//! Changes: response-only boundary, input/field/query/count/depth bounds, strict
//! UTF-8/BOM handling, closed errors, redacted candidates and unwind boundary.
//! Unlike upstream's first three unconditional additions, each addition honors max.
//! No HTTP, coordinator, request factory, URL admission or default activation.

use std::panic::catch_unwind;

use rust_websearch::html_utils::decode_html_entities;
use serde::Deserialize;

use super::{
    SearchBackendId, SearchBackendResult,
    ddg_parser::{DdgCandidates, DdgParseError, MAX_BODY_BYTES, MAX_FIELD_BYTES},
};

const MAX_TOPIC_DEPTH: usize = 32;

// Keep required-but-unused fields: projecting through Value or dropping them changes
// upstream schema errors, including duplicate known fields and missing/null fields.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    abstract_text: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "AbstractSource")]
    _abstract_source: String,
    heading: String,
    answer: String,
    answer_type: String,
    definition: String,
    definition_source: String,
    related_topics: Vec<Topic>,
    infobox: Option<Infobox>,
    #[serde(rename = "Type")]
    _response_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Topic {
    text: String,
    #[serde(rename = "FirstURL")]
    first_url: String,
    result: Option<String>,
    #[serde(rename = "Icon")]
    _icon: Option<Icon>,
    #[serde(default)]
    topics: Vec<Topic>,
}

#[derive(Deserialize)]
struct Icon {
    #[serde(rename = "URL")]
    _url: String,
}

#[derive(Deserialize)]
struct Infobox {
    #[serde(default)]
    content: Vec<Content>,
    #[serde(default)]
    meta: Vec<Meta>,
}

#[derive(Deserialize)]
struct Content {
    label: String,
    value: String,
    #[serde(rename = "data_type")]
    _data_type: String,
}

#[derive(Deserialize)]
struct Meta {
    label: String,
    value: String,
}

pub(super) fn parse(
    status: u16,
    body: &[u8],
    query: &str,
    max_results: u32,
) -> Result<DdgCandidates, DdgParseError> {
    if body.len() > MAX_BODY_BYTES
        || query.len() > MAX_FIELD_BYTES
        || !(1..=20).contains(&max_results)
    {
        return Err(DdgParseError::Limit);
    }
    if status != 200 {
        return Err(DdgParseError::Status);
    }
    if body.starts_with(b"\xef\xbb\xbf") {
        return Err(DdgParseError::Encoding);
    }
    let json = std::str::from_utf8(body).map_err(|_| DdgParseError::Encoding)?;
    // Default serde_json recursion guard stays enabled. Rust unwind only, not OOM,
    // abort, a synchronous CPU deadline or suppression of the process panic hook.
    catch_unwind(|| parse_response(json, query, max_results as usize))
        .map_err(|_| DdgParseError::Parse)?
}

fn bounded(value: &str) -> Result<&str, DdgParseError> {
    if value.len() > MAX_FIELD_BYTES {
        Err(DdgParseError::Limit)
    } else {
        Ok(value)
    }
}

fn clean(text: &str) -> Result<String, DdgParseError> {
    let mut text = bounded(text)?.to_owned();
    for tag in [
        "<b>", "</b>", "<i>", "</i>", "<code>", "</code>", "<pre>", "</pre>",
    ] {
        text = text.replace(tag, "");
    }
    for tag in ["<br>", "<br/>", "<br />"] {
        text = text.replace(tag, " ");
    }
    let text = decode_html_entities(&text);
    bounded(&text)?;
    Ok(text)
}

fn append(target: &mut String, part: &str) -> Result<(), DdgParseError> {
    target
        .len()
        .checked_add(part.len())
        .filter(|n| *n <= MAX_FIELD_BYTES)
        .ok_or(DdgParseError::Limit)?;
    target.push_str(part);
    Ok(())
}

fn title(prefix: &str, query: &str) -> Result<String, DdgParseError> {
    let mut result = prefix.to_owned();
    append(&mut result, query)?;
    Ok(result)
}

fn fallback_url(query: &str) -> Result<String, DdgParseError> {
    let mut result = String::from("https://duckduckgo.com/?q=");
    for byte in query.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                append(&mut result, &char::from(byte).to_string())?;
            }
            b' ' => append(&mut result, "+")?,
            _ => append(&mut result, &format!("%{byte:02X}"))?,
        }
    }
    Ok(result)
}

fn push(
    results: &mut Vec<SearchBackendResult>,
    title: &str,
    url: &str,
    snippet: &str,
) -> Result<(), DdgParseError> {
    results.push(SearchBackendResult {
        backend: SearchBackendId::DuckDuckGo,
        title: bounded(title)?.to_owned(),
        url: bounded(url)?.to_owned(),
        snippet: bounded(snippet)?.to_owned(),
    });
    Ok(())
}

fn topics(
    topic: &Topic,
    results: &mut Vec<SearchBackendResult>,
    max: usize,
    depth: usize,
) -> Result<(), DdgParseError> {
    if results.len() >= max {
        return Ok(());
    }
    if depth > MAX_TOPIC_DEPTH {
        return Err(DdgParseError::Limit);
    }
    let text = topic.result.as_deref().unwrap_or(&topic.text);
    if !text.is_empty() && !topic.first_url.is_empty() {
        let text = bounded(text)?;
        let first = text
            .split(['.', '!', '?', '。', '？', '！'])
            .next()
            .unwrap_or(text);
        push(results, &clean(first)?, &topic.first_url, &clean(text)?)?;
    }
    for child in &topic.topics {
        if results.len() >= max {
            break;
        }
        topics(child, results, max, depth + 1)?;
    }
    Ok(())
}

fn parse_response(json: &str, query: &str, max: usize) -> Result<DdgCandidates, DdgParseError> {
    let data: Response = serde_json::from_str(json).map_err(|_| DdgParseError::Parse)?;
    let mut results = Vec::new();
    if !data.abstract_text.is_empty() {
        let url = if data.abstract_url.is_empty() {
            fallback_url(query)?
        } else {
            data.abstract_url
        };
        let heading = if data.heading.is_empty() {
            query
        } else {
            &data.heading
        };
        push(&mut results, heading, &url, &clean(&data.abstract_text)?)?;
    }
    if results.len() < max && !data.answer.is_empty() && data.answer_type != "calc" {
        push(
            &mut results,
            &title("Answer: ", query)?,
            &fallback_url(query)?,
            &clean(&data.answer)?,
        )?;
    }
    if results.len() < max && !data.definition.is_empty() {
        let url = if data.definition_source.is_empty() {
            fallback_url(query)?
        } else {
            data.definition_source
        };
        push(
            &mut results,
            &title("Definition: ", query)?,
            &url,
            &clean(&data.definition)?,
        )?;
    }
    for topic in &data.related_topics {
        if results.len() >= max {
            break;
        }
        topics(topic, &mut results, max, 1)?;
    }
    if let Some(infobox) = data.infobox {
        for meta in &infobox.meta {
            if results.len() >= max {
                break;
            }
            push(&mut results, &meta.label, "", &meta.value)?;
        }
        for content in &infobox.content {
            if results.len() >= max {
                break;
            }
            push(&mut results, &content.label, "", &content.value)?;
        }
    }
    if results.is_empty() {
        return Err(DdgParseError::Empty);
    }
    Ok(DdgCandidates::new(results))
}

#[cfg(test)]
mod tests;
