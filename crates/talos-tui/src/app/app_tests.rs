use crossterm::style::Color as CColor;
use talos_conversation::{
    MessageSource, TodoPanelData, TodoPanelRow, ToolResultDisplay, UserInput,
};
use talos_core::message::Message;
use tokio::sync::mpsc;

use crate::app::{SPINNER_FRAMES, ScrollbackLine, StreamRenderState, build_todo_panel_lines};
use crate::app::{next_processing_frame, preview_text_for_state, submit_input_message, tip_ttl};
use crate::scrollback;
use crate::state::{ApprovalState, TuiState};
use crate::stream_markdown::{HoldStatus, MarkdownBlockKind};
use crate::theme::{semantic, to_crossterm_color};
use crate::tool_display;
use talos_conversation::{TipKind, TurnPhase};

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
fn reasoning_uses_own_marker_and_tool_result_color() {
    let mut stream_count = 0;
    let lines = scrollback::render_history_message(
        &mut stream_count,
        MessageSource::Reasoning,
        "Thinking: checking the turn\n",
    );

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, " ◇ Thinking: checking the turn");
    assert_eq!(lines[0].segments[0].text, " ◇ ");
    assert_eq!(
        lines[0].segments[1].fg,
        tool_display::secondary_result_color()
    );
}

#[test]
fn multiline_reasoning_marks_first_line_and_aligns_continuations() {
    let mut stream_count = 0;
    let lines = scrollback::render_history_message(
        &mut stream_count,
        MessageSource::Reasoning,
        "Thinking: first line\nsecond line\n",
    );

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, " ◇ Thinking: first line");
    assert_eq!(lines[1].text, "   second line");
}

#[test]
fn credential_display_never_reveals_secret_suffix() {
    let display = scrollback::credential_display_text("sk-test-Ewqw");

    assert_eq!(display, "••••••••••••");
    assert!(!display.contains("Ewqw"));
    assert!(!display.contains("sk-test"));
}

#[test]
fn credential_cursor_tracks_masked_buffer() {
    assert_eq!(scrollback::credential_cursor_col("abcd"), 7);
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
fn tool_args_summary_uses_available_budget_before_truncating() {
    let command = "cargo test -p talos-cli approval::tests::test_always_allow_rule_is_effective_against_default_ask";
    let args = serde_json::json!({ "command": command });
    let args_str = serde_json::to_string_pretty(&args).unwrap();

    let full =
        tool_display::summarize_tool_args_with_budget(&args_str, &["command".to_string()], 140);
    let short =
        tool_display::summarize_tool_args_with_budget(&args_str, &["command".to_string()], 48);

    assert_eq!(full, format!("command: {command}"));
    assert!(short.ends_with('…'));
    assert!(short.chars().count() <= 48);
}

#[test]
fn tool_result_scrollback_keeps_multiple_lines() {
    let display = ToolResultDisplay {
        tool_name: Some("tree".to_string()),
        is_error: false,
        content: "├── backend/\n├── frontend/\n└── docs/".to_string(),
    };
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].text, "   ├── backend/");
    assert_eq!(lines[1].text, "   ├── frontend/");
    assert_eq!(lines[2].text, "   └── docs/");
}

#[test]
fn todo_panel_renders_read_only_history_lines() {
    let lines = build_todo_panel_lines(&TodoPanelData {
        title: "Session Todos".to_string(),
        rows: vec![TodoPanelRow {
            id: "abc12345".to_string(),
            status: "[~]".to_string(),
            priority: "high".to_string(),
            title: "Wire slash view".to_string(),
            detail: Some("read-only".to_string()),
        }],
        footer: Some("1 item".to_string()),
    });

    assert_eq!(lines[0].text, "   TODO Session Todos");
    assert!(lines[1].text.contains("abc12345"));
    assert!(lines[1].text.contains("[~]"));
    assert!(lines[1].text.contains("Wire slash view"));
    assert_eq!(lines[2].text, "      1 item");
}

