//! Shared built-in tool contribution selection for product and runtime adapters.
//!
//! This module owns capability selection and construction inputs only. Product
//! adapters remain responsible for permission wrappers, scheduler/todo/plugin
//! additions, presentation policy, and lifecycle-specific behavior.

use std::path::PathBuf;

use talos_core::tool::SharedAtomicCreateCapability;
use talos_core::tool::ToolContribution;
use talos_tools::{
    git_mutation_tool_contributions, git_read_tool_contributions, network_tool_contributions,
    ordinary_file_tool_contributions, read_image_tool_contribution, shell_tool_contributions,
    symbol_tool_contributions, workspace_tool_contributions,
};

/// Explicit consumer profile for the shared built-in contribution inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedToolProfile {
    /// Print, inline, and TUI-style product composition with snapshot-aware files.
    Product,
    /// MCP composition, which preserves ordinary file constructors and omits image input.
    Mcp,
    /// Explicit embedded runtime composition with the product capability inventory.
    Runtime,
}

/// Contribution groups selected by one shared profile.
///
/// Fields are public so the CLI adapter can retain its existing per-group
/// wrapper policy while using the same construction and selection logic.
pub struct SharedToolContributions {
    /// Shell and direct command tools.
    pub shell: Vec<ToolContribution>,
    /// File tools, either snapshot-aware or ordinary for MCP.
    pub files: Vec<ToolContribution>,
    /// Workspace search and inspection tools.
    pub workspace: Vec<ToolContribution>,
    /// Network and web tools.
    pub network: Vec<ToolContribution>,
    /// Optional image input contribution.
    pub image: Option<ToolContribution>,
    /// Code-intelligence tools.
    pub symbols: Vec<ToolContribution>,
    /// Read-only Git tools.
    pub git_read: Vec<ToolContribution>,
    /// Mutating Git tools.
    pub git_mutation: Vec<ToolContribution>,
}

/// Selects all shared contribution groups for one explicit profile.
#[must_use]
pub fn contribution_groups(
    profile: SharedToolProfile,
    workspace_root: PathBuf,
) -> SharedToolContributions {
    contribution_groups_with_capability(profile, workspace_root, None)
}

/// Selects shared contributions with an optional directory capability for new-file creation.
#[cfg(feature = "shared-composition")]
pub fn contribution_groups_with_capability(
    profile: SharedToolProfile,
    workspace_root: PathBuf,
    atomic_create: Option<SharedAtomicCreateCapability>,
) -> SharedToolContributions {
    let image = (profile != SharedToolProfile::Mcp)
        .then(|| read_image_tool_contribution(workspace_root.clone()));
    SharedToolContributions {
        shell: shell_tool_contributions(workspace_root.clone()),
        files: if profile == SharedToolProfile::Mcp {
            ordinary_file_tool_contributions(workspace_root.clone())
        } else {
            talos_tools::snapshot_aware_file_tool_contributions_with_capability(
                workspace_root.clone(),
                atomic_create,
            )
        },
        workspace: workspace_tool_contributions(workspace_root.clone()),
        network: network_tool_contributions(),
        image,
        symbols: symbol_tool_contributions(workspace_root.clone()),
        git_read: git_read_tool_contributions(workspace_root.clone()),
        git_mutation: git_mutation_tool_contributions(workspace_root),
    }
}

/// Builds the contribution groups shared by CLI and runtime adapters.
///
/// The returned tools are not permission-wrapped. Callers must apply their
/// mode-specific wrapper before registering the contributions. The function is
/// intentionally explicit: it is not used by `RuntimeBuilder::new()`.
#[must_use]
pub fn tool_contributions(
    profile: SharedToolProfile,
    workspace_root: PathBuf,
) -> Vec<ToolContribution> {
    let groups = contribution_groups(profile, workspace_root);
    let mut contributions = groups.shell;
    contributions.extend(groups.files);
    contributions.extend(groups.workspace);
    contributions.extend(groups.network);
    if let Some(image) = groups.image {
        contributions.push(image);
    }
    contributions.extend(groups.symbols);
    contributions.extend(groups.git_read);
    contributions.extend(groups.git_mutation);
    contributions
}

/// Builds shared contributions with an optional atomic-create capability.
#[cfg(feature = "shared-composition")]
#[must_use]
pub fn tool_contributions_with_capability(
    profile: SharedToolProfile,
    workspace_root: PathBuf,
    atomic_create: Option<SharedAtomicCreateCapability>,
) -> Vec<ToolContribution> {
    let groups = contribution_groups_with_capability(profile, workspace_root, atomic_create);
    let mut contributions = groups.shell;
    contributions.extend(groups.files);
    contributions.extend(groups.workspace);
    contributions.extend(groups.network);
    if let Some(image) = groups.image {
        contributions.push(image);
    }
    contributions.extend(groups.symbols);
    contributions.extend(groups.git_read);
    contributions.extend(groups.git_mutation);
    contributions
}

/// Builds the explicit full capability inventory for an embedded runtime.
#[must_use]
pub fn runtime_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    tool_contributions(SharedToolProfile::Runtime, workspace_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(profile: SharedToolProfile) -> Vec<String> {
        tool_contributions(profile, PathBuf::from("workspace"))
            .into_iter()
            .map(|contribution| contribution.name().to_string())
            .collect()
    }

    #[test]
    fn product_and_runtime_profiles_have_the_same_builtin_inventory() {
        assert_eq!(
            names(SharedToolProfile::Product),
            names(SharedToolProfile::Runtime)
        );
    }

    #[test]
    fn mcp_profile_omits_image_and_uses_ordinary_file_group() {
        let mcp = names(SharedToolProfile::Mcp);
        assert!(!mcp.iter().any(|name| name == "read_image"));
        assert!(mcp.iter().any(|name| name == "read"));
        assert!(mcp.iter().any(|name| name == "write"));
        assert!(mcp.iter().any(|name| name == "document_extract"));
    }
}
