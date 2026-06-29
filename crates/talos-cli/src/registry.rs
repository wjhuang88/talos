//! Tool registry construction and permission-aware tool wrappers.
//!
//! Contains the permission-aware tool wrappers for interactive/TUI modes
//! and functions that build tool registries for different runtime modes.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use talos_core::ApprovalChoice;
use talos_core::tool::{AgentTool, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult};
use talos_permission::{
    PermissionDecision, PermissionEngine, PermissionRule, ResourceExtractor, ResourceKind,
};
use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::symbol::{FindReferencesTool, FindSymbolTool, ListImportsTool, ListSymbolsTool};
use talos_tools::{
    BashTool, DeleteTool, DiffTool, EditTool, GlobTool, GrepTool, HttpRequestTool, LsTool,
    ReadTool, SaveUrlTool, StatTool, TreeTool, WebSearchTool, WriteTool,
};
use tokio::sync::mpsc;

use crate::approval::ApprovalPrompt;
use crate::colors;
use talos_conversation::UiOutput;

/// Non-blocking approval handler for TUI mode.
///
/// Sends approval requests to the TUI via a channel and awaits responses
/// via oneshot channels. Unlike [`ApprovalPrompt`], this does not block
/// on stdin — the TUI renders an overlay and handles user interaction.
pub(crate) struct TuiApprovalHandler {
    ui_output_tx: mpsc::UnboundedSender<UiOutput>,
    engine: Mutex<PermissionEngine>,
}

impl TuiApprovalHandler {
    pub(crate) fn new(
        ui_output_tx: mpsc::UnboundedSender<UiOutput>,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            ui_output_tx,
            engine: Mutex::new(PermissionEngine::with_workspace_root(workspace_root)),
        }
    }

    async fn request_approval(
        &self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        input: &serde_json::Value,
        summary_fields: Vec<String>,
    ) -> ApprovalChoice {
        let decision = {
            let engine = self.engine.lock().expect("engine lock poisoned");
            engine.evaluate_profile(tool_name, profile, input)
        };
        match decision {
            PermissionDecision::Allow => ApprovalChoice::ApproveOnce,
            PermissionDecision::Deny(_) => ApprovalChoice::Deny,
            PermissionDecision::Ask => {
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();

                if self
                    .ui_output_tx
                    .send(UiOutput::ToolApprovalRequest {
                        tool_name: tool_name.to_string(),
                        arguments: input.clone(),
                        summary_fields,
                        response: response_tx,
                    })
                    .is_err()
                {
                    return ApprovalChoice::Deny;
                }

                match response_rx.await {
                    Ok(choice) => choice,
                    Err(_) => ApprovalChoice::Deny,
                }
            }
        }
    }

    fn add_always_allow_rules(&self, profile: &[ToolPermissionFacet], input: &serde_json::Value) {
        let mut engine = self.engine.lock().expect("engine lock poisoned");
        for facet in profile {
            let resource = facet
                .resource
                .clone()
                .or_else(|| ResourceExtractor::extract(facet.nature, input));
            let resource_kind = facet
                .resource_kind
                .map(ResourceKind::from)
                .or_else(|| Some(default_resource_kind(facet.nature)));
            engine.add_rule(PermissionRule::new_nature(
                facet.nature,
                resource,
                resource_kind,
                PermissionDecision::Allow,
            ));
        }
    }
}

fn default_resource_kind(nature: talos_core::tool::ToolNature) -> ResourceKind {
    match nature {
        talos_core::tool::ToolNature::Network => ResourceKind::Domain,
        talos_core::tool::ToolNature::Execute => ResourceKind::Command,
        talos_core::tool::ToolNature::Read | talos_core::tool::ToolNature::Write => {
            ResourceKind::Path
        }
    }
}

/// Permission-aware tool wrapper for TUI mode.
///
/// Unlike [`PermissionAwareTool`], this uses [`TuiApprovalHandler`] for
/// non-blocking approval via channels instead of blocking on stdin.
pub(crate) struct TuiPermissionAwareTool {
    inner: Arc<dyn AgentTool>,
    approval: Arc<TuiApprovalHandler>,
}

