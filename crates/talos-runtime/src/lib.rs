//! Embeddable Talos agent runtime facade.
//!
//! This crate is the SDK-style entrypoint for Rust projects that want to reuse
//! Talos's agent turn loop without depending on the Talos CLI or TUI crates.

#[cfg(feature = "shared-composition")]
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use talos_agent::session::{AppServerSession, RuntimeAdmissionControl};
use talos_agent::{Agent, AgentError, SandboxFallbackHandler};
use talos_core::ApprovalChoice;
use talos_core::message::Message;
use talos_core::provider::LanguageModel;
use talos_core::session::{
    RuntimePolicy, SessionConfig, SessionEvent, SessionOp, TurnCompletionStatus,
};
use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolPermissionFacet, ToolRegistry, ToolResult,
};
use talos_permission::{
    PermissionDecision, PermissionEngine, PermissionRule, ResourceExtractor, ResourceKind,
};
use talos_plugin::HookRegistry;
use talos_sandbox::SandboxProvider;
use talos_session::{DurableSession, PersistencePolicy};
use talos_skill::SkillIndex;
use thiserror::Error;
use tokio::sync::mpsc;

mod shutdown;

pub use shutdown::{
    ActiveTurnPolicy, RuntimeShutdownHandle, ShutdownActiveTurnOutcome, ShutdownActorOutcome,
    ShutdownDurableOutcome, ShutdownFinalizerId, ShutdownFinalizerOutcome,
    ShutdownFinalizerRegistryError, ShutdownFinalizerReport, ShutdownOptions, ShutdownOptionsError,
    ShutdownPlanId, ShutdownReport,
};
use shutdown::{RuntimeFinalizer, RuntimeFinalizerRegistry, ShutdownCoordinator};

#[cfg(feature = "shared-composition")]
#[doc(hidden)]
pub mod composition;

pub use talos_agent::{SandboxFallbackContext, SandboxFallbackDecision, SandboxFallbackPolicy};
pub use talos_core::message::{AgentEvent, MessageToolResult, StopReason, ToolCall, Usage};
pub use talos_core::provider::{ProviderError, ToolDefinition};
pub use talos_core::session::TurnCompletionStatus as RuntimeTurnCompletionStatus;
pub use talos_core::tool::{ToolNature, ToolProvenance};
pub use talos_plugin::HookRegistry as RuntimeHookRegistry;
pub use talos_skill::SkillIndex as RuntimeSkillIndex;

/// Explicit built-in capability preset for embedded runtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePreset {
    /// The same shared coding capability composition used by the CLI.
    Coding,
}

impl RuntimePreset {
    /// Selects the explicit coding capability composition.
    #[must_use]
    pub const fn coding() -> Self {
        Self::Coding
    }
}

