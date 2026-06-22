#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::colors;
    use crate::provider_setup::parse_provider;
    use crate::registry;
    use crate::tui_bridge::{SessionLifecycleRequest, run_conversation_loop};
    use talos_conversation::{ConversationEngine, UiOutput, UserInput};
    use talos_core::message::AgentEvent;

    #[test]
    fn parse_provider_anthropic() {
        assert_eq!(parse_provider("anthropic").unwrap(), "anthropic");
        assert_eq!(parse_provider("Anthropic").unwrap(), "anthropic");
        assert_eq!(parse_provider("ANTHROPIC").unwrap(), "anthropic");
    }

    #[test]
    fn parse_provider_openai() {
        assert_eq!(parse_provider("openai").unwrap(), "openai");
        assert_eq!(parse_provider("OpenAI").unwrap(), "openai");
    }

    #[test]
    fn parse_provider_custom_name() {
        assert_eq!(parse_provider("DashScope").unwrap(), "dashscope");
        assert!(parse_provider("").is_err());
    }

    // === Snippet Highlighting Tests ===

    #[test]
    fn highlight_snippet_replaces_b_tags() {
        let input = "This is a <b>matched</b> term in the snippet.";
        let output = registry::highlight_snippet(input);
        assert!(output.contains(colors::NORD13));
        assert!(output.contains(colors::BOLD));
        assert!(!output.contains("BOLD"));
        assert!(!output.contains("<b>"));
        assert!(!output.contains("</b>"));
    }

    #[test]
    fn highlight_snippet_multiple_matches() {
        let input = "<b>first</b> and <b>second</b> match";
        let output = registry::highlight_snippet(input);
        let nord13_count = output.matches(colors::NORD13).count();
        assert_eq!(
            nord13_count, 4,
            "Should have 4 NORD13 sequences (2 per match)"
        );
    }

    #[test]
    fn highlight_snippet_no_tags_passthrough() {
        let input = "No matches in this snippet.";
        let output = registry::highlight_snippet(input);
        assert_eq!(output, input);
    }

    #[test]
    fn highlight_snippet_empty_string() {
        let output = registry::highlight_snippet("");
        assert_eq!(output, "");
    }

    // === Session ID Parsing Tests ===

    #[test]
    fn session_id_valid_uuid_parses() {
        let valid_id = "550e8400-e29b-41d4-a716-446655440000";
        let result = uuid::Uuid::parse_str(valid_id);
        assert!(result.is_ok());
    }

    #[test]
    fn session_id_invalid_uuid_fails() {
        let invalid_ids = vec!["not-a-uuid", "550e8400-e29b-41d4-a716", ""];
        for invalid_id in invalid_ids {
            let result = uuid::Uuid::parse_str(invalid_id);
            assert!(result.is_err(), "Should fail to parse: {invalid_id}");
        }
    }

    // === Color Constant Tests ===

    #[test]
    fn color_constants_are_non_empty() {
        assert!(!colors::RESET.is_empty());
        assert!(!colors::BOLD.is_empty());
        assert!(!colors::NORD3.is_empty());
        assert!(!colors::NORD8.is_empty());
        assert!(!colors::NORD13.is_empty());
        assert!(!colors::NORD14.is_empty());
    }

    #[test]
    fn color_constants_contain_ansi_escape() {
        for color in [colors::NORD3, colors::NORD8, colors::NORD13, colors::NORD14] {
            assert!(
                color.starts_with("\x1b["),
                "Color constant should start with ANSI escape: {color:?}"
            );
        }
        assert!(colors::RESET.starts_with("\x1b["));
        assert!(colors::BOLD.starts_with("\x1b["));
    }

    #[tokio::test]
    async fn conversation_loop_displays_drained_queued_input() {
        let engine = ConversationEngine::new("test-model".to_string());
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (submit_tx, mut submit_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (session_tx, _session_rx) = tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            agent_rx,
            user_rx,
            ui_tx,
            submit_tx,
            interrupt_tx,
            session_tx,
        ));

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        user_tx
            .send(UserInput::Message("queued follow-up".to_string()))
            .unwrap();
        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                usage: Default::default(),
            })
            .unwrap();

        let mut saw_queued_user_stream = false;
        let mut saw_queue_drained_status = false;
        for _ in 0..8 {
            let Some(output) = ui_rx.recv().await else {
                break;
            };
            match output {
                UiOutput::Stream(msg) if msg.source == talos_conversation::MessageSource::User => {
                    saw_queued_user_stream = true;
                }
                UiOutput::Status(status) if status.is_processing && status.steering_count == 0 => {
                    saw_queue_drained_status = true;
                }
                _ => {}
            }
            if saw_queued_user_stream && saw_queue_drained_status {
                break;
            }
        }

        assert!(saw_queued_user_stream);
        assert!(saw_queue_drained_status);
        assert!(matches!(
            submit_rx.try_recv(),
            Ok(message) if message == "queued follow-up"
        ));

        drop(agent_tx);
        drop(user_tx);
        loop_handle.await.unwrap();
    }

    // === Error Handling Tests ===

    #[test]
    fn session_manager_resume_invalid_id() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let result = manager.resume_session("not-a-valid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn session_manager_resume_nonexistent_session() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let result = manager.resume_session(&valid_uuid);
        assert!(result.is_err());
        match result.unwrap_err() {
            talos_session::SessionError::SessionNotFound(_) => {}
            other => panic!("expected SessionNotFound, got {other:?}"),
        }
    }

    #[test]
    fn session_manager_search_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let results = manager.search("nonexistent", 10);
        if let Ok(r) = results {
            assert!(r.is_empty());
        }
    }

    #[test]
    fn session_manager_list_recent_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());

        let results = manager.list_recent(10);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }
}
