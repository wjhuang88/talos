//! Configuration-backed provider construction for the Desktop host.

use std::sync::Arc;

use talos_config::{Config, ProviderProtocol};
use talos_runtime::LanguageModel;

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
            Ok(Arc::new(provider))
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
            Ok(Arc::new(provider))
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
}
