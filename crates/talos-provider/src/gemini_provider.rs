//! Composition of admitted Gemini HTTP, request and SSE boundaries into LanguageModel.

use crate::gemini_stream::GeminiOutput;
use crate::gemini_transport::GeminiTransport;
use crate::openai_sse::InvocationSender;
use std::time::{Duration, Instant};
use talos_config::ProviderTimeoutConfig;
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::{
    LanguageModel, ProviderError, ProviderProgress, ProviderResult, ToolDefinition,
};
use talos_core::tool::{CapabilityProbe, ProtocolCapabilities};
use tokio::sync::mpsc;

#[derive(Clone)]
pub(crate) struct GeminiProvider {
    transport: Option<GeminiTransport>,
    vertex: Option<std::sync::Arc<VertexProfile>>,
    provider: String,
    model: String,
    output_limit: Option<u32>,
    timeout: ProviderTimeoutConfig,
}

struct VertexProfile {
    project: String,
    location: String,
    allow_metadata: bool,
    transport: tokio::sync::OnceCell<GeminiTransport>,
}

impl GeminiProvider {
    /// Transport must already be admitted for the same provider/model identity.
    pub(crate) fn new(
        transport: GeminiTransport,
        provider: String,
        model: String,
        output_limit: Option<u32>,
        timeout: ProviderTimeoutConfig,
    ) -> ProviderResult<Self> {
        if provider.is_empty() || model != transport.model() || !transport.is_streaming() {
            return Err(ProviderError::InvalidResponse(
                "Gemini transport identity mismatch".into(),
            ));
        }
        Ok(Self {
            transport: Some(transport),
            vertex: None,
            provider,
            model,
            output_limit,
            timeout,
        })
    }

