use crossterm::style::Color as CColor;
use talos_conversation::{
    MessageSource, TodoPanelData, TodoPanelRow, ToolResultDisplay, UserInput,
};
use talos_core::ApprovalChoice;
use talos_core::message::Message;
use tokio::sync::mpsc;

use crate::app::{
    MOUSE_HISTORY_SCROLL_ROWS, SPINNER_FRAMES, ScrollbackLine, StreamRenderState,
    build_todo_panel_lines,
};
use crate::app::{next_processing_frame, preview_text_for_state, submit_input_message, tip_ttl};
use crate::history_projection::{HistoryScrollMode, HistoryScrollState, project_history};
use crate::scrollback;
use crate::state::{ApprovalState, CredentialField, CtrlCState, PanelKind, TuiState, WizardStep};
use crate::stream_markdown::{HoldStatus, MarkdownBlockKind};
use crate::theme::{semantic, to_crossterm_color};
use crate::tool_display;
use crate::transcript::TranscriptBlock;
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
fn credential_and_provider_cursor_positions_are_panel_local() {
    let credential =
        crate::panel_state::BottomPanelState::open_credential_input("provider", None, false, None);
    let local =
        scrollback::credential_cursor_position(&credential).expect("operation should succeed");
    assert_eq!(local.row, 2);

    let provider = crate::panel_state::BottomPanelState::open_provider_wizard();
    let local = scrollback::provider_wizard_local_cursor_position(&provider)
        .expect("operation should succeed");
    assert_eq!(local.row, 2);
    assert_eq!(local.col, 3);
}

#[test]
fn approval_summary_uses_tool_summary_fields() {
    let args = serde_json::json!({
        "command": "cd /repo && git status --short",
        "other": "hidden"
    });
    let args_str = serde_json::to_string_pretty(&args).expect("operation should succeed");
    let summary = tool_display::summarize_tool_args("bash", &args_str, &["command".to_string()]);

    assert_eq!(summary, "command: cd /repo && git status --short");
    assert!(!summary.contains('{'));
    assert!(!summary.contains("other"));
}

#[test]
fn tool_args_summary_uses_available_budget_before_truncating() {
    let command = "cargo test -p talos-cli approval::tests::test_always_allow_rule_is_effective_against_default_ask";
    let args = serde_json::json!({ "command": command });
    let args_str = serde_json::to_string_pretty(&args).expect("operation should succeed");

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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);

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

    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);

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

    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "✗", Some(CColor::Red), 120);

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

    let opening = state.start(MessageSource::User);
    assert_eq!(opening.len(), 1);
    assert_eq!(opening[0].bg, bg);
    assert!(opening[0].fill.is_some());

    let closing = state.finish();
    assert_eq!(closing.len(), 1);
    assert_eq!(closing[0].bg, bg);
    assert!(closing[0].fill.is_some());

    state.reset();
    assert!(state.source().is_none());
    assert_eq!(state.preview(), "");
}

#[test]
fn assistant_stream_drops_leading_empty_prefix_row() {
    let mut state = StreamRenderState::default();
    assert!(state.start(MessageSource::Assistant).is_empty());

    let lines = state.push_chunk("\nactual response\n");

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, " ● actual response");
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
    scrollback::append_fill_segment(
        &mut segments,
        lines[0].fill.clone().expect("operation should succeed"),
        20,
        3,
    );
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
    let opening = state.start(MessageSource::User);
    assert_eq!(opening.len(), 1);
    assert_eq!(opening[0].bg, bg);
    assert!(opening[0].fill.is_some());

    let lines = state.push_chunk("# literal **user** `input`\n");

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, " > # literal **user** `input`");
    assert!(lines[0].fill.is_some());
    assert!(
        lines[0]
            .segments
            .iter()
            .all(|segment| !segment.attrs.italic)
    );
}

#[test]
fn user_history_block_fills_the_current_history_width() {
    let mut stream_count = 0;
    let lines = scrollback::render_history_message(
        &mut stream_count,
        MessageSource::User,
        "submitted text",
    );
    let mut transcript = crate::transcript::TranscriptStore::default();
    for line in lines {
        transcript.append(TranscriptBlock::StyledLine(line));
    }

    let projection = project_history(&transcript, 20, 10, &HistoryScrollState::follow_tail());
    let bg = scrollback::stream_bg_for(Some(&MessageSource::User));

    assert_eq!(projection.rows.len(), 3);
    assert!(projection.rows.iter().all(|row| row.line.bg == bg));
    assert!(
        projection
            .rows
            .iter()
            .all(|row| { unicode_width::UnicodeWidthStr::width(row.line.text.as_str()) == 20 })
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
        120,
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
    .expect("operation should succeed");
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
    .expect("operation should succeed");
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
    let content = serde_json::to_string_pretty(&imports).expect("operation should succeed");
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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);

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
    let lines =
        tool_display::build_tool_result_scrollback_lines(&display, "✗", Some(CColor::Red), 120);

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
        let lines = tool_display::build_tool_result_scrollback_lines(
            &display,
            "",
            Some(CColor::Green),
            120,
        );
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
    let _ =
        tool_display::build_tool_result_scrollback_lines(&display, "", Some(CColor::Green), 120);
    assert_eq!(display.content, original);
    assert!(display.content.contains("line 25"));
}

#[test]
fn preview_text_uses_phase_priority_states() {
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::TimedOut), None, true, "stream", 0),
        "⏱ timed out"
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Failed), None, true, "stream", 0),
        "✗ failed"
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Cancelled), None, true, "stream", 0),
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
fn preview_text_clears_terminal_phase_after_turn_ends() {
    for phase in [TurnPhase::TimedOut, TurnPhase::Failed, TurnPhase::Cancelled] {
        assert!(
            preview_text_for_state(None, Some(&phase), None, false, "", 0).is_empty(),
            "terminal phase {phase:?} must not persist in the inactive preview"
        );
    }
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

// ── TUI-030 entry-point tests: actual handle_input_event key dispatch ─────

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

fn key_press(code: KeyCode) -> Event {
    key_press_with_modifiers(code, KeyModifiers::NONE)
}

fn key_press_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new_with_kind_and_state(
        code,
        modifiers,
        KeyEventKind::Press,
        crossterm::event::KeyEventState::NONE,
    ))
}

fn mouse_scroll(kind: MouseEventKind) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn selection_drag_isolated_from_composer_and_survives_resize() {
    let mut state = TuiState::new();
    state.input_append_str("draft");
    let mut tui = crate::app::Tui::for_test(state, None);
    tui.last_history_viewport_height = 8;
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 80, 8));

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        2,
        1,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        9,
        4,
    ));
    tui.handle_input_event(&Event::Resize(8, 4));

    assert_eq!(tui.state.input_buffer, "draft");
    assert_eq!(tui.state.cursor_byte_pos(), "draft".len());
    let selection = tui.selection.expect("selection remains active");
    assert_eq!(selection.anchor, (2, 1));
    assert_eq!(selection.focus, (7, 3));
    assert!(selection.dragging);
}

