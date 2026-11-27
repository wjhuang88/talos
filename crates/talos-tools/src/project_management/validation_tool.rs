//! Model-facing validation of project governance metadata.

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_conversation::{
    EvidenceStatus, ValidationHostExecutor, ValidationProfile, agile_summary_available,
    collect_validation_plan, render_json_evidence, render_json_plan, run_governance_validation,
};
use talos_core::tool::{
    AgentTool, ToolCleanupOwner, ToolContribution, ToolContributionSource, ToolNature, ToolResult,
};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Input {
    mode: Mode,
    #[serde(default, rename = "profile")]
    _profile: Governance,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Plan,
    Run,
}

#[derive(Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Governance {
    #[default]
    Governance,
}

/// Creates independent invocation scopes under one retained shutdown owner.
/// Implementors must not share invocation cancellation state across calls.
pub trait ValidationHostFactory: ToolCleanupOwner {
    /// Create an executor whose operation cleanup excludes unrelated invocations.
    /// Construction must not start checks. Prepared execution requires a real
    /// operation-local `ValidationHostExecutor::cleanup_receipt`; None refuses
    /// execution and reports unconfirmed cleanup. Factory and receipt methods
    /// must be prompt and nonblocking; the host retains aggregate ownership.
    fn invocation(&self) -> Box<dyn ValidationHostExecutor>;
}

/// Construct the governance validator. The executor parameter preserves the existing host API.
pub fn validation_tool_contributions<T: ValidationHostFactory + 'static>(
    workspace: PathBuf,
    _executor: Arc<T>,
) -> Vec<ToolContribution> {
    governance_validation_contributions(workspace)
}

pub(super) fn governance_validation_contributions(workspace: PathBuf) -> Vec<ToolContribution> {
    if !agile_summary_available(&workspace) {
        return Vec::new();
    }
    vec![ToolContribution::new(
        ToolContributionSource::new("builtin.project-management"),
        Arc::new(ValidationTool { workspace }),
    )]
}

struct ValidationTool {
    workspace: PathBuf,
}

#[async_trait]
impl AgentTool for ValidationTool {
    fn name(&self) -> &str {
        "validate_project"
    }
    fn description(&self) -> &str {
        "Plan or run the built-in, read-only validation of the selected project's governance metadata."
    }
    fn execution_working_directory(&self) -> Option<PathBuf> {
        Some(self.workspace.clone())
    }
    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(Input))
            .expect("serializable validation input schema")
    }
    async fn execute(&self, input: Value) -> ToolResult {
        let Ok(input) = serde_json::from_value::<Input>(input) else {
            return ToolResult::error(
                "validate_project accepts mode plan/run and optional profile governance",
            );
        };
        if !agile_summary_available(&self.workspace) {
            return ToolResult::error("governance metadata unavailable");
        }
        if matches!(input.mode, Mode::Plan) {
            return ToolResult::success(render_json_plan(&collect_validation_plan(
                &self.workspace,
                ValidationProfile::Governance,
            )));
        }
        let evidence = run_governance_validation(&self.workspace);
        let failed = evidence
            .findings
            .iter()
            .any(|finding| finding.severity == talos_conversation::FindingSeverity::Blocked)
            || evidence
                .records
                .iter()
                .any(|record| record.required && record.status != EvidenceStatus::Passed);
        let content = render_json_evidence(&evidence);
        if failed {
            ToolResult::error(content)
        } else {
            ToolResult::success(content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_governance_profiles() {
        for mode in ["run", "plan"] {
            for profile in ["workspace", "i076", "cargo"] {
                assert!(
                    serde_json::from_value::<Input>(
                        serde_json::json!({ "mode": mode, "profile": profile })
                    )
                    .is_err()
                );
            }
            for value in [
                serde_json::json!({"mode":mode}),
                serde_json::json!({"mode":mode,"profile":"governance"}),
            ] {
                assert!(serde_json::from_value::<Input>(value).is_ok());
            }
        }
    }
}
