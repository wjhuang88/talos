use std::sync::Arc;

use futures_util::future::join_all;
use talos_core::message::{AgentEvent, Message, MessageToolResult, StopReason, ToolCall};
use talos_core::tool::{
    ToolPresentationPolicy, ToolProvenance, ToolRegistry, ToolResult as ToolExecutionResult,
};
use talos_permission::PermissionDecision;
use talos_plugin::{
    HookContext, HookEvent, HookOutcome, ToolObservation, TurnEndReason, TurnStatus,
};
use talos_sandbox::{SandboxConfig, SandboxError, SandboxProvider, SandboxResult};
use tokio::sync::Semaphore;
use tokio::sync::mpsc;

use crate::compression::BashOutputCompressor;
use crate::{Agent, AgentError, AgentResult, MAX_CONCURRENT_READ_ONLY, PendingToolCall};

impl Agent {
    /// Executes a batch of tool calls, running read-only tools concurrently
    /// (up to [`MAX_CONCURRENT_READ_ONLY`]) and write tools serially.
    ///
    /// Each tool call goes through the security pipeline: permission check,
    /// sandbox execution (for bash tools), and direct execution.
    ///
    /// Results are returned in the same order as the input calls.
    pub(crate) async fn execute_tools_with_presentation(
        &self,
        hook_ctx: &HookContext,
        calls: &[ToolCall],
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
    ) -> AgentResult<Vec<ToolExecutionResult>> {
        if calls.is_empty() {
            return Ok(Vec::new());
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
                    async move {
                        let _permit = sem.acquire().await.expect("semaphore closed");
                        let result = agent
                            .execute_single_tool_with_presentation(
                                &ctx,
                                registry,
                                call,
                                policy,
                                presented_tool_names,
                            )
                            .await;
                        (idx, result)
                    }
                })
                .collect();