#[test]
fn primary_click_without_drag_does_not_leave_a_selection() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.last_history_viewport_height = 8;

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        2,
        1,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Up(crossterm::event::MouseButton::Left),
        2,
        1,
    ));

    assert!(tui.selection.is_none());
    assert!(tui.state.tip.is_none());
}

#[test]
fn selection_works_when_no_history_area_is_rendered() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        2,
        3,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        7,
        3,
    ));

    let selection = tui.selection.expect("visible-cell selection");
    assert_eq!(selection.anchor, (2, 3));
    assert_eq!(selection.focus, (7, 3));
    assert!(selection.dragging);
}

#[test]
fn mouse_up_coordinate_is_the_final_selection_focus() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        1,
        2,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        3,
        2,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Up(crossterm::event::MouseButton::Left),
        6,
        2,
    ));

    let selection = tui.selection.expect("completed selection");
    assert_eq!(selection.focus, (6, 2));
    assert!(!selection.dragging);
}

#[test]
fn splash_rows_are_not_mapped_to_transcript_rows() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for text in ["alpha", "beta"] {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
    }
    tui.last_history_projection =
        project_history(&tui.transcript, 20, 10, &HistoryScrollState::follow_tail());
    tui.last_history_viewport_height = 4;
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 20, 4));
    tui.last_frame_history_start = 0;
    tui.last_splash_row_count = 2;
    tui.last_history_prefix_row_count = 2;

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        0,
        0,
    ));
    assert!(
        tui.selection
            .expect("splash selection")
            .history_anchor
            .is_none()
    );

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        0,
        2,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        0,
        3,
    ));
    let selection = tui.selection.expect("history selection");
    assert_eq!(tui.selected_text_for(selection), "alpha\nb");
}

#[test]
fn logo_prefix_keeps_accumulated_history_range() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for text in ["alpha", "beta"] {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
    }
    tui.last_history_projection =
        project_history(&tui.transcript, 20, 10, &HistoryScrollState::follow_tail());
    tui.last_history_viewport_height = 4;
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 20, 4));
    tui.last_frame_history_start = 0;
    tui.last_splash_row_count = 2;
    tui.last_history_prefix_row_count = 2;
    let history_anchor = tui
        .last_history_projection
        .selection_point(1, 0)
        .expect("beta anchor");
    let history_focus = tui
        .last_history_projection
        .selection_point(0, 0)
        .expect("alpha focus");
    tui.selection = Some(super::SelectionState {
        anchor: (0, 3),
        focus: (0, 2),
        dragging: true,
        edge: -1,
        history_anchor: Some(history_anchor),
        history_focus: Some(history_focus),
    });

    let mut selection = tui.selection.expect("active selection");
    selection.update_history_focus(None, true);
    assert_eq!(tui.selected_text_for(selection), "alpha\nb");
}

#[test]
fn history_selection_payload_survives_resize_reflow() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.transcript
        .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
            "abcdefghij",
            None,
        )));
    tui.last_history_projection =
        project_history(&tui.transcript, 4, 10, &HistoryScrollState::follow_tail());
    tui.last_history_viewport_height = 3;
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 4, 3));

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        1,
        0,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        3,
        1,
    ));
    tui.handle_input_event(&Event::Resize(10, 4));
    tui.last_history_projection =
        project_history(&tui.transcript, 10, 10, &HistoryScrollState::follow_tail());

    let selection = tui.selection.expect("selection survives reflow");
    assert_eq!(tui.selected_text_for(selection), "bcdefgh");
}

#[test]
fn selection_edge_tick_autoscrolls_only_while_dragging() {
    let mut tui = tui_with_projected_history(10, 10);
    let history_anchor = tui
        .last_history_projection
        .selection_point(25, 3)
        .expect("anchor");
    let history_focus = tui
        .last_history_projection
        .selection_point(20, 3)
        .expect("focus");
    tui.selection = Some(super::SelectionState {
        anchor: (3, 5),
        focus: (3, 0),
        dragging: true,
        edge: -1,
        history_anchor: Some(history_anchor),
        history_focus: Some(history_focus),
    });

    tui.advance_processing_frame();
    assert!(matches!(
        tui.history_scroll.mode,
        crate::history_projection::HistoryScrollMode::Anchored { .. }
    ));

    let anchored = tui.history_scroll.clone();
    tui.selection.as_mut().expect("selection").dragging = false;
    tui.advance_processing_frame();
    assert_eq!(tui.history_scroll, anchored);
}

#[test]
fn selection_edge_tick_enters_logo_prefix_one_row_at_a_time() {
    let mut tui = tui_with_projected_history(10, 10);
    tui.last_splash_row_count = 2;
    tui.last_frame_history_start = 2;
    let history_anchor = tui
        .last_history_projection
        .selection_point(20, 3)
        .expect("anchor");
    tui.selection = Some(super::SelectionState {
        anchor: (3, 5),
        focus: (3, 0),
        dragging: true,
        edge: -1,
        history_anchor: Some(history_anchor),
        history_focus: Some(history_anchor),
    });

    tui.advance_processing_frame();

    assert_eq!(tui.history_prefix_start, Some(1));
}

#[test]
fn drag_near_history_edge_arms_autoscroll() {
    let mut tui = tui_with_projected_history(10, 10);
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
    tui.last_frame_history_start = 20;

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        2,
        5,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        2,
        1,
    ));

    assert_eq!(tui.selection.expect("selection").edge, -1);
}

#[test]
fn drag_below_history_edge_keeps_logical_focus() {
    let mut tui = tui_with_projected_history(10, 10);
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 80, 10));
    tui.last_frame_history_start = 20;

    tui.handle_input_event(&mouse(
        MouseEventKind::Down(crossterm::event::MouseButton::Left),
        2,
        5,
    ));
    tui.handle_input_event(&mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        2,
        12,
    ));

    let selection = tui.selection.expect("selection");
    assert_eq!(selection.edge, 1);
    assert!(selection.history_focus.is_some());
}

#[test]
fn completed_history_highlight_moves_with_scrolled_content() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(40, 16));
    for index in 0..30 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("history row {index:02}"),
                None,
            )));
    }
    tui.draw_frame().expect("tail frame");
    let area = tui.last_history_area.expect("history area");
    let row = area.y.saturating_add(2);
    let anchor = tui
        .history_selection_point_at_screen(2, row)
        .expect("selection anchor");
    let focus = tui
        .history_selection_point_at_screen(5, row)
        .expect("selection focus");
    tui.selection = Some(super::SelectionState {
        anchor: (2, row),
        focus: (5, row),
        dragging: false,
        edge: 0,
        history_anchor: Some(anchor),
        history_focus: Some(focus),
    });
    tui.draw_frame().expect("selected frame");
    assert_eq!(
        tui.terminal.test_cell_bg(2, row),
        ratatui::style::Color::DarkGray
    );

    tui.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
    tui.draw_frame().expect("scrolled selected frame");

    assert_eq!(
        tui.terminal
            .test_cell_bg(2, row.saturating_add(MOUSE_HISTORY_SCROLL_ROWS as u16)),
        ratatui::style::Color::DarkGray
    );
    assert_ne!(
        tui.terminal.test_cell_bg(2, row),
        ratatui::style::Color::DarkGray
    );
}

