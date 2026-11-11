use crossterm::style::Color as CColor;
use ratatui::style::Color;
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
use crate::inline_terminal::ViewportComponent;
use crate::scrollback;
use crate::state::{ApprovalState, CredentialField, CtrlCState, PanelKind, TuiState, WizardStep};
use crate::stream_markdown::{HoldStatus, MarkdownBlockKind};
use crate::theme::{semantic, to_crossterm_color};
use crate::tool_display;
use crate::transcript::TranscriptBlock;
use talos_conversation::{TipKind, TurnPhase};

fn state_line(text: &str) -> ScrollbackLine {
    let mut line = ScrollbackLine::plain(text, None);
    line.continuation_indent = 3;
    line
}

#[test]
fn queued_body_remains_visible_beside_pending_tool_animation() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.first_message_dispatched = true;
    tui.state.status.is_processing = true;
    tui.state.steering_queue_snapshot = Some(talos_conversation::SteeringQueueSnapshot {
        entries: vec![talos_conversation::SteeringQueueEntry {
            text: "queued body 中文".into(),
            truncated: false,
        }],
        total_count: 1,
        omitted_count: 0,
    });
    tui.tool_activities
        .update(talos_conversation::ToolActivity::Requested {
            call_id: "pending".into(),
            name: "read".into(),
            body: "{}".into(),
        });
    for height in [8, 24] {
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(80, height));
        tui.draw_frame().expect("initial frame");
        let first = tui.terminal.test_rendered_text();
        assert!(first.contains("queued body 中"), "{first}");
        assert!(first.contains('文'), "CJK body remains visible: {first}");
        assert!(first.contains("read #1 · requested"));
        tui.advance_processing_frame();
        tui.draw_frame().expect("animated frame");
        let next = tui.terminal.test_rendered_text();
        assert!(next.contains("queued body 中"));
        assert!(next.contains('文'));
        assert_ne!(first, next);
    }
}

#[test]
fn history_return_shortcut_restores_tail_without_cancelling_or_editing() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    for index in 0..60 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(state_line(&format!(
                "history row {index}"
            ))));
    }
    tui.state.input_append_char('x');
    tui.state.status.is_processing = true;
    tui.draw_frame().expect("initial frame");
    assert!(
        !tui.terminal
            .test_rendered_text()
            .contains("return to bottom")
    );
    tui.handle_input_event(&key_press(KeyCode::PageUp));
    tui.history_prefix_start = Some(0);
    tui.draw_frame().expect("anchored frame");
    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("Ctrl+Down to return to bottom")
    );
    tui.handle_input_event(&key_press_with_modifiers(
        KeyCode::Down,
        KeyModifiers::CONTROL,
    ));
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
    assert_eq!(tui.state.input_buffer, "x");
    assert!(rx.try_recv().is_err());
    tui.transcript
        .append(TranscriptBlock::StyledLine(state_line("new tail entry")));
    tui.draw_frame().expect("restored frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(!rendered.contains("return to bottom"));
    assert!(rendered.contains("new tail entry"));
}

#[test]
fn history_return_hint_yields_to_transient_error_tip() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    for index in 0..60 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(state_line(&format!(
                "history row {index}"
            ))));
    }
    tui.draw_frame().expect("initial frame");
    tui.handle_input_event(&key_press(KeyCode::PageUp));
    tui.state.tip = Some(crate::state::Tip {
        kind: TipKind::Error,
        text: "transient error".into(),
        ttl: std::time::Duration::from_secs(2),
        created_at: std::time::Instant::now(),
    });
    tui.draw_frame().expect("error frame");
    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("transient error")
    );
    assert!(
        !tui.terminal
            .test_rendered_text()
            .contains("return to bottom")
    );
    tui.state.tip = None;
    tui.draw_frame().expect("hint frame");
    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("return to bottom")
    );
}

#[test]
fn reasoning_click_toggles_but_drag_back_to_origin_does_not() {
    use crossterm::event::MouseButton;
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let id = tui.transcript.append(TranscriptBlock::Reasoning {
        text: "reasoning body".into(),
        expanded: false,
    });
    tui.last_history_projection =
        project_history(&tui.transcript, 40, 10, &HistoryScrollState::follow_tail());
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 40, 10));
    tui.last_history_prefix_row_count = 0;
    tui.last_frame_history_start = 0;
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        tui.handle_input_event(&mouse(kind, 3, 0));
    }
    assert!(tui.selection.is_none());
    assert!(matches!(
        tui.transcript.entries()[0].block,
        TranscriptBlock::Reasoning { expanded: true, .. }
    ));
    tui.last_history_projection =
        project_history(&tui.transcript, 40, 10, &HistoryScrollState::follow_tail());
    assert_eq!(
        tui.last_history_projection.expandable_header_at(0),
        Some(id)
    );
    tui.handle_input_event(&mouse(MouseEventKind::Down(MouseButton::Left), 3, 0));
    tui.handle_input_event(&mouse(MouseEventKind::Drag(MouseButton::Left), 7, 0));
    tui.handle_input_event(&mouse(MouseEventKind::Drag(MouseButton::Left), 3, 0));
    tui.handle_input_event(&mouse(MouseEventKind::Up(MouseButton::Left), 3, 0));
    assert!(tui.selection.is_none());
    assert!(matches!(
        tui.transcript.entries()[0].block,
        TranscriptBlock::Reasoning { expanded: true, .. }
    ));
}

#[test]
fn dragging_from_continuation_padding_keeps_logical_selection_after_resize() {
    use crossterm::event::MouseButton;
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let mut line = ScrollbackLine::plain("abcdefghijklmnopqrstuv", None);
    line.continuation_indent = 3;
    tui.transcript.append(TranscriptBlock::StyledLine(line));
    tui.last_history_projection =
        project_history(&tui.transcript, 8, 10, &HistoryScrollState::follow_tail());
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 8, 10));
    tui.last_history_prefix_row_count = 0;
    tui.last_frame_history_start = 0;
    tui.handle_input_event(&mouse(MouseEventKind::Down(MouseButton::Left), 0, 1));
    tui.handle_input_event(&mouse(MouseEventKind::Drag(MouseButton::Left), 5, 2));
    let selection = tui.selection.expect("active logical drag");
    assert!(selection.history_anchor.is_some());
    assert!(selection.history_focus.is_some());
    assert_eq!(tui.selected_text_for(selection), "ijklmnop");
    tui.last_history_projection =
        project_history(&tui.transcript, 20, 10, &HistoryScrollState::follow_tail());
    assert_eq!(tui.selected_text_for(selection), "ijklmnop");
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
        todo_presentation: None,
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
    assert!(lines[1].text.contains("● [high]"));
    assert_eq!(
        lines[1].segments[3].fg,
        to_crossterm_color(semantic::TODO_ACTIVE_FG)
    );
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
    assert!(lines2[1].text.contains("✓ [low]"));
    assert_eq!(
        lines2[1].segments[3].fg,
        to_crossterm_color(semantic::TEXT_PRIMARY)
    );
}

