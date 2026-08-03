//! Session protocol types for the AppServerSession seam (ADR-005).
//!
//! SQ (Submission Queue): bounded `mpsc::Sender<SessionOp>` (cap=512) for commands TO the session actor.
//! EQ (Event Queue): unbounded `mpsc::UnboundedSender<SessionEvent>` for events FROM the session actor.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::message::{AgentEvent, Message};
pub use crate::submission::{
    MAX_PENDING_SUBMISSION_BYTES, MAX_PENDING_SUBMISSIONS, MAX_STEERING_QUEUE_BYTES,
    MAX_STEERING_QUEUE_IMAGE_BYTES, MAX_STEERING_QUEUE_IMAGES, MAX_STEERING_QUEUE_ITEMS,
    MAX_SUBMISSION_BATCH_BYTES, MAX_SUBMISSION_BATCH_ITEMS, MAX_SUBMISSION_IMAGE_BYTES,
    MAX_SUBMISSION_IMAGE_COUNT, MAX_SUBMISSION_ITEM_BYTES, MAX_SUBMISSION_TOTAL_IMAGE_BYTES,
    PendingSubmissionState, StructuredSubmission, SubmissionItem, SubmissionKind,
    SubmissionReceipt, SubmissionReceiptDisposition, SubmissionRejectionReason, SubmissionSource,
};

/// Commands sent to the session actor via the bounded SQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionOp {
    /// Submit a user message for the agent to process.
    Submit { message: String },
    /// Submit a user message with image attachments (MODEL-009-D/I152).
    SubmitMultimodal {
        text: String,
        attachments: Vec<crate::message::ContentPart>,
    },
    /// Build a provider request preview without calling the provider.
    PreviewRequest { message: String },
    /// Submit an immutable source-aware batch through the Actor boundary.
    SubmitStructured { submission: StructuredSubmission },
    /// Submit a batch and receive the same canonical durable receipt projected on EQ.
    ///
    /// The sender is runtime-local and deliberately excluded from serialization.
    SubmitStructuredTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Reconcile a sent batch without creating a second execution authority.
    SubmitStructuredReconcile { submission: StructuredSubmission },
    /// Reconcile a batch and receive the canonical durable result directly.
    ///
    /// Reconciliation is observational: it never grants execution authority or
    /// resumes paused work, even when the immutable submission belongs to an
    /// older Actor generation.
    SubmitStructuredReconcileTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Replace the model-visible activated Skill context.
    SetSkillContext {
        /// Active Skill name, or `None` to clear activation.
        name: Option<String>,
        /// Bounded Skill body/reference content, or `None` to clear activation.
        content: Option<String>,
    },
    /// Interrupt the current turn through the legacy compatibility path.
    ///
    /// New interactive paths must use [`SessionOp::InterruptTurn`].
    Interrupt,
    /// Interrupt exactly one Actor generation and one active structured Turn.
    InterruptTurn {
        /// Authoritative Actor generation captured with the command sender.
        session_generation: u64,
        /// Exact active Turn identity observed from `StructuredTurnEvent::Started`.
        turn_id: String,
    },
    /// Shut down the session actor.
    Shutdown,
}

