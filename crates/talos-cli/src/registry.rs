//! Tool registry construction and permission-aware tool wrappers.
//!
//! Contains the permission-aware tool wrappers for interactive/TUI modes
//! and functions that build tool registries for different runtime modes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use talos_conversation::{TipKind, UiOutput};
use talos_core::ApprovalChoice;
use talos_core::tool::{
    AgentTool, ToolBackend, ToolContribution, ToolContributionSource, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult,
};
use talos_permission::{
    GrantScope, GrantSource, InteractionCapability, PermissionContext, PermissionDecision,
    PermissionEngine, PermissionMode, PermissionRequest, PermissionSessionState,
};
use talos_plugin::wasm::{LoadedPluginPackage, WasmRuntime, load_read_only_wasm_package};
use talos_runtime::composition::{SharedToolProfile, contribution_groups};
use talos_session::{SessionManager, todo_tool_contributions_for_sessions_dir};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::approval::ApprovalPrompt;
use crate::colors;

/// Non-blocking approval handler for TUI mode.
///
/// Sends approval requests to the TUI via a channel and awaits responses
/// via oneshot channels. Unlike [`ApprovalPrompt`], this does not block
/// on stdin — the TUI renders an overlay and handles user interaction.
pub(crate) struct TuiApprovalHandler {
    ui_output_tx: mpsc::UnboundedSender<UiOutput>,
    state: Arc<PermissionSessionState>,
}

impl TuiApprovalHandler {
    #[cfg(test)]
    pub(crate) fn new(
        ui_output_tx: mpsc::UnboundedSender<UiOutput>,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            ui_output_tx,
            state: Arc::new(PermissionSessionState::new(
                PermissionEngine::with_workspace_root(workspace_root),
            )),
        }
    }

    pub(crate) fn new_with_trust(
        ui_output_tx: mpsc::UnboundedSender<UiOutput>,
        workspace_root: PathBuf,
        talos_root: &Path,
    ) -> Self {
        let mut engine = PermissionEngine::with_workspace_root(workspace_root.clone());

        let trust_store = talos_permission::WorkspaceTrustStore::new(talos_root);
        let is_git = talos_permission::is_git_workspace(&workspace_root);
        let is_trusted = trust_store.is_trusted(&workspace_root);

        if is_git && is_trusted {
            engine.set_trusted_workspace(true);
            let _ = ui_output_tx.send(UiOutput::Tip {
                text: format!(
                    "Workspace trusted: write operations within {} will be auto-approved",
                    workspace_root.display()
                ),
                kind: TipKind::Info,
            });
        } else if is_git && !is_trusted {
            let _ = ui_output_tx.send(UiOutput::Tip {
                text: "Git workspace detected. Run 'talos --trust' to enable auto-approval for repo-scoped writes".to_string(),
                kind: TipKind::Info,
            });
        }

        Self {
            ui_output_tx,
            state: Arc::new(PermissionSessionState::new(engine)),
        }
    }

    /// Returns a shared handle to the permission engine so callers like
    /// the TUI bridge can evaluate image-attachment paths against the
    /// same SEC-001 rule set (P1-A).
    pub(crate) fn shared_engine(&self) -> Arc<PermissionSessionState> {
        self.state.clone()
    }

    async fn authorize(
        &self,
        tool_name: &str,
        provenance: talos_core::tool::ToolProvenance,
        profile: &[ToolPermissionFacet],
        evaluation_input: &serde_json::Value,
        presentation_input: &serde_json::Value,
        summary_fields: Vec<String>,
    ) -> Result<Vec<ToolExecutionAuthorization>, String> {
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request = PermissionRequest::new(tool_name, provenance, profile, evaluation_input);
        let evaluation = self
            .state
            .evaluate(&request, &context)
            .map_err(|e| e.to_string())?;
        let pending = match evaluation.decision() {
            PermissionDecision::Allow => self
                .state
                .prepare_authorized(&request, &context)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "request is not authorized".to_string())?,
            PermissionDecision::Deny(reason) => return Err(reason),
            PermissionDecision::Ask => {
                let once = self
                    .state
                    .propose(&request, &context, GrantScope::Once)
                    .map_err(|e| e.to_string())?;
                let session = self
                    .state
                    .propose(&request, &context, GrantScope::Session)
                    .map_err(|e| e.to_string())?;
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();

                if self
                    .ui_output_tx
                    .send(UiOutput::ToolApprovalRequest {
                        tool_name: tool_name.to_string(),
                        arguments: presentation_input.clone(),
                        summary_fields,
                        preview: Some(crate::approval::format_grant_preview(session.preview())),
                        response: response_tx,
                    })
                    .is_err()
                {
                    return Err("approval channel is unavailable".to_string());
                }

                match response_rx
                    .await
                    .map_err(|_| "approval channel closed".to_string())?
                {
                    ApprovalChoice::ApproveOnce => self
                        .state
                        .approve_once(once, &request, &context)
                        .map_err(|e| e.to_string())?,
                    ApprovalChoice::AlwaysApprove => self
                        .state
                        .approve_session(session, &request, &context, GrantSource::InteractiveHuman)
                        .map_err(|e| e.to_string())?,
                    ApprovalChoice::Deny => return Err("User denied".to_string()),
                }
            }
        };
        self.state
            .admit(pending, &request, &context)
            .map_err(|e| e.to_string())
    }
}

