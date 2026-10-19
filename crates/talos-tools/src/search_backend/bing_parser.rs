//! Dormant parser for the existing compound compatibility family's Bing CN response.
//!
//! Derived from rust-websearch 0.1.2 src/bing.rs at
//! b30180a03609580e8b28dfb682284cb2ba249fba (crates/websearch).
//! Copyright (c) 2026 Rust-Framework Contributors. Full MIT notice:
//! ../../LICENSES/rust-websearch-MIT.txt
//! Changes: response-only boundary, bounded input/fields/count, strict UTF-8/BOM,
//! closed errors, redacted candidates, incremental text collection and unwind boundary.
//! No HTTP, retry, request factory, URL admission or new provider/backend activation.

use std::{fmt, panic::catch_unwind};

use rust_websearch::html_utils::clean_html;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use super::{
    SearchBackendId, SearchBackendResult,
    ddg_parser::{MAX_BODY_BYTES, MAX_FIELD_BYTES},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(super) enum BingParseError {
    #[error("search_bing_response_limit")]
    Limit,
    #[error("search_bing_response_encoding")]
    Encoding,
    #[error("search_bing_response_status")]
    Status,
    #[error("search_bing_response_rate_limited")]
    RateLimited,
    #[error("search_bing_response_challenge")]
    Challenge,
    #[error("search_bing_response_parse")]
    Parse,
    #[error("search_bing_response_empty")]
    Empty,
}

pub(super) struct BingCandidates(Vec<SearchBackendResult>);

impl fmt::Debug for BingCandidates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BingCandidates(<redacted>)")
    }
}

impl BingCandidates {
    /// Raw candidates are neither approved URLs nor authorized fetch destinations.
    pub(super) fn into_candidates(self) -> Vec<SearchBackendResult> {
        self.0
    }
}

pub(super) fn parse(
    status: u16,
    body: &[u8],
    max_results: u32,
) -> Result<BingCandidates, BingParseError> {
    if body.len() > MAX_BODY_BYTES || !(1..=20).contains(&max_results) {
        return Err(BingParseError::Limit);
    }
    if matches!(status, 429 | 503) {
        return Err(BingParseError::RateLimited);
    }
    if status != 200 {
        return Err(BingParseError::Status);
    }
    if body.starts_with(b"\xef\xbb\xbf") {
        return Err(BingParseError::Encoding);
    }
    let html = std::str::from_utf8(body).map_err(|_| BingParseError::Encoding)?;
    if [
        "g-recaptcha",
        "cf-challenge",
        "verifyContainer",
        "bnp_dialog",
    ]
    .iter()
    .any(|marker| html.contains(marker))
    {
        return Err(BingParseError::Challenge);
    }
    // Rust unwinding only: not abort/OOM recovery, a hard memory/CPU bound or
    // suppression of the process panic hook. scraper still builds the full DOM.
    catch_unwind(|| parse_document(html, max_results as usize))
        .map_err(|_| BingParseError::Parse)?
}

fn bounded(value: String) -> Result<String, BingParseError> {
    if value.len() > MAX_FIELD_BYTES {
        Err(BingParseError::Limit)
    } else {
        Ok(value)
    }
}

fn text(element: ElementRef<'_>) -> Result<String, BingParseError> {
    let mut result = String::new();
    for (index, fragment) in element.text().enumerate() {
        result
            .len()
            .checked_add(usize::from(index != 0))
            .and_then(|n| n.checked_add(fragment.len()))
            .filter(|n| *n <= MAX_FIELD_BYTES)
            .ok_or(BingParseError::Limit)?;
        if index != 0 {
            result.push(' ');
        }
        result.push_str(fragment);
    }
    bounded(clean_html(&result))
}

fn parse_document(html: &str, max: usize) -> Result<BingCandidates, BingParseError> {
    let document = Html::parse_document(html);
    let rows = Selector::parse("li.b_algo").map_err(|_| BingParseError::Parse)?;
    let links = Selector::parse("h2 a").map_err(|_| BingParseError::Parse)?;
    let snippets = Selector::parse(".b_caption p").map_err(|_| BingParseError::Parse)?;
    let mut results: Vec<SearchBackendResult> = Vec::new();
    for row in document.select(&rows) {
        if results.len() >= max {
            break;
        }
        let Some(link) = row.select(&links).next() else {
            continue;
        };
        let raw_url = link.value().attr("href").unwrap_or("");
        // Check before trim; even discarded raw surrounding whitespace is bounded.
        if raw_url.len() > MAX_FIELD_BYTES {
            return Err(BingParseError::Limit);
        }
        let url = raw_url.trim().to_owned();
        // Upstream cleans title before URL rejection, snippet before deduplication.
        let title = text(link)?;
        if url.is_empty() || url.starts_with("javascript:") {
            continue;
        }
        let snippet = row
            .select(&snippets)
            .next()
            .map(text)
            .transpose()?
            .unwrap_or_default();
        if results.iter().any(|r| r.url == url) {
            continue;
        }
        results.push(SearchBackendResult {
            // Existing normalize_compatibility_results uses the compound family's
            // DuckDuckGo ID even for opaque Bing results. Not provider attribution
            // or native Bing admission; no new backend variant is introduced.
            backend: SearchBackendId::DuckDuckGo,
            title: if title.is_empty() { url.clone() } else { title },
            url,
            snippet,
        });
    }
    if results.is_empty() {
        return Err(BingParseError::Empty);
    }
    Ok(BingCandidates(results))
}

#[cfg(test)]
mod tests;
