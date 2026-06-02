//! Provider trait and error types for LLM backends.

use tokio::sync::mpsc;

use crate::message::{AgentEvent, Message};

/// A receiver channel for streaming [`AgentEvent`]s from a language model.
pub type Receiver<T> = mpsc::Receiver<T>;

/// Errors that can occur when interacting with an LLM provider.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// The API key is missing or invalid.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// The provider returned a rate-limit response (HTTP 429).
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// The provider returned a server error (HTTP 5xx).
    #[error("server error: {0}")]
    ServerError(String),

    /// A network-level error occurred.
    #[error("network error: {0}")]
    NetworkError(String),

    /// The response from the provider could not be parsed.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

/// Result alias for provider operations.
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Abstraction over a language model provider.
///
/// Implementors handle communication with a specific LLM API,
/// streaming [`AgentEvent`]s back through an async channel.
#[async_trait::async_trait]
pub trait LanguageModel: Send + Sync {
    /// Stream events from the language model for the given messages.
    ///
    /// Returns a [`Receiver`] that yields [`AgentEvent`]s as they arrive
    /// from the provider. The caller should consume events until the
    /// channel is closed (typically after a [`AgentEvent::TurnEnd`] or
    /// [`AgentEvent::Error`]).
    ///
    /// # Errors
    ///
    /// Returns a [`ProviderError`] if the request cannot be initiated
    /// (e.g., authentication failure, network error, invalid response).
    async fn stream(&self, messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>>;
}
