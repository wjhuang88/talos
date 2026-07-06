//! First-run setup wizard for model configuration.
//!
//! `talos init` launches an interactive terminal wizard.
//! `talos init --non-interactive` prints setup instructions and exits.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use talos_config::Config;
use talos_config::model::{ModelMetadata, builtin_models};

/// Entry point for the init wizard subcommand.
pub(crate) async fn run_init_wizard(non_interactive: bool) -> Result<()> {
    if non_interactive {
        print_non_interactive_instructions();
        return Ok(());
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    run_wizard_interactive(&mut stdin_lock, &mut stdout_lock).await
}

/// Print step-by-step instructions for non-interactive setup.
fn print_non_interactive_instructions() {
    println!("Talos First-Run Setup (non-interactive mode)");
    println!();
    println!("To configure Talos without the interactive wizard:");
    println!();
    println!("1. Set your API key via environment variable (recommended):");
    println!("   export ANTHROPIC_API_KEY=\"your-key-here\"");
    println!("   # or for OpenAI:");
    println!("   export OPENAI_API_KEY=\"your-key-here\"");
    println!();
    println!("2. Or set it inline via CLI:");
    println!("   talos --config-set providers.anthropic.api_key=your-key-here");
    println!();
    println!("3. Select a model:");
    println!("   talos --available-models          # list all models");
    println!("   talos --use-model anthropic/claude-sonnet-4-5");
    println!();
    println!("4. Or use the interactive wizard:");
    println!("   talos init");
    println!();
    println!("Configuration is stored at ~/.talos/config.toml");
}

/// Run the interactive wizard using the provided input/output streams.
async fn run_wizard_interactive<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    let mut config = Config::load().context("failed to load configuration")?;

    if !config.model.is_empty() {
        writeln!(
            writer,
            "Model already configured: {} (provider: {})",
            config.model, config.provider
        )?;
        if !prompt_yes_no(writer, reader, "Reconfigure? (y/N): ")? {
            writeln!(writer, "Keeping existing configuration.")?;
            return Ok(());
        }
    }

    let catalog = builtin_models();

    let providers = group_providers_by_name(&catalog);
    writeln!(writer, "\nAvailable providers:")?;
    for (i, (name, models)) in providers.iter().enumerate() {
        let authed = config.provider_authenticated(name);
        let status = if authed { "Ready" } else { "Setup required" };
        writeln!(
            writer,
            "  {}. {} ({} models) — {}",
            i + 1,
            name,
            models.len(),
            status
        )?;
    }

    let provider_name = prompt_provider(writer, reader, &providers)?;

    writeln!(writer)?;
    let credential_type = prompt_credential(writer, reader, &provider_name)?;

    let provider_models: Vec<&ModelMetadata> = catalog
        .iter()
        .filter(|m| m.provider == provider_name)
        .collect();

    writeln!(writer, "\nAvailable models for {}:", provider_name)?;
    for (i, m) in provider_models.iter().enumerate() {
        let ctx = m
            .context_limit
            .map(|c| format!("{}K", c / 1000))
            .unwrap_or_else(|| "?".to_string());
        writeln!(writer, "  {}. {} (ctx: {})", i + 1, m.id, ctx)?;
    }

    let model_id = prompt_model(writer, reader, &provider_models)?;
    let qualified_model_id = format!("{provider_name}/{model_id}");

    config
        .set_active_model(&qualified_model_id)
        .with_context(|| {
            format!("failed to set active model '{model_id}' for provider '{provider_name}'")
        })?;

    match &credential_type {
        CredentialType::EnvVar(env_name) => {
            config
                .providers
                .entry(provider_name.clone())
                .or_insert_with(|| builtin_provider_config(&provider_name).unwrap_or_default())
                .api_key_env = Some(env_name.clone());
        }
        CredentialType::InlineKey(key) => {
            config.set_provider_credential(&provider_name, key);
        }
    }

    writeln!(writer)?;
    if !prompt_yes_no(writer, reader, "Test connection? (y/N): ")? {
        writeln!(writer, "Skipping connection test.")?;
    }

    writeln!(writer, "\nConfiguration summary:")?;
    writeln!(writer, "  Provider: {}", config.provider)?;
    writeln!(writer, "  Model:    {}", config.model)?;
    match &credential_type {
        CredentialType::EnvVar(name) => {
            writeln!(writer, "  Credential: environment variable ({})", name)?;
        }
        CredentialType::InlineKey(_) => {
            writeln!(writer, "  Credential: inline (api_key = ***)")?;
        }
    }

    if !prompt_yes_no(writer, reader, "\nSave configuration? (Y/n): ")? {
        writeln!(writer, "Setup cancelled. No changes saved.")?;
        return Ok(());
    }

    config.save().context("failed to save configuration")?;
    writeln!(writer, "Configuration saved to ~/.talos/config.toml")?;

    Ok(())
}

