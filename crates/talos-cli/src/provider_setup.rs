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
            Arc::new(provider)
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = talos_provider::openai::OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = config.base_url() {
                provider = provider.with_base_url(base_url);
            }
            Arc::new(provider)
        }
    }
}
