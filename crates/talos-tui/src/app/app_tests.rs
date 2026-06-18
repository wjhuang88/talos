use crossterm::style::Color as CColor;
use talos_conversation::{MessageSource, ToolResultDisplay};
use talos_core::message::Message;

use crate::app::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState};
use crate::scrollback;
use crate::stream_markdown::{HoldStatus, MarkdownBlockKind};
use crate::theme::{semantic, to_crossterm_color};
use crate::tool_display;

fn state_line(text: &str) -> ScrollbackLine {
    ScrollbackLine::plain(text, None)
}

#[test]
fn truncate_to_width_ascii() {
    assert_eq!(scrollback::truncate_end_to_width("hello world", 5), "world");
}

#[test]
fn truncate_to_width_cjk() {
    assert_eq!(scrollback::truncate_end_to_width("你好世界", 4), "世界");
}

#[test]
fn truncate_to_width_short_enough() {
    assert_eq!(scrollback::truncate_end_to_width("hi", 10), "hi");
}

#[test]
fn approval_summary_uses_tool_summary_fields() {
    let args = serde_json::json!({
        "command": "cd /repo && git status --short",
        "other": "hidden"
    });
    let args_str = serde_json::to_string_pretty(&args).unwrap();
    let summary = tool_display::summarize_tool_args("bash", &args_str, &["command".to_string()]);

    assert_eq!(summary, "command: cd /repo && git status --short");
    assert!(!summary.contains('{'));
    assert!(!summary.contains("other"));
}

#[test]
fn tool_result_scrollback_keeps_multiple_lines() {
    let display = ToolResultDisplay {
        tool_name: Some("tree".to_string()),
        is_error: false,
        content: "├── backend/\n├── frontend/\n└── docs/".to_string(),
    };
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "✓", Some(CColor::Green));

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].text, "   ✓ ├── backend/");
    assert_eq!(lines[1].text, "     ├── frontend/");
    assert_eq!(lines[2].text, "     └── docs/");
}

#[test]
fn read_tool_result_hides_content_from_scrollback() {
    let display = ToolResultDisplay {
        tool_name: Some("read".to_string()),
        is_error: false,
        content: "secret line\nanother line\n".to_string(),
    };

    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "✓", Some(CColor::Green));

    assert_eq!(lines.len(), 1);
    assert!(lines[0].text.contains("2 lines"));
    assert!(!lines[0].text.contains("secret line"));
}

#[test]
fn read_tool_error_result_remains_visible() {
    let display = ToolResultDisplay {
        tool_name: Some("read".to_string()),
        is_error: true,
        content: "file not found".to_string(),
    };

    let lines = tool_display::build_tool_result_scrollback_lines(&display, "✗", Some(CColor::Red));

    assert_eq!(lines.len(), 1);
    assert!(lines[0].text.contains("file not found"));
}

#[test]
fn stream_render_state_tracks_lines_and_preview() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    assert_eq!(state.push_chunk("first\nsec"), vec![state_line(" ● first")]);
    assert_eq!(state.preview(), "sec");
    assert_eq!(
        state.push_chunk("ond\nthird"),
        vec![state_line("   second")]
    );
    assert_eq!(state.finish(), vec![state_line("   third")]);
    assert!(state.source().is_none());
    assert_eq!(state.preview(), "");
}

#[test]
fn stream_render_state_wraps_user_blocks_with_background_rows() {
    let mut state = StreamRenderState::default();
    let bg = scrollback::stream_bg_for(Some(&MessageSource::User));

    assert_eq!(
        state.start(MessageSource::User),
        vec![ScrollbackLine::plain(String::new(), bg)]
    );
    assert_eq!(
        state.finish(),
        vec![ScrollbackLine::plain(String::new(), bg)]
    );

    state.reset();
    assert!(state.source().is_none());
    assert_eq!(state.preview(), "");
}

