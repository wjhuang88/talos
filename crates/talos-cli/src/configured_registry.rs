//! Configured CLI composition for protected search backends.
//!
//! This adapter keeps persisted/config parsing above `talos-tools` while reusing the shared
//! runtime contribution inventory. Existing unconfigured registry builders remain the Auto
//! compatibility path for tests and callers that do not consume persisted search intent.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use talos_core::tool::{AgentTool, ToolContribution, ToolContributionSource, ToolRegistry};
use talos_runtime::composition::{
    SharedToolContributions, SharedToolProfile,
    contribution_groups_with_language_provider_and_search, contribution_groups_with_search,
};
use talos_session::{SessionManager, todo_tool_contributions_for_sessions_dir};
use talos_tools::SearchToolConfiguration;
use uuid::Uuid;

use crate::registry::{LoadedLanguageContext, StatusTool};

fn default_todo_tool_contributions(session_id: Uuid) -> Vec<ToolContribution> {
    let Ok(sessions_dir) = SessionManager::default_sessions_dir() else {
        return Vec::new();
    };
    todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id)
}

fn register_builtin(
    registry: &mut ToolRegistry,
    contributions: impl IntoIterator<Item = ToolContribution>,
) -> Result<()> {
    for contribution in contributions {
        registry.register_builtin_contributions([contribution])?;
    }
    Ok(())
}

fn build_product_registry(
    shared: SharedToolContributions,
    scheduler_tools: Vec<Arc<dyn AgentTool>>,
    todo_contributions: Vec<ToolContribution>,
) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::new();
    register_builtin(&mut registry, shared.shell)?;
    register_builtin(&mut registry, shared.files)?;
    register_builtin(
        &mut registry,
        [shared
            .image
            .expect("product profile must include read_image")],
    )?;
    register_builtin(&mut registry, shared.workspace)?;
    register_builtin(&mut registry, shared.network)?;
    register_builtin(&mut registry, shared.symbols)?;
    register_builtin(&mut registry, shared.git_read)?;
    register_builtin(&mut registry, shared.git_mutation)?;
    register_builtin(&mut registry, todo_contributions)?;
    for tool in scheduler_tools {
        registry.register(tool);
    }
    Ok(registry)
}

pub(crate) fn build_print_tool_registry_with_search(
    scheduler_tools: Vec<Arc<dyn AgentTool>>,
    search: SearchToolConfiguration,
) -> Result<ToolRegistry> {
    let shared =
        contribution_groups_with_search(SharedToolProfile::Product, PathBuf::from("."), search)?;
    build_product_registry(
        shared,
        scheduler_tools,
        default_todo_tool_contributions(Uuid::new_v4()),
    )
}

pub(crate) fn build_tui_tool_registry_with_search(
    workspace_root: PathBuf,
    session_id: Uuid,
    scheduler_tools: Vec<Arc<dyn AgentTool>>,
    atomic_create: Option<talos_core::tool::SharedAtomicCreateCapability>,
    language_provider: Option<LoadedLanguageContext>,
    search: SearchToolConfiguration,
) -> Result<ToolRegistry> {
    let shared = contribution_groups_with_language_provider_and_search(
        SharedToolProfile::Product,
        workspace_root,
        atomic_create,
        language_provider,
        search,
    )?;
    build_product_registry(
        shared,
        scheduler_tools,
        default_todo_tool_contributions(session_id),
    )
}

pub(crate) fn build_mcp_tool_registry_with_search(
    search: SearchToolConfiguration,
) -> Result<ToolRegistry> {
    let shared =
        contribution_groups_with_search(SharedToolProfile::Mcp, PathBuf::from("."), search)?;
    let mut registry = ToolRegistry::new();
    register_builtin(&mut registry, shared.shell)?;
    register_builtin(&mut registry, shared.files)?;
    register_builtin(&mut registry, shared.workspace)?;
    register_builtin(&mut registry, shared.network)?;
    register_builtin(
        &mut registry,
        [ToolContribution::new(
            ToolContributionSource::new("talos-cli:status"),
            Arc::new(StatusTool),
        )],
    )?;
    register_builtin(&mut registry, shared.symbols)?;
    register_builtin(&mut registry, shared.git_read)?;
    register_builtin(&mut registry, shared.git_mutation)?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_product_and_mcp_keep_web_search_builtin_identity() {
        let product =
            build_print_tool_registry_with_search(Vec::new(), SearchToolConfiguration::DuckDuckGo)
                .expect("configured product registry");
        let mcp = build_mcp_tool_registry_with_search(SearchToolConfiguration::DuckDuckGo)
            .expect("configured mcp registry");
        assert!(product.is_builtin_instance("web_search"));
        assert!(mcp.is_builtin_instance("web_search"));
        assert!(mcp.is_builtin_instance("status"));
    }
}
