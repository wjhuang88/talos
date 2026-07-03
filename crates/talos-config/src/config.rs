use crate::{
    Config, ConfigError, Credentials, ModelConfig, ProviderConfig, ProviderProtocol,
    builtin_provider_config, home_dir, model, opencode, substitute_env_vars,
};
use std::env;
use std::fs;
use std::path::PathBuf;

impl Config {
    /// Returns the default path for the configuration file: `~/.talos/config.toml`.
    pub fn default_path() -> PathBuf {
        let mut path = home_dir();
        path.push(".talos");
        path.push("config.toml");
        path
    }

    /// Loads configuration from the default path `~/.talos/config.toml`.
    ///
    /// If the file exists, it is read and environment variable substitution
    /// is performed. If the file does not exist, returns a default config
    /// (env-only mode).
    ///
    /// Does **not** enforce [`Config::validate`]'s "must be runnable" rules
    /// (non-empty `model`/`provider`, credential presence). An on-disk config
    /// with an empty `model` is a legitimate, expected state — for example,
    /// before the first-run setup wizard runs, or before `talos config set`
    /// has a chance to populate it. Callers that need a fully-configured,
    /// runnable session (interactive TUI/print/RPC modes) check
    /// `config.model.is_empty()` themselves and route to the setup wizard or
    /// a helpful error. Callers that persist an edit (`talos config set`)
    /// call [`Config::validate`] explicitly after applying the edit, before
    /// saving.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::default_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = fs::read_to_string(&path)?;
        let substituted = substitute_env_vars(&raw);
        let mut config: Config =
            toml::from_str(&substituted).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        if let Ok(creds) = Credentials::load() {
            config.merge_credentials(&creds);
        }

