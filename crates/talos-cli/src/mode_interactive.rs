//! Interactive REPL mode.

use super::*;
use crate::approval::terminal_approval_channel;
use talos_agent::auto_resolver::{
    AutoPermissionControl, AutoPermissionResolver, ManagedWorkspaceLease,
    ProviderAutoPermissionAssessor,
};
use talos_core::tool::SharedAtomicCreateCapability;
use talos_tools::CapStdAtomicCreateCapability;

fn register_interactive_builtin_contributions(
    registry: &mut ToolRegistry,
    _approval: Arc<std::sync::Mutex<ApprovalPrompt>>,
    workspace_root: &Path,
) -> Result<Option<SharedAtomicCreateCapability>> {
    let bash = bash_tool_contribution(workspace_root.to_path_buf());
    registry.register_builtin_contributions([bash])?;

    let capability = CapStdAtomicCreateCapability::open(workspace_root)
        .ok()
        .map(|value| Arc::new(value) as SharedAtomicCreateCapability);
    for contribution in talos_tools::snapshot_aware_file_tool_contributions_with_capability(
        workspace_root.to_path_buf(),
        capability.clone(),
    ) {
        registry.register_builtin_contributions([contribution])?;
    }

    for contribution in workspace_non_document_tool_contributions(workspace_root.to_path_buf()) {
        registry.register_builtin_contributions([contribution])?;
    }

    for contribution in git_read_tool_contributions(workspace_root.to_path_buf()) {
        registry.register_builtin_contributions([contribution])?;
    }
    for contribution in git_mutation_tool_contributions(workspace_root.to_path_buf()) {
        registry.register_builtin_contributions([contribution])?;
    }

    Ok(capability)
}

