//! MCP server handler exposing Talos tools.

use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorCode,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool, ToolAnnotations,
};
use talos_core::tool::ToolRegistry;
use talos_plugin::{HookContext, TurnId};

use super::permission::McpPermissionGate;

/// MCP handler that republishes Talos tools to MCP clients.
///
/// Create one handler per connection. Prepared invocation request IDs remain
/// reserved for its lifetime, including cancelled or indeterminate operations.
pub struct TalosMcpHandler {
    tool_registry: Arc<ToolRegistry>,
    permission_gate: Arc<McpPermissionGate>,
    prepared_requests: Mutex<HashSet<rmcp::model::RequestId>>,
    closing: AtomicBool,
    cleanup_failed: AtomicBool,
    owns_cleanup_lifecycle: bool,
    local_receipts: Mutex<Vec<Arc<dyn talos_core::tool::ToolCleanupReceipt>>>,
    active: Arc<(AtomicUsize, tokio::sync::Notify)>,
}

const MAX_PREPARED_REQUESTS: usize = 4096;
const MAX_PREPARED_REQUEST_ID_BYTES: usize = 1024;

fn reserve_prepared_request(
    requests: &Mutex<HashSet<rmcp::model::RequestId>>,
    id: &rmcp::model::RequestId,
) -> Result<(), McpError> {
    if matches!(id, rmcp::model::RequestId::String(value) if value.len() > MAX_PREPARED_REQUEST_ID_BYTES)
    {
        return Err(McpError::new(
            ErrorCode::INVALID_PARAMS,
            "prepared request ID exceeds limit",
            None,
        ));
    }
    let mut requests = requests.lock().map_err(|_| {
        McpError::new(
            ErrorCode::INTERNAL_ERROR,
            "prepared request ledger unavailable",
            None,
        )
    })?;
    if requests.contains(id) {
        return Err(McpError::new(
            ErrorCode::INVALID_REQUEST,
            "prepared request ID already used; execution will not be replayed",
            None,
        ));
    }
    if requests.len() >= MAX_PREPARED_REQUESTS {
        return Err(McpError::new(
            ErrorCode::INTERNAL_ERROR,
            "prepared request ledger full; establish a new connection",
            None,
        ));
    }
    requests.insert(id.clone());
    Ok(())
}

impl TalosMcpHandler {
    /// Creates a connection-local handler without owning the registry lifecycle.
    /// Shutdown waits registered prepared invocation receipts. The registry host
    /// remains responsible for aggregate cleanup of other tool execution paths.
    #[must_use]
    pub fn new(tool_registry: Arc<ToolRegistry>, permission_gate: Arc<McpPermissionGate>) -> Self {
        Self::new_with_owner_lifecycle(tool_registry, permission_gate, false)
    }

    /// Creates a handler that may seal shared tool owners during shutdown.
    /// Set this only for the composition root that owns the registry lifecycle.
    pub fn new_with_owner_lifecycle(
        tool_registry: Arc<ToolRegistry>,
        permission_gate: Arc<McpPermissionGate>,
        owns_cleanup_lifecycle: bool,
    ) -> Self {
        Self {
            tool_registry,
            permission_gate,
            prepared_requests: Mutex::new(HashSet::new()),
            closing: AtomicBool::new(false),
            cleanup_failed: AtomicBool::new(false),
            owns_cleanup_lifecycle,
            local_receipts: Mutex::new(Vec::new()),
            active: Arc::new((AtomicUsize::new(0), tokio::sync::Notify::new())),
        }
    }

