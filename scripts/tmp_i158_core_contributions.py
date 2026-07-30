from pathlib import Path

path = Path("crates/talos-core/src/tool.rs")
text = path.read_text()

registry_marker = '''/// A registry for dynamically managing agent tools.
///
/// Tools are registered under their [`AgentTool::name`] and can be retrieved,
/// listed, or have their inputs validated against their parameter schemas.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
}
'''

contract = '''/// Stable, display-safe identity for the crate, product profile, or plugin
/// that contributed a tool.
///
/// Sources are diagnostics only: they do not grant permissions and must not
/// contain credentials, workspace paths, tool arguments, or projected input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ToolContributionSource(String);

impl ToolContributionSource {
    /// Creates a stable contribution source identity.
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    /// Returns the source identity as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolContributionSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One explicitly sourced tool instance ready for product composition.
#[derive(Clone)]
pub struct ToolContribution {
    source: ToolContributionSource,
    tool: Arc<dyn AgentTool>,
}

impl ToolContribution {
    /// Creates a sourced tool contribution.
    pub fn new(source: ToolContributionSource, tool: Arc<dyn AgentTool>) -> Self {
        Self { source, tool }
    }

    /// Returns the stable source identity.
    #[must_use]
    pub fn source(&self) -> &ToolContributionSource {
        &self.source
    }

    /// Returns the contributed tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.tool.name()
    }

    /// Returns the contributed tool instance.
    #[must_use]
    pub fn tool(&self) -> &Arc<dyn AgentTool> {
        &self.tool
    }

    /// Applies an outer composition wrapper while preserving source identity.
    #[must_use]
    pub fn map_tool(
        mut self,
        wrap: impl FnOnce(Arc<dyn AgentTool>) -> Arc<dyn AgentTool>,
    ) -> Self {
        self.tool = wrap(self.tool);
        self
    }
}

/// Deterministic duplicate-name diagnostic for checked tool composition.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "duplicate tool registration '{tool_name}': existing source '{existing_source}', incoming source '{incoming_source}'"
)]
pub struct ToolRegistrationError {
    /// Duplicate tool name.
    pub tool_name: String,
    /// Source that already owns the registered name.
    pub existing_source: ToolContributionSource,
    /// Source that attempted the duplicate registration.
    pub incoming_source: ToolContributionSource,
}

const LEGACY_TOOL_REGISTRATION_SOURCE: &str = "legacy:unchecked";

/// A registry for dynamically managing agent tools.
///
/// Tools are registered under their [`AgentTool::name`] and can be retrieved,
/// listed, or have their inputs validated against their parameter schemas.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn AgentTool>>,
    contribution_sources: HashMap<String, ToolContributionSource>,
}
'''

if registry_marker not in text:
    raise SystemExit("registry marker not found")
text = text.replace(registry_marker, contract, 1)

old_register = '''    /// Registers a tool in the registry, replacing any existing tool with the
    /// same name.
    pub fn register(&mut self, tool: Arc<dyn AgentTool>) {
        self.tools.insert(tool.name().to_owned(), tool);
    }
'''
new_register = '''    /// Registers a tool in the registry, replacing any existing tool with the
    /// same name.
    ///
    /// This historical unchecked API remains temporarily source-compatible
    /// during I158 migration. New product composition should use
    /// [`register_contribution`](Self::register_contribution).
    pub fn register(&mut self, tool: Arc<dyn AgentTool>) {
        let name = tool.name().to_owned();
        self.contribution_sources.remove(&name);
        self.tools.insert(name, tool);
    }

    /// Registers one explicitly sourced contribution without replacing an
    /// existing tool.
    ///
    /// A duplicate returns both source identities and leaves the current
    /// registry entry unchanged.
    pub fn register_contribution(
        &mut self,
        contribution: ToolContribution,
    ) -> Result<(), ToolRegistrationError> {
        let ToolContribution { source, tool } = contribution;
        let tool_name = tool.name().to_owned();

        if self.tools.contains_key(&tool_name) {
            let existing_source = self
                .contribution_sources
                .get(&tool_name)
                .cloned()
                .unwrap_or_else(|| ToolContributionSource::new(LEGACY_TOOL_REGISTRATION_SOURCE));
            return Err(ToolRegistrationError {
                tool_name,
                existing_source,
                incoming_source: source,
            });
        }

        self.contribution_sources
            .insert(tool_name.clone(), source);
        self.tools.insert(tool_name, tool);
        Ok(())
    }
'''
if old_register not in text:
    raise SystemExit("register method marker not found")
