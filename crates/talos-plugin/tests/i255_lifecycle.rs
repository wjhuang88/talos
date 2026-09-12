//! Public lifecycle conformance: no availability before activation or after stop.
#![cfg(feature = "wasm")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use talos_core::{CapabilityRegistry, CapabilityRequest, ResolutionResult};
use talos_plugin::lifecycle::{LifecycleError, PluginLifecycle, PluginState};
use talos_plugin::wasm::WasmRuntime;

fn runtime() -> Arc<WasmRuntime> {
    Arc::new(WasmRuntime::new(100_000, 250).expect("runtime"))
}
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/capability-demo")
}
fn resolve(registry: &Mutex<CapabilityRegistry>) -> ResolutionResult {
    registry
        .lock()
        .expect("registry")
        .resolve(&CapabilityRequest {
            capability_id: "example.answer.value".into(),
            version: "1.0.0".into(),
        })
}

#[tokio::test]
async fn activate_execute_stop_rejects_stale_handles_and_allows_new_owner() {
    let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
    let mut plugin = PluginLifecycle::new(registry.clone());
    assert_eq!(plugin.state(), PluginState::Unloaded);
    plugin.load(&fixture()).expect("load");
    assert_eq!(plugin.state(), PluginState::Loaded);
    assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    plugin.initialize(runtime()).expect("initialize");
    let stale = plugin.tools().remove(0);
    assert!(
        !talos_core::tool::ToolPresentationPolicy::runtime_default().allows_tool(stale.as_ref())
    );
    assert!(
        talos_core::tool::ToolPresentationPolicy::with_tool(stale.name())
            .allows_tool(stale.as_ref())
    );
    assert!(stale.execute(serde_json::json!({})).await.is_error);
    assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    plugin.activate().expect("activate");
    assert!(matches!(resolve(&registry), ResolutionResult::Available(_)));
    let result = stale.execute(serde_json::json!({})).await;
    assert!(!result.is_error);
    assert!(result.content.contains("returned 7"));
    let mut duplicate = PluginLifecycle::new(registry.clone());
    duplicate.load(&fixture()).expect("load duplicate");
    duplicate
        .initialize(runtime())
        .expect("initialize duplicate");
    assert!(duplicate.activate().is_err());
    duplicate.stop().expect("stop failed activation");
    assert!(matches!(resolve(&registry), ResolutionResult::Available(_)));
    plugin.stop().expect("stop");
    plugin.stop().expect("repeat stop");
    assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    assert!(stale.execute(serde_json::json!({})).await.is_error);
    let mut replacement = PluginLifecycle::new(registry.clone());
    replacement.load(&fixture()).expect("replacement load");
    replacement
        .initialize(runtime())
        .expect("replacement initialize");
    replacement.activate().expect("replacement activate");
    plugin.stop().expect("old stop cannot remove replacement");
    assert!(matches!(resolve(&registry), ResolutionResult::Available(_)));
    drop(replacement);
    assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
}

struct Package(PathBuf);
impl Package {
    fn new(manifest: &str, module: &str) -> Self {
        static SEQUENCE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "talos-i255-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("unique temp package");
        std::fs::write(path.join("talos-plugin.toml"), manifest).expect("manifest");
        std::fs::write(path.join("answer.wat"), module).expect("module");
        Self(path)
    }
}
impl Drop for Package {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove fixture");
    }
}
const MANIFEST: &str = include_str!("fixtures/capability-demo/talos-plugin.toml");
const MODULE: &str = include_str!("fixtures/capability-demo/answer.wat");

#[test]
fn legacy_file_loader_never_ignores_explicit_declarations() {
    for text in [
        MANIFEST.to_owned(),
        MANIFEST.replace("schema_version = 1", "schema_version = 9"),
        format!(
            "capability_provider = false\n{}",
            MANIFEST
                .split("[capability_provider]")
                .next()
                .expect("legacy part")
        ),
    ] {
        let package = Package::new(&text, "not a valid module");
        let result = talos_plugin::wasm::load_read_only_wasm_package(runtime(), &package.0);
        assert!(
            matches!(result, Err(talos_plugin::wasm::WasmError::Manifest(message)) if message.contains("PluginLifecycle"))
        );
    }
}

