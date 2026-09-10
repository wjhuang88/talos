//! Public API conformance: registry selection does not grant tool disclosure.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use talos_core::capability::{Carrier, Provenance};
use talos_core::tool::{AgentTool, ToolRegistry, ToolResult};
use talos_core::{
    CapabilityDescriptor, CapabilityRegistry, CapabilityRequest, ProviderDescriptor,
    ResolutionResult,
};

struct ExistingTool;

#[async_trait]
impl AgentTool for ExistingTool {
    fn name(&self) -> &str {
        "existing"
    }
    fn description(&self) -> &str {
        "Previously registered tool"
    }
    fn parameters(&self) -> Value {
        json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _: Value) -> ToolResult {
        ToolResult::success("existing")
    }
}

#[test]
fn offline_public_registry_does_not_change_tool_or_schema_inventory() {
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(ExistingTool));
    let before: Vec<_> = tools
        .list()
        .iter()
        .map(|tool| (tool.name().to_owned(), tool.parameters()))
        .collect();
    let mut registry = CapabilityRegistry::default();
    let request = CapabilityRequest {
        capability_id: "text.search".into(),
        version: "1.0.0".into(),
    };
    assert_eq!(registry.resolve(&request), ResolutionResult::Unavailable);
    let descriptor = ProviderDescriptor {
        id: "host.search".into(),
        version: "7.0.0".into(),
        provenance: Provenance::BuiltIn,
        carrier: Carrier::BuiltIn,
        metadata: BTreeMap::new(),
        capabilities: vec![CapabilityDescriptor {
            id: request.capability_id.clone(),
            version: "1.2.0".into(),
            name: "Search".into(),
            provenance: Provenance::BuiltIn,
            carrier: Carrier::BuiltIn,
            metadata: BTreeMap::new(),
        }],
    };
    let wire_before = serde_json::to_value(&descriptor).expect("serialize descriptor");
    assert!(registry.register(descriptor).is_ok());
    let ResolutionResult::Available(selected) = registry.resolve(&request) else {
        panic!("registered compatible capability must resolve");
    };
    assert_eq!(
        serde_json::to_value(selected).expect("serialize selected"),
        wire_before
    );
    let after: Vec<_> = tools
        .list()
        .iter()
        .map(|tool| (tool.name().to_owned(), tool.parameters()))
        .collect();
    assert_eq!(before, after);
    assert!(tools.get("text.search").is_none());
    assert!(tools.get("host.search").is_none());
}
