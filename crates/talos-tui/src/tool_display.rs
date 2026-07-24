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
const HEAD_LINES: usize = 3;
/// Trailing lines retained when a non-summarized tool result exceeds the shared
/// threshold and is rendered with head+tail truncation (TUI-015).
const TAIL_LINES: usize = 3;
const TOOL_CALL_ARGS_BUDGET_CHARS: usize = 180;

pub(crate) fn truncate_single_line(s: &str, max: usize) -> String {
    let single = s.replace('\n', " ");
    let chars: Vec<char> = single.chars().collect();
    if chars.len() <= max {
        single
    } else if max == 0 {
        String::new()
    } else {
        format!(
            "{}…",
            chars[..max.saturating_sub(1)].iter().collect::<String>()
        )
    }
}

pub(crate) fn summarize_tool_args(
    _tool_name: &str,
    args_str: &str,
    summary_fields: &[String],
) -> String {
    summarize_tool_args_with_budget(args_str, summary_fields, TOOL_CALL_ARGS_BUDGET_CHARS)
}

pub(crate) fn summarize_tool_args_with_budget(
    args_str: &str,
    summary_fields: &[String],
    budget_chars: usize,
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
                format!("{field}: {display}")
            })
        })
        .collect();

    let summary = if parts.is_empty() {
        args_str.replace('\n', " ")
    } else {
        parts.join(", ")
    };
    truncate_single_line(&summary, budget_chars)
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
    viewport_width: u16,
) -> Vec<ScrollbackLine> {
    let prefix_len = result_line_prefix(icon, true).len();
    let budget = (viewport_width as usize).saturating_sub(prefix_len).max(20);

    if should_suppress_tool_result_content(display) {
        let (line_color, attrs) = if display.is_error {
            (color, primary_result_attrs())
        } else {
            (secondary_result_color(), secondary_result_attrs())
        };
        return vec![ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!(
                    "{}{}",
                    result_line_prefix(icon, true),
                    suppressed_tool_result_summary(display)
                ),
                line_color,
                attrs,
            )],
            None,
        )];
    }

    if display.content.is_empty() {
        let (line_color, attrs) = if display.is_error {
            (color, primary_result_attrs())
        } else {
            (secondary_result_color(), secondary_result_attrs())
        };
        return vec![ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("{}(no output)", result_line_prefix(icon, true)),
                line_color,
                attrs,
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
        return build_head_tail_scrollback_lines(display, &all_lines, icon, color, viewport_width);
    }

    let diff_aware = is_diff_content(&display.content, display.tool_name.as_deref());
    let mut lines = Vec::with_capacity(all_lines.len());
    for (idx, line) in all_lines.iter().enumerate() {
        let truncated = crate::scrollback::truncate_to_display_width(line, budget);
        let (line_color, attrs) = if diff_aware {
            diff_line_style(line).unwrap_or_else(|| result_line_style(display, idx, color))
        } else {
            result_line_style(display, idx, color)
        };
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("{}{truncated}", result_line_prefix(icon, idx == 0)),
                line_color,
                attrs,
            )],
            None,
        ));
    }

    lines
}

fn build_head_tail_scrollback_lines(
    display: &ToolResultDisplay,
    all_lines: &[&str],
    icon: &str,
    color: Option<CColor>,
    viewport_width: u16,
) -> Vec<ScrollbackLine> {
    let prefix_len = result_line_prefix(icon, true).len();
    let budget = (viewport_width as usize).saturating_sub(prefix_len).max(20);
    let secondary = secondary_result_color();
    let mut lines = Vec::with_capacity(HEAD_LINES + 1 + TAIL_LINES);

    for (idx, line) in all_lines.iter().take(HEAD_LINES).enumerate() {
        let truncated = crate::scrollback::truncate_to_display_width(line, budget);
        let (line_color, attrs) = result_line_style(display, idx, color);
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("{}{truncated}", result_line_prefix(icon, idx == 0)),
                line_color,
                attrs,
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
            secondary,
            secondary_result_attrs(),
        )],
        None,
    ));

    let tail_start = all_lines.len().saturating_sub(TAIL_LINES);
    for line in all_lines.iter().skip(tail_start) {
        let truncated = crate::scrollback::truncate_to_display_width(line, budget);
        let (line_color, attrs) = if display.is_error {
            (color, primary_result_attrs())
        } else {
            (secondary, secondary_result_attrs())
        };
        lines.push(ScrollbackLine::styled(
            vec![HistorySegment::styled(
                format!("{}{truncated}", result_line_prefix(icon, false)),
                line_color,
                attrs,
            )],
            None,
        ));
    }

    lines
}

