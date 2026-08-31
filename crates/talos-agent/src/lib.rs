//! Talos agent — core orchestration logic and the agent turn loop.
//!
//! The agent manages a conversation turn with an LLM provider, executing tool
//! calls when the model requests them and feeding results back until a final
//! text response is produced.
//!
//! # Security Pipeline
//!
//! Every tool call goes through a security pipeline:
//! 1. **Permission pipeline** — the Agent normalizes, evaluates, resolves and
//!    admits the exact request through [`permission_pipeline::PermissionPipeline`]
//! 2. **Final permission hook** — the admitted Allow or final Deny gates execution
//! 3. **Sandbox execution** — bash tools run through the sandbox when available
//! 4. **Execute** — the tool receives the admitted authorization
//! 5. **Retry on denial** — denied calls return an error result
//!
//! The `Ask` decision defaults to `Deny` at the agent level. Both the CLI layer
//! and an embedded runtime may bridge `Ask` to an interactive approval handler;
//! with no approval handler configured, `Ask` still fails closed (`Deny`).
//!
//! # Support Boundary
//!
//! This crate owns the **turn-loop implementation**. It may be published to
//! crates.io only to satisfy the `talos-runtime` dependency closure under
//! [ADR-052](../../docs/decisions/052-sdk-publication-and-composition-boundary.md)
//! (route A). It is **not** a recommended or supported SDK entrypoint.
//!
//! - Embedders should use `talos_runtime::RuntimeBuilder` (in the `talos-runtime`
//!   facade crate) to construct a safe runtime that wraps permission, approval,
//!   and sandbox policy.
//! - Direct users of `talos-agent` bypass that wrapping and are themselves
//!   responsible for installing equivalent permission rules, an approval
//!   handler, and a sandbox policy.
//! - Its public constructors and configuration methods are NOT covered by the
//!   runtime SDK contract and may change more frequently than the facade
//!   surface during the pre-1.0 period.
//!
//! See `docs/reference/RUNTIME-SDK-CONTRACT.md` for the supported embedding
//! surface.

mod background_jobs;
pub mod compaction;
pub mod compression;
mod process_tool;
pub mod token;
mod tool_output;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub mod auto_resolver;
pub mod caching;
mod configuration;
pub mod context;
pub mod evaluator;
mod helpers;
pub mod permission_pipeline;
pub mod prompt;
mod request_plan;
mod scheduler;
pub mod session;
mod tool_execution;

pub use scheduler::{
    PendingSchedulerActor, create_delay_tool_and_scheduler, create_scheduler_tools,
};

use talos_core::message::{
    AgentEvent, AssistantReasoning, Message, MessageToolResult, ReasoningBlock, StopReason,
    ToolCall,
};
use talos_core::provider::{LanguageModel, ProviderError};
use talos_core::tool::{ToolPresentationPolicy, ToolProvenance, ToolRegistry};
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
pub(crate) use request_plan::PreparedSessionTurn;

/// Maximum number of tool calls allowed per turn before budget exhaustion.
const MAX_TOOL_CALLS_PER_TURN: usize = 50;

/// Maximum number of concurrent read-only tool executions.
const MAX_CONCURRENT_READ_ONLY: usize = 10;

/// Threshold for doom loop detection — same tool+args this many times triggers
/// an early stop.
const DOOM_LOOP_THRESHOLD: u32 = 3;

fn should_compress_shell_output(tool_name: &str) -> bool {
    matches!(tool_name, "bash" | "powershell")
}

/// Shared admission contract for one complete Provider request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBudgetSpec {
    /// Exact output token limit requested in the Provider body.
    pub requested_output_tokens: u32,
    /// Conservative margin applied to approximate text/tool/image input cost.
    pub input_safety_margin_bps: u16,
    /// Fixed parser/protocol overhead added after the proportional margin.
    pub fixed_overhead_tokens: u32,
}

impl RequestBudgetSpec {
    #[must_use]
    pub const fn new(requested_output_tokens: u32) -> Self {
        Self {
            requested_output_tokens,
            input_safety_margin_bps: 2_500,
            fixed_overhead_tokens: 256,
        }
    }
}

