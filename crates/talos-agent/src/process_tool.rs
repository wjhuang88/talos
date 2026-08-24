use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};

use crate::background_jobs::{BackgroundJobSupervisor, ProcessAction};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ProcessActionInput {
    Read,
    Status,
    List,
    Cancel,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ProcessInput {
    action: ProcessActionInput,
    #[serde(default)]
    job_id: Option<String>,
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    max_bytes: Option<usize>,
    #[serde(default)]
    wait_ms: Option<u64>,
}

/// Model-visible control surface for live, session-owned background jobs.
pub(crate) struct ProcessTool {
    supervisor: BackgroundJobSupervisor,
}

impl ProcessTool {
    pub(crate) fn new(supervisor: BackgroundJobSupervisor) -> Self {
        Self { supervisor }
    }
}

#[async_trait]
impl AgentTool for ProcessTool {
    fn name(&self) -> &str {
        "process"
    }

    fn description(&self) -> &str {
        "Inspect or cancel a Talos-supervised background command owned by this session. Use process(read) with the returned byte cursor, bounded wait_ms, and do not busy-poll; cancel jobs when no longer needed."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ProcessInput))
            .unwrap_or_else(|_| serde_json::json!({}))
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        if input.get("action").and_then(Value::as_str) == Some("cancel") {
            return input
                .get("job_id")
                .and_then(Value::as_str)
                .and_then(|id| self.supervisor.permission_resource(id))
                .map_or_else(
                    || {
                        vec![ToolPermissionFacet::with_resource(
                            ToolNature::Execute,
                            format!(
                                "background:unknown:{}",
                                input
                                    .get("job_id")
                                    .and_then(Value::as_str)
                                    .unwrap_or("missing")
                            ),
                            ToolResourceKind::Command,
                        )]
                    },
                    |resource| {
                        vec![
                            ToolPermissionFacet::with_resource(
                                ToolNature::Execute,
                                resource,
                                ToolResourceKind::Command,
                            )
                            .with_description("cancel session-owned background job"),
                        ]
                    },
                );
        }
        vec![
            ToolPermissionFacet::new(ToolNature::Read)
                .with_description("session-owned background job control"),
        ]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn is_always_on(&self) -> bool {
        true
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed: ProcessInput = match serde_json::from_value(input) {
            Ok(parsed) => parsed,
            Err(error) => return ToolResult::error(format!("invalid process input: {error}")),
        };
        let action = match parsed.action {
            ProcessActionInput::Read => ProcessAction::Read,
            ProcessActionInput::Status => ProcessAction::Status,
            ProcessActionInput::List => ProcessAction::List,
            ProcessActionInput::Cancel => ProcessAction::Cancel,
        };
        self.supervisor
            .process_action(
                action,
                parsed.job_id.as_deref(),
                parsed.cursor,
                parsed.max_bytes,
                parsed.wait_ms,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use talos_core::background_job::{BackgroundJobHost, BackgroundJobRequest};
    use tokio::sync::mpsc;

    #[test]
    fn process_schema_requires_action_and_has_bounded_controls() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let tool = ProcessTool::new(supervisor);
        let schema = tool.parameters();
        assert_eq!(schema["required"][0], "action");
        assert!(schema["properties"]["max_bytes"].is_object());
        assert!(schema["properties"]["wait_ms"].is_object());
    }

    #[test]
    fn cancel_uses_execute_facet_and_reads_use_read_facet() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let tool = ProcessTool::new(supervisor);
        assert_eq!(
            tool.permission_profile(&serde_json::json!({"action": "cancel"}))[0].nature,
            ToolNature::Execute
        );
        assert_eq!(
            tool.permission_profile(&serde_json::json!({"action": "read"}))[0].nature,
            ToolNature::Read
        );
    }

    #[tokio::test]
    async fn cancel_uses_the_exact_jobs_background_command_resource() {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        let supervisor = BackgroundJobSupervisor::new(event_tx, "session".to_owned(), 1);
        let _permit1 = supervisor
            .reserve_with_permission_resource(
                BackgroundJobRequest {
                    tool_name: "bash".to_owned(),
                    timeout: Duration::from_secs(5),
                },
                Some("background:bash:exact:test".to_owned()),
            )
            .await
            .unwrap();
        let _permit2 = supervisor
            .reserve_with_permission_resource(
                BackgroundJobRequest {
                    tool_name: "bash".to_owned(),
                    timeout: Duration::from_secs(5),
                },
                Some("background:bash:exact:test".to_owned()),
            )
            .await
            .unwrap();
        let job_ids = supervisor.job_ids();
        assert_eq!(job_ids.len(), 2);
        let tool = ProcessTool::new(supervisor);
        let resources = job_ids
            .iter()
            .map(|job_id| {
                let facet = tool
                    .permission_profile(&serde_json::json!({
                        "action": "cancel",
                        "job_id": job_id,
                    }))
                    .remove(0);
                assert_eq!(facet.nature, ToolNature::Execute);
                assert_eq!(facet.resource_kind, Some(ToolResourceKind::Command));
                facet.resource.unwrap()
            })
            .collect::<Vec<_>>();
        assert_ne!(resources[0], resources[1]);
        assert!(
            resources
                .iter()
                .all(|resource| resource.starts_with("background:bash:exact:test:job:"))
        );
    }
}
