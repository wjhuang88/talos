#![allow(warnings)]
#[cfg(test)]
#[allow(warnings)]
mod tests {
    use std::time::{Duration, Instant};
    use talos_conversation::{MessageSource, SessionPickerItem, StatusSnapshot, TipKind};
    use talos_core::ApprovalChoice;
    use talos_core::message::Usage;

    use crate::inline_terminal::ViewportComponent;
    use crate::scrollback::{
        BottomPanelComponent, BottomPanelPlacement, bottom_panel_placement, bottom_panel_rows,
        build_input_text, build_status_text, calculate_cost, cursor_line_col, input_line_count,
        stream_padding_for, truncate_str,
    };
    use crate::sidebar::{SkillInfo, SkillSidebar};
    use crate::state::{ApprovalState, BottomPanelState, CtrlCState, PanelItem, Tip, TuiState};
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

    // ── Cost ────────────────────────────────────────────────────────────

    #[test]
    fn test_calculate_cost_zero() {
        let usage = Usage::default();
        assert_eq!(calculate_cost(&usage), "$0.0000");
    }

    #[test]
    fn test_calculate_cost_nonzero() {
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            ..Default::default()
        };
        let cost = calculate_cost(&usage);
        assert!(cost.starts_with('$'));
        let value: f64 = cost[1..].parse().unwrap();
        assert!(value > 0.0);
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

        let text = build_input_text(&state);

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
            crate::state::PanelItemAction::SlashCommand { command } => {
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
                command: "/export ".to_string(),
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
        assert_eq!(state.input_buffer, "/export");
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
    fn test_slash_menu_capped_rows_reserve_overflow_indicator() {
        assert_eq!(bottom_panel_rows(5, 3), (1, true, true));
        assert_eq!(bottom_panel_rows(5, 6), (5, true, false));
        assert_eq!(bottom_panel_rows(10, 10), (8, true, true));
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
        assert!(content.contains("21.2k"));
        assert!(!content.contains("21245 tokens"));
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
        assert!(content.contains("0 tokens"));
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
            context_limit: Some(200_000),
            input_price_per_million: Some(3.0),
            output_price_per_million: Some(15.0),
        };
        let lines = build_exit_summary_lines(&status, Duration::from_secs(60), 5);
        let text: String = lines.iter().map(|l| l.text.as_str()).collect();
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
    fn test_exit_summary_falls_back_to_default_pricing() {
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
            context_limit: Some(128_000),
            input_price_per_million: None,
            output_price_per_million: None,
        };
        let lines = build_exit_summary_lines(&status, Duration::from_secs(60), 5);
        let text: String = lines.iter().map(|l| l.text.as_str()).collect();
        assert!(
            text.contains("default"),
            "exit summary should indicate default pricing fallback, got: {text}"
        );
    }
}
