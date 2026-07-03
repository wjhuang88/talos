//! Hook registry and dispatch runtime.

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use futures_util::FutureExt;
use tokio::time::timeout;

use crate::error::HookError;
use crate::event::{HookEvent, HookEventKind};
use crate::handler::{HookContext, HookHandler, HookResult};

/// Read-only diagnostic view of a registered hook handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRegistration {
    /// Event kind the handler subscribed to.
    pub event: HookEventKind,
    /// Stable handler name.
    pub handler: String,
}

/// Final outcome of dispatching a hook event through the registry.
#[derive(Debug)]
pub enum HookOutcome<'a> {
    /// Dispatch completed normally.
    Continue(HookEvent<'a>),
    /// A handler requested that the wrapped action be skipped.
    Skip(HookEvent<'a>),
    /// A handler denied the wrapped action.
    Deny {
        /// Event state when dispatch stopped.
        event: HookEvent<'a>,
        /// Denial reason.
        reason: String,
    },
}

impl<'a> HookOutcome<'a> {
    /// Returns the final event state.
    #[must_use]
    pub fn into_event(self) -> HookEvent<'a> {
        match self {
            Self::Continue(event) | Self::Skip(event) => event,
            Self::Deny { event, .. } => event,
        }
    }
}

/// Per-agent hook registry.
#[derive(Default)]
pub struct HookRegistry {
    handlers: HashMap<HookEventKind, Vec<Arc<dyn HookHandler>>>,
}

impl HookRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler for all of its subscribed event kinds.
    pub fn register(&mut self, handler: Arc<dyn HookHandler>) {
        for kind in handler.subscribed() {
            self.handlers
                .entry(*kind)
                .or_default()
                .push(handler.clone());
        }
    }

    /// Returns the number of handlers registered for an event kind.
    #[must_use]
    pub fn handlers_for(&self, kind: HookEventKind) -> usize {
        self.handlers.get(&kind).map_or(0, Vec::len)
    }

    /// Returns a read-only diagnostic snapshot of registered handlers.
    #[must_use]
    pub fn registrations(&self) -> Vec<HookRegistration> {
        let mut registrations = Vec::new();
        for kind in crate::event::ALL_HOOK_EVENT_KINDS {
            if let Some(handlers) = self.handlers.get(&kind) {
                for handler in handlers {
                    registrations.push(HookRegistration {
                        event: kind,
                        handler: handler.name().to_string(),
                    });
                }
            }
        }
        registrations
    }

    /// Dispatches a hook event sequentially to all subscribed handlers.
    pub async fn dispatch<'a>(
        &self,
        ctx: &HookContext,
        mut event: HookEvent<'a>,
    ) -> HookOutcome<'a> {
        let kind = event.kind();
        let Some(handlers) = self.handlers.get(&kind) else {
            return HookOutcome::Continue(event);
        };

        for handler in handlers {
            let handler_name = handler.name().to_owned();
            let handler_future = AssertUnwindSafe(handler.on_event(ctx, &mut event)).catch_unwind();

            match timeout(handler.timeout(), handler_future).await {
                Ok(Ok(HookResult::Continue)) => {}
                Ok(Ok(HookResult::Skip)) => return HookOutcome::Skip(event),
                Ok(Ok(HookResult::Deny { reason })) => {
                    let error = HookError::Denied {
                        handler: handler_name,
                        reason: reason.clone(),
                    };
                    tracing::warn!(error = %error, event = %kind, "hook denied event");
                    return HookOutcome::Deny { event, reason };
                }
                Ok(Ok(HookResult::Modify(modified))) => {
                    if event.is_permission_boundary() {
                        tracing::error!(
                            handler = %handler.name(),
                            event = %kind,
                            "hook modify ignored for permission boundary"
                        );
                    } else {
                        event = modified;
                    }
                }
                Ok(Err(_)) => {
                    let error = HookError::Panic {
                        handler: handler_name,
                    };
                    tracing::error!(error = %error, event = %kind, "hook panicked; aborting hook chain");
                    return HookOutcome::Continue(event);
                }
                Err(_) => {
                    let error = HookError::Timeout {
                        handler: handler_name,
                        timeout: handler.timeout(),
                    };
                    tracing::warn!(error = %error, event = %kind, "hook timed out");
                }
            }
        }

        HookOutcome::Continue(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::TurnId;
    use crate::handler::HookContext;
    use async_trait::async_trait;
    use std::time::Duration;

    struct DiagnosticHook;

    #[async_trait]
    impl HookHandler for DiagnosticHook {
        fn name(&self) -> &str {
            "diagnostic"
        }

        fn subscribed(&self) -> &'static [HookEventKind] {
            &[HookEventKind::TurnStart, HookEventKind::TurnComplete]
        }

        fn timeout(&self) -> Duration {
            Duration::from_millis(10)
        }

        async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
            HookResult::Continue
        }
    }

    #[test]
    fn pre_filter_by_kind() {
        let registry = HookRegistry::new();
        let ctx = HookContext::new(TurnId::new(), std::path::PathBuf::from("."));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let outcome = registry
                .dispatch(
                    &ctx,
                    HookEvent::TurnStart {
                        turn_id: ctx.turn_id,
                    },
                )
                .await;
            assert!(matches!(
                outcome,
                HookOutcome::Continue(HookEvent::TurnStart { .. })
            ));
        });
    }

    #[test]
    fn registrations_reports_handlers_without_dispatch() {
        let mut registry = HookRegistry::new();
        registry.register(Arc::new(DiagnosticHook));

        let registrations = registry.registrations();

        assert_eq!(registrations.len(), 2);
        assert_eq!(
            registrations[0],
            HookRegistration {
                event: HookEventKind::TurnStart,
                handler: "diagnostic".to_string(),
            }
        );
        assert_eq!(
            registrations[1],
            HookRegistration {
                event: HookEventKind::TurnComplete,
                handler: "diagnostic".to_string(),
            }
        );
    }
}
