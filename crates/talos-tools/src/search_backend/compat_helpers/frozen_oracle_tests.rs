//! Frozen actual-upstream comparison vectors for SEARCH-001-L / I319.
//!
//! The pinned `rust-websearch` 0.1.2 functions and Talos K1 equivalents were
//! directly compared in exact-head CI3862. These literal expectations are
//! verified against BOTH implementations in this incremental pre-retirement
//! test. A later separately reviewed L1 step may remove the upstream calls
//! only after this exact-head comparison passes; expected literals stay.
//!
//! Provenance: Rust-Framework/rust-agent-framework
//! b30180a03609580e8b28dfb682284cb2ba249fba.
//! Original MIT license: crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//! These vectors deliberately retain historical quirks; no parser upgrade.

use super::*;

// Exact literal bytes are the compatibility contract. Do not normalize them.
#[rustfmt::skip]
const ENTITY_ORACLE: &[(&str, &str)] = &[
    ("", ""),
    ("&amp;", "&"),
    ("&lt;&gt;&quot;&#x27;&#39;&nbsp;", "<>\"'' "),
    ("&amp;amp;", "&amp;"),
    ("&amp;lt;", "<"),
    ("a &amp; b &nbsp; c", "a & b   c"),
    ("&apos;&copy;&#X27;&#x27;", "&apos;&copy;&#X27;'"),
    ("&#x27;&#39; &#39;&#x27;", "'' ''"),
    ("中&音符;&amp;🙂", "中&音符;&🙂"),
];

#[rustfmt::skip]
const CLEAN_ORACLE: &[(&str, &str, &str)] = &[
    ("", "", ""),
    (" plain  text ", " plain  text ", "plain text"),
    (" \t<b>Hello</b>  &amp;  World\n ", " \tHello  &amp;  World\n ", "Hello & World"),
    ("<p>hi</p><span>there</span>", "hithere", "hithere"),
    ("a<b>nested<i>words</i></b>z", "anestedwordsz", "anestedwordsz"),
    ("2 < 3 and > 1", "2  1", "2 1"),
    ("orphan <tag", "orphan ", "orphan"),
    ("mismatched >tag< x", "mismatched tag", "mismatched tag"),
    ("a&nbsp;&nbsp;b \n\tc", "a&nbsp;&nbsp;b \n\tc", "a b c"),
    ("😀 <a href='?x=1&amp;y=2'>你好</a> &amp; fin", "😀 你好 &amp; fin", "😀 你好 & fin"),
];

#[rustfmt::skip]
const REDIRECT_ORACLE: &[(&str, &str)] = &[
    ("", ""),
    ("https://example.org/a?x=1&amp;y=2", "https://example.org/a?x=1&y=2"),
    ("//example.org/相对?x=1&amp;y=2", "https://example.org/相对?x=1&y=2"),
    ("/l/?uddg=https%3A%2F%2Fexample.org%2Fa%3Fx%3D1%26b%3D2", "https://example.org/a?x=1&b=2"),
    ("/l/?uddg=https%3A%2F%2Fexample.org%2Ffoo+bar", "https://example.org/foo bar"),
    ("/l/?uddg=%E4%B8%AD%E6%96%87", "中文"),
    ("/l/?uddg=%F0%9F%98%80", "😀"),
    ("/l/?uddg=literal中文", "literal"),
    ("/l/?uddg=%FF", "/l/?uddg=%FF"),
    ("/l/?uddg=%ZZ", "/l/?uddg=%ZZ"),
    ("/l/?uddg=%A", "/l/?uddg=%A"),
    ("/l/?uddg=%", "/l/?uddg=%"),
    ("/l/?uddg=abc&amp;def=ghi", "abc&def=ghi"),
    ("/l/?uddg=https%3a%2f%2fexample.org%2f%00", "https://example.org/\0"),
    ("/l/?uddg=%C0%AF", "/l/?uddg=%C0%AF"),
    ("/L/?uddg=https%3A%2F%2Fexample.org", "/L/?uddg=https%3A%2F%2Fexample.org"),
];

#[test]
fn frozen_entity_values_match_owned_and_pinned_upstream() {
    for &(input, expected) in ENTITY_ORACLE {
        assert_eq!(
            decode_html_entities(input),
            expected,
            "owned entity {input:?}"
        );
        assert_eq!(
            rust_websearch::html_utils::decode_html_entities(input),
            expected,
            "pinned upstream entity {input:?}"
        );
    }
}

#[test]
fn frozen_clean_values_match_owned_and_pinned_upstream() {
    for &(input, stripped, cleaned) in CLEAN_ORACLE {
        assert_eq!(strip_html_tags(input), stripped, "owned strip {input:?}");
        assert_eq!(clean_html(input), cleaned, "owned clean {input:?}");
        assert_eq!(
            rust_websearch::html_utils::strip_html_tags(input),
            stripped,
            "pinned upstream strip {input:?}"
        );
        assert_eq!(
            rust_websearch::html_utils::clean_html(input),
            cleaned,
            "pinned upstream clean {input:?}"
        );
    }
}

#[test]
fn frozen_redirect_values_match_owned_and_pinned_upstream() {
    for &(input, expected) in REDIRECT_ORACLE {
        assert_eq!(
            resolve_duckduckgo_url(input),
            expected,
            "owned redirect {input:?}"
        );
        assert_eq!(
            rust_websearch::html_utils::resolve_duckduckgo_url(input),
            expected,
            "pinned upstream redirect {input:?}"
        );
    }
}

#[test]
fn frozen_upstream_user_agent_pool_hash_remains_stable() {
    // Independently recorded FNV-1a64 of the ordered 12 strings joined by LF.
    assert_eq!(USER_AGENTS.len(), 12);
    let hash = USER_AGENTS
        .join("\n")
        .bytes()
        .fold(0xcbf29ce484222325_u64, |state, byte| {
            (state ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
    assert_eq!(hash, 0xe8d044e22b377885);
}
