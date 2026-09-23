//! Thin Desktop adapter for the existing Talos Runtime.
//!
//! The adapter owns one Tokio runtime task and forwards bounded commands and
//! authoritative session events to the presentation layer. It deliberately
//! does not create tools, permissions, storage, or a second execution engine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use talos_runtime::{
    AgentEvent, ApprovalChoice, ApprovalHandler, GrantPreview, LanguageModel, RuntimeBuilder,
    RuntimeHandle, SessionEvent,
};
use tokio::sync::{mpsc, oneshot};

const COMMAND_CAPACITY: usize = 16;
const EVENT_CAPACITY: usize = 128;
const MAX_PENDING_BYTES: usize = 1024 * 1024;

#[derive(Default)]
struct PendingOutput {
    queue: std::collections::VecDeque<RuntimeOutput>,
    bytes: usize,
}

impl PendingOutput {
    fn push(&mut self, output: RuntimeOutput) -> Result<(), ()> {
        let bytes = output_bytes(&output);
        if self.queue.len() >= EVENT_CAPACITY
            || bytes > MAX_PENDING_BYTES.saturating_sub(self.bytes)
        {
            return Err(());
        }
        self.bytes += bytes;
        self.queue.push_back(output);
        Ok(())
    }

    fn pop(&mut self) -> Option<RuntimeOutput> {
        let output = self.queue.pop_front()?;
        self.bytes -= output_bytes(&output);
        Some(output)
    }
}

fn output_bytes(output: &RuntimeOutput) -> usize {
    match output {
        RuntimeOutput::Text(text) | RuntimeOutput::Error(text) => text.len(),
        RuntimeOutput::ToolStarted { call_id, name } => call_id.len() + name.len(),
        RuntimeOutput::ToolEvidence {
            call_id,
            provenance,
        } => call_id.len() + provenance.len(),
        RuntimeOutput::ToolResult {
            call_id, content, ..
        } => call_id.len() + content.len(),
        RuntimeOutput::ApprovalRequested {
            tool_name,
            scope,
            explanation,
            ..
        } => tool_name.len() + scope.len() + explanation.len(),
        RuntimeOutput::AutoDecision {
            outcome,
            reason,
            evaluator,
        } => outcome.len() + reason.len() + evaluator.len(),
        RuntimeOutput::Started { turn_id } => turn_id.len(),
        RuntimeOutput::Completed {
            status: TerminalStatus::Error(error),
        } => error.len(),
        _ => 0,
    }
}

/// Commands accepted by a live Desktop runtime host.
#[derive(Debug)]
pub(crate) enum RuntimeCommand {
    /// Submit one user message to the existing Runtime.
    Submit(String),
    /// Interrupt the current Runtime turn.
    Interrupt,
    /// Request bounded Runtime shutdown.
    Shutdown,
    /// Resolve one exact pending approval request.
    ApprovalResponse {
        request_id: u64,
        choice: ApprovalChoice,
    },
}

/// Presentation-safe projection of an authoritative Runtime event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeOutput {
    /// A tool call was requested; this does not imply successful execution.
    ToolStarted { call_id: String, name: String },
    /// Read-only provenance for a tool call; absence is represented explicitly.
    ToolEvidence { call_id: String, provenance: String },
    /// A tool result projected from the Runtime.
    ToolResult {
        call_id: String,
        content: String,
        is_error: bool,
    },
    /// Redacted result of the existing Auto evaluator.
    AutoDecision {
        outcome: String,
        reason: String,
        evaluator: String,
    },
    /// A permission-gated request awaiting the user.
    ApprovalRequested {
        request_id: u64,
        tool_name: String,
        scope: String,
        explanation: String,
    },
    /// The request lifetime ended; any old UI controls must be discarded.
    ApprovalClosed { request_id: u64 },
    /// A model text fragment.
    Text(String),
    /// A turn began.
    Started { turn_id: String },
    /// A turn reached a terminal status.
    Completed { status: TerminalStatus },
    /// The Runtime reported an error.
    Error(String),
    /// The host has stopped accepting work.
    Stopped,
}

/// Small terminal projection; never exposes Runtime message-history internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalStatus {
    Success,
    Cancelled,
    Error(String),
}

struct DesktopApprovalHandler {
    events: mpsc::Sender<RuntimeOutput>,
    output_overflow: tokio::sync::Notify,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ApprovalChoice>>>>,
    workspace_root: PathBuf,
}

static NEXT_APPROVAL_ID: AtomicU64 = AtomicU64::new(1);

fn workspace_external_id(workspace_root: &std::path::Path) -> String {
    format!(
        "desktop-workspace-{}",
        workspace_root
            .to_string_lossy()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>()
    )
}

pub(crate) fn task_external_id(workspace_root: &std::path::Path, goal: &str) -> String {
    let goal = goal
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .take(80)
        .collect::<String>();
    format!(
        "desktop-task-{}-{}",
        workspace_external_id(workspace_root),
        goal
    )
}

struct ApprovalLifetime<'a> {
    handler: &'a DesktopApprovalHandler,
    request_id: u64,
}

impl Drop for ApprovalLifetime<'_> {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.handler.pending.lock() {
            pending.remove(&self.request_id);
        }
        self.handler.publish(RuntimeOutput::ApprovalClosed {
            request_id: self.request_id,
        });
    }
}