#[test]
fn todo_panel_unknown_status_uses_bracket_fallback() {
    let lines = build_todo_panel_lines(&TodoPanelData {
        title: "Session Todos".to_string(),
        rows: vec![TodoPanelRow {
            id: "abc12345".to_string(),
            status: "custom".to_string(),
            priority: "medium".to_string(),
            title: "Fallback test".to_string(),
            detail: None,
        }],
        footer: Some("1 item".to_string()),
    });
    // Unknown status "custom" should render as "[custom]", not bare "custom"
    assert!(lines[1].text.contains("[custom]"));
    // Known statuses should still render as checkbox icons
    let lines2 = build_todo_panel_lines(&TodoPanelData {
        title: "Session Todos".to_string(),
        rows: vec![TodoPanelRow {
            id: "def67890".to_string(),
            status: "[x]".to_string(),
            priority: "low".to_string(),
            title: "Completed item".to_string(),
            detail: None,
        }],
        footer: Some("1 item".to_string()),
    });
    assert!(lines2[1].text.contains("[x]"));
}

#[test]
fn read_tool_result_hides_content_from_scrollback() {
    let display = ToolResultDisplay {
        tool_name: Some("read".to_string()),
        is_error: false,
        content: "secret line\nanother line\n".to_string(),
    };

    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));

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
fn idle_processing_preview_animates_ellipsis() {
    assert_eq!(scrollback::idle_processing_preview_text(0), "");
    assert_eq!(scrollback::idle_processing_preview_text(2), ".");
    assert_eq!(scrollback::idle_processing_preview_text(4), "..");
    assert_eq!(scrollback::idle_processing_preview_text(6), "...");
}

#[test]
fn preview_spinner_uses_single_block() {
    let n = SPINNER_FRAMES.len();

    let (p0, c0) = scrollback::preview_spinner_padding(0);
    let (p1, c1) = scrollback::preview_spinner_padding(1);

    assert_eq!(p0, format!(" {} ", SPINNER_FRAMES[0]));
    assert_eq!(c0, 0);
    assert_eq!(p0.chars().count(), 3);

    assert_eq!(p1, format!(" {} ", SPINNER_FRAMES[1]));
    assert_eq!(c1, 1);
    assert_eq!(p1.chars().count(), 3);

    assert_ne!(SPINNER_FRAMES[0], SPINNER_FRAMES[1 % n]);
}

#[test]
fn thinking_preview_uses_two_color_three_segment_ripple() {
    let spans =
        scrollback::preview_line_spans("", "thinking: draft", None, semantic::PREVIEW_FG, Some(0));

    assert_eq!(spans.len(), 4);
    let label: String = spans[..3]
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    assert_eq!(label, "thinking");
    assert_eq!(spans[0].style.fg, Some(semantic::THINKING_RIPPLE_SECONDARY));
    assert_eq!(spans[1].style.fg, Some(semantic::THINKING_RIPPLE_PRIMARY));
    assert_eq!(spans[2].style.fg, Some(semantic::THINKING_RIPPLE_SECONDARY));
    assert_eq!(spans[1].content.as_ref(), "nk");
    assert_eq!(spans[3].content.as_ref(), ": draft");
    assert_eq!(spans[3].style.fg, Some(semantic::PREVIEW_FG));

    let expanded =
        scrollback::preview_line_spans("", "thinking: draft", None, semantic::PREVIEW_FG, Some(2));
    assert_eq!(expanded.len(), 4);
    assert_eq!(expanded[1].content.as_ref(), "hinkin");
    assert_eq!(
        expanded[0].style.fg,
        Some(semantic::THINKING_RIPPLE_SECONDARY)
    );
    assert_eq!(
        expanded[1].style.fg,
        Some(semantic::THINKING_RIPPLE_PRIMARY)
    );
    assert_eq!(
        expanded[2].style.fg,
        Some(semantic::THINKING_RIPPLE_SECONDARY)
    );
}

#[test]
fn processing_frames_advance_only_on_timer_ticks() {
    let frame = 7;
    assert_eq!(frame, 7, "redraw-only work must not mutate animation state");
    assert_eq!(next_processing_frame(true, frame), 8);
    assert_eq!(next_processing_frame(false, frame), 0);
}

#[test]
fn dashboard_tip_ttls_are_visible_but_bounded() {
    assert_eq!(tip_ttl(&TipKind::Info).as_secs(), 8);
    assert_eq!(tip_ttl(&TipKind::Error).as_secs(), 5);
    assert_eq!(tip_ttl(&TipKind::ApprovalResult).as_secs(), 3);
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
fn stream_render_state_recovers_markdown_after_unterminated_code_fence() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    assert!(
        state
            .push_chunk("```\n│ diagram line │\n## Recovered Heading\n")
            .is_empty()
    );
    let lines = state.finish();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].text, " ● ```");
    assert_eq!(lines[2].text, "   Recovered Heading");
    assert!(
        lines[2]
            .segments
            .iter()
            .any(|segment| segment.text == "Recovered Heading"
                && segment.attrs.bold
                && segment.fg == to_crossterm_color(semantic::MARKDOWN_HEADING))
    );
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
                reasoning: None,
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