/// Events emitted by the session actor on the unbounded EQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionEvent {
    /// Compatibility projection emitted after a batch is accepted for Actor arbitration.
    SubmissionQueued {
        /// Session that accepted the submission.
        session_id: String,
        /// Stable submission identity.
        submission_id: String,
        /// Runtime generation echoed from the submission.
        sender_generation: u64,
        /// Source used by Actor arbitration.
        source: SubmissionSource,
        /// Number of original items.
        item_count: usize,
        /// Aggregate UTF-8 text bytes.
        total_text_bytes: usize,
    },
    /// Compatibility projection emitted when an accepted batch starts a Turn.
    SubmissionStarted {
        /// Session that owns the Turn.
        session_id: String,
        /// Stable submission identity.
        submission_id: String,
        /// Runtime generation echoed from the submission.
        sender_generation: u64,
        /// Canonical Turn identity.
        turn_id: String,
    },
    /// Rejection before durable Actor custody transfers.
    SubmissionRejected {
        /// Session that rejected the submission.
        session_id: String,
        /// Stable submission identity.
        submission_id: String,
        /// Runtime generation echoed from the submission.
        sender_generation: u64,
        /// Bounded content-free reason.
        reason: SubmissionRejectionReason,
    },
    /// A durably accepted submission could not start and remains paused/recoverable.
    SubmissionPaused {
        /// Session that retains custody.
        session_id: String,
        /// Authoritative Actor generation emitting the pause.
        session_generation: u64,
        /// Stable submission identity.
        submission_id: String,
        /// Durable receipt identity.
        receipt_id: String,
        /// Content-free pre-start failure reason.
        reason: SubmissionRejectionReason,
    },
    /// Durable result of structured submission acceptance or reconciliation.
    SubmissionReceipt {
        /// Logical Session addressed by the operation.
        session_id: String,
        /// Runtime generation that owns the receipt projection.
        session_generation: u64,
        /// Stable batch identity.
        submission_id: String,
        /// Exact frozen-prefix reservation identity.
        reservation_id: String,
        /// Durable receipt identity, empty only for NotAccepted/rejection.
        receipt_id: String,
        /// Source used by Actor arbitration.
        source: SubmissionSource,
        /// Number of original recoverable items.
        item_count: usize,
        /// Aggregate original UTF-8 text bytes.
        total_text_bytes: usize,
        /// Durable content-free disposition.
        disposition: SubmissionReceiptDisposition,
    },
    /// Generation-aware lifecycle for a structured Actor-owned Turn.
    StructuredTurnEvent {
        /// Logical Session identity.
        session_id: String,
        /// Runtime generation that owns the Turn.
        session_generation: u64,
        /// Actor-owned submission identity.
        submission_id: String,
        /// Durable receipt identity.
        receipt_id: String,
        /// Stable actor-local Turn identity.
        turn_id: String,
        /// Monotonic sequence within this Turn.
        sequence: u64,
        /// Ordered lifecycle payload.
        payload: TurnEventPayload,
    },
    /// A durable embedded session atomically committed a completed turn.
    EntriesCommitted {
        /// UUID-backed durable session identity.
        session_id: String,
        /// Idempotency identity of the committed turn.
        turn_id: String,
        /// Stable persisted entry IDs in transcript order.
        entry_ids: Vec<String>,
    },
    /// Canonical ordered event for one legacy user turn.
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
    /// An agent event from the current turn.
    AgentEvent { event: AgentEvent },
    /// A tool requires user approval.
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
    /// The Actor accepted and started the user Turn.
    Started,
    /// Streaming progress produced while executing the Turn.
    Progress { event: AgentEvent },
    /// The whole user Turn completed.
    Completed { status: TurnCompletionStatus },
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        new_messages: Vec<crate::message::Message>,
    },
    /// Turn was cancelled by user interrupt.
    Cancelled,
    /// Turn ended with an error.
    Error { message: String },
}

/// Handle returned to the UI layer for interacting with a session.
pub struct SessionHandle {
    /// Bounded submission queue sender (cap=512).
    pub sq_tx: mpsc::Sender<SessionOp>,
    /// Unbounded event queue receiver.
    pub eq_rx: mpsc::UnboundedReceiver<SessionEvent>,
}

/// Configuration for creating a session actor.
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
    /// Approval behavior when no caller-specific handler handles a request.
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

    /// Headless policy for non-interactive sessions.
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
    /// Approval requests are denied because no user channel exists.
    HeadlessDeny,
}

fn default_model_context_limit() -> u32 {
    128_000
}

#[cfg(test)]
#[allow(warnings)]
mod tests {
    use super::*;

    fn structured_submission() -> StructuredSubmission {
        StructuredSubmission {
            id: "batch_1".into(),
            source: SubmissionSource::User,
            sender_generation: 7,
            items: vec![SubmissionItem {
                id: "item_1".into(),
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
            SessionOp::SubmitStructuredTracked {
                submission: submission.clone(),
                receipt_tx: None,
            },
            SessionOp::SubmitStructuredReconcile {
                submission: submission.clone(),
            },
            SessionOp::SubmitStructuredReconcileTracked {
                submission,
                receipt_tx: None,
            },
            SessionOp::Interrupt,
            SessionOp::InterruptTurn {
                session_generation: 7,
                turn_id: "turn_7".into(),
            },
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
                sender_generation: 7,
                source: SubmissionSource::User,
                item_count: 1,
                total_text_bytes: 10,
            },
            SessionEvent::SubmissionStarted {
                session_id: "session_1".into(),
                submission_id: "batch_1".into(),
                sender_generation: 7,
                turn_id: "turn_1".into(),
            },
            SessionEvent::SubmissionRejected {
                session_id: "session_1".into(),
                submission_id: "batch_2".into(),
                sender_generation: 7,
                reason: SubmissionRejectionReason::LimitExceeded,
            },
            SessionEvent::SubmissionPaused {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                receipt_id: "receipt_1".into(),
                reason: SubmissionRejectionReason::ContextBudgetExceeded,
            },
            SessionEvent::SubmissionReceipt {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                reservation_id: "reservation_batch_1".into(),
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
            SessionEvent::TurnCompleted {
                turn_id: "2".into(),
                status: TurnCompletionStatus::Cancelled,
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
}