impl DesktopApprovalHandler {
    fn new(events: mpsc::Sender<RuntimeOutput>, workspace_root: PathBuf) -> Self {
        Self {
            events,
            output_overflow: tokio::sync::Notify::new(),
            pending: Arc::new(Mutex::new(HashMap::new())),
            workspace_root,
        }
    }

    async fn resolve(&self, request_id: u64, choice: ApprovalChoice) -> bool {
        self.pending
            .lock()
            .ok()
            .and_then(|mut pending| pending.remove(&request_id))
            .map(|sender| sender.send(choice).is_ok())
            .unwrap_or(false)
    }

    // Synchronous observers cannot await channel space. Overflow stops the host
    // through its independent terminal receipt instead of losing approval facts.
    fn publish(&self, output: RuntimeOutput) {
        if self.events.try_send(output).is_err() {
            self.output_overflow.notify_one();
        }
    }
}

#[async_trait]
impl ApprovalHandler for DesktopApprovalHandler {
    async fn request_approval(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _summary_fields: &[String],
    ) -> ApprovalChoice {
        // No compiler-derived scope means there is nothing safe to offer for approval.
        ApprovalChoice::Deny
    }

    async fn request_scoped_approval(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        summary_fields: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        self.request_scoped_approval_with_explanation(
            tool_name,
            arguments,
            summary_fields,
            preview,
            "",
        )
        .await
    }

