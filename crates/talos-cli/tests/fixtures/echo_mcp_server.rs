//! Std-only fixture MCP server binary for tests.

use std::io::{self, BufRead, Write};

fn extract_id(line: &str) -> String {
    if let Some(idx) = line.find("\"id\":") {
        let after = &line[idx + 5..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix('"') {
            if let Some(end) = rest.find('"') {
                return format!("\"{}\"", &rest[..end]);
            }
        }
        let mut out = String::new();
        for ch in trimmed.chars() {
            if ch.is_ascii_digit() {
                out.push(ch);
            } else {
                break;
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    "null".to_string()
}

fn extract_text_arg(line: &str) -> String {
    if let Some(idx) = line.find("\"text\":\"") {
        let rest = &line[idx + "\"text\":\"".len()..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    String::new()
}

fn main() {
    let prefix = std::env::var("ECHO_PREFIX").unwrap_or_else(|_| "echo".to_string());
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        let id = extract_id(&line);

        let response = if line.contains("\"method\":\"tools/list\"") {
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"tools\":[{{\"name\":\"echo\",\"description\":\"Echo text back\",\"inputSchema\":{{\"type\":\"object\",\"properties\":{{\"text\":{{\"type\":\"string\"}}}},\"required\":[\"text\"]}},\"annotations\":{{\"readOnlyHint\":true}}}}]}}}}"
            )
        } else if line.contains("\"method\":\"tools/call\"") {
            let text = extract_text_arg(&line);
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{{\"content\":[{{\"type\":\"text\",\"text\":\"{}:{}\"}}]}}}}",
                prefix, text
            )
        } else {
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{{\"code\":-32601,\"message\":\"unknown method\"}}}}"
            )
        };

        if writeln!(stdout, "{response}").is_err() {
            break;
        }
        if stdout.flush().is_err() {
            break;
        }
    }
}
