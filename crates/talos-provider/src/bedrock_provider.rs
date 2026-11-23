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
    budget: Option<u32>,
    allow_replay: bool,
    timeout: ProviderTimeoutConfig,
}

impl BedrockProvider {
    pub(crate) fn new(
        provider: &str,
        model: &str,
        region: String,
        limit: Option<u32>,
        allow_replay: bool,
        timeout: ProviderTimeoutConfig,
    ) -> ProviderResult<Self> {
        for seconds in [
            timeout.dispatch_timeout_secs,
            timeout.first_packet_timeout_secs,
            timeout.stream_idle_timeout_secs,
        ] {
            deadline_after(tokio::time::Instant::now(), seconds)?;
        }
        bedrock_endpoint::converse_stream(&region, model)
            .map_err(|e| ProviderError::InvalidResponse(e.into()))?;
        Ok(Self {
            #[cfg(test)]
            fixture: FIXTURE.try_with(Clone::clone).ok(),
            provider: provider.into(),
            model: model.into(),
            region,
            limit,
            budget: None,
            allow_replay,
            timeout,
        })
    }
    pub(crate) fn with_reasoning(
        mut self,
        options: Option<talos_config::ReasoningOptions>,
    ) -> ProviderResult<Self> {
        let Some(options) = options else {
            return Ok(self);
        };
        if options.effort.is_some() {
            return Err(ProviderError::InvalidResponse(
                "Bedrock reasoning effort is unsupported on this route".into(),
            ));
        }
        if let Some(budget) = options.budget_tokens {
            if self.model != "us.anthropic.claude-sonnet-4-20250514-v1:0"
                || !self.allow_replay
                || budget < 1024
                || self.limit.is_none_or(|limit| budget >= limit)
            {
                return Err(ProviderError::InvalidResponse("Bedrock thinking budget requires the supported Sonnet 4 profile, replay and an output limit above the budget (minimum 1024)".into()));
            }
            self.budget = Some(budget);
        }
        Ok(self)
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
        self.dispatch(
            messages,
            tools,
            talos_core::tool::ToolProtocol::Native,
            None,
        )
        .await
    }

    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: talos_core::tool::ToolProtocol,
        progress: mpsc::UnboundedSender<talos_core::provider::ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        use talos_core::tool::ToolProtocol;
        match protocol {
            ToolProtocol::Native => {
                self.dispatch(messages, tools, protocol, Some(&progress))
                    .await
            }
            ToolProtocol::Compat | ToolProtocol::TalosStrict => {
                bedrock_request::validate_tool_history(messages)?;
                let projected = crate::compatibility_messages(messages);
                self.dispatch(&projected, &[], protocol, Some(&progress))
                    .await
            }
            ToolProtocol::Auto => Err(ProviderError::InvalidResponse(
                "resolve the Bedrock tool protocol before dispatch".into(),
            )),
        }
    }
    async fn stream_with_ephemeral_images(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress: mpsc::UnboundedSender<ProviderProgress>,
        browser_tools: &[String],
        images: Vec<EphemeralImage>,
    ) -> ProviderResult<ProviderInvocationStream> {
        self.stream_with_replay_context(
            messages,
            tools,
            protocol,
            progress,
            browser_tools,
            images,
            vec![],
        )
        .await
    }
    async fn stream_with_replay_context(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        protocol: ToolProtocol,
        progress: mpsc::UnboundedSender<ProviderProgress>,
        browser_tools: &[String],
        images: Vec<EphemeralImage>,
        replay: Vec<Arc<TransientReplay>>,
    ) -> ProviderResult<ProviderInvocationStream> {
        if !browser_tools.is_empty() || protocol == ToolProtocol::Auto {
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
        if !self.allow_replay
            && (!replay.is_empty()
                || messages.iter().any(|m| {
                    matches!(
                        m,
                        Message::Assistant {
                            reasoning: Some(_),
                            ..
                        }
                    )
                }))
        {
            return Err(ProviderError::InvalidResponse(
                "Bedrock reasoning replay disabled".into(),
            ));
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
        let mut body = bedrock_request::build_bound(messages, tools, self.limit, Some(&route))?;
        let expires = crate::image_io::append_bedrock_ephemeral_images(&mut body, images)?;
        if !self.allow_replay {
            let (tx, rx) = mpsc::channel(32);
            self.run(body, protocol, Sink::Legacy(tx), Some(&progress), expires)
                .await?;
            return Ok(ProviderInvocationStream::legacy(rx));
        }
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
            Some(&progress),
            expires,
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
        progress: Option<&mpsc::UnboundedSender<ProviderProgress>>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let body = bedrock_request::build(messages, tools, self.limit)?;
        let (tx, rx) = mpsc::channel(32);
        self.run(body, protocol, Sink::Legacy(tx), progress, None)
            .await?;
        Ok(rx)
    }

    async fn run(
        &self,
        mut body: serde_json::Value,
        protocol: ToolProtocol,
        sender: Sink,
        progress: Option<&mpsc::UnboundedSender<ProviderProgress>>,
        expires: Option<std::time::Instant>,
    ) -> ProviderResult<()> {
        if let Some(budget) = self.budget {
            body["additionalModelRequestFields"] =
                serde_json::json!({"thinking":{"type":"enabled","budget_tokens":budget}});
        }
        let preview_summary = self.budget.is_some();
        let payload = serde_json::to_vec(&body)
            .map_err(|_| ProviderError::InvalidResponse("invalid Bedrock request body".into()))?;
        let deadline = deadline_after(
            tokio::time::Instant::now(),
            self.timeout.dispatch_timeout_secs,
        )?;
        let deadline = expires.map_or(deadline, |t| deadline.min(t.into()));
        let max_attempts = self.timeout.max_attempts;
        let mut attempt = 0;
        let payload = bytes::Bytes::from(payload);
        let (mut reader, started) = tokio::time::timeout_at(deadline, async {
            loop {
                if tokio::time::Instant::now() >= deadline {
                    return Err(ProviderError::InvalidResponse(
                        "Bedrock dispatch deadline exceeded".into(),
                    ));
                }
                let started = tokio::time::Instant::now();
                if let Some(tx) = progress {
                    let _ = tx.send(if attempt == 0 {
                        ProviderProgress::InitialDispatch {
                            attempt,
                            max_attempts,
                        }
                    } else {
                        ProviderProgress::RetryDispatch {
                            attempt,
                            max_attempts,
                        }
                    });
                }
                let response = bedrock_transport::dispatch(
                    &self.region,
                    &self.model,
                    payload.clone(),
                    expires,
                    #[cfg(test)]
                    self.fixture.as_ref(),
                )
                .await
                .and_then(ResponseReader::new);
                match response {
                    Ok(reader) => return Ok((reader, started)),
                    Err(error) => match crate::retry::classify_retry_with_backoff(
                        &error,
                        attempt,
                        max_attempts,
                        self.timeout.backoff_base_ms,
                        self.timeout.backoff_max_ms,
                    ) {
                        crate::retry::RetryDecision::DoNotRetry => return Err(error),
                        crate::retry::RetryDecision::Retry {
                            attempt: next,
                            delay_ms,
                        } => {
                            if let Some(tx) = progress {
                                let _ = tx.send(ProviderProgress::ScheduledBackoff {
                                    attempt: next,
                                    max_attempts,
                                    delay_ms,
                                });
                            }
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            attempt = next;
                        }
                    },
                }
            }
        })
        .await
        .map_err(|_| {
            ProviderError::InvalidResponse("Bedrock dispatch deadline exceeded".into())
        })??;
        if let Some(tx) = progress {
            let _ = tx.send(ProviderProgress::FirstPacketWait {
                attempt,
                max_attempts,
            });
        }
        let first = deadline_after(started, self.timeout.first_packet_timeout_secs)?
            .saturating_duration_since(tokio::time::Instant::now());
        let idle = Duration::from_secs(self.timeout.stream_idle_timeout_secs);
        tokio::spawn(async move {
            let mut stream = ConverseStream::new(protocol);
            stream.preview_summary = preview_summary;
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

fn deadline_after(
    start: tokio::time::Instant,
    seconds: u64,
) -> ProviderResult<tokio::time::Instant> {
    start
        .checked_add(Duration::from_secs(seconds))
        .ok_or_else(|| ProviderError::InvalidResponse("Bedrock timeout exceeds clock range".into()))
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
