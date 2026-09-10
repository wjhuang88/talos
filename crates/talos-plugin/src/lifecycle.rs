//! Explicit local Plugin lifecycle. Declarations are requests, never permissions.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use talos_core::capability::{Carrier, Provenance, ProviderRegistration};
use talos_core::tool::{AgentTool, ToolFamily, ToolPermissionFacet, ToolProvenance, ToolResult};
use talos_core::{CapabilityDescriptor, CapabilityRegistry, ProviderDescriptor};

use crate::PluginManifest;
use crate::manifest::parse_manifest;
use crate::wasm::{LoadedPluginPackage, PLUGIN_MANIFEST_FILE, WasmPluginTool, WasmRuntime};

/// Observable lifecycle of an explicitly selected Plugin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginState {
    /// No manifest has been loaded.
    Unloaded,
    /// Manifest parsed without executing code.
    Loaded,
    /// Executable bindings validated, but not published.
    Initialized,
    /// Provider registered and new tool calls admitted.
    Active,
    /// New calls denied and owned providers withdrawn.
    Stopped,
}

/// Recoverable lifecycle failure, separate from legacy public error enums.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    /// Invalid declaration or executable binding.
    #[error("invalid plugin: {0}")]
    Invalid(String),
    /// Declared carrier has no authorized adapter.
    #[error("plugin carrier unavailable: {0}")]
    Unavailable(String),
    /// Operation is not valid in the current state.
    #[error("invalid plugin lifecycle transition from {0:?}")]
    State(PluginState),
    /// Synchronization failed; no new invocation is admitted.
    #[error("plugin lifecycle synchronization failed")]
    Synchronization,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declarations {
    schema_version: u32,
    provider_id: String,
    version: String,
    capabilities: Vec<Binding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    version: String,
    tool: String,
}

#[derive(Default, Deserialize)]
struct Envelope {
    capability_provider: Option<Declarations>,
}

struct Gate {
    state: PluginState,
    registration: Option<ProviderRegistration>,
}

struct Shared {
    gate: Mutex<Gate>,
    registry: Arc<Mutex<CapabilityRegistry>>,
}

impl Shared {
    fn stop(&self) -> Result<(), LifecycleError> {
        let mut gate = self
            .gate
            .lock()
            .map_err(|_| LifecycleError::Synchronization)?;
        gate.state = PluginState::Stopped;
        if let Some(registration) = &gate.registration {
            self.registry
                .lock()
                .map_err(|_| LifecycleError::Synchronization)?
                .withdraw(registration);
        }
        gate.registration = None;
        Ok(())
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        // Last controller/tool handle releases its publication. Poison recovery is
        // cleanup-only: it never admits execution or publishes a provider.
        let gate = self
            .gate
            .get_mut()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(registration) = &gate.registration {
            self.registry
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .withdraw(registration);
        }
    }
}

/// Controller for one explicit package, sharing a stop/admission fence with its tools.
///
/// No files are rewritten. The supplied registry is the host's single capability
/// registry. Dropping this controller alone does not stop tools retained by a host;
/// call [`Self::stop`] explicitly, or drop all controller and tool handles.
pub struct PluginLifecycle {
    shared: Arc<Shared>,
    manifest: Option<PluginManifest>,
    descriptor: Option<ProviderDescriptor>,
    root: PathBuf,
    tools: Vec<Arc<dyn AgentTool>>,
    package: Option<LoadedPluginPackage>,
}

impl PluginLifecycle {
    /// Create an unloaded controller without filesystem access or guest execution.
    pub fn new(registry: Arc<Mutex<CapabilityRegistry>>) -> Self {
        Self {
            shared: Arc::new(Shared {
                gate: Mutex::new(Gate {
                    state: PluginState::Unloaded,
                    registration: None,
                }),
                registry,
            }),
            manifest: None,
            descriptor: None,
            root: PathBuf::new(),
            tools: Vec::new(),
            package: None,
        }
    }

    /// Read current state; poisoned synchronization fails closed.
    pub fn state(&self) -> PluginState {
        self.shared
            .gate
            .lock()
            .map(|gate| gate.state)
            .unwrap_or(PluginState::Stopped)
    }

    /// Load declarations only. Legacy manifests map each tool to a version 1
    /// capability; original names and versions remain in provenance/metadata.
    pub fn load(&mut self, root: &Path) -> Result<(), LifecycleError> {
        if self.state() != PluginState::Unloaded {
            return Err(LifecycleError::State(self.state()));
        }
        let text = std::fs::read_to_string(root.join(PLUGIN_MANIFEST_FILE))
            .map_err(|error| LifecycleError::Invalid(error.to_string()))?;
        let raw: PluginManifest =
            toml::from_str(&text).map_err(|error| LifecycleError::Invalid(error.to_string()))?;
        if raw.plugin.carrier != "wasm" {
            return Err(LifecycleError::Unavailable(raw.plugin.carrier));
        }
        let manifest =
            parse_manifest(&text).map_err(|error| LifecycleError::Invalid(error.to_string()))?;
        let envelope: Envelope =
            toml::from_str(&text).map_err(|error| LifecycleError::Invalid(error.to_string()))?;
        let descriptor = descriptor(&manifest, envelope.capability_provider)?;
        self.manifest = Some(manifest);
        self.descriptor = Some(descriptor);
        self.root = root.to_owned();
        self.shared
            .gate
            .lock()
            .map_err(|_| LifecycleError::Synchronization)?
            .state = PluginState::Loaded;
        Ok(())
    }

