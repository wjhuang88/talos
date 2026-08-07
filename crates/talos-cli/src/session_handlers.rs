//! Session and provider handler functions.

#[path = "session_handlers/lifecycle.rs"]
mod lifecycle;
#[path = "session_handlers/provider_model.rs"]
mod provider_model;

#[cfg(test)]
use lifecycle::{emit_session_identity_after_queue_clear, rollback_owned_session_message};
pub(crate) use lifecycle::{
    handle_session_delete, handle_session_fork, handle_session_new, handle_session_resume,
};
#[cfg(test)]
pub(crate) use provider_model::build_connect_picker_data;
#[cfg(test)]
use provider_model::provider_qualified_model_reference;
pub(crate) use provider_model::{
    handle_connect, handle_connect_with_credential, handle_provider_setup,
    handle_register_custom_provider, handle_session_model, handle_session_model_with_credential,
};

/// Maximum discovered-model entries to persist into the provider's
/// `models` map during a single registration. Caps config growth when
/// a provider returns hundreds of model IDs.
pub(crate) const MAX_DISCOVERED_MODELS_TO_PERSIST: usize = 32;

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn rollback_diagnostic_preserves_primary_and_cleanup_failures() {
        let dir = tempfile::tempdir().expect("temporary rollback test directory");
        let manager = talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let child = manager
            .defer_create_session("talos", "/workspace")
            .expect("defer child Session");
        std::fs::create_dir_all(
            child
                .file_path
                .parent()
                .expect("child transcript has a parent"),
        )
        .expect("create child directory");
        std::fs::write(&child.file_path, b"non-sensitive transcript fixture")
            .expect("write child transcript");
        let sqlite = child
            .file_path
            .with_file_name(format!("{}.pending.sqlite", child.id));
        std::fs::create_dir(&sqlite).expect("create blocked SQLite target");
        std::fs::write(sqlite.join("held"), b"held").expect("make target non-empty");

        let message = rollback_owned_session_message(
            &manager,
            &child,
            "Failed to construct fork runtime",
            "provider build failed",
        );
        assert!(message.contains("provider build failed"));
        assert!(message.contains("Cleanup also failed"));
        assert!(message.contains(&child.id.to_string()));
        assert!(message.contains(&sqlite.display().to_string()));
        assert!(message.contains("retryable"));
        assert!(child.file_path.exists());
        assert!(!message.contains("non-sensitive transcript fixture"));

        std::fs::remove_file(sqlite.join("held")).expect("release blocked target");
        std::fs::remove_dir(&sqlite).expect("remove blocked target");
        manager
            .rollback_session_artifacts(&child)
            .expect("retry cleanup succeeds");
    }

    #[test]
    fn rollback_diagnostic_reports_complete_success_without_content() {
        let dir = tempfile::tempdir().expect("temporary rollback test directory");
        let manager = talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let child = manager
            .create_session("talos", "/workspace")
            .expect("create child Session");
        talos_session::PendingSubmissionStore::for_session(&child)
            .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                "provider", "model", None,
            ))
            .expect("initialize child identity");

        let message = rollback_owned_session_message(
            &manager,
            &child,
            "Failed to commit fork",
            "durable fence failed",
        );
        assert!(message.contains("durable fence failed"));
        assert!(message.contains("Rolled back Session"));
        assert!(message.contains(&child.id.to_string()));
        assert!(!child.file_path.exists());
        assert!(!message.contains("submission"));
    }

    #[test]
    fn committed_session_boundary_clears_preview_before_identity() {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::unbounded_channel();
        emit_session_identity_after_queue_clear(&ui_tx, "new-session".to_string());

        assert!(matches!(
            ui_rx.try_recv().expect("queue-clear output"),
            UiOutput::SteeringQueueSnapshot(talos_conversation::SteeringQueueSnapshot {
                entries,
                total_count: 0,
                omitted_count: 0,
            }) if entries.is_empty()
        ));
        assert!(matches!(
            ui_rx.try_recv().expect("session identity output"),
            UiOutput::SessionIdentity { id } if id == "new-session"
        ));
        assert!(
            ui_rx.try_recv().is_err(),
            "boundary helper emits only two outputs"
        );
    }

    #[tokio::test]
    async fn normalized_default_model_selection_is_a_true_noop() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let session_manager = talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let session = session_manager
            .create_session("project", "")
            .expect("operation should succeed");
        let (raw_sq_tx, _raw_sq_rx) = mpsc::channel(4);
        let transition = Arc::new(Mutex::new(
            SessionTransition::new(raw_sq_tx.clone(), session.clone())
                .expect("operation should succeed"),
        ));
        let generation_before = transition.lock().await.active_generation();

        let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
        let (session_watch_tx, session_watch_rx) = watch::channel(session.clone());
        let (sq_tx_watch_tx, _sq_tx_watch_rx) = watch::channel(raw_sq_tx);
        let (bridge_rx_update_tx, _bridge_rx_update_rx) = mpsc::unbounded_channel();
        let hooks = build_hook_registry(true);
        let approval_handler = Arc::new(TuiApprovalHandler::new(
            ui_tx.clone(),
            dir.path().to_path_buf(),
        ));
        let runtime_builder = TuiRuntimeBuilder::new(
            hooks,
            dir.path().to_path_buf(),
            session_manager.clone(),
            approval_handler,
            Vec::new(),
            false,
            true,
        );

        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut config, None);

        let result = handle_session_model(
            &transition,
            &ui_tx,
            &config,
            &runtime_builder,
            &session_watch_tx,
            &sq_tx_watch_tx,
            &bridge_rx_update_tx,
            &session_watch_rx,
            "o3@DEFAULT".to_string(),
            Some("openai".to_string()),
        )
        .await;

        assert!(result.is_none());
        assert_eq!(
            transition.lock().await.active_generation(),
            generation_before,
            "equivalent baseline selection must not fence or replace the Actor"
        );
        assert!(
            session
                .read_entries()
                .expect("operation should succeed")
                .iter()
                .all(|entry| {
                    crate::mode_runtime::session_model_activation_from_metadata(&entry.metadata)
                        .is_none()
                }),
            "equivalent baseline selection must not append an activation record"
        );
        assert!(
            ui_rx.try_recv().is_err(),
            "a true no-op must not publish status or error output"
        );
    }

    #[test]
    fn credential_first_duplicate_model_id_stays_provider_qualified() {
        use talos_config::ProviderConfig;

        let qualified = provider_qualified_model_reference("zhipuai", "glm-5.2");
        assert_eq!(qualified, "zhipuai/glm-5.2");
        assert_eq!(
            provider_qualified_model_reference("zhipuai", &qualified),
            qualified
        );

        let mut live = Config::default();
        live.providers.insert(
            "zai".to_string(),
            ProviderConfig {
                api_key: Some("credential-a".to_string()),
                base_url: Some("https://provider-a.invalid/v1".to_string()),
                ..Default::default()
            },
        );
        live.providers.insert(
            "zhipuai".to_string(),
            ProviderConfig {
                api_key: Some("credential-b".to_string()),
                base_url: Some("https://provider-b.invalid/v1".to_string()),
                ..Default::default()
            },
        );
        live.set_active_model(&qualified)
            .expect("credential-selected duplicate must resolve to Provider B");

        let mut persisted = live.clone();
        persisted
            .set_active_model(&qualified)
            .expect("ConfigStore update uses the same provider-qualified reference");

        for config in [&live, &persisted] {
            assert_eq!(config.provider, "zhipuai");
            assert_eq!(config.model, "glm-5.2");
            assert_eq!(
                config.api_key().expect("operation should succeed"),
                "credential-b"
            );
            assert_eq!(
                config.base_url().as_deref(),
                Some("https://provider-b.invalid/v1")
            );
        }
    }
}

