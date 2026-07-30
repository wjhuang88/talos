//! Session protocol types for the AppServerSession seam (ADR-005).
//!
//! SQ (Submission Queue): bounded `mpsc::Sender<SessionOp>` (cap=512) for commands TO the session actor.
//! EQ (Event Queue): unbounded `mpsc::UnboundedSender<SessionEvent>` for events FROM the session actor.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::message::AgentEvent;
use crate::message::ContentPart;
use crate::message::Message;

/// Maximum UTF-8 bytes accepted for one structured submission item (ADR-056).
pub const MAX_SUBMISSION_ITEM_BYTES: usize = 64 * 1024;
/// Maximum items retained in one interactive steering queue (ADR-056).
pub const MAX_STEERING_QUEUE_ITEMS: usize = 128;
/// Maximum UTF-8 text bytes retained in one interactive steering queue (ADR-056).
pub const MAX_STEERING_QUEUE_BYTES: usize = 1024 * 1024;
/// Maximum compatible items projected into one actor turn (ADR-056).
pub const MAX_SUBMISSION_BATCH_ITEMS: usize = 32;
/// Maximum UTF-8 text bytes projected into one actor turn (ADR-056).
pub const MAX_SUBMISSION_BATCH_BYTES: usize = 256 * 1024;

/// Origin of a structured session submission (ADR-056).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionSource {
    /// Interactive user input accepted by a product bridge.
    User,
    /// A scheduled follow-up produced by the session scheduler.
    Scheduler,
    /// A legacy or external caller using the compatibility operations.
    Compatibility,
}

/// Dispatch semantics fixed before an item enters a queue (ADR-056).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionKind {
    /// A normal model-visible user turn.
    UserTurn,
    /// A request-preview diagnostic that must not call the provider.
    PreviewRequest,
}

/// One recoverable item inside a structured session submission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionItem {
    /// Opaque producer-assigned item identity.
    pub id: String,
    /// Monotonic order assigned by the producer within its source domain.
    pub enqueue_sequence: u64,
    /// Dispatch kind fixed before queue insertion.
    pub kind: SubmissionKind,
    /// Original text without delimiter rewriting.
    pub text: String,
    /// Image parts bound to this item before queue insertion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<ContentPart>,
}

impl SubmissionItem {
    /// Returns the original UTF-8 text size used by queue and batch budgets.
    #[must_use]
    pub fn text_bytes(&self) -> usize {
        self.text.len()
    }
}

/// One actor-owned submission containing a bounded ordered item batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredSubmission {
    /// Opaque producer-assigned batch identity used for acknowledgement.
    pub id: String,
    /// Producer/source used by actor arbitration and diagnostics.
    pub source: SubmissionSource,
    /// Ordered recoverable items. A batch never mixes dispatch kinds.
    pub items: Vec<SubmissionItem>,
}

impl StructuredSubmission {
    /// Returns the aggregate UTF-8 text size without inspecting user content.
    #[must_use]
    pub fn total_text_bytes(&self) -> usize {
        self.items.iter().map(SubmissionItem::text_bytes).sum()
    }

    /// Returns the common dispatch kind when the batch is non-empty and homogeneous.
    #[must_use]
    pub fn common_kind(&self) -> Option<SubmissionKind> {
        let first = self.items.first()?.kind;
        self.items
            .iter()
            .all(|item| item.kind == first)
            .then_some(first)
    }
}

/// Commands sent to the session actor via the bounded SQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionOp {
    /// Submit a user message for the agent to process.
    Submit { message: String },
    /// Submit a user message with image attachments (MODEL-009-D/I152).
    /// The session actor constructs `Message::Multimodal` from these parts.
    SubmitMultimodal {
        text: String,
        attachments: Vec<crate::message::ContentPart>,
    },
    /// Build a provider request preview for diagnostics without calling the provider.
    PreviewRequest { message: String },
    /// Submit a source-aware, recoverable item batch (ADR-056 / TUI-041).
    ///
    /// Legacy Submit variants remain supported and are normalized by the
    /// session actor as one-item compatibility submissions.
    SubmitStructured { submission: StructuredSubmission },
    /// Replace the model-visible activated Skill context.
    ///
    /// The CLI/runtime layer is responsible for validating paths and budgets
    /// before sending this operation. The session actor only updates prompt
    /// state and invalidates the agent's stable prompt prefix.
    SetSkillContext {
        /// Active Skill name, or `None` to clear activation.
        name: Option<String>,
        /// Bounded Skill body/reference content, or `None` to clear activation.
        content: Option<String>,
    },
    /// Interrupt the current turn.
    Interrupt,
    /// Shut down the session actor.
    Shutdown,
}