    async fn request_scoped_approval_with_explanation(
        &self,
        tool_name: &str,
        _arguments: &serde_json::Value,
        _summary_fields: &[String],
        preview: &GrantPreview,
        explanation: &str,
    ) -> ApprovalChoice {
        let scope = preview
            .facets()
            .iter()
            .map(|facet| {
                format!(
                    "{:?} {:?}: {}",
                    facet.nature, facet.resource_kind, facet.normalized_scope
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let scope = self.redact_scope(&scope);
        self.request_ui(tool_name, &scope, explanation).await
    }
}

impl DesktopApprovalHandler {
    fn redact_scope(&self, scope: &str) -> String {
        let mut value = scope.replace(&*self.workspace_root.to_string_lossy(), "<workspace>");
        for variable in ["HOME", "USERPROFILE"] {
            if let Ok(home) = std::env::var(variable) {
                value = value.replace(&home, "<home>");
            }
        }
        for key in ["api_key", "token", "password", "secret"] {
            if value.to_ascii_lowercase().contains(key) {
                return "<redacted scope>".into();
            }
        }
        value
    }

    async fn request_ui(&self, tool_name: &str, scope: &str, explanation: &str) -> ApprovalChoice {
        if tool_name.len() + scope.len() + explanation.len() > 16 * 1024 {
            return ApprovalChoice::Deny;
        }
        let Ok(request_id) =
            NEXT_APPROVAL_ID
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        else {
            return ApprovalChoice::Deny;
        };
        let (sender, receiver) = oneshot::channel();
        {
            let Ok(mut pending) = self.pending.lock() else {
                return ApprovalChoice::Deny;
            };
            if !pending.is_empty() {
                return ApprovalChoice::Deny;
            }
            pending.insert(request_id, sender);
        }
        let _lifetime = ApprovalLifetime {
            handler: self,
            request_id,
        };
        let request = RuntimeOutput::ApprovalRequested {
            request_id,
            tool_name: tool_name.to_owned(),
            scope: scope.to_owned(),
            explanation: explanation.to_owned(),
        };
        if self.events.send(request).await.is_err() {
            return ApprovalChoice::Deny;
        }
        tokio::select! {
            result = receiver => result.unwrap_or(ApprovalChoice::Deny),
            _ = self.events.closed() => ApprovalChoice::Deny,
        }
    }
}

/// Handle used by the Desktop presentation to communicate with a live host.
pub(crate) struct RuntimeHost {
    commands: mpsc::Sender<RuntimeCommand>,
    outputs: mpsc::Receiver<RuntimeOutput>,
    terminal: Option<tokio::sync::oneshot::Receiver<RuntimeOutput>>,
    exit: Option<HostExit>,
}

/// Application-owned completion receipt, independent of GPUI task lifetime.
pub(crate) struct HostExit {
    commands: mpsc::Sender<RuntimeCommand>,
    result: std::sync::mpsc::Receiver<RuntimeOutput>,
}

impl HostExit {
    /// Called only after the GUI event loop has ended.
    pub(crate) fn finish(self, timeout: std::time::Duration) -> Result<(), String> {
        let _ = self.commands.try_send(RuntimeCommand::Shutdown);
        match self.result.recv_timeout(timeout) {
            Ok(RuntimeOutput::Stopped) => Ok(()),
            Ok(RuntimeOutput::Error(error)) => Err(error),
            Ok(_) => Err("host returned an invalid shutdown result".into()),
            Err(error) => Err(format!("host shutdown was not confirmed: {error}")),
        }
    }
}

impl RuntimeHost {
    /// Start one host task around an already constructed provider.
    #[cfg(test)]
    pub(crate) fn start(
        provider: Arc<dyn LanguageModel>,
        workspace_root: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        Self::start_with(move || Ok((provider, 128_000, false)), workspace_root, None)
    }

    /// Load configuration and bind the host to a caller-selected durable session identity.
    pub(crate) fn configured_for_session(
        workspace_root: impl Into<PathBuf>,
        external_id: impl Into<String>,
    ) -> Result<Self, String> {
        Self::configured_with_identity(workspace_root.into(), Some(external_id.into()))
    }

    fn configured_with_identity(
        workspace_root: PathBuf,
        external_id: Option<String>,
    ) -> Result<Self, String> {
        let session_root = workspace_root.join(".talos").join("desktop-sessions");
        let external_id = external_id.unwrap_or_else(|| workspace_external_id(&workspace_root));
        Self::start_with(
            || {
                let config = talos_config::Config::load().map_err(|error| error.to_string())?;
                let provider = crate::provider::configured_provider(&config)?;
                Ok((
                    provider,
                    config.resolve_model_limits().0,
                    config.auto.enabled,
                ))
            },
            workspace_root,
            Some((session_root, external_id)),
        )
    }

    fn start_with(
        provider: impl FnOnce() -> Result<(Arc<dyn LanguageModel>, u32, bool), String> + Send + 'static,
        workspace_root: impl Into<PathBuf>,
        durable_identity: Option<(PathBuf, String)>,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root.into();
        let (commands, command_rx) = mpsc::channel(COMMAND_CAPACITY);
        let (outputs, output_rx) = mpsc::channel(EVENT_CAPACITY);
        let (terminal_tx, terminal_rx) = tokio::sync::oneshot::channel();
        let (exit_tx, exit_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("talos-desktop-runtime".into())
            .spawn(move || {
                let executor = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let terminal = match executor {
                    Ok(executor) => executor.block_on(async move {
                        let (provider, context_limit, auto_enabled) = match provider() {
                            Ok(provider) => provider,
                            Err(error) => {
                                return RuntimeOutput::Error(error);
                            }
                        };
                        let approval_root = workspace_root.clone();
                        let approval =
                            Arc::new(DesktopApprovalHandler::new(outputs.clone(), approval_root));
                        let report_output = approval.clone();
                        let durable_session = durable_identity
                            .as_ref()
                            .map(|(root, external_id)| {
                                talos_session::DurableSession::open_or_create(root, external_id)
                                    .map_err(|error| error.to_string())
                            })
                            .transpose();
                        let durable_session = match durable_session {
                            Ok(session) => session,
                            Err(error) => return RuntimeOutput::Error(error),
                        };
                        let builder = RuntimeBuilder::new()
                            .provider(provider)
                            .model_context_limit(context_limit)
                            .workspace_root(workspace_root)
                            .shared_tools()
                            .sandbox(talos_sandbox::create_sandbox())
                            .sandbox_fallback_policy(talos_runtime::SandboxFallbackPolicy::Deny)
                            .approval_handler(approval.clone())
                            .permission_mode(talos_runtime::PermissionMode::Interactive)
                            .auto_assistance(auto_enabled)
                            .auto_report_sink(Arc::new(move |report| {
                                report_output.publish(RuntimeOutput::AutoDecision {
                                    outcome: report.outcome,
                                    reason: report.reason,
                                    evaluator: report.evaluator,
                                });
                            }));
                        let builder = match durable_session {
                            Some(session) => builder.durable_session(session),
                            None => builder,
                        };
                        match builder.build() {
                            Ok(handle) => {
                                run_host(command_rx, outputs, handle, Some(approval)).await
                            }
                            Err(error) => RuntimeOutput::Error(error.to_string()),
                        }
                    }),
                    Err(error) => {
                        RuntimeOutput::Error(format!("runtime host unavailable: {error}"))
                    }
                };
                let _ = terminal_tx.send(terminal.clone());
                let _ = exit_tx.send(terminal);
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            exit: Some(HostExit {
                commands: commands.clone(),
                result: exit_rx,
            }),
            commands,
            outputs: output_rx,
            terminal: Some(terminal_rx),
        })
    }

    /// Queue a command without blocking the UI thread.
    #[cfg(test)]
    pub(crate) fn try_send(&self, command: RuntimeCommand) -> Result<(), RuntimeCommand> {
        self.commands
            .try_send(command)
            .map_err(|error| error.into_inner())
    }

    /// Separate command submission from event observation without sharing the receiver.
    pub(crate) fn command_sender(&self) -> mpsc::Sender<RuntimeCommand> {
        self.commands.clone()
    }

    pub(crate) fn take_exit(&mut self) -> Option<HostExit> {
        self.exit.take()
    }

    /// Receive the next bounded presentation event.
    pub(crate) async fn recv(&mut self) -> Option<RuntimeOutput> {
        if let Some(output) = self.outputs.recv().await {
            return Some(output);
        }
        match self.terminal.take() {
            Some(terminal) => Some(terminal.await.unwrap_or_else(|_| {
                RuntimeOutput::Error("runtime host exited without a shutdown result".into())
            })),
            None => None,
        }
    }
}

async fn run_host(
    mut commands: mpsc::Receiver<RuntimeCommand>,
    outputs: mpsc::Sender<RuntimeOutput>,
    mut handle: RuntimeHandle,
    approval: Option<Arc<DesktopApprovalHandler>>,
) -> RuntimeOutput {
    let approval = approval.unwrap_or_else(|| {
        Arc::new(DesktopApprovalHandler::new(
            outputs.clone(),
            PathBuf::from("."),
        ))
    });
    // Continue draining authoritative events while the presentation is slow.
    // Overflow stops the runtime explicitly rather than silently dropping text.
    let mut pending = PendingOutput::default();
    loop {
        tokio::select! {
            _ = approval.output_overflow.notified() => return overflow_host(handle).await,
            _ = outputs.closed() => return stop_host(handle).await,
            permit = outputs.reserve(), if !pending.queue.is_empty() => {
                match permit {
                    Ok(permit) => {
                        if let Some(output) = pending.pop() {
                            permit.send(output);
                        }
                    }
                    Err(_) => {
                        return stop_host(handle).await;
                    }
                }
            }
            command = commands.recv() => {
                match command {
                    Some(RuntimeCommand::Submit(message)) => {
                        if let Err(error) = handle.submit(message).await
                            && pending.push(RuntimeOutput::Error(error.to_string())).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                    }
                    Some(RuntimeCommand::Interrupt) => {
                        if let Err(error) = handle.interrupt().await
                            && pending.push(RuntimeOutput::Error(error.to_string())).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                    }
                    Some(RuntimeCommand::Shutdown) | None => {
                        return stop_host(handle).await;
                    }
                    Some(RuntimeCommand::ApprovalResponse { request_id, choice }) => {
                        let _ = approval.resolve(request_id, choice).await;
                    }
                }
            }
            event = handle.next_event() => {
                let Some(event) = event else {
                    return stop_host(handle).await;
                };
                for output in project_event(event) {
                    if pending.push(output).is_err() {
                        return overflow_host(handle).await;
                    }
                }
            }
        }
    }
}

async fn stop_host(handle: RuntimeHandle) -> RuntimeOutput {
    match handle.shutdown().await {
        Ok(()) => RuntimeOutput::Stopped,
        Err(error) => RuntimeOutput::Error(format!("runtime shutdown failed: {error}")),
    }
}

async fn overflow_host(handle: RuntimeHandle) -> RuntimeOutput {
    let shutdown = stop_host(handle).await;
    let detail = match shutdown {
        RuntimeOutput::Error(error) => format!("; {error}"),
        _ => String::new(),
    };
    RuntimeOutput::Error(format!(
        "Presentation buffer exceeded; runtime stopped; displayed output is incomplete{detail}"
    ))
}

fn project_event(event: SessionEvent) -> Vec<RuntimeOutput> {
    match event {
        SessionEvent::SubmissionStarted { turn_id, .. } => {
            vec![RuntimeOutput::Started { turn_id }]
        }
        SessionEvent::TurnEvent { payload, .. } => match payload {
            talos_runtime::TurnEventPayload::Progress {
                event:
                    AgentEvent::ToolCall {
                        call, provenance, ..
                    },
            } => vec![
                RuntimeOutput::ToolStarted {
                    call_id: call.id.clone(),
                    name: call.name,
                },
                RuntimeOutput::ToolEvidence {
                    call_id: call.id,
                    provenance: format_tool_provenance(&provenance),
                },
            ],
            talos_runtime::TurnEventPayload::Progress {
                event: AgentEvent::ToolResult { result },
            } => vec![RuntimeOutput::ToolResult {
                call_id: result.tool_use_id,
                content: result.content,
                is_error: result.is_error,
            }],
            talos_runtime::TurnEventPayload::Progress {
                event: AgentEvent::TextDelta { delta },
            } => vec![RuntimeOutput::Text(delta)],
            talos_runtime::TurnEventPayload::Completed { status } => {
                let status = match status {
                    talos_runtime::TurnCompletionStatus::Success { .. } => TerminalStatus::Success,
                    talos_runtime::TurnCompletionStatus::Cancelled => TerminalStatus::Cancelled,
                    talos_runtime::TurnCompletionStatus::Error { message } => {
                        TerminalStatus::Error(message)
                    }
                };
                vec![RuntimeOutput::Completed { status }]
            }
            _ => Vec::new(),
        },
        SessionEvent::Error { message } => vec![RuntimeOutput::Error(message)],
        _ => Vec::new(),
    }
}

fn format_tool_provenance(provenance: &talos_runtime::ToolProvenance) -> String {
    match provenance {
        talos_runtime::ToolProvenance::Native => "native".into(),
        talos_runtime::ToolProvenance::McpRemote { server } => {
            format!("mcp:{server}")
        }
        talos_runtime::ToolProvenance::Plugin {
            name,
            version,
            carrier,
        } => format!("plugin:{name}@{version} ({carrier})"),
    }
}

#[cfg(test)]
mod auto_tests;

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod cancellation_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use talos_provider::mock::MockProvider;

    #[test]
    fn workspace_identity_is_stable_and_safe_for_durable_bindings() {
        let first = workspace_external_id(std::path::Path::new("/tmp/project/one"));
        let second = workspace_external_id(std::path::Path::new("/tmp/project/one"));
        assert_eq!(first, second);
        assert!(first.starts_with("desktop-workspace-"));
        assert!(!first.contains('/'));
        assert!(!first.contains('\\'));
        assert!(!first.contains(".."));
        let task = task_external_id(std::path::Path::new("/tmp/project/one"), "Review /tmp");
        assert!(task.starts_with("desktop-task-"));
        assert!(!task.contains('/'));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn configured_host_persists_transcript_to_workspace_session() {
        let workspace = TestWorkspace::new();
        let storage = workspace.0.join(".talos").join("desktop-sessions");
        let external_id = "desktop-test-session".to_owned();
        let provider = Arc::new(MockProvider::new().with_response("durable response"));
        let configured = provider.clone();
        let mut host = RuntimeHost::start_with(
            move || Ok((configured, 128_000, false)),
            workspace.0.clone(),
            Some((storage.clone(), external_id.clone())),
        )
        .expect("host starts");
        host.try_send(RuntimeCommand::Submit("persist this turn".into()))
            .expect("submit");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                match host.recv().await.expect("host event") {
                    RuntimeOutput::Completed { .. } => break,
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    _ => {}
                }
            }
        })
        .await
        .expect("turn completes");
        host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while !matches!(host.recv().await, Some(RuntimeOutput::Stopped) | None) {}
        })
        .await
        .expect("host stops");

