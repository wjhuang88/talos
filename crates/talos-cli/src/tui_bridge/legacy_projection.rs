use talos_conversation::{ConversationEngine, UiOutput};
use talos_core::message::AgentEvent;
use talos_core::session::{TurnCompletionStatus, TurnEventPayload};

use super::{BridgeTurnState, ProgressMode, emit_bridge_error};

pub(super) fn handle_legacy_turn_event(
    engine: &mut ConversationEngine,
    turn_state: &mut BridgeTurnState,
    ui_tx: &tokio::sync::mpsc::UnboundedSender<UiOutput>,
    session_id: String,
    turn_id: String,
    sequence: u64,
    payload: TurnEventPayload,
) -> bool {
    if matches!(
        turn_state,
        BridgeTurnState::StructuredRunning { .. } | BridgeTurnState::StructuredCancelling { .. }
    ) {
        return handle_structured_legacy_projection(
            engine, turn_state, ui_tx, session_id, turn_id, sequence, payload,
        );
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
            };
            false
        }
        BridgeTurnState::LegacyRunning {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
        }
        | BridgeTurnState::LegacyCancelling {
            session_id: expected_session,
            turn_id: expected_turn,
            mut next_sequence,
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
                    }
                } else {
                    BridgeTurnState::LegacyRunning {
                        session_id: expected_session,
                        turn_id: expected_turn,
                        next_sequence,
                    }
                };
                emit_bridge_error(ui_tx, "ignored stale or out-of-order legacy session event");
                return false;
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
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
                TurnEventPayload::Progress {
                    event: AgentEvent::Error { .. },
                } => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
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
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
                TurnEventPayload::Completed { status } => {
                    for output in engine.handle_turn_completed(&status) {
                        let _ = ui_tx.send(output);
                    }
                    let success = matches!(status, TurnCompletionStatus::Success { .. });
                    *turn_state = if success {
                        BridgeTurnState::Idle
                    } else {
                        BridgeTurnState::PausedAfterFailure
                    };
                    let _ = ui_tx.send(UiOutput::SteeringQueueSnapshot(
                        engine.steering_queue_snapshot(),
                    ));
                    success
                }
                _ => {
                    *turn_state = if cancelling {
                        BridgeTurnState::LegacyCancelling {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    } else {
                        BridgeTurnState::LegacyRunning {
                            session_id,
                            turn_id,
                            next_sequence,
                        }
                    };
                    false
                }
            }
        }
        other => {
            *turn_state = other;
            false
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