fn is_diff_content(content: &str, tool_name: Option<&str>) -> bool {
    match tool_name {
        Some("edit") | Some("diff") => true,
        _ => content.lines().any(|line| {
            line.starts_with("diff --git")
                || line.starts_with("@@")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
        }),
    }
}

fn diff_line_style(line: &str) -> Option<(Option<CColor>, HistoryAttrs)> {
    if line.starts_with('+') && !line.starts_with("+++") {
        Some((
            to_crossterm_color(semantic::TEXT_SUCCESS),
            HistoryAttrs::default(),
        ))
    } else if line.starts_with('-') && !line.starts_with("---") {
        Some((
            to_crossterm_color(semantic::TEXT_ERROR),
            HistoryAttrs::default(),
        ))
    } else if line.starts_with("@@") {
        Some((
            to_crossterm_color(semantic::TEXT_ACCENT),
            HistoryAttrs::default(),
        ))
    } else {
        None
    }
}

fn result_line_style(
    display: &ToolResultDisplay,
    _line_index: usize,
    primary_color: Option<CColor>,
) -> (Option<CColor>, HistoryAttrs) {
    if display.is_error {
        return (primary_color, primary_result_attrs());
    }

    (secondary_result_color(), secondary_result_attrs())
}

fn result_line_prefix(icon: &str, is_first_line: bool) -> String {
    if icon.trim().is_empty() {
        "   ".to_string()
    } else if is_first_line {
        format!("   {icon} ")
    } else {
        "     ".to_string()
    }
}

fn primary_result_attrs() -> HistoryAttrs {
    HistoryAttrs {
        bold: true,
        ..HistoryAttrs::default()
    }
}

fn secondary_result_attrs() -> HistoryAttrs {
    HistoryAttrs::default()
}

pub(crate) fn secondary_result_color() -> Option<CColor> {
    Some(CColor::Rgb {
        r: 0x9A,
        g: 0xA4,
        b: 0xB2,
    })
}

