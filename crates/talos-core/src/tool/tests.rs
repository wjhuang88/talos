use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use super::*;
use schemars::JsonSchema;
use serde::Deserialize;

/// Mock tool for testing.
struct MockTool {
    tool_name: String,
    tool_description: String,
    read_only: bool,
    family: ToolFamily,
    always_on: bool,
}

impl MockTool {
    fn new(name: &str, description: &str) -> Self {
        Self {
            tool_name: name.to_owned(),
            tool_description: description.to_owned(),
            read_only: true,
            family: ToolFamily::Extension,
            always_on: false,
        }
    }

    fn with_family(mut self, family: ToolFamily) -> Self {
        self.family = family;
        self
    }

    fn always_on(mut self) -> Self {
        self.always_on = true;
        self
    }
}

#[async_trait]
impl AgentTool for MockTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "A message to echo"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, input: Value) -> ToolResult {
        if let Some(msg) = input.get("message").and_then(Value::as_str) {
            ToolResult::success(format!("echo: {msg}"))
        } else {
            ToolResult::error("missing 'message' field".to_owned())
        }
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn family(&self) -> ToolFamily {
        self.family
    }

    fn is_always_on(&self) -> bool {
        self.always_on
    }
}

/// Mock tool with typed parameters for schema generation testing.
#[derive(JsonSchema, Deserialize)]
#[allow(dead_code)]
struct GreetParams {
    /// The name to greet.
    name: String,
    /// Whether to use formal greeting.
    #[serde(default)]
    formal: bool,
}

#[allow(dead_code)]
struct TypedMockTool;

#[async_trait]
impl AgentTool for TypedMockTool {
    fn name(&self) -> &str {
        "greet"
    }

    fn description(&self) -> &str {
        "Greet someone by name"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(GreetParams)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let name = input.get("name").and_then(Value::as_str).unwrap_or("World");
        ToolResult::success(format!("Hello, {name}!"))
    }
}

#[test]
fn test_register_and_get_tool() {
    let mut registry = ToolRegistry::new();
    let tool = Arc::new(MockTool::new("echo", "Echoes a message"));
    registry.register(tool);

    let retrieved = registry.get("echo");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.expect("operation should succeed").name(), "echo");
}

#[test]
fn test_tool_not_found() {
    let registry = ToolRegistry::new();
    assert!(registry.get("nonexistent").is_none());

    let result = registry.validate_input("nonexistent", &serde_json::json!({}));
    assert!(matches!(result, Err(ToolError::ToolNotFound(_))));
}

#[test]
fn test_list_tools() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));
    registry.register(Arc::new(MockTool::new("reverse", "Reverses a string")));

    let tools = registry.list();
    assert_eq!(tools.len(), 2);
}

#[test]
fn test_validate_input_valid() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

    let input = serde_json::json!({ "message": "hello" });
    assert!(registry.validate_input("echo", &input).is_ok());
}

#[test]
fn test_validate_input_missing_required() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

    let input = serde_json::json!({});
    let result = registry.validate_input("echo", &input);
    assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    assert!(
        result
            .expect_err("operation should fail")
            .to_string()
            .contains("missing required field 'message'")
    );
}

#[test]
fn test_validate_input_not_object() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

    let input = serde_json::json!("not an object");
    let result = registry.validate_input("echo", &input);
    assert!(matches!(result, Err(ToolError::InvalidInput(_))));
}

#[tokio::test]
async fn test_tool_execute() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

    let tool = registry.get("echo").expect("operation should succeed");
    let result = tool
        .execute(serde_json::json!({ "message": "hello" }))
        .await;
    assert!(!result.is_error);
    assert_eq!(result.content, "echo: hello");
}

#[tokio::test]
async fn test_tool_execute_error() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Echoes a message")));

    let tool = registry.get("echo").expect("operation should succeed");
    let result = tool.execute(serde_json::json!({})).await;
    assert!(result.is_error);
}

