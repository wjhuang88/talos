//! Talos-owned helpers for the protected zero-key compatibility search path.
//!
//! Adapted from rust-websearch 0.1.2, Rust-Framework/rust-agent-framework
//! commit b30180a03609580e8b28dfb682284cb2ba249fba:
//! crates/websearch/src/html_utils.rs and anti_detection/user_agent.rs.
//! Copyright (c) 2026 Rust-Framework Contributors, MIT:
//! crates/talos-tools/LICENSES/rust-websearch-MIT.txt.
//!
//! Preserve upstream HTML/entity/redirect behaviors. Only User-Agent selection
//! uses UUID v4 entropy already available in the existing network feature.

/// 解码常见的 HTML 实体。
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

/// 去除 HTML 标签，返回纯文本。
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

/// 清理文本：去除 HTML 标签 + HTML 实体解码 + 合并空白。
pub(super) fn clean_html(input: &str) -> String {
    let stripped = strip_html_tags(input);
    let decoded = decode_html_entities(&stripped);
    // 合并连续空白
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

/// 解析 DuckDuckGo 跳转 URL（`/l/?uddg=...` 格式）。
/// 返回解码后的真实目标 URL。
pub(super) fn resolve_duckduckgo_url(url: &str) -> String {
    let decoded = decode_html_entities(url);

    // 处理 /l/?uddg=... 跳转链接
    if let Some(rest) = decoded.strip_prefix("/l/?uddg=") {
        return urlencoding_decode(rest).unwrap_or_else(|| decoded.clone());
    }

    // 协议相对 URL（如 //example.com/page）
    if let Some(rest) = decoded.strip_prefix("//") {
        return format!("https://{rest}");
    }

    decoded
}

/// 最小化的 URL 解码（%XX 格式）。
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
            _ => {} // 跳过非 ASCII
        }
    }

    String::from_utf8(bytes).ok()
}

/// Select from the exact, pinned upstream 12-element UA pool.
pub(super) fn random_user_agent() -> &'static str {
    let draw = uuid::Uuid::new_v4().as_u128();
    USER_AGENTS[(draw % USER_AGENTS.len() as u128) as usize]
}

const USER_AGENTS: &[&str] = &[
    // ── Chrome 130+ (Windows) ──
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    // ── Chrome 130+ (macOS) ──
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    // ── Chrome 130+ (Linux) ──
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    // ── Edge 130+ (Windows) ──
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
    // ── Edge 130+ (macOS) ──
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
    // ── Firefox 132+ (Windows) ──
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    // ── Firefox 132+ (macOS) ──
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:132.0) Gecko/20100101 Firefox/132.0",
    // ── Firefox 132+ (Linux) ──
    "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn html_helpers_preserve_upstream_bytes() {
        for input in [
            "",
            "plain text",
            "&amp;&lt;&gt;&quot;&#x27;&#39;&nbsp;",
            "&amp;lt;&unknown;&#x27;&#39;&#39;",
            "<p> Hello </p>  <em>world</em>",
            "malformed <tag unclosed and > more",
            "<<nested>>",
            "a\r\n  b\u{00a0}c",
            "中文 café 😀 <strong>é</strong>",
            "&#x1F600; &apos; &#039; &NbSp;",
            " <a href='a&b'>X &amp; Y</a> ",
        ] {
            assert_eq!(
                decode_html_entities(input),
                rust_websearch::html_utils::decode_html_entities(input)
            );
            assert_eq!(
                strip_html_tags(input),
                rust_websearch::html_utils::strip_html_tags(input)
            );
            assert_eq!(
                clean_html(input),
                rust_websearch::html_utils::clean_html(input)
            );
        }
    }

    #[test]
    fn ddg_redirect_percent_utf8_and_error_parity() {
        for input in [
            "",
            "https://example.org/path?a=1&amp;b=2",
            "//example.org/path?q=é",
            "/l/?uddg=https%3A%2F%2Fexample.org%2Fa%3Fb%3D1%26c%3D2",
            "/l/?uddg=https%3A%2F%2Fexample.org%2Fa+b",
            "/l/?uddg=%E4%B8%AD%E6%96%87",
            "/l/?uddg=%FF",
            "/l/?uddg=%GG",
            "/l/?uddg=%2",
            "/l/?uddg=abc%20def&amp;unused=1",
            "/l/?uddg=非ASCII",
            "/l/?uddg=abc%00def",
            "/l/?uddg=",
            "/l/?other=foo&uddg=https%3A%2F%2Fexample.org",
        ] {
            assert_eq!(
                resolve_duckduckgo_url(input),
                rust_websearch::html_utils::resolve_duckduckgo_url(input)
            );
        }
        assert_eq!(resolve_duckduckgo_url("/l/?uddg=%FF"), "/l/?uddg=%FF");
        assert_eq!(resolve_duckduckgo_url("/l/?uddg=%GG"), "/l/?uddg=%GG");
        assert_eq!(
            resolve_duckduckgo_url("//example.org/a"),
            "https://example.org/a"
        );
        assert_eq!(resolve_duckduckgo_url("/l/?uddg=%E4%B8%AD"), "中");
    }

    #[test]
    fn pinned_upstream_user_agent_pool_is_byte_identical() {
        assert_eq!(USER_AGENTS.len(), 12);
        let pool: HashSet<&str> = USER_AGENTS.iter().copied().collect();
        assert_eq!(pool.len(), 12);
        for ua in USER_AGENTS {
            assert!(ua.starts_with("Mozilla/5.0 "));
            assert!(ua.is_ascii() && !ua.contains('\r') && !ua.contains('\n'));
        }
        let mut upstream_observed = HashSet::new();
        let mut owned_observed = HashSet::new();
        for _ in 0..4096 {
            let original = rust_websearch::anti_detection::random_user_agent();
            assert!(pool.contains(original), "unrecognized upstream UA");
            upstream_observed.insert(original);
            let selected = random_user_agent();
            assert!(pool.contains(selected), "unrecognized owned UA");
            owned_observed.insert(selected);
        }
        assert_eq!(upstream_observed, pool);
        assert_eq!(owned_observed, pool);
    }
}
