//! Dormant compatibility response parsing; candidates still require router admission.
//!
//! Derived from rust-websearch 0.1.2, src/duckduckgo/{lite,html}.rs at
//! b30180a03609580e8b28dfb682284cb2ba249fba (crates/websearch).
//! Copyright (c) 2026 Rust-Framework Contributors. MIT notice:
//! ../../LICENSES/rust-websearch-MIT.txt
//! Changes: shared selector traversal, bounded input/fields/results, strict UTF-8/BOM
//! handling, closed errors, redacted output Debug and unwind boundary; no HTTP/coordinator.
//! Uses existing network-free html_utils helpers. Talos scraper 0.27 versus upstream
//! 0.22 and charset/bounds differences require parity review before activation.

use std::{fmt, panic::catch_unwind};

use rust_websearch::html_utils::{clean_html, resolve_duckduckgo_url};
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use super::{SearchBackendId, SearchBackendResult};

pub(super) const MAX_BODY_BYTES: usize = 1_048_576;
pub(super) const MAX_FIELD_BYTES: usize = 16_384;

#[derive(Debug, Clone, Copy)]
pub(super) enum DdgFormat {
    Lite,
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(super) enum DdgParseError {
    #[error("search_ddg_response_limit")]
    Limit,
    #[error("search_ddg_response_encoding")]
    Encoding,
    #[error("search_ddg_response_status")]
    Status,
    #[error("search_ddg_response_challenge")]
    Challenge,
    #[error("search_ddg_response_parse")]
    Parse,
    #[error("search_ddg_response_empty")]
    Empty,
}

pub(super) struct DdgCandidates(Vec<SearchBackendResult>);

impl fmt::Debug for DdgCandidates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DdgCandidates(<redacted>)")
    }
}

impl DdgCandidates {
    pub(super) fn new(candidates: Vec<SearchBackendResult>) -> Self {
        Self(candidates)
    }

    /// These are untrusted candidates, not approved URLs or fetch destinations.
    pub(super) fn into_candidates(self) -> Vec<SearchBackendResult> {
        self.0
    }
}

pub(super) fn parse(
    format: DdgFormat,
    status: u16,
    body: &[u8],
    max_results: u32,
) -> Result<DdgCandidates, DdgParseError> {
    if body.len() > MAX_BODY_BYTES || !(1..=20).contains(&max_results) {
        return Err(DdgParseError::Limit);
    }
    if status == 202 && matches!(format, DdgFormat::Html) {
        return Err(DdgParseError::Challenge);
    }
    if status != 200 {
        return Err(DdgParseError::Status);
    }
    if body.starts_with(b"\xef\xbb\xbf") {
        return Err(DdgParseError::Encoding);
    }
    let html = std::str::from_utf8(body).map_err(|_| DdgParseError::Encoding)?;
    if html.contains("challenge-form")
        || html.contains("g-recaptcha")
        || (matches!(format, DdgFormat::Lite) && html.contains("cf-challenge"))
    {
        return Err(DdgParseError::Challenge);
    }
    // Handles Rust unwinding, not OOM/abort or a synchronous CPU deadline.
    catch_unwind(|| parse_document(format, html, max_results as usize))
        .map_err(|_| DdgParseError::Parse)?
}

fn bounded(value: String) -> Result<String, DdgParseError> {
    if value.len() > MAX_FIELD_BYTES {
        Err(DdgParseError::Limit)
    } else {
        Ok(value)
    }
}

fn text(element: ElementRef<'_>) -> Result<String, DdgParseError> {
    let mut result = String::new();
    for (index, fragment) in element.text().enumerate() {
        result
            .len()
            .checked_add(usize::from(index != 0))
            .and_then(|n| n.checked_add(fragment.len()))
            .filter(|n| *n <= MAX_FIELD_BYTES)
            .ok_or(DdgParseError::Limit)?;
        if index != 0 {
            result.push(' ');
        }
        result.push_str(fragment);
    }
    bounded(clean_html(&result))
}

fn parse_document(
    format: DdgFormat,
    html: &str,
    max: usize,
) -> Result<DdgCandidates, DdgParseError> {
    let document = Html::parse_document(html);
    let (links, snippets) = match format {
        DdgFormat::Lite => ("a.result-link", "td.result-snippet"),
        DdgFormat::Html => ("a.result__a", "a.result__snippet"),
    };
    let links = Selector::parse(links).map_err(|_| DdgParseError::Parse)?;
    let snippets = Selector::parse(snippets).map_err(|_| DdgParseError::Parse)?;
    let mut snippets = document.select(&snippets);
    let mut candidates: Vec<SearchBackendResult> = Vec::new();
    for link in document.select(&links) {
        if candidates.len() >= max {
            break;
        }
        // Consume by original link position, even if this URL is later skipped/deduplicated.
        let snippet = snippets.next().map(text).transpose()?.unwrap_or_default();
        let raw_url = link.value().attr("href").unwrap_or("");
        if raw_url.len() > MAX_FIELD_BYTES {
            return Err(DdgParseError::Limit);
        }
        let title = text(link)?;
        let url = bounded(resolve_duckduckgo_url(raw_url))?;
        if url.is_empty() || url == "https:" || candidates.iter().any(|r| r.url == url) {
            continue;
        }
        candidates.push(SearchBackendResult {
            backend: SearchBackendId::DuckDuckGo,
            title: if title.is_empty() { url.clone() } else { title },
            url,
            snippet,
        });
    }
    // Upstream Lite's second selector equals the first; it cannot produce a first valid
    // result after an empty first pass with positive max. Errors never trigger fallback.
    if candidates.is_empty() {
        return Err(DdgParseError::Empty);
    }
    Ok(DdgCandidates(candidates))
}

#[cfg(test)]
mod tests;
