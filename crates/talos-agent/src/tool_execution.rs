use std::hash::{Hash, Hasher};
use std::sync::Arc;

use futures_util::future::join_all;
use talos_core::background_job::{BackgroundJobPermit, ToolExecutionAdmission};
use talos_core::message::{
    AgentEvent, ContentPart, Message, MessageToolResult, StopReason, ToolCall,
};
use talos_core::tool::{
    ToolExecutionAuthorization, ToolPresentationPolicy, ToolProvenance, ToolRegistry,
    ToolResult as ToolExecutionResult,
};
use talos_permission::PermissionDecision;
use talos_plugin::{
    HookContext, HookEvent, HookOutcome, ToolObservation, TurnEndReason, TurnStatus,
};
use talos_sandbox::{SandboxConfig, SandboxError, SandboxProvider, SandboxResult};
use tokio::sync::Semaphore;
use tokio::sync::mpsc;

use crate::compression::BashOutputCompressor;
use crate::permission_pipeline::PermissionAuthorizationRequest;
use crate::transient_text::PrivateTokens;
use crate::{
    Agent, AgentError, AgentResult, MAX_CONCURRENT_READ_ONLY, PendingToolCall,
    SandboxFallbackContext, SandboxFallbackDecision, SandboxFallbackPolicy,
};

/// Retained across asynchronous approval and final execution gates.
#[derive(PartialEq)]
struct InvocationExecutionContext {
    permission: talos_permission::PermissionStateSnapshot,
    configuration: serde_json::Value,
    directories: Vec<(
        std::path::PathBuf,
        crate::auto_resolver::ExecutionDirectoryIdentity,
    )>,
    directories_resolved: bool,
    sandbox_available: Option<bool>,
    sandbox_fallback: Option<(SandboxFallbackPolicy, bool)>,
}

impl InvocationExecutionContext {
    fn review_configuration(&self) -> serde_json::Value {
        serde_json::json!({
            "tool_configuration": self.configuration,
            "execution_directories": self.directories,
            "execution_directories_resolved": self.directories_resolved,
            "sandbox": self.sandbox_available.map(|available| serde_json::json!({
                "available": available,
                "allow_network": false,
                "extra_read_paths": [],
                "fallback_policy": self.sandbox_fallback.map(|(policy, _)| match policy {
                    SandboxFallbackPolicy::Deny => "deny",
                    SandboxFallbackPolicy::Ask => "separate-once-approval",
                    SandboxFallbackPolicy::AllowUnsandboxed => "unsandboxed-after-permission",
                }),
                "fallback_handler_available": self.sandbox_fallback.map(|(_, available)| available),
                "fallback_configuration": self.configuration,
            })),
        })
    }
    fn capture(
        agent: &Agent,
        tool: &dyn talos_core::tool::AgentTool,
        input: &serde_json::Value,
    ) -> Option<Self> {
        let permission = agent
            .permission_pipeline
            .as_ref()?
            .state()
            .state_snapshot()
            .ok()?;
        let configuration = tool.invocation_review_configuration()?;
        let resolved_paths = tool.invocation_execution_directories(input);
        let directories_resolved = resolved_paths.is_some();
        // Existing sequential commands may create a later step's cwd. Preserve
        // their ordinary permission path, but disclose incomplete review evidence.
        let mut paths = resolved_paths
            .unwrap_or_else(|| tool.execution_working_directory().into_iter().collect());
        let sandbox_available = if tool.name() == "bash" {
            agent.sandbox.as_ref().map(|sandbox| {
                paths.push(agent.workspace_root.clone());
                sandbox.is_available()
            })
        } else {
            None
        };
        let directories = paths
            .into_iter()
            .map(|path| {
                crate::auto_resolver::execution_directory_identity(&path)
                    .map(|identity| (path, identity))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            permission,
            configuration,
            directories,
            directories_resolved,
            sandbox_available,
            sandbox_fallback: sandbox_available.map(|_| {
                (
                    agent.sandbox_fallback_policy,
                    agent.sandbox_fallback_handler.is_some(),
                )
            }),
        })
    }

    fn matches(
        &self,
        agent: &Agent,
        tool: &dyn talos_core::tool::AgentTool,
        input: &serde_json::Value,
    ) -> bool {
        Self::capture(agent, tool, input).as_ref() == Some(self)
    }
}

