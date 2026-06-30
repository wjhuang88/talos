use crossterm::style::Color as CColor;
use talos_conversation::{ToolCallDisplay, ToolResultDisplay};
use talos_core::tool::ToolProvenance;

use crate::app::ScrollbackLine;
use crate::inline_terminal::{HistoryAttrs, HistorySegment};
use crate::theme::{semantic, to_crossterm_color};

/// When a tool's result exceeds this many lines, the scrollback shows a summary
/// instead of the full content. Only applies to tools in the threshold-summarize set.
const SUMMARIZE_OUTPUT_THRESHOLD_LINES: usize = 30;

pub(crate) fn truncate_single_line(s: &str, max: usize) -> String {
    let single = s.replace('\n', " ");
    let chars: Vec<char> = single.chars().collect();
    if chars.len() <= max {
        single
    } else {
        format!("{}…", chars[..max].iter().collect::<String>())
    }
}

pub(crate) fn summarize_tool_args(
    _tool_name: &str,
    args_str: &str,
    summary_fields: &[String],
) -> String {
    let obj: serde_json::Value =
        serde_json::from_str(args_str).unwrap_or(serde_json::Value::Object(Default::default()));

    let parts: Vec<String> = summary_fields
        .iter()
        .filter_map(|field| {
            obj.get(field.as_str()).map(|val| {
                let display = match val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => {
                        let strs: Vec<String> = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        strs.join(", ")
                    }
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => val.to_string(),
                };
                let truncated = truncate_single_line(&display, 60);
                format!("{field}: {truncated}")
            })
        })
        .collect();

    if parts.is_empty() {
        truncate_single_line(&args_str.replace('\n', " "), 120)
    } else {
        parts.join(", ")
    }
}

pub(crate) fn summarize_symbol_results(content: &str, noun: &str) -> String {
    let count = serde_json::from_str::<serde_json::Value>(content)
        .ok()
        .and_then(|v| v.as_array().map(|a| a.len()))
        .unwrap_or_else(|| content.lines().count());
    let singular = noun.strip_suffix('s').unwrap_or(noun);
    let label = if count == 1 { singular } else { noun };
    format!("found {count} {label}")
}

pub(crate) fn should_suppress_tool_result_content(display: &ToolResultDisplay) -> bool {
    if display.is_error {
        return false;
    }
    let Some(name) = display.tool_name.as_deref() else {
        return false;
    };
    const ALWAYS_SUMMARIZE: &[&str] = &[
        "read",
        "list_symbols",
        "find_symbol",
        "find_references",
        "fetch_url",
        "http_request",
        "web_search",
    ];
    if ALWAYS_SUMMARIZE.contains(&name) {
        return true;
    }
    const THRESHOLD_SUMMARIZE: &[&str] = &["glob", "ls", "list_imports"];
    if THRESHOLD_SUMMARIZE.contains(&name) {
        return display.content.lines().count() > SUMMARIZE_OUTPUT_THRESHOLD_LINES;
    }
    false
}

pub(crate) fn suppressed_tool_result_summary(display: &ToolResultDisplay) -> String {
    let line_count = display.content.lines().count();
    let byte_count = display.content.len();
    let name = display.tool_name.as_deref().unwrap_or("tool");
    match name {
        "read" => {
            let label = if line_count == 1 { "line" } else { "lines" };
            format!("read {line_count} {label}, {byte_count} bytes")
        }
        "list_symbols" => summarize_symbol_results(&display.content, "symbols"),
        "find_symbol" => summarize_symbol_results(&display.content, "matching symbols"),
        "find_references" => summarize_symbol_results(&display.content, "references"),
        "glob" => {
            let label = if line_count == 1 { "file" } else { "files" };
            format!("glob matched {line_count} {label}, {byte_count} bytes")
        }
        "ls" => {
            let label = if line_count == 1 { "entry" } else { "entries" };
            format!("ls returned {line_count} {label}, {byte_count} bytes")
        }
        "list_imports" => summarize_symbol_results(&display.content, "imports"),
        "fetch_url" => summarize_http_like_result("fetch_url", &display.content),
        "http_request" => summarize_http_request(&display.content),
        "web_search" => summarize_web_search(&display.content),
        _ => {
            let label = if line_count == 1 { "line" } else { "lines" };
            format!("{line_count} {label}, {byte_count} bytes")
        }
    }
}