#[async_trait]
impl AgentTool for TuiPermissionAwareTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let tool_name = self.inner.name().to_owned();
        let summary_fields = self
            .inner
            .summary_fields()
            .iter()
            .map(|field| (*field).to_string())
            .collect();
        let profile = self.inner.permission_profile(&input);
        let choice = self
            .approval
            .request_approval(&tool_name, &profile, &input, summary_fields)
            .await;

        match choice {
            ApprovalChoice::ApproveOnce => self.inner.execute(input).await,
            ApprovalChoice::AlwaysApprove => {
                self.approval.add_always_allow_rules(&profile, &input);
                self.inner.execute(input).await
            }
            ApprovalChoice::Deny => ToolResult::error("Permission denied: User denied".to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        self.inner.nature()
    }

    fn family(&self) -> ToolFamily {
        self.inner.family()
    }

    fn is_always_on(&self) -> bool {
        self.inner.is_always_on()
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        self.inner.permission_profile(input)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        self.inner.summary_fields()
    }

    fn provenance(&self) -> talos_core::tool::ToolProvenance {
        self.inner.provenance()
    }
}

/// Permission-aware tool wrapper that checks the permission engine before
/// executing the underlying tool. In interactive mode, [`PermissionDecision::Ask`]
/// triggers a user prompt. In print mode, it defaults to deny.
pub(crate) struct PermissionAwareTool {
    pub(crate) inner: Arc<dyn AgentTool>,
    pub(crate) approval: Arc<Mutex<ApprovalPrompt>>,
    pub(crate) print_mode: bool,
}

#[async_trait]
impl AgentTool for PermissionAwareTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let tool_name = self.inner.name().to_owned();
        let profile = self.inner.permission_profile(&input);
        let decision = {
            let mut approval = self.approval.lock().expect("approval lock poisoned");
            let engine_decision = approval
                .engine()
                .evaluate_profile(&tool_name, &profile, &input);

            match engine_decision {
                PermissionDecision::Allow => PermissionDecision::Allow,
                PermissionDecision::Deny(reason) => PermissionDecision::Deny(reason),
                PermissionDecision::Ask => {
                    if self.print_mode {
                        PermissionDecision::Deny(
                            "Print mode: interactive approval unavailable".to_string(),
                        )
                    } else {
                        match approval.prompt_profile(&tool_name, &profile, &input) {
                            Ok(decision) => decision,
                            Err(e) => PermissionDecision::Deny(format!("Approval error: {e}")),
                        }
                    }
                }
            }
        };

        match decision {
            PermissionDecision::Allow => self.inner.execute(input).await,
            PermissionDecision::Deny(reason) => {
                ToolResult::error(format!("Permission denied: {reason}"))
            }
            PermissionDecision::Ask => {
                unreachable!(
                    "Ask decision should have been resolved by prompt or print-mode default"
                )
            }
        }
    }

    fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        self.inner.nature()
    }

    fn family(&self) -> ToolFamily {
        self.inner.family()
    }

    fn is_always_on(&self) -> bool {
        self.inner.is_always_on()
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        self.inner.permission_profile(input)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        self.inner.summary_fields()
    }

    fn provenance(&self) -> talos_core::tool::ToolProvenance {
        self.inner.provenance()
    }
}

pub(crate) fn register_permission_aware_tools(
    registry: &mut ToolRegistry,
    tools: &[Arc<dyn AgentTool>],
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
) {
    for tool in tools {
        registry.register(Arc::new(PermissionAwareTool {
            inner: tool.clone(),
            approval: approval.clone(),
            print_mode,
        }));
    }
}

pub(crate) fn register_tui_permission_aware_tools(
    registry: &mut ToolRegistry,
    tools: &[Arc<dyn AgentTool>],
    approval: Arc<TuiApprovalHandler>,
) {
    for tool in tools {
        registry.register(Arc::new(TuiPermissionAwareTool {
            inner: tool.clone(),
            approval: approval.clone(),
        }));
    }
}

/// A lightweight health/status tool for MCP mode.
struct StatusTool;

#[async_trait]
impl AgentTool for StatusTool {
    fn name(&self) -> &str {
        "status"
    }

