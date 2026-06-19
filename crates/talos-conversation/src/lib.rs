//! Talos Conversation Engine.
//!
//! Owns all conversation state and event processing, sitting between the
//! Agent Loop and the UI Loop. Communicates via typed async channels.

mod engine;
#[cfg(test)]
mod engine_tests;
mod types;

pub use engine::ConversationEngine;
pub use types::{
    ChatMessage, MessageRole, MessageSource, MessageStatus, PluginObservation, ScrollbackState,
    SkillDiagnostic, StatusSnapshot, StreamMessage, TipKind, ToolCallDisplay, ToolCallInfo,
    ToolResultDisplay, UiOutput, UserInput,
};
