//! Talos Conversation Engine.
//!
//! Owns all conversation state and event processing, sitting between the
//! Agent Loop and the UI Loop. Communicates via typed async channels.

mod command_registry;
mod engine;
#[cfg(test)]
mod engine_tests;
mod types;

pub use command_registry::{
    AvailabilityPredicate, CommandDefinition, CommandOrigin, CommandRegistry, always_available,
    command_registry,
};
pub use engine::ConversationEngine;
pub use types::{
    ChatMessage, CopyScope, CredentialRequestData, CredentialResponseData, McpServerDiagnostic,
    MessageRole, MessageSource, MessageStatus, ModelPickerData, ModelPickerItem,
    ModelSwitchRequest, PluginObservation, ProviderSetupItem, ScrollbackState,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionPickerItem,
    SessionResumeRequest, SkillCommandRequest, SkillDiagnostic, StatusSnapshot, StreamMessage,
    TipKind, ToolCallDisplay, ToolCallInfo, ToolResultDisplay, UiOutput, UserInput,
};