impl Agent {
    /// Executes a batch of tool calls, running read-only tools concurrently
    /// (up to [`MAX_CONCURRENT_READ_ONLY`]) and write tools serially.
    ///
    /// Each tool call goes through the security pipeline: permission check,
    /// sandbox execution (for bash tools), and direct execution.
    ///
    /// Results are returned in the same order as the input calls.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_tools_with_presentation(
        &self,
        hook_ctx: &HookContext,
        calls: &[ToolCall],
        user_intent: Option<&str>,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
        private_tokens: &PrivateTokens,
    ) -> AgentResult<(Vec<ToolExecutionResult>, Vec<ContentPart>)> {
        if calls.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let deduped: Vec<ToolCall> = calls
            .iter()
            .filter(|c| seen.insert((c.name.clone(), c.input.to_string())))
            .cloned()
            .collect();
        let calls: &[ToolCall] = &deduped;

        let mut results: Vec<Option<ToolExecutionResult>> = vec![None; calls.len()];
        let mut all_parts: Vec<ContentPart> = Vec::new();
        let read_image_quota = std::sync::atomic::AtomicUsize::new(0);

        let read_only_indices: Vec<usize> = calls
            .iter()
            .enumerate()
            .filter(|(_, call)| {
                self.tools
                    .get(&call.name)
                    .map(|t| t.is_read_only())
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        let write_indices: Vec<usize> = calls
            .iter()
            .enumerate()
            .filter(|(_, call)| {
                !self
                    .tools
                    .get(&call.name)
                    .map(|t| t.is_read_only())
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();

        if !read_only_indices.is_empty() {
            let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_READ_ONLY));
            let registry = &self.tools;
            let futures: Vec<_> = read_only_indices
                .iter()
                .map(|&idx| {
                    let call = &calls[idx];
                    let sem = semaphore.clone();
                    let agent = self;
                    let ctx = hook_ctx.clone();
                    let quota = &read_image_quota;
                    async move {
                        let _permit = sem.acquire().await.expect("semaphore closed");
                        let result = agent
                            .execute_single_tool_with_presentation(
                                &ctx,
                                registry,
                                call,
                                user_intent,
                                policy,
                                presented_tool_names,
                                quota,
                                private_tokens,
                            )
                            .await;
                        (idx, result)
                    }
                })
                .collect();

            for (idx, result) in join_all(futures).await {
                let (result, parts) = result?;
                results[idx] = Some(result);
                all_parts.extend(parts);
            }
        }

        for idx in write_indices {
            let call = &calls[idx];
            let (result, parts) = self
                .execute_single_tool_with_presentation(
                    hook_ctx,
                    &self.tools,
                    call,
                    user_intent,
                    policy,
                    presented_tool_names,
                    &read_image_quota,
                    private_tokens,
                )
                .await?;
            results[idx] = Some(result);
            all_parts.extend(parts);
        }

        Ok((
            results
                .into_iter()
                .map(|r| r.expect("all results should be populated"))
                .collect(),
            all_parts,
        ))
    }
    pub(crate) fn tool_call_event(
        &self,
        call: &ToolCall,
        provenance: &ToolProvenance,
    ) -> AgentEvent {
        let tool = self.tools.get(&call.name);
        let summary_fields = tool
            .as_ref()
            .map(|tool| {
                tool.summary_fields()
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect()
            })
            .unwrap_or_default();
        let projected_call = self.project_tool_call(call);

        AgentEvent::ToolCall {
            call: projected_call,
            provenance: provenance.clone(),
            summary_fields,
        }
    }

    pub(crate) fn project_tool_call(&self, call: &ToolCall) -> ToolCall {
        let mut projected = call.clone();
        if let Some(tool) = self.tools.get(&call.name) {
            projected.input = tool.project_input(&call.input);
        }
        projected
    }

    pub(crate) fn project_tool_result(
        &self,
        tool_name: &str,
        result: &ToolExecutionResult,
    ) -> ToolExecutionResult {
        let content = self
            .tools
            .get(tool_name)
            .map(|tool| tool.project_result(result).persistence_content)
            .unwrap_or_else(|| result.content.clone());
        ToolExecutionResult {
            content,
            is_error: result.is_error,
            continuations: result.continuations.clone(),
            // Observer hooks do not receive or manufacture trusted display geometry.
            todo_presentation: None,
        }
    }

    pub(crate) fn restore_private_call_if_unchanged(
        original: &ToolCall,
        projected: &ToolCall,
        observed: &ToolCall,
    ) -> ToolCall {
        if observed == projected {
            original.clone()
        } else {
            observed.clone()
        }
    }

    pub(crate) fn restore_private_result_if_unchanged(
        original: &ToolExecutionResult,
        projected: &ToolExecutionResult,
        observed: &ToolExecutionResult,
    ) -> ToolExecutionResult {
        if observed.content == projected.content && observed.is_error == projected.is_error {
            ToolExecutionResult {
                content: original.content.clone(),
                is_error: original.is_error,
                continuations: observed.continuations.clone(),
                todo_presentation: if !original.is_error && original.content == projected.content {
                    original.todo_presentation.clone()
                } else {
                    None
                },
            }
        } else {
            let mut result = observed.clone();
            result.todo_presentation = None;
            result
        }
    }

    pub(crate) fn pending_calls_with_provenance(
        &self,
        calls: &[ToolCall],
        proposed: &[PendingToolCall],
    ) -> Vec<PendingToolCall> {
        calls
            .iter()
            .map(|call| {
                let provenance = proposed
                    .iter()
                    .find(|pending| pending.call.id == call.id)
                    .map(|pending| pending.provenance.clone())
                    .unwrap_or_default();
                PendingToolCall {
                    call: call.clone(),
                    provenance,
                }
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_tools_for_ui_with_presentation(
        &self,
        hook_ctx: &HookContext,
        calls: &[PendingToolCall],
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
        messages: &mut Vec<Message>,
        user_intent: Option<&str>,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
        private_tokens: &mut PrivateTokens,
    ) -> AgentResult<(Vec<ToolExecutionResult>, Vec<ContentPart>)> {
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let deduped: Vec<PendingToolCall> = calls
            .iter()
            .filter(|pending| {
                seen.insert((pending.call.name.clone(), pending.call.input.to_string()))
            })
            .cloned()
            .collect();

        let mut results = Vec::with_capacity(deduped.len());
        let mut all_parts: Vec<ContentPart> = Vec::new();
        let read_image_quota = std::sync::atomic::AtomicUsize::new(0);
        for pending in deduped {
            let mut visible_call = pending.call.clone();
            private_tokens.project_value(&mut visible_call.input);
            let _ = event_tx.send(self.tool_call_event(&visible_call, &pending.provenance));

            let (result, parts) = self
                .execute_single_tool_with_presentation(
                    hook_ctx,
                    &self.tools,
                    &pending.call,
                    user_intent,
                    policy,
                    presented_tool_names,
                    &read_image_quota,
                    private_tokens,
                )
                .await?;
            all_parts.extend(parts);
            results.push(
                self.observe_tool_result(
                    hook_ctx,
                    &pending.call,
                    result,
                    event_tx,
                    messages,
                    private_tokens,
                )
                .await?,
            );
        }

        Ok((results, all_parts))
    }

    pub(crate) async fn observe_tool_result(
        &self,
        hook_ctx: &HookContext,
        call: &ToolCall,
        result: ToolExecutionResult,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
        messages: &mut Vec<Message>,
        private_tokens: &mut PrivateTokens,
    ) -> AgentResult<ToolExecutionResult> {
        if let Some(tool) = self.tools.get(&call.name) {
            private_tokens.extend(tool.model_private_tokens(&result));
        }
        let mut projected_call = self.project_tool_call(call);
        private_tokens.project_value(&mut projected_call.input);
        let mut projected_result = self.project_tool_result(&call.name, &result);
        projected_result.content = private_tokens.project(&projected_result.content);
        let observation = ToolObservation {
            call: projected_call.clone(),
            result: projected_result.clone(),
        };
        let observed = match self
            .run_hook(
                hook_ctx,
                HookEvent::OnToolResultObserved {
                    observation: &observation,
                },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::OnToolResultObserved { observation }))
            | Ok(HookOutcome::Skip(HookEvent::OnToolResultObserved { observation })) => {
                observation.clone()
            }
            Ok(_) => observation,
            Err(error) => return Err(error),
        };
        let observed = ToolObservation {
            call: Self::restore_private_call_if_unchanged(call, &projected_call, &observed.call),
            result: Self::restore_private_result_if_unchanged(
                &result,
                &projected_result,
                &observed.result,
            ),
        };

        let projection = self
            .tools
            .get(&observed.call.name)
            .map(|tool| tool.project_result(&observed.result))
            .unwrap_or_else(|| {
                talos_core::tool::ToolResultProjection::shared(observed.result.content.clone())
            });
        let ui_result = MessageToolResult {
            tool_use_id: observed.call.id.clone(),
            content: private_tokens.project(&projection.display_content),
            is_error: observed.result.is_error,
        };
        let original_only = self
            .tools
            .get(&call.name)
            .is_some_and(|tool| tool.requires_original_invocation());
        let llm_result = if observed.result.is_error && !original_only {
            MessageToolResult {
                content: format!(
                    "{}\n\n[Analyze the error above and try a different approach.]",
                    projection.model_content
                ),
                ..ui_result.clone()
            }
        } else if self.bash_compression_enabled
            && matches!(observed.call.name.as_str(), "bash" | "powershell")
        {
            let compressed = BashOutputCompressor::new().compress(&projection.model_content);
            MessageToolResult {
                content: compressed.content,
                ..ui_result.clone()
            }
        } else {
            MessageToolResult {
                content: projection.model_content,
                ..ui_result.clone()
            }
        };
        messages.push(Message::Tool { result: llm_result });
        let todo_presentation = if !ui_result.is_error
            && observed.call == *call
            && self.tools.get(&call.name).is_some_and(|tool| {
                tool.produces_todo_presentation()
                    && tool.provenance() == talos_core::tool::ToolProvenance::Native
            }) {
            use sha2::{Digest, Sha256};
            observed
                .result
                .todo_presentation
                .as_ref()
                .and_then(|metadata| {
                    metadata.bind_visible_content(
                        ui_result.content.clone(),
                        Sha256::digest(ui_result.content.as_bytes()).into(),
                    )
                })
        } else {
            None
        };
        let _ = event_tx.send(AgentEvent::ToolResult {
            result: ui_result,
            todo_presentation,
        });
        Ok(observed.result)
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_single_tool_with_presentation(
        &self,
        hook_ctx: &HookContext,
        registry: &ToolRegistry,
        call: &ToolCall,
        user_intent: Option<&str>,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
        read_image_quota: &std::sync::atomic::AtomicUsize,
        private_tokens: &PrivateTokens,
    ) -> AgentResult<(ToolExecutionResult, Vec<ContentPart>)> {
        let original_call = call.clone();
        let mut projected_call = self.project_tool_call(call);
        private_tokens.project_value(&mut projected_call.input);
        let effective_call = match self
            .run_hook(
                hook_ctx,
                HookEvent::BeforeToolCall {
                    call: &projected_call,
                },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeToolCall {
                call: observed_call,
            })) => Some(Self::restore_private_call_if_unchanged(
                &original_call,
                &projected_call,
                observed_call,
            )),
            Ok(HookOutcome::Skip(_)) => {
                return Ok((ToolExecutionResult::success(String::new()), Vec::new()));
            }
            Ok(_) => Some(original_call),
            Err(error) => return Err(error),
        };
        let mut call = effective_call.expect("tool call should be present");

        let tool = match registry.get(&call.name) {
            Some(t) => t,
            None => {
                return Ok((
                    ToolExecutionResult::error(format!("tool not found: {}", call.name)),
                    Vec::new(),
                ));
            }
        };
        if tool.requires_original_invocation() {
            return Ok((
                ToolExecutionResult::error("protected tool requires original prepared invocation"),
                Vec::new(),
            ));
        }
        if self.enforce_tool_presentation_policy && !presented_tool_names.contains(&call.name) {
            return Ok((
                ToolExecutionResult::error(format!(
                    "tool family not loaded for '{}'; continue with a presented tool or request the relevant tool family",
                    call.name
                )),
                Vec::new(),
            ));
        }
        call.input = crate::permission_pipeline::normalize_permission_input(&call.name, call.input);
        if let Err(e) = registry.validate_input(&call.name, &call.input) {
            return Ok((
                ToolExecutionResult::error(format!("invalid input for {e}")),
                Vec::new(),
            ));
        }
        if self.enforce_tool_presentation_policy
            && let Some(backend) = tool.backend_for_input(&call.input)
            && !policy.allows_backend(&call.name, &backend)
        {
            return Ok((
                ToolExecutionResult::error(format!(
                    "tool backend '{backend}' for '{}' is not loaded; continue with a disclosed backend or retry the base tool path",
                    call.name
                )),
                Vec::new(),
            ));
        }

        let prepared_network = if tool.requires_network_invocation() {
            if self.permission_pipeline.is_none() {
                return Ok((
                    ToolExecutionResult::error(
                        "protected network tool requires the Agent permission pipeline",
                    ),
                    Vec::new(),
                ));
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tool.prepare_network_invocation(&call.input)
            })) {
                Ok(Ok(Some(prepared))) => Some(prepared),
                Ok(Err(error)) => {
                    return Ok((ToolExecutionResult::error(error.to_string()), Vec::new()));
                }
                _ => {
                    return Ok((
                        ToolExecutionResult::error("network_preparation_unavailable"),
                        Vec::new(),
                    ));
                }
            }
        } else {
            None
        };
        let mut prepared = if prepared_network.is_some() {
            None
        } else {
            match tool.prepare_normalized_with_provenance(&call.input, tool.provenance()) {
                Ok(prepared) => prepared,
                Err(error) => return Ok((ToolExecutionResult::error(error), Vec::new())),
            }
        };
        if prepared.is_some() && call.name == "bash" {
            return Ok((
                ToolExecutionResult::error(
                    "prepared operation cannot replace bash sandbox dispatch",
                ),
                Vec::new(),
            ));
        }
        if prepared.is_some() && self.permission_pipeline.is_none() {
            return Ok((
                ToolExecutionResult::error(
                    "prepared operation requires the Agent permission pipeline",
                ),
                Vec::new(),
            ));
        }
        // Prepared profiles are reserved by the stage budget before any cloning below.
        // Prepared operations cannot use background dispatch.
        let permission_profile = if let Some(network) = prepared_network.as_ref() {
            network.permission_profile().to_vec()
        } else if prepared.is_none() {
            tool.permission_profile(&call.input)
        } else {
            Vec::new()
        };
        let background_start_resource = permission_profile.iter().find_map(|facet| {
            facet
                .resource
                .as_ref()
                .filter(|resource| resource.starts_with("background:"))
                .cloned()
        });
        let mut background_permit: Option<Box<dyn BackgroundJobPermit>> = match tool
            .execution_admission(&call.input)
        {
            Ok(ToolExecutionAdmission::Foreground) => None,
            Ok(ToolExecutionAdmission::Background(request)) => {
                if prepared.is_some() {
                    return Ok((
                        ToolExecutionResult::error(
                            "prepared operation cannot use background dispatch",
                        ),
                        Vec::new(),
                    ));
                }
                if self.permission_pipeline.is_none() {
                    return Ok((
                        ToolExecutionResult::error(
                            "background execution requires the Agent permission pipeline",
                        ),
                        Vec::new(),
                    ));
                }
                if call.name == "bash"
                    && self
                        .sandbox
                        .as_deref()
                        .is_some_and(SandboxProvider::is_available)
                {
                    return Ok((
                        ToolExecutionResult::error(
                            "background execution is unavailable because the configured sandbox does not expose a managed process handle",
                        ),
                        Vec::new(),
                    ));
                }
                let Some(host) = self.background_jobs.as_deref() else {
                    return Ok((
                        ToolExecutionResult::error(
                            "background execution requires a live AppServerSession",
                        ),
                        Vec::new(),
                    ));
                };
                match host
                    .reserve_with_permission_resource(request, background_start_resource.clone())
                    .await
                {
                    Ok(permit) => Some(permit),
                    Err(error) => {
                        return Ok((ToolExecutionResult::error(error), Vec::new()));
                    }
                }
            }
            Err(error) => return Ok((ToolExecutionResult::error(error), Vec::new())),
        };

        if call.name == "read_image" && !self.image_input_supported {
            return Ok((
                ToolExecutionResult::error(
                    "read_image is not available for the current model; image input is not supported."
                        .to_string(),
                ),
                Vec::new(),
            ));
        }

        if call.name == "read_image" {
            let prev = read_image_quota.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if prev > 0 {
                return Ok((
                    ToolExecutionResult::error(
                        "Only one image can be read per tool batch; \
                         a previous read_image already produced an image artifact."
                            .to_string(),
                    ),
                    Vec::new(),
                ));
            }
        }

        let mut prepared_output = None;
        let mut prepared_reserved = false;
        let mut stage_history = talos_core::tool::PreparedStageHistory::default();
        let prepared_deadline_at = prepared
            .as_ref()
            .map(|_| tokio::time::Instant::now() + self.permission_deadline);
        let prepared_invocation_deadline_at = prepared
            .as_ref()
            .map(|_| tokio::time::Instant::now() + talos_core::tool::PREPARED_INVOCATION_TIMEOUT);
        let mut authorizations: Option<Vec<ToolExecutionAuthorization>> = None;
        let mut network_authority = None;
        let mut execution_context = None;
        let permission_allowed = loop {
            if let Some(operation) = prepared.as_ref() {
                let Some(identity) = operation.plan_identity().cloned() else {
                    return Ok((
                        ToolExecutionResult::error("prepared plan identity required"),
                        Vec::new(),
                    ));
                };
                if !stage_history.record_profile(&identity, operation.permission_profile()) {
                    return Ok((
                        ToolExecutionResult::error(
                            "prepared stage limit or reused identity; authorization profile budget may be exhausted",
                        ),
                        Vec::new(),
                    ));
                }
            }

            let permission_profile = prepared.as_ref().map_or_else(
                || permission_profile.clone(),
                |operation| operation.permission_profile().to_vec(),
            );
            let mut prepared_admission: Option<talos_permission::AdmittedInvocation> = None;
            let permission_allowed = if let Some(pipeline) = self.permission_pipeline.as_deref() {
                let permission_deadline_at = prepared_deadline_at
                    .unwrap_or_else(|| tokio::time::Instant::now() + self.permission_deadline);
                if tokio::time::Instant::now() >= permission_deadline_at {
                    return Ok((
                        ToolExecutionResult::error("Permission denied: approval deadline exceeded"),
                        Vec::new(),
                    ));
                }
                let mut permission_call = ToolCall {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    input: crate::permission_pipeline::project_permission_input(&call.input),
                };
                private_tokens.project_value(&mut permission_call.input);
                for event in [
                    HookEvent::OnToolCallProposed {
                        call: &permission_call,
                    },
                    HookEvent::BeforePermissionCheck {
                        call: &permission_call,
                    },
                ] {
                    match tokio::time::timeout_at(
                        permission_deadline_at,
                        self.hook_registry.dispatch_permission_gate(hook_ctx, event),
                    )
                    .await
                    {
                        Ok(HookOutcome::Continue(_)) => {}
                        Ok(HookOutcome::Deny { reason, .. }) => {
                            return Ok((
                                ToolExecutionResult::error(format!("Permission denied: {reason}")),
                                Vec::new(),
                            ));
                        }
                        Ok(HookOutcome::Skip(_)) => {
                            return Ok((
                                ToolExecutionResult::error(
                                    "Permission denied: permission hook failed closed",
                                ),
                                Vec::new(),
                            ));
                        }
                        Err(_) => {
                            return Ok((
                                ToolExecutionResult::error(
                                    "Permission denied: approval deadline exceeded",
                                ),
                                Vec::new(),
                            ));
                        }
                    }
                }

                let working_directory = tool.execution_working_directory();
                if prepared.is_none()
                    && prepared_network.is_none()
                    && matches!(
                        call.name.as_str(),
                        "bash" | "powershell" | "exec" | "http_request" | "fetch_url"
                    )
                    && tool.invocation_review_configuration().is_some()
                {
                    execution_context =
                        InvocationExecutionContext::capture(self, tool, &call.input);
                    if execution_context.is_none() {
                        return Ok((
                            ToolExecutionResult::error("invocation execution context unavailable"),
                            Vec::new(),
                        ));
                    }
                }
                let mut presentation_input = tool.project_input(&call.input);
                private_tokens.project_value(&mut presentation_input);
                let authorization_request = PermissionAuthorizationRequest {
                    tool_name: &call.name,
                    provenance: tool.provenance(),
                    profile: &permission_profile,
                    input: &call.input,
                    presentation_input,
                    summary_fields: tool
                        .summary_fields()
                        .iter()
                        .map(|field| (*field).to_string())
                        .collect(),
                    deadline: permission_deadline_at
                        .saturating_duration_since(tokio::time::Instant::now()),
                };
                let authorization = if let Some(network) = prepared_network.as_ref() {
                    pipeline
                        .authorize_registered_network(
                            authorization_request,
                            network.manifest(),
                            user_intent,
                            registry.is_builtin_instance(&call.name),
                        )
                        .await
                        .map(|authority| {
                            network_authority = Some(authority);
                            Vec::new()
                        })
                } else if let Some(operation) = prepared.as_ref() {
                    let Some(plan) = operation.plan_identity() else {
                        return Ok((
                            ToolExecutionResult::error("prepared plan identity required"),
                            Vec::new(),
                        ));
                    };
                    let evidence = operation.prepared_command_evidence();
                    let mutation = operation.prepared_file_mutation_evidence();
                    let configuration = operation.invocation_review_configuration();
                    if usize::from(evidence.is_some())
                        + usize::from(mutation.is_some())
                        + usize::from(configuration.is_some())
                        > 1
                    {
                        return Ok((
                            ToolExecutionResult::error("ambiguous prepared review evidence"),
                            Vec::new(),
                        ));
                    }
                    let admission = if let Some(mutation) = mutation.as_ref() {
                        pipeline
                            .authorize_registered_file_mutation(
                                authorization_request,
                                user_intent,
                                plan,
                                mutation,
                                registry.is_builtin_instance(&call.name),
                            )
                            .await
                    } else if let Some(configuration) = configuration.as_ref() {
                        pipeline
                            .authorize_registered_prepared_invocation(
                                authorization_request,
                                user_intent,
                                plan,
                                tool.description(),
                                configuration,
                                registry.is_builtin_instance(&call.name),
                            )
                            .await
                    } else {
                        pipeline
                            .authorize_prepared_plan_with_evidence(
                                authorization_request,
                                user_intent,
                                plan,
                                evidence
                                    .as_ref()
                                    .filter(|_| registry.is_builtin_instance(&call.name)),
                            )
                            .await
                    };
                    admission.map(|admission| {
                        prepared_admission = Some(admission);
                        Vec::new()
                    })
                } else if matches!(call.name.as_str(), "bash" | "powershell")
                    && !registry.is_builtin_instance(&call.name)
                    && working_directory.is_some()
                {
                    let working_directory = working_directory.as_deref().expect("checked above");
                    pipeline
                        .authorize_with_execution_context(
                            authorization_request,
                            user_intent,
                            Some(working_directory),
                        )
                        .await
                } else if registry.is_builtin_instance(&call.name) {
                    let review_configuration = execution_context
                        .as_ref()
                        .map(|context| context.review_configuration())
                        .or_else(|| tool.invocation_review_configuration());
                    pipeline
                        .authorize_registered_invocation(
                            authorization_request,
                            user_intent,
                            working_directory.as_deref(),
                            tool.description(),
                            review_configuration.as_ref(),
                        )
                        .await
                } else {
                    pipeline
                        .authorize_with_user_intent(authorization_request, user_intent)
                        .await
                };
                let decision = authorization
                    .as_ref()
                    .err()
                    .map_or(PermissionDecision::Allow, |error| error.final_decision());

                match tokio::time::timeout_at(
                    permission_deadline_at,
                    self.hook_registry.dispatch_permission_gate(
                        hook_ctx,
                        HookEvent::AfterPermissionCheck {
                            call: &permission_call,
                            decision: decision.clone(),
                        },
                    ),
                )
                .await
                {
                    Ok(HookOutcome::Continue(_)) => {}
                    Ok(HookOutcome::Deny { reason, .. }) => {
                        return Ok((
                            ToolExecutionResult::error(format!("Permission denied: {reason}")),
                            Vec::new(),
                        ));
                    }
                    Ok(HookOutcome::Skip(_)) => {
                        return Ok((
                            ToolExecutionResult::error(
                                "Permission denied: final hook failed closed",
                            ),
                            Vec::new(),
                        ));
                    }
                    Err(_) => {
                        return Ok((
                            ToolExecutionResult::error(
                                "Permission denied: approval deadline exceeded",
                            ),
                            Vec::new(),
                        ));
                    }
                }

                match authorization {
                    Ok(grants) => authorizations = Some(grants),
                    Err(error) => {
                        return Ok((
                            ToolExecutionResult::error(format!("Permission denied: {error}")),
                            Vec::new(),
                        ));
                    }
                }
                true
            } else {
                true
            };

            let Some(operation) = prepared.take() else {
                break permission_allowed;
            };
            if !prepared_reserved {
                if self
                    .execution_ledger
                    .reserve(hook_ctx.turn_id, &call)
                    .is_err()
                {
                    return Ok((
                        ToolExecutionResult::error(
                            "tool call was already admitted or may have started; refusing replay",
                        ),
                        Vec::new(),
                    ));
                }
                prepared_reserved = true;
            }
            let Some(admission) = prepared_admission.take() else {
                return Ok((
                    ToolExecutionResult::error("prepared operation has no permission admission"),
                    Vec::new(),
                ));
            };
            if let Some(receipt) = operation.cleanup_receipt() {
                self.tool_cleanup.register(hook_ctx.turn_id, receipt);
            }
            let Some(deadline_at) = prepared_invocation_deadline_at else {
                return Ok((
                    ToolExecutionResult::error("prepared invocation deadline unavailable"),
                    Vec::new(),
                ));
            };
            if tokio::time::Instant::now() >= deadline_at {
                return Ok((
                    ToolExecutionResult::error(
                        "prepared invocation deadline exceeded; effects may be incomplete",
                    ),
                    Vec::new(),
                ));
            }
            let stage = match tokio::time::timeout_at(
                deadline_at,
                operation.execute_stage(Box::new(admission)),
            )
            .await
            {
                Ok(stage) => stage,
                Err(_) => {
                    return Ok((
                        ToolExecutionResult::error(
                            "prepared invocation deadline exceeded; effects may be incomplete",
                        ),
                        Vec::new(),
                    ));
                }
            };
            match stage {
                talos_core::tool::PreparedNormalStage::Complete(output) => {
                    prepared_output = Some(output);
                    break permission_allowed;
                }
                talos_core::tool::PreparedNormalStage::Continue(next) => prepared = Some(next),
            }
        };

        let execution_context_matches = || {
            execution_context
                .as_ref()
                .is_none_or(|context| context.matches(self, tool, &call.input))
        };
        if !execution_context_matches() {
            return Ok((
                ToolExecutionResult::error(
                    "invocation execution context changed after approval; submit a new tool call",
                ),
                Vec::new(),
            ));
        }

        // Reserve the logical call only after final permission admission and
        // before crossing the actual tool execution boundary. If execution is
        // cancelled or times out after this point, retaining the reservation is
        // intentional: the side effect may already have reached the tool.
        if !prepared_reserved
            && let Err(error) = self.execution_ledger.reserve(hook_ctx.turn_id, &call)
        {
            let message = match error {
                crate::execution_ledger::ReservationError::AlreadyReserved => {
                    "tool call was already admitted or may have started; refusing replay"
                }
                crate::execution_ledger::ReservationError::IdentityConflict => {
                    "tool call id was reused with different arguments; refusing execution"
                }
            };
            return Ok((ToolExecutionResult::error(message), Vec::new()));
        }

        let normalized_input = call.input.clone();

        let (mut result, parts) = if let Some(network) = prepared_network {
            let Some(authority) = network_authority else {
                return Ok((
                    ToolExecutionResult::error("network_authority_unavailable"),
                    Vec::new(),
                ));
            };
            let output = network.execute(authority).await;
            (output.result, output.next_provider_parts)
        } else if let Some(output) = prepared_output {
            (output.result, output.next_provider_parts)
        } else if call.name == "bash" {
            if let Some(sb) = self.sandbox.as_deref() {
                if sb.is_available() {
                    (
                        self.execute_bash_in_sandbox_bound(
                            hook_ctx,
                            sb,
                            &normalized_input,
                            &execution_context_matches,
                        )
                        .await,
                        Vec::new(),
                    )
                } else {
                    let fallback = match self.sandbox_fallback_policy {
                        SandboxFallbackPolicy::Deny => false,
                        SandboxFallbackPolicy::AllowUnsandboxed => permission_allowed,
                        SandboxFallbackPolicy::Ask if permission_allowed => {
                            match &self.sandbox_fallback_handler {
                                Some(handler) => {
                                    let context = SandboxFallbackContext {
                                        tool_name: call.name.clone(),
                                        arguments: redacted_fallback_arguments(
                                            tool,
                                            &normalized_input,
                                        ),
                                        summary_fields: tool
                                            .summary_fields()
                                            .iter()
                                            .map(|field| (*field).to_string())
                                            .collect(),
                                    };
                                    let decision = handler.request_fallback(context).await;
                                    tracing::debug!(
                                        tool = %call.name,
                                        decision = ?decision,
                                        "sandbox fallback decision"
                                    );
                                    matches!(decision, SandboxFallbackDecision::ApproveOnce)
                                }
                                None => false,
                            }
                        }
                        SandboxFallbackPolicy::Ask => false,
                    };
                    if fallback {
                        if !execution_context_matches() {
                            return Ok((
                                ToolExecutionResult::error(
                                    "invocation execution context changed during fallback; submit a new tool call",
                                ),
                                Vec::new(),
                            ));
                        }
                        let output = match background_permit.take() {
                            Some(permit) => {
                                tool.execute_background_authorized_with_output(
                                    normalized_input,
                                    permit,
                                    authorizations.as_deref().unwrap_or(&[]),
                                )
                                .await
                            }
                            None => match authorizations.as_deref() {
                                Some(grants) => {
                                    tool.execute_authorized_with_output(normalized_input, grants)
                                        .await
                                }
                                None => tool.execute_with_output(normalized_input).await,
                            },
                        };
                        (output.result, output.next_provider_parts)
                    } else {
                        (
                            ToolExecutionResult::error(
                                "sandbox unavailable and unsandboxed fallback was denied"
                                    .to_owned(),
                            ),
                            Vec::new(),
                        )
                    }
                }
            } else {
                let output = match background_permit.take() {
                    Some(permit) => {
                        tool.execute_background_authorized_with_output(
                            normalized_input,
                            permit,
                            authorizations.as_deref().unwrap_or(&[]),
                        )
                        .await
                    }
                    None => match authorizations.as_deref() {
                        Some(grants) => {
                            tool.execute_authorized_with_output(normalized_input, grants)
                                .await
                        }
                        None => tool.execute_with_output(normalized_input).await,
                    },
                };
                (output.result, output.next_provider_parts)
            }
        } else {
            let output = match background_permit.take() {
                Some(permit) => {
                    tool.execute_background_authorized_with_output(
                        normalized_input,
                        permit,
                        authorizations.as_deref().unwrap_or(&[]),
                    )
                    .await
                }
                None => match authorizations.as_deref() {
                    Some(grants) => {
                        tool.execute_authorized_with_output(normalized_input, grants)
                            .await
                    }
                    None => tool.execute_with_output(normalized_input).await,
                },
            };
            (output.result, output.next_provider_parts)
        };

        // Eligibility is an explicit registered producer capability, never a
        // match on a tool name or data supplied by an extension.
        if result.is_error
            || !tool.produces_todo_presentation()
            || tool.provenance() != talos_core::tool::ToolProvenance::Native
        {
            result.todo_presentation = None;
        }

        self.execution_ledger.complete(hook_ctx.turn_id, &call);

        let mut private_tokens = private_tokens.clone();
        if let Some(tool) = registry.get(&call.name) {
            private_tokens.extend(tool.model_private_tokens(&result));
        }
        let mut projected_call = self.project_tool_call(&call);
        private_tokens.project_value(&mut projected_call.input);
        let mut projected_result = self.project_tool_result(&call.name, &result);
        projected_result.content = private_tokens.project(&projected_result.content);
        let original_result = result;
        let result = match self
            .run_hook(
                hook_ctx,
                HookEvent::AfterToolCall {
                    call: &projected_call,
                    result: &projected_result,
                },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::AfterToolCall {
                result: observed_result,
                ..
            }))
            | Ok(HookOutcome::Skip(HookEvent::AfterToolCall {
                result: observed_result,
                ..
            })) => Self::restore_private_result_if_unchanged(
                &original_result,
                &projected_result,
                observed_result,
            ),
            Ok(_) => original_result,
            Err(error) => return Err(error),
        };

        Ok((result, parts))
    }

    /// Executes a bash command through the sandbox provider.
    ///
    /// Extracts the `command` field from the tool input and runs it within
    /// the sandbox environment. Returns a [`ToolExecutionResult`] with the
    /// combined stdout/stderr output.
    #[cfg(test)]
    pub(crate) async fn execute_bash_in_sandbox(
        &self,
        hook_ctx: &HookContext,
        sandbox: &dyn SandboxProvider,
        input: &serde_json::Value,
    ) -> ToolExecutionResult {
        self.execute_bash_in_sandbox_bound(hook_ctx, sandbox, input, &|| true)
            .await
    }

    async fn execute_bash_in_sandbox_bound(
        &self,
        hook_ctx: &HookContext,
        sandbox: &dyn SandboxProvider,
        input: &serde_json::Value,
        execution_context_matches: &(dyn Fn() -> bool + Sync),
    ) -> ToolExecutionResult {
        let command = match input.get("command").and_then(serde_json::Value::as_str) {
            Some(cmd) => cmd.to_owned(),
            None => {
                return ToolExecutionResult::error(
                    "bash tool input missing required field 'command'".to_owned(),
                );
            }
        };

        let directory_identity =
            crate::auto_resolver::execution_directory_identity(&self.workspace_root);
        let permission_state = self
            .permission_pipeline
            .as_ref()
            .map(|pipeline| pipeline.state());
        let permission_snapshot = permission_state
            .as_ref()
            .map(|state| state.state_snapshot().ok());
        if directory_identity.is_none() || permission_snapshot == Some(None) {
            return ToolExecutionResult::error("sandbox execution identity unavailable");
        }

        let command = match self
            .run_hook(
                hook_ctx,
                HookEvent::BeforeBashSandboxExec { command: &command },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeBashSandboxExec { command })) => {
                if input.get("command").and_then(serde_json::Value::as_str) != Some(command) {
                    return ToolExecutionResult::error(
                        "sandbox hook changed the admitted command; submit a new tool call",
                    );
                }
                command.to_string()
            }
            Ok(HookOutcome::Skip(_)) => return ToolExecutionResult::success(String::new()),
            Ok(_) => command,
            Err(_) => return ToolExecutionResult::error("hook denied bash execution".to_owned()),
        };

        if !execution_context_matches()
            || crate::auto_resolver::execution_directory_identity(&self.workspace_root)
                != directory_identity
            || permission_state
                .as_ref()
                .map(|state| state.state_snapshot().ok())
                != permission_snapshot
        {
            return ToolExecutionResult::error(
                "sandbox execution context changed during hook; submit a new tool call",
            );
        }

        let config = SandboxConfig {
            workspace_root: self.workspace_root.clone(),
            allow_network: false,
            extra_read_paths: vec![],
        };

        let start = std::time::Instant::now();

        let result = match sandbox.execute(&command, &config).await {
            Ok(result) => Self::sandbox_result_to_tool_result(result),
            Err(SandboxError::NotAvailable) => {
                ToolExecutionResult::error("sandbox became unavailable during execution".to_owned())
            }
            Err(SandboxError::ExecutionFailed(reason)) => {
                ToolExecutionResult::error(format!("sandbox execution failed: {reason}"))
            }
            Err(SandboxError::PermissionDenied(reason)) => {
                ToolExecutionResult::error(format!("sandbox permission denied: {reason}"))
            }
        };

        let exit = if result.is_error { 1 } else { 0 };
        let _ = self
            .run_hook(
                hook_ctx,
                HookEvent::AfterBashSandboxExec {
                    exit,
                    duration: start.elapsed(),
                },
            )
            .await;

        result
    }

    pub(crate) async fn run_hook<'a>(
        &self,
        hook_ctx: &HookContext,
        event: HookEvent<'a>,
    ) -> AgentResult<HookOutcome<'a>> {
        let outcome = self.hook_registry.dispatch(hook_ctx, event).await;
        if let HookOutcome::Deny { reason, .. } = &outcome {
            return Err(AgentError::HookDenied(reason.clone()));
        }
        Ok(outcome)
    }

    pub(crate) async fn emit_turn_complete(&self, hook_ctx: &HookContext, status: TurnStatus) {
        let _ = self
            .hook_registry
            .dispatch(
                hook_ctx,
                HookEvent::TurnComplete {
                    turn_id: hook_ctx.turn_id,
                    status,
                },
            )
            .await;
    }

    pub(crate) fn turn_end_reason(stop_reason: StopReason) -> TurnEndReason {
        match stop_reason {
            StopReason::EndTurn => TurnEndReason::EndTurn,
            StopReason::ToolUse => TurnEndReason::ToolUse,
            StopReason::MaxTokens => TurnEndReason::MaxTokens,
        }
    }

    /// Converts a [`SandboxResult`] to a [`ToolExecutionResult`].
    pub(crate) fn sandbox_result_to_tool_result(result: SandboxResult) -> ToolExecutionResult {
        let mut content = String::new();
        if !result.stdout.is_empty() {
            content.push_str(&result.stdout);
        }
        if !result.stderr.is_empty() {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&result.stderr);
        }

        ToolExecutionResult {
            content,
            is_error: result.exit_code != 0,
            continuations: Vec::new(),
            todo_presentation: None,
        }
    }
}