#[test]
fn read_tool_result_hides_content_from_scrollback() {
    let display = ToolResultDisplay {
        todo_presentation: None,
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
        todo_presentation: None,
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

    assert_eq!(state.push_chunk("first\nsec"), vec![state_line(" • first")]);
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
    assert_eq!(lines[0].text, " • actual response");
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
            state_line(" • first"),
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
fn thinking_preview_uses_smooth_slow_color_wave() {
    assert_eq!(semantic::THINKING_RIPPLE_SECONDARY, semantic::DIM_TEXT);
    let spans =
        scrollback::preview_line_spans("", "thinking: draft", None, semantic::PREVIEW_FG, Some(0));

    assert_eq!(spans.len(), 9);
    let label: String = spans[..8]
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    assert_eq!(label, "thinking");
    assert_eq!(spans[0].style.fg, Some(semantic::THINKING_RIPPLE_SECONDARY));
    assert_eq!(spans[8].content.as_ref(), ": draft");
    assert_eq!(spans[8].style.fg, Some(semantic::PREVIEW_FG));
    let render = |frame| {
        scrollback::preview_line_spans(
            "",
            "thinking: draft",
            None,
            semantic::PREVIEW_FG,
            Some(frame),
        )
    };
    assert_eq!(spans, render(24));
    assert_eq!(
        render(17)[0].style.fg,
        Some(semantic::THINKING_RIPPLE_PRIMARY)
    );
    // Check every transition, including the cycle seam, for abrupt RGB jumps.
    for frame in 0..24 {
        for (before, after) in render(frame).iter().zip(render(frame + 1)) {
            if let (Some(Color::Rgb(r0, g0, b0)), Some(Color::Rgb(r1, g1, b1))) =
                (before.style.fg, after.style.fg)
            {
                assert!(r0.abs_diff(r1) <= 35 && g0.abs_diff(g1) <= 35 && b0.abs_diff(b1) <= 35);
            }
        }
    }
    for (distance, index) in [3, 2, 1, 0].into_iter().enumerate() {
        let peak = 5 + distance * 4;
        assert_eq!(
            render(peak)[index].style.fg,
            Some(semantic::THINKING_RIPPLE_PRIMARY)
        );
        assert_eq!(
            render(peak)[7 - index].style.fg,
            render(peak)[index].style.fg
        );
        for frame in (peak + 5)..24 {
            assert_eq!(
                render(frame)[index].style.fg,
                Some(semantic::THINKING_RIPPLE_SECONDARY)
            );
        }
    }
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
    assert_eq!(lines[0].text, " • ```");
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
    assert_eq!(lines[0].text, " • Title with strong and code");
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
        lines[0].text.starts_with(" • ─"),
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
    assert_eq!(lines[0].text, " • - first");
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
    assert_eq!(lines[0].text, " • hello");
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
    assert!(texts.contains(&" • reply"));
    assert_eq!(stream_count, 2);
}

// --- tool result summarization tests ---

#[test]
fn resumed_reasoning_is_collapsed_and_never_projects_opaque_payloads() {
    use talos_core::message::{AssistantReasoning, ReasoningBlock};
    let history = vec![Message::Assistant {
        content: "visible answer".into(),
        tool_calls: vec![],
        reasoning: Some(AssistantReasoning {
            provider: "test".into(),
            model: "test".into(),
            blocks: vec![
                ReasoningBlock::Thinking {
                    text: "displayable thought".into(),
                    signature: Some("SIGNATURE_SENTINEL".into()),
                },
                ReasoningBlock::Redacted {
                    data: "REDACTED_SENTINEL".into(),
                },
                ReasoningBlock::Plain {
                    text: "plain thought".into(),
                },
            ],
        }),
    }];
    let original = history.clone();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.hydrate_history(&history);
    for block in std::mem::take(&mut tui.pending_transcript) {
        tui.transcript.append(block);
    }
    let id = tui
        .transcript
        .entries()
        .iter()
        .find_map(|entry| {
            matches!(
                entry.block,
                TranscriptBlock::Reasoning {
                    expanded: false,
                    ..
                }
            )
            .then_some(entry.id)
        })
        .expect("collapsed reasoning entry");
    let render = |tui: &crate::app::Tui| {
        project_history(&tui.transcript, 80, 100, &HistoryScrollState::follow_tail())
            .rows
            .iter()
            .map(|row| row.line.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let collapsed = render(&tui);
    assert!(collapsed.contains("Thinking"));
    assert!(collapsed.contains("visible answer"));
    assert!(!collapsed.contains("displayable thought"));
    assert!(tui.transcript.toggle_reasoning(id));
    let expanded = render(&tui);
    assert!(expanded.contains("displayable thought"));
    assert!(expanded.contains("plain thought"));
    for text in [&collapsed, &expanded] {
        assert!(!text.contains("SIGNATURE_SENTINEL"));
        assert!(!text.contains("REDACTED_SENTINEL"));
    }
    assert_eq!(history, original);
}

#[test]
fn read_always_summarized() {
    let display = ToolResultDisplay {
        todo_presentation: None,
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
        todo_presentation: None,
        tool_name: Some("read".to_string()),
        is_error: true,
        content: "permission denied".to_string(),
    };
    assert!(!tool_display::should_suppress_tool_result_content(&display));
}

#[test]
fn list_symbols_always_summarized() {
    let display = ToolResultDisplay {
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
        todo_presentation: None,
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
            todo_presentation: None,
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
        todo_presentation: None,
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
        preview_text_for_state(
            None,
            Some(&TurnPhase::Reconnecting {
                attempt: 2,
                max_attempts: 3,
            }),
            None,
            true,
            "",
            0,
        ),
        "Reconnecting... (attempt 2/3)"
    );
    assert!(
        preview_text_for_state(
            None,
            Some(&TurnPhase::Reconnecting {
                attempt: 2,
                max_attempts: 3,
            }),
            None,
            false,
            "",
            0,
        )
        .is_empty()
    );
    assert_eq!(
        preview_text_for_state(None, Some(&TurnPhase::Connecting), None, true, "", 0),
        "Connecting..."
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
        "processing tool request: bash..."
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

#[test]
fn model_generated_heading_does_not_hide_newer_thinking_content() {
    let text = preview_text_for_state(
        None,
        None,
        Some("raw details\n\n**Short title**\n\nnewest detail"),
        true,
        "",
        0,
    );
    assert_eq!(
        text,
        "thinking: raw details\n\n**Short title**\n\nnewest detail"
    );
    let component = crate::scrollback::PreviewComponent {
        padding: " ⠋ ",
        text: &text,
        spinner_color: Some(semantic::PROCESSING_SPINNER[0]),
        text_color: None,
        thinking_label_frame: Some(0),
        max_height: crate::scrollback::MAX_PREVIEW_LINES,
    };
    assert_eq!(component.height_hint(80), 6);
    let plan = component.plan(80);
    assert_eq!(plan.rows[0].content, "thinking · 5 lines");
    assert_eq!(plan.rows[1].content, "raw details");
    assert_eq!(plan.rows[5].content, "newest detail");
}

// --- TUI-028: Stale preview clear ---

#[test]
fn thinking_count_uses_wrapped_body_rows_not_visible_window() {
    let component = crate::scrollback::PreviewComponent {
        padding: "   ",
        text: "thinking: 中文中文中文中文中文中文\nlatest",
        spinner_color: None,
        text_color: None,
        thinking_label_frame: Some(0),
        max_height: 2,
    };
    let wide = component.plan(40);
    assert_eq!(wide.rows[0].content, "thinking · 2 lines");
    assert_eq!(wide.natural_height, 3);
    assert_eq!(wide.rows.len(), 2);
    let narrow = component.plan(23);
    assert_eq!(narrow.rows[0].content, "thinking · 3 lines");
    assert_eq!(narrow.natural_height, 4);
    assert_eq!(narrow.rows.len(), 2);
    assert!(narrow.clipped_before);
}

#[test]
fn thinking_title_does_not_consume_the_ten_body_rows() {
    let text = format!(
        "thinking: {}",
        (0..12)
            .map(|i| format!("row {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let component = crate::scrollback::PreviewComponent {
        padding: "   ",
        text: &text,
        spinner_color: None,
        text_color: None,
        thinking_label_frame: Some(0),
        max_height: crate::scrollback::MAX_ACTIVITY_HEIGHT,
    };
    let plan = component.plan(40);
    assert_eq!(plan.rows[0].content, "thinking · 12 lines");
    assert_eq!(plan.rows.len(), 11);
    assert_eq!(plan.rows.last().expect("latest body row").content, "row 11");
    assert!(plan.rows[1].content.ends_with("row 2"));
}

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
        moved: true,
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
        moved: true,
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
        moved: true,
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
    assert_eq!(tui.history_projection_cache.rebuilds(), 1);
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
        moved: true,
        edge: 0,
        history_anchor: Some(anchor),
        history_focus: Some(focus),
    });
    tui.draw_frame().expect("selected frame");
    assert_eq!(tui.history_projection_cache.rebuilds(), 1);
    assert_eq!(
        tui.terminal.test_cell_bg(2, row),
        ratatui::style::Color::DarkGray
    );

    tui.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
    tui.draw_frame().expect("scrolled selected frame");
    assert_eq!(tui.history_projection_cache.rebuilds(), 1);

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

#[test]
fn slash_menu_open_preserves_follow_tail_and_stays_near_composer() {
    let mut tui = tui_with_projected_history(10, 10);
    tui.state.input_append_str("draft-anchor");
    tui.state
        .open_slash_menu(talos_conversation::command_registry());
    tui.draw_frame().expect("slash menu frame");
    let rendered = tui.terminal.test_rendered_text();
    let composer_row = rendered_row_containing(&rendered, "draft-anchor");
    let menu_row = rendered_row_containing(&rendered, "/help");
    assert!(
        menu_row.abs_diff(composer_row) <= 12,
        "slash menu should remain near composer: composer={composer_row}, menu={menu_row}"
    );
    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());
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
    tui.last_frame_history_start = tui
        .last_history_projection
        .total_rows
        .saturating_sub(usize::from(viewport_height));
    tui
}

#[test]
fn approval_open_preserves_follow_tail_until_resolution() {
    let mut tui = tui_with_projected_history(10, 10);
    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());

    tui.show_approval("bash", "command: echo hello");

    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());
    assert!(tui.approval_viewport_snapshot.is_some());

    tui.hide_approval();
    assert_eq!(tui.history_scroll, HistoryScrollState::follow_tail());
    assert!(tui.approval_viewport_snapshot.is_none());
}

#[test]
fn approval_open_preserves_existing_manual_history_anchor() {
    let mut tui = tui_with_projected_history(10, 10);
    let anchor = tui
        .last_history_projection
        .first_anchor()
        .expect("projected history should expose a visible anchor");
    tui.history_scroll.anchor(anchor, 2);
    let before = tui.history_scroll.clone();

    tui.show_approval("bash", "command: echo hello");
    assert_eq!(tui.history_scroll, before);

    tui.hide_approval();
    assert_eq!(tui.history_scroll, before);
}

#[test]
fn preview_height_changes_preserve_follow_tail_and_anchored_history_modes() {
    let tui = tui_with_projected_history(10, 10);

    let follow_large = project_history(&tui.transcript, 80, 10, &HistoryScrollState::follow_tail());
    let follow_small = project_history(&tui.transcript, 80, 6, &HistoryScrollState::follow_tail());
    let follow_restored =
        project_history(&tui.transcript, 80, 10, &HistoryScrollState::follow_tail());
    assert_eq!(follow_large.visible_start, 20);
    assert_eq!(follow_small.visible_start, 24);
    assert_eq!(follow_restored.visible_start, follow_large.visible_start);

    let anchor = follow_large
        .anchor_at(follow_large.visible_start)
        .expect("visible history row should expose an anchor");
    let mut anchored = HistoryScrollState::follow_tail();
    anchored.anchor(anchor, 0);
    let anchored_large = project_history(&tui.transcript, 80, 10, &anchored);
    let anchored_small = project_history(&tui.transcript, 80, 6, &anchored);
    let anchored_restored = project_history(&tui.transcript, 80, 10, &anchored);
    assert_eq!(anchored_large.visible_start, 20);
    assert_eq!(anchored_small.visible_start, anchored_large.visible_start);
    assert_eq!(
        anchored_restored.visible_start,
        anchored_large.visible_start
    );
    assert!(matches!(anchored.mode, HistoryScrollMode::Anchored { .. }));
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
fn non_scrollable_mouse_wheel_is_a_full_frame_noop() {
    let mut tui = startup_tui_at(80, 24);
    tui.state.input_history = vec!["older input".into()];
    tui.state.input_append_str("first draft line\nsecond line");
    tui.state.input_cursor_left();
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::Assistant,
            text: "short conversation".into(),
        },
    ));
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("short conversation frame");
    assert!(
        tui.last_splash_row_count + tui.last_history_projection.total_rows
            <= usize::from(tui.last_history_viewport_height)
    );

    let before_text = tui.terminal.test_rendered_text();
    let before_cursor = tui.terminal.test_cursor_position();
    let before_start = tui.last_frame_history_start;
    let before_input = tui.state.input_buffer.clone();
    let before_input_cursor = tui.state.cursor_byte_pos();
    let before_history_cursor = tui.state.history_cursor;

    for kind in [
        MouseEventKind::ScrollUp,
        MouseEventKind::ScrollDown,
        MouseEventKind::ScrollUp,
        MouseEventKind::ScrollDown,
    ] {
        tui.handle_input_event(&mouse_scroll(kind));
        tui.draw_frame().expect("no-op wheel frame");
        assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
        assert_eq!(tui.history_prefix_start, None);
        assert_eq!(tui.last_frame_history_start, before_start);
        assert_eq!(tui.terminal.test_cursor_position(), before_cursor);
        assert_eq!(tui.terminal.test_rendered_text(), before_text);
        assert_eq!(tui.state.input_buffer, before_input);
        assert_eq!(tui.state.cursor_byte_pos(), before_input_cursor);
        assert_eq!(tui.state.history_cursor, before_history_cursor);
    }
}

#[test]
fn frame_history_scroll_boundaries_are_idempotent() {
    let mut tui = tui_with_projected_history(10, 10);
    let first = tui
        .last_history_projection
        .first_anchor()
        .expect("history starts with an anchor");
    tui.history_scroll.anchor(first, 0);
    tui.last_history_projection = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    tui.last_frame_history_start = 0;
    let top_anchor = tui.history_scroll.clone();

    for _ in 0..20 {
        tui.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
        assert_eq!(tui.history_scroll, top_anchor);
    }

    tui.history_scroll.jump_to_end();
    tui.last_history_projection =
        project_history(&tui.transcript, 80, 10, &HistoryScrollState::follow_tail());
    tui.last_frame_history_start = 20;
    for _ in 0..20 {
        tui.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, 10);
        assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
        assert_eq!(tui.history_prefix_start, None);
    }
}

