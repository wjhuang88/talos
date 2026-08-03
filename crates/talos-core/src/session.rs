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
    SubmitStructuredTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Reconcile a sent batch without creating a second execution authority.
    ReconcileStructured { submission: StructuredSubmission },
    /// Reconcile a batch and receive the canonical durable result directly.
    ///
    /// Reconciliation is observational: it never grants execution authority or
    /// resumes paused work, including for an older Actor generation.
    ReconcileStructuredTracked {
        submission: StructuredSubmission,
        #[serde(skip)]
        receipt_tx: Option<mpsc::UnboundedSender<SubmissionReceipt>>,
    },
    /// Replace the model-visible activated Skill context.
    SetSkillContext {
        name: Option<String>,
        content: Option<String>,
    },
    /// Interrupt the current turn through the legacy compatibility path.
    Interrupt,
    /// Interrupt exactly one Actor generation and one active structured Turn.
    InterruptTurn {
        session_generation: u64,
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
    /// Rejection before durable Actor custody transfers.
    SubmissionRejected {
        session_id: String,
        submission_id: String,
        sender_generation: u64,
        reason: SubmissionRejectionReason,
    },
    /// A durably accepted submission could not start and remains recoverable.
    SubmissionPaused {
        session_id: String,
        session_generation: u64,
        submission_id: String,
        receipt_id: String,
        reason: SubmissionRejectionReason,
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
    AgentEvent { event: AgentEvent },
    ApprovalRequired {
        tool_name: String,
        arguments: String,
        call_id: String,
    },
    TurnStarted { turn_id: String },
    TurnCompleted {
        turn_id: String,
        status: TurnCompletionStatus,
    },
    Error { message: String },
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
    Error { message: String },
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
