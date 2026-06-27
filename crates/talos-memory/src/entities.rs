use crate::EntityKind;
use std::collections::HashSet;

pub fn extract_entities(content: &str) -> Vec<(String, EntityKind)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<(String, EntityKind)> = Vec::new();

    let mut add = |name: String, kind: EntityKind| {
        if name.len() < 2 || seen.contains(&name) || result.len() >= 20 {
            return;
        }
        seen.insert(name.clone());
        result.push((name, kind));
    };

    // Scan for URLs: http:// or https://
    let lower = content.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..]
        .find("https://")
        .or(lower[pos..].find("http://"))
    {
        let abs_start = pos + start;
        // Extract URL characters.
        let url_end = content[abs_start..]
            .chars()
            .take_while(|c| {
                let ch = *c;
                ch.is_alphanumeric()
                    || matches!(
                        ch,
                        '/' | '.' | '-' | '_' | '?' | '#' | '&' | '=' | '%' | ':' | '~' | '+' | ','
                    )
            })
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let url = content[abs_start..abs_start + url_end].to_string();
        if url_end > 10 {
            add(url, EntityKind::Url);
        }
        pos = abs_start + url_end;
    }

    // Scan for file paths and code symbols by splitting on whitespace/punctuation.
    for token in content.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '(' | ')'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | ','
                    | ';'
                    | ':'
                    | '"'
                    | '\''
                    | '`'
                    | '\t'
                    | '\n'
                    | '\r'
            )
    }) {
        if token.is_empty() {
            continue;
        }

        // File path: contains '/' and has an extension.
        if token.contains('/') {
            let parts: Vec<&str> = token.split('/').collect();
            if parts.len() >= 2
                && let Some(last) = parts.last()
                && let Some(dot_pos) = last.rfind('.')
            {
                let ext = &last[dot_pos + 1..];
                if !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_alphanumeric()) {
                    add(token.to_string(), EntityKind::File);
                    continue;
                }
            }
        }

        // Bare filename with extension (e.g., Cargo.toml, main.rs) — must have at least one char before dot.
        if let Some(dot_pos) = token.rfind('.') {
            let name_part = &token[..dot_pos];
            let ext = &token[dot_pos + 1..];
            if !name_part.is_empty()
                && !ext.is_empty()
                && ext.len() <= 10
                && ext.chars().all(|c| c.is_alphanumeric())
                && name_part
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                add(token.to_string(), EntityKind::File);
                continue;
            }
        }

        // CamelCase: at least 2 uppercase letters indicating word boundaries.
        let mut upper_count = 0;
        let mut has_lower = false;
        for ch in token.chars() {
            if ch.is_uppercase() {
                upper_count += 1;
            }
            if ch.is_lowercase() {
                has_lower = true;
            }
        }
        if upper_count >= 2
            && has_lower
            && token.len() >= 4
            && token.chars().all(|c| c.is_alphanumeric())
        {
            add(token.to_string(), EntityKind::Code);
            continue;
        }

        // snake_case: at least one underscore separating lowercase words.
        if token.contains('_') {
            let parts: Vec<&str> = token.split('_').collect();
            if parts.len() >= 2
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_lowercase() || c.is_numeric()))
                && token
                    .chars()
                    .all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            {
                add(token.to_string(), EntityKind::Code);
            }
        }
    }

    result
}
