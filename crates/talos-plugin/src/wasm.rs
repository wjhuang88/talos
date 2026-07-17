//! WASM plugin runtime adapter (T46, ADR-032).
//!
//! Loads and executes a WASM module with deterministic resource limits (fuel)
//! and a wall-clock timeout guard (epoch interruption). No host calls are
//! provided — the module runs in full sandbox isolation. All failures degrade
//! to recoverable errors; none may panic the host process (Hard Constraint #9).

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolProvenance, ToolRegistry,
    ToolResourceKind, ToolResult,
};
use thiserror::Error;

use crate::manifest::parse_manifest;
use crate::{PluginManifest, PluginTool};

const MAX_PLUGIN_TOOL_OUTPUT: usize = 2_000;
/// Manifest filename for an explicitly selected local plugin package.
pub const PLUGIN_MANIFEST_FILE: &str = "talos-plugin.toml";

/// Typed runtime state for one successfully loaded plugin package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPluginPackage {
    /// Stable manifest package name.
    pub name: String,
    /// Manifest package version.
    pub version: String,
    /// Runtime carrier, currently `wasm`.
    pub carrier: String,
    /// Namespaced tool capabilities registered by the package.
    pub capabilities: Vec<String>,
}

#[derive(Debug, Error)]
pub enum WasmError {
    /// The conventional package manifest could not be read or validated.
    #[error("plugin manifest failed: {0}")]
    Manifest(String),
    #[error("module compilation failed: {0}")]
    Compile(String),
    #[error("module instantiation failed: {0}")]
    Instantiate(String),
    #[error("exported function 'run' not found or has wrong signature")]
    MissingExport,
    #[error("execution trapped: {0}")]
    Trap(String),
    #[error("execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("plugin path escapes package root: {0}")]
    PathEscape(String),
    #[error("plugin artifact I/O failed: {0}")]
    Io(String),
    #[error("plugin tool name collides with registered tool: {0}")]
    ToolCollision(String),
}

pub struct WasmRuntime {
    engine: Arc<wasmtime::Engine>,
    fuel: u64,
    timeout: Duration,
}

impl WasmRuntime {
    pub fn new(fuel: u64, timeout_ms: u64) -> Result<Self, WasmError> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine =
            wasmtime::Engine::new(&config).map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self {
            engine: Arc::new(engine),
            fuel,
            timeout: Duration::from_millis(timeout_ms),
        })
    }

    pub fn engine(&self) -> &wasmtime::Engine {
        &self.engine
    }
}

pub struct WasmModule {
    module: wasmtime::Module,
    runtime: Arc<WasmRuntime>,
}

impl WasmModule {
    pub fn from_wat(runtime: Arc<WasmRuntime>, wat: &str) -> Result<Self, WasmError> {
        let module = wasmtime::Module::new(&runtime.engine, wat)
            .map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self { module, runtime })
    }

    pub fn from_bytes(runtime: Arc<WasmRuntime>, bytes: &[u8]) -> Result<Self, WasmError> {
        let module = wasmtime::Module::from_binary(&runtime.engine, bytes)
            .map_err(|e| WasmError::Compile(e.to_string()))?;
        Ok(Self { module, runtime })
    }

    pub fn execute(&self) -> Result<i32, WasmError> {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.execute_inner()));
        match result {
            Ok(inner) => inner,
            Err(_) => Err(WasmError::Trap("host panic during execution".into())),
        }
    }

    fn execute_inner(&self) -> Result<i32, WasmError> {
        let engine = self.runtime.engine.clone();
        let mut store = wasmtime::Store::new(&engine, ());
        store
            .set_fuel(self.runtime.fuel)
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;

        store.epoch_deadline_trap();
        store.set_epoch_deadline(1);

        let (completion_tx, completion_rx) = mpsc::channel();
        let engine_for_timeout = engine.clone();
        let timeout = self.runtime.timeout;
        thread::spawn(move || {
            thread::sleep(timeout);
            if completion_tx.send(()).is_ok() {
                engine_for_timeout.increment_epoch();
            }
        });

        let instance = wasmtime::Instance::new(&mut store, &self.module, &[])
            .map_err(|e| WasmError::Instantiate(e.to_string()))?;

        let func = instance
            .get_typed_func::<(), i32>(&mut store, "run")
            .map_err(|_| WasmError::MissingExport)?;

        let result = func.call(&mut store, ());
        drop(completion_rx);

        match result {
            Ok(val) => Ok(val),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("epoch") || msg.contains("interrupt") || msg.contains("fuel") {
                    Err(WasmError::Timeout {
                        timeout_ms: self.runtime.timeout.as_millis() as u64,
                    })
                } else {
                    Err(WasmError::Trap(msg))
                }
            }
        }
    }
}