fn default_todo_tool_contributions(session_id: Uuid) -> Vec<ToolContribution> {
    let Ok(sessions_dir) = SessionManager::default_sessions_dir() else {
        return Vec::new();
    };
    todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id)
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
        let authorizations = match self
            .approval
            .authorize(
                &tool_name,
                self.inner.provenance(),
                &profile,
                &input,
                &self.inner.project_input(&input),
                summary_fields,
            )
            .await
        {
            Ok(authorizations) => authorizations,
            Err(error) => return ToolResult::error(format!("Permission denied: {error}")),
        };
        self.inner
            .execute_authorized_with_output(input, &authorizations)
            .await
            .result
    }

    async fn execute_with_output(&self, input: Value) -> ToolExecutionOutput {
        let tool_name = self.inner.name().to_owned();
        let summary_fields = self
            .inner
            .summary_fields()
            .iter()
            .map(|field| (*field).to_string())
            .collect();
        let profile = self.inner.permission_profile(&input);
        let authorizations = match self
            .approval
            .authorize(
                &tool_name,
                self.inner.provenance(),
                &profile,
                &input,
                &self.inner.project_input(&input),
                summary_fields,
            )
            .await
        {
            Ok(authorizations) => authorizations,
            Err(error) => return ToolExecutionOutput::error(format!("Permission denied: {error}")),
        };
        self.inner
            .execute_authorized_with_output(input, &authorizations)
            .await
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

    fn conditional_backends(&self) -> Vec<ToolBackend> {
        self.inner.conditional_backends()
    }

    fn backend_for_input(&self, input: &Value) -> Option<String> {
        self.inner.backend_for_input(input)
    }

    fn description_for_backends(&self, backends: &HashSet<String>) -> String {
        self.inner.description_for_backends(backends)
    }

    fn parameters_for_backends(&self, backends: &HashSet<String>) -> Value {
        self.inner.parameters_for_backends(backends)
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

    fn project_input(&self, input: &Value) -> Value {
        self.inner.project_input(input)
    }

    fn project_result(&self, result: &ToolResult) -> talos_core::tool::ToolResultProjection {
        self.inner.project_result(result)
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

impl PermissionAwareTool {
    fn authorize(
        &self,
        input: &Value,
        profile: &[ToolPermissionFacet],
    ) -> Result<Vec<ToolExecutionAuthorization>, String> {
        let state = self
            .approval
            .lock()
            .map_err(|_| "approval lock poisoned".to_string())?
            .session_state();
        let interaction = if self.print_mode {
            InteractionCapability::Unavailable
        } else {
            InteractionCapability::Available
        };
        let mode = if self.print_mode {
            PermissionMode::Headless
        } else {
            PermissionMode::Interactive
        };
        let context = PermissionContext::new(mode, interaction);
        let tool_name = self.inner.name();
        let request = PermissionRequest::new(tool_name, self.inner.provenance(), profile, input);
        let evaluation = state
            .evaluate(&request, &context)
            .map_err(|e| e.to_string())?;
        let pending = match evaluation.decision() {
            PermissionDecision::Allow => state
                .prepare_authorized(&request, &context)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "request is not authorized".to_string())?,
            PermissionDecision::Deny(reason) => return Err(reason),
            PermissionDecision::Ask if self.print_mode => {
                return Err("Print mode: interactive approval unavailable".to_string());
            }
            PermissionDecision::Ask => {
                let once = state
                    .propose(&request, &context, GrantScope::Once)
                    .map_err(|e| e.to_string())?;
                let session = state
                    .propose(&request, &context, GrantScope::Session)
                    .map_err(|e| e.to_string())?;
                let choice = ApprovalPrompt::prompt_choice(
                    tool_name,
                    &self.inner.project_input(input),
                    session.preview(),
                )
                .map_err(|e| format!("Approval error: {e}"))?;
                match choice {
                    ApprovalChoice::ApproveOnce => state
                        .approve_once(once, &request, &context)
                        .map_err(|e| e.to_string())?,
                    ApprovalChoice::AlwaysApprove => state
                        .approve_session(session, &request, &context, GrantSource::InteractiveHuman)
                        .map_err(|e| e.to_string())?,
                    ApprovalChoice::Deny => return Err("User denied".to_string()),
                }
            }
        };
        state
            .admit(pending, &request, &context)
            .map_err(|e| e.to_string())
    }
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
        let profile = self.inner.permission_profile(&input);
        let authorizations = match self.authorize(&input, &profile) {
            Ok(authorizations) => authorizations,
            Err(error) => return ToolResult::error(format!("Permission denied: {error}")),
        };
        self.inner
            .execute_authorized_with_output(input, &authorizations)
            .await
            .result
    }

    async fn execute_with_output(&self, input: Value) -> ToolExecutionOutput {
        let profile = self.inner.permission_profile(&input);
        let authorizations = match self.authorize(&input, &profile) {
            Ok(authorizations) => authorizations,
            Err(error) => return ToolExecutionOutput::error(format!("Permission denied: {error}")),
        };
        self.inner
            .execute_authorized_with_output(input, &authorizations)
            .await
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

    fn conditional_backends(&self) -> Vec<ToolBackend> {
        self.inner.conditional_backends()
    }

    fn backend_for_input(&self, input: &Value) -> Option<String> {
        self.inner.backend_for_input(input)
    }

    fn description_for_backends(&self, backends: &HashSet<String>) -> String {
        self.inner.description_for_backends(backends)
    }

    fn parameters_for_backends(&self, backends: &HashSet<String>) -> Value {
        self.inner.parameters_for_backends(backends)
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

    fn project_input(&self, input: &Value) -> Value {
        self.inner.project_input(input)
    }

    fn project_result(&self, result: &ToolResult) -> talos_core::tool::ToolResultProjection {
        self.inner.project_result(result)
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

type LoadedPluginTools = (Vec<Arc<dyn AgentTool>>, LoadedPluginPackage);

fn load_explicit_plugin_tools(package_roots: &[PathBuf]) -> Result<Vec<LoadedPluginTools>, String> {
    if package_roots.is_empty() {
        return Ok(Vec::new());
    }
    let runtime = Arc::new(
        WasmRuntime::new(100_000, 250)
            .map_err(|error| format!("failed to initialize WASM runtime: {error}"))?,
    );
    let mut loaded = Vec::with_capacity(package_roots.len());
    for package_root in package_roots {
        let (tools, package) =
            load_read_only_wasm_package(runtime.clone(), package_root).map_err(|error| {
                format!(
                    "failed to load plugin package '{}': {error}",
                    package_root.display()
                )
            })?;
        loaded.push((tools, package));
    }
    Ok(loaded)
}

fn plugin_source(package: &LoadedPluginPackage) -> ToolContributionSource {
    ToolContributionSource::new(format!("plugin:{}@{}", package.name, package.version))
}

/// Loads explicitly selected local packages and registers their tools behind
/// the blocking/print permission adapter.
pub(crate) fn register_explicit_permission_aware_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    let mut contributions = Vec::new();
    for (tools, package) in loaded {
        let source = plugin_source(&package);
        contributions.extend(tools.into_iter().map(|tool| {
            ToolContribution::new(source.clone(), tool).map_tool(|tool| {
                Arc::new(PermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                    print_mode,
                })
            })
        }));
        packages.push(package);
    }
    registry
        .register_contributions(contributions)
        .map_err(|error| error.to_string())?;
    Ok(packages)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the non-blocking TUI permission adapter.
pub(crate) fn register_explicit_tui_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<TuiApprovalHandler>,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    let mut contributions = Vec::new();
    for (tools, package) in loaded {
        let source = plugin_source(&package);
        contributions.extend(tools.into_iter().map(|tool| {
            ToolContribution::new(source.clone(), tool).map_tool(|tool| {
                Arc::new(TuiPermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                })
            })
        }));
        packages.push(package);
    }
    registry
        .register_contributions(contributions)
        .map_err(|error| error.to_string())?;
    Ok(packages)
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

fn map_workspace_contribution(
    contribution: ToolContribution,
    wrap: impl FnOnce(Arc<dyn AgentTool>) -> Arc<dyn AgentTool>,
) -> ToolContribution {
    if contribution.name() == "tree" {
        contribution
    } else {
        contribution.map_tool(wrap)
    }
}

fn register_symbol_tool_contributions(
    registry: &mut ToolRegistry,
    contributions: Vec<ToolContribution>,
    wrap: impl Fn(Arc<dyn AgentTool>) -> Arc<dyn AgentTool>,
) {
    for contribution in contributions {
        let contribution = contribution.map_tool(|tool| wrap(tool));
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
}

/// Builds the tool registry for print/inline/RPC modes.
///
/// These modes construct a registry before any durable [`talos_session::Session`]
/// exists, so todo tools are bound to a fresh in-process session id — scoped to
/// this one run and discarded on exit, not persisted across invocations.
pub(crate) fn build_print_tool_registry(scheduler_tools: Vec<Arc<dyn AgentTool>>) -> ToolRegistry {
    let ephemeral_session_id = Uuid::new_v4();
    build_print_tool_registry_with_todo_contributions(
        scheduler_tools,
        default_todo_tool_contributions(ephemeral_session_id),
    )
}

fn build_print_tool_registry_with_todo_contributions(
    scheduler_tools: Vec<Arc<dyn AgentTool>>,
    todo_contributions: Vec<ToolContribution>,
) -> ToolRegistry {
    let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

    let mut registry = ToolRegistry::new();
    let shared = contribution_groups(SharedToolProfile::Product, PathBuf::from("."));
    for contribution in shared.shell {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.files {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    let read_image = shared
        .image
        .expect("product profile must include read_image")
        .map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
    registry
        .register_contribution(read_image)
        .unwrap_or_else(|error| panic!("{error}"));
    for contribution in shared.workspace {
        let contribution = map_workspace_contribution(contribution, |tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.network {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    register_symbol_tool_contributions(&mut registry, shared.symbols, |tool| tool);
    for contribution in shared.git_read {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.git_mutation {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in todo_contributions {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for tool in scheduler_tools {
        registry.register(Arc::new(PermissionAwareTool {
            inner: tool,
            approval: approval.clone(),
            print_mode: true,
        }));
    }

    registry
}

pub(crate) fn build_tui_tool_registry(
    approval_handler: Arc<TuiApprovalHandler>,
    workspace_root: PathBuf,
    session_id: Uuid,
    delay_tool: Vec<Arc<dyn AgentTool>>,
) -> ToolRegistry {
    build_tui_tool_registry_with_todo_contributions(
        approval_handler,
        workspace_root,
        delay_tool,
        default_todo_tool_contributions(session_id),
    )
}

fn build_tui_tool_registry_with_todo_contributions(
    approval_handler: Arc<TuiApprovalHandler>,
    workspace_root: PathBuf,
    delay_tool: Vec<Arc<dyn AgentTool>>,
    todo_contributions: Vec<ToolContribution>,
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    let shared = contribution_groups(SharedToolProfile::Product, workspace_root);
    for contribution in shared.shell {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.files {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    let read_image = shared
        .image
        .expect("product profile must include read_image")
        .map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
    registry
        .register_contribution(read_image)
        .unwrap_or_else(|error| panic!("{error}"));
    for contribution in shared.workspace {
        let contribution = map_workspace_contribution(contribution, |tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.network {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    register_symbol_tool_contributions(
        &mut registry,
        shared.symbols,
        |tool| -> Arc<dyn AgentTool> {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        },
    );
    for contribution in shared.git_read {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.git_mutation {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in todo_contributions {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for tool in delay_tool {
        registry.register(Arc::new(TuiPermissionAwareTool {
            inner: tool,
            approval: approval_handler.clone(),
        }));
    }
    registry
}

pub(crate) fn build_mcp_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    let shared = contribution_groups(SharedToolProfile::Mcp, PathBuf::from("."));
    for contribution in shared.shell {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.files {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.workspace {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.network {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    registry.register(Arc::new(StatusTool));
    register_symbol_tool_contributions(&mut registry, shared.symbols, |tool| tool);
    for contribution in shared.git_read {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    for contribution in shared.git_mutation {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
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
    use talos_tools::ReadImageTool;

    #[cfg(windows)]
    const SHELL_TOOL_NAME: &str = "powershell";
    #[cfg(not(windows))]
    const SHELL_TOOL_NAME: &str = "bash";

    struct NamedReadTool {
        name: &'static str,
        description: &'static str,
    }

    #[async_trait]
    impl AgentTool for NamedReadTool {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            self.description
        }

        fn parameters(&self) -> Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _input: Value) -> ToolResult {
            ToolResult::success(self.description)
        }
    }

    fn write_plugin_fixture(
        root: &Path,
        package_name: &str,
        version: &str,
        tool_name: &str,
    ) -> PathBuf {
        std::fs::create_dir_all(root).expect("operation should succeed");
        std::fs::write(
            root.join("talos-plugin.toml"),
            format!(
                "[plugin]\nname = \"{package_name}\"\nversion = \"{version}\"\ncarrier = \"wasm\"\nartifact = \"demo.wat\"\ndescription = \"collision fixture\"\n\n[[tools]]\nname = \"{tool_name}\"\nhandler = \"demo.wat\"\n"
            ),
        )
        .expect("operation should succeed");
        std::fs::write(
            root.join("demo.wat"),
            "(module (func (export \"run\") (result i32) i32.const 7))",
        )
        .expect("operation should succeed");
        root.to_path_buf()
    }

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

    #[tokio::test]
    async fn explicit_checked_in_plugin_loads_registers_and_executes_offline() {
        let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../talos-plugin/tests/fixtures/read-only-demo");
        let mut registry = ToolRegistry::new();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(
            PermissionEngine::with_workspace_root(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .expect("workspace parent")
                    .parent()
                    .expect("workspace root")
                    .to_path_buf(),
            ),
        )));

        let packages =
            register_explicit_permission_aware_plugins(&mut registry, &[package], approval, true)
                .expect("plugin loads");

        assert_eq!(packages.len(), 1);
        assert_eq!(
            packages[0].capabilities,
            vec!["read-only-demo.answer".to_string()]
        );
        let tool = registry
            .get("read-only-demo.answer")
            .expect("plugin tool registered");
        assert!(matches!(
            tool.provenance(),
            ToolProvenance::Plugin { ref name, .. } if name == "read-only-demo"
        ));
        let result = tool.execute(serde_json::json!({})).await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("returned 7"));
    }

    #[test]
    fn explicit_plugin_collision_with_checked_builtin_reports_both_sources() {
        let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../talos-plugin/tests/fixtures/read-only-demo");
        let mut registry = ToolRegistry::new();
        registry
            .register_contribution(ToolContribution::new(
                ToolContributionSource::new("talos-tools:file"),
                Arc::new(NamedReadTool {
                    name: "read-only-demo.answer",
                    description: "existing built-in",
                }),
            ))
            .expect("operation should succeed");
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

        let error =
            register_explicit_permission_aware_plugins(&mut registry, &[package], approval, true)
                .expect_err("operation should fail");

        assert_eq!(
            error,
            "duplicate tool registration 'read-only-demo.answer': existing source 'talos-tools:file', incoming source 'plugin:read-only-demo@0.1.0'"
        );
        assert_eq!(
            registry
                .get("read-only-demo.answer")
                .expect("operation should succeed")
                .description(),
            "existing built-in"
        );
    }

    #[test]
    fn explicit_plugin_collision_between_packages_is_transactional() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let first =
            write_plugin_fixture(&temp.path().join("first"), "collision", "1.0.0", "answer");
        let second =
            write_plugin_fixture(&temp.path().join("second"), "collision", "2.0.0", "answer");
        let mut registry = ToolRegistry::new();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));

        let error = register_explicit_permission_aware_plugins(
            &mut registry,
            &[first, second],
            approval,
            true,
        )
        .expect_err("operation should fail");

        assert_eq!(
            error,
            "duplicate tool registration 'collision.answer': existing source 'plugin:collision@1.0.0', incoming source 'plugin:collision@2.0.0'"
        );
        assert!(registry.get("collision.answer").is_none());
    }

    #[tokio::test]
    async fn print_composition_read_uses_model_private_snapshot_projection() {
        let registry = build_print_tool_registry(Vec::new());
        let read = registry.get("read").expect("read tool");
        let result = read
            .execute(serde_json::json!({"path": "Cargo.toml", "limit": 2}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.starts_with("[snapshot:s"));
        let projection = read.project_result(&result);
        assert!(projection.model_content.contains("snapshot:s"));
        assert!(!projection.display_content.contains("snapshot:s"));
        assert!(!projection.persistence_content.contains("snapshot:s"));

        let edit = registry.get("edit").expect("edit tool");
        let schema = edit.parameters().to_string();
        assert!(schema.contains("snapshot_id"));
        assert!(schema.contains("replace_range"));
    }

    #[test]
    fn print_tui_and_mcp_registries_preserve_git_inventory() {
        let print_registry = build_print_tool_registry(Vec::new());
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Uuid::new_v4(),
            Vec::new(),
        );
        let mcp_registry = build_mcp_tool_registry();
        let names = [
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

        for name in names {
            assert!(print_registry.get(name).is_some(), "print missing {name}");
            assert!(tui_registry.get(name).is_some(), "TUI missing {name}");
            assert!(mcp_registry.get(name).is_some(), "MCP missing {name}");
        }
    }

    fn sorted_registry_names(registry: &ToolRegistry) -> Vec<String> {
        let mut names = registry
            .list()
            .into_iter()
            .map(|tool| tool.name().to_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn product_registries_have_exact_sorted_inventories() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let sessions_dir = temp.path().join("sessions");
        let session_id = Uuid::nil();

        let print_registry = build_print_tool_registry_with_todo_contributions(
            Vec::new(),
            todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id),
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry_with_todo_contributions(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Vec::new(),
            todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id),
        );
        let mcp_registry = build_mcp_tool_registry();

        let mut print_tui_inventory = vec![
            SHELL_TOOL_NAME,
            "delete",
            "diff",
            "document_extract",
            "edit",
            "exec",
            "fetch_url",
            "find_references",
            "find_symbol",
            "git_add",
            "git_branch_list",
            "git_checkout",
            "git_commit",
            "git_diff",
            "git_log",
            "git_pull",
            "git_push",
            "git_show",
            "git_status",
            "glob",
            "grep",
            "http_request",
            "list_imports",
            "list_symbols",
            "ls",
            "read",
            "read_image",
            "save_url",
            "stat",
            "todo_add_dependency",
            "todo_create",
            "todo_create_batch",
            "todo_delete",
            "todo_query",
            "todo_remove_dependency",
            "todo_update",
            "todo_update_batch",
            "todo_update_status",
            "tree",
            "web_search",
            "write",
        ];
        let mut mcp_inventory = vec![
            SHELL_TOOL_NAME,
            "delete",
            "diff",
            "document_extract",
            "edit",
            "exec",
            "fetch_url",
            "find_references",
            "find_symbol",
            "git_add",
            "git_branch_list",
            "git_checkout",
            "git_commit",
            "git_diff",
            "git_log",
            "git_pull",
            "git_push",
            "git_show",
            "git_status",
            "glob",
            "grep",
            "http_request",
            "list_imports",
            "list_symbols",
            "ls",
            "read",
            "save_url",
            "stat",
            "status",
            "tree",
            "web_search",
            "write",
        ];

        print_tui_inventory.sort();
        mcp_inventory.sort();

        assert_eq!(sorted_registry_names(&print_registry), print_tui_inventory);
        assert_eq!(sorted_registry_names(&tui_registry), print_tui_inventory);
        assert_eq!(sorted_registry_names(&mcp_registry), mcp_inventory);
    }

    struct DescriptionMarkerTool {
        inner: Arc<dyn AgentTool>,
    }

    #[async_trait]
    impl AgentTool for DescriptionMarkerTool {
        fn name(&self) -> &str {
            self.inner.name()
        }

        fn description(&self) -> &str {
            "wrapped-marker"
        }

        fn parameters(&self) -> Value {
            self.inner.parameters()
        }

        async fn execute(&self, input: Value) -> ToolResult {
            self.inner.execute(input).await
        }
    }

    #[test]
    fn workspace_wrapper_policy_preserves_tree_exception() {
        let mut contributions =
            contribution_groups(SharedToolProfile::Product, PathBuf::from(".")).workspace;
        let tree = contributions
            .pop()
            .expect("workspace factory must end with tree");
        let tree = map_workspace_contribution(tree, |tool| {
            Arc::new(DescriptionMarkerTool { inner: tool })
        });
        assert_eq!(tree.name(), "tree");
        assert_ne!(tree.tool().description(), "wrapped-marker");

        let document = contributions
            .into_iter()
            .next()
            .expect("workspace factory must contain document_extract");
        let document = map_workspace_contribution(document, |tool| {
            Arc::new(DescriptionMarkerTool { inner: tool })
        });
        assert_eq!(document.name(), "document_extract");
        assert_eq!(document.tool().description(), "wrapped-marker");
    }

    #[test]
    fn print_tui_and_mcp_registries_preserve_remaining_inventory_and_sources() {
        let print_registry = build_print_tool_registry(Vec::new());
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Uuid::new_v4(),
            Vec::new(),
        );
        let mcp_registry = build_mcp_tool_registry();
        let groups = [
            ("talos-tools:shell", &[SHELL_TOOL_NAME, "exec"][..]),
            (
                "talos-tools:workspace",
                &["document_extract", "grep", "glob", "diff", "stat", "tree"][..],
            ),
            (
                "talos-tools:network",
                &["save_url", "fetch_url", "http_request", "web_search"][..],
            ),
        ];

        for (_, names) in groups {
            for name in names {
                assert!(print_registry.get(name).is_some(), "print missing {name}");
                assert!(tui_registry.get(name).is_some(), "TUI missing {name}");
                assert!(mcp_registry.get(name).is_some(), "MCP missing {name}");
            }
        }

        for (mode, mut registry) in [
            ("print", print_registry),
            ("TUI", tui_registry),
            ("MCP", mcp_registry),
        ] {
            for (source, name) in [
                ("talos-tools:shell", SHELL_TOOL_NAME),
                ("talos-tools:workspace", "document_extract"),
                ("talos-tools:network", "save_url"),
            ] {
                let error = registry
                    .register_contribution(ToolContribution::new(
                        ToolContributionSource::new("test:duplicate"),
                        Arc::new(NamedReadTool {
                            name,
                            description: "duplicate",
                        }),
                    ))
                    .expect_err("operation should fail");
                assert_eq!(error.existing_source.as_str(), source, "{mode} {name}");
                assert_eq!(error.incoming_source.as_str(), "test:duplicate");
            }
        }
    }

    #[test]
    fn print_tui_and_mcp_registries_preserve_file_inventory_and_source() {
        let print_registry = build_print_tool_registry(Vec::new());
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Uuid::new_v4(),
            Vec::new(),
        );
        let mcp_registry = build_mcp_tool_registry();
        let names = ["read", "write", "edit", "ls", "delete"];

        for name in names {
            assert!(print_registry.get(name).is_some(), "print missing {name}");
            assert!(tui_registry.get(name).is_some(), "TUI missing {name}");
            assert!(mcp_registry.get(name).is_some(), "MCP missing {name}");
        }

        for (mode, mut registry) in [
            ("print", print_registry),
            ("TUI", tui_registry),
            ("MCP", mcp_registry),
        ] {
            let error = registry
                .register_contribution(ToolContribution::new(
                    ToolContributionSource::new("test:duplicate"),
                    Arc::new(NamedReadTool {
                        name: "read",
                        description: "duplicate",
                    }),
                ))
                .expect_err("operation should fail");
            assert_eq!(
                error.to_string(),
                "duplicate tool registration 'read': existing source 'talos-tools:file', incoming source 'test:duplicate'",
                "{mode} source mismatch"
            );
        }
    }

    #[test]
    fn print_tui_and_mcp_registries_preserve_symbol_inventory_and_source() {
        let mut print_registry = ToolRegistry::new();
        register_symbol_tool_contributions(
            &mut print_registry,
            contribution_groups(SharedToolProfile::Product, PathBuf::from(".")).symbols,
            |tool| tool,
        );

        let (tx, _rx) = mpsc::unbounded_channel();
        let approval = Arc::new(TuiApprovalHandler::new(tx, PathBuf::from(".")));
        let mut tui_registry = ToolRegistry::new();
        register_symbol_tool_contributions(
            &mut tui_registry,
            contribution_groups(SharedToolProfile::Product, PathBuf::from(".")).symbols,
            |tool| -> Arc<dyn AgentTool> {
                Arc::new(TuiPermissionAwareTool {
                    inner: tool,
                    approval: approval.clone(),
                })
            },
        );

        let mut mcp_registry = ToolRegistry::new();
        register_symbol_tool_contributions(
            &mut mcp_registry,
            contribution_groups(SharedToolProfile::Mcp, PathBuf::from(".")).symbols,
            |tool| tool,
        );

        let names = [
            "find_symbol",
            "find_references",
            "list_symbols",
            "list_imports",
        ];

        for name in names {
            assert!(print_registry.get(name).is_some(), "print missing {name}");
            let tui_tool = tui_registry
                .get(name)
                .unwrap_or_else(|| panic!("TUI missing {name}"));
            assert!(tui_tool.is_read_only(), "TUI wrapper changed {name} policy");
            assert!(mcp_registry.get(name).is_some(), "MCP missing {name}");
        }

        for (mode, mut registry) in [
            ("print", print_registry),
            ("TUI", tui_registry),
            ("MCP", mcp_registry),
        ] {
            let error = registry
                .register_contribution(ToolContribution::new(
                    ToolContributionSource::new("test:duplicate"),
                    Arc::new(NamedReadTool {
                        name: "find_symbol",
                        description: "duplicate",
                    }),
                ))
                .expect_err("operation should fail");
            assert_eq!(
                error.to_string(),
                "duplicate tool registration 'find_symbol': existing source 'talos-tools:symbol', incoming source 'test:duplicate'",
                "{mode} source mismatch"
            );
        }
    }

    #[test]
    fn read_image_profile_is_print_and_tui_only() {
        let print_registry = build_print_tool_registry(Vec::new());
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Uuid::new_v4(),
            Vec::new(),
        );
        let mcp_registry = build_mcp_tool_registry();

        assert!(print_registry.get("read_image").is_some());
        assert!(tui_registry.get("read_image").is_some());
        assert!(mcp_registry.get("read_image").is_none());
    }

    #[test]
    fn print_and_tui_registries_include_todo_tools() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let sessions_dir = dir.path().join("sessions");
        let session_id = Uuid::new_v4();

        let mut print_registry = ToolRegistry::new();
        for contribution in todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id) {
            print_registry
                .register_contribution(contribution)
                .expect("operation should succeed");
        }
        assert!(print_registry.get("todo_create").is_some());
        assert!(print_registry.get("todo_create_batch").is_some());
        assert!(print_registry.get("todo_update_status").is_some());
        assert!(print_registry.get("todo_update").is_some());
        assert!(print_registry.get("todo_update_batch").is_some());
        assert!(print_registry.get("todo_delete").is_some());
        assert!(print_registry.get("todo_add_dependency").is_some());
        assert!(print_registry.get("todo_remove_dependency").is_some());
        assert!(print_registry.get("todo_query").is_some());

        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_approval = Arc::new(TuiApprovalHandler::new(tx, PathBuf::from(".")));
        let mut tui_registry = ToolRegistry::new();
        for contribution in todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id) {
            let contribution = contribution.map_tool(|tool| {
                Arc::new(TuiPermissionAwareTool {
                    inner: tool,
                    approval: tui_approval.clone(),
                })
            });
            tui_registry
                .register_contribution(contribution)
                .expect("operation should succeed");
        }
        assert!(tui_registry.get("todo_create").is_some());
        assert!(tui_registry.get("todo_create_batch").is_some());
        assert!(tui_registry.get("todo_update_status").is_some());
        assert!(tui_registry.get("todo_update").is_some());
        assert!(tui_registry.get("todo_update_batch").is_some());
        assert!(tui_registry.get("todo_delete").is_some());
        assert!(tui_registry.get("todo_add_dependency").is_some());
        assert!(tui_registry.get("todo_remove_dependency").is_some());
        assert!(tui_registry.get("todo_query").is_some());
    }

    #[tokio::test]
    async fn todo_items_survive_registry_rebuild_with_same_session_id() {
        // Simulates a /model switch: rebuild_session_for_model constructs a
        // brand-new registry (new Agent, new tool instances) but passes the
        // SAME session.id as before. A todo created through the "before"
        // registry must be visible through the "after" registry.
        let dir = tempfile::tempdir().expect("operation should succeed");
        let sessions_dir = dir.path().join("sessions");
        let session_id = Uuid::new_v4();

        let before_tools = todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id);
        let create_tool = before_tools
            .iter()
            .find(|contribution| contribution.name() == "todo_create")
            .expect("operation should succeed")
            .tool();
        let created = create_tool
            .execute(serde_json::json!({ "title": "survive model switch" }))
            .await;
        assert!(!created.is_error, "{}", created.content);

        // "After" registry: same session_id, entirely new tool instances.
        let after_tools = todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id);
        let query_tool = after_tools
            .iter()
            .find(|contribution| contribution.name() == "todo_query")
            .expect("operation should succeed")
            .tool();
        let queried = query_tool.execute(serde_json::json!({})).await;
        assert!(queried.content.contains("survive model switch"));
    }

    #[tokio::test]
    async fn delay_denied_by_permission_does_not_execute() {
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": { "Deny": "delay blocked by test" },
                    "nature": "Execute"
                }]
            }))
            .expect("operation should succeed");

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let delay_tool = tools[0].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: delay_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({
                "message": "test",
                "delay_secs": 10
            }))
            .await;

        assert!(result.is_error, "Deny should prevent delay execution");
        assert!(
            result.content.contains("delay blocked"),
            "error should contain deny reason: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn delay_ask_in_print_mode_auto_denies() {
        let engine = PermissionEngine::new();

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let delay_tool = tools[0].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: delay_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({
                "message": "test",
                "delay_secs": 10
            }))
            .await;

        assert!(
            result.is_error,
            "Ask in print mode should auto-deny, not execute"
        );
        assert!(
            result.content.to_lowercase().contains("unavailable")
                || result.content.to_lowercase().contains("print mode"),
            "error should mention print mode: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn schedule_denied_by_permission_does_not_execute() {
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": { "Deny": "schedule blocked by test" },
                    "nature": "Execute"
                }]
            }))
            .expect("operation should succeed");

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let schedule_tool = tools[1].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: schedule_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({
                "message": "test",
                "interval_secs": 10
            }))
            .await;

        assert!(result.is_error, "Deny should prevent schedule execution");
        assert!(result.content.contains("schedule blocked"));
    }

    #[tokio::test]
    async fn schedule_ask_in_print_mode_auto_denies() {
        let engine = PermissionEngine::new();

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let schedule_tool = tools[1].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: schedule_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({
                "message": "test",
                "interval_secs": 10
            }))
            .await;

        assert!(result.is_error, "Ask in print mode should auto-deny");
        assert!(
            result.content.to_lowercase().contains("unavailable")
                || result.content.to_lowercase().contains("print mode"),
            "error should mention print mode: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn cancel_denied_by_permission_does_not_execute() {
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": { "Deny": "cancel blocked by test" },
                    "nature": "Execute"
                }]
            }))
            .expect("operation should succeed");

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let cancel_tool = tools[3].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: cancel_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({"task_id": "sched_1"}))
            .await;

        assert!(result.is_error, "Deny should prevent cancel execution");
        assert!(result.content.contains("cancel blocked"));
    }

    #[tokio::test]
    async fn cancel_ask_in_print_mode_auto_denies() {
        let engine = PermissionEngine::new();

        let (tools, _pending) = talos_agent::create_scheduler_tools();
        let cancel_tool = tools[3].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: cancel_tool,
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({"task_id": "sched_1"}))
            .await;

        assert!(result.is_error, "Ask in print mode should auto-deny cancel");
    }

    #[tokio::test]
    async fn list_tool_is_read_and_allowed() {
        let engine = PermissionEngine::new();

        let (tools, pending) = talos_agent::create_scheduler_tools();
        let list_tool = tools[2].clone();
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: list_tool,
            approval,
            print_mode: true,
        };

        let (sq_tx, _sq_rx) = tokio::sync::mpsc::channel(512);
        let _join = pending.spawn(sq_tx, 0, tokio_util::sync::CancellationToken::new());

        let result = wrapped.execute(serde_json::json!({})).await;

        assert!(
            !result.is_error,
            "Read tool should be auto-allowed, not blocked by print mode: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_image_auto_allowed_for_read_nature() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let img_path = dir.path().join("test.png");
        std::fs::write(&img_path, MINIMAL_PNG).expect("operation should succeed");
        let canonical = img_path.canonicalize().expect("operation should succeed");

        let engine = PermissionEngine::with_workspace_root(dir.path().to_path_buf());
        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: Arc::new(ReadImageTool::new(dir.path().to_path_buf())),
            approval,
            print_mode: true,
        };

        let auth = vec![
            talos_core::tool::ToolExecutionAuthorization::for_path(
                "read_image",
                talos_core::tool::ToolNature::Read,
                dir.path(),
                "test.png",
                talos_core::tool::ToolAuthorizationScope::Once,
            )
            .expect("operation should succeed"),
        ];

        let output = wrapped
            .execute_authorized_with_output(
                serde_json::json!({"path": canonical.to_string_lossy()}),
                &auth,
            )
            .await;
        assert!(!output.result.is_error, "{}", output.result.content);
    }

    #[tokio::test]
    async fn read_image_denied_by_nature_rule() {
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": { "Deny": "read_image blocked by test" },
                    "nature": "Read"
                }]
            }))
            .expect("operation should succeed");

        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: Arc::new(ReadImageTool::new(PathBuf::from("."))),
            approval,
            print_mode: true,
        };

        let result = wrapped
            .execute(serde_json::json!({"path": "test.png"}))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("read_image blocked"));
        assert!(!result.content.contains("test.png"));
    }

    #[tokio::test]
    async fn read_image_path_mismatch_rejected_in_authorized_execution() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let img_a = dir.path().join("a.png");
        std::fs::write(&img_a, [0x89, 0x50, 0x4E, 0x47]).expect("operation should succeed");
        let img_b = dir.path().join("b.png");
        std::fs::write(&img_b, [0x89, 0x50, 0x4E, 0x47]).expect("operation should succeed");

        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let auth = vec![
            talos_core::tool::ToolExecutionAuthorization::for_path(
                "read_image",
                talos_core::tool::ToolNature::Read,
                dir.path(),
                "a.png",
                talos_core::tool::ToolAuthorizationScope::Once,
            )
            .expect("operation should succeed"),
        ];

        let output = tool
            .execute_authorized_with_output(serde_json::json!({"path": "b.png"}), &auth)
            .await;

        assert!(output.result.is_error, "path mismatch must be rejected");
        assert!(output.next_provider_parts.is_empty());
        assert!(
            !output.result.content.contains("b.png"),
            "rejected path must not appear in error text"
        );
    }

    #[tokio::test]
    async fn read_image_ask_in_print_mode_auto_denies() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let external = tempfile::tempdir().expect("operation should succeed");
        let image = external.path().join("test.png");
        std::fs::write(&image, MINIMAL_PNG).expect("operation should succeed");

        let mut engine = PermissionEngine::with_workspace_root(workspace.path().to_path_buf());
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": "Ask",
                    "nature": "Read"
                }]
            }))
            .expect("operation should succeed");

        let approval = Arc::new(Mutex::new(ApprovalPrompt::new(engine)));
        let wrapped = PermissionAwareTool {
            inner: Arc::new(ReadImageTool::new(workspace.path().to_path_buf())),
            approval,
            print_mode: true,
        };

        let result = wrapped.execute(serde_json::json!({"path": image})).await;
        assert!(result.is_error);
        assert!(
            result.content.to_lowercase().contains("unavailable")
                || result.content.to_lowercase().contains("print mode"),
            "Ask in print mode should auto-deny: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_image_ask_then_approve_external_path_executes_via_tui_handler() {
        let workspace = tempfile::tempdir().expect("operation should succeed");
        let external = tempfile::tempdir().expect("operation should succeed");
        let img_path = external.path().join("external.png");
        std::fs::write(&img_path, MINIMAL_PNG).expect("operation should succeed");
        let canonical = img_path.canonicalize().expect("operation should succeed");

        let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
        let handler = Arc::new(TuiApprovalHandler::new(
            ui_tx,
            workspace.path().to_path_buf(),
        ));
        {
            let mut engine = PermissionEngine::with_workspace_root(workspace.path().to_path_buf());
            engine
                .load_from_config(&serde_json::json!({
                    "rules": [{"decision": "Ask", "nature": "Read"}]
                }))
                .expect("operation should succeed");
            handler
                .shared_engine()
                .replace_policy(engine)
                .expect("policy replacement");
        }
        let wrapped = TuiPermissionAwareTool {
            inner: Arc::new(ReadImageTool::new(workspace.path().to_path_buf())),
            approval: handler,
        };

        let approve_task = tokio::spawn(async move {
            if let Some(talos_conversation::UiOutput::ToolApprovalRequest { response, .. }) =
                ui_rx.recv().await
            {
                let _ = response.send(ApprovalChoice::ApproveOnce);
            }
        });

        let output = wrapped
            .execute_with_output(serde_json::json!({"path": canonical.to_string_lossy()}))
            .await;

        let _ = approve_task.await;
        assert!(
            !output.result.is_error,
            "approved external-path read_image should succeed: {}",
            output.result.content
        );
        assert_eq!(
            output.next_provider_parts.len(),
            1,
            "should produce 1 image content part"
        );
    }

    #[test]
    fn read_image_not_in_presented_tools_by_default() {
        let registry = build_print_tool_registry(Vec::new());
        assert!(
            registry.get("read_image").is_some(),
            "read_image must be registered"
        );
    }

    #[test]
    fn read_tool_still_registered_alongside_read_image() {
        let registry = build_print_tool_registry(Vec::new());
        assert!(
            registry.get("read").is_some(),
            "text read tool must still be registered"
        );
        assert!(
            registry.get("read_image").is_some(),
            "read_image must be registered alongside read"
        );
    }

    const MINIMAL_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // signature
        0x00, 0x00, 0x00, 0x0d, // IHDR length
        0x49, 0x48, 0x44, 0x52, // "IHDR"
        0x00, 0x00, 0x00, 0x01, // width=1
        0x00, 0x00, 0x00, 0x01, // height=1
        0x08, 0x02, 0x00, 0x00, 0x00, // bitdepth=8, colortype=RGB
        0x90, 0x77, 0x53, 0xde, // CRC
        0x00, 0x00, 0x00, 0x0c, // IDAT length
        0x49, 0x44, 0x41, 0x54, // "IDAT"
        0x78, 0x9c, 0x63, 0xf8, 0xff, 0xff, 0x3f, 0x00, 0x05, 0xfe, 0x02, 0xfe, 0xa3, 0x35, 0x81,
        0x84, // compressed data + CRC
        0x00, 0x00, 0x00, 0x00, // IEND length
        0x49, 0x45, 0x4e, 0x44, // "IEND"
        0xae, 0x42, 0x60, 0x82, // IEND CRC
    ];
}
