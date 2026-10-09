//! Transient ingress evidence, deliberately separate from serializable conversation events.

use crate::{message::AgentEvent, tool::BrowserRawArguments};
use tokio::sync::mpsc;

/// One provider event owned by the active request; never persisted or cloned for replay.
pub enum ProviderInvocationEvent {
    /// Existing event without original browser argument evidence.
    Legacy(AgentEvent),
    /// Complete browser invocation validated at trusted original transport ingress.
    /// Arguments remain sensitive and must not be forwarded to observer channels.
    BrowserToolCall {
        /// Provider call identity within this request.
        id: String,
        /// Tool name resolved against the request's explicit browser tool definitions.
        name: String,
        /// Original argument syntax evidence; not semantic admission or permission.
        arguments: BrowserRawArguments,
    },
}

impl std::fmt::Debug for ProviderInvocationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Legacy(_) => f.write_str("ProviderInvocationEvent::Legacy { .. }"),
            Self::BrowserToolCall { .. } => {
                f.write_str("ProviderInvocationEvent::BrowserToolCall { .. }")
            }
        }
    }
}

enum Source {
    Legacy(mpsc::Receiver<AgentEvent>),
    Original(mpsc::Receiver<ProviderInvocationEvent>),
}

/// Bounded provider receiver with no forwarding task or persistent evidence side channel.
/// Dropping this stream closes its underlying receiver.
pub struct ProviderInvocationStream(Source);

impl ProviderInvocationStream {
    /// Wraps a legacy stream. Its events supply no original-argument evidence.
    pub fn legacy(receiver: mpsc::Receiver<AgentEvent>) -> Self {
        Self(Source::Legacy(receiver))
    }

    /// Wraps a trusted adapter's transient invocation stream.
    pub fn original(receiver: mpsc::Receiver<ProviderInvocationEvent>) -> Self {
        Self(Source::Original(receiver))
    }

    /// Receives the next event. Cancellation preserves the underlying channel's semantics.
    pub async fn recv(&mut self) -> Option<ProviderInvocationEvent> {
        match &mut self.0 {
            Source::Legacy(receiver) => receiver.recv().await.map(ProviderInvocationEvent::Legacy),
            Source::Original(receiver) => receiver.recv().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn original_evidence_stays_transient_and_receiver_drop_closes_producer() {
        let (tx, rx) = mpsc::channel(1);
        let event = ProviderInvocationEvent::BrowserToolCall {
            id: "private-id".into(),
            name: "private-name".into(),
            arguments: BrowserRawArguments::parse_original(r#"{"text":"private-text"}"#)
                .expect("original syntax"),
        };
        assert!(!format!("{event:?}").contains("private"));
        tx.send(event).await.expect("receiver live");
        let mut stream = ProviderInvocationStream::original(rx);
        let Some(ProviderInvocationEvent::BrowserToolCall { arguments, .. }) = stream.recv().await
        else {
            panic!("original event")
        };
        assert_eq!(arguments.fields()["text"], "private-text");
        drop(stream);
        assert!(tx.is_closed());
    }
}
