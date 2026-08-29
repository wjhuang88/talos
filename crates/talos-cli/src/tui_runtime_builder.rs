//! Single production construction boundary for TUI Session runtimes.
//!
//! Every initial or replacement Actor is built from the same authoritative
//! Config and shared permission state. Derived credentials, model limits,
//! provider options, MCP topology, plugins and capabilities are resolved at the
//! moment of construction rather than cached by the lifecycle loop.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use talos_agent::{
    Agent, PendingSchedulerActor,
    auto_resolver::{
        AutoPermissionResolver, ManagedWorkspaceLease, ProviderAutoPermissionAssessor,
    },
    session::AppServerSession,
};
use talos_config::Config;
use talos_core::message::Message;
use talos_core::session::{RuntimePolicy, SessionConfig, SessionHandle};
use talos_core::tool::ToolPresentationPolicy;
use talos_plugin::HookRegistry;
use talos_plugin::wasm::LoadedPluginPackage;
use talos_session::{Session, SessionManager};

use crate::mcp_runtime::McpSessionRuntime;
use crate::mode_runtime::{
    context_files_for_agent, maybe_set_memory_provider, session_metadata_for_model,
    set_image_input_capability, set_request_budget_spec, set_todo_prompt_provider,
};
use crate::model_lifecycle::materialize_runtime_model_config;
use crate::provider_setup::build_provider;
use crate::registry::{
    TuiApprovalHandler, build_tui_tool_registry_with_capability, register_explicit_tui_plugins,
    register_tui_permission_aware_tools,
};
use crate::skill_runtime::{RuntimeSkills, apply_runtime_skills, discover_runtime_skills};
use talos_tools::CapStdAtomicCreateCapability;

/// Inputs that are stable across all runtime reconstructions in one TUI process.
#[derive(Clone)]
pub(crate) struct TuiRuntimeBuilder {
    hooks: Arc<HookRegistry>,
    workspace_root: PathBuf,
    session_manager: SessionManager,
    approval_handler: Arc<TuiApprovalHandler>,
    plugin_packages: Arc<Vec<PathBuf>>,
    include_workspace_context: bool,
    mock: bool,
}

pub(crate) struct PreparedTuiRuntime {
    agent: Agent,
    pending_scheduler: PendingSchedulerActor,
    mcp_runtime: McpSessionRuntime,
    runtime_config: Config,
    runtime_skills: RuntimeSkills,
    loaded_plugin_packages: Vec<LoadedPluginPackage>,
    session: Session,
    workspace_root: PathBuf,
}

impl PreparedTuiRuntime {
    pub(crate) fn finish(self, initial_history: Vec<Message>) -> BuiltTuiRuntime {
        let (model_context_limit, _) = self.runtime_config.resolve_model_limits();
        let session_config = SessionConfig {
            runtime_policy: RuntimePolicy::interactive(),
            workspace_root: self.workspace_root,
            initial_history,
            model_context_limit,
        };
        let (handle, mut actor) = AppServerSession::new(self.agent, session_config);
        actor.set_persistence(
            self.session,
            session_metadata_for_model(&self.runtime_config.model, &self.runtime_config.provider),
        );
        BuiltTuiRuntime {
            handle,
            actor,
            pending_scheduler: self.pending_scheduler,
            mcp_runtime: self.mcp_runtime,
            runtime_skills: self.runtime_skills,
            loaded_plugin_packages: self.loaded_plugin_packages,
        }
    }
}

pub(crate) struct BuiltTuiRuntime {
    pub handle: SessionHandle,
    pub actor: AppServerSession,
    pub pending_scheduler: PendingSchedulerActor,
    pub mcp_runtime: McpSessionRuntime,
    pub runtime_skills: RuntimeSkills,
    pub loaded_plugin_packages: Vec<LoadedPluginPackage>,
}

