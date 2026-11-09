//! Explicit project-management contributions; discovery grants no permissions.

mod validation_tool;
pub use validation_tool::{ValidationHostFactory, validation_tool_contributions};
#[cfg(unix)]
mod unix_validation;
#[cfg(unix)]
pub use unix_validation::UnixValidationHost;
#[cfg(windows)]
mod windows_validation;
#[cfg(windows)]
pub use windows_validation::WindowsValidationHost;

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolContribution, ToolContributionSource, ToolNature, ToolResult,
};

/// Construct applicable project tools with platform-owned validation execution.
/// These contributions still require the product's normal permission wrappers.
pub fn project_tool_contributions(workspace: PathBuf) -> Vec<ToolContribution> {
    let mut contributions = agile_tool_contributions(workspace.clone());
    #[cfg(unix)]
    contributions.extend(validation_tool_contributions(
        workspace,
        Arc::new(UnixValidationHost::default()),
    ));
    #[cfg(windows)]
    contributions.extend(validation_tool_contributions(
        workspace,
        Arc::new(WindowsValidationHost::default()),
    ));
    contributions
}

/// Construct the supported read-only agile tool when governance markers qualify.
/// Product composition must still apply its ordinary permission wrappers.
pub fn agile_tool_contributions(workspace: PathBuf) -> Vec<ToolContribution> {
    if !talos_conversation::agile_summary_available(&workspace) {
        return Vec::new();
    }
    vec![ToolContribution::new(
        ToolContributionSource::new("builtin.project-management"),
        Arc::new(AgileSummaryTool { workspace }),
    )]
}

struct AgileSummaryTool {
    workspace: PathBuf,
}

#[async_trait]
impl AgentTool for AgileSummaryTool {
    fn name(&self) -> &str {
        "agile_summary"
    }

    fn description(&self) -> &str {
        "Read the host-selected workspace's existing governance, iteration and backlog summary. Use for requests such as 'report project progress', 'what backlog remains?' or 'check iteration status', or when that status is needed for an accepted task. Do not invoke for ordinary chat, isolated code explanations, or merely because project files exist; do not repeatedly refresh unchanged status without a task need. Small projects and Planned iterations qualify: discovery requires a governance manifest or any two canonical Board/iteration-index/backlog files, not an Active iteration. Missing information is unavailable, not proof of completion. This read-only summary cannot create, edit, activate or close tasks, change Issues, validate code, or release anything. Use validate_project separately for supported validation when available; execution requires its normal permission admission."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({"type":"object","properties":{},"additionalProperties":false})
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }

    async fn execute(&self, input: Value) -> ToolResult {
        if input.as_object().is_none_or(|object| !object.is_empty()) {
            return ToolResult::error("agile_summary accepts an empty object only");
        }
        let workspace = self.workspace.clone();
        match tokio::task::spawn_blocking(move || {
            if !talos_conversation::agile_summary_available(&workspace) {
                return ToolResult::error("agile summary unavailable: governance markers missing");
            }
            ToolResult::success(talos_conversation::format_governance_summary(&workspace))
        })
        .await
        {
            Ok(result) => result,
            Err(_) => ToolResult::error("agile summary worker failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn summary_reuses_service_rejects_overrides_and_rechecks_discovery() {
        let workspace = tempfile::tempdir().expect("workspace");
        assert!(agile_tool_contributions(workspace.path().to_owned()).is_empty());
        let directory = workspace.path().join(".agent-governance");
        std::fs::create_dir(&directory).expect("directory");
        let manifest = directory.join("manifest.yaml");
        std::fs::write(&manifest, "profile: minimal\nstatus: adopting\n").expect("manifest");
        let tools = agile_tool_contributions(workspace.path().to_owned());
        assert_eq!(tools.len(), 1);
        let tool = tools[0].tool();
        assert!(tool.is_read_only());
        for input in [
            Value::Null,
            serde_json::json!({"workspace":"/"}),
            serde_json::json!({"action":"activate"}),
        ] {
            assert!(tool.execute(input).await.is_error);
        }
        let result = tool.execute(serde_json::json!({})).await;
        assert!(!result.is_error);
        assert_eq!(
            result.content,
            talos_conversation::format_governance_summary(workspace.path())
        );
        assert_eq!(
            std::fs::read_to_string(&manifest).expect("unchanged"),
            "profile: minimal\nstatus: adopting\n"
        );
        std::fs::remove_file(manifest).expect("remove marker");
        assert!(tool.execute(serde_json::json!({})).await.is_error);
    }
}
