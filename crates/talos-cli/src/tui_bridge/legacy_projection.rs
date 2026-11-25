use talos_conversation::{ConversationEngine, UiOutput};
use talos_core::message::AgentEvent;
use talos_core::session::TurnEventPayload;

use super::{
    BridgeTurnState, ProgressMode, QueuedDispatch, completion_allows_queued_continuation,
    emit_bridge_error,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_legacy_turn_event(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
    current_sender_generation: u64,
) -> QueuedDispatch {
    if matches!(
        turn_state,
        BridgeTurnState::StructuredRunning { .. } | BridgeTurnState::StructuredCancelling { .. }
    ) {
        handle_structured_legacy_projection(
            engine, turn_state, ui_tx, session_id, turn_id, sequence, payload,
        );
        return QueuedDispatch::None;
    }

    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    let state_was_legacy_cancelling = matches!(&state, BridgeTurnState::LegacyCancelling { .. });
    match state {
        BridgeTurnState::Idle if sequence == 0 && matches!(payload, TurnEventPayload::Started) => {
            for output in engine.handle_turn_started() {
                let _ = ui_tx.send(output);
            }
            *turn_state = BridgeTurnState::LegacyRunning {
                session_id,
                turn_id,
                next_sequence: 1,
                sender_generation: current_sender_generation,
            };
            QueuedDispatch::None
        }
        BridgeTurnState::LegacyRunning {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
            sender_generation,
        }
        | BridgeTurnState::LegacyCancelling {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
            sender_generation,
        } => {
            let cancelling = state_was_legacy_cancelling;
            if expected_session != session_id
                || expected_turn != turn_id
                || next_sequence != sequence
            {
                *turn_state = if cancelling {
                    BridgeTurnState::LegacyCancelling {
                        session_id: expected_session,
                        turn_id: expected_turn,
                        next_sequence,
                        sender_generation,
                    }
                } else {
                    BridgeTurnState::LegacyRunning {
                        session_id: expected_session,
                        turn_id: expected_turn,
                        next_sequence,
                        sender_generation,
                    }
                };
                emit_bridge_error(ui_tx, "ignored stale or out-of-order legacy session event");
                return QueuedDispatch::None;
            }
            next_sequence = next_sequence.saturating_add(1);
            match payload {
                TurnEventPayload::Started => {
                    for output in engine.handle_turn_started() {
                        let _ = ui_tx.send(output);
                    }
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    };
                    QueuedDispatch::None
                }
                TurnEventPayload::Progress {
                    event: AgentEvent::Error { .. },
                } => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    };
                    QueuedDispatch::None
                }
                TurnEventPayload::Progress { event } => {
                    for output in engine.handle_agent_event(&event) {
                        let _ = ui_tx.send(output);
                    }
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    };
                    QueuedDispatch::None
                }
                TurnEventPayload::Completed { status } => {
                    for output in engine.handle_turn_completed(&status) {
                        let _ = ui_tx.send(output);
                    }
                    let continuation_allowed =
                        completion_allows_queued_continuation(&status, cancelling);
                    *turn_state = if continuation_allowed {
                        BridgeTurnState::Idle
                    } else {
                        BridgeTurnState::PausedAfterFailure
                    };
                    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                        engine.steering_queue_snapshot(),
                    ));
                    if continuation_allowed {
                        QueuedDispatch::Generation(sender_generation)
                    } else {
                        QueuedDispatch::None
                    }
                }
                _ => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                            sender_generation,
                        }
                    };
                    QueuedDispatch::None
                }
            }
        }
        other => {
            *turn_state = other;
            QueuedDispatch::None
        }
    }
}

