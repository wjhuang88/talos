use std::io::{self, Write};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use talos_agent::Agent;
use talos_agent::session::AppServerSession;
use talos_config::Config;
use talos_core::message::AgentEvent;
use talos_core::model::ImageInputCapability;
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnEventPayload,
};
use talos_core::tool::ToolPresentationPolicy;

use crate::approval::ApprovalPrompt;
use crate::image_validation::{ImageValidationError, create_image_content_part, max_image_count};
use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    apply_mcp_fixture_config, context_files_for_agent, maybe_set_memory_provider,
    request_preview_payload,
};
use crate::provider_setup::{build_provider, parse_provider};
use crate::registry::{
    build_print_tool_registry, register_explicit_permission_aware_plugins,
    register_permission_aware_tools,
};
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
    let prompt = resolve_prompt(cli.prompt.clone())?;

    let hooks = build_hook_registry(true);
    let (sched_tools, sched_pending) = talos_agent::create_scheduler_tools();
    let mut registry = build_print_tool_registry(sched_tools);

    #[cfg(debug_assertions)]
    let fixture_mode = cli.mcp_server_fixture.is_some();
    #[cfg(not(debug_assertions))]
    let fixture_mode = false;
    let request_preview = request_preview_payload(&prompt);
    if request_preview.is_some() && !cli.attach.is_empty() {
        bail!(
            "--attach cannot be combined with the request-preview magic prefix. \
               Drop the prefix or remove --attach."
        );
    }

    let mcp_runtime = McpSessionRuntime::start(&config.mcp, hooks.clone()).await?;
    mcp_runtime.report_startup_failures();
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
    let _sched_join = sched_pending.spawn(
        handle.sq_tx.clone(),
        tokio_util::sync::CancellationToken::new(),
    );
    tokio::spawn(async move { actor.run().await });

    handle
        .sq_tx
        .send(build_print_submit_op(
            &cli,
            &config,
            prompt,
            request_preview,
            &config.all_models(),
        )?)
        .await
        .context("failed to submit message to session")?;

    let mut stdout = io::stdout().lock();
    while let Some(event) = handle.eq_rx.recv().await {
        match event {
            SessionEvent::TurnEvent {
                payload:
                    TurnEventPayload::Progress {
                        event: AgentEvent::TextDelta { delta },
                    },
                ..
            } => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { status },
                ..
            } => match status {
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
            _ => {}
        }
    }
    Ok(())
}

/// Choose the SessionOp for print mode based on --attach and the preview
/// magic prefix. `--attach` enforces the capability gate and path
/// validation up front so that misconfiguration fails before any network
/// call.
fn build_print_submit_op(
    cli: &Cli,
    config: &Config,
    prompt: String,
    request_preview: Option<String>,
    all_models: &[talos_config::model::ModelMetadata],
) -> Result<SessionOp> {
    if cli.attach.is_empty() {
        return Ok(match request_preview {
            Some(message) => SessionOp::PreviewRequest { message },
            None => SessionOp::Submit { message: prompt },
        });
    }

    // Capability gate (R3 equivalent for print mode): refuse before any
    // file-system probe when the active model is not confirmed Supported.
    let metadata =
        talos_config::model::find_model_by_provider(all_models, &config.provider, &config.model);
    let capability = ImageInputCapability::from_metadata(metadata);
    if !capability.allows_attachment() {
        bail!(
            "Active model {}/{} does not support image input (capability: {:?}). \
             --attach refused before any file read. \
             Use --model to select a vision-capable model.",
            config.provider,
            config.model,
            capability
        );
    }

    if cli.attach.len() > max_image_count() {
        bail!(
            "Too many --attach paths: {} provided, limit is {}. \
             Drop some attachments or run multiple prompts.",
            cli.attach.len(),
            max_image_count()
        );
    }

    let mut attachments = Vec::with_capacity(cli.attach.len());
    let mut total_bytes: u64 = 0;
    for path in &cli.attach {
        let part = create_image_content_part(path, attachments.len(), total_bytes)
            .map_err(|e| anyhow!(attachment_error_message(&e, path)))?;
        if let talos_core::message::ContentPart::Image { byte_count, .. } = &part {
            total_bytes = total_bytes.saturating_add(*byte_count);
        }
        attachments.push(part);
    }

    Ok(SessionOp::SubmitMultimodal {
        text: prompt,
        attachments,
    })
}

