//! Interactive REPL mode.

use super::*;

fn register_interactive_builtin_contributions(
    registry: &mut ToolRegistry,
    approval: Arc<std::sync::Mutex<ApprovalPrompt>>,
    workspace_root: &Path,
) -> Result<()> {
    let bash = bash_tool_contribution(workspace_root.to_path_buf()).map_tool(|tool| {
        Arc::new(PermissionAwareTool {
            inner: tool,
            approval: approval.clone(),
            print_mode: false,
        })
    });
    registry.register_contribution(bash)?;

    for contribution in snapshot_aware_file_tool_contributions(workspace_root.to_path_buf()) {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: false,
            })
        });
        registry.register_contribution(contribution)?;
    }

    for contribution in workspace_non_document_tool_contributions(workspace_root.to_path_buf()) {
        let contribution = if contribution.name() == "tree" {
            contribution
        } else {
            contribution.map_tool(|tool| {
                Arc::new(PermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                    print_mode: false,
                })
            })
        };
        registry.register_contribution(contribution)?;
    }

    for contribution in git_read_tool_contributions(workspace_root.to_path_buf()) {
        registry.register_contribution(contribution)?;
    }
    for contribution in git_mutation_tool_contributions(workspace_root.to_path_buf()) {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: false,
            })
        });
        registry.register_contribution(contribution)?;
    }

    Ok(())
}

pub(crate) async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let workspace_root = resolve_workspace_root(&cli)?;

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let session = resolve_session_for_workspace(
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
        talos_permission::PermissionEngine::new(),
    )));

    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = ToolRegistry::new();
    for tool in sched_tools {
        registry.register(Arc::new(PermissionAwareTool {
            inner: tool,
            approval: approval.clone(),
            print_mode: false,
        }));
    }
    register_interactive_builtin_contributions(&mut registry, approval.clone(), &workspace_root)?;

    let hooks = build_hook_registry(true);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), approval.clone(), false);
    let loaded_plugin_packages = register_explicit_permission_aware_plugins(
        &mut registry,
        &cli.plugin_packages,
        approval,
        false,
    )
    .map_err(anyhow::Error::msg)?;

    let mut agent = Agent::with_security_and_hooks(
        build_provider(&config, &api_key, cli.mock),
        registry,
        Some(Arc::new(talos_permission::PermissionEngine::new())),
        None,
        workspace_root.to_path_buf(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
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
    let runtime_skills = discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
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

    let initial_history = session.read_messages().unwrap_or_default();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::interactive(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history,
        model_context_limit,
    };
    let (handle, mut actor) = AppServerSession::new(agent, session_config);
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    actor.set_persistence(
        session.clone(),
        session_metadata_for_model(&config.model, &config.provider),
    );
    tokio::spawn(async move { actor.run().await });

    let event_loop = event_loop::EventLoop::new(workspace_root, session, session_manager, handle);
    event_loop.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_builtin_profile_preserves_current_inventory() {
        let mut registry = ToolRegistry::new();
        let approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
            talos_permission::PermissionEngine::new(),
        )));
        register_interactive_builtin_contributions(&mut registry, approval, Path::new("."))
            .unwrap();

        let mut names = registry
            .list()
            .into_iter()
            .map(|tool| tool.name().to_owned())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                "bash",
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
            ]
        );
        assert!(registry.get("exec").is_none());
        assert!(registry.get("document_extract").is_none());
    }
}
