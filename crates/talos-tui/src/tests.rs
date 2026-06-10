#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use talos_conversation::{StatusSnapshot, TipKind};
    use talos_core::ApprovalChoice;
    use talos_core::message::Usage;

    use crate::app::{build_status_text, calculate_cost};
    use crate::sidebar::{SkillInfo, SkillSidebar};
    use crate::state::{ApprovalState, CtrlCState, Tip, TuiState};
    use crate::{contrast_ratio, rgb_components};
    use crate::theme::nord;

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
            usage: Usage::default(),
            branch_id: None,
            steering_count: 0,
            followup_count: 0,
            is_processing: false,
        };
        let text = build_status_text(&status);
        let content = format!("{:?}", text);
        assert!(!content.contains("S:"));
        assert!(!content.contains("F:"));
    }

    #[test]
    fn test_queue_indicator_in_status_when_steering_queued() {
        let status = StatusSnapshot {
            model_name: "test".to_string(),
            usage: Usage::default(),
            branch_id: None,
            steering_count: 3,
            followup_count: 0,
            is_processing: false,
        };
        let text = build_status_text(&status);
        let content = format!("{:?}", text);
        assert!(content.contains("S:3"));
    }
}