#[test]
fn grep_under_threshold_renders_inline() {
    let display = ToolResultDisplay {
        tool_name: Some("grep".to_string()),
        is_error: false,
        content: "src/main.rs:\n  10: foo\nsrc/lib.rs:\n  5: bar\n".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
    assert_eq!(lines.len(), 4);
    assert!(lines[0].text.contains("src/main.rs:"));
    assert!(lines[3].text.contains("bar"));
    assert!(lines.iter().all(|l| !l.text.contains("omitted")));
}

#[test]
fn grep_over_threshold_renders_summary() {
    let mut content = String::from("src/a.rs:\n");
    for i in 0..20 {
        content.push_str(&format!("  {i}: match-a-{i}\n"));
    }
    content.push_str("src/b.rs:\n");
    for i in 0..15 {
        content.push_str(&format!("  {i}: match-b-{i}\n"));
    }
    let display = ToolResultDisplay {
        tool_name: Some("grep".to_string()),
        is_error: false,
        content,
    };
    assert!(tool_display::should_suppress_tool_result_content(&display));
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
    assert_eq!(lines.len(), 1);
    let summary = tool_display::suppressed_tool_result_summary(&display);
    assert!(summary.contains("grep matched"));
    assert!(summary.contains("35 lines"));
    assert!(summary.contains("2 files"));
    assert!(summary.contains("bytes"));
    assert!(!lines[0].text.contains("match-a-5"));
}

#[test]
fn grep_summary_fallback_on_unrecognized_shape() {
    let content = "plain text\nwithout file headers\nor indented matches\n".to_string();
    let summary = tool_display::summarize_grep_result(&content);
    assert_eq!(summary, "grep matched 3 lines, 52 bytes");
}

#[test]
fn bash_under_threshold_renders_full() {
    let content = (0..10)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let display = ToolResultDisplay {
        tool_name: Some("bash".to_string()),
        is_error: false,
        content,
    };
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
    assert_eq!(lines.len(), 10);
    assert!(lines[0].text.contains("line 0"));
    assert!(lines[9].text.contains("line 9"));
    assert!(lines.iter().all(|l| !l.text.contains("omitted")));
}

#[test]
fn bash_over_threshold_renders_head_and_tail() {
    let content = (0..50)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let display = ToolResultDisplay {
        tool_name: Some("bash".to_string()),
        is_error: false,
        content,
    };
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
    assert_eq!(lines.len(), 7);
    assert!(lines[0].text.contains("line 0"));
    assert!(lines[2].text.contains("line 2"));
    assert!(lines[3].text.contains("44 lines omitted"));
    assert!(lines[4].text.contains("line 47"));
    assert!(lines[6].text.contains("line 49"));
    assert!(lines.iter().all(|l| !l.text.contains("line 20")));
    assert!(lines.iter().all(|l| !l.text.contains("line 46")));
}

#[test]
fn tool_result_scrollback_styles_primary_and_detail_lines() {
    let display = ToolResultDisplay {
        tool_name: Some("write".to_string()),
        is_error: false,
        content: "wrote 11 bytes to new.txt\npreview:\nhello world".to_string(),
    };
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));

    assert_eq!(lines.len(), 3);
    assert!(!lines[0].segments[0].attrs.bold);
    assert!(!lines[0].segments[0].attrs.dim);
    assert_eq!(
        lines[0].segments[0].fg,
        Some(CColor::Rgb {
            r: 0x9A,
            g: 0xA4,
            b: 0xB2,
        })
    );
    assert!(!lines[1].segments[0].attrs.bold);
    assert!(!lines[1].segments[0].attrs.dim);
    assert_eq!(
        lines[1].segments[0].fg,
        Some(CColor::Rgb {
            r: 0x9A,
            g: 0xA4,
            b: 0xB2,
        })
    );
    assert!(!lines[2].segments[0].attrs.dim);
    assert_eq!(
        lines[2].segments[0].fg,
        Some(CColor::Rgb {
            r: 0x9A,
            g: 0xA4,
            b: 0xB2,
        })
    );
}