fn redacted_fallback_arguments(
    tool: &dyn talos_core::tool::AgentTool,
    input: &serde_json::Value,
) -> serde_json::Value {
    let projected = tool.project_input(input);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    projected.to_string().hash(&mut hasher);
    let fields = projected
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    serde_json::json!({
        "redacted": true,
        "fingerprint": format!("{:016x}", hasher.finish()),
        "fields": fields,
    })
}

#[allow(dead_code)]
fn format_access_evidence_diagnostic(ev: &talos_permission::AccessEvidence) -> String {
    use talos_permission::{AccessKind, EvidenceState};
    let kind_str = match ev.kind {
        AccessKind::Read => "read",
        AccessKind::Write => "write",
        AccessKind::Delete => "delete",
        AccessKind::Spawn => "spawn",
        AccessKind::Network => "network",
        AccessKind::Unknown => "unknown",
    };
    let state_str = match ev.state {
        EvidenceState::Declared => "declared",
        EvidenceState::Observed => "observed",
        EvidenceState::Unknown => "unknown",
    };
    let paths_str = if ev.paths.is_empty() {
        String::new()
    } else {
        format!(
            " paths=[{}]",
            ev.paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    format!("{kind_str}:{state_str}{paths_str}")
}

#[allow(dead_code)]
fn collect_access_evidence_diagnostics(
    call: &ToolCall,
) -> Vec<(String, talos_permission::AccessEvidence)> {
    if matches!(call.name.as_str(), "bash" | "powershell") {
        return call
            .input
            .get("command")
            .and_then(serde_json::Value::as_str)
            .map(|command| {
                vec![(
                    command.to_string(),
                    talos_permission::classify_command_access(command),
                )]
            })
            .unwrap_or_default();
    }
    if call.name != "exec" {
        return Vec::new();
    }

    let mut commands = Vec::new();
    if let Some(command) = call
        .input
        .get("command")
        .and_then(serde_json::Value::as_str)
    {
        commands.push(classify_exec_argv(command, call.input.get("args")));
    }
    if let Some(steps) = call
        .input
        .get("steps")
        .and_then(serde_json::Value::as_array)
    {
        commands.extend(steps.iter().filter_map(classify_exec_step));
    }
    if let Some(pipes) = call
        .input
        .get("pipes")
        .and_then(serde_json::Value::as_array)
    {
        for pipe in pipes {
            if let Some(steps) = pipe.get("steps").and_then(serde_json::Value::as_array) {
                commands.extend(steps.iter().filter_map(classify_exec_step));
            }
        }
    }
    commands
}

#[allow(dead_code)]
fn classify_exec_step(
    step: &serde_json::Value,
) -> Option<(String, talos_permission::AccessEvidence)> {
    let command = step.get("command")?.as_str()?;
    Some(classify_exec_argv(command, step.get("args")))
}

#[allow(dead_code)]
fn classify_exec_argv(
    command: &str,
    args: Option<&serde_json::Value>,
) -> (String, talos_permission::AccessEvidence) {
    let args = args
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let display = std::iter::once(command)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    let structurally_simple = std::iter::once(command)
        .chain(args.iter().copied())
        .all(|part| {
            !part.is_empty()
                && !part.chars().any(char::is_whitespace)
                && !part.contains(['|', ';', '&', '>', '<', '`', '$', '\\', '\'', '"'])
        });
    let evidence = if structurally_simple {
        talos_permission::classify_command_access(&display)
    } else {
        talos_permission::AccessEvidence::unknown()
    };
    (display, evidence)
}

#[cfg(test)]
mod access_evidence_tests {
    use super::*;
    use serde_json::json;

    fn call(name: &str, input: serde_json::Value) -> ToolCall {
        ToolCall {
            id: "test".to_string(),
            name: name.to_string(),
            input,
        }
    }

    #[test]
    fn bash_production_input_produces_one_diagnostic() {
        let evidence = collect_access_evidence_diagnostics(&call(
            "bash",
            json!({"command": "find . -delete"}),
        ));
        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].1.is_unknown());
    }

    #[test]
    fn powershell_production_input_produces_one_diagnostic() {
        let evidence = collect_access_evidence_diagnostics(&call(
            "powershell",
            json!({"command": "Remove-Item -Recurse ."}),
        ));
        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].1.is_unknown());
    }

    #[test]
    fn exec_single_steps_and_pipes_all_produce_diagnostics() {
        let single = collect_access_evidence_diagnostics(&call(
            "exec",
            json!({"command": "cat", "args": ["Cargo.toml"]}),
        ));
        assert_eq!(single.len(), 1);

        let steps = collect_access_evidence_diagnostics(&call(
            "exec",
            json!({"steps": [
                {"command": "cat", "args": ["Cargo.toml"]},
                {"command": "find", "args": [".", "-delete"]}
            ]}),
        ));
        assert_eq!(steps.len(), 2);
        assert!(steps[1].1.is_unknown());

        let pipes = collect_access_evidence_diagnostics(&call(
            "exec",
            json!({"pipes": [{"steps": [
                {"command": "cat", "args": ["Cargo.toml"]},
                {"command": "wc", "args": ["-l"]}
            ]}]}),
        ));
        assert_eq!(pipes.len(), 2);
    }

    #[test]
    fn exec_argv_with_shell_like_or_spaced_argument_is_unknown() {
        for args in [json!(["a b"]), json!(["$(touch", "x)"]), json!(["a;b"])] {
            let evidence = collect_access_evidence_diagnostics(&call(
                "exec",
                json!({"command": "printf", "args": args}),
            ));
            assert!(evidence[0].1.is_unknown());
        }
    }
}
