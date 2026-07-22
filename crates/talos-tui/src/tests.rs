#![allow(warnings)]
#[cfg(test)]
#[allow(warnings)]
mod tests {
    use std::time::{Duration, Instant};
    use talos_conversation::{
        MessageSource, SessionPickerItem, StatusSnapshot, TipKind, TurnPhase,
    };
    use talos_core::ApprovalChoice;
    use talos_core::message::Usage;
    use unicode_width::UnicodeWidthStr;

    use crate::inline_terminal::{InlineFrame, ViewportComponent};
    use crate::panel_state::PanelItem;
    use crate::scrollback::{
        BottomPanelComponent, BottomPanelPlacement, approval_natural_height,
        bottom_panel_placement, bottom_panel_rows, build_input_text, build_status_text,
        cursor_line_col, extract_thinking_title, input_line_count, stream_padding_for,
        truncate_str, wrap_text_to_lines,
    };
    use crate::sidebar::{SkillInfo, SkillSidebar};
    use crate::state::{ApprovalState, BottomPanelState, CtrlCState, Tip, TuiState};
    use crate::{contrast_ratio, rgb_components};

    // ── TuiState (pure UI) ─────────────────────────────────────────────

    #[test]
    fn test_state_new() {
        let state = TuiState::new();
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
        assert!(!state.should_exit);
    }