#[test]
fn partial_initialization_and_path_escape_never_publish_tools() {
    for text in [
        format!("{MANIFEST}\n[[tools]]\nname = \"broken\"\nhandler = \"missing.wat\"\n"),
        MANIFEST.replace("handler = \"answer.wat\"", "handler = \"../answer.wat\""),
    ] {
        let package = Package::new(&text, MODULE);
        let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
        let mut plugin = PluginLifecycle::new(registry.clone());
        plugin.load(&package.0).expect("manifest");
        assert!(plugin.initialize(runtime()).is_err());
        assert_eq!(plugin.state(), PluginState::Loaded);
        assert!(plugin.tools().is_empty());
        assert!(plugin.package().is_none());
        assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    }
}

#[test]
fn malformed_explicit_declarations_and_unsupported_carriers_fail_closed() {
    for text in [
        MANIFEST.replace("schema_version = 1", "schema_version = 2"),
        MANIFEST.replace("tool = \"answer\"", "tool = \"missing\""),
        MANIFEST.replace("version = \"1.0.0\"", "version = \"invalid\""),
        MANIFEST.replace("schema_version = 1", "schema_version = 1\nunknown = true"),
    ] {
        let package = Package::new(&text, MODULE);
        let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
        let mut plugin = PluginLifecycle::new(registry.clone());
        assert!(plugin.load(&package.0).is_err());
        assert_eq!(plugin.state(), PluginState::Unloaded);
        assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    }
    for carrier in ["builtin", "mcp", "helper", "remote"] {
        let package = Package::new(
            &MANIFEST.replace("carrier = \"wasm\"", &format!("carrier = \"{carrier}\"")),
            MODULE,
        );
        let mut plugin = PluginLifecycle::new(Arc::new(Mutex::new(CapabilityRegistry::default())));
        assert!(matches!(
            plugin.load(&package.0),
            Err(LifecycleError::Unavailable(_))
        ));
    }
}

#[tokio::test]
async fn initialization_checks_bindings_without_executing_start_and_trap_withdraws() {
    for module in [
        "(module)",
        "(module (func (export \"run\") (result i64) i64.const 1))",
        "(module (import \"host\" \"call\" (func)) (func (export \"run\") (result i32) i32.const 1))",
    ] {
        let package = Package::new(MANIFEST, module);
        let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
        let mut plugin = PluginLifecycle::new(registry.clone());
        plugin.load(&package.0).expect("parse only");
        assert!(plugin.initialize(runtime()).is_err());
        assert!(plugin.activate().is_err());
        assert!(plugin.tools().is_empty());
        assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
    }
    for module in [
        "(module (func $start unreachable) (start $start) (func (export \"run\") (result i32) i32.const 1))",
        "(module (func (export \"run\") (result i32) (loop $forever br $forever) i32.const 1))",
    ] {
        let package = Package::new(MANIFEST, module);
        let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
        let mut plugin = PluginLifecycle::new(registry.clone());
        plugin.load(&package.0).expect("parse");
        plugin.initialize(runtime()).expect("must not run start");
        plugin.activate().expect("activate");
        let tool = plugin.tools().remove(0);
        assert!(tool.execute(serde_json::json!({})).await.is_error);
        assert_eq!(plugin.state(), PluginState::Stopped);
        assert_eq!(resolve(&registry), ResolutionResult::Unavailable);
        assert!(tool.execute(serde_json::json!({})).await.is_error);
    }
}

#[tokio::test]
async fn legacy_non_semver_and_names_preserved_without_public_struct_changes() {
    let text = MANIFEST
        .split("[capability_provider]")
        .next()
        .expect("legacy part")
        .replace("capability-demo", "old plugin!")
        .replace("0.1.0", "development");
    let package = Package::new(&text, MODULE);
    let registry = Arc::new(Mutex::new(CapabilityRegistry::default()));
    let mut plugin = PluginLifecycle::new(registry);
    plugin.load(&package.0).expect("legacy spelling");
    plugin.initialize(runtime()).expect("legacy binding");
    plugin.activate().expect("safe internal identity");
    let tool = plugin.tools().remove(0);
    assert_eq!(tool.name(), "old plugin!.answer");
    assert!(!tool.execute(serde_json::json!({})).await.is_error);
    let legacy = talos_plugin::PluginManifest {
        plugin: talos_plugin::PluginMetadata {
            name: "old".into(),
            version: "dev".into(),
            carrier: "wasm".into(),
            artifact: "answer.wat".into(),
            description: None,
            talos_protocol: None,
        },
        tools: vec![],
        hooks: vec![],
        skills: vec![],
        language_provider: None,
    };
    assert!(legacy.validate().is_ok());
}
