use super::*;

struct Tool;
#[async_trait::async_trait]
impl AgentTool for Tool {
    fn name(&self) -> &str {
        "write"
    }
    fn description(&self) -> &str {
        "fixture"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type": "object"})
    }
    async fn execute(&self, _: Value) -> super::super::ToolResult {
        super::super::ToolResult::success("fixture")
    }
}
fn contribution() -> ToolContribution {
    ToolContribution::new(
        ToolContributionSource::new("talos-tools:file"),
        Arc::new(Tool),
    )
}

#[test]
fn native_provenance_name_and_source_do_not_establish_builtin_ownership() {
    let mut registry = ToolRegistry::new();
    registry
        .register_contribution(contribution())
        .expect("register");
    assert!(!registry.is_builtin_instance("write"));
    assert_eq!(
        registry.get("write").expect("tool").provenance(),
        super::super::ToolProvenance::Native
    );
}

#[test]
fn replacement_revokes_explicit_builtin_ownership() {
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions([contribution()])
        .expect("builtin");
    assert!(registry.is_builtin_instance("write"));
    registry.register(Arc::new(Tool));
    assert!(!registry.is_builtin_instance("write"));
}

#[test]
fn failed_builtin_batch_never_promotes_existing_or_partial_entries() {
    let mut registry = ToolRegistry::new();
    registry
        .register_contribution(contribution())
        .expect("external");
    assert!(
        registry
            .register_builtin_contributions([contribution()])
            .is_err()
    );
    assert!(!registry.is_builtin_instance("write"));
    let mut empty = ToolRegistry::new();
    assert!(
        empty
            .register_builtin_contributions([contribution(), contribution()])
            .is_err()
    );
    assert!(empty.get("write").is_none());
    assert!(!empty.is_builtin_instance("write"));
}
