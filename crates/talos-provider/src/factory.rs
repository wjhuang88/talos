//! Centralized construction of configured wire-protocol adapters.
//!
//! Hosts remain responsible for credential lookup, runtime model/variant materialization, mocks,
//! and local failure injection. This module owns only the protocol-to-adapter boundary so adding a
//! wire family does not require parallel construction logic in CLI and Desktop hosts.

use std::sync::Arc;

use talos_config::{Config, ProviderProtocol, resolve_config_provider_protocol};
use talos_core::provider::{LanguageModel, ProviderError, ProviderResult};

use crate::{AnthropicProvider, OpenAIResponsesProvider, openai::OpenAIProvider};

#[cfg(test)]
#[path = "factory_copilot_tests.rs"]
mod copilot_tests;

/// Exact output-token limit shared by provider construction and request admission.
///
/// Models without an explicit configured limit receive the same conservative fallback used by the
/// existing CLI/Desktop hosts: one quarter of context, bounded to 4K..32K and never above context.
#[must_use]
pub fn effective_output_limit(config: &Config) -> u32 {
    config.resolve_model_limits().1.unwrap_or_else(|| {
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
    build_with_connection(config, api_key, crate::copilot::connection)
}

// The production entry point always supplies the origin-checked Copilot connection builder.
// Keeping this composition private also lets tests route its real headers through a local server.
fn build_with_connection(
    config: &Config,
    api_key: &str,
    connect: impl FnOnce(
        Option<&str>,
        &ProviderProtocol,
    ) -> ProviderResult<Option<crate::copilot::CopilotConnection>>,
) -> ProviderResult<Arc<dyn LanguageModel>> {
    let provider_config = config.active_provider_config();
    let model_config = provider_config.models.get(&config.model).cloned();
    let reasoning_disabled = config.resolve_model_fields().reasoning_supported.value == Some(false);
    let replay_disabled = model_config
        .as_ref()
        .and_then(|model| model.reasoning.as_ref())
        .is_some_and(|options| !options.replay);
    let reasoning = model_config
        .and_then(|model| model.reasoning)
        .filter(|_| !reasoning_disabled);
    let output_limit = Some(effective_output_limit(config));
    let timeout = provider_config.timeout;
    let endpoint = config
        .resolve_provider_endpoint()
        .map_err(|error| ProviderError::InvalidResponse(error.to_string()))?;
    let mut base_url = endpoint.base_url;
    // A manually configured Copilot model can omit its URL. Its credential must stay at the
    // selected packaged provider's root, never at the generic wire adapter's official vendor.
    // This selects a reviewed provider default, not a wire family or a custom model's vendor.
    if base_url.is_none() && config.provider == "github-copilot" {
        base_url = Some(
            talos_config::model::builtin_providers()
                .into_iter()
                .find(|provider| provider.id == config.provider)
                .and_then(|provider| provider.api_base_url)
                .ok_or_else(|| {
                    ProviderError::InvalidResponse(
                        "Copilot provider default is unavailable; configure its endpoint".into(),
                    )
                })?,
        );
    }
    let resolution = resolve_config_provider_protocol(config);
    // Preserve the original full wire path and explicit model URL precedence for validation.
    let original_endpoint = config.configured_endpoint_url();
    let copilot = connect(
        original_endpoint.as_deref().or(base_url.as_deref()),
        &resolution.protocol,
    )?;
    if let Some(connection) = &copilot {
        base_url = Some(connection.base_url.clone());
    }

    tracing::debug!(
        provider = %config.provider,
        model = %config.model,
        effective_protocol = ?resolution.protocol,
        protocol_source = ?resolution.source,
        endpoint_source = ?endpoint.source,
        "resolved provider wire protocol"
    );

    match resolution.protocol {
        ProviderProtocol::AnthropicMessages => {
            let mut provider = AnthropicProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                // Mixed-protocol catalogs advertise an API root; this adapter consumes a full
                // Messages endpoint. Preserve existing full endpoints and append only once.
                let base_url = base_url.trim().trim_end_matches('/');
                let endpoint = if base_url.ends_with("/messages") {
                    base_url.to_string()
                } else {
                    format!("{base_url}/messages")
                };
                provider = provider.with_base_url(endpoint);
            }
            if let Some(connection) = copilot {
                provider =
                    provider.with_copilot_client(connection.client, connection.preview_headers);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_reasoning_disabled(reasoning_disabled)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::OpenAIChat => {
            let mut provider = OpenAIProvider::new(api_key, &config.model);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            if let Some(connection) = copilot {
                provider =
                    provider.with_copilot_client(connection.client, connection.preview_headers);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_reasoning_disabled(reasoning_disabled)
                .with_timeout_config(timeout);
            Ok(Arc::new(provider))
        }
        ProviderProtocol::GoogleGemini => {
            use talos_config::GeminiTransportConfig;
            if replay_disabled {
                return Err(ProviderError::InvalidResponse(
                    "Gemini requires native signature replay; replay=false is unsupported".into(),
                ));
            }
            let profile = provider_config.gemini.as_ref().ok_or_else(|| {
                ProviderError::InvalidResponse(
                    "google-gemini requires an explicit gemini transport profile".into(),
                )
            })?;
            let provider = match profile {
                GeminiTransportConfig::AiStudio => {
                    if base_url.as_deref().is_some_and(|base| {
                        !matches!(
                            base.trim_end_matches('/'),
                            "https://generativelanguage.googleapis.com"
                                | "https://generativelanguage.googleapis.com/v1beta"
                        )
                    }) {
                        return Err(ProviderError::InvalidResponse(
                            "custom Gemini destinations require a separate credential profile"
                                .into(),
                        ));
                    }
                    let transport = crate::gemini_transport::GeminiTransport::ai_studio(
                        &config.model,
                        api_key,
                        true,
                    )?;
                    crate::gemini_provider::GeminiProvider::new(
                        transport,
                        config.provider.clone(),
                        config
                            .model
                            .strip_prefix("models/")
                            .unwrap_or(&config.model)
                            .to_owned(),
                        output_limit,
                        timeout,
                    )?
                }
                GeminiTransportConfig::Vertex {
                    project,
                    location,
                    allow_metadata,
                } => {
                    if original_endpoint.is_some() {
                        return Err(ProviderError::InvalidResponse(
                            "Vertex ADC refuses replacement endpoints".into(),
                        ));
                    }
                    crate::gemini_provider::GeminiProvider::vertex(
                        project.clone(),
                        location.clone(),
                        *allow_metadata,
                        config.provider.clone(),
                        config.model.clone(),
                        output_limit,
                        timeout,
                    )?
                }
            };
            Ok(Arc::new(
                provider
                    .with_reasoning(reasoning.as_ref())?
                    .with_reasoning_disabled(reasoning_disabled),
            ))
        }
        ProviderProtocol::OpenAIResponses => {
            let mut provider = OpenAIResponsesProvider::new(api_key, &config.model)
                .with_provider_key(&config.provider);
            if let Some(base_url) = base_url {
                provider = provider.with_base_url(base_url);
            }
            if let Some(connection) = copilot {
                provider =
                    provider.with_copilot_client(connection.client, connection.preview_headers);
            }
            provider = provider
                .with_reasoning(reasoning, output_limit)
                .with_reasoning_disabled(reasoning_disabled)
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
    fn gemini_factory_preview_preserves_configured_wire_and_masks_credentials() {
        use talos_config::{GeminiTransportConfig, ModelConfig, ProviderConfig, ReasoningOptions};
        for profile in [
            GeminiTransportConfig::AiStudio,
            GeminiTransportConfig::Vertex {
                project: "fixture-project".into(),
                location: "global".into(),
                allow_metadata: false,
            },
        ] {
            let vertex = matches!(&profile, GeminiTransportConfig::Vertex { .. });
            let config = Config {
                provider: "fixture-google".into(),
                model: "gemini-2.5-flash".into(),
                providers: std::collections::HashMap::from([(
                    "fixture-google".into(),
                    ProviderConfig {
                        protocol: ProviderProtocol::GoogleGemini,
                        gemini: Some(profile),
                        api_key: (!vertex).then(|| "private-fixture-key".into()),
                        models: std::collections::HashMap::from([(
                            "gemini-2.5-flash".into(),
                            ModelConfig {
                                output_limit: Some(2048),
                                reasoning_supported: Some(true),
                                reasoning: Some(ReasoningOptions {
                                    budget_tokens: Some(1024),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            config.validate().expect("explicit profile");
            let key = config.api_key().expect("host credential admission");
            let provider = build_configured_provider(&config, &key).expect("host factory");
            let preview = provider
                .request_preview(&[talos_core::message::Message::User {
                    content: "visible prompt".into(),
                }])
                .expect("host diagnostic preview");
            assert!(preview.get("error").is_none(), "{preview}");
            assert_eq!(preview["body"]["generationConfig"]["maxOutputTokens"], 2048);
            assert_eq!(
                preview["body"]["generationConfig"]["thinkingConfig"]["thinkingBudget"],
                1024
            );
            assert_eq!(
                preview["body"]["contents"][0]["parts"][0]["text"],
                "visible prompt"
            );
            let url = preview["url"].as_str().expect("wire URL");
            if vertex {
                assert!(url.starts_with("https://aiplatform.googleapis.com/v1/projects/fixture-project/locations/global/"));
                assert_eq!(preview["headers"]["Authorization"], "Bearer ***");
                assert!(preview["headers"].get("x-goog-api-key").is_none());
            } else {
                assert!(
                    url.starts_with("https://generativelanguage.googleapis.com/v1beta/models/")
                );
                assert_eq!(preview["headers"]["x-goog-api-key"], "***");
                assert!(preview["headers"].get("Authorization").is_none());
            }
            assert!(url.ends_with(":streamGenerateContent?alt=sse"));
            assert!(!preview.to_string().contains("private-fixture-key"));
        }
    }

    #[test]
    fn vertex_factory_defers_isolated_adc_and_never_masks_invalid_explicit_file() {
        let directory = tempfile::tempdir().expect("fixture directory");
        let path = directory.path().join("private-fixture-account.json");
        std::fs::write(&path, b"invalid explicit ADC fixture").expect("fixture");
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .arg("--exact")
            .arg("openai_responses::factory::tests::vertex_factory_subprocess_probe")
            .arg("--nocapture")
            .env("TALOS_GEMINI_ADC_CHILD", "1")
            .env("GOOGLE_APPLICATION_CREDENTIALS", &path)
            .env_remove("HOME")
            .env_remove("APPDATA")
            .env("GCE_METADATA_HOST", "127.0.0.1:9")
            .output()
            .expect("isolated subprocess");
        assert!(
            output.status.success(),
            "Vertex factory subprocess failed: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }

    #[tokio::test]
    async fn vertex_factory_subprocess_probe() {
        if std::env::var_os("TALOS_GEMINI_ADC_CHILD").is_none() {
            return;
        }
        let config = Config {
            provider: "fixture-vertex".into(),
            model: "fixture-model".into(),
            providers: std::collections::HashMap::from([(
                "fixture-vertex".into(),
                talos_config::ProviderConfig {
                    protocol: ProviderProtocol::GoogleGemini,
                    gemini: Some(talos_config::GeminiTransportConfig::Vertex {
                        project: "fixture-project".into(),
                        location: "global".into(),
                        allow_metadata: true,
                    }),
                    timeout: talos_config::ProviderTimeoutConfig {
                        dispatch_timeout_secs: 1,
                        max_attempts: 0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        config
            .validate()
            .expect("valid ADC config without key or home");
        let key = config.api_key().expect("ADC host admission");
        assert!(key.is_empty());
        // Factory construction must not inspect even an invalid selected credential file.
        let provider = build_configured_provider(&config, &key).expect("lazy factory");
        let result = provider
            .stream(&[talos_core::message::Message::User {
                content: "hello".into(),
            }])
            .await;
        let message = match result {
            Ok(_) => panic!("invalid explicit ADC unexpectedly admitted"),
            Err(error) => error.to_string(),
        };
        assert!(message.contains("invalid or unsupported Vertex credentials"));
        assert!(!message.contains("private-fixture-account"));
        assert!(!message.contains("invalid explicit ADC fixture"));
    }

    #[test]
    fn gemini_reasoning_factory_validates_before_lazy_vertex_credentials() {
        use talos_config::{
            GeminiTransportConfig, ModelConfig, ProviderConfig, ReasoningEffort, ReasoningOptions,
        };
        let mut config = Config {
            provider: "fixture-google".into(),
            model: "gemini-2.5-flash".into(),
            providers: std::collections::HashMap::from([(
                "fixture-google".into(),
                ProviderConfig {
                    protocol: ProviderProtocol::GoogleGemini,
                    gemini: Some(GeminiTransportConfig::Vertex {
                        project: "fixture-project".into(),
                        location: "global".into(),
                        allow_metadata: false,
                    }),
                    models: std::collections::HashMap::from([(
                        "gemini-2.5-flash".into(),
                        ModelConfig {
                            reasoning_supported: Some(true),
                            reasoning: Some(ReasoningOptions {
                                budget_tokens: Some(1024),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        // A valid reasoning configuration does not require opening an ADC file in the factory.
        assert!(build_configured_provider(&config, "").is_ok());
        let model = config
            .providers
            .get_mut("fixture-google")
            .expect("provider")
            .models
            .get_mut("gemini-2.5-flash")
            .expect("model");
        model.reasoning = Some(ReasoningOptions {
            effort: Some(ReasoningEffort::Low),
            ..Default::default()
        });
        let error = match build_configured_provider(&config, "") {
            Ok(_) => panic!("unsupported effort admitted"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("requires a thinking budget"));
        config
            .providers
            .get_mut("fixture-google")
            .expect("provider")
            .models
            .get_mut("gemini-2.5-flash")
            .expect("model")
            .reasoning_supported = Some(false);
        assert!(
            build_configured_provider(&config, "").is_ok(),
            "explicit unsupported capability suppresses controls"
        );
        config
            .providers
            .get_mut("fixture-google")
            .expect("provider")
            .models
            .get_mut("gemini-2.5-flash")
            .expect("model")
            .reasoning
            .as_mut()
            .expect("options")
            .replay = false;
        assert!(
            build_configured_provider(&config, "").is_err(),
            "capability suppression must not silently ignore replay=false"
        );
    }

    #[test]
    fn explicit_gemini_profiles_build_without_ambient_adc_and_refuse_relocation() {
        use talos_config::{GeminiTransportConfig, ProviderConfig};
        let mut config = Config {
            provider: "fixture-google".into(),
            model: "gemini-2.5-flash".into(),
            providers: std::collections::HashMap::from([(
                "fixture-google".into(),
                ProviderConfig {
                    protocol: ProviderProtocol::GoogleGemini,
                    gemini: Some(GeminiTransportConfig::AiStudio),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(build_configured_provider(&config, "fixture-key").is_ok());
        config
            .providers
            .get_mut("fixture-google")
            .expect("provider")
            .base_url = Some("https://custom.invalid/v1beta".into());
        assert!(build_configured_provider(&config, "fixture-key").is_err());
        let entry = config
            .providers
            .get_mut("fixture-google")
            .expect("provider");
        entry.base_url = None;
        entry.gemini = Some(GeminiTransportConfig::Vertex {
            project: "fixture-project".into(),
            location: "global".into(),
            allow_metadata: false,
        });
        // No files/env/metadata are consulted until dispatch.
        assert!(matches!(
            config.active_provider_config().gemini,
            Some(GeminiTransportConfig::Vertex { .. })
        ));
        assert!(config.validate().is_ok());
        assert_eq!(config.api_key().expect("ADC needs no API key"), "");
        assert!(build_configured_provider(&config, "").is_ok());
        config
            .providers
            .get_mut("fixture-google")
            .expect("provider")
            .base_url = Some("https://custom.invalid".into());
        assert!(build_configured_provider(&config, "").is_err());
        let entry = config
            .providers
            .get_mut("fixture-google")
            .expect("provider");
        entry.base_url = None;
        entry.gemini = None;
        assert!(build_configured_provider(&config, "fixture-key").is_err());
    }

    #[test]
    fn model014_direct_factory_suppresses_disabled_reasoning_parameters() {
        for (protocol, field) in [
            (ProviderProtocol::OpenAIChat, "reasoning_effort"),
            (ProviderProtocol::OpenAIResponses, "reasoning"),
            (ProviderProtocol::AnthropicMessages, "thinking"),
        ] {
            let config = Config {
                provider: "gateway".into(),
                model: "unknown-fixture".into(),
                providers: std::collections::HashMap::from([(
                    "gateway".into(),
                    talos_config::ProviderConfig {
                        protocol,
                        base_url: Some("https://gateway.test/v1".into()),
                        models: std::collections::HashMap::from([(
                            "unknown-fixture".into(),
                            talos_config::ModelConfig {
                                reasoning_supported: Some(false),
                                reasoning: Some(talos_config::ReasoningOptions::default()),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let provider = build_configured_provider(&config, "fixture-key").expect("provider");
            let history = vec![talos_core::message::Message::Assistant {
                content: "visible answer".into(),
                tool_calls: vec![],
                reasoning: Some(talos_core::message::AssistantReasoning {
                    provider: "gateway".into(),
                    model: "unknown-fixture".into(),
                    blocks: vec![talos_core::message::ReasoningBlock::Plain {
                        text: "private-replay-sentinel".into(),
                    }],
                }),
            }];
            let before = serde_json::to_value(&history).expect("history");
            let preview = provider.request_preview(&history).expect("preview");
            assert!(preview["body"].get(field).is_none(), "{preview}");
            assert!(
                !preview["body"]
                    .to_string()
                    .contains("private-replay-sentinel")
            );
            assert_eq!(serde_json::to_value(&history).expect("history"), before);
            assert!(
                config.providers["gateway"].models["unknown-fixture"]
                    .reasoning
                    .is_some()
            );
        }
    }

    #[tokio::test]
    async fn model014_disabled_reasoning_http_preserves_tool_pairing_and_raw_history() {
        use serde_json::{Value, json};
        use talos_core::message::{
            AssistantReasoning, Message, MessageToolResult, ReasoningBlock, ToolCall,
        };
        for (protocol, path, field, output_field) in [
            (
                ProviderProtocol::OpenAIChat,
                "/chat/completions",
                "reasoning_effort",
                "max_completion_tokens",
            ),
            (
                ProviderProtocol::OpenAIResponses,
                "/responses",
                "reasoning",
                "max_output_tokens",
            ),
            (
                ProviderProtocol::AnthropicMessages,
                "/messages",
                "thinking",
                "max_tokens",
            ),
        ] {
            let mut server = mockito::Server::new_async().await;
            let config = Config {
                provider: "gateway".into(),
                model: "unknown-fixture".into(),
                providers: std::collections::HashMap::from([(
                    "gateway".into(),
                    talos_config::ProviderConfig {
                        protocol,
                        base_url: Some(server.url()),
                        models: std::collections::HashMap::from([(
                            "unknown-fixture".into(),
                            talos_config::ModelConfig {
                                reasoning_supported: Some(false),
                                reasoning: Some(talos_config::ReasoningOptions::default()),
                                output_limit: Some(1234),
                                ..Default::default()
                            },
                        )]),
                        timeout: talos_config::ProviderTimeoutConfig {
                            max_attempts: 0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let encrypted = crate::openai_responses_reasoning::encode_completed_reasoning_item(&json!({
                "type": "reasoning", "id": "rs_fixture", "summary": [], "encrypted_content": "private-cipher-sentinel"
            })).expect("encode").expect("encrypted item");
            let history = vec![
                Message::User {
                    content: "run probe".into(),
                },
                Message::Assistant {
                    content: "visible answer".into(),
                    tool_calls: vec![ToolCall {
                        id: "call_fixture".into(),
                        name: "probe".into(),
                        input: json!({}),
                    }],
                    reasoning: Some(AssistantReasoning {
                        provider: "gateway".into(),
                        model: "unknown-fixture".into(),
                        blocks: vec![
                            ReasoningBlock::Plain {
                                text: "private-plain-sentinel".into(),
                            },
                            ReasoningBlock::Thinking {
                                text: "private-thinking-sentinel".into(),
                                signature: Some("private-signature-sentinel".into()),
                            },
                            ReasoningBlock::Redacted { data: encrypted },
                        ],
                    }),
                },
                Message::Tool {
                    result: MessageToolResult {
                        tool_use_id: "call_fixture".into(),
                        content: "probe-result".into(),
                        is_error: false,
                    },
                },
            ];
            let before = serde_json::to_value(&history).expect("history");
            let mock = server
                .mock("POST", path)
                .match_request(move |request| {
                    let body: Value =
                        serde_json::from_slice(request.body().expect("body")).expect("json");
                    let wire = body.to_string();
                    body.get(field).is_none()
                        && body[output_field] == 1234
                        && !wire.contains("private-")
                        && wire.contains("visible answer")
                        && wire.contains("probe-result")
                        && wire.matches("call_fixture").count() == 2
                })
                .with_status(200)
                .with_header("content-type", "text/event-stream")
                .with_body("")
                .expect(1)
                .create_async()
                .await;
            let provider = build_configured_provider(&config, "fixture-key").expect("provider");
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                let mut events = provider.stream(&history).await.expect("request accepted");
                while events.recv().await.is_some() {}
            })
            .await
            .expect("bounded request");
            mock.assert_async().await;
            assert_eq!(serde_json::to_value(&history).expect("history"), before);
        }
    }

    #[tokio::test]
    async fn model014_negative_override_refuses_glm_auto_review_before_dispatch() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .expect(0)
            .create_async()
            .await;
        let config = Config {
            provider: "gateway".into(),
            model: "glm-5.3".into(),
            providers: std::collections::HashMap::from([(
                "gateway".into(),
                talos_config::ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    base_url: Some(server.url()),
                    models: std::collections::HashMap::from([(
                        "glm-5.3".into(),
                        talos_config::ModelConfig {
                            reasoning_supported: Some(false),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let provider = build_configured_provider(&config, "fixture-key").expect("provider");
        let error = provider
            .stream_auto_review(
                &[talos_core::message::Message::User {
                    content: "review this request".into(),
                }],
                talos_core::provider::DecisionRequestLimits {
                    max_output_tokens: 128,
                    max_retries: 0,
                },
            )
            .await
            .expect_err("explicit negative must not be overridden by Auto");
        assert!(
            error
                .to_string()
                .contains("reasoning capability is disabled"),
            "{error}"
        );
        mock.assert_async().await;
    }

    #[test]
    fn model014_output_ceiling_is_sent_without_reasoning() {
        let catalog = talos_config::model::builtin_models();
        let model = catalog
            .iter()
            .find(|model| {
                model.output_limit.is_some()
                    && catalog
                        .iter()
                        .filter(|candidate| candidate.id == model.id)
                        .count()
                        == 1
            })
            .expect("unique catalog model with output");
        for (protocol, field) in [
            (ProviderProtocol::OpenAIChat, "max_completion_tokens"),
            (ProviderProtocol::OpenAIResponses, "max_output_tokens"),
            (ProviderProtocol::AnthropicMessages, "max_tokens"),
        ] {
            let mut config = Config {
                provider: "gateway".into(),
                model: model.id.clone(),
                providers: std::collections::HashMap::from([(
                    "gateway".into(),
                    talos_config::ProviderConfig {
                        base_url: Some("https://gateway.test/v1".into()),
                        protocol,
                        models: std::collections::HashMap::from([(
                            model.id.clone(),
                            talos_config::ModelConfig {
                                reasoning_supported: Some(false),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            assert_eq!(
                effective_output_limit(&config),
                model.output_limit.expect("output")
            );
            let provider = build_configured_provider(&config, "fixture-key").expect("provider");
            let preview = provider.request_preview(&[]).expect("preview");
            assert_eq!(preview["body"][field], model.output_limit.expect("output"));
            config
                .providers
                .get_mut("gateway")
                .expect("provider")
                .models
                .get_mut(&model.id)
                .expect("model")
                .output_limit = Some(1234);
            let provider = build_configured_provider(&config, "fixture-key").expect("provider");
            assert_eq!(
                provider.request_preview(&[]).expect("preview")["body"][field],
                1234
            );
        }
    }

    #[test]
    fn model_messages_override_constructs_a_full_endpoint_from_a_gateway_root() {
        for base_url in [
            "https://gateway.test/v1",
            "https://gateway.test/v1/messages/",
        ] {
            let config = Config {
                provider: "gateway".into(),
                model: "test-model".into(),
                providers: std::collections::HashMap::from([(
                    "gateway".into(),
                    talos_config::ProviderConfig {
                        base_url: Some(base_url.into()),
                        models: std::collections::HashMap::from([(
                            "test-model".into(),
                            talos_config::ModelConfig {
                                protocol: Some(ProviderProtocol::AnthropicMessages),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let provider = build_configured_provider(&config, "test-key").expect("adapter");
            let preview = provider.request_preview(&[]).expect("request preview");
            assert_eq!(preview["url"], "https://gateway.test/v1/messages");
        }
    }

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
                compatibility: true,
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
        let preview = provider.request_preview(&[]).expect("request preview");
        assert!(preview["body"].get("messages").is_some());
        assert!(preview["body"].get("input").is_none());
    }
    #[test]
    fn model_endpoint_reaches_each_real_adapter_and_never_changes_wire() {
        for (protocol, suffix) in [
            (ProviderProtocol::OpenAIResponses, "/responses"),
            (ProviderProtocol::OpenAIChat, "/chat/completions"),
            (ProviderProtocol::AnthropicMessages, "/messages"),
        ] {
            let config = Config {
                provider: "fixture".into(),
                model: "model".into(),
                providers: std::collections::HashMap::from([(
                    "fixture".into(),
                    talos_config::ProviderConfig {
                        base_url: Some("https://provider.test/v1".into()),
                        models: std::collections::HashMap::from([(
                            "model".into(),
                            talos_config::ModelConfig {
                                base_url: Some("https://model.test/wire/v1".into()),
                                protocol: Some(protocol),
                                ..Default::default()
                            },
                        )]),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            };
            let preview = build_configured_provider(&config, "test-key")
                .expect("model adapter")
                .request_preview(&[])
                .expect("preview");
            assert_eq!(
                preview["url"],
                format!("https://model.test/wire/v1{suffix}")
            );
        }
    }

    #[test]
    fn unsupported_model_transport_is_rejected_without_building_a_fallback_adapter() {
        let config = Config {
            provider: "openai".into(),
            model: "o3".into(),
            providers: std::collections::HashMap::from([(
                "openai".into(),
                talos_config::ProviderConfig {
                    models: std::collections::HashMap::from([(
                        "o3".into(),
                        talos_config::ModelConfig {
                            base_url: Some("wss://model.test/realtime".into()),
                            protocol: Some(ProviderProtocol::OpenAIResponses),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        assert!(build_configured_provider(&config, "test-key").is_err());
    }
}
