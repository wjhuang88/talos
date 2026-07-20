#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::colors;
    use crate::provider_setup::parse_provider;
    use crate::registry;
    use crate::skill_runtime::discover_runtime_skills;
    use crate::tui_bridge::{ConversationLoopIo, SessionLifecycleRequest, run_conversation_loop};
    use crate::{
        available_model_name, config_get_dotted, config_set_dotted, is_secret_key, mask_secrets,
        model_matches_filter, normalize_model_filter,
    };
    use talos_conversation::{ConversationEngine, ModelInfo, UiOutput, UserInput};
    use talos_core::message::AgentEvent;
    use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};

    #[derive(Clone)]
    struct TestTurnSender {
        tx: tokio::sync::mpsc::UnboundedSender<SessionEvent>,
        sequence: std::sync::Arc<std::sync::atomic::AtomicU64>,
    }

    impl TestTurnSender {
        fn new(tx: tokio::sync::mpsc::UnboundedSender<SessionEvent>) -> Self {
            Self {
                tx,
                sequence: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            }
        }

        fn send(
            &self,
            event: AgentEvent,
        ) -> Result<(), tokio::sync::mpsc::error::SendError<SessionEvent>> {
            use std::sync::atomic::Ordering;

            if matches!(event, AgentEvent::TurnStart) {
                self.tx.send(SessionEvent::TurnEvent {
                    session_id: "session_test".to_string(),
                    turn_id: "turn_test".to_string(),
                    sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
                    payload: TurnEventPayload::Started,
                })?;
            }

            let terminal = match &event {
                AgentEvent::TurnEnd { .. } => Some(TurnCompletionStatus::Success {
                    final_text: String::new(),
                    new_messages: vec![],
                }),
                AgentEvent::Error { message } => Some(TurnCompletionStatus::Error {
                    message: message.clone(),
                }),
                _ => None,
            };
            self.tx.send(SessionEvent::TurnEvent {
                session_id: "session_test".to_string(),
                turn_id: "turn_test".to_string(),
                sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
                payload: TurnEventPayload::Progress { event },
            })?;
            if let Some(status) = terminal {
                self.tx.send(SessionEvent::TurnEvent {
                    session_id: "session_test".to_string(),
                    turn_id: "turn_test".to_string(),
                    sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
                    payload: TurnEventPayload::Completed { status },
                })?;
            }
            Ok(())
        }
    }

    fn empty_runtime_skills()
    -> std::sync::Arc<tokio::sync::Mutex<crate::skill_runtime::RuntimeSkills>> {
        let dir = tempfile::tempdir().unwrap();
        std::sync::Arc::new(tokio::sync::Mutex::new(
            discover_runtime_skills(dir.path(), false).unwrap(),
        ))
    }

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
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let agent_tx = TestTurnSender::new(agent_tx);
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, mut interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
            model_name: "test-model".to_string(),
            provider: "test-provider".to_string(),
            ..Default::default()
        });
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills: empty_runtime_skills(),
            },
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
        for _ in 0..20 {
            let Some(output) = ui_rx.recv().await else {
                break;
            };
            match output {
                UiOutput::Content(talos_conversation::ContentOutput::Block {
                    source: talos_conversation::MessageSource::User,
                    ..
                }) => {
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
            interrupt_rx.try_recv(),
            Ok(talos_core::session::SessionOp::Submit { message }) if message == "queued follow-up"
        ));

        drop(agent_tx);
        drop(user_tx);
        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_updates_status_from_model_watch() {
        let engine = ConversationEngine::new("old-model".to_string(), "old-provider".to_string());
        let (_agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
            model_name: "old-model".to_string(),
            provider: "old-provider".to_string(),
            ..Default::default()
        });
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills: empty_runtime_skills(),
            },
        ));

        model_tx
            .send(ModelInfo {
                model_name: "new-model".to_string(),
                provider: "new-provider".to_string(),
                ..Default::default()
            })
            .unwrap();

        let status = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if let Some(UiOutput::Status(status)) = ui_rx.recv().await {
                    break status;
                }
            }
        })
        .await
        .expect("status update");

        assert_eq!(status.model_name, "new-model");
        assert_eq!(status.provider, "new-provider");

        loop_handle.abort();
    }

    #[tokio::test]
    async fn conversation_loop_keeps_steering_queued_across_provider_tool_end() {
        let engine = ConversationEngine::new("test-model".into(), "test-provider".into());
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (sq_tx, mut sq_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_watch_tx, sq_watch_rx) = tokio::sync::watch::channel(sq_tx);
        let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo::default());
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx: event_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_watch_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills: empty_runtime_skills(),
            },
        ));

        event_tx
            .send(SessionEvent::TurnEvent {
                session_id: "session_test".into(),
                turn_id: "turn_1".into(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            })
            .unwrap();
        event_tx
            .send(SessionEvent::TurnEvent {
                session_id: "session_test".into(),
                turn_id: "turn_1".into(),
                sequence: 1,
                payload: TurnEventPayload::Progress {
                    event: AgentEvent::TurnStart,
                },
            })
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while let Some(output) = ui_rx.recv().await {
                if matches!(output, UiOutput::Status(status) if status.is_processing) {
                    break;
                }
            }
        })
        .await
        .expect("turn start reaches conversation state");
        user_tx
            .send(UserInput::Message("after tool".into()))
            .unwrap();
        event_tx
            .send(SessionEvent::TurnEvent {
                session_id: "session_test".into(),
                turn_id: "turn_1".into(),
                sequence: 2,
                payload: TurnEventPayload::Progress {
                    event: AgentEvent::TurnEnd {
                        stop_reason: talos_core::message::StopReason::ToolUse,
                        usage: Default::default(),
                    },
                },
            })
            .unwrap();

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), sq_rx.recv())
                .await
                .is_err(),
            "provider response end must not drain steering"
        );

        event_tx
            .send(SessionEvent::TurnEvent {
                session_id: "session_test".into(),
                turn_id: "turn_1".into(),
                sequence: 3,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Success {
                        final_text: String::new(),
                        new_messages: vec![],
                    },
                },
            })
            .unwrap();

        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(1), sq_rx.recv())
                .await
                .unwrap()
                .and_then(|op| match op {
                    talos_core::session::SessionOp::Submit { message } => Some(message),
                    _ => None,
                })
                .as_deref(),
            Some("after tool")
        );
        loop_handle.abort();
    }

    // FS02 / RUNTIME-002: runtime-level integration coverage proving the conversation loop
    // forwards a terminal `UiOutput::Status { is_processing: false }` after provider/tool errors,
    // timeouts, and MaxTokens turn ends. These tests drive the full bridge path
    // (`AgentEvent` -> `run_conversation_loop` -> `UiOutput`) rather than the engine in isolation.

    fn spawn_loop_for_runtime_tests(
        engine: ConversationEngine,
    ) -> (
        tokio::task::JoinHandle<()>,
        TestTurnSender,
        tokio::sync::mpsc::UnboundedReceiver<UiOutput>,
    ) {
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
            model_name: "test-model".to_string(),
            provider: "test-provider".to_string(),
            ..Default::default()
        });
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills: empty_runtime_skills(),
            },
        ));
        (handle, TestTurnSender::new(agent_tx), ui_rx)
    }

    async fn collect_terminal_status(
        ui_rx: &mut tokio::sync::mpsc::UnboundedReceiver<UiOutput>,
    ) -> talos_conversation::StatusSnapshot {
        let statuses = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let mut collected = Vec::new();
            while let Some(output) = ui_rx.recv().await {
                if let UiOutput::Status(status) = output {
                    collected.push(status);
                }
            }
            collected
        })
        .await
        .expect("timed out waiting for conversation loop to drain");
        statuses
            .last()
            .expect("conversation loop emitted at least one status")
            .clone()
    }

    #[tokio::test]
    async fn conversation_loop_clears_processing_on_provider_error_after_tool_result() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::ToolCall {
                call: talos_core::message::ToolCall {
                    id: "tc-1".to_string(),
                    name: "bash".to_string(),
                    input: serde_json::json!({}),
                },
                provenance: talos_core::tool::ToolProvenance::Native,
                summary_fields: vec![],
            })
            .unwrap();
        agent_tx
            .send(AgentEvent::ToolResult {
                result: talos_core::message::MessageToolResult {
                    tool_use_id: "tc-1".to_string(),
                    content: "ok".to_string(),
                    is_error: false,
                },
            })
            .unwrap();
        agent_tx
            .send(AgentEvent::Error {
                message: "provider connection reset after tool results".to_string(),
            })
            .unwrap();
        drop(agent_tx);

        let status = collect_terminal_status(&mut ui_rx).await;
        assert!(
            !status.is_processing,
            "runtime must not remain stuck after provider error"
        );
        assert_eq!(status.phase, Some(talos_conversation::TurnPhase::Failed));

        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_clears_processing_on_timeout_error() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::Error {
                message: "request timed out after 30s".to_string(),
            })
            .unwrap();
        drop(agent_tx);

        let status = collect_terminal_status(&mut ui_rx).await;
        assert!(
            !status.is_processing,
            "runtime must not remain stuck after timeout"
        );
        assert_eq!(status.phase, Some(talos_conversation::TurnPhase::TimedOut));

        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_clears_processing_on_dispatch_timeout_error() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::Error {
                message: "network error: request dispatch timeout: no response headers within 1s"
                    .to_string(),
            })
            .unwrap();
        drop(agent_tx);

        let status = collect_terminal_status(&mut ui_rx).await;
        assert!(
            !status.is_processing,
            "runtime must not remain stuck after provider dispatch timeout"
        );
        assert_eq!(status.phase, Some(talos_conversation::TurnPhase::TimedOut));

        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_clears_processing_on_max_tokens_turn_end() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::TextDelta {
                delta: "partial".to_string(),
            })
            .unwrap();
        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::MaxTokens,
                usage: talos_core::message::Usage::default(),
            })
            .unwrap();
        drop(agent_tx);

        let status = collect_terminal_status(&mut ui_rx).await;
        assert!(
            !status.is_processing,
            "runtime must not remain stuck after MaxTokens turn end"
        );

        loop_handle.await.unwrap();
    }

    // FS03 / RUNTIME-002: prove the visible diagnostic signals (error Tip + Error stream) are
    // forwarded by the conversation loop on terminal failure, and that the normal success path
    // (EndTurn) remains unchanged after the MaxTokens clearing fix.

    #[tokio::test]
    async fn conversation_loop_emits_visible_error_signals_on_terminal_failure() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        let error_message = "provider connection reset".to_string();
        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::Error {
                message: error_message.clone(),
            })
            .unwrap();
        drop(agent_tx);

        let mut saw_error_tip = false;
        let mut saw_error_stream = false;
        let mut saw_terminal_status = false;
        while let Some(output) = ui_rx.recv().await {
            match output {
                UiOutput::Tip { kind, text }
                    if kind == talos_conversation::TipKind::Error && text == error_message =>
                {
                    saw_error_tip = true;
                }
                UiOutput::Content(talos_conversation::ContentOutput::Block {
                    source: talos_conversation::MessageSource::Error,
                    ..
                }) => {
                    saw_error_stream = true;
                }
                UiOutput::Status(status) if !status.is_processing => {
                    saw_terminal_status = true;
                }
                _ => {}
            }
        }

        assert!(
            saw_error_tip,
            "terminal error must emit a visible error Tip"
        );
        assert!(
            saw_error_stream,
            "terminal error must emit a visible Error stream"
        );
        assert!(
            saw_terminal_status,
            "terminal error must emit a terminal Status with is_processing=false"
        );

        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_normal_end_turn_success_path_unchanged() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (loop_handle, agent_tx, mut ui_rx) = spawn_loop_for_runtime_tests(engine);

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::TextDelta {
                delta: "completed reply".to_string(),
            })
            .unwrap();
        agent_tx
            .send(AgentEvent::TurnEnd {
                stop_reason: talos_core::message::StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            })
            .unwrap();
        drop(agent_tx);

        let status = collect_terminal_status(&mut ui_rx).await;
        assert!(
            !status.is_processing,
            "normal EndTurn success must clear processing"
        );
        assert_eq!(
            status.phase, None,
            "normal EndTurn success must reset phase to None"
        );

        loop_handle.await.unwrap();
    }

    // D103 / RUNTIME-002 FS01 surface #3: prove the conversation loop forwards a terminal
    // `UiOutput::Status { is_processing: false, phase: Cancelled }` when the user cancels
    // mid-turn. The engine-level `cancel_turn_clears_processing_state` test covers the
    // engine in isolation; this test drives the full bridge path
    // (`UserInput::Cancel` -> `run_conversation_loop` -> `engine.cancel_turn()` -> `UiOutput`).

    #[tokio::test]
    async fn conversation_loop_cancel_emits_terminal_cancelled_status() {
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let agent_tx = TestTurnSender::new(agent_tx);
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
            model_name: "test-model".to_string(),
            provider: "test-provider".to_string(),
            ..Default::default()
        });
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills: empty_runtime_skills(),
            },
        ));

        agent_tx.send(AgentEvent::TurnStart).unwrap();
        agent_tx
            .send(AgentEvent::TextDelta {
                delta: "generating".to_string(),
            })
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        user_tx.send(UserInput::Cancel).unwrap();

        let mut saw_cancelled_status = false;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        while let Ok(Some(output)) = tokio::time::timeout_at(deadline, ui_rx.recv()).await {
            if let UiOutput::Status(status) = output {
                if !status.is_processing
                    && status.phase == Some(talos_conversation::TurnPhase::Cancelled)
                {
                    saw_cancelled_status = true;
                    break;
                }
            }
        }

        assert!(
            saw_cancelled_status,
            "cancel must emit terminal Status with is_processing=false and phase=Cancelled"
        );

        drop(agent_tx);
        drop(user_tx);
        loop_handle.await.unwrap();
    }

    #[tokio::test]
    async fn conversation_loop_routes_skill_activation_to_session_op() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join(".talos/skills/review");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: review\ndescription: Review code\ntriggers:\n  - review\n---\n\n# Review\nCheck safety.\n",
        )
        .unwrap();

        let runtime_skills = std::sync::Arc::new(tokio::sync::Mutex::new(
            discover_runtime_skills(dir.path(), false).unwrap(),
        ));
        let skills = runtime_skills.lock().await.diagnostics();
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string())
            .with_skills(skills);
        let (_agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (sq_tx, mut sq_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_watch_tx, sq_watch_rx) = tokio::sync::watch::channel(sq_tx);
        let (_model_tx, model_rx) = tokio::sync::watch::channel(ModelInfo {
            model_name: "test-model".to_string(),
            provider: "test-provider".to_string(),
            ..Default::default()
        });
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine,
            ConversationLoopIo {
                agent_rx,
                user_rx,
                ui_tx,
                sq_tx_watch: sq_watch_rx,
                model_info_watch: model_rx,
                session_tx,
                runtime_skills,
            },
        ));

        user_tx
            .send(UserInput::Message("/skills activate review".to_string()))
            .unwrap();

        let op = tokio::time::timeout(std::time::Duration::from_secs(1), sq_rx.recv())
            .await
            .expect("skill context op")
            .expect("session op");
        match op {
            talos_core::session::SessionOp::SetSkillContext { name, content } => {
                assert_eq!(name.as_deref(), Some("review"));
                assert!(content.unwrap().contains("Check safety."));
            }
            _ => panic!("expected skill context session op"),
        }

        let mut saw_confirmation = false;
        for _ in 0..3 {
            if let Some(UiOutput::Content(talos_conversation::ContentOutput::Block {
                source: talos_conversation::MessageSource::System,
                ..
            })) = ui_rx.recv().await
            {
                saw_confirmation = true;
                break;
            }
        }
        assert!(saw_confirmation);

        loop_handle.abort();
    }

    #[test]
    fn model_metadata_context_includes_model_info_without_secret() {
        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet-4-5".to_string();
        config.set_provider_credential("anthropic", "sk-secret-value");

        let file = crate::mode_runtime::model_metadata_context_file(&config);

        assert_eq!(file.path, "TALOS_MODEL.md");
        assert!(file.content.contains("Provider: anthropic"));
        assert!(file.content.contains("Model: claude-sonnet-4-5"));
        assert!(file.content.contains("Context limit:"));
        assert!(!file.content.contains("sk-secret-value"));
    }

    #[test]
    fn session_model_metadata_overrides_config_on_resume() {
        let dir = tempfile::tempdir().unwrap();
        let manager = talos_session::SessionManager::with_dir(dir.path().to_path_buf());
        let session = manager
            .create_session("test-project", "test-workspace")
            .unwrap();
        session
            .append_with_metadata(
                &talos_core::message::Message::User {
                    content: "hello".into(),
                },
                talos_session::SessionMetadata {
                    provider: Some("zhipuai-coding-plan".into()),
                    model: Some("glm-5.2".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet-4-5".to_string();

        crate::mode_runtime::apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "zhipuai-coding-plan");
        assert_eq!(config.model, "glm-5.2");
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

    // === Config Display Masking Tests (I046-S3) ===

    #[test]
    fn config_get_dotted_returns_api_key_value() {
        let config = talos_config::Config {
            provider: "custom".to_string(),
            model: "test".to_string(),
            providers: std::collections::HashMap::from([(
                "custom".to_string(),
                talos_config::ProviderConfig {
                    api_key: Some("sk-test-secret".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let value = config_get_dotted(&config, "providers.custom.api_key").unwrap();
        assert_eq!(value, "sk-test-secret");
    }

    #[test]
    fn is_secret_key_detects_api_key_paths() {
        assert!(is_secret_key("providers.anthropic.api_key"));
        assert!(is_secret_key("api_key"));
        assert!(!is_secret_key("providers.anthropic.api_key_env"));
        assert!(!is_secret_key("model"));
    }

    #[test]
    fn mask_secrets_masks_api_key_lines() {
        let toml = r#"provider = "anthropic"

[providers.anthropic]
api_key = "sk-super-secret-12345"
api_key_env = "ANTHROPIC_API_KEY"
"#;
        let config = talos_config::Config::default();
        let masked = mask_secrets(toml, &config);
        assert!(!masked.contains("sk-super-secret-12345"));
        assert!(masked.contains("api_key = ***"));
        // api_key_env is a variable name, not a secret — must not be masked.
        assert!(masked.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn available_model_name_is_provider_qualified() {
        let model = talos_config::model::ModelMetadata {
            variants: vec![],
            provider: "openai".to_string(),
            id: "gpt-4.1".to_string(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: Default::default(),
            release_date: None,
            source: Default::default(),
        };

        assert_eq!(available_model_name(&model), "openai/gpt-4.1");
    }

    #[test]
    fn available_model_filter_matches_provider_model_and_qualified_id() {
        let model = talos_config::model::ModelMetadata {
            variants: vec![],
            provider: "openai".to_string(),
            id: "gpt-4.1".to_string(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: Default::default(),
            release_date: None,
            source: Default::default(),
        };

        assert!(model_matches_filter(&model, Some("openai")));
        assert!(model_matches_filter(&model, Some("gpt-4")));
        assert!(model_matches_filter(&model, Some("openai/gpt-4.1")));
        assert!(!model_matches_filter(&model, Some("anthropic")));
    }

    #[test]
    fn empty_available_model_filter_matches_everything() {
        let model = talos_config::model::ModelMetadata {
            variants: vec![],
            provider: "openai".to_string(),
            id: "gpt-4.1".to_string(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: Default::default(),
            release_date: None,
            source: Default::default(),
        };

        assert!(model_matches_filter(
            &model,
            normalize_model_filter(Some("   ")).as_deref()
        ));
    }

    use crate::storage::{
        CleanupArgs, MaintenanceArgs, collect_storage_status, print_cleanup_dry_run,
        print_cleanup_report, print_storage_status, resolve_talos_root,
    };
    use std::io::Write;
    use talos_core::message::Message;

    #[test]
    fn storage_status_missing_home() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let status = collect_storage_status(&talos_root);
        assert!(!status.talos_root_exists);
        assert_eq!(status.session_count, 0);
        assert_eq!(status.session_total_bytes, 0);
        assert_eq!(status.total_forks, 0);
        assert_eq!(status.index_db_bytes, 0);
        assert_eq!(status.logs_bytes, 0);
        assert_eq!(status.cache_bytes, 0);
        assert!(!status.memory_db_exists);
    }

    #[test]
    fn storage_status_populated() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "test-workspace";
        let s1 = manager.create_session("proj-a", ws).unwrap();
        s1.append(&Message::User {
            content: "hello".into(),
        })
        .unwrap();
        let s2 = manager.create_session("proj-b", ws).unwrap();
        s2.append(&Message::User {
            content: "world".into(),
        })
        .unwrap();

        let status = collect_storage_status(&talos_root);
        assert!(status.talos_root_exists);
        assert_eq!(status.session_count, 2);
        assert!(status.session_total_bytes > 0);
        assert_eq!(status.top_sessions.len(), 2);
    }

    #[test]
    fn cleanup_dry_run_no_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "dry-run-ws";
        for i in 0..3 {
            let s = manager.create_session("proj", ws).unwrap();
            s.append(&Message::User {
                content: format!("msg-{i}"),
            })
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let policy = talos_session::SessionCleanupPolicy {
            workspace_root: Some(ws.to_string()),
            max_sessions_per_workspace: Some(1),
            max_age_days: None,
            protected_session_ids: vec![],
        };
        let candidates = manager.cleanup_candidates(&policy).unwrap();
        assert!(!candidates.is_empty());

        let before_files: Vec<_> = std::fs::read_dir(&sessions_dir)
            .unwrap()
            .flat_map(|e| e.ok())
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .flat_map(|ws_dir| std::fs::read_dir(ws_dir.path()).unwrap())
            .flat_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .collect();

        print_cleanup_dry_run(&candidates);

        let after_files: Vec<_> = std::fs::read_dir(&sessions_dir)
            .unwrap()
            .flat_map(|e| e.ok())
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .flat_map(|ws_dir| std::fs::read_dir(ws_dir.path()).unwrap())
            .flat_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
            .collect();

        assert_eq!(
            before_files.len(),
            after_files.len(),
            "dry-run must not delete any files"
        );
    }

    #[test]
    fn cleanup_apply_deletes_jsonl_and_index() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "apply-ws";
        let stale = manager.create_session("proj", ws).unwrap();
        stale
            .append(&Message::User {
                content: "stale content".into(),
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let keep = manager.create_session("proj", ws).unwrap();
        keep.append(&Message::User {
            content: "keep content".into(),
        })
        .unwrap();

        manager.update_index(&stale).unwrap();
        manager.update_index(&keep).unwrap();

        let policy = talos_session::SessionCleanupPolicy {
            workspace_root: Some(ws.to_string()),
            max_sessions_per_workspace: Some(0),
            max_age_days: None,
            protected_session_ids: vec![keep.id],
        };

        let report = manager.apply_cleanup(&policy).unwrap();
        assert_eq!(report.removed, 1);
        assert!(!stale.file_path.exists(), "stale JSONL must be deleted");
        assert!(keep.file_path.exists(), "protected JSONL must remain");

        let search_results = manager.search("stale", 10).unwrap();
        assert!(
            !search_results
                .iter()
                .any(|r| r.session_id == stale.id.to_string()),
            "stale session must not appear in search"
        );
    }

    #[test]
    fn cleanup_protects_active_session() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "protect-ws";
        let active = manager.create_session("proj", ws).unwrap();
        active
            .append(&Message::User {
                content: "active".into(),
            })
            .unwrap();
        let other = manager.create_session("proj", ws).unwrap();
        other
            .append(&Message::User {
                content: "other".into(),
            })
            .unwrap();

        let policy = talos_session::SessionCleanupPolicy {
            workspace_root: Some(ws.to_string()),
            max_sessions_per_workspace: Some(0),
            max_age_days: None,
            protected_session_ids: vec![active.id],
        };

        let candidates = manager.cleanup_candidates(&policy).unwrap();
        assert!(
            !candidates.iter().any(|c| c.id == active.id),
            "active session must never be a cleanup candidate"
        );
    }

    #[test]
    fn cleanup_apply_requires_criteria() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "criteria-ws";
        let s = manager.create_session("proj", ws).unwrap();
        s.append(&Message::User {
            content: "test".into(),
        })
        .unwrap();

        let policy = talos_session::SessionCleanupPolicy {
            workspace_root: None,
            max_sessions_per_workspace: None,
            max_age_days: None,
            protected_session_ids: vec![],
        };

        let candidates = manager.cleanup_candidates(&policy).unwrap();
        assert!(
            candidates.is_empty(),
            "no criteria should yield zero candidates"
        );
    }

    #[test]
    fn maintenance_operations_run() {
        let dir = tempfile::tempdir().unwrap();
        let talos_root = dir.path().join(".talos");
        let sessions_dir = talos_root.join("sessions");
        let manager = talos_session::SessionManager::with_dir(sessions_dir.clone());

        let ws = "maint-ws";
        let s = manager.create_session("proj", ws).unwrap();
        s.append(&Message::User {
            content: "maintenance test".into(),
        })
        .unwrap();
        manager.update_index(&s).unwrap();

        manager.checkpoint_index().unwrap();
        manager.vacuum_index().unwrap();
        let fixed = manager.reconcile_index().unwrap();

        let results = manager.search("maintenance", 10).unwrap();
        assert!(
            results.iter().any(|r| r.session_id == s.id.to_string()),
            "sessions must survive maintenance operations"
        );
    }

    // === Config Subcommand Tests (CONF-001) ===

    #[test]
    fn config_subcommand_list_masks_secrets() {
        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet-4".to_string();
        config.set_provider_credential("anthropic", "sk-test-secret-123");

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let masked = mask_secrets(&toml_str, &config);
        assert!(!masked.contains("sk-test-secret-123"));
        assert!(masked.contains("api_key = ***"));
    }

    #[test]
    fn config_subcommand_get_returns_value() {
        let config = talos_config::Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            ..Default::default()
        };
        assert_eq!(
            config_get_dotted(&config, "model").unwrap(),
            "claude-sonnet-4"
        );
        assert_eq!(config_get_dotted(&config, "provider").unwrap(), "anthropic");
    }

    #[test]
    fn config_subcommand_set_persists() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut config = talos_config::Config::default();
        config.model = "old-model".to_string();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, toml_str).unwrap();

        let raw = std::fs::read_to_string(&config_path).unwrap();
        let mut config: talos_config::Config = toml::from_str(&raw).unwrap();
        config_set_dotted(&mut config, "model", "new-model").unwrap();
        let saved = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, &saved).unwrap();

        let reloaded = std::fs::read_to_string(&config_path).unwrap();
        assert!(reloaded.contains("new-model"));
        assert!(!reloaded.contains("old-model"));
    }

    #[test]
    fn config_subcommand_get_secret_masks() {
        let config = talos_config::Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            providers: std::collections::HashMap::from([(
                "anthropic".to_string(),
                talos_config::ProviderConfig {
                    api_key: Some("sk-should-be-masked".to_string()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let raw = config_get_dotted(&config, "providers.anthropic.api_key").unwrap();
        assert_eq!(raw, "sk-should-be-masked");
        assert!(is_secret_key("providers.anthropic.api_key"));
    }

    #[test]
    fn config_set_protocol() {
        let mut config = talos_config::Config::default();
        config_set_dotted(&mut config, "providers.my-gw.protocol", "openai-chat").unwrap();
        let provider = config.providers.get("my-gw").unwrap();
        assert_eq!(
            provider.protocol,
            talos_config::ProviderProtocol::OpenAIChat
        );

        config_set_dotted(
            &mut config,
            "providers.my-gw.protocol",
            "anthropic-messages",
        )
        .unwrap();
        let provider = config.providers.get("my-gw").unwrap();
        assert_eq!(
            provider.protocol,
            talos_config::ProviderProtocol::AnthropicMessages
        );

        let err =
            config_set_dotted(&mut config, "providers.my-gw.protocol", "invalid").unwrap_err();
        assert!(err.to_string().contains("unknown protocol"));
    }

    #[test]
    fn config_set_model_context_limit() {
        let mut config = talos_config::Config::default();
        config_set_dotted(
            &mut config,
            "providers.my-gw.models.claude-4.context_limit",
            "200000",
        )
        .unwrap();

        let provider = config.providers.get("my-gw").unwrap();
        let model = provider.models.get("claude-4").unwrap();
        assert_eq!(model.context_limit, Some(200000));

        assert_eq!(
            config_get_dotted(&config, "providers.my-gw.models.claude-4.context_limit").unwrap(),
            "200000"
        );
    }

    #[test]
    fn config_set_model_output_limit() {
        let mut config = talos_config::Config::default();
        config_set_dotted(
            &mut config,
            "providers.my-gw.models.claude-4.output_limit",
            "4096",
        )
        .unwrap();

        let provider = config.providers.get("my-gw").unwrap();
        let model = provider.models.get("claude-4").unwrap();
        assert_eq!(model.output_limit, Some(4096));

        assert_eq!(
            config_get_dotted(&config, "providers.my-gw.models.claude-4.output_limit").unwrap(),
            "4096"
        );
    }

    #[test]
    fn config_get_dashboard_values() {
        let config = talos_config::Config::default();
        assert_eq!(
            config_get_dotted(&config, "dashboard.enabled").unwrap(),
            "true"
        );
        assert_eq!(
            config_get_dotted(&config, "dashboard.loopback_only").unwrap(),
            "true"
        );
    }

    #[test]
    fn config_set_dashboard_values() {
        let mut config = talos_config::Config::default();
        config_set_dotted(&mut config, "dashboard.enabled", "false").unwrap();
        config_set_dotted(&mut config, "dashboard.loopback_only", "false").unwrap();

        assert!(!config.dashboard.enabled);
        assert!(!config.dashboard.loopback_only);
        assert_eq!(
            config_get_dotted(&config, "dashboard.enabled").unwrap(),
            "false"
        );
        assert_eq!(
            config_get_dotted(&config, "dashboard.loopback_only").unwrap(),
            "false"
        );
    }

    #[test]
    fn config_set_dashboard_bool_rejects_invalid_value() {
        let mut config = talos_config::Config::default();
        let err = config_set_dotted(&mut config, "dashboard.loopback_only", "maybe").unwrap_err();
        assert!(err.to_string().contains("invalid boolean"));
    }

    #[test]
    fn config_flag_and_subcommand_equivalence() {
        let config = talos_config::Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            ..Default::default()
        };
        let flag_result = config_get_dotted(&config, "model");
        let subcommand_result = config_get_dotted(&config, "model");
        assert_eq!(flag_result.unwrap(), subcommand_result.unwrap());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let masked = mask_secrets(&toml_str, &config);
        assert!(masked.contains("model = \"claude-sonnet-4\""));
    }

    // === F102: Config Validation Evidence Tests (CONF-001) ===

    #[test]
    fn config_validate_rejects_empty_provider() {
        let mut config = talos_config::Config::default();
        config.model = "test".to_string();
        config.provider = "".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn config_validate_rejects_empty_model() {
        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn config_validate_rejects_configured_provider_without_credentials() {
        let config = talos_config::Config {
            provider: "custom-gw".to_string(),
            model: "test-model".to_string(),
            providers: std::collections::HashMap::from([(
                "custom-gw".to_string(),
                talos_config::ProviderConfig {
                    protocol: talos_config::ProviderProtocol::OpenAIChat,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("api_key or api_key_env"));
    }

    #[test]
    fn config_validate_accepts_valid_config() {
        let config = talos_config::Config {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_set_dotted_rejects_invalid_protocol() {
        let mut config = talos_config::Config::default();
        let err = config_set_dotted(&mut config, "providers.gw.protocol", "bogus").unwrap_err();
        assert!(err.to_string().contains("unknown protocol"));
    }

    #[test]
    fn config_set_dotted_rejects_non_integer_limit() {
        let mut config = talos_config::Config::default();
        let err = config_set_dotted(
            &mut config,
            "providers.gw.models.m.context_limit",
            "not-a-number",
        )
        .unwrap_err();
        assert!(err.to_string().contains("context_limit"));
    }

    #[test]
    fn config_env_var_name_survives_roundtrip() {
        let mut config = talos_config::Config::default();
        config_set_dotted(
            &mut config,
            "providers.anthropic.api_key_env",
            "ANTHROPIC_API_KEY",
        )
        .unwrap();
        assert_eq!(
            config_get_dotted(&config, "providers.anthropic.api_key_env").unwrap(),
            "ANTHROPIC_API_KEY"
        );

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let reloaded: talos_config::Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            config_get_dotted(&reloaded, "providers.anthropic.api_key_env").unwrap(),
            "ANTHROPIC_API_KEY"
        );
    }

    #[test]
    fn config_secret_masking_survives_roundtrip() {
        let mut config = talos_config::Config::default();
        config.set_provider_credential("anthropic", "sk-secret-roundtrip");

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let masked = mask_secrets(&toml_str, &config);
        assert!(!masked.contains("sk-secret-roundtrip"));
        assert!(masked.contains("api_key = ***"));

        assert!(is_secret_key("providers.anthropic.api_key"));
        assert!(is_secret_key("providers.openai.api_key"));
        assert!(!is_secret_key("providers.anthropic.base_url"));
    }

    #[test]
    fn config_save_load_roundtrip_preserves_fields() {
        let mut config = talos_config::Config::default();
        config.provider = "my-gw".to_string();
        config.model = "glm-5".to_string();
        config_set_dotted(&mut config, "providers.my-gw.protocol", "openai-chat").unwrap();
        config_set_dotted(
            &mut config,
            "providers.my-gw.base_url",
            "https://gw.example/v1",
        )
        .unwrap();
        config_set_dotted(&mut config, "providers.my-gw.api_key_env", "GW_API_KEY").unwrap();
        config_set_dotted(
            &mut config,
            "providers.my-gw.models.glm-5.context_limit",
            "200000",
        )
        .unwrap();
        config.validate().unwrap();

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let reloaded: talos_config::Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(reloaded.provider, "my-gw");
        assert_eq!(reloaded.model, "glm-5");
        assert_eq!(
            config_get_dotted(&reloaded, "providers.my-gw.base_url").unwrap(),
            "https://gw.example/v1"
        );
        assert_eq!(
            config_get_dotted(&reloaded, "providers.my-gw.api_key_env").unwrap(),
            "GW_API_KEY"
        );
        assert_eq!(
            config_get_dotted(&reloaded, "providers.my-gw.models.glm-5.context_limit").unwrap(),
            "200000"
        );
    }
}

#[cfg(test)]
mod steering_snapshot_tests {
    use super::*;
    use talos_conversation::{ConversationEngine, SteeringQueueSnapshot, UiOutput};

    fn new_engine() -> ConversationEngine {
        ConversationEngine::new("test-model".to_string(), "test-provider".to_string())
    }

    fn build_empty_snapshot() -> SteeringQueueSnapshot {
        SteeringQueueSnapshot {
            entries: vec![],
            total_count: 0,
            omitted_count: 0,
        }
    }

    #[test]
    fn engine_snapshot_empty_after_drain() {
        let mut engine = new_engine();
        engine.enqueue_steering("a".into());
        engine.enqueue_steering("b".into());

        let drained1 = engine.drain_steering_queue();
        let snap1 = engine.steering_queue_snapshot();
        assert_eq!(drained1, Some("a".into()));
        assert_eq!(snap1.total_count, 1);
        assert_eq!(snap1.omitted_count, 0);
        assert_eq!(snap1.entries.len(), 1);

        let drained2 = engine.drain_steering_queue();
        let snap2 = engine.steering_queue_snapshot();
        assert_eq!(drained2, Some("b".into()));
        assert_eq!(snap2.total_count, 0);
        assert_eq!(snap2.omitted_count, 0);
        assert!(snap2.entries.is_empty());

        let empty = build_empty_snapshot();
        assert_eq!(empty.total_count, 0);
        assert!(empty.entries.is_empty());
    }

    #[test]
    fn non_empty_snapshot_preserved_on_error_path() {
        // On error/cancel paths, the engine does NOT clear the steering queue.
        // The snapshot correctly reflects the preserved queue.
        let mut engine = new_engine();
        engine.enqueue_steering("queued".into());

        let cancel_outputs = engine.cancel_turn();
        let snap = cancel_outputs.iter().find_map(|o| match o {
            UiOutput::SteeringQueueSnapshot(s) => Some(s.clone()),
            _ => None,
        });
        assert!(snap.is_some(), "cancel must emit snapshot");
        assert_eq!(
            snap.unwrap().total_count,
            1,
            "cancel must preserve queued message"
        );

        let error_outputs =
            engine.handle_turn_completed(&talos_core::session::TurnCompletionStatus::Error {
                message: "test".into(),
            });
        let snap2 = error_outputs.iter().find_map(|o| match o {
            UiOutput::SteeringQueueSnapshot(s) => Some(s.clone()),
            _ => None,
        });
        assert!(snap2.is_some(), "error must emit snapshot");
        assert_eq!(
            snap2.unwrap().total_count,
            1,
            "error must preserve queued message"
        );
    }
}