impl TuiRuntimeBuilder {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        hooks: Arc<HookRegistry>,
        workspace_root: PathBuf,
        session_manager: SessionManager,
        approval_handler: Arc<TuiApprovalHandler>,
        plugin_packages: Vec<PathBuf>,
        include_workspace_context: bool,
        mock: bool,
    ) -> Self {
        Self {
            hooks,
            workspace_root,
            session_manager,
            approval_handler,
            plugin_packages: Arc::new(plugin_packages),
            include_workspace_context,
            mock,
        }
    }

    #[must_use]
    pub(crate) fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    /// Prepares Provider, MCP, tools, plugins, permissions and prompt policy
    /// without crossing a Session generation fence.
    pub(crate) async fn prepare(
        &self,
        config: &Config,
        session: &Session,
    ) -> Result<PreparedTuiRuntime> {
        let (runtime_config, resolution) = materialize_runtime_model_config(config);
        if let Some(diagnostic) = resolution.diagnostic.as_deref() {
            tracing::warn!(
                provider = %config.provider,
                model = %config.model,
                variant = ?config.variant,
                "{diagnostic}"
            );
        }

        let api_key = match runtime_config.api_key() {
            Ok(key) => key,
            Err(error) if self.mock => {
                tracing::warn!(%error, "using empty mock credential for TUI runtime construction");
                String::new()
            }
            Err(error) => return Err(anyhow!(error.to_string())),
        };
        let provider = build_provider(&runtime_config, &api_key, self.mock);

        let mcp_runtime = McpSessionRuntime::start(&runtime_config.mcp, self.hooks.clone())
            .await
            .context("failed to start MCP runtime")?;
        mcp_runtime.report_startup_failures();

        let (scheduler_tools, pending_scheduler) = talos_agent::create_scheduler_tools();
        let atomic_create_capability = CapStdAtomicCreateCapability::open(&self.workspace_root)
            .ok()
            .map(|value| Arc::new(value) as talos_core::tool::SharedAtomicCreateCapability);
        let mut registry = build_tui_tool_registry_with_capability(
            self.approval_handler.clone(),
            self.workspace_root.clone(),
            session.id,
            scheduler_tools,
            atomic_create_capability.clone(),
        );
        register_tui_permission_aware_tools(
            &mut registry,
            mcp_runtime.tools(),
            self.approval_handler.clone(),
        );
        let loaded_plugin_packages = register_explicit_tui_plugins(
            &mut registry,
            self.plugin_packages.as_slice(),
            self.approval_handler.clone(),
        )
        .map_err(anyhow::Error::msg)?;

        let fallback = self.approval_handler.clone();
        let resolver: Arc<dyn talos_agent::permission_pipeline::ApprovalResolver> =
            if runtime_config.auto.enabled {
                atomic_create_capability
                    .clone()
                    .and_then(|capability| {
                        ManagedWorkspaceLease::new(&self.workspace_root, session.id.to_string())
                            .ok()
                            .map(|lease| {
                                let lease = lease.with_atomic_create_capability(capability);
                                Arc::new(AutoPermissionResolver::new(
                                    Arc::new(ProviderAutoPermissionAssessor::new(provider.clone())),
                                    fallback.clone(),
                                    lease,
                                    std::time::Duration::from_secs(8),
                                ))
                                    as Arc<dyn talos_agent::permission_pipeline::ApprovalResolver>
                            })
                    })
                    .unwrap_or(fallback)
            } else {
                fallback
            };
        let mut agent = Agent::with_security_and_hooks(
            provider,
            registry,
            None,
            None,
            self.workspace_root.clone(),
            self.hooks.clone(),
        )
        .with_permission_pipeline(
            self.approval_handler.shared_engine(),
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(resolver),
        );
        agent.set_tool_protocol(runtime_config.tool_protocol());
        set_image_input_capability(&mut agent, &runtime_config);
        set_request_budget_spec(&mut agent, &runtime_config);
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
            discover_runtime_skills(&self.workspace_root, runtime_config.skills.discover_shared)?;
        apply_runtime_skills(&mut agent, &runtime_skills);
        maybe_set_memory_provider(&mut agent, &runtime_config);
        set_todo_prompt_provider(&mut agent, &self.session_manager, session);
        agent.set_context_files(context_files_for_agent(
            &runtime_config,
            &self.workspace_root,
            self.include_workspace_context,
        )?);

        Ok(PreparedTuiRuntime {
            agent,
            pending_scheduler,
            mcp_runtime,
            runtime_config,
            runtime_skills,
            loaded_plugin_packages,
            session: session.clone(),
            workspace_root: self.workspace_root.clone(),
        })
    }

    /// Builds one complete TUI Actor contract from the current authoritative Config.
    pub(crate) async fn build(
        &self,
        config: &Config,
        session: &Session,
        initial_history: Vec<Message>,
    ) -> Result<BuiltTuiRuntime> {
        Ok(self.prepare(config, session).await?.finish(initial_history))
    }
}