    /// Compile and validate every executable binding without instantiating guest code.
    pub fn initialize(&mut self, runtime: Arc<WasmRuntime>) -> Result<(), LifecycleError> {
        if self.state() != PluginState::Loaded {
            return Err(LifecycleError::State(self.state()));
        }
        let manifest = self
            .manifest
            .as_ref()
            .ok_or_else(|| LifecycleError::State(self.state()))?;
        let mut tools: Vec<Arc<dyn AgentTool>> = Vec::new();
        for tool in &manifest.tools {
            let executable = contain_initialization(|| {
                let executable =
                    WasmPluginTool::from_manifest_tool(runtime.clone(), &self.root, manifest, tool)
                        .map_err(|error| LifecycleError::Invalid(error.to_string()))?;
                executable
                    .validate_binding()
                    .map_err(|error| LifecycleError::Invalid(error.to_string()))?;
                Ok(executable)
            })?;
            tools.push(Arc::new(executable));
        }
        let package = LoadedPluginPackage {
            name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            carrier: manifest.plugin.carrier.clone(),
            capabilities: tools.iter().map(|tool| tool.name().to_owned()).collect(),
        };
        self.tools = tools
            .into_iter()
            .map(|inner| {
                Arc::new(LifecycleTool {
                    inner,
                    shared: self.shared.clone(),
                }) as Arc<dyn AgentTool>
            })
            .collect();
        self.package = Some(package);
        self.shared
            .gate
            .lock()
            .map_err(|_| LifecycleError::Synchronization)?
            .state = PluginState::Initialized;
        Ok(())
    }

    /// Publish the validated provider exclusively. Collision leaves no partial entry.
    pub fn activate(&mut self) -> Result<(), LifecycleError> {
        let mut gate = self
            .shared
            .gate
            .lock()
            .map_err(|_| LifecycleError::Synchronization)?;
        if gate.state != PluginState::Initialized {
            return Err(LifecycleError::State(gate.state));
        }
        let descriptor = self
            .descriptor
            .clone()
            .ok_or(LifecycleError::State(gate.state))?;
        let registration = self
            .shared
            .registry
            .lock()
            .map_err(|_| LifecycleError::Synchronization)?
            .register_owned(vec![descriptor])
            .map_err(|error| LifecycleError::Invalid(error.to_string()))?;
        gate.registration = Some(registration);
        gate.state = PluginState::Active;
        Ok(())
    }

    /// Stop new admissions and withdraw owned capabilities. Calls admitted before
    /// this fence may finish under their existing WASM fuel/wall-clock bounds.
    pub fn stop(&self) -> Result<(), LifecycleError> {
        self.shared.stop()
    }

    /// Return initialized handles; execution still requires Active and host permission.
    pub fn tools(&self) -> Vec<Arc<dyn AgentTool>> {
        self.tools.clone()
    }

    /// Return legacy package diagnostics after successful initialization.
    pub fn package(&self) -> Option<&LoadedPluginPackage> {
        self.package.as_ref()
    }
}

fn contain_initialization<T>(
    operation: impl FnOnce() -> Result<T, LifecycleError>,
) -> Result<T, LifecycleError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
        .map_err(|_| LifecycleError::Invalid("host panic during plugin initialization".into()))?
}

fn safe_identity(text: &str) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for byte in text.as_bytes() {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn descriptor(
    manifest: &PluginManifest,
    declarations: Option<Declarations>,
) -> Result<ProviderDescriptor, LifecycleError> {
    let metadata = BTreeMap::from([
        ("plugin.name".into(), manifest.plugin.name.clone()),
        ("plugin.version".into(), manifest.plugin.version.clone()),
    ]);
    let (id, version, bindings) = match declarations {
        Some(declared) => {
            if declared.schema_version != 1 || declared.capabilities.is_empty() {
                return Err(LifecycleError::Invalid(
                    "unsupported or empty capability declaration".into(),
                ));
            }
            (
                declared.provider_id,
                declared.version,
                declared.capabilities,
            )
        }
        None => (
            format!("plugin.{}", safe_identity(&manifest.plugin.name)),
            "1.0.0".into(),
            manifest
                .tools
                .iter()
                .map(|tool| Binding {
                    id: format!(
                        "plugin.{}.tool.{}",
                        safe_identity(&manifest.plugin.name),
                        safe_identity(&tool.name)
                    ),
                    version: "1.0.0".into(),
                    tool: tool.name.clone(),
                })
                .collect(),
        ),
    };
    let mut seen = BTreeSet::new();
    let mut capabilities = Vec::new();
    for binding in bindings {
        if !seen.insert(binding.id.clone())
            || !manifest.tools.iter().any(|tool| tool.name == binding.tool)
        {
            return Err(LifecycleError::Invalid(
                "duplicate capability or missing tool binding".into(),
            ));
        }
        capabilities.push(CapabilityDescriptor {
            id: binding.id,
            version: binding.version,
            name: binding.tool.clone(),
            provenance: Provenance::Plugin,
            carrier: Carrier::Wasm,
            metadata: BTreeMap::from([(
                "tool".into(),
                format!("{}.{}", manifest.plugin.name, binding.tool),
            )]),
        });
    }
    let descriptor = ProviderDescriptor {
        id,
        version,
        provenance: Provenance::Plugin,
        carrier: Carrier::Wasm,
        capabilities,
        metadata,
    };
    descriptor
        .validate()
        .map_err(|error| LifecycleError::Invalid(error.to_string()))?;
    Ok(descriptor)
}

struct LifecycleTool {
    inner: Arc<dyn AgentTool>,
    shared: Arc<Shared>,
}

struct Invocation {
    shared: Arc<Shared>,
    finished: bool,
}

impl Drop for Invocation {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.shared.stop();
        }
    }
}

