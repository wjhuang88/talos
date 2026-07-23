use std::sync::Arc;

use futures_util::future::join_all;
use talos_core::message::{
    AgentEvent, ContentPart, Message, MessageToolResult, StopReason, ToolCall,
};
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
                    policy,
                    presented_tool_names,
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
            }
        } else {
            observed.clone()
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
        for pending in deduped {
            let _ = event_tx.send(self.tool_call_event(&pending.call, &pending.provenance));

            let (result, parts) = self
                .execute_single_tool_with_presentation(
                    hook_ctx,
                    &self.tools,
                    &pending.call,
                    policy,
                    presented_tool_names,
                )
                .await?;
            all_parts.extend(parts);
            let projected_call = self.project_tool_call(&pending.call);
            let projected_result = self.project_tool_result(&pending.call.name, &result);
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
                call: Self::restore_private_call_if_unchanged(
                    &pending.call,
                    &projected_call,
                    &observed.call,
                ),
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
                content: projection.display_content,
                is_error: observed.result.is_error,
            };
            let llm_result = if observed.result.is_error {
                MessageToolResult {
                    content: format!(
                        "{}\n\n[Analyze the error above and try a different approach.]",
                        projection.model_content
                    ),
                    ..ui_result.clone()
                }
            } else if self.bash_compression_enabled && observed.call.name == "bash" {
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
            let _ = event_tx.send(AgentEvent::ToolResult { result: ui_result });
            results.push(observed.result);
        }

        Ok((results, all_parts))
    }

    async fn execute_single_tool_with_presentation(
        &self,
        hook_ctx: &HookContext,
        registry: &ToolRegistry,
        call: &ToolCall,
        policy: &ToolPresentationPolicy,
        presented_tool_names: &std::collections::HashSet<String>,
    ) -> AgentResult<(ToolExecutionResult, Vec<ContentPart>)> {
        let original_call = call.clone();
        let projected_call = self.project_tool_call(call);
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
        let call = effective_call.expect("tool call should be present");

        let tool = match registry.get(&call.name) {
            Some(t) => t,
            None => {
                return Ok((
                    ToolExecutionResult::error(format!("tool not found: {}", call.name)),
                    Vec::new(),
                ));
            }
        };
        if self.enforce_tool_presentation_policy && !presented_tool_names.contains(&call.name) {
            return Ok((
                ToolExecutionResult::error(format!(
                    "tool family not loaded for '{}'; continue with a presented tool or request the relevant tool family",
                    call.name
                )),
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

        if let Some(engine) = self.permission_engine.as_deref() {
            let projected_call = self.project_tool_call(&call);
            self.run_hook(
                hook_ctx,
                HookEvent::BeforePermissionCheck {
                    call: &projected_call,
                },
            )
            .await?;

            let profile = tool.permission_profile(&call.input);
            let decision = engine.evaluate_profile(&call.name, &profile, &call.input);

            let evidence_diagnostics = collect_access_evidence_diagnostics(&call)
                .into_iter()
                .map(|(command, evidence)| {
                    let _ = engine.evaluate_command_with_evidence(
                        &call.name,
                        &command,
                        &evidence,
                        &call.input,
                    );
                    format!(
                        "{} => {}",
                        command,
                        format_access_evidence_diagnostic(&evidence)
                    )
                })
                .collect::<Vec<_>>();

            self.run_hook(
                hook_ctx,
                HookEvent::AfterPermissionCheck {
                    call: &projected_call,
                    decision: decision.clone(),
                },
            )
            .await?;

            match decision {
                PermissionDecision::Allow => {}
                PermissionDecision::Deny(reason) => {
                    return Ok((
                        ToolExecutionResult::error(format!("permission denied: {reason}")),
                        Vec::new(),
                    ));
                }
                PermissionDecision::Ask => {}
            }

            for diag in evidence_diagnostics {
                tracing::debug!("access evidence for {}: {}", call.name, diag);
            }
        }

        if let Err(e) = registry.validate_input(&call.name, &call.input) {
            return Ok((
                ToolExecutionResult::error(format!("invalid input for {e}")),
                Vec::new(),
            ));
        }

        let normalized_input = crate::helpers::normalize_tool_input(&call.name, call.input.clone());

        let (result, parts) = if call.name == "bash" {
            if let Some(sb) = self.sandbox.as_deref() {
                if sb.is_available() {
                    (
                        self.execute_bash_in_sandbox(hook_ctx, sb, &normalized_input)
                            .await,
                        Vec::new(),
                    )
                } else {
                    let output = tool.execute_with_output(normalized_input).await;
                    (output.result, output.next_provider_parts)
                }
            } else {
                let output = tool.execute_with_output(normalized_input).await;
                (output.result, output.next_provider_parts)
            }
        } else {
            let output = tool.execute_with_output(normalized_input).await;
            (output.result, output.next_provider_parts)
        };

        let projected_call = self.project_tool_call(&call);
        let projected_result = self.project_tool_result(&call.name, &result);
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

fn collect_access_evidence_diagnostics(
    call: &ToolCall,
) -> Vec<(String, talos_permission::AccessEvidence)> {
    if call.name == "bash" {
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

fn classify_exec_step(
    step: &serde_json::Value,
) -> Option<(String, talos_permission::AccessEvidence)> {
    let command = step.get("command")?.as_str()?;
    Some(classify_exec_argv(command, step.get("args")))
}

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
