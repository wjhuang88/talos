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
    workspace_tool_contributions,
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
    contribution_groups_with_language_provider(profile, workspace_root, atomic_create, None)
}

/// Build contributions with one shared language provider for symbol consumers.
#[cfg(feature = "shared-composition")]
pub fn contribution_groups_with_language_provider(
    profile: SharedToolProfile,
    workspace_root: PathBuf,
    atomic_create: Option<SharedAtomicCreateCapability>,
    language_provider: Option<talos_text::SharedLanguageProvider>,
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
        symbols: talos_tools::symbol_tool_contributions_with_provider(
            workspace_root.clone(),
            language_provider,
        ),
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

    const DIRECT_ROUTE_CHILD: &str = "TALOS_SEARCH_DIRECT_ROUTE_CHILD";
    const PROXY_ENVIRONMENT: [&str; 6] = [
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ];

    fn rerun_without_proxy(filter: &str) -> bool {
        if std::env::var_os(DIRECT_ROUTE_CHILD).is_some() {
            return false;
        }
        let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
        command
            .arg(filter)
            .arg("--nocapture")
            .env(DIRECT_ROUTE_CHILD, "1");
        for name in PROXY_ENVIRONMENT {
            command.env_remove(name);
        }
        let output = command.output().expect("isolated direct-route child");
        assert!(
            output.status.success(),
            "direct-route child failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        true
    }

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
    fn shared_profiles_reject_uninventoried_native_operations() {
        // Coverage inventory, never an admission allowlist. Equality between two
        // profiles alone would miss an unreviewed addition shared by both.
        let shared = [
            "exec",
            "read",
            "write",
            "edit",
            "ls",
            "delete",
            "document_extract",
            "grep",
            "glob",
            "diff",
            "stat",
            "tree",
            "save_url",
            "fetch_url",
            "http_request",
            "web_search",
            "find_symbol",
            "find_references",
            "list_symbols",
            "list_imports",
            "git_status",
            "git_diff",
            "git_log",
            "git_show",
            "git_branch_list",
            "git_add",
            "git_commit",
            "git_push",
            "git_pull",
            "git_checkout",
        ];
        for profile in [
            SharedToolProfile::Product,
            SharedToolProfile::Runtime,
            SharedToolProfile::Mcp,
        ] {
            let mut expected: std::collections::BTreeSet<_> = shared.into_iter().collect();
            expected.insert(if cfg!(windows) { "powershell" } else { "bash" });
            if profile != SharedToolProfile::Mcp {
                expected.insert("read_image");
            }
            let actual = names(profile);
            let unique: std::collections::BTreeSet<_> = actual.iter().map(String::as_str).collect();
            assert_eq!(
                actual.len(),
                unique.len(),
                "duplicate operation in {profile:?}"
            );
            assert_eq!(
                expected, unique,
                "{profile:?}: update I308 disposition for inventory changes"
            );
        }
    }

    #[test]
    fn mcp_profile_omits_image_and_uses_ordinary_file_group() {
        let mcp = names(SharedToolProfile::Mcp);
        assert!(!mcp.iter().any(|name| name == "read_image"));
        assert!(mcp.iter().any(|name| name == "read"));
        assert!(mcp.iter().any(|name| name == "write"));
        assert!(mcp.iter().any(|name| name == "document_extract"));
    }

    #[test]
    fn every_shared_product_root_receives_prepared_web_search() {
        if rerun_without_proxy("every_shared_product_root_receives_prepared_web_search") {
            return;
        }
        for name in PROXY_ENVIRONMENT {
            assert!(std::env::var_os(name).is_none(), "{name} must be isolated");
        }
        for profile in [
            SharedToolProfile::Product,
            SharedToolProfile::Mcp,
            SharedToolProfile::Runtime,
        ] {
            let groups = contribution_groups(profile, PathBuf::from("workspace"));
            let search = groups
                .network
                .iter()
                .find(|contribution| contribution.name() == "web_search")
                .expect("web_search contribution");
            assert!(
                search.tool().requires_network_invocation(),
                "{profile:?} must receive the ADR-090 protected route"
            );
            let prepared = search
                .tool()
                .prepare_network_invocation(&serde_json::json!({
                    "query": "shared root evidence",
                    "max_results": 1,
                    "include_snippets": false
                }))
                .expect("protected preparation")
                .expect("prepared invocation");
            assert_eq!(prepared.manifest().scope().endpoints().len(), 9);
        }
    }
}
