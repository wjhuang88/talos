//! Configuration-backed provider construction for the Desktop host.

use std::sync::Arc;

use talos_config::{Config, ProviderProtocol};
#[cfg(any(debug_assertions, test))]
use talos_core::message::{AgentEvent, Message};
#[cfg(any(debug_assertions, test))]
use talos_core::provider::{ProviderProgress, ToolDefinition};
#[cfg(any(debug_assertions, test))]
use talos_core::tool::ToolProtocol;
#[cfg(not(any(debug_assertions, test)))]
use talos_runtime::LanguageModel;
#[cfg(any(debug_assertions, test))]
use talos_runtime::{LanguageModel, ProviderError, ProviderResult, Receiver};
#[cfg(any(debug_assertions, test))]
use tokio::sync::mpsc;

/// Explicit local-only provider failure injection used by the Desktop H1 acceptance path.
///
/// This wrapper is opt-in through `TALOS_DESKTOP_PROVIDER_FAILURE` and is never enabled by
/// normal configuration. It does not mutate credentials or touch the network.
#[derive(Clone, Copy)]
#[cfg(any(debug_assertions, test))]
pub(crate) enum FailureInjection {
    Error,
    Timeout(std::time::Duration),
}

#[cfg(any(debug_assertions, test))]
pub(crate) struct FailureInjectedProvider {
    pub(crate) mode: FailureInjection,
}

#[cfg(any(debug_assertions, test))]
impl FailureInjectedProvider {
    fn from_environment() -> Option<Arc<dyn LanguageModel>> {
        let mode = match std::env::var("TALOS_DESKTOP_PROVIDER_FAILURE")
            .ok()
            .as_deref()
        {
            Some("error") => FailureInjection::Error,
            Some("timeout") => FailureInjection::Timeout(std::time::Duration::from_secs(5)),
            _ => return None,
        };
        Some(Arc::new(Self { mode }))
    }
}

#[cfg(any(debug_assertions, test))]
#[async_trait::async_trait]
impl LanguageModel for FailureInjectedProvider {
    async fn stream(&self, _messages: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        match self.mode {
            FailureInjection::Error => Err(ProviderError::NetworkError(
                "Desktop acceptance provider failure injection".into(),
            )),
            FailureInjection::Timeout(delay) => {
                let (sender, receiver) = mpsc::channel(1);
                tokio::spawn(async move {
                    tokio::select! {
                        _ = sender.closed() => {}
                        _ = tokio::time::sleep(delay) => {
                            let _ = sender.send(AgentEvent::Error {
                                message: "first-packet timeout: injected Desktop acceptance failure".into(),
                            }).await;
                        }
                    }
                });
                Ok(receiver)
            }
        }
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
    ) -> ProviderResult<Receiver<AgentEvent>> {
        self.stream(messages).await
    }

    async fn stream_with_protocol(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
        _protocol: ToolProtocol,
        progress_tx: mpsc::UnboundedSender<ProviderProgress>,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        drop(progress_tx);
        self.stream(messages).await
    }
}

#[cfg(any(debug_assertions, test))]
fn maybe_inject_provider(provider: Arc<dyn LanguageModel>) -> Arc<dyn LanguageModel> {
    FailureInjectedProvider::from_environment().unwrap_or(provider)
}

#[cfg(not(any(debug_assertions, test)))]
fn maybe_inject_provider(provider: Arc<dyn LanguageModel>) -> Arc<dyn LanguageModel> {
    provider
}

/// Builds the configured provider without creating an async runtime.
pub(crate) fn configured_provider(config: &Config) -> Result<Arc<dyn LanguageModel>, String> {
    let (config, _resolution) = talos_config::variant::materialize_runtime_model_config(config);
    if config.provider.trim().is_empty() || config.model.trim().is_empty() {
        return Err("provider setup is incomplete; configure a provider and model first".into());
    }
    let api_key = config.api_key().map_err(|error| error.to_string())?;
    let provider_config = config.active_provider_config();
    let model_config = provider_config.models.get(&config.model).cloned();
    let context_limit = config.resolve_model_limits().0;
    let output_limit = Some(
        config
            .output_limit()
            .unwrap_or_else(|| (context_limit / 4).clamp(4096, 32_768).min(context_limit)),
    );

    match config.provider_protocol() {
        ProviderProtocol::AnthropicMessages => {
            let mut provider = talos_provider::AnthropicProvider::new(api_key, &config.model);
            if let Some(base_url) = config.base_url() {
                provider = provider.with_base_url(base_url);
            }
            provider = provider.with_reasoning(
                model_config
                    .as_ref()
                    .and_then(|model| model.reasoning.clone()),
                output_limit,
            );
            provider = provider.with_timeout_config(provider_config.timeout);
            Ok(maybe_inject_provider(Arc::new(provider)))
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = talos_provider::openai::OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = config.base_url() {
                provider = provider.with_base_url(base_url);
            }
            provider = provider.with_reasoning(
                model_config
                    .as_ref()
                    .and_then(|model| model.reasoning.clone()),
                output_limit,
            );
            provider = provider.with_timeout_config(provider_config.timeout);
            Ok(maybe_inject_provider(Arc::new(provider)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_variant_reaches_the_actual_provider_request() {
        let mut config = Config::default();
        config.set_active_model("openai/o3").expect("catalog model");
        config.variant = Some("high-reasoning".into());
        config.providers.entry("openai".into()).or_default().api_key =
            Some("test-only-fixture".into());
        let provider = configured_provider(&config).expect("provider builds");
        let preview = provider
            .request_preview(&[talos_runtime::Message::User {
                content: "probe".into(),
            }])
            .expect("preview supported");
        assert_eq!(
            preview
                .pointer("/body/reasoning_effort")
                .and_then(|v| v.as_str()),
            Some("high")
        );
        assert!(
            config.providers["openai"]
                .models
                .get("o3")
                .and_then(|m| m.reasoning.as_ref())
                .is_none()
        );
    }

    #[test]
    fn incomplete_config_is_rejected_before_credentials_are_read() {
        let config = Config::default();
        let error = configured_provider(&config)
            .err()
            .expect("empty config must fail");
        assert!(error.contains("provider setup is incomplete"));
    }

    #[tokio::test]
    async fn failure_injection_surfaces_provider_error_without_network() {
        let provider = FailureInjectedProvider {
            mode: FailureInjection::Error,
        };
        let error = provider.stream(&[]).await.expect_err("injected error");
        assert!(
            error
                .to_string()
                .contains("Desktop acceptance provider failure injection")
        );
    }

    #[tokio::test]
    async fn timeout_injection_surfaces_bounded_timeout_event() {
        let provider = FailureInjectedProvider {
            mode: FailureInjection::Timeout(std::time::Duration::from_millis(100)),
        };
        let mut events = provider.stream(&[]).await.expect("injected stream");
        assert!(matches!(
            events.recv().await,
            Some(AgentEvent::Error { message }) if message.contains("first-packet timeout")
        ));
    }

    #[tokio::test]
    async fn injection_reaches_selected_protocol_dispatch() {
        let provider = FailureInjectedProvider {
            mode: FailureInjection::Error,
        };
        let (progress, _receiver) = mpsc::unbounded_channel();
        let error = provider
            .stream_with_protocol(&[], &[], ToolProtocol::Compat, progress)
            .await
            .expect_err("injected error must bypass protocol selection");
        assert!(matches!(error, ProviderError::NetworkError(_)));
    }
}
