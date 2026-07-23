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
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult,
};
use talos_permission::{PermissionDecision, PermissionEngine};
use talos_plugin::wasm::{LoadedPluginPackage, WasmRuntime, load_read_only_wasm_package};
use talos_session::{
    SessionManager, TodoAddDependencyTool, TodoCreateBatchTool, TodoCreateTool, TodoDeleteTool,
    TodoQueryTool, TodoRemoveDependencyTool, TodoUpdateBatchTool, TodoUpdateStatusTool,
    TodoUpdateTool,
};
use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::symbol::{FindReferencesTool, FindSymbolTool, ListImportsTool, ListSymbolsTool};
use talos_tools::{
    BashTool, DeleteTool, DiffTool, DocumentExtractTool, EditTool, ExecTool, FetchUrlTool,
    GlobTool, GrepTool, HttpRequestTool, LsTool, ReadImageTool, ReadTool, SaveUrlTool, StatTool,
    TreeTool, WebSearchTool, WriteTool, snapshot_aware_file_tools,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::approval::{ApprovalPrompt, add_always_allow_rules, always_allow_rule_descriptions};
use crate::colors;

/// Non-blocking approval handler for TUI mode.
///
/// Sends approval requests to the TUI via a channel and awaits responses
/// via oneshot channels. Unlike [`ApprovalPrompt`], this does not block
/// on stdin — the TUI renders an overlay and handles user interaction.
pub(crate) struct TuiApprovalHandler {
    ui_output_tx: mpsc::UnboundedSender<UiOutput>,
    engine: Arc<Mutex<PermissionEngine>>,
}

impl TuiApprovalHandler {
    pub(crate) fn new(
        ui_output_tx: mpsc::UnboundedSender<UiOutput>,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            ui_output_tx,
            engine: Arc::new(Mutex::new(PermissionEngine::with_workspace_root(
                workspace_root,
            ))),
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
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    /// Returns a shared handle to the permission engine so callers like
    /// the TUI bridge can evaluate image-attachment paths against the
    /// same SEC-001 rule set (P1-A).
    pub(crate) fn shared_engine(&self) -> Arc<Mutex<PermissionEngine>> {
        self.engine.clone()
    }

    async fn request_approval(
        &self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        evaluation_input: &serde_json::Value,
        presentation_input: &serde_json::Value,
        summary_fields: Vec<String>,
    ) -> ApprovalChoice {
        let decision = {
            let engine = self.engine.lock().expect("engine lock poisoned");
            engine.evaluate_profile(tool_name, profile, evaluation_input)
        };
        match decision {
            PermissionDecision::Allow => ApprovalChoice::ApproveOnce,
            PermissionDecision::Deny(_) => ApprovalChoice::Deny,
            PermissionDecision::Ask => {
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                let always_scopes =
                    always_allow_rule_descriptions(tool_name, profile, evaluation_input);
                let mut approval_arguments = presentation_input.clone();
                let mut approval_summary_fields = summary_fields;
                if !always_scopes.is_empty()
                    && let Some(obj) = approval_arguments.as_object_mut()
                {
                    obj.insert(
                        "_always_approve_scope".to_string(),
                        serde_json::Value::Array(
                            always_scopes
                                .into_iter()
                                .map(serde_json::Value::String)
                                .collect(),
                        ),
                    );
                    approval_summary_fields.push("_always_approve_scope".to_string());
                }

                if self
                    .ui_output_tx
                    .send(UiOutput::ToolApprovalRequest {
                        tool_name: tool_name.to_string(),
                        arguments: approval_arguments,
                        summary_fields: approval_summary_fields,
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

    fn add_always_allow_rules(
        &self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        input: &serde_json::Value,
    ) {
        let mut engine = self.engine.lock().expect("engine lock poisoned");
        add_always_allow_rules(&mut engine, tool_name, profile, input);
    }

    fn execution_authorizations(
        &self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        input: &Value,
        scope: ToolAuthorizationScope,
    ) -> Result<Vec<ToolExecutionAuthorization>, String> {
        self.engine
            .lock()
            .map_err(|_| "permission engine lock poisoned".to_string())?
            .execution_authorizations(tool_name, profile, input, scope)
            .map_err(|error| error.to_string())
    }
}

fn default_todo_tools(session_id: Uuid) -> Vec<Arc<dyn AgentTool>> {
    let Ok(sessions_dir) = SessionManager::default_sessions_dir() else {
        return Vec::new();
    };
    todo_tools_for_sessions_dir(&sessions_dir, session_id)
}

fn todo_tools_for_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Vec<Arc<dyn AgentTool>> {
    vec![
        Arc::new(TodoCreateTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoCreateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoUpdateStatusTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoUpdateTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoUpdateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoDeleteTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoAddDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoRemoveDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoQueryTool::from_sessions_dir(sessions_dir, session_id)),
    ]
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
            .request_approval(
                &tool_name,
                &profile,
                &input,
                &self.inner.project_input(&input),
                summary_fields,
            )
            .await;

        match choice {
            ApprovalChoice::ApproveOnce => {
                let authorizations = match self.approval.execution_authorizations(
                    &tool_name,
                    &profile,
                    &input,
                    ToolAuthorizationScope::Once,
                ) {
                    Ok(authorizations) => authorizations,
                    Err(error) => {
                        return ToolResult::error(format!(
                            "Permission denied: invalid execution authorization: {error}"
                        ));
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
                    .result
            }
            ApprovalChoice::AlwaysApprove => {
                self.approval
                    .add_always_allow_rules(&tool_name, &profile, &input);
                let authorizations = match self.approval.execution_authorizations(
                    &tool_name,
                    &profile,
                    &input,
                    ToolAuthorizationScope::Persisted,
                ) {
                    Ok(authorizations) => authorizations,
                    Err(error) => {
                        return ToolResult::error(format!(
                            "Permission denied: invalid execution authorization: {error}"
                        ));
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
                    .result
            }
            ApprovalChoice::Deny => ToolResult::error("Permission denied: User denied".to_string()),
        }
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
        let choice = self
            .approval
            .request_approval(
                &tool_name,
                &profile,
                &input,
                &self.inner.project_input(&input),
                summary_fields,
            )
            .await;

        match choice {
            ApprovalChoice::ApproveOnce => {
                let authorizations = match self.approval.execution_authorizations(
                    &tool_name,
                    &profile,
                    &input,
                    ToolAuthorizationScope::Once,
                ) {
                    Ok(authorizations) => authorizations,
                    Err(error) => {
                        return ToolExecutionOutput::error(format!(
                            "Permission denied: invalid execution authorization: {error}"
                        ));
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
            }
            ApprovalChoice::AlwaysApprove => {
                self.approval
                    .add_always_allow_rules(&tool_name, &profile, &input);
                let authorizations = match self.approval.execution_authorizations(
                    &tool_name,
                    &profile,
                    &input,
                    ToolAuthorizationScope::Persisted,
                ) {
                    Ok(authorizations) => authorizations,
                    Err(error) => {
                        return ToolExecutionOutput::error(format!(
                            "Permission denied: invalid execution authorization: {error}"
                        ));
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
            }
            ApprovalChoice::Deny => {
                ToolExecutionOutput::error("Permission denied: User denied".to_string())
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
                        let presentation_input = self.inner.project_input(&input);
                        match approval.prompt_profile(&tool_name, &profile, &presentation_input) {
                            Ok(decision) => decision,
                            Err(e) => PermissionDecision::Deny(format!("Approval error: {e}")),
                        }
                    }
                }
            }
        };

        match decision {
            PermissionDecision::Allow => {
                let authorizations = {
                    let approval = match self.approval.lock() {
                        Ok(approval) => approval,
                        Err(_) => {
                            return ToolResult::error("Permission denied: approval lock poisoned");
                        }
                    };
                    match approval.engine().execution_authorizations(
                        &tool_name,
                        &profile,
                        &input,
                        ToolAuthorizationScope::Persisted,
                    ) {
                        Ok(authorizations) => authorizations,
                        Err(error) => {
                            return ToolResult::error(format!(
                                "Permission denied: invalid execution authorization: {error}"
                            ));
                        }
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
                    .result
            }
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

    async fn execute_with_output(&self, input: Value) -> ToolExecutionOutput {
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
                        let presentation_input = self.inner.project_input(&input);
                        match approval.prompt_profile(&tool_name, &profile, &presentation_input) {
                            Ok(decision) => decision,
                            Err(e) => PermissionDecision::Deny(format!("Approval error: {e}")),
                        }
                    }
                }
            }
        };

        match decision {
            PermissionDecision::Allow => {
                let authorizations = {
                    let approval = match self.approval.lock() {
                        Ok(approval) => approval,
                        Err(_) => {
                            return ToolExecutionOutput::error(
                                "Permission denied: approval lock poisoned",
                            );
                        }
                    };
                    match approval.engine().execution_authorizations(
                        &tool_name,
                        &profile,
                        &input,
                        ToolAuthorizationScope::Persisted,
                    ) {
                        Ok(authorizations) => authorizations,
                        Err(error) => {
                            return ToolExecutionOutput::error(format!(
                                "Permission denied: invalid execution authorization: {error}"
                            ));
                        }
                    }
                };
                self.inner
                    .execute_authorized_with_output(input, &authorizations)
                    .await
            }
            PermissionDecision::Deny(reason) => {
                ToolExecutionOutput::error(format!("Permission denied: {reason}"))
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

fn load_explicit_plugin_tools(
    registry: &ToolRegistry,
    package_roots: &[PathBuf],
) -> Result<Vec<LoadedPluginTools>, String> {
    if package_roots.is_empty() {
        return Ok(Vec::new());
    }
    let runtime = Arc::new(
        WasmRuntime::new(100_000, 250)
            .map_err(|error| format!("failed to initialize WASM runtime: {error}"))?,
    );
    let mut loaded = Vec::with_capacity(package_roots.len());
    let mut pending_names = HashSet::new();
    for package_root in package_roots {
        let (tools, package) =
            load_read_only_wasm_package(runtime.clone(), package_root).map_err(|error| {
                format!(
                    "failed to load plugin package '{}': {error}",
                    package_root.display()
                )
            })?;
        for tool in &tools {
            let name = tool.name().to_string();
            if registry.get(&name).is_some() || !pending_names.insert(name.clone()) {
                return Err(format!(
                    "plugin tool name collides with registered tool: {name}"
                ));
            }
        }
        loaded.push((tools, package));
    }
    Ok(loaded)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the blocking/print permission adapter.
pub(crate) fn register_explicit_permission_aware_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<Mutex<ApprovalPrompt>>,
    print_mode: bool,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(registry, package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    for (tools, package) in loaded {
        register_permission_aware_tools(registry, &tools, approval.clone(), print_mode);
        packages.push(package);
    }
    Ok(packages)
}

/// Loads explicitly selected local packages and registers their tools behind
/// the non-blocking TUI permission adapter.
pub(crate) fn register_explicit_tui_plugins(
    registry: &mut ToolRegistry,
    package_roots: &[PathBuf],
    approval: Arc<TuiApprovalHandler>,
) -> Result<Vec<LoadedPluginPackage>, String> {
    let loaded = load_explicit_plugin_tools(registry, package_roots)?;
    let mut packages = Vec::with_capacity(loaded.len());
    for (tools, package) in loaded {
        register_tui_permission_aware_tools(registry, &tools, approval.clone());
        packages.push(package);
    }
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

/// Builds the tool registry for print/inline/RPC modes.
///
/// These modes construct a registry before any durable [`talos_session::Session`]
/// exists, so todo tools are bound to a fresh in-process session id — scoped to
/// this one run and discarded on exit, not persisted across invocations.
pub(crate) fn build_print_tool_registry(scheduler_tools: Vec<Arc<dyn AgentTool>>) -> ToolRegistry {
    let approval = Arc::new(Mutex::new(ApprovalPrompt::new(PermissionEngine::new())));
    let ephemeral_session_id = Uuid::new_v4();
    let (read_tool, write_tool, edit_tool, delete_tool) =
        snapshot_aware_file_tools(PathBuf::from("."));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(BashTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ExecTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(read_tool),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(ReadImageTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(DocumentExtractTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(write_tool),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(edit_tool),
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
        inner: Arc::new(delete_tool),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(SaveUrlTool::new()),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(FetchUrlTool::new()),
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
    for tool in default_todo_tools(ephemeral_session_id) {
        registry.register(Arc::new(PermissionAwareTool {
            inner: tool,
            approval: approval.clone(),
            print_mode: true,
        }));
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
    let mut registry = ToolRegistry::new();
    let (read_tool, write_tool, edit_tool, delete_tool) =
        snapshot_aware_file_tools(workspace_root.clone());
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(BashTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(ExecTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(read_tool),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(ReadImageTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(DocumentExtractTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(write_tool),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(edit_tool),
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
        inner: Arc::new(delete_tool),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(SaveUrlTool::new()),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(FetchUrlTool::new()),
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
    for tool in default_todo_tools(session_id) {
        registry.register(Arc::new(TuiPermissionAwareTool {
            inner: tool,
            approval: approval_handler.clone(),
        }));
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
    registry.register(Arc::new(BashTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ExecTool::new(PathBuf::from("."))));
    registry.register(Arc::new(ReadTool::new(PathBuf::from("."))));
    registry.register(Arc::new(DocumentExtractTool::new(PathBuf::from("."))));
    registry.register(Arc::new(WriteTool::new(PathBuf::from("."))));
    registry.register(Arc::new(EditTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GrepTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GlobTool::new(PathBuf::from("."))));
    registry.register(Arc::new(LsTool::new(PathBuf::from("."))));
    registry.register(Arc::new(DeleteTool::new(PathBuf::from("."))));
    registry.register(Arc::new(SaveUrlTool::new()));
    registry.register(Arc::new(FetchUrlTool::new()));
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
    fn print_and_tui_registries_include_todo_tools() {
        let dir = tempfile::tempdir().unwrap();
        let sessions_dir = dir.path().join("sessions");
        let session_id = Uuid::new_v4();

        let mut print_registry = ToolRegistry::new();
        for tool in todo_tools_for_sessions_dir(&sessions_dir, session_id) {
            print_registry.register(tool);
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
        for tool in todo_tools_for_sessions_dir(&sessions_dir, session_id) {
            tui_registry.register(Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: tui_approval.clone(),
            }));
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
        let dir = tempfile::tempdir().unwrap();
        let sessions_dir = dir.path().join("sessions");
        let session_id = Uuid::new_v4();

        let before_tools = todo_tools_for_sessions_dir(&sessions_dir, session_id);
        let create_tool = before_tools
            .iter()
            .find(|t| t.name() == "todo_create")
            .unwrap();
        let created = create_tool
            .execute(serde_json::json!({ "title": "survive model switch" }))
            .await;
        assert!(!created.is_error, "{}", created.content);

        // "After" registry: same session_id, entirely new tool instances.
        let after_tools = todo_tools_for_sessions_dir(&sessions_dir, session_id);
        let query_tool = after_tools
            .iter()
            .find(|t| t.name() == "todo_query")
            .unwrap();
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
            .unwrap();

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
            .unwrap();

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
            .unwrap();

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
        let _join = pending.spawn(sq_tx, tokio_util::sync::CancellationToken::new());

        let result = wrapped.execute(serde_json::json!({})).await;

        assert!(
            !result.is_error,
            "Read tool should be auto-allowed, not blocked by print mode: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_image_auto_allowed_for_read_nature() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.png");
        std::fs::write(&img_path, MINIMAL_PNG).unwrap();
        let canonical = img_path.canonicalize().unwrap();

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
            .unwrap(),
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
            .unwrap();

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
        let dir = tempfile::tempdir().unwrap();
        let img_a = dir.path().join("a.png");
        std::fs::write(&img_a, &[0x89, 0x50, 0x4E, 0x47]).unwrap();
        let img_b = dir.path().join("b.png");
        std::fs::write(&img_b, &[0x89, 0x50, 0x4E, 0x47]).unwrap();

        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let auth = vec![
            talos_core::tool::ToolExecutionAuthorization::for_path(
                "read_image",
                talos_core::tool::ToolNature::Read,
                dir.path(),
                "a.png",
                talos_core::tool::ToolAuthorizationScope::Once,
            )
            .unwrap(),
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
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "decision": "Ask",
                    "nature": "Read"
                }]
            }))
            .unwrap();

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
        assert!(
            result.content.to_lowercase().contains("unavailable")
                || result.content.to_lowercase().contains("print mode"),
            "Ask in print mode should auto-deny: {}",
            result.content
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
