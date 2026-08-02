use std::collections::HashSet;

use tokio::sync::mpsc;

use talos_core::message::{AgentEvent, ContentPart, Message};
use talos_core::provider::ToolDefinition;
use talos_core::session::SubmissionItem;
use talos_core::tool::ToolPresentationPolicy;
use talos_plugin::{HookContext, HookEvent, HookOutcome, TurnId, TurnStatus};

use crate::configuration::describe_presented_tools;
use crate::{Agent, AgentError, AgentResult};

/// One owned Provider request that is budgeted and dispatched without rebuild.
#[derive(Debug, Clone)]
pub(super) struct ProviderRequestPlan {
    pub(super) messages: Vec<Message>,
    pub(super) tool_definitions: Vec<ToolDefinition>,
    pub(super) estimated_tokens: u32,
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
    fn structured_session_inputs(items: &[SubmissionItem]) -> (String, Vec<Message>) {
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
    ) -> (AgentResult<String>, Vec<Message>) {
        self.run_prepared_inner(prepared, Some(event_tx)).await
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

        let (mut messages, persist_start) = match self
            .build_provider_messages(memory_query, history, &hook_ctx)
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

        let active_tool_presentation_policy = self.tool_presentation_policy.clone();
        let (_, mut active_tool_definitions, mut active_presented_tool_names) =
            describe_presented_tools(&self.tools, &active_tool_presentation_policy);
        if !self.image_input_supported {
            active_tool_definitions.retain(|definition| definition.name != "read_image");
            active_presented_tool_names.retain(|name| name != "read_image");
        }

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
    ) -> AgentResult<ProviderRequestPlan> {
        let hook_messages = self.persistence_projection(messages);
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
            Self::estimate_provider_request_tokens(&owned_messages, &tool_definitions);
        if let Some(limit) = request_context_limit
            && estimated_tokens > limit
        {
            return Err(AgentError::ContextBudgetExceeded {
                estimated: estimated_tokens,
                limit,
            });
        }

        Ok(ProviderRequestPlan {
            messages: owned_messages,
            tool_definitions,
            estimated_tokens,
        })
    }
}