pub(crate) fn build_tool_call_scrollback_line(tool_call: &ToolCallDisplay) -> ScrollbackLine {
    let args_str = serde_json::to_string_pretty(&tool_call.arguments)
        .unwrap_or_else(|_| tool_call.arguments.to_string());
    let args_summary =
        summarize_tool_args(&tool_call.tool_name, &args_str, &tool_call.summary_fields);
    let provenance_marker = match &tool_call.provenance {
        ToolProvenance::Native => None,
        ToolProvenance::McpRemote { server } => Some(format!("[mcp:{}]", server)),
        ToolProvenance::Plugin {
            name,
            version,
            carrier,
        } => Some(format!("[plugin:{}@{}/{}]", name, version, carrier)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use talos_conversation::ToolCallDisplay;

    fn make_display(provenance: ToolProvenance) -> ToolCallDisplay {
        ToolCallDisplay {
            tool_name: "custom_tool".to_string(),
            arguments: serde_json::json!({}),
            provenance,
            summary_fields: vec![],
        }
    }

    #[test]
    fn plugin_provenance_scrollback_marker() {
        let display = make_display(ToolProvenance::Plugin {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            carrier: "wasm".to_string(),
        });
        let line = build_tool_call_scrollback_line(&display);
        assert!(line.text.contains("[plugin:my-plugin@0.1.0/wasm]"));
    }

    #[test]
    fn native_provenance_has_no_marker() {
        let display = make_display(ToolProvenance::Native);
        let line = build_tool_call_scrollback_line(&display);
        assert!(!line.text.contains("[mcp:"));
        assert!(!line.text.contains("[plugin:"));
    }

    #[test]
    fn mcp_provenance_scrollback_marker_unchanged() {
        let display = make_display(ToolProvenance::McpRemote {
            server: "github".to_string(),
        });
        let line = build_tool_call_scrollback_line(&display);
        assert!(line.text.contains("[mcp:github]"));
    }

    #[test]
    fn tool_result_success_single_line_rendering() {
        let display = ToolResultDisplay {
            tool_name: Some("test_tool".to_string()),
            content: "output line".to_string(),
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
        assert_eq!(lines.len(), 1);

        assert!(!lines[0].text.contains('✓'));
        assert_eq!(lines[0].segments[0].fg, secondary_result_color());
        assert!(!lines[0].segments[0].attrs.bold);

        assert_eq!(lines[0].text, "   output line");
    }

    #[test]
    fn tool_result_error_rendering_unchanged() {
        let display = ToolResultDisplay {
            tool_name: Some("test_tool".to_string()),
            content: "error line".to_string(),
            is_error: true,
        };
        let lines = build_tool_result_scrollback_lines(&display, "✗", Some(CColor::Red), 120);
        assert_eq!(lines.len(), 1);

        assert!(lines[0].text.contains('✗'));
        assert_eq!(lines[0].segments[0].fg, Some(CColor::Red));
        assert!(lines[0].segments[0].attrs.bold);
        assert!(lines[0].text.starts_with("   ✗ "));
    }

    #[test]
    fn tool_result_success_special_cases_rendering() {
        let display_empty = ToolResultDisplay {
            tool_name: Some("test_tool".to_string()),
            content: "".to_string(),
            is_error: false,
        };
        let lines_empty =
            build_tool_result_scrollback_lines(&display_empty, "", Some(CColor::Green), 120);
        assert_eq!(lines_empty.len(), 1);
        assert!(!lines_empty[0].text.contains('✓'));
        assert_eq!(lines_empty[0].text, "   (no output)");
        assert_eq!(lines_empty[0].segments[0].fg, secondary_result_color());
        assert!(!lines_empty[0].segments[0].attrs.bold);

        let display_suppressed = ToolResultDisplay {
            tool_name: Some("read".to_string()),
            content: "line 1\nline 2".to_string(),
            is_error: false,
        };
        let lines_suppressed =
            build_tool_result_scrollback_lines(&display_suppressed, "", Some(CColor::Green), 120);
        assert_eq!(lines_suppressed.len(), 1);
        assert!(!lines_suppressed[0].text.contains('✓'));
        assert!(lines_suppressed[0].text.starts_with("   read 2 lines"));
        assert_eq!(lines_suppressed[0].segments[0].fg, secondary_result_color());
        assert!(!lines_suppressed[0].segments[0].attrs.bold);
    }

    #[test]
    fn tool_result_edit_diff_gets_semantic_styling() {
        let display = ToolResultDisplay {
            tool_name: Some("edit".to_string()),
            content: "edited src/main.rs\ndiff:\n- old line\n+ new line".to_string(),
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", None, 120);
        assert_eq!(lines.len(), 4);

        // "edited src/main.rs" — not a diff line, default styling
        assert_eq!(lines[0].segments[0].fg, secondary_result_color());

        // "diff:" — not a diff line, default styling
        assert_eq!(lines[1].segments[0].fg, secondary_result_color());

        // "- old line" — removed, red foreground
        assert_eq!(
            lines[2].segments[0].fg,
            to_crossterm_color(semantic::TEXT_ERROR)
        );

        // "+ new line" — added, green foreground
        assert_eq!(
            lines[3].segments[0].fg,
            to_crossterm_color(semantic::TEXT_SUCCESS)
        );
    }

    #[test]
    fn tool_result_unified_diff_gets_semantic_styling() {
        let display = ToolResultDisplay {
            tool_name: Some("git_diff".to_string()),
            content: "diff --git a/foo b/foo\n--- a/foo\n+++ b/foo\n@@ -1 +1 @@\n-old\n+new\n context line".to_string(),
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", None, 120);
        assert_eq!(lines.len(), 7);

        // "-old" — removed
        assert_eq!(
            lines[4].segments[0].fg,
            to_crossterm_color(semantic::TEXT_ERROR)
        );
        // "+new" — added
        assert_eq!(
            lines[5].segments[0].fg,
            to_crossterm_color(semantic::TEXT_SUCCESS)
        );
        // " context line" — not a diff line, default
        assert_eq!(lines[6].segments[0].fg, secondary_result_color());
    }

    #[test]
    fn tool_result_prose_with_dash_not_styled_as_diff() {
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            content: "- this is a bullet\n+ another bullet\nnormal text".to_string(),
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", None, 120);
        assert_eq!(lines.len(), 3);

        // All lines use default styling — no diff markers present and tool is not edit/diff
        for line in &lines {
            assert_eq!(line.segments[0].fg, secondary_result_color());
        }
    }

    // ── Schedule tool ratatui Buffer rendering (I126 acceptance) ────────

    fn render_tool_result_to_buffer(
        display: &ToolResultDisplay,
        width: u16,
    ) -> (ratatui::buffer::Buffer, u16) {
        let lines = build_tool_result_scrollback_lines(display, "", None, 120);
        let text = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        // Eight rows leaves room for the longest accepted schedule outcome to
        // wrap at 40 columns while exercising the real scrollback formatter.
        let height = 8;
        let area = ratatui::layout::Rect::new(0, 0, width, height);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        use ratatui::widgets::{Paragraph, Widget, Wrap};
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .render(area, &mut buf);
        (buf, height)
    }

    fn buffer_text(buf: &ratatui::buffer::Buffer, width: u16, height: u16) -> String {
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn schedule_list_buffer_renders_at_all_widths() {
        let display = ToolResultDisplay {
            tool_name: Some("list_scheduled_tasks".to_string()),
            content: "2 active task(s):\n  sched_1 | one-shot | next: 45s\n  sched_2 | recurring (30s) | next: 12s".to_string(),
            is_error: false,
        };
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_tool_result_to_buffer(&display, width);
            let content = buffer_text(&buf, width, h);
            assert!(
                content.contains("sched_1"),
                "list at {width} cols must retain task ID: {content:?}"
            );
            assert!(
                content.contains("one-shot"),
                "list at {width} cols must retain kind: {content:?}"
            );
            assert!(
                content.contains("next: 45s"),
                "list at {width} cols must retain timing/actionable state: {content:?}"
            );
        }
    }

    #[test]
    fn schedule_cancelled_buffer_renders_at_all_widths() {
        let display = ToolResultDisplay {
            tool_name: Some("cancel_scheduled_task".to_string()),
            content: "Task sched_1 cancelled. No further fires will occur.".to_string(),
            is_error: false,
        };
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_tool_result_to_buffer(&display, width);
            let content = buffer_text(&buf, width, h);
            assert!(
                content.contains("sched_1"),
                "cancelled at {width} cols must retain task ID: {content:?}"
            );
            assert!(
                content.contains("cancelled"),
                "cancelled at {width} cols must retain state/actionable result: {content:?}"
            );
        }
    }

    #[test]
    fn schedule_not_found_buffer_renders_at_all_widths() {
        let display = ToolResultDisplay {
            tool_name: Some("cancel_scheduled_task".to_string()),
            content: "Task sched_99 not found or already completed.".to_string(),
            is_error: false,
        };
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_tool_result_to_buffer(&display, width);
            let content = buffer_text(&buf, width, h);
            assert!(
                content.contains("sched_99"),
                "not-found at {width} cols must retain task ID: {content:?}"
            );
            assert!(
                content.contains("not found"),
                "not-found at {width} cols must retain state/actionable result: {content:?}"
            );
        }
    }

    #[test]
    fn schedule_unavailable_buffer_renders_at_all_widths() {
        let display = ToolResultDisplay {
            tool_name: Some("cancel_scheduled_task".to_string()),
            content: "scheduler is not available".to_string(),
            is_error: true,
        };
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_tool_result_to_buffer(&display, width);
            let content = buffer_text(&buf, width, h);
            assert!(
                content.contains("scheduler"),
                "unavailable at {width} cols must retain state subject: {content:?}"
            );
            assert!(
                content.contains("not available"),
                "unavailable at {width} cols must retain actionable state: {content:?}"
            );
        }
    }

    #[test]
    fn schedule_empty_list_buffer_renders_at_all_widths() {
        let display = ToolResultDisplay {
            tool_name: Some("list_scheduled_tasks".to_string()),
            content: "No active scheduled tasks.".to_string(),
            is_error: false,
        };
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_tool_result_to_buffer(&display, width);
            let content = buffer_text(&buf, width, h);
            assert!(
                content.contains("No active"),
                "empty list at {width} cols must retain message: {content:?}"
            );
        }
    }

    // TUI-034: Adaptive width tests

    #[test]
    fn long_line_visible_at_wide_viewport_without_ellipsis() {
        let long = "a".repeat(140);
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            content: long.clone(),
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", None, 160);
        assert_eq!(lines.len(), 1);
        assert!(
            !lines[0].text.contains('…'),
            "at 160 cols a 140-char line should not be ellipsized: {}",
            lines[0].text.len()
        );
    }

    #[test]
    fn long_line_truncated_at_narrow_viewport_with_ellipsis() {
        let long = "a".repeat(200);
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            content: long,
            is_error: false,
        };
        let lines = build_tool_result_scrollback_lines(&display, "", None, 40);
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].text.contains('…'),
            "at 40 cols a 200-char line must be ellipsized"
        );
    }

    #[test]
    fn cjk_output_uses_display_width_not_char_count() {
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            content: "你好".repeat(50),
            is_error: false,
        };
        let lines_narrow = build_tool_result_scrollback_lines(&display, "", None, 40);
        let lines_wide = build_tool_result_scrollback_lines(&display, "", None, 160);
        assert!(
            lines_narrow[0].text.contains('…'),
            "CJK at 40 cols must be truncated"
        );
        assert!(
            lines_wide[0].text.chars().count() > lines_narrow[0].text.chars().count(),
            "CJK at 160 cols must show more content than at 40 cols"
        );
    }

    #[test]
    fn head_tail_behavior_preserved_at_all_widths() {
        let content: String = (1..=40)
            .map(|i| format!("line {i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            content,
            is_error: false,
        };
        for width in [40u16, 80, 120, 160] {
            let lines = build_tool_result_scrollback_lines(&display, "", None, width);
            assert!(
                lines.len() >= HEAD_LINES + 1 + TAIL_LINES,
                "head+tail+omitted must be present at {width} cols"
            );
            assert!(
                lines.iter().any(|l| l.text.contains("lines omitted")),
                "omitted counter must appear at {width} cols"
            );
            assert!(
                lines[0].text.contains("line 01"),
                "first head line must be present at {width} cols"
            );
            assert!(
                lines.last().unwrap().text.contains("line 40"),
                "last tail line must be present at {width} cols"
            );
        }
    }
}
