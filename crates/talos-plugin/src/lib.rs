//! Talos plugin — lifecycle hooks and built-in hook handlers.

pub mod builtin;
pub mod error;
pub mod event;
pub mod handler;
pub mod registry;

pub use builtin::LoggingHandler;
pub use error::HookError;
pub use event::{
    ALL_HOOK_EVENT_KINDS, BudgetKind, HookEvent, HookEventKind, ToolObservation, TurnEndReason,
    TurnId, TurnStatus,
};
pub use handler::{HookContext, HookHandler, HookResult};
pub use registry::{HookOutcome, HookRegistry};
