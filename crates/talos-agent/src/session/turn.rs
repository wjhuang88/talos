use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use talos_core::message::{AgentEvent, Message};
use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};
use talos_session::{TurnTranscriptOutcome, TurnTranscriptOutcomeRecord};

use crate::{Agent, PreparedSessionTurn};

#[derive(Clone)]
pub(super) struct TurnPersistence {
    pub(super) session: talos_session::Session,
    pub(super) metadata: talos_session::SessionMetadata,
}

#[derive(Clone)]
pub(super) struct DurableTurnPersistence {
    pub(super) session: talos_session::DurableSession,
    pub(super) policy: talos_session::PersistencePolicy,
}

pub(super) struct TurnRecord {
    pub(super) new_messages: Vec<Message>,
    pub(super) status: TurnRecordStatus,
    pub(super) completion: TurnCompletionStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TurnRecordStatus {
    Success,
    Cancelled,
    Error,
}

pub(super) struct TurnForwarding {
    pub(super) agent: Arc<Agent>,
    pub(super) prepared: PreparedSessionTurn,
    pub(super) event_tx: mpsc::UnboundedSender<AgentEvent>,
    pub(super) event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    pub(super) eq_tx: mpsc::UnboundedSender<SessionEvent>,
    pub(super) cancel_token: CancellationToken,
    pub(super) turn_id: String,
    pub(super) session_id: String,
    pub(super) sequence: Arc<AtomicU64>,
    pub(super) persistence: Option<TurnPersistence>,
    pub(super) durable_persistence: Option<DurableTurnPersistence>,
    pub(super) result_tx: tokio::sync::oneshot::Sender<TurnRecord>,
}

pub(super) async fn run_turn_with_forwarding(turn: TurnForwarding) {
    let TurnForwarding {
        agent,
        prepared,
        event_tx,
        mut event_rx,
        eq_tx,
        cancel_token,
        turn_id,
        session_id,
        sequence,
        persistence,
        durable_persistence,
        result_tx,
    } = turn;

    let eq_tx_clone = eq_tx.clone();
    let cancel_clone = cancel_token.clone();
    let progress_sequence = sequence.clone();
    let progress_turn_id = turn_id.clone();
    let progress_session_id = session_id.clone();
    let raw_tool_outputs = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let progress_raw_tool_outputs = raw_tool_outputs.clone();
    let diagnostic_persistence = persistence.clone();
    let diagnostic_failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let progress_diagnostic_failures = diagnostic_failures.clone();
    let diagnostic_turn_id = turn_id.clone();

    let forwarder = tokio::spawn(async move {
        let mut response_ordinal = 0_u32;
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => break,
                event = event_rx.recv() => {
                    match event {
                        Some(event) => {
                            let mut forwarded_event = event.clone();
                            if matches!(event, AgentEvent::TurnEnd { .. } | AgentEvent::Error { .. }) {
                                response_ordinal = response_ordinal.saturating_add(1);
                                if let Some(persistence) = &diagnostic_persistence
                                    && let Some(diagnostic) = talos_session::ProviderTerminalDiagnostic::from_agent_event(
                                        &diagnostic_turn_id,
                                        response_ordinal,
                                        &event,
                                        persistence.metadata.provider.as_deref(),
                                        persistence.metadata.model.as_deref(),
                                    )
                                    && let Err(error) = persistence.session.append_terminal_diagnostic(&diagnostic)
                                {
                                    if let Ok(mut failures) = progress_diagnostic_failures.lock() {
                                        failures.push(error.to_string());
                                    }
                                    forwarded_event = AgentEvent::Error {
                                        message: "failed to persist provider terminal diagnostic".into(),
                                    };
                                }
                            }
                            if let AgentEvent::ToolResult { result } = &event
                                && let Ok(mut outputs) = progress_raw_tool_outputs.lock()
                            {
                                outputs.insert(
                                    result.tool_use_id.clone(),
                                    result.content.clone(),
                                );
                            }
                            let sequence = progress_sequence.fetch_add(1, Ordering::Relaxed);
                            let _ = eq_tx.send(SessionEvent::TurnEvent {
                                session_id: progress_session_id.clone(),
                                turn_id: progress_turn_id.clone(),
                                sequence,
                                payload: TurnEventPayload::Progress {
                                    event: forwarded_event,
                                },
                            });
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let mut agent_task =
        tokio::spawn(async move { agent.run_prepared_session_turn(prepared, event_tx).await });

    let agent_result = tokio::select! {
        result = &mut agent_task => result,
        _ = cancel_token.cancelled() => {
            agent_task.abort();
            let _ = forwarder.await;
            if let Some(persistence) = &persistence
                && let Err(message) = persist_turn_outcome(
                    persistence,
                    &turn_id,
                    TurnTranscriptOutcome::Cancelled,
                )
            {
                let completion = TurnCompletionStatus::Error { message };
                let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence,
                    payload: TurnEventPayload::Completed {
                        status: completion.clone(),
                    },
                });
                let _ = result_tx.send(TurnRecord {
                    new_messages: Vec::new(),
                    status: TurnRecordStatus::Error,
                    completion,
                });
                return;
            }
            let completion = TurnCompletionStatus::Cancelled;
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: completion.clone(),
                },
            });
            let _ = result_tx.send(TurnRecord {
                new_messages: Vec::new(),
                status: TurnRecordStatus::Cancelled,
                completion,
            });
            return;
        }
    };

    let _ = forwarder.await;
    let diagnostic_failure = diagnostic_failures
        .lock()
        .ok()
        .and_then(|failures| failures.first().cloned());

    match agent_result {
        Ok((Ok(final_text), new_messages)) => {
            if diagnostic_failure.is_some() {
                let completion = TurnCompletionStatus::Error {
                    message: "failed to persist provider terminal diagnostic".into(),
                };
                let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence,
                    payload: TurnEventPayload::Completed {
                        status: completion.clone(),
                    },
                });
                let _ = result_tx.send(TurnRecord {
                    new_messages: Vec::new(),
                    status: TurnRecordStatus::Error,
                    completion,
                });
                return;
            }
            let cloned_messages = new_messages.clone();
            if let Some(persistence) = &persistence
                && let Err(message) = persist_turn_messages(
                    persistence,
                    &turn_id,
                    &new_messages,
                    &raw_tool_outputs,
                    true,
                )
            {
                let completion = TurnCompletionStatus::Error { message };
                let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence,
                    payload: TurnEventPayload::Completed {
                        status: completion.clone(),
                    },
                });
                let _ = result_tx.send(TurnRecord {
                    new_messages: Vec::new(),
                    status: TurnRecordStatus::Error,
                    completion,
                });
                return;
            }
            if let Some(persistence) = &persistence
                && let Err(message) =
                    persist_turn_outcome(persistence, &turn_id, TurnTranscriptOutcome::Success)
            {
                let completion = TurnCompletionStatus::Error { message };
                let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence,
                    payload: TurnEventPayload::Completed {
                        status: completion.clone(),
                    },
                });
                let _ = result_tx.send(TurnRecord {
                    new_messages: Vec::new(),
                    status: TurnRecordStatus::Error,
                    completion,
                });
                return;
            }
            let persisted_entry_ids = if let Some(persistence) = &durable_persistence {
                match persistence
                    .session
                    .commit_turn(&turn_id, &new_messages, &persistence.policy)
                {
                    Ok(commit) => commit.entry_ids,
                    Err(error) => {
                        let completion = TurnCompletionStatus::Error {
                            message: format!("failed to persist completed turn: {error}"),
                        };
                        let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                        let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                            session_id,
                            turn_id,
                            sequence,
                            payload: TurnEventPayload::Completed {
                                status: completion.clone(),
                            },
                        });
                        let _ = result_tx.send(TurnRecord {
                            new_messages: Vec::new(),
                            status: TurnRecordStatus::Error,
                            completion,
                        });
                        return;
                    }
                }
            } else {
                Vec::new()
            };
            if !persisted_entry_ids.is_empty() {
                let _ = eq_tx_clone.send(SessionEvent::EntriesCommitted {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    entry_ids: persisted_entry_ids,
                });
            }
            let completion = TurnCompletionStatus::Success {
                final_text,
                new_messages: cloned_messages,
            };
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: completion.clone(),
                },
            });
            let _ = result_tx.send(TurnRecord {
                new_messages,
                status: TurnRecordStatus::Success,
                completion,
            });
        }
        Ok((Err(e), partial_messages)) => {
            let mut error_message = e.to_string();
            if diagnostic_failure.is_some() {
                error_message
                    .push_str("\n[warning: failed to persist provider terminal diagnostic]");
            }
            let mut transcript_persisted = true;
            if !partial_messages.is_empty()
                && let Some(persistence) = &persistence
            {
                match persist_turn_messages(
                    persistence,
                    &turn_id,
                    &partial_messages,
                    &raw_tool_outputs,
                    false,
                ) {
                    Ok(()) => {}
                    Err(persist_err) => {
                        transcript_persisted = false;
                        error_message = format!(
                            "{error_message}\n[warning: failed to persist partial turn messages: {persist_err}]"
                        );
                    }
                }
            }
            if transcript_persisted
                && let Some(persistence) = &persistence
                && let Err(persist_err) =
                    persist_turn_outcome(persistence, &turn_id, TurnTranscriptOutcome::Error)
            {
                error_message = format!(
                    "{error_message}\n[warning: failed to persist terminal transcript outcome: {persist_err}]"
                );
            }
            let completion = TurnCompletionStatus::Error {
                message: error_message,
            };
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: completion.clone(),
                },
            });
            let _ = result_tx.send(TurnRecord {
                new_messages: partial_messages,
                status: TurnRecordStatus::Error,
                completion,
            });
        }
        Err(_join_error) => {
            error!("agent panicked during turn");
            let mut message = "agent panicked".to_string();
            if let Some(persistence) = &persistence
                && let Err(persist_err) =
                    persist_turn_outcome(persistence, &turn_id, TurnTranscriptOutcome::Error)
            {
                message = format!(
                    "{message}\n[warning: failed to persist terminal transcript outcome: {persist_err}]"
                );
            }
            let completion = TurnCompletionStatus::Error { message };
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: completion.clone(),
                },
            });
            let _ = result_tx.send(TurnRecord {
                new_messages: Vec::new(),
                status: TurnRecordStatus::Error,
                completion,
            });
        }
    }
}

fn persist_turn_messages(
    persistence: &TurnPersistence,
    turn_id: &str,
    messages: &[Message],
    raw_tool_outputs: &Arc<Mutex<HashMap<String, String>>>,
    bind_turn_id: bool,
) -> Result<(), String> {
    for message in messages {
        let mut metadata = persistence.metadata.clone();
        metadata.turn_id = bind_turn_id.then(|| turn_id.to_owned());
        if let Message::Tool { result } = message
            && let Ok(outputs) = raw_tool_outputs.lock()
            && let Some(raw) = outputs.get(&result.tool_use_id)
            && raw != &result.content
        {
            metadata.raw_content = Some(raw.clone());
        }
        persistence
            .session
            .append_with_metadata(message, metadata)
            .map_err(|error| format!("failed to persist completed turn: {error}"))?;
    }
    Ok(())
}

fn persist_turn_outcome(
    persistence: &TurnPersistence,
    turn_id: &str,
    outcome: TurnTranscriptOutcome,
) -> Result<(), String> {
    persistence
        .session
        .append_turn_transcript_outcome(&TurnTranscriptOutcomeRecord::new(turn_id, outcome))
        .map_err(|error| format!("failed to persist terminal transcript outcome: {error}"))
}
