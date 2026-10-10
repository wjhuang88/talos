use std::collections::HashSet;

use tokio::sync::mpsc;

use talos_core::message::{AgentEvent, ContentPart, Message};
use talos_core::provider::ToolDefinition;
use talos_core::session::SubmissionItem;
use talos_core::tool::ToolPresentationPolicy;
use talos_plugin::{HookContext, HookEvent, HookOutcome, TurnId, TurnStatus};

use crate::configuration::describe_presented_tools;
use crate::transient_text::PrivateTokens;
use crate::{Agent, AgentError, AgentResult};

/// One owned Provider request that is budgeted and dispatched without rebuild.
#[derive(Debug, Clone)]
pub(super) struct ProviderRequestPlan {
    pub(super) messages: Vec<Message>,
    pub(super) tool_definitions: Vec<ToolDefinition>,
    pub(super) estimated_tokens: u32,
    pub(super) omitted_tool_exchanges: u32,
    pub(super) tool_protocol: talos_core::tool::ToolProtocol,
}

/// Canonical turn state plus the already sealed initial Provider request.
pub(crate) struct PreparedSessionTurn {
    pub(super) hook_ctx: HookContext,
    pub(super) messages: Vec<Message>,
    pub(super) persist_start: usize,
    pub(super) active_tool_presentation_policy: ToolPresentationPolicy,
    pub(super) active_tool_definitions: Vec<ToolDefinition>,
    pub(super) active_presented_tool_names: HashSet<String>,
    pub(super) initial_plan: ProviderRequestPlan,
    pub(super) request_context_limit: Option<u32>,
}

impl Agent {
    pub(super) fn emit_budget_rejection(
        &self,
        error: &AgentError,
        event_tx: Option<&mpsc::UnboundedSender<AgentEvent>>,
    ) {
        if let (AgentError::ContextBudgetExceeded { estimated, limit }, Some(tx)) =
            (error, event_tx)
        {
            let _ = tx.send(AgentEvent::ContextBudget {
                budget: talos_core::message::ContextBudget {
                    estimated_tokens: *estimated,
                    limit: Some(*limit),
                    omitted_tool_exchanges: 0,
                },
            });
        }
    }
    pub(super) fn admit_ephemeral_images(
        &self,
        plan: &mut ProviderRequestPlan,
        images: &[talos_core::provider::EphemeralImage],
        limit: Option<u32>,
    ) -> AgentResult<()> {
        if !images.is_empty() && !self.image_input_supported {
            return Err(AgentError::ToolError(
                "selected model does not support image input".into(),
            ));
        }
        // Match the existing provider-independent durable-image reserve. Only metadata
        // participates in admission; bytes never enter messages, hooks or previews.
        let image_tokens = images.iter().fold(0_u32, |total, image| {
            total.saturating_add(
                u32::try_from(image.byte_count().div_ceil(3))
                    .unwrap_or(u32::MAX)
                    .saturating_add(crate::token::TokenEstimator::estimate_text("image/png"))
                    .saturating_add(1024),
            )
        });
        let margin = u64::from(image_tokens)
            .saturating_mul(u64::from(self.request_budget_spec.input_safety_margin_bps))
            .div_ceil(10_000);
        let estimated = self
            .estimate_provider_request_tokens(&plan.messages, &plan.tool_definitions)
            .saturating_add(image_tokens)
            .saturating_add(u32::try_from(margin).unwrap_or(u32::MAX));
        plan.estimated_tokens = estimated;
        self.recover_request_budget(
            plan,
            limit,
            image_tokens.saturating_add(u32::try_from(margin).unwrap_or(u32::MAX)),
        )
    }

    pub(super) fn structured_session_inputs(items: &[SubmissionItem]) -> (String, Vec<Message>) {
        let memory_query = items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let input_messages = items
            .iter()
            .map(|item| {
                if item.attachments.is_empty() {
                    Message::User {
                        content: item.text.clone(),
                    }
                } else {
                    let mut parts = Vec::with_capacity(item.attachments.len() + 1);
                    if !item.text.is_empty() {
                        parts.push(ContentPart::Text {
                            text: item.text.clone(),
                        });
                    }
                    parts.extend(item.attachments.clone());
                    Message::Multimodal { parts }
                }
            })
            .collect();
        (memory_query, input_messages)
    }

