//! Agent construction and runtime configuration.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use talos_core::provider::ToolDefinition;
use talos_core::tool::{AgentTool, ToolPresentationPolicy, ToolProtocol, ToolRegistry};
use talos_permission::PermissionEngine;
use talos_plugin::HookRegistry;
use talos_sandbox::SandboxProvider;
use talos_skill::SkillIndex;
use tokio_util::sync::CancellationToken;

use crate::prompt::{ActivatedSkillContext, ContextFile, SystemPromptBuilder, ToolDescription};
use crate::{Agent, MemoryProviderCallback, TodoSectionProviderCallback, prompt};

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
            presented_tool_names: HashSet::new(),
            enforce_tool_presentation_policy: false,
            tool_presentation_policy: ToolPresentationPolicy::full(),
            cached_stable_prefix: std::sync::Mutex::new(None),
            memory_provider: None,
            todo_section_provider: None,
            provider_key: None,
            model_id: None,
            replay_reasoning: true,
            bash_compression_enabled: false,
            tool_output_threshold: 4000,
            image_input_supported: false,
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
        let tool_presentation_policy = ToolPresentationPolicy::runtime_default();
        let (descriptions, tool_definitions, presented_tool_names) =
            describe_presented_tools(&tools, &tool_presentation_policy);

        // read_image is registered but gated by image_input_supported;
        // filter it from the initial presentation until a caller enables
        // it via with_image_input_supported(true) (ADR-051 / I154).
        let descriptions: Vec<_> = descriptions
            .into_iter()
            .filter(|d| d.name != "read_image")
            .collect();
        let tool_definitions: Vec<_> = tool_definitions
            .into_iter()
            .filter(|td| td.name != "read_image")
            .collect();
        let presented_tool_names: HashSet<_> = presented_tool_names
            .into_iter()
            .filter(|n| n != "read_image")
            .collect();

        let prompt_builder = SystemPromptBuilder::new()
            .with_workspace_info(format!("Workspace root: {}", workspace_root.display()))
            .with_tools(descriptions.clone());

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
            presented_tool_names,
            enforce_tool_presentation_policy: true,
            tool_presentation_policy,
            cached_stable_prefix: std::sync::Mutex::new(None),
            memory_provider: None,
            todo_section_provider: None,
            provider_key: None,
            model_id: None,
            replay_reasoning: true,
            bash_compression_enabled: false,
            tool_output_threshold: 4000,
            image_input_supported: false,
        }
    }

    /// Configures reasoning origin identity and replay behavior (ADR-034).
    #[must_use]
    pub fn with_reasoning_identity(
        mut self,
        provider_key: Option<String>,
        model_id: Option<String>,
        replay: bool,
    ) -> Self {
        self.provider_key = provider_key;
        self.model_id = model_id;
        self.replay_reasoning = replay;
        self
    }

    /// Sets a memory provider callback for injecting memory into the system prompt.
    ///
    /// The callback receives the user's query and returns an optional formatted
    /// memory section string. When `None` is returned, no memory is injected.
    pub fn set_memory_provider(&mut self, provider: Arc<MemoryProviderCallback>) {
        self.memory_provider = Some(provider);
    }

    /// Sets a callback for injecting bounded active session todos into the dynamic prompt suffix.
    ///
    /// The callback returns already-formatted advisory text. It is evaluated once per provider
    /// request and does not invalidate the stable prompt prefix cache.
    pub fn set_todo_section_provider(&mut self, provider: Arc<TodoSectionProviderCallback>) {
        self.todo_section_provider = Some(provider);
    }

    /// Enables or disables bash output compression for model context.
    ///
    /// When enabled, bash tool output exceeding 30 lines is compressed to the
    /// last 30 lines plus a truncation marker before entering model context.
    /// The raw output is preserved on the UI event/export surface.
    ///
    /// Default: disabled (false).
    #[must_use]
    pub fn with_bash_compression(mut self, enabled: bool) -> Self {
        self.bash_compression_enabled = enabled;
        self
    }

    /// Enables or disables the `read_image` tool presentation based on the
    /// active model's image input capability (ADR-051 / I154).
    ///
    /// When `true`, `read_image` is included in the tool definitions sent to
    /// the provider. When `false` (default), the tool is registered but not
    /// presented — model calls to it are rejected by the presentation policy.
    #[must_use]
    pub fn with_image_input_supported(mut self, supported: bool) -> Self {
        self.image_input_supported = supported;
        self
    }

    /// Sets image input capability on an existing agent (ADR-051 / I154).
    pub fn set_image_input_supported(&mut self, supported: bool) {
        self.image_input_supported = supported;
    }

    /// Sets the tool descriptions for the system prompt builder.
    ///
    /// Tools are sorted alphabetically by name in the assembled prompt
    /// to ensure stable ordering across turns.
    pub fn set_tools(&mut self, tools: Vec<ToolDescription>) {
        self.tool_definitions = tools
            .iter()
            .map(|tool| ToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.parameters.clone(),
            })
            .collect();
        self.presented_tool_names = tools.iter().map(|tool| tool.name.clone()).collect();
        self.enforce_tool_presentation_policy = true;
        self.update_prompt_builder(true, |builder| builder.with_tools(tools));
    }

    /// Sets which registered tool families are presented to the model.
    ///
    /// The executable [`ToolRegistry`] is unchanged. Calls to registered tools
    /// that were not presented return a recoverable tool error instead of
    /// executing silently.
    pub fn set_tool_presentation_policy(&mut self, policy: ToolPresentationPolicy) {
        self.tool_presentation_policy = policy;
        let (descriptions, tool_definitions, presented_tool_names) =
            describe_presented_tools(&self.tools, &self.tool_presentation_policy);
        self.tool_definitions = tool_definitions;
        self.presented_tool_names = presented_tool_names;
        self.enforce_tool_presentation_policy = true;
        self.update_prompt_builder(true, |builder| builder.with_tools(descriptions));
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

pub(crate) fn describe_presented_tools(
    tools: &ToolRegistry,
    policy: &ToolPresentationPolicy,
) -> (Vec<ToolDescription>, Vec<ToolDefinition>, HashSet<String>) {
    let mut selected: Vec<&dyn AgentTool> = tools
        .list()
        .into_iter()
        .filter(|tool| policy.allows_tool(*tool))
        .collect();
    selected.sort_by(|a, b| a.name().cmp(b.name()));

    let descriptions: Vec<ToolDescription> = selected
        .iter()
        .map(|tool| {
            let backends = policy.backend_set_for(tool.name());
            ToolDescription {
                name: tool.name().to_string(),
                description: tool.description_for_backends(&backends),
                parameters: tool.parameters_for_backends(&backends),
                family: tool.family(),
            }
        })
        .collect();

    let tool_definitions = descriptions
        .iter()
        .map(|tool| ToolDefinition {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.parameters.clone(),
        })
        .collect();
    let presented_tool_names = descriptions.iter().map(|tool| tool.name.clone()).collect();

    (descriptions, tool_definitions, presented_tool_names)
}
