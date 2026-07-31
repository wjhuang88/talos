from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, got {count}")
    return text.replace(old, new, 1)


path = Path("crates/talos-cli/src/registry.rs")
text = path.read_text()

text = replace_once(
    text,
    '''use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolContribution, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult,
};
''',
    '''use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolContribution, ToolContributionSource,
    ToolExecutionAuthorization, ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry,
    ToolResult,
};
''',
    "core tool imports",
)

old_plugin_block = '''type LoadedPluginTools = (Vec<Arc<dyn AgentTool>>, LoadedPluginPackage);

fn load_explicit_plugin_tools(
    registry: &ToolRegistry,
    package_roots: &[PathBuf],
) -> Result<Vec<LoadedPluginTools>, String> {
    if package_roots.is_empty() {
        return Ok(Vec::new());
    }
    let runtime = Arc::new(
        WasmRuntime::new(100_000, 250)
            .map_err(|error| format!("failed to initialize WASM runtime: {error}"))?,
    );
    let mut loaded = Vec::with_capacity(package_roots.len());
    let mut pending_names = HashSet::new();
    for package_root in package_roots {
        let (tools, package) =
            load_read_only_wasm_package(runtime.clone(), package_root).map_err(|error| {
                format!(
                    "failed to load plugin package '{}': {error}",
                    package_root.display()
                )
            })?;
        for tool in &tools {
            let name = tool.name().to_string();
            if registry.get(&name).is_some() || !pending_names.insert(name.clone()) {
                return Err(format!(
                    "plugin tool name collides with registered tool: {name}"
                ));
            }
        }
        loaded.push((tools, package));
    }
    Ok(loaded)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the blocking/print permission adapter.
pub(crate) fn register_explicit_permission_aware_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(registry, package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    for (tools, package) in loaded {
        register_permission_aware_tools(registry, &tools, approval.clone(), print_mode);
        packages.push(package);
    }
    Ok(packages)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the non-blocking TUI permission adapter.
pub(crate) fn register_explicit_tui_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<TuiApprovalHandler>,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(registry, package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    for (tools, package) in loaded {
        register_tui_permission_aware_tools(registry, &tools, approval.clone());
        packages.push(package);
    }
    Ok(packages)
}
'''
new_plugin_block = '''type LoadedPluginTools = (Vec<Arc<dyn AgentTool>>, LoadedPluginPackage);

fn load_explicit_plugin_tools(
    package_roots: &[PathBuf],
) -> Result<Vec<LoadedPluginTools>, String> {
    if package_roots.is_empty() {
        return Ok(Vec::new());
    }
    let runtime = Arc::new(
        WasmRuntime::new(100_000, 250)
            .map_err(|error| format!("failed to initialize WASM runtime: {error}"))?,
    );
    let mut loaded = Vec::with_capacity(package_roots.len());
    for package_root in package_roots {
        let (tools, package) =
            load_read_only_wasm_package(runtime.clone(), package_root).map_err(|error| {
                format!(
                    "failed to load plugin package '{}': {error}",
                    package_root.display()
                )
            })?;
        loaded.push((tools, package));
    }
    Ok(loaded)
}

fn plugin_source(package: &LoadedPluginPackage) -> ToolContributionSource {
    ToolContributionSource::new(format!("plugin:{}@{}", package.name, package.version))
}

/// Loads explicitly selected local packages and registers their tools behind
/// the blocking/print permission adapter.
pub(crate) fn register_explicit_permission_aware_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    let mut contributions = Vec::new();
    for (tools, package) in loaded {
        let source = plugin_source(&package);
        contributions.extend(tools.into_iter().map(|tool| {
            ToolContribution::new(source.clone(), tool).map_tool(|tool| {
                Arc::new(PermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                    print_mode,
                })
            })
        }));
        packages.push(package);
    }
    registry
        .register_contributions(contributions)
        .map_err(|error| error.to_string())?;
    Ok(packages)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the non-blocking TUI permission adapter.
pub(crate) fn register_explicit_tui_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<TuiApprovalHandler>,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    let mut contributions = Vec::new();
    for (tools, package) in loaded {
        let source = plugin_source(&package);
        contributions.extend(tools.into_iter().map(|tool| {
            ToolContribution::new(source.clone(), tool).map_tool(|tool| {
                Arc::new(TuiPermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                })
            })
        }));
        packages.push(package);
    }
    registry
        .register_contributions(contributions)
        .map_err(|error| error.to_string())?;
    Ok(packages)
}
'''
text = replace_once(text, old_plugin_block, new_plugin_block, "explicit plugin registration block")

