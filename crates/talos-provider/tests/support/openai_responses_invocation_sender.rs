//! Minimal invocation sink used by staged OpenAI Responses integration wrappers.
//!
//! The production Responses bridge currently reuses `openai_sse::InvocationSender`. Standalone
//! path-based integration crates do not need to compile the full Chat/Anthropic parsers (and their
//! crate-root tests) just to exercise that ingress contract, so this support module mirrors only the
//! `send` and `native_call` behavior used by Responses.

use talos_core::message::{AgentEvent, ToolCall};
use tokio::sync::mpsc;

pub(crate) enum InvocationSender {
    Legacy(mpsc::Sender<AgentEvent>),
    Original {
        tx: mpsc::Sender<talos_core::provider::ProviderInvocationEvent>,
        browser_tools: Vec<String>,
    },
}

impl InvocationSender {
    pub(crate) async fn send(&self, event: AgentEvent) -> Result<(), ()> {
        match self {
            Self::Legacy(tx) => tx.send(event).await.map_err(|_| ()),
            Self::Original { tx, .. } => tx
                .send(talos_core::provider::ProviderInvocationEvent::Legacy(event))
                .await
                .map_err(|_| ()),
        }
    }

    pub(crate) async fn native_call(&self, id: String, name: String, raw: &str) -> Result<(), ()> {
        if let Self::Original { tx, browser_tools } = self
            && browser_tools.contains(&name)
        {
            let arguments = match talos_core::tool::BrowserRawArguments::parse_original(raw) {
                Ok(arguments) => arguments,
                Err(_) => {
                    let _ = self
                        .send(AgentEvent::Error {
                            message: "invalid original browser arguments".into(),
                        })
                        .await;
                    return Err(());
                }
            };
            return tx
                .send(
                    talos_core::provider::ProviderInvocationEvent::BrowserToolCall {
                        id,
                        name,
                        arguments,
                    },
                )
                .await
                .map_err(|_| ());
        }

        let input = match serde_json::from_str(raw) {
            Ok(input) => input,
            Err(_) => {
                let _ = self
                    .send(AgentEvent::Error {
                        message: "invalid tool arguments JSON".into(),
                    })
                    .await;
                return Err(());
            }
        };
        self.send(AgentEvent::ToolCall {
            call: ToolCall { id, name, input },
            provenance: Default::default(),
            summary_fields: vec![],
        })
        .await
    }
}
