use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use talos_core::message::{AgentEvent, Message};
use talos_core::session::{SessionEvent, TurnCompletionStatus};

use crate::Agent;

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
        result_tx,
    } = turn;

    let eq_tx_clone = eq_tx.clone();
    let cancel_clone = cancel_token.clone();

    let forwarder = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => break,
                event = event_rx.recv() => {
                    match event {
                        Some(event) => {
                            let _ = eq_tx.send(SessionEvent::AgentEvent { event });
                        }
                        None => break,
                    }
                }
            }
        }
    });

    let mut agent_task =
        tokio::spawn(async move { agent.run_streaming(message, history, event_tx).await });

    let agent_result = tokio::select! {
        result = &mut agent_task => result,
        _ = cancel_token.cancelled() => {
            agent_task.abort();
            let _ = forwarder.await;
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Cancelled,
            });
            return;
        }
    };

    let _ = forwarder.await;

    match agent_result {
        Ok(Ok((final_text, new_messages))) => {
            let cloned_messages = new_messages.clone();
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Success {
                    final_text: final_text.clone(),
                    new_messages: cloned_messages,
                },
            });
            let _ = result_tx.send(TurnRecord { new_messages });
        }
        Ok(Err(e)) => {
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Error {
                    message: e.to_string(),
                },
            });
        }
        Err(_join_error) => {
            error!("agent panicked during turn");
            let _ = eq_tx_clone.send(SessionEvent::TurnCompleted {
                turn_id,
                status: TurnCompletionStatus::Error {
                    message: "agent panicked".into(),
                },
            });
        }
    }
}