    /// Builds and validates the exact initial request before Actor Turn start.
    pub(crate) async fn prepare_session_turn(
        &self,
        items: &[SubmissionItem],
        history: Vec<Message>,
        request_context_limit: u32,
    ) -> AgentResult<PreparedSessionTurn> {
        let (memory_query, input_messages) = Self::structured_session_inputs(items);
        self.prepare_turn_start(
            memory_query,
            input_messages,
            history,
            Some(request_context_limit),
        )
        .await
    }

    /// Consumes a previously validated initial plan without rebuilding it.
    pub(crate) async fn run_prepared_session_turn(
        &self,
        prepared: PreparedSessionTurn,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        snapshot_tx: Option<mpsc::UnboundedSender<Vec<Message>>>,
        boundary_tx: Option<mpsc::UnboundedSender<crate::SteeringBoundaryRequest>>,
        boundary_ack_tx: Option<mpsc::UnboundedSender<crate::SteeringBoundaryAcknowledgement>>,
    ) -> (AgentResult<String>, Vec<Message>) {
        self.run_prepared_inner(
            prepared,
            Some(event_tx),
            snapshot_tx,
            boundary_tx,
            boundary_ack_tx,
        )
        .await
    }

    pub(super) async fn prepare_turn_start(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        request_context_limit: Option<u32>,
    ) -> AgentResult<PreparedSessionTurn> {
        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());

        let active_tool_presentation_policy = self.tool_presentation_policy.clone();
        let (_, mut active_tool_definitions, mut active_presented_tool_names) =
            describe_presented_tools(&self.tools, &active_tool_presentation_policy);
        if !self.image_input_supported {
            active_tool_definitions.retain(|definition| definition.name != "read_image");
            active_presented_tool_names.retain(|name| name != "read_image");
        }

        let selected_protocol = self
            .resolve_tool_protocol_for_definitions(self.tool_protocol, &active_tool_definitions)
            .await;
        let (mut messages, persist_start) = match self
            .build_provider_messages_with_protocol(
                memory_query,
                history,
                &hook_ctx,
                selected_protocol,
            )
            .await
        {
            Ok(messages) => messages,
            Err(error) => {
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return Err(error);
            }
        };
        messages.pop();
        messages.extend(input_messages);

        if let Err(error) = self
            .run_hook(&hook_ctx, HookEvent::TurnStart { turn_id })
            .await
        {
            self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
            return Err(error);
        }

        let mut continuation_parts = Vec::new();
        let initial_plan = match self
            .seal_provider_request_plan(
                &hook_ctx,
                &messages,
                &active_tool_definitions,
                &mut continuation_parts,
                request_context_limit,
                &PrivateTokens::default(),
            )
            .await
        {
            Ok(plan) => plan,
            Err(error) => {
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return Err(error);
            }
        };