        let session = talos_session::DurableSession::open_or_create(
            &workspace.0.join(".talos").join("desktop-sessions"),
            "desktop-test-session",
        )
        .expect("session opens");
        let transcript = session.transcript(None, 50).expect("transcript reads");
        assert!(
            transcript
                .iter()
                .any(|entry| entry.content == "persist this turn")
        );
        assert_eq!(
            talos_session::DurableSession::list_external_ids(
                &workspace.0.join(".talos").join("desktop-sessions")
            )
            .expect("session index reads"),
            vec!["desktop-test-session"]
        );
    }

    pub(super) struct TestWorkspace(pub(super) PathBuf);

    impl TestWorkspace {
        pub(super) fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "talos-desktop-permission-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).expect("unique test workspace");
            Self(path.canonicalize().expect("canonical workspace"))
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn denied_real_write_has_one_error_result_and_no_file() {
        real_write_choice(ApprovalChoice::Deny, false).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn approved_once_real_write_creates_exact_content() {
        real_write_choice(ApprovalChoice::ApproveOnce, true).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn approved_session_real_write_creates_exact_content() {
        real_write_choice(ApprovalChoice::AlwaysApprove, true).await;
    }

    async fn real_write_choice(choice: ApprovalChoice, allowed: bool) {
        let workspace = TestWorkspace::new();
        let mut provider = MockProvider::new()
            .with_tool_call(
                "write",
                serde_json::json!({"path": "denied.txt", "content": "must not exist"}),
            )
            .with_response("Denied.");
        if allowed {
            provider = provider
                .with_tool_call(
                    "write",
                    serde_json::json!({
                        "path": "denied.txt", "content": "must not overwrite"
                    }),
                )
                .with_response("Second request completed.");
        }
        let mut host = RuntimeHost::start(Arc::new(provider), workspace.0.clone()).expect("host");
        host.try_send(RuntimeCommand::Submit("write fixture".into()))
            .expect("submit");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let mut approvals = 0;
            let mut results = 0;
            let mut call = None;
            loop {
                match host.recv().await.expect("event") {
                    RuntimeOutput::ToolStarted { call_id, name } => {
                        assert_eq!(name, "write");
                        call = Some(call_id);
                    }
                    RuntimeOutput::ApprovalRequested {
                        request_id,
                        tool_name,
                        ..
                    } => {
                        assert_eq!(tool_name, "write");
                        approvals += 1;
                        host.try_send(RuntimeCommand::ApprovalResponse {
                            request_id,
                            choice: choice.clone(),
                        })
                        .expect("deny");
                    }
                    RuntimeOutput::ToolResult {
                        call_id, is_error, ..
                    } => {
                        assert_eq!(Some(call_id), call);
                        assert_eq!(is_error, !allowed || results > 0);
                        results += 1;
                    }
                    RuntimeOutput::Completed { .. } => {
                        if allowed && results == 1 {
                            host.try_send(RuntimeCommand::Submit("repeat the write".into()))
                                .expect("second turn");
                        } else {
                            break;
                        }
                    }
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    _ => {}
                }
            }
            let expected_approvals = if allowed && matches!(choice, ApprovalChoice::ApproveOnce) {
                2
            } else {
                1
            };
            assert_eq!(approvals, expected_approvals);
            assert_eq!(results, if allowed { 2 } else { 1 });
            if allowed {
                assert_eq!(
                    std::fs::read_to_string(workspace.0.join("denied.txt")).expect("approved file"),
                    "must not exist"
                );
            } else {
                assert!(!workspace.0.join("denied.txt").exists());
            }
            host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded denied write");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancelled_real_approval_rejects_late_allow_without_writing() {
        let workspace = TestWorkspace::new();
        let provider = MockProvider::new()
            .with_tool_call(
                "write",
                serde_json::json!({
                    "path": "cancelled.txt", "content": "must never execute"
                }),
            )
            .with_response("After cancellation.");
        let mut host = RuntimeHost::start(Arc::new(provider), workspace.0.clone()).expect("host");
        host.try_send(RuntimeCommand::Submit("request a write".into()))
            .expect("submit");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let request_id = loop {
                match host.recv().await.expect("event") {
                    RuntimeOutput::ApprovalRequested { request_id, .. } => break request_id,
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    RuntimeOutput::Completed { .. } => panic!("completed without approval"),
                    _ => {}
                }
            };
            host.try_send(RuntimeCommand::Interrupt).expect("interrupt");
            loop {
                if let RuntimeOutput::Completed { status, .. } =
                    host.recv().await.expect("cancel event")
                {
                    assert_eq!(status, TerminalStatus::Cancelled);
                    break;
                }
            }
            host.try_send(RuntimeCommand::ApprovalResponse {
                request_id,
                choice: ApprovalChoice::ApproveOnce,
            })
            .expect("late reply");
            host.try_send(RuntimeCommand::Submit("continue without tools".into()))
                .expect("next turn");
            loop {
                match host.recv().await.expect("next event") {
                    RuntimeOutput::Completed { status, .. } => {
                        assert_eq!(status, TerminalStatus::Success);
                        break;
                    }
                    RuntimeOutput::ToolStarted { .. } => panic!("cancelled tool replayed"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
            assert!(!workspace.0.join("cancelled.txt").exists());
        })
        .await
        .expect("bounded cancellation");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn full_observer_channel_signals_host_failure_without_blocking() {
        let (events, _received) = mpsc::channel(1);
        let handler = DesktopApprovalHandler::new(events, PathBuf::from("."));
        handler.publish(RuntimeOutput::Text("occupy channel".into()));
        handler.publish(RuntimeOutput::ApprovalClosed { request_id: 7 });
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            handler.output_overflow.notified(),
        )
        .await
        .expect("lost approval event must stop the host");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn disconnect_after_delivering_approval_releases_waiter() {
        let (events, mut received) = mpsc::channel(8);
        let handler = Arc::new(DesktopApprovalHandler::new(events, PathBuf::from(".")));
        let waiting = {
            let handler = handler.clone();
            tokio::spawn(async move { handler.request_ui("write", "scope", "").await })
        };
        let Some(RuntimeOutput::ApprovalRequested { request_id, .. }) = received.recv().await
        else {
            panic!("approval expected");
        };
        drop(received);
        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(1), waiting)
                .await
                .expect("disconnect must release waiter")
                .expect("approval task"),
            ApprovalChoice::Deny
        );
        assert!(
            !handler
                .resolve(request_id, ApprovalChoice::ApproveOnce)
                .await
        );
        assert!(handler.pending.lock().expect("pending lock").is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn approval_response_is_exactly_once_and_stale_ids_are_rejected() {
        let (events, mut received) = mpsc::channel(8);
        let handler = Arc::new(DesktopApprovalHandler::new(events, PathBuf::from(".")));
        let waiting = {
            let handler = handler.clone();
            tokio::spawn(async move { handler.request_ui("bash", "workspace", "reason").await })
        };
        let request_id = match received.recv().await.expect("approval event") {
            RuntimeOutput::ApprovalRequested { request_id, .. } => request_id,
            other => panic!("unexpected event: {other:?}"),
        };
        assert!(
            handler
                .resolve(request_id, ApprovalChoice::ApproveOnce)
                .await
        );
        assert!(
            !handler
                .resolve(request_id, ApprovalChoice::AlwaysApprove)
                .await
        );
        assert_eq!(
            waiting.await.expect("approval task"),
            ApprovalChoice::ApproveOnce
        );
        assert!(
            matches!(received.recv().await, Some(RuntimeOutput::ApprovalClosed { request_id: closed }) if closed == request_id)
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancelled_approval_cannot_authorize_a_later_request() {
        let (events, mut received) = mpsc::channel(8);
        let handler = Arc::new(DesktopApprovalHandler::new(events, PathBuf::from(".")));
        let task = {
            let handler = handler.clone();
            tokio::spawn(async move { handler.request_ui("write", "first", "").await })
        };
        let Some(RuntimeOutput::ApprovalRequested {
            request_id: old, ..
        }) = received.recv().await
        else {
            panic!("request expected")
        };
        task.abort();
        assert!(task.await.expect_err("cancelled").is_cancelled());
        assert!(handler.pending.lock().expect("pending lock").is_empty());
        assert!(
            matches!(received.recv().await, Some(RuntimeOutput::ApprovalClosed { request_id }) if request_id == old)
        );
        let next = {
            let handler = handler.clone();
            tokio::spawn(async move { handler.request_ui("write", "second", "").await })
        };
        let Some(RuntimeOutput::ApprovalRequested {
            request_id: current,
            ..
        }) = received.recv().await
        else {
            panic!("request expected")
        };
        assert_ne!(old, current);
        assert!(!handler.resolve(old, ApprovalChoice::AlwaysApprove).await);
        assert!(handler.resolve(current, ApprovalChoice::Deny).await);
        assert_eq!(next.await.expect("request completes"), ApprovalChoice::Deny);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn closed_approval_surface_fails_closed_and_releases_pending_request() {
        let (events, received) = mpsc::channel(1);
        drop(received);
        let handler = DesktopApprovalHandler::new(events, PathBuf::from("."));
        assert_eq!(
            handler.request_ui("write", "scope", "").await,
            ApprovalChoice::Deny
        );
        assert!(handler.pending.lock().expect("pending lock").is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_read_result_keeps_the_requested_call_identity() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let provider = MockProvider::new()
            .with_tool_call("read", serde_json::json!({"path": "Cargo.toml"}))
            .with_response("Read completed.");
        let mut host = RuntimeHost::start(Arc::new(provider), workspace).expect("host starts");
        host.try_send(RuntimeCommand::Submit("read the crate manifest".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let mut started = None;
            let mut results = 0;
            loop {
                match host.recv().await.expect("turn event") {
                    RuntimeOutput::ToolStarted { call_id, name } => {
                        assert_eq!(name, "read");
                        assert!(started.replace(call_id).is_none());
                    }
                    RuntimeOutput::ToolResult {
                        call_id,
                        content,
                        is_error,
                    } => {
                        assert_eq!(Some(&call_id), started.as_ref());
                        assert!(!is_error, "read failed: {content}");
                        assert!(content.contains("talos-desktop"));
                        results += 1;
                    }
                    RuntimeOutput::Completed { .. } => {
                        assert_eq!(results, 1);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("host failure: {error}"),
                    RuntimeOutput::ApprovalRequested { .. } => {
                        panic!("workspace read unexpectedly needs approval")
                    }
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded real read turn");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn requested_tool_is_projected_without_claiming_execution() {
        let provider = MockProvider::new()
            .with_tool_call("bash", Default::default())
            .with_response("No tool ran.");
        let mut host = RuntimeHost::start(Arc::new(provider), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("request a tool".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut unavailable = false;
            loop {
                match host.recv().await.expect("turn event") {
                    RuntimeOutput::ToolStarted { name, .. } => {
                        assert_eq!(name, "bash");
                        unavailable = true;
                    }
                    RuntimeOutput::Completed { .. } => {
                        assert!(unavailable);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected host error: {error}"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded tool-unavailable turn");
    }

    struct PausedProvider {
        entered: tokio::sync::Notify,
        disconnected: Arc<tokio::sync::Notify>,
    }

    struct PendingConnection {
        entered: tokio::sync::Notify,
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_during_history_compaction_reaches_cancelled_terminal() {
        let provider = Arc::new(PendingConnection {
            entered: tokio::sync::Notify::new(),
        });
        let mut handle = RuntimeBuilder::new()
            .provider(provider.clone())
            .model_context_limit(1024)
            .initial_history(
                (0..40)
                    .map(|index| talos_runtime::Message::User {
                        content: format!("turn {index}: {}", "previous history ".repeat(128)),
                    })
                    .collect(),
            )
            .build()
            .expect("runtime starts");
        handle.submit("next").await.expect("submission accepted");
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            provider.entered.notified(),
        )
        .await
        .expect("compaction provider reached");
        handle.interrupt().await.expect("interrupt accepted");
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                let event = handle.next_event().await.expect("terminal event");
                if let Some(RuntimeOutput::Completed { status }) = project_event(event)
                    .into_iter()
                    .find(|output| matches!(output, RuntimeOutput::Completed { .. }))
                {
                    return status;
                }
            }
        })
        .await;
        handle.shutdown().await.expect("cleanup");
        assert_eq!(
            result.expect("compaction cancellation must not wait for provider"),
            TerminalStatus::Cancelled
        );
    }

    #[async_trait::async_trait]
    impl LanguageModel for PendingConnection {
        async fn stream(
            &self,
            _: &[talos_runtime::Message],
        ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<AgentEvent>> {
            self.entered.notify_one();
            std::future::pending().await
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_cancels_before_provider_connection_returns() {
        let provider = Arc::new(PendingConnection {
            entered: tokio::sync::Notify::new(),
        });
        let mut host = RuntimeHost::start(provider.clone(), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            provider.entered.notified().await;
            host.try_send(RuntimeCommand::Interrupt)
                .expect("interrupt queues");
            loop {
                match host.recv().await.expect("cancel terminal") {
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Cancelled);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected error: {error}"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded connection cancellation");
    }

    #[async_trait::async_trait]
    impl LanguageModel for PausedProvider {
        async fn stream(
            &self,
            _: &[talos_runtime::Message],
        ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<AgentEvent>> {
            let (sender, receiver) = mpsc::channel(1);
            let disconnected = self.disconnected.clone();
            tokio::spawn(async move {
                sender.closed().await;
                disconnected.notify_one();
            });
            self.entered.notify_one();
            Ok(receiver)
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_cancels_a_paused_provider_and_closes_its_stream() {
        let provider = Arc::new(PausedProvider {
            entered: tokio::sync::Notify::new(),
            disconnected: Arc::new(tokio::sync::Notify::new()),
        });
        let mut host = RuntimeHost::start(provider.clone(), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            provider.entered.notified().await;
            host.try_send(RuntimeCommand::Interrupt)
                .expect("interrupt queues");
            loop {
                match host.recv().await.expect("cancel terminal") {
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Cancelled);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected error: {error}"),
                    _ => {}
                }
            }
            provider.disconnected.notified().await;
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded cancellation");
    }

    #[test]
    fn application_exit_observes_cleanup_after_view_disposal() {
        let mut host = RuntimeHost::start(Arc::new(MockProvider::new()), ".").expect("host starts");
        let exit = host.take_exit().expect("application receipt");
        drop(host);
        exit.finish(std::time::Duration::from_secs(5))
            .expect("confirmed shutdown");
    }

    #[test]
    fn application_exit_timeout_is_reported_without_claiming_success() {
        let (commands, _commands_rx) = mpsc::channel(1);
        let (_result_tx, result) = std::sync::mpsc::channel();
        let exit = HostExit { commands, result };
        assert!(
            exit.finish(std::time::Duration::ZERO)
                .expect_err("no completion")
                .contains("not confirmed")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn oversized_output_stops_with_an_explicit_incomplete_output_error() {
        let mut host = RuntimeHost::start(
            Arc::new(MockProvider::new().with_response("x".repeat(MAX_PENDING_BYTES + 1))),
            ".",
        )
        .expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match host.recv().await.expect("overflow must be reported") {
                    RuntimeOutput::Error(message) => {
                        assert!(message.contains("displayed output is incomplete"));
                        break;
                    }
                    RuntimeOutput::Completed { .. } => panic!("overflow must not report success"),
                    _ => {}
                }
            }
            assert_eq!(host.recv().await, None);
        })
        .await
        .expect("bounded overflow shutdown");
    }

    #[test]
    fn pending_output_enforces_bytes_and_count_without_overwriting() {
        let mut pending = PendingOutput::default();
        assert!(
            pending
                .push(RuntimeOutput::Text("x".repeat(MAX_PENDING_BYTES)))
                .is_ok()
        );
        assert!(
            pending
                .push(RuntimeOutput::Text("overflow".into()))
                .is_err()
        );
        assert!(
            matches!(pending.pop(), Some(RuntimeOutput::Text(text)) if text.len() == MAX_PENDING_BYTES)
        );
        assert_eq!(pending.bytes, 0);
        for _ in 0..EVENT_CAPACITY {
            pending
                .push(RuntimeOutput::Stopped)
                .expect("within event limit");
        }
        assert!(pending.push(RuntimeOutput::Stopped).is_err());
        assert_eq!(pending.queue.len(), EVENT_CAPACITY);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_finishes_with_full_undrained_presentation_queue() {
        let handle = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .build()
            .expect("runtime builds");
        let (commands, command_rx) = mpsc::channel(1);
        let (outputs, mut output_rx) = mpsc::channel(1);
        outputs
            .try_send(RuntimeOutput::Text("queued".into()))
            .expect("queue filled");
        commands
            .try_send(RuntimeCommand::Shutdown)
            .expect("shutdown queued");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_host(command_rx, outputs, handle, None),
        )
        .await
        .expect("shutdown must not await presentation");
        assert_eq!(result, RuntimeOutput::Stopped);
        assert_eq!(
            output_rx.try_recv(),
            Ok(RuntimeOutput::Text("queued".into()))
        );
        assert!(output_rx.is_closed());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn receiver_disconnection_is_detected_without_pending_output() {
        let handle = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .build()
            .expect("runtime builds");
        let (_commands, command_rx) = mpsc::channel(1);
        let (outputs, output_rx) = mpsc::channel(1);
        drop(output_rx);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_host(command_rx, outputs, handle, None),
        )
        .await
        .expect("receiver closure must be observed");
        assert_eq!(result, RuntimeOutput::Stopped);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn provider_failure_is_not_projected_as_success() {
        let mut host = RuntimeHost::start(Arc::new(MockProvider::new().with_error(401)), ".")
            .expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match host.recv().await.expect("terminal event") {
                    RuntimeOutput::Completed {
                        status: TerminalStatus::Error(message),
                    } => {
                        assert!(message.contains("authentication failed"));
                        break;
                    }
                    RuntimeOutput::Completed { status } => {
                        panic!("unexpected terminal: {status:?}")
                    }
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded provider failure");
    }

    #[test]
    fn full_command_queue_returns_the_unsent_message() {
        let (commands, _receiver) = mpsc::channel(1);
        let (_sender, outputs) = mpsc::channel(1);
        let host = RuntimeHost {
            commands,
            outputs,
            terminal: None,
            exit: None,
        };
        host.try_send(RuntimeCommand::Submit("first".into()))
            .expect("first queues");
        assert!(
            matches!(host.try_send(RuntimeCommand::Submit("second".into())),
            Err(RuntimeCommand::Submit(message)) if message == "second")
        );
    }

    #[test]
    fn host_starts_without_an_ambient_tokio_runtime() {
        let provider = Arc::new(MockProvider::new().with_response("hello"));
        let mut host = RuntimeHost::start(provider, ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        let executor = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test executor");
        executor.block_on(async {
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    match host.recv().await.expect("host event") {
                        RuntimeOutput::Completed { .. } => break,
                        RuntimeOutput::Error(error) => panic!("host failed: {error}"),
                        _ => {}
                    }
                }
                host.try_send(RuntimeCommand::Shutdown)
                    .expect("shutdown queues");
                assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
            })
            .await
            .expect("bounded host completion");
        });
    }

    #[tokio::test(flavor = "current_thread")]
    async fn host_forwards_mock_text_and_terminal_event() {
        let provider = Arc::new(MockProvider::new().with_response("hello"));
        let mut host = RuntimeHost::start(provider, ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("say hello".into()))
            .expect("submit queues");
        let mut text = String::new();
        let mut completed = false;
        for _ in 0..16 {
            match host.recv().await.expect("output") {
                RuntimeOutput::Text(value) => text.push_str(&value),
                RuntimeOutput::Completed { .. } => {
                    completed = true;
                    break;
                }
                RuntimeOutput::Error(error) => panic!("unexpected runtime error: {error}"),
                _ => {}
            }
        }
        assert_eq!(text, "hello");
        assert!(completed);
        host.try_send(RuntimeCommand::Shutdown)
            .expect("shutdown queues");
    }
}