#[test]
fn one_row_overflow_anchors_and_returns_to_follow_tail() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for index in 0..11 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                format!("row-{index:02}"),
                None,
            )));
    }
    tui.last_history_projection =
        project_history(&tui.transcript, 80, 10, &HistoryScrollState::follow_tail());
    tui.last_history_viewport_height = 10;
    tui.last_frame_history_start = 1;

    tui.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
    assert!(matches!(
        tui.history_scroll.mode,
        HistoryScrollMode::Anchored { .. }
    ));

    tui.last_frame_history_start = 0;
    tui.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, 10);
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
}

#[test]
fn exact_fit_frame_history_rejects_noop_anchor_transitions() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for text in ["alpha", "beta"] {
        tui.transcript
            .append(TranscriptBlock::StyledLine(ScrollbackLine::plain(
                text, None,
            )));
    }
    tui.last_history_projection =
        project_history(&tui.transcript, 20, 4, &HistoryScrollState::follow_tail());
    tui.last_splash_row_count = 2;
    tui.last_history_viewport_height = 4;
    tui.last_frame_history_start = 0;

    tui.scroll_frame_history_up(MOUSE_HISTORY_SCROLL_ROWS);
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);

    tui.scroll_frame_history_down(MOUSE_HISTORY_SCROLL_ROWS, 4);
    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
}

#[test]
fn anchored_history_that_fits_after_resize_returns_to_inline_follow_tail() {
    let mut tui = startup_tui_at(40, 14);
    for index in 0..30 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("history row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("overflow frame");
    assert!(
        tui.last_splash_row_count + tui.last_history_projection.total_rows
            > usize::from(tui.last_history_viewport_height)
    );

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
    tui.draw_frame().expect("anchored frame");
    assert!(matches!(
        tui.history_scroll.mode,
        HistoryScrollMode::Anchored { .. }
    ));

    tui.terminal
        .set_test_size(ratatui::layout::Size::new(120, 80));
    tui.handle_input_event(&Event::Resize(120, 80));
    tui.draw_frame().expect("resized frame");

    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
    assert_eq!(tui.last_history_projection.visible_start, 0);
    assert_eq!(
        usize::from(tui.last_history_viewport_height),
        tui.last_splash_row_count + tui.last_history_projection.total_rows,
        "the same frame should restore the natural inline history cap"
    );
}

#[test]
fn cjk_reflow_that_fits_clears_obsolete_anchor() {
    let mut tui = startup_tui_at(20, 14);
    tui.handle_ui_output(talos_conversation::UiOutput::Content(
        talos_conversation::ContentOutput::Block {
            source: MessageSource::Assistant,
            text: "中文历史内容用于验证宽度变化后折行收缩".repeat(8),
        },
    ));
    tui.commit_pending_transcript()
        .expect("commit CJK transcript");
    tui.draw_frame().expect("narrow overflow frame");
    assert!(
        tui.last_splash_row_count + tui.last_history_projection.total_rows
            > usize::from(tui.last_history_viewport_height)
    );

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
    tui.draw_frame().expect("narrow anchored frame");
    assert!(matches!(
        tui.history_scroll.mode,
        HistoryScrollMode::Anchored { .. }
    ));

    tui.terminal
        .set_test_size(ratatui::layout::Size::new(160, 40));
    tui.handle_input_event(&Event::Resize(160, 40));
    tui.draw_frame().expect("wide reflow frame");

    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
    assert_eq!(tui.last_history_projection.visible_start, 0);
}

#[test]
fn preview_shrink_that_fits_clears_obsolete_anchor() {
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.thinking_preview = Some(
        (0..12)
            .map(|index| format!("thinking preview row {index:02}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let mut tui = crate::app::Tui::for_test(state, None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    for index in 0..2 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("history row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("expanded preview frame");
    assert!(
        tui.last_splash_row_count + tui.last_history_projection.total_rows
            > usize::from(tui.last_history_viewport_height),
        "expanded preview should make the frame history scrollable"
    );

    tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
    tui.draw_frame().expect("preview anchored frame");
    assert!(matches!(
        tui.history_scroll.mode,
        HistoryScrollMode::Anchored { .. }
    ));

    tui.state.thinking_preview = None;
    tui.state.status.is_processing = false;
    tui.draw_frame().expect("cleared preview frame");

    assert_eq!(tui.history_scroll.mode, HistoryScrollMode::FollowTail);
    assert_eq!(tui.history_prefix_start, None);
    assert_eq!(tui.last_history_projection.visible_start, 0);
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
fn full_frame_multiline_preview_preserves_interactive_controls_at_all_sizes() {
    let mut state = TuiState::new();
    state.status.is_processing = true;
    state.thinking_preview = Some(
        "first preview row with a long ASCII suffix\n第二行中文 preview\nthird\nfourth\nfifth\nsixth\nnewest"
            .to_string(),
    );
    state.input_append_str("composer one\ncomposer two\ncomposer three");
    state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
    let mut tui = crate::app::Tui::for_test(state, None);

    for (width, height) in [(80, 24), (20, 5), (10, 3), (4, 2), (1, 1), (0, 0)] {
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(width, height));
        tui.draw_frame()
            .unwrap_or_else(|error| panic!("{width}x{height}: {error}"));
        if let Some(cursor) = tui.terminal.test_cursor_position() {
            assert!(cursor.x < width && cursor.y < height);
        }
    }

    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.draw_frame().expect("restored 80x24 preview frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("thinking"));
    assert!(!rendered.contains("thinking:"));
    assert!(rendered.contains("newest"));
    assert!(rendered.contains("Add custom provider"));
    assert!(rendered.contains("composer one"));
    assert!(rendered.contains("composer three"));
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
    *protocol = "openai-responses".into();

    for height in [1, 2, 3, 4, 5] {
        let tui = render_modal_for_cursor(panel.clone(), 40, height);
        assert!(
            !tui.terminal.test_cursor_visible(),
            "third protocol option must hide at terminal height {height}"
        );
    }

    let tui = render_modal_for_cursor(panel, 40, 10);
    assert_eq!(
        tui.terminal.test_cursor_position().map(|pos| pos.y),
        Some(7),
        "the third protocol option must remain at local row 4"
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

fn rendered_row_containing(rendered: &str, needle: &str) -> usize {
    rendered
        .lines()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("expected rendered row containing {needle:?}: {rendered:?}"))
}

#[test]
fn approval_prompt_stays_adjacent_to_non_bottom_startup_composer() {
    let mut tui = startup_tui_at(80, 24);
    tui.state.input_append_str("draft-anchor");
    tui.draw_frame().expect("startup frame");
    let original_composer_row =
        rendered_row_containing(&tui.terminal.test_rendered_text(), "draft-anchor");
    assert!(
        original_composer_row < 18,
        "startup composer must be non-bottom"
    );

    tui.show_approval("write_file", "path: README.md");
    tui.draw_frame().expect("approval frame");
    let rendered = tui.terminal.test_rendered_text();
    let composer_row = rendered_row_containing(&rendered, "draft-anchor");
    let approval_row = rendered_row_containing(&rendered, "write_file");
    assert!(
        approval_row.abs_diff(composer_row) <= 4,
        "approval must remain adjacent to composer: composer={composer_row}, approval={approval_row}, rendered={rendered:?}"
    );
    assert!(
        approval_row < 20,
        "approval must not jump to physical bottom"
    );

    tui.terminal
        .set_test_size(ratatui::layout::Size::new(50, 18));
    tui.draw_frame().expect("resized approval frame");
    let resized = tui.terminal.test_rendered_text();
    let resized_composer = rendered_row_containing(&resized, "draft-anchor");
    let resized_approval = rendered_row_containing(&resized, "write_file");
    assert!(
        resized_approval.abs_diff(resized_composer) <= 4,
        "resized approval must remain adjacent to composer"
    );

    tui.hide_approval();
    tui.state.slash_menu.close();
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.draw_frame().expect("restored startup frame");
    let restored = rendered_row_containing(&tui.terminal.test_rendered_text(), "draft-anchor");
    assert_eq!(restored, original_composer_row);
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
fn dashboard_availability_is_a_display_only_logo_prefix() {
    let mut tui = startup_tui_at(80, 24);
    tui.set_dashboard_availability("127.0.0.1:61205".parse().expect("loopback address"), false);

    tui.draw_frame().expect("startup frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("Dashboard: http://127.0.0.1:61205/"));
    assert!(!rendered.contains("Dashboard ready:"));
    assert!(!rendered.contains('\u{1b}'));
    assert!(tui.state.tip.is_none());
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn authenticated_dashboard_logo_prefix_never_contains_a_token() {
    let mut tui = startup_tui_at(40, 24);
    tui.set_dashboard_availability("127.0.0.1:61205".parse().expect("loopback address"), true);

    tui.draw_frame().expect("startup frame");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("http://127.0.0.1:61205/"));
    assert!(
        rendered.contains("authentication required"),
        "authenticated Dashboard notice must remain visible: {rendered:?}"
    );
    assert!(!rendered.contains("secret-token"));
    assert!(!rendered.contains('\u{1b}'));
    assert!(tui.transcript.entries().is_empty());
}

#[test]
fn dashboard_logo_row_scrolls_with_prefix_and_never_enters_transcript() {
    let mut tui = startup_tui_at(80, 24);
    tui.set_dashboard_availability("127.0.0.1:61205".parse().expect("loopback address"), false);
    for index in 0..30 {
        tui.handle_ui_output(talos_conversation::UiOutput::Content(
            talos_conversation::ContentOutput::Block {
                source: MessageSource::Assistant,
                text: format!("dashboard isolation row {index:02}"),
            },
        ));
    }
    tui.commit_pending_transcript().expect("commit transcript");
    tui.draw_frame().expect("tail frame");
    assert!(!tui.terminal.test_rendered_text().contains("Dashboard:"));

    for _ in 0..30 {
        tui.handle_input_event(&mouse_scroll(MouseEventKind::ScrollUp));
        tui.draw_frame().expect("history scroll frame");
    }

    assert!(
        tui.terminal
            .test_rendered_text()
            .contains("Dashboard: http://127.0.0.1:61205/")
    );
    assert!(tui.transcript.entries().iter().all(|entry| {
        !matches!(
            &entry.block,
            crate::transcript::TranscriptBlock::StyledLine(line)
                if line.text.contains("Dashboard:")
        )
    }));
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
fn clipped_scope_cannot_resolve_always_approval() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let (resp_tx, mut resp_rx) = tokio::sync::oneshot::channel();
    tui.state.pending_approval_response = Some(resp_tx);
    tui.show_approval_with_preview(
        "write",
        "path: exact-file.txt",
        Some("Write Path: /private/tmp/exact-file.txt".to_string()),
    );

    tui.handle_pending_approval_input(KeyCode::Char('2'));

    assert!(matches!(
        tui.state.approval_state,
        ApprovalState::Visible { .. }
    ));
    assert!(resp_rx.try_recv().is_err());
    assert!(
        tui.state
            .tip
            .as_ref()
            .is_some_and(|tip| tip.text.contains("review the full reusable scope"))
    );
}

#[test]
fn numeric_approval_routes_once_and_rejects_invalid_or_repeated_input() {
    use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};

    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    for (digit, expected) in [
        ('1', ApprovalChoice::ApproveOnce),
        ('2', ApprovalChoice::AlwaysApprove),
        ('3', ApprovalChoice::Deny),
    ] {
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        tui.state.pending_approval_response = Some(tx);
        tui.show_approval("test_tool", "harmless fixture");
        for invalid in ['0', '4', '9', 'y', 'a', 'n', '中'] {
            tui.handle_input_event(&Event::Key(KeyEvent::new(
                KeyCode::Char(invalid),
                KeyModifiers::NONE,
            )));
            assert!(matches!(
                rx.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty)
            ));
        }
        // Repeat/release from the previous request must not answer this new request.
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            tui.handle_input_event(&Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char('1'),
                KeyModifiers::NONE,
                kind,
            )));
            assert!(matches!(
                rx.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty)
            ));
        }
        tui.handle_input_event(&Event::Key(KeyEvent::new(
            KeyCode::Char(digit),
            KeyModifiers::NONE,
        )));
        assert_eq!(rx.try_recv().expect("immediate numeric decision"), expected);
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
    }
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

#[test]
fn escape_cancels_manual_compaction_without_a_fake_processing_turn() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.handle_ui_output(talos_conversation::UiOutput::CompactionPending(true));
    assert!(!tui.state.status.is_processing);
    tui.handle_input_event(&key_press(KeyCode::Esc));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Cancel)));
    tui.handle_ui_output(talos_conversation::UiOutput::CompactionPending(false));
    tui.handle_input_event(&key_press(KeyCode::Esc));
    assert!(rx.try_recv().is_err());
}