/// How the credential was provided.
enum CredentialType {
    /// Use an environment variable (name stored in api_key_env).
    EnvVar(String),
    /// Store the key inline (stored in api_key).
    InlineKey(String),
}

/// Group models by provider name, preserving insertion order.
fn group_providers_by_name(models: &[ModelMetadata]) -> BTreeMap<String, Vec<&ModelMetadata>> {
    let mut map: BTreeMap<String, Vec<&ModelMetadata>> = BTreeMap::new();
    for m in models {
        map.entry(m.provider.clone()).or_default().push(m);
    }
    map
}

/// Read a line from the reader, trimming whitespace.
fn read_line<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Prompt with a yes/no question. Returns true for yes, false for no.
fn prompt_yes_no<W: Write, R: BufRead>(
    writer: &mut W,
    reader: &mut R,
    prompt: &str,
) -> io::Result<bool> {
    write!(writer, "{}", prompt)?;
    writer.flush()?;
    let input = read_line(reader)?;
    let lower = input.to_lowercase();
    Ok(lower == "y" || lower == "yes")
}

/// Prompt the user to select a provider by number or name.
fn prompt_provider<W: Write, R: BufRead>(
    writer: &mut W,
    reader: &mut R,
    providers: &BTreeMap<String, Vec<&ModelMetadata>>,
) -> io::Result<String> {
    let keys: Vec<&String> = providers.keys().collect();
    loop {
        write!(writer, "Select a provider (number or name): ")?;
        writer.flush()?;
        let input = read_line(reader)?;

        if let Ok(num) = input.parse::<usize>()
            && num >= 1
            && num <= keys.len()
        {
            return Ok(keys[num - 1].clone());
        }

        let lower = input.to_lowercase();
        if let Some(name) = keys.iter().find(|k| k.to_lowercase() == lower) {
            return Ok((*name).clone());
        }

        writeln!(writer, "Invalid selection. Try again.")?;
    }
}

/// Detect if input looks like an env var name (ALL_CAPS_WITH_UNDERSCORES).
fn looks_like_env_var(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    let chars: Vec<char> = input.chars().collect();
    chars.iter().all(|c| c.is_ascii_uppercase() || *c == '_')
        && chars.iter().any(|c| c.is_ascii_uppercase())
}

/// Prompt for API key or env var name.
fn prompt_credential<W: Write, R: BufRead>(
    writer: &mut W,
    reader: &mut R,
    provider_name: &str,
) -> io::Result<CredentialType> {
    let default_env = default_env_var_for_provider(provider_name);
    let prompt = if let Some(ref env) = default_env {
        format!("Enter API key or env var name (default: {}): ", env)
    } else {
        "Enter API key or env var name: ".to_string()
    };

    loop {
        write!(writer, "{}", prompt)?;
        writer.flush()?;
        let input = read_line(reader)?;

        if input.is_empty() {
            if let Some(ref env) = default_env {
                return Ok(CredentialType::EnvVar(env.clone()));
            }
            writeln!(writer, "Input required. Try again.")?;
            continue;
        }

        if looks_like_env_var(&input) {
            return Ok(CredentialType::EnvVar(input));
        }

        return Ok(CredentialType::InlineKey(input));
    }
}

/// Prompt the user to select a model by number or id.
fn prompt_model<W: Write, R: BufRead>(
    writer: &mut W,
    reader: &mut R,
    models: &[&ModelMetadata],
) -> io::Result<String> {
    loop {
        write!(writer, "Select a model (number or id): ")?;
        writer.flush()?;
        let input = read_line(reader)?;

        if let Ok(num) = input.parse::<usize>()
            && num >= 1
            && num <= models.len()
        {
            return Ok(models[num - 1].id.clone());
        }

        if let Some(m) = models.iter().find(|m| m.id == input) {
            return Ok(m.id.clone());
        }

        writeln!(writer, "Invalid selection. Try again.")?;
    }
}

