use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message};
use talos_core::session::{
    SessionEvent, SessionHandle, SessionOp, TurnCompletionStatus, TurnEventPayload,
};
use talos_rpc::{Runtime, RuntimeError};
use tokio::sync::{Mutex, mpsc};

/// RPC adapter backed by the canonical `AppServerSession` SQ/EQ seam.
pub struct AgentRuntime {
    command_tx: mpsc::Sender<SessionOp>,
    event_rx: Mutex<mpsc::UnboundedReceiver<SessionEvent>>,
}

impl AgentRuntime {
    /// Creates an RPC adapter from a running session handle.
    pub fn new(handle: SessionHandle) -> Self {
        Self {
            command_tx: handle.sq_tx,
            event_rx: Mutex::new(handle.eq_rx),
        }
    }

    async fn execute(
        &self,
        user_message: String,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
    ) -> Result<String, RuntimeError> {
        // One EQ consumer owns a turn from submit through completion. RPC calls
        // are intentionally serialized until the protocol gains request-id
        // multiplexing at the public RPC boundary.
        let mut event_rx = self.event_rx.lock().await;
        self.command_tx
            .send(SessionOp::Submit {
                message: user_message,
            })
            .await
            .map_err(|_| runtime_error("session command channel closed"))?;

        let mut interrupt_on_drop = InterruptOnDrop::new(self.command_tx.clone());
        while let Some(event) = event_rx.recv().await {
            match event {
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Progress { event },
                    ..
                } => {
                    if let Some(tx) = &event_tx {
                        let _ = tx.send(event);
                    }
                }
                SessionEvent::TurnEvent {
                    payload: TurnEventPayload::Completed { status },
                    ..
                } => {
                    interrupt_on_drop.disarm();
                    return match status {
                        TurnCompletionStatus::Success { final_text, .. } => Ok(final_text),
                        TurnCompletionStatus::Cancelled => Err(runtime_error("turn cancelled")),
                        TurnCompletionStatus::Error { message } => Err(runtime_error(message)),
                    };
                }
                SessionEvent::Error { message } => {
                    interrupt_on_drop.disarm();
                    return Err(runtime_error(message));
                }
                _ => {}
            }
        }

        interrupt_on_drop.disarm();
        Err(runtime_error("session event channel closed"))
    }
}

#[async_trait]
impl Runtime for AgentRuntime {
    async fn run(&self, user_message: String) -> Result<String, RuntimeError> {
        self.execute(user_message, None).await
    }

    async fn run_streaming(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<String, RuntimeError> {
        if !history.is_empty() {
            return Err(runtime_error(
                "RPC per-call history is unsupported; resume through the session runtime",
            ));
        }
        self.execute(user_message, Some(event_tx)).await
    }
}

struct InterruptOnDrop {
    command_tx: mpsc::Sender<SessionOp>,
    armed: bool,
}

impl InterruptOnDrop {
    fn new(command_tx: mpsc::Sender<SessionOp>) -> Self {
        Self {
            command_tx,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for InterruptOnDrop {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.command_tx.try_send(SessionOp::Interrupt);
        }
    }
}

fn runtime_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::from(anyhow::anyhow!(message.into()))
}
