//! Hook handler traits and context.

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;

use crate::event::{HookEvent, HookEventKind, TurnId};

/// Shared context passed to every hook handler invocation.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Current turn identifier.
    pub turn_id: TurnId,
    /// Workspace root associated with the agent.
    pub workspace_root: PathBuf,
}

impl HookContext {
    /// Creates a new hook context.
    #[must_use]
    pub fn new(turn_id: TurnId, workspace_root: PathBuf) -> Self {
        Self {
            turn_id,
            workspace_root,
        }
    }
}

/// The outcome returned by an individual hook handler.
#[derive(Debug)]
pub enum HookResult {
    /// Continue dispatching to the next subscribed handler.
    Continue,
    /// Stop dispatching and ask the caller to skip the wrapped action.
    Skip,
    /// Stop dispatching and deny the wrapped action.
    Deny {
        /// Human-readable denial reason.
        reason: String,
    },
    /// Replace the current event and continue dispatching.
    Modify(HookEvent<'static>),
}

/// A lifecycle hook handler.
#[async_trait]
pub trait HookHandler: Send + Sync {
    /// Stable handler name used in logs and tests.
    fn name(&self) -> &str;

    /// Events this handler subscribes to.
    fn subscribed(&self) -> &'static [HookEventKind];

    /// Maximum time the framework will wait for `on_event`.
    fn timeout(&self) -> Duration {
        Duration::from_millis(500)
    }

    /// Handles a runtime event.
    async fn on_event(&self, ctx: &HookContext, event: &mut HookEvent<'_>) -> HookResult;
}
