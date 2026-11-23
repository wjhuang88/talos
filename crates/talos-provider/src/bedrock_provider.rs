//! Compose the signed HTTP transport and ConverseStream event reader.
use crate::{
    bedrock_endpoint, bedrock_request, bedrock_response::ResponseReader,
    bedrock_stream::ConverseStream, bedrock_transport,
};
use std::time::Duration;
use talos_config::ProviderTimeoutConfig;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult, ToolDefinition};
use tokio::sync::mpsc;

#[cfg(test)]
tokio::task_local! { pub(crate) static FIXTURE: reqwest::Url; }

pub(crate) struct BedrockProvider {
    #[cfg(test)]
    fixture: Option<reqwest::Url>,
    model: String,
    region: String,
    limit: Option<u32>,
    timeout: ProviderTimeoutConfig,
}

impl BedrockProvider {
    pub(crate) fn new(
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
}

impl BedrockProvider {
    async fn dispatch(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: talos_core::tool::ToolProtocol,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let body = bedrock_request::build(messages, tools, self.limit)?;
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
        let (sender, receiver) = mpsc::channel(32);
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
                    if let Some(event) = stream.accept(&name,body)? && sender.send(event).await.is_err() { return Ok(()); }
                }
                for event in stream.finish()? {
                    if sender.send(event).await.is_err() {break;}
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
        Ok(receiver)
    }
}
