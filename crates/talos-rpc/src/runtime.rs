//! Narrow runtime contract that the RPC layer needs from the agent runtime.
//!
//! This trait lives in `talos-rpc` so the RPC crate does not depend on
//! `talos-agent` directly. The concrete implementation is provided by the
//! composition root (today: `talos-cli`).

use async_trait::async_trait;
use talos_core::message::{AgentEvent, Message};
use tokio::sync::mpsc;

/// Error surfaced by the runtime to the RPC layer. The concrete agent error
/// is mapped into this type at the composition root.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("runtime failure: {0}")]
    Runtime(#[from] anyhow::Error),
}

/// Narrow runtime contract that the RPC layer needs from the agent runtime.
///
/// This trait is object-safe (uses `#[async_trait]`) so that `talos-rpc`
/// can accept `Arc<dyn Runtime>` without depending on `talos-agent`.
#[async_trait]
pub trait Runtime: Send + Sync {
    /// Run a single non-streaming turn and return the final text.
    async fn run(&self, user_message: String) -> Result<String, RuntimeError>;

    /// Run a single streaming turn, emitting `AgentEvent`s on `event_tx`.
    ///
    /// Returns the final assistant text.
    async fn run_streaming(
        &self,
        user_message: String,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
    ) -> Result<String, RuntimeError>;
}
