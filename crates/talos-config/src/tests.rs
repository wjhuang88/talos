use super::*;
use crate::env::substitute_env_vars;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_substitute_env_vars_replaces_known_vars() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("TALOS_TEST_KEY", "secret123") };
    let input = "key = \"${TALOS_TEST_KEY}\"";
    let output = substitute_env_vars(input);
    assert_eq!(output, "key = \"secret123\"");
    unsafe { env::remove_var("TALOS_TEST_KEY") };
}

#[test]
fn test_substitute_env_vars_leaves_unknown_vars() {
    let input = "key = \"${NONEXISTENT_VAR_12345}\"";
    let output = substitute_env_vars(input);
    assert_eq!(output, input);
}

#[test]
fn test_substitute_env_vars_multiple_substitutions() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe {
        env::set_var("TALOS_A", "hello");
        env::set_var("TALOS_B", "world");
    }
    let input = "${TALOS_A} ${TALOS_B}";
    let output = substitute_env_vars(input);
    assert_eq!(output, "hello world");
    unsafe {
        env::remove_var("TALOS_A");
        env::remove_var("TALOS_B");
    }
}

#[test]
fn test_substitute_env_vars_no_vars() {
    let input = "plain text with no vars";
    let output = substitute_env_vars(input);
    assert_eq!(output, input);
}

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.provider, "anthropic");
    assert!(config.model.is_empty());
    assert!(config.providers.is_empty());
    assert_eq!(config.log, LogConfig::default());
    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
}

#[test]
fn test_api_key_from_env_anthropic() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("ANTHROPIC_API_KEY", "env-key-anthropic") };
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(config.api_key().unwrap(), "env-key-anthropic");
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
}

#[test]
fn test_dashboard_enabled_by_default() {
    let config = Config::default();
    assert!(config.dashboard.enabled);
}

#[test]
fn test_dashboard_loopback_only_defaults_true() {
    let config = Config::default();
    assert!(
        config.dashboard.loopback_only,
        "loopback_only must default to true so the loopback bind is the only access control by default"
    );
}

#[test]
fn test_dashboard_loopback_only_deserializes() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[dashboard]
enabled = true
loopback_only = true
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.dashboard.enabled);
    assert!(config.dashboard.loopback_only);
}

#[test]
fn test_dashboard_loopback_only_absent_keeps_default() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[dashboard]
enabled = true
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.dashboard.loopback_only);
}

#[test]
fn test_api_key_from_env_openai() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("OPENAI_API_KEY", "env-key-openai") };
    let config = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "gpt-test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(config.api_key().unwrap(), "env-key-openai");
    unsafe { env::remove_var("OPENAI_API_KEY") };
}

#[test]
fn test_api_key_from_env_openai_compat() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::remove_var("OPENAI_API_KEY") };
    unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "bailian-style-key") };
    let config = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "glm-5".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(config.api_key().unwrap(), "bailian-style-key");
    unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
}

#[test]
fn test_api_key_openai_prefers_explicit_env_over_compat_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("OPENAI_API_KEY", "real-openai-key") };
    unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "bailian-key") };
    let config = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(config.api_key().unwrap(), "real-openai-key");
    unsafe { env::remove_var("OPENAI_API_KEY") };
    unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
}

#[test]
fn test_api_key_anthropic_does_not_check_openai_compat_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    unsafe { env::set_var("OPENAI_COMPAT_API_KEY", "should-not-be-used") };
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    let err = config.api_key().unwrap_err();
    assert!(matches!(err, ConfigError::MissingApiKey(_, _)));
    let msg = err.to_string();
    assert!(msg.contains("ANTHROPIC_API_KEY"));
    assert!(!msg.contains("OPENAI_COMPAT_API_KEY"));
    unsafe { env::remove_var("OPENAI_COMPAT_API_KEY") };
}

#[test]
fn test_api_key_missing_error() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    let err = config.api_key().unwrap_err();
    assert!(matches!(err, ConfigError::MissingApiKey(_, _)));
    let msg = err.to_string();
    assert!(msg.contains("ANTHROPIC_API_KEY"));
}

#[test]
fn test_base_url_getter() {
    let config = Config {
        variant: None,
        provider: "dashscope".to_string(),
        model: "glm-5".to_string(),
        providers: HashMap::from([(
            "dashscope".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                base_url: Some("https://example.com/v1".to_string()),
                api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                ..Default::default()
            },
        )]),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(config.base_url().as_deref(), Some("https://example.com/v1"));
}

#[test]
fn test_base_url_default_is_none() {
    let config = Config::default();
    assert_eq!(config.base_url(), None);
}

#[test]
fn test_base_url_parsed_from_toml() {
    let toml_str = r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
            api_key_env = "DASHSCOPE_API_KEY"
        "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(
        config.base_url().as_deref(),
        Some("https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1")
    );
}

