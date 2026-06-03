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
pub mod prompt;
pub mod session;

use futures_util::future::join_all;
use talos_core::message::{
    AgentEvent, Message, StopReason, ToolCall, ToolResult as MessageToolResult,
};
use talos_core::provider::{LanguageModel, ProviderError};
use talos_core::tool::{ToolRegistry, ToolResult as ToolExecutionResult};
use talos_permission::{PermissionDecision, PermissionEngine};
use talos_plugin::{
    BudgetKind, HookContext, HookEvent, HookOutcome, HookRegistry, ToolObservation, TurnEndReason,
    TurnId, TurnStatus,
};
use talos_sandbox::{SandboxConfig, SandboxError, SandboxProvider, SandboxResult};
use talos_skill::SkillIndex;
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub use prompt::{ContextFile, SystemPromptBuilder, ToolDescription};

/// Maximum number of tool calls allowed per turn before budget exhaustion.
const MAX_TOOL_CALLS_PER_TURN: usize = 50;

/// Maximum number of concurrent read-only tool executions.
const MAX_CONCURRENT_READ_ONLY: usize = 10;

/// Threshold for doom loop detection — same tool+args this many times triggers
/// an early stop.
const DOOM_LOOP_THRESHOLD: u32 = 3;

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
            prompt_builder: SystemPromptBuilder::new(),
            hook_registry: Arc::new(HookRegistry::new()),
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
        Self {
            provider,
            tools,
            permission_engine,
            sandbox: sandbox.map(Arc::from),
            workspace_root,
            prompt_builder: SystemPromptBuilder::new(),
            hook_registry,
        }
    }

    /// Sets the tool descriptions for the system prompt builder.
    ///
    /// Tools are sorted alphabetically by name in the assembled prompt
    /// to ensure stable ordering across turns.
    pub fn set_tools(&mut self, tools: Vec<ToolDescription>) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_tools(tools);
    }

    /// Sets the skill index for the system prompt builder.
    ///
    /// Only Level 0 metadata (name, description, triggers) is included.
    pub fn set_skill_index(&mut self, skills: Vec<SkillIndex>) {
        self.prompt_builder = std::mem::take(&mut self.prompt_builder).with_skill_index(skills);
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
        self.run_inner(user_message, None).await
    }

    /// Runs a single turn with streaming events forwarded to the given
    /// broadcast channel.
    ///
    /// This method behaves like [`Agent::run`] but also sends every
    /// [`AgentEvent`] to `event_tx`, allowing external consumers to receive
    /// real-time updates (e.g., for UI streaming).
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Agent::run`].
    pub async fn run_streaming(
        &self,
        user_message: String,
        event_tx: broadcast::Sender<AgentEvent>,
    ) -> AgentResult<String> {
        self.run_inner(user_message, Some(event_tx)).await
    }

    /// Internal implementation shared by [`run`] and [`run_streaming`].
    ///
    /// Executes the full turn loop: user message → provider → tool calls →
    /// execute → tool results → provider → ... → final response.
    async fn run_inner(
        &self,
        user_message: String,
        event_tx: Option<broadcast::Sender<AgentEvent>>,
    ) -> AgentResult<String> {
        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());

        let system_prompt = match self
            .prompt_builder
            .build_with_hooks(self.hook_registry.as_ref(), &hook_ctx)
            .await
        {
            Ok(prompt) => prompt,
            Err(reason) => {
                let error = AgentError::HookDenied(reason);
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return Err(error);
            }
        };

        let full_message = if system_prompt.is_empty() {
            user_message
        } else {
            format!("{system_prompt}\n\n{user_message}")
        };

        let mut messages = vec![Message::User {
            content: full_message,
        }];
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

            let mut rx = match self.provider.stream(provider_messages).await {
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

            let mut turn_tool_calls: Vec<ToolCall> = Vec::new();
            let mut turn_text = String::new();
            let mut saw_turn_end = false;
            let mut usage = talos_core::message::Usage::default();

            while let Some(event) = rx.recv().await {
                if let Some(ref tx) = event_tx {
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
                    AgentEvent::ToolCall { call, .. } => {
                        match self
                            .run_hook(&hook_ctx, HookEvent::OnToolCallProposed { call: &call })
                            .await
                        {
                            Ok(HookOutcome::Continue(HookEvent::OnToolCallProposed { call }))
                            | Ok(HookOutcome::Skip(HookEvent::OnToolCallProposed { call })) => {
                                turn_tool_calls.push(call.clone());
                            }
                            Ok(_) => turn_tool_calls.push(call),
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
                break (Ok(turn_text), TurnStatus::Success);
            }

            let effective_tool_calls = match self
                .run_hook(
                    &hook_ctx,
                    HookEvent::BeforeToolBatch {
                        calls: &turn_tool_calls,
                    },
                )
                .await
            {
                Ok(HookOutcome::Continue(HookEvent::BeforeToolBatch { calls })) => calls.to_vec(),
                Ok(HookOutcome::Skip(_)) => Vec::new(),
                Ok(_) => turn_tool_calls.clone(),
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
                break 'turn_loop (
                    Err(AgentError::TurnBudgetExceeded),
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
                        Err(AgentError::DoomLoopDetected(signature)),
                        TurnStatus::DoomLoopDetected,
                    );
                }
            }

            let tool_results = match self.execute_tools(&hook_ctx, &effective_tool_calls).await {
                Ok(results) => results,
                Err(error) => {
                    break 'turn_loop (Err(error), TurnStatus::Denied);
                }
            };

            let _ = self
                .run_hook(
                    &hook_ctx,
                    HookEvent::AfterToolBatch {
                        results: &tool_results,
                    },
                )
                .await;

            let assistant_msg = Message::Assistant {
                content: turn_text.clone(),
                tool_calls: effective_tool_calls.clone(),
            };
            messages.push(assistant_msg);

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
                    Ok(HookOutcome::Continue(HookEvent::OnToolResultObserved { observation }))
                    | Ok(HookOutcome::Skip(HookEvent::OnToolResultObserved { observation })) => {
                        observation.clone()
                    }
                    Ok(_) => observation,
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                };

                let msg_result = MessageToolResult {
                    tool_use_id: observed.call.id.clone(),
                    content: observed.result.content.clone(),
                    is_error: observed.result.is_error,
                };
                messages.push(Message::Tool {
                    result: msg_result.clone(),
                });

                if let Some(ref tx) = event_tx {
                    let _ = tx.send(AgentEvent::ToolResult { result: msg_result });
                }
            }
        };

        self.emit_turn_complete(&hook_ctx, final_status).await;
        result
    }

    /// Executes a batch of tool calls, running read-only tools concurrently
    /// (up to [`MAX_CONCURRENT_READ_ONLY`]) and write tools serially.
    ///
    /// Each tool call goes through the security pipeline: permission check,
    /// sandbox execution (for bash tools), and direct execution.
    ///
    /// Results are returned in the same order as the input calls.
    async fn execute_tools(
        &self,
        hook_ctx: &HookContext,
        calls: &[ToolCall],
    ) -> AgentResult<Vec<ToolExecutionResult>> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<Option<ToolExecutionResult>> = vec![None; calls.len()];

        let read_only_indices: Vec<usize> = calls
            .iter()
            .enumerate()
            .filter(|(_, call)| {
                self.tools
                    .get(&call.name)
                    .map(|t| t.is_read_only())
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        let write_indices: Vec<usize> = calls
            .iter()
            .enumerate()
            .filter(|(_, call)| {
                !self
                    .tools
                    .get(&call.name)
                    .map(|t| t.is_read_only())
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        if !read_only_indices.is_empty() {
            let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_READ_ONLY));
            let registry = &self.tools;
            let futures: Vec<_> = read_only_indices
                .iter()
                .map(|&idx| {
                    let call = &calls[idx];
                    let sem = semaphore.clone();
                    let agent = self;
                    let ctx = hook_ctx.clone();
                    async move {
                        let _permit = sem.acquire().await.expect("semaphore closed");
                        let result = agent.execute_single_tool(&ctx, registry, call).await;
                        (idx, result)
                    }
                })
                .collect();

            for (idx, result) in join_all(futures).await {
                results[idx] = Some(result?);
            }
        }

        for idx in write_indices {
            let call = &calls[idx];
            let result = self
                .execute_single_tool(hook_ctx, &self.tools, call)
                .await?;
            results[idx] = Some(result);
        }

        Ok(results
            .into_iter()
            .map(|r| r.expect("all results should be populated"))
            .collect())
    }

    /// Executes a single tool call through the security pipeline.
    ///
    /// The pipeline is:
    /// 1. **Permission check** — if a permission engine is configured, evaluate
    ///    the call. `Allow` proceeds, `Deny` returns an error result, `Ask`
    ///    defaults to `Deny`.
    /// 2. **Sandbox execution** — for bash tools, if a sandbox is available,
    ///    execute through the sandbox. Falls back to direct execution if the
    ///    sandbox reports `NotAvailable`.
    /// 3. **Direct execution** — invoke the tool directly.
    ///
    /// Returns an error result if the tool is not found in the registry.
    async fn execute_single_tool(
        &self,
        hook_ctx: &HookContext,
        registry: &ToolRegistry,
        call: &ToolCall,
    ) -> AgentResult<ToolExecutionResult> {
        let effective_call = match self
            .run_hook(hook_ctx, HookEvent::BeforeToolCall { call })
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeToolCall { call })) => Some(call),
            Ok(HookOutcome::Skip(_)) => return Ok(ToolExecutionResult::success(String::new())),
            Ok(_) => Some(call),
            Err(error) => return Err(error),
        };
        let call = effective_call.expect("tool call should be present");

        if let Some(engine) = self.permission_engine.as_deref() {
            self.run_hook(hook_ctx, HookEvent::BeforePermissionCheck { call })
                .await?;

            let decision = engine.evaluate(&call.name, &call.input);
            self.run_hook(
                hook_ctx,
                HookEvent::AfterPermissionCheck {
                    call,
                    decision: decision.clone(),
                },
            )
            .await?;

            match decision {
                PermissionDecision::Allow => {}
                PermissionDecision::Deny(reason) => {
                    return Ok(ToolExecutionResult::error(format!(
                        "permission denied: {reason}"
                    )));
                }
                PermissionDecision::Ask => {
                    return Ok(ToolExecutionResult::error(format!(
                        "permission denied: tool '{}' requires approval (interactive approval not available at agent level)",
                        call.name
                    )));
                }
            }
        }

        let tool = match registry.get(&call.name) {
            Some(t) => t,
            None => {
                return Ok(ToolExecutionResult::error(format!(
                    "tool not found: {}",
                    call.name
                )));
            }
        };

        let result = if call.name == "bash" {
            if let Some(sb) = self.sandbox.as_deref() {
                if sb.is_available() {
                    self.execute_bash_in_sandbox(hook_ctx, sb, &call.input)
                        .await
                } else {
                    tool.execute(call.input.clone()).await
                }
            } else {
                tool.execute(call.input.clone()).await
            }
        } else {
            tool.execute(call.input.clone()).await
        };

        let result = match self
            .run_hook(
                hook_ctx,
                HookEvent::AfterToolCall {
                    call,
                    result: &result,
                },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::AfterToolCall { result, .. }))
            | Ok(HookOutcome::Skip(HookEvent::AfterToolCall { result, .. })) => result.clone(),
            Ok(_) => result,
            Err(error) => return Err(error),
        };

        Ok(result)
    }

    /// Executes a bash command through the sandbox provider.
    ///
    /// Extracts the `command` field from the tool input and runs it within
    /// the sandbox environment. Returns a [`ToolExecutionResult`] with the
    /// combined stdout/stderr output.
    async fn execute_bash_in_sandbox(
        &self,
        hook_ctx: &HookContext,
        sandbox: &dyn SandboxProvider,
        input: &serde_json::Value,
    ) -> ToolExecutionResult {
        let command = match input.get("command").and_then(serde_json::Value::as_str) {
            Some(cmd) => cmd.to_owned(),
            None => {
                return ToolExecutionResult::error(
                    "bash tool input missing required field 'command'".to_owned(),
                );
            }
        };

        let command = match self
            .run_hook(
                hook_ctx,
                HookEvent::BeforeBashSandboxExec { command: &command },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeBashSandboxExec { command })) => {
                command.to_string()
            }
            Ok(HookOutcome::Skip(_)) => return ToolExecutionResult::success(String::new()),
            Ok(_) => command,
            Err(_) => return ToolExecutionResult::error("hook denied bash execution".to_owned()),
        };

        let config = SandboxConfig {
            workspace_root: self.workspace_root.clone(),
            allow_network: false,
            extra_read_paths: vec![],
        };

        let start = std::time::Instant::now();

        let result = match sandbox.execute(&command, &config).await {
            Ok(result) => Self::sandbox_result_to_tool_result(result),
            Err(SandboxError::NotAvailable) => {
                ToolExecutionResult::error("sandbox became unavailable during execution".to_owned())
            }
            Err(SandboxError::ExecutionFailed(reason)) => {
                ToolExecutionResult::error(format!("sandbox execution failed: {reason}"))
            }
            Err(SandboxError::PermissionDenied(reason)) => {
                ToolExecutionResult::error(format!("sandbox permission denied: {reason}"))
            }
        };

        let exit = if result.is_error { 1 } else { 0 };
        let _ = self
            .run_hook(
                hook_ctx,
                HookEvent::AfterBashSandboxExec {
                    exit,
                    duration: start.elapsed(),
                },
            )
            .await;

        result
    }

    async fn run_hook<'a>(
        &self,
        hook_ctx: &HookContext,
        event: HookEvent<'a>,
    ) -> AgentResult<HookOutcome<'a>> {
        let outcome = self.hook_registry.dispatch(hook_ctx, event).await;
        if let HookOutcome::Deny { reason, .. } = &outcome {
            return Err(AgentError::HookDenied(reason.clone()));
        }
        Ok(outcome)
    }

    async fn emit_turn_complete(&self, hook_ctx: &HookContext, status: TurnStatus) {
        let _ = self
            .hook_registry
            .dispatch(
                hook_ctx,
                HookEvent::TurnComplete {
                    turn_id: hook_ctx.turn_id,
                    status,
                },
            )
            .await;
    }

    fn turn_end_reason(stop_reason: StopReason) -> TurnEndReason {
        match stop_reason {
            StopReason::EndTurn => TurnEndReason::EndTurn,
            StopReason::ToolUse => TurnEndReason::ToolUse,
            StopReason::MaxTokens => TurnEndReason::MaxTokens,
        }
    }

    /// Converts a [`SandboxResult`] to a [`ToolExecutionResult`].
    fn sandbox_result_to_tool_result(result: SandboxResult) -> ToolExecutionResult {
        let mut content = String::new();
        if !result.stdout.is_empty() {
            content.push_str(&result.stdout);
        }
        if !result.stderr.is_empty() {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&result.stderr);
        }

        ToolExecutionResult {
            content,
            is_error: result.exit_code != 0,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use talos_core::message::StopReason;
    use talos_core::provider::ProviderResult;
    use talos_core::tool::{AgentTool, ToolResult as ToolExecutionResult};
    use tokio::sync::Mutex;
    use tokio::sync::mpsc;

    type Receiver<T> = mpsc::Receiver<T>;

    /// Mock language model that returns a predefined sequence of event batches,
    /// one batch per call to `stream`.
    struct MockModel {
        responses: Arc<Mutex<Vec<Vec<AgentEvent>>>>,
    }

    impl MockModel {
        fn new(responses: Vec<Vec<AgentEvent>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
            }
        }
    }

    #[async_trait]
    impl LanguageModel for MockModel {
        async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
            let (tx, rx) = mpsc::channel(64);
            let responses = self.responses.clone();
            tokio::spawn(async move {
                let mut responses = responses.lock().await;
                let events = responses.pop_front().unwrap_or_default();
                for event in events {
                    tx.send(event).await.expect("receiver dropped");
                }
            });
            Ok(rx)
        }
    }

    trait VecDequeExt<T> {
        fn pop_front(&mut self) -> Option<T>;
    }

    impl<T> VecDequeExt<T> for Vec<T> {
        fn pop_front(&mut self) -> Option<T> {
            if self.is_empty() {
                None
            } else {
                Some(self.remove(0))
            }
        }
    }

    /// Mock tool that records execution timing and returns a fixed result.
    struct TimedMockTool {
        tool_name: String,
        read_only: bool,
        delay_ms: u64,
        result: ToolExecutionResult,
        execution_log: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl AgentTool for TimedMockTool {
        fn name(&self) -> &str {
            &self.tool_name
        }

        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({})
        }

        fn is_read_only(&self) -> bool {
            self.read_only
        }

        async fn execute(&self, input: Value) -> ToolExecutionResult {
            self.execution_log
                .lock()
                .await
                .push(format!("start:{}:{}", self.tool_name, input));
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            self.execution_log
                .lock()
                .await
                .push(format!("end:{}:{}", self.tool_name, input));
            self.result.clone()
        }
    }

    struct CountingHook {
        events: Arc<Mutex<Vec<talos_plugin::HookEventKind>>>,
    }

    #[async_trait]
    impl talos_plugin::HookHandler for CountingHook {
        fn name(&self) -> &str {
            "counting"
        }

        fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
            &[
                talos_plugin::HookEventKind::TurnStart,
                talos_plugin::HookEventKind::BeforeProviderCall,
                talos_plugin::HookEventKind::TurnComplete,
            ]
        }

        async fn on_event(
            &self,
            _ctx: &talos_plugin::HookContext,
            event: &mut talos_plugin::HookEvent<'_>,
        ) -> talos_plugin::HookResult {
            self.events.lock().await.push(event.kind());
            talos_plugin::HookResult::Continue
        }
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_run_collects_text_deltas() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Hello, ".into(),
            },
            AgentEvent::TextDelta {
                delta: "world!".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(vec![events])), ToolRegistry::new());
        let response = agent.run("Hi".into()).await.unwrap();
        assert_eq!(response, "Hello, world!");
    }

    #[tokio::test]
    async fn test_turn_start_hook_fires_once_for_tool_turn() {
        let call = ToolCall {
            id: "call-1".into(),
            name: "read".into(),
            input: serde_json::json!({}),
        };
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call,
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let events = Arc::new(Mutex::new(Vec::new()));
        let mut hooks = HookRegistry::new();
        hooks.register(Arc::new(CountingHook {
            events: events.clone(),
        }));

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "read".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("file content"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::with_security_and_hooks(
            Arc::new(MockModel::new(responses)),
            registry,
            Some(Arc::new(PermissionEngine::new())),
            None,
            PathBuf::from("/tmp"),
            Arc::new(hooks),
        );

        let response = agent.run("read file".into()).await.unwrap();
        assert_eq!(response, "done");

        let events = events.lock().await;
        let turn_start_count = events
            .iter()
            .filter(|kind| **kind == talos_plugin::HookEventKind::TurnStart)
            .count();
        let provider_call_count = events
            .iter()
            .filter(|kind| **kind == talos_plugin::HookEventKind::BeforeProviderCall)
            .count();
        assert_eq!(
            turn_start_count, 1,
            "TurnStart should fire once per user turn"
        );
        assert_eq!(
            provider_call_count, 2,
            "provider can be called multiple times in one user turn"
        );
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_run_handles_error_event() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::Error {
                message: "something went wrong".into(),
            },
        ];

        let agent = Agent::new(Arc::new(MockModel::new(vec![events])), ToolRegistry::new());
        let result = agent.run("Hi".into()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AgentError::UnexpectedEvent(_)));
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_run_handles_channel_close_without_turn_end() {
        let agent = Agent::new(Arc::new(MockModel::new(vec![])), ToolRegistry::new());
        let result = agent.run("Hi".into()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AgentError::UnexpectedEvent(_)));
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_run_streaming_forwards_events() {
        let events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Streaming".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ];

        let agent = Agent::new(
            Arc::new(MockModel::new(vec![events.clone()])),
            ToolRegistry::new(),
        );
        let (tx, mut rx) = broadcast::channel::<AgentEvent>(32);

        let response = agent.run_streaming("Hi".into(), tx).await.unwrap();
        assert_eq!(response, "Streaming");

        let mut received = Vec::new();
        while let Ok(event) = rx.try_recv() {
            received.push(event);
        }
        assert_eq!(received.len(), events.len());
        assert_eq!(received, events);
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_tool_execution_loop_single_call() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Let me check ".into(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "The result is: hello".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("hello"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let response = agent.run("Echo hello".into()).await.unwrap();
        assert_eq!(response, "The result is: hello");
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_tool_execution_loop_multiple_calls() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "read".into(),
                        input: serde_json::json!({ "path": "a.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_2".into(),
                        name: "read".into(),
                        input: serde_json::json!({ "path": "b.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Done reading both files".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "read".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("file content"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let response = agent.run("Read files".into()).await.unwrap();
        assert_eq!(response, "Done reading both files");
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_concurrent_read_only_tools() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "fast_read".into(),
                        input: serde_json::json!({ "path": "a.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_2".into(),
                        name: "fast_read".into(),
                        input: serde_json::json!({ "path": "b.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_3".into(),
                        name: "fast_read".into(),
                        input: serde_json::json!({ "path": "c.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "All done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "fast_read".into(),
            read_only: true,
            delay_ms: 50,
            result: ToolExecutionResult::success("ok"),
            execution_log: log.clone(),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let _response = agent.run("Read files".into()).await.unwrap();

        let log_entries = log.lock().await;
        let starts: Vec<_> = log_entries
            .iter()
            .filter(|e| e.starts_with("start:"))
            .collect();
        let ends: Vec<_> = log_entries
            .iter()
            .filter(|e| e.starts_with("end:"))
            .collect();

        assert_eq!(starts.len(), 3);
        assert_eq!(ends.len(), 3);

        let last_start_idx = log_entries
            .iter()
            .position(|e| e.starts_with("end:"))
            .unwrap_or(3);
        assert!(
            last_start_idx >= 3,
            "Expected all starts before any end, but log was: {:?}",
            log_entries
        );
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_serial_write_tools() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "write".into(),
                        input: serde_json::json!({ "path": "a.txt", "content": "a" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_2".into(),
                        name: "write".into(),
                        input: serde_json::json!({ "path": "b.txt", "content": "b" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Files written".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "write".into(),
            read_only: false,
            delay_ms: 30,
            result: ToolExecutionResult::success("written"),
            execution_log: log.clone(),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let _response = agent.run("Write files".into()).await.unwrap();

        let log_entries = log.lock().await;
        assert_eq!(log_entries.len(), 4);

        assert!(log_entries[0].starts_with("start:"));
        assert!(log_entries[1].starts_with("end:"));
        assert!(log_entries[2].starts_with("start:"));
        assert!(log_entries[3].starts_with("end:"));
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_turn_budget_enforcement() {
        let mut events = vec![AgentEvent::TurnStart];
        for i in 0..51 {
            events.push(AgentEvent::ToolCall {
                call: ToolCall {
                    id: format!("call_{i}"),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": format!("msg_{i}") }),
                },
                provenance: Default::default(),
            });
        }
        events.push(AgentEvent::TurnEnd {
            stop_reason: StopReason::ToolUse,
            usage: talos_core::message::Usage::default(),
        });

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("ok"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(vec![events])), registry);
        let result = agent.run("Many tools".into()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::TurnBudgetExceeded
        ));
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_turn_budget_allows_50_calls() {
        let mut tool_events = vec![AgentEvent::TurnStart];
        for i in 0..50 {
            tool_events.push(AgentEvent::ToolCall {
                call: ToolCall {
                    id: format!("call_{i}"),
                    name: "echo".into(),
                    input: serde_json::json!({ "message": format!("msg_{i}") }),
                },
                provenance: Default::default(),
            });
        }
        tool_events.push(AgentEvent::TurnEnd {
            stop_reason: StopReason::ToolUse,
            usage: talos_core::message::Usage::default(),
        });

        let text_events = vec![
            AgentEvent::TurnStart,
            AgentEvent::TextDelta {
                delta: "Done".into(),
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: talos_core::message::Usage::default(),
            },
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("ok"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(
            Arc::new(MockModel::new(vec![tool_events, text_events])),
            registry,
        );
        let result = agent.run("50 tools".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Done");
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_doom_loop_detection() {
        let tool_call_event = AgentEvent::ToolCall {
            call: ToolCall {
                id: "call_1".into(),
                name: "echo".into(),
                input: serde_json::json!({ "message": "same" }),
            },
            provenance: Default::default(),
        };

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                tool_call_event.clone(),
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                tool_call_event.clone(),
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                tool_call_event,
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("same"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let result = agent.run("Loop".into()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::DoomLoopDetected(_)
        ));
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_doom_loop_different_args_allowed() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "first" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_2".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "second" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("ok"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let result = agent.run("Different args".into()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_tool_not_found_returns_error_result() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "nonexistent_tool".into(),
                        input: serde_json::json!({}),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Tool not available".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let agent = Agent::new(Arc::new(MockModel::new(responses)), ToolRegistry::new());
        let result = agent.run("Missing tool".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Tool not available");
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_tool_execution_error_feeds_back_to_provider() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "failing".into(),
                        input: serde_json::json!({}),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Tool failed, trying alternative".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "failing".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::error("internal failure"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let result = agent.run("Failing tool".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Tool failed, trying alternative");
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_mixed_read_only_and_write_tools() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "read".into(),
                        input: serde_json::json!({ "path": "a.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_2".into(),
                        name: "write".into(),
                        input: serde_json::json!({ "path": "b.txt", "content": "b" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_3".into(),
                        name: "read".into(),
                        input: serde_json::json!({ "path": "c.txt" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Mixed tools done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "read".into(),
            read_only: true,
            delay_ms: 20,
            result: ToolExecutionResult::success("read ok"),
            execution_log: log.clone(),
        }));
        registry.register(Arc::new(TimedMockTool {
            tool_name: "write".into(),
            read_only: false,
            delay_ms: 20,
            result: ToolExecutionResult::success("write ok"),
            execution_log: log.clone(),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let result = agent.run("Mixed".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mixed tools done");

        let log_entries = log.lock().await;
        let write_start_idx = log_entries
            .iter()
            .position(|e| e.starts_with("start:write:"))
            .unwrap();
        let write_end_idx = log_entries
            .iter()
            .position(|e| e.starts_with("end:write:"))
            .unwrap();
        assert_eq!(
            write_end_idx,
            write_start_idx + 1,
            "Write tool should be serial: {:?}",
            log_entries
        );
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_cancellation_token_is_created() {
        let agent = Agent::new(Arc::new(MockModel::new(vec![])), ToolRegistry::new());
        let token = agent.cancellation_token();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    async fn test_tool_result_events_broadcast() {
        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "test" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("test result"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::new(Arc::new(MockModel::new(responses)), registry);
        let (tx, mut rx) = broadcast::channel::<AgentEvent>(32);

        let _response = agent.run_streaming("Echo test".into(), tx).await.unwrap();

        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        let tool_result_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ToolResult { .. }))
            .collect();
        assert_eq!(
            tool_result_events.len(),
            1,
            "Expected 1 ToolResult event, got: {:?}",
            events
        );
    }

    /// Mock sandbox that tracks execution and returns configurable results.
    struct MockSandbox {
        available: bool,
        execution_log: Arc<Mutex<Vec<String>>>,
        result: Option<SandboxResult>,
    }

    impl MockSandbox {
        fn new(available: bool, result: SandboxResult) -> Self {
            Self {
                available,
                execution_log: Arc::new(Mutex::new(Vec::new())),
                result: Some(result),
            }
        }

        fn unavailable() -> Self {
            Self {
                available: false,
                execution_log: Arc::new(Mutex::new(Vec::new())),
                result: None,
            }
        }
    }

    #[async_trait]
    impl SandboxProvider for MockSandbox {
        async fn execute(
            &self,
            command: &str,
            _config: &SandboxConfig,
        ) -> Result<SandboxResult, SandboxError> {
            self.execution_log
                .lock()
                .await
                .push(format!("sandbox_execute:{command}"));
            if !self.available {
                return Err(SandboxError::NotAvailable);
            }
            Ok(self.result.clone().unwrap_or_else(|| SandboxResult {
                stdout: "sandboxed".into(),
                stderr: String::new(),
                exit_code: 0,
            }))
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[tokio::test]
    async fn test_permission_check_blocks_denied_tool() {
        let mut engine = PermissionEngine { rules: Vec::new() };
        engine.add_rule(talos_permission::PermissionRule {
            tool_name: "echo".into(),
            path_pattern: None,
            decision: PermissionDecision::Deny("not allowed".into()),
        });

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("should not reach"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            Some(Arc::new(engine)),
            None,
            PathBuf::from("/tmp"),
        );

        let result = agent.run("Test".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Done");
    }

    #[tokio::test]
    async fn test_permission_check_allows_permitted_tool() {
        let mut engine = PermissionEngine { rules: Vec::new() };
        engine.add_rule(talos_permission::PermissionRule {
            tool_name: "echo".into(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Result: hello".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("hello"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            Some(Arc::new(engine)),
            None,
            PathBuf::from("/tmp"),
        );

        let result = agent.run("Test".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Result: hello");
    }

    #[tokio::test]
    async fn test_permission_ask_defaults_to_deny() {
        let mut engine = PermissionEngine { rules: Vec::new() };
        engine.add_rule(talos_permission::PermissionRule {
            tool_name: "echo".into(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "echo".into(),
                        input: serde_json::json!({ "message": "hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Denied".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "echo".into(),
            read_only: true,
            delay_ms: 0,
            result: ToolExecutionResult::success("should not reach"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            Some(Arc::new(engine)),
            None,
            PathBuf::from("/tmp"),
        );

        let result = agent.run("Test".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Denied");
    }

    #[tokio::test]
    async fn test_sandbox_execution_for_bash_tool() {
        let sandbox_result = SandboxResult {
            stdout: "sandboxed output".into(),
            stderr: String::new(),
            exit_code: 0,
        };
        let sandbox = MockSandbox::new(true, sandbox_result);
        let log = sandbox.execution_log.clone();

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "bash".into(),
                        input: serde_json::json!({ "command": "echo hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Done".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "bash".into(),
            read_only: false,
            delay_ms: 0,
            result: ToolExecutionResult::success("direct execution"),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }));

        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            None,
            Some(Box::new(sandbox)),
            PathBuf::from("/tmp"),
        );

        let result = agent.run("Test".into()).await;
        assert!(result.is_ok());

        let log_entries = log.lock().await;
        assert!(
            log_entries.iter().any(|e| e.contains("echo hello")),
            "Sandbox should have been called with the command, log: {:?}",
            log_entries
        );
    }

    #[tokio::test]
    async fn test_sandbox_fallback_when_not_available() {
        let sandbox = MockSandbox::unavailable();

        let responses = vec![
            vec![
                AgentEvent::TurnStart,
                AgentEvent::ToolCall {
                    call: ToolCall {
                        id: "call_1".into(),
                        name: "bash".into(),
                        input: serde_json::json!({ "command": "echo hello" }),
                    },
                    provenance: Default::default(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::ToolUse,
                    usage: talos_core::message::Usage::default(),
                },
            ],
            vec![
                AgentEvent::TurnStart,
                AgentEvent::TextDelta {
                    delta: "Fallback worked".into(),
                },
                AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: talos_core::message::Usage::default(),
                },
            ],
        ];

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(TimedMockTool {
            tool_name: "bash".into(),
            read_only: false,
            delay_ms: 0,
            result: ToolExecutionResult::success("direct execution"),
            execution_log: log.clone(),
        }));

        let agent = Agent::with_security(
            Arc::new(MockModel::new(responses)),
            registry,
            None,
            Some(Box::new(sandbox)),
            PathBuf::from("/tmp"),
        );

        let result = agent.run("Test".into()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Fallback worked");

        let log_entries = log.lock().await;
        assert!(
            log_entries.iter().any(|e| e.starts_with("start:bash:")),
            "Direct execution should have been used as fallback, log: {:?}",
            log_entries
        );
    }

    #[test]
    fn test_agent_with_security_constructor() {
        let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
        let tools = ToolRegistry::new();
        let permission = PermissionEngine::new();

        let agent = Agent::with_security(
            provider.clone(),
            tools,
            Some(Arc::new(permission)),
            None,
            PathBuf::from("/tmp/workspace"),
        );

        let _token = agent.cancellation_token();
        assert!(!_token.is_cancelled());
    }

    #[test]
    #[allow(deprecated)] // Agent::new is correct for unit tests
    fn test_agent_new_has_no_security() {
        let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
        let tools = ToolRegistry::new();

        let agent = Agent::new(provider, tools);
        let _token = agent.cancellation_token();
        assert!(!_token.is_cancelled());
    }

    #[tokio::test]
    async fn test_sandbox_result_to_tool_result_success() {
        let sandbox_result = SandboxResult {
            stdout: "hello".into(),
            stderr: "warning".into(),
            exit_code: 0,
        };

        let tool_result = Agent::sandbox_result_to_tool_result(sandbox_result);
        assert!(!tool_result.is_error);
        assert!(tool_result.content.contains("hello"));
        assert!(tool_result.content.contains("warning"));
    }

    #[tokio::test]
    async fn test_sandbox_result_to_tool_result_error() {
        let sandbox_result = SandboxResult {
            stdout: String::new(),
            stderr: "error occurred".into(),
            exit_code: 1,
        };

        let tool_result = Agent::sandbox_result_to_tool_result(sandbox_result);
        assert!(tool_result.is_error);
        assert!(tool_result.content.contains("error occurred"));
    }

    #[tokio::test]
    async fn test_execute_bash_in_sandbox_missing_command() {
        let sandbox = MockSandbox::new(
            true,
            SandboxResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );

        let input = serde_json::json!({});
        let agent = Agent::with_security(
            Arc::new(MockModel::new(vec![])),
            ToolRegistry::new(),
            None,
            Some(Box::new(sandbox)),
            PathBuf::from("/tmp"),
        );
        let hook_ctx = HookContext::new(TurnId::new(), PathBuf::from("/tmp"));

        let result = agent
            .execute_bash_in_sandbox(
                &hook_ctx,
                agent.sandbox.as_deref().expect("sandbox should be present"),
                &input,
            )
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("missing required field 'command'"));
    }

    #[test]
    fn test_clear_append_prompt() {
        let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
        let tools = ToolRegistry::new();

        let mut agent = Agent::new(provider, tools);

        // Set an append prompt
        agent.set_append_prompt("test".to_string());

        // Verify it is set
        let prompt = agent.prompt_builder.build();
        assert!(prompt.contains("test"), "Append prompt should be set");

        // Clear the append prompt
        agent.clear_append_prompt();

        // Verify it is cleared
        let prompt = agent.prompt_builder.build();
        assert!(
            !prompt.contains("test"),
            "Append prompt should be cleared after clear_append_prompt()"
        );
    }

    #[test]
    fn test_set_append_prompt_opt_none() {
        let provider: Arc<dyn LanguageModel> = Arc::new(MockModel::new(vec![]));
        let tools = ToolRegistry::new();

        let mut agent = Agent::new(provider, tools);

        // Set an append prompt
        agent.set_append_prompt("test".to_string());

        // Verify it is set
        let prompt = agent.prompt_builder.build();
        assert!(prompt.contains("test"), "Append prompt should be set");

        // Clear using set_append_prompt_opt(None)
        agent.set_append_prompt_opt(None);

        // Verify it is cleared (same as clear_append_prompt)
        let prompt = agent.prompt_builder.build();
        assert!(
            !prompt.contains("test"),
            "Append prompt should be cleared after set_append_prompt_opt(None)"
        );
    }
}