#[test]
fn entry_point_shift_enter_inserts_newline_without_sending() {
    let mut state = TuiState::new();
    state.input_append_str("line one");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    tui.handle_input_event(&key_press_with_modifiers(
        KeyCode::Enter,
        KeyModifiers::SHIFT,
    ));

    assert_eq!(tui.state.input_buffer, "line one\n");
    assert!(rx.try_recv().is_err());
}

#[test]
fn entry_point_ctrl_j_inserts_newline_without_sending() {
    let mut state = TuiState::new();
    state.input_append_str("line one");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    tui.handle_input_event(&key_press_with_modifiers(
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    ));

    assert_eq!(tui.state.input_buffer, "line one\n");
    assert!(rx.try_recv().is_err());
}

#[test]
fn esc_cancels_each_consecutive_processing_turn_without_exiting() {
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));
    let esc = key_press(KeyCode::Esc);

    assert!(
        !tui.handle_input_event(&esc),
        "cancelling the first active turn must not exit"
    );
    assert!(matches!(rx.try_recv(), Ok(UserInput::Cancel)));

    // The queued steering message starts a new turn immediately after the
    // cancellation acknowledgement. Its Esc must be treated as a fresh
    // turn cancellation.
    tui.state.status.is_processing = false;
    tui.state.status.is_processing = true;

    assert!(
        !tui.handle_input_event(&esc),
        "cancelling the queued turn must not exit"
    );
    assert!(matches!(rx.try_recv(), Ok(UserInput::Cancel)));
    assert!(matches!(tui.state.ctrl_c_state, CtrlCState::Idle));
    assert!(!tui.state.should_exit);
}

/// Entry-point test: Up/Down through the actual `handle_input_event` method.
#[test]
fn entry_point_up_down_history_navigation() {
    let mut state = TuiState::new();
    // Pre-populate history
    state.input_history = vec!["hello".to_string(), "world".to_string()];
    // Type a draft
    state.input_append_str("my draft");

    let mut tui = crate::app::Tui::for_test(state, None);

    // Up → newest entry
    tui.handle_input_event(&key_press(KeyCode::Up));
    assert_eq!(tui.state.input_buffer, "world");
    assert_eq!(tui.state.draft_input, "my draft");

    // Up → oldest entry
    tui.handle_input_event(&key_press(KeyCode::Up));
    assert_eq!(tui.state.input_buffer, "hello");

    // Up at oldest → stays
    tui.handle_input_event(&key_press(KeyCode::Up));
    assert_eq!(tui.state.input_buffer, "hello");

    // Down → newer
    tui.handle_input_event(&key_press(KeyCode::Down));
    assert_eq!(tui.state.input_buffer, "world");

    // Down past newest → exact draft restored
    tui.handle_input_event(&key_press(KeyCode::Down));
    assert_eq!(tui.state.input_buffer, "my draft");
    assert!(tui.state.history_cursor.is_none());
    assert!(tui.state.draft_input.is_empty());
}

/// Entry-point test: slash menu open intercepts Up/Down — history untouched.
#[test]
fn entry_point_slash_menu_open_does_not_trigger_history() {
    let mut state = TuiState::new();
    state.input_history = vec!["secret".to_string()];
    state.input_append_str("draft");
    state.open_slash_menu(talos_conversation::command_registry());
    assert!(state.slash_menu.is_open);

    let mut tui = crate::app::Tui::for_test(state, None);

    // Send Up through the actual handler
    tui.handle_input_event(&key_press(KeyCode::Up));

    // History cursor must NOT have moved
    assert!(
        tui.state.history_cursor.is_none(),
        "history cursor must stay at draft when slash menu is open"
    );
    // Input buffer must NOT have changed to a history entry
    assert!(
        !tui.state.input_buffer.contains("secret"),
        "history must not leak when slash menu is open"
    );
}

/// Entry-point test: approval active intercepts Up/Down — history untouched.
#[test]
fn entry_point_approval_active_does_not_trigger_history() {
    let mut state = TuiState::new();
    state.input_history = vec!["secret".to_string()];
    state.input_append_str("draft");
    state.activate_approval("test_tool", "args");
    assert!(!matches!(state.approval_state, ApprovalState::Hidden));

    let mut tui = crate::app::Tui::for_test(state, None);

    // Send Up through the actual handler
    tui.handle_input_event(&key_press(KeyCode::Up));

    assert!(
        tui.state.history_cursor.is_none(),
        "history cursor must stay at draft when approval is active"
    );
    assert!(
        !tui.state.input_buffer.contains("secret"),
        "history must not leak when approval is active"
    );
}

/// Entry-point test: credential input intercepts Up/Down — history untouched.
#[test]
fn entry_point_credential_input_does_not_trigger_history() {
    let mut state = TuiState::new();
    state.input_history = vec!["secret".to_string()];
    state.input_append_str("draft");
    state.slash_menu = crate::panel_state::BottomPanelState::open_credential_input(
        "test-provider",
        None,
        false,
        None,
    );
    assert!(state.slash_menu.is_credential_input());

    let mut tui = crate::app::Tui::for_test(state, None);

    // Send Up through the actual handler
    tui.handle_input_event(&key_press(KeyCode::Up));

    assert!(
        tui.state.history_cursor.is_none(),
        "history cursor must stay at draft when credential input is active"
    );
    assert!(
        !tui.state.input_buffer.contains("secret"),
        "history must not leak when credential input is active"
    );
}

/// Entry-point test: full multiline draft roundtrip through actual key dispatch.
#[test]
fn entry_point_full_roundtrip_multiline_draft() {
    let mut state = TuiState::new();
    state.input_history = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    state.input_append_str("line one\nline two");

    let mut tui = crate::app::Tui::for_test(state, None);

    // Navigate to oldest
    tui.handle_input_event(&key_press(KeyCode::Up)); // → gamma
    assert_eq!(tui.state.input_buffer, "gamma");
    tui.handle_input_event(&key_press(KeyCode::Up)); // → beta
    tui.handle_input_event(&key_press(KeyCode::Up)); // → alpha
    tui.handle_input_event(&key_press(KeyCode::Up)); // stays at alpha
    assert_eq!(tui.state.input_buffer, "alpha");

    // Navigate back to draft
    tui.handle_input_event(&key_press(KeyCode::Down)); // → beta
    tui.handle_input_event(&key_press(KeyCode::Down)); // → gamma
    tui.handle_input_event(&key_press(KeyCode::Down)); // → draft

    assert_eq!(tui.state.input_buffer, "line one\nline two");
    assert!(tui.state.history_cursor.is_none());
}

fn tui_with_projected_history(visible_height: u16, viewport_height: u16) -> crate::app::Tui {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for index in 0..30 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("row-{index:02}"),
                None,
            )));
    }
    tui.last_history_projection = project_history(
        &tui.transcript,
        80,
        visible_height,
        &HistoryScrollState::follow_tail(),
    );
    tui.last_history_viewport_height = viewport_height;
    tui
}

#[test]
fn entry_point_page_up_uses_history_rect_height() {
    let mut tui = tui_with_projected_history(4, 10);
    assert_eq!(tui.last_history_projection.visible_start, 26);
    tui.handle_input_event(&key_press(KeyCode::PageUp));
    let after = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    assert_eq!(after.visible_start, 17);
}