#[cfg(test)]
mod cleanup_recovery_command_tests {
    use super::*;
    use crate::session_setup::canonical_workspace_root;
    use clap::Parser;

    #[test]
    fn storage_reconcile_instruction_parses_on_the_real_cli_surface() {
        let cli = crate::Cli::try_parse_from(["talos", "storage", "maintenance", "--reconcile"])
            .expect("emitted storage maintenance instruction must parse");

        match cli.command {
            Some(crate::TalosCommand::Storage {
                command:
                    crate::storage::StorageCommand::Maintenance {
                        checkpoint,
                        vacuum,
                        reconcile,
                    },
            }) => {
                assert!(!checkpoint);
                assert!(!vacuum);
                assert!(reconcile);
            }
            _ => panic!("emitted instruction must select storage maintenance reconcile"),
        }
    }

    #[tokio::test]
    async fn cleanup_failure_instructions_parse_and_execute() {
        let temp = tempfile::tempdir().expect("create cleanup recovery fixture");
        let sessions_dir = temp.path().join("sessions");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        let workspace_root = canonical_workspace_root(&workspace);
        let manager = talos_session::SessionManager::with_dir(sessions_dir);
        let active = manager
            .create_session("active", &workspace_root)
            .expect("create active Session");
        let target = manager
            .create_session("target", &workspace_root)
            .expect("create target Session");

        let sidecar = target
            .file_path
            .with_file_name(format!("{}.pending.sqlite", target.id));
        std::fs::create_dir(&sidecar).expect("create blocked sidecar path");
        std::fs::write(sidecar.join("held"), b"held").expect("hold sidecar path");

        let diagnostic = rollback_owned_session_message(
            &manager,
            &target,
            "forced runtime construction failure",
            "primary failure",
        );
        assert!(diagnostic.contains(&format!("/delete {}", target.id)));
        assert!(diagnostic.contains("talos storage maintenance --reconcile"));
        assert!(!diagnostic.contains("--storage-maintenance"));
        assert!(target.file_path.exists());

        std::fs::remove_file(sidecar.join("held")).expect("release sidecar path");
        std::fs::remove_dir(&sidecar).expect("remove blocked sidecar directory");

        let (ui_tx, _ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_active_tx, active_rx) = tokio::sync::watch::channel(active.clone());
        handle_session_delete(
            &ui_tx,
            &workspace,
            &manager,
            &active_rx,
            Some(target.id.to_string()),
        )
        .await;

        assert!(manager.get_session(&target.id).is_err());
        assert!(manager.get_session(&active.id).is_ok());
    }
}
