//! Embeddable Talos agent runtime facade.
//!
//! This crate is the SDK-style entrypoint for Rust projects that want to reuse
//! Talos's agent turn loop without depending on the Talos CLI or TUI crates.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use talos_agent::session::AppServerSession;
use talos_agent::{Agent, AgentError};
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
use tokio::task::JoinHandle;

pub use talos_core::message::{AgentEvent, MessageToolResult, StopReason, ToolCall, Usage};
pub use talos_core::provider::{ProviderError, ToolDefinition};
pub use talos_core::tool::{ToolNature, ToolProvenance};
pub use talos_plugin::HookRegistry as RuntimeHookRegistry;
pub use talos_skill::SkillIndex as RuntimeSkillIndex;

/// Errors returned by the embeddable runtime facade.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// The builder cannot create a runtime without a provider.
    #[error("runtime provider is required")]
    MissingProvider,

    /// A command could not be sent because the runtime actor is closed.
    #[error("runtime command channel is closed")]
    CommandChannelClosed,

    /// The runtime actor task failed to join.
    #[error("runtime actor failed: {0}")]
    ActorJoin(#[from] tokio::task::JoinError),

    /// The underlying agent returned an error.
    #[error("agent error: {0}")]
    Agent(#[from] AgentError),

    /// A durable session could not be read or committed.
    #[error("durable session error: {0}")]
    Session(#[from] talos_session::SessionError),
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
    initial_history: Vec<Message>,
    model_context_limit: u32,
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
    custom_prompt: Option<String>,
    append_prompt: Option<String>,
    hook_registry: Option<Arc<HookRegistry>>,
    skill_index: Vec<SkillIndex>,
    durable_session: Option<(DurableSession, PersistencePolicy)>,
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
            initial_history: Vec::new(),
            model_context_limit: 128_000,
            approval_handler: None,
            custom_prompt: None,
            append_prompt: None,
            hook_registry: None,
            skill_index: Vec::new(),
            durable_session: None,
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

    /// Builds and starts the runtime actor.
    ///
    /// The returned handle owns the command sender, event receiver, and actor
    /// task. Dropping the handle drops those channels; prefer
    /// [`RuntimeHandle::shutdown`] for orderly shutdown.
    pub fn build(self) -> RuntimeResult<RuntimeHandle> {
        let provider = self.provider.ok_or(RuntimeError::MissingProvider)?;
        let tool_engine = Arc::new(Mutex::new(build_permission_engine(
            self.workspace_root.clone(),
            &self.permission_rules,
        )));
        let agent_engine = Arc::new(build_permission_engine(
            self.workspace_root.clone(),
            &self.permission_rules,
        ));

        let mut registry = ToolRegistry::new();
        for tool in self.tools {
            registry.register(Arc::new(RuntimePermissionAwareTool {
                inner: tool,
                engine: tool_engine.clone(),
                approval_handler: self.approval_handler.clone(),
            }));
        }

        let mut agent = if let Some(hooks) = self.hook_registry {
            Agent::with_security_and_hooks(
                provider,
                registry,
                Some(agent_engine),
                self.sandbox,
                self.workspace_root.clone(),
                hooks,
            )
        } else {
            Agent::with_security(
                provider,
                registry,
                Some(agent_engine),
                self.sandbox,
                self.workspace_root.clone(),
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
        if let Some((session, policy)) = self.durable_session {
            actor.set_durable_persistence(session, policy);
        }
        let actor_task = tokio::spawn(async move {
            actor.run().await;
        });

        Ok(RuntimeHandle {
            command_tx: handle.sq_tx,
            event_rx: handle.eq_rx,
            actor_task,
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
    actor_task: JoinHandle<()>,
}

impl RuntimeHandle {
    /// Submits a user message as a new turn.
    pub async fn submit(&self, message: impl Into<String>) -> RuntimeResult<()> {
        self.command_tx
            .send(SessionOp::Submit {
                message: message.into(),
            })
            .await
            .map_err(|_| RuntimeError::CommandChannelClosed)
    }

    /// Requests a provider request preview without making a provider call.
    pub async fn preview_request(&self, message: impl Into<String>) -> RuntimeResult<()> {
        self.command_tx
            .send(SessionOp::PreviewRequest {
                message: message.into(),
            })
            .await
            .map_err(|_| RuntimeError::CommandChannelClosed)
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

    /// Shuts down the runtime actor and waits for it to finish.
    pub async fn shutdown(self) -> RuntimeResult<()> {
        let _ = self.command_tx.send(SessionOp::Shutdown).await;
        self.actor_task.await?;
        Ok(())
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use talos_core::message::Message;
    use talos_core::provider::ProviderResult;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
    use talos_permission::PermissionDecision;
    use talos_provider::mock::MockProvider;
    use talos_session::SessionManager;
    use talos_tools::{ReadTool, snapshot_aware_file_tools};

    use super::*;

    struct RecordingWriteTool {
        executions: Arc<AtomicUsize>,
    }

    struct RecordingHybridTool {
        executions: Arc<AtomicUsize>,
    }

    struct PrivateInputWriteTool;

    struct PrivateResultReadTool;

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
        let records = approval_records.lock().expect("records lock is available");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool_name, "record_write");
        assert_eq!(
            records[0].arguments,
            serde_json::json!({"message": "approved"})
        );
        assert_eq!(records[0].summary_fields, vec!["message"]);

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
        let records = approval_records.lock().expect("records lock");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].arguments,
            serde_json::json!({"path": "src/lib.rs"})
        );
        drop(records);
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
