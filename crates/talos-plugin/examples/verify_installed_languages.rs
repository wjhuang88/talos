//! Verify installation, explicit activation and retained-context revocation with real Bundles.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use talos_core::capability::CapabilityRegistry;
use talos_plugin::{InstallError, install_bundle, lifecycle::PluginLifecycle, wasm::WasmRuntime};
use talos_text::{HighlightResult, LanguageId};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!(
                "could not clean verification directory {}: {error}",
                self.0.display()
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: verify_installed_languages RUST_BUNDLE PYTHON_BUNDLE".into());
    }
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "talos-i278-installed-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&root)?;
    let scratch = Scratch(root);
    for (language, bundle, sources) in [
        ("rust", &args[0], ["fn first() {}", "fn second() {}"]),
        (
            "python",
            &args[1],
            ["def first():\n    pass\n", "def second():\n    pass\n"],
        ),
    ] {
        let destination = scratch.0.join(language);
        let manifest = install_bundle(std::path::Path::new(bundle), &destination)?;
        if manifest.bundle.digest.is_none() {
            return Err("real bundles require a digest".into());
        }
        let installed_bytes = std::fs::read(destination.join("provider.wasm"))?;
        let broken = scratch.0.join(format!("broken-{language}"));
        std::fs::create_dir(&broken)?;
        std::fs::copy(
            std::path::Path::new(bundle).join("manifest.toml"),
            broken.join("manifest.toml"),
        )?;
        if !matches!(
            install_bundle(&broken, &destination),
            Err(InstallError::MissingArtifact(_))
        ) {
            return Err("missing real artifact was not rejected".into());
        }
        std::fs::write(broken.join("provider.wasm"), b"corrupt artifact")?;
        if !matches!(
            install_bundle(&broken, &destination),
            Err(InstallError::DigestMismatch)
        ) {
            return Err("corrupt real artifact was not rejected".into());
        }
        if std::fs::read(destination.join("provider.wasm"))? != installed_bytes {
            return Err("failed replacement changed installed artifact".into());
        }
        let mut lifecycle =
            PluginLifecycle::new(Arc::new(Mutex::new(CapabilityRegistry::default())));
        lifecycle.load(&destination)?;
        lifecycle.initialize(Arc::new(WasmRuntime::new(100_000, 250)?))?;
        if lifecycle.take_language_provider_context().is_ok() {
            return Err("activation gate bypassed".into());
        }
        lifecycle.activate()?;
        let context = lifecycle
            .take_language_provider_context()?
            .ok_or("missing provider")?;
        let id = LanguageId::parse(language).ok_or("language identity")?;
        for (source, name) in sources.into_iter().zip(["first", "second"]) {
            let result = context.highlight(&id, source);
            if !result
                .validated_spans(source)
                .is_some_and(|spans| !spans.is_empty())
            {
                return Err(format!("{language} installed highlight failed: {result:?}").into());
            }
            let symbols = context
                .with_symbols(|provider| {
                    provider.list_symbols(language, source, "caller-file", Some("function"))
                })
                .ok_or("inactive symbol context")?
                .map_err(std::io::Error::other)?;
            if symbols.len() != 1 || symbols[0].name != name || symbols[0].file != "caller-file" {
                return Err(format!("{language} source-dependent symbol/path failure").into());
            }
        }
        lifecycle.stop()?;
        if !matches!(
            context.highlight(&id, sources[0]),
            HighlightResult::PlainText
        ) || context.with_symbols(|_| ()).is_some()
        {
            return Err("retained context survived stop".into());
        }
        println!(
            "{language}: digest install, activation gate, real shared context, trusted path and stop passed"
        );
    }
    Ok(())
}
