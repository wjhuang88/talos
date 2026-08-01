//! Session protocol types for the AppServerSession seam (ADR-005).
//!
//! SQ (Submission Queue): bounded `mpsc::Sender<SessionOp>` (cap=512) for commands TO the session actor.
//! EQ (Event Queue): unbounded `mpsc::UnboundedSender<SessionEvent>` for events FROM the session actor.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::message::{AgentEvent, Message};
use crate::submission::{
    StructuredSubmission, SubmissionReceiptDisposition, SubmissionSource,
};

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
    /// Submit an immutable source-aware batch through the transactional Actor boundary.
    SubmitStructured { submission: StructuredSubmission },
    /// Reconcile a sent batch without creating a second execution authority.
    ReconcileStructured { submission: StructuredSubmission },
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
    /// Durable result of structured submission acceptance or reconciliation.
    SubmissionReceipt {
        /// Logical Session addressed by the operation.
        session_id: String,
        /// Runtime generation that owns the receipt.
        session_generation: u64,
        /// Stable batch identity.
        submission_id: String,
        /// Exact Engine reservation identity.
        reservation_id: String,
        /// Durable receipt identity, empty only for NotAccepted/rejection.
        receipt_id: String,
        /// Source used by Actor arbitration.
        source: SubmissionSource,
        /// Number of original recoverable items.
        item_count: usize,
        /// Aggregate UTF-8 text bytes; content is never included.
        total_text_bytes: usize,
        /// Durable, content-free receipt disposition.
        disposition: SubmissionReceiptDisposition,
    },
    /// Generation-aware canonical lifecycle for a structured Actor-owned Turn.
    StructuredTurnEvent {
        /// Logical Session identity.
        session_id: String,
        /// Runtime generation that owns the Turn.
        session_generation: u64,
        /// Actor-owned batch identity.
        submission_id: String,
        /// Durable receipt identity.
        receipt_id: String,
        /// Stable actor-local Turn identity.
        turn_id: String,
        /// Monotonic sequence within this Turn, starting at zero.
        sequence: u64,
        /// Ordered lifecycle payload.
        payload: TurnEventPayload,
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
    /// Canonical ordered event for one legacy user turn.
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

/// Ordered payload carried by canonical Turn events.
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
mod tests {
    use super::*;
    use crate::submission::{
        PendingSubmissionState, SubmissionItem, SubmissionKind, SubmissionRejectionReason,
    };

    fn structured_submission() -> StructuredSubmission {
        StructuredSubmission {
            batch_id: "batch_1".into(),
            reservation_id: "reservation_1".into(),
            transfer_attempt_id: "attempt_1".into(),
            session_id: "session_1".into(),
            session_generation: 7,
            source: SubmissionSource::User,
            items: vec![SubmissionItem {
                item_id: "item_1".into(),
                enqueue_sequence: 1,
                kind: SubmissionKind::UserTurn,
                text: "structured".into(),
                attachments: Vec::new(),
            }],
        }
    }

    #[test]
    fn session_op_serde_roundtrip() {
        let submission = structured_submission();
        let ops = vec![
            SessionOp::Submit {
                message: "hello".into(),
            },
            SessionOp::PreviewRequest {
                message: "diagnostic".into(),
            },
            SessionOp::SubmitStructured {
                submission: submission.clone(),
            },
            SessionOp::ReconcileStructured { submission },
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
            SessionEvent::SubmissionReceipt {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                reservation_id: "reservation_1".into(),
                receipt_id: "receipt_1".into(),
                source: SubmissionSource::User,
                item_count: 1,
                total_text_bytes: 10,
                disposition: SubmissionReceiptDisposition::AlreadyAccepted {
                    state: PendingSubmissionState::AcceptedPending,
                    turn_id: None,
                },
            },
            SessionEvent::StructuredTurnEvent {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                receipt_id: "receipt_1".into(),
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
    fn rejected_receipt_is_content_free() {
        let event = SessionEvent::SubmissionReceipt {
            session_id: "session_1".into(),
            session_generation: 7,
            submission_id: "batch_1".into(),
            reservation_id: "reservation_1".into(),
            receipt_id: String::new(),
            source: SubmissionSource::User,
            item_count: 1,
            total_text_bytes: 10,
            disposition: SubmissionReceiptDisposition::Rejected {
                reason: SubmissionRejectionReason::LimitExceeded,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.contains("structured"));
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