/// Events emitted by the session actor on the unbounded EQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionEvent {
    /// The actor accepted ownership of one structured submission from the SQ.
    SubmissionQueued {
        /// Session that accepted the submission.
        session_id: String,
        /// Opaque producer-assigned submission identity.
        submission_id: String,
        /// Source used by actor arbitration.
        source: SubmissionSource,
        /// Number of original recoverable items.
        item_count: usize,
        /// Aggregate text bytes; user content is never included.
        total_text_bytes: usize,
    },
    /// The actor correlated an accepted structured submission to a new turn.
    SubmissionStarted {
        /// Session that owns the turn.
        session_id: String,
        /// Submission that started.
        submission_id: String,
        /// Canonical actor turn identity.
        turn_id: String,
    },
    /// The actor rejected a structured submission before starting a turn.
    SubmissionRejected {
        /// Session that rejected the submission.
        session_id: String,
        /// Submission that was rejected.
        submission_id: String,
        /// Bounded, content-free rejection reason.
        reason: SubmissionRejectionReason,
    },
    /// A durable embedded session has atomically committed a completed turn.
    ///
    /// Emitted only after durable storage reports success. Existing unbound
    /// runtimes never emit this event.
    EntriesCommitted {
        /// UUID-backed durable session identity.
        session_id: String,
        /// Idempotency identity of the committed turn.
        turn_id: String,
        /// Stable persisted entry IDs in transcript order.
        entry_ids: Vec<String>,
    },
    /// Canonical ordered event for one user turn.
    ///
    /// In-tree runtime consumers must use this envelope instead of inferring
    /// user-turn lifecycle from provider-level [`AgentEvent::TurnStart`] or
    /// [`AgentEvent::TurnEnd`](crate::message::AgentEvent::TurnEnd).
    TurnEvent {
        /// Stable durable session UUID or process-local runtime identity.
        session_id: String,
        /// Stable actor-local identifier for the user turn.
        turn_id: String,
        /// Monotonic sequence within `turn_id`, starting at zero.
        sequence: u64,
        /// Ordered turn payload.
        payload: TurnEventPayload,
    },
    /// An agent event (text delta, tool call, etc.) from the current turn.
    AgentEvent {
        /// The inner streaming agent event.
        event: AgentEvent,
    },
    /// A tool requires user approval. The consumer must respond via the approval channel.
    ApprovalRequired {
        tool_name: String,
        arguments: String,
        call_id: String,
    },
    /// A new turn has started.
    TurnStarted { turn_id: String },
    /// A turn has completed.
    TurnCompleted {
        turn_id: String,
        status: TurnCompletionStatus,
    },
    /// A session-level error.
    Error { message: String },
}

/// Content-free reason a structured submission could not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionRejectionReason {
    /// The submission was empty or mixed incompatible dispatch kinds.
    InvalidStructure,
    /// An item or batch exceeded a hard byte/item budget.
    LimitExceeded,
    /// Compacted history plus input still exceeded the model context budget.
    ContextBudgetExceeded,
    /// The actor was shutting down before the submission could start.
    SessionClosed,
}

/// Ordered payload carried by [`SessionEvent::TurnEvent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TurnEventPayload {
    /// The session actor accepted and started the user turn.
    Started,
    /// Streaming progress produced while executing the turn.
    Progress {
        /// Provider/agent progress event. Its `TurnStart`/`TurnEnd` variants
        /// delimit provider responses, not the enclosing user turn.
        event: AgentEvent,
    },
    /// The whole user turn completed.
    Completed {
        /// Authoritative terminal status and message sequence.
        status: TurnCompletionStatus,
    },
}

/// Status of a completed turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TurnCompletionStatus {
    /// Turn completed normally.
    Success {
        /// The final assistant response text.
        #[serde(default)]
        final_text: String,
        /// Messages produced during this turn, in chronological order.
        /// This is the authoritative sequence for persistence/replay.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        new_messages: Vec<crate::message::Message>,
    },
    /// Turn was cancelled by user interrupt.
    Cancelled,
    /// Turn ended with an error.
    Error {
        /// Error message.
        message: String,
    },
}

/// Handle returned to the UI layer for interacting with a session.
///
/// The UI sends commands via `sq_tx` and receives events via `eq_rx`.
pub struct SessionHandle {
    /// Bounded submission queue sender (cap=512).
    pub sq_tx: mpsc::Sender<SessionOp>,
    /// Unbounded event queue receiver.
    pub eq_rx: mpsc::UnboundedReceiver<SessionEvent>,
}

/// Configuration for creating a session actor.
///
/// Captures CLI-layer decisions that the session actor needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Product-neutral runtime policy for the session actor.
    #[serde(default)]
    pub runtime_policy: RuntimePolicy,
    /// Workspace root path for file operations.
    pub workspace_root: PathBuf,
    /// Prior conversation messages to include in the first turn.
    #[serde(default)]
    pub initial_history: Vec<Message>,
    /// Model context token limit for compaction triggering.
    #[serde(default = "default_model_context_limit")]
    pub model_context_limit: u32,
}

/// Product-neutral policy for session runtime behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimePolicy {
    /// How the runtime should behave when a tool requests approval and no
    /// caller-specific approval handler handles it first.
    pub approval_mode: ApprovalMode,
}