    #[test]
    fn test_input_append_char() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_append_char('c');
        assert_eq!(state.input_buffer, "abc");
        assert_eq!(state.cursor_pos, 3);
    }

    #[test]
    fn test_input_append_char_at_position() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('c');
        state.input_cursor_left();
        state.input_append_char('b');
        assert_eq!(state.input_buffer, "abc");
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_input_backspace() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_backspace();
        assert_eq!(state.input_buffer, "a");
        assert_eq!(state.cursor_pos, 1);
    }

    #[test]
    fn test_input_backspace_at_start() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_cursor_left();
        state.input_backspace();
        assert_eq!(state.input_buffer, "a");
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_movement() {
        let mut state = TuiState::new();
        state.input_append_char('a');
        state.input_append_char('b');
        state.input_append_char('c');

        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 2);

        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 1);

        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 2);
    }

    #[test]
    fn test_input_cursor_bounds() {
        let mut state = TuiState::new();
        state.input_append_char('a');

        state.input_cursor_left();
        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 0);

        state.input_cursor_right();
        state.input_cursor_right();
        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 1);
    }

    #[test]
    fn test_input_cursor_to_line_start_single_line() {
        let mut state = TuiState::new();
        state.input_append_str("hello world");
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_to_line_start_mid_line() {
        let mut state = TuiState::new();
        state.input_append_str("hello world");
        state.input_cursor_left();
        state.input_cursor_left();
        assert_eq!(state.cursor_pos, 9);
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_to_line_start_multiline() {
        let mut state = TuiState::new();
        state.input_append_str("first\nsecond line");
        state.cursor_pos = state.input_buffer.chars().count();
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 6);
    }

    #[test]
    fn test_input_cursor_to_line_start_already_at_start() {
        let mut state = TuiState::new();
        state.input_append_str("hello");
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_to_line_end_single_line() {
        let mut state = TuiState::new();
        state.input_append_str("hello world");
        state.input_cursor_to_line_start();
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 11);
    }

    #[test]
    fn test_input_cursor_to_line_end_mid_line() {
        let mut state = TuiState::new();
        state.input_append_str("hello world");
        state.input_cursor_to_line_start();
        state.input_cursor_right();
        state.input_cursor_right();
        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 3);
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 11);
    }

    #[test]
    fn test_input_cursor_to_line_end_multiline() {
        let mut state = TuiState::new();
        state.input_append_str("first\nsecond");
        let total = state.input_buffer.chars().count();
        state.input_cursor_to_line_start();
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, total);
    }

    #[test]
    fn test_input_cursor_to_line_end_from_second_line() {
        let mut state = TuiState::new();
        state.input_append_str("first\nsecond\nthird");
        state.cursor_pos = 12;
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 6);
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 12);
    }

    #[test]
    fn test_input_cursor_to_line_end_already_at_end() {
        let mut state = TuiState::new();
        state.input_append_str("hello");
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 5);
    }

    #[test]
    fn test_input_cursor_to_line_start_empty() {
        let mut state = TuiState::new();
        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_to_line_end_empty() {
        let mut state = TuiState::new();
        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_cursor_to_line_boundaries_idempotent() {
        let mut state = TuiState::new();
        state.input_append_str("line one\nline two\nline three");
        state.cursor_pos = 0;

        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 0);

        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 8);

        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 9);

        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 9);

        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 17);

        state.input_cursor_right();
        assert_eq!(state.cursor_pos, 18);

        state.input_cursor_to_line_start();
        assert_eq!(state.cursor_pos, 18);

        state.input_cursor_to_line_end();
        assert_eq!(state.cursor_pos, 28);
    }

    #[test]
    fn test_input_clear() {
        let mut state = TuiState::new();
        state.input_append_char('h');
        state.input_append_char('i');
        state.input_clear();
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
    }

    #[test]
    fn test_input_submit() {
        let mut state = TuiState::new();
        state.input_append_char('h');
        state.input_append_char('i');
        let result = state.input_submit();
        assert_eq!(result, "hi");
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
    }

    // ── Ctrl+C ─────────────────────────────────────────────────────────

    #[test]
    fn test_ctrl_c_single_press_idle() {
        let mut state = TuiState::new();
        let should_exit = state.handle_ctrl_c();
        assert!(!should_exit);
        assert!(matches!(state.ctrl_c_state, CtrlCState::Waiting(_)));
    }

    #[test]
    fn test_ctrl_c_double_press_exits() {
        let mut state = TuiState::new();
        let should_exit = state.handle_ctrl_c();
        assert!(!should_exit);

        let should_exit = state.handle_ctrl_c();
        assert!(should_exit);
        assert!(state.should_exit);
    }

    #[test]
    fn test_ctrl_c_reset_on_char() {
        let mut state = TuiState::new();
        state.handle_ctrl_c();
        assert!(matches!(state.ctrl_c_state, CtrlCState::Waiting(_)));

        state.ctrl_c_state = CtrlCState::Idle;
        assert!(matches!(state.ctrl_c_state, CtrlCState::Idle));
    }

    // ── Approval ────────────────────────────────────────────────────────

    #[test]
    fn test_approval_state_default_hidden() {
        assert!(matches!(ApprovalState::default(), ApprovalState::Hidden));
    }

    #[test]
    fn test_approval_state_transitions() {
        let state = ApprovalState::Visible {
            tool_name: "bash".to_string(),
            arguments: "{}".to_string(),
            selected: ApprovalChoice::ApproveOnce,
        };
        assert!(matches!(state, ApprovalState::Visible { .. }));
    }

    // ── Tip ─────────────────────────────────────────────────────────────

    #[test]
    fn test_tip_auto_expires() {
        let mut state = TuiState::new();
        state.tip = Some(Tip {
            kind: TipKind::ExitHint,
            text: "test".to_string(),
            ttl: Duration::from_millis(1),
            created_at: Instant::now() - Duration::from_secs(1),
        });
        state.expire_tip();
        assert!(state.tip.is_none());
    }

    #[test]
    fn test_tip_does_not_expire_before_ttl() {
        let mut state = TuiState::new();
        state.tip = Some(Tip {
            kind: TipKind::ExitHint,
            text: "test".to_string(),
            ttl: Duration::from_secs(10),
            created_at: Instant::now(),
        });
        state.expire_tip();
        assert!(state.tip.is_some());
    }

    // ── Theme ───────────────────────────────────────────────────────────

    #[test]
    fn test_nord_palette_defines_all_colors() {
        use crate::theme::nord::*;
        assert!(rgb_components(NORD0).is_some());
        assert!(rgb_components(NORD4).is_some());
        assert!(rgb_components(NORD11).is_some());
        assert!(rgb_components(NORD14).is_some());
    }

    #[test]
    fn test_nord_primary_text_contrast_is_wcag_aa() {
        use crate::theme::nord::*;
        let cr = contrast_ratio(NORD4, NORD0).unwrap();
        assert!(
            cr >= 4.5,
            "NORD4 against NORD0 has contrast ratio {} (need >= 4.5)",
            cr
        );
    }

    // ── Skill Sidebar ──────────────────────────────────────────────────

    #[test]
    fn test_skill_sidebar_new_is_hidden() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
    }

    #[test]
    fn test_skill_sidebar_default_is_hidden() {
        let sidebar = SkillSidebar::default();
        assert!(!sidebar.visible);
    }

    #[test]
    fn test_skill_sidebar_toggle_visibility() {
        let mut sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
        sidebar.toggle();
        assert!(sidebar.visible);
        sidebar.toggle();
        assert!(!sidebar.visible);
    }

    #[test]
    fn test_skill_info_fields() {
        let info = SkillInfo {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            active: true,
        };
        assert_eq!(info.name, "test-skill");
        assert_eq!(info.description, "A test skill");
        assert!(info.active);
    }

    // ── Status text ────────────────────────────────────────────────────

    #[test]
    fn test_queue_indicator_absent_when_no_queues() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(!content.contains("S:"));
        assert!(!content.contains("F:"));
        assert!(!content.contains("queued"));
    }

    #[test]
    fn test_queue_indicator_in_status_when_steering_queued() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 3,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("3 queued"));
    }

    #[test]
    fn test_multiline_input_uses_prompt_only_on_first_line() {
        let mut state = TuiState::new();
        state.input_append_str("first\nsecond");

        let text = build_input_text(&state, 77);

        assert_eq!(text.lines.len(), 2);
        assert_eq!(text.lines[0].to_string(), " > first");
        assert_eq!(text.lines[1].to_string(), "   second");
        assert_eq!(input_line_count(&state.input_buffer), 2);
    }

    #[test]
    fn test_cursor_line_col_tracks_multiline_buffer() {
        assert_eq!(cursor_line_col("abc"), (0, 3));
        assert_eq!(cursor_line_col("abc\nde"), (1, 2));
    }

    #[test]
    fn test_stream_padding_only_marks_first_line() {
        assert_eq!(stream_padding_for(Some(&MessageSource::User), 0), " > ");
        assert_eq!(stream_padding_for(Some(&MessageSource::User), 1), "   ");
        assert_eq!(
            stream_padding_for(Some(&MessageSource::Assistant), 0),
            " ● "
        );
        assert_eq!(
            stream_padding_for(Some(&MessageSource::Assistant), 1),
            "   "
        );
        assert_eq!(stream_padding_for(Some(&MessageSource::System), 0), " # ");
        assert_eq!(stream_padding_for(Some(&MessageSource::System), 1), "   ");
        assert_eq!(stream_padding_for(Some(&MessageSource::Error), 0), " ! ");
        assert_eq!(stream_padding_for(Some(&MessageSource::Error), 1), "   ");
        assert_eq!(
            stream_padding_for(
                Some(&MessageSource::Tool {
                    name: "bash".to_string()
                }),
                0
            ),
            " ● "
        );
        assert_eq!(
            stream_padding_for(
                Some(&MessageSource::Tool {
                    name: "bash".to_string()
                }),
                1
            ),
            "   "
        );
    }

    // ── Slash Menu ─────────────────────────────────────────────────────

    #[test]
    fn test_slash_menu_default_is_closed() {
        let menu = BottomPanelState::default();
        assert!(!menu.is_open);
        assert!(menu.items.is_empty());
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_slash_menu_opens_with_commands() {
        let registry = talos_conversation::command_registry();
        let menu = BottomPanelState::open_slash(registry);
        assert!(menu.is_open);
        assert!(!menu.items.is_empty());
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_slash_menu_filters_by_name() {
        let registry = talos_conversation::command_registry();
        let mut menu = BottomPanelState::open_slash(registry);
        let filtered = menu.filtered_items("help");
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|item| item.label == "/help"));
    }

    #[test]
    fn test_slash_menu_filters_by_description() {
        let registry = talos_conversation::command_registry();
        let mut menu = BottomPanelState::open_slash(registry);
        let filtered = menu.filtered_items("exit");
        assert!(!filtered.is_empty());
        assert!(
            filtered
                .iter()
                .any(|item| item.label == "/quit" || item.label == "/exit")
        );
    }

    #[test]
    fn test_slash_menu_selection_wraps() {
        let registry = talos_conversation::command_registry();
        let mut menu = BottomPanelState::open_slash(registry);
        let len = menu.filtered_items("").len();
        assert!(len > 0);

        menu.select_prev("");
        assert_eq!(menu.selected_index, len - 1);

        menu.select_next("");
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_slash_menu_selected_command_returns_name() {
        let registry = talos_conversation::command_registry();
        let menu = BottomPanelState::open_slash(registry);
        let item = &menu.items[menu.selected_index];
        match &item.action {
            crate::state::PanelItemAction::SlashCommand { command, .. } => {
                assert!(command.starts_with('/'));
            }
            _ => panic!("expected SlashCommand action"),
        }
    }

    #[test]
    fn test_slash_menu_close_clears_state() {
        let registry = talos_conversation::command_registry();
        let mut menu = BottomPanelState::open_slash(registry);
        menu.select_next("");
        menu.close();
        assert!(!menu.is_open);
        assert!(menu.items.is_empty());
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn test_slash_menu_no_match_returns_empty() {
        let registry = talos_conversation::command_registry();
        let menu = BottomPanelState::open_slash(registry);
        let filtered = menu.filtered_items("zzzznonexistent");
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_slash_menu_item_with_arg_hint() {
        let item = PanelItem {
            label: "/export".to_string(),
            description: "Export transcript [path]".to_string(),
            action: crate::state::PanelItemAction::SlashCommand {
                command: "/export".to_string(),
                arg_hint: Some("<path>".to_string()),
                execution_mode: talos_conversation::CommandExecutionMode::RequireInput,
            },
            is_current: false,
        };
        assert_eq!(item.label, "/export");
        assert!(item.description.contains('['));
    }

    #[test]
    fn test_session_picker_accept_emits_correct_command() {
        let items = vec![
            SessionPickerItem {
                command: "/resume".to_string(),
                ordinal: 1,
                timestamp: "2026-06-22 19:20".to_string(),
                message_count: 5,
                preview: "hello".to_string(),
            },
            SessionPickerItem {
                command: "/delete".to_string(),
                ordinal: 1,
                timestamp: "2026-06-22 19:00".to_string(),
                message_count: 3,
                preview: "older".to_string(),
            },
        ];
        let panel = BottomPanelState::open_session_picker(&items);
        assert!(panel.is_picker());
        let mut state = TuiState::new();
        state.slash_menu = panel;

        state.slash_menu.selected_index = 1;
        let action = state.accept_selected_panel_item();
        match action {
            crate::state::PanelAction::SendMessage(msg) => {
                assert_eq!(msg, "/delete 1", "picker must echo item's command");
            }
            other => panic!("expected SendMessage, got {other:?}"),
        }
        assert!(!state.slash_menu.is_open, "picker closes on accept");
    }

    #[test]
    fn test_session_picker_accept_resume_default_command() {
        // When an item's command is empty (legacy callers), fall back to /resume.
        let items = vec![SessionPickerItem {
            command: String::new(),
            ordinal: 1,
            timestamp: "2026-06-22 19:20".to_string(),
            message_count: 5,
            preview: "hello".to_string(),
        }];
        let panel = BottomPanelState::open_session_picker(&items);
        let mut state = TuiState::new();
        state.slash_menu = panel;
        let action = state.accept_selected_panel_item();
        match action {
            crate::state::PanelAction::SendMessage(msg) => {
                assert_eq!(msg, "/resume 1");
            }
            other => panic!("expected SendMessage, got {other:?}"),
        }
    }

    #[test]
    fn test_slash_menu_query_is_visible_and_backspace_edits_it() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        state.append_slash_query_char('h');
        state.append_slash_query_char('e');
        assert_eq!(state.input_buffer, "/he");
        assert_eq!(state.slash_query(), "he");

        state.backspace_slash_query();
        assert_eq!(state.input_buffer, "/h");
        assert!(state.slash_menu.is_open);
        state.backspace_slash_query();
        state.backspace_slash_query();
        assert!(state.input_buffer.is_empty());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_accept_inserts_command_and_closes() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "export".chars() {
            state.append_slash_query_char(ch);
        }
        state.accept_selected_panel_item();
        assert_eq!(state.input_buffer, "/export ");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_enter_executes_parameterless_command() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "help".chars() {
            state.append_slash_query_char(ch);
        }

        let action = state.accept_selected_panel_item();

        assert_eq!(
            action,
            crate::state::PanelAction::SendMessage("/help".to_string())
        );
        assert!(state.input_buffer.is_empty());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_enter_uses_first_command_prefix_match() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        state.append_slash_query_char('m');
        state.append_slash_query_char('o');

        let visible: Vec<&str> = state
            .slash_menu
            .filtered_items(state.panel_query())
            .into_iter()
            .map(|item| item.label.as_str())
            .collect();
        assert_eq!(visible.first(), Some(&"/model"));

        // TUI-033: /model is now DirectExecution (no arg_hint) — Enter opens
        // the picker directly by sending the bare command, not by filling the
        // composer with "/model " (trailing parameter space).
        let action = state.accept_selected_panel_item();
        assert_eq!(
            action,
            crate::state::PanelAction::SendMessage("/model".to_string())
        );
        assert!(state.input_buffer.is_empty());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_pasted_prefix_selects_first_match_before_enter() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        state.input_paste("mo");

        let visible: Vec<&str> = state
            .slash_menu
            .filtered_items(state.panel_query())
            .into_iter()
            .map(|item| item.label.as_str())
            .collect();
        assert_eq!(visible.first(), Some(&"/model"));

        // TUI-033: /model is DirectExecution — Enter sends bare "/model".
        let action = state.accept_selected_panel_item();
        assert_eq!(
            action,
            crate::state::PanelAction::SendMessage("/model".to_string())
        );
        assert!(state.input_buffer.is_empty());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn parameterized_model_command_leaves_slash_menu_for_bridge_correction() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "model gpt-4o".chars() {
            state.append_slash_query_char(ch);
        }

        assert_eq!(state.input_buffer, "/model gpt-4o");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn pasted_parameterized_connect_command_leaves_slash_menu_for_bridge_correction() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        state.input_paste("connect openai");

        assert_eq!(state.input_buffer, "/connect openai");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_uses_registry_execution_mode() {
        let registry = talos_conversation::command_registry();
        let menu = BottomPanelState::open_slash(registry);

        let help = menu
            .items
            .iter()
            .find(|item| item.label == "/help")
            .expect("/help item");
        let export = menu
            .items
            .iter()
            .find(|item| item.label == "/export")
            .expect("/export item");
        let model = menu
            .items
            .iter()
            .find(|item| item.label == "/model")
            .expect("/model item");
        let connect = menu
            .items
            .iter()
            .find(|item| item.label == "/connect")
            .expect("/connect item");

        for (label, item) in [("/help", help), ("/model", model), ("/connect", connect)] {
            match &item.action {
                crate::state::PanelItemAction::SlashCommand {
                    execution_mode,
                    arg_hint,
                    ..
                } => {
                    assert_eq!(
                        *execution_mode,
                        talos_conversation::CommandExecutionMode::DirectExecution,
                        "{label} should be DirectExecution (TUI-033)"
                    );
                    assert!(
                        arg_hint.is_none(),
                        "{label} should have no arg_hint (TUI-033)"
                    );
                }
                other => panic!("{label}: expected SlashCommand action, got {other:?}"),
            }
        }
        match &export.action {
            crate::state::PanelItemAction::SlashCommand {
                execution_mode,
                arg_hint,
                ..
            } => {
                assert_eq!(
                    *execution_mode,
                    talos_conversation::CommandExecutionMode::RequireInput
                );
                assert_eq!(arg_hint.as_deref(), Some("<path>"));
            }
            other => panic!("expected SlashCommand action, got {other:?}"),
        }
    }

    #[test]
    fn test_slash_menu_enter_executes_connect_directly() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "connect".chars() {
            state.append_slash_query_char(ch);
        }

        let action = state.accept_selected_panel_item();
        assert_eq!(
            action,
            crate::state::PanelAction::SendMessage("/connect".to_string())
        );
        assert!(state.input_buffer.is_empty());
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_tab_completes_model_without_trailing_space() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "model".chars() {
            state.append_slash_query_char(ch);
        }

        let action = state.complete_selected_panel_item();
        assert_eq!(action, crate::state::PanelAction::None);
        assert_eq!(state.input_buffer, "/model");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_slash_menu_tab_completes_connect_without_trailing_space() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "connect".chars() {
            state.append_slash_query_char(ch);
        }

        let action = state.complete_selected_panel_item();
        assert_eq!(action, crate::state::PanelAction::None);
        assert_eq!(state.input_buffer, "/connect");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_model_picker_switch_produces_structured_action() {
        let data = talos_conversation::ModelPickerData {
            recent: vec![],
            ready_models: vec![talos_conversation::ModelPickerItem {
                command: "/model".to_string(),
                model_id: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                label: "gpt-4o   OpenAI  128K".to_string(),
                context_limit: Some(128_000),
                pricing: None,
                authenticated: true,
                is_current: false,
                variants: vec![],
                variant: None,
            }],
            setup_providers: vec![],
        };
        // Level 2 (open_model_list) is where individual model rows live;
        // variant-less models produce SwitchModel, variant-bearing ones
        // produce OpenVariantPicker.
        let panel = BottomPanelState::open_model_list("openai", &data);
        let switch_item = panel
            .items
            .iter()
            .find(|i| matches!(i.action, crate::state::PanelItemAction::SwitchModel { .. }))
            .expect("Level 2 should have a SwitchModel item for a variant-less model");

        match &switch_item.action {
            crate::state::PanelItemAction::SwitchModel {
                provider,
                model_id,
                variant,
            } => {
                assert_eq!(provider, "openai");
                assert_eq!(model_id, "gpt-4o");
                assert!(variant.is_none());
            }
            other => panic!("expected SwitchModel, got {other:?}"),
        }
    }

    #[test]
    fn test_model_picker_switch_with_variant_produces_structured_action() {
        use talos_conversation::ModelPickerVariantItem;
        let data = talos_conversation::ModelPickerData {
            recent: vec![],
            ready_models: vec![talos_conversation::ModelPickerItem {
                command: "/model".to_string(),
                model_id: "o3".to_string(),
                provider: "openai".to_string(),
                label: "o3   OpenAI  200K".to_string(),
                context_limit: Some(200_000),
                pricing: None,
                authenticated: true,
                is_current: false,
                variants: vec![ModelPickerVariantItem {
                    variant_id: "high-reasoning".to_string(),
                    label: "High Reasoning".to_string(),
                    provider: "openai".to_string(),
                    model_id: "o3".to_string(),
                }],
                variant: None,
            }],
            setup_providers: vec![],
        };
        let panel = BottomPanelState::open_model_list("openai", &data);
        let variant_item = panel
            .items
            .iter()
            .find(|i| {
                matches!(
                    &i.action,
                    crate::state::PanelItemAction::OpenVariantPicker { .. }
                )
            })
            .expect("should have an OpenVariantPicker item");

        match &variant_item.action {
            crate::state::PanelItemAction::OpenVariantPicker {
                provider,
                model_id,
                variants,
            } => {
                assert_eq!(provider, "openai");
                assert_eq!(model_id, "o3");
                assert_eq!(variants.len(), 1);
                assert_eq!(variants[0].variant_id, "high-reasoning");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_connect_picker_select_produces_structured_action() {
        let data = talos_conversation::ConnectPickerData {
            connected: vec![],
            available: vec![talos_conversation::ConnectPickerItem {
                provider: "openai".to_string(),
                name: "OpenAI".to_string(),
                model_count: 42,
                api_base_url: Some("https://api.openai.com/v1".to_string()),
                has_credential: false,
                doc_url: None,
            }],
        };
        let panel = BottomPanelState::open_connect_picker(&data);
        let select_item = panel
            .items
            .iter()
            .find(|i| {
                matches!(
                    &i.action,
                    crate::state::PanelItemAction::ConnectSelect { .. }
                )
            })
            .expect("should have a ConnectSelect item");

        match &select_item.action {
            crate::state::PanelItemAction::ConnectSelect { provider } => {
                assert_eq!(provider, "openai");
            }
            other => panic!("expected ConnectSelect, got {other:?}"),
        }
    }

    #[test]
    fn test_connect_picker_does_not_produce_select_command_string() {
        let data = talos_conversation::ConnectPickerData {
            connected: vec![],
            available: vec![talos_conversation::ConnectPickerItem {
                provider: "anthropic".to_string(),
                name: "Anthropic".to_string(),
                model_count: 10,
                api_base_url: None,
                has_credential: false,
                doc_url: None,
            }],
        };
        let panel = BottomPanelState::open_connect_picker(&data);
        for item in &panel.items {
            if !matches!(item.action, crate::state::PanelItemAction::Header) {
                assert!(
                    !matches!(
                        &item.action,
                        crate::state::PanelItemAction::Select { command, .. }
                        if command == "/connect"
                    ),
                    "connect picker must not use Select with /connect command (TUI-033 structured identity)"
                );
            }
        }
    }

    #[test]
    fn test_slash_menu_tab_completes_parameterless_command_without_execution() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        for ch in "help".chars() {
            state.append_slash_query_char(ch);
        }

        let action = state.complete_selected_panel_item();

        assert_eq!(action, crate::state::PanelAction::None);
        assert_eq!(state.input_buffer, "/help");
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn test_approval_activation_closes_slash_menu() {
        let registry = talos_conversation::command_registry();
        let mut state = TuiState::new();
        state.open_slash_menu(registry);
        state.activate_approval("bash", "command: cargo test");
        assert!(!state.slash_menu.is_open);
        assert!(matches!(
            state.approval_state,
            ApprovalState::Visible { .. }
        ));
    }

    #[test]
    fn test_slash_menu_placement_prefers_below_and_falls_back_above() {
        assert_eq!(
            bottom_panel_placement(24, 8, 10),
            BottomPanelPlacement::BelowInput
        );
        assert_eq!(
            bottom_panel_placement(12, 8, 10),
            BottomPanelPlacement::AboveInput
        );
    }

    #[test]
    fn test_slash_menu_height_is_exact_and_capped() {
        let registry = talos_conversation::command_registry();
        let menu = BottomPanelState::open_slash(registry);
        let full = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        let capped = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: 3,
        };
        assert!(full.height_hint(80) > 3);
        assert_eq!(capped.height_hint(80), 3);
    }

    #[test]
    fn provider_wizard_renders_steps_instead_of_no_matches() {
        use ratatui::{buffer::Buffer, layout::Rect};

        let menu = BottomPanelState::open_provider_wizard();
        let component = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        assert_eq!(component.height_hint(80), 3);

        let area = Rect::new(0, 0, 80, 3);
        let mut buffer = Buffer::empty(area);
        let mut frame = InlineFrame::new(area, &mut buffer);
        component.render(&mut frame, area);
        let rendered: String = buffer
            .content()
            .iter()
            .flat_map(|cell| cell.symbol().chars())
            .collect();
        assert!(rendered.contains("Add custom provider"), "{rendered:?}");
        assert!(rendered.contains("Step 1/5: Name"), "{rendered:?}");
        assert!(!rendered.contains("No matches"), "{rendered:?}");
    }

    #[test]
    fn test_slash_menu_capped_rows_reserve_overflow_indicator() {
        assert_eq!(bottom_panel_rows(5, 3, 0), (1, true, true));
        assert_eq!(bottom_panel_rows(5, 6, 0), (5, true, false));
        assert_eq!(bottom_panel_rows(10, 10, 0), (8, true, true));
    }

    // ── Approval panel height_hint (F110) ──────────────────────────────

    #[test]
    fn test_approval_height_wide_returns_6() {
        let menu = BottomPanelState::open_approval("bash", "command: echo hello world");
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        assert_eq!(comp.height_hint(80), 6);
        assert_eq!(comp.height_hint(120), 6);
        assert_eq!(comp.height_hint(60), 6);
    }

    #[test]
    fn test_approval_height_narrow_returns_more_than_6() {
        let long_args = "command: cargo test --workspace --locked --all-features";
        let menu = BottomPanelState::open_approval("bash", long_args);
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        let h = comp.height_hint(40);
        assert!(
            h > 6,
            "narrow width with long args should need >6 rows, got {h}"
        );
    }

    #[test]
    fn test_approval_height_empty_args_returns_6() {
        let menu = BottomPanelState::open_approval("read", "");
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        assert_eq!(comp.height_hint(40), 6);
        assert_eq!(comp.height_hint(80), 6);
    }

    #[test]
    fn test_approval_height_capped_by_max_height() {
        let menu = BottomPanelState::open_approval("bash", "command: echo hello");
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: 3,
        };
        assert_eq!(comp.height_hint(80), 3);
        assert_eq!(comp.height_hint(40), 3);
    }

    // ── wrap_text_to_lines helper (F110) ───────────────────────────────

    #[test]
    fn test_wrap_text_fits_in_one_line() {
        let lines = wrap_text_to_lines("hello world", 20, 2);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello world");
    }

    #[test]
    fn test_wrap_text_truncates_with_ellipsis() {
        let lines = wrap_text_to_lines("abcdefghij", 4, 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "abcd");
        assert!(
            lines[1].ends_with('…'),
            "last line should end with ellipsis: {}",
            lines[1]
        );
    }

    #[test]
    fn test_wrap_text_no_truncation_when_exact_fit() {
        let lines = wrap_text_to_lines("abcdefgh", 4, 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "abcd");
        assert_eq!(lines[1], "efgh");
    }

    #[test]
    fn test_wrap_text_empty_input() {
        assert!(wrap_text_to_lines("", 10, 2).is_empty());
        assert!(wrap_text_to_lines("hello", 0, 2).is_empty());
        assert!(wrap_text_to_lines("hello", 10, 0).is_empty());
    }

    #[test]
    fn test_wrap_text_replaces_newlines() {
        let lines = wrap_text_to_lines("hello\nworld", 20, 2);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello world");
    }

    // ── approval_natural_height (F110) ─────────────────────────────────

    #[test]
    fn test_approval_natural_height_wide() {
        assert_eq!(approval_natural_height(80, "some args"), 6);
        assert_eq!(approval_natural_height(60, "some args"), 6);
    }

    #[test]
    fn test_approval_natural_height_narrow_empty_args() {
        assert_eq!(approval_natural_height(40, ""), 6);
        assert_eq!(approval_natural_height(40, "   "), 6);
    }

    #[test]
    fn test_approval_natural_height_narrow_with_args() {
        let h = approval_natural_height(40, "command: echo hello world test");
        assert!(h > 6, "narrow with args should need >6 rows, got {h}");
    }

    // ── Approval panel buffer rendering (F110) ────────────────────────

    fn render_approval_to_buffer(
        tool_name: &str,
        arguments: &str,
        width: u16,
        selected: usize,
    ) -> (ratatui::buffer::Buffer, u16) {
        let menu = BottomPanelState::open_approval(tool_name, arguments);
        let mut menu = menu;
        menu.selected_index = selected;
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        let h = comp.height_hint(width);
        let area = ratatui::layout::Rect::new(0, 0, width, h);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let mut frame = InlineFrame::new(area, &mut buf);
        comp.render(&mut frame, area);
        (buf, h)
    }

    fn buffer_line_content(buf: &ratatui::buffer::Buffer, y: u16, width: u16) -> String {
        (0..width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    fn buffer_contains(
        buf: &ratatui::buffer::Buffer,
        width: u16,
        height: u16,
        needle: &str,
    ) -> bool {
        (0..height).any(|y| buffer_line_content(buf, y, width).contains(needle))
    }

    #[test]
    fn test_approval_render_80_cols_shows_tool_name_and_options() {
        let (buf, h) = render_approval_to_buffer("bash", "command: echo hello", 80, 0);
        assert!(h >= 5);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 80))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("bash"), "tool name must be visible");
        assert!(
            all.contains("[y] approve"),
            "approve option must be visible"
        );
        assert!(
            all.contains("[a] always approve"),
            "always option must be visible"
        );
        assert!(all.contains("[n] deny"), "deny option must be visible");
    }

    #[test]
    fn test_approval_render_40_cols_shows_tool_name_and_options() {
        let (buf, h) = render_approval_to_buffer("bash", "command: echo hello world test", 40, 0);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 40))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("bash"), "tool name must be visible at 40 cols");
        assert!(
            all.contains("[y] approve"),
            "approve option must be visible at 40 cols"
        );
        assert!(
            all.contains("[n] deny"),
            "deny option must be visible at 40 cols"
        );
    }

    #[test]
    fn test_approval_render_120_cols_shows_all() {
        let (buf, h) =
            render_approval_to_buffer("write", "path: /some/long/path/to/file.rs", 120, 0);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 120))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("write"));
        assert!(all.contains("[y] approve"));
        assert!(all.contains("[n] deny"));
    }

    #[test]
    fn test_approval_render_selected_style_differs() {
        let (buf, _) = render_approval_to_buffer("bash", "echo hi", 80, 0);
        let selected_line = buffer_line_content(&buf, 2, 80);
        let unselected_line = buffer_line_content(&buf, 3, 80);
        assert!(
            selected_line.contains("[y] approve"),
            "first option should be selected: {selected_line}"
        );
        assert!(
            unselected_line.contains("[a] always approve"),
            "second option should be unselected: {unselected_line}"
        );
        let selected_cell = &buf[(2, 2)];
        let unselected_cell = &buf[(2, 3)];
        assert_ne!(
            selected_cell.bg, unselected_cell.bg,
            "selected and unselected must have different background"
        );
    }

    #[test]
    fn test_approval_render_40_cols_args_max_2_lines() {
        let long_args = "a".repeat(200);
        let (buf, h) = render_approval_to_buffer("bash", &long_args, 40, 0);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 40))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("bash"), "tool name must be visible");
        assert!(all.contains("[y] approve"), "options must be visible");
        assert!(all.contains("[n] deny"));
    }

    #[test]
    fn test_approval_render_insufficient_height_keeps_options() {
        let menu = BottomPanelState::open_approval("bash", "command: echo hello world");
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: 5,
        };
        let h = comp.height_hint(40);
        assert_eq!(h, 5, "should be capped at 5");
        let area = ratatui::layout::Rect::new(0, 0, 40, h);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let mut frame = InlineFrame::new(area, &mut buf);
        comp.render(&mut frame, area);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 40))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("bash"), "tool name must survive clipping");
        assert!(all.contains("[y] approve"), "options must survive clipping");
        assert!(
            all.contains("[n] deny"),
            "all 3 options should fit at height 5"
        );
    }

    #[test]
    fn test_approval_render_no_overflow_at_various_widths() {
        for width in [40u16, 60, 80, 120] {
            let (buf, h) = render_approval_to_buffer(
                "bash",
                "command: echo test args here with some length",
                width,
                0,
            );
            for y in 0..h {
                let line = buffer_line_content(&buf, y, width);
                let display_w = unicode_width::UnicodeWidthStr::width(line.as_str());
                assert!(
                    display_w <= width as usize,
                    "line {y} at width {width} has display width {display_w}: {line:?}"
                );
            }
        }
    }

    #[test]
    fn test_approval_render_cjk_display_width_no_overflow() {
        let cjk_args = "路径: /测试目录/文件名.txt 参数: 执行命令";
        let lines = wrap_text_to_lines(cjk_args, 36, 2);
        for line in &lines {
            let dw = unicode_width::UnicodeWidthStr::width(line.as_str());
            assert!(
                dw <= 36,
                "CJK wrap line display width {dw} exceeds 36: {line:?}"
            );
        }
        let menu = BottomPanelState::open_approval("文件操作", cjk_args);
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        let h = comp.height_hint(40);
        let area = ratatui::layout::Rect::new(0, 0, 40, h);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let mut frame = InlineFrame::new(area, &mut buf);
        comp.render(&mut frame, area);
    }

    #[test]
    fn test_approval_render_cjk_tool_name() {
        let menu = BottomPanelState::open_approval("文件操作", "路径: /测试/文件.txt");
        let comp = BottomPanelComponent {
            menu: &menu,
            query: "",
            max_height: u16::MAX,
        };
        let h = comp.height_hint(80);
        assert!(h >= 6, "CJK tool name should produce valid height");
        let area = ratatui::layout::Rect::new(0, 0, 80, h);
        let mut buf = ratatui::buffer::Buffer::empty(area);
        let mut frame = InlineFrame::new(area, &mut buf);
        comp.render(&mut frame, area);
        let all = (0..h)
            .map(|y| buffer_line_content(&buf, y, 80))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            all.contains("[y] approve"),
            "options must be visible with CJK tool name"
        );
        assert!(all.contains("[n] deny"));
    }

    // ── extract_thinking_title (F112) ──────────────────────────────────

    #[test]
    fn test_thinking_title_standalone_bold() {
        assert_eq!(
            extract_thinking_title("**Analyzing the problem**\n\nLet me think about..."),
            Some("Analyzing the problem")
        );
    }

    #[test]
    fn test_thinking_title_eof_after_title() {
        assert_eq!(extract_thinking_title("**Title**"), Some("Title"));
    }

    #[test]
    fn test_thinking_title_trailing_newline() {
        assert_eq!(extract_thinking_title("**Title**\n"), Some("Title"));
    }

    #[test]
    fn test_thinking_title_most_recent_wins() {
        assert_eq!(
            extract_thinking_title("prefix\n\n**First**\n\nbody\n\n**Second**\n\nmore"),
            Some("Second")
        );
    }

    #[test]
    fn test_thinking_title_crlf_input() {
        assert_eq!(
            extract_thinking_title("**Title**\r\n\r\nbody"),
            Some("Title")
        );
    }

    #[test]
    fn test_thinking_title_inline_bold_does_not_match() {
        assert_eq!(
            extract_thinking_title("Here is **bold** text in a line"),
            None
        );
    }

    #[test]
    fn test_thinking_title_no_blank_line_after_does_not_match() {
        assert_eq!(extract_thinking_title("**Title**\nbody"), None);
    }

    #[test]
    fn test_thinking_title_inline_suffix_does_not_match() {
        assert_eq!(extract_thinking_title("**Important:** inline text"), None);
    }

    #[test]
    fn test_thinking_title_empty_markers_do_not_match() {
        assert_eq!(extract_thinking_title("****\n\nbody"), None);
    }

    #[test]
    fn test_thinking_title_unclosed_does_not_match() {
        assert_eq!(extract_thinking_title("**未闭合\n\nbody"), None);
    }

    #[test]
    fn test_thinking_title_inner_asterisk_does_not_match() {
        assert_eq!(extract_thinking_title("**含*星号*的标题**\n\nbody"), None);
    }

    #[test]
    fn test_thinking_title_no_title_returns_none() {
        assert_eq!(extract_thinking_title("just regular thinking text"), None);
        assert_eq!(extract_thinking_title(""), None);
    }

    #[test]
    fn test_thinking_title_cjk_title() {
        assert_eq!(
            extract_thinking_title("**分析问题**\n\n接下来..."),
            Some("分析问题")
        );
    }

    #[test]
    fn test_thinking_title_title_without_body_section() {
        assert_eq!(
            extract_thinking_title("intro\n\n**Step 1**\n\ndetails\n\n**Step 2**\n"),
            Some("Step 2")
        );
    }

    // ── Status bar redesign ────────────────────────────────────────────

    #[test]
    fn test_status_bar_uses_accent_for_model() {
        let status = StatusSnapshot {
            model_name: "claude-sonnet-4".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("⬡ claude-sonnet-4"));
    }

    #[test]
    fn test_status_bar_omits_processing_text() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: true,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(!content.contains("◷"));
        assert!(!content.contains("processing"));
    }

    #[test]
    fn test_status_bar_no_spinner_when_idle() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(!content.contains("processing"));
    }

    #[test]
    fn test_status_bar_shows_terminal_phase_labels() {
        let base = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };

        let timed_out = StatusSnapshot {
            phase: Some(TurnPhase::TimedOut),
            ..base.clone()
        };
        let failed = StatusSnapshot {
            phase: Some(TurnPhase::Failed),
            ..base.clone()
        };
        let cancelled = StatusSnapshot {
            phase: Some(TurnPhase::Cancelled),
            ..base.clone()
        };
        let running_tool = StatusSnapshot {
            phase: Some(TurnPhase::RunningTool {
                name: "bash".to_string(),
            }),
            ..base
        };

        assert!(format!("{:?}", build_status_text(&timed_out, 120)).contains("timed out"));
        assert!(format!("{:?}", build_status_text(&failed, 120)).contains("failed"));
        assert!(format!("{:?}", build_status_text(&cancelled, 120)).contains("cancelled"));
        assert!(format!("{:?}", build_status_text(&running_tool, 120)).contains("tool: bash"));
    }

    #[test]
    fn test_status_bar_uses_human_readable_tokens() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 12_345,
                output_tokens: 8_900,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("8.9k out"));
        assert!(!content.contains("21245 tokens"));
    }

    #[test]
    fn test_status_bar_shows_reasoning_token_breakdown_when_present() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 12_345,
                output_tokens: 8_900,
                reasoning_tokens: 1_200,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };

        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("8.9k out"));
        assert!(content.contains("(1.2k thinking)"));
    }

    #[test]
    fn test_status_bar_formats_million_context_limit() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            context_limit: Some(1_000_000),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("1M ctx"));
        assert!(!content.contains("1000k ctx"));

        let two_million_status = StatusSnapshot {
            context_limit: Some(2_000_000),
            ..status
        };
        let two_million_content = format!("{:?}", build_status_text(&two_million_status, 120));
        assert!(two_million_content.contains("2M ctx"));
        assert!(!two_million_content.contains("2000k ctx"));
    }

    #[test]
    fn test_status_bar_preserves_sub_million_context_formats() {
        let raw_status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            context_limit: Some(999),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let kilo_status = StatusSnapshot {
            context_limit: Some(200_000),
            ..raw_status.clone()
        };
        let missing_status = StatusSnapshot {
            context_limit: None,
            ..raw_status.clone()
        };

        let raw_content = format!("{:?}", build_status_text(&raw_status, 120));
        let kilo_content = format!("{:?}", build_status_text(&kilo_status, 120));
        let missing_content = format!("{:?}", build_status_text(&missing_status, 120));

        assert!(raw_content.contains("999 ctx"));
        assert!(kilo_content.contains("200k ctx"));
        assert!(!missing_content.contains("ctx"));
    }

    #[test]
    fn test_status_bar_shows_context_usage_percentage() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 50_000,
                output_tokens: 10_000,
                ..Default::default()
            },
            context_limit: Some(200_000),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(content.contains("200k ctx"));
        assert!(content.contains("30%"));
    }

    #[test]
    fn test_status_bar_compact_mode_keeps_context_percentage_readable() {
        let status = StatusSnapshot {
            model_name: "claude-sonnet-4-20250514".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 40_000,
                output_tokens: 25_000,
                ..Default::default()
            },
            context_limit: Some(100_000),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 60);
        let content = format!("{:?}", text);
        assert!(content.contains("100k ctx"));
        assert!(content.contains("65%"));
    }

    #[test]
    fn test_status_bar_compact_mode_at_narrow_width() {
        let status = StatusSnapshot {
            model_name: "claude-sonnet-4-20250514".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 5_000,
                output_tokens: 3_000,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 2,
            followup_count: 1,
            is_processing: true,
            ..Default::default()
        };
        let wide = build_status_text(&status, 120);
        let narrow = build_status_text(&status, 60);
        let wide_text = format!("{:?}", wide);
        let narrow_text = format!("{:?}", narrow);
        assert!(!wide_text.contains("processing"));
        assert!(!narrow_text.contains("processing"));
        assert!(!narrow_text.contains("◷"));
    }

    #[test]
    fn test_truncate_str_short_enough() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_truncates_with_ellipsis() {
        let result = truncate_str("claude-sonnet-4-20250514", 12);
        assert!(result.ends_with('…'));
        assert_eq!(result.chars().count(), 12);
    }

    #[test]
    fn test_truncate_str_empty_string() {
        assert_eq!(truncate_str("", 10), "");
    }

    #[test]
    fn test_status_model_names_are_display_width_aware_without_padding_gap() {
        let base = StatusSnapshot {
            model_name: "short".to_string(),
            provider: "provider".to_string(),
            workspace_path: "workspace".to_string(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let unicode = StatusSnapshot {
            model_name: "非常长的模型名称-2026".to_string(),
            ..base.clone()
        };

        let short = build_status_text(&base, 120);
        let wide = build_status_text(&unicode, 120);
        let short_model = short.lines[0].spans[1].content.as_ref();
        let wide_model = wide.lines[0].spans[1].content.as_ref();

        assert!(short_model.starts_with("⬡ short (provider)"));
        assert!(wide_model.starts_with("⬡ 非常长的模型名称-2026 (provider)"));
        assert!(
            !short_model.contains("short                         (provider)"),
            "model and provider must remain visually adjacent"
        );
        assert!(UnicodeWidthStr::width(short.lines[0].to_string().as_str()) <= 120);
        assert!(UnicodeWidthStr::width(wide.lines[0].to_string().as_str()) <= 120);
    }

    #[test]
    fn test_truncate_str_respects_unicode_display_width() {
        let result = truncate_str("模型名称", 5);
        assert_eq!(result, "模型…");
        assert_eq!(UnicodeWidthStr::width(result.as_str()), 5);
    }

    #[test]
    fn test_status_bar_shows_workspace_path() {
        let status = StatusSnapshot {
            model_name: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
            workspace_path: "talos".to_string(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(
            content.contains("talos"),
            "status bar must include workspace path"
        );
    }

    #[test]
    fn test_status_bar_does_not_repeat_provider_qualified_model() {
        let status = StatusSnapshot {
            model_name: "zhipu-coding-plan/glm-5.2".to_string(),
            provider: "zhipu-coding-plan".to_string(),
            workspace_path: "~/WorkSpace/RustProjects/talos".to_string(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };

        let text = build_status_text(&status, 100);
        let content = format!("{:?}", text);

        assert!(content.contains("zhipu-coding-plan/glm"));
        assert!(!content.contains("(zhipu-coding"));
        assert!(content.contains("talos"));
        assert!(content.contains("0 out"));
    }

    #[test]
    fn test_status_bar_omits_workspace_when_empty() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(!content.contains("  t") || content.contains("test"));
    }

    #[test]
    fn test_status_bar_shows_context_limit() {
        let status = StatusSnapshot {
            model_name: "claude-sonnet-4".to_string(),
            provider: "anthropic".to_string(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            context_limit: Some(200_000),
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(
            content.contains("200k ctx"),
            "status bar must show context limit, got: {content}"
        );
    }

    #[test]
    fn test_status_bar_omits_context_when_none() {
        let status = StatusSnapshot {
            model_name: "unknown-model".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            context_limit: None,
            ..Default::default()
        };
        let text = build_status_text(&status, 120);
        let content = format!("{:?}", text);
        assert!(
            !content.contains("ctx"),
            "status bar must not show ctx when limit is None, got: {content}"
        );
    }

    #[test]
    fn test_exit_summary_uses_catalog_pricing() {
        use crate::app_summary::build_exit_summary_lines;
        use std::time::Duration;

        let status = StatusSnapshot {
            model_name: "claude-sonnet-4".to_string(),
            provider: "anthropic".to_string(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 1_000_000,
                output_tokens: 500_000,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            phase: None,
            context_limit: Some(200_000),
            input_price_per_million: Some(3.0),
            output_price_per_million: Some(15.0),
            variant: None,
            attachment_count: 0,
        };
        let lines = build_exit_summary_lines(
            &status,
            Duration::from_secs(60),
            5,
            Some("5a570406-d49e-48d6-9dc9-dde3548a3287"),
        );
        let text: String = lines.iter().map(|l| l.text.as_str()).collect();
        assert!(
            text.contains("session 5a570406-d49e-48d6-9dc9-dde3548a3287"),
            "exit summary should include session id, got: {text}"
        );
        assert!(
            text.contains("Est") && !text.contains("default"),
            "exit summary should use catalog pricing, got: {text}"
        );
        let expected = 1_000_000.0 * 3.0 / 1_000_000.0 + 500_000.0 * 15.0 / 1_000_000.0;
        assert!(
            text.contains(&format!("${expected:.2}")),
            "exit summary should show correct cost, got: {text}"
        );
    }

    #[test]
    fn test_exit_summary_omits_cost_without_pricing() {
        use crate::app_summary::build_exit_summary_lines;
        use std::time::Duration;

        let status = StatusSnapshot {
            model_name: "unknown-model".to_string(),
            provider: String::new(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 1_000_000,
                output_tokens: 500_000,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            phase: None,
            context_limit: Some(128_000),
            input_price_per_million: None,
            output_price_per_million: None,
            variant: None,
            attachment_count: 0,
        };
        let lines = build_exit_summary_lines(&status, Duration::from_secs(60), 5, None);
        let text: String = lines.iter().map(|l| l.text.as_str()).collect();
        assert!(
            !text.contains("cost") && !text.contains("default"),
            "exit summary must omit cost line when no pricing, got: {text}"
        );
    }

    #[test]
    fn test_exit_summary_shows_thinking_line_when_reasoning_tokens_present() {
        use crate::app_summary::build_exit_summary_lines;

        let status = StatusSnapshot {
            model_name: "claude-sonnet-4".to_string(),
            provider: "anthropic".to_string(),
            workspace_path: String::new(),
            usage: Usage {
                input_tokens: 1_000,
                output_tokens: 500,
                reasoning_tokens: 200,
                ..Default::default()
            },
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
            context_limit: Some(200_000),
            ..Default::default()
        };

        let lines = build_exit_summary_lines(&status, Duration::from_secs(60), 1, None);
        let text: String = lines.iter().map(|l| l.text.as_str()).collect();
        assert!(text.contains("200 thinking"));
    }

    // ── I147 wizard state-machine tests ──────────────────────────────

    #[test]
    fn wizard_opens_at_name_step() {
        let state = TuiState::new();
        let panel = crate::panel_state::BottomPanelState::open_provider_wizard();
        assert!(panel.is_open);
        assert!(panel.is_provider_wizard());
    }

    #[test]
    fn wizard_name_step_appends_and_backspaces() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();

        state.wizard_append_char('m');
        state.wizard_append_char('y');
        state.wizard_append_char('-');
        state.wizard_append_char('g');
        state.wizard_append_char('w');

        let name = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { name, .. }) => name.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(name, "my-gw");

        state.wizard_backspace();
        let name = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { name, .. }) => name.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(name, "my-g");
    }

    #[test]
    fn wizard_name_advances_to_protocol() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "my-gw".chars() {
            state.wizard_append_char(ch);
        }
        let action = state.wizard_advance();
        assert_eq!(action, None);
        let step = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { step, .. }) => *step,
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(step, crate::state::WizardStep::Protocol);
    }

    #[test]
    fn wizard_empty_name_does_not_advance() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        let action = state.wizard_advance();
        assert_eq!(action, None);
        let step = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { step, .. }) => *step,
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(step, crate::state::WizardStep::Name);
    }

    #[test]
    fn wizard_protocol_cycles_between_openai_and_anthropic() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "gw".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();

        let protocol = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { protocol, .. }) => protocol.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(protocol, "openai-chat");

        state.wizard_cycle_protocol();
        let protocol = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { protocol, .. }) => protocol.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(protocol, "anthropic-messages");

        state.wizard_cycle_protocol();
        let protocol = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { protocol, .. }) => protocol.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(protocol, "openai-chat");
    }

    #[test]
    fn wizard_full_flow_emits_register_action() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();

        // Step 1: Name
        for ch in "my-gw".chars() {
            state.wizard_append_char(ch);
        }
        assert_eq!(state.wizard_advance(), None);

        // Step 2: Protocol (default openai-chat, cycle to anthropic)
        state.wizard_cycle_protocol();
        assert_eq!(state.wizard_advance(), None);

        // Step 3: Base URL
        for ch in "https://api.example.com/v1".chars() {
            state.wizard_append_char(ch);
        }
        assert_eq!(state.wizard_advance(), None);

        // Step 4: API Key
        for ch in "secret-key".chars() {
            state.wizard_append_char(ch);
        }
        assert_eq!(state.wizard_advance(), None);

        // Step 5: Confirm
        let action = state.wizard_advance();
        assert!(action.is_some());
        match action.unwrap() {
            crate::state::PanelAction::RegisterCustomProvider {
                name,
                protocol,
                base_url,
                api_key,
            } => {
                assert_eq!(name, "my-gw");
                assert_eq!(protocol, "anthropic-messages");
                assert_eq!(base_url, "https://api.example.com/v1");
                assert_eq!(api_key, "secret-key");
            }
            other => panic!("expected RegisterCustomProvider, got {other:?}"),
        }
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn wizard_cancel_at_any_step_closes_without_side_effects() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "gw".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();
        state.wizard_cancel();
        assert!(!state.slash_menu.is_open);
    }

    #[test]
    fn wizard_empty_base_url_does_not_advance() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "gw".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();
        state.wizard_advance();
        let step = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { step, .. }) => *step,
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(step, crate::state::WizardStep::BaseUrl);
        assert_eq!(state.wizard_advance(), None);
        let step = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { step, .. }) => *step,
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(step, crate::state::WizardStep::BaseUrl);
    }

    #[test]
    fn wizard_empty_api_key_does_not_advance() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "gw".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();
        state.wizard_advance();
        for ch in "https://api.example.com/v1".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();
        assert_eq!(state.wizard_advance(), None);
        let step = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { step, .. }) => *step,
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(step, crate::state::WizardStep::ApiKey);
    }

    #[test]
    fn wizard_protocol_step_default_is_openai_chat() {
        let mut state = TuiState::new();
        state.slash_menu = crate::panel_state::BottomPanelState::open_provider_wizard();
        for ch in "gw".chars() {
            state.wizard_append_char(ch);
        }
        state.wizard_advance();
        let protocol = match &state.slash_menu.kind {
            Some(crate::state::PanelKind::ProviderWizard { protocol, .. }) => protocol.clone(),
            _ => panic!("expected ProviderWizard"),
        };
        assert_eq!(protocol, "openai-chat");
    }
}
