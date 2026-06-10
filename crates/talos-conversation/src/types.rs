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
    pub result: Option<talos_core::message::ToolResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginObservation {
    pub key: String,
    pub count: usize,
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
    Tip { text: String, kind: TipKind },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    Message(String),
    Cancel,
    Exit,
}