impl Default for RequestBudgetSpec {
    fn default() -> Self {
        Self::new(4096)
    }
}

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

    /// The next provider request would exceed the configured model context.
    #[error("request context budget exceeded: estimated {estimated} tokens, limit {limit}")]
    ContextBudgetExceeded {
        /// Estimated request tokens, including tool definitions and output reserve.
        estimated: u32,
        /// Configured model context limit.
        limit: u32,
    },

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

/// Controls what happens when a configured sandbox is unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SandboxFallbackPolicy {
    /// Reject the invocation when isolation is unavailable.
    #[default]
    Deny,
    /// Ask a dedicated fallback handler for a one-invocation approval.
    Ask,
    /// Continue without isolation after permission has already allowed the tool.
    AllowUnsandboxed,
}

/// A redacted, typed request for a sandbox fallback decision.
#[derive(Debug, Clone, PartialEq)]
pub struct SandboxFallbackContext {
    /// The tool requiring the fallback.
    pub tool_name: String,
    /// Observer-safe tool input for the approval surface.
    pub arguments: serde_json::Value,
    /// Stable summary fields for display or audit projection.
    pub summary_fields: Vec<String>,
}

/// A dedicated sandbox fallback decision. It intentionally has no permanent
/// or reusable approval variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxFallbackDecision {
    /// Approve only the current invocation.
    ApproveOnce,
    /// Reject the fallback.
    Deny,
}

/// Resolves typed sandbox fallback requests independently of normal tool
/// permission approval.
#[async_trait::async_trait]
pub trait SandboxFallbackHandler: Send + Sync {
    /// Decides whether this one fallback invocation may proceed.
    async fn request_fallback(&self, context: SandboxFallbackContext) -> SandboxFallbackDecision;
}

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
/// sandbox environment. If the sandbox is unavailable, the configured
/// [`SandboxFallbackPolicy`] decides whether the invocation is denied or may
/// continue through an explicitly approved unsandboxed path.
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
    provider: Arc<dyn LanguageModel>,
    tools: ToolRegistry,
    /// Agent-owned permission pipeline used by migrated composition roots.
    permission_pipeline: Option<Arc<permission_pipeline::PermissionPipeline>>,
    /// Total budget for permission hooks, resolution, and admission.
    permission_deadline: std::time::Duration,
    /// Optional sandbox provider for bash tool execution.
    sandbox: Option<Arc<dyn SandboxProvider>>,
    /// Policy used only when a configured sandbox reports unavailable.
    sandbox_fallback_policy: SandboxFallbackPolicy,
    /// Dedicated handler for one-shot sandbox fallback approval.
    sandbox_fallback_handler: Option<Arc<dyn SandboxFallbackHandler>>,
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
    /// Config provider key for reasoning origin stamping and replay gating.
    provider_key: Option<String>,
    /// Model id for reasoning origin stamping and replay gating.
    model_id: Option<String>,
    /// Whether to replay reasoning in request history (ADR-034 replay policy).
    replay_reasoning: bool,
    /// When true, bash tool output exceeding the line threshold is compressed
    /// before entering model context. Default: false.
    bash_compression_enabled: bool,
    tool_output_threshold: usize,
    /// Whether the active model supports image input. When false, the
    /// `read_image` tool is registered but not presented to the model
    /// (ADR-051 / I154 capability gate).
    image_input_supported: bool,
    /// Exact output reserve and conservative input-estimation policy.
    request_budget_spec: RequestBudgetSpec,
    background_jobs: Option<Arc<dyn talos_core::background_job::BackgroundJobHost>>,
}
impl Agent {
    pub(crate) fn set_background_job_host(
        &mut self,
        host: Arc<dyn talos_core::background_job::BackgroundJobHost>,
    ) {
        self.background_jobs = Some(host);
    }