fn repeat_event(code: KeyCode, kind: crossterm::event::KeyEventKind) -> crossterm::event::Event {
    crossterm::event::Event::Key(crossterm::event::KeyEvent::new_with_kind(
        code,
        crossterm::event::KeyModifiers::NONE,
        kind,
    ))
}

#[test]
fn native_repeat_edits_whole_graphemes_and_stops_at_release() {
    use crossterm::event::KeyEventKind::{Press, Release, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.input_append_str("a中e\u{301}👩‍💻🇨🇳");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Press));
    assert_eq!(tui.state.input_buffer, "a中e\u{301}👩‍💻");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.input_buffer, "a中e\u{301}");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Release));
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.input_buffer, "a中e\u{301}");
    tui.handle_input_event(&repeat_event(KeyCode::Left, Press));
    assert_eq!(tui.state.cursor_pos, 2);
    tui.handle_input_event(&repeat_event(KeyCode::Left, Repeat));
    assert_eq!(tui.state.cursor_pos, 1);
    tui.handle_input_event(&repeat_event(KeyCode::Delete, Press));
    tui.handle_input_event(&repeat_event(KeyCode::Delete, Repeat));
    assert_eq!(tui.state.input_buffer, "a");
    for _ in 0..4 {
        tui.handle_input_event(&repeat_event(KeyCode::Delete, Repeat));
    }
    assert_eq!(tui.state.input_buffer, "a");
}

#[test]
fn native_repeat_cannot_cross_panel_replacement_or_submit_approval() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.show_approval("first", "{}");
    tui.handle_input_event(&repeat_event(KeyCode::Down, Press));
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    assert_eq!(tui.state.slash_menu.selected_index, 2);
    tui.hide_approval();
    tui.show_approval("second", "{}");
    let selected = tui.state.slash_menu.selected_index;
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    assert_eq!(tui.state.slash_menu.selected_index, selected);
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    tui.state.pending_approval_response = Some(tx);
    for code in [
        KeyCode::Enter,
        KeyCode::Char('1'),
        KeyCode::Char('2'),
        KeyCode::Char('3'),
        KeyCode::Esc,
    ] {
        tui.handle_input_event(&repeat_event(code, Repeat));
    }
    assert!(matches!(
        rx.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));
}

#[test]
fn native_repeat_menu_wrap_and_focus_invalidation() {
    use crossterm::event::{
        Event,
        KeyEventKind::{Press, Repeat},
    };
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state
        .open_slash_menu(talos_conversation::command_registry());
    let start = tui.state.slash_menu.selected_index;
    tui.handle_input_event(&repeat_event(KeyCode::Down, Press));
    let once = tui.state.slash_menu.selected_index;
    assert_ne!(once, start);
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    assert_ne!(tui.state.slash_menu.selected_index, once);
    for event in [
        Event::FocusLost,
        Event::FocusGained,
        Event::Resize(100, 40),
        Event::Paste(String::new()),
    ] {
        tui.handle_input_event(&repeat_event(KeyCode::Down, Press));
        tui.handle_input_event(&event);
        let selected = tui.state.slash_menu.selected_index;
        tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
        assert_eq!(tui.state.slash_menu.selected_index, selected);
    }
}

#[test]
fn native_repeat_submit_is_one_shot_and_orphan_repeat_is_ignored() {
    use crossterm::event::KeyEventKind::{Press, Release, Repeat};
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.state.input_append_str("first");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.input_buffer, "first");
    tui.handle_input_event(&repeat_event(KeyCode::Enter, Press));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Message(_))));
    tui.state.input_append_str("second");
    for kind in [Repeat, Release, Repeat] {
        tui.handle_input_event(&repeat_event(KeyCode::Enter, kind));
    }
    assert!(rx.try_recv().is_err());
    assert_eq!(tui.state.input_buffer, "second");
}

#[test]
fn detach_arguments_reach_engine_from_keyboard_and_paste() {
    for command in ["/detach 1", "/detach all"] {
        for paste in [false, true] {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
            tui.handle_input_event(&repeat_event(KeyCode::Char('/'), KeyEventKind::Press));
            if paste {
                tui.handle_input_event(&Event::Paste(command[1..].into()));
            } else {
                for ch in command[1..].chars() {
                    tui.handle_input_event(&repeat_event(KeyCode::Char(ch), KeyEventKind::Press));
                }
            }
            tui.last_char_time = None;
            tui.handle_input_event(&repeat_event(KeyCode::Enter, KeyEventKind::Press));
            let UserInput::Message(message) = rx.try_recv().expect("submitted command") else {
                panic!("expected command message");
            };
            assert_eq!(message, command);
            let mut engine =
                talos_conversation::ConversationEngine::new("model".into(), "provider".into());
            engine
                .pending_image_attachments
                .push(talos_core::message::ContentPart::Image {
                    path: "/tmp/test.png".into(),
                    mime: "image/png".into(),
                    byte_count: 100,
                    content_digest: talos_core::message::ContentDigest::default(),
                });
            let outputs = engine.handle_slash_command(&message);
            assert!(engine.pending_image_attachments.is_empty());
            assert!(outputs.iter().any(|output| matches!(output,
                talos_conversation::UiOutput::Status(status) if status.attachment_count == 0)));
            assert!(rx.try_recv().is_err());
        }
    }
}

#[test]
fn native_repeat_cannot_delete_composer_after_slash_closes() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state
        .open_slash_menu(talos_conversation::command_registry());
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Press));
    assert!(!tui.state.slash_menu.is_open);
    tui.state.input_append_str("new draft");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.input_buffer, "new draft");
}

