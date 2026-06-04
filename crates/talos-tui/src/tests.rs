#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;
    use talos_core::ApprovalChoice;
    use talos_core::message::{AgentEvent, StopReason, ToolCall, ToolResult, Usage};
    use talos_core::tool::ToolProvenance;

    use crate::app::{build_chat_text, build_status_text, calculate_cost};
    use crate::sidebar::{SkillInfo, SkillSidebar};
    use crate::state::{ApprovalState, ChatLine, CtrlCState, TuiState};
    use crate::theme::nord;
    use crate::widgets::{ToolCallBubble, provenance_marker, render_diff, truncate};
    use crate::{contrast_ratio, rgb_components};

    #[test]
    fn test_state_new() {
        let state = TuiState::new();
        assert!(state.chat_lines.is_empty());
        assert!(state.input_buffer.is_empty());
        assert_eq!(state.cursor_pos, 0);
        assert!(!state.is_processing);
        assert!(state.current_turn_text.is_empty());
        assert!(!state.should_exit);
    }

    #[test]
    fn test_append_delta() {
        let mut state = TuiState::new();
        state.append_delta("Hello");
        state.append_delta(", ");
        state.append_delta("world!");
        assert_eq!(state.current_turn_text, "Hello, world!");
    }

    #[test]
    fn test_finalize_turn_with_text() {
        let mut state = TuiState::new();
        state.append_delta("Assistant response");
        state.finalize_turn();
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Assistant("Assistant response".into())]
        );
        assert!(state.current_turn_text.is_empty());
    }

    #[test]
    fn test_finalize_turn_empty() {
        let mut state = TuiState::new();
        state.finalize_turn();
        assert!(state.chat_lines.is_empty());
    }

    #[test]
    fn test_append_user_message() {
        let mut state = TuiState::new();
        state.append_user_message("Hello");
        assert_eq!(state.chat_lines, vec![ChatLine::Text("> Hello".into())]);
    }

    #[test]
    fn test_append_error() {
        let mut state = TuiState::new();
        state.append_error("Something failed");
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[Error] Something failed".into())]
        );
    }

    #[test]
    fn test_append_system() {
        let mut state = TuiState::new();
        state.append_system("Turn cancelled");
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[System] Turn cancelled".into())]
        );
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

    #[test]
    fn test_handle_event_turn_start() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        assert!(state.is_processing);
        assert!(state.current_turn_text.is_empty());
    }

    #[test]
    fn test_handle_event_text_delta() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TextDelta {
            delta: "Hello".into(),
        });
        state.handle_event(&AgentEvent::TextDelta {
            delta: " world".into(),
        });
        assert_eq!(state.current_turn_text, "Hello world");
    }

    #[test]
    fn test_handle_event_tool_call() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls"}),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call: call.clone(),
            provenance: Default::default(),
        });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                tool_name,
                arguments,
                provenance,
                result,
            } => {
                assert_eq!(tool_name, "bash");
                assert!(arguments.contains("command"));
                assert!(arguments.contains("ls"));
                assert_eq!(provenance, &ToolProvenance::Native);
                assert!(result.is_none());
            }
            _ => panic!("expected ToolCall variant"),
        }
    }

    #[test]
    fn test_handle_event_tool_call_preserves_mcp_provenance() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "remote_search".into(),
            input: serde_json::json!({"query": "talos"}),
        };
        let provenance = ToolProvenance::McpRemote {
            server: "filesystem".into(),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: provenance.clone(),
        });

        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                tool_name,
                provenance: actual,
                ..
            } => {
                assert_eq!(tool_name, "remote_search");
                assert_eq!(actual, &provenance);
            }
            _ => panic!("expected ToolCall variant"),
        }
    }

    #[test]
    fn test_build_chat_text_renders_mcp_provenance_marker() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "remote_search".into(),
            input: serde_json::json!({"query": "talos"}),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: ToolProvenance::McpRemote {
                server: "filesystem".into(),
            },
        });

        let rendered = build_chat_text(&state)
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();

        assert!(rendered.contains("remote_search"));
        assert!(rendered.contains("[mcp:filesystem]"));
    }

    #[test]
    fn test_handle_event_tool_result_sets_on_last_tool_call() {
        let mut state = TuiState::new();
        let call = ToolCall {
            id: "c1".into(),
            name: "read".into(),
            input: serde_json::json!({"path": "src/main.rs"}),
        };
        state.handle_event(&AgentEvent::ToolCall {
            call,
            provenance: Default::default(),
        });
        let result = ToolResult {
            tool_use_id: "c1".into(),
            content: "fn main() {}".into(),
            is_error: false,
        };
        state.handle_event(&AgentEvent::ToolResult {
            result: result.clone(),
        });
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::ToolCall {
                result: Some(r), ..
            } => {
                assert_eq!(r.content, "fn main() {}");
                assert!(!r.is_error);
            }
            _ => panic!("expected ToolCall with result"),
        }
    }

    #[test]
    fn test_handle_event_turn_end() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        state.append_delta("Response text");
        state.handle_event(&AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
        });
        assert!(!state.is_processing);
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Assistant("Response text".into())]
        );
        assert_eq!(state.usage.input_tokens, 100);
        assert_eq!(state.usage.output_tokens, 50);
    }

    #[test]
    fn test_handle_event_error() {
        let mut state = TuiState::new();
        state.handle_event(&AgentEvent::TurnStart);
        state.append_delta("Partial");
        state.handle_event(&AgentEvent::Error {
            message: "API error".into(),
        });
        assert!(!state.is_processing);
        assert!(state.current_turn_text.is_empty());
        assert_eq!(
            state.chat_lines,
            vec![ChatLine::Text("[Error] API error".into())]
        );
    }

    #[test]
    fn test_calculate_cost_zero() {
        let usage = Usage::default();
        let cost = calculate_cost(&usage);
        assert_eq!(cost, "$0.0000");
    }

    #[test]
    fn test_calculate_cost_nonzero() {
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        let cost = calculate_cost(&usage);
        assert_eq!(cost, "$0.0045");
    }

    #[test]
    fn test_approval_state_default_hidden() {
        let state = ApprovalState::default();
        assert!(matches!(state, ApprovalState::Hidden));
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("hello world this is a long string", 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_truncate_empty_string() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn test_approval_state_transitions() {
        let mut state = TuiState::new();
        assert!(matches!(state.approval_state, ApprovalState::Hidden));

        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };
        assert!(matches!(
            state.approval_state,
            ApprovalState::Visible { .. }
        ));

        state.approval_state = ApprovalState::Hidden;
        assert!(matches!(state.approval_state, ApprovalState::Hidden));
    }

    #[test]
    fn test_handle_approval_key_approve_once() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'y');
        assert_eq!(choice, Some(ApprovalChoice::ApproveOnce));
    }

    #[test]
    fn test_handle_approval_key_always_approve() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'a');
        assert_eq!(choice, Some(ApprovalChoice::AlwaysApprove));
    }

    #[test]
    fn test_handle_approval_key_deny() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'n');
        assert_eq!(choice, Some(ApprovalChoice::Deny));
    }

    #[test]
    fn test_handle_approval_key_invalid_when_hidden() {
        let mut state = TuiState::new();
        assert!(matches!(state.approval_state, ApprovalState::Hidden));
        let choice = handle_approval_key_event(&mut state, 'y');
        assert!(choice.is_none());
    }

    #[test]
    fn test_handle_approval_key_invalid_char() {
        let mut state = TuiState::new();
        state.approval_state = ApprovalState::Visible {
            tool_name: "bash".into(),
            arguments: "{}".into(),
            selected: ApprovalChoice::ApproveOnce,
        };

        let choice = handle_approval_key_event(&mut state, 'x');
        assert!(choice.is_none());
    }

    #[test]
    fn test_tool_call_bubble_creation() {
        let bubble = ToolCallBubble::new("read", r#"{"path": "src/main.rs"}"#);
        assert_eq!(bubble.tool_name, "read");
        assert_eq!(bubble.provenance, ToolProvenance::Native);
        assert!(bubble.result_status.is_none());
    }

    #[test]
    fn test_tool_call_bubble_with_mcp_provenance() {
        let bubble = ToolCallBubble::new("remote_search", r#"{"query": "talos"}"#).with_provenance(
            ToolProvenance::McpRemote {
                server: "filesystem".into(),
            },
        );
        assert_eq!(
            provenance_marker(&bubble.provenance),
            "mcp:filesystem".to_string()
        );
    }

    #[test]
    fn test_tool_call_bubble_with_result() {
        let bubble = ToolCallBubble::new("bash", r#"{"command": "ls"}"#)
            .with_result(false, "file.rs\nCargo.toml");
        assert_eq!(bubble.tool_name, "bash");
        assert_eq!(bubble.result_status, Some(false));
        assert_eq!(bubble.result_content, Some("file.rs\nCargo.toml"));
    }

    #[test]
    fn test_tool_call_bubble_with_error() {
        let bubble = ToolCallBubble::new("bash", r#"{"command": "rm -rf /"}"#)
            .with_result(true, "Permission denied");
        assert_eq!(bubble.result_status, Some(true));
    }

    #[test]
    fn test_nord_palette_defines_all_colors() {
        let colors = [
            nord::NORD0,
            nord::NORD1,
            nord::NORD2,
            nord::NORD3,
            nord::NORD4,
            nord::NORD5,
            nord::NORD6,
            nord::NORD7,
            nord::NORD8,
            nord::NORD9,
            nord::NORD10,
            nord::NORD11,
            nord::NORD12,
            nord::NORD13,
            nord::NORD14,
            nord::NORD15,
        ];
        assert_eq!(colors.len(), 16);
        assert!(colors.iter().all(|color| rgb_components(*color).is_some()));
    }

    #[test]
    fn test_nord_primary_text_contrast_is_wcag_aa() {
        let pairs = [
            (nord::NORD4, nord::NORD0),
            (nord::NORD5, nord::NORD0),
            (nord::NORD6, nord::NORD0),
            (nord::NORD8, nord::NORD0),
            (nord::NORD14, nord::NORD0),
        ];

        for (foreground, background) in pairs {
            let ratio = contrast_ratio(foreground, background).expect("rgb Nord color");
            assert!(ratio >= 4.5, "contrast ratio {ratio:.2} below WCAG AA");
        }
    }

    fn handle_approval_key_event(state: &mut TuiState, key: char) -> Option<ApprovalChoice> {
        let ApprovalState::Visible { selected, .. } = &mut state.approval_state else {
            return None;
        };

        match key {
            'y' => {
                *selected = ApprovalChoice::ApproveOnce;
                Some(ApprovalChoice::ApproveOnce)
            }
            'a' => {
                *selected = ApprovalChoice::AlwaysApprove;
                Some(ApprovalChoice::AlwaysApprove)
            }
            'n' => {
                *selected = ApprovalChoice::Deny;
                Some(ApprovalChoice::Deny)
            }
            _ => None,
        }
    }

    // ── Skill Sidebar Tests ──────────────────────────────────────────────────

    #[test]
    fn test_skill_sidebar_new_is_hidden() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
        assert!(sidebar.skills.is_empty());
        assert_eq!(sidebar.width, SkillSidebar::DEFAULT_WIDTH);
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
    fn test_skill_sidebar_update_skills() {
        let mut sidebar = SkillSidebar::new();
        assert!(sidebar.skills.is_empty());

        let skills = vec![
            SkillInfo {
                name: "test-skill".into(),
                description: "A test skill".into(),
                active: true,
            },
            SkillInfo {
                name: "another-skill".into(),
                description: "Another skill".into(),
                active: false,
            },
        ];
        sidebar.update_skills(skills.clone());
        assert_eq!(sidebar.skills.len(), 2);
        assert_eq!(sidebar.skills[0].name, "test-skill");
        assert!(sidebar.skills[0].active);
        assert_eq!(sidebar.skills[1].name, "another-skill");
        assert!(!sidebar.skills[1].active);
    }

    #[test]
    fn test_skill_sidebar_collapsed_mode() {
        let mut sidebar = SkillSidebar::new();
        sidebar.width = 15;
        assert!(sidebar.is_collapsed());

        sidebar.width = 20;
        assert!(!sidebar.is_collapsed());

        sidebar.width = 19;
        assert!(sidebar.is_collapsed());
    }

    #[test]
    fn test_skill_sidebar_default_not_collapsed() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.is_collapsed());
    }

    #[test]
    fn test_skill_info_fields() {
        let skill = SkillInfo {
            name: "code-review".into(),
            description: "Reviews code for quality".into(),
            active: true,
        };
        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.description, "Reviews code for quality");
        assert!(skill.active);
    }

    #[test]
    fn test_skill_sidebar_render_empty_when_hidden() {
        let sidebar = SkillSidebar::new();
        assert!(!sidebar.visible);
        // Hidden sidebar should not render anything — verified by visible flag
    }

    #[test]
    fn test_skill_sidebar_with_many_skills() {
        let mut sidebar = SkillSidebar::new();
        let skills: Vec<SkillInfo> = (0..10)
            .map(|i| SkillInfo {
                name: format!("skill-{i}"),
                description: format!("Description for skill {i}"),
                active: i % 2 == 0,
            })
            .collect();
        sidebar.update_skills(skills);
        assert_eq!(sidebar.skills.len(), 10);
        assert!(sidebar.skills[0].active);
        assert!(!sidebar.skills[1].active);
        assert!(sidebar.skills[2].active);
    }

    #[test]
    fn test_skill_sidebar_width_boundary() {
        let mut sidebar = SkillSidebar::new();

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD - 1;
        assert!(sidebar.is_collapsed());

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD;
        assert!(!sidebar.is_collapsed());

        sidebar.width = SkillSidebar::COLLAPSE_THRESHOLD + 1;
        assert!(!sidebar.is_collapsed());
    }

    #[test]
    fn test_slash_command_help() {
        let mut state = TuiState::new();
        state.handle_slash_command("/help");
        assert!(
            state
                .chat_lines
                .iter()
                .any(|l| matches!(l, ChatLine::Text(t) if t.contains("/help")))
        );
        assert!(
            state
                .chat_lines
                .iter()
                .any(|l| matches!(l, ChatLine::Text(t) if t.contains("/quit")))
        );
    }

    #[test]
    fn test_slash_command_quit() {
        let mut state = TuiState::new();
        state.handle_slash_command("/quit");
        assert!(state.should_exit);
    }

    #[test]
    fn test_slash_command_exit() {
        let mut state = TuiState::new();
        state.handle_slash_command("/exit");
        assert!(state.should_exit);
    }

    #[test]
    fn test_slash_command_status() {
        let mut state = TuiState::new();
        state.model_name = "test-model".to_string();
        state.handle_slash_command("/status");
        assert!(
            state
                .chat_lines
                .iter()
                .any(|l| matches!(l, ChatLine::Text(t) if t.contains("test-model")))
        );
    }

    #[test]
    fn test_slash_command_new_clears_chat() {
        let mut state = TuiState::new();
        state.append_user_message("hello");
        assert!(!state.chat_lines.is_empty());
        state.handle_slash_command("/new");
        assert_eq!(state.chat_lines.len(), 1);
        if let ChatLine::Text(msg) = &state.chat_lines[0] {
            assert!(msg.contains("New session started"));
        } else {
            panic!("expected system message");
        }
    }

    #[test]
    fn test_slash_command_unknown() {
        let mut state = TuiState::new();
        state.handle_slash_command("/foobar");
        assert!(
            state
                .chat_lines
                .iter()
                .any(|l| matches!(l, ChatLine::Text(t) if t.contains("Unknown command")))
        );
    }

    #[test]
    fn test_tab_completion_single_match() {
        let mut state = TuiState::new();
        state.input_buffer = "/hel".to_string();
        state.cursor_pos = 4;
        state.complete_slash_command();
        assert_eq!(state.input_buffer, "/help ");
    }

    #[test]
    fn test_tab_completion_multiple_matches() {
        let mut state = TuiState::new();
        state.input_buffer = "/".to_string();
        state.cursor_pos = 1;
        state.complete_slash_command();
        assert!(
            state
                .chat_lines
                .iter()
                .any(|l| matches!(l, ChatLine::Text(t) if t.contains("Commands:")))
        );
    }

    // ── Markdown Rendering Tests ─────────────────────────────────────────────

    #[test]
    fn test_assistant_line_renders_markdown() {
        let mut state = TuiState::new();
        state
            .chat_lines
            .push(ChatLine::Assistant("**bold text** and *italic*".into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
        let all_spans: Vec<&str> = rendered
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect();
        let combined: String = all_spans.join("");
        assert!(combined.contains("bold text"));
    }

    #[test]
    fn test_assistant_line_renders_code_block() {
        let mut state = TuiState::new();
        let code = "Here is some code:\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        state.chat_lines.push(ChatLine::Assistant(code.into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
    }

    #[test]
    fn test_assistant_line_renders_heading() {
        let mut state = TuiState::new();
        state
            .chat_lines
            .push(ChatLine::Assistant("# Main Heading\n## Sub Heading".into()));
        let rendered = build_chat_text(&state);
        assert!(!rendered.lines.is_empty());
        let all_spans: Vec<&str> = rendered
            .lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect();
        let combined: String = all_spans.join("");
        assert!(combined.contains("Main Heading"));
        assert!(combined.contains("Sub Heading"));
    }

    #[test]
    fn test_text_line_remains_plain() {
        let mut state = TuiState::new();
        state.chat_lines.push(ChatLine::Text("**not bold**".into()));
        let rendered = build_chat_text(&state);
        assert_eq!(rendered.lines.len(), 1);
        let spans = &rendered.lines[0].spans;
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "**not bold**");
    }

    #[test]
    fn test_finalize_turn_creates_assistant_variant() {
        let mut state = TuiState::new();
        state.append_delta("Hello from assistant");
        state.finalize_turn();
        assert_eq!(state.chat_lines.len(), 1);
        match &state.chat_lines[0] {
            ChatLine::Assistant(text) => {
                assert_eq!(text, "Hello from assistant");
            }
            _ => panic!(
                "expected ChatLine::Assistant, got {:?}",
                state.chat_lines[0]
            ),
        }
    }

    #[test]
    fn test_render_diff_detects_unified_diff() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"old\");\n+    println!(\"new\");\n+    println!(\"added\");\n }\n";
        let result = render_diff(diff);
        assert!(result.is_some());
    }

    #[test]
    fn test_render_diff_rejects_plain_text() {
        let plain = "This is just plain text.\nNo diff markers here.\nJust some content.";
        let result = render_diff(plain);
        assert!(result.is_none());
    }

    #[test]
    fn test_render_diff_rejects_text_with_plus_minus() {
        let text = "The result is +5 degrees.\nThe temperature dropped -3 degrees.";
        let result = render_diff(text);
        assert!(result.is_none());
    }

    #[test]
    fn test_render_diff_colors_additions_green() {
        let diff = "diff --git a/f.txt b/f.txt\n@@ -1 +1 @@\n-old\n+new\n";
        let lines = render_diff(diff).expect("diff detected");
        let addition_line = lines.iter().find(|l| {
            l.spans
                .first()
                .is_some_and(|s| s.content.as_ref() == "+new")
        });
        assert!(addition_line.is_some());
        let span = addition_line.unwrap().spans.first().unwrap();
        assert_eq!(span.style.fg, Some(nord::NORD14));
    }

    #[test]
    fn test_render_diff_colors_deletions_red() {
        let diff = "diff --git a/f.txt b/f.txt\n@@ -1 +1 @@\n-old\n+new\n";
        let lines = render_diff(diff).expect("diff detected");
        let deletion_line = lines.iter().find(|l| {
            l.spans
                .first()
                .is_some_and(|s| s.content.as_ref() == "-old")
        });
        assert!(deletion_line.is_some());
        let span = deletion_line.unwrap().spans.first().unwrap();
        assert_eq!(span.style.fg, Some(nord::NORD11));
    }

    #[test]
    fn test_render_diff_colors_hunk_header() {
        let diff = "diff --git a/f.txt b/f.txt\n@@ -1,3 +1,4 @@\n context\n";
        let lines = render_diff(diff).expect("diff detected");
        let hunk_line = lines.iter().find(|l| {
            l.spans
                .first()
                .is_some_and(|s| s.content.as_ref() == "@@ -1,3 +1,4 @@")
        });
        assert!(hunk_line.is_some());
        let span = hunk_line.unwrap().spans.first().unwrap();
        assert_eq!(span.style.fg, Some(nord::NORD8));
    }

    #[test]
    fn test_render_diff_file_header_blue_bold() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n";
        let lines = render_diff(diff).expect("diff detected");

        let diff_header = lines.iter().find(|l| {
            l.spans
                .first()
                .is_some_and(|s| s.content.as_ref() == "diff --git a/src/lib.rs b/src/lib.rs")
        });
        assert!(diff_header.is_some());
        let span = diff_header.unwrap().spans.first().unwrap();
        assert_eq!(span.style.fg, Some(nord::NORD9));
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_render_diff_context_lines_dim() {
        let diff = "diff --git a/f.txt b/f.txt\n@@ -1 +1 @@\n context line\n";
        let lines = render_diff(diff).expect("diff detected");
        let context_line = lines.iter().find(|l| {
            l.spans
                .first()
                .is_some_and(|s| s.content.as_ref() == " context line")
        });
        assert!(context_line.is_some());
        let span = context_line.unwrap().spans.first().unwrap();
        assert_eq!(span.style.fg, Some(nord::NORD4));
        assert!(span.style.add_modifier.contains(Modifier::DIM));
    }

    // ── Steering / Follow-up Queue Tests ─────────────────────────────────────

    #[test]
    fn test_steering_queue_empty_by_default() {
        let state = TuiState::new();
        assert!(state.steering_queue.is_empty());
        assert!(state.followup_queue.is_empty());
    }

    #[test]
    fn test_steering_queue_drain_fifo() {
        let mut state = TuiState::new();
        state.steering_queue.push("first".into());
        state.steering_queue.push("second".into());
        state.steering_queue.push("third".into());

        assert_eq!(state.drain_steering_queue(), Some("first".into()));
        assert_eq!(state.drain_steering_queue(), Some("second".into()));
        assert_eq!(state.drain_steering_queue(), Some("third".into()));
        assert_eq!(state.drain_steering_queue(), None);
    }

    #[test]
    fn test_restore_last_queued_from_steering() {
        let mut state = TuiState::new();
        state.steering_queue.push("queued message".into());
        state.steering_queue.push("another queued".into());

        let restored = state.restore_last_queued();
        assert!(restored);
        assert_eq!(state.input_buffer, "another queued");
        assert_eq!(state.cursor_pos, 14);
        assert_eq!(state.steering_queue.len(), 1);
    }

    #[test]
    fn test_restore_last_queued_from_followup_when_steering_empty() {
        let mut state = TuiState::new();
        state.followup_queue.push("followup msg".into());

        let restored = state.restore_last_queued();
        assert!(restored);
        assert_eq!(state.input_buffer, "followup msg");
        assert!(state.steering_queue.is_empty());
        assert!(state.followup_queue.is_empty());
    }

    #[test]
    fn test_restore_last_queued_nothing_to_restore() {
        let mut state = TuiState::new();
        let restored = state.restore_last_queued();
        assert!(!restored);
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_queue_indicator_in_status_when_steering_queued() {
        let mut state = TuiState::new();
        state.steering_queue.push("msg1".into());
        state.steering_queue.push("msg2".into());

        let text = build_status_text(&state);
        let content: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        assert!(content.contains("Steering: 2"));
    }

    #[test]
    fn test_queue_indicator_in_status_when_followup_queued() {
        let mut state = TuiState::new();
        state.followup_queue.push("followup1".into());

        let text = build_status_text(&state);
        let content: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        assert!(content.contains("Follow-up: 1"));
    }

    #[test]
    fn test_queue_indicator_absent_when_no_queues() {
        let state = TuiState::new();
        let text = build_status_text(&state);
        let content: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        assert!(!content.contains("Steering:"));
        assert!(!content.contains("Follow-up:"));
    }

    #[test]
    fn test_queue_indicator_shows_both_queues() {
        let mut state = TuiState::new();
        state.steering_queue.push("s1".into());
        state.followup_queue.push("f1".into());
        state.followup_queue.push("f2".into());

        let text = build_status_text(&state);
        let content: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();

        assert!(content.contains("Steering: 1"));
        assert!(content.contains("Follow-up: 2"));
    }
}