fn attachment_error_message(err: &ImageValidationError, path: &std::path::Path) -> String {
    format!("--attach {} failed: {err}", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cli;

    fn metadata_for(
        provider: &str,
        model: &str,
        image_input: bool,
    ) -> talos_config::model::ModelMetadata {
        talos_config::model::ModelMetadata {
            id: model.to_string(),
            provider: provider.to_string(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: talos_core::model::ModelCapabilities {
                image_input,
                ..Default::default()
            },
            release_date: None,
            source: talos_core::model::ModelSource::Manual,
            variants: vec![],
        }
    }

    fn config_with(provider: &str, model: &str) -> Config {
        let mut config = Config::default();
        config.provider = provider.to_string();
        config.model = model.to_string();
        config
    }

    fn cli_with_attach(paths: Vec<std::path::PathBuf>) -> Cli {
        let mut cli: Cli = clap::Parser::parse_from(["talos"]);
        cli.attach = paths;
        cli
    }

    #[test]
    fn submit_op_plain_when_no_attach_and_no_preview() {
        let cli = cli_with_attach(vec![]);
        let config = config_with("anthropic", "claude-sonnet-4-5");
        let catalog = vec![metadata_for("anthropic", "claude-sonnet-4-5", true)];
        let op = build_print_submit_op(&cli, &config, "hello".to_string(), None, &catalog).unwrap();
        match op {
            SessionOp::Submit { message } => assert_eq!(message, "hello"),
            other => panic!("expected Submit, got {other:?}"),
        }
    }

    #[test]
    fn submit_op_preview_when_magic_prefix_and_no_attach() {
        let cli = cli_with_attach(vec![]);
        let config = config_with("anthropic", "claude-sonnet-4-5");
        let catalog = vec![metadata_for("anthropic", "claude-sonnet-4-5", true)];
        let op = build_print_submit_op(
            &cli,
            &config,
            "/mock-request hello".to_string(),
            Some("hello".to_string()),
            &catalog,
        )
        .unwrap();
        match op {
            SessionOp::PreviewRequest { message } => assert_eq!(message, "hello"),
            other => panic!("expected PreviewRequest, got {other:?}"),
        }
    }

    /// R8 regression: --attach on a model whose capability is
    /// `Unsupported` must be refused before any file probe. We do not
    /// even create the file — if validation reached the filesystem it
    /// would return IoError instead of the capability bail.
    #[test]
    fn attach_refused_when_capability_unsupported() {
        let cli = cli_with_attach(vec![std::path::PathBuf::from(
            "/tmp/r8-must-not-be-read.png",
        )]);
        let config = config_with("anthropic", "claude-haiku-text-only");
        let catalog = vec![metadata_for("anthropic", "claude-haiku-text-only", false)];
        let result = build_print_submit_op(&cli, &config, "describe".to_string(), None, &catalog);
        let err = result.err().expect("Unsupported capability must bail");
        let msg = format!("{err}");
        assert!(
            msg.contains("does not support image input"),
            "expected capability-refusal message, got: {msg}"
        );
        assert!(msg.contains("Unsupported"));
    }

    #[test]
    fn attach_refused_when_capability_unknown_no_metadata() {
        let cli = cli_with_attach(vec![std::path::PathBuf::from(
            "/tmp/r8-must-not-be-read.png",
        )]);
        let config = config_with("custom", "discovered-model");
        let catalog: Vec<talos_config::model::ModelMetadata> = vec![];
        let result = build_print_submit_op(&cli, &config, "describe".to_string(), None, &catalog);
        let err = result.err().expect("Unknown capability must bail");
        let msg = format!("{err}");
        assert!(msg.contains("Unknown"));
    }

    #[test]
    fn attach_refused_when_too_many_paths() {
        let too_many: Vec<_> = (0..(max_image_count() + 1))
            .map(|i| std::path::PathBuf::from(format!("/tmp/r8-{i}.png")))
            .collect();
        let cli = cli_with_attach(too_many);
        let config = config_with("openai", "gpt-4o");
        let catalog = vec![metadata_for("openai", "gpt-4o", true)];
        let result = build_print_submit_op(&cli, &config, "describe".to_string(), None, &catalog);
        let err = result.err().expect("too many attach paths must bail");
        let msg = format!("{err}");
        assert!(msg.contains("Too many --attach paths"));
    }

    /// R8 positive: --attach on a Supported model with a real PNG flows
    /// through SubmitMultimodal. Uses a real encoded image so R4's
    /// decoder succeeds.
    #[test]
    fn attach_succeeds_for_supported_model_with_real_png() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("r8.png");
        let img = image::RgbaImage::new(4, 4);
        img.save_with_format(&img_path, image::ImageFormat::Png)
            .unwrap();

        let cli = cli_with_attach(vec![img_path]);
        let config = config_with("openai", "gpt-4o");
        let catalog = vec![metadata_for("openai", "gpt-4o", true)];
        let op =
            build_print_submit_op(&cli, &config, "describe".to_string(), None, &catalog).unwrap();
        match op {
            SessionOp::SubmitMultimodal { text, attachments } => {
                assert_eq!(text, "describe");
                assert_eq!(attachments.len(), 1);
                match &attachments[0] {
                    talos_core::message::ContentPart::Image { mime, .. } => {
                        assert_eq!(mime, "image/png");
                    }
                    _ => panic!("expected ContentPart::Image"),
                }
            }
            other => panic!("expected SubmitMultimodal, got {other:?}"),
        }
    }
}
