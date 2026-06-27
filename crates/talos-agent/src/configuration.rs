//! Agent construction and runtime configuration.

use std::path::PathBuf;
use std::sync::Arc;

use talos_core::tool::{ToolProtocol, ToolRegistry};
use talos_permission::PermissionEngine;
use talos_plugin::HookRegistry;
use talos_sandbox::SandboxProvider;
use talos_skill::SkillIndex;
use tokio_util::sync::CancellationToken;

use crate::prompt::{ActivatedSkillContext, ContextFile, SystemPromptBuilder, ToolDescription};
use crate::{Agent, MemoryProviderCallback, prompt};

impl Agent {
    /// Creates a new agent with the given language model provider and tool
    /// registry.
    ///
    /// # Security
    ///
    /// **This constructor is unsafe-by-policy**: no permission engine and no
    /// sandbox are configured. Every tool call is executed directly without
    /// any security gating. It exists **for unit tests only**; production
    /// run paths must use [`Agent::with_security`] to attach a permission
    /// engine and a sandbox provider.
    ///
    /// See `docs/decisions/007-process-hardening-unsafe.md` and the ARCH
    /// remediation review (R0 #ARCH-S2) for context.
    #[deprecated(
        note = "Agent::new() has NO permission engine and NO sandbox; use Agent::with_security(). See docs/decisions/007-process-hardening-unsafe.md and ARCH review."
    )]
    #[must_use]
    pub fn new(
        provider: Arc<dyn talos_core::provider::LanguageModel>,
        tools: ToolRegistry,
    ) -> Self {
        Self {
            provider,
            tools,
            permission_engine: None,
            sandbox: None,
            workspace_root: PathBuf::from("."),
            prompt_builder: SystemPromptBuilder::new().with_workspace_info("Workspace root: ."),
            hook_registry: Arc::new(HookRegistry::new()),
            workspace_context: None,
            tool_definitions: Vec::new(),
            cached_stable_prefix: std::sync::Mutex::new(None),
            memory_provider: None,
        }
    }

    /// Creates a new agent with security controls enabled.
    ///
    /// # Arguments
    ///
    /// * `provider` — The language model provider.
    /// * `tools` — Registry of tools available to the agent.
    /// * `permission_engine` — Optional permission engine for gating tool calls.
    ///   When `Some`, every tool call is evaluated before execution.
    /// * `sandbox` — Optional sandbox provider for bash tool execution.
    ///   When `Some`, bash commands run within the sandbox environment.
    /// * `workspace_root` — The workspace root directory, used for sandbox
    ///   configuration and path resolution.
    #[must_use]
    pub fn with_security(
        provider: Arc<dyn talos_core::provider::LanguageModel>,
        tools: ToolRegistry,
        permission_engine: Option<Arc<PermissionEngine>>,
        sandbox: Option<Box<dyn SandboxProvider>>,
        workspace_root: PathBuf,
    ) -> Self {
        Self::with_security_and_hooks(
            provider,
            tools,
            permission_engine,
            sandbox,
            workspace_root,
            Arc::new(HookRegistry::new()),
        )
    }

    /// Creates a new agent with security controls and a shared hook registry.
    #[must_use]
    pub fn with_security_and_hooks(
        provider: Arc<dyn talos_core::provider::LanguageModel>,
        tools: ToolRegistry,
        permission_engine: Option<Arc<PermissionEngine>>,
        sandbox: Option<Box<dyn SandboxProvider>>,
        workspace_root: PathBuf,
        hook_registry: Arc<HookRegistry>,
    ) -> Self {
        let descriptions: Vec<ToolDescription> = tools
            .list()
            .into_iter()
            .map(|tool| ToolDescription {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            })
            .collect();

        let prompt_builder = SystemPromptBuilder::new()
            .with_workspace_info(format!("Workspace root: {}", workspace_root.display()))
            .with_tools(descriptions.clone());

        let tool_definitions: Vec<talos_core::provider::ToolDefinition> = descriptions
            .into_iter()
            .map(|d| talos_core::provider::ToolDefinition {
                name: d.name,
                description: d.description,
                parameters: d.parameters,
            })
            .collect();

        Self {
            provider,
            tools,
            permission_engine,
            sandbox: sandbox.map(Arc::from),
            workspace_root,
            prompt_builder,
            hook_registry,
            workspace_context: None,
            tool_definitions,
            cached_stable_prefix: std::sync::Mutex::new(None),
            memory_provider: None,
        }
    }

    /// Sets a memory provider callback for injecting memory into the system prompt.
    ///
    /// The callback receives the user's query and returns an optional formatted
    /// memory section string. When `None` is returned, no memory is injected.
    pub fn set_memory_provider(&mut self, provider: Arc<MemoryProviderCallback>) {
        self.memory_provider = Some(provider);
    }

    /// Sets the tool descriptions for the system prompt builder.
    ///
    /// Tools are sorted alphabetically by name in the assembled prompt
    /// to ensure stable ordering across turns.
    pub fn set_tools(&mut self, tools: Vec<ToolDescription>) {
        self.update_prompt_builder(true, |builder| builder.with_tools(tools));
    }

    /// Sets the provider tool-call protocol.
    pub fn set_tool_protocol(&mut self, protocol: ToolProtocol) {
        self.update_prompt_builder(true, |builder| match protocol {
            ToolProtocol::TalosStrict => builder.with_strict_tool_format(),
            ToolProtocol::Compat => builder.with_tool_format(prompt::TOOL_CALLING_FORMAT),
            ToolProtocol::Native => builder.with_tool_format(""),
        });
    }

    /// Sets the skill index for the system prompt builder.
    ///
    /// Only Level 0 metadata (name, description, triggers) is included.
    pub fn set_skill_index(&mut self, skills: Vec<SkillIndex>) {
        self.update_prompt_builder(true, |builder| builder.with_skill_index(skills));
    }

    /// Sets explicitly activated Level 1/2 Skill content for the system prompt.
    ///
    /// The caller must load, bound, and validate this content before passing it
    /// here. Changing activated Skill content invalidates the stable prefix.
    pub fn set_activated_skill_context(&mut self, context: Option<ActivatedSkillContext>) {
        self.update_prompt_builder(true, |builder| builder.with_activated_skill(context));
    }

    /// Sets the context files for the system prompt builder.
    ///
    /// Typically loaded from `AGENTS.md` files via [`crate::context::ContextLoader`].
    pub fn set_context_files(&mut self, files: Vec<ContextFile>) {
        self.update_prompt_builder(false, |builder| builder.with_context_files(files));
    }

    /// Sets user-specific instructions for the system prompt builder.
    pub fn set_user_preferences(&mut self, prefs: String) {
        self.update_prompt_builder(false, |builder| builder.with_user_preferences(prefs));
    }

    /// Sets a custom prompt that replaces the default identity.
    pub fn set_custom_prompt(&mut self, prompt: String) {
        self.update_prompt_builder(true, |builder| builder.with_custom_prompt(prompt));
    }

    /// Sets an append prompt that is added at the end of the system prompt.
    pub fn set_append_prompt(&mut self, prompt: String) {
        self.update_prompt_builder(false, |builder| builder.with_append_prompt(prompt));
    }

    /// Clears the append prompt, removing any previously set value.
    pub fn clear_append_prompt(&mut self) {
        self.prompt_builder.clear_append_prompt();
    }

    /// Sets the append prompt to an optional value.
    ///
    /// Use `None` to clear the append prompt, or `Some(prompt)` to set it.
    pub fn set_append_prompt_opt(&mut self, prompt: Option<String>) {
        self.prompt_builder.set_append_prompt_opt(prompt);
    }

    /// Assembles and returns the full system prompt from all configured components.
    ///
    /// Components are assembled in the optimal order for caching:
    /// identity, tools, skill index, context files, user preferences,
    /// and append prompt (if provided).
    #[must_use]
    pub fn build_system_prompt(&self) -> String {
        self.prompt_builder.build()
    }

    /// Returns a [`CancellationToken`] that can be used to cancel the current
    /// turn. The caller is responsible for storing and triggering this token.
    ///
    /// Note: The token itself does not interrupt the provider stream; it is
    /// provided for the caller to coordinate cancellation at a higher level.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        CancellationToken::new()
    }

    fn update_prompt_builder(
        &mut self,
        invalidate_stable_prefix: bool,
        update: impl FnOnce(SystemPromptBuilder) -> SystemPromptBuilder,
    ) {
        self.prompt_builder = update(std::mem::take(&mut self.prompt_builder));
        if invalidate_stable_prefix {
            self.invalidate_stable_prefix_cache();
        }
    }

    pub(crate) fn invalidate_stable_prefix_cache(&self) {
        *self
            .cached_stable_prefix
            .lock()
            .expect("cache lock poisoned") = None;
    }
}
