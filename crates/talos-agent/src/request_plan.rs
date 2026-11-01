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
    /// Request-only image reserve, including its input safety margin.
    pub(super) ephemeral_image_tokens: u32,
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
    pub(super) history_source: Option<crate::compaction::HistorySource>,
    pub(super) checkpoint_observer: Option<crate::checkpoint_progress::CheckpointObserver>,
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
        self.reserve_ephemeral_images(plan, images)?;
        self.recover_request_budget(plan, limit)
    }

    /// Reserve metadata before checkpoint policy, without prematurely trimming.
    /// Repeated reservation replaces the prior reserve and never double-counts it.
    pub(super) fn reserve_ephemeral_images(
        &self,
        plan: &mut ProviderRequestPlan,
        images: &[talos_core::provider::EphemeralImage],
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
        plan.ephemeral_image_tokens =
            image_tokens.saturating_add(u32::try_from(margin).unwrap_or(u32::MAX));
        plan.estimated_tokens = self
            .estimate_provider_request_tokens(&plan.messages, &plan.tool_definitions)
            .saturating_add(plan.ephemeral_image_tokens);
        Ok(())
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
    #[cfg(test)]
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
        snapshot_tx: Option<
            mpsc::UnboundedSender<crate::persistence_snapshot::PersistenceSnapshot>,
        >,
        boundary_tx: Option<mpsc::UnboundedSender<crate::AgentActorRequest>>,
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
        let source = crate::compaction::HistorySource::bind_content_until(
            0,
            history.len() as u64,
            &history,
            tokio::time::Instant::now() + std::time::Duration::from_secs(120),
        )
        .await;
        self.prepare_turn_start_with_source(
            memory_query,
            input_messages,
            history,
            request_context_limit,
            source,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn prepare_session_turn_at_revision(
        &self,
        items: &[SubmissionItem],
        history: Vec<Message>,
        request_context_limit: u32,
        generation: u64,
        revision: u64,
        recovered: Option<&crate::compaction::HistorySource>,
        checkpoint_observer: Option<crate::checkpoint_progress::CheckpointObserver>,
    ) -> AgentResult<PreparedSessionTurn> {
        let source = crate::compaction::HistorySource::bind_recovered_until(
            generation,
            revision,
            &history,
            recovered,
            tokio::time::Instant::now() + std::time::Duration::from_secs(120),
        )
        .await;
        let (query, inputs) = Self::structured_session_inputs(items);
        self.prepare_turn_start_with_source(
            query,
            inputs,
            history,
            Some(request_context_limit),
            source,
            checkpoint_observer,
        )
        .await
    }

    async fn prepare_turn_start_with_source(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        request_context_limit: Option<u32>,
        history_source: Option<crate::compaction::HistorySource>,
        checkpoint_observer: Option<crate::checkpoint_progress::CheckpointObserver>,
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
                history_source.as_ref(),
                checkpoint_observer.as_ref(),
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
            history_source,
            checkpoint_observer,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn observe_provider_request_plan(
        &self,
        hook_ctx: &HookContext,
        messages: &[Message],
        tool_definitions: &[ToolDefinition],
        continuation_parts: &mut Vec<ContentPart>,
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
        let plan = ProviderRequestPlan {
            messages: owned_messages,
            tool_definitions,
            estimated_tokens,
            ephemeral_image_tokens: 0,
            omitted_tool_exchanges: 0,
            tool_protocol,
        };
        Ok(plan)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn seal_provider_request_plan(
        &self,
        hook_ctx: &HookContext,
        messages: &[Message],
        tool_definitions: &[ToolDefinition],
        continuation_parts: &mut Vec<ContentPart>,
        request_context_limit: Option<u32>,
        private_tokens: &PrivateTokens,
        history_source: Option<&crate::compaction::HistorySource>,
        checkpoint_observer: Option<&crate::checkpoint_progress::CheckpointObserver>,
    ) -> AgentResult<ProviderRequestPlan> {
        let plan = self
            .observe_provider_request_plan(
                hook_ctx,
                messages,
                tool_definitions,
                continuation_parts,
                private_tokens,
            )
            .await?;
        self.seal_observed_request_plan(
            plan,
            request_context_limit,
            private_tokens,
            history_source,
            checkpoint_observer,
        )
        .await
    }

    /// Complete budget/projection admission without invoking the request hook again.
    pub(super) async fn seal_observed_request_plan(
        &self,
        mut plan: ProviderRequestPlan,
        request_context_limit: Option<u32>,
        private_tokens: &PrivateTokens,
        history_source: Option<&crate::compaction::HistorySource>,
        checkpoint_observer: Option<&crate::checkpoint_progress::CheckpointObserver>,
    ) -> AgentResult<ProviderRequestPlan> {
        if let Some(limit) = request_context_limit {
            use crate::compaction::RequestPressure;
            let reused = if let Some(source) = history_source {
                self.reuse_checkpoint_projection(&mut plan, source, limit, private_tokens)
                    .await
            } else {
                false
            };
            let source_verified = !reused
                && history_source.is_some_and(|source| {
                    crate::checkpoint_request::eligible_prefix_end(&plan.messages).is_some_and(
                        |end| {
                            plan.messages[..end]
                                .iter()
                                .enumerate()
                                .all(|(index, message)| {
                                    matches!(
                                        message,
                                        Message::System { .. } | Message::Context { .. }
                                    ) || source
                                        .resolve(
                                            source.generation(),
                                            source.revision(),
                                            index,
                                            message,
                                        )
                                        .is_some()
                                })
                        },
                    )
                });
            if source_verified
                && matches!(
                    RequestPressure::assess(plan.estimated_tokens, limit),
                    RequestPressure::Critical | RequestPressure::Emergency
                )
            {
                // Failure leaves the sealed projection intact. Existing bounded
                // recovery must still admit it or return an explicit refusal.
                let progress = crate::checkpoint_progress::CheckpointProgress::start(
                    checkpoint_observer,
                    plan.estimated_tokens,
                );
                let deadline = history_source
                    .and_then(crate::compaction::HistorySource::deadline)
                    .unwrap_or_else(|| {
                        tokio::time::Instant::now() + std::time::Duration::from_secs(120)
                    });
                let mut candidate = plan.clone();
                let mut projection = None;
                let prepared = self
                    .checkpoint_request_once_deferred(
                        &mut candidate,
                        limit,
                        private_tokens,
                        history_source,
                        Some(&mut projection),
                    )
                    .await;
                if prepared.is_ok()
                    && let Some(projection) = projection
                {
                    let install = || self.install_checkpoint_projection(projection, deadline);
                    let committed = if let Some(control) =
                        checkpoint_observer.and_then(|observer| observer.admission.as_ref())
                    {
                        control.commit_checkpoint(install)
                    } else {
                        Some(install())
                    };
                    match committed {
                        Some(Ok(())) => {
                            plan = candidate;
                            progress.finish(Ok(plan.estimated_tokens));
                        }
                        Some(Err(_)) => progress.finish(Err(
                            talos_core::session::AutomaticCompactFailure::CommitRejected,
                        )),
                        None => progress.cancel(),
                    }
                } else {
                    let reason = prepared
                        .as_ref()
                        .err()
                        .map(crate::checkpoint_progress::failure_reason)
                        .unwrap_or(talos_core::session::AutomaticCompactFailure::Preparation);
                    progress.finish(Err(reason));
                }
            } else if !reused
                && !source_verified
                && matches!(
                    RequestPressure::assess(plan.estimated_tokens, limit),
                    RequestPressure::Critical | RequestPressure::Emergency
                )
                && let Some(observer) = checkpoint_observer
            {
                observer.emit(
                    talos_core::session::AutomaticCompactState::SkippedSourceUnavailable {
                        before: plan.estimated_tokens,
                    },
                );
            }
        }
        self.recover_request_budget(&mut plan, request_context_limit)?;
        Ok(plan)
    }

    /// Recover only the sealed projection. Hooks, canonical history and tool execution
    /// are deliberately outside this boundary. At most 64 old exchanges are omitted.
    fn recover_request_budget(
        &self,
        plan: &mut ProviderRequestPlan,
        limit: Option<u32>,
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
                .saturating_add(plan.ephemeral_image_tokens);
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
