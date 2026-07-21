//! Inline runtime mode for the Talos CLI.

use std::io::{self, Write};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use talos_agent::Agent;
use talos_agent::context::ContextLoader;
use talos_agent::prompt::ContextFile;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_core::message::AgentEvent;
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnEventPayload,
};
use talos_core::tool::ToolPresentationPolicy;
use tokio::sync::mpsc;

use crate::approval::ApprovalPrompt;
use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    apply_mcp_fixture_config, maybe_set_memory_provider, request_preview_payload,
    session_metadata_for_model, set_todo_prompt_provider,
};
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{
    build_print_tool_registry, register_explicit_permission_aware_plugins,
    register_permission_aware_tools,
};
use crate::session_setup::{
    ResumeSelection, canonical_workspace_root, resolve_session_for_workspace,
    resolve_workspace_root, workspace_display_name,
};
use crate::skill_runtime::{RuntimeSkills, apply_runtime_skills, discover_runtime_skills};
use crate::{Cli, build_hook_registry};

pub(crate) async fn run_inline_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if !cli.attach.is_empty() {
        bail!(
            "--attach is not yet wired into inline mode. Use print mode (-p) for one-shot \
             image prompts, or run the TUI and use /attach."
        );
    }

    if let Some(ref model) = cli.model {
        config.model = model.clone();
    }
    if let Some(ref provider_str) = cli.provider {
        config.provider = parse_provider(provider_str)?;
    }

    if config.model.is_empty() && !cli.mock {
        bail!("no model configured. Set 'model' in ~/.talos/config.toml or pass --model.");
    }

    let api_key = if cli.mock {
        config.api_key().unwrap_or_default()
    } else {
        config.api_key().map_err(|e| anyhow!("{e}"))?
    };

    let workspace_root = resolve_workspace_root(&cli)?;
    let hooks = build_hook_registry(true);
    let provider = build_provider(&config, &api_key, cli.mock);
    apply_mcp_fixture_config(&mut config, &cli);
    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_print_tool_registry(sched_tools);
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(
        &mut registry,
        mcp_runtime.tools(),
        mcp_approval.clone(),
        true,
    );
    let loaded_plugin_packages = register_explicit_permission_aware_plugins(
        &mut registry,
        &cli.plugin_packages,
        mcp_approval,
        true,
    )
    .map_err(anyhow::Error::msg)?;

    let mut agent = Agent::with_security_and_hooks(
        provider,
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
    let mut runtime_skills =
        discover_runtime_skills(&workspace_root, config.skills.discover_shared)?;
    apply_runtime_skills(&mut agent, &runtime_skills);
    maybe_set_memory_provider(&mut agent, &config);

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

    if let Some(ref prompt) = cli.system_prompt {
        agent.set_custom_prompt(prompt.clone());
    }
    if let Some(ref append) = cli.append_system_prompt {
        agent.set_append_prompt(append.clone());
    }

    let session_manager =
        talos_session::SessionManager::new().context("failed to initialize session manager")?;
    let display_name = workspace_display_name(&workspace_root);
    let workspace_root_str = canonical_workspace_root(&workspace_root);
    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Disabled,
        false,
    )?;

    set_todo_prompt_provider(&mut agent, &session_manager, &session);

    let initial_history = session.read_messages().unwrap_or_default();

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::headless_deny(),
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

    let sq_tx = handle.sq_tx.clone();
    let mut eq_rx = handle.eq_rx;

    let stdin = io::stdin();

    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c().await.ok();
            let _ = sq_tx.try_send(SessionOp::Interrupt);
        }
    });

    println!("Talos inline mode. Type /quit to exit.");
    println!();

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => bail!("stdin error: {e}"),
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == "/quit" || input == "/exit" {
            break;
        }
        if input == "/skills" || input.starts_with("/skills ") {
            handle_inline_skills_command(input, &mut runtime_skills, &handle.sq_tx).await;
            continue;
        }

        let _ = handle
            .sq_tx
            .send(match request_preview_payload(input) {
                Some(message) => SessionOp::PreviewRequest { message },
                None => SessionOp::Submit {
                    message: input.to_string(),
                },
            })
            .await;

        let mut turn_done = false;
        while let Some(event) = eq_rx.recv().await {
            match event {
                SessionEvent::TurnEvent {
                    payload:
                        TurnEventPayload::Progress {
                            event: AgentEvent::TextDelta { delta },
                        },
                    ..
                } => {
                    print!("{delta}");
                    let _ = io::stdout().flush();
                }
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Completed { status },
                    ..
                } => {
                    match status {
                        talos_core::session::TurnCompletionStatus::Success { .. } => {
                            println!();
                            if let Err(e) = session_manager.update_index(&session) {
                                eprintln!("Warning: failed to update session index: {e}");
                            }
                        }
                        talos_core::session::TurnCompletionStatus::Cancelled => {
                            println!("\n(turn cancelled)");
                        }
                        talos_core::session::TurnCompletionStatus::Error { message } => {
                            eprintln!("\nError: {message}");
                        }
                    }
                    turn_done = true;
                    break;
                }
                SessionEvent::Error { message } => {
                    eprintln!("\nError: {message}");
                    turn_done = true;
                    break;
                }
                _ => {}
            }
        }

        if !turn_done {
            break;
        }
    }

    let _ = handle.sq_tx.send(SessionOp::Shutdown).await;
    Ok(())
}

async fn handle_inline_skills_command(
    input: &str,
    runtime_skills: &mut RuntimeSkills,
    sq_tx: &mpsc::Sender<SessionOp>,
) {
    let arg = input.strip_prefix("/skills").unwrap_or(input).trim();
    if arg.is_empty() {
        let diagnostics = runtime_skills.diagnostics();
        if diagnostics.is_empty() {
            println!("No runtime skills discovered.");
            return;
        }
        println!("Runtime skills:");
        for skill in diagnostics {
            let marker = if skill.active { " (active)" } else { "" };
            println!("- {}{}: {}", skill.name, marker, skill.description);
        }
        return;
    }

    let mut parts = arg.splitn(2, char::is_whitespace);
    let action = parts.next().unwrap_or_default();
    let value = parts.next().unwrap_or_default().trim();

    match action {
        "activate" if value.is_empty() => {
            println!("Usage: /skills activate <name>");
        }
        "activate" => match runtime_skills.activate(value) {
            Ok(content) => {
                let name = runtime_skills.active_name().map(str::to_string);
                if sq_tx
                    .send(SessionOp::SetSkillContext {
                        name,
                        content: Some(content),
                    })
                    .await
                    .is_ok()
                {
                    println!("Skill activated: {value}. Content added to provider context only.");
                }
            }
            Err(error) => println!("Error: {error}"),
        },
        "reference" if value.is_empty() => {
            println!("Usage: /skills reference <path>");
        }
        "reference" => match runtime_skills.load_reference(value) {
            Ok(content) => {
                let name = runtime_skills.active_name().map(str::to_string);
                if sq_tx
                    .send(SessionOp::SetSkillContext {
                        name,
                        content: Some(content),
                    })
                    .await
                    .is_ok()
                {
                    println!(
                        "Skill loaded reference: {value}. Content added to provider context only."
                    );
                }
            }
            Err(error) => println!("Error: {error}"),
        },
        _ => println!(
            "Unknown /skills action. Use /skills, /skills activate <name>, or /skills reference <path>."
        ),
    }
}