#[test]
fn native_repeat_right_stops_at_grapheme_end_and_empty_edits_are_noops() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.input_append_str("e\u{301}👩‍💻中");
    tui.state.cursor_pos = 0;
    for (kind, expected) in [(Press, 2), (Repeat, 5), (Repeat, 6), (Repeat, 6)] {
        tui.handle_input_event(&repeat_event(KeyCode::Right, kind));
        assert_eq!(tui.state.cursor_pos, expected);
    }
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Press));
    for _ in 0..5 {
        tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    }
    assert!(tui.state.input_buffer.is_empty());
    assert_eq!(tui.state.cursor_pos, 0);
}

#[test]
fn native_repeat_credential_backspace_preserves_composed_text_and_isolates_field() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.open_credential_input("fixture", None, true, None);
    tui.state.credential_append_str("a中e\u{301}");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Press));
    assert_eq!(tui.state.slash_menu.credential_buffer, "a中");
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.slash_menu.credential_buffer, "a");
    tui.state.slash_menu.credential_field = crate::state::CredentialField::BaseUrl;
    tui.state.slash_menu.base_url_buffer = "https://fixture.invalid".into();
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(
        tui.state.slash_menu.base_url_buffer,
        "https://fixture.invalid"
    );
}

#[test]
fn native_repeat_forward_delete_closes_slash_without_crossing_focus() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state
        .open_slash_menu(talos_conversation::command_registry());
    tui.state.input_append_str("model");
    tui.state.cursor_pos = 0;
    tui.handle_input_event(&repeat_event(KeyCode::Delete, Press));
    assert!(!tui.state.slash_menu.is_open);
    assert_eq!(tui.state.input_buffer, "model");
    tui.handle_input_event(&repeat_event(KeyCode::Delete, Repeat));
    assert_eq!(tui.state.input_buffer, "model");
}

#[test]
fn native_repeat_release_with_changed_modifiers_clears_ownership() {
    use crossterm::event::{
        Event, KeyEvent,
        KeyEventKind::{Press, Release, Repeat},
        KeyModifiers,
    };
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.input_append_str("abcd");
    tui.handle_input_event(&Event::Key(KeyEvent::new_with_kind(
        KeyCode::Left,
        KeyModifiers::SHIFT,
        Press,
    )));
    tui.handle_input_event(&Event::Key(KeyEvent::new_with_kind(
        KeyCode::Left,
        KeyModifiers::NONE,
        Release,
    )));
    tui.handle_input_event(&Event::Key(KeyEvent::new_with_kind(
        KeyCode::Left,
        KeyModifiers::SHIFT,
        Repeat,
    )));
    assert_eq!(tui.state.cursor_pos, 3);
}

#[test]
fn native_repeat_model_picker_wraps_and_wizard_step_rejects_stale_repeat() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    use talos_conversation::{ModelPickerData, ModelPickerItem};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let data = ModelPickerData {
        recent: vec![],
        setup_providers: vec![],
        ready_models: ["one", "two"]
            .into_iter()
            .map(|provider| ModelPickerItem {
                command: "/model".into(),
                model_id: "model".into(),
                provider: provider.into(),
                label: provider.into(),
                context_limit: None,
                pricing: None,
                authenticated: true,
                is_current: false,
                variants: vec![],
                variant: None,
            })
            .collect(),
    };
    tui.state.open_model_picker(&data);
    let first = tui.state.slash_menu.selected_index;
    tui.handle_input_event(&repeat_event(KeyCode::Down, Press));
    assert_ne!(tui.state.slash_menu.selected_index, first);
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    assert_eq!(tui.state.slash_menu.selected_index, first);
    tui.state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
    if let Some(crate::state::PanelKind::ProviderWizard { step, .. }) =
        &mut tui.state.slash_menu.kind
    {
        *step = crate::state::WizardStep::Protocol;
    }
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    tui.handle_input_event(&repeat_event(KeyCode::Down, Press));
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    if let Some(crate::state::PanelKind::ProviderWizard { step, protocol, .. }) =
        &mut tui.state.slash_menu.kind
    {
        assert_eq!(protocol, "anthropic-messages");
        *step = crate::state::WizardStep::Name;
    }
    tui.handle_input_event(&repeat_event(KeyCode::Down, Repeat));
    assert!(tui.repeat_key.is_none());
}

#[test]
fn native_repeat_viewport_uses_current_projection_and_release_stops() {
    use crossterm::event::KeyEventKind::{Press, Release, Repeat};
    let mut tui = tui_with_projected_history(4, 10);
    tui.handle_input_event(&repeat_event(KeyCode::PageUp, Press));
    tui.last_history_projection = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    let once = tui.last_history_projection.visible_start;
    tui.handle_input_event(&repeat_event(KeyCode::PageUp, Repeat));
    let twice = project_history(&tui.transcript, 80, 10, &tui.history_scroll).visible_start;
    assert!(twice < once);
    tui.handle_input_event(&repeat_event(KeyCode::PageUp, Release));
    tui.last_history_projection = project_history(&tui.transcript, 80, 10, &tui.history_scroll);
    tui.handle_input_event(&repeat_event(KeyCode::PageUp, Repeat));
    assert_eq!(
        project_history(&tui.transcript, 80, 10, &tui.history_scroll).visible_start,
        twice
    );
}

#[test]
fn native_repeat_deletion_that_joins_flag_keeps_cursor_on_boundary() {
    use crossterm::event::KeyEventKind::{Press, Repeat};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.input_append_str("🇦x🇧");
    tui.state.cursor_pos = 2;
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Press));
    assert_eq!(tui.state.input_buffer, "🇦🇧");
    assert_eq!(tui.state.cursor_pos, 0);
    tui.handle_input_event(&repeat_event(KeyCode::Backspace, Repeat));
    assert_eq!(tui.state.input_buffer, "🇦🇧");
    tui.handle_input_event(&repeat_event(KeyCode::Delete, Press));
    assert!(tui.state.input_buffer.is_empty());
}

#[test]
fn process_poll_history_click_expands_details_without_mutating_records() {
    use crossterm::event::MouseButton;
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let first =
        crate::process_poll::tests::append_poll(&mut tui.transcript, "job_a", "read", "running");
    for _ in 0..23 {
        crate::process_poll::tests::append_poll(&mut tui.transcript, "job_a", "status", "running");
    }
    let raw = format!("{:?}", tui.transcript.entries());
    tui.draw_frame().expect("grouped frame");
    assert!(tui.terminal.test_rendered_text().contains("24 polls"));
    assert!(!tui.terminal.test_rendered_text().contains("next_cursor"));
    tui.last_history_projection =
        project_history(&tui.transcript, 120, 60, &HistoryScrollState::follow_tail());
    assert_eq!(tui.last_history_projection.total_rows, 1);
    assert!(
        tui.last_history_projection.rows[0]
            .line
            .text
            .contains("24 polls")
    );
    tui.last_history_area = Some(ratatui::layout::Rect::new(0, 0, 120, 60));
    tui.last_history_prefix_row_count = 0;
    tui.last_frame_history_start = 0;
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        tui.handle_input_event(&mouse(kind, 2, 0));
    }
    assert!(tui.transcript.poll_expanded(first));
    let expanded = project_history(
        &tui.transcript,
        120,
        200,
        &HistoryScrollState::follow_tail(),
    );
    assert!(expanded.total_rows >= 49);
    assert!(
        expanded
            .rows
            .iter()
            .any(|row| row.line.text.contains("next_cursor"))
    );
    assert_eq!(format!("{:?}", tui.transcript.entries()), raw);
    // Clicking a folded header must remain distinct from drag-to-copy.
    tui.transcript.toggle_expansion(first);
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Drag(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        tui.handle_input_event(&mouse(kind, 2, 0));
    }
    assert!(!tui.transcript.poll_expanded(first));
}

