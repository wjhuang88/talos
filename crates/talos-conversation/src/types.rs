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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatusSnapshot {
    pub model_name: String,
    pub usage: Usage,
    pub branch_id: Option<String>,
    pub steering_count: usize,
    pub followup_count: usize,
    pub is_processing: bool,
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
    Tip {
        text: String,
        kind: TipKind,
    },
    ToolCallStarted {
        name: String,
    },
    ToolCall(ToolCallDisplay),
    ToolResult(ToolResultDisplay),
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
    /// Open the bottom panel as a model picker with the given candidates.
    ModelPicker(Vec<ModelPickerItem>),
    /// Request to switch the active model.
    ModelSwitchRequest(ModelSwitchRequest),
    HydrateHistory(Vec<talos_core::message::Message>),
    Exit,
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
    Cancel,
    Exit,
}