fn handle_structured_legacy_projection(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
) -> bool {
    let state = std::mem::replace(turn_state, BridgeTurnState::Idle);
    match state {
        BridgeTurnState::StructuredRunning {
            session_id: expected_session,
            session_generation,
            submission_id,
            receipt_id,
            turn_id: expected_turn,
            next_structured_sequence,
            mut next_legacy_sequence,
            mut progress_mode,
        } => {
            if matches!(
                payload,
                TurnEventPayload::Started | TurnEventPayload::Completed { .. }
            ) {
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                return false;
            }
            if expected_session != session_id
                || expected_turn != turn_id
                || next_legacy_sequence != sequence
            {
                *turn_state = BridgeTurnState::StructuredRunning {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                emit_bridge_error(ui_tx, "ignored stale structured compatibility progress");
                return false;
            }
            next_legacy_sequence = next_legacy_sequence.saturating_add(1);
            let should_emit = progress_mode != ProgressMode::Structured;
            if progress_mode == ProgressMode::Unknown {
                progress_mode = ProgressMode::Legacy;
            }
            if should_emit
                && let TurnEventPayload::Progress { event } = payload
                && !matches!(event, AgentEvent::Error { .. })
            {
                for output in engine.handle_agent_event(&event) {
                    let _ = ui_tx.send(output);
                }
            }
            *turn_state = BridgeTurnState::StructuredRunning {
                session_id: expected_session,
                session_generation,
                submission_id,
                receipt_id,
                turn_id: expected_turn,
                next_structured_sequence,
                next_legacy_sequence,
                progress_mode,
            };
            false
        }
        BridgeTurnState::StructuredCancelling {
            session_id: expected_session,
            session_generation,
            submission_id,
            receipt_id,
            turn_id: expected_turn,
            next_structured_sequence,
            mut next_legacy_sequence,
            mut progress_mode,
        } => {
            if matches!(
                payload,
                TurnEventPayload::Started | TurnEventPayload::Completed { .. }
            ) {
                *turn_state = BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                return false;
            }
            if expected_session != session_id
                || expected_turn != turn_id
                || next_legacy_sequence != sequence
            {
                *turn_state = BridgeTurnState::StructuredCancelling {
                    session_id: expected_session,
                    session_generation,
                    submission_id,
                    receipt_id,
                    turn_id: expected_turn,
                    next_structured_sequence,
                    next_legacy_sequence,
                    progress_mode,
                };
                emit_bridge_error(ui_tx, "ignored stale structured compatibility progress");
                return false;
            }
            next_legacy_sequence = next_legacy_sequence.saturating_add(1);
            let should_emit = progress_mode != ProgressMode::Structured;
            if progress_mode == ProgressMode::Unknown {
                progress_mode = ProgressMode::Legacy;
            }
            if should_emit
                && let TurnEventPayload::Progress { event } = payload
                && !matches!(event, AgentEvent::Error { .. })
            {
                for output in engine.handle_agent_event(&event) {
                    let _ = ui_tx.send(output);
                }
            }
            *turn_state = BridgeTurnState::StructuredCancelling {
                session_id: expected_session,
                session_generation,
                submission_id,
                receipt_id,
                turn_id: expected_turn,
                next_structured_sequence,
                next_legacy_sequence,
                progress_mode,
            };
            false
        }
        other => {
            *turn_state = other;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::session::TurnCompletionStatus;

    #[test]
    fn progress_error_keeps_processing_and_queue_until_authoritative_failure() {
        for cancelling in [false, true] {
            let mut engine = ConversationEngine::new("model".into(), "provider".into());
            engine.handle_turn_started();
            engine.enqueue_steering("queued original text 中文".into());
            let mut state = if cancelling {
                BridgeTurnState::LegacyCancelling {
                    session_id: "session".into(),
                    turn_id: "turn".into(),
                    next_sequence: 1,
                    sender_generation: 7,
                }
            } else {
                BridgeTurnState::LegacyRunning {
                    session_id: "session".into(),
                    turn_id: "turn".into(),
                    next_sequence: 1,
                    sender_generation: 7,
                }
            };
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let dispatch = handle_legacy_turn_event(
                &mut engine,
                &mut state,
                &tx,
                "session".into(),
                "turn".into(),
                1,
                TurnEventPayload::Progress {
                    event: AgentEvent::Error {
                        message: "provider failed".into(),
                    },
                },
                7,
            );
            assert!(matches!(dispatch, QueuedDispatch::None));
            assert!(engine.status_snapshot().is_processing);
            assert!(
                rx.try_recv().is_err(),
                "no premature terminal/error projection"
            );
            assert_eq!(
                engine.steering_queue_snapshot().entries[0].text,
                "queued original text 中文"
            );

            let dispatch = handle_legacy_turn_event(
                &mut engine,
                &mut state,
                &tx,
                "session".into(),
                "turn".into(),
                2,
                TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Error {
                        message: "provider failed".into(),
                    },
                },
                7,
            );
            assert!(matches!(dispatch, QueuedDispatch::None));
            assert!(matches!(state, BridgeTurnState::PausedAfterFailure));
            assert!(!engine.status_snapshot().is_processing);
            assert_eq!(
                engine.steering_queue_snapshot().entries[0].text,
                "queued original text 中文"
            );
            let mut terminal_statuses = 0;
            let mut error_blocks = 0;
            while let Ok(output) = rx.try_recv() {
                match output {
                    UiOutput::Status(status) if !status.is_processing => terminal_statuses += 1,
                    UiOutput::Content(talos_conversation::ContentOutput::Block {
                        source: talos_conversation::MessageSource::Error,
                        ..
                    }) => error_blocks += 1,
                    _ => {}
                }
            }
            assert_eq!(terminal_statuses, 1);
            assert_eq!(error_blocks, 1);
        }
    }
}
