//! Compose the signed HTTP transport and ConverseStream event reader.
use crate::{
    bedrock_endpoint, bedrock_request, bedrock_response::ResponseReader,
    bedrock_stream::ConverseStream, bedrock_transport,
};
use std::{sync::Arc, time::Duration};
use talos_config::ProviderTimeoutConfig;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use talos_core::{
    provider::{
        EphemeralImage, ProviderInvocationEvent, ProviderInvocationStream, ProviderProgress,
        TransientReplay,
    },
    tool::ToolProtocol,
};
use tokio::sync::mpsc;

#[cfg(test)]
tokio::task_local! { pub(crate) static FIXTURE: reqwest::Url; }

pub(crate) struct BedrockProvider {
    #[cfg(test)]
    fixture: Option<reqwest::Url>,
    provider: String,
    model: String,
    region: String,
    limit: Option<u32>,
    timeout: ProviderTimeoutConfig,
}

impl BedrockProvider {
    pub(crate) fn new(
        provider: &str,
        model: &str,
        region: String,
        limit: Option<u32>,
        timeout: ProviderTimeoutConfig,
    ) -> ProviderResult<Self> {
        bedrock_endpoint::converse_stream(&region, model)
            .map_err(|e| ProviderError::InvalidResponse(e.into()))?;
        Ok(Self {
            #[cfg(test)]
            fixture: FIXTURE.try_with(Clone::clone).ok(),
            provider: provider.into(),
            model: model.into(),
            region,
            limit,
            timeout,
        })
    }
}

#[async_trait::async_trait]
impl LanguageModel for BedrockProvider {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.stream_with_tools(messages, &[]).await
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.dispatch(messages, tools, talos_core::tool::ToolProtocol::Native)
            .await
    }

    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: talos_core::tool::ToolProtocol,
        _progress: mpsc::UnboundedSender<talos_core::provider::ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        use talos_core::tool::ToolProtocol;
        match protocol {
            ToolProtocol::Native => self.stream_with_tools(messages, tools).await,
            ToolProtocol::Compat | ToolProtocol::TalosStrict => {
                bedrock_request::validate_tool_history(messages)?;
                let projected = crate::compatibility_messages(messages);
                self.dispatch(&projected, &[], protocol).await
            }
            ToolProtocol::Auto => Err(ProviderError::InvalidResponse(
                "resolve the Bedrock tool protocol before dispatch".into(),
            )),
        }
    }
    async fn stream_with_replay_context(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        _progress: mpsc::UnboundedSender<ProviderProgress>,
        browser_tools: &[String],
        images: Vec<EphemeralImage>,
        replay: Vec<Arc<TransientReplay>>,
    ) -> ProviderResult<ProviderInvocationStream> {
        if !browser_tools.is_empty() || !images.is_empty() || protocol == ToolProtocol::Auto {
            return Err(ProviderError::InvalidResponse(
                "unsupported Bedrock invocation context".into(),
            ));
        }
        for message in messages {
            if matches!(
                message,
                Message::Assistant {
                    reasoning: Some(_),
                    ..
                }
            ) && TransientReplay::marker(message)?.is_none()
            {
                return Err(ProviderError::InvalidResponse(
                    "unverified Bedrock reasoning history".into(),
                ));
            }
        }
        let restored =
            TransientReplay::restore_history(messages, &self.provider, &self.model, &replay)?;
        bedrock_request::validate_tool_history(&restored)?;
        let projected;
        let (messages, tools) = if protocol == ToolProtocol::Native {
            (restored.as_slice(), tools)
        } else {
            projected = crate::compatibility_messages(&restored);
            (projected.as_slice(), &[][..])
        };
        let route =
            serde_json::json!([self.provider, self.model, self.region, protocol]).to_string();
        let body = bedrock_request::build_bound(messages, tools, self.limit, Some(&route))?;
        let binding = bedrock_request::fingerprint(
            body.get("system").unwrap_or(&serde_json::json!([])),
            &body["messages"],
            &route,
        );
        let (tx, rx) = mpsc::channel(32);
        self.run(
            body,
            protocol,
            Sink::Private(tx, self.provider.clone(), self.model.clone(), binding),
        )
        .await?;
        Ok(ProviderInvocationStream::original(rx))
    }
}

impl BedrockProvider {
    async fn dispatch(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: talos_core::tool::ToolProtocol,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let body = bedrock_request::build(messages, tools, self.limit)?;
        let (tx, rx) = mpsc::channel(32);
        self.run(body, protocol, Sink::Legacy(tx)).await?;
        Ok(rx)
    }

    async fn run(
        &self,
        body: serde_json::Value,
        protocol: ToolProtocol,
        sender: Sink,
    ) -> ProviderResult<()> {
        let payload = serde_json::to_vec(&body)
            .map_err(|_| ProviderError::InvalidResponse("invalid Bedrock request body".into()))?;
        let response = tokio::time::timeout(
            Duration::from_secs(self.timeout.dispatch_timeout_secs),
            bedrock_transport::dispatch(
                &self.region,
                &self.model,
                payload,
                #[cfg(test)]
                self.fixture.as_ref(),
            ),
        )
        .await
        .map_err(|_| ProviderError::InvalidResponse("Bedrock dispatch timed out".into()))??;
        let mut reader = ResponseReader::new(response)?;

        let first = Duration::from_secs(self.timeout.first_packet_timeout_secs);
        let idle = Duration::from_secs(self.timeout.stream_idle_timeout_secs);
        tokio::spawn(async move {
            let mut stream = ConverseStream::new(protocol);
            let mut timeout = first;
            let result: ProviderResult<()> = async {
                loop {
                    let packet = tokio::select! {
                        () = sender.closed() => return Ok(()),
                        packet = tokio::time::timeout(timeout,reader.next()) => packet.map_err(|_|ProviderError::InvalidResponse("Bedrock event stream timed out".into()))??,
                    };
                    let Some((name,body)) = packet else { break };
                    timeout = idle;
                    if let Some(event) = stream.accept(&name,body)? && !sender.send(event).await { return Ok(()); }
                }
                let events = match &sender {
                    Sink::Private(tx,provider,model,binding) => {
                        let (replay,events) = stream.finish_bound(provider,model,binding)?;
                        if let Some(replay) = replay && tx.send(ProviderInvocationEvent::TransientReplay(Arc::new(replay))).await.is_err() { return Ok(()); }
                        events
                    }
                    Sink::Legacy(_) => stream.finish()?,
                };
                for event in events {
                    if !sender.send(event).await {break;}
                }
                Ok(())
            }.await;
            if let Err(error) = result {
                let _ = sender
                    .send(AgentEvent::Error {
                        message: error.to_string(),
                    })
                    .await;
            }
        });
        Ok(())
    }
}

enum Sink {
    Legacy(mpsc::Sender<AgentEvent>),
    Private(
        mpsc::Sender<ProviderInvocationEvent>,
        String,
        String,
        String,
    ),
}
impl Sink {
    async fn closed(&self) {
        match self {
            Self::Legacy(tx) => tx.closed().await,
            Self::Private(tx, ..) => tx.closed().await,
        }
    }
    async fn send(&self, event: AgentEvent) -> bool {
        match self {
            Self::Legacy(tx) => tx.send(event).await.is_ok(),
            Self::Private(tx, ..) => tx
                .send(ProviderInvocationEvent::Legacy(event))
                .await
                .is_ok(),
        }
    }
}
