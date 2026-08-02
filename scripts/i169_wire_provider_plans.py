#!/usr/bin/env python3
"""Wire I169 sealed Provider request plans into Agent and Session call sites.

This script is intentionally single-use. Every source anchor must match exactly
once so repository drift fails closed instead of producing a partial rewrite.
"""

from __future__ import annotations

import re
from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one {label}, found {count}")
    return source.replace(old, new, 1)


def main() -> None:
    lib_path = Path("crates/talos-agent/src/lib.rs")
    lib = lib_path.read_text()
    lib = replace_once(
        lib,
        "mod helpers;\npub mod prompt;",
        "mod helpers;\nmod request_plan;\npub mod prompt;",
        "request_plan module insertion",
    )
    lib = replace_once(
        lib,
        "pub use prompt::{ActivatedSkillContext, ContextFile, SystemPromptBuilder, ToolDescription};\n",
        "pub use prompt::{ActivatedSkillContext, ContextFile, SystemPromptBuilder, ToolDescription};\npub(crate) use request_plan::PreparedSessionTurn;\n",
        "PreparedSessionTurn re-export",
    )

    old_run_items = '''    pub(crate) async fn run_for_session_turn_items(
        &self,
        items: Vec<talos_core::session::SubmissionItem>,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        request_context_limit: u32,
    ) -> (AgentResult<String>, Vec<Message>) {
        let memory_query = items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\\n");
        let input_messages = items
            .into_iter()
            .map(|item| {
                if item.attachments.is_empty() {
                    Message::User { content: item.text }
                } else {
                    let mut parts = Vec::with_capacity(item.attachments.len() + 1);
                    if !item.text.is_empty() {
                        parts.push(talos_core::message::ContentPart::Text { text: item.text });
                    }
                    parts.extend(item.attachments);
                    Message::Multimodal { parts }
                }
            })
            .collect();
        self.run_inner_with_messages(
            memory_query,
            input_messages,
            history,
            Some(event_tx),
            Some(request_context_limit),
        )
        .await
    }
'''
    new_run_items = '''    pub(crate) async fn run_for_session_turn_items(
        &self,
        items: Vec<talos_core::session::SubmissionItem>,
        history: Vec<Message>,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        request_context_limit: u32,
    ) -> (AgentResult<String>, Vec<Message>) {
        let prepared = match self
            .prepare_session_turn(&items, history, request_context_limit)
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => return (Err(error), Vec::new()),
        };
        self.run_prepared_session_turn(prepared, event_tx).await
    }
'''
    lib = replace_once(lib, old_run_items, new_run_items, "structured turn runner")
    lib = replace_once(
        lib,
        "    /// reserve. The session actor calls this after compaction and before it\n    /// acknowledges that a turn started.\n    pub(crate) async fn estimate_session_request_tokens(",
        "    /// reserve. This remains a diagnostic estimate only; Session execution is\n    /// authorized by the sealed plan returned from `prepare_session_turn`.\n    #[allow(dead_code)]\n    pub(crate) async fn estimate_session_request_tokens(",
        "estimate API documentation",
    )

    old_setup = '''    async fn run_inner_with_messages(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
        request_context_limit: Option<u32>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());

        let (mut messages, persist_start) = match self
            .build_provider_messages(memory_query, history, &hook_ctx)
            .await
        {
            Ok(messages) => messages,
            Err(error) => {
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return (Err(error), Vec::new());
            }
        };
        messages.pop();
        messages.extend(input_messages);

        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();
        let mut active_tool_presentation_policy = self.tool_presentation_policy.clone();
        let (_, mut active_tool_definitions, mut active_presented_tool_names) =
            describe_presented_tools(&self.tools, &active_tool_presentation_policy);

        if !self.image_input_supported {
            active_tool_definitions.retain(|td| td.name != "read_image");
            active_presented_tool_names.retain(|n| n != "read_image");
        }

        // Transient continuation parts collected from tool execution (ADR-051).
        // Consumed once by the next stream_with_tools call as a
        // Message::Multimodal overlay; never persisted. If the provider
        // call fails or the turn ends, the parts are discarded.
        let mut pending_continuation_parts: Vec<talos_core::message::ContentPart> = Vec::new();

        if let Err(error) = self
            .run_hook(&hook_ctx, HookEvent::TurnStart { turn_id })
            .await
        {
            self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
            return (Err(error), Vec::new());
        }

        let (result, final_status) = 'turn_loop: loop {'''
    new_setup = '''    async fn run_inner_with_messages(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
        request_context_limit: Option<u32>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let prepared = match self
            .prepare_turn_start(
                memory_query,
                input_messages,
                history,
                request_context_limit,
            )
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => return (Err(error), Vec::new()),
        };
        self.run_prepared_inner(prepared, event_tx).await
    }

    pub(super) async fn run_prepared_inner(
        &self,
        prepared: PreparedSessionTurn,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let PreparedSessionTurn {
            hook_ctx,
            mut messages,
            persist_start,
            mut active_tool_presentation_policy,
            mut active_tool_definitions,
            mut active_presented_tool_names,
            initial_plan,
            request_context_limit,
        } = prepared;
        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();
        let mut pending_continuation_parts: Vec<talos_core::message::ContentPart> = Vec::new();
        let mut initial_plan = Some(initial_plan);

        let (result, final_status) = 'turn_loop: loop {'''
    lib = replace_once(lib, old_setup, new_setup, "prepared turn setup")

    old_dispatch = '''            let hook_messages = self.persistence_projection(&messages);
            let observed_provider_messages = match self
                .run_hook(
                    &hook_ctx,
                    HookEvent::BeforeProviderCall {
                        messages: &hook_messages,
                    },
                )
                .await
            {
                Ok(HookOutcome::Continue(HookEvent::BeforeProviderCall { messages }))
                | Ok(HookOutcome::Skip(HookEvent::BeforeProviderCall { messages })) => messages,
                Ok(_) => messages.as_slice(),
                Err(error) => {
                    break (Err(error), TurnStatus::Denied);
                }
            };
            let provider_messages = if observed_provider_messages == hook_messages.as_slice() {
                messages.as_slice()
            } else {
                observed_provider_messages
            };

            let filtered_provider_messages;
            let provider_messages = if self.replay_reasoning {
                provider_messages
            } else {
                filtered_provider_messages = provider_messages
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
                    .collect::<Vec<_>>();
                filtered_provider_messages.as_slice()
            };

            let continuation_overlay: Vec<Message>;
            let provider_messages = if pending_continuation_parts.is_empty() {
                provider_messages
            } else {
                let mut msgs = provider_messages.to_vec();
                msgs.push(Message::Multimodal {
                    parts: std::mem::take(&mut pending_continuation_parts),
                });
                continuation_overlay = msgs;
                continuation_overlay.as_slice()
            };

            if let Some(limit) = request_context_limit {
                let estimated = Self::estimate_provider_request_tokens(
                    provider_messages,
                    &active_tool_definitions,
                );
                if estimated > limit {
                    break 'turn_loop (
                        Err(AgentError::ContextBudgetExceeded { estimated, limit }),
                        TurnStatus::Denied,
                    );
                }
            }

            let mut rx = match self
                .provider
                .stream_with_tools(provider_messages, &active_tool_definitions)
                .await'''
    new_dispatch = '''            let plan = if let Some(plan) = initial_plan.take() {
                plan
            } else {
                match self
                    .seal_provider_request_plan(
                        &hook_ctx,
                        &messages,
                        &active_tool_definitions,
                        &mut pending_continuation_parts,
                        request_context_limit,
                    )
                    .await
                {
                    Ok(plan) => plan,
                    Err(error) => break (Err(error), TurnStatus::Denied),
                }
            };
            tracing::trace!(
                estimated_tokens = plan.estimated_tokens,
                "dispatching sealed provider request plan"
            );

            let mut rx = match self
                .provider
                .stream_with_tools(&plan.messages, &plan.tool_definitions)
                .await'''
    lib = replace_once(lib, old_dispatch, new_dispatch, "provider dispatch planning")
    lib_path.write_text(lib)

    session_path = Path("crates/talos-agent/src/session.rs")
    session = session_path.read_text()
    request_pattern = re.compile(
        r"        let mut request_tokens = self\n.*?        if !matches!\(request_tokens, Ok\(tokens\) if tokens <= self\.model_context_limit\) \{\n.*?        \}\n\n",
        re.DOTALL,
    )
    prepared_block = '''        let prepared_turn = if submission.common_kind()
            == Some(SubmissionKind::PreviewRequest)
        {
            None
        } else {
            match self
                .agent
                .prepare_session_turn(
                    &submission.items,
                    self.history.clone(),
                    self.model_context_limit,
                )
                .await
            {
                Ok(prepared) => Some(prepared),
                Err(crate::AgentError::ContextBudgetExceeded { .. }) => {
                    self.reject_submission(
                        &submission.id,
                        submission.sender_generation,
                        SubmissionRejectionReason::ContextBudgetExceeded,
                    );
                    return None;
                }
                Err(error) => {
                    let _ = self.eq_tx.send(SessionEvent::Error {
                        message: format!(
                            "failed to seal Provider request plan for {}: {error}",
                            submission.id
                        ),
                    });
                    self.reject_submission(
                        &submission.id,
                        submission.sender_generation,
                        SubmissionRejectionReason::InvalidStructure,
                    );
                    return None;
                }
            }
        };

'''
    session, count = request_pattern.subn(prepared_block, session, count=1)
    if count != 1:
        raise SystemExit(f"expected exactly one Actor request estimate block, found {count}")
    session = replace_once(
        session,
        '''        let history = self.history.clone();
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
        let items = submission.items;
        let request_context_limit = self.model_context_limit;
''',
        '''        let prepared = prepared_turn.expect("non-preview submission must be prepared");
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
''',
        "prepared turn spawn variables",
    )
    session = replace_once(
        session,
        '''                agent,
                items,
                history,
                event_tx,
''',
        '''                agent,
                prepared,
                event_tx,
''',
        "TurnForwarding prepared field",
    )
    session = replace_once(
        session,
        '''                durable_persistence,
                result_tx,
                request_context_limit,
''',
        '''                durable_persistence,
                result_tx,
''',
        "TurnForwarding request limit removal",
    )
    session_path.write_text(session)

    turn_path = Path("crates/talos-agent/src/session/turn.rs")
    turn = turn_path.read_text()
    turn = replace_once(
        turn,
        "use talos_core::session::{SessionEvent, SubmissionItem, TurnCompletionStatus, TurnEventPayload};",
        "use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};",
        "unused SubmissionItem import",
    )
    turn = replace_once(
        turn,
        "use crate::Agent;",
        "use crate::{Agent, PreparedSessionTurn};",
        "PreparedSessionTurn import",
    )
    turn = replace_once(
        turn,
        '''    pub(super) agent: Arc<Agent>,
    pub(super) items: Vec<SubmissionItem>,
    pub(super) history: Vec<Message>,
''',
        '''    pub(super) agent: Arc<Agent>,
    pub(super) prepared: PreparedSessionTurn,
''',
        "TurnForwarding input fields",
    )
    turn = replace_once(
        turn,
        '''    pub(super) durable_persistence: Option<DurableTurnPersistence>,
    pub(super) result_tx: tokio::sync::oneshot::Sender<TurnRecord>,
    pub(super) request_context_limit: u32,
''',
        '''    pub(super) durable_persistence: Option<DurableTurnPersistence>,
    pub(super) result_tx: tokio::sync::oneshot::Sender<TurnRecord>,
''',
        "TurnForwarding request limit field",
    )
    turn = replace_once(
        turn,
        '''        agent,
        items,
        history,
        event_tx,
''',
        '''        agent,
        prepared,
        event_tx,
''',
        "TurnForwarding destructure inputs",
    )
    turn = replace_once(
        turn,
        '''        durable_persistence,
        result_tx,
        request_context_limit,
''',
        '''        durable_persistence,
        result_tx,
''',
        "TurnForwarding destructure limit",
    )
    turn = replace_once(
        turn,
        '''        agent
            .run_for_session_turn_items(items, history, event_tx, request_context_limit)
            .await
''',
        '''        agent.run_prepared_session_turn(prepared, event_tx).await
''',
        "prepared Agent task",
    )
    turn_path.write_text(turn)


if __name__ == "__main__":
    main()