    pub(crate) fn register_process_tool(
        &mut self,
        supervisor: crate::background_jobs::BackgroundJobSupervisor,
    ) {
        self.tools
            .register(Arc::new(crate::process_tool::ProcessTool::new(supervisor)));
        let (descriptions, definitions, names) = crate::configuration::describe_presented_tools(
            &self.tools,
            &self.tool_presentation_policy,
        );
        self.tool_definitions = definitions;
        self.presented_tool_names = names;
        self.update_prompt_builder(true, |builder| builder.with_tools(descriptions));
    }

    pub fn provider(&self) -> &dyn LanguageModel {
        self.provider.as_ref()
    }

    /// Runs a single turn with the given user message and returns the complete
    /// assistant response.
    ///
    /// If the model emits tool calls during the turn, they are executed and
    /// results are fed back until the model produces a final text response.
    /// [`AgentError::TurnBudgetExceeded`] if the tool call budget is exceeded,
    /// or [`AgentError::DoomLoopDetected`] if a doom loop is detected.
    pub async fn run(&self, user_message: String) -> AgentResult<String> {
        let (result, _) = self.run_inner(user_message, vec![], None, None).await;
        result
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
        let (result, messages) = self
            .run_inner(user_message, history, Some(event_tx), None)
            .await;
        result.map(|text| (text, messages))
    }