#[test]
fn test_tool_is_read_only() {
    let tool = MockTool::new("echo", "Echoes a message");
    assert!(tool.is_read_only());
}

#[test]
fn test_tool_parameters_macro() {
    let schema = tool_parameters!(GreetParams);
    assert!(schema.is_object());
    let obj = schema.as_object().expect("operation should succeed");
    assert!(obj.contains_key("properties"));
}

#[test]
fn test_register_replaces_existing() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(MockTool::new("echo", "Original")));
    registry.register(Arc::new(MockTool::new("echo", "Replacement")));

    let tool = registry.get("echo").expect("operation should succeed");
    assert_eq!(tool.description(), "Replacement");
}

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
    registry
        .register_contribution(contribution)
        .expect("operation should succeed");

    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Checked"
    );
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
        .expect("operation should succeed");

    let error = registry
        .register_contribution(ToolContribution::new(
            ToolContributionSource::new("plugin:demo@0.1.0"),
            Arc::new(MockTool::new("echo", "Replacement")),
        ))
        .expect_err("operation should fail");

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
    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Original"
    );
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
        .expect_err("operation should fail");

    assert_eq!(error.existing_source.as_str(), "legacy:unchecked");
    assert_eq!(error.incoming_source.as_str(), "talos-tools:file");
    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Legacy"
    );
}

#[test]
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
        .expect("operation should succeed");

    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Echo"
    );
    assert_eq!(
        registry
            .get("reverse")
            .expect("operation should succeed")
            .description(),
        "Reverse"
    );
    let error = registry
        .register_contribution(ToolContribution::new(
            ToolContributionSource::new("plugin:other@1.0.0"),
            Arc::new(MockTool::new("reverse", "Other")),
        ))
        .expect_err("operation should fail");
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
        .expect("operation should succeed");

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
        .expect_err("operation should fail");

    assert_eq!(error.tool_name, "echo");
    assert_eq!(error.existing_source.as_str(), "talos-tools:file");
    assert_eq!(error.incoming_source.as_str(), "plugin:demo@0.1.0");
    assert!(registry.get("reverse").is_none());
    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Original"
    );
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
        .expect_err("operation should fail");

    assert_eq!(error.tool_name, "echo");
    assert_eq!(error.existing_source.as_str(), "plugin:first@1.0.0");
    assert_eq!(error.incoming_source.as_str(), "plugin:second@2.0.0");
    assert!(registry.list().is_empty());
}

#[test]
fn legacy_register_still_replaces_checked_registration() {
    let mut registry = ToolRegistry::new();
    registry
        .register_contribution(ToolContribution::new(
            ToolContributionSource::new("talos-tools:file"),
            Arc::new(MockTool::new("echo", "Checked")),
        ))
        .expect("operation should succeed");
    registry.register(Arc::new(MockTool::new("echo", "Legacy replacement")));

    assert_eq!(
        registry
            .get("echo")
            .expect("operation should succeed")
            .description(),
        "Legacy replacement"
    );
    let error = registry
        .register_contribution(ToolContribution::new(
            ToolContributionSource::new("plugin:demo@0.1.0"),
            Arc::new(MockTool::new("echo", "Plugin")),
        ))
        .expect_err("operation should fail");
    assert_eq!(error.existing_source.as_str(), "legacy:unchecked");
}

#[test]
fn test_tool_result_helpers() {
    let success = ToolResult::success("ok");
    assert!(!success.is_error);
    assert_eq!(success.content, "ok");
    assert!(success.continuations.is_empty());

    let error = ToolResult::error("failed");
    assert!(error.is_error);
    assert_eq!(error.content, "failed");
}

