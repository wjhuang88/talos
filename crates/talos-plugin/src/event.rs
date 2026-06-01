//! Hook event types.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use talos_core::message::{Message, ToolCall};
use talos_core::provider::ProviderError;
use talos_core::tool::ToolResult;
use talos_permission::PermissionDecision;

static NEXT_TURN_ID: AtomicU64 = AtomicU64::new(1);

/// A stable identifier for a single agent turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnId(pub u64);

impl TurnId {
    /// Creates a new unique turn identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_TURN_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TurnId {
    fn default() -> Self {
        Self::new()
    }
}

/// Final status recorded for a completed turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnStatus {
    /// The turn completed successfully.
    Success,
    /// The provider failed before the turn could complete.
    ProviderError,
    /// The provider emitted an unexpected event sequence.
    UnexpectedEvent,
    /// The turn exceeded its tool-call budget.
    BudgetExceeded,
    /// The turn was terminated due to doom-loop detection.
    DoomLoopDetected,
    /// The turn was denied by a hook.
    Denied,
}

/// Stop reason observed from the provider stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEndReason {
    /// The assistant finished naturally.
    EndTurn,
    /// The assistant requested tool use.
    ToolUse,
    /// The provider hit its maximum token limit.
    MaxTokens,
}

/// Budget category exceeded by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetKind {
    /// The turn exceeded the tool-call count limit.
    ToolCalls,
}

/// Owned observation of a completed tool result.
#[derive(Debug, Clone)]
pub struct ToolObservation {
    /// The tool call that produced the result.
    pub call: ToolCall,
    /// The observed tool result.
    pub result: ToolResult,
}

/// Discriminant for hook event subscription and dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEventKind {
    /// `HookEvent::TurnStart`.
    TurnStart,
    /// `HookEvent::OnSystemPromptBuilt`.
    OnSystemPromptBuilt,
    /// `HookEvent::BeforeProviderCall`.
    BeforeProviderCall,
    /// `HookEvent::AfterProviderCall`.
    AfterProviderCall,
    /// `HookEvent::OnTextDelta`.
    OnTextDelta,
    /// `HookEvent::OnToolCallProposed`.
    OnToolCallProposed,
    /// `HookEvent::BeforeToolBatch`.
    BeforeToolBatch,
    /// `HookEvent::BeforePermissionCheck`.
    BeforePermissionCheck,
    /// `HookEvent::AfterPermissionCheck`.
    AfterPermissionCheck,
    /// `HookEvent::BeforeBashSandboxExec`.
    BeforeBashSandboxExec,
    /// `HookEvent::AfterBashSandboxExec`.
    AfterBashSandboxExec,
    /// `HookEvent::BeforeToolCall`.
    BeforeToolCall,
    /// `HookEvent::AfterToolCall`.
    AfterToolCall,
    /// `HookEvent::OnToolResultObserved`.
    OnToolResultObserved,
    /// `HookEvent::AfterToolBatch`.
    AfterToolBatch,
    /// `HookEvent::OnDoomLoopDetected`.
    OnDoomLoopDetected,
    /// `HookEvent::OnBudgetExceeded`.
    OnBudgetExceeded,
    /// `HookEvent::OnProviderError`.
    OnProviderError,
    /// `HookEvent::OnTurnEnd`.
    OnTurnEnd,
    /// `HookEvent::TurnComplete`.
    TurnComplete,
}

impl std::fmt::Display for HookEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// All currently defined hook event kinds.
pub const ALL_HOOK_EVENT_KINDS: [HookEventKind; 20] = [
    HookEventKind::TurnStart,
    HookEventKind::OnSystemPromptBuilt,
    HookEventKind::BeforeProviderCall,
    HookEventKind::AfterProviderCall,
    HookEventKind::OnTextDelta,
    HookEventKind::OnToolCallProposed,
    HookEventKind::BeforeToolBatch,
    HookEventKind::BeforePermissionCheck,
    HookEventKind::AfterPermissionCheck,
    HookEventKind::BeforeBashSandboxExec,
    HookEventKind::AfterBashSandboxExec,
    HookEventKind::BeforeToolCall,
    HookEventKind::AfterToolCall,
    HookEventKind::OnToolResultObserved,
    HookEventKind::AfterToolBatch,
    HookEventKind::OnDoomLoopDetected,
    HookEventKind::OnBudgetExceeded,
    HookEventKind::OnProviderError,
    HookEventKind::OnTurnEnd,
    HookEventKind::TurnComplete,
];