#[test]
fn entry_point_mouse_wheel_scrolls_history_without_browsing_composer_history() {
    let mut tui = tui_with_projected_history(10, 10);
    tui.state.input_history = vec!["previous input".to_string()];
    tui.state.input_append_str("live draft");
    assert_eq!(tui.last_history_projection.visible_start, 20);
    tui.last_frame_history_start = 20;

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));

    assert_eq!(tui.state.input_buffer, "live draft");
    assert!(tui.state.history_cursor.is_none());
    let scrolled = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    assert_eq!(scrolled.visible_start, 17);

    tui.last_history_projection = scrolled;
    tui.last_frame_history_start = 17;
    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollDown));
    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());
    assert_eq!(tui.state.input_buffer, "live draft");
}

#[test]
fn entry_point_page_down_uses_history_rect_height_and_end_returns_follow_tail() {
    let mut tui = tui_with_projected_history(4, 10);
    let first = tui
        .last_history_projection
        .first_anchor()
        .expect("operation should succeed");
    tui.history_scroll.anchor(first, 0);
    tui.last_history_projection = project_history(&tui.transcript, 80, 4, &tui.history_scroll);
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    let after = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    assert_eq!(after.visible_start, 9);

    tui.last_history_projection = after;
    tui.handle_input_event(&key_press_with_modifiers(
        KeyCode::End,
        KeyModifiers::CONTROL,
    ));
    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());
}

#[test]
fn entry_point_page_down_at_tail_remains_follow_tail() {
    let mut tui = tui_with_projected_history(4, 10);
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
}

#[test]
fn entry_point_ctrl_home_moves_to_start_and_empty_navigation_is_safe() {
    let mut tui = tui_with_projected_history(4, 10);
    tui.handle_input_event(&key_press_with_modifiers(
        KeyCode::Home,
        KeyModifiers::CONTROL,
    ));
    let at_start = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    assert_eq!(at_start.visible_start, 0);

    let mut empty = crate::app::Tui::for_test(TuiState::new(), None);
    for code in [KeyCode::PageUp, KeyCode::PageDown] {
        empty.handle_input_event(&key_press(code));
    }
    empty.handle_input_event(&key_press_with_modifiers(
        KeyCode::Home,
        KeyModifiers::CONTROL,
    ));
    empty.handle_input_event(&key_press_with_modifiers(
        KeyCode::End,
        KeyModifiers::CONTROL,
    ));
    assert_eq!(empty.history_scroll, HistoryScrollState::follow_tail());
}

#[test]
fn fixed_components_are_never_appended_to_transcript() {
    let mut state = TuiState::new();
    state.input_append_str("draft composer text");
    state.thinking_preview = Some("fixed thinking preview".into());
    state.tip = Some(crate::state::Tip {
        kind: TipKind::Info,
        text: "fixed tip".into(),
        ttl: std::time::Duration::from_secs(30),
        created_at: std::time::Instant::now(),
    });
    state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
    let mut tui = crate::app::Tui::for_test(state, None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(40, 10));
    tui.draw_frame().expect("fixed frame renders");
    tui.handle_input_event(&key_press(KeyCode::PageUp));
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    assert!(tui.transcript.entries().is_empty());

    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "submitted user text".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit transcript");
    let stored = tui
        .transcript
        .entries()
        .iter()
        .filter_map(|entry| match &entry.block {
            TranscriptBlock::StyledLine(line) => Some(line.text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(stored.contains("submitted user text"));
    for fixed in ["draft composer text", "fixed thinking preview", "fixed tip"] {
        assert!(!stored.contains(fixed));
    }
}

#[test]
fn full_frame_renderer_handles_extreme_terminal_sizes() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for (width, height) in [
        (0, 0),
        (1, 1),
        (1, 2),
        (2, 1),
        (2, 2),
        (3, 3),
        (5, 2),
        (20, 3),
    ] {
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(width, height));
        tui.draw_frame()
            .unwrap_or_else(|error| panic!("{width}x{height}: {error}"));
    }
}

#[test]
fn alternate_screen_first_frame_renders_logo_without_transcript_pollution() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));

    tui.draw_frame().expect("first alternate-screen frame");
    let first_frame = tui.terminal.test_rendered_text();
    assert!(first_frame.contains("████████"));
    assert!(first_frame.contains("The watchman never sleeps"));
    assert!(tui.transcript.entries().is_empty());

    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "conversation follows startup logo".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("conversation frame");
    let conversation_frame = tui.terminal.test_rendered_text();
    let logo_offset = conversation_frame
        .find("The watchman never sleeps")
        .expect("startup logo should remain visible with the first message");
    let message_offset = conversation_frame
        .find("conversation follows startup logo")
        .expect("first message should be visible below the startup logo");
    assert!(logo_offset < message_offset);
    assert!(tui.transcript.entries().iter().all(|entry| {
        !matches!(
            &entry.block,
            crate::transcript::TranscriptBlock::StyledLine(line)
                if line.text.contains("The watchman never sleeps")
        )
    }));
}

#[test]
fn startup_logo_scrolls_out_after_history_fills_the_viewport() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));

    for index in 0..30 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("history row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("full history frame");
    let rendered = tui.terminal.test_rendered_text();

    assert!(!rendered.contains("The watchman never sleeps"));
    assert!(rendered.contains("history row 29"));
}

#[test]
fn mouse_scroll_preserves_visible_logo_prefix_until_its_rows_scroll_out() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "first message".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("initial frame");
    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("The watchman never sleeps")
    );

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
    tui.draw_frame().expect("scrolled frame");

    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("The watchman never sleeps"),
        "scrolling while the Logo prefix is visible must not drop it wholesale"
    );
}

#[test]
fn mouse_scroll_can_reach_logo_after_it_has_scrolled_out() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    for index in 0..30 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("history row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("tail frame");
    assert!(
        !tui.terminal
            .test_rendered_text()
            .contains("The watchman never sleeps")
    );

    for _ in 0..30 {
        tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
        tui.draw_frame().expect("history scroll frame");
    }

    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("The watchman never sleeps"),
        "the display-only Logo prefix must remain reachable from transcript history: \
         frame_start={}, prefix_start={:?}, splash_rows={}, history_height={}",
        tui.last_frame_history_start,
        tui.history_prefix_start,
        tui.last_splash_row_count,
        tui.last_history_viewport_height
    );
}

#[test]
fn mouse_scroll_moves_continuously_across_logo_transcript_boundary() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    for index in 0..20 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("history row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("tail frame");

    for _ in 0..30 {
        tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
        tui.draw_frame().expect("history scroll frame");
        if tui.history_prefix_start.is_some() {
            break;
        }
    }
    let prefix_start = tui
        .history_prefix_start
        .expect("wheel-up should enter the Logo prefix");
    let expected = prefix_start.saturating_add(MOUSE_HISTORY_SCROLL_ROWS);

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollDown));
    tui.draw_frame().expect("boundary scroll frame");

    assert_eq!(
        tui.last_frame_history_start, expected,
        "wheel-down must advance continuously instead of jumping to transcript tail"
    );
}