pub(crate) async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let workspace_root = resolve_workspace_root(&cli)?;

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let mut session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Prompt,
        true,
    )?;

    let mut config = Config::load().context("failed to load configuration")?;

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    let mut event_loop: Option<event_loop::EventLoop> = None;
    loop {
        let outcome: Result<Option<talos_session::Session>> = async {
            let runtime_lease = talos_session::SessionRuntimeLease::acquire(&session.file_path)
                .context("cannot acquire interactive session runtime ownership")?;
            prepare_interactive_session_identity(&mut config, &session)?;
            if config.model.is_empty() && !cli.mock {
                bail!(
                    "no model configured. Set 'model' in ~/.talos/config.toml, pass --model, or run `talos` in TUI mode for the setup wizard."
                );
            }

            let api_key = if cli.mock {
                config.api_key().unwrap_or_default()
            } else {
                config.api_key().map_err(|e| anyhow!("{e}"))?
            };

            let approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
                crate::registry::product_permission_engine(workspace_root.clone()),
            )));

            let mut registry = ToolRegistry::new();
            let sched_pending = crate::registry::register_builtin_scheduler(&mut registry)?;
            let atomic_create_capability = register_interactive_builtin_contributions(
                &mut registry,
                approval.clone(),
                &workspace_root,
            )?;

            let hooks = build_hook_registry(true);
            apply_mcp_fixture_config(&mut config, &cli);
            let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
            mcp_runtime.report_startup_failures();
            register_permission_aware_tools(
                &mut registry,
                mcp_runtime.tools(),
                approval.clone(),
                false,
            );
            let loaded_plugin_packages = register_explicit_permission_aware_plugins(
                &mut registry,
                &cli.plugin_packages,
                approval.clone(),
                false,
            )
            .map_err(anyhow::Error::msg)?;

            let permission_state = approval
                .lock()
                .map_err(|_| anyhow!("approval lock poisoned"))?
                .session_state();
            crate::registry::bind_session_worktrees(
                &mut registry,
                &workspace_root,
                session.id,
                session_manager.sessions_dir(),
                permission_state.clone(),
            )
            .map_err(anyhow::Error::msg)?;
            let (terminal_approval, terminal_approval_rx) = terminal_approval_channel();
            let provider = build_provider(&config, &api_key, cli.mock)?;
            let fallback = terminal_approval.clone();
            let auto_control = AutoPermissionControl::new(config.auto.enabled);
            let resolver: Arc<dyn talos_agent::permission_pipeline::ApprovalResolver> =
                ManagedWorkspaceLease::for_permission_session(
                    &workspace_root,
                    permission_state.clone(),
                )
                .ok()
                .map(|lease| {
                    let lease = atomic_create_capability
                        .clone()
                        .map_or(lease.clone(), |capability| {
                            lease.with_atomic_create_capability(capability)
                        });
                    Arc::new(
                        AutoPermissionResolver::new(
                            Arc::new(ProviderAutoPermissionAssessor::new(provider.clone())),
                            fallback.clone(),
                            lease,
                            std::time::Duration::MAX,
                            auto_control,
                        )
                        .with_locale(config.auto.locale.as_deref()),
                    )
                        as Arc<dyn talos_agent::permission_pipeline::ApprovalResolver>
                })
                .unwrap_or(fallback);
            let mut agent = Agent::with_security_and_hooks(
                provider,
                registry,
                None,
                None,
                workspace_root.to_path_buf(),
                hooks,
            )
            .with_permission_pipeline(
                permission_state,
                talos_permission::PermissionContext::new(
                    talos_permission::PermissionMode::Interactive,
                    talos_permission::InteractionCapability::Available,
                ),
                Some(resolver),
            );
            agent.enable_model_checkpoint_tools()?;
            agent.set_tool_protocol(config.tool_protocol());
            crate::mode_runtime::set_request_budget_spec(&mut agent, &config);
            if !loaded_plugin_packages.is_empty() {
                let mut policy = ToolPresentationPolicy::runtime_default();
                for capability in loaded_plugin_packages
                    .iter()
                    .flat_map(|package| package.capabilities.iter())
                {
                    policy = policy.disclose_tool(capability.clone());
                }
                agent.set_tool_presentation_policy(policy);
            }
            let runtime_skills =
                discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
            apply_runtime_skills(&mut agent, &runtime_skills);
            maybe_set_memory_provider(&mut agent, &config);
            set_todo_prompt_provider(&mut agent, &session_manager, &session);

            if !cli.no_context {
                let context = ContextLoader::new(workspace_root.to_path_buf())
                    .load()
                    .map_err(|e| anyhow!("{e}"))?;
                if !context.is_empty() {
                    agent.set_context_files(vec![ContextFile {
                        path: "AGENTS.md".into(),
                        content: context,
                    }]);
                }
            }

            if let Some(ref system_prompt) = cli.system_prompt {
                agent.set_custom_prompt(system_prompt.clone());
            }

            if let Some(ref append_prompt) = cli.append_system_prompt {
                agent.set_append_prompt(append_prompt.clone());
            }

            let initial_history = crate::history_restore::read_session_history_owned(&session, &runtime_lease).await?;

            let (model_context_limit, _) = config.resolve_model_limits();
            let session_config = SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: workspace_root.to_path_buf(),
                initial_history,
                model_context_limit,
            };
            let (handle, mut actor) = AppServerSession::new(agent, session_config);
            actor.set_persistence(
                session.clone(),
                session_metadata_for_model(&config.model, &config.provider),
            );
            let session_generation = crate::mode_runtime::runtime_generation_for_session(&session)?;
            actor.set_generation(session_generation);
            let scheduler_cancel = tokio_util::sync::CancellationToken::new();
            let sched_join = sched_pending.spawn(
                handle.sq_tx.clone(),
                session_generation,
                scheduler_cancel.clone(),
            );
            let actor_join = tokio::spawn(async move { actor.run_with_runtime_lease(runtime_lease).await });

            let mut current = match event_loop.take() {
                Some(previous) => previous.rebind(session.clone(), handle, terminal_approval_rx),
                None => event_loop::EventLoop::new(
                    session.clone(),
                    session_manager.clone(),
                    handle,
                    terminal_approval_rx,
                ),
            };
            let next = current.run().await;
            scheduler_cancel.cancel();
            let _ = sched_join.await;
            actor_join
                .await
                .context("interactive Actor shutdown failed")?;
            match next? {
                Some(child) => {
                    event_loop = Some(current);
                    Ok(Some(child))
                }
                None => Ok(None),
            }
        }
        .await;
        match outcome.with_context(|| format!(
            "interactive runtime for session {} at {} stopped; session data retained. Resume with talos --repl --session {}",
            session.id, session.file_path.display(), session.id,
        ))? {
            Some(child) => session = child,
            None => return Ok(()),
        }
    }
}

