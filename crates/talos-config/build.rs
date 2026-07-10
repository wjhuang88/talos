//! Build script for talos-config.
//!
//! Normal builds: no-op (the committed `src/models.toml` is used as-is).
//!
//! When `BUILD_MODELS=1` is set, fetches the models.dev `api.json` dataset
//! and regenerates `src/models.toml` deterministically. This is an explicit
//! developer action; normal builds never require network access.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Command;

const API_JSON_URL: &str = "https://models.dev/api.json";

fn main() {
    println!("cargo:rerun-if-changed=src/models.toml");

    generate_compiled_models();

    if std::env::var("BUILD_MODELS").is_err() {
        return;
    }

    println!("cargo:rerun-if-env-changed=BUILD_MODELS");

    match refresh_models_toml() {
        Ok(count) => {
            println!("cargo:warning=BUILD_MODELS: refreshed {count} models into src/models.toml");
            generate_compiled_models();
        }
        Err(e) => {
            println!("cargo:warning=BUILD_MODELS: failed to refresh models.toml: {e}");
            println!("cargo:warning=BUILD_MODELS: keeping existing committed models.toml");
        }
    }
}

fn refresh_models_toml() -> Result<usize, String> {
    let json = fetch_api_json()?;
    let (providers, models) = parse_api_json(&json)?;
    if models.is_empty() {
        return Err("no models with tool_call=true found in api.json".to_string());
    }
    let count = models.len();
    let toml_output = generate_toml(providers, models);
    let toml_path = std::path::Path::new("src/models.toml");
    std::fs::write(toml_path, toml_output).map_err(|e| format!("write models.toml: {e}"))?;
    Ok(count)
}

fn fetch_api_json() -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSL", API_JSON_URL])
        .output()
        .map_err(|e| format!("failed to run curl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed: {stderr}"));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("invalid UTF-8 in response: {e}"))
}

#[derive(Deserialize)]
struct ApiProvider {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    api: Option<String>,
    #[serde(default)]
    npm: Option<String>,
    #[serde(default, deserialize_with = "deser_env")]
    env: Option<String>,
    #[serde(default)]
    doc: Option<String>,
    #[serde(default)]
    models: BTreeMap<String, ApiModel>,
}

fn deser_env<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EnvField {
        Single(String),
        Array(Vec<String>),
    }

    match Option::<EnvField>::deserialize(d)? {
        Some(EnvField::Single(s)) => Ok(Some(s)),
        Some(EnvField::Array(arr)) => Ok(arr.into_iter().next()),
        None => Ok(None),
    }
}

#[derive(Deserialize)]
struct ApiModel {
    #[serde(default)]
    limit: Option<ApiLimit>,
    #[serde(default)]
    cost: Option<ApiCost>,
    #[serde(default)]
    tool_call: Option<bool>,
    #[serde(default)]
    reasoning: Option<bool>,
    #[serde(default)]
    structured_output: Option<bool>,
    #[serde(default)]
    attachment: Option<bool>,
    #[serde(default)]
    release_date: Option<String>,
}

#[derive(Deserialize)]
struct ApiLimit {
    #[serde(default)]
    context: Option<u64>,
    #[serde(default)]
    output: Option<u64>,
}

