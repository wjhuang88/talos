#[cfg(any(
    feature = "file-write",
    feature = "shell",
    feature = "git",
    feature = "image",
    feature = "code-intelligence"
))]
use std::path::PathBuf;
use std::sync::Arc;

use talos_core::tool::{AgentTool, ToolContribution, ToolContributionSource};

#[cfg(feature = "document")]
use crate::DocumentExtractTool;
#[cfg(feature = "image")]
use crate::ReadImageTool;
#[cfg(all(feature = "network", feature = "file-write"))]
use crate::SaveUrlTool;
#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
use crate::TreeTool;
#[cfg(all(feature = "file-read", feature = "file-write"))]
use crate::file_tools::{
    DeleteTool, EditTool, LsTool, ReadTool, WriteTool, snapshot_aware_file_tools,
};
#[cfg(feature = "git")]
use crate::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
#[cfg(feature = "code-intelligence")]
use crate::symbol::{FindReferencesTool, FindSymbolTool, ListImportsTool, ListSymbolsTool};
#[cfg(feature = "shell")]
use crate::{BashTool, ExecTool};
#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
use crate::{DiffTool, StatTool};
#[cfg(feature = "network")]
use crate::{FetchUrlTool, HttpRequestTool, WebSearchTool};
#[cfg(feature = "search")]
use crate::{GlobTool, GrepTool};

#[cfg(all(feature = "file-read", feature = "file-write"))]
const FILE_CONTRIBUTION_SOURCE: &str = "talos-tools:file";
#[cfg(feature = "git")]
const GIT_CONTRIBUTION_SOURCE: &str = "talos-tools:git";
#[cfg(feature = "network")]
const NETWORK_CONTRIBUTION_SOURCE: &str = "talos-tools:network";
#[cfg(feature = "shell")]
const SHELL_CONTRIBUTION_SOURCE: &str = "talos-tools:shell";
#[cfg(feature = "image")]
const IMAGE_CONTRIBUTION_SOURCE: &str = "talos-tools:image";
#[cfg(feature = "code-intelligence")]
const SYMBOL_CONTRIBUTION_SOURCE: &str = "talos-tools:symbol";
#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
const WORKSPACE_CONTRIBUTION_SOURCE: &str = "talos-tools:workspace";

fn contribution(source: &'static str, tool: Arc<dyn AgentTool>) -> ToolContribution {
    ToolContribution::new(ToolContributionSource::new(source), tool)
}

#[cfg(all(feature = "file-read", feature = "file-write"))]
fn file_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(FILE_CONTRIBUTION_SOURCE, tool)
}

#[cfg(feature = "git")]
fn git_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(GIT_CONTRIBUTION_SOURCE, tool)
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn workspace_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(WORKSPACE_CONTRIBUTION_SOURCE, tool)
}

/// Builds the single authoritative platform-shell contribution for one explicit workspace root.
///
/// Product composition roots can select the shell escape hatch without constructing the excluded
/// `exec` tool. The contribution is named `bash` on Unix and `powershell` on Windows by the tool.
#[must_use]
#[cfg(feature = "shell")]
pub fn bash_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    contribution(
        SHELL_CONTRIBUTION_SOURCE,
        Arc::new(BashTool::new(workspace_root)),
    )
}

#[cfg(feature = "shell")]
fn exec_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    contribution(
        SHELL_CONTRIBUTION_SOURCE,
        Arc::new(ExecTool::new(workspace_root)),
    )
}

/// Builds the shell/command tool group for one explicit workspace root.
///
/// Permission and sandbox policy remain outer-composition concerns.
#[must_use]
#[cfg(feature = "shell")]
pub fn shell_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        bash_tool_contribution(workspace_root.clone()),
        exec_tool_contribution(workspace_root),
    ]
}

#[cfg(all(
    feature = "file-read",
    feature = "search",
    feature = "git",
    feature = "document"
))]
fn document_extract_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(DocumentExtractTool::new(workspace_root)))
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn grep_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(GrepTool::new(workspace_root)))
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn glob_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(GlobTool::new(workspace_root)))
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn diff_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(DiffTool::new(workspace_root)))
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn stat_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(StatTool::new(workspace_root)))
}

#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
fn tree_tool_contribution(workspace_root: PathBuf) -> ToolContribution {
    workspace_contribution(Arc::new(TreeTool::new(workspace_root)))
}

/// Builds workspace-scoped search and inspection tools without constructing document extraction.
///
/// The legacy interactive profile uses this explicit group because `document_extract` is excluded.
#[must_use]
#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
pub fn workspace_non_document_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        grep_tool_contribution(workspace_root.clone()),
        glob_tool_contribution(workspace_root.clone()),
        diff_tool_contribution(workspace_root.clone()),
        stat_tool_contribution(workspace_root.clone()),
        tree_tool_contribution(workspace_root),
    ]
}

/// Builds workspace-scoped search, inspection, and document tools.
///
/// These instances carry only the explicit workspace root. Product permission
/// wrappers remain at the outer composition root.
#[must_use]
#[cfg(all(
    feature = "file-read",
    feature = "search",
    feature = "git",
    feature = "document"
))]
pub fn workspace_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    let mut contributions = vec![document_extract_tool_contribution(workspace_root.clone())];
    contributions.extend(workspace_non_document_tool_contributions(workspace_root));
    contributions
}