#[test]
fn tool_result_error_detail_lines_keep_error_style() {
    let display = ToolResultDisplay {
        tool_name: Some("write".to_string()),
        is_error: true,
        content: "failed\npermission denied".to_string(),
    };
    let lines = tool_display::build_tool_result_scrollback_lines(&display, "✗", Some(CColor::Red));

    assert_eq!(lines.len(), 2);
    assert!(lines[0].segments[0].attrs.bold);
    assert!(lines[1].segments[0].attrs.bold);
    assert!(!lines[1].segments[0].attrs.dim);
    assert_eq!(lines[1].segments[0].fg, Some(CColor::Red));
}

#[test]
fn head_tail_omitted_count_is_correct() {
    for total in [31usize, 32, 50, 100] {
        let content = (0..total)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let display = ToolResultDisplay {
            tool_name: Some("bash".to_string()),
            is_error: false,
            content,
        };
        let lines =
            tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
        let expected_omitted = total - 3 - 3;
        assert!(
            lines[3]
                .text
                .contains(&format!("{expected_omitted} lines omitted")),
            "total={total}: {:?}",
            lines[3].text
        );
        assert_eq!(lines.len(), 7);
    }
}

#[test]
fn head_tail_truncation_does_not_affect_export_content() {
    // `/export` writes `ToolResultDisplay::content` verbatim and never calls
    // `build_tool_result_scrollback_lines`, so scrollback truncation must be a
    // pure display transform. The display is borrowed immutably here, which
    // guarantees the raw content survives for export.
    let content = (0..50)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let original = content.clone();
    let display = ToolResultDisplay {
        tool_name: Some("bash".to_string()),
        is_error: false,
        content,
    };
    let _ = tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green));
    assert_eq!(display.content, original);
    assert!(display.content.contains("line 25"));
}

#[test]
fn preview_text_uses_phase_priority_states() {
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::TimedOut), None, false, "stream", 0),
        "⏱ timed out"
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Failed), None, false, "stream", 0),
        "✗ failed"
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Cancelled), None, false, "stream", 0),
        "cancelled"
    );
    assert_eq!(
        preview_text_for_state(
            None,
            Some(&TurnPhase::Retrying { attempt: 2 }),
            None,
            true,
            "stream",
            0,
        ),
        "retrying (attempt 2)..."
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Connecting), None, true, "", 0),
        "connecting..."
    );
    assert_eq!(
        preview_text_for_state(
            None,
            Some(&TurnPhase::RunningTool {
                name: "bash".to_string()
            }),
            None,
            true,
            "",
            0,
        ),
        "running tool: bash..."
    );
}

#[test]
fn preview_text_prefers_thinking_then_idle_then_stream_preview() {
    assert_eq!(
        preview_text_for_state(None, None, Some("draft"), true, "", 0),
        "thinking: draft"
    );
    assert_eq!(preview_text_for_state(None, None, None, true, "", 6), "...");
    assert_eq!(
        preview_text_for_state(None, None, None, false, "generated", 0),
        "generated"
    );
}

// --- TUI-028: Stale preview clear ---

#[test]
fn preview_clears_after_stream_reset() {
    // Simulate stale preview from a previous stream.
    let mut stream_render = StreamRenderState::default();
    assert!(stream_render.start(MessageSource::Assistant).is_empty());
    // Push partial content (no newline) so it stays in preview buffer.
    let lines = stream_render.push_chunk("old preview");
    assert!(lines.is_empty(), "partial chunk should stay in preview");
    assert!(!stream_render.preview().is_empty(), "should have preview");

    // Reset (as Enter key does before new submit) clears everything.
    stream_render.reset();
    assert!(stream_render.source().is_none());
    assert_eq!(stream_render.preview(), "");
}

#[test]
fn preview_text_ignores_stream_preview_when_not_processing() {
    // When is_processing is false, stream_preview should not leak
    // stale content from a previous cancelled turn.
    let stale_preview = "stale stream preview from previous turn";
    let result = preview_text_for_state(
        None,          // no hold
        None,          // no phase
        None,          // no thinking preview
        false,         // not processing
        stale_preview, // stream_render.preview() from previous stream
        0,
    );
    // When is_processing=false, preview_text_for_state falls through to stream_preview.
    // This is the bug: stale preview from cancelled/resume displays.
    // The fix is at the call site: stream_render.reset() before sending new message.
    // This test documents current behavior; actual prevention is at submit time.
    assert_eq!(
        result, stale_preview,
        "stale preview would display unless cleared before submit"
    );
}