/// A lifecycle event emitted by the Talos runtime.
#[derive(Debug)]
#[non_exhaustive]
pub enum HookEvent<'a> {
    /// The turn has started.
    TurnStart {
        /// Current turn identifier.
        turn_id: TurnId,
    },
    /// The system prompt has been assembled.
    OnSystemPromptBuilt {
        /// Final prompt text.
        prompt: &'a str,
    },
    /// Immediately before invoking the provider.
    BeforeProviderCall {
        /// Messages sent to the provider.
        messages: &'a [Message],
    },
    /// After the provider stream has completed.
    AfterProviderCall {
        /// Input tokens consumed.
        tokens_in: u32,
        /// Output tokens produced.
        tokens_out: u32,
    },
    /// A text delta was observed.
    OnTextDelta {
        /// Delta text.
        text: &'a str,
    },
    /// A tool call was proposed by the model.
    OnToolCallProposed {
        /// Proposed tool call.
        call: &'a ToolCall,
    },
    /// Immediately before executing a batch of tool calls.
    BeforeToolBatch {
        /// Tool calls to execute.
        calls: &'a [ToolCall],
    },
    /// Immediately before evaluating permissions for a tool call.
    BeforePermissionCheck {
        /// Tool call being evaluated.
        call: &'a ToolCall,
    },
    /// Immediately after evaluating permissions for a tool call.
    AfterPermissionCheck {
        /// Tool call being evaluated.
        call: &'a ToolCall,
        /// Permission decision returned by the engine.
        decision: PermissionDecision,
    },
    /// Immediately before executing a bash command in the sandbox.
    BeforeBashSandboxExec {
        /// Bash command string.
        command: &'a str,
    },
    /// Immediately after sandboxed bash execution completes.
    AfterBashSandboxExec {
        /// Process exit code.
        exit: i32,
        /// Total execution duration.
        duration: Duration,
    },
    /// Immediately before invoking a tool.
    BeforeToolCall {
        /// Tool call being executed.
        call: &'a ToolCall,
    },
    /// Immediately after invoking a tool.
    AfterToolCall {
        /// Tool call that was executed.
        call: &'a ToolCall,
        /// Tool result returned by the tool.
        result: &'a ToolResult,
    },
    /// A tool result was observed and added back into the conversation.
    OnToolResultObserved {
        /// Owned observation payload.
        observation: &'a ToolObservation,
    },
    /// After a batch of tool calls has completed.
    AfterToolBatch {
        /// Tool results in input order.
        results: &'a [ToolResult],
    },
    /// Doom-loop detection fired.
    OnDoomLoopDetected {
        /// Doom-loop signature string.
        signature: &'a str,
    },
    /// A runtime budget was exceeded.
    OnBudgetExceeded {
        /// Budget category.
        kind: BudgetKind,
        /// Amount used.
        used: u64,
        /// Configured limit.
        limit: u64,
    },
    /// A provider error occurred.
    OnProviderError {
        /// Provider error instance.
        error: &'a ProviderError,
    },
    /// The provider signaled turn end.
    OnTurnEnd {
        /// Turn-end reason.
        reason: TurnEndReason,
    },
    /// The turn completed.
    TurnComplete {
        /// Current turn identifier.
        turn_id: TurnId,
        /// Final turn status.
        status: TurnStatus,
    },
}

impl HookEvent<'_> {
    /// Returns the discriminant used for subscription and pre-filtering.
    #[must_use]
    pub fn kind(&self) -> HookEventKind {
        match self {
            Self::TurnStart { .. } => HookEventKind::TurnStart,
            Self::OnSystemPromptBuilt { .. } => HookEventKind::OnSystemPromptBuilt,
            Self::BeforeProviderCall { .. } => HookEventKind::BeforeProviderCall,
            Self::AfterProviderCall { .. } => HookEventKind::AfterProviderCall,
            Self::OnTextDelta { .. } => HookEventKind::OnTextDelta,
            Self::OnToolCallProposed { .. } => HookEventKind::OnToolCallProposed,
            Self::BeforeToolBatch { .. } => HookEventKind::BeforeToolBatch,
            Self::BeforePermissionCheck { .. } => HookEventKind::BeforePermissionCheck,
            Self::AfterPermissionCheck { .. } => HookEventKind::AfterPermissionCheck,
            Self::BeforeBashSandboxExec { .. } => HookEventKind::BeforeBashSandboxExec,
            Self::AfterBashSandboxExec { .. } => HookEventKind::AfterBashSandboxExec,
            Self::BeforeToolCall { .. } => HookEventKind::BeforeToolCall,
            Self::AfterToolCall { .. } => HookEventKind::AfterToolCall,
            Self::OnToolResultObserved { .. } => HookEventKind::OnToolResultObserved,
            Self::AfterToolBatch { .. } => HookEventKind::AfterToolBatch,
            Self::OnDoomLoopDetected { .. } => HookEventKind::OnDoomLoopDetected,
            Self::OnBudgetExceeded { .. } => HookEventKind::OnBudgetExceeded,
            Self::OnProviderError { .. } => HookEventKind::OnProviderError,
            Self::OnTurnEnd { .. } => HookEventKind::OnTurnEnd,
            Self::TurnComplete { .. } => HookEventKind::TurnComplete,
        }
    }

    /// Returns whether this event is inside the permission read-only boundary.
    #[must_use]
    pub fn is_permission_boundary(&self) -> bool {
        matches!(
            self,
            Self::BeforePermissionCheck { .. } | Self::AfterPermissionCheck { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trip() {
        let turn_id = TurnId::new();
        let event = HookEvent::TurnStart { turn_id };
        assert_eq!(event.kind(), HookEventKind::TurnStart);
        assert_eq!(ALL_HOOK_EVENT_KINDS.len(), 20);
    }
}