    /// Closes connection admission and waits for its handlers and tool cleanup.
    /// Registry owners are sealed and awaited only with explicit lifecycle ownership.
    /// The connection owner must first request transport cancellation, then call
    /// this method while Tokio remains alive. Drop alone cannot confirm cleanup.
    pub async fn shutdown(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), talos_core::tool::ToolCleanupError> {
        use futures_util::FutureExt;
        use talos_core::tool::ToolCleanupError;
        self.closing.store(true, Ordering::SeqCst);
        let deadline = tokio::time::Instant::now() + timeout;
        let owners = self.tool_registry.cleanup_receipts();
        if self.owns_cleanup_lifecycle {
            for owner in &owners {
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| owner.seal())).is_err()
                {
                    self.cleanup_failed.store(true, Ordering::SeqCst);
                }
            }
        }
        loop {
            let changed = self.active.1.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.active.0.load(Ordering::SeqCst) == 0 {
                break;
            }
            tokio::time::timeout_at(deadline, changed)
                .await
                .map_err(|_| ToolCleanupError::Timeout)?;
        }
        let mut receipts = self
            .local_receipts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if self.owns_cleanup_lifecycle {
            receipts.extend(
                owners
                    .into_iter()
                    .map(|owner| owner as Arc<dyn talos_core::tool::ToolCleanupReceipt>),
            );
        }
        for owner in receipts {
            let observed = std::panic::AssertUnwindSafe(async {
                owner
                    .wait(deadline.saturating_duration_since(tokio::time::Instant::now()))
                    .await
            })
            .catch_unwind();
            match tokio::time::timeout_at(deadline, observed).await {
                Ok(Ok(Ok(()))) => {}
                Err(_) => return Err(ToolCleanupError::Timeout),
                _ => self.cleanup_failed.store(true, Ordering::SeqCst),
            }
        }
        if self.cleanup_failed.load(Ordering::SeqCst) {
            Err(ToolCleanupError::Unconfirmed)
        } else {
            Ok(())
        }
    }

    fn all_tools(&self) -> Vec<Tool> {
        // TODO: I009-future OAuth/auth for MCP server requests.
        // TODO: I009-future rate limiting for multi-client/high-throughput MCP usage.
        self.tool_registry
            .list()
            .into_iter()
            .filter(|tool| !tool.requires_original_invocation())
            .map(to_mcp_tool)
            .collect()
    }
}

