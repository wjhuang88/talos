//! Talos Conversation Engine.
//!
//! Owns all conversation state and event processing, sitting between the
//! Agent Loop and the UI Loop. Communicates via typed async channels.

mod engine;
#[cfg(test)]
mod engine_tests;
mod types;

pub use engine::{
    AvailabilityPredicate, CommandDefinition, CommandOrigin, CommandRegistry, ConversationEngine,
    always_available, command_registry,
};
pub use types::{
    ChatMessage, CopyScope, McpServerDiagnostic, MessageRole, MessageSource, MessageStatus,
    PluginObservation, ScrollbackState, SkillDiagnostic, StatusSnapshot, StreamMessage, TipKind,
    ToolCallDisplay, ToolCallInfo, ToolResultDisplay, UiOutput, UserInput,
};