#[test]
fn stream_render_state_can_hold_complete_lines_until_finish() {
    let mut state = StreamRenderState::default();
    assert!(
        state
            .start_with_hold(MessageSource::Assistant, true)
            .is_empty()
    );

    assert!(state.push_chunk("first\nsecond\nthi").is_empty());
    assert_eq!(state.preview(), "thi");
    assert!(state.push_chunk("rd").is_empty());
    assert_eq!(state.preview(), "third");

    assert_eq!(
        state.finish(),
        vec![
            state_line(" ● first"),
            state_line("   second"),
            state_line("   third")
        ]
    );
    assert!(state.source().is_none());
    assert_eq!(state.preview(), "");
}

#[test]
fn stream_render_state_holds_table_and_flushes_aligned_rows() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    assert!(state.push_chunk("| **A** | Longer `code` |\n").is_empty());
    assert_eq!(state.preview(), "rendering table...");
    assert!(state.push_chunk("| --- | --- |\n").is_empty());
    assert_eq!(state.preview(), "rendering table...");
    assert!(state.push_chunk("| x | yy |\n").is_empty());

    let lines = state.finish();
    assert_eq!(lines.len(), 5, "header + sep + 2 rows + footer");
    assert!(lines[0].text.contains("╭"), "rounded top border");
    assert!(lines[2].text.contains("┼"), "separator");
    assert!(lines[4].text.contains("╰"), "rounded bottom border");
    assert!(
        lines[1]
            .segments
            .iter()
            .any(|segment| segment.text == "A" && segment.attrs.bold)
    );
    assert!(
        lines[1]
            .segments
            .iter()
            .any(|segment| segment.text == "code"
                && segment.fg == to_crossterm_color(semantic::MARKDOWN_CODE))
    );
    assert_eq!(state.preview(), "");
}

#[test]
fn markdown_hold_preview_animates_text_and_color() {
    let status = HoldStatus {
        kind: MarkdownBlockKind::Table,
        lines: 2,
        bytes: 24,
        boundary_hint: crate::stream_markdown::BoundaryHint::TableEnd,
    };

    assert_eq!(
        scrollback::animated_hold_preview_text(&status, 0),
        "rendering table"
    );
    assert_eq!(
        scrollback::animated_hold_preview_text(&status, 2),
        "rendering table."
    );
    assert_eq!(
        scrollback::animated_hold_preview_text(&status, 4),
        "rendering table.."
    );
    assert_eq!(
        scrollback::animated_hold_preview_text(&status, 6),
        "rendering table..."
    );
    assert_eq!(
        scrollback::hold_preview_color(0),
        scrollback::hold_preview_color(1)
    );
    assert_ne!(
        scrollback::hold_preview_color(0),
        scrollback::hold_preview_color(2)
    );
}

#[test]
fn preview_spinner_uses_canon_rhythm() {
    let n = SPINNER_FRAMES.len();
    let phase = n / 2;

    let (p0, _) = scrollback::preview_spinner_padding(0, 0);
    let (p1, _) = scrollback::preview_spinner_padding(1, 0);

    let lead0 = SPINNER_FRAMES[(n - phase) % n];
    let chase0 = SPINNER_FRAMES[0];
    assert_eq!(p0, format!(" {lead0}{chase0}"));

    let lead1 = SPINNER_FRAMES[(1 + n - phase) % n];
    let chase1 = SPINNER_FRAMES[1];
    assert_eq!(p1, format!(" {lead1}{chase1}"));

    assert_ne!(lead0, chase0);
}

#[test]
fn stream_render_state_renders_code_fence_on_finish() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    let mut lines = state.push_chunk("```rust\nfn main() {}\n```\n");
    lines.extend(state.finish());

    assert!(!lines.is_empty(), "code block lines returned");
}

#[test]
fn render_code_block_produces_header_and_line_numbers() {
    let block_lines = vec![
        "```rust".to_string(),
        "fn main() {}".to_string(),
        "```".to_string(),
    ];
    let result = scrollback::render_code_block(&block_lines, None);
    assert_eq!(result.len(), 3, "header + one code line + footer");
    assert!(result[0].text.contains("rust"), "language label");
    assert!(result[1].text.contains("1"), "line number");
    assert!(result[1].text.contains("fn main() {}"), "code content");
}