impl RuntimePolicy {
    /// Interactive policy for UI-owned sessions.
    #[must_use]
    pub fn interactive() -> Self {
        Self {
            approval_mode: ApprovalMode::Interactive,
        }
    }

    /// Headless policy for non-interactive sessions that cannot ask a user.
    #[must_use]
    pub fn headless_deny() -> Self {
        Self {
            approval_mode: ApprovalMode::HeadlessDeny,
        }
    }
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self::interactive()
    }
}

/// Approval behavior for a session runtime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    /// Approval prompts may be surfaced by the product/UI layer.
    #[default]
    Interactive,
    /// Approval requests are denied because no user approval channel exists.
    HeadlessDeny,
}

fn default_model_context_limit() -> u32 {
    128_000
}

#[cfg(test)]
#[allow(warnings)]
#[allow(warnings)]
#[allow(warnings)]
#[allow(warnings)]
mod tests {
    use super::*;

    #[test]
    fn session_op_serde_roundtrip() {
        let ops = vec![
            SessionOp::Submit {
                message: "hello".into(),
            },
            SessionOp::PreviewRequest {
                message: "diagnostic".into(),
            },
            SessionOp::SubmitStructured {
                submission: StructuredSubmission {
                    id: "batch_1".into(),
                    source: SubmissionSource::User,
                    items: vec![SubmissionItem {
                        id: "item_1".into(),
                        enqueue_sequence: 1,
                        kind: SubmissionKind::UserTurn,
                        text: "structured".into(),
                        attachments: Vec::new(),
                    }],
                },
            },
            SessionOp::Interrupt,
            SessionOp::Shutdown,
        ];
        for op in &ops {
            let json = serde_json::to_string(op).unwrap();
            let back: SessionOp = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_value(op).unwrap(),
                serde_json::to_value(&back).unwrap()
            );
        }
    }

    #[test]
    fn session_event_serde_roundtrip() {
        let events = vec![
            SessionEvent::SubmissionQueued {
                session_id: "session_1".into(),
                submission_id: "batch_1".into(),
                source: SubmissionSource::User,
                item_count: 1,
                total_text_bytes: 10,
            },
            SessionEvent::SubmissionStarted {
                session_id: "session_1".into(),
                submission_id: "batch_1".into(),
                turn_id: "turn_1".into(),
            },
            SessionEvent::SubmissionRejected {
                session_id: "session_1".into(),
                submission_id: "batch_2".into(),
                reason: SubmissionRejectionReason::LimitExceeded,
            },
            SessionEvent::TurnEvent {
                session_id: "session_1".into(),
                turn_id: "turn_1".into(),
                sequence: 0,
                payload: TurnEventPayload::Started,
            },
            SessionEvent::TurnEvent {
                session_id: "session_1".into(),
                turn_id: "turn_1".into(),
                sequence: 1,
                payload: TurnEventPayload::Progress {
                    event: AgentEvent::TextDelta {
                        delta: "hello".into(),
                    },
                },
            },
            SessionEvent::TurnEvent {
                session_id: "session_1".into(),
                turn_id: "turn_1".into(),
                sequence: 2,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Success {
                        final_text: "hello".into(),
                        new_messages: vec![],
                    },
                },
            },
            SessionEvent::AgentEvent {
                event: AgentEvent::TextDelta {
                    delta: "hello".into(),
                },
            },
            SessionEvent::ApprovalRequired {
                tool_name: "write".into(),
                arguments: "{}".into(),
                call_id: "call_1".into(),
            },
            SessionEvent::TurnStarted {
                turn_id: "1".into(),
            },
            SessionEvent::TurnCompleted {
                turn_id: "1".into(),
                status: TurnCompletionStatus::Success {
                    final_text: String::new(),
                    new_messages: vec![],
                },
            },
            SessionEvent::TurnCompleted {
                turn_id: "2".into(),
                status: TurnCompletionStatus::Cancelled,
            },
            SessionEvent::TurnCompleted {
                turn_id: "3".into(),
                status: TurnCompletionStatus::Error {
                    message: "boom".into(),
                },
            },
            SessionEvent::Error {
                message: "fail".into(),
            },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let back: SessionEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_value(event).unwrap(),
                serde_json::to_value(&back).unwrap()
            );
        }
    }

    #[test]
    fn session_config_serde_roundtrip() {
        let config = SessionConfig {
            runtime_policy: RuntimePolicy::headless_deny(),
            workspace_root: PathBuf::from("/tmp/test"),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: SessionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.runtime_policy, back.runtime_policy);
        assert_eq!(config.workspace_root, back.workspace_root);
        assert_eq!(config.initial_history, back.initial_history);
        assert_eq!(config.model_context_limit, back.model_context_limit);
    }

    #[test]
    fn session_config_defaults_to_interactive_runtime_policy() {
        let json = r#"{
            "workspace_root": "/tmp/test",
            "initial_history": [],
            "model_context_limit": 128000
        }"#;
        let back: SessionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(back.runtime_policy, RuntimePolicy::interactive());
    }
}
