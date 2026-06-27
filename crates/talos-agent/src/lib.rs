//! Talos agent — core orchestration logic and the agent turn loop.
//!
//! The agent manages a conversation turn with an LLM provider, executing tool
//! calls when the model requests them and feeding results back until a final
//! text response is produced.
//!
//! # Security Pipeline
//!
//! Every tool call goes through a security pipeline:
//! 1. **Permission check** — the [`PermissionEngine`] evaluates the call
//! 2. **Sandbox execution** — bash tools run through the sandbox when available
//! 3. **Execute** — the tool is invoked directly
//! 4. **Retry on denial** — denied calls return an error result
//!
//! The `Ask` decision defaults to `Deny` at the agent level; interactive
//! approval is handled by the CLI layer.

pub mod compaction;
pub mod token;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub mod caching;
pub mod context;
mod helpers;
pub mod prompt;
pub mod session;
mod tool_execution;

use talos_core::message::{AgentEvent, Message, MessageToolResult, StopReason, ToolCall, Usage};
use talos_core::provider::{LanguageModel, ProviderError};
use talos_core::tool::{ToolProtocol, ToolProvenance, ToolRegistry};
use talos_permission::PermissionEngine;
use talos_plugin::{
    BudgetKind, HookContext, HookEvent, HookOutcome, HookRegistry, ToolObservation, TurnId,
    TurnStatus,
};
use talos_sandbox::SandboxProvider;
use talos_skill::SkillIndex;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub use prompt::{ActivatedSkillContext, ContextFile, SystemPromptBuilder, ToolDescription};

/// Maximum number of tool calls allowed per turn before budget exhaustion.
const MAX_TOOL_CALLS_PER_TURN: usize = 50;

/// Maximum number of concurrent read-only tool executions.
const MAX_CONCURRENT_READ_ONLY: usize = 10;

/// Threshold for doom loop detection — same tool+args this many times triggers
/// an early stop.
const DOOM_LOOP_THRESHOLD: u32 = 3;

#[derive(Debug, Clone)]
struct PendingToolCall {
    call: ToolCall,
    provenance: ToolProvenance,
}