/// Errors returned by the embeddable runtime facade.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RuntimeError {
    /// The builder cannot create a runtime without a provider.
    #[error("runtime provider is required")]
    MissingProvider,

    /// A command could not be sent because the runtime actor is closed.
    #[error("runtime command channel is closed")]
    CommandChannelClosed,

    /// A submission was rejected because the runtime shutdown fence has closed.
    #[error("runtime is closing")]
    RuntimeClosing,

    /// Runtime construction requires an active Tokio runtime.
    #[error("runtime construction requires an active Tokio runtime")]
    AsyncRuntimeUnavailable,

    /// The runtime actor task failed to join.
    #[error("runtime actor failed: {0}")]
    ActorJoin(#[from] tokio::task::JoinError),

    /// The underlying agent returned an error.
    #[error("agent error: {0}")]
    Agent(#[from] AgentError),

    /// The coding preset requires the optional shared-composition feature.
    #[error("RuntimePreset::coding() requires the `shared-composition` feature")]
    CodingPresetRequiresFeature,

    /// A durable session could not be read or committed.
    #[error("durable session error: {0}")]
    Session(#[from] talos_session::SessionError),

    /// The legacy shutdown wrapper observed incomplete bounded cleanup.
    #[error("runtime shutdown did not complete")]
    ShutdownIncomplete {
        /// Redacted structured report describing the incomplete stages.
        report: ShutdownReport,
    },

    /// Talos-owned shutdown finalizers could not be frozen safely.
    #[error("invalid runtime shutdown finalizer registry: {0}")]
    InvalidShutdownFinalizerRegistry(#[from] ShutdownFinalizerRegistryError),
}

/// Result alias for runtime facade operations.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Handles approval requests for permission-gated runtime tool calls.
///
/// Embedders can provide an implementation through
/// [`RuntimeBuilder::approval_handler`] to bridge `Ask` decisions into their
/// own UI, RPC, or policy layer. If no handler is configured, the runtime keeps
/// the safe headless default and denies approval-gated calls.
#[async_trait]
pub trait ApprovalHandler: Send + Sync {
    /// Requests a decision for a tool call whose permission policy returned
    /// [`PermissionDecision::Ask`].
    async fn request_approval(
        &self,
        tool_name: &str,
        arguments: &Value,
        summary_fields: &[String],
    ) -> ApprovalChoice;

    /// Requests a one-invocation approval to continue without sandbox
    /// isolation. This is distinct from normal tool permission approval and
    /// defaults to denial; `AlwaysApprove` is never accepted for fallback.
    async fn request_sandbox_fallback(
        &self,
        _context: &SandboxFallbackContext,
    ) -> SandboxFallbackDecision {
        SandboxFallbackDecision::Deny
    }
}

struct RuntimeSandboxFallbackHandler {
    inner: Arc<dyn ApprovalHandler>,
}

struct RuntimeActorExitGuard {
    admission: RuntimeAdmissionControl,
}

impl Drop for RuntimeActorExitGuard {
    fn drop(&mut self) {
        self.admission.record_actor_stopped();
    }
}

#[async_trait]
impl SandboxFallbackHandler for RuntimeSandboxFallbackHandler {
    async fn request_fallback(&self, context: SandboxFallbackContext) -> SandboxFallbackDecision {
        self.inner.request_sandbox_fallback(&context).await
    }
}

/// Builder for an embeddable Talos runtime.
///
/// The safe default is conservative: registered tools are wrapped in a
/// permission-aware adapter, and unresolved `Ask` decisions are denied instead
/// of being executed.
pub struct RuntimeBuilder {
    provider: Option<Arc<dyn LanguageModel>>,
    tools: Vec<Arc<dyn AgentTool>>,
    workspace_root: PathBuf,
    permission_rules: Vec<PermissionRule>,
    sandbox: Option<Box<dyn SandboxProvider>>,
    sandbox_fallback_policy: SandboxFallbackPolicy,
    preset: Option<RuntimePreset>,
    initial_history: Vec<Message>,
    model_context_limit: u32,
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
    custom_prompt: Option<String>,
    append_prompt: Option<String>,
    hook_registry: Option<Arc<HookRegistry>>,
    skill_index: Vec<SkillIndex>,
    durable_session: Option<(DurableSession, PersistencePolicy)>,
    shutdown_finalizers: Vec<Arc<dyn RuntimeFinalizer>>,
}

impl RuntimeBuilder {
    /// Creates a builder with no provider and the current directory as the
    /// workspace root.
    #[must_use]
    pub fn new() -> Self {
        Self {
            provider: None,
            tools: Vec::new(),
            workspace_root: PathBuf::from("."),
            permission_rules: Vec::new(),
            sandbox: None,
            sandbox_fallback_policy: SandboxFallbackPolicy::Deny,
            preset: None,
            initial_history: Vec::new(),
            model_context_limit: 128_000,
            approval_handler: None,
            custom_prompt: None,
            append_prompt: None,
            hook_registry: None,
            skill_index: Vec::new(),
            durable_session: None,
            shutdown_finalizers: Vec::new(),
        }
    }

    /// Sets the language model provider used by the runtime.
    #[must_use]
    pub fn provider(mut self, provider: Arc<dyn LanguageModel>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Sets the workspace root used for path-sensitive runtime behavior.
    #[must_use]
    pub fn workspace_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.workspace_root = root.into();
        self
    }

    /// Registers a tool with runtime-level permission gating.
    #[must_use]
    pub fn tool(mut self, tool: Arc<dyn AgentTool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Adds the explicit shared Talos tool composition to this runtime.
    ///
    /// This method is available only with the `shared-composition` feature. It
    /// does not change `RuntimeBuilder::new()` and does not grant permission:
    /// every added tool still passes through the runtime permission adapter.
    #[cfg(feature = "shared-composition")]
    #[must_use]
    pub fn shared_tools(mut self) -> Self {
        self.tools.extend(
            composition::runtime_tool_contributions(self.workspace_root.clone())
                .into_iter()
                .map(|contribution| contribution.tool().clone()),
        );
        self
    }

    /// Adds an extra permission rule to the runtime permission engine.
    ///
    /// Runtime rules are evaluated before the engine's default fallback, so
    /// embedders can add narrow allow-list or deny-list rules without changing
    /// the safe default for unmatched write, execute, and network tools. Richer
    /// policy import remains a later RUNTIME-001 follow-up.
    #[must_use]
    pub fn permission_rule(mut self, rule: PermissionRule) -> Self {
        self.permission_rules.push(rule);
        self
    }

    /// Sets an optional sandbox provider for sandbox-capable tools.
    #[must_use]
    pub fn sandbox(mut self, sandbox: Box<dyn SandboxProvider>) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Sets the sandbox fallback policy. The default is `Deny`.
    #[must_use]
    pub fn sandbox_fallback_policy(mut self, policy: SandboxFallbackPolicy) -> Self {
        self.sandbox_fallback_policy = policy;
        self
    }

    /// Sets the sandbox fallback policy using the SDK contract name.
    #[must_use]
    pub fn sandbox_fallback(self, policy: SandboxFallbackPolicy) -> Self {
        self.sandbox_fallback_policy(policy)
    }

    /// Selects an explicit runtime capability preset.
    #[must_use]
    pub fn preset(mut self, preset: RuntimePreset) -> Self {
        self.preset = Some(preset);
        self
    }

    /// Selects the shared coding capability composition.
    #[must_use]
    pub fn coding_preset(self) -> Self {
        self.preset(RuntimePreset::coding())
    }

    /// Seeds the runtime with existing conversation history.
    #[must_use]
    pub fn initial_history(mut self, history: Vec<Message>) -> Self {
        self.initial_history = history;
        self
    }

    /// Sets the model context limit used by the session compactor.
    #[must_use]
    pub fn model_context_limit(mut self, limit: u32) -> Self {
        self.model_context_limit = limit;
        self
    }

    /// Sets the approval handler for tools whose permission policy returns
    /// `Ask`.
    ///
    /// Without a handler, `Ask` decisions are denied. `AlwaysApprove` choices
    /// install in-memory allow rules for the current runtime only; they are not
    /// persisted to user configuration.
    #[must_use]
    pub fn approval_handler(mut self, handler: Arc<dyn ApprovalHandler>) -> Self {
        self.approval_handler = Some(handler);
        self
    }

    /// Replaces the default Talos identity/system prompt.
    ///
    /// This is intended for embedders that reuse the runtime in a product with
    /// its own identity. Use [`RuntimeBuilder::append_prompt`] when the default
    /// identity should remain and only extra instructions are needed.
    #[must_use]
    pub fn custom_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.custom_prompt = Some(prompt.into());
        self
    }

    /// Appends extra instructions to the system prompt.
    #[must_use]
    pub fn append_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.append_prompt = Some(prompt.into());
        self
    }

    /// Binds this runtime to a host-selected durable Talos session.
    ///
    /// The session's model history is restored during [`Self::build`]. Every
    /// successful turn is atomically persisted by Talos before its success
    /// completion event is emitted; failed, cancelled, and denied turns are
    /// not persisted. Calling this is optional and does not alter the existing
    /// in-memory runtime behavior when omitted.
    #[must_use]
    pub fn durable_session(mut self, session: DurableSession) -> Self {
        self.durable_session = Some((session, PersistencePolicy::default()));
        self
    }

    /// Sets the filtering policy for a durable session binding.
    #[must_use]
    pub fn durable_session_with_policy(
        mut self,
        session: DurableSession,
        policy: PersistencePolicy,
    ) -> Self {
        self.durable_session = Some((session, policy));
        self
    }

    /// Injects a pre-populated [`HookRegistry`] into the runtime.
    ///
    /// This allows embedders to register reviewed hooks (e.g., from a curated
    /// hook catalog) before the runtime starts, following the same builder
    /// pattern as [`RuntimeBuilder::approval_handler`].
    ///
    /// If not called, the runtime uses an empty `HookRegistry`.
    #[must_use]
    pub fn hook_registry(mut self, registry: Arc<HookRegistry>) -> Self {
        self.hook_registry = Some(registry);
        self
    }

    /// Sets the skill index for the runtime's system prompt.
    ///
    /// Embedders that discover skills via [`talos_skill::SkillLoader`] (e.g.,
    /// from a local or remote skill store) can inject the Level 0 index here,
    /// following the same pattern as the CLI's `skill_runtime.rs`.
    ///
    /// If not called, the runtime has no skills in its system prompt.
    #[must_use]
    pub fn skill_index(mut self, skills: Vec<SkillIndex>) -> Self {
        self.skill_index = skills;
        self
    }

    #[cfg(test)]
    fn runtime_finalizer(mut self, finalizer: Arc<dyn RuntimeFinalizer>) -> Self {
        self.shutdown_finalizers.push(finalizer);
        self
    }

    /// Builds and starts the runtime actor.
    ///
    /// The returned primary handle owns submission and event access. Dropping
    /// it initiates the non-blocking default shutdown plan; use
    /// [`RuntimeHandle::shutdown`] or [`RuntimeHandle::shutdown_with`] when the
    /// host must observe terminal cleanup.
    pub fn build(self) -> RuntimeResult<RuntimeHandle> {
        let finalizers = RuntimeFinalizerRegistry::freeze(self.shutdown_finalizers)?;
        let provider = self.provider.ok_or(RuntimeError::MissingProvider)?;
        #[allow(unused_mut)]
        let mut tools = self.tools;
        if matches!(self.preset, Some(RuntimePreset::Coding)) {
            #[cfg(feature = "shared-composition")]
            {
                // Explicit caller tools are authoritative. A preset must not
                // replace an embedder's hardened implementation by name.
                let caller_tool_names = tools
                    .iter()
                    .map(|tool| tool.name().to_owned())
                    .collect::<HashSet<_>>();
                tools.extend(
                    composition::runtime_tool_contributions(self.workspace_root.clone())
                        .into_iter()
                        .filter(|contribution| !caller_tool_names.contains(contribution.name()))
                        .map(|contribution| contribution.tool().clone()),
                );
            }
            #[cfg(not(feature = "shared-composition"))]
            return Err(RuntimeError::CodingPresetRequiresFeature);
        }
        let tool_engine = Arc::new(Mutex::new(build_permission_engine(
            self.workspace_root.clone(),
            &self.permission_rules,
        )));
        let agent_engine = Arc::new(build_permission_engine(
            self.workspace_root.clone(),
            &self.permission_rules,
        ));
        let mut registry = ToolRegistry::new();
        for tool in tools {
            registry.register(Arc::new(RuntimePermissionAwareTool {
                inner: tool,
                engine: tool_engine.clone(),
                approval_handler: self.approval_handler.clone(),
            }));
        }

        let fallback_handler = self.approval_handler.as_ref().map(|handler| {
            Arc::new(RuntimeSandboxFallbackHandler {
                inner: handler.clone(),
            }) as Arc<dyn SandboxFallbackHandler>
        });
        let mut agent = if let Some(hooks) = self.hook_registry {
            Agent::with_security_and_hooks_and_sandbox_fallback(
                provider,
                registry,
                Some(agent_engine.clone()),
                self.sandbox,
                self.workspace_root.clone(),
                hooks,
                self.sandbox_fallback_policy,
                fallback_handler,
            )
        } else {
            Agent::with_security_and_sandbox_fallback(
                provider,
                registry,
                Some(agent_engine),
                self.sandbox,
                self.workspace_root.clone(),
                self.sandbox_fallback_policy,
                fallback_handler,
            )
        };
        if let Some(prompt) = self.custom_prompt {
            agent.set_custom_prompt(prompt);
        }
        if let Some(prompt) = self.append_prompt {
            agent.set_append_prompt(prompt);
        }
        if !self.skill_index.is_empty() {
            agent.set_skill_index(self.skill_index);
        }
        let initial_history = if let Some((session, _)) = &self.durable_session {
            session.read_messages()?
        } else {
            self.initial_history
        };
        let config = SessionConfig {
            runtime_policy: RuntimePolicy::headless_deny(),
            workspace_root: self.workspace_root,
            initial_history,
            model_context_limit: self.model_context_limit,
        };
        let (handle, mut actor) = AppServerSession::new(agent, config);
        let admission = talos_agent::session::RuntimeAdmissionControl::new();
        actor.set_runtime_admission(admission.clone());
        if let Some((session, policy)) = self.durable_session {
            actor.set_durable_persistence(session, policy);
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| RuntimeError::AsyncRuntimeUnavailable)?;
        let actor_exit_guard = RuntimeActorExitGuard {
            admission: admission.clone(),
        };
        let actor_task = runtime.spawn(async move {
            let _actor_exit_guard = actor_exit_guard;
            actor.run().await;
        });
        let coordinator = ShutdownCoordinator::new(
            admission,
            handle.sq_tx.clone(),
            actor_task,
            runtime,
            finalizers,
        );

        Ok(RuntimeHandle {
            command_tx: handle.sq_tx,
            event_rx: handle.eq_rx,
            coordinator,
            primary_drop_armed: true,
        })
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for interacting with a running embedded Talos runtime.
pub struct RuntimeHandle {
    command_tx: mpsc::Sender<SessionOp>,
    event_rx: mpsc::UnboundedReceiver<SessionEvent>,
    coordinator: Arc<ShutdownCoordinator>,
    primary_drop_armed: bool,
}

impl RuntimeHandle {
    /// Submits a user message as a new turn.
    pub async fn submit(&self, message: impl Into<String>) -> RuntimeResult<()> {
        if !self.coordinator.is_admission_open() {
            return Err(RuntimeError::RuntimeClosing);
        }
        let permit = self.command_tx.reserve().await.map_err(|_| {
            if self.coordinator.is_admission_open() {
                RuntimeError::CommandChannelClosed
            } else {
                RuntimeError::RuntimeClosing
            }
        })?;
        self.coordinator
            .commit_reserved(
                permit,
                SessionOp::Submit {
                    message: message.into(),
                },
            )
            .map_err(|_| RuntimeError::RuntimeClosing)
    }

    /// Requests a provider request preview without making a provider call.
    pub async fn preview_request(&self, message: impl Into<String>) -> RuntimeResult<()> {
        if !self.coordinator.is_admission_open() {
            return Err(RuntimeError::RuntimeClosing);
        }
        let permit = self.command_tx.reserve().await.map_err(|_| {
            if self.coordinator.is_admission_open() {
                RuntimeError::CommandChannelClosed
            } else {
                RuntimeError::RuntimeClosing
            }
        })?;
        self.coordinator
            .commit_reserved(
                permit,
                SessionOp::PreviewRequest {
                    message: message.into(),
                },
            )
            .map_err(|_| RuntimeError::RuntimeClosing)
    }

    /// Interrupts the active turn, if any.
    pub async fn interrupt(&self) -> RuntimeResult<()> {
        self.command_tx
            .send(SessionOp::Interrupt)
            .await
            .map_err(|_| RuntimeError::CommandChannelClosed)
    }

    /// Receives the next runtime event.
    pub async fn next_event(&mut self) -> Option<SessionEvent> {
        self.event_rx.recv().await
    }

    /// Returns a cloneable controller that can only initiate or join shutdown.
    #[must_use]
    pub fn shutdown_controller(&self) -> RuntimeShutdownHandle {
        RuntimeShutdownHandle {
            coordinator: self.coordinator.clone(),
        }
    }

    /// Starts or joins structured bounded shutdown without consuming the handle.
    pub async fn shutdown_with(&self, options: ShutdownOptions) -> RuntimeResult<ShutdownReport> {
        self.coordinator.shutdown(options).await
    }

    /// Shuts down the runtime actor and waits for it to finish.
    pub async fn shutdown(mut self) -> RuntimeResult<()> {
        self.primary_drop_armed = false;
        let report = self
            .coordinator
            .shutdown(ShutdownOptions::legacy_default())
            .await?;
        if let Some(error) = self.coordinator.take_actor_join_error() {
            return Err(RuntimeError::ActorJoin(error));
        }
        if report.is_complete() {
            Ok(())
        } else {
            Err(RuntimeError::ShutdownIncomplete { report })
        }
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        if self.primary_drop_armed {
            self.primary_drop_armed = false;
            self.coordinator.initiate_default();
        }
    }
}

fn build_permission_engine(root: PathBuf, rules: &[PermissionRule]) -> PermissionEngine {
    PermissionEngine {
        rules: rules.to_vec(),
        workspace_root: Some(root),
        trusted_workspace: false,
    }
}

struct RuntimePermissionAwareTool {
    inner: Arc<dyn AgentTool>,
    engine: Arc<Mutex<PermissionEngine>>,
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
}

impl RuntimePermissionAwareTool {
    async fn execute_with_authorization(
        &self,
        input: Value,
        profile: &[ToolPermissionFacet],
        scope: ToolAuthorizationScope,
    ) -> ToolResult {
        let authorizations = match self.engine.lock() {
            Ok(engine) => {
                match engine.execution_authorizations(self.inner.name(), profile, &input, scope) {
                    Ok(authorizations) => authorizations,
                    Err(error) => {
                        return ToolResult::error(format!(
                            "Permission denied: invalid execution authorization: {error}"
                        ));
                    }
                }
            }
            Err(_) => {
                return ToolResult::error("Permission denied: permission engine lock poisoned");
            }
        };
        self.inner.execute_authorized(input, &authorizations).await
    }
}

#[async_trait]
impl AgentTool for RuntimePermissionAwareTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let profile = self.inner.permission_profile(&input);
        let decision = {
            match self.engine.lock() {
                Ok(engine) => engine.evaluate_profile(self.inner.name(), &profile, &input),
                Err(_) => {
                    return ToolResult::error("Permission denied: permission engine lock poisoned");
                }
            }
        };

        match decision {
            PermissionDecision::Allow => {
                self.execute_with_authorization(input, &profile, ToolAuthorizationScope::Persisted)
                    .await
            }
            PermissionDecision::Deny(reason) => {
                ToolResult::error(format!("Permission denied: {reason}"))
            }
            PermissionDecision::Ask => {
                let Some(handler) = &self.approval_handler else {
                    return ToolResult::error(
                        "Permission denied: approval required but no runtime approval handler is configured",
                    );
                };
                let summary_fields = self
                    .inner
                    .summary_fields()
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect::<Vec<_>>();
                match handler
                    .request_approval(
                        self.inner.name(),
                        &self.inner.project_input(&input),
                        &summary_fields,
                    )
                    .await
                {
                    ApprovalChoice::ApproveOnce => {
                        self.execute_with_authorization(
                            input,
                            &profile,
                            ToolAuthorizationScope::Once,
                        )
                        .await
                    }
                    ApprovalChoice::AlwaysApprove => {
                        add_always_allow_rules(&self.engine, &profile, &input);
                        self.execute_with_authorization(
                            input,
                            &profile,
                            ToolAuthorizationScope::Persisted,
                        )
                        .await
                    }
                    ApprovalChoice::Deny => ToolResult::error("Permission denied: User denied"),
                }
            }
        }
    }

    fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        self.inner.nature()
    }

    fn family(&self) -> talos_core::tool::ToolFamily {
        self.inner.family()
    }

    fn is_always_on(&self) -> bool {
        self.inner.is_always_on()
    }

    fn permission_profile(&self, input: &Value) -> Vec<talos_core::tool::ToolPermissionFacet> {
        self.inner.permission_profile(input)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        self.inner.summary_fields()
    }

    fn project_input(&self, input: &Value) -> Value {
        self.inner.project_input(input)
    }

    fn project_result(&self, result: &ToolResult) -> talos_core::tool::ToolResultProjection {
        self.inner.project_result(result)
    }

    fn provenance(&self) -> talos_core::tool::ToolProvenance {
        self.inner.provenance()
    }
}

fn add_always_allow_rules(
    engine: &Arc<Mutex<PermissionEngine>>,
    profile: &[ToolPermissionFacet],
    input: &Value,
) {
    let Ok(mut engine) = engine.lock() else {
        return;
    };
    for facet in profile {
        let resource = facet
            .resource
            .clone()
            .or_else(|| ResourceExtractor::extract(facet.nature, input));
        let resource_kind = facet
            .resource_kind
            .map(ResourceKind::from)
            .or_else(|| Some(default_resource_kind(facet.nature)));
        engine.add_runtime_allow_rule(PermissionRule::new_nature(
            facet.nature,
            resource,
            resource_kind,
            PermissionDecision::Allow,
        ));
    }
}

fn default_resource_kind(nature: ToolNature) -> ResourceKind {
    match nature {
        ToolNature::Network => ResourceKind::Domain,
        ToolNature::Execute => ResourceKind::Command,
        ToolNature::Read | ToolNature::Write => ResourceKind::Path,
        ToolNature::Internal => ResourceKind::Remote,
    }
}

/// Collects events until the current turn completes.
///
/// This helper is intended for embedders that want a simple per-turn API on top
/// of the streaming event channel.
pub async fn collect_until_turn_completed(
    runtime: &mut RuntimeHandle,
) -> Option<TurnCompletionStatus> {
    while let Some(event) = runtime.next_event().await {
        if let SessionEvent::TurnEvent {
            payload: talos_core::session::TurnEventPayload::Completed { status },
            ..
        } = event
        {
            return Some(status);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    use talos_core::message::Message;
    use talos_core::provider::ProviderResult;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
    use talos_permission::PermissionDecision;
    use talos_provider::mock::MockProvider;
    use talos_session::SessionManager;
    use talos_tools::{ReadTool, snapshot_aware_file_tools};
    use tokio::sync::Notify;

    use super::*;

    struct RecordingWriteTool {
        executions: Arc<AtomicUsize>,
    }

    struct RecordingHybridTool {
        executions: Arc<AtomicUsize>,
    }

    struct PrivateInputWriteTool;

    struct PrivateResultReadTool;

    struct GatedModel {
        entered: Arc<Notify>,
        release: Arc<Notify>,
    }

    #[derive(Clone)]
    enum TestFinalizerBehavior {
        Complete,
        Fail,
        Panic,
        Delay(Duration),
        Pending(Arc<AtomicBool>),
    }

    struct TestFinalizer {
        identifier: ShutdownFinalizerId,
        order: u16,
        cap: Duration,
        behavior: TestFinalizerBehavior,
        starts: Arc<StdMutex<Vec<&'static str>>>,
    }

    impl RuntimeFinalizer for TestFinalizer {
        fn identifier(&self) -> ShutdownFinalizerId {
            self.identifier
        }

        fn order(&self) -> u16 {
            self.order
        }

        fn cap(&self) -> Duration {
            self.cap
        }

        fn finalize(&self) -> shutdown::RuntimeFinalizerFuture {
            let identifier = self.identifier.as_str();
            let starts = self.starts.clone();
            let behavior = self.behavior.clone();
            Box::pin(async move {
                starts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(identifier);
                match behavior {
                    TestFinalizerBehavior::Complete => Ok(()),
                    TestFinalizerBehavior::Fail => Err(shutdown::RuntimeFinalizerError),
                    TestFinalizerBehavior::Panic => panic!("intentional runtime finalizer panic"),
                    TestFinalizerBehavior::Delay(duration) => {
                        tokio::time::sleep(duration).await;
                        Ok(())
                    }
                    TestFinalizerBehavior::Pending(cancelled) => {
                        struct CancellationMarker(Arc<AtomicBool>);
                        impl Drop for CancellationMarker {
                            fn drop(&mut self) {
                                self.0.store(true, Ordering::SeqCst);
                            }
                        }
                        let _marker = CancellationMarker(cancelled);
                        std::future::pending::<()>().await;
                        Ok(())
                    }
                }
            })
        }
    }

    fn test_finalizer(
        identifier: &'static str,
        order: u16,
        cap: Duration,
        behavior: TestFinalizerBehavior,
        starts: Arc<StdMutex<Vec<&'static str>>>,
    ) -> Arc<dyn RuntimeFinalizer> {
        Arc::new(TestFinalizer {
            identifier: ShutdownFinalizerId::new(identifier),
            order,
            cap,
            behavior,
            starts,
        })
    }

    #[async_trait]
    impl LanguageModel for GatedModel {
        async fn stream(
            &self,
            _messages: &[Message],
        ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let (tx, rx) = mpsc::channel(8);
            let release = self.release.clone();
            self.entered.notify_one();
            tokio::spawn(async move {
                release.notified().await;
                let events = [
                    AgentEvent::TurnStart,
                    AgentEvent::TextDelta {
                        delta: "finished".into(),
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::EndTurn,
                        usage: Usage::default(),
                    },
                ];
                for event in events {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            });
            Ok(rx)
        }
    }

    async fn gated_runtime() -> (RuntimeHandle, Arc<Notify>, Arc<Notify>) {
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(GatedModel {
                entered: entered.clone(),
                release: release.clone(),
            }))
            .build()
            .expect("gated runtime builds");
        (runtime, entered, release)
    }

    #[test]
    fn shutdown_options_reject_invalid_drafts_before_runtime_access() {
        assert_eq!(
            ShutdownOptions::interrupt(Duration::ZERO),
            Err(ShutdownOptionsError::ZeroTotalTimeout)
        );
        assert_eq!(
            ShutdownOptions::finish_current(Duration::from_secs(1), Duration::from_secs(1)),
            Err(ShutdownOptionsError::FinishGraceNotLessThanTotal)
        );
        assert_eq!(
            ShutdownOptions::interrupt(Duration::MAX),
            Err(ShutdownOptionsError::TotalTimeoutOutOfRange)
        );
    }

    #[test]
    fn runtime_build_freezes_and_validates_finalizer_identity_and_order() {
        let starts = Arc::new(StdMutex::new(Vec::new()));
        let duplicate_identifier = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.duplicate",
                10,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.duplicate",
                20,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .build();
        assert!(matches!(
            duplicate_identifier,
            Err(RuntimeError::InvalidShutdownFinalizerRegistry(
                ShutdownFinalizerRegistryError::DuplicateIdentifier
            ))
        ));

        let duplicate_order = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.first",
                10,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.second",
                10,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .build();
        assert!(matches!(
            duplicate_order,
            Err(RuntimeError::InvalidShutdownFinalizerRegistry(
                ShutdownFinalizerRegistryError::DuplicateOrder
            ))
        ));

        let zero_cap = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.zero-cap",
                10,
                Duration::ZERO,
                TestFinalizerBehavior::Complete,
                starts,
            ))
            .build();
        assert!(matches!(
            zero_cap,
            Err(RuntimeError::InvalidShutdownFinalizerRegistry(
                ShutdownFinalizerRegistryError::ZeroCap
            ))
        ));
    }

    #[tokio::test]
    async fn durable_reconciliation_precedes_ordered_finalizers() {
        let admission = talos_agent::session::RuntimeAdmissionControl::new();
        let actor_admission = admission.clone();
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let order = Arc::new(StdMutex::new(Vec::new()));
        let actor_order = order.clone();
        let actor_task = tokio::spawn(async move {
            assert!(matches!(command_rx.recv().await, Some(SessionOp::Shutdown)));
            actor_order
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("durable");
            actor_admission.record_reconciliation(0, true);
        });
        let registry = RuntimeFinalizerRegistry::freeze(vec![test_finalizer(
            "test.finalizer",
            10,
            Duration::from_secs(1),
            TestFinalizerBehavior::Complete,
            order.clone(),
        )])
        .expect("registry is valid");
        let coordinator = ShutdownCoordinator::new(
            admission,
            command_tx,
            actor_task,
            tokio::runtime::Handle::current(),
            registry,
        );

        let report = coordinator
            .shutdown(ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"))
            .await
            .expect("shutdown report");

        assert_eq!(
            *order
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec!["durable", "test.finalizer"]
        );
        assert_eq!(report.finalizers().len(), 1);
        assert_eq!(
            report.finalizers()[0].identifier(),
            ShutdownFinalizerId::new("test.finalizer")
        );
        assert_eq!(
            report.finalizers()[0].outcome(),
            ShutdownFinalizerOutcome::Completed
        );
        assert!(report.is_complete());
    }

    #[tokio::test]
    async fn finalizer_failure_and_panic_are_typed_and_do_not_stop_later_entries() {
        let starts = Arc::new(StdMutex::new(Vec::new()));
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.third",
                30,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.first",
                10,
                Duration::from_secs(1),
                TestFinalizerBehavior::Fail,
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.second",
                20,
                Duration::from_secs(1),
                TestFinalizerBehavior::Panic,
                starts.clone(),
            ))
            .build()
            .expect("runtime builds");

        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
            )
            .await
            .expect("structured report");

        assert_eq!(
            *starts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec!["test.first", "test.second", "test.third"]
        );
        assert_eq!(
            report
                .finalizers()
                .iter()
                .map(ShutdownFinalizerReport::outcome)
                .collect::<Vec<_>>(),
            vec![
                ShutdownFinalizerOutcome::Failed,
                ShutdownFinalizerOutcome::Panicked,
                ShutdownFinalizerOutcome::Completed,
            ]
        );
        assert!(!report.is_complete());
        assert!(matches!(
            runtime.shutdown().await,
            Err(RuntimeError::ShutdownIncomplete { .. })
        ));
        assert_eq!(
            *starts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec!["test.first", "test.second", "test.third"]
        );
    }

    #[tokio::test]
    async fn finalizer_cap_contains_timeout_and_allows_later_entry() {
        let starts = Arc::new(StdMutex::new(Vec::new()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.timeout",
                10,
                Duration::from_millis(20),
                TestFinalizerBehavior::Pending(cancelled.clone()),
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.after-timeout",
                20,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .build()
            .expect("runtime builds");

        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
            )
            .await
            .expect("structured report");
        tokio::task::yield_now().await;

        assert!(cancelled.load(Ordering::SeqCst));
        assert_eq!(
            *starts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec!["test.timeout", "test.after-timeout"]
        );
        assert_eq!(
            report
                .finalizers()
                .iter()
                .map(ShutdownFinalizerReport::outcome)
                .collect::<Vec<_>>(),
            vec![
                ShutdownFinalizerOutcome::TimedOut,
                ShutdownFinalizerOutcome::Completed,
            ]
        );
        assert!(!report.deadline_exhausted());
    }

    #[tokio::test]
    async fn finalizers_share_the_original_global_deadline_without_resetting_it() {
        let starts = Arc::new(StdMutex::new(Vec::new()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .runtime_finalizer(test_finalizer(
                "test.delay",
                10,
                Duration::from_secs(1),
                TestFinalizerBehavior::Delay(Duration::from_millis(50)),
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.consume-remaining",
                20,
                Duration::from_secs(1),
                TestFinalizerBehavior::Pending(cancelled.clone()),
                starts.clone(),
            ))
            .runtime_finalizer(test_finalizer(
                "test.not-run",
                30,
                Duration::from_secs(1),
                TestFinalizerBehavior::Complete,
                starts.clone(),
            ))
            .build()
            .expect("runtime builds");

        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_millis(200)).expect("valid options"),
            )
            .await
            .expect("structured report");
        tokio::task::yield_now().await;

        assert!(cancelled.load(Ordering::SeqCst));
        assert!(report.elapsed() < Duration::from_millis(500));
        assert!(report.deadline_exhausted());
        assert_eq!(
            report
                .finalizers()
                .iter()
                .map(ShutdownFinalizerReport::outcome)
                .collect::<Vec<_>>(),
            vec![
                ShutdownFinalizerOutcome::Completed,
                ShutdownFinalizerOutcome::TimedOut,
                ShutdownFinalizerOutcome::NotRunDeadline,
            ]
        );
        assert_eq!(
            *starts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec!["test.delay", "test.consume-remaining"]
        );
    }

    #[tokio::test]
    async fn invalid_options_leave_the_primary_runtime_usable() {
        let mut runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new().with_response("still open")))
            .build()
            .expect("runtime builds");
        assert!(ShutdownOptions::interrupt(Duration::ZERO).is_err());

        runtime
            .submit("continue")
            .await
            .expect("submit still succeeds");
        assert!(matches!(
            collect_until_turn_completed(&mut runtime).await,
            Some(TurnCompletionStatus::Success { .. })
        ));
        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn concurrent_shutdown_callers_share_one_cached_report() {
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new().with_response("unused")))
            .build()
            .expect("runtime builds");
        let first = runtime.shutdown_controller();
        let second = first.clone();
        let first_task = tokio::spawn(async move {
            first
                .shutdown(
                    ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
                )
                .await
        });
        let second_task = tokio::spawn(async move {
            second
                .shutdown(
                    ShutdownOptions::finish_current(
                        Duration::from_secs(2),
                        Duration::from_millis(10),
                    )
                    .expect("valid options"),
                )
                .await
        });
        let first_report = first_task.await.expect("caller joins").expect("report");
        let second_report = second_task.await.expect("caller joins").expect("report");

        assert_eq!(first_report, second_report);
        assert!(first_report.is_complete());
        assert_eq!(first_report.active_turn(), ShutdownActiveTurnOutcome::Idle);
        runtime
            .shutdown()
            .await
            .expect("legacy caller joins cached result");
    }

    #[tokio::test]
    async fn interrupt_closes_admission_and_finalizes_the_active_turn() {
        let (runtime, entered, _release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;

        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
            )
            .await
            .expect("structured report");
        assert_eq!(
            report.active_turn(),
            ShutdownActiveTurnOutcome::InterruptedAndFinalized
        );
        assert!(report.is_complete());
        assert!(matches!(
            runtime.submit("too late").await,
            Err(RuntimeError::RuntimeClosing)
        ));
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[tokio::test]
    async fn finish_current_uses_grace_without_starting_post_fence_work() {
        let (runtime, entered, release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;
        let controller = runtime.shutdown_controller();
        let shutdown = tokio::spawn(async move {
            controller
                .shutdown(
                    ShutdownOptions::finish_current(
                        Duration::from_secs(1),
                        Duration::from_millis(500),
                    )
                    .expect("valid options"),
                )
                .await
        });
        tokio::task::yield_now().await;
        assert!(matches!(
            runtime.submit("post-fence").await,
            Err(RuntimeError::RuntimeClosing)
        ));
        release.notify_one();
        let report = shutdown.await.expect("caller joins").expect("report");

        assert_eq!(report.active_turn(), ShutdownActiveTurnOutcome::Finished);
        assert!(report.is_complete());
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[tokio::test]
    async fn finish_current_grace_expiry_uses_actor_owned_interrupt() {
        let (runtime, entered, _release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;

        let report = runtime
            .shutdown_with(
                ShutdownOptions::finish_current(Duration::from_secs(1), Duration::from_millis(10))
                    .expect("valid options"),
            )
            .await
            .expect("structured report");
        assert_eq!(
            report.active_turn(),
            ShutdownActiveTurnOutcome::InterruptedAndFinalized
        );
        assert!(report.is_complete());
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[tokio::test]
    async fn finish_current_never_starts_pre_fence_pending_work() {
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let calls = Arc::new(AtomicUsize::new(0));
        struct CountingGatedModel {
            entered: Arc<Notify>,
            release: Arc<Notify>,
            calls: Arc<AtomicUsize>,
        }
        #[async_trait]
        impl LanguageModel for CountingGatedModel {
            async fn stream(
                &self,
                _messages: &[Message],
            ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                let (tx, rx) = mpsc::channel(8);
                let release = self.release.clone();
                self.entered.notify_one();
                tokio::spawn(async move {
                    release.notified().await;
                    for event in [
                        AgentEvent::TurnStart,
                        AgentEvent::TurnEnd {
                            stop_reason: StopReason::EndTurn,
                            usage: Usage::default(),
                        },
                    ] {
                        let _ = tx.send(event).await;
                    }
                });
                Ok(rx)
            }
        }
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(CountingGatedModel {
                entered: entered.clone(),
                release: release.clone(),
                calls: calls.clone(),
            }))
            .build()
            .expect("runtime builds");
        let entered_wait = entered.notified();
        runtime.submit("active").await.expect("first submit");
        entered_wait.await;
        runtime.submit("pending").await.expect("pre-fence submit");
        let controller = runtime.shutdown_controller();
        let shutdown = tokio::spawn(async move {
            controller
                .shutdown(
                    ShutdownOptions::finish_current(
                        Duration::from_secs(1),
                        Duration::from_millis(500),
                    )
                    .expect("valid options"),
                )
                .await
        });
        tokio::task::yield_now().await;
        release.notify_one();
        let report = shutdown.await.expect("caller joins").expect("report");

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(report.active_turn(), ShutdownActiveTurnOutcome::Finished);
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[tokio::test]
    async fn cancelling_one_waiter_does_not_cancel_the_runtime_driver() {
        let (runtime, entered, _release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;
        let first = runtime.shutdown_controller();
        let later = first.clone();
        let waiter = tokio::spawn(async move {
            first
                .shutdown(
                    ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
                )
                .await
        });
        loop {
            if matches!(
                runtime.submit("fence probe").await,
                Err(RuntimeError::RuntimeClosing)
            ) {
                break;
            }
            tokio::task::yield_now().await;
        }
        waiter.abort();

        let report = later
            .shutdown(
                ShutdownOptions::finish_current(Duration::from_secs(2), Duration::from_millis(100))
                    .expect("valid options"),
            )
            .await
            .expect("later caller receives cached report");
        assert_eq!(report.active_turn_policy(), ActiveTurnPolicy::Interrupt);
        assert!(report.is_complete());
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[tokio::test]
    async fn primary_drop_initiates_default_plan_and_controller_drop_is_inert() {
        let (runtime, entered, _release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;
        let controller = runtime.shutdown_controller();
        drop(runtime.shutdown_controller());
        drop(runtime);

        let report = controller
            .shutdown(
                ShutdownOptions::finish_current(Duration::from_secs(2), Duration::from_millis(100))
                    .expect("valid options"),
            )
            .await
            .expect("controller joins Drop-initiated report");
        assert_eq!(report.active_turn_policy(), ActiveTurnPolicy::Interrupt);
        assert!(report.is_complete());
    }

    #[tokio::test]
    async fn primary_drop_cannot_replace_an_explicit_winning_plan() {
        let (runtime, entered, release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime.submit("block").await.expect("submit succeeds");
        entered_wait.await;
        let controller = runtime.shutdown_controller();
        let observer = controller.clone();
        let shutdown = tokio::spawn(async move {
            controller
                .shutdown(
                    ShutdownOptions::finish_current(
                        Duration::from_secs(1),
                        Duration::from_millis(500),
                    )
                    .expect("valid options"),
                )
                .await
        });
        loop {
            if matches!(
                runtime.submit("fence probe").await,
                Err(RuntimeError::RuntimeClosing)
            ) {
                break;
            }
            tokio::task::yield_now().await;
        }
        drop(runtime);
        release.notify_one();

        let report = shutdown.await.expect("caller joins").expect("report");
        let cached = observer
            .shutdown(ShutdownOptions::interrupt(Duration::from_secs(2)).expect("valid options"))
            .await
            .expect("observer joins cached report");
        assert_eq!(report, cached);
        assert!(matches!(
            report.active_turn_policy(),
            ActiveTurnPolicy::FinishCurrent { .. }
        ));
    }

    #[tokio::test]
    async fn legacy_shutdown_preserves_actor_join_errors() {
        let admission = talos_agent::session::RuntimeAdmissionControl::new();
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_rx);
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let actor_exit_guard = RuntimeActorExitGuard {
            admission: admission.clone(),
        };
        let actor_task = tokio::spawn(async move {
            let _actor_exit_guard = actor_exit_guard;
            panic!("intentional actor join failure");
        });
        let runtime_handle = tokio::runtime::Handle::current();
        let coordinator = ShutdownCoordinator::new(
            admission,
            command_tx.clone(),
            actor_task,
            runtime_handle,
            RuntimeFinalizerRegistry::freeze(Vec::new()).expect("empty registry is valid"),
        );
        let runtime = RuntimeHandle {
            command_tx,
            event_rx,
            coordinator,
            primary_drop_armed: true,
        };

        assert!(matches!(
            runtime.shutdown().await,
            Err(RuntimeError::ActorJoin(_))
        ));
    }

    #[tokio::test]
    async fn exhausted_total_deadline_returns_a_redacted_incomplete_report() {
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new().with_response("unused")))
            .build()
            .expect("runtime builds");
        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_nanos(1)).expect("valid options"),
            )
            .await
            .expect("structured timeout remains observable");

        assert!(report.deadline_exhausted());
        assert!(!report.is_complete());
        assert!(matches!(
            runtime.shutdown().await,
            Err(RuntimeError::ShutdownIncomplete { .. })
        ));
    }

    #[tokio::test]
    async fn durable_reconciliation_failure_is_typed_and_incomplete() {
        let blocked_root = tempfile::NamedTempFile::new().expect("temporary file");
        let runtime = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new().with_response("unused")))
            .workspace_root(blocked_root.path())
            .build()
            .expect("runtime construction is lazy over pending custody");
        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
            )
            .await
            .expect("structured report");

        assert!(matches!(
            report.durable_reconciliation(),
            ShutdownDurableOutcome::Failed { .. }
        ));
        assert!(!report.is_complete());
        assert!(matches!(
            runtime.shutdown().await,
            Err(RuntimeError::ShutdownIncomplete { .. })
        ));
    }

    #[tokio::test]
    async fn shutdown_report_never_contains_submitted_content() {
        let (runtime, entered, _release) = gated_runtime().await;
        let entered_wait = entered.notified();
        runtime
            .submit("secret-prompt-and-credential")
            .await
            .expect("submit succeeds");
        entered_wait.await;
        let report = runtime
            .shutdown_with(
                ShutdownOptions::interrupt(Duration::from_secs(1)).expect("valid options"),
            )
            .await
            .expect("structured report");

        let projection = format!("{report:?}");
        assert!(!projection.contains("secret-prompt-and-credential"));
        assert!(!projection.contains("GatedModel"));
        runtime
            .shutdown()
            .await
            .expect("legacy wrapper joins report");
    }

    #[cfg(feature = "shared-composition")]
    struct PresetOverrideTool {
        executions: Arc<AtomicUsize>,
    }

    struct SnapshotEditingModel {
        step: AtomicUsize,
        observed_snapshot: Arc<StdMutex<Option<String>>>,
    }

    impl SnapshotEditingModel {
        fn new(observed_snapshot: Arc<StdMutex<Option<String>>>) -> Self {
            Self {
                step: AtomicUsize::new(0),
                observed_snapshot,
            }
        }
    }

    #[async_trait]
    impl LanguageModel for SnapshotEditingModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let step = self.step.fetch_add(1, Ordering::SeqCst);
            let events = if step == 0 {
                vec![
                    AgentEvent::TurnStart,
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "snapshot-read".into(),
                            name: "read".into(),
                            input: serde_json::json!({"path": "source.txt"}),
                        },
                        provenance: ToolProvenance::default(),
                        summary_fields: Vec::new(),
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: Usage::default(),
                    },
                ]
            } else if step == 1 {
                let content = messages
                    .iter()
                    .rev()
                    .find_map(|message| match message {
                        Message::Tool { result } => Some(result.content.as_str()),
                        _ => None,
                    })
                    .expect("model receives read result");
                let mut lines = content.lines();
                let snapshot_id = lines
                    .next()
                    .and_then(|line| line.strip_prefix("[snapshot:"))
                    .and_then(|line| line.strip_suffix(']'))
                    .expect("model receives snapshot handle")
                    .to_string();
                let target = lines
                    .next()
                    .and_then(|line| line.split_once('|'))
                    .map(|(reference, _)| reference.to_string())
                    .expect("model receives line reference");
                *self
                    .observed_snapshot
                    .lock()
                    .expect("snapshot capture lock") = Some(snapshot_id.clone());
                vec![
                    AgentEvent::TurnStart,
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "snapshot-edit".into(),
                            name: "edit".into(),
                            input: serde_json::json!({
                                "path": "source.txt",
                                "snapshot_id": snapshot_id,
                                "operations": [{
                                    "op": "replace",
                                    "target": target,
                                    "content": "updated"
                                }]
                            }),
                        },
                        provenance: ToolProvenance::default(),
                        summary_fields: Vec::new(),
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: Usage::default(),
                    },
                ]
            } else {
                vec![
                    AgentEvent::TurnStart,
                    AgentEvent::TextDelta {
                        delta: "done".into(),
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::EndTurn,
                        usage: Usage::default(),
                    },
                ]
            };
            let (tx, rx) = mpsc::channel(8);
            for event in events {
                tx.send(event).await.expect("runtime receiver remains open");
            }
            Ok(rx)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ApprovalRecord {
        tool_name: String,
        arguments: Value,
        summary_fields: Vec<String>,
    }

    struct RecordingApprovalHandler {
        choice: ApprovalChoice,
        records: Arc<StdMutex<Vec<ApprovalRecord>>>,
    }

    impl RecordingApprovalHandler {
        fn new(choice: ApprovalChoice, records: Arc<StdMutex<Vec<ApprovalRecord>>>) -> Self {
            Self { choice, records }
        }
    }

    #[async_trait]
    impl ApprovalHandler for RecordingApprovalHandler {
        async fn request_approval(
            &self,
            tool_name: &str,
            arguments: &Value,
            summary_fields: &[String],
        ) -> ApprovalChoice {
            self.records
                .lock()
                .expect("records lock is available")
                .push(ApprovalRecord {
                    tool_name: tool_name.to_string(),
                    arguments: arguments.clone(),
                    summary_fields: summary_fields.to_vec(),
                });
            self.choice.clone()
        }
    }

    #[cfg(feature = "shared-composition")]
    #[test]
    fn shared_tools_are_explicit_and_keep_a_unique_inventory() {
        let builder = RuntimeBuilder::new()
            .workspace_root("workspace")
            .shared_tools();
        let mut names = builder
            .tools
            .iter()
            .map(|tool| tool.name().to_string())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();

        assert_eq!(names.len(), builder.tools.len());
        assert!(names.iter().any(|name| name == "read"));
        assert!(
            names
                .iter()
                .any(|name| name == "bash" || name == "powershell")
        );
        assert!(names.iter().any(|name| name == "document_extract"));
        assert!(names.iter().any(|name| name == "read_image"));
    }

    #[cfg(feature = "shared-composition")]
    #[tokio::test]
    async fn coding_preset_is_explicit_and_builds_shared_inventory() {
        let builder = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new().with_response("done")))
            .workspace_root("workspace")
            .coding_preset();
        assert_eq!(builder.preset, Some(RuntimePreset::Coding));
        assert!(builder.tools.is_empty());
        let mut runtime = builder.build().expect("coding preset builds");
        runtime.submit("hello").await.expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[cfg(feature = "shared-composition")]
    #[tokio::test]
    async fn coding_preset_preserves_caller_tool_overrides() {
        let executions = Arc::new(AtomicUsize::new(0));
        let provider = MockProvider::new()
            .with_tool_call("bash", serde_json::json!({"command": "echo custom"}))
            .with_response("done");
        let mut runtime = RuntimeBuilder::new()
            .provider(Arc::new(provider))
            .tool(Arc::new(PresetOverrideTool {
                executions: executions.clone(),
            }))
            .permission_rule(PermissionRule {
                tool_name: "bash".into(),
                path_pattern: None,
                decision: PermissionDecision::Allow,
                nature: None,
                resource: None,
                resource_kind: None,
            })
            .coding_preset()
            .build()
            .expect("coding preset builds with caller override");

        runtime
            .submit("run the custom tool")
            .await
            .expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        assert_eq!(executions.load(Ordering::SeqCst), 1);
        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[cfg(not(feature = "shared-composition"))]
    #[test]
    fn coding_preset_requires_opt_in_feature() {
        let result = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .coding_preset()
            .build();
        assert!(matches!(
            result,
            Err(RuntimeError::CodingPresetRequiresFeature)
        ));
    }

    #[async_trait]
    impl AgentTool for RecordingWriteTool {
        fn name(&self) -> &str {
            "record_write"
        }

        fn description(&self) -> &str {
            "Records a write-like operation"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            })
        }

        async fn execute(&self, input: Value) -> ToolResult {
            self.executions.fetch_add(1, Ordering::SeqCst);
            let message = input
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default();
            ToolResult::success(format!("recorded: {message}"))
        }

        fn nature(&self) -> ToolNature {
            ToolNature::Write
        }

        fn summary_fields(&self) -> &'static [&'static str] {
            &["message"]
        }
    }

    #[cfg(feature = "shared-composition")]
    #[async_trait]
    impl AgentTool for PresetOverrideTool {
        fn name(&self) -> &str {
            "bash"
        }

        fn description(&self) -> &str {
            "Test-only caller-provided bash override"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"]
            })
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            self.executions.fetch_add(1, Ordering::SeqCst);
            ToolResult::success("custom bash")
        }

        fn nature(&self) -> ToolNature {
            ToolNature::Execute
        }
    }

    #[async_trait]
    impl AgentTool for RecordingHybridTool {
        fn name(&self) -> &str {
            "record_hybrid"
        }

        fn description(&self) -> &str {
            "Records a network plus write operation"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" },
                    "destination": { "type": "string" }
                },
                "required": ["url", "destination"]
            })
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            self.executions.fetch_add(1, Ordering::SeqCst);
            ToolResult::success("hybrid executed")
        }

        fn nature(&self) -> ToolNature {
            ToolNature::Write
        }

        fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
            vec![
                ToolPermissionFacet::with_resource(
                    ToolNature::Network,
                    "example.com",
                    ToolResourceKind::Domain,
                ),
                ToolPermissionFacet::with_resource(
                    ToolNature::Write,
                    "blocked/output.txt",
                    ToolResourceKind::Path,
                ),
            ]
        }
    }

    #[async_trait]
    impl AgentTool for PrivateInputWriteTool {
        fn name(&self) -> &str {
            "private_write"
        }

        fn description(&self) -> &str {
            "Projection test write"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            ToolResult::success("written")
        }

        fn nature(&self) -> ToolNature {
            ToolNature::Write
        }

        fn project_input(&self, input: &Value) -> Value {
            let mut input = input.clone();
            if let Some(object) = input.as_object_mut() {
                object.remove("snapshot_id");
            }
            input
        }
    }

    #[async_trait]
    impl AgentTool for PrivateResultReadTool {
        fn name(&self) -> &str {
            "private_read"
        }

        fn description(&self) -> &str {
            "Projection test read"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            ToolResult::success("[snapshot:s1]\n1:aa|private line")
        }

        fn is_read_only(&self) -> bool {
            true
        }

        fn project_result(&self, result: &ToolResult) -> talos_core::tool::ToolResultProjection {
            talos_core::tool::ToolResultProjection {
                model_content: result.content.clone(),
                display_content: "read 1 line".into(),
                persistence_content: "read 1 line".into(),
            }
        }
    }

    #[tokio::test]
    async fn runtime_streams_mock_response() {
        let provider = Arc::new(MockProvider::new().with_response("hello from runtime"));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .build()
            .expect("runtime builds");

        runtime.submit("hello").await.expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        match status {
            TurnCompletionStatus::Success { final_text, .. } => {
                assert_eq!(final_text, "hello from runtime");
            }
            other => panic!("unexpected status: {other:?}"),
        }

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_denies_ask_tools_by_default() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("record_write", serde_json::json!({"message": "secret"}))
                .with_response("done"),
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let tool = Arc::new(RecordingWriteTool {
            executions: executions.clone(),
        });
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .tool(tool)
            .build()
            .expect("runtime builds");

        runtime
            .submit("write something")
            .await
            .expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        assert!(matches!(
            status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "done"
        ));
        assert_eq!(executions.load(Ordering::SeqCst), 0);

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_allows_tool_when_rule_allows_write() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("record_write", serde_json::json!({"message": "allowed"}))
                .with_response("done"),
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let tool = Arc::new(RecordingWriteTool {
            executions: executions.clone(),
        });
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .permission_rule(PermissionRule::new_nature(
                ToolNature::Write,
                None,
                None,
                PermissionDecision::Allow,
            ))
            .tool(tool)
            .build()
            .expect("runtime builds");

        runtime
            .submit("write something")
            .await
            .expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        assert!(matches!(
            status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "done"
        ));
        assert_eq!(executions.load(Ordering::SeqCst), 1);

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_approval_handler_can_approve_ask_tool() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("record_write", serde_json::json!({"message": "approved"}))
                .with_response("done"),
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let approval_records = Arc::new(StdMutex::new(Vec::new()));
        let tool = Arc::new(RecordingWriteTool {
            executions: executions.clone(),
        });
        let approval_handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::ApproveOnce,
            approval_records.clone(),
        ));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .approval_handler(approval_handler)
            .tool(tool)
            .build()
            .expect("runtime builds");

        runtime
            .submit("write something")
            .await
            .expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        assert!(matches!(
            status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "done"
        ));
        assert_eq!(executions.load(Ordering::SeqCst), 1);
        {
            let records = approval_records.lock().expect("records lock is available");
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].tool_name, "record_write");
            assert_eq!(
                records[0].arguments,
                serde_json::json!({"message": "approved"})
            );
            assert_eq!(records[0].summary_fields, vec!["message"]);
        }

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn external_read_requires_approval_and_receives_exact_path_authorization() {
        let workspace = tempfile::tempdir().expect("workspace");
        let external = tempfile::NamedTempFile::new().expect("external file");
        std::fs::write(external.path(), "external content").expect("write fixture");
        let records = Arc::new(StdMutex::new(Vec::new()));
        let handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::ApproveOnce,
            records.clone(),
        ));
        let tool = RuntimePermissionAwareTool {
            inner: Arc::new(ReadTool::new(workspace.path().to_path_buf())),
            engine: Arc::new(Mutex::new(PermissionEngine::with_workspace_root(
                workspace.path().to_path_buf(),
            ))),
            approval_handler: Some(handler),
        };

        let result = tool
            .execute(serde_json::json!({"path": external.path().to_string_lossy()}))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("external content"));
        assert_eq!(records.lock().expect("records lock").len(), 1);
    }

    #[tokio::test]
    async fn external_read_without_handler_fails_closed() {
        let workspace = tempfile::tempdir().expect("workspace");
        let external = tempfile::NamedTempFile::new().expect("external file");
        std::fs::write(external.path(), "must not be read").expect("write fixture");
        let tool = RuntimePermissionAwareTool {
            inner: Arc::new(ReadTool::new(workspace.path().to_path_buf())),
            engine: Arc::new(Mutex::new(PermissionEngine::with_workspace_root(
                workspace.path().to_path_buf(),
            ))),
            approval_handler: None,
        };

        let result = tool
            .execute(serde_json::json!({"path": external.path().to_string_lossy()}))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("approval required"));
        assert!(!result.content.contains("must not be read"));
    }

    #[tokio::test]
    async fn external_read_explicit_denial_fails_closed() {
        let workspace = tempfile::tempdir().expect("workspace");
        let external = tempfile::NamedTempFile::new().expect("external file");
        std::fs::write(external.path(), "private").expect("write fixture");
        let handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::Deny,
            Arc::new(StdMutex::new(Vec::new())),
        ));
        let tool = RuntimePermissionAwareTool {
            inner: Arc::new(ReadTool::new(workspace.path().to_path_buf())),
            engine: Arc::new(Mutex::new(PermissionEngine::with_workspace_root(
                workspace.path().to_path_buf(),
            ))),
            approval_handler: Some(handler),
        };

        let result = tool
            .execute(serde_json::json!({"path": external.path().to_string_lossy()}))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("User denied"));
        assert!(!result.content.contains("private"));
    }

    #[tokio::test]
    async fn external_read_always_approve_reuses_exact_rule_without_second_prompt() {
        let workspace = tempfile::tempdir().expect("workspace");
        let external = tempfile::NamedTempFile::new().expect("external file");
        std::fs::write(external.path(), "external content").expect("write fixture");
        let records = Arc::new(StdMutex::new(Vec::new()));
        let handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::AlwaysApprove,
            records.clone(),
        ));
        let tool = RuntimePermissionAwareTool {
            inner: Arc::new(ReadTool::new(workspace.path().to_path_buf())),
            engine: Arc::new(Mutex::new(PermissionEngine::with_workspace_root(
                workspace.path().to_path_buf(),
            ))),
            approval_handler: Some(handler),
        };
        let input = serde_json::json!({"path": external.path().to_string_lossy()});

        let first = tool.execute(input.clone()).await;
        let second = tool.execute(input).await;

        assert!(!first.is_error, "{}", first.content);
        assert!(!second.is_error, "{}", second.content);
        assert_eq!(
            records.lock().expect("records lock").len(),
            1,
            "persisted exact-path Allow must suppress the second prompt"
        );
    }

    #[tokio::test]
    async fn runtime_approval_receives_projected_input_without_private_snapshot_id() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call(
                    "private_write",
                    serde_json::json!({"path": "src/lib.rs", "snapshot_id": "s1"}),
                )
                .with_response("done"),
        );
        let approval_records = Arc::new(StdMutex::new(Vec::new()));
        let approval_handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::ApproveOnce,
            approval_records.clone(),
        ));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .approval_handler(approval_handler)
            .tool(Arc::new(PrivateInputWriteTool))
            .build()
            .expect("runtime builds");

        runtime.submit("write").await.expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        {
            let records = approval_records.lock().expect("records lock");
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].arguments,
                serde_json::json!({"path": "src/lib.rs"})
            );
        }
        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_always_approve_installs_in_memory_rule() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("record_write", serde_json::json!({"message": "first"}))
                .with_response("first done")
                .with_tool_call("record_write", serde_json::json!({"message": "second"}))
                .with_response("second done"),
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let approval_records = Arc::new(StdMutex::new(Vec::new()));
        let tool = Arc::new(RecordingWriteTool {
            executions: executions.clone(),
        });
        let approval_handler = Arc::new(RecordingApprovalHandler::new(
            ApprovalChoice::AlwaysApprove,
            approval_records.clone(),
        ));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .approval_handler(approval_handler)
            .tool(tool)
            .build()
            .expect("runtime builds");

        runtime
            .submit("write first")
            .await
            .expect("first submit succeeds");
        let first_status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("first turn completes");
        runtime
            .submit("write second")
            .await
            .expect("second submit succeeds");
        let second_status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("second turn completes");

        assert!(matches!(
            first_status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "first done"
        ));
        assert!(matches!(
            second_status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "second done"
        ));
        assert_eq!(executions.load(Ordering::SeqCst), 2);
        assert_eq!(
            approval_records
                .lock()
                .expect("records lock is available")
                .len(),
            1
        );

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_denies_hybrid_tool_when_write_facet_is_denied() {
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call(
                    "record_hybrid",
                    serde_json::json!({
                        "url": "https://example.com/file",
                        "destination": "blocked/output.txt"
                    }),
                )
                .with_response("done"),
        );
        let executions = Arc::new(AtomicUsize::new(0));
        let tool = Arc::new(RecordingHybridTool {
            executions: executions.clone(),
        });
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(".")
            .permission_rule(PermissionRule::new_nature(
                ToolNature::Network,
                Some("example.com".to_string()),
                Some(talos_permission::ResourceKind::Domain),
                PermissionDecision::Allow,
            ))
            .permission_rule(PermissionRule::new_nature(
                ToolNature::Write,
                Some("blocked/**".to_string()),
                Some(talos_permission::ResourceKind::Path),
                PermissionDecision::Deny("write blocked".to_string()),
            ))
            .tool(tool)
            .build()
            .expect("runtime builds");

        runtime
            .submit("fetch and save")
            .await
            .expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        assert!(matches!(
            status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "done"
        ));
        assert_eq!(executions.load(Ordering::SeqCst), 0);

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_accepts_initial_history() {
        let provider = Arc::new(MockProvider::new().with_response("continued"));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .initial_history(vec![Message::User {
                content: "earlier".into(),
            }])
            .build()
            .expect("runtime builds");

        runtime.submit("continue").await.expect("submit succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(
            status,
            TurnCompletionStatus::Success { final_text, .. } if final_text == "continued"
        ));

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn durable_runtime_restores_history_and_reports_committed_entries() {
        let directory = std::env::temp_dir().join(format!(
            "talos-runtime-durable-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        let manager = SessionManager::with_dir(directory.clone());
        let session = manager
            .create_or_open_session("task:durable-runtime")
            .expect("durable session");
        let mut runtime = RuntimeBuilder::new()
            .provider(Arc::new(
                MockProvider::new().with_response("persisted answer"),
            ))
            .durable_session(session.clone())
            .build()
            .expect("runtime builds");

        runtime
            .submit("persist this user turn")
            .await
            .expect("submit");
        let mut committed = false;
        while let Some(event) = runtime.next_event().await {
            if let SessionEvent::EntriesCommitted { entry_ids, .. } = &event {
                committed = !entry_ids.is_empty();
            }
            if matches!(
                event,
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { .. },
                    ..
                }
            ) {
                break;
            }
        }
        assert!(committed, "durable success must report committed entry IDs");
        runtime.shutdown().await.expect("shutdown");

        let restored = manager
            .get_session_by_external_id("task:durable-runtime")
            .expect("lookup")
            .expect("binding exists");
        let history = restored.read_messages().expect("history");
        assert!(history.iter().any(|message| matches!(message, Message::User { content } if content == "persist this user turn")));
        assert!(history.iter().any(|message| matches!(message, Message::Assistant { content, .. } if content == "persisted answer")));
        std::fs::remove_dir_all(directory).expect("cleanup");
    }

    #[tokio::test]
    async fn durable_runtime_never_persists_model_private_tool_result() {
        let directory = std::env::temp_dir().join(format!(
            "talos-runtime-private-projection-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        let manager = SessionManager::with_dir(directory.clone());
        let session = manager
            .create_or_open_session("private-projection")
            .expect("durable session");
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("private_read", serde_json::json!({"path": "src/lib.rs"}))
                .with_response("done"),
        );
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .tool(Arc::new(PrivateResultReadTool))
            .durable_session(session.clone())
            .build()
            .expect("runtime builds");

        runtime.submit("read").await.expect("submit");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        runtime.shutdown().await.expect("shutdown");

        let messages = session.read_messages().expect("messages");
        let serialized = serde_json::to_string(&messages).expect("serialize messages");
        assert!(serialized.contains("read 1 line"));
        assert!(!serialized.contains("snapshot:s1"));
        assert!(!serialized.contains("1:aa|"));
        for entry in std::fs::read_dir(&directory).expect("session directory") {
            let path = entry.expect("directory entry").path();
            if path.extension().and_then(|value| value.to_str()) == Some("tlog") {
                let bytes = std::fs::read(&path).expect("tlog bytes");
                let text = String::from_utf8_lossy(&bytes);
                assert!(!text.contains("snapshot:s1"));
                assert!(!text.contains("1:aa|"));
            }
        }
        std::fs::remove_dir_all(directory).expect("cleanup");
    }

    #[cfg(feature = "shared-composition")]
    #[tokio::test]
    async fn shared_composition_runtime_executes_read_tool() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("note.txt"), "shared runtime\n").expect("fixture");
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call("read", serde_json::json!({"path": "note.txt"}))
                .with_response("read complete"),
        );
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(workspace.path())
            .shared_tools()
            .build()
            .expect("shared runtime builds");

        runtime.submit("read note").await.expect("submit");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        runtime.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    async fn real_snapshot_read_to_edit_is_atomic_permission_gated_and_never_durable() {
        let workspace = tempfile::tempdir().expect("workspace");
        let session_root = tempfile::tempdir().expect("session root");
        let source = workspace.path().join("source.txt");
        std::fs::write(&source, "original\n").expect("fixture write");
        let manager = SessionManager::with_dir(session_root.path().join("messages"));
        let session = manager
            .create_or_open_session("snapshot:e2e")
            .expect("durable session");
        let observed_snapshot = Arc::new(StdMutex::new(None));
        let provider = Arc::new(SnapshotEditingModel::new(observed_snapshot.clone()));
        let (read, write, edit, delete) = snapshot_aware_file_tools(workspace.path().to_path_buf());
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(workspace.path())
            .permission_rule(PermissionRule::new_nature(
                ToolNature::Write,
                None,
                None,
                PermissionDecision::Allow,
            ))
            .tool(Arc::new(read))
            .tool(Arc::new(write))
            .tool(Arc::new(edit))
            .tool(Arc::new(delete))
            .durable_session(session.clone())
            .build()
            .expect("runtime builds");

        runtime
            .submit("update the first line")
            .await
            .expect("submit");
        let mut serialized_events = Vec::new();
        while let Some(event) = runtime.next_event().await {
            serialized_events.push(serde_json::to_string(&event).expect("serialize event"));
            if matches!(
                event,
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { .. },
                    ..
                }
            ) {
                break;
            }
        }
        runtime.shutdown().await.expect("shutdown");

        assert_eq!(
            std::fs::read_to_string(&source).expect("read source"),
            "updated\n"
        );
        let snapshot_id = observed_snapshot
            .lock()
            .expect("snapshot lock")
            .clone()
            .expect("model observed snapshot");
        for event in &serialized_events {
            assert!(!event.contains(&snapshot_id));
            assert!(!event.contains("snapshot_id"));
        }
        let messages = session.read_messages().expect("durable messages");
        let serialized = serde_json::to_string(&messages).expect("serialize messages");
        assert!(!serialized.contains(&snapshot_id));
        assert!(!serialized.contains("snapshot_id"));
        assert!(serialized.contains("1: original"));
        for entry in
            std::fs::read_dir(session_root.path().join("messages")).expect("session directory")
        {
            let path = entry.expect("directory entry").path();
            if path.extension().and_then(|value| value.to_str()) == Some("tlog") {
                let bytes = std::fs::read(path).expect("tlog bytes");
                let text = String::from_utf8_lossy(&bytes);
                assert!(!text.contains(&snapshot_id));
                assert!(!text.contains("snapshot_id"));
            }
        }
    }

    #[tokio::test]
    async fn denied_real_snapshot_edit_leaves_the_file_unchanged() {
        let workspace = tempfile::tempdir().expect("workspace");
        let source = workspace.path().join("source.txt");
        std::fs::write(&source, "original\n").expect("fixture write");
        let observed_snapshot = Arc::new(StdMutex::new(None));
        let provider = Arc::new(SnapshotEditingModel::new(observed_snapshot.clone()));
        let (read, _, edit, _) = snapshot_aware_file_tools(workspace.path().to_path_buf());
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(workspace.path())
            .tool(Arc::new(read))
            .tool(Arc::new(edit))
            .build()
            .expect("runtime builds");

        runtime
            .submit("update the first line")
            .await
            .expect("submit");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");
        runtime.shutdown().await.expect("shutdown");

        assert!(matches!(status, TurnCompletionStatus::Success { .. }));
        assert!(observed_snapshot.lock().expect("snapshot lock").is_some());
        assert_eq!(
            std::fs::read_to_string(source).expect("read source"),
            "original\n"
        );
    }

    #[tokio::test]
    async fn runtime_previews_request_without_submit_magic_string() {
        let provider = Arc::new(MockProvider::new().with_request_debug_builder(|messages| {
            serde_json::to_string(messages).expect("messages serialize")
        }));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .build()
            .expect("runtime builds");

        runtime
            .preview_request("inspect request")
            .await
            .expect("preview request succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        match status {
            TurnCompletionStatus::Success { final_text, .. } => {
                assert!(final_text.contains("Request preview (no API call made)"));
                assert!(final_text.contains("inspect request"));
            }
            other => panic!("unexpected status: {other:?}"),
        }

        runtime.shutdown().await.expect("shutdown succeeds");
    }

    #[tokio::test]
    async fn runtime_builder_custom_prompt_replaces_default_identity() {
        let provider = Arc::new(MockProvider::new().with_request_debug_builder(|messages| {
            serde_json::to_string(messages).expect("messages serialize")
        }));
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .custom_prompt("You are Obei Buddy, a zh-CN office assistant.")
            .append_prompt("Answer in concise business Chinese.")
            .build()
            .expect("runtime builds");

        runtime
            .preview_request("inspect request")
            .await
            .expect("preview request succeeds");
        let status = collect_until_turn_completed(&mut runtime)
            .await
            .expect("turn completes");

        match status {
            TurnCompletionStatus::Success { final_text, .. } => {
                assert!(final_text.contains("You are Obei Buddy"));
                assert!(final_text.contains("Answer in concise business Chinese."));
                assert!(
                    !final_text.contains("You are Talos, an AI coding assistant"),
                    "custom prompt should replace the default Talos identity"
                );
            }
            other => panic!("unexpected status: {other:?}"),
        }

        runtime.shutdown().await.expect("shutdown succeeds");
    }
}