#[test]
fn test_tool_result_can_carry_continuation() {
    let result = ToolResult::success("needs browser").with_continuation(
        ToolContinuation::disclose_backend("fetch_url", "browser_page", "login_redirect")
            .with_permission_preview("read visible browser page text"),
    );

    assert_eq!(result.continuations.len(), 1);
    assert_eq!(result.continuations[0].tool, "fetch_url");
    assert_eq!(result.continuations[0].backend, "browser_page".to_string());
}

#[test]
fn tool_continuation_can_disclose_tool_without_backend() {
    let result = ToolResult::success("static fetch needs advanced HTTP").with_continuation(
        ToolContinuation::disclose_tool("http_request", "advanced_http_required"),
    );

    assert_eq!(result.continuations[0].tool, "http_request");
    assert_eq!(result.continuations[0].backend, "");
    assert!(result.continuations[0].is_tool_disclosure());
    assert_eq!(result.continuations[0].reason, "advanced_http_required");
}

#[test]
fn test_tool_presentation_policy_selects_always_on_baseline() {
    let baseline = MockTool::new("read", "Read file").always_on();
    let shell = MockTool::new("bash", "Run command").with_family(ToolFamily::Shell);

    let policy = ToolPresentationPolicy::always_on();

    assert!(policy.allows_tool(&baseline));
    assert!(!policy.allows_tool(&shell));
}

#[test]
fn test_tool_presentation_policy_selects_explicit_family() {
    let git = MockTool::new("git_status", "Git status").with_family(ToolFamily::Git);
    let network = MockTool::new("web_search", "Search web").with_family(ToolFamily::Network);

    let policy = ToolPresentationPolicy::with_families([ToolFamily::Git]);

    assert!(policy.allows_tool(&git));
    assert!(!policy.allows_tool(&network));
    assert!(policy.family_set().contains(&ToolFamily::Git));
}

#[test]
fn test_tool_presentation_policy_discloses_backend() {
    let network = MockTool::new("fetch_url", "Fetch URL").with_family(ToolFamily::Network);

    let policy = ToolPresentationPolicy::always_on().disclose_backend("fetch_url", "browser_page");

    assert!(policy.allows_tool(&network));
    assert!(policy.allows_backend("fetch_url", "browser_page"));
    assert!(!policy.allows_backend("fetch_url", "advanced_http"));
    assert!(policy.backend_set_for("fetch_url").contains("browser_page"));
}

#[test]
fn runtime_default_does_not_present_plugin_family() {
    let plugin = MockTool::new("plugin.demo", "Plugin demo").with_family(ToolFamily::Plugin);

    let policy = ToolPresentationPolicy::runtime_default();

    assert!(!policy.allows_tool(&plugin));
    assert!(ToolPresentationPolicy::with_tool("plugin.demo").allows_tool(&plugin));
}

#[test]
fn tool_provenance_plugin_serde_roundtrip() {
    let provenance = ToolProvenance::Plugin {
        name: "my-plugin".to_string(),
        version: "0.1.0".to_string(),
        carrier: "wasm".to_string(),
    };
    let json = serde_json::to_string(&provenance).expect("operation should succeed");
    assert!(json.contains("\"type\":\"plugin\""));
    assert!(json.contains("\"name\":\"my-plugin\""));
    assert!(json.contains("\"carrier\":\"wasm\""));
    let back: ToolProvenance = serde_json::from_str(&json).expect("operation should succeed");
    assert_eq!(provenance, back);
}

#[test]
fn tool_provenance_all_variants_serde_roundtrip() {
    let variants = [
        ToolProvenance::Native,
        ToolProvenance::McpRemote {
            server: "test".to_string(),
        },
        ToolProvenance::Plugin {
            name: "p".to_string(),
            version: "1.0.0".to_string(),
            carrier: "wasm".to_string(),
        },
    ];
    for provenance in variants {
        let json = serde_json::to_string(&provenance).expect("operation should succeed");
        let back: ToolProvenance = serde_json::from_str(&json).expect("operation should succeed");
        assert_eq!(provenance, back);
    }
}