#[test]
fn test_anthropic_catalog_endpoint_normalized_for_legacy_minimax_config() {
    let config = Config {
        variant: None,
        provider: "minimax-coding-plan".to_string(),
        model: "MiniMax-M2.7".to_string(),
        providers: HashMap::from([(
            "minimax-coding-plan".to_string(),
            ProviderConfig {
                base_url: Some("https://api.minimax.io/anthropic/v1".to_string()),
                api_key: Some("minimax-secret".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
    assert_eq!(
        config.base_url().as_deref(),
        Some("https://api.minimax.io/anthropic/v1/messages")
    );
}

#[test]
fn test_builtin_anthropic_custom_endpoint_keeps_anthropic_protocol() {
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-test".to_string(),
        providers: HashMap::from([(
            "anthropic".to_string(),
            ProviderConfig {
                base_url: Some("https://gateway.example.com/v1/messages".to_string()),
                api_key: Some("sk-ant".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    assert_eq!(
        config.provider_protocol(),
        ProviderProtocol::AnthropicMessages
    );
    assert_eq!(
        config.base_url().as_deref(),
        Some("https://gateway.example.com/v1/messages")
    );
}

#[test]
fn test_custom_provider_api_key_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("DASHSCOPE_API_KEY", "dashscope-key") };
    let config = Config {
        variant: None,
        provider: "dashscope".to_string(),
        model: "glm-5".to_string(),
        providers: HashMap::from([(
            "dashscope".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                base_url: Some("https://example.com/v1".to_string()),
                api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                ..Default::default()
            },
        )]),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };

    assert_eq!(config.api_key().unwrap(), "dashscope-key");
    unsafe { env::remove_var("DASHSCOPE_API_KEY") };
}

#[test]
fn test_model_limits_from_builtin_and_custom_providers() {
    // Builtin limits resolve via resolve_model_limits() (catalog lookup),
    // not context_limit() (user-config only).
    let builtin = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    let (builtin_ctx, builtin_out) = builtin.resolve_model_limits();
    assert_eq!(builtin_ctx, 1_047_576);
    assert_eq!(builtin_out, Some(32_768));

    let custom = Config {
        variant: None,
        provider: "dashscope".to_string(),
        model: "glm-5".to_string(),
        providers: HashMap::from([(
            "dashscope".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                base_url: Some("https://example.com/v1".to_string()),
                api_key: None,
                api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
                models: HashMap::from([(
                    "glm-5".to_string(),
                    ModelConfig {
                        context_limit: Some(202_752),
                        output_limit: Some(4096),
                        reasoning: None,
                        image_input: None,
                    },
                )]),
                timeout: Default::default(),
            },
        )]),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    assert_eq!(custom.context_limit(), Some(202_752));
    assert_eq!(custom.output_limit(), Some(4096));
}

#[test]
fn test_log_config_parsed_from_toml() {
    let toml_str = r#"
            provider = "openai"
            model = "glm-5"

            [log]
            level = "warn"
            format = "compact"
            filter = "talos_provider=debug,talos_agent=info"
        "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.log.level.as_deref(), Some("warn"));
    assert_eq!(config.log.format, LogFormat::Compact);
    assert_eq!(
        config.log.filter.as_deref(),
        Some("talos_provider=debug,talos_agent=info")
    );
}

#[test]
fn test_log_config_defaults() {
    let config = Config::default();
    assert_eq!(config.log.level, None);
    assert_eq!(config.log.format, LogFormat::Pretty);
    assert_eq!(config.log.filter, None);
}

#[test]
fn test_load_nonexistent_file() {
    let path = Config::default_path();
    if path.exists() {
        return;
    }
    let config = Config::load().unwrap();
    assert_eq!(config.provider, "anthropic");
    assert!(config.model.is_empty());
}

/// Regression test: an on-disk `config.toml` with an empty `model` field
/// must load successfully so callers (TUI/print/RPC mode setup-wizard
/// logic) can detect the empty model and route to first-run setup or a
/// helpful message. Before this fix, `Config::load()` called `validate()`
/// internally and hard-failed with `ConfigError::InvalidConfig` whenever
/// the file existed with an empty model — making the on-disk state
/// unrecoverable via `talos config set` too, since that command's own
/// `Config::load()` call would fail identically.
#[test]
fn test_load_existing_file_with_empty_model_succeeds() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_load_empty_model");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(tmp_dir.join(".talos")).unwrap();
    let prev_home = std::env::var_os("HOME");
    unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

    let config_toml = r#"
provider = "anthropic"
model = ""
"#;
    fs::write(Config::default_path(), config_toml).unwrap();

    let result = Config::load();
    assert!(
        result.is_ok(),
        "loading a config.toml with an empty model must succeed, not error: {:?}",
        result.err()
    );
    let config = result.unwrap();
    assert!(config.model.is_empty());
    assert_eq!(config.provider, "anthropic");

    match prev_home {
        Some(value) => unsafe { env::set_var("HOME", value) },
        None => unsafe { env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&tmp_dir);
}

/// Companion regression test: `talos config set` must remain able to fix
/// an on-disk config that currently has an empty model, i.e. loading it
/// (to then apply an edit) must not fail before the edit has a chance to
/// run.
#[test]
fn test_load_then_set_model_recovers_from_empty_model_on_disk() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_recover_empty_model");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(tmp_dir.join(".talos")).unwrap();
    let prev_home = std::env::var_os("HOME");
    unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

    fs::write(
        Config::default_path(),
        "provider = \"anthropic\"\nmodel = \"\"\n",
    )
    .unwrap();

    let mut config = Config::load().expect("load must succeed even with empty model on disk");
    config.model = "claude-sonnet-4-5".to_string();
    assert!(
        config.validate().is_ok(),
        "config must be valid after the user sets a model"
    );

    match prev_home {
        Some(value) => unsafe { env::set_var("HOME", value) },
        None => unsafe { env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_provider_serialization() {
    let config_anthropic = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };
    let config_openai = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "test".to_string(),
        providers: HashMap::new(),
        log: LogConfig::default(),
        hooks: HookConfig::default(),
        mcp: McpConfig::default(),
        rpc: RpcConfig::default(),
        memory_prompt: MemoryPromptConfig::default(),
        skills: SkillConfig::default(),
        dashboard: DashboardConfig::default(),
    };

    let a_str = toml::to_string(&config_anthropic).unwrap();
    let o_str = toml::to_string(&config_openai).unwrap();

    assert!(a_str.contains("anthropic"));
    assert!(o_str.contains("openai"));
}

#[test]
fn test_config_from_toml() {
    let toml_str = r#"
            provider = "openai"
            model = "gpt-4"
        "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, "gpt-4");
}

#[test]
fn test_inline_api_key_parsed_from_toml() {
    let toml_str = r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            base_url = "https://example.com/v1"
            api_key = "sk-inline-secret"
        "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.api_key().unwrap(), "sk-inline-secret");
}

#[test]
fn test_inline_api_key_precedence_over_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("DASHSCOPE_API_KEY", "env-key-should-not-be-used") };
    let config: Config = toml::from_str(
        r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            api_key = "inline-key-wins"
            api_key_env = "DASHSCOPE_API_KEY"
        "#,
    )
    .unwrap();
    assert_eq!(config.api_key().unwrap(), "inline-key-wins");
    unsafe { env::remove_var("DASHSCOPE_API_KEY") };
}

#[test]
fn test_inline_api_key_anthropic_overrides_builtin() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    let config: Config = toml::from_str(
        r#"
            provider = "anthropic"
            model = "claude-test"

            [providers.anthropic]
            api_key = "inline-anthropic-key"
        "#,
    )
    .unwrap();
    assert_eq!(config.api_key().unwrap(), "inline-anthropic-key");
}

#[test]
fn test_validate_accepts_either_api_key_or_api_key_env() {
    let with_inline = Config {
        provider: "custom".to_string(),
        model: "model-x".to_string(),
        providers: HashMap::from([(
            "custom".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                api_key: Some("inline".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    assert!(with_inline.validate().is_ok());

    let with_env = Config {
        provider: "custom".to_string(),
        model: "model-x".to_string(),
        providers: HashMap::from([(
            "custom".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                api_key_env: Some("CUSTOM_KEY".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    assert!(with_env.validate().is_ok());
}

#[test]
fn test_validate_rejects_neither_api_key_nor_api_key_env() {
    let config = Config {
        variant: None,
        provider: "custom".to_string(),
        model: "model-x".to_string(),
        providers: HashMap::from([(
            "custom".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                tool_protocol: Default::default(),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let err = config.validate().unwrap_err();
    assert!(err.to_string().contains("api_key or api_key_env"));
}

#[test]
fn test_inline_api_key_is_serialized_in_config_toml() {
    // I045 reverted skip_serializing: api_key is now stored directly in
    // config.toml (the file lives in the user's home directory, chmod 600
    // recommended). Display masking is the responsibility of
    // `talos config list`/`get`, not the serializer.
    let config: Config = toml::from_str(
        r#"
            provider = "dashscope"
            model = "glm-5"

            [providers.dashscope]
            protocol = "openai-chat"
            api_key = "sk-very-secret"
            api_key_env = "DASHSCOPE_API_KEY"
        "#,
    )
    .unwrap();
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("sk-very-secret"));
    assert!(serialized.contains("api_key ="));
}

#[test]
fn test_resolve_model_limits_returns_user_config_when_set() {
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        providers: HashMap::from([(
            "anthropic".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "claude-sonnet-4-5".to_string(),
                    ModelConfig {
                        context_limit: Some(150_000),
                        output_limit: Some(8000),
                        reasoning: None,
                        image_input: None,
                    },
                )]),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let (ctx, out) = config.resolve_model_limits();
    assert_eq!(ctx, 150_000);
    assert_eq!(out, Some(8000));
}

#[test]
fn test_resolve_model_limits_falls_back_to_builtin_catalog() {
    let config = Config {
        variant: None,
        provider: "google".to_string(),
        model: "gemini-2.5-pro".to_string(),
        providers: HashMap::from([(
            "google".to_string(),
            ProviderConfig {
                api_key_env: Some("GOOGLE_API_KEY".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let (ctx, out) = config.resolve_model_limits();
    assert_eq!(ctx, 1_048_576);
    assert_eq!(out, Some(65536));
}

#[test]
fn test_resolve_model_limits_falls_back_to_conservative_when_not_in_catalog() {
    let config = Config {
        variant: None,
        provider: "custom-provider".to_string(),
        model: "unknown-model-xyz".to_string(),
        providers: HashMap::from([(
            "custom-provider".to_string(),
            ProviderConfig {
                api_key_env: Some("CUSTOM_KEY".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let (ctx, out) = config.resolve_model_limits();
    assert_eq!(ctx, 128_000);
    assert_eq!(out, None);
}

#[test]
fn test_resolve_model_limits_output_limit_from_catalog() {
    let config = Config {
        variant: None,
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx, out) = config.resolve_model_limits();
    assert_eq!(ctx, 1_047_576);
    assert_eq!(out, Some(32768));
}

#[test]
fn test_resolve_model_limits_user_config_takes_precedence_over_catalog() {
    let config = Config {
        variant: None,
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        providers: HashMap::from([(
            "anthropic".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "claude-sonnet-4-5".to_string(),
                    ModelConfig {
                        context_limit: Some(100_000),
                        output_limit: None,
                        reasoning: None,
                        image_input: None,
                    },
                )]),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let (ctx, out) = config.resolve_model_limits();
    assert_eq!(ctx, 100_000);
    assert_eq!(out, None);
}

#[test]
fn test_credentials_default_path() {
    let path = Credentials::default_path();
    assert!(path.to_string_lossy().contains(".talos"));
    assert!(path.to_string_lossy().contains("credentials.toml"));
}

#[test]
fn test_credentials_load_nonexistent_returns_empty() {
    let creds = Credentials::load().unwrap();
    assert!(creds.keys.is_empty());
}

#[test]
fn test_credentials_save_and_load_roundtrip() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_creds");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let creds_path = tmp_dir.join("credentials.toml");
    unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("anthropic".to_string(), "sk-test-key".to_string());
    creds
        .keys
        .insert("openai".to_string(), "sk-openai-key".to_string());
    creds.save().unwrap();

    let loaded = Credentials::load().unwrap();
    assert_eq!(
        loaded.keys.get("anthropic"),
        Some(&"sk-test-key".to_string())
    );
    assert_eq!(
        loaded.keys.get("openai"),
        Some(&"sk-openai-key".to_string())
    );

    unsafe { env::remove_var("HOME") };
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_provider_authenticated_with_inline_key() {
    let mut config = Config::default();
    config.set_provider_credential("anthropic", "sk-inline-key");
    assert!(config.provider_authenticated("anthropic"));
}

#[test]
fn test_provider_authenticated_with_env_var() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("ANTHROPIC_API_KEY", "env-key") };
    let config = Config::default();
    assert!(config.provider_authenticated("anthropic"));
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
}

#[test]
fn test_provider_authenticated_returns_false_when_no_key() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::remove_var("ANTHROPIC_API_KEY") };
    let config = Config {
        variant: None,
        providers: HashMap::from([(
            "custom".to_string(),
            ProviderConfig {
                protocol: ProviderProtocol::OpenAIChat,
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    assert!(!config.provider_authenticated("custom"));
    assert!(!config.provider_authenticated("nonexistent"));
}

#[test]
fn test_set_active_model_sets_provider_from_catalog() {
    let mut config = Config::default();
    config
        .set_active_model("anthropic/claude-sonnet-4-5")
        .unwrap();
    assert_eq!(config.model, "claude-sonnet-4-5");
    assert_eq!(config.provider, "anthropic");
    assert!(config.providers.contains_key("anthropic"));
}

#[test]
fn test_set_active_model_openai() {
    let mut config = Config::default();
    config.set_active_model("gpt-4o-2024-05-13").unwrap();
    assert_eq!(config.model, "gpt-4o-2024-05-13");
    assert_eq!(config.provider, "openai");
    assert!(config.providers.contains_key("openai"));
}

#[test]
fn test_set_active_model_unknown_model_errors() {
    let mut config = Config::default();
    let err = config
        .set_active_model("nonexistent-model-xyz")
        .unwrap_err();
    assert!(err.to_string().contains("nonexistent-model-xyz"));
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_set_provider_credential_creates_new_provider() {
    let mut config = Config::default();
    config.set_provider_credential("custom-provider", "sk-custom-key");
    assert!(config.providers.contains_key("custom-provider"));
    let provider = config.providers.get("custom-provider").unwrap();
    assert_eq!(provider.api_key.as_deref(), Some("sk-custom-key"));
}

#[test]
fn test_set_provider_credential_overwrites_existing() {
    let mut config = Config::default();
    config.set_provider_credential("anthropic", "old-key");
    config.set_provider_credential("anthropic", "new-key");
    let provider = config.providers.get("anthropic").unwrap();
    assert_eq!(provider.api_key.as_deref(), Some("new-key"));
}

#[test]
fn test_save_writes_api_key_in_config_toml() {
    // I045 fix: api_key is now serialized in config.toml. This avoids
    // the silent data-loss bug where keys were moved to a separate
    // credentials.toml without the user knowing. Display masking
    // remains the responsibility of `talos config list`/`get`.
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_save");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir.join(".talos")).unwrap();
    let prev_home = std::env::var_os("HOME");
    unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

    let mut config = Config::default();
    config.model = "claude-sonnet-4-5".to_string();
    config.set_provider_credential("anthropic", "sk-secret-key");
    config.save().unwrap();

    let config_path = Config::default_path();
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("sk-secret-key"));

    // No credentials.toml should be written anymore.
    let creds_path = Credentials::default_path();
    assert!(
        !creds_path.exists(),
        "credentials.toml should not be created"
    );

    match prev_home {
        Some(v) => unsafe { env::set_var("HOME", v) },
        None => unsafe { env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_load_merges_credentials() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_load_merge");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(tmp_dir.join(".talos")).unwrap();
    unsafe { env::set_var("HOME", tmp_dir.to_string_lossy().as_ref()) };

    let config_toml = r#"
provider = "anthropic"
model = "claude-sonnet-4-5"
"#;
    fs::write(Config::default_path(), config_toml).unwrap();

    let creds_toml = r#"
anthropic = "sk-merged-key"
"#;
    fs::write(Credentials::default_path(), creds_toml).unwrap();

    let config = Config::load().unwrap();
    let provider = config.providers.get("anthropic").unwrap();
    assert_eq!(provider.api_key.as_deref(), Some("sk-merged-key"));

    unsafe { env::remove_var("HOME") };
    let _ = fs::remove_dir_all(&tmp_dir);
    let _ = fs::remove_dir_all(&tmp_dir);
}

/// Regression test for the I045 data-loss bug: inline api_key in
/// config.toml must be preserved across load+save round-trips and
/// visible to anyone reading the file. The fix was to drop the
/// `skip_serializing` attribute (which was quietly moving keys to
/// a separate credentials.toml). Display masking is handled by
/// `talos config list`/`get`, not the serializer.
#[test]
fn test_save_preserves_inline_api_key_from_config_toml() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp_dir = env::temp_dir().join("talos_test_roundtrip");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();
    let talos_dir = &tmp_dir.join(".talos");
    fs::create_dir_all(&talos_dir).unwrap();
    let prev_home = std::env::var_os("HOME");
    unsafe { env::set_var("HOME", &tmp_dir.to_string_lossy().as_ref()) };

    let config_toml = r#"
provider = "anthropic"
model = "claude-sonnet-4-5"

[providers.anthropic]
protocol = "anthropic-messages"
api_key = "sk-inline-secret-from-config"
"#;
    fs::write(Config::default_path(), config_toml).unwrap();

    let config = Config::load().unwrap();
    let provider = config.providers.get("anthropic").unwrap();
    assert_eq!(
        provider.api_key.as_deref(),
        Some("sk-inline-secret-from-config"),
        "api_key must be loaded from config.toml during deserialization"
    );

    config.save().unwrap();

    let saved_config = fs::read_to_string(Config::default_path()).unwrap();
    assert!(
        saved_config.contains("sk-inline-secret-from-config"),
        "api_key must be present in saved config.toml (regression for I045 data-loss bug)"
    );

    // No credentials.toml should be written.
    assert!(
        !Credentials::default_path().exists(),
        "credentials.toml should not be written anymore"
    );

    let config2 = Config::load().unwrap();
    let provider2 = config2.providers.get("anthropic").unwrap();
    assert_eq!(
        provider2.api_key.as_deref(),
        Some("sk-inline-secret-from-config"),
        "api_key must survive a second load round-trip"
    );

    match prev_home {
        Some(v) => unsafe { env::set_var("HOME", v) },
        None => unsafe { env::remove_var("HOME") },
    }
    let _ = fs::remove_dir_all(&tmp_dir);
}

/// When the I045 fix is applied (no skip_serializing), api_key is
/// serialized in config.toml and survives a load+save round-trip
/// in the main config file alone — no credentials.toml needed.
#[test]
fn test_skip_serializing_does_not_skip_deserialization() {
    let toml_str = r#"
            provider = "test"
            model = "test-model"

            [providers.test]
            api_key = "hello-from-toml"
        "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    let provider = config.providers.get("test").unwrap();
    assert_eq!(provider.api_key.as_deref(), Some("hello-from-toml"));
}

#[test]
fn test_resolve_model_limits_provider_aware_for_duplicate_ids() {
    // glm-5.2 exists under many providers (models.dev). The lookup must
    // succeed for the specified provider, not fall back to the conservative
    // default or silently resolve to a different provider's entry.
    let aihubmix = Config {
        provider: "aihubmix".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx, _) = aihubmix.resolve_model_limits();
    assert_eq!(ctx, 1_000_000);

    let cortecs = Config {
        provider: "cortecs".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx2, _) = cortecs.resolve_model_limits();
    assert_eq!(ctx2, 1_000_000);

    // A wrong provider+model combo must NOT resolve via a different
    // provider's catalog entry — it falls to the conservative default.
    let wrong = Config {
        provider: "openai".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx3, out3) = wrong.resolve_model_limits();
    assert_eq!(ctx3, 128_000);
    assert_eq!(out3, None);
}

#[test]
fn test_set_active_model_errors_on_ambiguous_bare_id() {
    let mut config = Config {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let err = config.set_active_model("glm-5.2").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("multiple providers"),
        "expected ambiguity error, got: {msg}"
    );
    assert!(msg.contains("aihubmix"));
    assert!(msg.contains("cortecs"));
}

#[test]
fn test_set_active_model_provider_qualified_resolves_correctly() {
    let mut config = Config {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    config.set_active_model("aihubmix/glm-5.2").unwrap();
    assert_eq!(config.model, "glm-5.2");
    assert_eq!(config.provider, "aihubmix");
    assert!(config.providers.contains_key("aihubmix"));
}

#[test]
fn test_set_active_model_unique_bare_id_still_works() {
    let mut config = Config {
        provider: "aihubmix".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    config.set_active_model("gpt-4o-2024-05-13").unwrap();
    assert_eq!(config.model, "gpt-4o-2024-05-13");
    assert_eq!(config.provider, "openai");
}

#[test]
fn test_all_models_preserves_duplicates_across_providers() {
    let config = Config::default();
    let all = config.all_models();
    let glm52: Vec<_> = all.iter().filter(|m| m.id == "glm-5.2").collect();
    assert!(
        glm52.len() >= 2,
        "glm-5.2 should appear under multiple providers, got {}",
        glm52.len()
    );
    let providers: Vec<_> = glm52.iter().map(|m| m.provider.as_str()).collect();
    assert!(providers.contains(&"aihubmix"));
    assert!(providers.contains(&"cortecs"));
}

#[test]
fn test_all_models_user_override_matches_by_provider_and_id() {
    let config = Config {
        variant: None,
        provider: "cortecs".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::from([(
            "cortecs".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "glm-5.2".to_string(),
                    ModelConfig {
                        context_limit: Some(50_000),
                        output_limit: Some(1000),
                        reasoning: None,
                        image_input: None,
                    },
                )]),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let all = config.all_models();
    // The cortecs entry should be overridden, NOT the aihubmix entry.
    let cortecs_entry = all
        .iter()
        .find(|m| m.id == "glm-5.2" && m.provider == "cortecs")
        .unwrap();
    assert_eq!(cortecs_entry.context_limit, Some(50_000));
    assert_eq!(cortecs_entry.output_limit, Some(1000));
    // The aihubmix entry should be untouched.
    let aihubmix_entry = all
        .iter()
        .find(|m| m.id == "glm-5.2" && m.provider == "aihubmix")
        .unwrap();
    assert_eq!(aihubmix_entry.context_limit, Some(1_000_000));
}

#[test]
fn test_provider_config_debug_masks_api_key() {
    let provider = ProviderConfig {
        api_key: Some("sk-super-secret".to_string()),
        api_key_env: Some("MY_KEY".to_string()),
        ..Default::default()
    };
    let debug = format!("{provider:?}");
    assert!(!debug.contains("sk-super-secret"));
    assert!(debug.contains("***"));
}

#[test]
fn test_credentials_debug_masks_keys() {
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("anthropic".to_string(), "sk-secret-key".to_string());
    let debug = format!("{creds:?}");
    assert!(!debug.contains("sk-secret-key"));
    assert!(debug.contains("redacted"));
}

#[test]
fn test_config_debug_masks_provider_api_keys() {
    let config = Config {
        variant: None,
        provider: "custom".to_string(),
        model: "test".to_string(),
        providers: HashMap::from([(
            "custom".to_string(),
            ProviderConfig {
                api_key: Some("sk-leak-test".to_string()),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let debug = format!("{config:?}");
    assert!(
        !debug.contains("sk-leak-test"),
        "Config Debug must not leak api_key"
    );
    assert!(debug.contains("***"));
}

#[test]
fn test_skill_config_default_on() {
    let config = Config::default();
    assert!(config.skills.discover_shared);
}

#[test]
fn test_skill_config_deserializes() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[skills]
discover_shared = true
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.skills.discover_shared);
}

#[test]
fn test_skill_config_serializes() {
    let config = Config {
        variant: None,
        skills: SkillConfig {
            discover_shared: true,
        },
        ..Default::default()
    };
    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("discover_shared"));
    assert!(serialized.contains("true"));
}

#[test]
fn skill_config_defaults_shared_on_when_section_missing() {
    let toml_str = r#"
provider = "anthropic"
model = "test"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(
        config.skills.discover_shared,
        "missing [skills] section must default to true"
    );
}

#[test]
fn skill_config_defaults_shared_on_for_empty_section() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[skills]
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(
        config.skills.discover_shared,
        "empty [skills] section must default to true"
    );
}

#[test]
fn skill_config_explicit_false_is_preserved() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[skills]
discover_shared = false
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(
        !config.skills.discover_shared,
        "explicit false must be preserved"
    );
}

#[test]
fn skill_config_explicit_true_is_preserved() {
    let toml_str = r#"
provider = "anthropic"
model = "test"

[skills]
discover_shared = true
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(
        config.skills.discover_shared,
        "explicit true must be preserved"
    );
}

#[test]
fn test_provider_timeout_config_defaults() {
    let timeout = ProviderTimeoutConfig::default();
    assert_eq!(timeout.first_packet_timeout_secs, 30);
    assert_eq!(timeout.stream_idle_timeout_secs, 90);
    assert_eq!(timeout.max_attempts, 3);
    assert_eq!(timeout.backoff_base_ms, 500);
    assert_eq!(timeout.backoff_max_ms, 8_000);
}

#[test]
fn test_provider_timeout_config_parsed_from_toml() {
    let config: Config = toml::from_str(
        r#"
            provider = "openai"
            model = "gpt-4o"

            [providers.openai]
            api_key_env = "OPENAI_API_KEY"

            [providers.openai.timeout]
            first_packet_timeout_secs = 12
            stream_idle_timeout_secs = 34
            max_attempts = 4
            backoff_base_ms = 250
            backoff_max_ms = 2000
        "#,
    )
    .unwrap();

    let timeout = &config.providers["openai"].timeout;
    assert_eq!(timeout.first_packet_timeout_secs, 12);
    assert_eq!(timeout.stream_idle_timeout_secs, 34);
    assert_eq!(timeout.max_attempts, 4);
    assert_eq!(timeout.backoff_base_ms, 250);
    assert_eq!(timeout.backoff_max_ms, 2000);
}

#[test]
fn test_all_models_with_catalog_overlays_builtin() {
    let config = Config::default();
    let builtin_count = config.all_models().len();

    let catalog_models = vec![model::ModelMetadata {
        variants: vec![],
        id: "catalog-only-model".to_string(),
        provider: "catalog-provider".to_string(),
        context_limit: Some(500_000),
        output_limit: Some(10_000),
        pricing: None,
        capabilities: model::ModelCapabilities {
            tools: true,
            ..Default::default()
        },
        release_date: None,
        source: model::ModelSource::ModelsDev {
            refreshed_at: "2025-07-03T00:00:00Z".to_string(),
        },
    }];

    let merged = config.all_models_with_catalog(Some(&catalog_models));
    assert_eq!(merged.len(), builtin_count + 1);

    let found = model::find_model_by_provider(&merged, "catalog-provider", "catalog-only-model");
    assert!(found.is_some());
    assert_eq!(found.unwrap().context_limit, Some(500_000));
}

#[test]
fn test_all_models_with_catalog_replaces_builtin_entry() {
    let config = Config::default();
    let builtins = model::builtin_models();
    let first = &builtins[0];

    let catalog_models = vec![model::ModelMetadata {
        variants: vec![],
        id: first.id.clone(),
        provider: first.provider.clone(),
        context_limit: Some(999_999),
        output_limit: Some(99_999),
        pricing: None,
        capabilities: model::ModelCapabilities::default(),
        release_date: None,
        source: model::ModelSource::ModelsDev {
            refreshed_at: "2025-07-03T00:00:00Z".to_string(),
        },
    }];

    let merged = config.all_models_with_catalog(Some(&catalog_models));
    let found = model::find_model_by_provider(&merged, &first.provider, &first.id).unwrap();
    assert_eq!(found.context_limit, Some(999_999));
    assert_eq!(found.output_limit, Some(99_999));
}

#[test]
fn test_all_models_with_catalog_user_config_overrides_catalog() {
    let mut config = Config::default();
    config.provider = "test".to_string();
    config.model = "m1".to_string();
    config.providers.insert(
        "test".to_string(),
        ProviderConfig {
            models: HashMap::from([(
                "m1".to_string(),
                ModelConfig {
                    context_limit: Some(42_000),
                    output_limit: Some(4_200),
                    reasoning: None,
                    image_input: None,
                },
            )]),
            ..Default::default()
        },
    );

    let catalog_models = vec![model::ModelMetadata {
        variants: vec![],
        id: "m1".to_string(),
        provider: "test".to_string(),
        context_limit: Some(500_000),
        output_limit: Some(50_000),
        pricing: None,
        capabilities: model::ModelCapabilities::default(),
        release_date: None,
        source: model::ModelSource::ModelsDev {
            refreshed_at: "t".to_string(),
        },
    }];

    let merged = config.all_models_with_catalog(Some(&catalog_models));
    let found = model::find_model_by_provider(&merged, "test", "m1").unwrap();
    assert_eq!(found.context_limit, Some(42_000));
    assert_eq!(found.output_limit, Some(4_200));
    assert_eq!(found.source, model::ModelSource::Manual);
}

#[test]
fn test_all_models_with_catalog_none_matches_all_models() {
    let config = Config::default();
    let without = config.all_models();
    let with_none = config.all_models_with_catalog(None);
    assert_eq!(without.len(), with_none.len());
}

#[test]
fn test_resolve_model_limits_with_catalog_precedence() {
    let mut config = Config::default();
    config.provider = "test-provider".to_string();
    config.model = "test-model".to_string();

    let catalog_models = vec![model::ModelMetadata {
        variants: vec![],
        id: "test-model".to_string(),
        provider: "test-provider".to_string(),
        context_limit: Some(300_000),
        output_limit: Some(30_000),
        pricing: None,
        capabilities: model::ModelCapabilities::default(),
        release_date: None,
        source: model::ModelSource::Builtin,
    }];

    let (ctx, out) = config.resolve_model_limits_with_catalog(Some(&catalog_models));
    assert_eq!(ctx, 300_000);
    assert_eq!(out, Some(30_000));
}

#[test]
fn test_resolve_model_limits_with_catalog_user_overrides_catalog() {
    let mut config = Config::default();
    config.provider = "tp".to_string();
    config.model = "tm".to_string();
    config.providers.insert(
        "tp".to_string(),
        ProviderConfig {
            models: HashMap::from([(
                "tm".to_string(),
                ModelConfig {
                    context_limit: Some(111_000),
                    output_limit: Some(11_100),
                    reasoning: None,
                    image_input: None,
                },
            )]),
            ..Default::default()
        },
    );

    let catalog_models = vec![model::ModelMetadata {
        variants: vec![],
        id: "tm".to_string(),
        provider: "tp".to_string(),
        context_limit: Some(300_000),
        output_limit: Some(30_000),
        pricing: None,
        capabilities: model::ModelCapabilities::default(),
        release_date: None,
        source: model::ModelSource::Builtin,
    }];

    let (ctx, out) = config.resolve_model_limits_with_catalog(Some(&catalog_models));
    assert_eq!(ctx, 111_000);
    assert_eq!(out, Some(11_100));
}

#[test]
fn test_resolve_model_limits_with_catalog_none_falls_back_to_builtin() {
    let mut config = Config::default();
    config.provider = "anthropic".to_string();
    config.model = "claude-sonnet-4-5".to_string();

    let from_catalog = config.resolve_model_limits_with_catalog(None);
    let from_builtin = config.resolve_model_limits();
    assert_eq!(from_catalog, from_builtin);
}

#[test]
fn test_resolve_model_limits_with_catalog_fallback_for_unknown() {
    let mut config = Config::default();
    config.provider = "unknown".to_string();
    config.model = "unknown-model".to_string();

    let catalog_models: Vec<model::ModelMetadata> = vec![];
    let (ctx, out) = config.resolve_model_limits_with_catalog(Some(&catalog_models));
    assert_eq!(ctx, 128_000);
    assert!(out.is_none());
}

#[test]
fn test_resolve_model_limits_with_empty_catalog_does_not_block() {
    let mut config = Config::default();
    config.provider = "anthropic".to_string();
    config.model = "claude-sonnet-4-5".to_string();

    let empty_catalog: Vec<model::ModelMetadata> = vec![];
    let (ctx, _) = config.resolve_model_limits_with_catalog(Some(&empty_catalog));
    assert!(ctx > 0, "should fall back to builtin, not block");
}

#[test]
fn test_custom_model_image_input_override_enables_capability() {
    use talos_core::model::ImageInputCapability;

    let mut config = Config::default();
    config.provider = "my-gateway".to_string();
    config.model = "custom-vision-model".to_string();

    config.providers.insert(
        "my-gateway".to_string(),
        ProviderConfig {
            protocol: ProviderProtocol::AnthropicMessages,
            ..Default::default()
        },
    );

    let provider = config.providers.get_mut("my-gateway").unwrap();
    provider.models.insert(
        "custom-vision-model".to_string(),
        ModelConfig {
            image_input: Some(true),
            ..Default::default()
        },
    );

    let all = config.all_models();
    let meta = model::find_model_by_provider(&all, "my-gateway", "custom-vision-model");
    assert!(meta.is_some());
    assert!(meta.unwrap().capabilities.image_input);

    let cap = ImageInputCapability::from_metadata(model::find_model_by_provider(
        &all,
        "my-gateway",
        "custom-vision-model",
    ));
    assert_eq!(cap, ImageInputCapability::Supported);
    assert!(cap.allows_attachment());
}

#[test]
fn test_custom_model_without_image_input_override_is_unknown() {
    use talos_core::model::ImageInputCapability;

    let mut config = Config::default();
    config.provider = "my-gateway".to_string();
    config.model = "custom-model".to_string();

    config.providers.insert(
        "my-gateway".to_string(),
        ProviderConfig {
            protocol: ProviderProtocol::AnthropicMessages,
            ..Default::default()
        },
    );

    config
        .providers
        .get_mut("my-gateway")
        .unwrap()
        .models
        .insert("custom-model".to_string(), ModelConfig::default());

    let all = config.all_models();
    let meta = model::find_model_by_provider(&all, "my-gateway", "custom-model");
    assert!(meta.is_some());
    assert!(!meta.unwrap().capabilities.image_input);

    let cap = ImageInputCapability::from_metadata(model::find_model_by_provider(
        &all,
        "my-gateway",
        "custom-model",
    ));
    assert_eq!(cap, ImageInputCapability::Unsupported);
    assert!(!cap.allows_attachment());
}

#[test]
fn test_image_input_override_on_existing_catalog_model() {
    use talos_core::model::ImageInputCapability;

    let mut config = Config::default();
    config.provider = "anthropic".to_string();
    config.model = "claude-sonnet-4-5".to_string();

    config.providers.insert(
        "anthropic".to_string(),
        ProviderConfig {
            ..Default::default()
        },
    );

    config
        .providers
        .get_mut("anthropic")
        .unwrap()
        .models
        .insert(
            "claude-sonnet-4-5".to_string(),
            ModelConfig {
                image_input: Some(false),
                ..Default::default()
            },
        );

    let all = config.all_models();
    let meta = model::find_model_by_provider(&all, "anthropic", "claude-sonnet-4-5");
    assert!(meta.is_some());
    assert!(!meta.unwrap().capabilities.image_input);

    let cap = ImageInputCapability::from_metadata(model::find_model_by_provider(
        &all,
        "anthropic",
        "claude-sonnet-4-5",
    ));
    assert_eq!(cap, ImageInputCapability::Unsupported);
    assert!(!cap.allows_attachment());
}

// ---------------------------------------------------------------------------
// MODEL-010 / I157: Config::unset_dotted provider removal and credential clear
// ---------------------------------------------------------------------------

fn make_config_with_two_custom_providers() -> Config {
    let mut config = Config::default();
    config.providers.insert(
        "custom-a".to_string(),
        ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://a.example.com/v1".to_string()),
            api_key: Some("key-a".to_string()),
            api_key_env: Some("CUSTOM_A_KEY".to_string()),
            models: HashMap::from([(
                "model-a".to_string(),
                ModelConfig {
                    context_limit: Some(128_000),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    );
    config.providers.insert(
        "custom-b".to_string(),
        ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://b.example.com/v1".to_string()),
            api_key: Some("key-b".to_string()),
            models: HashMap::from([(
                "model-b".to_string(),
                ModelConfig {
                    output_limit: Some(4096),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    );
    config
}

#[test]
fn unset_custom_provider_removes_only_target_entry() {
    let mut config = make_config_with_two_custom_providers();
    let outcome = config.unset_dotted("providers.custom-a").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-a".to_string()
        }
    );
    assert!(!config.providers.contains_key("custom-a"));
    assert!(config.providers.contains_key("custom-b"));
    let b = config.providers.get("custom-b").unwrap();
    assert_eq!(b.base_url.as_deref(), Some("https://b.example.com/v1"));
}

#[test]
fn unset_builtin_provider_removes_only_user_configuration() {
    let mut config = Config::default();
    config.providers.insert(
        "anthropic".to_string(),
        ProviderConfig {
            api_key: Some("sk-ant-test".to_string()),
            models: HashMap::from([(
                "claude-test".to_string(),
                ModelConfig {
                    context_limit: Some(200_000),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    );
    config.providers.insert(
        "custom-x".to_string(),
        ProviderConfig {
            api_key: Some("key-x".to_string()),
            ..Default::default()
        },
    );

    let outcome = config.unset_dotted("providers.anthropic").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::BuiltinProviderDisconnected {
            name: "anthropic".to_string()
        }
    );
    assert!(!config.providers.contains_key("anthropic"));
    assert!(config.providers.contains_key("custom-x"));
    assert!(builtin_provider_config("anthropic").is_some());
}

#[test]
fn unset_provider_api_key_preserves_other_fields() {
    let mut config = make_config_with_two_custom_providers();
    let outcome = config.unset_dotted("providers.custom-a.api_key").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::ApiKeyCleared {
            name: "custom-a".to_string()
        }
    );
    let a = config.providers.get("custom-a").unwrap();
    assert!(a.api_key.is_none());
    assert_eq!(a.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(a.base_url.as_deref(), Some("https://a.example.com/v1"));
    assert_eq!(a.api_key_env.as_deref(), Some("CUSTOM_A_KEY"));
}

#[test]
fn unset_provider_api_key_preserves_model_overrides() {
    let mut config = make_config_with_two_custom_providers();
    config.unset_dotted("providers.custom-a.api_key").unwrap();
    let a = config.providers.get("custom-a").unwrap();
    assert!(a.models.contains_key("model-a"));
    assert_eq!(
        a.models.get("model-a").unwrap().context_limit,
        Some(128_000)
    );
}

#[test]
fn unset_provider_api_key_is_omitted_from_toml() {
    let mut config = make_config_with_two_custom_providers();
    config.unset_dotted("providers.custom-a.api_key").unwrap();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let reloaded: Config = toml::from_str(&toml_str).unwrap();
    assert!(
        reloaded
            .providers
            .get("custom-a")
            .unwrap()
            .api_key
            .is_none()
    );
    assert!(
        !toml_str.contains("key-a"),
        "cleared credential value must not appear in serialized TOML"
    );
    let a = reloaded.providers.get("custom-a").unwrap();
    assert_eq!(a.base_url.as_deref(), Some("https://a.example.com/v1"));
    assert_eq!(a.api_key_env.as_deref(), Some("CUSTOM_A_KEY"));
}

#[test]
fn unset_provider_does_not_modify_unrelated_providers() {
    let mut config = make_config_with_two_custom_providers();
    let b_snapshot = config.providers.get("custom-b").cloned();
    config.unset_dotted("providers.custom-a").unwrap();
    assert_eq!(
        config.providers.get("custom-b"),
        b_snapshot.as_ref(),
        "unrelated provider must be byte-identical"
    );
}

#[test]
fn unset_unknown_provider_does_not_mutate_config() {
    let mut config = make_config_with_two_custom_providers();
    let snapshot = toml::to_string_pretty(&config).unwrap();
    let err = config.unset_dotted("providers.nonexistent").unwrap_err();
    assert!(err.to_string().contains("not found"));
    let after = toml::to_string_pretty(&config).unwrap();
    assert_eq!(
        snapshot, after,
        "config must be unchanged on not-found error"
    );
}

#[test]
fn unset_invalid_dotted_key_does_not_mutate_config() {
    let mut config = make_config_with_two_custom_providers();
    let snapshot = toml::to_string_pretty(&config).unwrap();

    let err = config.unset_dotted("model").unwrap_err();
    assert!(err.to_string().contains("unsupported unset key"));
    assert_eq!(snapshot, toml::to_string_pretty(&config).unwrap());

    let err = config
        .unset_dotted("providers.custom-a.base_url")
        .unwrap_err();
    assert!(err.to_string().contains("unsupported unset key"));
    assert_eq!(snapshot, toml::to_string_pretty(&config).unwrap());
}

// ---------------------------------------------------------------------------
// MODEL-010 / I157 correction: ConfigStore persisted unset with atomic writes
// and credentials.toml resurrection prevention
// ---------------------------------------------------------------------------

use crate::ConfigStore;
use std::path::{Path, PathBuf};

fn unique_test_dir(label: &str) -> PathBuf {
    let unique = format!(
        "talos-store-test-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(&unique);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_test_config(dir: &Path, config: &Config) {
    let toml_str = toml::to_string_pretty(config).unwrap();
    fs::write(dir.join("config.toml"), &toml_str).unwrap();
}

fn write_test_credentials(dir: &Path, creds: &Credentials) {
    let toml_str = toml::to_string_pretty(creds).unwrap();
    fs::write(dir.join("credentials.toml"), &toml_str).unwrap();
}

fn make_store(dir: &Path) -> ConfigStore {
    ConfigStore::with_paths(dir.join("config.toml"), dir.join("credentials.toml"))
}

fn reload_via_config_load(dir: &Path) -> Config {
    let raw = fs::read_to_string(dir.join("config.toml")).unwrap();
    let mut config: Config = toml::from_str(&raw).unwrap();
    let creds_path = dir.join("credentials.toml");
    if creds_path.exists() {
        let raw_creds = fs::read_to_string(&creds_path).unwrap();
        if let Ok(creds) = toml::from_str::<Credentials>(&raw_creds) {
            for (name, key) in &creds.keys {
                if let Some(provider) = config.providers.get_mut(name) {
                    if provider.api_key.is_none() {
                        provider.api_key = Some(key.clone());
                    }
                } else if let Some(builtin) = crate::builtin_provider_config(name) {
                    let mut provider = builtin;
                    provider.api_key = Some(key.clone());
                    config.providers.insert(name.clone(), provider);
                }
            }
        }
    }
    config
}

#[test]
fn store_unset_custom_provider_removes_from_both_files() {
    let dir = unique_test_dir("custom-both");
    let mut config = Config::default();
    config.provider = "custom-a".to_string();
    config.model = "model-a".to_string();
    config.providers.insert(
        "custom-a".to_string(),
        ProviderConfig {
            api_key: Some("sk-inline-a".to_string()),
            base_url: Some("https://a.example.com/v1".to_string()),
            ..Default::default()
        },
    );
    config.providers.insert(
        "custom-b".to_string(),
        ProviderConfig {
            api_key: Some("sk-inline-b".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("custom-a".to_string(), "sk-creds-a".to_string());
    creds
        .keys
        .insert("custom-b".to_string(), "sk-creds-b".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    let outcome = store.unset_provider("providers.custom-a").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "custom-a".to_string()
        }
    );

    let reloaded = reload_via_config_load(&dir);
    assert!(!reloaded.providers.contains_key("custom-a"));
    assert!(reloaded.providers.contains_key("custom-b"));

    let creds_raw = fs::read_to_string(dir.join("credentials.toml")).unwrap();
    assert!(
        !creds_raw.contains("custom-a"),
        "credential for removed provider must be gone from credentials.toml"
    );
    assert!(creds_raw.contains("custom-b"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_builtin_provider_clears_credentials() {
    let dir = unique_test_dir("builtin-creds");
    let mut config = Config::default();
    config.providers.insert(
        "anthropic".to_string(),
        ProviderConfig {
            api_key: Some("sk-ant-inline".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("anthropic".to_string(), "sk-ant-creds".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    let outcome = store.unset_provider("providers.anthropic").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::BuiltinProviderDisconnected {
            name: "anthropic".to_string()
        }
    );

    let reloaded = reload_via_config_load(&dir);
    assert!(!reloaded.providers.contains_key("anthropic"));
    assert!(
        reloaded.api_key().is_err(),
        "builtin provider must not have resurrected credential"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_api_key_clears_from_both_sources() {
    let dir = unique_test_dir("apikey-both");
    let mut config = Config::default();
    config.providers.insert(
        "my-gw".to_string(),
        ProviderConfig {
            api_key: Some("sk-inline".to_string()),
            base_url: Some("https://gw.example.com/v1".to_string()),
            protocol: ProviderProtocol::OpenAIChat,
            api_key_env: Some("GW_KEY".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("my-gw".to_string(), "sk-creds".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    let outcome = store.unset_provider("providers.my-gw.api_key").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::ApiKeyCleared {
            name: "my-gw".to_string()
        }
    );

    let reloaded = reload_via_config_load(&dir);
    let gw = reloaded.providers.get("my-gw").unwrap();
    assert!(gw.api_key.is_none(), "api_key must be None after clear");
    assert_eq!(
        gw.base_url.as_deref(),
        Some("https://gw.example.com/v1"),
        "base_url must be preserved"
    );
    assert_eq!(gw.protocol, ProviderProtocol::OpenAIChat);
    assert_eq!(gw.api_key_env.as_deref(), Some("GW_KEY"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_no_credential_resurrection_on_reload() {
    let dir = unique_test_dir("no-resurrection");
    let mut config = Config::default();
    config.provider = "custom-x".to_string();
    config.model = "model-x".to_string();
    config.providers.insert(
        "custom-x".to_string(),
        ProviderConfig {
            api_key: Some("sk-secret-x".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("custom-x".to_string(), "sk-secret-x-creds".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    store.unset_provider("providers.custom-x").unwrap();

    let reloaded = reload_via_config_load(&dir);
    assert!(
        !reloaded.providers.contains_key("custom-x"),
        "provider must NOT be resurrected from credentials.toml on reload"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_credentials_only_builtin() {
    let dir = unique_test_dir("creds-only-builtin");
    let config = Config::default();
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("anthropic".to_string(), "sk-legacy-ant".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    let outcome = store.unset_provider("providers.anthropic").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::BuiltinProviderDisconnected {
            name: "anthropic".to_string()
        }
    );

    let reloaded = reload_via_config_load(&dir);
    assert!(
        !reloaded.providers.contains_key("anthropic"),
        "credentials-only provider must not be recreated on reload"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_credentials_only_custom() {
    let dir = unique_test_dir("creds-only-custom");
    let config = Config::default();
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("old-custom".to_string(), "sk-old".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    let outcome = store.unset_provider("providers.old-custom").unwrap();
    assert_eq!(
        outcome,
        ConfigUnsetOutcome::CustomProviderRemoved {
            name: "old-custom".to_string()
        }
    );

    let reloaded = reload_via_config_load(&dir);
    assert!(
        !reloaded.providers.contains_key("old-custom"),
        "credentials-only custom provider must not be recreated on reload"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_not_found_leaves_both_files_unchanged() {
    let dir = unique_test_dir("not-found");
    let mut config = make_config_with_two_custom_providers();
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("custom-a".to_string(), "sk-a".to_string());
    write_test_credentials(&dir, &creds);

    let config_before = fs::read(dir.join("config.toml")).unwrap();
    let creds_before = fs::read(dir.join("credentials.toml")).unwrap();

    let store = make_store(&dir);
    let err = store.unset_provider("providers.nonexistent").unwrap_err();
    assert!(err.to_string().contains("not found"));

    let config_after = fs::read(dir.join("config.toml")).unwrap();
    let creds_after = fs::read(dir.join("credentials.toml")).unwrap();
    assert_eq!(
        config_before, config_after,
        "config.toml must be byte-identical"
    );
    assert_eq!(
        creds_before, creds_after,
        "credentials.toml must be byte-identical"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_invalid_key_leaves_both_files_unchanged() {
    let dir = unique_test_dir("invalid-key");
    write_test_config(&dir, &make_config_with_two_custom_providers());

    let config_before = fs::read(dir.join("config.toml")).unwrap();

    let store = make_store(&dir);
    let err = store.unset_provider("model").unwrap_err();
    assert!(err.to_string().contains("unsupported unset key"));

    let config_after = fs::read(dir.join("config.toml")).unwrap();
    assert_eq!(config_before, config_after);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_no_temp_residual() {
    let dir = unique_test_dir("no-residual");
    write_test_config(&dir, &make_config_with_two_custom_providers());

    let store = make_store(&dir);
    store.unset_provider("providers.custom-a").unwrap();

    let entries: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !entries.iter().any(|n| n.contains(".atomic-tmp")),
        "no temp files must remain: {entries:?}"
    );
    assert!(
        !entries.iter().any(|n| n.ends_with(".tmp")),
        "no .tmp files must remain: {entries:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_unrelated_credentials_preserved() {
    let dir = unique_test_dir("unrelated-creds");
    let mut config = Config::default();
    config.providers.insert(
        "target".to_string(),
        ProviderConfig {
            api_key: Some("sk-target".to_string()),
            ..Default::default()
        },
    );
    config.providers.insert(
        "keeper".to_string(),
        ProviderConfig {
            api_key: Some("sk-keeper".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("target".to_string(), "sk-target-creds".to_string());
    creds
        .keys
        .insert("keeper".to_string(), "sk-keeper-creds".to_string());
    creds
        .keys
        .insert("orphan".to_string(), "sk-orphan-creds".to_string());
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    store.unset_provider("providers.target").unwrap();

    let creds_raw = fs::read_to_string(dir.join("credentials.toml")).unwrap();
    assert!(!creds_raw.contains("target"));
    assert!(creds_raw.contains("keeper"));
    assert!(creds_raw.contains("orphan"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn store_unset_credential_not_in_files_after_success() {
    let dir = unique_test_dir("no-leak");
    let mut config = Config::default();
    config.providers.insert(
        "secret-gw".to_string(),
        ProviderConfig {
            api_key: Some("sk-super-secret-DO-NOT-LEAK".to_string()),
            ..Default::default()
        },
    );
    write_test_config(&dir, &config);

    let mut creds = Credentials::default();
    creds.keys.insert(
        "secret-gw".to_string(),
        "sk-super-secret-creds-DO-NOT-LEAK".to_string(),
    );
    write_test_credentials(&dir, &creds);

    let store = make_store(&dir);
    store.unset_provider("providers.secret-gw").unwrap();

    let config_raw = fs::read_to_string(dir.join("config.toml")).unwrap();
    let creds_path = dir.join("credentials.toml");
    let creds_raw = if creds_path.exists() {
        fs::read_to_string(&creds_path).unwrap()
    } else {
        String::new()
    };

    assert!(
        !config_raw.contains("DO-NOT-LEAK"),
        "secret must not appear in config.toml after removal"
    );
    assert!(
        !creds_raw.contains("DO-NOT-LEAK"),
        "secret must not appear in credentials.toml after removal"
    );

    let _ = fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// I157 Transaction Atomicity Correction: failure injection + byte identity
// ---------------------------------------------------------------------------

use crate::store::TransactionFs;
use std::cell::Cell;

struct FaultyFs {
    fail_at_write: Option<usize>,
    write_count: Cell<usize>,
}

impl FaultyFs {
    fn new(fail_at_write: Option<usize>) -> Self {
        Self {
            fail_at_write,
            write_count: Cell::new(0),
        }
    }
}

impl TransactionFs for FaultyFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> Result<String, ConfigError> {
        std::fs::read_to_string(path).map_err(ConfigError::IoError)
    }

    fn atomic_write(&self, path: &Path, content: &str) -> Result<(), ConfigError> {
        let n = self.write_count.get();
        self.write_count.set(n + 1);
        if self.fail_at_write == Some(n) {
            return Err(ConfigError::IoError(std::io::Error::other(
                "injected write failure",
            )));
        }
        crate::store::atomic_write_for_test(path, content)
    }

    fn remove_file(&self, path: &Path) -> Result<(), ConfigError> {
        let n = self.write_count.get();
        self.write_count.set(n + 1);
        if self.fail_at_write == Some(n) {
            return Err(ConfigError::IoError(std::io::Error::other(
                "injected remove failure",
            )));
        }
        std::fs::remove_file(path).map_err(ConfigError::IoError)
    }
}

fn write_both_files(
    dir: &Path,
    config: &Config,
    creds: Option<&Credentials>,
) -> (Vec<u8>, Vec<u8>) {
    let config_toml = toml::to_string_pretty(config).unwrap();
    fs::write(dir.join("config.toml"), &config_toml).unwrap();
    let config_bytes = config_toml.as_bytes().to_vec();

    let creds_bytes = if let Some(c) = creds {
        let creds_toml = toml::to_string_pretty(c).unwrap();
        fs::write(dir.join("credentials.toml"), &creds_toml).unwrap();
        creds_toml.as_bytes().to_vec()
    } else {
        Vec::new()
    };

    (config_bytes, creds_bytes)
}

fn read_both_files(dir: &Path) -> (Vec<u8>, Vec<u8>) {
    let config_bytes = fs::read(dir.join("config.toml")).unwrap_or_default();
    let creds_bytes = fs::read(dir.join("credentials.toml")).unwrap_or_default();
    (config_bytes, creds_bytes)
}

#[test]
fn config_write_failure_leaves_both_files_byte_identical() {
    let dir = unique_test_dir("config-fail");
    let mut config = Config::default();
    config.provider = "custom-a".to_string();
    config.model = "model-a".to_string();
    config.providers.insert(
        "custom-a".to_string(),
        ProviderConfig {
            api_key: Some("sk-inline-a".to_string()),
            ..Default::default()
        },
    );
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("custom-a".to_string(), "sk-creds-a".to_string());
    let (config_before, creds_before) = write_both_files(&dir, &config, Some(&creds));

    let store = make_store(&dir);
    let fs = FaultyFs::new(Some(0)); // Fail at first write (config)
    let result = store.unset_provider_inner("providers.custom-a", &fs);
    assert!(result.is_err(), "must return error on config write failure");

    let (config_after, creds_after) = read_both_files(&dir);
    assert_eq!(
        config_before, config_after,
        "config.toml must be byte-identical after config write failure"
    );
    assert_eq!(
        creds_before, creds_after,
        "credentials.toml must be byte-identical after config write failure"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn credentials_write_failure_leaves_config_updated_and_orphan_reconciled_on_reload() {
    let dir = unique_test_dir("creds-fail");
    let mut config = Config::default();
    config.provider = "custom-a".to_string();
    config.model = "model-a".to_string();
    config.providers.insert(
        "custom-a".to_string(),
        ProviderConfig {
            api_key: Some("sk-inline-a".to_string()),
            ..Default::default()
        },
    );
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("custom-a".to_string(), "sk-creds-a".to_string());
    let (_, creds_before) = write_both_files(&dir, &config, Some(&creds));

    let store = make_store(&dir);
    let fs = FaultyFs::new(Some(1)); // Fail at second write (credentials)
    let result = store.unset_provider_inner("providers.custom-a", &fs);
    assert!(
        result.is_err(),
        "must return error on credentials write failure"
    );

    let config_raw = std::fs::read_to_string(dir.join("config.toml")).unwrap();
    let parsed: Config = toml::from_str(&config_raw).unwrap();
    assert!(
        !parsed.providers.contains_key("custom-a"),
        "config must be updated (config-first order)"
    );

    let creds_after = fs::read(dir.join("credentials.toml")).unwrap_or_default();
    assert_eq!(
        creds_before, creds_after,
        "credentials.toml must be unchanged when credentials write fails"
    );

    let reloaded = reload_via_config_load(&dir);
    assert!(
        !reloaded.providers.contains_key("custom-a"),
        "orphan credential must NOT be resurrected on reload"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn credentials_removal_failure_leaves_config_updated_and_orphan_reconciled() {
    let dir = unique_test_dir("creds-remove-fail");
    let mut config = Config::default();
    config.provider = "only-one".to_string();
    config.model = "model-a".to_string();
    config.providers.insert(
        "only-one".to_string(),
        ProviderConfig {
            api_key: Some("sk-only".to_string()),
            ..Default::default()
        },
    );
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("only-one".to_string(), "sk-only-creds".to_string());
    let (_, creds_before) = write_both_files(&dir, &config, Some(&creds));

    let store = make_store(&dir);
    let fs = FaultyFs::new(Some(1)); // Fail at second mutation (credentials remove)
    let result = store.unset_provider_inner("providers.only-one", &fs);
    assert!(result.is_err());

    let config_raw = std::fs::read_to_string(dir.join("config.toml")).unwrap();
    let parsed: Config = toml::from_str(&config_raw).unwrap();
    assert!(!parsed.providers.contains_key("only-one"));

    let creds_after = std::fs::read(dir.join("credentials.toml")).unwrap_or_default();
    assert_eq!(creds_before, creds_after);

    let reloaded = reload_via_config_load(&dir);
    assert!(!reloaded.providers.contains_key("only-one"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn transaction_success_no_temp_residual() {
    let dir = unique_test_dir("no-tmp-residual");
    let mut config = Config::default();
    config.providers.insert(
        "target".to_string(),
        ProviderConfig {
            api_key: Some("sk-target".to_string()),
            ..Default::default()
        },
    );
    write_both_files(&dir, &config, None);

    let store = make_store(&dir);
    store.unset_provider("providers.target").unwrap();

    let entries: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !entries.iter().any(|n| n.contains(".tmp")),
        "no temp files must remain: {entries:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn orphan_credential_for_non_builtin_not_injected_on_load() {
    let dir = unique_test_dir("orphan-skip");
    let mut config = Config::default();
    config.provider = "real-provider".to_string();
    config.model = "model-a".to_string();
    config.providers.insert(
        "real-provider".to_string(),
        ProviderConfig {
            api_key: Some("sk-real".to_string()),
            ..Default::default()
        },
    );
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("orphan-custom".to_string(), "sk-orphan".to_string());
    creds
        .keys
        .insert("real-provider".to_string(), "sk-real-creds".to_string());
    write_both_files(&dir, &config, Some(&creds));

    let reloaded = reload_via_config_load(&dir);
    assert!(
        !reloaded.providers.contains_key("orphan-custom"),
        "orphan credential for non-builtin provider must not be injected"
    );
    assert!(reloaded.providers.contains_key("real-provider"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn builtin_credential_without_config_entry_still_injected() {
    let dir = unique_test_dir("builtin-inject");
    let mut config = Config::default();
    config.provider = "anthropic".to_string();
    config.model = "claude-test".to_string();
    write_both_files(&dir, &config, None);

    let mut creds = Credentials::default();
    creds
        .keys
        .insert("anthropic".to_string(), "sk-ant-legacy".to_string());
    write_both_files(&dir, &config, Some(&creds));

    let reloaded = reload_via_config_load(&dir);
    assert!(
        reloaded.providers.contains_key("anthropic"),
        "builtin provider credential must still be injected"
    );
    assert_eq!(
        reloaded
            .providers
            .get("anthropic")
            .unwrap()
            .api_key
            .as_deref(),
        Some("sk-ant-legacy")
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn success_writes_config_first_then_credentials() {
    let dir = unique_test_dir("order-check");
    let mut config = Config::default();
    config.providers.insert(
        "gw-a".to_string(),
        ProviderConfig {
            api_key: Some("sk-a".to_string()),
            ..Default::default()
        },
    );
    config.providers.insert(
        "gw-b".to_string(),
        ProviderConfig {
            api_key: Some("sk-b".to_string()),
            ..Default::default()
        },
    );
    let mut creds = Credentials::default();
    creds
        .keys
        .insert("gw-a".to_string(), "sk-a-creds".to_string());
    creds
        .keys
        .insert("gw-b".to_string(), "sk-b-creds".to_string());
    write_both_files(&dir, &config, Some(&creds));

    let store = make_store(&dir);
    store.unset_provider("providers.gw-a").unwrap();

    let (config_after, creds_after) = read_both_files(&dir);
    let parsed_config: Config = toml::from_str(&String::from_utf8_lossy(&config_after)).unwrap();
    assert!(!parsed_config.providers.contains_key("gw-a"));
    assert!(parsed_config.providers.contains_key("gw-b"));

    let parsed_creds: Credentials =
        toml::from_str(&String::from_utf8_lossy(&creds_after)).unwrap_or_default();
    assert!(!parsed_creds.keys.contains_key("gw-a"));
    assert!(parsed_creds.keys.contains_key("gw-b"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unrelated_config_sections_preserved_after_unset() {
    let dir = unique_test_dir("unrelated-sections");
    let mut config = Config::default();
    config.provider = "keep-me".to_string();
    config.model = "model-x".to_string();
    config.providers.insert(
        "keep-me".to_string(),
        ProviderConfig {
            api_key: Some("sk-keep".to_string()),
            base_url: Some("https://keep.example.com".to_string()),
            protocol: ProviderProtocol::OpenAIChat,
            api_key_env: Some("KEEP_KEY".to_string()),
            models: HashMap::from([(
                "model-x".to_string(),
                ModelConfig {
                    context_limit: Some(200_000),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    );
    config.providers.insert(
        "remove-me".to_string(),
        ProviderConfig {
            api_key: Some("sk-remove".to_string()),
            ..Default::default()
        },
    );
    write_both_files(&dir, &config, None);

    let store = make_store(&dir);
    store.unset_provider("providers.remove-me").unwrap();

    let config_after = std::fs::read_to_string(dir.join("config.toml")).unwrap();
    let parsed: Config = toml::from_str(&config_after).unwrap();
    let keep = parsed.providers.get("keep-me").unwrap();
    assert_eq!(keep.base_url.as_deref(), Some("https://keep.example.com"));
    assert_eq!(keep.api_key_env.as_deref(), Some("KEEP_KEY"));
    assert!(keep.models.contains_key("model-x"));
    assert_eq!(
        keep.models.get("model-x").unwrap().context_limit,
        Some(200_000)
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn credential_not_in_error_or_debug() {
    let dir = unique_test_dir("no-secret-err");
    let mut config = Config::default();
    config.providers.insert(
        "secret-gw".to_string(),
        ProviderConfig {
            api_key: Some("sk-MARKER-DO-NOT-LEAK".to_string()),
            ..Default::default()
        },
    );
    write_both_files(&dir, &config, None);

    let store = make_store(&dir);
    let err = store.unset_provider("providers.nonexistent").unwrap_err();
    assert!(
        !err.to_string().contains("MARKER-DO-NOT-LEAK"),
        "error must not contain credential marker"
    );
    assert!(
        !format!("{err:?}").contains("MARKER-DO-NOT-LEAK"),
        "Debug must not contain credential marker"
    );

    let _ = fs::remove_dir_all(&dir);
}
