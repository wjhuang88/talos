//! Thin Desktop adapter for the existing Talos Runtime.
//!
//! The adapter owns one Tokio runtime task and forwards bounded commands and
//! authoritative session events to the presentation layer. It deliberately
//! does not create tools, permissions, storage, or a second execution engine.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
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
        RuntimeOutput::ToolRequestContext {
            call_id,
            provenance,
            requested_path,
        } => call_id.len() + provenance.len() + requested_path.as_deref().unwrap_or_default().len(),
        RuntimeOutput::HistoryRestored { entries } => entries.iter().map(String::len).sum(),
        RuntimeOutput::WorkProjection { state } => match state {
            WorkProjectionState::Available { items, .. } => {
                items.iter().map(|item| item.title.len()).sum()
            }
            WorkProjectionState::Loading
            | WorkProjectionState::Unavailable
            | WorkProjectionState::ReadError => 0,
        },
        RuntimeOutput::ArtifactChanged {
            session_id,
            call_id,
            operation,
            path,
            ..
        } => session_id.to_string().len() + call_id.len() + operation.len() + path.len(),
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
        RuntimeOutput::EvaluationResult { state, .. } => state.len(),
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
    /// Explicitly request evaluation of the current task.
    Evaluate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkUnitStatus {
    Todo,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkItemProjection {
    pub title: String,
    pub title_truncated: bool,
    pub status: WorkUnitStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum WorkProjectionState {
    #[default]
    Loading,
    Unavailable,
    Available {
        items: Vec<WorkItemProjection>,
        truncated: bool,
    },
    ReadError,
}

/// Presentation-safe projection of an authoritative Runtime event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeOutput {
    /// A tool call was requested; this does not imply successful execution.
    ToolStarted { call_id: String, name: String },
    /// Metadata from the tool request; requested paths are not proof of a completed change.
    ToolRequestContext {
        call_id: String,
        provenance: String,
        requested_path: Option<String>,
    },
    /// Read-only history restored at host startup; does not represent a new turn or tool run.
    HistoryRestored { entries: Vec<String> },
    /// Canonical WorkUnit projection for the exact durable session.
    WorkProjection { state: WorkProjectionState },
    /// A bounded workspace file differed between snapshots around a successful built-in tool call.
    ArtifactChanged {
        session_id: uuid::Uuid,
        turn_id: u64,
        call_id: String,
        operation: String,
        path: String,
    },
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
    /// Evaluation could not start because the shared evidence source is unavailable.
    EvaluationUnavailable { reason: String },
    /// Evaluation projection returned by the shared evaluator and Delivery gate.
    EvaluationStarted,
    EvaluationResult {
        state: String,
        delivery: talos_core::work::DeliveryEligibility,
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

const MAX_ARTIFACT_EVIDENCE_BYTES: u64 = 1024 * 1024;
const MAX_PENDING_ARTIFACT_CALLS: usize = 128;
const MAX_ARTIFACT_PATH_BYTES: usize = 16 * 1024;
const MAX_ARTIFACT_CALL_ID_BYTES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileRevision {
    Missing,
    Present { bytes: u64, digest: [u8; 32] },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSnapshot {
    relative_path: String,
    revision: FileRevision,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct ArtifactCallKey {
    turn_id: u64,
    call_id: String,
}

struct PendingArtifactSnapshot {
    operation: String,
    snapshot: FileSnapshot,
}

struct EvaluationHarness {
    service: talos_runtime::RuntimeEvaluationService,
    claim: talos_core::evaluation::CompletionClaim,
    current_subject: talos_core::evaluation::EvaluationSubject,
    evidence: Vec<talos_runtime::ValidationEvidence>,
    mission: talos_core::work::WorkIdentity,
    required_goals: Vec<talos_core::work::WorkIdentity>,
    mission_evaluation: Option<talos_core::work::MissionEvaluation>,
}

struct DesktopArtifactCaptureHook {
    session_id: uuid::Uuid,
    events: Arc<DesktopApprovalHandler>,
    pending: Mutex<HashMap<ArtifactCallKey, PendingArtifactSnapshot>>,
    proposed: Mutex<HashMap<ArtifactCallKey, talos_runtime::ToolCall>>,
    capture_root: Option<Arc<ArtifactCaptureRoot>>,
}

struct ArtifactCaptureRoot {
    path: PathBuf,
    directory: cap_std::fs::Dir,
    slots: Arc<tokio::sync::Semaphore>,
}

impl ArtifactCaptureRoot {
    fn open(path: &Path) -> Option<Self> {
        let path = std::fs::canonicalize(path).ok()?;
        let directory =
            cap_std::fs::Dir::open_ambient_dir(&path, cap_std::ambient_authority()).ok()?;
        Some(Self {
            path,
            directory,
            slots: Arc::new(tokio::sync::Semaphore::new(1)),
        })
    }
}

impl DesktopArtifactCaptureHook {
    #[cfg(test)]
    fn new(session_id: uuid::Uuid, events: Arc<DesktopApprovalHandler>) -> Self {
        Self::with_capture_slots(session_id, events, Arc::new(tokio::sync::Semaphore::new(1)))
    }

    fn with_capture_slots(
        session_id: uuid::Uuid,
        events: Arc<DesktopApprovalHandler>,
        slots: Arc<tokio::sync::Semaphore>,
    ) -> Self {
        let capture_root = ArtifactCaptureRoot::open(&events.workspace_root).map(|mut root| {
            root.slots = slots;
            Arc::new(root)
        });
        Self {
            session_id,
            events,
            pending: Mutex::new(HashMap::new()),
            proposed: Mutex::new(HashMap::new()),
            capture_root,
        }
    }

    async fn observe_before(&self, turn_id: u64, call: &talos_runtime::ToolCall) {
        if !matches!(call.name.as_str(), "write" | "edit" | "delete") {
            return;
        }
        let Some(requested_path) = call.input.get("path").and_then(serde_json::Value::as_str)
        else {
            return;
        };
        let Some(snapshot) =
            capture_workspace_file(self.capture_root.clone(), requested_path.to_owned()).await
        else {
            return;
        };
        let key = ArtifactCallKey {
            turn_id,
            call_id: call.id.clone(),
        };
        if let Ok(mut pending) = self.pending.lock()
            && pending.len() < MAX_PENDING_ARTIFACT_CALLS
        {
            pending.entry(key).or_insert(PendingArtifactSnapshot {
                operation: call.name.clone(),
                snapshot,
            });
        }
    }

    async fn observe_after(
        &self,
        turn_id: u64,
        call: &talos_runtime::ToolCall,
        result: &talos_runtime::ToolResult,
    ) {
        let key = ArtifactCallKey {
            turn_id,
            call_id: call.id.clone(),
        };
        let pending = self
            .pending
            .lock()
            .ok()
            .and_then(|mut pending| pending.remove(&key));
        let Some(pending) = pending else {
            return;
        };
        if result.is_error || pending.operation != call.name {
            return;
        }
        let Some(requested_path) = call.input.get("path").and_then(serde_json::Value::as_str)
        else {
            return;
        };
        if requested_path != pending.snapshot.relative_path {
            let Some(root) = &self.capture_root else {
                return;
            };
            let Some(before_path) = requested_path_to_relative(&root.path, requested_path) else {
                return;
            };
            if before_path != pending.snapshot.relative_path {
                return;
            }
        }
        let Some(current) =
            capture_workspace_file(self.capture_root.clone(), requested_path.to_owned()).await
        else {
            return;
        };
        if current.relative_path != pending.snapshot.relative_path {
            return;
        }
        if pending.snapshot == current {
            return;
        }
        self.events.publish(RuntimeOutput::ArtifactChanged {
            session_id: self.session_id,
            turn_id,
            call_id: call.id.clone(),
            operation: pending.operation,
            path: current.relative_path,
        });
    }
}

#[async_trait]
impl talos_plugin::HookHandler for DesktopArtifactCaptureHook {
    fn name(&self) -> &str {
        "desktop-artifact-capture"
    }

    fn subscribed(&self) -> &'static [talos_plugin::HookEventKind] {
        &[
            talos_plugin::HookEventKind::BeforeToolCall,
            talos_plugin::HookEventKind::AfterPermissionCheck,
            talos_plugin::HookEventKind::AfterToolCall,
            talos_plugin::HookEventKind::TurnComplete,
        ]
    }

    async fn on_event(
        &self,
        context: &talos_plugin::HookContext,
        event: &mut talos_plugin::HookEvent<'_>,
    ) -> talos_plugin::HookResult {
        match event {
            talos_plugin::HookEvent::BeforeToolCall { call } => {
                if matches!(call.name.as_str(), "write" | "edit" | "delete")
                    && call.id.len() <= MAX_ARTIFACT_CALL_ID_BYTES
                    && let Some(path) = call.input.get("path").and_then(serde_json::Value::as_str)
                    && path.len() <= MAX_ARTIFACT_PATH_BYTES
                    && let Ok(mut proposed) = self.proposed.lock()
                    && proposed.len() < MAX_PENDING_ARTIFACT_CALLS
                {
                    proposed.insert(
                        ArtifactCallKey {
                            turn_id: context.turn_id.0,
                            call_id: call.id.clone(),
                        },
                        talos_runtime::ToolCall {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: serde_json::json!({"path": path}),
                        },
                    );
                }
            }
            talos_plugin::HookEvent::AfterPermissionCheck { call, decision } => {
                let proposed = self.proposed.lock().ok().and_then(|mut proposed| {
                    proposed.remove(&ArtifactCallKey {
                        turn_id: context.turn_id.0,
                        call_id: call.id.clone(),
                    })
                });
                if matches!(decision, talos_runtime::PermissionDecision::Allow)
                    && let Some(proposed) = proposed
                    && proposed.name == call.name
                {
                    self.observe_before(context.turn_id.0, &proposed).await;
                }
            }
            talos_plugin::HookEvent::AfterToolCall { call, result } => {
                self.observe_after(context.turn_id.0, call, result).await;
            }
            talos_plugin::HookEvent::TurnComplete { .. } => {
                if let Ok(mut proposed) = self.proposed.lock() {
                    proposed.retain(|key, _| key.turn_id != context.turn_id.0);
                }
                if let Ok(mut pending) = self.pending.lock() {
                    pending.retain(|key, _| key.turn_id != context.turn_id.0);
                }
            }
            _ => {}
        }
        talos_plugin::HookResult::Continue
    }
}

async fn capture_workspace_file(
    root: Option<Arc<ArtifactCaptureRoot>>,
    requested_path: String,
) -> Option<FileSnapshot> {
    let root = root?;
    let permit = root.slots.clone().try_acquire_owned().ok()?;
    let read = tokio::task::spawn_blocking(move || {
        // Retain capacity until the OS call returns, even when the caller times out.
        let _permit = permit;
        capture_file_from_root(&root, &requested_path)
    });
    tokio::time::timeout(std::time::Duration::from_millis(500), read)
        .await
        .ok()?
        .ok()?
}

#[cfg(test)]
fn capture_workspace_file_sync(
    workspace_root: &Path,
    requested_path: &str,
) -> Option<FileSnapshot> {
    capture_file_from_root(&ArtifactCaptureRoot::open(workspace_root)?, requested_path)
}

fn capture_file_from_root(
    root: &ArtifactCaptureRoot,
    requested_path: &str,
) -> Option<FileSnapshot> {
    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
    use sha2::Digest;
    use std::path::Component;

    let requested = Path::new(requested_path);
    let relative = if requested.is_absolute() {
        requested.strip_prefix(&root.path).ok()?.to_path_buf()
    } else {
        requested.to_path_buf()
    };
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if normalized.as_os_str().is_empty() {
        return None;
    }

    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut current = PathBuf::new();
    for component in normalized.parent()?.components() {
        let Component::Normal(part) = component else {
            return None;
        };
        current.push(part);
        match root.directory.symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => return None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) => return None,
        }
    }
    let relative_path = match root.directory.open_with(&normalized, &options) {
        Ok(mut file) => {
            // Check the opened object, not a path that can change before open.
            let metadata = file.metadata().ok()?;
            if !metadata.is_file() || metadata.len() > MAX_ARTIFACT_EVIDENCE_BYTES {
                return None;
            }
            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            file.by_ref()
                .take(MAX_ARTIFACT_EVIDENCE_BYTES + 1)
                .read_to_end(&mut bytes)
                .ok()?;
            if bytes.len() as u64 > MAX_ARTIFACT_EVIDENCE_BYTES {
                return None;
            }
            return Some(FileSnapshot {
                relative_path: normalized.to_str()?.to_owned(),
                revision: FileRevision::Present {
                    bytes: bytes.len() as u64,
                    digest: sha2::Sha256::digest(bytes).into(),
                },
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            normalized.to_str()?.to_owned()
        }
        Err(_) => return None,
    };
    Some(FileSnapshot {
        relative_path,
        revision: FileRevision::Missing,
    })
}

fn requested_path_to_relative(workspace_root: &Path, requested_path: &str) -> Option<String> {
    let requested = Path::new(requested_path);
    let relative = if requested.is_absolute() {
        requested.strip_prefix(workspace_root).ok()?.to_path_buf()
    } else {
        requested.to_path_buf()
    };
    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => return None,
        }
    }
    (!normalized.as_os_str().is_empty())
        .then(|| normalized.to_str().map(str::to_owned))
        .flatten()
}

static NEXT_APPROVAL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
enum DurableOpenMode {
    Create,
    Existing,
}

struct DurableBinding {
    root: PathBuf,
    external_id: String,
    mode: DurableOpenMode,
}

fn identity_digest(domain: &[u8], workspace_root: &std::path::Path, value: &[u8]) -> String {
    use sha2::Digest;

    let mut digest = sha2::Sha256::new();
    digest.update(domain);
    digest.update([0]);
    digest.update(workspace_root.as_os_str().as_encoded_bytes());
    digest.update([0]);
    digest.update(value);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn workspace_external_id(workspace_root: &std::path::Path) -> String {
    format!(
        "desktop-workspace-v1-{}",
        identity_digest(b"talos-desktop-workspace-v1", workspace_root, b"")
    )
}

fn legacy_workspace_external_id(workspace_root: &std::path::Path) -> String {
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
}

#[cfg(test)]
pub(crate) fn task_external_id(workspace_root: &std::path::Path, goal: &str) -> String {
    format!(
        "desktop-task-v1-{}-{}",
        identity_digest(b"talos-desktop-task-workspace-v1", workspace_root, b""),
        identity_digest(
            b"talos-desktop-task-goal-v1",
            workspace_root,
            goal.as_bytes()
        )
    )
}

/// Create a fresh, opaque identity for one explicitly created task.
///
/// The goal is deliberately not part of the identity: two tasks with identical text are still
/// separate conversations. Existing v1 identities remain readable for explicit resume.
pub(crate) fn new_task_external_id(workspace_root: &std::path::Path) -> String {
    format!(
        "desktop-task-v2-{}-{}",
        identity_digest(b"talos-desktop-task-workspace-v1", workspace_root, b""),
        uuid::Uuid::new_v4()
    )
}

/// List existing durable task identities owned by one workspace.
///
/// This is read-only: it never creates a binding and never returns identities from another
/// workspace. The returned values are opaque keys suitable for an explicit resume action.
pub(crate) fn list_task_external_ids(
    workspace_root: &std::path::Path,
) -> Result<Vec<String>, String> {
    let root = workspace_root.join(".talos").join("desktop-sessions");
    let current_prefix = format!(
        "desktop-task-v2-{}-",
        identity_digest(b"talos-desktop-task-workspace-v1", workspace_root, b"")
    );
    let legacy_v1_prefix = format!(
        "desktop-task-v1-{}-",
        identity_digest(b"talos-desktop-task-workspace-v1", workspace_root, b"")
    );
    let legacy_prefix = format!(
        "desktop-task-desktop-workspace-{}-",
        legacy_workspace_external_id(workspace_root)
    );
    talos_session::DurableSession::list_external_ids(&root)
        .map(|ids| {
            ids.into_iter()
                .filter(|id| {
                    id.starts_with(&current_prefix)
                        || id.starts_with(&legacy_v1_prefix)
                        || id.starts_with(&legacy_prefix)
                })
                .collect()
        })
        .map_err(|error| error.to_string())
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

fn finish_host_executor(
    executor: tokio::runtime::Runtime,
    terminal: RuntimeOutput,
    capture_slots: &tokio::sync::Semaphore,
) -> RuntimeOutput {
    // Blocking OS reads cannot be forcibly cancelled. Bound teardown and expose
    // incomplete cleanup rather than falsely returning a successful stop receipt.
    executor.shutdown_timeout(std::time::Duration::from_millis(100));
    if capture_slots.available_permits() == 0 {
        RuntimeOutput::Error(
            "artifact read cleanup incomplete; a read-only filesystem operation is still running"
                .into(),
        )
    } else {
        terminal
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
        Self::configured_with_identity(workspace_root.into(), Some(external_id.into()), false)
    }

    /// Load configuration and open only an existing durable session identity.
    pub(crate) fn configured_for_existing_session(
        workspace_root: impl Into<PathBuf>,
        external_id: impl Into<String>,
    ) -> Result<Self, String> {
        Self::configured_with_identity(workspace_root.into(), Some(external_id.into()), true)
    }

    fn configured_with_identity(
        workspace_root: PathBuf,
        external_id: Option<String>,
        existing_only: bool,
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
            Some(DurableBinding {
                root: session_root,
                external_id,
                mode: if existing_only {
                    DurableOpenMode::Existing
                } else {
                    DurableOpenMode::Create
                },
            }),
        )
    }

    fn start_with(
        provider: impl FnOnce() -> Result<(Arc<dyn LanguageModel>, u32, bool), String> + Send + 'static,
        workspace_root: impl Into<PathBuf>,
        durable_identity: Option<DurableBinding>,
    ) -> Result<Self, String> {
        Self::start_with_evaluation(provider, workspace_root, durable_identity, None)
    }

    fn start_with_evaluation(
        provider: impl FnOnce() -> Result<(Arc<dyn LanguageModel>, u32, bool), String> + Send + 'static,
        workspace_root: impl Into<PathBuf>,
        durable_identity: Option<DurableBinding>,
        evaluation_harness: Option<EvaluationHarness>,
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
                let capture_slots = Arc::new(tokio::sync::Semaphore::new(1));
                let hook_slots = capture_slots.clone();
                let terminal = match executor {
                    Ok(executor) => {
                        let terminal = executor.block_on(async move {
                            let (provider, context_limit, auto_enabled) = match provider() {
                                Ok(provider) => provider,
                                Err(error) => {
                                    return RuntimeOutput::Error(error);
                                }
                            };
                            let approval_root = workspace_root.clone();
                            let approval = Arc::new(DesktopApprovalHandler::new(
                                outputs.clone(),
                                approval_root,
                            ));
                            let report_output = approval.clone();
                            let durable_session = durable_identity
                                .as_ref()
                                .map(|binding| {
                                    let opened = match binding.mode {
                                        DurableOpenMode::Create => {
                                            talos_session::DurableSession::open_or_create(
                                                &binding.root,
                                                &binding.external_id,
                                            )
                                            .map(Some)
                                        }
                                        DurableOpenMode::Existing => {
                                            talos_session::DurableSession::open_existing(
                                                &binding.root,
                                                &binding.external_id,
                                            )
                                        }
                                    };
                                    opened
                                        .map_err(|error| error.to_string())?
                                        .ok_or_else(|| "durable task no longer exists".to_owned())
                                })
                                .transpose();
                            let durable_session = match durable_session {
                                Ok(session) => session,
                                Err(error) => return RuntimeOutput::Error(error),
                            };
                            if let Some(session) = &durable_session {
                                match session.transcript(None, 200) {
                                    Ok(entries) if !entries.is_empty() => {
                                        let restored = entries
                                            .into_iter()
                                            .map(|entry| {
                                                format!("{}: {}", entry.role, entry.content)
                                            })
                                            .collect::<Vec<_>>();
                                        if outputs
                                            .send(RuntimeOutput::HistoryRestored {
                                                entries: restored,
                                            })
                                            .await
                                            .is_err()
                                        {
                                            return RuntimeOutput::Error(
                                                "Desktop history surface closed during restore"
                                                    .into(),
                                            );
                                        }
                                    }
                                    Ok(_) => {}
                                    Err(error) => {
                                        return RuntimeOutput::Error(format!(
                                            "durable transcript restore failed: {error}"
                                        ));
                                    }
                                }
                            }
                            let work_source = durable_session.as_ref().and_then(|session| {
                                talos_runtime::SessionManager::default_sessions_dir()
                                    .ok()
                                    .map(|sessions_dir| (sessions_dir, session.id()))
                            });
                            let mut builder = RuntimeBuilder::new()
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
                            if let Some((sessions_dir, session_id)) = &work_source {
                                for contribution in
                                    talos_session::todo_tool_contributions_for_sessions_dir(
                                        sessions_dir,
                                        *session_id,
                                    )
                                {
                                    builder = builder.tool(contribution.tool().clone());
                                }
                            }
                            if let Some(session) = &durable_session {
                                let mut hooks = talos_runtime::RuntimeHookRegistry::new();
                                hooks.register(Arc::new(
                                    DesktopArtifactCaptureHook::with_capture_slots(
                                        session.id(),
                                        approval.clone(),
                                        hook_slots,
                                    ),
                                ));
                                builder = builder.hook_registry(Arc::new(hooks));
                            }
                            let builder = match durable_session {
                                Some(session) => builder.durable_session(session),
                                None => builder,
                            };
                            match builder.build() {
                                Ok(handle) => {
                                    run_host(
                                        command_rx,
                                        outputs,
                                        handle,
                                        Some(approval),
                                        work_source,
                                        evaluation_harness,
                                    )
                                    .await
                                }
                                Err(error) => RuntimeOutput::Error(error.to_string()),
                            }
                        });
                        finish_host_executor(executor, terminal, &capture_slots)
                    }
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
    work_source: Option<(PathBuf, uuid::Uuid)>,
    evaluation_harness: Option<EvaluationHarness>,
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
    // Poll the evaluator alongside commands; never detach it from the host lifetime.
    let mut evaluation: Option<
        std::pin::Pin<Box<dyn std::future::Future<Output = RuntimeOutput> + Send + '_>>,
    > = None;
    let initial_work = load_work_projection(work_source.clone()).await;
    if pending
        .push(RuntimeOutput::WorkProjection {
            state: work_projection_state(initial_work),
        })
        .is_err()
    {
        return overflow_host(handle).await;
    }
    loop {
        tokio::select! {
            result = async {
                match evaluation.as_mut() {
                    Some(task) => task.await,
                    None => std::future::pending().await,
                }
            } => {
                evaluation = None;
                if pending.push(result).is_err() {
                    return overflow_host(handle).await;
                }
            }
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
                        if evaluation.take().is_some()
                            && pending.push(RuntimeOutput::EvaluationUnavailable {
                                reason: "evaluation cancelled".into(),
                            }).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                        if let Err(error) = handle.interrupt().await
                            && pending.push(RuntimeOutput::Error(error.to_string())).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                    }
                    Some(RuntimeCommand::Shutdown) | None => {
                        drop(evaluation.take());
                        return stop_host(handle).await;
                    }
                    Some(RuntimeCommand::ApprovalResponse { request_id, choice }) => {
                        let _ = approval.resolve(request_id, choice).await;
                    }
                    Some(RuntimeCommand::Evaluate) => {
                        if evaluation.is_some() {
                            continue;
                        }
                        if evaluation_harness.is_none() {
                            if pending.push(RuntimeOutput::EvaluationUnavailable {
                                reason: "no authoritative claim, acceptance criteria, or evidence source is connected".into(),
                            }).is_err() {
                                return overflow_host(handle).await;
                            }
                            continue;
                        }
                        if pending.push(RuntimeOutput::EvaluationStarted).is_err() {
                            return overflow_host(handle).await;
                        }
                        let harness = evaluation_harness.as_ref();
                        evaluation = Some(Box::pin(async move {
                        match harness {
                            Some(harness) => {
                                let mut result = harness
                                    .service
                                    .evaluate(&harness.claim, harness.evidence.clone())
                                    .await;
                                if let Err(error) = talos_runtime::RuntimeEvaluationService::observe_current_subject(
                                    &mut result,
                                    harness.current_subject,
                                ) {
                                    result = talos_runtime::EvaluatorOutcome::Failure(talos_runtime::EvaluatorFailure {
                                        evaluator: "runtime-subject-validation".into(),
                                        reason: error.to_string(),
                                    });
                                }
                                let (state, evaluations): (String, Vec<_>) = match result {
                                    talos_runtime::EvaluatorOutcome::Report { evaluation } => {
                                        (format!("{:?}", evaluation.state), vec![*evaluation])
                                    }
                                    talos_runtime::EvaluatorOutcome::Failure(failure) => {
                                        (format!("Failure: {}", failure.reason), Vec::new())
                                    }
                                };
                                let gate = talos_runtime::RuntimeEvaluationService::delivery_gate(
                                    harness.mission,
                                    &harness.required_goals,
                                    &evaluations,
                                    harness.mission_evaluation,
                                );
                                RuntimeOutput::EvaluationResult {
                                    state,
                                    delivery: gate.delivery,
                                }
                            }
                            None => RuntimeOutput::EvaluationUnavailable {
                                reason: "no authoritative claim, acceptance criteria, or evidence source is connected".into(),
                            },
                        }
                        }));
                    }
                }
            }
            event = handle.next_event() => {
                let Some(event) = event else {
                    return stop_host(handle).await;
                };
                let projected = project_event(event);
                let refresh_work = projected
                    .iter()
                    .any(|output| matches!(output, RuntimeOutput::ToolResult { .. }));
                for output in projected {
                    if pending.push(output).is_err() {
                        return overflow_host(handle).await;
                    }
                }
                if refresh_work {
                    let work = load_work_projection(work_source.clone()).await;
                    if pending
                        .push(RuntimeOutput::WorkProjection {
                            state: work_projection_state(work),
                        })
                        .is_err()
                    {
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

async fn load_work_projection(
    source: Option<(PathBuf, uuid::Uuid)>,
) -> Result<Option<(Vec<WorkItemProjection>, bool)>, WorkProjectionFailure> {
    let Some((sessions_dir, session_id)) = source else {
        return Ok(None);
    };
    let graph = tokio::task::spawn_blocking(move || {
        talos_runtime::SessionManager::with_dir(sessions_dir).load_work_graph_read_only(session_id)
    })
    .await
    .map_err(|_| WorkProjectionFailure::Worker)?
    .map_err(|_| WorkProjectionFailure::Storage)?;
    Ok(graph.map(project_work_graph))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkProjectionFailure {
    Worker,
    Storage,
}

fn work_projection_state(
    result: Result<Option<(Vec<WorkItemProjection>, bool)>, WorkProjectionFailure>,
) -> WorkProjectionState {
    match result {
        Ok(None) => WorkProjectionState::Unavailable,
        Ok(Some((items, truncated))) => WorkProjectionState::Available { items, truncated },
        Err(WorkProjectionFailure::Worker | WorkProjectionFailure::Storage) => {
            WorkProjectionState::ReadError
        }
    }
}

fn project_work_graph(graph: talos_core::work::WorkGraph) -> (Vec<WorkItemProjection>, bool) {
    use talos_core::work::{WorkKind, WorkStatus};

    let mut items = graph
        .nodes
        .into_iter()
        .filter(|node| node.identity.kind == WorkKind::WorkUnit);
    let mut projected = Vec::new();
    for node in items.by_ref().take(MAX_WORK_ITEMS) {
        let (title, title_truncated) = truncate_work_title(&node.title);
        let status = match node.status {
            WorkStatus::Todo => WorkUnitStatus::Todo,
            WorkStatus::InProgress => WorkUnitStatus::InProgress,
            WorkStatus::Completed => WorkUnitStatus::Completed,
            WorkStatus::Blocked => WorkUnitStatus::Blocked,
        };
        projected.push(WorkItemProjection {
            title,
            title_truncated,
            status,
        });
    }
    let truncated = items.next().is_some();
    (projected, truncated)
}

const MAX_WORK_ITEMS: usize = 100;
const MAX_WORK_TITLE_BYTES: usize = 512;

fn truncate_work_title(title: &str) -> (String, bool) {
    let mut end = title.len().min(MAX_WORK_TITLE_BYTES);
    while !title.is_char_boundary(end) {
        end -= 1;
    }
    (title[..end].to_owned(), end < title.len())
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
                RuntimeOutput::ToolRequestContext {
                    call_id: call.id,
                    provenance: format_tool_provenance(&provenance),
                    requested_path: requested_tool_path(&call.input),
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

fn requested_tool_path(input: &serde_json::Value) -> Option<String> {
    ["path", "file_path", "filename"]
        .into_iter()
        .find_map(|key| input.get(key).and_then(serde_json::Value::as_str))
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod auto_tests;

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod cancellation_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use talos_plugin::HookHandler;
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
        assert!(!task.contains("Review"));
        assert_ne!(
            task,
            task_external_id(std::path::Path::new("/tmp/project/one"), "Review _tmp")
        );
        assert_ne!(
            task_external_id(std::path::Path::new("/tmp/project/one"), "中文任务甲"),
            task_external_id(std::path::Path::new("/tmp/project/one"), "中文任务乙")
        );
        assert_ne!(
            task_external_id(std::path::Path::new("/tmp/project/one"), "same goal"),
            task_external_id(std::path::Path::new("/tmp/project/one_"), "same goal")
        );
        let first = new_task_external_id(std::path::Path::new("/tmp/project/one"));
        let second = new_task_external_id(std::path::Path::new("/tmp/project/one"));
        assert_ne!(
            first, second,
            "each explicitly created task needs a fresh identity"
        );
        assert!(first.starts_with("desktop-task-v2-"));
        assert!(!first.contains("/tmp/project/one"));
    }

    #[cfg(unix)]
    #[test]
    fn workspace_identity_distinguishes_non_utf8_paths() {
        use std::os::unix::ffi::OsStringExt;

        let first =
            std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/project/\xff".to_vec()));
        let second =
            std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/project/\xfe".to_vec()));

        assert_ne!(
            workspace_external_id(&first),
            workspace_external_id(&second)
        );
    }

    #[test]
    fn task_identity_listing_is_workspace_scoped_and_read_only() {
        let workspace = TestWorkspace::new();
        let other = TestWorkspace::new();
        let root = workspace.0.join(".talos").join("desktop-sessions");
        let other_root = other.0.join(".talos").join("desktop-sessions");
        let own = task_external_id(&workspace.0, "own");
        let fresh = new_task_external_id(&workspace.0);
        let fresh_again = new_task_external_id(&workspace.0);
        let legacy = format!(
            "desktop-task-desktop-workspace-{}-legacy-goal",
            legacy_workspace_external_id(&workspace.0)
        );
        let foreign = task_external_id(&other.0, "foreign");
        talos_session::DurableSession::open_or_create(&root, &own).expect("own binding");
        talos_session::DurableSession::open_or_create(&root, &fresh).expect("fresh binding");
        talos_session::DurableSession::open_or_create(&root, &fresh_again)
            .expect("second fresh binding");
        talos_session::DurableSession::open_or_create(&root, &legacy).expect("legacy binding");
        talos_session::DurableSession::open_or_create(&root, &foreign).expect("foreign binding");
        let mut expected = vec![legacy, own, fresh, fresh_again];
        expected.sort();
        assert_eq!(
            list_task_external_ids(&workspace.0).expect("list own tasks"),
            expected
        );
        assert!(
            list_task_external_ids(&other.0)
                .expect("missing other binding remains empty")
                .is_empty()
        );
        assert!(!other_root.join("durable-bindings.sqlite").exists());
    }

    #[test]
    fn requested_tool_path_is_not_mistaken_for_execution_evidence() {
        assert_eq!(
            requested_tool_path(&serde_json::json!({"command": "pwd"})),
            None
        );
        assert_eq!(
            requested_tool_path(&serde_json::json!({"file_path": "src/main.rs"})),
            Some("src/main.rs".into())
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn artifact_proposal_retains_bounded_metadata_without_file_contents() {
        let workspace = TestWorkspace::new();
        let (events, _received) = mpsc::channel(16);
        let approval = Arc::new(DesktopApprovalHandler::new(events, workspace.0.clone()));
        let hook = DesktopArtifactCaptureHook::new(uuid::Uuid::new_v4(), approval);
        let context = talos_plugin::HookContext::new(talos_plugin::TurnId(7), workspace.0.clone());
        let mut call = talos_runtime::ToolCall {
            id: "bounded-call".into(),
            name: "write".into(),
            input: serde_json::json!({"path": "file.txt", "content": "private content"}),
        };
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
        )
        .await;
        {
            let proposed = hook.proposed.lock().expect("proposals");
            assert_eq!(proposed.len(), 1);
            assert_eq!(
                proposed.values().next().expect("proposal").input,
                serde_json::json!({"path": "file.txt"})
            );
        }
        call.id = "x".repeat(MAX_ARTIFACT_CALL_ID_BYTES + 1);
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
        )
        .await;
        call.id = "oversized-path".into();
        call.input = serde_json::json!({"path": "x".repeat(MAX_ARTIFACT_PATH_BYTES + 1)});
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
        )
        .await;
        assert_eq!(hook.proposed.lock().expect("proposals").len(), 1);
        assert!(hook.pending.lock().expect("snapshots").is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn artifact_hook_requires_resolved_permission_before_capture() {
        let workspace = TestWorkspace::new();
        let (events, _received) = mpsc::channel(16);
        let approval = Arc::new(DesktopApprovalHandler::new(events, workspace.0.clone()));
        let hook = DesktopArtifactCaptureHook::new(uuid::Uuid::new_v4(), approval);
        let context = talos_plugin::HookContext::new(talos_plugin::TurnId(7), workspace.0.clone());
        let call = talos_runtime::ToolCall {
            id: "permission-bound-call".into(),
            name: "write".into(),
            input: serde_json::json!({"path": "created.txt"}),
        };
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
        )
        .await;
        assert!(hook.pending.lock().expect("pending lock").is_empty());
        for decision in [
            talos_runtime::PermissionDecision::Ask,
            talos_runtime::PermissionDecision::Deny("rejected".into()),
        ] {
            hook.on_event(
                &context,
                &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
            )
            .await;
            hook.on_event(
                &context,
                &mut talos_plugin::HookEvent::AfterPermissionCheck {
                    call: &call,
                    decision,
                },
            )
            .await;
            assert!(hook.pending.lock().expect("pending lock").is_empty());
        }
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::BeforeToolCall { call: &call },
        )
        .await;
        let redacted = talos_runtime::ToolCall {
            input: serde_json::json!({"path": "<redacted>"}),
            ..call.clone()
        };
        hook.on_event(
            &context,
            &mut talos_plugin::HookEvent::AfterPermissionCheck {
                call: &redacted,
                decision: talos_runtime::PermissionDecision::Allow,
            },
        )
        .await;
        assert_eq!(hook.pending.lock().expect("pending lock").len(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn artifact_hook_reports_only_successful_same_path_changes() {
        let workspace = TestWorkspace::new();
        let (events, mut received) = mpsc::channel(16);
        let approval = Arc::new(DesktopApprovalHandler::new(events, workspace.0.clone()));
        let session_id = uuid::Uuid::new_v4();
        let hook = DesktopArtifactCaptureHook::new(session_id, approval);

        let create = talos_runtime::ToolCall {
            id: "create-call".into(),
            name: "write".into(),
            input: serde_json::json!({"path": "created.txt"}),
        };
        hook.observe_before(7, &create).await;
        std::fs::write(workspace.0.join("created.txt"), "created").expect("fixture write");
        hook.observe_after(7, &create, &talos_runtime::ToolResult::success("ok"))
            .await;
        assert_eq!(
            received.try_recv().expect("observed creation"),
            RuntimeOutput::ArtifactChanged {
                session_id,
                turn_id: 7,
                call_id: "create-call".into(),
                operation: "write".into(),
                path: "created.txt".into(),
            }
        );

        let unchanged = talos_runtime::ToolCall {
            id: "unchanged-call".into(),
            name: "edit".into(),
            input: serde_json::json!({"path": "created.txt"}),
        };
        hook.observe_before(7, &unchanged).await;
        hook.observe_after(7, &unchanged, &talos_runtime::ToolResult::success("no-op"))
            .await;
        assert!(received.try_recv().is_err(), "same bytes are not a change");

        let failed = talos_runtime::ToolCall {
            id: "failed-call".into(),
            name: "edit".into(),
            input: serde_json::json!({"path": "created.txt"}),
        };
        hook.observe_before(7, &failed).await;
        std::fs::write(workspace.0.join("created.txt"), "changed despite failure")
            .expect("fixture write");
        hook.observe_after(7, &failed, &talos_runtime::ToolResult::error("failed"))
            .await;
        assert!(
            received.try_recv().is_err(),
            "failed calls are not evidence"
        );

        let outside = talos_runtime::ToolCall {
            id: "outside-call".into(),
            name: "write".into(),
            input: serde_json::json!({"path": "../outside.txt"}),
        };
        hook.observe_before(7, &outside).await;
        assert_eq!(hook.pending.lock().expect("pending lock").len(), 0);
        assert!(received.try_recv().is_err(), "outside paths are ignored");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn artifact_hook_discards_path_mismatch_and_turn_completion_clears_pending() {
        let workspace = TestWorkspace::new();
        std::fs::write(workspace.0.join("one.txt"), "one").expect("fixture write");
        let (events, mut received) = mpsc::channel(16);
        let approval = Arc::new(DesktopApprovalHandler::new(events, workspace.0.clone()));
        let hook = DesktopArtifactCaptureHook::new(uuid::Uuid::new_v4(), approval);

        let before = talos_runtime::ToolCall {
            id: "same-call".into(),
            name: "edit".into(),
            input: serde_json::json!({"path": "one.txt"}),
        };
        hook.observe_before(11, &before).await;
        std::fs::write(workspace.0.join("one.txt"), "changed").expect("fixture write");
        let after = talos_runtime::ToolCall {
            input: serde_json::json!({"path": "other.txt"}),
            ..before
        };
        hook.observe_after(11, &after, &talos_runtime::ToolResult::success("ok"))
            .await;
        assert!(
            received.try_recv().is_err(),
            "changed call identity is ignored"
        );

        let pending = talos_runtime::ToolCall {
            id: "unfinished-call".into(),
            name: "edit".into(),
            input: serde_json::json!({"path": "one.txt"}),
        };
        hook.observe_before(11, &pending).await;
        assert_eq!(hook.pending.lock().expect("pending lock").len(), 1);
        let mut event = talos_plugin::HookEvent::TurnComplete {
            turn_id: talos_plugin::TurnId(11),
            status: talos_plugin::TurnStatus::ProviderError,
        };
        assert!(matches!(
            hook.on_event(
                &talos_plugin::HookContext::new(talos_plugin::TurnId(11), workspace.0.clone()),
                &mut event,
            )
            .await,
            talos_plugin::HookResult::Continue
        ));
        assert!(hook.pending.lock().expect("pending lock").is_empty());
    }

    #[test]
    fn artifact_path_normalization_needs_no_existing_directory() {
        let workspace = TestWorkspace::new();
        let missing_root = workspace.0.join("never-created");
        let absolute = missing_root.join("file.txt");
        assert_eq!(
            requested_path_to_relative(&missing_root, absolute.to_str().expect("fixture path")),
            Some("file.txt".into())
        );
        assert_eq!(
            requested_path_to_relative(&missing_root, "./file.txt"),
            Some("file.txt".into())
        );
        assert_eq!(
            requested_path_to_relative(&missing_root, "../file.txt"),
            None
        );
        assert!(!missing_root.exists());
    }

    #[test]
    #[cfg(unix)]
    fn artifact_capture_keeps_original_root_after_path_replacement() {
        let workspace = TestWorkspace::new();
        let original = workspace.0.join("root");
        std::fs::create_dir(&original).expect("create root");
        std::fs::write(original.join("file.txt"), "original").expect("original file");
        let root = ArtifactCaptureRoot::open(&original).expect("capture root");
        let before = capture_file_from_root(&root, "file.txt").expect("snapshot");
        std::fs::rename(&original, workspace.0.join("moved")).expect("move original root");
        std::fs::create_dir(&original).expect("replacement root");
        std::fs::write(original.join("file.txt"), "replacement").expect("replacement file");
        assert_eq!(capture_file_from_root(&root, "file.txt"), Some(before));
    }

    #[test]
    fn artifact_stalled_read_returns_incomplete_shutdown_receipt() {
        let executor = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let slots = Arc::new(tokio::sync::Semaphore::new(1));
        let permit = slots.clone().try_acquire_owned().expect("capture slot");
        let (release, wait) = std::sync::mpsc::channel();
        let (started, ready) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        executor.spawn_blocking(move || {
            let _permit = permit;
            started.send(()).expect("start receipt");
            let _ = wait.recv();
            drop(_permit);
            let _ = finished.send(());
        });
        ready
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("worker started");
        let terminal = finish_host_executor(executor, RuntimeOutput::Stopped, &slots);
        // Always release the fixture worker before asserting the returned receipt.
        release.send(()).expect("release fixture worker");
        done.recv_timeout(std::time::Duration::from_secs(5))
            .expect("worker finished");
        assert!(
            matches!(terminal, RuntimeOutput::Error(message) if message.contains("cleanup incomplete"))
        );
        assert_eq!(slots.available_permits(), 1);
    }

    #[test]
    #[cfg(unix)]
    fn artifact_fifo_capture_is_bounded_in_isolated_process() {
        const CHILD: &str = "TALOS_ARTIFACT_FIFO_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let workspace = TestWorkspace::new();
            let fifo = workspace.0.join("pipe");
            // Test-only host utility: fail explicitly if unavailable; never skip this boundary.
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(&fifo)
                    .status()
                    .expect("mkfifo required for Unix FIFO regression")
                    .success()
            );
            assert!(capture_workspace_file_sync(&workspace.0, "pipe").is_none());
            return;
        }
        let mut child =
            std::process::Command::new(std::env::current_exe().expect("test executable"))
                .args([
                    "--exact",
                    "runtime_host::tests::artifact_fifo_capture_is_bounded_in_isolated_process",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .spawn()
                .expect("start isolated FIFO test");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("poll isolated FIFO test") {
                assert!(status.success(), "isolated FIFO capture failed: {status}");
                break;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().expect("terminate blocked FIFO test");
                child.wait().expect("reap blocked FIFO test");
                panic!("artifact capture blocked on FIFO without a writer");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn artifact_snapshot_enforces_size_and_symlink_boundaries() {
        use std::io::Write as _;

        let workspace = TestWorkspace::new();
        let bounded_path = workspace.0.join("bounded.bin");
        std::fs::write(
            &bounded_path,
            vec![0_u8; MAX_ARTIFACT_EVIDENCE_BYTES as usize],
        )
        .expect("write bounded fixture");
        assert!(capture_workspace_file_sync(&workspace.0, "bounded.bin").is_some());
        std::fs::OpenOptions::new()
            .append(true)
            .open(&bounded_path)
            .expect("open bounded fixture")
            .write_all(&[0])
            .expect("extend fixture");
        assert!(capture_workspace_file_sync(&workspace.0, "bounded.bin").is_none());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let outside = TestWorkspace::new();
            std::fs::write(outside.0.join("secret.txt"), "not inspected")
                .expect("write external fixture");
            symlink(outside.0.join("secret.txt"), workspace.0.join("linked.txt"))
                .expect("create symlink fixture");
            assert!(capture_workspace_file_sync(&workspace.0, "linked.txt").is_none());
            std::fs::create_dir(workspace.0.join("real")).expect("real directory");
            std::fs::write(workspace.0.join("real/existing.txt"), "fixture")
                .expect("existing file");
            symlink(workspace.0.join("real"), workspace.0.join("alias"))
                .expect("internal directory symlink");
            assert!(capture_workspace_file_sync(&workspace.0, "alias/existing.txt").is_none());
            assert!(capture_workspace_file_sync(&workspace.0, "alias/missing.txt").is_none());
            assert!(capture_workspace_file_sync(&workspace.0, "real/existing.txt").is_some());
            assert!(capture_workspace_file_sync(&workspace.0, "real/missing.txt").is_some());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn opening_a_missing_existing_session_does_not_create_a_binding() {
        let workspace = TestWorkspace::new();
        let storage = workspace.0.join(".talos").join("desktop-sessions");
        let external_id = "desktop-task-v1-missing";
        let provider = Arc::new(MockProvider::new());
        let mut host = RuntimeHost::start_with(
            move || Ok((provider, 128_000, false)),
            workspace.0.clone(),
            Some(DurableBinding {
                root: storage.clone(),
                external_id: external_id.into(),
                mode: DurableOpenMode::Existing,
            }),
        )
        .expect("host starts");

        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(2), host.recv())
                .await
                .expect("host responds"),
            Some(RuntimeOutput::Error(error)) if error.contains("no longer exists")
        ));
        assert!(
            talos_session::DurableSession::list_external_ids(&storage)
                .expect("binding listing succeeds")
                .is_empty()
        );
        assert!(!storage.exists());
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
            Some(DurableBinding {
                root: storage.clone(),
                external_id: external_id.clone(),
                mode: DurableOpenMode::Create,
            }),
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

        let provider = Arc::new(MockProvider::new().with_response("second host"));
        let mut reopened = RuntimeHost::start_with(
            move || Ok((provider, 128_000, false)),
            workspace.0.clone(),
            Some(DurableBinding {
                root: storage.clone(),
                external_id: external_id.clone(),
                mode: DurableOpenMode::Existing,
            }),
        )
        .expect("reopened host starts");
        match tokio::time::timeout(std::time::Duration::from_secs(2), reopened.recv())
            .await
            .expect("history restore event")
            .expect("history restore output")
        {
            RuntimeOutput::HistoryRestored { entries } => {
                assert!(
                    entries
                        .iter()
                        .any(|entry| entry.contains("persist this turn"))
                );
            }
            other => panic!("expected restored history, got {other:?}"),
        }
        reopened
            .try_send(RuntimeCommand::Shutdown)
            .expect("shutdown reopened");
        let mut stopped = false;
        while let Some(output) = reopened.recv().await {
            match output {
                RuntimeOutput::WorkProjection { .. } => {}
                RuntimeOutput::Stopped => {
                    stopped = true;
                    break;
                }
                other => panic!("unexpected output during shutdown: {other:?}"),
            }
        }
        assert!(stopped, "host must report its terminal stop");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn successful_durable_write_emits_execution_bound_artifact_evidence() {
        let workspace = TestWorkspace::new();
        let storage = workspace.0.join(".talos").join("desktop-sessions");
        let external_id = "desktop-artifact-evidence".to_owned();
        let session_id = talos_session::DurableSession::open_or_create(&storage, &external_id)
            .expect("durable session opens")
            .id();
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call(
                    "write",
                    serde_json::json!({
                        "path": "observed.txt",
                        "content": "written by the shared Runtime tool"
                    }),
                )
                .with_response("The file was written."),
        );
        let configured = provider.clone();
        let mut host = RuntimeHost::start_with(
            move || Ok((configured, 128_000, false)),
            workspace.0.clone(),
            Some(DurableBinding {
                root: storage,
                external_id,
                mode: DurableOpenMode::Create,
            }),
        )
        .expect("durable host starts");
        host.try_send(RuntimeCommand::Submit("write a file".into()))
            .expect("submit");

        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let mut observed_change = false;
            loop {
                match host.recv().await.expect("host event") {
                    RuntimeOutput::ApprovalRequested { request_id, .. } => {
                        host.try_send(RuntimeCommand::ApprovalResponse {
                            request_id,
                            choice: ApprovalChoice::ApproveOnce,
                        })
                        .expect("approve this write");
                    }
                    RuntimeOutput::ArtifactChanged {
                        session_id: observed_session,
                        turn_id,
                        call_id,
                        operation,
                        path,
                    } => {
                        assert_eq!(observed_session, session_id);
                        assert_ne!(turn_id, 0);
                        assert!(!call_id.is_empty());
                        assert_eq!(operation, "write");
                        assert_eq!(path, "observed.txt");
                        observed_change = true;
                    }
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Success);
                        assert!(observed_change, "successful write must emit its evidence");
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    _ => {}
                }
            }
        })
        .await
        .expect("approved write and evidence complete");

        assert_eq!(
            std::fs::read_to_string(workspace.0.join("observed.txt")).expect("written file"),
            "written by the shared Runtime tool"
        );
        host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
        assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupted_durable_task_resume_restores_history_without_replaying_write() {
        let workspace = TestWorkspace::new();
        let storage = workspace.0.join(".talos").join("desktop-sessions");
        let external_id = "desktop-task-v1-interrupted".to_owned();
        let provider = Arc::new(MockProvider::new().with_tool_call(
            "write",
            serde_json::json!({
                "path": "interrupted.txt",
                "content": "must not be replayed"
            }),
        ));
        let configured = provider.clone();
        let mut host = RuntimeHost::start_with(
            move || Ok((configured, 128_000, false)),
            workspace.0.clone(),
            Some(DurableBinding {
                root: storage.clone(),
                external_id: external_id.clone(),
                mode: DurableOpenMode::Create,
            }),
        )
        .expect("host starts");
        host.try_send(RuntimeCommand::Submit(
            "request an interrupted write".into(),
        ))
        .expect("submit");
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                match host.recv().await.expect("approval event") {
                    RuntimeOutput::ApprovalRequested { .. } => break,
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    RuntimeOutput::Completed { .. } => panic!("turn completed before approval"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Interrupt)
                .expect("interrupt accepted");
            loop {
                match host.recv().await.expect("cancel terminal") {
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Cancelled);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("host error: {error}"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown accepted");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("interrupted turn shuts down");

        let replay_provider = Arc::new(MockProvider::new().with_tool_call(
            "write",
            serde_json::json!({
                "path": "interrupted.txt",
                "content": "must not be replayed"
            }),
        ));
        let mut reopened = RuntimeHost::start_with(
            move || Ok((replay_provider, 128_000, false)),
            workspace.0.clone(),
            Some(DurableBinding {
                root: storage.clone(),
                external_id: external_id.clone(),
                mode: DurableOpenMode::Existing,
            }),
        )
        .expect("existing session reopens");
        match tokio::time::timeout(std::time::Duration::from_secs(2), reopened.recv())
            .await
            .expect("history restore event")
            .expect("history restore output")
        {
            RuntimeOutput::HistoryRestored { entries } => assert!(
                entries
                    .iter()
                    .any(|entry| entry.contains("request an interrupted write"))
            ),
            other => panic!("expected restored history, got {other:?}"),
        }
        reopened
            .try_send(RuntimeCommand::Shutdown)
            .expect("shutdown reopened session");
        let mut stopped = false;
        while let Some(output) = reopened.recv().await {
            match output {
                RuntimeOutput::WorkProjection { .. } => {}
                RuntimeOutput::Stopped => {
                    stopped = true;
                    break;
                }
                other => panic!("unexpected output during shutdown: {other:?}"),
            }
        }
        assert!(stopped, "host must report its terminal stop");
        assert!(!workspace.0.join("interrupted.txt").exists());
    }

    fn evaluation_harness(
        verdict: talos_core::evaluation::CriterionVerdict,
        current_goal_revision: u64,
    ) -> EvaluationHarness {
        use talos_core::evaluation::{
            AcceptanceCriterion, CompletionClaim, CriterionEvaluation, CriterionKind,
            EvaluationReport, EvaluationSubject, EvaluationVerdict, EvidenceRef, WorkspaceRevision,
        };
        use talos_core::work::{MissionEvaluation, WorkIdentity, WorkKind};

        let mission = WorkIdentity {
            id: uuid::Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: uuid::Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };
        let criterion = AcceptanceCriterion {
            id: uuid::Uuid::new_v4(),
            kind: CriterionKind::Validation,
            statement: "the declared validation passes".into(),
            required: true,
        };
        let evidence_ref = EvidenceRef {
            id: uuid::Uuid::new_v4(),
            kind: "validation-run".into(),
        };
        let subject = EvaluationSubject {
            mission,
            goal,
            workspace: WorkspaceRevision {
                id: uuid::Uuid::new_v4(),
                revision: 1,
            },
        };
        let claim = CompletionClaim::new(
            subject,
            vec![criterion.clone()],
            Vec::new(),
            Vec::new(),
            "test-only Desktop host fixture",
        )
        .expect("claim");
        let report = EvaluationReport::new(
            &claim,
            subject,
            vec![CriterionEvaluation {
                criterion_id: criterion.id,
                verdict,
                evidence: if verdict == talos_core::evaluation::CriterionVerdict::Pass {
                    vec![evidence_ref.clone()]
                } else {
                    Vec::new()
                },
                finding_ids: Vec::new(),
            }],
            Vec::new(),
        )
        .expect("report");
        let service = talos_runtime::RuntimeEvaluationService::new(
            Arc::new(EvaluationFixtureAssessor(
                serde_json::to_string(&report).expect("serialize report"),
            )),
            std::time::Duration::from_secs(1),
        );
        let evidence = if verdict == talos_core::evaluation::CriterionVerdict::Pass {
            vec![
                talos_runtime::ValidationEvidence::new(
                    evidence_ref,
                    talos_runtime::ValidationEvidenceStatus::Passed,
                    "test-only-fixture-digest",
                )
                .expect("integrity-bound fixture evidence"),
            ]
        } else {
            Vec::new()
        };

        EvaluationHarness {
            service,
            claim,
            current_subject: EvaluationSubject {
                goal: WorkIdentity {
                    revision: current_goal_revision,
                    ..goal
                },
                ..subject
            },
            evidence,
            mission,
            required_goals: vec![WorkIdentity {
                revision: current_goal_revision,
                ..goal
            }],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
    }

    struct EvaluationFixtureAssessor(String);

    struct PausedEvaluationAssessor {
        started: Arc<tokio::sync::Notify>,
        dropped: Arc<tokio::sync::Notify>,
    }

    struct EvaluationDropReceipt(Arc<tokio::sync::Notify>);

    impl Drop for EvaluationDropReceipt {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }

    #[async_trait::async_trait]
    impl talos_runtime::EvaluatorAssessor for PausedEvaluationAssessor {
        async fn assess(
            &self,
            _request: talos_runtime::EvaluatorRequest,
            _deadline: std::time::Duration,
        ) -> Result<String, String> {
            let _receipt = EvaluationDropReceipt(self.dropped.clone());
            self.started.notify_one();
            std::future::pending().await
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn paused_evaluation_does_not_block_interrupt_or_shutdown() {
        for interrupt in [true, false] {
            let workspace = TestWorkspace::new();
            let started = Arc::new(tokio::sync::Notify::new());
            let dropped = Arc::new(tokio::sync::Notify::new());
            let mut harness = evaluation_harness(talos_core::evaluation::CriterionVerdict::Pass, 1);
            harness.service = talos_runtime::RuntimeEvaluationService::new(
                Arc::new(PausedEvaluationAssessor {
                    started: started.clone(),
                    dropped: dropped.clone(),
                }),
                std::time::Duration::from_secs(30),
            );
            let mut host = RuntimeHost::start_with_evaluation(
                || Ok((Arc::new(MockProvider::new()), 128_000, false)),
                workspace.0.clone(),
                None,
                Some(harness),
            )
            .expect("host");
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                host.try_send(RuntimeCommand::Evaluate).expect("evaluate");
                started.notified().await;
                host.try_send(if interrupt {
                    RuntimeCommand::Interrupt
                } else {
                    RuntimeCommand::Shutdown
                })
                .expect("stop command");
                dropped.notified().await;
                if interrupt {
                    loop {
                        match host.recv().await.expect("cancel output") {
                            RuntimeOutput::EvaluationUnavailable { reason } => {
                                assert_eq!(reason, "evaluation cancelled");
                                break;
                            }
                            RuntimeOutput::EvaluationResult { .. } => {
                                panic!("cancelled evaluator returned a verdict")
                            }
                            _ => {}
                        }
                    }
                    host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
                }
                loop {
                    match host.recv().await.expect("stop receipt") {
                        RuntimeOutput::Stopped => break,
                        RuntimeOutput::EvaluationResult { .. } => panic!("late evaluator verdict"),
                        _ => {}
                    }
                }
            })
            .await
            .expect("host must respond before the 30-second evaluator deadline");
        }
    }

    #[async_trait::async_trait]
    impl talos_runtime::EvaluatorAssessor for EvaluationFixtureAssessor {
        async fn assess(
            &self,
            _request: talos_runtime::EvaluatorRequest,
            _deadline: std::time::Duration,
        ) -> Result<String, String> {
            Ok(self.0.clone())
        }

        fn identity(&self) -> &str {
            "desktop-integration-fixture"
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn evaluate_command_projects_shared_pass_fail_and_stale_delivery() {
        use talos_core::evaluation::CriterionVerdict;
        use talos_core::work::{DeliveryBlockReason, DeliveryEligibility};

        for (verdict, goal_revision, expected_state, expected_delivery) in [
            (
                CriterionVerdict::Pass,
                1,
                "Verdict(Pass)",
                DeliveryEligibility::Eligible,
            ),
            (
                CriterionVerdict::Fail,
                1,
                "Verdict(Fail)",
                DeliveryEligibility::Blocked {
                    reason: DeliveryBlockReason::GoalNotPassed,
                },
            ),
            (
                CriterionVerdict::Pass,
                2,
                "Stale",
                DeliveryEligibility::Blocked {
                    reason: DeliveryBlockReason::StaleGoalEvaluation,
                },
            ),
        ] {
            let workspace = TestWorkspace::new();
            let provider = Arc::new(MockProvider::new());
            let configured = provider.clone();
            let mut host = RuntimeHost::start_with_evaluation(
                move || Ok((configured, 128_000, false)),
                workspace.0.clone(),
                None,
                Some(evaluation_harness(verdict, goal_revision)),
            )
            .expect("host starts with test-only evaluator fixture");
            host.try_send(RuntimeCommand::Evaluate)
                .expect("queue explicit evaluation");

            let projected = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    match host.recv().await.expect("host event") {
                        RuntimeOutput::EvaluationResult { state, delivery } => {
                            break (state, delivery);
                        }
                        RuntimeOutput::Error(error) => panic!("host error: {error}"),
                        _ => {}
                    }
                }
            })
            .await
            .expect("evaluation projection arrives");
            assert_eq!(projected, (expected_state.to_owned(), expected_delivery));

            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown fixture host");
            while !matches!(host.recv().await, Some(RuntimeOutput::Stopped) | None) {}
        }
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
    async fn explicit_evaluation_without_authoritative_source_reports_unavailable() {
        let workspace = TestWorkspace::new();
        let provider = Arc::new(MockProvider::new());
        let configured = provider.clone();
        let mut host = RuntimeHost::start_with(
            move || Ok((configured, 128_000, false)),
            workspace.0.clone(),
            None,
        )
        .expect("host starts");
        let command = host.command_sender();
        command
            .send(RuntimeCommand::Evaluate)
            .await
            .expect("evaluate");
        host.try_send(RuntimeCommand::Evaluate)
            .expect("second evaluate");
        let mut unavailable_count = 0;
        while unavailable_count < 2 {
            match tokio::time::timeout(std::time::Duration::from_secs(5), host.recv())
                .await
                .expect("evaluation response arrives")
                .expect("host event")
            {
                RuntimeOutput::EvaluationUnavailable { reason } => {
                    assert!(reason.contains("no authoritative claim"));
                    unavailable_count += 1;
                }
                RuntimeOutput::Error(error) => panic!("host error: {error}"),
                _ => {}
            }
        }
        host.try_send(RuntimeCommand::Shutdown).expect("shutdown");
        while !matches!(host.recv().await, Some(RuntimeOutput::Stopped) | None) {}
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
    fn work_projection_distinguishes_missing_data_from_read_failure() {
        assert_eq!(
            work_projection_state(Ok(None)),
            WorkProjectionState::Unavailable
        );
        assert_eq!(
            work_projection_state(Err(WorkProjectionFailure::Storage)),
            WorkProjectionState::ReadError
        );
        assert_eq!(
            work_projection_state(Ok(Some((Vec::new(), false)))),
            WorkProjectionState::Available {
                items: Vec::new(),
                truncated: false,
            }
        );
    }

    #[tokio::test]
    async fn desktop_work_projection_reads_only_the_bound_shared_session() {
        let sessions_dir = TestWorkspace::new();
        let manager = talos_runtime::SessionManager::with_dir(sessions_dir.0.clone());
        let repository = manager.todo_repository().expect("todo repository");
        let session_id = uuid::Uuid::new_v4();
        let other_session_id = uuid::Uuid::new_v4();
        for (session_id, title, status) in [
            (
                session_id,
                "current task",
                talos_session::todo::TodoStatus::InProgress,
            ),
            (
                other_session_id,
                "other task",
                talos_session::todo::TodoStatus::Blocked,
            ),
        ] {
            repository
                .create(talos_session::todo::CreateTodo {
                    session_id,
                    title: title.to_owned(),
                    description: None,
                    priority: talos_session::todo::TodoPriority::Medium,
                    assigned_to_turn: None,
                    tags: Vec::new(),
                })
                .expect("create session todo");
            if status != talos_session::todo::TodoStatus::Todo {
                let item = repository
                    .list(session_id, talos_session::todo::TodoQuery::default())
                    .expect("query created todo")
                    .remove(0);
                repository
                    .update_status(session_id, item.id, status)
                    .expect("set status");
            }
        }
        drop(repository);

        let projection = load_work_projection(Some((sessions_dir.0.clone(), session_id)))
            .await
            .expect("work projection loads")
            .expect("existing shared store is available");
        assert_eq!(projection.0.len(), 1);
        assert_eq!(projection.0[0].title, "current task");
        assert_eq!(projection.0[0].status, WorkUnitStatus::InProgress);
        assert!(!projection.1);
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
            run_host(command_rx, outputs, handle, None, None, None),
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
            run_host(command_rx, outputs, handle, None, None, None),
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