    /// Like [`run_streaming`] but returns partial messages even on error,
    /// enabling the session layer to persist valid completed tool exchanges
    /// across provider failures (SESSION-006 / I135).
    ///
    /// The returned messages are always the normalized slice from
    /// `persist_start..` — i.e., the user message and any completed
    /// assistant/tool messages. On error, this may contain a valid prefix
    /// of completed exchanges that should be persisted; incomplete streamed
    /// assistant fragments are never included.
    #[allow(dead_code)]
    pub(crate) async fn run_for_session_turn(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> (AgentResult<String>, Vec<Message>) {
        self.run_inner(user_message, history, Some(event_tx), None)
            .await
    }

    /// Session turn with multimodal content (MODEL-009-D/I152).
    /// Constructs `Message::Multimodal` instead of `Message::User`
    /// when image attachments are present.
    #[allow(dead_code)]
    pub(crate) async fn run_for_session_turn_multimodal(
        &self,
        user_message: String,
        attachments: Vec<talos_core::message::ContentPart>,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> (AgentResult<String>, Vec<Message>) {
        self.run_inner(
            user_message,
            history,
            Some(event_tx),
            if attachments.is_empty() {
                None
            } else {
                Some(attachments)
            },
        )
        .await
    }

    /// Runs one actor turn from an ordered structured submission.
    ///
    /// Each item remains a distinct persisted user message. Text projection is
    /// used only for memory lookup; it is never the authoritative transcript.
    /// Estimates the initial provider request produced for a structured
    /// session submission, including dynamic prompt sections, workspace
    /// context, native tool definitions, multimodal inputs, and an output
    /// reserve. This remains a diagnostic estimate only; Session execution is
    /// authorized by the sealed plan returned from `prepare_session_turn`.
    #[allow(dead_code)]
    pub(crate) async fn estimate_session_request_tokens(
        &self,
        items: &[talos_core::session::SubmissionItem],
        history: Vec<Message>,
    ) -> AgentResult<u32> {
        let memory_query = items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let hook_ctx = HookContext::new(TurnId::new(), self.workspace_root.clone());
        let (mut messages, _) = self
            .build_provider_messages(memory_query, history, &hook_ctx)
            .await?;
        messages.pop();
        messages.extend(items.iter().map(|item| {
            if item.attachments.is_empty() {
                Message::User {
                    content: item.text.clone(),
                }
            } else {
                let mut parts = Vec::with_capacity(item.attachments.len() + 1);
                if !item.text.is_empty() {
                    parts.push(talos_core::message::ContentPart::Text {
                        text: item.text.clone(),
                    });
                }
                parts.extend(item.attachments.clone());
                Message::Multimodal { parts }
            }
        }));

        let (_, mut tool_definitions, _) =
            describe_presented_tools(&self.tools, &self.tool_presentation_policy);
        if !self.image_input_supported {
            tool_definitions.retain(|definition| definition.name != "read_image");
        }
        Ok(self.estimate_provider_request_tokens(&messages, &tool_definitions))
    }

    fn estimate_provider_request_tokens(
        &self,
        messages: &[Message],
        tool_definitions: &[talos_core::provider::ToolDefinition],
    ) -> u32 {
        let tool_tokens = tool_definitions.iter().fold(0_u32, |total, definition| {
            total
                .saturating_add(crate::token::TokenEstimator::estimate_text(
                    &definition.name,
                ))
                .saturating_add(crate::token::TokenEstimator::estimate_text(
                    &definition.description,
                ))
                .saturating_add(crate::token::TokenEstimator::estimate_text(
                    &definition.parameters.to_string(),
                ))
        });
        let raw_input = crate::token::TokenEstimator::new()
            .estimate(messages)
            .saturating_add(tool_tokens);
        let proportional_margin = u64::from(raw_input)
            .saturating_mul(u64::from(self.request_budget_spec.input_safety_margin_bps))
            .div_ceil(10_000);
        raw_input
            .saturating_add(u32::try_from(proportional_margin).unwrap_or(u32::MAX))
            .saturating_add(self.request_budget_spec.fixed_overhead_tokens)
            .saturating_add(self.request_budget_spec.requested_output_tokens)
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

    /// Like `build_provider_messages` but pushes `Message::Multimodal`
    /// instead of `Message::User` when attachments are present.
    #[allow(dead_code)]
    async fn build_provider_messages_with_attachments(
        &self,
        user_message: String,
        history: Vec<Message>,
        hook_ctx: &HookContext,
        attachments: Vec<talos_core::message::ContentPart>,
    ) -> AgentResult<(Vec<Message>, usize)> {
        let (mut messages, persist_start) = self
            .build_provider_messages(user_message, history, hook_ctx)
            .await?;

        if let Some(Message::User { content: _ }) = messages.last() {
            let mut parts = Vec::new();
            if let Some(Message::User { content }) = messages.last_mut()
                && !content.is_empty()
            {
                parts.push(talos_core::message::ContentPart::Text {
                    text: content.clone(),
                });
            }
            parts.extend(attachments);
            if let Some(last) = messages.last_mut() {
                *last = Message::Multimodal { parts };
            }
        }

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
        attachments: Option<Vec<talos_core::message::ContentPart>>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let input_messages = if let Some(atts) = attachments {
            let mut parts = Vec::with_capacity(atts.len() + 1);
            if !user_message.is_empty() {
                parts.push(talos_core::message::ContentPart::Text {
                    text: user_message.clone(),
                });
            }
            parts.extend(atts);
            vec![Message::Multimodal { parts }]
        } else {
            vec![Message::User {
                content: user_message.clone(),
            }]
        };
        self.run_inner_with_messages(user_message, input_messages, history, event_tx, None)
            .await
    }

    async fn run_inner_with_messages(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
        request_context_limit: Option<u32>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let prepared = match self
            .prepare_turn_start(memory_query, input_messages, history, request_context_limit)
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => return (Err(error), Vec::new()),
        };
        self.run_prepared_inner(prepared, event_tx, None).await
    }

    async fn run_prepared_inner(
        &self,
        prepared: PreparedSessionTurn,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
        snapshot_tx: Option<mpsc::UnboundedSender<Vec<Message>>>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let PreparedSessionTurn {
            hook_ctx,
            mut messages,
            persist_start,
            mut active_tool_presentation_policy,
            mut active_tool_definitions,
            mut active_presented_tool_names,
            initial_plan,
            request_context_limit,
        } = prepared;
        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();
        let mut pending_continuation_parts: Vec<talos_core::message::ContentPart> = Vec::new();
        let mut initial_plan = Some(initial_plan);

        if let Some(snapshot_tx) = &snapshot_tx {
            let _ = snapshot_tx.send(self.persistence_projection(&messages[persist_start..]));
        }

        let (result, final_status) = 'turn_loop: loop {
            let plan = if let Some(plan) = initial_plan.take() {
                plan
            } else {
                match self
                    .seal_provider_request_plan(
                        &hook_ctx,
                        &messages,
                        &active_tool_definitions,
                        &mut pending_continuation_parts,
                        request_context_limit,
                    )
                    .await
                {
                    Ok(plan) => plan,
                    Err(error) => break (Err(error), TurnStatus::Denied),
                }
            };
            tracing::trace!(
                estimated_tokens = plan.estimated_tokens,
                "dispatching sealed provider request plan"
            );

            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();
            let provider_request = self.provider.stream_with_tools_and_progress(
                &plan.messages,
                &plan.tool_definitions,
                progress_tx,
            );
            tokio::pin!(provider_request);
            let provider_result = loop {
                tokio::select! {
                    biased;
                    progress = progress_rx.recv() => {
                        match progress {
                            Some(progress) => {
                                if let Some(ref tx) = event_tx {
                                    let _ = tx.send(AgentEvent::ProviderProgress { progress });
                                }
                            }
                            None => break provider_request.await,
                        }
                    }
                    result = &mut provider_request => {
                        while let Ok(progress) = progress_rx.try_recv() {
                            if let Some(ref tx) = event_tx {
                                let _ = tx.send(AgentEvent::ProviderProgress { progress });
                            }
                        }
                        break result;
                    }
                }
            };

            let mut rx = match provider_result {
                Ok(rx) => rx,
                Err(error) => {
                    if let Some(ref tx) = event_tx {
                        let _ = tx.send(AgentEvent::Error {
                            message: error.to_string(),
                        });
                    }
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
            let mut turn_reasoning_blocks: Option<Vec<ReasoningBlock>> = None;
            let mut saw_turn_end = false;
            let mut turn_stop_reason: Option<StopReason> = None;
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
                        mut call,
                        provenance,
                        ..
                    } => {
                        call.input =
                            permission_pipeline::normalize_permission_input(&call.name, call.input);
                        turn_tool_calls.push(PendingToolCall { call, provenance });
                    }
                    AgentEvent::TurnEnd {
                        stop_reason,
                        usage: turn_usage,
                    } => {
                        saw_turn_end = true;
                        turn_stop_reason = Some(stop_reason.clone());
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
                    AgentEvent::ReasoningComplete { blocks } => {
                        turn_reasoning_blocks = Some(blocks);
                    }
                    AgentEvent::TurnStart
                    | AgentEvent::ProviderProgress { .. }
                    | AgentEvent::ToolResult { .. } => {}
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

            if matches!(turn_stop_reason, Some(StopReason::ToolUse)) && turn_tool_calls.is_empty() {
                break 'turn_loop (
                    Err(AgentError::UnexpectedEvent(
                        "provider ended with tool_use but emitted no tool calls".into(),
                    )),
                    TurnStatus::UnexpectedEvent,
                );
            }

            if !turn_tool_calls.is_empty() {
                let mut seen_ids: HashSet<&str> = HashSet::new();
                let duplicate_id = turn_tool_calls
                    .iter()
                    .find(|pending| !seen_ids.insert(pending.call.id.as_str()))
                    .map(|pending| pending.call.id.clone());
                if let Some(id) = duplicate_id {
                    break 'turn_loop (
                        Err(AgentError::UnexpectedEvent(format!(
                            "provider emitted duplicate tool call id: {id}"
                        ))),
                        TurnStatus::UnexpectedEvent,
                    );
                }

                // Defensive invariant: every emitted ToolCall must carry a
                // non-empty id and name. The OpenAI-compatible SSE parser
                // already synthesizes ids and skips empty names, but other
                // providers (Anthropic, MCP bridging, future runtimes) must
                // not be able to silently push a degenerate ToolCall that
                // would later fail tool lookup or produce ambiguous
                // request/response pairing on the next provider turn.
                let degenerate = turn_tool_calls.iter().find(|pending| {
                    pending.call.id.trim().is_empty() || pending.call.name.trim().is_empty()
                });
                if let Some(pending) = degenerate {
                    break 'turn_loop (
                        Err(AgentError::UnexpectedEvent(format!(
                            "provider emitted tool call with empty id or name (id={:?}, name={:?})",
                            pending.call.id, pending.call.name
                        ))),
                        TurnStatus::UnexpectedEvent,
                    );
                }
            }

            if turn_tool_calls.is_empty() {
                let reasoning = turn_reasoning_blocks
                    .take()
                    .map(|blocks| AssistantReasoning {
                        provider: self.provider_key.clone().unwrap_or_default(),
                        model: self.model_id.clone().unwrap_or_default(),
                        blocks,
                    });
                messages.push(Message::Assistant {
                    content: talos_core::message::strip_tool_syntax(&turn_text),
                    tool_calls: vec![],
                    reasoning,
                });
                break (Ok(turn_text), TurnStatus::Success);
            }

            let proposed_tool_calls: Vec<ToolCall> = turn_tool_calls
                .iter()
                .map(|pending| pending.call.clone())
                .collect();
            let projected_tool_calls = proposed_tool_calls
                .iter()
                .map(|call| self.project_tool_call(call))
                .collect::<Vec<_>>();

            let effective_tool_calls = match self
                .run_hook(
                    &hook_ctx,
                    HookEvent::BeforeToolBatch {
                        calls: &projected_tool_calls,
                    },
                )
                .await
            {
                Ok(HookOutcome::Continue(HookEvent::BeforeToolBatch { calls })) => {
                    if calls == projected_tool_calls.as_slice() {
                        proposed_tool_calls
                    } else {
                        calls.to_vec()
                    }
                }
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
                break 'turn_loop (
                    Ok(format!(
                        "Reached the per-turn tool call limit ({MAX_TOOL_CALLS_PER_TURN}). \
                             All results so far are preserved above — reply \"continue\" to resume."
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
                        Ok(format!(
                            "Detected a repeated call pattern ({signature}). Paused for \
                                 review — all results are preserved above. Adjust your approach \
                                 and reply \"continue\" to resume."
                        )),
                        TurnStatus::DoomLoopDetected,
                    );
                }
            }

            let cleaned_turn_text = talos_core::message::strip_tool_syntax(&turn_text);
            let reasoning = turn_reasoning_blocks
                .take()
                .map(|blocks| AssistantReasoning {
                    provider: self.provider_key.clone().unwrap_or_default(),
                    model: self.model_id.clone().unwrap_or_default(),
                    blocks,
                });
            let assistant_msg = Message::Assistant {
                content: cleaned_turn_text,
                tool_calls: effective_tool_calls.clone(),
                reasoning,
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
                    Ok((results, parts)) => {
                        pending_continuation_parts.extend(parts);
                        results
                    }
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                }
            } else {
                let (tool_results, parts) = match self
                    .execute_tools_with_presentation(
                        &hook_ctx,
                        &effective_tool_calls,
                        &active_tool_presentation_policy,
                        &active_presented_tool_names,
                    )
                    .await
                {
                    Ok((results, parts)) => (results, parts),
                    Err(error) => {
                        break 'turn_loop (Err(error), TurnStatus::Denied);
                    }
                };
                pending_continuation_parts.extend(parts);

                for (call, result) in effective_tool_calls.iter().zip(tool_results.iter()) {
                    let projected_call = self.project_tool_call(call);
                    let projected_result = self.project_tool_result(&call.name, result);
                    let observation = ToolObservation {
                        call: projected_call.clone(),
                        result: projected_result.clone(),
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
                    let observed = ToolObservation {
                        call: Self::restore_private_call_if_unchanged(
                            call,
                            &projected_call,
                            &observed.call,
                        ),
                        result: Self::restore_private_result_if_unchanged(
                            result,
                            &projected_result,
                            &observed.result,
                        ),
                    };

                    let projection = self
                        .tools
                        .get(&observed.call.name)
                        .map(|tool| tool.project_result(&observed.result))
                        .unwrap_or_else(|| {
                            talos_core::tool::ToolResultProjection::shared(
                                observed.result.content.clone(),
                            )
                        });
                    let ui_result = MessageToolResult {
                        tool_use_id: observed.call.id.clone(),
                        content: projection.display_content,
                        is_error: observed.result.is_error,
                    };
                    let llm_result = if observed.result.is_error {
                        MessageToolResult {
                            content: format!(
                                "{}\n\n[Analyze the error above and try a different approach.]",
                                projection.model_content
                            ),
                            ..ui_result.clone()
                        }
                    } else if self.bash_compression_enabled
                        && should_compress_shell_output(&observed.call.name)
                    {
                        let compressed =
                            BashOutputCompressor::new().compress(&projection.model_content);
                        MessageToolResult {
                            content: compressed.content,
                            ..ui_result.clone()
                        }
                    } else if projection.model_content.len() > self.tool_output_threshold {
                        let compressed = crate::tool_output::compress_tool_output(
                            &projection.model_content,
                            self.tool_output_threshold,
                        );
                        MessageToolResult {
                            content: compressed.model_content,
                            ..ui_result.clone()
                        }
                    } else {
                        MessageToolResult {
                            content: projection.model_content,
                            ..ui_result.clone()
                        }
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

            let projected_batch = effective_tool_calls
                .iter()
                .zip(tool_results.iter())
                .map(|(call, result)| self.project_tool_result(&call.name, result))
                .collect::<Vec<_>>();
            let _ = self
                .run_hook(
                    &hook_ctx,
                    HookEvent::AfterToolBatch {
                        results: &projected_batch,
                    },
                )
                .await;
            if let Some(snapshot_tx) = &snapshot_tx {
                // This is the first safe boundary after a complete tool batch:
                // the projection excludes private fields and incomplete calls.
                let _ = snapshot_tx.send(self.persistence_projection(&messages[persist_start..]));
            }
        };

        self.emit_turn_complete(&hook_ctx, final_status).await;

        // Always extract the normalized partial messages from persist_start.
        // On success, these are the complete turn messages. On error, they may
        // contain valid completed tool exchanges that the session layer should
        // persist. Incomplete streamed assistant fragments are never pushed to
        // `messages` — only finalized assistant messages with complete tool
        // calls are (SESSION-006 / I135).
        let partial_messages = self.persistence_projection(&messages[persist_start..]);
        (result, partial_messages)
    }

    fn persistence_projection(&self, messages: &[Message]) -> Vec<Message> {
        let mut tool_names = HashMap::<String, String>::new();
        messages
            .iter()
            .map(|message| match message {
                Message::Assistant {
                    content,
                    tool_calls,
                    reasoning,
                } => Message::Assistant {
                    content: content.clone(),
                    tool_calls: tool_calls
                        .iter()
                        .map(|call| {
                            tool_names.insert(call.id.clone(), call.name.clone());
                            let mut projected = call.clone();
                            if let Some(tool) = self.tools.get(&call.name) {
                                projected.input = tool.project_input(&call.input);
                            }
                            projected
                        })
                        .collect(),
                    reasoning: reasoning.clone(),
                },
                Message::Tool { result } => {
                    let content = tool_names
                        .get(&result.tool_use_id)
                        .and_then(|name| self.tools.get(name))
                        .map(|tool| {
                            let execution = talos_core::tool::ToolResult {
                                content: result.content.clone(),
                                is_error: result.is_error,
                                continuations: Vec::new(),
                            };
                            tool.project_result(&execution).persistence_content
                        })
                        .unwrap_or_else(|| result.content.clone());
                    Message::Tool {
                        result: MessageToolResult {
                            tool_use_id: result.tool_use_id.clone(),
                            content,
                            is_error: result.is_error,
                        },
                    }
                }
                _ => message.clone(),
            })
            .collect()
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

#[cfg(test)]
mod i169_shell_compression_regression {
    use super::should_compress_shell_output;

    #[test]
    fn production_shell_compression_predicate_covers_bash_and_powershell_only() {
        assert!(should_compress_shell_output("bash"));
        assert!(should_compress_shell_output("powershell"));
        assert!(!should_compress_shell_output("read"));
        assert!(!should_compress_shell_output("fetch_url"));
    }
}
