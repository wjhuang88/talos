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
fn test_custom_provider_api_key_env() {
    let _lock = ENV_MUTEX.lock().unwrap();
    unsafe { env::set_var("DASHSCOPE_API_KEY", "dashscope-key") };
    let config = Config {
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
    config.model = "claude-sonnet-4-5-20250929".to_string();
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
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5-20250929".to_string(),
        providers: HashMap::from([(
            "anthropic".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "claude-sonnet-4-5-20250929".to_string(),
                    ModelConfig {
                        context_limit: Some(150_000),
                        output_limit: Some(8000),
                        reasoning: None,
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
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5-20250929".to_string(),
        providers: HashMap::from([(
            "anthropic".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "claude-sonnet-4-5-20250929".to_string(),
                    ModelConfig {
                        context_limit: Some(100_000),
                        output_limit: None,
                        reasoning: None,
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
        .set_active_model("claude-sonnet-4-5-20250929")
        .unwrap();
    assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    assert_eq!(config.provider, "anthropic");
    assert!(config.providers.contains_key("anthropic"));
}

#[test]
fn test_set_active_model_openai() {
    let mut config = Config::default();
    config.set_active_model("gpt-4.1").unwrap();
    assert_eq!(config.model, "gpt-4.1");
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
    config.model = "claude-sonnet-4-5-20250929".to_string();
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
model = "claude-sonnet-4-5-20250929"
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
model = "claude-sonnet-4-5-20250929"

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
    // glm-5.2 exists under zhipu and zai (among others). The lookup must
    // succeed for the specified provider, not fall back to the conservative
    // default or silently resolve to a different provider's entry.
    let zhipu = Config {
        provider: "zhipu".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx, _) = zhipu.resolve_model_limits();
    assert_eq!(ctx, 1_000_000);

    let zai = Config {
        provider: "zai".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let (ctx2, _) = zai.resolve_model_limits();
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
        model: "claude-sonnet-4-5-20250929".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    let err = config.set_active_model("glm-5.2").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("multiple providers"),
        "expected ambiguity error, got: {msg}"
    );
    assert!(msg.contains("zhipu"));
    assert!(msg.contains("zai"));
}

#[test]
fn test_set_active_model_provider_qualified_resolves_correctly() {
    let mut config = Config {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-5-20250929".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    config.set_active_model("zai/glm-5.2").unwrap();
    assert_eq!(config.model, "glm-5.2");
    assert_eq!(config.provider, "zai");
    assert!(config.providers.contains_key("zai"));
}

#[test]
fn test_set_active_model_unique_bare_id_still_works() {
    let mut config = Config {
        provider: "zai".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::new(),
        ..Default::default()
    };
    // claude-sonnet-4-5 is unique to anthropic — bare ID should resolve.
    config.set_active_model("claude-sonnet-4-5").unwrap();
    assert_eq!(config.model, "claude-sonnet-4-5");
    assert_eq!(config.provider, "anthropic");
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
    assert!(providers.contains(&"zhipu"));
    assert!(providers.contains(&"zai"));
}

#[test]
fn test_all_models_user_override_matches_by_provider_and_id() {
    let config = Config {
        provider: "zai".to_string(),
        model: "glm-5.2".to_string(),
        providers: HashMap::from([(
            "zai".to_string(),
            ProviderConfig {
                models: HashMap::from([(
                    "glm-5.2".to_string(),
                    ModelConfig {
                        context_limit: Some(50_000),
                        output_limit: Some(1000),
                        reasoning: None,
                    },
                )]),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    let all = config.all_models();
    // The zai entry should be overridden, NOT the zhipu entry.
    let zai = all
        .iter()
        .find(|m| m.id == "glm-5.2" && m.provider == "zai")
        .unwrap();
    assert_eq!(zai.context_limit, Some(50_000));
    assert_eq!(zai.output_limit, Some(1000));
    // The zhipu entry should be untouched.
    let zhipu = all
        .iter()
        .find(|m| m.id == "glm-5.2" && m.provider == "zhipu")
        .unwrap();
    assert_eq!(zhipu.context_limit, Some(1_000_000));
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
fn test_skill_config_default_off() {
    let config = Config::default();
    assert!(!config.skills.discover_shared);
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
                },
            )]),
            ..Default::default()
        },
    );

    let catalog_models = vec![model::ModelMetadata {
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
                },
            )]),
            ..Default::default()
        },
    );

    let catalog_models = vec![model::ModelMetadata {
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
    config.model = "claude-sonnet-4-5-20250929".to_string();

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
    config.model = "claude-sonnet-4-5-20250929".to_string();

    let empty_catalog: Vec<model::ModelMetadata> = vec![];
    let (ctx, _) = config.resolve_model_limits_with_catalog(Some(&empty_catalog));
    assert!(ctx > 0, "should fall back to builtin, not block");
}
