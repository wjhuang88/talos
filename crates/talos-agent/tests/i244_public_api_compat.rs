use async_trait::async_trait;
use talos_agent::auto_resolver::{
    AUTO_EVALUATOR_SCHEMA_VERSION, AutoConfidence, AutoContentShape, AutoDecision, AutoOperation,
    AutoPermissionAssessor, AutoPermissionRequest, AutoPermissionResponse, AutoProvenance,
    AutoReasonCode, AutoRiskClass,
};
use talos_agent::permission_pipeline::{
    PermissionApprovalRequest, PermissionAuthorizationRequest, PermissionBinding,
};
use talos_core::tool::{ToolPermissionFacet, ToolProvenance};
use talos_permission::{GrantPreview, InteractionCapability, PermissionMode};

struct LegacyAssessor;

#[async_trait]
impl AutoPermissionAssessor for LegacyAssessor {
    async fn assess(
        &self,
        request: AutoPermissionRequest,
        _remaining: std::time::Duration,
    ) -> Result<String, String> {
        Ok(request.request_digest)
    }
}

#[test]
fn legacy_assessor_request_response_and_closed_enums_remain_source_compatible() {
    let _legacy_assessor: Box<dyn AutoPermissionAssessor> = Box::new(LegacyAssessor);
    let request = AutoPermissionRequest {
        schema_version: AUTO_EVALUATOR_SCHEMA_VERSION,
        tool: "exec".to_owned(),
        provenance: AutoProvenance::Native,
        risk_class: AutoRiskClass::BoundedReadOnlyCommand,
        target_label: "managed_workspace".to_owned(),
        operation: AutoOperation::ExecuteReadOnlyCommand,
        content_shape: AutoContentShape {
            extension: "git".to_owned(),
            bytes: 6,
            lines: 1,
            argument_digest: "sha256:arguments".to_owned(),
        },
        session_binding: "sha256:session".to_owned(),
        revisions: [0; 6],
        mode: PermissionMode::Interactive,
        request_digest: "sha256:request".to_owned(),
    };
    let response = AutoPermissionResponse {
        schema_version: AUTO_EVALUATOR_SCHEMA_VERSION,
        request_digest: request.request_digest.clone(),
        decision: AutoDecision::AllowOnce,
        reason_code: AutoReasonCode::BoundedReadOnlyCommand,
        confidence: AutoConfidence::High,
    };
    let old_approval_literal: fn(GrantPreview) -> PermissionApprovalRequest =
        |preview| PermissionApprovalRequest {
            tool_name: "bash".to_owned(),
            provenance: ToolProvenance::Native,
            arguments: serde_json::json!({"command": "ls -la"}),
            summary_fields: vec!["command".to_owned()],
            preview,
            binding: PermissionBinding {
                session_id: "session".to_owned(),
                revisions: [0; 6],
                mode: PermissionMode::Interactive,
                interaction: InteractionCapability::Available,
            },
        };
    let input = serde_json::json!({"command": "ls -la"});
    let profile: Vec<ToolPermissionFacet> = Vec::new();
    let _old_authorization_literal = PermissionAuthorizationRequest {
        tool_name: "bash",
        provenance: ToolProvenance::Native,
        profile: &profile,
        input: &input,
        presentation_input: input.clone(),
        summary_fields: vec!["command".to_owned()],
        deadline: std::time::Duration::from_secs(1),
    };

    let risk = match request.risk_class {
        AutoRiskClass::BoundedWorkspaceTextCreate => "create",
        AutoRiskClass::BoundedReadOnlyCommand => "read",
        AutoRiskClass::BoundedLocalValidation => "validate",
    };
    let operation = match request.operation {
        AutoOperation::CreateTextFile => "create",
        AutoOperation::ExecuteReadOnlyCommand => "read",
        AutoOperation::ExecuteLocalValidation => "validate",
    };

    assert_eq!(risk, "read");
    assert_eq!(operation, "read");
    assert_eq!(response.request_digest, "sha256:request");
    let _ = old_approval_literal;
}