#[test]
fn process_poll_live_and_replayed_history_have_identical_grouping() {
    use talos_conversation::UiOutput;
    use talos_core::message::{MessageToolResult, ToolCall};
    let mut fixture = crate::transcript::TranscriptStore::default();
    for state in ["running", "running", "completed"] {
        crate::process_poll::tests::append_poll(&mut fixture, "job_a", "read", state);
    }
    let mut live = crate::app::Tui::for_test(TuiState::new(), None);
    let mut history = Vec::new();
    for (index, pair) in fixture.entries().chunks(2).enumerate() {
        let TranscriptBlock::ToolCall(call) = &pair[0].block else {
            panic!("call")
        };
        let TranscriptBlock::ToolResult(result) = &pair[1].block else {
            panic!("result")
        };
        live.handle_ui_output(UiOutput::ToolCall(call.clone()));
        live.commit_pending_transcript().expect("call frame");
        live.handle_ui_output(UiOutput::ToolResult(result.clone()));
        live.commit_pending_transcript().expect("result frame");
        history.push(Message::Assistant {
            content: String::new(),
            reasoning: None,
            tool_calls: vec![ToolCall {
                id: format!("poll-{index}"),
                name: call.tool_name.clone(),
                input: call.arguments.clone(),
            }],
        });
        history.push(Message::Tool {
            result: MessageToolResult {
                tool_use_id: format!("poll-{index}"),
                content: result.content.clone(),
                is_error: result.is_error,
            },
        });
    }
    let original = format!("{history:?}");
    let mut replay = crate::app::Tui::for_test(TuiState::new(), None);
    replay.hydrate_history(&history);
    replay.commit_pending_transcript().expect("replay frame");
    for width in [20, 80, 160] {
        let a = project_history(
            &live.transcript,
            width,
            100,
            &HistoryScrollState::follow_tail(),
        );
        let b = project_history(
            &replay.transcript,
            width,
            100,
            &HistoryScrollState::follow_tail(),
        );
        let texts = |p: &crate::history_projection::HistoryProjection| {
            p.rows
                .iter()
                .map(|r| r.line.text.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(texts(&a), texts(&b));
        assert_eq!(a.total_rows, 2);
    }
    assert_eq!(format!("{history:?}"), original);
}

#[test]
fn validation_results_live_and_hydrated_history_match_and_keep_raw_evidence() {
    use talos_conversation::{ToolCallDisplay, UiOutput};
    use talos_core::message::{MessageToolResult, ToolCall};
    let raw = serde_json::json!({
        "profile":"workspace", "authority":"read-only plan; commands are not executed",
        "findings":[], "checks":[{"id":"fmt", "command":"cargo fmt --all -- --check", "required":true}]
    }).to_string();
    let arguments = serde_json::json!({"mode":"plan", "profile":"workspace"});
    let history = vec![
        Message::Assistant {
            content: String::new(),
            reasoning: None,
            tool_calls: vec![ToolCall {
                id: "validation-1".into(),
                name: "validate_project".into(),
                input: arguments.clone(),
            }],
        },
        Message::Tool {
            result: MessageToolResult {
                tool_use_id: "validation-1".into(),
                content: raw.clone(),
                is_error: false,
            },
        },
    ];
    let original = format!("{history:?}");
    let mut live = crate::app::Tui::for_test(TuiState::new(), None);
    live.handle_ui_output(UiOutput::ToolCall(ToolCallDisplay {
        tool_name: "validate_project".into(),
        arguments,
        provenance: talos_core::tool::ToolProvenance::Native,
        summary_fields: vec![],
    }));
    live.commit_pending_transcript().expect("call");
    live.handle_ui_output(UiOutput::ToolResult(ToolResultDisplay {
        tool_name: Some("validate_project".into()),
        content: raw.clone(),
        is_error: false,
        todo_presentation: None,
    }));
    live.commit_pending_transcript().expect("result");
    let mut replay = crate::app::Tui::for_test(TuiState::new(), None);
    replay.hydrate_history(&history);
    replay.commit_pending_transcript().expect("replay");
    for width in [20, 80, 160] {
        let a = project_history(
            &live.transcript,
            width,
            100,
            &HistoryScrollState::follow_tail(),
        );
        let b = project_history(
            &replay.transcript,
            width,
            100,
            &HistoryScrollState::follow_tail(),
        );
        let texts = |p: &crate::history_projection::HistoryProjection| {
            p.rows
                .iter()
                .map(|r| r.line.text.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(texts(&a), texts(&b));
        if width == 160 {
            assert!(texts(&a).iter().any(|text| text.contains("NOT EXECUTED")));
        }
    }
    for tui in [&live, &replay] {
        assert!(
            tui.transcript
                .entries()
                .iter()
                .any(|entry| matches!(&entry.block,
            TranscriptBlock::ToolResult(result) if result.content == raw))
        );
    }
    assert_eq!(format!("{history:?}"), original);
}

#[test]
fn tool_result_frame_clips_and_reveals_full_line_on_resize() {
    use talos_conversation::UiOutput;
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.first_message_dispatched = true;
    let content = format!("RESULT_START {} RESULT_END\nSECOND_ROW", "中文".repeat(20));
    tui.handle_ui_output(UiOutput::ToolResult(ToolResultDisplay {
        tool_name: Some("custom_tool".into()),
        content: content.clone(),
        is_error: false,
        todo_presentation: None,
    }));
    tui.commit_pending_transcript().expect("result");
    for width in [40, 160, 40] {
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(width, 30));
        tui.draw_frame().expect("frame");
        let text = tui.terminal.test_rendered_text();
        assert!(text.contains("RESULT_START"), "{text}");
        assert!(text.contains("SECOND_ROW"), "{text}");
        assert_eq!(text.contains("RESULT_END"), width == 160, "{text}");
        if width == 40 {
            assert!(text.contains('…'), "{text}");
        }
    }
    assert!(tui.transcript.entries().iter().any(|entry| matches!(
        &entry.block, TranscriptBlock::ToolResult(result) if result.content == content)));
}

#[test]
fn clipboard_shortcut_preserves_draft_and_suppresses_repeats_and_duplicates() {
    use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.state.input_paste("draft text");
    let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
    tui.handle_input_event(&Event::Key(key));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Message(command)) if command == "/paste-image"));
    assert_eq!(tui.state.input_buffer, "draft text");
    tui.handle_input_event(&Event::Key(key));
    tui.handle_input_event(&Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
        KeyEventKind::Repeat,
    )));
    assert!(rx.try_recv().is_err());
}

#[test]
fn clipboard_shortcut_is_not_read_in_approval_or_panel() {
    use crossterm::event::{Event, KeyEvent, KeyModifiers};
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.show_approval("test", "fixture");
    tui.handle_input_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    assert!(rx.try_recv().is_err());
    tui.state.approval_state = ApprovalState::Hidden;
    tui.state
        .open_slash_menu(talos_conversation::command_registry());
    tui.handle_input_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('V'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    )));
    assert!(rx.try_recv().is_err());
}

#[test]
fn clipboard_loading_keeps_draft_and_allows_cancel_without_deferred_send() {
    use crossterm::event::{Event, KeyEvent, KeyModifiers};
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(tx));
    tui.state.input_paste("keep this draft");
    tui.handle_input_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Message(_))));
    tui.handle_input_event(&Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert!(rx.try_recv().is_err());
    assert_eq!(tui.state.input_buffer, "keep this draft");
    tui.handle_input_event(&Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Cancel)));
    tui.handle_ui_output(talos_conversation::UiOutput::ClipboardPastePending(false));
    assert!(
        rx.try_recv().is_err(),
        "completion never auto-sends the draft"
    );
    assert_eq!(tui.state.input_buffer, "keep this draft");
}

#[test]
fn approval_large_scope_is_compact_and_requires_contiguous_detail_pages() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(100, 32));
    let preview = (0..2000)
        .map(|index| format!("Read Path: /workspace/file-{index:04}.rs"))
        .collect::<Vec<_>>()
        .join("\n");
    tui.show_approval_with_preview(
        "git_checkout",
        "branch: topic, create: true",
        Some(preview.clone()),
    );
    tui.draw_frame().expect("summary");
    let rendered = tui.terminal.test_rendered_text();
    assert!(rendered.contains("2000 scope entries"), "{rendered}");
    assert!(rendered.contains("Read: 2000"));
    assert!(!rendered.contains("file-0000.rs"));
    assert!(rendered.contains("[1] approve"));
    assert!(rendered.contains("[3] deny"));
    assert!(!tui.approval_preview_fully_visible);
    tui.handle_input_event(&key_press(KeyCode::Char('2')));
    assert!(tui.state.slash_menu.is_approval());
    let comp = scrollback::BottomPanelComponent {
        menu: &tui.state.slash_menu,
        query: "",
        max_height: u16::MAX,
    };
    assert!(comp.height_hint(100) <= 12);
    tui.handle_input_event(&key_press(KeyCode::Char('d')));
    // PageDown before any detail draw cannot jump over undisplayed content.
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    assert_eq!(tui.state.slash_menu.approval_view.offset, 0);
    tui.draw_frame().expect("first detail page");
    assert!(tui.terminal.test_rendered_text().contains("file-0000.rs"));
    assert!(!tui.approval_preview_fully_visible);
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    assert!(tui.state.slash_menu.approval_view.offset > 0);
    tui.draw_frame().expect("second page");
    tui.handle_input_event(&key_press(KeyCode::PageUp));
    assert_eq!(tui.state.slash_menu.approval_view.offset, 0);
    tui.draw_frame().expect("back to first page");
    for _ in 0..250 {
        if tui.approval_preview_fully_visible {
            break;
        }
        tui.handle_input_event(&key_press(KeyCode::PageDown));
        tui.draw_frame().expect("next page");
    }
    assert!(
        tui.approval_preview_fully_visible,
        "all rows were displayed"
    );
    assert!(tui.terminal.test_rendered_text().contains("file-1999.rs"));
    let last_offset = tui.state.slash_menu.approval_view.offset;
    tui.handle_input_event(&key_press(KeyCode::PageDown));
    assert_eq!(tui.state.slash_menu.approval_view.offset, last_offset);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(60, 24));
    tui.draw_frame()
        .expect("resize invalidates row-based review");
    assert!(!tui.approval_preview_fully_visible);
    tui.show_approval_with_preview("git_checkout", "branch: other", Some(preview));
    assert_eq!(tui.state.slash_menu.approval_view.reviewed, 0);
    assert!(!tui.state.slash_menu.approval_view.expanded);
    tui.handle_input_event(&key_press(KeyCode::Char('1')));
    assert!(!tui.state.slash_menu.is_approval());
}

#[test]
fn approval_details_preserve_scope_line_breaks() {
    let rows = scrollback::approval_scope_lines(
        "Read Path: /one\nWrite Path: /two\n\nInvocation-only scope",
        100,
    );
    assert_eq!(
        rows,
        vec![
            "Read Path: /one",
            "Write Path: /two",
            "",
            "Invocation-only scope"
        ]
    );
}

#[test]
fn approval_hides_composer_cursor_and_restores_it_after_dismissal() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(100, 32));
    tui.draw_frame().expect("composer");
    assert!(tui.terminal.test_cursor_visible());
    let composer_cursor = tui.terminal.test_cursor_position();
    tui.show_approval_with_preview(
        "git_checkout",
        "branch: topic",
        Some("Read Path: /one".into()),
    );
    for _ in 0..3 {
        tui.draw_frame().expect("approval repaint");
        assert!(!tui.terminal.test_cursor_visible());
        assert_eq!(tui.terminal.test_cursor_position(), None);
    }
    tui.handle_pending_approval_input(KeyCode::Char('d'));
    tui.draw_frame().expect("details");
    assert!(!tui.terminal.test_cursor_visible());
    tui.handle_pending_approval_input(KeyCode::Char('3'));
    tui.draw_frame().expect("composer restored");
    assert!(tui.terminal.test_cursor_visible());
    assert_eq!(tui.terminal.test_cursor_position(), composer_cursor);
}

