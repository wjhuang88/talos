//! Talos Conversation Engine.
//!
//! Owns all conversation state and event processing, sitting between the
//! Agent Loop and the UI Loop. Communicates via typed async channels.
//!
//! This crate is the reusable state/command layer for Talos-style interfaces. Its pre-1.0 support
//! boundary is:
//!
//! - typed conversation inputs and UI outputs are public integration surfaces;
//! - terminal rendering, keyboard handling, and visual layout are owned by `talos-tui`;
//! - provider execution and tool execution are owned by the runtime/agent layers;
//! - command registration is reusable, but built-in command semantics may still evolve before 1.0;
//! - consumers should expect additive events and fields while the external UI contract settles.
//!
//! Alternate UIs can depend on this crate to share conversation state transitions without pulling
//! in the Talos terminal UI.

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
    MessageRole, MessageSource, MessageStatus, ModelInfo, ModelPickerData, ModelPickerItem,
    ModelSwitchRequest, PluginObservation, ProviderSetupItem, ScrollbackState,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionPickerItem,
    SessionResumeRequest, SkillCommandRequest, SkillDiagnostic, StatusSnapshot, StreamMessage,
    TipKind, ToolCallDisplay, ToolCallInfo, ToolResultDisplay, UiOutput, UserInput,
};
