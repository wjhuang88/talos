//! Real installed guests behind the production Agent permission controller.
#![cfg(feature = "plugin-acceptance")]

use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use talos_agent::Agent;
use talos_core::{
    capability::CapabilityRegistry,
    message::AgentEvent,
    tool::{AgentTool, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult},
};
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
use talos_plugin::{install_bundle, lifecycle::PluginLifecycle, wasm::WasmRuntime};
use talos_provider::mock::MockProvider;
use talos_tools::symbol::ListSymbolsTool;

struct CountedTool {
    inner: ListSymbolsTool,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl AgentTool for CountedTool {
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
    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        self.inner.permission_profile(input)
    }
    async fn execute(&self, input: Value) -> ToolResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.execute(input).await
    }
}

#[tokio::test]
async fn installed_language_tool_obeys_production_agent_deny() {
    let bundles = std::path::PathBuf::from(
        std::env::var_os("TALOS_LANGUAGE_BUNDLE_ROOT")
            .expect("provide real packaged Bundles for plugin-acceptance"),
    );
    let temp = tempfile::tempdir().expect("test directory");
    for (language, extension, source) in [
        ("rust", "rs", "fn acceptance_symbol() {}"),
        ("python", "py", "def acceptance_symbol():\n    pass\n"),
    ] {
        let package = temp.path().join(format!("package-{language}"));
        install_bundle(&bundles.join(language), &package).expect("verified install");
        let mut lifecycle =
            PluginLifecycle::new(Arc::new(Mutex::new(CapabilityRegistry::default())));
        lifecycle.load(&package).expect("load");
        lifecycle
            .initialize(Arc::new(WasmRuntime::new(100_000, 250).expect("runtime")))
            .expect("initialize");
        lifecycle.activate().expect("activate");
        let context = lifecycle
            .take_language_provider_context()
            .expect("context state")
            .expect("provider");
        let workspace = temp.path().join(language);
        std::fs::create_dir(&workspace).expect("workspace");
        std::fs::write(workspace.join(format!("sample.{extension}")), source).expect("source");
        let calls = Arc::new(AtomicUsize::new(0));
        let tool = Arc::new(CountedTool {
            inner: ListSymbolsTool::with_provider(workspace.clone(), context),
            calls: calls.clone(),
        });
        // Both controls traverse the same production controller and installed guest.
        for allowed in [true, false] {
            calls.store(0, Ordering::SeqCst);
            let mut registry = ToolRegistry::new();
            registry.register(tool.clone());
            let mut permissions = PermissionEngine::empty();
            permissions.add_rule(PermissionRule::new(
                "list_symbols",
                None,
                if allowed {
                    PermissionDecision::Allow
                } else {
                    PermissionDecision::Deny("blocked by plugin acceptance".into())
                },
            ));
            let model = MockProvider::new()
                .with_tool_call("list_symbols", json!({"path":".", "kind":"function"}))
                .with_response("done");
            let agent = Agent::with_security(
                Arc::new(model),
                registry,
                Some(Arc::new(permissions)),
                None,
                workspace.clone(),
            );
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            agent
                .run_streaming("list functions".into(), vec![], tx)
                .await
                .expect("agent turn");
            let mut results = Vec::new();
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result } = event {
                    results.push(result);
                }
            }
            assert_eq!(results.len(), 1);
            assert_eq!(calls.load(Ordering::SeqCst), usize::from(allowed));
            assert_eq!(results[0].is_error, !allowed);
            if allowed {
                assert!(results[0].content.contains("acceptance_symbol"));
            } else {
                assert!(
                    results[0].content.contains("Permission denied"),
                    "{:?}",
                    results[0]
                );
            }
        }
        lifecycle.stop().expect("stop");
    }
}