/// Read-only `AgentTool` adapter for one explicitly loaded local WASM plugin tool.
///
/// This adapter is intentionally narrow: it loads a handler from a package-confined path, exposes
/// no host calls, reports plugin provenance, and stays out of runtime-default model presentation.
pub struct WasmPluginTool {
    name: String,
    description: String,
    module: WasmModule,
    provenance: ToolProvenance,
    package_root: String,
}

impl WasmPluginTool {
    /// Builds a read-only plugin tool from one validated manifest tool entry.
    pub fn from_manifest_tool(
        runtime: Arc<WasmRuntime>,
        package_root: &Path,
        manifest: &PluginManifest,
        tool: &PluginTool,
    ) -> Result<Self, WasmError> {
        let _plugin_artifact = confined_package_path(package_root, &manifest.plugin.artifact)?;
        let handler = confined_package_path(package_root, &tool.handler)?;
        let bytes = std::fs::read(&handler).map_err(|e| WasmError::Io(e.to_string()))?;
        let module = if handler
            .extension()
            .is_some_and(|extension| extension == "wat")
        {
            let text = std::str::from_utf8(&bytes)
                .map_err(|error| WasmError::Compile(error.to_string()))?;
            WasmModule::from_wat(runtime, text)?
        } else {
            WasmModule::from_bytes(runtime, &bytes)?
        };
        let name = plugin_tool_name(&manifest.plugin.name, &tool.name);
        Ok(Self {
            description: format!(
                "Run read-only WASM plugin tool '{}' from plugin '{}'",
                tool.name, manifest.plugin.name
            ),
            name,
            module,
            provenance: ToolProvenance::Plugin {
                name: manifest.plugin.name.clone(),
                version: manifest.plugin.version.clone(),
                carrier: manifest.plugin.carrier.clone(),
            },
            package_root: package_root.display().to_string(),
        })
    }
}

#[async_trait]
impl AgentTool for WasmPluginTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        match self.module.execute() {
            Ok(value) => {
                let output = format!("plugin tool '{}' returned {value}", self.name);
                ToolResult::success(bound_plugin_output(&output))
            }
            Err(e) => ToolResult::error(bound_plugin_output(&e.to_string())),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Plugin
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Read,
                self.package_root.clone(),
                ToolResourceKind::Path,
            )
            .with_description("read-only local plugin package"),
        ]
    }

    fn provenance(&self) -> ToolProvenance {
        self.provenance.clone()
    }
}

/// Registers all read-only WASM tools declared by a local explicit plugin manifest.
///
/// Tool names are namespaced as `{plugin}.{tool}`. Existing registry entries with the same name
/// are rejected before any new tool is registered.
pub fn register_read_only_wasm_tools(
    registry: &mut ToolRegistry,
    runtime: Arc<WasmRuntime>,
    package_root: &Path,
    manifest: &PluginManifest,
) -> Result<usize, WasmError> {
    let (tools, _) = build_read_only_wasm_tools(runtime, package_root, manifest)?;
    let mut registered = 0;
    for plugin_tool in tools {
        let name = plugin_tool.name().to_string();
        if registry.get(&name).is_some() {
            return Err(WasmError::ToolCollision(name));
        }
        registry.register(plugin_tool);
        registered += 1;
    }
    Ok(registered)
}

