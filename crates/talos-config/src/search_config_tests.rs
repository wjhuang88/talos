use crate::{Config, SearchBackendConfig};

#[test]
fn missing_search_section_defaults_to_zero_key_auto() {
    let config: Config = toml::from_str(
        r#"
provider = "openai"
model = "gpt-test"
"#,
    )
    .expect("config parses");

    assert_eq!(config.search.backend, SearchBackendConfig::Auto);
    assert!(!config.search.fallback_to_auto);
    assert!(config.search.url.is_none());
}

#[test]
fn explicit_tavily_persists_intent_and_fallback_without_a_search_secret() {
    let config: Config = toml::from_str(
        r#"
provider = "openai"
model = "gpt-test"

[search]
backend = "tavily"
fallback_to_auto = true
"#,
    )
    .expect("config parses");

    assert_eq!(config.search.backend, SearchBackendConfig::Tavily);
    assert!(config.search.fallback_to_auto);
    assert!(config.search.url.is_none());
    let serialized = toml::to_string(&config).expect("config serializes");
    assert!(serialized.contains("backend = \"tavily\""));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("TAVILY_API_KEY"));
}

#[test]
fn explicit_searxng_round_trip_keeps_endpoint_but_debug_redacts_it() {
    let endpoint = "https://private-search.example/prefix";
    let config: Config = toml::from_str(&format!(
        r#"
provider = "openai"
model = "gpt-test"

[search]
backend = "searxng"
fallback_to_auto = true
url = "{endpoint}"
"#
    ))
    .expect("config parses");

    assert_eq!(config.search.backend, SearchBackendConfig::SearXng);
    assert_eq!(config.search.url.as_deref(), Some(endpoint));
    assert!(!format!("{:?}", config.search).contains(endpoint));
    let serialized = toml::to_string(&config).expect("config serializes");
    assert!(serialized.contains(endpoint));
}