        Ok(PreparedSessionTurn {
            hook_ctx,
            messages,
            persist_start,
            active_tool_presentation_policy,
            active_tool_definitions,
            active_presented_tool_names,
            initial_plan,
            request_context_limit,
        })
    }

    pub(super) async fn seal_provider_request_plan(
        &self,
        hook_ctx: &HookContext,
        messages: &[Message],
        tool_definitions: &[ToolDefinition],
        continuation_parts: &mut Vec<ContentPart>,
        request_context_limit: Option<u32>,
        private_tokens: &PrivateTokens,
    ) -> AgentResult<ProviderRequestPlan> {
        let hook_messages = self.persistence_projection_with_tokens(messages, private_tokens);
        let observed_provider_messages = match self
            .run_hook(
                hook_ctx,
                HookEvent::BeforeProviderCall {
                    messages: &hook_messages,
                },
            )
            .await?
        {
            HookOutcome::Continue(HookEvent::BeforeProviderCall { messages })
            | HookOutcome::Skip(HookEvent::BeforeProviderCall { messages }) => messages,
            _ => messages,
        };
        let provider_messages = if observed_provider_messages == hook_messages.as_slice() {
            messages
        } else {
            observed_provider_messages
        };

        let mut owned_messages = if self.replay_reasoning {
            provider_messages.to_vec()
        } else {
            provider_messages
                .iter()
                .map(|message| {
                    if let Message::Assistant {
                        content,
                        tool_calls,
                        reasoning: Some(_),
                    } = message
                    {
                        Message::Assistant {
                            content: content.clone(),
                            tool_calls: tool_calls.clone(),
                            reasoning: None,
                        }
                    } else {
                        message.clone()
                    }
                })
                .collect()
        };

        if !continuation_parts.is_empty() {
            owned_messages.push(Message::Multimodal {
                parts: std::mem::take(continuation_parts),
            });
        }

        let tool_definitions = tool_definitions.to_vec();
        let estimated_tokens =
            self.estimate_provider_request_tokens(&owned_messages, &tool_definitions);
        // The plan is the request's single source of protocol truth. Configuration normally
        // resolves `Auto` eagerly; resolve defensively here so a plan can never dispatch an
        // ambiguous protocol if a caller constructed an agent before configuration settled.
        let tool_protocol = self.resolve_tool_protocol(self.tool_protocol);
        let mut plan = ProviderRequestPlan {
            messages: owned_messages,
            tool_definitions,
            estimated_tokens,
            omitted_tool_exchanges: 0,
            tool_protocol,
        };
        self.recover_request_budget(&mut plan, request_context_limit, 0)?;
        Ok(plan)
    }

    /// Recover only the sealed projection. Hooks, canonical history and tool execution
    /// are deliberately outside this boundary. At most 64 old exchanges are omitted.
    fn recover_request_budget(
        &self,
        plan: &mut ProviderRequestPlan,
        limit: Option<u32>,
        additional_tokens: u32,
    ) -> AgentResult<()> {
        let Some(limit) = limit else {
            return Ok(());
        };
        if plan.estimated_tokens <= limit {
            return Ok(());
        }
        let exchanges = complete_tool_exchanges(&plan.messages);
        // Keep the most recent complete exchange, even when it is too large.
        let removable = exchanges.len().saturating_sub(1).min(64);
        let mut removed = vec![false; plan.messages.len()];
        let mut omitted = 0_u32;
        for range in exchanges.into_iter().take(removable) {
            // Reasoning replay payloads must remain byte-for-byte intact.
            if matches!(
                &plan.messages[range.start],
                Message::Assistant {
                    reasoning: Some(_),
                    ..
                }
            ) {
                continue;
            }
            removed[range].fill(true);
            let mut candidate: Vec<Message> = plan
                .messages
                .iter()
                .enumerate()
                .filter(|(index, _)| !removed[*index])
                .map(|(_, message)| message.clone())
                .collect();
            candidate.push(Message::Context {
                content: "[Older completed tool exchanges omitted from this request to fit the context budget. Their results remain in session history; do not assume their contents or repeat tools solely to reconstruct them.]".into(),
            });
            let estimated = self
                .estimate_provider_request_tokens(&candidate, &plan.tool_definitions)
                .saturating_add(additional_tokens);
            omitted += 1;
            if estimated <= limit {
                plan.messages = candidate;
                plan.estimated_tokens = estimated;
                plan.omitted_tool_exchanges = plan.omitted_tool_exchanges.saturating_add(omitted);
                return Ok(());
            }
        }
        Err(AgentError::ContextBudgetExceeded {
            estimated: plan.estimated_tokens,
            limit,
        })
    }
}

fn complete_tool_exchanges(messages: &[Message]) -> Vec<std::ops::Range<usize>> {
    let mut exchanges = Vec::new();
    for (start, message) in messages.iter().enumerate() {
        let Message::Assistant { tool_calls, .. } = message else {
            continue;
        };
        if tool_calls.is_empty() {
            continue;
        }
        let mut pending: HashSet<&str> = tool_calls.iter().map(|call| call.id.as_str()).collect();
        if pending.len() != tool_calls.len() {
            continue;
        }
        let mut end = start + 1;
        let mut valid = true;
        while let Some(Message::Tool { result }) = messages.get(end) {
            if !pending.remove(result.tool_use_id.as_str()) {
                valid = false;
                break;
            }
            end += 1;
        }
        if valid && pending.is_empty() {
            exchanges.push(start..end);
        }
    }
    exchanges
}

#[cfg(test)]
mod tests;
