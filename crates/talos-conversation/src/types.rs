use std::pin::Pin;
use std::time::Instant;

use talos_core::message::Usage;
use talos_core::tool::ToolProvenance;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub status: MessageStatus,
    pub content: String,
    pub tool_call: Option<ToolCallInfo>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageStatus {
    Pending,
    Accepted,
    Streaming,
    Completed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub arguments: String,
    pub provenance: ToolProvenance,
    pub result: Option<talos_core::message::MessageToolResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginObservation {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerDiagnostic {
    /// Stable configured MCP server name.
    pub name: String,
    /// Whether startup and initial tool discovery succeeded.
    pub connected: bool,
    /// Number of tools discovered at session startup.
    pub tool_count: usize,
    /// Non-fatal startup error when unavailable.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDiagnostic {
    pub name: String,
    pub description: String,
    pub active: bool,
    /// Where this skill was discovered from (e.g. "project", "user", "shared").
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillCommandRequest {
    Activate { name: String },
    Reference { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoCommandAction {
    List,
    Show { id: String },
    Stats,
    Export { format: TodoExportFormat },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoExportFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoCommandRequest {
    pub action: TodoCommandAction,
    pub status_filter: Option<String>,
    pub priority_filter: Option<String>,
    pub tag_filter: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoPanelData {
    pub title: String,
    pub rows: Vec<TodoPanelRow>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoPanelRow {
    pub id: String,
    pub status: String,
    pub priority: String,
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScrollbackState {
    pub scrolled_line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TipKind {
    ExitHint,
    QueueHint,
    ApprovalResult,
    LagWarning,
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnPhase {
    Connecting,
    Retrying { attempt: u32 },
    Thinking,
    Generating,
    TimedOut,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatusSnapshot {
    pub model_name: String,
    pub provider: String,
    pub workspace_path: String,
    pub usage: Usage,
    pub branch_id: Option<String>,
    pub steering_count: usize,
    pub followup_count: usize,
    pub is_processing: bool,
    pub phase: Option<TurnPhase>,
    pub context_limit: Option<u32>,
    pub input_price_per_million: Option<f64>,
    pub output_price_per_million: Option<f64>,
}

/// Model metadata passed from the CLI layer to the conversation engine.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModelInfo {
    pub model_name: String,
    pub provider: String,
    pub context_limit: Option<u32>,
    pub input_price_per_million: Option<f64>,
    pub output_price_per_million: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSource {
    User,
    Assistant,
    Tool { name: String },
    System,
    Error,
}

pub struct StreamMessage {
    pub source: MessageSource,
    pub stream: Pin<Box<dyn futures::Stream<Item = String> + Send>>,
}

pub enum UiOutput {
    Stream(StreamMessage),
    Status(StatusSnapshot),
    /// Current durable session identity for UI summary surfaces.
    SessionIdentity {
        id: String,
    },
    Tip {
        text: String,
        kind: TipKind,
    },
    ToolCallStarted {
        name: String,
    },
    ToolCall(ToolCallDisplay),
    ToolResult(ToolResultDisplay),
    /// Replace or clear transient thinking preview text.
    ThinkingPreview {
        /// Current thinking preview text; `None` clears it.
        text: Option<String>,
    },
    ToolApprovalRequest {
        tool_name: String,
        arguments: serde_json::Value,
        summary_fields: Vec<String>,
        response: tokio::sync::oneshot::Sender<talos_core::ApprovalChoice>,
    },
    /// Request the TUI/bridge to copy text to clipboard.
    /// The engine prepares the text; the bridge executes the I/O.
    CopyToClipboard {
        text: String,
        scope: CopyScope,
    },
    /// Request the TUI/bridge to write transcript content to a file.
    /// The engine prepares the content; the bridge handles permissions and I/O.
    ExportToFile {
        path: std::path::PathBuf,
        content: String,
    },
    /// Request a session transition to a fresh session (`/new`).
    /// The mode runner creates the new session, swaps the agent context,
    /// and reports success or failure back to the UI.
    SessionNew(SessionNewRequest),
    /// Request a session transition to an existing session (`/resume`).
    /// The mode runner validates the target, hydrates history, and swaps
    /// the agent context.
    SessionResume(SessionResumeRequest),
    /// Request a session fork — clone the active session's durable history
    /// into a distinct child identity (`/fork`). The mode runner copies the
    /// source JSONL, creates a new session, and swaps the agent context.
    SessionFork(SessionForkRequest),
    SessionDelete(SessionDeleteRequest),
    SessionPicker(Vec<SessionPickerItem>),
    /// Open the bottom panel as a two-layer model picker.
    ModelPicker(ModelPickerData),
    /// Open the bottom panel as a connect provider picker.
    ConnectPicker(ConnectPickerData),
    /// Request to switch the active model.
    ModelSwitchRequest(ModelSwitchRequest),
    /// Request to connect (register credential for) a provider.
    /// Empty `provider` signals "show picker".
    ConnectProviderRequest {
        provider: String,
    },
    /// Request runtime Skill activation or reference loading.
    SkillCommand(SkillCommandRequest),
    /// Request a read-only todo list/status/export view for the active session.
    TodoCommand(TodoCommandRequest),
    /// Read-only todo panel data rendered by TUI/bridges.
    TodoPanel(TodoPanelData),
    /// Ask the TUI to collect an API key for the named provider.
    CredentialRequest(CredentialRequestData),
    /// TUI returns a collected API key to the lifecycle handler.
    CredentialResponse(CredentialResponseData),
    HydrateHistory(Vec<talos_core::message::Message>),
    Exit,
}

/// Provider + optional model context for a credential collection prompt.
///
/// When `model_id` is `Some`, the credential is collected for a specific
/// model switch. When `None`, the credential is collected at the provider
/// level (normally from `/connect`) and the picker should re-open after the
/// key is saved.
#[derive(Debug, Clone)]
pub struct CredentialRequestData {
    pub provider: String,
    pub model_id: Option<String>,
    pub connect_mode: bool,
    /// Catalog/builtin base URL suggested as the default endpoint.
    /// Used when the user leaves the base URL field blank in `/connect`.
    pub default_base_url: Option<String>,
}

/// User-entered credential returned from the TUI credential input panel.
///
/// Mirrors [`CredentialRequestData`]: `model_id` is `Some` for a direct
/// model switch, `None` for provider-level setup.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialResponseData {
    pub provider: String,
    pub api_key: String,
    pub model_id: Option<String>,
    pub connect_mode: bool,
    /// Resolved base URL: user-entered value, falling back to the request's
    /// `default_base_url` when the user left the field blank. `None` means
    /// no base URL should be written (use the provider adapter's default).
    pub base_url: Option<String>,
}

impl std::fmt::Debug for CredentialResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialResponseData")
            .field("provider", &self.provider)
            .field("api_key", &"***")
            .field("model_id", &self.model_id)
            .field("connect_mode", &self.connect_mode)
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ToolCallDisplay {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub provenance: ToolProvenance,
    pub summary_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ToolResultDisplay {
    pub tool_name: Option<String>,
    pub is_error: bool,
    pub content: String,
}

/// Scope for the `/copy` slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyScope {
    /// Copy only the last assistant text message.
    Last,
    /// Copy the full transcript as plain text.
    All,
}

/// Request to transition to a new session (created by `/new`).
///
/// The bridge forwards this to the mode runner, which creates a fresh
/// [`talos_session::Session`] and swaps the active agent context.
pub struct SessionNewRequest;

/// Request to resume an existing session (created by `/resume`).
///
/// If `session_id` is `None`, the mode runner lists workspace-scoped candidates.
/// If `Some`, the mode runner validates and loads the specified session.
pub struct SessionResumeRequest {
    /// Optional explicit session ID to resume.
    pub session_id: Option<String>,
}

/// Request to fork the active session (created by `/fork`).
///
/// The mode runner clones the source session's JSONL file to a new path
/// with a fresh UUID, creates a new [`talos_session::Session`], and swaps
/// the active agent context. The source session remains byte-for-byte unchanged.
pub struct SessionForkRequest;

pub struct SessionDeleteRequest {
    pub selection: Option<String>,
}

/// A candidate model displayed in the interactive model picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerItem {
    /// Slash command the picker stands in for, always `"/model"`.
    pub command: String,
    /// Model identifier, e.g. `"claude-sonnet-4-20250514"`.
    pub model_id: String,
    /// Provider name, e.g. `"anthropic"`.
    pub provider: String,
    /// Display line shown in the picker, e.g.
    /// `"claude-sonnet-4-20250514   Anthropic  200K  $3/$15"`.
    pub label: String,
    /// Context window limit in tokens, if known.
    pub context_limit: Option<u32>,
    /// Pre-formatted pricing string, e.g. `"$3/$15 per 1M"` or `None`.
    pub pricing: Option<String>,
    /// `true` → Ready group; `false` → Setup required group.
    pub authenticated: bool,
    /// `true` → this is the currently active model+provider.
    pub is_current: bool,
}

/// Payload for [`UiOutput::ModelPicker`] — drives the two-layer picker.
///
/// `ready_models` are authenticated and listed individually. `setup_providers`
/// is retained for wire compatibility with older callers, but current `/model`
/// UX leaves unauthenticated provider setup to `/connect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerData {
    pub ready_models: Vec<ModelPickerItem>,
    pub setup_providers: Vec<ProviderSetupItem>,
}

/// Legacy unauthenticated provider row for model picker payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSetupItem {
    pub provider: String,
    pub model_count: usize,
}

/// A candidate provider displayed in the interactive connect picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPickerItem {
    pub provider: String,
    pub name: String,
    pub model_count: usize,
    pub api_base_url: Option<String>,
    pub has_credential: bool,
    pub doc_url: Option<String>,
}

/// Payload for [`UiOutput::ConnectPicker`] — drives the two-group provider picker.
///
/// `connected` providers have credentials configured. `available` providers
/// are in the catalog but not yet connected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectPickerData {
    pub connected: Vec<ConnectPickerItem>,
    pub available: Vec<ConnectPickerItem>,
}

/// Request to switch the active model (created by `/model`).
///
/// If `model_id` is empty, the mode runner should list available models
/// and open the model picker. If `Some`, the mode runner should attempt
/// to switch to the specified model directly.
pub struct ModelSwitchRequest {
    /// Target model ID. Empty string signals "show picker".
    pub model_id: String,
    /// Whether the provider needs credential setup before this model can be used.
    pub provider_needs_credential: bool,
}

/// A candidate session displayed in the interactive session picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPickerItem {
    /// Slash command the picker stands in for, e.g. `"/resume"` or `"/delete"`.
    /// When the user accepts a row, the TUI submits `"{command} {ordinal}"`
    /// back into the composer, letting the same picker UI serve any
    /// session-list command.
    pub command: String,
    /// 1-based ordinal for `/resume <N>` selection.
    pub ordinal: usize,
    /// Human-readable timestamp (e.g., "2026-06-22 19:20").
    pub timestamp: String,
    /// Number of messages in the session.
    pub message_count: usize,
    /// Truncated preview of the last message.
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    Message(String),
    Credential(CredentialResponseData),
    /// User selected a provider-level setup row. The bridge routes this to
    /// provider-level credential entry.
    ProviderSetup(String),
    Cancel,
    Exit,
}