#[test]
fn mermaid_block_renders_diagram() {
    let src = "flowchart LR\n    A[Start] --> B[End]";
    let result = scrollback::render_mermaid_block(src, None);
    assert!(!result.is_empty(), "should produce output lines");
    assert!(
        result[0].text.contains("mermaid"),
        "should have mermaid header"
    );
}

#[test]
fn mermaid_block_falls_back_on_invalid_syntax() {
    let src = "this is not valid mermaid at all";
    let result = scrollback::render_mermaid_block(src, None);
    assert!(!result.is_empty(), "fallback should still produce output");
}

#[test]
fn stream_render_state_renders_inline_markdown_segments() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    let lines = state.push_chunk("# Title with **strong** and `code`\n");

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, " ● Title with strong and code");
    assert!(lines[0].segments.iter().any(|segment| segment.attrs.bold));
    assert!(
        lines[0]
            .segments
            .iter()
            .any(|segment| segment.text == "code"
                && segment.fg == to_crossterm_color(semantic::MARKDOWN_CODE))
    );
}

#[test]
fn stream_render_state_renders_horizontal_rule() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    let lines = state.push_chunk("---\n");

    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].text.starts_with(" ● ─"),
        "horizontal rule with prefix and dashes"
    );
    assert!(
        lines[0].fill.is_some(),
        "horizontal rule should fill the history row"
    );

    let mut segments = lines[0].segments.clone();
    scrollback::append_fill_segment(&mut segments, lines[0].fill.clone().unwrap(), 20, 3);
    assert_eq!(scrollback::history_segments_width(&segments), 20);
}

#[test]
fn stream_render_state_styles_block_markdown_rows() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    assert!(state.push_chunk("- **first**\n").is_empty());
    assert!(state.push_chunk("- second\n").is_empty());
    let lines = state.finish();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, " ● - first");
    assert_eq!(lines[1].text, "   - second");
    assert!(
        lines[0]
            .segments
            .iter()
            .any(|segment| segment.text == "- " && segment.attrs.bold)
    );
}

#[test]
fn stream_render_state_keeps_user_markdown_literal() {
    let mut state = StreamRenderState::default();
    let bg = scrollback::stream_bg_for(Some(&MessageSource::User));
    assert_eq!(
        state.start(MessageSource::User),
        vec![ScrollbackLine::plain(String::new(), bg)]
    );

    let lines = state.push_chunk("# literal **user** `input`\n");

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, " > # literal **user** `input`");
    assert!(
        lines[0]
            .segments
            .iter()
            .all(|segment| !segment.attrs.italic)
    );
}

#[test]
fn stream_opening_lines_adds_separator_only_after_first_stream() {
    let bg = scrollback::stream_bg_for(Some(&MessageSource::User));
    let opening = vec![ScrollbackLine::plain(String::new(), bg)];

    assert_eq!(
        scrollback::stream_opening_lines(0, opening.clone()),
        opening
    );
    assert_eq!(
        scrollback::stream_opening_lines(1, opening.clone()),
        vec![
            ScrollbackLine::plain(String::new(), None),
            ScrollbackLine::plain(String::new(), bg)
        ]
    );
}

#[test]
fn render_history_message_reuses_completed_stream_rendering() {
    let mut stream_count = 0;
    let lines = scrollback::render_history_message(
        &mut stream_count,
        MessageSource::Assistant,
        "hello\n| A | B |\n| --- | --- |\n| x | y |",
    );

    assert_eq!(stream_count, 1);
    assert_eq!(lines[0].text, " ● hello");
    assert!(lines.iter().any(|line| line.text.contains("╭")));
    assert!(
        lines
            .iter()
            .any(|line| line.text.contains("│ x") && line.text.contains("│ y"))
    );
}

#[test]
fn hydrate_history_preserves_prefixes_and_stream_count() {
    let mut stream_count = 0;
    let lines = scrollback::render_history_messages(
        &mut stream_count,
        &[
            Message::User {
                content: "first\nsecond".to_string(),
            },
            Message::Assistant {
                content: "reply".to_string(),
                tool_calls: vec![],
            },
        ],
    );

    let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
    assert!(texts.contains(&" > first"));
    assert!(texts.contains(&"   second"));
    assert!(texts.contains(&" ● reply"));
    assert_eq!(stream_count, 2);
}