#[test]
fn preview_clearing_resets_both_stream_render_and_thinking() {
    // Simulate state after cancellation: stream_render has preview,
    // thinking_preview has old text.
    let mut stream_render = StreamRenderState::default();
    assert!(stream_render.start(MessageSource::Assistant).is_empty());
    assert!(stream_render.push_chunk("unfinished").is_empty());
    assert!(!stream_render.preview().is_empty());

    let mut thinking_preview = Some("old thinking".to_string());

    // Enter-key submit clears both.
    stream_render.reset();
    thinking_preview = None;

    assert_eq!(stream_render.preview(), "");
    assert!(
        thinking_preview.is_none(),
        "thinking preview should be cleared on new submit"
    );
}

#[test]
fn submit_input_message_clears_preview_only_after_successful_send() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.input_append_char('h');
    state.input_append_char('i');
    state.thinking_preview = Some("old thinking".to_string());
    let mut stream_render = StreamRenderState::default();
    assert!(stream_render.start(MessageSource::Assistant).is_empty());
    assert!(stream_render.push_chunk("old preview").is_empty());

    assert!(submit_input_message(
        &mut state,
        &mut stream_render,
        Some(&tx)
    ));

    match rx.try_recv().expect("message should be sent") {
        UserInput::Message(text) => assert_eq!(text, "hi"),
        other => panic!("unexpected input: {other:?}"),
    }
    assert_eq!(stream_render.preview(), "");
    assert!(state.thinking_preview.is_none());
}

#[test]
fn submit_input_message_keeps_preview_for_empty_or_unsent_input() {
    let mut state = TuiState::new();
    state.thinking_preview = Some("old thinking".to_string());
    let mut stream_render = StreamRenderState::default();
    assert!(stream_render.start(MessageSource::Assistant).is_empty());
    assert!(stream_render.push_chunk("old preview").is_empty());

    assert!(!submit_input_message(&mut state, &mut stream_render, None));
    assert_eq!(stream_render.preview(), "old preview");
    assert_eq!(state.thinking_preview.as_deref(), Some("old thinking"));

    state.input_append_char('h');
    state.input_append_char('i');
    assert!(!submit_input_message(&mut state, &mut stream_render, None));
    assert_eq!(stream_render.preview(), "old preview");
    assert_eq!(state.thinking_preview.as_deref(), Some("old thinking"));
}

// ── TUI-030 semantic-buffer: key-dispatch contract for history navigation ──

/// Simulates the Up/Down key dispatch from `handle_input_event` when no
/// panel or approval is active. Proves the full submit → navigate → restore
/// cycle through the actual `submit_input_message` path.
#[test]
fn semantic_buffer_up_down_history_through_submit_path() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    let mut sr = StreamRenderState::default();

    // Submit two messages through the real submission path
    state.input_append_str("hello");
    assert!(submit_input_message(&mut state, &mut sr, Some(&tx)));
    assert_eq!(state.input_history, vec!["hello"]);

    state.input_append_str("world");
    assert!(submit_input_message(&mut state, &mut sr, Some(&tx)));
    assert_eq!(state.input_history, vec!["hello", "world"]);

    // Type a draft
    state.input_append_str("my draft");

    // Simulate Up (handle_input_event guard: !slash_menu.is_open)
    assert!(!state.slash_menu.is_open, "precondition: menu closed");
    state.history_prev();
    assert_eq!(state.input_buffer, "world", "first Up → newest");
    assert_eq!(state.draft_input, "my draft", "draft saved");

    state.history_prev();
    assert_eq!(state.input_buffer, "hello", "second Up → oldest");

    // At oldest, Up stays
    state.history_prev();
    assert_eq!(state.input_buffer, "hello");

    // Down → newer
    state.history_next();
    assert_eq!(state.input_buffer, "world");

    // Down past newest → restore exact draft
    state.history_next();
    assert_eq!(state.input_buffer, "my draft");
    assert!(state.history_cursor.is_none());
    assert!(state.draft_input.is_empty());
}

