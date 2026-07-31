use std::path::PathBuf;
use std::sync::Arc;

use talos_core::tool::{AgentTool, ToolContribution, ToolContributionSource};

use crate::ReadImageTool;
use crate::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use crate::symbol::{FindReferencesTool, FindSymbolTool, ListImportsTool, ListSymbolsTool};

const GIT_CONTRIBUTION_SOURCE: &str = "talos-tools:git";
const IMAGE_CONTRIBUTION_SOURCE: &str = "talos-tools:image";
const SYMBOL_CONTRIBUTION_SOURCE: &str = "talos-tools:symbol";

fn contribution(source: &'static str, tool: Arc<dyn AgentTool>) -> ToolContribution {
    ToolContribution::new(ToolContributionSource::new(source), tool)
}

fn git_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(GIT_CONTRIBUTION_SOURCE, tool)
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

/// Builds the single authoritative `read_image` contribution for one explicit
/// workspace root.
///
/// Contribution ownership is independent of model capability. The outer
/// product/agent composition remains responsible for permission wrapping and
/// the existing Supported-model presentation gate.
#[must_use]
pub fn read_image_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    contribution(
        IMAGE_CONTRIBUTION_SOURCE,
        Arc::new(ReadImageTool::new(workspace_root)),
    )
}

/// Builds the complete code-intelligence tool group for one explicit workspace root.
///
/// These tools are contributed without product wrappers. The outer composition
/// root preserves the existing print, TUI, and MCP wrapper policy.
#[must_use]
pub fn symbol_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        contribution(
            SYMBOL_CONTRIBUTION_SOURCE,
            Arc::new(FindSymbolTool::new(workspace_root.clone())),
        ),
        contribution(
            SYMBOL_CONTRIBUTION_SOURCE,
            Arc::new(FindReferencesTool::new(workspace_root.clone())),
        ),
        contribution(
            SYMBOL_CONTRIBUTION_SOURCE,
            Arc::new(ListSymbolsTool::new(workspace_root.clone())),
        ),
        contribution(
            SYMBOL_CONTRIBUTION_SOURCE,
            Arc::new(ListImportsTool::new(workspace_root)),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::ToolFamily;

    fn names(contributions: &[ToolContribution]) -> Vec<&str> {
        contributions.iter().map(ToolContribution::name).collect()
    }

    fn assert_source(contributions: &[ToolContribution], expected: &str) {
        assert!(
            contributions
                .iter()
                .all(|contribution| contribution.source().as_str() == expected)
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
        assert_source(&contributions, GIT_CONTRIBUTION_SOURCE);
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
        assert_source(&contributions, GIT_CONTRIBUTION_SOURCE);
        assert!(
            contributions
                .iter()
                .all(|contribution| !contribution.tool().is_read_only())
        );
    }

    #[test]
    fn read_image_has_one_stable_authoritative_contribution() {
        let contribution = read_image_tool_contribution(PathBuf::from("workspace"));

        assert_eq!(contribution.name(), "read_image");
        assert_eq!(contribution.source().as_str(), IMAGE_CONTRIBUTION_SOURCE);
        assert!(contribution.tool().is_read_only());
    }

    #[test]
    fn symbol_group_has_stable_inventory_source_and_family() {
        let contributions = symbol_tool_contributions(PathBuf::from("workspace"));

        assert_eq!(
            names(&contributions),
            [
                "find_symbol",
                "find_references",
                "list_symbols",
                "list_imports",
            ]
        );
        assert_source(&contributions, SYMBOL_CONTRIBUTION_SOURCE);
        assert!(contributions.iter().all(|contribution| {
            contribution.tool().is_read_only()
                && contribution.tool().family() == ToolFamily::CodeIntelligence
        }));
    }
}