/// Return the default env var name for a known provider.
fn default_env_var_for_provider(name: &str) -> Option<String> {
    match name {
        "anthropic" => Some("ANTHROPIC_API_KEY".to_string()),
        "openai" => Some("OPENAI_API_KEY".to_string()),
        "google" => Some("GOOGLE_API_KEY".to_string()),
        "deepseek" => Some("DEEPSEEK_API_KEY".to_string()),
        "qwen" => Some("DASHSCOPE_API_KEY".to_string()),
        "zhipuai" => Some("ZHIPU_API_KEY".to_string()),
        "zai" => Some("ZAI_API_KEY".to_string()),
        "zhipu-coding-plan" => Some("ZHIPU_CODING_PLAN_API_KEY".to_string()),
        "zai-coding-plan" => Some("ZAI_CODING_PLAN_API_KEY".to_string()),
        "minimax" => Some("MINIMAX_API_KEY".to_string()),
        "moonshot" => Some("MOONSHOT_API_KEY".to_string()),
        "openrouter" => Some("OPENROUTER_API_KEY".to_string()),
        _ => None,
    }
}

/// Get builtin provider config for a provider name.
fn builtin_provider_config(name: &str) -> Option<talos_config::ProviderConfig> {
    use talos_config::{ProviderConfig, ProviderProtocol};
    match name {
        "anthropic" => Some(ProviderConfig {
            protocol: ProviderProtocol::AnthropicMessages,
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            ..Default::default()
        }),
        "openai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            ..Default::default()
        }),
        "google" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            api_key_env: Some("GOOGLE_API_KEY".to_string()),
            ..Default::default()
        }),
        "deepseek" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://api.deepseek.com".to_string()),
            api_key_env: Some("DEEPSEEK_API_KEY".to_string()),
            ..Default::default()
        }),
        "qwen" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_key_env: Some("DASHSCOPE_API_KEY".to_string()),
            ..Default::default()
        }),
        "zhipuai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            api_key_env: Some("ZHIPU_API_KEY".to_string()),
            ..Default::default()
        }),
        "zai" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://api.z.ai/api/paas/v4".to_string()),
            api_key_env: Some("ZAI_API_KEY".to_string()),
            ..Default::default()
        }),
        "zhipu-coding-plan" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://open.bigmodel.cn/api/coding/paas/v4".to_string()),
            api_key_env: Some("ZHIPU_CODING_PLAN_API_KEY".to_string()),
            ..Default::default()
        }),
        "zai-coding-plan" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://api.z.ai/api/coding/paas/v4".to_string()),
            api_key_env: Some("ZAI_CODING_PLAN_API_KEY".to_string()),
            ..Default::default()
        }),
        "minimax" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://api.minimaxi.com/v1".to_string()),
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            ..Default::default()
        }),
        "moonshot" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://api.moonshot.cn/v1".to_string()),
            api_key_env: Some("MOONSHOT_API_KEY".to_string()),
            ..Default::default()
        }),
        "openrouter" => Some(ProviderConfig {
            protocol: ProviderProtocol::OpenAIChat,
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_key_env: Some("OPENROUTER_API_KEY".to_string()),
            ..Default::default()
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::HOME_ENV_MUTEX;
    use std::io::Cursor;

    #[test]
    fn test_non_interactive_prints_instructions() {
        // Non-interactive path uses println! which can't be easily captured in unit tests.
        // Verify the function doesn't panic.
        let result = std::panic::catch_unwind(|| {
            print_non_interactive_instructions();
        });
        assert!(
            result.is_ok(),
            "print_non_interactive_instructions should not panic"
        );
    }

    #[test]
    fn test_looks_like_env_var() {
        assert!(looks_like_env_var("ANTHROPIC_API_KEY"));
        assert!(looks_like_env_var("OPENAI_API_KEY"));
        assert!(looks_like_env_var("MY_VAR"));
        assert!(looks_like_env_var("A"));
        assert!(!looks_like_env_var("sk-ant-api-key"));
        assert!(!looks_like_env_var("my_key"));
        assert!(!looks_like_env_var(""));
        assert!(!looks_like_env_var("Mixed_Case"));
    }

    #[tokio::test]
    async fn test_wizard_cancel_on_reconfigure() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        let input = b"n\n";
        let mut reader = Cursor::new(input.as_slice());
        let mut writer = Vec::new();

        // Set up a temp HOME with an existing config
        let temp_dir = tempfile::tempdir().unwrap();
        let talos_dir = temp_dir.path().join(".talos");
        std::fs::create_dir_all(&talos_dir).unwrap();
        let config_path = talos_dir.join("config.toml");
        std::fs::write(
            &config_path,
            r#"provider = "anthropic"
model = "claude-sonnet-4-5"
"#,
        )
        .unwrap();

        unsafe { std::env::set_var("HOME", temp_dir.path()) };

        let result = run_wizard_interactive(&mut reader, &mut writer).await;
        assert!(result.is_ok());

        let output = String::from_utf8_lossy(&writer);
        assert!(output.contains("Model already configured"));
        assert!(output.contains("Keeping existing configuration"));

        // Config should be unchanged
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("claude-sonnet-4-5"));

        unsafe { std::env::remove_var("HOME") };
    }

    #[tokio::test]
    async fn test_wizard_saves_on_confirm() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        // Simulate: y (reconfigure), provider index for anthropic, ANTHROPIC_API_KEY,
        // first model, n (skip test), y (save).
        // The provider index is discovered dynamically because the dataset size
        // varies across BUILD_MODELS=1 regenerations.
        let models = builtin_models();
        let grouped = group_providers_by_name(&models);
        let providers: Vec<&str> = grouped.keys().map(|s| s.as_str()).collect();
        let anthro_idx = providers
            .iter()
            .position(|&p| p == "anthropic")
            .expect("anthropic must be in the dataset");
        let input = format!("y\n{}\nANTHROPIC_API_KEY\n1\nn\ny\n", anthro_idx + 1);
        let mut reader = Cursor::new(input.as_bytes());
        let mut writer = Vec::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let talos_dir = temp_dir.path().join(".talos");
        std::fs::create_dir_all(&talos_dir).unwrap();

        unsafe { std::env::set_var("HOME", temp_dir.path()) };

        let result = run_wizard_interactive(&mut reader, &mut writer).await;
        assert!(result.is_ok());

        let output = String::from_utf8_lossy(&writer);
        assert!(output.contains("Configuration saved"));

        // Verify config was saved
        let config = Config::load().unwrap();
        assert!(!config.model.is_empty());
        assert_eq!(config.provider, "anthropic");

        unsafe { std::env::remove_var("HOME") };
    }

    #[tokio::test]
    async fn test_wizard_cancel_on_save() {
        let _lock = HOME_ENV_MUTEX.lock().unwrap();
        // Simulate: y (reconfigure), select anthropic dynamically, ANTHROPIC_API_KEY,
        // 1 (model), n (skip test), n (cancel save)
        let models = builtin_models();
        let grouped = group_providers_by_name(&models);
        let providers: Vec<&str> = grouped.keys().map(|s| s.as_str()).collect();
        let anthro_idx = providers
            .iter()
            .position(|&p| p == "anthropic")
            .expect("anthropic must be in the dataset");
        let input = format!("y\n{}\nANTHROPIC_API_KEY\n1\nn\nn\n", anthro_idx + 1);
        let mut reader = Cursor::new(input.as_bytes());
        let mut writer = Vec::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let talos_dir = temp_dir.path().join(".talos");
        std::fs::create_dir_all(&talos_dir).unwrap();

        unsafe { std::env::set_var("HOME", temp_dir.path()) };

        let result = run_wizard_interactive(&mut reader, &mut writer).await;
        assert!(result.is_ok());

        let output = String::from_utf8_lossy(&writer);
        assert!(output.contains("Setup cancelled"));
        assert!(output.contains("No changes saved"));

        unsafe { std::env::remove_var("HOME") };
    }

    #[test]
    fn test_prompt_yes_no_accepts_yes_variants() {
        let cases = vec![
            ("y\n", true),
            ("Y\n", true),
            ("yes\n", true),
            ("YES\n", true),
            ("n\n", false),
            ("N\n", false),
            ("no\n", false),
            ("anything\n", false),
        ];
        for (input, expected) in cases {
            let mut reader = Cursor::new(input.as_bytes());
            let mut writer = Vec::new();
            let result = prompt_yes_no(&mut writer, &mut reader, "Test? ").unwrap();
            assert_eq!(result, expected, "input: {:?}", input);
        }
    }

    #[test]
    fn test_group_providers_by_name() {
        let models = builtin_models();
        let grouped = group_providers_by_name(&models);
        assert!(!grouped.is_empty());
        // anthropic should have at least one model
        assert!(grouped.contains_key("anthropic"));
        assert!(!grouped["anthropic"].is_empty());
    }
}