impl ServerHandler for TalosMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some("Talos MCP server exposing local tools".to_string());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.all_tools()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_registry
            .get(name)
            .filter(|tool| !tool.requires_original_invocation())
            .map(to_mcp_tool)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        if self.closing.load(Ordering::SeqCst) {
            return Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                "MCP connection is closing",
                None,
            ));
        }
        self.active.0.fetch_add(1, Ordering::SeqCst);
        struct ActiveGuard(Arc<(AtomicUsize, tokio::sync::Notify)>);
        impl Drop for ActiveGuard {
            fn drop(&mut self) {
                self.0.0.fetch_sub(1, Ordering::SeqCst);
                self.0.1.notify_waiters();
            }
        }
        let _active_guard = ActiveGuard(self.active.clone());
        // A shutdown racing the first check must not admit this handler.
        if self.closing.load(Ordering::SeqCst) {
            return Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                "MCP connection is closing",
                None,
            ));
        }

        let hook_context = HookContext::new(TurnId::new(), std::path::PathBuf::from("."));
        let Some(tool) = self.tool_registry.get(request.name.as_ref()) else {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("tool not found: {}", request.name),
                None,
            ));
        };
        // MCP has already parsed arguments, so duplicate-key evidence is irrecoverable.
        if tool.requires_original_invocation() {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                "tool unavailable through standalone MCP".to_owned(),
                None,
            ));
        }

        let input = request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));
        let input = talos_agent::permission_pipeline::normalize_permission_input(
            request.name.as_ref(),
            input,
        );
        self.tool_registry
            .validate_input(request.name.as_ref(), &input)
            .map_err(|error| {
                McpError::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("invalid tool input: {error}"),
                    None,
                )
            })?;
        let talos_request = crate::types::McpCallRequest {
            name: request.name.to_string(),
            arguments: input.as_object().cloned(),
        };
        let registered_builtin = self
            .tool_registry
            .is_builtin_instance(request.name.as_ref());
        let prepared = tool
            .prepare_normalized_with_provenance(
                &input,
                if registered_builtin {
                    talos_core::tool::ToolProvenance::Native
                } else {
                    talos_core::tool::ToolProvenance::McpRemote {
                        server: "standalone-mcp".into(),
                    }
                },
            )
            .map_err(|error| McpError::new(ErrorCode::INVALID_PARAMS, error, None))?;
        let result = match prepared {
            Some(operation) => {
                let mut operation_receipts = Vec::new();
                let _plan = operation.plan_identity().cloned().ok_or_else(|| {
                    McpError::new(ErrorCode::INTERNAL_ERROR, "prepared plan identity required", None)
                })?;
                // Reserve before awaiting admission. Never remove on cancellation,
                // denial, or an ambiguous execution result; no inputs are retained.
                reserve_prepared_request(&self.prepared_requests, &context.id)?;
                let prepared_permission_deadline_at = self.permission_gate.prepared_deadline_at();
                let prepared_invocation_deadline_at = tokio::time::Instant::now()
                    + talos_core::tool::PREPARED_INVOCATION_TIMEOUT;
                let execution_result = tokio::select! {
                    biased;
                    _ = context.ct.cancelled() => {
                        Err(McpError::new(
                            ErrorCode::INTERNAL_ERROR,
                            "prepared request cancelled; effects may be incomplete; request ID remains reserved",
                            None,
                        ))
                    }
                    result = async {
                        let mut operation = operation;
                        let mut stage_history = talos_core::tool::PreparedStageHistory::default();
                        loop {
                            if tokio::time::Instant::now() >= prepared_invocation_deadline_at {
                                return Err(McpError::new(ErrorCode::INTERNAL_ERROR,
                                    "prepared invocation deadline exceeded; effects may be incomplete", None));
                            }
                            if let Some(receipt) = operation.cleanup_receipt() {
                                self.local_receipts.lock().unwrap_or_else(|e| e.into_inner()).push(receipt.clone());
                                operation_receipts.push(receipt);
                            }
                            let plan = operation.plan_identity().cloned().ok_or_else(|| McpError::new(ErrorCode::INTERNAL_ERROR, "prepared plan identity required", None))?;
                            if !stage_history.record_profile(&plan, operation.permission_profile()) {
                                return Err(McpError::new(ErrorCode::INTERNAL_ERROR, "prepared stage limit or reused identity; authorization profile budget may be exhausted", None));
                            }
                            let profile = operation.permission_profile().to_vec();
                            // A matching name or caller-provided provenance cannot
                            // establish ownership of automatic-review evidence.
                            let evidence = operation.prepared_command_evidence();
                            let mutation = operation.prepared_file_mutation_evidence();
                            let configuration = operation.invocation_review_configuration();
                            if usize::from(evidence.is_some()) + usize::from(mutation.is_some())
                                + usize::from(configuration.is_some()) > 1 {
                                return Err(McpError::new(ErrorCode::INTERNAL_ERROR,
                                    "ambiguous prepared review evidence", None));
                            }
                            let review = super::permission::PreparedReview {
                                plan: &plan,
                                command: evidence.as_ref().filter(|_| registered_builtin),
                                mutation: mutation.as_ref().filter(|_| registered_builtin),
                                configuration: configuration.as_ref().filter(|_| registered_builtin),
                                description: tool.description(),
                                builtin: registered_builtin,
                            };
                            let admission = self.permission_gate
                                .authorize_prepared_command_until(&hook_context, &talos_request, &profile, review, prepared_permission_deadline_at).await?;
                            if context.ct.is_cancelled() || tokio::time::Instant::now() >= prepared_invocation_deadline_at {
                                return Err(McpError::new(ErrorCode::INTERNAL_ERROR,
                                    "prepared invocation cancelled or expired; effects may be incomplete", None));
                            }
                            let stage = tokio::time::timeout_at(prepared_invocation_deadline_at,
                                operation.execute_stage(Box::new(admission)))
                                .await
                                .map_err(|_| McpError::new(ErrorCode::INTERNAL_ERROR,
                                    "prepared invocation deadline exceeded; effects may be incomplete", None))?;
                            match stage {
                                talos_core::tool::PreparedNormalStage::Complete(output) => break Ok::<_, McpError>(output),
                                talos_core::tool::PreparedNormalStage::Continue(next) => operation = next,
                            }
                        }
                    } => result,
                };
                // select's execution future is dropped before waiting. No
                // detached invocation task survives this scope.
                let cleanup_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
                for receipt in operation_receipts {
                    use futures_util::FutureExt;
                    let observed = std::panic::AssertUnwindSafe(async {
                        receipt.wait(cleanup_deadline.saturating_duration_since(tokio::time::Instant::now())).await
                    }).catch_unwind();
                    if !matches!(tokio::time::timeout_at(cleanup_deadline, observed).await, Ok(Ok(Ok(())))) {
                        self.cleanup_failed.store(true, Ordering::SeqCst);
                        return Err(McpError::new(ErrorCode::INTERNAL_ERROR,
                            "tool cleanup was not confirmed; effects may be incomplete; request ID remains reserved", None));
                    }
                    self.local_receipts.lock().unwrap_or_else(|e| e.into_inner())
                        .retain(|saved| !Arc::ptr_eq(saved, &receipt));
                }
                execution_result?

            }
            None => {
                let profile = tool.permission_profile(&input);
                let authorizations = self
                    .permission_gate
                    .authorize_call(&hook_context, &talos_request, &profile)
                    .await?;
                tool.execute_authorized_with_output(input, &authorizations)
                    .await
            }
        }
        .result;

        let content = vec![ContentBlock::text(result.content)];
        if result.is_error {
            Ok(CallToolResponse::Complete(CallToolResult::error(content)))
        } else {
            Ok(CallToolResponse::Complete(CallToolResult::success(content)))
        }
    }
}