/// Builds the network/web tool group.
///
/// Selection and permission wrapping remain explicit product decisions.
#[must_use]
#[cfg(feature = "network")]
pub fn network_tool_contributions() -> Vec<ToolContribution> {
    let mut contributions = Vec::new();
    #[cfg(feature = "file-write")]
    contributions.push(contribution(
        NETWORK_CONTRIBUTION_SOURCE,
        Arc::new(SaveUrlTool::new()),
    ));
    contributions.extend([
        contribution(NETWORK_CONTRIBUTION_SOURCE, Arc::new(FetchUrlTool::new())),
        contribution(
            NETWORK_CONTRIBUTION_SOURCE,
            Arc::new(HttpRequestTool::new()),
        ),
        contribution(NETWORK_CONTRIBUTION_SOURCE, Arc::new(WebSearchTool::new())),
    ]);
    contributions
}

/// Builds the core file tool group with one shared model-private snapshot registry.
///
/// Print and TUI composition use this variant to preserve snapshot-bound write,
/// edit, and delete behavior. Permission wrappers remain an outer-product policy.
#[must_use]
#[cfg(all(feature = "file-read", feature = "file-write"))]
pub fn snapshot_aware_file_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    let (read, write, edit, delete) = snapshot_aware_file_tools(workspace_root.clone());
    vec![
        file_contribution(Arc::new(read)),
        file_contribution(Arc::new(write)),
        file_contribution(Arc::new(edit)),
        file_contribution(Arc::new(LsTool::new(workspace_root))),
        file_contribution(Arc::new(delete)),
    ]
}

/// Builds the core file tool group with ordinary independent constructors.
///
/// MCP composition uses this variant to preserve its current registry behavior.
#[must_use]
#[cfg(all(feature = "file-read", feature = "file-write"))]
pub fn ordinary_file_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        file_contribution(Arc::new(ReadTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(WriteTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(EditTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(LsTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(DeleteTool::new(workspace_root))),
    ]
}

/// Builds the read-only Git tool group for one explicit workspace root.
///
/// The outer product composition root remains responsible for selecting this
/// group, applying any product-specific wrappers, and registering it.
#[must_use]
#[cfg(feature = "git")]
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
#[cfg(feature = "git")]
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
#[cfg(feature = "image")]
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
#[cfg(feature = "code-intelligence")]
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

#[cfg(all(test, feature = "coding"))]
mod tests {
    use super::*;
    use talos_core::tool::ToolFamily;

    #[cfg(windows)]
    const SHELL_TOOL_NAME: &str = "powershell";
    #[cfg(not(windows))]
    const SHELL_TOOL_NAME: &str = "bash";

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
    fn shell_group_has_stable_inventory_and_source() {
        let contributions = shell_tool_contributions(PathBuf::from("workspace"));

        assert_eq!(names(&contributions), [SHELL_TOOL_NAME, "exec"]);
        assert_source(&contributions, SHELL_CONTRIBUTION_SOURCE);
    }

    #[test]
    fn workspace_group_has_stable_inventory_and_source() {
        let contributions = workspace_tool_contributions(PathBuf::from("workspace"));

        assert_eq!(
            names(&contributions),
            ["document_extract", "grep", "glob", "diff", "stat", "tree"]
        );
        assert_source(&contributions, WORKSPACE_CONTRIBUTION_SOURCE);
    }

    #[test]
    fn selective_groups_exclude_tools_without_constructing_full_groups() {
        let shell = vec![bash_tool_contribution(PathBuf::from("workspace"))];
        assert_eq!(names(&shell), [SHELL_TOOL_NAME]);
        assert_source(&shell, SHELL_CONTRIBUTION_SOURCE);

        let workspace = workspace_non_document_tool_contributions(PathBuf::from("workspace"));
        assert_eq!(names(&workspace), ["grep", "glob", "diff", "stat", "tree"]);
        assert_source(&workspace, WORKSPACE_CONTRIBUTION_SOURCE);
    }

    #[test]
    fn network_group_has_stable_inventory_and_source() {
        let contributions = network_tool_contributions();

        assert_eq!(
            names(&contributions),
            ["save_url", "fetch_url", "http_request", "web_search"]
        );
        assert_source(&contributions, NETWORK_CONTRIBUTION_SOURCE);
    }

    #[test]
    fn file_groups_have_stable_equivalent_inventory_and_source() {
        let snapshot = snapshot_aware_file_tool_contributions(PathBuf::from("workspace"));
        let ordinary = ordinary_file_tool_contributions(PathBuf::from("workspace"));
        let expected = ["read", "write", "edit", "ls", "delete"];

        assert_eq!(names(&snapshot), expected);
        assert_eq!(names(&ordinary), expected);
        assert_source(&snapshot, FILE_CONTRIBUTION_SOURCE);
        assert_source(&ordinary, FILE_CONTRIBUTION_SOURCE);
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
