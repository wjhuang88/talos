//! Centralized construction of configured wire-protocol adapters.
//!
//! Hosts remain responsible for credential lookup, runtime model/variant materialization, mocks,
//! and local failure injection. This module owns only the protocol-to-adapter boundary so adding a
//! wire family does not require parallel construction logic in CLI and Desktop hosts.

use std::sync::Arc;

use talos_config::{Config, ProviderProtocol, resolve_config_provider_protocol};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};

use crate::{AnthropicProvider, OpenAIResponsesProvider, openai::OpenAIProvider};

/// Exact output-token limit shared by provider construction and request admission.
///
/// Models without an explicit configured limit receive the same conservative fallback used by the
/// existing CLI/Desktop hosts: one quarter of context, bounded to 4K..32K and never above context.
#[must_use]
pub fn effective_output_limit(config: &Config) -> u32 {
    config.output_limit().unwrap_or_else(|| {
        let (context_limit, _) = config.resolve_model_limits();
        (context_limit / 4).clamp(4096, 32_768).min(context_limit)
    })
}

/// Build the adapter selected by the effective wire-protocol resolver.
///
/// `config` is expected to have already gone through host-specific model/variant materialization.
/// Protocol selection itself is centralized here and never probes an endpoint or retries using a
/// different wire family.
pub fn build_configured_provider(
    config: &Config,
    api_key: &str,
) -> ProviderResult<Arc<dyn LanguageModel>> {
    let provider_config = config.active_provider_config();
    let model_config = provider_config.models.get(&config.model).cloned();
    let reasoning = model_config.and_then(|model| model.reasoning);
    let output_limit = Some(effective_output_limit(config));
    let timeout = provider_config.timeout;
    let base_url = config.base_url();
    let resolution = resolve_config_provider_protocol(config);

    match resolution.protocol {
        ProviderProtocol::AnthropicMessages => {
            let mut provider = AnthropicProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::OpenAIResponses => {
            let mut provider = OpenAIResponsesProvider::new(api_key, &config.model)
                .with_provider_key(&config.provider);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        _ => Err(ProviderError::InvalidResponse(format!(
            "unsupported provider protocol for {} / {}",
            config.provider, config.model
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{CapabilityProbe, ProtocolCapabilities};

    #[test]
    fn official_openai_default_constructs_responses_adapter() {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("packaged OpenAI model");
        let provider = build_configured_provider(&config, "sk-test").expect("provider builds");
        assert_eq!(
            provider.protocol_capabilities(),
            CapabilityProbe::Known(ProtocolCapabilities {
                native_tools: true,
                compatibility: false,
            })
        );
    }

    #[test]
    fn explicit_openai_chat_remains_a_rollback_override() {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("packaged OpenAI model");
        config
            .providers
            .entry("openai".into())
            .or_default()
            .protocol = ProviderProtocol::OpenAIChat;
        let provider = build_configured_provider(&config, "sk-test").expect("provider builds");
        assert_ne!(
            provider.protocol_capabilities(),
            CapabilityProbe::Known(ProtocolCapabilities {
                native_tools: true,
                compatibility: false,
            })
        );
    }
}