fn to_mcp_tool(tool: &dyn talos_core::tool::AgentTool) -> Tool {
    let input_schema = tool.parameters().as_object().cloned().unwrap_or_default();

    Tool::new(
        Cow::Owned(tool.name().to_string()),
        Cow::Owned(tool.description().to_string()),
        Arc::new(input_schema),
    )
    .with_annotations(ToolAnnotations::new().read_only(tool.is_read_only()))
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use rmcp::model::RequestId;

    #[test]
    fn concurrent_duplicate_reservation_has_one_winner() {
        let ledger = Mutex::new(HashSet::new());
        let accepted = std::thread::scope(|scope| {
            let tasks: Vec<_> = (0..16)
                .map(|_| {
                    scope.spawn(|| reserve_prepared_request(&ledger, &RequestId::Number(7)).is_ok())
                })
                .collect();
            tasks
                .into_iter()
                .map(|task| usize::from(task.join().expect("worker")))
                .sum::<usize>()
        });
        assert_eq!(accepted, 1);
        assert_eq!(ledger.lock().expect("ledger").len(), 1);
        assert!(reserve_prepared_request(&ledger, &RequestId::Number(7)).is_err());
        assert!(reserve_prepared_request(&ledger, &RequestId::Number(8)).is_ok());
    }

    #[test]
    fn ledger_never_evicts_and_connection_instances_are_independent() {
        let ledger = Mutex::new(HashSet::new());
        for id in 0..MAX_PREPARED_REQUESTS {
            reserve_prepared_request(&ledger, &RequestId::Number(id as i64)).expect("reserve");
        }
        assert!(reserve_prepared_request(&ledger, &RequestId::Number(0)).is_err());
        assert!(
            reserve_prepared_request(&ledger, &RequestId::Number(MAX_PREPARED_REQUESTS as i64))
                .is_err()
        );
        let other_connection = Mutex::new(HashSet::new());
        assert!(reserve_prepared_request(&other_connection, &RequestId::Number(0)).is_ok());
    }

    #[test]
    fn string_ids_are_bounded_and_not_aliased_to_numbers() {
        let ledger = Mutex::new(HashSet::new());
        assert!(reserve_prepared_request(&ledger, &RequestId::Number(7)).is_ok());
        assert!(reserve_prepared_request(&ledger, &RequestId::String("7".into())).is_ok());
        assert!(
            reserve_prepared_request(
                &ledger,
                &RequestId::String("x".repeat(MAX_PREPARED_REQUEST_ID_BYTES + 1).into())
            )
            .is_err()
        );
        assert_eq!(ledger.lock().expect("ledger").len(), 2);
    }
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;
    use async_trait::async_trait;
    use talos_core::tool::{
        AgentTool, ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt, ToolResult,
    };
    #[tokio::test]
    async fn shutdown_does_not_report_success_while_a_handler_is_active() {
        let handler = TalosMcpHandler::new(
            Arc::new(ToolRegistry::new()),
            Arc::new(McpPermissionGate::new(
                Arc::new(talos_permission::PermissionEngine::new()),
                Arc::new(talos_plugin::HookRegistry::new()),
            )),
        );
        handler.active.0.store(1, Ordering::SeqCst);
        assert_eq!(
            handler.shutdown(std::time::Duration::from_millis(1)).await,
            Err(ToolCleanupError::Timeout)
        );
        assert!(handler.closing.load(Ordering::SeqCst));
        handler.active.0.store(0, Ordering::SeqCst);
        handler.active.1.notify_waiters();
        handler
            .shutdown(std::time::Duration::from_secs(1))
            .await
            .expect("confirmed idle");
        handler.cleanup_failed.store(true, Ordering::SeqCst);
        assert_eq!(
            handler.shutdown(std::time::Duration::from_secs(1)).await,
            Err(ToolCleanupError::Unconfirmed)
        );
    }
    struct Probe(AtomicBool);
    #[async_trait]
    impl ToolCleanupReceipt for Probe {
        async fn wait(&self, _: std::time::Duration) -> Result<(), ToolCleanupError> {
            Ok(())
        }
    }
    impl ToolCleanupOwner for Probe {
        fn seal(&self) {
            if !self.0.swap(true, Ordering::SeqCst) {
                panic!("first seal failure");
            }
        }
    }
    struct Tool(Arc<Probe>);
    #[async_trait]
    impl AgentTool for Tool {
        fn name(&self) -> &str {
            "cleanup_fixture"
        }
        fn description(&self) -> &str {
            "fixture"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({"type":"object"})
        }
        fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupOwner>> {
            Some(self.0.clone())
        }
        async fn execute(&self, _: serde_json::Value) -> ToolResult {
            ToolResult::success("unused")
        }
    }
    struct PendingReceipt;
    #[async_trait]
    impl ToolCleanupReceipt for PendingReceipt {
        async fn wait(&self, _: std::time::Duration) -> Result<(), ToolCleanupError> {
            std::future::pending().await
        }
    }

    #[tokio::test]
    async fn shared_connections_wait_only_their_own_retained_receipts() {
        let owner = Arc::new(Probe(AtomicBool::new(false)));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(Tool(owner.clone())));
        let registry = Arc::new(registry);
        let gate = Arc::new(McpPermissionGate::new(
            Arc::new(talos_permission::PermissionEngine::new()),
            Arc::new(talos_plugin::HookRegistry::new()),
        ));
        let first = TalosMcpHandler::new(registry.clone(), gate.clone());
        let second = TalosMcpHandler::new(registry, gate);
        second
            .local_receipts
            .lock()
            .expect("receipts")
            .push(Arc::new(PendingReceipt));
        first
            .shutdown(std::time::Duration::from_millis(20))
            .await
            .expect("unrelated receipt must not block");
        assert!(
            !owner.0.load(Ordering::SeqCst),
            "shared owner must remain open"
        );
        assert!(!second.closing.load(Ordering::SeqCst));
        assert_eq!(
            second.shutdown(std::time::Duration::from_millis(20)).await,
            Err(ToolCleanupError::Timeout)
        );
        assert!(!owner.0.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn exclusive_shutdown_also_waits_receipts_without_registry_owner() {
        let handler = TalosMcpHandler::new_with_owner_lifecycle(
            Arc::new(ToolRegistry::new()),
            Arc::new(McpPermissionGate::new(
                Arc::new(talos_permission::PermissionEngine::new()),
                Arc::new(talos_plugin::HookRegistry::new()),
            )),
            true,
        );
        handler
            .local_receipts
            .lock()
            .expect("receipts")
            .push(Arc::new(PendingReceipt));
        assert_eq!(
            handler.shutdown(std::time::Duration::from_millis(20)).await,
            Err(ToolCleanupError::Timeout)
        );
    }

    #[tokio::test]
    async fn seal_panic_remains_failure_after_later_successful_seal() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(Tool(Arc::new(Probe(AtomicBool::new(false))))));
        let handler = TalosMcpHandler::new_with_owner_lifecycle(
            Arc::new(registry),
            Arc::new(McpPermissionGate::new(
                Arc::new(talos_permission::PermissionEngine::new()),
                Arc::new(talos_plugin::HookRegistry::new()),
            )),
            true,
        );
        for _ in 0..2 {
            assert_eq!(
                handler.shutdown(std::time::Duration::from_secs(1)).await,
                Err(ToolCleanupError::Unconfirmed)
            );
        }
        assert!(handler.closing.load(Ordering::SeqCst));
    }
}
