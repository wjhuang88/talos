use std::io::{self, Write};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_core::message::AgentEvent;
use talos_core::session::{RuntimePolicy, SessionConfig, SessionEvent, SessionOp};

use crate::approval::ApprovalPrompt;
use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    apply_mcp_fixture_config, context_files_for_agent, maybe_set_memory_provider,
    request_preview_payload,
};
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{build_print_tool_registry, register_permission_aware_tools};
use crate::session_setup::{resolve_prompt, resolve_workspace_root};
use crate::skill_runtime::{apply_runtime_skills, discover_runtime_skills};
use crate::{Cli, build_hook_registry};

pub(crate) async fn run_print_mode(cli: Cli) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

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
    apply_mcp_fixture_config(&mut config, &cli);
    let prompt = resolve_prompt(cli.prompt)?;

    let hooks = build_hook_registry(true);
    let mut registry = build_print_tool_registry();

    #[cfg(debug_assertions)]
    let fixture_mode = cli.mcp_server_fixture.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
    let request_preview = request_preview_payload(&prompt);

    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
    let mcp_approval = Arc::new(std::sync::Mutex::new(ApprovalPrompt::new(
        talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
    )));
    register_permission_aware_tools(&mut registry, mcp_runtime.tools(), mcp_approval, true);

    let provider = if fixture_mode && cli.mock && request_preview.is_none() {
        use talos_provider::mock::MockProvider;
        Arc::new(
            MockProvider::new()
                .with_tool_call("mcp:fixture:echo", serde_json::json!({ "text": "ping" }))
                .with_response("fixture tool call complete"),
        ) as Arc<dyn talos_core::provider::LanguageModel>
    } else {
        build_provider(&config, &api_key, cli.mock)
    };

    let mut agent = Agent::with_security_and_hooks(
        provider,
        registry,
        Some(Arc::new(
            talos_permission::PermissionEngine::with_workspace_root(workspace_root.to_path_buf()),
        )),
        None,
        workspace_root.to_path_buf(),
        hooks,
    );
    agent.set_tool_protocol(config.tool_protocol());
    let runtime_skills = discover_runtime_skills(&workspace_root)?;
    apply_runtime_skills(&mut agent, &runtime_skills);
    maybe_set_memory_provider(&mut agent, &config);

    agent.set_context_files(context_files_for_agent(
        &config,
        &workspace_root,
        !cli.no_context,
    )?);

    if let Some(ref system_prompt) = cli.system_prompt {
        agent.set_custom_prompt(system_prompt.clone());
    }

    if let Some(ref append_prompt) = cli.append_system_prompt {
        agent.set_append_prompt(append_prompt.clone());
    }

    let (model_context_limit, _) = config.resolve_model_limits();
    let session_config = SessionConfig {
        runtime_policy: RuntimePolicy::headless_deny(),
        workspace_root: workspace_root.to_path_buf(),
        initial_history: vec![],
        model_context_limit,
    };
    let (mut handle, mut actor) = AppServerSession::new(agent, session_config);
    tokio::spawn(async move { actor.run().await });

    handle
        .sq_tx
        .send(match request_preview {
            Some(message) => SessionOp::PreviewRequest { message },
            None => SessionOp::Submit { message: prompt },
        })
        .await
        .context("failed to submit message to session")?;

    let mut stdout = io::stdout().lock();
    while let Some(event) = handle.eq_rx.recv().await {
        match event {
            SessionEvent::AgentEvent {
                event: AgentEvent::TextDelta { delta },
            } => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
            SessionEvent::AgentEvent {
                event: AgentEvent::TurnEnd { .. },
            } => {
                println!();
                return Ok(());
            }
            SessionEvent::AgentEvent {
                event: AgentEvent::Error { message },
            } => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
            SessionEvent::TurnCompleted { status, .. } => match status {
                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    println!();
                    return Ok(());
                }
                talos_core::session::TurnCompletionStatus::Cancelled => {
                    return Ok(());
                }
                talos_core::session::TurnCompletionStatus::Error { message } => {
                    eprintln!("Error: {message}");
                    std::process::exit(1);
                }
            },
            SessionEvent::Error { message } => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
            SessionEvent::AgentEvent { .. } => {}
            _ => {}
        }
    }
    Ok(())
}
