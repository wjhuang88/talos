use crossterm::style::Color as CColor;
use talos_conversation::{ToolCallDisplay, ToolResultDisplay};
use talos_core::tool::ToolProvenance;

use crate::app::ScrollbackLine;
use crate::inline_terminal::{HistoryAttrs, HistorySegment};
use crate::theme::{semantic, to_crossterm_color};

/// When a tool's result exceeds this many lines, the scrollback shows a summary
/// instead of the full content. Only applies to tools in the threshold-summarize set.
const SUMMARIZE_OUTPUT_THRESHOLD_LINES: usize = 30;
/// Leading lines retained when a non-summarized tool result exceeds the shared
/// threshold and is rendered with head+tail truncation (TUI-015).
const HEAD_LINES: usize = 10;
/// Trailing lines retained when a non-summarized tool result exceeds the shared
/// threshold and is rendered with head+tail truncation (TUI-015).
const TAIL_LINES: usize = 10;

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

/// One-line summary for a `grep` result. Grep output uses `path:` file headers
/// followed by indented `  line_num: content` match lines; we count each shape
/// and fall back to a raw line count if neither is recognizable.
pub(crate) fn summarize_grep_result(content: &str) -> String {
    let byte_count = content.len();
    let mut file_count = 0usize;
    let mut match_count = 0usize;
    for line in content.lines() {
        let indented = line.len() > line.trim_start().len();
        if indented && line.contains(':') {
            match_count += 1;
        } else if !indented && line.ends_with(':') {
            file_count += 1;
        }
    }
    if match_count == 0 {
        let line_count = content.lines().count();
        let label = if line_count == 1 { "line" } else { "lines" };
        return format!("grep matched {line_count} {label}, {byte_count} bytes");
    }
    let line_label = if match_count == 1 { "line" } else { "lines" };
    let file_label = if file_count == 1 { "file" } else { "files" };
    format!(
        "grep matched {match_count} {line_label} in {file_count} {file_label}, {byte_count} bytes"
    )
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
    const THRESHOLD_SUMMARIZE: &[&str] = &["glob", "grep", "ls", "list_imports"];
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
        "grep" => summarize_grep_result(&display.content),
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

    let all_lines: Vec<&str> = display.content.lines().collect();

    // TUI-015: tools that are not summarized still collapse once they cross the
    // shared threshold, keeping the first and last lines visible with an omitted
    // counter in between. This is scrollback-display only; `/export` writes the
    // raw `ToolResultDisplay::content` and never enters this path.
    if all_lines.len() > SUMMARIZE_OUTPUT_THRESHOLD_LINES {
        return build_head_tail_scrollback_lines(&all_lines, icon, color);
    }

    let mut lines = Vec::with_capacity(all_lines.len());
    for (idx, line) in all_lines.iter().enumerate() {
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

fn build_head_tail_scrollback_lines(
    all_lines: &[&str],
    icon: &str,
    color: Option<CColor>,
) -> Vec<ScrollbackLine> {
    const MAX_RESULT_LINE_CHARS: usize = 120;
    let dim = to_crossterm_color(semantic::DIM_TEXT);
    let mut lines = Vec::with_capacity(HEAD_LINES + 1 + TAIL_LINES);

    for (idx, line) in all_lines.iter().take(HEAD_LINES).enumerate() {
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

    let omitted = all_lines
        .len()
        .saturating_sub(HEAD_LINES)
        .saturating_sub(TAIL_LINES);
    lines.push(ScrollbackLine::styled(
        vec![HistorySegment::styled(
            format!("   ⋯ {omitted} lines omitted"),
            dim,
            HistoryAttrs::default(),
        )],
        None,
    ));

    let tail_start = all_lines.len().saturating_sub(TAIL_LINES);
    for line in all_lines.iter().skip(tail_start) {
        let truncated = truncate_single_line(line, MAX_RESULT_LINE_CHARS);
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("     {truncated}"),
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