    pub(crate) fn vertex(
        project: String,
        location: String,
        allow_metadata: bool,
        provider: String,
        model: String,
        output_limit: Option<u32>,
        timeout: ProviderTimeoutConfig,
    ) -> ProviderResult<Self> {
        crate::gemini_endpoint::vertex(&project, &location, &model, true)
            .map_err(|message| ProviderError::InvalidResponse(message.into()))?;
        if provider.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "missing Gemini provider identity".into(),
            ));
        }
        Ok(Self {
            transport: None,
            vertex: Some(std::sync::Arc::new(VertexProfile {
                project,
                location,
                allow_metadata,
                transport: tokio::sync::OnceCell::new(),
            })),
            provider,
            model,
            output_limit,
            timeout,
        })
    }

    async fn dispatch(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress: Option<&mpsc::UnboundedSender<ProviderProgress>>,
        decision_only: bool,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        let started = Instant::now();
        let deadline = started
            .checked_add(Duration::from_secs(self.timeout.dispatch_timeout_secs))
            .ok_or_else(|| ProviderError::InvalidResponse("invalid Gemini timeout".into()))?;
        let body = crate::gemini_request::build_request_for_identity(
            &self.provider,
            &self.model,
            messages,
            tools,
            self.output_limit,
        )?;
        let transport = match &self.transport {
            Some(transport) => transport,
            None => {
                let profile = self.vertex.as_ref().ok_or_else(|| {
                    ProviderError::InvalidResponse("missing Gemini transport".into())
                })?;
                tokio::time::timeout_at(
                    deadline.into(),
                    profile.transport.get_or_try_init(|| async {
                        GeminiTransport::vertex_adc(
                            &profile.project,
                            &profile.location,
                            &self.model,
                            true,
                            profile.allow_metadata,
                            deadline,
                        )
                        .await
                    }),
                )
                .await
                .map_err(|_| {
                    ProviderError::InvalidResponse("Vertex initialization timed out".into())
                })??
            }
        };
        let mut attempt = 0;
        let (response, attempt_started) = loop {
            if Instant::now() >= deadline {
                return Err(ProviderError::InvalidResponse(
                    "Gemini dispatch timed out".into(),
                ));
            }
            if let Some(tx) = progress {
                let event = if attempt == 0 {
                    ProviderProgress::InitialDispatch {
                        attempt,
                        max_attempts: self.timeout.max_attempts,
                    }
                } else {
                    ProviderProgress::RetryDispatch {
                        attempt,
                        max_attempts: self.timeout.max_attempts,
                    }
                };
                let _ = tx.send(event);
            }
            let attempt_started = Instant::now();
            match transport.dispatch(&body, deadline).await {
                Ok(response) => break (response, attempt_started),
                Err(error) => match crate::retry::classify_retry_with_backoff(
                    &error,
                    attempt,
                    self.timeout.max_attempts,
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
                                max_attempts: self.timeout.max_attempts,
                                delay_ms,
                            });
                        }
                        tokio::time::timeout_at(
                            deadline.into(),
                            tokio::time::sleep(Duration::from_millis(delay_ms)),
                        )
                        .await
                        .map_err(|_| {
                            ProviderError::InvalidResponse("Gemini retry deadline exceeded".into())
                        })?;
                        attempt = next;
                    }
                },
            }
        };
        if let Some(tx) = progress {
            let _ = tx.send(ProviderProgress::FirstPacketWait {
                attempt,
                max_attempts: self.timeout.max_attempts,
            });
        }
        let first_deadline = attempt_started
            .checked_add(Duration::from_secs(self.timeout.first_packet_timeout_secs))
            .unwrap_or(deadline)
            .min(deadline);
        if Instant::now() >= first_deadline {
            return Err(ProviderError::InvalidResponse(
                "Gemini first event timed out".into(),
            ));
        }
        let first = first_deadline.saturating_duration_since(Instant::now());
        let idle = Duration::from_secs(self.timeout.stream_idle_timeout_secs);
        let provider = self.provider.clone();
        let model = self.model.clone();
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            let sink = InvocationSender::Legacy(tx.clone());
            if tx.send(AgentEvent::TurnStart).await.is_err() {
                return;
            }
            let (raw_tx, mut raw_rx) = mpsc::channel(32);
            let reader = async move {
                let result = crate::gemini_sse::read_response(
                    response, &provider, &model, &raw_tx, deadline, first, idle,
                )
                .await;
                drop(raw_tx);
                result
            };
            tokio::pin!(reader);
            let mut finished = false;
            let mut terminal = Vec::new();
            loop {
                tokio::select! {
                    _ = tx.closed() => return,
                    result = &mut reader, if !finished => {
                        finished = true;
                        if let Err(error) = result {
                            let _ = tx.try_send(AgentEvent::Error {message:error.to_string()});
                            return;
                        }
                    }
                    event = raw_rx.recv() => match event {
                        Some(GeminiOutput::Event(event @ (AgentEvent::TextDelta{..} | AgentEvent::ThinkingDelta{..}))) => {
                            if decision_only && matches!(event, AgentEvent::ThinkingDelta{..}) { continue; }
                            if Instant::now() >= deadline { return; }
                            if !matches!(tokio::time::timeout_at(deadline.into(), sink.send(event)).await, Ok(Ok(()))) { return; }
                        }
                        Some(event) => terminal.push(event),
                        None => {
                            // The reader drops its sender only after return. Poll its result first.
                            if !finished {
                                if let Err(error) = (&mut reader).await {
                                    let _ = tx.try_send(AgentEvent::Error{message:error.to_string()});
                                    return;
                                }
                            }
                            if decision_only && terminal.iter().any(|event| matches!(event,GeminiOutput::FunctionCall{..})) {
                                let _ = tx.try_send(AgentEvent::Error{message:"Gemini isolated decision returned a tool call".into()});
                                return;
                            }
                            for event in terminal {
                                if decision_only && matches!(event,GeminiOutput::Event(AgentEvent::ReasoningComplete{..})) { continue; }
                                if Instant::now() >= deadline { return; }
                                let sent = tokio::time::timeout_at(deadline.into(), async { match event {
                                    GeminiOutput::Event(event) => sink.send(event).await,
                                    GeminiOutput::FunctionCall{id,name,arguments} => sink.native_call(id,name,&arguments.to_string()).await,
                                }}).await;
                                if !matches!(sent, Ok(Ok(()))) { return; }
                            }
                            return;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }
}

#[async_trait::async_trait]
impl LanguageModel for GeminiProvider {
    async fn stream_decision(
        &self,
        messages: &[Message],
        limits: talos_core::provider::DecisionRequestLimits,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        crate::validate_decision_messages(messages, limits)?;
        let mut isolated = self.clone();
        isolated.output_limit = Some(limits.max_output_tokens);
        isolated.timeout.max_attempts = limits.max_retries.min(self.timeout.max_attempts);
        isolated.dispatch(messages, &[], None, true).await
    }

    fn protocol_capabilities(&self) -> CapabilityProbe {
        CapabilityProbe::Known(ProtocolCapabilities {
            native_tools: true,
            compatibility: false,
        })
    }
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.dispatch(messages, &[], None, false).await
    }
    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.dispatch(messages, tools, None, false).await
    }
    async fn stream_with_tools_and_progress(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        progress: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.dispatch(messages, tools, Some(&progress), false).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn isolated_decision_caps_output_without_mutating_conversation_settings() {
        use talos_core::provider::DecisionRequestLimits;
        let mut server = mockito::Server::new_async().await;
        let wire = format!(
            "data: {}\n\n",
            json!({"candidates":[{"content":{"parts":[{"text":"private-decision-thought","thought":true,"thoughtSignature":"private-decision-signature"},{"text":"yes"}]},"finishReason":"STOP"}]})
        );
        let decision = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(
                json!({"generationConfig":{"maxOutputTokens":17}}),
            ))
            .match_request(|request| {
                let body: serde_json::Value =
                    serde_json::from_slice(request.body().expect("body")).expect("JSON");
                body.get("tools").is_none()
                    && body["generationConfig"].get("thinkingConfig").is_none()
            })
            .with_body(wire.clone())
            .create_async()
            .await;
        let conversation = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::PartialJson(
                json!({"generationConfig":{"maxOutputTokens":100}}),
            ))
            .with_body(wire)
            .create_async()
            .await;
        let transport = GeminiTransport::ai_studio("model", "key", true)
            .expect("transport")
            .with_test_endpoint(server.url());
        let provider = GeminiProvider::new(
            transport,
            "google".into(),
            "model".into(),
            Some(100),
            Default::default(),
        )
        .expect("provider");
        let messages = [Message::User {
            content: "decide".into(),
        }];
        let mut events = provider
            .stream_auto_review(
                &messages,
                DecisionRequestLimits {
                    max_output_tokens: 17,
                    max_retries: 0,
                },
            )
            .await
            .expect("decision");
        let mut ended = false;
        while let Some(event) = events.recv().await {
            assert!(!matches!(
                event,
                AgentEvent::ToolCall { .. }
                    | AgentEvent::ReasoningComplete { .. }
                    | AgentEvent::ThinkingDelta { .. }
                    | AgentEvent::Error { .. }
            ));
            ended |= matches!(event, AgentEvent::TurnEnd { .. });
        }
        assert!(ended);
        let mut events = provider.stream(&messages).await.expect("conversation");
        while events.recv().await.is_some() {}
        decision.assert_async().await;
        conversation.assert_async().await;
    }

    #[tokio::test]
    async fn isolated_decision_refuses_tools_and_respects_zero_retries() {
        use talos_core::provider::DecisionRequestLimits;
        let mut server = mockito::Server::new_async().await;
        let wire = format!(
            "data: {}\n\n",
            json!({"candidates":[{"content":{"parts":[{"functionCall":{"name":"write","args":{}}}]},"finishReason":"STOP"}]})
        );
        let malicious = server
            .mock("POST", "/")
            .with_body(wire)
            .expect(1)
            .create_async()
            .await;
        let transport = GeminiTransport::ai_studio("model", "key", true)
            .expect("transport")
            .with_test_endpoint(server.url());
        let provider = GeminiProvider::new(
            transport,
            "google".into(),
            "model".into(),
            None,
            Default::default(),
        )
        .expect("provider");
        let messages = [Message::User {
            content: "decide".into(),
        }];
        let limits = DecisionRequestLimits {
            max_output_tokens: 17,
            max_retries: 0,
        };
        let mut events = provider
            .stream_decision(&messages, limits)
            .await
            .expect("decision");
        let mut failed = false;
        while let Some(event) = events.recv().await {
            assert!(!matches!(
                event,
                AgentEvent::ToolCall { .. }
                    | AgentEvent::TurnEnd { .. }
                    | AgentEvent::ReasoningComplete { .. }
            ));
            failed |= matches!(event, AgentEvent::Error { .. });
        }
        assert!(failed);
        malicious.assert_async().await;
        malicious.remove_async().await;
        let limited = server
            .mock("POST", "/")
            .with_status(429)
            .expect(1)
            .create_async()
            .await;
        assert!(matches!(
            provider.stream_decision(&messages, limits).await,
            Err(ProviderError::RateLimited(_))
        ));
        limited.assert_async().await;
        assert!(
            provider
                .stream_decision(
                    &[Message::Assistant {
                        content: "history".into(),
                        tool_calls: vec![],
                        reasoning: None
                    }],
                    limits
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn rate_limit_retries_are_bounded_and_report_progress() {
        let mut server = mockito::Server::new_async().await;
        let request = server
            .mock("POST", "/")
            .with_status(429)
            .expect(2)
            .create_async()
            .await;
        let transport = GeminiTransport::ai_studio("model", "key", true)
            .expect("transport")
            .with_test_endpoint(server.url());
        let config = ProviderTimeoutConfig {
            max_attempts: 1,
            backoff_base_ms: 1,
            backoff_max_ms: 1,
            ..ProviderTimeoutConfig::default()
        };
        let provider =
            GeminiProvider::new(transport, "google".into(), "model".into(), None, config)
                .expect("provider");
        let (tx, mut rx) = mpsc::unbounded_channel();
        let result = provider
            .stream_with_tools_and_progress(
                &[Message::User {
                    content: "hello".into(),
                }],
                &[],
                tx,
            )
            .await;
        assert!(matches!(result, Err(ProviderError::RateLimited(_))));
        assert!(matches!(
            rx.recv().await,
            Some(ProviderProgress::InitialDispatch { attempt: 0, .. })
        ));
        assert!(matches!(
            rx.recv().await,
            Some(ProviderProgress::ScheduledBackoff { attempt: 1, .. })
        ));
        assert!(matches!(
            rx.recv().await,
            Some(ProviderProgress::RetryDispatch { attempt: 1, .. })
        ));
        assert!(rx.recv().await.is_none());
        request.assert_async().await;
    }

    #[test]
    fn rejects_nonstreaming_or_mismatched_transport_before_dispatch() {
        for (stream, model) in [(false, "model"), (true, "other")] {
            let transport = GeminiTransport::ai_studio("model", "key", stream).expect("transport");
            assert!(
                GeminiProvider::new(
                    transport,
                    "google".into(),
                    model.into(),
                    None,
                    ProviderTimeoutConfig::default()
                )
                .is_err()
            );
        }
    }

    #[tokio::test]
    async fn stalled_consumer_does_not_release_tools_after_deadline() {
        let mut server = mockito::Server::new_async().await;
        let mut parts = (0..100).map(|_| json!({"text":"x"})).collect::<Vec<_>>();
        parts.push(json!({"functionCall":{"name":"lookup","args":{}}}));
        let wire = format!(
            "data: {}\n\n",
            json!({"candidates":[{"content":{"parts":parts},"finishReason":"STOP"}]})
        );
        let request = server
            .mock("POST", "/")
            .with_body(wire)
            .create_async()
            .await;
        let transport = GeminiTransport::ai_studio("model", "key", true)
            .expect("transport")
            .with_test_endpoint(server.url());
        let timeout = ProviderTimeoutConfig {
            dispatch_timeout_secs: 1,
            ..ProviderTimeoutConfig::default()
        };
        let provider =
            GeminiProvider::new(transport, "google".into(), "model".into(), None, timeout)
                .expect("identity");
        let mut events = provider
            .stream(&[Message::User {
                content: "hello".into(),
            }])
            .await
            .expect("stream");
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let drain = async {
            while let Some(event) = events.recv().await {
                assert!(!matches!(
                    event,
                    AgentEvent::ToolCall { .. }
                        | AgentEvent::TurnEnd { .. }
                        | AgentEvent::ReasoningComplete { .. }
                ));
            }
        };
        tokio::time::timeout(Duration::from_secs(1), drain)
            .await
            .expect("sender closed at deadline");
        request.assert_async().await;
    }

    #[tokio::test]
    async fn language_model_composes_real_http_signed_tools_and_terminal_usage() {
        let mut server = mockito::Server::new_async().await;
        let wire = format!(
            "data: {}\n\n",
            json!({"candidates":[{"content":{"parts":[
            {"functionCall":{"name":"lookup","args":{"q":1}},"thoughtSignature":"fixture-signature"}]},"finishReason":"STOP"}],
            "usageMetadata":{"promptTokenCount":7,"candidatesTokenCount":2}})
        );
        let request = server
            .mock("POST", "/")
            .match_header("x-goog-api-key", "fixture-key")
            .match_body(mockito::Matcher::PartialJson(
                json!({"contents":[{"role":"user","parts":[{"text":"hello"}]}]}),
            ))
            .with_body(wire)
            .create_async()
            .await;
        let transport = GeminiTransport::ai_studio("model", "fixture-key", true)
            .expect("transport")
            .with_test_endpoint(server.url());
        let provider = GeminiProvider::new(
            transport,
            "google".into(),
            "model".into(),
            Some(100),
            ProviderTimeoutConfig::default(),
        )
        .expect("provider identity");
        let mut events = provider
            .stream_with_tools(
                &[Message::User {
                    content: "hello".into(),
                }],
                &[ToolDefinition {
                    name: "lookup".into(),
                    description: "lookup".into(),
                    parameters: json!({"type":"object"}),
                }],
            )
            .await
            .expect("stream");
        let mut saw_replay = false;
        let mut saw_call = false;
        let mut saw_end = false;
        while let Some(event) = events.recv().await {
            match event {
                AgentEvent::ReasoningComplete { blocks } => {
                    assert!(!format!("{blocks:?}").contains("fixture-signature"));
                    saw_replay = true;
                }
                AgentEvent::ToolCall { call, .. } => {
                    assert!(saw_replay);
                    assert_eq!(call.name, "lookup");
                    saw_call = true;
                }
                AgentEvent::TurnEnd { usage, .. } => {
                    assert!(saw_call);
                    assert_eq!(usage.input_tokens, 7);
                    saw_end = true;
                }
                AgentEvent::Error { message } => panic!("unexpected error: {message}"),
                _ => {}
            }
        }
        assert!(saw_end);
        request.assert_async().await;
    }
}
