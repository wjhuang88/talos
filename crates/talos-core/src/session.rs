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
    Submit {
        message: String,
    },
    SubmitMultimodal {
        text: String,
        attachments: Vec<crate::message::ContentPart>,
    },
    PreviewRequest {
        message: String,
    },
    SubmitStructured {
        submission: StructuredSubmission,
    },
    SubmitStructuredTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Stable public compatibility operation.
    ReconcileStructured {
        submission: StructuredSubmission,
    },
    /// Stable public compatibility operation with a direct receipt channel.
    ReconcileStructuredTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Transitional additive alias routed to the same observational reconcile path.
    SubmitStructuredReconcile {
        submission: StructuredSubmission,
    },
    /// Transitional additive alias routed to the same observational reconcile path.
    SubmitStructuredReconcileTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    SetSkillContext {
        name: Option<String>,
        content: Option<String>,
    },
    /// Legacy unqualified compatibility interrupt.
    Interrupt,
    /// Generation- and Turn-targeted interrupt for new interactive paths.
    InterruptTurn {
        session_generation: u64,
        turn_id: String,
    },
    /// Explicitly terminalizes one durably accepted submission that paused
    /// before Provider execution. The immutable submission is never replayed
    /// or rewritten; only the exact current generation may resolve it.
    CancelPausedSubmission {
        session_generation: u64,
        submission_id: String,
    },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionEvent {
    /// One live-session background job reached its unique terminal state.
    /// This event is not persisted and never starts a provider turn.
    BackgroundJobTerminal {
        session_id: String,
        session_generation: u64,
        summary: crate::background_job::BackgroundJobTerminalSummary,
    },
    SubmissionQueued {
        session_id: String,
        submission_id: String,
        sender_generation: u64,
        source: SubmissionSource,
        item_count: usize,
        total_text_bytes: usize,
    },
    SubmissionStarted {
        session_id: String,
        submission_id: String,
        sender_generation: u64,
        turn_id: String,
    },
    /// Authoritative model-visible projection emitted immediately before a
    /// structured Turn starts. The Actor, not the receipt observer, defines
    /// the visible ordering when retained work precedes a new resume item.
    StructuredSubmissionStarted {
        session_id: String,
        session_generation: u64,
        submission: StructuredSubmission,
        receipt_id: String,
        turn_id: String,
    },
    /// A durably accepted steering submission was committed to the continuation
    /// context of a running structured Turn at a complete tool-call boundary.
    /// Cancellation can still prevent the subsequent Provider request.
    StructuredSubmissionInjected {
        session_id: String,
        session_generation: u64,
        submission: StructuredSubmission,
        receipt_id: String,
        turn_id: String,
    },
    SubmissionRejected {
        session_id: String,
        submission_id: String,
        sender_generation: u64,
        reason: SubmissionRejectionReason,
    },
    SubmissionPaused {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        reason: SubmissionRejectionReason,
    },
    /// A durable submission was resolved without Provider execution: a
    /// pre-start cancellation, or an unacknowledged boundary handoff failure.
    SubmissionResolved {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        state: PendingSubmissionState,
    },
    SubmissionReceipt {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        reservation_id: String,
        receipt_id: String,
        source: SubmissionSource,
        item_count: usize,
        total_text_bytes: usize,
        disposition: SubmissionReceiptDisposition,
    },
    StructuredTurnEvent {
        session_id: String,
        session_generation: u64,
        source: SubmissionSource,
        submission_id: String,
        receipt_id: String,
        turn_id: String,
        sequence: u64,
        payload: TurnEventPayload,
    },
    EntriesCommitted {
        session_id: String,
        turn_id: String,
        entry_ids: Vec<String>,
    },
    TurnEvent {
        session_id: String,
        turn_id: String,
        sequence: u64,
        payload: TurnEventPayload,
    },
    AgentEvent {
        event: AgentEvent,
    },
    ApprovalRequired {
        tool_name: String,
        arguments: String,
        call_id: String,
    },
    TurnStarted {
        turn_id: String,
    },
    TurnCompleted {
        turn_id: String,
        status: TurnCompletionStatus,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TurnEventPayload {
    Started,
    Progress { event: AgentEvent },
    Completed { status: TurnCompletionStatus },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TurnCompletionStatus {
    Success {
        #[serde(default)]
        final_text: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        new_messages: Vec<crate::message::Message>,
    },
    Cancelled,
    Error {
        message: String,
    },
}

pub struct SessionHandle {
    pub sq_tx: mpsc::Sender<SessionOp>,
    pub eq_rx: mpsc::UnboundedReceiver<SessionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default)]
    pub runtime_policy: RuntimePolicy,
    pub workspace_root: PathBuf,
    #[serde(default)]
    pub initial_history: Vec<Message>,
    #[serde(default = "default_model_context_limit")]
    pub model_context_limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimePolicy {
    pub approval_mode: ApprovalMode,
}

impl RuntimePolicy {
    #[must_use]
    pub fn interactive() -> Self {
        Self {
            approval_mode: ApprovalMode::Interactive,
        }
    }

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    #[default]
    Interactive,
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
            SessionOp::ReconcileStructured {
                submission: submission.clone(),
            },
            SessionOp::ReconcileStructuredTracked {
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
            SessionOp::CancelPausedSubmission {
                session_generation: 7,
                submission_id: "batch_1".into(),
            },
            SessionOp::Shutdown,
        ];
        for op in &ops {
            let json = serde_json::to_string(op).expect("operation should succeed");
            let back: SessionOp = serde_json::from_str(&json).expect("operation should succeed");
            assert_eq!(
                serde_json::to_value(op).expect("operation should succeed"),
                serde_json::to_value(&back).expect("operation should succeed")
            );
        }
    }

    #[test]
    fn session_event_serde_roundtrip() {
        let submission = structured_submission();
        let events = vec![
            SessionEvent::SubmissionQueued {
                session_id: "session_1".into(),
                submission_id: "batch_1".into(),
                sender_generation: 7,
                source: SubmissionSource::User,
                item_count: 1,
                total_text_bytes: 10,
            },
            SessionEvent::SubmissionPaused {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                receipt_id: "receipt_1".into(),
                reason: SubmissionRejectionReason::ContextBudgetExceeded,
            },
            SessionEvent::SubmissionResolved {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                receipt_id: "receipt_1".into(),
                state: PendingSubmissionState::TerminalCancelled,
            },
            SessionEvent::SubmissionReceipt {
                session_id: "session_1".into(),
                session_generation: 7,
                submission_id: "batch_1".into(),
                reservation_id: "reservation:batch_1".into(),
                receipt_id: "receipt_1".into(),
                source: SubmissionSource::User,
                item_count: 1,
                total_text_bytes: 10,
                disposition: SubmissionReceiptDisposition::AlreadyAccepted {
                    state: PendingSubmissionState::AcceptedPending,
                    turn_id: None,
                },
            },
            SessionEvent::StructuredSubmissionStarted {
                session_id: "session_1".into(),
                session_generation: 7,
                submission: submission.clone(),
                receipt_id: "receipt_1".into(),
                turn_id: "turn_1".into(),
            },
            SessionEvent::StructuredTurnEvent {
                session_id: "session_1".into(),
                session_generation: 7,
                source: SubmissionSource::User,
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
            let json = serde_json::to_string(event).expect("operation should succeed");
            let back: SessionEvent = serde_json::from_str(&json).expect("operation should succeed");
            assert_eq!(
                serde_json::to_value(event).expect("operation should succeed"),
                serde_json::to_value(&back).expect("operation should succeed")
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
        let json = serde_json::to_string(&config).expect("operation should succeed");
        let back: SessionConfig = serde_json::from_str(&json).expect("operation should succeed");
        assert_eq!(config.runtime_policy, back.runtime_policy);
        assert_eq!(config.workspace_root, back.workspace_root);
        assert_eq!(config.initial_history, back.initial_history);
        assert_eq!(config.model_context_limit, back.model_context_limit);
    }
}