fn summarize_http_request(content: &str) -> String {
    summarize_http_like_result("http_request", content)
}

fn summarize_http_like_result(tool: &str, content: &str) -> String {
    let status = content
        .lines()
        .find(|l| l.starts_with("Status: "))
        .map(|l| l.trim_start_matches("Status: ").to_string())
        .unwrap_or_else(|| "?".to_string());
    let content_type = content
        .lines()
        .find(|line| line.trim().to_lowercase().starts_with("content-type:"))
        .map(|line| {
            line.trim()
                .split_once(':')
                .map(|(_, value)| value.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let size = content
        .lines()
        .find(|l| l.starts_with("Content ("))
        .map(|l| {
            l.split('(')
                .nth(1)
                .and_then(|s| s.split(')').next())
                .unwrap_or("?")
                .to_string()
        })
        .unwrap_or_else(|| "? bytes".to_string());
    format!("{tool}: {status}, {size}, {content_type}")
}

fn summarize_web_search(content: &str) -> String {
    let query = content
        .lines()
        .find(|l| l.starts_with("Searched: "))
        .map(|l| {
            l.trim_start_matches("Searched: ")
                .trim_matches('"')
                .to_string()
        })
        .unwrap_or_else(|| "?".to_string());
    let source = content
        .lines()
        .find(|l| l.starts_with("Source: "))
        .map(|l| l.trim_start_matches("Source: ").to_string())
        .unwrap_or_else(|| "?".to_string());
    let results = content
        .lines()
        .find(|l| l.starts_with("Results: "))
        .map(|l| l.trim_start_matches("Results: ").to_string())
        .unwrap_or_else(|| "0".to_string());
    format!("web_search: {results} results for \"{query}\" via {source}")
}

pub(crate) fn build_tool_result_scrollback_lines(
    display: &ToolResultDisplay,
    icon: &str,
    color: Option<CColor>,
) -> Vec<ScrollbackLine> {
    const MAX_RESULT_LINE_CHARS: usize = 120;

    if should_suppress_tool_result_content(display) {
        return vec![ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("   {icon} {}", suppressed_tool_result_summary(display)),
                color,
                HistoryAttrs::default(),
            )],
            None,
        )];
    }

    if display.content.is_empty() {
        return vec![ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("   {icon} (no output)"),
                color,
                HistoryAttrs::default(),
            )],
            None,
        )];
    }

    let mut lines = Vec::new();
    for (idx, line) in display.content.lines().enumerate() {
        let prefix = if idx == 0 { icon } else { " " };
        let truncated = truncate_single_line(line, MAX_RESULT_LINE_CHARS);
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("   {prefix} {truncated}"),
                color,
                HistoryAttrs::default(),
            )],
            None,
        ));
    }

    lines
}

pub(crate) fn build_tool_call_scrollback_line(tool_call: &ToolCallDisplay) -> ScrollbackLine {
    let args_str = serde_json::to_string_pretty(&tool_call.arguments)
        .unwrap_or_else(|_| tool_call.arguments.to_string());
    let args_summary =
        summarize_tool_args(&tool_call.tool_name, &args_str, &tool_call.summary_fields);
    let provenance_marker = match &tool_call.provenance {
        ToolProvenance::Native => None,
        ToolProvenance::McpRemote { server } => Some(format!("[mcp:{}]", server)),
    };
    let accent = to_crossterm_color(semantic::TEXT_ACCENT);
    let prefix_color = to_crossterm_color(semantic::PREFIX_ASSISTANT);
    let dim = to_crossterm_color(semantic::DIM_TEXT);
    let mut segments = vec![
        HistorySegment::styled(
            " → ",
            prefix_color,
            HistoryAttrs {
                bold: true,
                ..HistoryAttrs::default()
            },
        ),
        HistorySegment::styled(
            tool_call.tool_name.to_string(),
            accent,
            HistoryAttrs {
                bold: true,
                ..HistoryAttrs::default()
            },
        ),
    ];
    if let Some(marker) = provenance_marker {
        segments.push(HistorySegment::styled(marker, dim, HistoryAttrs::default()));
    }
    segments.push(HistorySegment::raw(", "));
    segments.push(HistorySegment::styled(
        args_summary,
        dim,
        HistoryAttrs::default(),
    ));
    ScrollbackLine::styled(segments, None)
}