/// Builds the tools and typed package state for one validated local manifest.
///
/// The returned tools are not registered automatically, allowing a host to
/// wrap them in its normal permission adapter before exposure.
pub fn build_read_only_wasm_tools(
    runtime: Arc<WasmRuntime>,
    package_root: &Path,
    manifest: &PluginManifest,
) -> Result<(Vec<Arc<dyn AgentTool>>, LoadedPluginPackage), WasmError> {
    let mut tools: Vec<Arc<dyn AgentTool>> = Vec::with_capacity(manifest.tools.len());
    let mut capabilities = Vec::with_capacity(manifest.tools.len());
    for tool in &manifest.tools {
        let plugin_tool =
            WasmPluginTool::from_manifest_tool(runtime.clone(), package_root, manifest, tool)?;
        capabilities.push(plugin_tool.name().to_string());
        tools.push(Arc::new(plugin_tool));
    }
    Ok((
        tools,
        LoadedPluginPackage {
            name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            carrier: manifest.plugin.carrier.clone(),
            capabilities,
        },
    ))
}

/// Loads the conventional manifest from an explicitly selected local package
/// directory and builds its read-only WASM tools.
pub fn load_read_only_wasm_package(
    runtime: Arc<WasmRuntime>,
    package_root: &Path,
) -> Result<(Vec<Arc<dyn AgentTool>>, LoadedPluginPackage), WasmError> {
    let manifest_path = package_root.join(PLUGIN_MANIFEST_FILE);
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| WasmError::Io(error.to_string()))?;
    let manifest = parse_manifest(&text).map_err(|error| WasmError::Manifest(error.to_string()))?;
    build_read_only_wasm_tools(runtime, package_root, &manifest)
}

fn plugin_tool_name(plugin_name: &str, tool_name: &str) -> String {
    format!("{plugin_name}.{tool_name}")
}

fn confined_package_path(package_root: &Path, relative: &str) -> Result<PathBuf, WasmError> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Err(WasmError::PathEscape(relative.to_string()));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => return Err(WasmError::PathEscape(relative.to_string())),
        }
    }
    let candidate = package_root.join(path);
    if candidate.exists() {
        let root = package_root
            .canonicalize()
            .map_err(|e| WasmError::Io(e.to_string()))?;
        let real = candidate
            .canonicalize()
            .map_err(|e| WasmError::Io(e.to_string()))?;
        if !real.starts_with(root) {
            return Err(WasmError::PathEscape(relative.to_string()));
        }
    }
    Ok(candidate)
}

