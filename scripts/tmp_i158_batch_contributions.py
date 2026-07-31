from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, got {count}")
    return text.replace(old, new, 1)


path = Path("crates/talos-core/src/tool.rs")
text = path.read_text()

old_method = '''    /// Registers one explicitly sourced contribution without replacing an
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

        self.contribution_sources.insert(tool_name.clone(), source);
        self.tools.insert(tool_name, tool);
        Ok(())
    }
'''
new_methods = '''    /// Registers one explicitly sourced contribution without replacing an
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

        if let Some(existing_source) = self.registered_source(&tool_name) {
            return Err(ToolRegistrationError {
                tool_name,
                existing_source,
                incoming_source: source,
            });
        }

        self.contribution_sources.insert(tool_name.clone(), source);
        self.tools.insert(tool_name, tool);
        Ok(())
    }

    /// Registers a contribution batch transactionally.
    ///
    /// The complete batch is checked against the current registry and against
    /// earlier entries in the same iteration order before any tool is inserted.
    /// On the first duplicate, the registry remains unchanged.
    pub fn register_contributions(
        &mut self,
        contributions: impl IntoIterator<Item = ToolContribution>,
    ) -> Result<(), ToolRegistrationError> {
        let contributions = contributions.into_iter().collect::<Vec<_>>();
        let mut pending_sources = HashMap::<String, ToolContributionSource>::new();

        for contribution in &contributions {
            let tool_name = contribution.name().to_owned();
            if let Some(existing_source) = self.registered_source(&tool_name) {
                return Err(ToolRegistrationError {
                    tool_name,
                    existing_source,
                    incoming_source: contribution.source().clone(),
                });
            }
            if let Some(existing_source) = pending_sources.get(&tool_name) {
                return Err(ToolRegistrationError {
                    tool_name,
                    existing_source: existing_source.clone(),
                    incoming_source: contribution.source().clone(),
                });
            }
            pending_sources.insert(tool_name, contribution.source().clone());
        }

        for ToolContribution { source, tool } in contributions {
            let tool_name = tool.name().to_owned();
            self.contribution_sources.insert(tool_name.clone(), source);
            self.tools.insert(tool_name, tool);
        }
        Ok(())
    }

    fn registered_source(&self, tool_name: &str) -> Option<ToolContributionSource> {
        self.tools.contains_key(tool_name).then(|| {
            self.contribution_sources
                .get(tool_name)
                .cloned()
                .unwrap_or_else(|| ToolContributionSource::new(LEGACY_TOOL_REGISTRATION_SOURCE))
        })
    }
'''
text = replace_once(text, old_method, new_methods, "registry contribution methods")

marker = '''    #[test]
    fn legacy_register_still_replaces_checked_registration() {
'''
tests = '''    #[test]
    fn checked_batch_registers_all_tools_and_sources() {
        let mut registry = ToolRegistry::new();
        registry
            .register_contributions([
                ToolContribution::new(
                    ToolContributionSource::new("plugin:demo@0.1.0"),
                    Arc::new(MockTool::new("echo", "Echo")),
                ),
                ToolContribution::new(
                    ToolContributionSource::new("plugin:demo@0.1.0"),
                    Arc::new(MockTool::new("reverse", "Reverse")),
                ),
            ])
            .unwrap();

        assert_eq!(registry.get("echo").unwrap().description(), "Echo");
        assert_eq!(registry.get("reverse").unwrap().description(), "Reverse");
        let error = registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("plugin:other@1.0.0"),
                Arc::new(MockTool::new("reverse", "Other")),
            ))
            .unwrap_err();
        assert_eq!(error.existing_source.as_str(), "plugin:demo@0.1.0");
    }

    #[test]
    fn checked_batch_collision_with_registry_is_transactional() {
        let mut registry = ToolRegistry::new();
        registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(MockTool::new("echo", "Original")),
            ))
            .unwrap();

        let error = registry
            .register_contributions([
                ToolContribution::new(
                    ToolContributionSource::new("plugin:demo@0.1.0"),
                    Arc::new(MockTool::new("reverse", "Would be inserted first")),
                ),
                ToolContribution::new(
                    ToolContributionSource::new("plugin:demo@0.1.0"),
                    Arc::new(MockTool::new("echo", "Collision")),
                ),
            ])
            .unwrap_err();

        assert_eq!(error.tool_name, "echo");
        assert_eq!(error.existing_source.as_str(), "talos-tools:file");
        assert_eq!(error.incoming_source.as_str(), "plugin:demo@0.1.0");
        assert!(registry.get("reverse").is_none());
        assert_eq!(registry.get("echo").unwrap().description(), "Original");
    }

    #[test]
    fn checked_batch_internal_collision_is_transactional() {
        let mut registry = ToolRegistry::new();
        let error = registry
            .register_contributions([
                ToolContribution::new(
                    ToolContributionSource::new("plugin:first@1.0.0"),
                    Arc::new(MockTool::new("echo", "First")),
                ),
                ToolContribution::new(
                    ToolContributionSource::new("plugin:first@1.0.0"),
                    Arc::new(MockTool::new("reverse", "Unique")),
                ),
                ToolContribution::new(
                    ToolContributionSource::new("plugin:second@2.0.0"),
                    Arc::new(MockTool::new("echo", "Duplicate")),
                ),
            ])
            .unwrap_err();

        assert_eq!(error.tool_name, "echo");
        assert_eq!(error.existing_source.as_str(), "plugin:first@1.0.0");
        assert_eq!(error.incoming_source.as_str(), "plugin:second@2.0.0");
        assert!(registry.list().is_empty());
    }

'''
text = replace_once(text, marker, tests + marker, "batch registration tests")
path.write_text(text)