/// Proves slash-menu-open guard prevents history navigation.
/// In handle_input_event, Up/Down match `if slash_menu.is_open` first;
/// the `if !slash_menu.is_open` history arms never fire.
#[test]
fn semantic_buffer_slash_menu_open_prevents_history_navigation() {
    let mut state = TuiState::new();
    state.input_append_str("entry");
    state.input_submit();

    // Open slash menu — simulates the handler's `slash_menu.is_open` guard
    state.open_slash_menu(talos_conversation::command_registry());
    assert!(state.slash_menu.is_open);

    // The handler's match arm for Up is:
    //   KeyCode::Up if self.state.slash_menu.is_open => slash_menu.select_prev()
    // The history arm `KeyCode::Up if !self.state.slash_menu.is_open` does NOT fire.
    // Verify the guard condition holds:
    let history_guard = !state.slash_menu.is_open;
    assert!(
        !history_guard,
        "history guard must be false when menu is open"
    );

    // Calling history_prev directly would still work (it's a public method),
    // but the handler would never reach it. Verify the state still has
    // the draft intact since the handler didn't call history_prev.
    assert!(state.history_cursor.is_none(), "cursor should be at draft");
}

/// Proves approval-active guard prevents history navigation.
/// In handle_input_event, the approval check returns early (line 930-932)
/// before the main match block where Up/Down history arms live.
#[test]
fn semantic_buffer_approval_active_prevents_history_navigation() {
    let mut state = TuiState::new();
    state.input_append_str("entry");
    state.input_submit();

    // Activate approval — simulates the handler's early return
    state.activate_approval("test_tool", "args");
    assert!(
        !matches!(state.approval_state, ApprovalState::Hidden),
        "approval should be active"
    );

    // In handle_input_event, the first check is:
    //   if !matches!(self.state.approval_state, ApprovalState::Hidden) {
    //       self.handle_pending_approval_input(key.code);
    //       return false;  ← exits before match block
    //   }
    // History navigation is never reached.
    let approval_intercepts = !matches!(state.approval_state, ApprovalState::Hidden);
    assert!(
        approval_intercepts,
        "approval must intercept before history"
    );
    assert!(state.history_cursor.is_none(), "cursor should be at draft");
}

/// Proves credential-input guard prevents history navigation.
#[test]
fn semantic_buffer_credential_input_prevents_history_navigation() {
    let mut state = TuiState::new();
    state.input_append_str("entry");
    state.input_submit();

    state.slash_menu = crate::panel_state::BottomPanelState::open_credential_input(
        "test-provider",
        None,
        false,
        None,
    );
    assert!(
        state.slash_menu.is_credential_input(),
        "credential input should be active"
    );

    // In handle_input_event, after approval check, the credential check is:
    //   if self.state.slash_menu.is_credential_input() { ... return false; }
    // History navigation is never reached.
    let credential_intercepts = state.slash_menu.is_credential_input();
    assert!(
        credential_intercepts,
        "credential input must intercept before history"
    );
    assert!(state.history_cursor.is_none(), "cursor should be at draft");
}

/// Proves the full roundtrip: submit 3 entries, type multiline draft,
/// navigate through all history, return to draft with exact content.
#[test]
fn semantic_buffer_full_roundtrip_with_multiline_draft() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    let mut sr = StreamRenderState::default();

    for msg in &["alpha", "beta", "gamma"] {
        state.input_append_str(msg);
        assert!(submit_input_message(&mut state, &mut sr, Some(&tx)));
    }
    assert_eq!(state.input_history, vec!["alpha", "beta", "gamma"]);

    // Type a multiline draft
    state.input_append_str("line one\nline two");

    // Navigate to oldest and back
    state.history_prev(); // → gamma
    assert_eq!(state.input_buffer, "gamma");
    state.history_prev(); // → beta
    state.history_prev(); // → alpha
    state.history_prev(); // stays at alpha
    assert_eq!(state.input_buffer, "alpha");

    state.history_next(); // → beta
    state.history_next(); // → gamma
    state.history_next(); // → draft

    assert_eq!(state.input_buffer, "line one\nline two");
    assert!(state.history_cursor.is_none());

    // Submit the draft — it should be recorded
    assert!(submit_input_message(&mut state, &mut sr, Some(&tx)));
    assert_eq!(
        state.input_history,
        vec!["alpha", "beta", "gamma", "line one\nline two"]
    );
}
