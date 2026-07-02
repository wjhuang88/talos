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
pub mod compression;
pub mod token;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub mod caching;
mod configuration;
pub mod context;
mod helpers;
pub mod prompt;
pub mod session;
mod tool_execution;

use talos_core::message::{AgentEvent, Message, MessageToolResult, ToolCall};
use talos_core::provider::{LanguageModel, ProviderError};
use talos_core::tool::{ToolPresentationPolicy, ToolProvenance, ToolRegistry};
use talos_permission::PermissionEngine;
use talos_plugin::{
    BudgetKind, HookContext, HookEvent, HookOutcome, HookRegistry, ToolObservation, TurnId,
    TurnStatus,
};
use talos_sandbox::SandboxProvider;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::compression::BashOutputCompressor;
use crate::configuration::describe_presented_tools;

pub use compression::{CompressionMetrics, RetrievalMetrics};
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
// Callback type for bounded session todo prompt injection.
type TodoSectionProviderCallback = dyn Fn() -> Option<String> + Send + Sync;

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
    /// Names of tools currently presented to the provider.
    presented_tool_names: HashSet<String>,
    /// Whether execution is restricted to provider-presented tools.
    enforce_tool_presentation_policy: bool,
    /// Current model-facing tool presentation policy.
    tool_presentation_policy: ToolPresentationPolicy,
    /// Cached stable prefix (Identity + Tools + Skills) computed once and
    /// reused across turns. Invalidated when tools, skills, or identity change.
    cached_stable_prefix: std::sync::Mutex<Option<String>>,
    /// Optional memory provider callback for injecting memory into prompts.
    memory_provider: Option<Arc<MemoryProviderCallback>>,
    /// Optional provider callback for injecting bounded active session todos.
    todo_section_provider: Option<Arc<TodoSectionProviderCallback>>,
    /// When true, bash tool output exceeding the line threshold is compressed
    /// before entering model context. Default: false.
    bash_compression_enabled: bool,
}

impl Agent {
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

    /// Builds a provider request preview without calling the provider.
    ///
    /// This is the explicit diagnostic API used by product layers that expose
    /// request-inspection commands. The normal turn loop treats all user
    /// messages literally and does not parse diagnostic magic strings.
    pub async fn preview_request(
        &self,
        user_message: String,
        history: Vec<Message>,
    ) -> AgentResult<Option<String>> {
        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());
        let (messages, _) = self
            .build_provider_messages(user_message, history, &hook_ctx)
            .await?;

        Ok(self.provider.request_preview(&messages).map(|preview| {
            let snapshot =
                serde_json::to_string_pretty(&preview).unwrap_or_else(|_| preview.to_string());
            format!("Request preview (no API call made):\n\n```json\n{snapshot}\n```")
        }))
    }

    async fn build_provider_messages(
        &self,
        user_message: String,
        history: Vec<Message>,
        hook_ctx: &HookContext,
    ) -> AgentResult<(Vec<Message>, usize)> {
        let mut prompt_builder = if let Some(ref mem_provider) = self.memory_provider {
            let memory_section = mem_provider(&user_message);
            self.prompt_builder
                .clone()
                .with_memory_section(memory_section)
        } else {
            self.prompt_builder.clone()
        };
        if let Some(ref todo_provider) = self.todo_section_provider {
            prompt_builder = prompt_builder.with_todo_section(todo_provider());
        }

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

        let (system_prompt, cache_markers) = prompt_builder
            .build_with_hooks_from_prompt(
                self.hook_registry.as_ref(),
                hook_ctx,
                &combined,
                stable_prefix_len,
            )
            .await
            .map_err(AgentError::HookDenied)?;

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
            content: user_message,
        });

        Ok((messages, persist_start))
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

        let (mut messages, persist_start) = match self
            .build_provider_messages(user_message, history, &hook_ctx)
            .await
        {
            Ok(messages) => messages,
            Err(error) => {
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return Err(error);
            }
        };

        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();
        let mut active_tool_presentation_policy = self.tool_presentation_policy.clone();
        let (_, mut active_tool_definitions, mut active_presented_tool_names) =
            describe_presented_tools(&self.tools, &active_tool_presentation_policy);

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
                .stream_with_tools(provider_messages, &active_tool_definitions)
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
                    .execute_tools_for_ui_with_presentation(
                        &hook_ctx,
                        &effective_pending,
                        tx,
                        &mut messages,
                        &active_tool_presentation_policy,
                        &active_presented_tool_names,
                    )
                    .await
                {
                    Ok(results) => results,
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                }
            } else {
                let tool_results = match self
                    .execute_tools_with_presentation(
                        &hook_ctx,
                        &effective_tool_calls,
                        &active_tool_presentation_policy,
                        &active_presented_tool_names,
                    )
                    .await
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
                    } else if self.bash_compression_enabled && observed.call.name == "bash" {
                        let compressed =
                            BashOutputCompressor::new().compress(&observed.result.content);
                        MessageToolResult {
                            content: compressed.content,
                            ..ui_result.clone()
                        }
                    } else {
                        ui_result.clone()
                    };
                    messages.push(Message::Tool { result: llm_result });
                }

                tool_results
            };

            self.apply_tool_continuations(
                &tool_results,
                &mut active_tool_presentation_policy,
                &mut active_tool_definitions,
                &mut active_presented_tool_names,
            );

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

    fn apply_tool_continuations(
        &self,
        results: &[talos_core::tool::ToolResult],
        policy: &mut ToolPresentationPolicy,
        tool_definitions: &mut Vec<talos_core::provider::ToolDefinition>,
        presented_tool_names: &mut HashSet<String>,
    ) {
        let mut changed = false;
        for continuation in results
            .iter()
            .flat_map(|result| result.continuations.iter())
        {
            if continuation.is_tool_disclosure() {
                if !policy.tools.iter().any(|tool| tool == &continuation.tool) {
                    policy.tools.push(continuation.tool.clone());
                    changed = true;
                }
            } else {
                let backend = &continuation.backend;
                if !policy.allows_backend(&continuation.tool, backend) {
                    policy
                        .backends
                        .push(talos_core::tool::ToolBackendDisclosure::new(
                            continuation.tool.clone(),
                            backend.clone(),
                        ));
                    changed = true;
                }
            }
        }

        if changed {
            let (_, definitions, names) = describe_presented_tools(&self.tools, policy);
            *tool_definitions = definitions;
            *presented_tool_names = names;
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests;