/// Errors that can occur during agent execution.
#[derive(Debug, Error)]
pub enum AgentError {
    /// An error from the underlying LLM provider.
    #[error("provider error: {0}")]
    ProviderError(#[from] ProviderError),

    /// The turn was cancelled via [`CancellationToken`].
    #[error("turn cancelled")]
    Cancelled,

    /// An unexpected event sequence was received.
    #[error("unexpected event: {0}")]
    UnexpectedEvent(String),

    /// A tool-related error occurred (lookup failure, execution panic, etc.).
    #[error("tool error: {0}")]
    ToolError(String),

    /// The turn exceeds the maximum allowed tool call budget.
    #[error("turn budget exceeded: maximum of {MAX_TOOL_CALLS_PER_TURN} tool calls per turn")]
    TurnBudgetExceeded,

    /// A potential doom loop was detected — the same tool was called with
    /// identical arguments multiple times in a single turn.
    #[error("doom loop detected: {0}")]
    DoomLoopDetected(String),

    /// A hook denied the current operation.
    #[error("hook denied operation: {0}")]
    HookDenied(String),
}

/// Result alias for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

// Callback type for memory prompt injection.
type MemoryProviderCallback = dyn Fn(&str) -> Option<String> + Send + Sync;

/// The agent orchestrates a conversation turn: takes a user message, calls the
/// LLM provider, streams events, executes tool calls when requested, and feeds
/// results back until a final text response is produced.
///
/// # Security Pipeline
///
/// When a permission engine is configured, every tool call is evaluated before
/// execution. Denied calls return an error result without invoking the tool.
/// The `Ask` decision defaults to `Deny` at the agent level.
///
/// When a sandbox is configured, bash tool calls are executed within the
/// sandbox environment. If the sandbox is unavailable, execution falls back
/// to direct invocation.
///
/// # Example
///
/// ```no_run
/// use talos_agent::Agent;
/// use talos_core::tool::ToolRegistry;
/// use std::sync::Arc;
/// # use talos_core::provider::{LanguageModel, ProviderResult, Receiver};
/// # use talos_core::message::{AgentEvent, Message};
/// # struct MyModel;
/// # #[async_trait::async_trait]
/// # impl LanguageModel for MyModel {
/// #     async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> { unimplemented!() }
/// # }
/// # async fn example() {
/// let provider: Arc<dyn LanguageModel> = Arc::new(MyModel);
/// let tools = ToolRegistry::new();
/// let agent = Agent::new(provider, tools);
/// let response = agent.run("Hello!".into()).await.unwrap();
/// # }
/// ```
pub struct Agent {
    /// The language model provider used for this agent.
    provider: Arc<dyn LanguageModel>,
    /// Registry of tools available to the agent.
    tools: ToolRegistry,
    /// Optional permission engine for gating tool execution.
    permission_engine: Option<Arc<PermissionEngine>>,
    /// Optional sandbox provider for bash tool execution.
    sandbox: Option<Arc<dyn SandboxProvider>>,
    /// Workspace root directory, used for sandbox configuration.
    workspace_root: PathBuf,
    /// Builder for assembling the system prompt.
    prompt_builder: SystemPromptBuilder,
    /// Per-agent lifecycle hook registry.
    hook_registry: Arc<HookRegistry>,
    /// Workspace context (AGENTS.md, history summary) for Context message.
    workspace_context: Option<String>,
    /// Cached tool definitions for native API calls.
    tool_definitions: Vec<talos_core::provider::ToolDefinition>,
    /// Cached stable prefix (Identity + Tools + Skills) computed once and
    /// reused across turns. Invalidated when tools, skills, or identity change.
    cached_stable_prefix: std::sync::Mutex<Option<String>>,
    /// Optional memory provider callback for injecting memory into prompts.
    memory_provider: Option<Arc<MemoryProviderCallback>>,
}

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
    pub fn new(provider: Arc<dyn LanguageModel>, tools: ToolRegistry) -> Self {
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use talos_agent::Agent;
    /// use talos_core::tool::ToolRegistry;
    /// use talos_permission::PermissionEngine;
    /// use talos_sandbox::create_sandbox;
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    /// # use talos_core::provider::{LanguageModel, ProviderResult, Receiver};
    /// # use talos_core::message::{AgentEvent, Message};
    /// # struct MyModel;
    /// # #[async_trait::async_trait]
    /// # impl LanguageModel for MyModel {
    /// #     async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> { unimplemented!() }
    /// # }
    /// # async fn example() {
    /// let provider: Arc<dyn LanguageModel> = Arc::new(MyModel);
    /// let tools = ToolRegistry::new();
    /// let permission = PermissionEngine::new();
    /// let sandbox = talos_sandbox::create_sandbox();
    /// let agent = Agent::with_security(
    ///     provider,
    ///     tools,
    ///     Some(Arc::new(permission)),
    ///     Some(sandbox),
    ///     PathBuf::from("/tmp/workspace"),
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn with_security(
        provider: Arc<dyn LanguageModel>,
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
        provider: Arc<dyn LanguageModel>,
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
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_tools(tools);
        self.invalidate_stable_prefix_cache();
    }

    pub fn set_tool_protocol(&mut self, protocol: ToolProtocol) {
        match protocol {
            ToolProtocol::TalosStrict => {
                self.prompt_builder =
                    std::mem::take(&mut self.prompt_builder).with_strict_tool_format();
            }
            ToolProtocol::Compat => {
                self.prompt_builder = std::mem::take(&mut self.prompt_builder)
                    .with_tool_format(prompt::TOOL_CALLING_FORMAT);
            }
            ToolProtocol::Native => {
                self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_tool_format("");
            }
        }
        self.invalidate_stable_prefix_cache();
    }

    /// Sets the skill index for the system prompt builder.
    ///
    /// Only Level 0 metadata (name, description, triggers) is included.
    pub fn set_skill_index(&mut self, skills: Vec<SkillIndex>) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_skill_index(skills);
        self.invalidate_stable_prefix_cache();
    }

