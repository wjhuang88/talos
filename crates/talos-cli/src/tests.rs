#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;
    use crate::colors;
    use crate::provider_setup::parse_provider;
    use crate::registry;
    use crate::tui_bridge::{SessionLifecycleRequest, run_conversation_loop};
    use crate::{config_get_dotted, is_secret_key, mask_secrets};
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
        let engine = ConversationEngine::new("test-model".to_string(), "test-provider".to_string());
        let (agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (submit_tx, mut submit_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (_model_tx, model_rx) =
            tokio::sync::watch::channel(("test-model".to_string(), "test-provider".to_string()));
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine, agent_rx, user_rx, ui_tx, submit_tx, sq_rx, model_rx, session_tx,
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

    #[tokio::test]
    async fn conversation_loop_updates_status_from_model_watch() {
        let engine = ConversationEngine::new("old-model".to_string(), "old-provider".to_string());
        let (_agent_tx, agent_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_user_tx, user_rx) = tokio::sync::mpsc::unbounded_channel();
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (submit_tx, _submit_rx) = tokio::sync::mpsc::unbounded_channel();
        let (interrupt_tx, _interrupt_rx) = tokio::sync::mpsc::channel(4);
        let (_sq_tx, sq_rx) = tokio::sync::watch::channel(interrupt_tx);
        let (model_tx, model_rx) =
            tokio::sync::watch::channel(("old-model".to_string(), "old-provider".to_string()));
        let (session_tx, _session_rx) =
            tokio::sync::mpsc::unbounded_channel::<SessionLifecycleRequest>();

        let loop_handle = tokio::spawn(run_conversation_loop(
            engine, agent_rx, user_rx, ui_tx, submit_tx, sq_rx, model_rx, session_tx,
        ));

        model_tx
            .send(("new-model".to_string(), "new-provider".to_string()))
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

    #[test]
    fn model_metadata_context_includes_model_info_without_secret() {
        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet-4-5-20250929".to_string();
        config.set_provider_credential("anthropic", "sk-secret-value");

        let file = crate::mode_runners::model_metadata_context_file(&config);

        assert_eq!(file.path, "TALOS_MODEL.md");
        assert!(file.content.contains("Provider: anthropic"));
        assert!(file.content.contains("Model: claude-sonnet-4-5-20250929"));
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
                    provider: Some("zhipu-coding-plan".into()),
                    model: Some("glm-5.2".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let mut config = talos_config::Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet-4-5-20250929".to_string();

        crate::mode_runners::apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "zhipu-coding-plan");
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
}