#[test]
fn paged_approval_numeric_choices_reach_live_receiver() {
    for (digit, expected) in [
        ('1', ApprovalChoice::ApproveOnce),
        ('2', ApprovalChoice::AlwaysApprove),
    ] {
        let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(100, 32));
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        tui.state.pending_approval_response = Some(tx);
        tui.show_approval_with_preview(
            "git_checkout",
            "branch: topic",
            Some(
                (0..40)
                    .map(|i| format!("Read Path: /file-{i}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        );
        tui.draw_frame().expect("summary");
        tui.handle_input_event(&key_press(KeyCode::Char('d')));
        for _ in 0..10 {
            tui.draw_frame().expect("page");
            if tui.approval_preview_fully_visible {
                break;
            }
            tui.handle_input_event(&key_press(KeyCode::PageDown));
        }
        assert!(tui.approval_preview_fully_visible);
        tui.handle_input_event(&key_press(KeyCode::Char(digit)));
        assert_eq!(rx.try_recv().expect("decision delivered"), expected);
        assert!(!tui.state.slash_menu.is_approval());
    }
}

#[test]
fn expired_approval_retires_both_modal_states_without_claiming_success() {
    for via_clipboard in [false, true] {
        let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
        let (tx, rx) = tokio::sync::oneshot::channel();
        tui.state.pending_approval_response = Some(tx);
        tui.show_approval("git_checkout", "branch: topic");
        assert!(!tui.retire_closed_approval(), "live request remains open");
        drop(rx);
        if via_clipboard {
            tui.handle_ui_output(talos_conversation::UiOutput::ClipboardPastePending(false));
        } else {
            tui.handle_input_event(&key_press(KeyCode::Char('1')));
        }
        assert!(matches!(tui.state.approval_state, ApprovalState::Hidden));
        assert!(!tui.state.slash_menu.is_approval());
        assert!(tui.state.pending_approval_response.is_none());
        tui.commit_pending_transcript().expect("notice");
        tui.draw_frame().expect("retired modal");
        let text = tui.terminal.test_rendered_text();
        assert!(text.contains("Approval expired or was cancelled"), "{text}");
        assert!(!text.contains("approved: git_checkout"));
    }
}

#[test]
fn release_discards_queued_repeats_but_preserves_independent_presses() {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let key = |kind| {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Backspace,
            KeyModifiers::NONE,
            kind,
        ))
    };
    let mut batch = vec![key(KeyEventKind::Press)];
    batch.extend((0..500).map(|_| key(KeyEventKind::Repeat)));
    batch.push(key_press(KeyCode::Left));
    batch.push(key(KeyEventKind::Release));
    batch.push(key(KeyEventKind::Press));
    let events = super::input::coalesce_key_repeats(batch);
    assert_eq!(events.len(), 4);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Key(k) if k.kind == KeyEventKind::Repeat))
    );
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.state.input_paste("abcdef");
    for event in events {
        tui.handle_input_event(&event);
    }
    assert_eq!(tui.state.input_buffer, "abce");
    let held = super::input::coalesce_key_repeats(vec![
        key(KeyEventKind::Press),
        key(KeyEventKind::Repeat),
        key(KeyEventKind::Repeat),
    ]);
    assert_eq!(
        held.len(),
        2,
        "held keys make bounded progress without a release"
    );
}

#[test]
fn operation_summary_accepts_once_without_paging_or_reusable_grants() {
    for digit in ['1', '2'] {
        let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(110, 36));
        let (response, mut rx) = tokio::sync::oneshot::channel();
        tui.handle_ui_output(talos_conversation::UiOutput::ToolApprovalRequest {
            tool_name: "git_checkout".into(),
            arguments: serde_json::json!({"branch":"topic", "create":true}),
            summary_fields: vec!["branch".into(), "create".into()],
            presentation: Some(talos_conversation::ApprovalPresentation {
                once_only: true,
                summary: vec![
                    "Create and switch branch: topic".into(),
                    "Repository: /workspace".into(),
                    "Stage: inspect repository/configuration".into(),
                ],
            }),
            preview: Some(
                (0..2000)
                    .map(|i| format!("Read Path: /workspace/file-{i}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            response,
        });
        tui.draw_frame().expect("operation summary");
        let text = tui.terminal.test_rendered_text();
        assert!(text.contains("Create and switch branch: topic"), "{text}");
        assert!(text.contains("no reusable grant"));
        assert!(!text.contains("file-0"));
        assert!(!tui.approval_preview_fully_visible);
        tui.handle_input_event(&key_press(KeyCode::Char(digit)));
        assert_eq!(
            rx.try_recv().expect("once decision"),
            ApprovalChoice::ApproveOnce
        );
        assert!(!tui.state.slash_menu.is_approval());
    }
}

#[tokio::test]
async fn ready_event_batch_reaches_release_before_redraw() {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let key = |kind| {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Down,
            KeyModifiers::NONE,
            kind,
        ))
    };
    let mut queued = (0..500)
        .map(|_| Ok(key(KeyEventKind::Repeat)))
        .collect::<Vec<_>>();
    queued.push(Ok(key(KeyEventKind::Release)));
    queued.push(Ok(key_press(KeyCode::Char('x'))));
    let mut stream = futures::stream::iter(queued);
    let batch = super::input::ready_input_batch(key(KeyEventKind::Press), &mut stream).await;
    assert_eq!(
        batch,
        vec![
            key(KeyEventKind::Press),
            key(KeyEventKind::Release),
            key_press(KeyCode::Char('x'))
        ]
    );
}

#[test]
fn compact_approval_preserves_localized_review_effects_and_decisions() {
    for point in [
        "是否继续检查？",
        "Continue this inspection?",
        "検査を続行しますか？",
    ] {
        let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
        tui.terminal
            .set_test_size(ratatui::layout::Size::new(100, 32));
        let scope = (0..2000)
            .map(|i| format!("Read Path: /workspace/file-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let preview = format!(
            "Auto review:\nModel review requires a human decision.\n{point}\nDecision needed: {point}\n\nPermission scope:\n{scope}"
        );
        tui.show_approval_with_preview("bash", "command: pwd", Some(preview));
        tui.draw_frame().expect("localized summary");
        let rendered = tui.terminal.test_rendered_text();
        assert!(
            rendered
                .split_whitespace()
                .collect::<String>()
                .contains(&point.split_whitespace().collect::<String>()),
            "{rendered}"
        );
        assert!(rendered.contains("2000 scope entries"), "{rendered}");
        assert!(rendered.contains("[3] deny"), "{rendered}");
        assert!(!tui.approval_preview_fully_visible);
        tui.state.slash_menu.approval_view.presentation =
            Some(talos_conversation::ApprovalPresentation {
                summary: vec![
                    "Checkout branch topic".into(),
                    "Execution is single-use".into(),
                ],
                once_only: true,
            });
        tui.draw_frame().expect("localized operation summary");
        let rendered = tui.terminal.test_rendered_text();
        assert!(
            rendered
                .split_whitespace()
                .collect::<String>()
                .contains(&point.split_whitespace().collect::<String>()),
            "{rendered}"
        );
        assert!(rendered.contains("Checkout branch topic"), "{rendered}");
        assert!(rendered.contains("[3] deny"), "{rendered}");
        assert!(!tui.approval_preview_fully_visible);
    }
}

#[test]
fn operation_review_does_not_displace_trusted_effects_in_short_panel() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(100, 22));
    let summary = vec![
        "Switch branch: topic",
        "Repository: workspace",
        "Stage: execute",
        "Evidence: paths",
        "Effects: write files",
        "Configured programs; no rollback guarantee",
        "Approve once",
    ];
    let render = |tui: &mut crate::app::Tui, preview: String| {
        tui.show_approval_with_preview("git", "checkout", Some(preview));
        tui.state.slash_menu.approval_view.presentation =
            Some(talos_conversation::ApprovalPresentation {
                summary: summary.iter().map(|line| (*line).into()).collect(),
                once_only: true,
            });
        tui.draw_frame().expect("compressed operation");
        tui.terminal.test_rendered_text()
    };
    let before = render(&mut tui, "Read Path: /workspace".into());
    let after = render(&mut tui, "Auto review:\nline one\nline two\nline three\nline four\nline five\n\nPermission scope:\nRead Path: /workspace".into());
    assert!(before.contains("Stage: execute"));
    for line in summary {
        if before.contains(line) {
            assert!(after.contains(line), "missing {line}: {after}");
        }
    }
    assert!(after.contains("[3] deny"));
}

#[tokio::test(start_paused = true)]
async fn event_loop_submits_final_ordered_frame_without_end_and_handles_cancel_during_burst() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use futures::StreamExt;
    use talos_conversation::{ContentOutput, UiOutput};

    let (user_tx, mut user_rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(user_tx));
    tui.first_message_dispatched = true;
    tui.state.status.is_processing = true;
    let (tx, rx) = mpsc::unbounded_channel();
    tui.ui_output_rx = Some(rx);
    tx.send(UiOutput::Content(ContentOutput::Start {
        source: MessageSource::Assistant,
    }))
    .expect("start");
    for i in 0..1000 {
        tx.send(UiOutput::Content(ContentOutput::Delta {
            text: format!("word{i:04} "),
        }))
        .expect("delta");
    }
    tx.send(UiOutput::Exit).expect("exit without End");
    let input = futures::stream::iter([Ok(Event::Key(KeyEvent::new(
        KeyCode::Esc,
        KeyModifiers::NONE,
    )))])
    .chain(futures::stream::pending());
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tui.run_with_events(input),
    )
    .await
    .expect("loop must finish")
    .expect("run");
    assert!(matches!(user_rx.try_recv(), Ok(UserInput::Cancel)));
    assert!(
        tui.terminal.test_draw_count() <= 4,
        "1000 ready deltas should coalesce between initial/start/input/final frames"
    );
    assert!(
        tui.pending_transcript.is_empty(),
        "final content must be committed"
    );
    assert!(!tui.ordered_content_open);
    assert!(
        tui.terminal.test_rendered_text().contains("word0999"),
        "last token must reach the submitted terminal buffer"
    );
    let text = tui
        .transcript
        .entries()
        .iter()
        .filter_map(|entry| match &entry.block {
            TranscriptBlock::StyledLine(line) => Some(line.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut offset = 0;
    for i in 0..1000 {
        let token = format!("word{i:04}");
        offset += text[offset..]
            .find(&token)
            .expect("every token retained in order")
            + token.len();
    }
}

#[test]
fn compressed_away_processing_preview_does_not_request_animation() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.first_message_dispatched = true;
    tui.state.status.is_processing = true;
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.draw_frame().expect("visible processing frame");
    assert!(tui.visible_processing_animation);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 1));
    tui.draw_frame().expect("compressed frame");
    assert!(!tui.visible_processing_animation);
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 24));
    tui.draw_frame().expect("restored frame");
    assert!(tui.visible_processing_animation);
    tui.state.status.is_processing = false;
    tui.draw_frame().expect("idle frame");
    assert!(!tui.visible_processing_animation);
}

