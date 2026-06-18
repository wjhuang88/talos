use async_trait::async_trait;
use talos_agent::Agent;
use talos_core::message::{AgentEvent, Message};
use talos_rpc::{Runtime, RuntimeError};
use tokio::sync::mpsc;

pub struct AgentRuntime(pub Agent);

#[async_trait]
impl Runtime for AgentRuntime {
    async fn run(&self, user_message: String) -> Result<String, RuntimeError> {
        self.0
            .run(user_message)
            .await
            .map_err(|e| RuntimeError::from(anyhow::Error::from(e)))
    }

    async fn run_streaming(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<String, RuntimeError> {
        let (text, _messages) = self
            .0
            .run_streaming(user_message, history, event_tx)
            .await
            .map_err(|e| RuntimeError::from(anyhow::Error::from(e)))?;
        Ok(text)
    }
}
