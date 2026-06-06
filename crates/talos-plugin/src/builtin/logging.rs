//! Built-in tracing-based hook logger.

use async_trait::async_trait;

use crate::event::{ALL_HOOK_EVENT_KINDS, HookEvent, HookEventKind};
use crate::handler::{HookContext, HookHandler, HookResult};

/// A built-in hook handler that logs every event via `tracing`.
#[derive(Debug, Default)]
pub struct LoggingHandler;

impl LoggingHandler {
    /// Creates a new logging handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HookHandler for LoggingHandler {
    fn name(&self) -> &str {
        "LoggingHandler"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &ALL_HOOK_EVENT_KINDS
    }

    async fn on_event(&self, ctx: &HookContext, event: &mut HookEvent<'_>) -> HookResult {
        tracing::debug!(
            handler = self.name(),
            event = %event.kind(),
            turn_id = ctx.turn_id.0,
            "hook event"
        );
        HookResult::Continue
    }
}