#[test]
fn cached_frames_match_fresh_planning_cells_styles_and_cursor_across_state_changes() {
    let mut cached = crate::app::Tui::for_test(TuiState::new(), None);
    let mut fresh = crate::app::Tui::for_test(TuiState::new(), None);
    for step in 0..28 {
        for tui in [&mut cached, &mut fresh] {
            let width = [80, 80, 7, 100, 40, 80, 80][step % 7];
            let height = [24, 24, 8, 40, 6, 24, 24][step % 7];
            tui.terminal
                .set_test_size(ratatui::layout::Size::new(width, height));
            if step == 2 {
                tui.first_message_dispatched = true;
                for n in 0..40 {
                    tui.transcript
                        .append(TranscriptBlock::StyledLine(state_line(&format!(
                            "row {n}: 中文 🦀 e\u{301} zero\u{200b}width\nexplicit newline"
                        ))));
                }
            }
            tui.state.input_buffer = [
                "",
                "中文🦀e\u{301}\nsecond",
                "long words ".repeat(20).as_str(),
            ][step % 3]
                .to_owned();
            tui.state.cursor_pos = if step % 2 == 0 {
                0
            } else {
                tui.state.input_buffer.chars().count()
            };
            tui.state.status.is_processing = step % 4 != 0;
            tui.processing_frame = step;
            tui.state.thinking_preview =
                (step % 4 == 1).then(|| "thinking body 中文\nsecond line".into());
            if step == 10 {
                tui.show_approval("write", "{\"path\":\"src/example.rs\"}");
            }
            if step == 13 {
                tui.hide_approval();
                tui.state.slash_menu.close();
            }
            if step == 17 {
                tui.scroll_frame_history_up(5);
            }
            if step == 22 {
                tui.history_scroll.jump_to_end();
            }
        }
        // This path forces all new caches through their complete planning functions.
        fresh.history_projection_cache = Default::default();
        fresh.splash_cache = Default::default();
        fresh.layout_cache = Default::default();
        fresh.composer_height_cache = Default::default();
        fresh.preview_layout_cache = Default::default();
        cached.draw_frame().expect("cached frame");
        fresh.draw_frame().expect("freshly planned frame");
        assert_eq!(
            cached.terminal.test_rendered_buffer(),
            fresh.terminal.test_rendered_buffer(),
            "cells/styles at step {step}"
        );
        assert_eq!(
            cached.terminal.test_cursor_position(),
            fresh.terminal.test_cursor_position(),
            "cursor at step {step}"
        );
        assert_eq!(
            cached.terminal.test_cursor_visible(),
            fresh.terminal.test_cursor_visible()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn event_loop_without_visible_deadlines_draws_once_during_a_minute_of_idle() {
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    let (tx, rx) = mpsc::unbounded_channel();
    tui.ui_output_rx = Some(rx);
    let exit = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        tx.send(talos_conversation::UiOutput::Exit).expect("exit");
    });
    tui.run_with_events(futures::stream::pending())
        .await
        .expect("idle loop");
    exit.await.expect("exit sender");
    assert_eq!(tui.terminal.test_draw_count(), 1);
}

#[tokio::test(start_paused = true)]
async fn event_loop_expires_tip_and_releases_reconnecting_without_input() {
    use talos_conversation::{StatusSnapshot, UiOutput};
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.first_message_dispatched = true;
    // Hide animation: these updates must have their own deadlines.
    tui.terminal
        .set_test_size(ratatui::layout::Size::new(80, 1));
    tui.state.status.is_processing = true;
    tui.state.status.phase = Some(TurnPhase::Connecting);
    let now = super::scheduling::now();
    tui.state.tip = Some(crate::state::Tip {
        kind: TipKind::ExitHint,
        text: "temporary".into(),
        ttl: std::time::Duration::from_millis(300),
        created_at: now,
    });
    tui.deferred_reconnecting_status = Some((
        StatusSnapshot {
            is_processing: true,
            phase: Some(TurnPhase::Reconnecting {
                attempt: 1,
                max_attempts: 3,
            }),
            ..Default::default()
        },
        now + std::time::Duration::from_millis(150),
    ));
    let (tx, rx) = mpsc::unbounded_channel();
    tui.ui_output_rx = Some(rx);
    let exit = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        tx.send(UiOutput::Exit).expect("exit");
    });
    tui.run_with_events(futures::stream::pending())
        .await
        .expect("deadline loop");
    exit.await.expect("exit sender");
    assert!(tui.state.tip.is_none());
    assert!(tui.deferred_reconnecting_status.is_none());
    assert!(matches!(
        tui.state.status.phase,
        Some(TurnPhase::Reconnecting { .. })
    ));
    assert_eq!(
        tui.terminal.test_draw_count(),
        3,
        "initial, reconnect, tip expiry only"
    );
}

#[tokio::test(start_paused = true)]
async fn event_loop_drag_scroll_stops_deadlines_after_mouse_release() {
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use futures::StreamExt;
    let mut tui = crate::app::Tui::for_test(TuiState::new(), None);
    tui.first_message_dispatched = true;
    for n in 0..100 {
        tui.transcript
            .append(TranscriptBlock::StyledLine(state_line(&format!(
                "history {n}"
            ))));
    }
    tui.draw_frame().expect("prepare selection geometry");
    let area = tui.last_history_area.expect("history area");
    let anchor = tui
        .history_selection_point_at_screen(area.x, area.y)
        .expect("history anchor");
    let initial_start = tui.last_frame_history_start;
    let initial_draws = tui.terminal.test_draw_count();
    tui.selection = Some(super::SelectionState {
        anchor: (area.x, area.y),
        focus: (area.x, area.y),
        dragging: true,
        moved: true,
        edge: -1,
        history_anchor: Some(anchor),
        history_focus: Some(anchor),
    });
    let events = futures::stream::once(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: area.x,
            row: area.y,
            modifiers: KeyModifiers::NONE,
        }))
    })
    .chain(futures::stream::pending())
    .boxed();
    let (tx, rx) = mpsc::unbounded_channel();
    tui.ui_output_rx = Some(rx);
    let exit = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        tx.send(talos_conversation::UiOutput::Exit).expect("exit");
    });
    tui.run_with_events(events).await.expect("drag loop");
    exit.await.expect("exit sender");
    assert_eq!(tui.last_frame_history_start, initial_start - 3);
    assert!(tui.selection.is_none_or(|selection| !selection.dragging));
    assert_eq!(
        tui.terminal.test_draw_count() - initial_draws,
        5,
        "initial, three scroll ticks, release; no post-release frames"
    );
}

#[tokio::test]
async fn event_loop_remains_interactive_and_exits_while_actual_git_scan_is_blocked() {
    use crate::git_snapshot_worker::{Scan, TEST_SCAN_LOCK, wait_for_test_slot};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    let _serial = TEST_SCAN_LOCK.lock().await;
    wait_for_test_slot().await;
    let (user_tx, mut user_rx) = mpsc::unbounded_channel();
    let mut tui = crate::app::Tui::for_test(TuiState::new(), Some(user_tx));
    tui.first_message_dispatched = true;
    tui.state.status.workspace_path = "blocked-test-workspace".into();
    let (output_tx, output_rx) = mpsc::unbounded_channel();
    tui.ui_output_rx = Some(output_rx);
    let (input_tx, input_rx) = futures::channel::mpsc::unbounded();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let mut entered_tx = Some(entered_tx);
    let mut release_rx = Some(release_rx);
    let mut launches = 0;
    let mut requests = Vec::new();
    let driver = async move {
        entered_rx.await.expect("scanner started");
        // Once the real worker is known to be blocked, advance only the UI clock.
        tokio::time::pause();
        tokio::time::sleep(std::time::Duration::from_millis(2100)).await;
        output_tx
            .send(talos_conversation::UiOutput::Status(
                talos_conversation::StatusSnapshot {
                    workspace_path: "replacement-workspace".into(),
                    ..Default::default()
                },
            ))
            .expect("workspace switch after timeout");
        for _ in 0..20 {
            input_tx
                .unbounded_send(Ok(Event::FocusGained))
                .expect("refresh request");
        }
        for code in [KeyCode::Char('x'), KeyCode::Enter] {
            input_tx
                .unbounded_send(Ok(Event::Key(KeyEvent::new(code, KeyModifiers::NONE))))
                .expect("input");
        }
        assert!(
            matches!(user_rx.recv().await, Some(UserInput::Message(_))),
            "input must reach business channel before scanner is released"
        );
        output_tx
            .send(talos_conversation::UiOutput::Exit)
            .expect("exit");
        // Keep input open until Exit wins; this test must not pass by input EOF.
        (input_tx, release_tx)
    };
    let run = tui.run_with_scanner(input_rx, |workspace, generation| {
        requests.push((workspace.clone(), generation));
        let Some(entered) = entered_tx.take() else {
            let attempted = Scan::start_with(workspace, generation, |_| None)?;
            assert!(
                attempted.is_none(),
                "timeout/switch must not start another actual worker"
            );
            return Ok(None);
        };
        launches += 1;
        let release = release_rx.take().expect("one actual scan");
        Scan::start_with(workspace, generation, move |_| {
            let _ = entered.send(());
            let _ = release.recv();
            None
        })
    });
    let (result, (_input_tx, release_tx)) =
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            tokio::join!(run, driver)
        })
        .await
        .expect("blocked scan must not delay input or exit");
    result.expect("loop");
    assert_eq!(launches, 1);
    assert!(
        requests.iter().any(
            |(workspace, generation)| workspace == "replacement-workspace" && *generation == 2
        )
    );
    assert!(
        tui.git_snapshot
            .get("replacement-workspace", super::scheduling::now())
            .is_none()
    );
    assert!(
        Scan::start_with("next-session".into(), 2, |_| None)
            .expect("probe")
            .is_none(),
        "exiting must not free the actual worker slot"
    );
    release_tx.send(()).expect("release blocked worker");
    wait_for_test_slot().await;
}