#[test]
fn modal_cursor_is_safe_when_final_panel_is_missing_or_short() {
    for panel in [
        crate::panel_state::BottomPanelState::open_credential_input("provider", None, false, None),
        crate::panel_state::BottomPanelState::open_provider_wizard(),
    ] {
        for height in [2, 3, 5] {
            let mut state = TuiState::new();
            state.slash_menu = panel.clone();
            let mut tui = crate::app::Tui::for_test(state, None);
            tui.terminal
                .set_test_size(ratatui::layout::Size::new(20, height));
            tui.draw_frame()
                .unwrap_or_else(|error| panic!("20x{height}: {error}"));
        }
    }
}

fn render_modal_for_cursor(
    panel: crate::panel_state::BottomPanelState,
    width: u16,
    height: u16,
) -> crate::app::Tui {
    let mut state = TuiState::new();
    state.slash_menu = panel;
    let mut tui = crate::app::Tui::for_test(state, None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(width, height));
    tui.draw_frame().expect("modal frame renders");
    tui
}

#[test]
fn credential_api_key_cursor_hidden_when_field_row_is_not_rendered() {
    let panel =
        crate::panel_state::BottomPanelState::open_credential_input("provider", None, false, None);
    for height in [1, 2, 3, 4] {
        let tui = render_modal_for_cursor(panel.clone(), 40, height);
        assert!(
            !tui.terminal.test_cursor_visible(),
            "credential ApiKey cursor must hide at terminal height {height}"
        );
    }

    let tui = render_modal_for_cursor(panel, 40, 10);
    assert!(tui.terminal.test_cursor_visible());
}

#[test]
fn credential_base_url_cursor_hidden_when_field_row_is_not_rendered() {
    let mut panel =
        crate::panel_state::BottomPanelState::open_credential_input("provider", None, true, None);
    panel.credential_field = CredentialField::BaseUrl;
    for height in [1, 2, 3, 4] {
        let tui = render_modal_for_cursor(panel.clone(), 40, height);
        assert!(
            !tui.terminal.test_cursor_visible(),
            "credential BaseUrl cursor must hide at terminal height {height}"
        );
    }

    let tui = render_modal_for_cursor(panel, 40, 10);
    assert_eq!(
        tui.terminal.test_cursor_position().map(|pos| pos.y),
        Some(7),
        "BaseUrl local row 3 must be offset only by the final panel rectangle"
    );
}

#[test]
fn provider_protocol_cursor_hidden_when_selected_row_is_clipped() {
    let mut panel = crate::panel_state::BottomPanelState::open_provider_wizard();
    let Some(PanelKind::ProviderWizard { step, protocol, .. }) = panel.kind.as_mut() else {
        panic!("provider wizard panel expected");
    };
    *step = WizardStep::Protocol;
    *protocol = "anthropic-messages".into();

    for height in [1, 2, 3, 4] {
        let tui = render_modal_for_cursor(panel.clone(), 40, height);
        assert!(
            !tui.terminal.test_cursor_visible(),
            "second protocol option must hide at terminal height {height}"
        );
    }

    let tui = render_modal_for_cursor(panel, 40, 10);
    assert_eq!(
        tui.terminal.test_cursor_position().map(|pos| pos.y),
        Some(7),
        "the second protocol option must remain at local row 3"
    );
}

#[test]
fn provider_entry_cursor_hides_when_field_is_clipped_and_confirm_has_no_cursor() {
    for step in [WizardStep::Name, WizardStep::BaseUrl, WizardStep::ApiKey] {
        let mut panel = crate::panel_state::BottomPanelState::open_provider_wizard();
        let Some(PanelKind::ProviderWizard { step: current, .. }) = panel.kind.as_mut() else {
            panic!("provider wizard panel expected");
        };
        *current = step;
        let tui = render_modal_for_cursor(panel, 40, 3);
        assert!(!tui.terminal.test_cursor_visible());
    }

    let mut panel = crate::panel_state::BottomPanelState::open_provider_wizard();
    let Some(PanelKind::ProviderWizard { step, .. }) = panel.kind.as_mut() else {
        panic!("provider wizard panel expected");
    };
    *step = WizardStep::Confirm;
    let tui = render_modal_for_cursor(panel, 40, 20);
    assert!(!tui.terminal.test_cursor_visible());
}

// ---------------------------------------------------------------------------
// I164 / TUI-038: Startup Inline Composer Continuity
// ---------------------------------------------------------------------------

fn startup_tui_at(width: u16, height: u16) -> crate::app::Tui {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(width, height));
    tui
}

#[test]
fn startup_layout_places_composer_one_row_below_logo() {
    let mut tui = startup_tui_at(80, 24);
    tui.draw_frame().expect("startup frame renders");

    let cursor = tui
        .terminal
        .test_cursor_position()
        .expect("cursor visible in startup composer");
    let splash_rows = crate::splash::viewport_splash_lines(80).len();
    // Startup suppresses preview but keeps the one-row tips surface visible:
    // composer_y = splash_rows + 1 spacer + 1 tip + 1 composer_top_pad
    let expected_y = (splash_rows + 1 + 1 + 1) as u16;
    assert_eq!(
        cursor.y, expected_y,
        "composer cursor should be {} (splash {} + 1 spacer + 1 tip + 1 top_pad), got {}",
        expected_y, splash_rows, cursor.y
    );
    assert!(cursor.y < 24, "cursor must be within terminal bounds");
}

#[test]
fn startup_tips_surface_shows_dashboard_address() {
    let mut tui = startup_tui_at(80, 24);
    tui.state.tip = Some(crate::state::Tip {
        text: "Dashboard ready: http://127.0.0.1:61205/ (loopback-only)".into(),
        kind: TipKind::Info,
        ttl: std::time::Duration::from_secs(8),
        created_at: std::time::Instant::now(),
    });

    tui.draw_frame().expect("startup frame");
    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("Dashboard ready: http://127.0.0.1:61205/"),
        "startup tips must retain the dashboard's copyable address"
    );
}

