//! Public contract tests for the OpenAI Responses provider and centralized factory.

use talos_config::Config;
use talos_core::provider::{DecisionRequestLimits, LanguageModel, ProviderError};
use talos_core::tool::{CapabilityProbe, ProtocolCapabilities, ToolProtocol};
use talos_provider::{OpenAIResponsesProvider, openai_responses::factory};
use tokio::sync::mpsc;

#[test]
fn public_provider_declares_native_tool_evidence() {
    let provider = OpenAIResponsesProvider::new("sk-test", "gpt-test");
    assert_eq!(
        provider.protocol_capabilities(),
        CapabilityProbe::Known(ProtocolCapabilities {
            native_tools: true,
            compatibility: true,
        })
    );
    assert_eq!(
        provider.protocol_capabilities().select(),
        ToolProtocol::Native
    );
}

#[tokio::test]
async fn public_provider_rejects_unresolved_auto_before_network_dispatch() {
    let provider = OpenAIResponsesProvider::new("sk-test", "gpt-test");
    {
        let protocol = ToolProtocol::Auto;
        let (progress, _rx) = mpsc::unbounded_channel();
        let error = provider
            .stream_with_protocol(&[], &[], protocol, progress)
            .await
            .expect_err("non-native protocol must fail before dispatch");
        assert!(matches!(error, ProviderError::InvalidResponse(_)));
    }
}

#[tokio::test]
async fn public_provider_rejects_invalid_decision_before_network_dispatch() {
    let provider = OpenAIResponsesProvider::new("sk-test", "gpt-test");
    let limits = DecisionRequestLimits {
        max_output_tokens: 0,
        max_retries: 0,
    };
    assert!(provider.stream_decision(&[], limits).await.is_err());
    assert!(provider.stream_auto_review(&[], limits).await.is_err());
}

#[test]
fn public_factory_builds_responses_for_confirmed_official_openai_default() {
    let mut config = Config::default();
    config
        .set_active_model("openai/o3")
        .expect("packaged OpenAI model");

    let provider =
        factory::build_configured_provider(&config, "sk-test").expect("configured provider builds");
    assert_eq!(
        provider.protocol_capabilities(),
        CapabilityProbe::Known(ProtocolCapabilities {
            native_tools: true,
            compatibility: true,
        })
    );
}