    /// Sets explicitly activated Level 1/2 Skill content for the system prompt.
    ///
    /// The caller must load, bound, and validate this content before passing it
    /// here. Changing activated Skill content invalidates the stable prefix.
    pub fn set_activated_skill_context(&mut self, context: Option<ActivatedSkillContext>) {
        self.prompt_builder =
            std::mem::take(&mut self.prompt_builder).with_activated_skill(context);
        self.invalidate_stable_prefix_cache();
    }

    /// Sets the context files for the system prompt builder.
    ///
    /// Typically loaded from `AGENTS.md` files via [`ContextLoader`].
    ///
    /// [`ContextLoader`]: crate::context::ContextLoader
    pub fn set_context_files(&mut self, files: Vec<ContextFile>) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_context_files(files);
    }

    /// Sets user-specific instructions for the system prompt builder.
    pub fn set_user_preferences(&mut self, prefs: String) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_user_preferences(prefs);
    }

    /// Sets a custom prompt that replaces the default identity.
    pub fn set_custom_prompt(&mut self, prompt: String) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_custom_prompt(prompt);
        self.invalidate_stable_prefix_cache();
    }

    /// Sets an append prompt that is added at the end of the system prompt.
    pub fn set_append_prompt(&mut self, prompt: String) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_append_prompt(prompt);
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

    /// Runs a single turn with the given user message and returns the complete
    /// assistant response.
    ///
    /// If the model emits tool calls during the turn, they are executed and
    /// results are fed back until the model produces a final text response.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError::ProviderError`] if the provider fails,
    /// [`AgentError::Cancelled`] if the cancellation token is triggered,
    /// [`AgentError::UnexpectedEvent`] if an error event is received,
    /// [`AgentError::TurnBudgetExceeded`] if the tool call budget is exceeded,
    /// or [`AgentError::DoomLoopDetected`] if a doom loop is detected.
    pub async fn run(&self, user_message: String) -> AgentResult<String> {
        let (text, _) = self.run_inner(user_message, vec![], None).await?;
        Ok(text)
    }

    /// Runs a single turn with streaming events forwarded to the given
    /// unbounded mpsc channel.
    ///
    /// This method behaves like [`Agent::run`] but also sends every
    /// [`AgentEvent`] to `event_tx`, allowing external consumers to receive
    /// real-time updates (e.g., for UI streaming).
    ///
    /// # Arguments
    ///
    /// * `user_message` — The current user message for this turn.
    /// * `history` — Prior conversation messages to include before the user message.
    /// * `event_tx` — Channel for streaming agent events.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Agent::run`].
    pub async fn run_streaming(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> AgentResult<(String, Vec<Message>)> {
        self.run_inner(user_message, history, Some(event_tx)).await
    }

    fn invalidate_stable_prefix_cache(&self) {
        *self
            .cached_stable_prefix
            .lock()
            .expect("cache lock poisoned") = None;
    }

    /// Internal implementation shared by [`run`] and [`run_streaming`].
    ///
    /// Executes the full turn loop: user message → provider → tool calls →
    /// execute → tool results → provider → ... → final response.
    async fn run_inner(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
    ) -> AgentResult<(String, Vec<Message>)> {
        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());

        const DEBUG_CMD: &str = "/mock-request";
        let is_debug = user_message.trim_start().starts_with(DEBUG_CMD);

        let actual_user_message = if is_debug {
            user_message.trim_start()[DEBUG_CMD.len()..]
                .trim()
                .to_string()
        } else {
            user_message
        };

        // Resolve the prompt builder: clone with memory section if provider is set.
        let prompt_builder = if let Some(ref mem_provider) = self.memory_provider {
            let memory_section = mem_provider(&actual_user_message);
            self.prompt_builder
                .clone()
                .with_memory_section(memory_section)
        } else {
            self.prompt_builder.clone()
        };

        let stable_prefix = {
            let mut cache = self
                .cached_stable_prefix
                .lock()
                .expect("cache lock poisoned");
            match cache.as_ref() {
                Some(cached) => cached.clone(),
                None => {
                    let prefix = prompt_builder.build_stable_prefix();
                    *cache = Some(prefix.clone());
                    prefix
                }
            }
        };
        let stable_prefix_len = stable_prefix.len();
        let dynamic_suffix = prompt_builder.build_dynamic_suffix();
        let combined = if stable_prefix.is_empty() {
            dynamic_suffix
        } else if dynamic_suffix.is_empty() {
            stable_prefix
        } else {
            format!("{stable_prefix}\n{dynamic_suffix}")
        };

        let (system_prompt, cache_markers) = match prompt_builder
            .build_with_hooks_from_prompt(
                self.hook_registry.as_ref(),
                &hook_ctx,
                &combined,
                stable_prefix_len,
            )
            .await
        {
            Ok(prompt) => prompt,
            Err(reason) => {
                let error = AgentError::HookDenied(reason);
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return Err(error);
            }
        };

        let mut messages = history;

        if !system_prompt.is_empty() {
            messages.push(Message::System {
                content: system_prompt,
                cache_markers,
            });
        }

        if let Some(ref context) = self.workspace_context
            && !context.is_empty()
        {
            messages.push(Message::Context {
                content: context.clone(),
            });
        }

        let persist_start = messages.len();

        messages.push(Message::User {
            content: actual_user_message,
        });

        if is_debug && let Some(preview) = self.provider.request_preview(&messages) {
            let snapshot =
                serde_json::to_string_pretty(&preview).unwrap_or_else(|_| preview.to_string());
            let result = format!("Request preview (no API call made):\n\n```json\n{snapshot}\n```");
            if let Some(ref tx) = event_tx {
                let _ = tx.send(AgentEvent::TurnStart);
                let _ = tx.send(AgentEvent::TextDelta {
                    delta: result.clone(),
                });
                let _ = tx.send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                });
            }
            return Ok((result, Vec::new()));
        }
        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();

        if let Err(error) = self
            .run_hook(&hook_ctx, HookEvent::TurnStart { turn_id })
            .await
        {
            self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
            return Err(error);
        }

        let (result, final_status) = 'turn_loop: loop {
            let provider_messages = match self
                .run_hook(
                    &hook_ctx,
                    HookEvent::BeforeProviderCall {
                        messages: &messages,
                    },
                )
                .await
            {
                Ok(HookOutcome::Continue(HookEvent::BeforeProviderCall { messages }))
                | Ok(HookOutcome::Skip(HookEvent::BeforeProviderCall { messages })) => messages,
                Ok(_) => messages.as_slice(),
                Err(error) => {
                    break (Err(error), TurnStatus::Denied);
                }
            };

            let mut rx = match self
                .provider
                .stream_with_tools(provider_messages, &self.tool_definitions)
                .await
            {
                Ok(rx) => rx,
                Err(error) => {
                    let _ = self
                        .run_hook(&hook_ctx, HookEvent::OnProviderError { error: &error })
                        .await;
                    break (
                        Err(AgentError::ProviderError(error)),
                        TurnStatus::ProviderError,
                    );
                }
            };

            let mut turn_tool_calls: Vec<PendingToolCall> = Vec::new();
            let mut turn_text = String::new();
            let mut saw_turn_end = false;
            let mut usage = talos_core::message::Usage::default();

            while let Some(event) = rx.recv().await {
                if let Some(ref tx) = event_tx
                    && !matches!(event, AgentEvent::ToolCall { .. })
                {
                    let _ = tx.send(event.clone());
                }

                match event {
                    AgentEvent::TextDelta { delta } => {
                        match self
                            .run_hook(&hook_ctx, HookEvent::OnTextDelta { text: &delta })
                            .await
                        {
                            Ok(HookOutcome::Continue(HookEvent::OnTextDelta { text }))
                            | Ok(HookOutcome::Skip(HookEvent::OnTextDelta { text })) => {
                                turn_text.push_str(text);
                            }
                            Ok(_) => turn_text.push_str(&delta),
                            Err(error) => {
                                break 'turn_loop (Err(error), TurnStatus::Denied);
                            }
                        }
                    }
                    AgentEvent::ToolCall {
                        call, provenance, ..
                    } => {
                        match self
                            .run_hook(&hook_ctx, HookEvent::OnToolCallProposed { call: &call })
                            .await
                        {
                            Ok(HookOutcome::Continue(HookEvent::OnToolCallProposed { call }))
                            | Ok(HookOutcome::Skip(HookEvent::OnToolCallProposed { call })) => {
                                turn_tool_calls.push(PendingToolCall {
                                    call: call.clone(),
                                    provenance,
                                });
                            }
                            Ok(_) => turn_tool_calls.push(PendingToolCall { call, provenance }),
                            Err(error) => {
                                break 'turn_loop (Err(error), TurnStatus::Denied);
                            }
                        }
                    }
                    AgentEvent::TurnEnd {
                        stop_reason,
                        usage: turn_usage,
                    } => {
                        saw_turn_end = true;
                        usage = turn_usage;
                        if usage.cache_read_tokens > 0 || usage.cache_write_tokens > 0 {
                            tracing::debug!(
                                cache_read = usage.cache_read_tokens,
                                cache_write = usage.cache_write_tokens,
                                input_tokens = usage.input_tokens,
                                "provider cache metadata"
                            );
                        }
                        let reason = Self::turn_end_reason(stop_reason);
                        if let Err(error) = self
                            .run_hook(&hook_ctx, HookEvent::OnTurnEnd { reason })
                            .await
                        {
                            break 'turn_loop (Err(error), TurnStatus::Denied);
                        }
                    }
                    AgentEvent::Error { message } => {
                        let provider_error = ProviderError::InvalidResponse(message.clone());
                        let _ = self
                            .run_hook(
                                &hook_ctx,
                                HookEvent::OnProviderError {
                                    error: &provider_error,
                                },
                            )
                            .await;
                        break 'turn_loop (
                            Err(AgentError::UnexpectedEvent(message)),
                            TurnStatus::UnexpectedEvent,
                        );
                    }
                    AgentEvent::TurnStart | AgentEvent::ToolResult { .. } => {}
                    _ => {}
                }
            }

            let _ = self
                .run_hook(
                    &hook_ctx,
                    HookEvent::AfterProviderCall {
                        tokens_in: usage.input_tokens,
                        tokens_out: usage.output_tokens,
                    },
                )
                .await;

            if !saw_turn_end {
                break 'turn_loop (
                    Err(AgentError::UnexpectedEvent(
                        "channel closed before TurnEnd".into(),
                    )),
                    TurnStatus::UnexpectedEvent,
                );
            }

            if turn_tool_calls.is_empty() {
                messages.push(Message::Assistant {
                    content: talos_core::message::strip_tool_syntax(&turn_text),
                    tool_calls: vec![],
                });
                let persisted = messages[persist_start..].to_vec();
                break (Ok((turn_text, persisted)), TurnStatus::Success);
            }

            let proposed_tool_calls: Vec<ToolCall> = turn_tool_calls
                .iter()
                .map(|pending| pending.call.clone())
                .collect();

            let effective_tool_calls = match self
                .run_hook(
                    &hook_ctx,
                    HookEvent::BeforeToolBatch {
                        calls: &proposed_tool_calls,
                    },
                )
                .await
            {
                Ok(HookOutcome::Continue(HookEvent::BeforeToolBatch { calls })) => calls.to_vec(),
                Ok(HookOutcome::Skip(_)) => Vec::new(),
                Ok(_) => proposed_tool_calls,
                Err(error) => {
                    break 'turn_loop (Err(error), TurnStatus::Denied);
                }
            };

            total_tool_calls += effective_tool_calls.len();
            if total_tool_calls > MAX_TOOL_CALLS_PER_TURN {
                let _ = self
                    .run_hook(
                        &hook_ctx,
                        HookEvent::OnBudgetExceeded {
                            kind: BudgetKind::ToolCalls,
                            used: total_tool_calls as u64,
                            limit: MAX_TOOL_CALLS_PER_TURN as u64,
                        },
                    )
                    .await;
                let persisted = messages[persist_start..].to_vec();
                break 'turn_loop (
                    Ok((
                        format!(
                            "Reached the per-turn tool call limit ({MAX_TOOL_CALLS_PER_TURN}). \
                             All results so far are preserved above — reply \"continue\" to resume."
                        ),
                        persisted,
                    )),
                    TurnStatus::BudgetExceeded,
                );
            }

            for call in &effective_tool_calls {
                let key = (call.name.clone(), call.input.to_string());
                let count = doom_tracker.entry(key).or_insert(0);
                *count += 1;
                if *count >= DOOM_LOOP_THRESHOLD {
                    let signature = format!(
                        "tool '{}' called {} times with identical arguments",
                        call.name, DOOM_LOOP_THRESHOLD
                    );
                    let _ = self
                        .run_hook(
                            &hook_ctx,
                            HookEvent::OnDoomLoopDetected {
                                signature: &signature,
                            },
                        )
                        .await;
                    break 'turn_loop (
                        Ok((
                            format!(
                                "Detected a repeated call pattern ({signature}). Paused for \
                                 review — all results are preserved above. Adjust your approach \
                                 and reply \"continue\" to resume."
                            ),
                            messages[persist_start..].to_vec(),
                        )),
                        TurnStatus::DoomLoopDetected,
                    );
                }
            }

            let cleaned_turn_text = talos_core::message::strip_tool_syntax(&turn_text);
            let assistant_msg = Message::Assistant {
                content: cleaned_turn_text,
                tool_calls: effective_tool_calls.clone(),
            };
            messages.push(assistant_msg);

            let tool_results = if let Some(ref tx) = event_tx {
                let effective_pending =
                    self.pending_calls_with_provenance(&effective_tool_calls, &turn_tool_calls);
                match self
                    .execute_tools_for_ui(&hook_ctx, &effective_pending, tx, &mut messages)
                    .await
                {
                    Ok(results) => results,
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                }
            } else {
                let tool_results = match self.execute_tools(&hook_ctx, &effective_tool_calls).await
                {
                    Ok(results) => results,
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                };

                for (call, result) in effective_tool_calls.iter().zip(tool_results.iter()) {
                    let observation = ToolObservation {
                        call: call.clone(),
                        result: result.clone(),
                    };
                    let observed = match self
                        .run_hook(
                            &hook_ctx,
                            HookEvent::OnToolResultObserved {
                                observation: &observation,
                            },
                        )
                        .await
                    {
                        Ok(HookOutcome::Continue(HookEvent::OnToolResultObserved {
                            observation,
                        }))
                        | Ok(HookOutcome::Skip(HookEvent::OnToolResultObserved { observation })) => {
                            observation.clone()
                        }
                        Ok(_) => observation,
                        Err(error) => {
                            break 'turn_loop (Err(error), TurnStatus::Denied);
                        }
                    };

                    let ui_result = MessageToolResult {
                        tool_use_id: observed.call.id.clone(),
                        content: observed.result.content.clone(),
                        is_error: observed.result.is_error,
                    };
                    let llm_result = if observed.result.is_error {
                        MessageToolResult {
                            content: format!(
                                "{}\n\n[Analyze the error above and try a different approach.]",
                                observed.result.content
                            ),
                            ..ui_result.clone()
                        }
                    } else {
                        ui_result.clone()
                    };
                    messages.push(Message::Tool { result: llm_result });
                }

                tool_results
            };

            let _ = self
                .run_hook(
                    &hook_ctx,
                    HookEvent::AfterToolBatch {
                        results: &tool_results,
                    },
                )
                .await;
        };

        self.emit_turn_complete(&hook_ctx, final_status).await;
        result
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
}

#[allow(warnings)]
#[cfg(test)]
mod tests;