fn bound_plugin_output(output: &str) -> String {
    if output.len() <= MAX_PLUGIN_TOOL_OUTPUT {
        return output.to_string();
    }
    const MARKER: &str = "...[truncated]";
    let mut end = MAX_PLUGIN_TOOL_OUTPUT.saturating_sub(MARKER.len());
    while !output.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &output[..end], MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::parse_manifest;
    use serde_json::json;
    use std::fs;
    use std::time::Instant;

    const FUEL: u64 = 100_000;
    const TIMEOUT_MS: u64 = 3_000;

    fn runtime() -> Arc<WasmRuntime> {
        Arc::new(WasmRuntime::new(FUEL, TIMEOUT_MS).expect("runtime"))
    }

    #[tokio::test]
    async fn checked_in_package_loads_with_typed_capabilities_and_executes_offline() {
        let package =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/read-only-demo");
        let (tools, loaded) =
            load_read_only_wasm_package(runtime(), &package).expect("fixture loads");

        assert_eq!(loaded.name, "read-only-demo");
        assert_eq!(loaded.version, "0.1.0");
        assert_eq!(loaded.carrier, "wasm");
        assert_eq!(loaded.capabilities, vec!["read-only-demo.answer"]);
        assert_eq!(tools.len(), 1);
        let result = tools[0].execute(json!({})).await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("returned 7"));
        assert!(matches!(
            tools[0].provenance(),
            ToolProvenance::Plugin { ref name, .. } if name == "read-only-demo"
        ));
    }

    fn wasm_i32_const(value: u8) -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, b'r', b'u', b'n', 0x00, 0x00,
            0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, value, 0x0b,
        ]
    }

    fn temp_package() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "talos-plugin-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp package");
        path
    }

    fn manifest_with_handler(handler: &str) -> PluginManifest {
        parse_manifest(&format!(
            r#"
[plugin]
name = "demo"
version = "0.1.0"
carrier = "wasm"
artifact = "plugin.wasm"

[[tools]]
name = "answer"
handler = "{handler}"
"#
        ))
        .expect("manifest")
    }

    #[test]
    fn success_fixture() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                i32.const 42))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        assert_eq!(module.execute().unwrap(), 42);
    }

    #[test]
    fn success_fixture_does_not_wait_for_timeout() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                i32.const 7))
        "#;
        let slow_timeout_runtime = Arc::new(WasmRuntime::new(FUEL, 1_000).expect("runtime"));
        let module = WasmModule::from_wat(slow_timeout_runtime, wat).expect("compile");
        let started = Instant::now();

        assert_eq!(module.execute().unwrap(), 7);
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "successful WASM execution should not wait for the timeout watchdog"
        );
    }

    #[test]
    fn invalid_module_rejected() {
        let result = WasmModule::from_bytes(runtime(), b"not a wasm module");
        assert!(result.is_err());
        assert!(matches!(result, Err(WasmError::Compile(_))));
    }

    #[test]
    fn trap_handled_gracefully() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                unreachable))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::Trap(_))));
    }

    #[test]
    fn fuel_exhaustion_handled() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                (loop $forever
                  (br $forever))
                (i32.const 0)))
        "#;
        let low_fuel_runtime = Arc::new(WasmRuntime::new(100, TIMEOUT_MS).expect("runtime"));
        let module = WasmModule::from_wat(low_fuel_runtime, wat).expect("compile");
        let result = module.execute();
        assert!(result.is_err());
    }

    #[test]
    fn timeout_handled() {
        let wat = r#"
            (module
              (func (export "run") (result i32)
                (loop $forever
                  (br $forever))
                (i32.const 0)))
        "#;
        let fast_timeout_runtime = Arc::new(WasmRuntime::new(1_000_000_000, 200).expect("runtime"));
        let module = WasmModule::from_wat(fast_timeout_runtime, wat).expect("compile");
        let result = module.execute();
        assert!(result.is_err());
    }

    #[test]
    fn memory_access_bounds_enforced() {
        let wat = r#"
            (module
              (memory (export "memory") 1)
              (func (export "run") (result i32)
                (i32.load (i32.const 0xFFFFFF00))))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::Trap(_))));
    }

    #[test]
    fn missing_export_rejected() {
        let wat = r#"
            (module
              (func (export "other") (result i32)
                i32.const 0))
        "#;
        let module = WasmModule::from_wat(runtime(), wat).expect("compile");
        let result = module.execute();
        assert!(matches!(result, Err(WasmError::MissingExport)));
    }

    #[test]
    fn no_host_imports_available() {
        let wat = r#"
            (module
              (import "env" "read_file" (func $read_file (param i32) (result i32)))
              (func (export "run") (result i32)
                (call $read_file (i32.const 0))))
        "#;
        let result = WasmModule::from_wat(runtime(), wat);
        assert!(result.is_err() || result.unwrap().execute().is_err());
    }

    #[tokio::test]
    async fn register_valid_local_package_registers_read_only_plugin_tool() {
        let package = temp_package();
        fs::write(package.join("plugin.wasm"), wasm_i32_const(7)).expect("artifact");
        fs::write(package.join("tool.wasm"), wasm_i32_const(42)).expect("handler");
        let manifest = manifest_with_handler("tool.wasm");
        let mut registry = ToolRegistry::new();

        let count =
            register_read_only_wasm_tools(&mut registry, runtime(), &package, &manifest).unwrap();

        assert_eq!(count, 1);
        let tool = registry.get("demo.answer").expect("tool registered");
        assert!(tool.is_read_only());
        assert_eq!(tool.family(), ToolFamily::Plugin);
        assert_eq!(
            tool.provenance(),
            ToolProvenance::Plugin {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                carrier: "wasm".to_string(),
            }
        );
        let profile = tool.permission_profile(&json!({}));
        assert_eq!(profile[0].nature, ToolNature::Read);

        let result = tool.execute(json!({})).await;
        assert!(!result.is_error);
        assert!(result.content.contains("returned 42"));
    }

    #[test]
    fn absolute_handler_path_is_rejected_before_loading() {
        let package = temp_package();
        fs::write(package.join("plugin.wasm"), wasm_i32_const(7)).expect("artifact");
        let manifest = manifest_with_handler("/tmp/tool.wasm");

        let result =
            WasmPluginTool::from_manifest_tool(runtime(), &package, &manifest, &manifest.tools[0]);

        assert!(matches!(result, Err(WasmError::PathEscape(_))));
    }

    #[test]
    fn parent_dir_handler_path_is_rejected_before_loading() {
        let package = temp_package();
        fs::write(package.join("plugin.wasm"), wasm_i32_const(7)).expect("artifact");
        let manifest = manifest_with_handler("../tool.wasm");

        let result =
            WasmPluginTool::from_manifest_tool(runtime(), &package, &manifest, &manifest.tools[0]);

        assert!(matches!(result, Err(WasmError::PathEscape(_))));
    }

    #[test]
    fn tool_name_collision_is_rejected() {
        let package = temp_package();
        fs::write(package.join("plugin.wasm"), wasm_i32_const(7)).expect("artifact");
        fs::write(package.join("tool.wasm"), wasm_i32_const(42)).expect("handler");
        let manifest = manifest_with_handler("tool.wasm");
        let mut registry = ToolRegistry::new();
        register_read_only_wasm_tools(&mut registry, runtime(), &package, &manifest).unwrap();

        let result = register_read_only_wasm_tools(&mut registry, runtime(), &package, &manifest);

        assert!(matches!(result, Err(WasmError::ToolCollision(name)) if name == "demo.answer"));
    }

    #[test]
    fn plugin_output_is_bounded_on_utf8_boundary() {
        let output = "a".repeat(MAX_PLUGIN_TOOL_OUTPUT + 10);

        let bounded = bound_plugin_output(&output);

        assert!(bounded.len() <= MAX_PLUGIN_TOOL_OUTPUT);
        assert!(bounded.ends_with("...[truncated]"));
    }

    #[test]
    fn runtime_default_does_not_present_registered_plugin_tool() {
        let package = temp_package();
        fs::write(package.join("plugin.wasm"), wasm_i32_const(7)).expect("artifact");
        fs::write(package.join("tool.wasm"), wasm_i32_const(42)).expect("handler");
        let manifest = manifest_with_handler("tool.wasm");
        let mut registry = ToolRegistry::new();
        register_read_only_wasm_tools(&mut registry, runtime(), &package, &manifest).unwrap();
        let tool = registry.get("demo.answer").expect("tool registered");

        assert!(!talos_core::tool::ToolPresentationPolicy::runtime_default().allows_tool(tool));
        assert!(
            talos_core::tool::ToolPresentationPolicy::with_tool("demo.answer").allows_tool(tool)
        );
    }
}