    fn description(&self) -> &str {
        "Return Talos MCP server status"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("talos mcp server alive")
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

pub(crate) fn build_print_tool_registry() -> ToolRegistry {
    let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(BashTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ReadTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(WriteTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(EditTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GrepTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GlobTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(LsTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DeleteTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(SaveUrlTool::new()),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DiffTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(StatTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(FindSymbolTool::new(PathBuf::from("."))));
    registry.register(Arc::new(FindReferencesTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ListSymbolsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ListImportsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitStatusTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitDiffTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitLogTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitShowTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitBranchListTool::new(PathBuf::from("."))));
    registry.register(Arc::new(TreeTool::new(PathBuf::from("."))));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitAddTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPushTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPullTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(HttpRequestTool::new()),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(WebSearchTool::new()),
        approval: approval.clone(),
        print_mode: true,
    }));

    registry
}

pub(crate) fn build_tui_tool_registry(
    approval_handler: Arc<TuiApprovalHandler>,
    workspace_root: PathBuf,
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(BashTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(ReadTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(WriteTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(EditTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GrepTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GlobTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(LsTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(DeleteTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(SaveUrlTool::new()),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(DiffTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(StatTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(FindSymbolTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(FindReferencesTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(ListSymbolsTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(ListImportsTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(GitStatusTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitDiffTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitLogTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitShowTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitBranchListTool::new(workspace_root.clone())));
    registry.register(Arc::new(TreeTool::new(workspace_root.clone())));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitAddTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitPushTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitPullTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(workspace_root)),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(HttpRequestTool::new()),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(WebSearchTool::new()),
        approval: approval_handler.clone(),
    }));
    registry
}

pub(crate) fn build_mcp_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ReadTool::new(PathBuf::from("."))));
    registry.register(Arc::new(WriteTool::new(PathBuf::from("."))));
    registry.register(Arc::new(EditTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GrepTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GlobTool::new(PathBuf::from("."))));
    registry.register(Arc::new(LsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(DeleteTool::new(PathBuf::from("."))));
    registry.register(Arc::new(SaveUrlTool::new()));
    registry.register(Arc::new(DiffTool::new(PathBuf::from("."))));
    registry.register(Arc::new(StatTool::new(PathBuf::from("."))));
    registry.register(Arc::new(StatusTool));
    registry.register(Arc::new(FindSymbolTool::new(PathBuf::from("."))));
    registry.register(Arc::new(FindReferencesTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ListSymbolsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ListImportsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitStatusTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitDiffTool::new(PathBuf::from("."))));
    registry.register(Arc::new(HttpRequestTool::new()));
    registry.register(Arc::new(WebSearchTool::new()));
    registry.register(Arc::new(GitLogTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitShowTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitBranchListTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitAddTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitCommitTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitPushTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitPullTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitCheckoutTool::new(PathBuf::from("."))));
    registry.register(Arc::new(TreeTool::new(PathBuf::from("."))));
    registry
}

/// Format a search snippet with Nord theme highlighting for matched terms.
///
/// Replaces FTS5 `<b>...</b>` markers with ANSI color codes.
pub(crate) fn highlight_snippet(snippet: &str) -> String {
    snippet
        .replace("<b>", &format!("{}{}", colors::NORD13, colors::BOLD))
        .replace("</b>", &format!("{}{}", colors::RESET, colors::NORD13))
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{ToolNature, ToolProvenance};

    struct RemoteTool {
        nature: ToolNature,
    }

    #[async_trait]
    impl AgentTool for RemoteTool {
        fn name(&self) -> &str {
            "mcp:test:fixture"
        }

        fn description(&self) -> &str {
            "fixture"
        }

        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            ToolResult::success("executed")
        }

        fn nature(&self) -> ToolNature {
            self.nature
        }

        fn provenance(&self) -> ToolProvenance {
            ToolProvenance::McpRemote {
                server: "test".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn print_wrapper_denies_write_mcp_tool_and_preserves_provenance() {
        let tool = PermissionAwareTool {
            inner: Arc::new(RemoteTool {
                nature: ToolNature::Write,
            }),
            approval: Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new()))),
            print_mode: true,
        };

        assert_eq!(
            tool.provenance(),
            ToolProvenance::McpRemote {
                server: "test".to_string()
            }
        );
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_error);
        assert!(result.content.contains("interactive approval unavailable"));
    }

    #[tokio::test]
    async fn tui_wrapper_allows_read_only_mcp_tool_without_prompt() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let tool = TuiPermissionAwareTool {
            inner: Arc::new(RemoteTool {
                nature: ToolNature::Read,
            }),
            approval: Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
        };

        let result = tool.execute(serde_json::json!({})).await;

        assert!(!result.is_error);
        assert_eq!(result.content, "executed");
        assert!(rx.try_recv().is_err());
        assert_eq!(
            tool.provenance(),
            ToolProvenance::McpRemote {
                server: "test".to_string()
            }
        );
    }
}