            for (idx, result) in join_all(futures).await {
                results[idx] = Some(result?);
            }
        }

        for idx in write_indices {
            let call = &calls[idx];
            let result = self
                .execute_single_tool_with_presentation(
                    hook_ctx,
                    &self.tools,
                    call,
                    policy,
                    presented_tool_names,
                )
                .await?;
            results[idx] = Some(result);
        }

        Ok(results
            .into_iter()
            .map(|r| r.expect("all results should be populated"))
            .collect())
    }

    pub(crate) fn tool_call_event(
        &self,
        call: &ToolCall,
        provenance: &ToolProvenance,
    ) -> AgentEvent {
        let summary_fields = self
            .tools
            .get(&call.name)
            .map(|tool| {
                tool.summary_fields()
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect()
            })
            .unwrap_or_default();

        AgentEvent::ToolCall {
            call: call.clone(),
            provenance: provenance.clone(),
            summary_fields,
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

    pub(crate) async fn execute_tools_for_ui_with_presentation(
        &self,
        hook_ctx: &HookContext,
        calls: &[PendingToolCall],
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
        messages: &mut Vec<Message>,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
    ) -> AgentResult<Vec<ToolExecutionResult>> {
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
        for pending in deduped {
            let _ = event_tx.send(self.tool_call_event(&pending.call, &pending.provenance));

            let result = self
                .execute_single_tool_with_presentation(
                    hook_ctx,
                    &self.tools,
                    &pending.call,
                    policy,
                    presented_tool_names,
                )
                .await?;
            let observation = ToolObservation {
                call: pending.call.clone(),
                result,
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

            let ui_result = MessageToolResult {
                tool_use_id: observed.call.id.clone(),
                content: observed.result.content.clone(),
                is_error: observed.result.is_error,
            };
            let llm_result = if observed.result.is_error {
                MessageToolResult {
                    content: format!(
                        "{}\n\n[Analyze the error above and try a different approach.]",
                        observed.result.content
                    ),
                    ..ui_result.clone()
                }
            } else if self.bash_compression_enabled && observed.call.name == "bash" {
                let compressed = BashOutputCompressor::new().compress(&observed.result.content);
                MessageToolResult {
                    content: compressed.content,
                    ..ui_result.clone()
                }
            } else {
                ui_result.clone()
            };
            messages.push(Message::Tool { result: llm_result });
            let _ = event_tx.send(AgentEvent::ToolResult { result: ui_result });
            results.push(observed.result);
        }

        Ok(results)
    }

    async fn execute_single_tool_with_presentation(
        &self,
        hook_ctx: &HookContext,
        registry: &ToolRegistry,
        call: &ToolCall,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
    ) -> AgentResult<ToolExecutionResult> {
        let effective_call = match self
            .run_hook(hook_ctx, HookEvent::BeforeToolCall { call })
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeToolCall { call })) => Some(call),
            Ok(HookOutcome::Skip(_)) => return Ok(ToolExecutionResult::success(String::new())),
            Ok(_) => Some(call),
            Err(error) => return Err(error),
        };
        let call = effective_call.expect("tool call should be present");

        let tool = match registry.get(&call.name) {
            Some(t) => t,
            None => {
                return Ok(ToolExecutionResult::error(format!(
                    "tool not found: {}",
                    call.name
                )));
            }
        };
        if self.enforce_tool_presentation_policy && !presented_tool_names.contains(&call.name) {
            return Ok(ToolExecutionResult::error(format!(
                "tool family not loaded for '{}'; continue with a presented tool or request the relevant tool family",
                call.name
            )));
        }
        if self.enforce_tool_presentation_policy
            && let Some(backend) = tool.backend_for_input(&call.input)
            && !policy.allows_backend(&call.name, &backend)
        {
            return Ok(ToolExecutionResult::error(format!(
                "tool backend '{backend}' for '{}' is not loaded; continue with a disclosed backend or retry the base tool path",
                call.name
            )));
        }

        if let Some(engine) = self.permission_engine.as_deref() {
            self.run_hook(hook_ctx, HookEvent::BeforePermissionCheck { call })
                .await?;

            let profile = tool.permission_profile(&call.input);
            let decision = engine.evaluate_profile(&call.name, &profile, &call.input);
            self.run_hook(
                hook_ctx,
                HookEvent::AfterPermissionCheck {
                    call,
                    decision: decision.clone(),
                },
            )
            .await?;

            match decision {
                PermissionDecision::Allow => {}
                PermissionDecision::Deny(reason) => {
                    return Ok(ToolExecutionResult::error(format!(
                        "permission denied: {reason}"
                    )));
                }
                PermissionDecision::Ask => {}
            }
        }

        if let Err(e) = registry.validate_input(&call.name, &call.input) {
            return Ok(ToolExecutionResult::error(format!("invalid input for {e}")));
        }

        let normalized_input = crate::helpers::normalize_tool_input(&call.name, call.input.clone());

        let result = if call.name == "bash" {
            if let Some(sb) = self.sandbox.as_deref() {
                if sb.is_available() {
                    self.execute_bash_in_sandbox(hook_ctx, sb, &normalized_input)
                        .await
                } else {
                    tool.execute(normalized_input).await
                }
            } else {
                tool.execute(normalized_input).await
            }
        } else {
            tool.execute(normalized_input).await
        };

        let result = match self
            .run_hook(
                hook_ctx,
                HookEvent::AfterToolCall {
                    call,
                    result: &result,
                },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::AfterToolCall { result, .. }))
            | Ok(HookOutcome::Skip(HookEvent::AfterToolCall { result, .. })) => result.clone(),
            Ok(_) => result,
            Err(error) => return Err(error),
        };

        Ok(result)
    }

    /// Executes a bash command through the sandbox provider.
    ///
    /// Extracts the `command` field from the tool input and runs it within
    /// the sandbox environment. Returns a [`ToolExecutionResult`] with the
    /// combined stdout/stderr output.
    pub(crate) async fn execute_bash_in_sandbox(
        &self,
        hook_ctx: &HookContext,
        sandbox: &dyn SandboxProvider,
        input: &serde_json::Value,
    ) -> ToolExecutionResult {
        let command = match input.get("command").and_then(serde_json::Value::as_str) {
            Some(cmd) => cmd.to_owned(),
            None => {
                return ToolExecutionResult::error(
                    "bash tool input missing required field 'command'".to_owned(),
                );
            }
        };

        let command = match self
            .run_hook(
                hook_ctx,
                HookEvent::BeforeBashSandboxExec { command: &command },
            )
            .await
        {
            Ok(HookOutcome::Continue(HookEvent::BeforeBashSandboxExec { command })) => {
                command.to_string()
            }
            Ok(HookOutcome::Skip(_)) => return ToolExecutionResult::success(String::new()),
            Ok(_) => command,
            Err(_) => return ToolExecutionResult::error("hook denied bash execution".to_owned()),
        };

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
        }
    }
}
