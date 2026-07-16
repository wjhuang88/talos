use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use talos_core::message::{AgentEvent, Message};
use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};

use crate::Agent;

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
}

pub(super) struct TurnForwarding {
    pub(super) agent: Arc<Agent>,
    pub(super) message: String,
    pub(super) history: Vec<Message>,
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
        message,
        history,
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

    let forwarder = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => break,
                event = event_rx.recv() => {
                    match event {
                        Some(event) => {
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
                                payload: TurnEventPayload::Progress { event },
                            });
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let mut agent_task =
        tokio::spawn(async move { agent.run_for_session_turn(message, history, event_tx).await });

    let agent_result = tokio::select! {
        result = &mut agent_task => result,
        _ = cancel_token.cancelled() => {
            agent_task.abort();
            let _ = forwarder.await;
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Cancelled,
                },
            });
            return;
        }
    };

    let _ = forwarder.await;

    match agent_result {
        Ok((Ok(final_text), new_messages)) => {
            let cloned_messages = new_messages.clone();
            if let Some(persistence) = &persistence
                && let Err(message) =
                    persist_turn_messages(persistence, &new_messages, &raw_tool_outputs)
            {
                let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                    session_id,
                    turn_id,
                    sequence,
                    payload: TurnEventPayload::Completed {
                        status: TurnCompletionStatus::Error { message },
                    },
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
                        let sequence = sequence.fetch_add(1, Ordering::Relaxed);
                        let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                            session_id,
                            turn_id,
                            sequence,
                            payload: TurnEventPayload::Completed {
                                status: TurnCompletionStatus::Error {
                                    message: format!("failed to persist completed turn: {error}"),
                                },
                            },
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
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Success {
                        final_text: final_text.clone(),
                        new_messages: cloned_messages,
                    },
                },
            });
            let _ = result_tx.send(TurnRecord { new_messages });
        }
        Ok((Err(e), partial_messages)) => {
            // SESSION-006 / I135: persist valid completed tool exchanges even
            // when the agent turn fails. The partial_messages contain only
            // normalized, complete exchanges — never half-streamed fragments
            // or fabricated tool results. Durable Runtime (ADR-042) still
            // aborts failed turns: no commit_turn call happens here.
            if !partial_messages.is_empty()
                && let Some(persistence) = &persistence
            {
                let _ = persist_turn_messages(persistence, &partial_messages, &raw_tool_outputs);
            }
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Error {
                        message: e.to_string(),
                    },
                },
            });
        }
        Err(_join_error) => {
            error!("agent panicked during turn");
            let sequence = sequence.fetch_add(1, Ordering::Relaxed);
            let _ = eq_tx_clone.send(SessionEvent::TurnEvent {
                session_id,
                turn_id,
                sequence,
                payload: TurnEventPayload::Completed {
                    status: TurnCompletionStatus::Error {
                        message: "agent panicked".into(),
                    },
                },
            });
        }
    }
}

fn persist_turn_messages(
    persistence: &TurnPersistence,
    messages: &[Message],
    raw_tool_outputs: &Arc<Mutex<HashMap<String, String>>>,
) -> Result<(), String> {
    for message in messages {
        let mut metadata = persistence.metadata.clone();
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
