use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde_json::{Value, json};
use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolBackendDisclosure, ToolContinuation,
    ToolContribution, ToolContributionSource, ToolError, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolNature, ToolPermissionFacet, ToolPresentationPolicy,
    ToolProtocol, ToolProtocolConfig, ToolProvenance, ToolRegistrationError, ToolRegistry,
    ToolResourceKind, ToolResult, ToolResultProjection,
};

#[derive(JsonSchema)]
#[allow(dead_code)]
struct ProbeParameters {
    value: String,
}

struct ProbeTool;

#[async_trait]
impl AgentTool for ProbeTool {
    fn name(&self) -> &str {
        "probe"
    }

    fn description(&self) -> &str {
        "public path probe"
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(ProbeParameters)
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("ok")
    }
}

#[tokio::test]
async fn downstream_tool_facade_paths_and_trait_defaults_compile() {
    let tool = ProbeTool;
    let object: &dyn AgentTool = &tool;
    let input = json!({ "value": "input" });
    let authorization = ToolExecutionAuthorization::for_path(
        "probe",
        ToolNature::Read,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        "src/tool.rs",
        ToolAuthorizationScope::Once,
    )
    .expect("create path authorization");

    assert!(authorization.authorizes_path(
        "probe",
        ToolNature::Read,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        "src/tool.rs"
    ));
    let _: &Path = authorization.normalized_path();
    let _: ToolAuthorizationScope = authorization.scope();
    let _: ToolResult = object.execute_authorized(input.clone(), &[]).await;
    let _: ToolExecutionOutput = object
        .execute_authorized_with_output(input.clone(), &[])
        .await;
    let _: ToolExecutionOutput = object.execute_with_output(input.clone()).await;
    let _: Value = object.project_input(&input);
    let _: ToolResultProjection = object.project_result(&ToolResult::success("ok"));
    let _: bool = object.is_read_only();
    let _: ToolNature = object.nature();
    let _: ToolFamily = object.family();
    let _: bool = object.is_always_on();
    let _: Vec<ToolBackend> = object.conditional_backends();
    let _: Option<String> = object.backend_for_input(&input);
    let backends = HashSet::new();
    let _: String = object.description_for_backends(&backends);
    let _: Value = object.parameters_for_backends(&backends);
    let _: Vec<ToolPermissionFacet> = object.permission_profile(&input);
    let _: &'static [&'static str] = object.summary_fields();
    let _: ToolProvenance = object.provenance();
}

#[test]
fn downstream_tool_facade_data_registry_protocol_and_macro_paths_compile() {
    let continuation = ToolContinuation::disclose_backend("probe", "native", "needed")
        .with_permission_preview("read probe input");
    let _ = ToolContinuation::disclose_tool("probe", "needed").is_tool_disclosure();
    let result = ToolResult::success("ok").with_continuation(continuation);
    let _ = ToolResult::error("error");
    let output = ToolExecutionOutput::from_result(result.clone());
    let _ = ToolExecutionOutput::success("ok");
    let _ = ToolExecutionOutput::error("error");
    let projection = ToolResultProjection::shared("ok");

    let _ = (
        output.result,
        output.next_provider_parts,
        projection.model_content,
        projection.display_content,
        projection.persistence_content,
    );

    let backend = ToolBackend::new("native", "native backend");
    let disclosure = ToolBackendDisclosure::new("probe", "native");
    let policy = ToolPresentationPolicy::always_on()
        .disclose_tool("probe")
        .disclose_backend("probe", "native");
    let _ = ToolPresentationPolicy::full();
    let _ = ToolPresentationPolicy::runtime_default();
    let _ = ToolPresentationPolicy::with_families([ToolFamily::Extension]);
    let _ = ToolPresentationPolicy::with_backend("probe", "native");
    let _ = ToolPresentationPolicy::with_tool("probe");
    let _ = (
        backend.id,
        backend.description,
        disclosure.tool,
        disclosure.backend,
        policy.include_all,
        policy.include_always_on,
        policy.families,
        policy.tools,
        policy.backends,
    );

    let facet =
        ToolPermissionFacet::with_resource(ToolNature::Read, "src/tool.rs", ToolResourceKind::Path)
            .with_description("probe");
    let _ = ToolPermissionFacet::new(ToolNature::Internal);
    let _ = (
        facet.nature,
        facet.resource,
        facet.resource_kind,
        facet.description,
    );

    let source = ToolContributionSource::new("talos-core:probe");
    let contribution =
        ToolContribution::new(source.clone(), Arc::new(ProbeTool)).map_tool(|tool| tool);
    assert_eq!(contribution.source().as_str(), source.as_str());
    assert_eq!(contribution.name(), contribution.tool().name());

    let mut registry = ToolRegistry::new();
    registry
        .register_contribution(contribution)
        .expect("register contribution");
    registry
        .register_contributions([ToolContribution::new(
            ToolContributionSource::new("talos-core:second"),
            Arc::new(SecondProbeTool),
        )])
        .expect("register contribution batch");
    let _ = registry.get("probe");
    let _ = registry.list();
    registry
        .validate_input("probe", &json!({ "value": "input" }))
        .expect("validate input");
    registry.register(Arc::new(ProbeTool));

    let _ = ToolError::ToolNotFound("missing".to_owned());
    let _ = ToolError::InvalidInput("invalid".to_owned());
    let _ = ToolError::ExecutionError("failed".to_owned());
    let _ = ToolRegistrationError {
        tool_name: "probe".to_owned(),
        existing_source: ToolContributionSource::new("existing"),
        incoming_source: ToolContributionSource::new("incoming"),
    };

    let _ = ToolProvenance::Native;
    let _ = ToolProvenance::McpRemote {
        server: "server".to_owned(),
    };
    let _ = ToolProvenance::Plugin {
        name: "plugin".to_owned(),
        version: "1.0.0".to_owned(),
        carrier: "wasm".to_owned(),
    };

    let protocol = ToolProtocol::parse("talos-strict").expect("parse protocol");
    let config = ToolProtocolConfig::for_protocol(protocol);
    let _ = (
        config.protocol,
        config.strict_prompt,
        config.stream_filter,
        config.schema_validate,
    );
    let _: Value = talos_core::tool_parameters!(ProbeParameters);
}

struct SecondProbeTool;

#[async_trait]
impl AgentTool for SecondProbeTool {
    fn name(&self) -> &str {
        "second_probe"
    }

    fn description(&self) -> &str {
        "second public path probe"
    }

    fn parameters(&self) -> Value {
        json!({ "type": "object" })
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("ok")
    }
}