remote_marker = '''    struct RemoteTool {
        nature: ToolNature,
    }
'''
named_tool = '''    struct NamedReadTool {
        name: &'static str,
        description: &'static str,
    }

    #[async_trait]
    impl AgentTool for NamedReadTool {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            self.description
        }

        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            ToolResult::success(self.description)
        }
    }

    fn write_plugin_fixture(
        root: &Path,
        package_name: &str,
        version: &str,
        tool_name: &str,
    ) -> PathBuf {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(
            root.join("talos-plugin.toml"),
            format!(
                "[plugin]\\nname = \\"{package_name}\\"\\nversion = \\"{version}\\"\\ncarrier = \\"wasm\\"\\nartifact = \\"demo.wat\\"\\ndescription = \\"collision fixture\\"\\n\\n[[tools]]\\nname = \\"{tool_name}\\"\\nhandler = \\"demo.wat\\"\\n"
            ),
        )
        .unwrap();
        std::fs::write(
            root.join("demo.wat"),
            "(module (func (export \\"run\\") (result i32) i32.const 7))",
        )
        .unwrap();
        root.to_path_buf()
    }

'''
text = replace_once(text, remote_marker, named_tool + remote_marker, "test helper insertion")

existing_plugin_test_end = '''        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("returned 7"));
    }

    #[tokio::test]
    async fn print_composition_read_uses_model_private_snapshot_projection() {
'''
new_plugin_tests = '''        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("returned 7"));
    }

    #[test]
    fn explicit_plugin_collision_with_checked_builtin_reports_both_sources() {
        let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../talos-plugin/tests/fixtures/read-only-demo");
        let mut registry = ToolRegistry::new();
        registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(NamedReadTool {
                    name: "read-only-demo.answer",
                    description: "existing built-in",
                }),
            ))
            .unwrap();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

        let error = register_explicit_permission_aware_plugins(
            &mut registry,
            &[package],
            approval,
            true,
        )
        .unwrap_err();

        assert_eq!(
            error,
            "duplicate tool registration 'read-only-demo.answer': existing source 'talos-tools:file', incoming source 'plugin:read-only-demo@0.1.0'"
        );
        assert_eq!(
            registry
                .get("read-only-demo.answer")
                .unwrap()
                .description(),
            "existing built-in"
        );
    }

    #[test]
    fn explicit_plugin_collision_between_packages_is_transactional() {
        let temp = tempfile::tempdir().unwrap();
        let first = write_plugin_fixture(
            &temp.path().join("first"),
            "collision",
            "1.0.0",
            "answer",
        );
        let second = write_plugin_fixture(
            &temp.path().join("second"),
            "collision",
            "2.0.0",
            "answer",
        );
        let mut registry = ToolRegistry::new();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

        let error = register_explicit_permission_aware_plugins(
            &mut registry,
            &[first, second],
            approval,
            true,
        )
        .unwrap_err();

        assert_eq!(
            error,
            "duplicate tool registration 'collision.answer': existing source 'plugin:collision@1.0.0', incoming source 'plugin:collision@2.0.0'"
        );
        assert!(registry.get("collision.answer").is_none());
    }

    #[tokio::test]
    async fn print_composition_read_uses_model_private_snapshot_projection() {
'''
text = replace_once(
    text,
    existing_plugin_test_end,
    new_plugin_tests,
    "plugin collision tests",
)

path.write_text(text)