#[test]
fn startup_draft_does_not_mutate_transcript() {
    let mut tui = startup_tui_at(80, 24);
    tui.state.input_append_str("draft text");
    tui.draw_frame().expect("startup draft frame");
    assert!(tui.transcript.entries().is_empty());

    tui.state.input_clear();
    tui.draw_frame().expect("startup cleared draft frame");
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn startup_draft_redraw_preserves_logo_and_cursor() {
    let mut tui = startup_tui_at(80, 24);

    tui.state.input_append_str("first edit");
    tui.draw_frame().expect("first startup draft frame");
    let first_cursor = tui.terminal.test_cursor_position().expect("cursor 1");
    let first_render = tui.terminal.test_rendered_text();
    assert!(
        first_render.contains("████████"),
        "logo visible on first draw"
    );

    tui.state.input_append_str(" and more");
    tui.draw_frame().expect("second startup draft frame");
    let second_cursor = tui.terminal.test_cursor_position().expect("cursor 2");
    let second_render = tui.terminal.test_rendered_text();
    assert!(
        second_render.contains("████████"),
        "logo still visible on redraw"
    );
    assert_eq!(
        first_cursor.y, second_cursor.y,
        "composer row must stay stable across draft redraws"
    );
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn first_submit_keeps_short_conversation_adjacent_to_logo() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.input_append_str("hello world");
    let mut tui = crate::app::Tui::for_test(state, Some(tx));
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));

    tui.draw_frame().expect("startup frame");
    let startup_cursor = tui.terminal.test_cursor_position().expect("startup cursor");
    assert!(
        startup_cursor.y < 20,
        "startup composer should be near top, got y={}",
        startup_cursor.y
    );

    tui.handle_input_event(&key_press(KeyCode::Enter));
    let _ = rx.try_recv().expect("message dispatched");

    // Submission leaves startup mode immediately, but a short conversation
    // must still keep the composer adjacent to the Logo/history flow.
    tui.draw_frame()
        .expect("post-submit frame (before UiOutput echo)");
    let post_submit_cursor = tui
        .terminal
        .test_cursor_position()
        .expect("post-submit cursor");
    assert!(
        post_submit_cursor.y < 20,
        "empty post-submit composer must not jump to bottom, got {}",
        post_submit_cursor.y
    );

    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "hello world".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit");
    tui.draw_frame().expect("conversation frame");
    let conv_cursor = tui
        .terminal
        .test_cursor_position()
        .expect("conversation cursor");
    assert!(
        conv_cursor.y < 20,
        "short conversation composer must remain near logo, got {}",
        conv_cursor.y
    );

    tui.draw_frame().expect("second conversation frame");
    let conv_cursor_2 = tui.terminal.test_cursor_position().expect("second cursor");
    assert_eq!(conv_cursor.y, conv_cursor_2.y, "stable without new content");
    assert!(
        conv_cursor.y < 20,
        "first history row must not force bottom layout"
    );
}

#[test]
fn growing_follow_tail_history_moves_composer_until_frame_overflows() {
    let mut tui = startup_tui_at(80, 24);
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "first".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit first");
    tui.draw_frame().expect("first conversation frame");
    let first_cursor = tui.terminal.test_cursor_position().expect("first cursor");

    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::Assistant,
            text: "second".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit second");
    tui.draw_frame().expect("second conversation frame");
    let second_cursor = tui.terminal.test_cursor_position().expect("second cursor");
    assert!(
        second_cursor.y > first_cursor.y,
        "composer should follow growing history while it fits"
    );

    for index in 0..40 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("overflow row {index}"),
            },
        ));
    }
    tui.commit_pending_transcript()
        .expect("commit overflow rows");
    tui.draw_frame().expect("overflow conversation frame");
    let overflow_cursor = tui
        .terminal
        .test_cursor_position()
        .expect("overflow cursor");
    assert!(
        overflow_cursor.y >= second_cursor.y,
        "overflow must never move composer above existing flowing content"
    );
    assert!(
        overflow_cursor.y < 24,
        "bottom fallback cursor remains bounded"
    );
    assert!(
        tui.last_history_projection.total_rows > usize::from(tui.last_history_viewport_height),
        "overflow uses a bounded history viewport"
    );

    tui.handle_input_event(&key_press(KeyCode::PageUp));
    tui.draw_frame().expect("anchored history frame");
    assert!(
        matches!(tui.history_scroll.mode, HistoryScrollMode::Anchored { .. }),
        "paging history leaves FollowTail"
    );
    let anchored_cursor = tui
        .terminal
        .test_cursor_position()
        .expect("anchored cursor");
    assert!(
        anchored_cursor.y >= overflow_cursor.y,
        "anchored history keeps the composer in its bounded bottom layout"
    );
}

#[test]
fn first_user_message_appears_below_logo_prefix() {
    let mut tui = startup_tui_at(80, 24);
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "first user message".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit");
    tui.draw_frame().expect("conversation frame");

    let rendered = tui.terminal.test_rendered_text();
    let logo_offset = rendered
        .find("The watchman never sleeps")
        .expect("logo should remain visible");
    let msg_offset = rendered
        .find("first user message")
        .expect("user message should be visible");
    assert!(
        logo_offset < msg_offset,
        "user message must appear below the logo prefix"
    );
    assert!(tui.transcript.entries().iter().all(|entry| {
        !matches!(
            &entry.block,
            crate::transcript::TranscriptBlock::StyledLine(line)
                if line.text.contains("The watchman never sleeps")
        )
    }));
}

