//! Provider trait and error types for LLM backends.

use serde_json::Value;
use tokio::sync::mpsc;

use crate::message::{AgentEvent, Message};

pub type Receiver<T> = mpsc::Receiver<T>;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl ToolDefinition {
    /// Creates a new tool definition.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    /// Formats this tool definition as a text block suitable for inclusion
    /// in the system prompt.
    #[must_use]
    pub fn to_prompt_text(&self) -> String {
        format!(
            "## {}\n{}\nParameters: {}",
            self.name,
            self.description,
            serde_json::to_string_pretty(&self.parameters).unwrap_or_default()
        )
    }
}

#[async_trait::async_trait]
pub trait LanguageModel: Send + Sync {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>>;

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<Receiver<AgentEvent>> {
        let _ = tools;
        self.stream(messages).await
    }

    fn request_preview(&self, _messages: &[Message]) -> Option<Value> {
        None
    }
}