#[derive(Deserialize)]
struct ApiCost {
    #[serde(default)]
    input: Option<f64>,
    #[serde(default)]
    output: Option<f64>,
    #[serde(default)]
    cache_read: Option<f64>,
}
fn parse_api_json(json: &str) -> Result<(Vec<TomlProvider>, Vec<TomlModel>), String> {
    let root: BTreeMap<String, ApiProvider> =
        serde_json::from_str(json).map_err(|e| format!("parse api.json: {e}"))?;

    let mut providers = Vec::new();
    let mut models = Vec::new();

    for (provider_id, provider) in &root {
        // Provider-level metadata from api.json
        let name = provider.name.clone().unwrap_or_else(|| provider_id.clone());
        let api_base_url = provider.api.clone();
        let protocol = infer_provider_protocol(provider.npm.as_deref(), api_base_url.as_deref());
        let env_var = provider.env.clone();
        let doc_url = provider.doc.clone();

        providers.push(TomlProvider {
            id: provider_id.clone(),
            name,
            api_base_url,
            protocol,
            env_var,
            doc_url,
        });

        for (model_id, model) in &provider.models {
            let tool_call = model.tool_call.unwrap_or(false);
            if !tool_call {
                continue;
            }

            let capabilities = TomlCapabilities {
                tools: true,
                structured_output: model.structured_output.unwrap_or(false),
                reasoning: model.reasoning.unwrap_or(false),
                image_input: model.attachment.unwrap_or(false),
            };

            let pricing = model.cost.as_ref().and_then(|c| {
                if c.input.is_some() || c.output.is_some() || c.cache_read.is_some() {
                    Some(TomlPricing {
                        input_per_1m: c.input,
                        output_per_1m: c.output,
                        cache_read_per_1m: c.cache_read,
                    })
                } else {
                    None
                }
            });

            let context_limit = model
                .limit
                .as_ref()
                .and_then(|l| l.context)
                .map(|v| v as u32);
            let output_limit = model
                .limit
                .as_ref()
                .and_then(|l| l.output)
                .map(|v| v as u32);

            models.push(TomlModel {
                id: model_id.clone(),
                provider: provider_id.clone(),
                context_limit,
                output_limit,
                release_date: model.release_date.clone(),
                pricing,
                capabilities,
            });
        }
    }

    providers.sort_by(|a, b| a.id.cmp(&b.id));
    models.sort_by(|a, b| a.provider.cmp(&b.provider).then_with(|| a.id.cmp(&b.id)));

    Ok((providers, models))
}

#[derive(Serialize)]
struct TomlDataset {
    providers: Vec<TomlProvider>,
    models: Vec<TomlModel>,
}

#[derive(Serialize)]
struct TomlProvider {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol: Option<TomlProviderProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env_var: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_url: Option<String>,
}

#[derive(Clone, Serialize)]
enum TomlProviderProtocol {
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
    #[serde(rename = "openai-chat")]
    OpenAIChat,
}

fn infer_provider_protocol(
    npm_package: Option<&str>,
    api_base_url: Option<&str>,
) -> Option<TomlProviderProtocol> {
    let package = npm_package.unwrap_or_default().to_ascii_lowercase();
    if package.contains("anthropic") {
        return Some(TomlProviderProtocol::AnthropicMessages);
    }

    let url = api_base_url.unwrap_or_default().to_ascii_lowercase();
    if url.contains("/anthropic/") {
        return Some(TomlProviderProtocol::AnthropicMessages);
    }

    if npm_package.is_some() || api_base_url.is_some() {
        return Some(TomlProviderProtocol::OpenAIChat);
    }

    None
}

#[derive(Serialize)]
struct TomlModel {
    id: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pricing: Option<TomlPricing>,
    capabilities: TomlCapabilities,
}

#[derive(Serialize)]
struct TomlPricing {
    #[serde(skip_serializing_if = "Option::is_none")]
    input_per_1m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_per_1m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_read_per_1m: Option<f64>,
}

