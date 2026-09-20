//! Run real, independently built guest artifacts through the bounded host ABI.

use std::sync::Arc;
use talos_plugin::wasm::{WasmLanguageProvider, WasmModule, WasmRuntime};
use talos_text::wasm_provider::WasmProviderLimits;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                tracing_subscriber::filter::Targets::new()
                    .with_target("talos_plugin::wasm", tracing::Level::DEBUG),
            ),
        )
        .init();
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&paths.len()) {
        return Err("usage: verify_language_guests RUST.wasm PYTHON.wasm [FUEL]".into());
    }
    for (language, path, sources) in [
        (
            "rust",
            &paths[0],
            ["fn greet() {}", "// fn fake() {}\nfn 中文() {}"],
        ),
        (
            "python",
            &paths[1],
            [
                "def greet():\n    pass\n",
                "# def fake():\ndef 中文():\n    pass\n",
            ],
        ),
    ] {
        let mut limits = WasmProviderLimits::default();
        if let Some(fuel) = paths.get(2) {
            limits.fuel = fuel.parse()?;
        }
        let runtime = Arc::new(WasmRuntime::new(limits.fuel, 500)?);
        let module = WasmModule::from_bytes(runtime, &std::fs::read(path)?)?;
        let provider = WasmLanguageProvider::new(limits);
        provider.validate_module(&module)?;
        let mut cases: Vec<_> = sources
            .into_iter()
            .zip(["greet", "中文"])
            .map(|(source, name)| (source.to_owned(), name, 1))
            .collect();
        cases.push((sources[0].repeat(1000), "greet", 1000));
        let comment = if language == "rust" {
            "// comment\n"
        } else {
            "# comment\n"
        };
        let mut near_limit = comment.repeat((256 * 1024 - sources[0].len()) / comment.len());
        near_limit.push_str(sources[0]);
        cases.push((near_limit, "greet", 1));
        for (source, name, expected_count) in cases {
            let highlight =
                serde_json::to_vec(&serde_json::json!({"language":language,"source":source}))?;
            let started = std::time::Instant::now();
            let result: serde_json::Value =
                serde_json::from_slice(&provider.execute_payload(&module, &highlight)?)?;
            println!(
                "{language} highlight: {} bytes, {:?}, fuel ceiling {}",
                source.len(),
                started.elapsed(),
                limits.fuel
            );
            let bounded_output_fallback =
                source.len() > 250_000 && result["Unavailable"] == "response exceeds guest limit";
            if !bounded_output_fallback
                && !result["Spans"]
                    .as_array()
                    .is_some_and(|spans| !spans.is_empty())
            {
                return Err(format!("{language} highlight failed: {result}").into());
            }
            if bounded_output_fallback {
                println!("{language}: near-limit highlight returned bounded output fallback");
            }
            let request = serde_json::to_vec(&serde_json::json!({
                "language":language,"source":source,"abi_version":1,
                "operation":{"operation":"list_symbols","kind":"function"}
            }))?;
            let result: serde_json::Value =
                serde_json::from_slice(&provider.execute_payload(&module, &request)?)?;
            let symbols = result["Result"]
                .as_array()
                .ok_or_else(|| format!("{language} symbol failure: {result}"))?;
            if symbols.len() != expected_count || symbols[0]["name"] != name {
                return Err(format!("{language} incorrect symbols: {result}").into());
            }
            for operation in ["find_symbol", "find_references", "list_imports"] {
                let request = serde_json::to_vec(&serde_json::json!({
                    "language":language,"source":source,"abi_version":1,
                    "operation":{"operation":operation,"name":name}
                }))?;
                let response = match provider.execute_payload(&module, &request) {
                    Ok(response) => response,
                    Err(talos_plugin::wasm::WasmError::Trap(reason))
                        if source.len() > 250_000 && reason.contains("fuel exhausted") =>
                    {
                        println!(
                            "{language} {operation}: near-limit input hit fuel ceiling safely"
                        );
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                };
                let result: serde_json::Value = serde_json::from_slice(&response)?;
                if result.get("Result").is_none() || result["Result"].is_null() {
                    return Err(format!("{language} {operation} failure: {result}").into());
                }
            }
        }
        println!(
            "{language}: real WASM ABI admission, highlights and Unicode/comment symbol checks passed"
        );
    }
    Ok(())
}