text = text.replace(old_register, new_register, 1)

old_test = '''    #[test]
    fn test_register_replaces_existing() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Original")));
        registry.register(Arc::new(MockTool::new("echo", "Replacement")));

        let tool = registry.get("echo").unwrap();
        assert_eq!(tool.description(), "Replacement");
    }
'''
new_tests = old_test + '''
    #[test]
    fn checked_contribution_registers_with_source_identity() {
        let mut registry = ToolRegistry::new();
        let contribution = ToolContribution::new(
            ToolContributionSource::new("talos-tools:test"),
            Arc::new(MockTool::new("echo", "Checked")),
        );

        assert_eq!(contribution.source().as_str(), "talos-tools:test");
        assert_eq!(contribution.name(), "echo");
        assert_eq!(contribution.tool().description(), "Checked");
        registry.register_contribution(contribution).unwrap();

        assert_eq!(registry.get("echo").unwrap().description(), "Checked");
    }

    #[test]
    fn contribution_wrapper_preserves_source_identity() {
        let contribution = ToolContribution::new(
            ToolContributionSource::new("talos-tools:test"),
            Arc::new(MockTool::new("echo", "Inner")),
        )
        .map_tool(|_| Arc::new(MockTool::new("echo", "Wrapped")));

        assert_eq!(contribution.source().as_str(), "talos-tools:test");
        assert_eq!(contribution.name(), "echo");
        assert_eq!(contribution.tool().description(), "Wrapped");
    }

    #[test]
    fn checked_duplicate_reports_both_sources_and_preserves_existing_tool() {
        let mut registry = ToolRegistry::new();
        registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(MockTool::new("echo", "Original")),
            ))
            .unwrap();

        let error = registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("plugin:demo@0.1.0"),
                Arc::new(MockTool::new("echo", "Replacement")),
            ))
            .unwrap_err();

        assert_eq!(
            error,
            ToolRegistrationError {
                tool_name: "echo".to_owned(),
                existing_source: ToolContributionSource::new("talos-tools:file"),
                incoming_source: ToolContributionSource::new("plugin:demo@0.1.0"),
            }
        );
        assert_eq!(
            error.to_string(),
            "duplicate tool registration 'echo': existing source 'talos-tools:file', incoming source 'plugin:demo@0.1.0'"
        );
        assert_eq!(registry.get("echo").unwrap().description(), "Original");
    }

    #[test]
    fn checked_duplicate_after_legacy_registration_has_stable_source() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool::new("echo", "Legacy")));

        let error = registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(MockTool::new("echo", "Checked")),
            ))
            .unwrap_err();

        assert_eq!(error.existing_source.as_str(), "legacy:unchecked");
        assert_eq!(error.incoming_source.as_str(), "talos-tools:file");
        assert_eq!(registry.get("echo").unwrap().description(), "Legacy");
    }

    #[test]
    fn legacy_register_still_replaces_checked_registration() {
        let mut registry = ToolRegistry::new();
        registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(MockTool::new("echo", "Checked")),
            ))
            .unwrap();
        registry.register(Arc::new(MockTool::new("echo", "Legacy replacement")));

        assert_eq!(
            registry.get("echo").unwrap().description(),
            "Legacy replacement"
        );
        let error = registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("plugin:demo@0.1.0"),
                Arc::new(MockTool::new("echo", "Plugin")),
            ))
            .unwrap_err();
        assert_eq!(error.existing_source.as_str(), "legacy:unchecked");
    }
'''
if old_test not in text:
    raise SystemExit("registry test marker not found")
text = text.replace(old_test, new_tests, 1)

path.write_text(text)
