//! Private zero-key compatibility helpers retained with pinned upstream semantics.
//!
//! Adapted from rust-websearch 0.1.2, Rust-Framework/rust-agent-framework
//! b30180a03609580e8b28dfb682284cb2ba249fba:
//! crates/websearch/src/html_utils.rs and anti_detection/user_agent.rs.
//! Copyright (c) 2026 Rust-Framework Contributors; MIT license:
//! crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//!
//! This is intentionally NOT an HTML/URL standards upgrade. Preserve even the upstream
//! limited entity table, sequential replacements, naive tag state and non-ASCII handling
//! until a separately reviewed migration changes parser compatibility semantics.
//! No network operations, source selection, rate limiter or transport policy live here.

/// Match the pinned upstream's exact seven, sequential entity replacements.
pub(super) fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

pub(super) fn clean_html(input: &str) -> String {
    let stripped = strip_html_tags(input);
    let decoded = decode_html_entities(&stripped);
    let mut result = String::with_capacity(decoded.len());
    let mut last_was_whitespace = false;
    for ch in decoded.chars() {
        if ch.is_whitespace() {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            result.push(ch);
            last_was_whitespace = false;
        }
    }
    result.trim().to_string()
}

/// Only the pinned /l/?uddg= prefix and scheme-relative case are special.
pub(super) fn resolve_duckduckgo_url(url: &str) -> String {
    let decoded = decode_html_entities(url);
    if let Some(rest) = decoded.strip_prefix("/l/?uddg=") {
        return urlencoding_decode(rest).unwrap_or_else(|| decoded.clone());
    }
    if let Some(rest) = decoded.strip_prefix("//") {
        return format!("https://{rest}");
    }
    decoded
}

fn urlencoding_decode(input: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => {
                let hi = chars.next()?;
                let lo = chars.next()?;
                let byte = u8::from_str_radix(&format!("{hi}{lo}"), 16).ok()?;
                bytes.push(byte);
            }
            '+' => bytes.push(b' '),
            c if c.is_ascii() => bytes.push(c as u8),
            _ => {} // Intentionally skip literal non-ASCII, matching pinned upstream.
        }
    }
    String::from_utf8(bytes).ok()
}

/// The exact upstream twelve-element desktop UA pool (2025-era strings).
/// This is a fixed compatibility fixture, not a claim that these browsers are current.
const USER_AGENTS: [&str; 12] = [
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:132.0) Gecko/20100101 Firefox/132.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
];

/// Draw from the same uniform pool without adding Cargo dependencies.
///
/// Network already enables uuid/v4. Rejection-sampling one OS-random UUID byte
/// avoids modulo bias (256 % 12 != 0) and does not create any I/O or network path.
pub(super) fn random_user_agent() -> &'static str {
    let accepted = (u8::MAX as usize + 1) / USER_AGENTS.len() * USER_AGENTS.len();
    loop {
        let sample = usize::from(uuid::Uuid::new_v4().as_bytes()[0]);
        if sample < accepted {
            return USER_AGENTS[sample % USER_AGENTS.len()];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // The frozen expected-value vectors now supply independent, pinned-upstream
    // entity, HTML-cleanup and DDG-redirect tests. Do not duplicate an active
    // dependency on the upstream helper functions here.

    #[test]
    fn fixed_user_agent_set_and_selection_membership() {
        assert_eq!(USER_AGENTS.len(), 12);
        let unique: HashSet<&str> = USER_AGENTS.into_iter().collect();
        assert_eq!(unique.len(), 12);
        for ua in USER_AGENTS {
            assert!(ua.starts_with("Mozilla/5.0 "));
            assert!(!ua.contains('\n'));
            assert!(!ua.contains('\r'));
        }
        for _ in 0..256 {
            assert!(USER_AGENTS.contains(&random_user_agent()));
        }
    }

    #[test]
    fn pinned_upstream_user_agent_pool_exact_bytes_and_order() {
        // FNV-1a64 of the twelve original UA strings joined with a single LF.
        // Expected value independently computed from upstream user_agent.rs at
        // b30180a03609580e8b28dfb682284cb2ba249fba.
        let mut fingerprint = 0xcbf29ce484222325_u64;
        for byte in USER_AGENTS.join("\n").bytes() {
            fingerprint ^= u64::from(byte);
            fingerprint = fingerprint.wrapping_mul(0x100000001b3);
        }
        assert_eq!(fingerprint, 0xe8d044e22b377885);
    }

    #[test]
    fn uniform_random_index_boundary_is_an_exact_multiple_of_pool_size() {
        let threshold = (u8::MAX as usize + 1) / USER_AGENTS.len() * USER_AGENTS.len();
        assert_eq!(threshold, 252);
        assert_eq!(threshold % USER_AGENTS.len(), 0);
    }
}

#[cfg(test)]
mod frozen_oracle_tests;
