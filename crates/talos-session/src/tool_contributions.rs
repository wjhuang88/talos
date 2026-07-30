use std::path::Path;
use std::sync::Arc;

use talos_core::tool::{AgentTool, ToolContribution, ToolContributionSource};
use uuid::Uuid;

use crate::{
    TodoAddDependencyTool, TodoCreateBatchTool, TodoCreateTool, TodoDeleteTool, TodoQueryTool,
    TodoRemoveDependencyTool, TodoUpdateBatchTool, TodoUpdateStatusTool, TodoUpdateTool,
};

const TODO_CONTRIBUTION_SOURCE: &str = "talos-session:todo";

fn todo_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    ToolContribution::new(ToolContributionSource::new(TODO_CONTRIBUTION_SOURCE), tool)
}

/// Builds the complete session-bound todo tool group for an explicit sessions directory.
///
/// The outer product composition root remains responsible for selecting this group,
/// applying permission wrappers, and registering it into the executable registry.
pub fn todo_tool_contributions_for_sessions_dir(
    sessions_dir: &Path,
    session_id: Uuid,
) -> Vec<ToolContribution> {
    vec![
        todo_contribution(Arc::new(TodoCreateTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoCreateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoUpdateStatusTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoUpdateTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoUpdateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoDeleteTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoAddDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoRemoveDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
        todo_contribution(Arc::new(TodoQueryTool::from_sessions_dir(
            sessions_dir,
            session_id,
        ))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_contribution_group_has_stable_names_order_and_source() {
        let contributions = todo_tool_contributions_for_sessions_dir(
            Path::new("/tmp/talos-session-contributions"),
            Uuid::nil(),
        );

        let names = contributions
            .iter()
            .map(ToolContribution::name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "todo_create",
                "todo_create_batch",
                "todo_update_status",
                "todo_update",
                "todo_update_batch",
                "todo_delete",
                "todo_add_dependency",
                "todo_remove_dependency",
                "todo_query",
            ]
        );
        assert!(
            contributions
                .iter()
                .all(|contribution| contribution.source().as_str() == TODO_CONTRIBUTION_SOURCE)
        );
    }
}