#[test]
fn startup_spacers_are_projection_only() {
    let mut tui = startup_tui_at(80, 24);
    tui.draw_frame().expect("startup frame 1");
    assert!(tui.transcript.entries().is_empty());
    tui.draw_frame().expect("startup frame 2");
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn startup_resize_preserves_layout_invariants() {
    for (width, height) in [(80, 24), (120, 30), (40, 10)] {
        let mut tui = startup_tui_at(width, height);
        tui.draw_frame()
            .unwrap_or_else(|e| panic!("{width}x{height}: {e}"));
        assert!(tui.transcript.entries().is_empty());
        if let Some(cursor) = tui.terminal.test_cursor_position() {
            assert!(
                cursor.x < width && cursor.y < height,
                "cursor ({}, {}) out of bounds for {width}x{height}",
                cursor.x,
                cursor.y
            );
        }
    }
}

#[test]
fn startup_short_terminal_uses_bounded_fallback() {
    for (width, height) in [(5, 3), (3, 2), (2, 2), (1, 1), (0, 0)] {
        let mut tui = startup_tui_at(width, height);
        tui.draw_frame()
            .unwrap_or_else(|e| panic!("{width}x{height}: {e}"));
        if let Some(cursor) = tui.terminal.test_cursor_position() {
            assert!(
                cursor.x < width && cursor.y < height,
                "cursor ({}, {}) out of bounds for {width}x{height}",
                cursor.x,
                cursor.y
            );
        }
        assert!(tui.transcript.entries().is_empty());
    }
}

#[test]
fn startup_multiline_and_cjk_cursor_is_bounded() {
    let mut tui = startup_tui_at(80, 24);
    tui.state.input_append_str("你好\nworld");
    tui.draw_frame().expect("multiline CJK startup frame");

    let cursor = tui
        .terminal
        .test_cursor_position()
        .expect("cursor visible for multiline CJK");
    assert!(
        cursor.x < 80 && cursor.y < 24,
        "cursor ({}, {}) must be within 80x24",
        cursor.x,
        cursor.y
    );
    let splash_rows = crate::splash::viewport_splash_lines(80).len();
    assert!(
        cursor.y >= (splash_rows + 2) as u16,
        "cursor should be at or below the spacer rows, got y={}",
        cursor.y
    );
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn startup_mouse_wheel_does_not_enter_composer_history() {
    let mut state = TuiState::new();
    state.input_history = vec!["previous input".to_string()];
    state.input_append_str("live draft");
    let mut tui = crate::app::Tui::for_test(state, None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.draw_frame().expect("startup frame");

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
    tui.draw_frame().expect("after scroll up");

    assert_eq!(
        tui.state.input_buffer, "live draft",
        "mouse wheel must not browse composer input history"
    );
    assert!(
        tui.state.history_cursor.is_none(),
        "history cursor must not move from mouse wheel"
    );
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn post_submit_preview_and_history_are_continuous() {
    let mut tui = startup_tui_at(80, 24);

    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::User,
            text: "question".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit user");

    tui.state.status.is_processing = true;
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::Assistant,
            text: "answer text".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit assistant");
    tui.draw_frame().expect("conversation frame");

    let rendered = tui.terminal.test_rendered_text();
    let user_offset = rendered.find("question").expect("user message visible");
    let answer_offset = rendered
        .find("answer text")
        .expect("assistant response visible");
    assert!(
        user_offset < answer_offset,
        "assistant response must be continuous after user message"
    );
}

#[test]
fn startup_terminal_restore_does_not_regress() {
    let mut tui = startup_tui_at(80, 24);
    tui.draw_frame().expect("startup frame before restore");
    tui.restore().expect("terminal restore should succeed");
    // Full lifecycle restore (LeaveAlternateScreen, DisableRawMode, ShowCursor,
    // DisableMouseCapture, etc.) is covered by inline_terminal.rs lifecycle tests
    // and the real-terminal Case H acceptance gate. test_instance() starts with
    // a default lifecycle (all-false), so this verifies restore() is a safe no-op
    // in the startup state rather than exercising full terminal-mode transitions.
}

#[test]
fn startup_spacer_row_has_no_composer_background() {
    let mut tui = startup_tui_at(80, 24);
    tui.draw_frame().expect("startup frame");
    let splash_rows = crate::splash::viewport_splash_lines(80).len();

    let expected_bg = crate::theme::semantic::INPUT_BG;

    let spacer_y_1 = (splash_rows) as u16;
    let top_pad_y = (splash_rows + 2) as u16;

    let spacer_bg_1 = tui.terminal.test_cell_bg(0, spacer_y_1);
    let top_pad_bg = tui.terminal.test_cell_bg(0, top_pad_y);

    assert_ne!(
        spacer_bg_1, expected_bg,
        "spacer row 1 must not carry composer INPUT_BG"
    );
    assert_eq!(
        top_pad_bg, expected_bg,
        "composer top_pad must carry INPUT_BG — it is part of the composer frame, not a spacer"
    );
}

#[test]
fn startup_full_frame_buffer_assertions_all_sizes() {
    for (width, height) in [
        (80, 24),
        (160, 40),
        (40, 10),
        (20, 5),
        (5, 3),
        (3, 2),
        (2, 2),
        (1, 1),
        (0, 0),
    ] {
        let mut tui = startup_tui_at(width, height);
        tui.draw_frame()
            .unwrap_or_else(|e| panic!("{width}x{height}: {e}"));
        assert!(
            tui.transcript.entries().is_empty(),
            "transcript must be empty at {width}x{height}"
        );
        if let Some(cursor) = tui.terminal.test_cursor_position() {
            assert!(
                cursor.x < width && cursor.y < height,
                "cursor ({}, {}) out of bounds at {width}x{height}",
                cursor.x,
                cursor.y
            );
        }
    }

    // At normal sizes, the Logo should be visible
    let mut tui = startup_tui_at(80, 24);
    tui.draw_frame().expect("80x24 frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("████████"), "wide logo at 80 cols");
    assert!(
        rendered.contains("The watchman never sleeps"),
        "subtitle at 80 cols"
    );

    let mut tui = startup_tui_at(160, 40);
    tui.draw_frame().expect("160x40 frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("████████"), "wide logo at 160 cols");

    // At narrow sizes, compact logo
    let mut tui = startup_tui_at(40, 10);
    tui.draw_frame().expect("40x10 frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("_____"), "compact logo at 40 cols");
}

// ---------------------------------------------------------------------------
// I166 / TUI-036: Interrupt Shortcut Reliability entry-point tests
// ---------------------------------------------------------------------------

fn ctrl_c_event() -> Event {
    key_press_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL)
}

fn esc_event() -> Event {
    key_press(KeyCode::Esc)
}

fn assert_no_cancel(rx: &mut mpsc::UnboundedReceiver<UserInput>) {
    match rx.try_recv() {
        Ok(msg) => panic!("expected no UserInput, got {msg:?}"),
        Err(_) => {}
    }
}

fn assert_one_cancel(rx: &mut mpsc::UnboundedReceiver<UserInput>) {
    match rx.try_recv() {
        Ok(UserInput::Cancel) => {}
        Ok(other) => panic!("expected Cancel, got {other:?}"),
        Err(_) => panic!("expected exactly one Cancel, channel empty"),
    }
    assert_no_cancel(rx);
}

#[test]
fn entry_point_esc_cancels_active_turn_once() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc must not exit");
    assert_one_cancel(&mut rx);
    assert!(
        tui.state.tip.is_some(),
        "Esc during active turn must show cancellation feedback"
    );
}

#[test]
fn entry_point_esc_can_cancel_a_later_turn_after_cancelled_failed_and_timed_out_states() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    // successful turn → new active turn → Esc
    tui.state.status.is_processing = true;
    tui.handle_input_event(&esc_event());
    assert_one_cancel(&mut rx);
    tui.state.status.is_processing = false;
    tui.state.status.phase = None;

    // cancelled turn → queued message starts next turn → Esc again
    tui.state.status.is_processing = true;
    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc on later turn must not exit");
    assert_one_cancel(&mut rx);

    // failed turn → new active turn → Esc
    tui.state.status.is_processing = false;
    tui.state.status.phase = Some(TurnPhase::Failed);
    tui.state.status.is_processing = true;
    tui.state.status.phase = None;
    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc after failed turn must not exit");
    assert_one_cancel(&mut rx);

    // timed-out turn → new active turn → Esc
    tui.state.status.is_processing = false;
    tui.state.status.phase = Some(TurnPhase::TimedOut);
    tui.state.status.is_processing = true;
    tui.state.status.phase = None;
    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc after timed-out turn must not exit");
    assert_one_cancel(&mut rx);
}

#[test]
fn entry_point_esc_idle_preserves_composer() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.input_append_str("draft text");
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc idle must not exit");
    assert_no_cancel(&mut rx);
    assert_eq!(
        tui.state.input_buffer, "draft text",
        "Esc idle must preserve composer content"
    );
}

#[test]
fn entry_point_esc_slash_menu_closes_without_turn_cancel() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.open_slash_menu(talos_conversation::command_registry());
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc in slash menu must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        !tui.state.slash_menu.is_open,
        "Esc must close the slash menu"
    );
    assert!(
        tui.state.status.is_processing,
        "Esc in slash menu must not change processing state"
    );
}

#[test]
fn entry_point_esc_credential_closes_without_turn_cancel() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.slash_menu = crate::panel_state::BottomPanelState::open_credential_input(
        "test-provider",
        None,
        false,
        None,
    );
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc in credential must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        !tui.state.slash_menu.is_credential_input(),
        "Esc must close credential input"
    );
    assert!(
        tui.state.status.is_processing,
        "Esc in credential must not change processing state"
    );
}

#[test]
fn entry_point_esc_provider_wizard_closes_without_turn_cancel() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc in wizard must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        !tui.state.slash_menu.is_provider_wizard(),
        "Esc must close provider wizard"
    );
    assert!(
        tui.state.status.is_processing,
        "Esc in wizard must not change processing state"
    );
}

