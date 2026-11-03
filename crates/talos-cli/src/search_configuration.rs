use anyhow::{Result, bail};
use talos_config::{Config, SearchBackendConfig};
use talos_tools::SearchToolConfiguration;

/// Project persisted user intent into the product-neutral protected-search configuration.
///
/// Optional provider material is consulted only after the corresponding backend was explicitly
/// selected. Ambient `TAVILY_API_KEY` / `SEARXNG_URL` are deliberately not read here.
pub(crate) fn project_search_configuration(config: &Config) -> Result<SearchToolConfiguration> {
    match config.search.backend {
        SearchBackendConfig::Auto => {
            reject_inapplicable_search_options(config)?;
            Ok(SearchToolConfiguration::Auto)
        }
        SearchBackendConfig::DuckDuckGo => {
            reject_inapplicable_search_options(config)?;
            Ok(SearchToolConfiguration::DuckDuckGo)
        }
        SearchBackendConfig::Tavily => {
            if config.search.url.is_some() {
                bail!("search.url is only valid when search.backend = searxng");
            }
            let api_key = config
                .providers
                .get("tavily")
                .and_then(|provider| provider.api_key.as_deref())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!(
                    "explicit Tavily search requires a configured tavily credential"
                ))?;
            Ok(SearchToolConfiguration::Tavily {
                api_key: api_key.to_owned(),
                fallback_to_auto: config.search.fallback_to_auto,
            })
        }
        SearchBackendConfig::SearXng => {
            let url = config
                .search
                .url
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!(
                    "explicit SearXNG search requires search.url"
                ))?;
            Ok(SearchToolConfiguration::SearXng {
                url: url.to_owned(),
                fallback_to_auto: config.search.fallback_to_auto,
            })
        }
    }
}

fn reject_inapplicable_search_options(config: &Config) -> Result<()> {
    if config.search.fallback_to_auto {
        bail!("search.fallback_to_auto is only valid for Tavily or SearXNG");
    }
    if config.search.url.is_some() {
        bail!("search.url is only valid when search.backend = searxng");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_config::ProviderConfig;

    #[test]
    fn auto_ignores_provider_availability_but_rejects_inapplicable_options() {
        let mut config = Config::default();
        config.providers.insert(
            "tavily".into(),
            ProviderConfig {
                api_key: Some("ambient-like-but-explicit-store-value".into()),
                ..ProviderConfig::default()
            },
        );
        assert!(matches!(
            project_search_configuration(&config).expect("auto"),
            SearchToolConfiguration::Auto
        ));

        config.search.fallback_to_auto = true;
        assert!(project_search_configuration(&config).is_err());
    }

    #[test]
    fn explicit_tavily_requires_effective_stored_credential_after_selection() {
        let mut config = Config::default();
        config.search.backend = SearchBackendConfig::Tavily;
        assert!(project_search_configuration(&config).is_err());

        config.providers.insert(
            "tavily".into(),
            ProviderConfig {
                api_key: Some("fixture-secret".into()),
                ..ProviderConfig::default()
            },
        );
        config.search.fallback_to_auto = true;
        let projected = project_search_configuration(&config).expect("tavily");
        assert!(matches!(
            projected,
            SearchToolConfiguration::Tavily {
                fallback_to_auto: true,
                ..
            }
        ));
        assert!(!format!("{projected:?}").contains("fixture-secret"));
    }

    #[test]
    fn explicit_searxng_requires_url_and_preserves_fallback_intent() {
        let mut config = Config::default();
        config.search.backend = SearchBackendConfig::SearXng;
        assert!(project_search_configuration(&config).is_err());

        let endpoint = "https://search.example/prefix/";
        config.search.url = Some(endpoint.into());
        config.search.fallback_to_auto = true;
        let projected = project_search_configuration(&config).expect("searxng");
        assert!(matches!(
            projected,
            SearchToolConfiguration::SearXng {
                fallback_to_auto: true,
                ..
            }
        ));
        assert!(!format!("{projected:?}").contains(endpoint));
    }

    #[test]
    fn duckduckgo_is_explicit_zero_key_and_does_not_consume_tavily_material() {
        let mut config = Config::default();
        config.search.backend = SearchBackendConfig::DuckDuckGo;
        config.providers.insert(
            "tavily".into(),
            ProviderConfig {
                api_key: Some("must-not-activate".into()),
                ..ProviderConfig::default()
            },
        );
        assert!(matches!(
            project_search_configuration(&config).expect("duckduckgo"),
            SearchToolConfiguration::DuckDuckGo
        ));
    }
}
