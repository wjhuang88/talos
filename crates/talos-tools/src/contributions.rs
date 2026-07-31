use std::path::PathBuf;
use std::sync::Arc;

use talos_core::tool::{AgentTool, ToolContribution, ToolContributionSource};

use crate::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};

const GIT_CONTRIBUTION_SOURCE: &str = "talos-tools:git";

fn git_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    ToolContribution::new(ToolContributionSource::new(GIT_CONTRIBUTION_SOURCE), tool)
}

/// Builds the read-only Git tool group for one explicit workspace root.
///
/// The outer product composition root remains responsible for selecting this
/// group, applying any product-specific wrappers, and registering it.
#[must_use]
pub fn git_read_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        git_contribution(Arc::new(GitStatusTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitDiffTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitLogTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitShowTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitBranchListTool::new(workspace_root))),
    ]
}

/// Builds the mutating Git tool group for one explicit workspace root.
///
/// Permission policy is deliberately not applied here. Print, TUI, MCP, or
/// another outer product root decides whether and how to wrap these tools.
#[must_use]
pub fn git_mutation_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        git_contribution(Arc::new(GitAddTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitCommitTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitPushTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitPullTool::new(workspace_root.clone()))),
        git_contribution(Arc::new(GitCheckoutTool::new(workspace_root))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(contributions: &[ToolContribution]) -> Vec<&str> {
        contributions.iter().map(ToolContribution::name).collect()
    }

    fn assert_git_source(contributions: &[ToolContribution]) {
        assert!(
            contributions
                .iter()
                .all(|contribution| contribution.source().as_str() == GIT_CONTRIBUTION_SOURCE)
        );
    }

    #[test]
    fn git_read_group_has_stable_inventory_and_source() {
        let contributions = git_read_tool_contributions(PathBuf::from("workspace"));

        assert_eq!(
            names(&contributions),
            [
                "git_status",
                "git_diff",
                "git_log",
                "git_show",
                "git_branch_list",
            ]
        );
        assert_git_source(&contributions);
        assert!(
            contributions
                .iter()
                .all(|contribution| contribution.tool().is_read_only())
        );
    }

    #[test]
    fn git_mutation_group_has_stable_inventory_and_source() {
        let contributions = git_mutation_tool_contributions(PathBuf::from("workspace"));

        assert_eq!(
            names(&contributions),
            [
                "git_add",
                "git_commit",
                "git_push",
                "git_pull",
                "git_checkout",
            ]
        );
        assert_git_source(&contributions);
        assert!(
            contributions
                .iter()
                .all(|contribution| !contribution.tool().is_read_only())
        );
    }
}