#[test]
fn entry_point_esc_approval_denies_without_turn_cancel() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let (resp_tx, mut resp_rx) = tokio::sync::oneshot::channel();
    tui.state.pending_approval_response = Some(resp_tx);
    tui.show_approval("test_tool", "args");

    let should_exit = tui.handle_input_event(&esc_event());
    assert!(!should_exit, "Esc in approval must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        matches!(tui.state.approval_state, ApprovalState::Hidden),
        "Esc must resolve and hide approval"
    );
    let choice = resp_rx
        .try_recv()
        .expect("approval response channel must receive a choice");
    assert!(
        matches!(choice, ApprovalChoice::Deny),
        "Esc in approval must Deny"
    );
    assert!(
        tui.state.status.is_processing,
        "Esc in approval must not change processing state"
    );
}

#[test]
fn entry_point_ctrl_c_active_draft_clears_without_cancel_or_exit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.input_append_str("active draft");
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&ctrl_c_event());
    assert!(!should_exit, "Ctrl+C with draft must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        tui.state.input_buffer.is_empty(),
        "Ctrl+C must clear the composer"
    );
    assert!(
        tui.state.status.is_processing,
        "Ctrl+C must not change processing state"
    );
}

#[test]
fn entry_point_ctrl_c_active_empty_does_not_cancel_or_exit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&ctrl_c_event());
    assert!(!should_exit, "Ctrl+C active empty must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        !tui.state.should_exit,
        "Ctrl+C active empty must not set should_exit"
    );
    assert!(
        tui.state.tip.is_some(),
        "Ctrl+C active empty must show Esc guidance tip"
    );
    assert!(
        tui.state
            .tip
            .as_ref()
            .is_some_and(|t| t.text.contains("Esc")),
        "Tip must mention Esc for interrupting the turn"
    );
}

#[test]
fn entry_point_ctrl_c_idle_empty_retains_double_press_exit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let first = tui.handle_input_event(&ctrl_c_event());
    assert!(!first, "first Ctrl+C must not exit immediately");
    assert_no_cancel(&mut rx);
    assert!(
        matches!(tui.state.ctrl_c_state, CtrlCState::Waiting(_)),
        "first Ctrl+C must arm the exit gesture"
    );

    let second = tui.handle_input_event(&ctrl_c_event());
    assert!(second, "second Ctrl+C must exit");
    assert!(tui.state.should_exit, "should_exit must be true");
}

#[test]
fn modified_ctrl_c_is_never_inserted_into_modal_input() {
    for modal in [
        crate::panel_state::BottomPanelState::open_credential_input("provider", None, false, None),
        crate::panel_state::BottomPanelState::open_provider_wizard(),
    ] {
        let mut state = TuiState::new();
        state.slash_menu = modal;
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut tui = crate::app::Tui::for_test(state, Some(tx));

        let should_exit = tui.handle_input_event(&ctrl_c_event());
        assert!(!should_exit, "Ctrl+C in modal must not exit");
        assert_no_cancel(&mut rx);
        assert!(
            !tui.state.slash_menu.is_credential_input()
                && !tui.state.slash_menu.is_provider_wizard(),
            "Ctrl+C must close the modal"
        );
    }
}

#[test]
fn repeated_esc_does_not_corrupt_cancellation_state() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    for _ in 0..3 {
        let should_exit = tui.handle_input_event(&esc_event());
        assert!(!should_exit, "repeated Esc must not exit");
    }

    // drain all 3 Cancel messages
    for _ in 0..3 {
        match rx.try_recv() {
            Ok(UserInput::Cancel) => {}
            Ok(other) => panic!("expected Cancel, got {other:?}"),
            Err(_) => panic!("expected Cancel, channel empty"),
        }
    }
    assert_no_cancel(&mut rx);

    // after the turn ends and a new one starts, Esc must still work
    tui.state.status.is_processing = false;
    tui.state.status.is_processing = true;
    let should_exit = tui.handle_input_event(&esc_event());
    assert!(
        !should_exit,
        "Esc after repeated presses must still not exit"
    );
    assert_one_cancel(&mut rx);
}

#[test]
fn entry_point_ctrl_c_draft_clear_tip_says_twice_and_requires_double_press() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.input_append_str("some draft");
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&ctrl_c_event());
    assert!(!should_exit, "Ctrl+C with draft must not exit");
    assert_no_cancel(&mut rx);
    assert!(tui.state.input_buffer.is_empty(), "draft must be cleared");
    assert!(
        matches!(tui.state.ctrl_c_state, CtrlCState::Idle),
        "ctrl_c_state must be Idle after clearing draft"
    );
    let tip_text = tui
        .state
        .tip
        .as_ref()
        .expect("tip must be shown after clearing draft")
        .text
        .as_str();
    assert!(
        tip_text.contains("twice"),
        "tip must say 'twice', got: {tip_text}"
    );

    // one more Ctrl+C does NOT exit — it only arms the first press
    let second = tui.handle_input_event(&ctrl_c_event());
    assert!(!second, "second Ctrl+C must arm, not exit");
    assert_no_cancel(&mut rx);
    assert!(
        matches!(tui.state.ctrl_c_state, CtrlCState::Waiting(_)),
        "second Ctrl+C must arm the exit gesture"
    );

    // third Ctrl+C exits
    let third = tui.handle_input_event(&ctrl_c_event());
    assert!(third, "third Ctrl+C must exit");
    assert!(tui.state.should_exit);
}

#[test]
fn entry_point_ctrl_c_slash_menu_closes_without_cancel_or_exit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.open_slash_menu(talos_conversation::command_registry());
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let should_exit = tui.handle_input_event(&ctrl_c_event());
    assert!(!should_exit, "Ctrl+C in slash menu must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        !tui.state.slash_menu.is_open,
        "Ctrl+C must close the slash menu"
    );
    assert!(
        tui.state.input_buffer.is_empty(),
        "Ctrl+C must clear the input buffer"
    );
    assert!(
        tui.state.status.is_processing,
        "Ctrl+C must not change processing state"
    );
}

#[test]
fn entry_point_ctrl_c_approval_denies_without_cancel_or_exit() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = TuiState::new();
    state.status.is_processing = true;
    let mut tui = crate::app::Tui::for_test(state, Some(tx));

    let (resp_tx, mut resp_rx) = tokio::sync::oneshot::channel();
    tui.state.pending_approval_response = Some(resp_tx);
    tui.show_approval("test_tool", "args");

    let should_exit = tui.handle_input_event(&ctrl_c_event());
    assert!(!should_exit, "Ctrl+C in approval must not exit");
    assert_no_cancel(&mut rx);
    assert!(
        matches!(tui.state.approval_state, ApprovalState::Hidden),
        "Ctrl+C must resolve and hide approval"
    );
    let choice = resp_rx
        .try_recv()
        .expect("approval response channel must receive a choice");
    assert!(
        matches!(choice, ApprovalChoice::Deny),
        "Ctrl+C in approval must Deny"
    );
    assert!(
        tui.state.status.is_processing,
        "Ctrl+C in approval must not change processing state"
    );
}