#[derive(Serialize)]
struct TomlCapabilities {
    #[serde(skip_serializing_if = "is_false")]
    tools: bool,
    #[serde(skip_serializing_if = "is_false")]
    structured_output: bool,
    #[serde(skip_serializing_if = "is_false")]
    reasoning: bool,
    #[serde(skip_serializing_if = "is_false")]
    image_input: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

fn generate_toml(providers: Vec<TomlProvider>, models: Vec<TomlModel>) -> String {
    let dataset = TomlDataset { providers, models };
    let body = toml::to_string_pretty(&dataset).unwrap_or_default();

    let mut output = String::new();
    output.push_str("# Built-in model dataset for Talos.\n");
    output.push_str("# Regenerated from models.dev via BUILD_MODELS=1.\n");
    output.push_str("# Only models with tool calling support are included.\n\n");
    output.push_str(&body);
    output
}

fn generate_compiled_models() {
    let toml_str = std::fs::read_to_string("src/models.toml").expect("models.toml must exist");
    let root: toml::Table = toml_str.parse().expect("models.toml must be valid TOML");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be set");
    let dest = std::path::Path::new(&out_dir).join("models_data.rs");

    let code = generate_models_rust(&root);
    std::fs::write(&dest, code).expect("failed to write generated models_data.rs");
}

fn generate_models_rust(root: &toml::Table) -> String {
    let mut s = String::new();
    s.push_str("// Auto-generated by build.rs from src/models.toml.\n");
    s.push_str("// Do not edit manually.\n\n");

    s.push_str("pub(crate) fn generated_providers() -> Vec<TomlProviderEntry> {\n");
    s.push_str("    vec![\n");
    if let Some(toml::Value::Array(providers)) = root.get("providers") {
        for p in providers {
            if let Some(t) = p.as_table() {
                let id = str_field(t, "id");
                let name = str_field(t, "name");
                let api_base_url = opt_str_field(t, "api_base_url");
                let env_var = opt_str_field(t, "env_var");
                let doc_url = opt_str_field(t, "doc_url");
                let protocol = if let Some(p) = t.get("protocol").and_then(|v| v.as_str()) {
                    match p {
                        "anthropic-messages" => "Some(CatalogProviderProtocol::AnthropicMessages)",
                        "openai-chat" => "Some(CatalogProviderProtocol::OpenAIChat)",
                        _ => "None",
                    }
                } else {
                    "None"
                };
                s.push_str(&format!(
                    "        TomlProviderEntry {{ id: {id}.into(), name: {name}.into(), api_base_url: {api_base_url}, protocol: {protocol}, env_var: {env_var}, doc_url: {doc_url} }},\n"
                ));
            }
        }
    }
    s.push_str("    ]\n");
    s.push_str("}\n\n");

    s.push_str("pub(crate) fn generated_models() -> Vec<talos_core::model::ModelMetadata> {\n");
    s.push_str("    vec![\n");
    if let Some(toml::Value::Array(models)) = root.get("models") {
        for m in models {
            if let Some(t) = m.as_table() {
                let id = str_field(t, "id");
                let provider = str_field(t, "provider");
                let context_limit = opt_num_field(t, "context_limit");
                let output_limit = opt_num_field(t, "output_limit");
                let release_date = opt_str_field(t, "release_date");
                let caps = t.get("capabilities").and_then(|v| v.as_table());
                let tools = caps
                    .and_then(|c| c.get("tools"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let so = caps
                    .and_then(|c| c.get("structured_output"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reasoning = caps
                    .and_then(|c| c.get("reasoning"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let image = caps
                    .and_then(|c| c.get("image_input"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let pricing = if let Some(p) = t.get("pricing").and_then(|v| v.as_table()) {
                    let inp = p.get("input_per_1m").and_then(|v| v.as_float());
                    let outp = p.get("output_per_1m").and_then(|v| v.as_float());
                    let cache = p.get("cache_read_per_1m").and_then(|v| v.as_float());
                    if inp.is_some() || outp.is_some() || cache.is_some() {
                        format!(
                            "Some(talos_core::model::ModelPricing {{ input_per_1m: {}, output_per_1m: {}, cache_read_per_1m: {} }})",
                            opt_f64(inp),
                            opt_f64(outp),
                            opt_f64(cache)
                        )
                    } else {
                        "None".to_string()
                    }
                } else {
                    "None".to_string()
                };
                s.push_str(&format!(
                    "        talos_core::model::ModelMetadata {{ id: {id}.into(), provider: {provider}.into(), context_limit: {context_limit}, output_limit: {output_limit}, pricing: {pricing}, capabilities: talos_core::model::ModelCapabilities {{ tools: {tools}, structured_output: {so}, reasoning: {reasoning}, image_input: {image} }}, release_date: {release_date}, source: talos_core::model::ModelSource::Builtin }},\n"
                ));
            }
        }
    }
    s.push_str("    ]\n");
    s.push_str("}\n");

    s
}

fn str_field(t: &toml::Table, key: &str) -> String {
    t.get(key)
        .and_then(|v| v.as_str())
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "\"\"".to_string())
}

fn opt_str_field(t: &toml::Table, key: &str) -> String {
    t.get(key)
        .and_then(|v| v.as_str())
        .map(|s| format!("Some({:?}.into())", s))
        .unwrap_or_else(|| "None".to_string())
}

fn opt_num_field(t: &toml::Table, key: &str) -> String {
    t.get(key)
        .and_then(|v| v.as_integer())
        .map(|v| format!("Some({})", v))
        .unwrap_or_else(|| "None".to_string())
}

fn opt_f64(v: Option<f64>) -> String {
    v.map(|f| format!("Some({}_f64)", f))
        .unwrap_or_else(|| "None".to_string())
}
