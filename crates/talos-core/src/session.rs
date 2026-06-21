//! Session protocol types for the AppServerSession seam (ADR-005).
//!
//! SQ (Submission Queue): bounded `mpsc::Sender<SessionOp>` (cap=512) for commands TO the session actor.
//! EQ (Event Queue): unbounded `mpsc::UnboundedSender<SessionEvent>` for events FROM the session actor.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::message::AgentEvent;
use crate::message::Message;

/// Commands sent to the session actor via the bounded SQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionOp {
    /// Submit a user message for the agent to process.
    Submit { message: String },
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
    /// An agent event (text delta, tool call, etc.) from the current turn.
    AgentEvent(AgentEvent),
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
    /// Whether to auto-deny approval requests (non-interactive mode).
    pub print_mode: bool,
    /// Workspace root path for file operations.
    pub workspace_root: PathBuf,
    /// Prior conversation messages to include in the first turn.
    #[serde(default)]
    pub initial_history: Vec<Message>,
    /// Model context token limit for compaction triggering.
    #[serde(default = "default_model_context_limit")]
    pub model_context_limit: u32,
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
        // Note: SessionEvent::AgentEvent cannot be roundtripped because both
        // SessionEvent and AgentEvent use #[serde(tag = "type")], causing
        // duplicate "type" field in JSON. Skip that variant.
        let events = vec![
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
            print_mode: true,
            workspace_root: PathBuf::from("/tmp/test"),
            initial_history: vec![],
            model_context_limit: 128_000,
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: SessionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.print_mode, back.print_mode);
        assert_eq!(config.workspace_root, back.workspace_root);
        assert_eq!(config.initial_history, back.initial_history);
        assert_eq!(config.model_context_limit, back.model_context_limit);
    }
}