        Ok(config)
    }

    /// Returns the API key for the current provider.
    ///
    /// Resolution order:
    /// 1. Inline `providers.<name>.api_key` from the config file.
    /// 2. Provider-specific env var from `providers.<name>.api_key_env`.
    /// 3. Built-in provider env var: `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`.
    /// 4. For the built-in OpenAI provider only, additionally `OPENAI_COMPAT_API_KEY`.
    ///    This is the conventional name for keys issued by OpenAI-compatible
    ///    gateways (DashScope / Bailian / Z.ai / self-hosted). It is checked
    ///    after `OPENAI_API_KEY` so existing setups are unaffected.
    pub fn api_key(&self) -> Result<String, ConfigError> {
        let provider = self.active_provider_config();

        if let Some(key) = provider.api_key.as_deref()
            && !key.is_empty()
        {
            return Ok(key.to_string());
        }

        let primary = provider
            .api_key_env
            .as_deref()
            .unwrap_or(match self.provider.as_str() {
                "anthropic" => "ANTHROPIC_API_KEY",
                "openai" => "OPENAI_API_KEY",
                _ => "",
            });

        if !primary.is_empty()
            && let Ok(key) = env::var(primary)
            && !key.is_empty()
        {
            return Ok(key);
        }

        if self.provider == "openai"
            && let Ok(key) = env::var("OPENAI_COMPAT_API_KEY")
            && !key.is_empty()
        {
            return Ok(key);
        }

        Err(ConfigError::MissingApiKey(
            self.provider.clone(),
            if self.provider == "openai" {
                "OPENAI_API_KEY or OPENAI_COMPAT_API_KEY".to_string()
            } else if primary.is_empty() {
                format!("providers.{}.api_key or api_key_env", self.provider)
            } else {
                primary.to_string()
            },
        ))
    }

    /// Returns the optional base URL override.
    ///
    /// `None` means "use the provider's hard-coded default endpoint".
    /// `Some(url)` is sent verbatim to the provider's HTTP client via
    /// `with_base_url()`. Honored for both OpenAI and Anthropic providers.
    pub fn base_url(&self) -> Option<String> {
        self.providers
            .get(&self.provider)
            .and_then(|p| p.base_url.clone())
            .or_else(|| builtin_provider_config(&self.provider).and_then(|p| p.base_url))
    }

    /// Returns the active provider protocol.
    #[must_use]
    pub fn provider_protocol(&self) -> ProviderProtocol {
        self.active_provider_config().protocol
    }

    #[must_use]
    pub fn tool_protocol(&self) -> talos_core::tool::ToolProtocol {
        self.active_provider_config().tool_protocol
    }

    /// Returns the configured context limit for the active provider/model.
    #[must_use]
    pub fn context_limit(&self) -> Option<u32> {
        self.active_model_config()
            .and_then(|model| model.context_limit)
    }

    /// Returns the configured output limit for the active provider/model.
    #[must_use]
    pub fn output_limit(&self) -> Option<u32> {
        self.active_model_config()
            .and_then(|model| model.output_limit)
    }

    /// Resolves model limits using the precedence chain:
    /// 1. User-configured limits in `~/.talos/config.toml`.
    /// 2. Built-in catalog from `builtin_models()`.
    /// 3. Conservative fallback `(128_000, None)`.
    #[must_use]
    pub fn resolve_model_limits(&self) -> (u32, Option<u32>) {
        self.resolve_model_limits_with_catalog(None)
    }

    /// Resolves model limits with an optional catalog overlay.
    ///
    /// Precedence (highest first):
    /// 1. User-configured limits in `~/.talos/config.toml`.
    /// 2. Catalog DB data (when `catalog` is `Some`).
    /// 3. Built-in catalog from `builtin_models()`.
    /// 4. Conservative fallback `(128_000, None)`.
    ///
    /// When `catalog` is `None` (catalog unavailable or corrupt), the chain
    /// degrades gracefully to built-in data then fallback — startup is never
    /// blocked by catalog DB failure.
    #[must_use]
    pub fn resolve_model_limits_with_catalog(
        &self,
        catalog: Option<&[model::ModelMetadata]>,
    ) -> (u32, Option<u32>) {
        const CONSERVATIVE_FALLBACK: u32 = 128_000;

        if let Some(model_config) = self.active_model_config()
            && let Some(ctx) = model_config.context_limit
        {
            return (ctx, model_config.output_limit);
        }

        if let Some(catalog_models) = catalog
            && let Some(meta) =
                model::find_model_by_provider(catalog_models, &self.provider, &self.model)
            && let Some(ctx) = meta.context_limit
        {
            return (ctx, meta.output_limit);
        }

        let builtins = model::builtin_models();
        if let Some(meta) = model::find_model_by_provider(&builtins, &self.provider, &self.model)
            && let Some(ctx) = meta.context_limit
        {
            return (ctx, meta.output_limit);
        }

        (CONSERVATIVE_FALLBACK, None)
    }

    /// Returns the active provider config with built-in defaults applied.
    #[must_use]
    pub fn active_provider_config(&self) -> ProviderConfig {
        let mut config = builtin_provider_config(&self.provider).unwrap_or_default();
        if let Some(user_config) = self.providers.get(&self.provider) {
            if !matches!(user_config.protocol, ProviderProtocol::OpenAIChat)
                || builtin_provider_config(&self.provider).is_none()
            {
                config.protocol = user_config.protocol.clone();
            }
            if user_config.base_url.is_some() {
                config.base_url = user_config.base_url.clone();
            }
            if user_config.api_key.is_some() {
                config.api_key = user_config.api_key.clone();
            }
            if user_config.api_key_env.is_some() {
                config.api_key_env = user_config.api_key_env.clone();
            }
            config.models.extend(user_config.models.clone());
            config.timeout = user_config.timeout.clone();
        }
        config
    }

    fn active_model_config(&self) -> Option<ModelConfig> {
        self.active_provider_config()
            .models
            .get(&self.model)
            .cloned()
    }

    /// Validates the configuration against its JSON Schema.
    ///
    /// The schema is generated via `schemars` and the same constraints are
    /// enforced manually (model must be non-empty, provider must be valid).
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Generate schema for external tooling (documentation, IDE support)
        let _schema = schemars::schema_for!(Config);

        // Manual validation of required constraints
        if self.provider.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "'provider' is required and must be non-empty".to_string(),
            ));
        }

        if self.model.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "'model' is required and must be non-empty".to_string(),
            ));
        }

        let provider = self.active_provider_config();
        if self.providers.contains_key(&self.provider)
            && provider.api_key.is_none()
            && provider.api_key_env.is_none()
        {
            return Err(ConfigError::InvalidConfig(format!(
                "provider '{}' must set api_key or api_key_env",
                self.provider
            )));
        }

        if let Some(ref mc) = self.active_model_config()
            && let Some(ref reasoning) = mc.reasoning
        {
            let builtin = model::builtin_models();
            if let Some(meta) = model::find_model_by_provider(&builtin, &self.provider, &self.model)
                && !meta.capabilities.reasoning
            {
                tracing::warn!(
                    provider = %self.provider,
                    model = %self.model,
                    "reasoning configured but model lacks reasoning capability in catalog; skipping reasoning fields"
                );
            }
            if !reasoning.replay {
                tracing::warn!(
                    provider = %self.provider,
                    model = %self.model,
                    "reasoning replay disabled: Anthropic tool continuations will run without thinking; local-server KV caches may be invalidated"
                );
            }
        }

        Ok(())
    }

    /// Import opencode-style provider definitions and merge them into this config.
    ///
    /// Imports are one-way: opencode config is translated into Talos
    /// `ProviderConfig` values, but Talos config remains the source of truth
    /// after import.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ParseError`] if the input is not valid JSON or
    /// does not match the expected opencode provider schema.
    pub fn import_opencode_providers(&mut self, json: &str) -> Result<(), ConfigError> {
        let imported = opencode::import_opencode_providers(json)?;
        self.providers.extend(imported);
        Ok(())
    }

    /// Persists the current configuration to `~/.talos/config.toml`.
    ///
    /// `api_key` values are serialized in the main config file. Run
    /// `talos config list` or `talos config get` to view config without
    /// leaking keys.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str =
            toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        fs::write(&path, toml_str)?;
        Ok(())
    }

    /// Checks whether the named provider has a usable API key.
    ///
    /// Returns `true` if the provider's inline `api_key` is set, or if its
    /// `api_key_env` resolves to a non-empty environment variable.
    pub fn provider_authenticated(&self, name: &str) -> bool {
        let provider = match self.providers.get(name) {
            Some(p) => p.clone(),
            None => match builtin_provider_config(name) {
                Some(p) => p,
                None => return false,
            },
        };

        if let Some(key) = provider.api_key.as_deref()
            && !key.is_empty()
        {
            return true;
        }

        let env_var = provider.api_key_env.as_deref().unwrap_or(match name {
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            _ => "",
        });

        if !env_var.is_empty()
            && let Ok(key) = env::var(env_var)
            && !key.is_empty()
        {
            return true;
        }

        if name == "openai"
            && let Ok(key) = env::var("OPENAI_COMPAT_API_KEY")
            && !key.is_empty()
        {
            return true;
        }

        false
    }

    /// Returns all available models: built-in catalog merged with user-configured
    /// models from `providers.*.models`.
    pub fn all_models(&self) -> Vec<model::ModelMetadata> {
        self.all_models_with_catalog(None)
    }

    /// Returns all available models with an optional catalog overlay.
    ///
    /// Merge precedence (each layer overrides the previous for matching
    /// `(provider, model_id)` pairs):
    /// 1. Built-in catalog from `builtin_models()`.
    /// 2. Catalog DB data (when `catalog` is `Some`).
    /// 3. User-configured models from `providers.*.models`.
    ///
    /// When `catalog` is `None`, behavior is identical to [`all_models`].
    pub fn all_models_with_catalog(
        &self,
        catalog: Option<&[model::ModelMetadata]>,
    ) -> Vec<model::ModelMetadata> {
        let mut models = model::builtin_models();

        if let Some(catalog_models) = catalog {
            for cm in catalog_models {
                if let Some(existing) = models
                    .iter_mut()
                    .find(|m| m.provider == cm.provider && m.id == cm.id)
                {
                    *existing = cm.clone();
                } else {
                    models.push(cm.clone());
                }
            }
        }

        for (provider_name, provider) in &self.providers {
            for (model_id, cfg) in &provider.models {
                if let Some(existing) = models
                    .iter_mut()
                    .find(|m| m.provider == *provider_name && m.id == *model_id)
                {
                    if cfg.context_limit.is_some() {
                        existing.context_limit = cfg.context_limit;
                    }
                    if cfg.output_limit.is_some() {
                        existing.output_limit = cfg.output_limit;
                    }
                    existing.source = model::ModelSource::Manual;
                } else {
                    models.push(model::ModelMetadata {
                        id: model_id.clone(),
                        provider: provider_name.clone(),
                        context_limit: cfg.context_limit,
                        output_limit: cfg.output_limit,
                        pricing: None,
                        capabilities: model::ModelCapabilities::default(),
                        release_date: None,
                        source: model::ModelSource::Manual,
                    });
                }
            }
        }
        models
    }

    /// Sets the active model by resolving it against all known models.
    ///
    /// Looks up `model_id` in [`Config::all_models`] to find the owning
    /// provider, then sets `self.model` and `self.provider` accordingly.
    /// Ensures the provider has an entry in `self.providers` (creates a
    /// default if missing).
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidConfig`] if the model ID is not found, or
    /// if a bare (unqualified) model ID matches multiple providers and the
    /// caller must disambiguate with `provider/model_id`.
    pub fn set_active_model(&mut self, model_id: &str) -> Result<(), ConfigError> {
        let all = self.all_models();

        // Handle provider-qualified IDs (e.g. "zhipu/glm-5.2") for models
        // that exist under multiple providers. The prefix is only treated as
        // a provider qualifier if it matches a known provider name.
        let known_providers: std::collections::HashSet<&str> =
            all.iter().map(|m| m.provider.as_str()).collect();
        let (resolved_provider, resolved_id) = match model_id.split_once('/') {
            Some((prefix, rest)) if known_providers.contains(prefix) && !rest.is_empty() => {
                (Some(prefix), rest)
            }
            _ => (None, model_id),
        };

        let meta = match resolved_provider {
            Some(provider) => all
                .iter()
                .find(|m| m.id == resolved_id && m.provider == provider)
                .ok_or_else(|| {
                    ConfigError::InvalidConfig(format!(
                        "model '{resolved_id}' not found for provider '{provider}'"
                    ))
                })?,
            None => {
                let matches: Vec<_> = all.iter().filter(|m| m.id == resolved_id).collect();
                match matches.len() {
                    0 => {
                        return Err(ConfigError::InvalidConfig(format!(
                            "model '{resolved_id}' not found"
                        )));
                    }
                    1 => matches[0],
                    _ => {
                        let providers = matches
                            .iter()
                            .map(|m| m.provider.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(ConfigError::InvalidConfig(format!(
                            "model '{resolved_id}' is available from multiple providers: {providers}. \
                             Qualify with 'provider/{resolved_id}', e.g. '{}/{resolved_id}'",
                            matches[0].provider
                        )));
                    }
                }
            }
        };

        self.model = meta.id.clone();
        self.provider = meta.provider.clone();

        if !self.providers.contains_key(&meta.provider) {
            if let Some(builtin) = builtin_provider_config(&meta.provider) {
                self.providers.insert(meta.provider.clone(), builtin);
            } else {
                self.providers.insert(
                    meta.provider.clone(),
                    ProviderConfig {
                        protocol: ProviderProtocol::OpenAIChat,
                        ..Default::default()
                    },
                );
            }
        }

        Ok(())
    }

    /// Sets the API key for the named provider.
    ///
    /// Creates a default `ProviderConfig` entry if the provider does not yet
    /// exist in `self.providers`. The key is stored in the in-memory
    /// `api_key` field for immediate runtime use.
    pub fn set_provider_credential(&mut self, name: &str, api_key: &str) {
        let entry = self.providers.entry(name.to_string()).or_insert_with(|| {
            if let Some(builtin) = builtin_provider_config(name) {
                builtin
            } else {
                ProviderConfig {
                    protocol: ProviderProtocol::OpenAIChat,
                    ..Default::default()
                }
            }
        });
        entry.api_key = Some(api_key.to_string());
    }

    /// Extracts all in-memory API keys into a [`Credentials`] store.
    /// Merges credentials into this config's provider `api_key` fields.
    ///
    /// For each provider that has a stored credential but no inline key,
    /// the credential is injected into `api_key`.
    fn merge_credentials(&mut self, creds: &Credentials) {
        for (name, key) in &creds.keys {
            if let Some(provider) = self.providers.get_mut(name) {
                if provider.api_key.is_none() {
                    provider.api_key = Some(key.clone());
                }
            } else {
                let mut provider =
                    builtin_provider_config(name).unwrap_or_else(|| ProviderConfig {
                        protocol: ProviderProtocol::OpenAIChat,
                        ..Default::default()
                    });
                provider.api_key = Some(key.clone());
                self.providers.insert(name.clone(), provider);
            }
        }
    }
}