fn prepare_interactive_session_identity(
    config: &mut Config,
    session: &talos_session::Session,
) -> Result<()> {
    crate::mode_runtime::reconcile_session_runtime_state(session)?;
    crate::mode_runtime::apply_session_model_to_config(config, session);
    if !config.model.is_empty() {
        crate::mode_runtime::ensure_session_runtime_identity(config, session)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_resume_uses_persisted_identity_after_default_model_changes() {
        let root = tempfile::tempdir().expect("root");
        let manager = talos_session::SessionManager::with_dir(root.path().join("sessions"));
        let session = manager.create_session("identity", "").expect("session");
        let mut original = Config::default();
        original.provider = "openai".into();
        original.model = "gpt-4o".into();
        prepare_interactive_session_identity(&mut original, &session).expect("initialize");
        let mut changed = original.clone();
        changed.model = "gpt-4o-mini".into();
        prepare_interactive_session_identity(&mut changed, &session).expect("resume");
        let state = talos_session::PendingSubmissionStore::for_session(&session)
            .runtime_state()
            .expect("state")
            .expect("identity");
        assert_eq!(state.activation.target.model, "gpt-4o");
        assert_eq!(changed.model, "gpt-4o");
        assert_eq!(changed.provider, "openai");
        changed.model.clear();
        prepare_interactive_session_identity(&mut changed, &session)
            .expect("resume with empty default");
        assert_eq!(changed.model, "gpt-4o");
        let store = talos_session::PendingSubmissionStore::for_session(&session);
        let activation = talos_session::SessionRuntimeActivation::new(
            1,
            state.activation.target,
            talos_session::SessionRuntimeIdentity::new("openai", "gpt-4o-mini", None),
        );
        store
            .stage_runtime_activation(0, &activation)
            .expect("stage without transcript commit");
        let before = store.runtime_state().expect("before");
        let transcript = std::fs::read(&session.file_path).expect("transcript");
        assert!(prepare_interactive_session_identity(&mut changed, &session).is_err());
        assert_eq!(store.runtime_state().expect("after"), before);
        assert_eq!(changed.model, "gpt-4o");
        assert_eq!(
            std::fs::read(&session.file_path).expect("unchanged"),
            transcript
        );
    }

    #[cfg(windows)]
    const SHELL_TOOL_NAME: &str = "powershell";
    #[cfg(not(windows))]
    const SHELL_TOOL_NAME: &str = "bash";

    #[test]
    fn interactive_builtin_profile_preserves_current_inventory() {
        let mut registry = ToolRegistry::new();
        let approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
            talos_permission::PermissionEngine::new(),
        )));
        register_interactive_builtin_contributions(&mut registry, approval, Path::new("."))
            .expect("operation should succeed");

        let mut names = registry
            .list()
            .into_iter()
            .map(|tool| tool.name().to_owned())
            .collect::<Vec<_>>();
        names.sort();
        let mut expected = vec![
            SHELL_TOOL_NAME,
            "delete",
            "diff",
            "edit",
            "git_add",
            "git_branch_list",
            "git_checkout",
            "git_commit",
            "git_diff",
            "git_log",
            "git_pull",
            "git_push",
            "git_show",
            "git_status",
            "glob",
            "grep",
            "ls",
            "read",
            "stat",
            "tree",
            "write",
        ];
        expected.sort();
        assert_eq!(names, expected);
        assert!(registry.get("exec").is_none());
        assert!(registry.get("document_extract").is_none());
    }
}
