//! Provider construction and configuration helpers.

use std::sync::Arc;

use talos_config::{Config, McpConfig, ProviderProtocol};
use talos_mcp::types::{McpClientConfig, McpServerLaunchConfig};

pub(crate) fn config_to_mcp_client_config(config: &McpConfig) -> McpClientConfig {
    McpClientConfig {
        servers: config
            .servers
            .iter()
            .map(|s| McpServerLaunchConfig {
                name: s.name.clone(),
                transport: s.transport.clone(),
                command: s.command.clone(),
                args: s.args.clone(),
                env: s.env.clone(),
                cwd: s.cwd.clone(),
                url: s.url.clone(),
                sse_post_url: s.sse_post_url.clone(),
                headers: s.headers.clone(),
                auth_token_env: s.auth_token_env.clone(),
                authorization_env: s.authorization_env.clone(),
            })
            .collect(),
    }
}

pub(crate) fn parse_provider(s: &str) -> anyhow::Result<String> {
    let provider = s.trim().to_lowercase();
    if provider.is_empty() {
        anyhow::bail!("provider must be non-empty");
    }
    Ok(provider)
}

pub(crate) fn build_provider(
    config: &Config,
    api_key: &str,
    mock: bool,
) -> Arc<dyn talos_core::provider::LanguageModel> {
    let (runtime_config, resolution) =
        crate::model_lifecycle::materialize_runtime_model_config(config);
    if let Some(diagnostic) = resolution.diagnostic.as_deref() {
        tracing::warn!(
            provider = %config.provider,
            model = %config.model,
            variant = ?config.variant,
            "{diagnostic}"
        );
    }
    let config = &runtime_config;

    if mock {
        use talos_provider::mock::MockProvider;
        let api_key = api_key.to_string();
        let model = config.model.clone();
        let base_url = config.base_url();
        let provider_protocol = config.provider_protocol();
        return Arc::new(
            MockProvider::new().with_request_debug_builder(move |messages| {
                let snapshot = match &provider_protocol {
                    ProviderProtocol::AnthropicMessages => {
                        talos_provider::anthropic_request_debug_snapshot(
                            &api_key,
                            &model,
                            base_url.as_deref(),
                            messages,
                        )
                    }
                    ProviderProtocol::OpenAIChat => {
                        talos_provider::openai::openai_request_debug_snapshot(
                            &api_key,
                            &model,
                            base_url.as_deref(),
                            messages,
                        )
                    }
                };
                serde_json::to_string(&snapshot).unwrap_or_else(|_| snapshot.to_string())
            }),
        );
    }
    match config.provider_protocol() {
        ProviderProtocol::AnthropicMessages => {
            let mut provider = talos_provider::AnthropicProvider::new(api_key, &config.model);
            if let Some(base_url) = config.base_url() {
                provider = provider.with_base_url(base_url);
            }
            let provider_config = config.active_provider_config();
            let model_config = provider_config.models.get(&config.model).cloned();
            provider = provider.with_reasoning(
                model_config.as_ref().and_then(|m| m.reasoning.clone()),
                config.output_limit(),
            );
            provider = provider.with_timeout_config(provider_config.timeout.clone());
            Arc::new(provider)
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = talos_provider::openai::OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = config.base_url() {
                provider = provider.with_base_url(base_url);
            }
            let provider_config = config.active_provider_config();
            let model_config = provider_config.models.get(&config.model).cloned();
            provider = provider.with_reasoning(
                model_config.as_ref().and_then(|m| m.reasoning.clone()),
                config.output_limit(),
            );
            provider = provider.with_timeout_config(provider_config.timeout.clone());
            Arc::new(provider)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::Message;

    fn openai_o3_config(variant: Option<&str>) -> Config {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut config, variant);
        config
    }

    fn reasoning_effort_from_real_request(config: &Config) -> Option<String> {
        let provider = build_provider(config, "sk-test-materialization", false);
        let preview = provider
            .request_preview(&[Message::User {
                content: "runtime materialization probe".to_string(),
            }])
            .expect("real provider request preview");
        preview
            .pointer("/body/reasoning_effort")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    #[test]
    fn provider_build_materializes_variant_reasoning_into_real_request() {
        let config = openai_o3_config(Some("high-reasoning"));
        assert_eq!(
            reasoning_effort_from_real_request(&config).as_deref(),
            Some("high")
        );
    }

    #[test]
    fn provider_build_normalizes_default_to_baseline_request_options() {
        let baseline = openai_o3_config(None);
        let explicit_default = openai_o3_config(Some("DEFAULT"));
        assert_eq!(explicit_default.variant, None);
        assert_eq!(
            reasoning_effort_from_real_request(&explicit_default),
            reasoning_effort_from_real_request(&baseline)
        );
    }

    #[test]
    fn provider_build_uses_safe_fallback_for_unknown_variant() {
        let baseline = openai_o3_config(None);
        let unknown = openai_o3_config(Some("deleted-variant"));
        let (_, resolution) = crate::model_lifecycle::materialize_runtime_model_config(&unknown);
        assert!(resolution.diagnostic.is_some());
        assert_eq!(
            reasoning_effort_from_real_request(&unknown),
            reasoning_effort_from_real_request(&baseline)
        );
    }
}