// --- tool result summarization tests ---

#[test]
fn read_always_summarized() {
    let display = ToolResultDisplay {
        tool_name: Some("read".to_string()),
        is_error: false,
        content: "single line".to_string(),
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "read 1 line, 11 bytes");
}

#[test]
fn read_error_not_suppressed() {
    let display = ToolResultDisplay {
        tool_name: Some("read".to_string()),
        is_error: true,
        content: "permission denied".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn list_symbols_always_summarized() {
    let display = ToolResultDisplay {
        tool_name: Some("list_symbols".to_string()),
        is_error: false,
        content: "[{\"name\": \"foo\", \"kind\": \"function\"}]\n".to_string(),
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "found 1 symbol");
}

#[test]
fn find_symbol_always_summarized() {
    let content = serde_json::to_string_pretty(&serde_json::json!([
        {"name": "App", "kind": "struct"},
        {"name": "App", "kind": "impl"}
    ]))
    .unwrap();
    let display = ToolResultDisplay {
        tool_name: Some("find_symbol".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "found 2 matching symbols");
}

#[test]
fn find_references_always_summarized() {
    let content = serde_json::to_string_pretty(&serde_json::json!([
        {"file": "main.rs", "line": 10},
        {"file": "main.rs", "line": 25},
        {"file": "lib.rs", "line": 5}
    ]))
    .unwrap();
    let display = ToolResultDisplay {
        tool_name: Some("find_references".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "found 3 references");
}

#[test]
fn glob_under_threshold_not_summarized() {
    let content = "src/main.rs\nsrc/lib.rs\nCargo.toml\n";
    let display = ToolResultDisplay {
        tool_name: Some("glob".to_string()),
        is_error: false,
        content: content.to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn glob_over_threshold_summarized() {
    let content = (0..35)
        .map(|i| format!("src/file_{i}.rs"))
        .collect::<Vec<_>>()
        .join("\n");
    let display = ToolResultDisplay {
        tool_name: Some("glob".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert!(summary.contains("35 files"));
    assert!(summary.contains("bytes"));
}

#[test]
fn ls_under_threshold_not_summarized() {
    let display = ToolResultDisplay {
        tool_name: Some("ls".to_string()),
        is_error: false,
        content: "drwxr-xr-x  src\n-rw-r--r--  Cargo.toml\n".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn ls_over_threshold_summarized() {
    let content = (0..35)
        .map(|i| format!("-rw-r--r--  file_{i}.txt"))
        .collect::<Vec<_>>()
        .join("\n");
    let display = ToolResultDisplay {
        tool_name: Some("ls".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert!(summary.contains("35 entries"));
}

#[test]
fn list_imports_under_threshold_not_summarized() {
    let display = ToolResultDisplay {
        tool_name: Some("list_imports".to_string()),
        is_error: false,
        content: "[{\"module\": \"std::fs\"}]\n".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn list_imports_over_threshold_summarized() {
    let imports: Vec<_> = (0..35)
        .map(|i| serde_json::json!({"module": format!("mod_{i}")}))
        .collect();
    let content = serde_json::to_string_pretty(&imports).unwrap();
    let display = ToolResultDisplay {
        tool_name: Some("list_imports".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "found 35 imports");
}

#[test]
fn unknown_tool_not_suppressed() {
    let display = ToolResultDisplay {
        tool_name: Some("bash".to_string()),
        is_error: false,
        content: "output\n".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn summarize_symbol_results_fallback_on_invalid_json() {
    let content = "not json\nline two\nline three\n";
    let summary = tool_display::summarize_symbol_results(content, "symbols");
    assert_eq!(summary, "found 3 symbols");
}

#[test]
fn suppressed_summary_fallback_for_unknown_tool() {
    let display = ToolResultDisplay {
        tool_name: Some("unknown_tool".to_string()),
        is_error: false,
        content: "line one\nline two\n".to_string(),
    };
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert_eq!(summary, "2 lines, 18 bytes");
}