#[async_trait]
impl AgentTool for LifecycleTool {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn description(&self) -> &str {
        self.inner.description()
    }
    fn parameters(&self) -> Value {
        self.inner.parameters()
    }
    fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }
    fn family(&self) -> ToolFamily {
        self.inner.family()
    }
    fn provenance(&self) -> ToolProvenance {
        self.inner.provenance()
    }
    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        self.inner.permission_profile(input)
    }
    async fn execute(&self, input: Value) -> ToolResult {
        // This lock acquisition is the admission linearization point. No guest
        // code runs under the lock; stop does not wait for admitted executions.
        match self.shared.gate.lock() {
            Ok(gate) if gate.state == PluginState::Active => {}
            _ => return ToolResult::error("plugin is not active"),
        }
        let mut invocation = Invocation {
            shared: self.shared.clone(),
            finished: false,
        };
        let result = self.inner.execute(input).await;
        if result.is_error {
            let _ = self.shared.stop();
        }
        invocation.finished = true;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::{Context, Poll};

    #[test]
    fn initialization_unwind_is_a_recoverable_error() {
        let result: Result<(), LifecycleError> =
            contain_initialization(|| panic!("injected compiler panic"));
        assert!(
            matches!(result, Err(LifecycleError::Invalid(message)) if message.contains("host panic"))
        );
    }

    struct ControlledTool(Arc<AtomicBool>);
    #[async_trait]
    impl AgentTool for ControlledTool {
        fn name(&self) -> &str {
            "controlled"
        }
        fn description(&self) -> &str {
            "controlled test admission"
        }
        fn parameters(&self) -> Value {
            serde_json::json!({})
        }
        async fn execute(&self, _: Value) -> ToolResult {
            std::future::poll_fn(|_| {
                if self.0.load(Ordering::SeqCst) {
                    Poll::Ready(ToolResult::success("finished"))
                } else {
                    Poll::Pending
                }
            })
            .await
        }
    }

    fn controlled() -> (LifecycleTool, Arc<AtomicBool>) {
        let done = Arc::new(AtomicBool::new(false));
        let shared = Arc::new(Shared {
            gate: Mutex::new(Gate {
                state: PluginState::Active,
                registration: None,
            }),
            registry: Arc::new(Mutex::new(CapabilityRegistry::default())),
        });
        (
            LifecycleTool {
                inner: Arc::new(ControlledTool(done.clone())),
                shared,
            },
            done,
        )
    }

    #[test]
    fn admitted_before_stop_can_finish_but_later_calls_are_denied() {
        let (tool, done) = controlled();
        let mut call = tool.execute(Value::Null);
        let mut context = Context::from_waker(futures_util::task::noop_waker_ref());
        assert!(call.as_mut().poll(&mut context).is_pending());
        tool.shared.stop().expect("stop while admitted");
        let mut later = tool.execute(Value::Null);
        assert!(
            matches!(later.as_mut().poll(&mut context), Poll::Ready(result) if result.is_error)
        );
        done.store(true, Ordering::SeqCst);
        assert!(
            matches!(call.as_mut().poll(&mut context), Poll::Ready(result) if !result.is_error)
        );
    }

    #[test]
    fn cancelled_invocation_closes_future_admission() {
        let (tool, _) = controlled();
        let mut call = tool.execute(Value::Null);
        let mut context = Context::from_waker(futures_util::task::noop_waker_ref());
        assert!(call.as_mut().poll(&mut context).is_pending());
        drop(call);
        assert_eq!(
            tool.shared.gate.lock().expect("gate").state,
            PluginState::Stopped
        );
        let mut later = tool.execute(Value::Null);
        assert!(
            matches!(later.as_mut().poll(&mut context), Poll::Ready(result) if result.is_error)
        );
    }
}
