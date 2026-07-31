from pathlib import Path

# Agent: introduce a frozen first-request snapshot shared by preflight and execution.
path = Path('crates/talos-agent/src/lib.rs')
text = path.read_text()

struct_anchor = '''#[derive(Debug, Clone)]
struct PendingToolCall {
    call: ToolCall,
    provenance: ToolProvenance,
}
'''
prepared_struct = '''#[derive(Debug, Clone)]
struct PendingToolCall {
    call: ToolCall,
    provenance: ToolProvenance,
}

/// Frozen first-provider-request state for one structured session turn.
///
/// Dynamic prompt callbacks and prompt hooks run exactly once while this value
/// is prepared. Context compaction may replace only the history prefix; the
/// dynamic system/context/input suffix, hook identity, and tool presentation
/// remain identical for preflight and the first provider call.
pub(crate) struct PreparedSessionTurn {
    messages: Vec<Message>,
    persist_start: usize,
    history_len: usize,
    hook_ctx: HookContext,
    tool_presentation_policy: ToolPresentationPolicy,
    tool_definitions: Vec<talos_core::provider::ToolDefinition>,
    presented_tool_names: HashSet<String>,
}
'''
if 'pub(crate) struct PreparedSessionTurn' not in text:
    if text.count(struct_anchor) != 1:
        raise SystemExit('expected PendingToolCall anchor')
    text = text.replace(struct_anchor, prepared_struct, 1)

session_api_start = text.index('    /// Runs one actor turn from an ordered structured submission.')
estimator_start = text.index('    fn estimate_provider_request_tokens(', session_api_start)
new_session_api = '''    /// Prepares the exact initial provider request for a structured session turn.
    ///
    /// The returned snapshot is the unique source for both context-budget
    /// preflight and the first provider call. Dynamic memory/todo sections and
    /// prompt hooks are therefore not evaluated again during execution.
    pub(crate) async fn prepare_session_turn(
        &self,
        items: &[talos_core::session::SubmissionItem],
        history: Vec<Message>,
    ) -> AgentResult<PreparedSessionTurn> {
        let memory_query = items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\\n");
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
                        parts.push(talos_core::message::ContentPart::Text {
                            text: item.text.clone(),
                        });
                    }
                    parts.extend(item.attachments.clone());
                    Message::Multimodal { parts }
                }
            })
            .collect();
        let hook_ctx = HookContext::new(TurnId::new(), self.workspace_root.clone());
        self.prepare_turn_with_messages(memory_query, input_messages, history, hook_ctx)
            .await
    }

    pub(crate) fn prepared_session_request_tokens(
        &self,
        prepared: &PreparedSessionTurn,
    ) -> u32 {
        Self::estimate_provider_request_tokens(&prepared.messages, &prepared.tool_definitions)
    }

    pub(crate) fn prepared_session_fixed_tokens(
        &self,
        prepared: &PreparedSessionTurn,
    ) -> u32 {
        Self::estimate_provider_request_tokens(
            &prepared.messages[prepared.history_len..],
            &prepared.tool_definitions,
        )
    }

    pub(crate) fn replace_prepared_session_history(
        prepared: &mut PreparedSessionTurn,
        history: Vec<Message>,
    ) {
        let non_history_prefix_len = prepared.persist_start - prepared.history_len;
        let suffix = prepared.messages.split_off(prepared.history_len);
        prepared.messages = history;
        prepared.history_len = prepared.messages.len();
        prepared.messages.extend(suffix);
        prepared.persist_start = prepared.history_len + non_history_prefix_len;
    }

    pub(crate) async fn run_prepared_session_turn(
        &self,
        prepared: PreparedSessionTurn,
        event_tx: mpsc::UnboundedSender<AgentEvent>,
        request_context_limit: u32,
    ) -> (AgentResult<String>, Vec<Message>) {
        self.run_inner_prepared(prepared, Some(event_tx), Some(request_context_limit))
            .await
    }

    pub(crate) fn preview_prepared_session_turn(
        &self,
        prepared: &PreparedSessionTurn,
    ) -> Option<String> {
        self.provider.request_preview(&prepared.messages).map(|preview| {
            let snapshot =
                serde_json::to_string_pretty(&preview).unwrap_or_else(|_| preview.to_string());
            format!("Request preview (no API call made):\\n\\n```json\\n{snapshot}\\n```")
        })
    }

'''
text = text[:session_api_start] + new_session_api + text[estimator_start:]

func_start = text.index('    async fn run_inner_with_messages(')
body_start = text.index('        let turn_id = TurnId::new();', func_start)
loop_marker = text.index('        // Transient continuation parts collected', body_start)
new_prepared_entry = '''        let turn_id = TurnId::new();
        let hook_ctx = HookContext::new(turn_id, self.workspace_root.clone());
        let prepared = match self
            .prepare_turn_with_messages(
                memory_query,
                input_messages,
                history,
                hook_ctx.clone(),
            )
            .await
        {
            Ok(prepared) => prepared,
            Err(error) => {
                self.emit_turn_complete(&hook_ctx, TurnStatus::Denied).await;
                return (Err(error), Vec::new());
            }
        };
        self.run_inner_prepared(prepared, event_tx, request_context_limit)
            .await
    }

    async fn prepare_turn_with_messages(
        &self,
        memory_query: String,
        input_messages: Vec<Message>,
        history: Vec<Message>,
        hook_ctx: HookContext,
    ) -> AgentResult<PreparedSessionTurn> {
        let history_len = history.len();
        let (mut messages, persist_start) = self
            .build_provider_messages(memory_query, history, &hook_ctx)
            .await?;
        messages.pop();
        messages.extend(input_messages);

        let tool_presentation_policy = self.tool_presentation_policy.clone();
        let (_, mut tool_definitions, mut presented_tool_names) =
            describe_presented_tools(&self.tools, &tool_presentation_policy);
        if !self.image_input_supported {
            tool_definitions.retain(|definition| definition.name != "read_image");
            presented_tool_names.retain(|name| name != "read_image");
        }

        Ok(PreparedSessionTurn {
            messages,
            persist_start,
            history_len,
            hook_ctx,
            tool_presentation_policy,
            tool_definitions,
            presented_tool_names,
        })
    }

    async fn run_inner_prepared(
        &self,
        prepared: PreparedSessionTurn,
        event_tx: Option<mpsc::UnboundedSender<AgentEvent>>,
        request_context_limit: Option<u32>,
    ) -> (AgentResult<String>, Vec<Message>) {
        let PreparedSessionTurn {
            mut messages,
            persist_start,
            history_len: _,
            hook_ctx,
            tool_presentation_policy: mut active_tool_presentation_policy,
            tool_definitions: mut active_tool_definitions,
            presented_tool_names: mut active_presented_tool_names,
        } = prepared;
        let turn_id = hook_ctx.turn_id;
        let mut total_tool_calls: usize = 0;
        let mut doom_tracker: HashMap<(String, String), u32> = HashMap::new();

'''
text = text[:body_start] + new_prepared_entry + text[loop_marker:]
path.write_text(text)

# Session actor: build once, compact by replacing only the prepared history prefix,
# then pass the exact snapshot to preview or execution.
path = Path('crates/talos-agent/src/session.rs')
text = path.read_text()
preflight_start = text.index('        if self.compactor.should_compact(&self.history) {', text.index('    async fn start_submission('))
turn_id_marker = text.index('        let turn_id = format!("{}_{}", self.turn_prefix, turn_counter);', preflight_start)
new_preflight = '''        let submission_kind = submission.common_kind();
        if submission_kind != Some(SubmissionKind::PreviewRequest)
            && let Some(agent_mut) = Arc::get_mut(&mut self.agent)
        {
            agent_mut.set_append_prompt_opt(None);
        }

        if self.compactor.should_compact(&self.history) {
            let compacted = self.compactor.apply_budget(self.history.clone());
            let compacted = self.compactor.apply_trim(compacted);
            let compacted = self.compactor.apply_microcompact(compacted);
            self.history = match self
                .compactor
                .compact(compacted, self.agent.provider())
                .await
            {
                Ok(history) => history,
                Err(_) => self.compactor.compact_deterministic(self.history.clone()).0,
            };
            if let (Some(file), Some(dir)) = (&self.session_file, &self.session_dir) {
                let _ = self.try_archive_session(file, dir, &self.history);
            }
        }

        let mut prepared = match self
            .agent
            .prepare_session_turn(&submission.items, self.history.clone())
            .await
        {
            Ok(prepared) => prepared,
            Err(_) => {
                self.reject_submission(
                    &submission.id,
                    submission.sender_generation,
                    SubmissionRejectionReason::ContextBudgetExceeded,
                );
                return None;
            }
        };
        let mut request_tokens = self.agent.prepared_session_request_tokens(&prepared);
        if request_tokens > self.model_context_limit {
            let fixed_tokens = self.agent.prepared_session_fixed_tokens(&prepared);
            let history_budget = self.model_context_limit.saturating_sub(fixed_tokens);
            let mut projected_compactor = Compactor::new(TokenEstimator::new(), history_budget);
            self.history = match projected_compactor
                .compact(self.history.clone(), self.agent.provider())
                .await
            {
                Ok(history) => history,
                Err(_) => projected_compactor
                    .compact_deterministic(self.history.clone())
                    .0,
            };
            Agent::replace_prepared_session_history(&mut prepared, self.history.clone());
            request_tokens = self.agent.prepared_session_request_tokens(&prepared);
            if let (Some(file), Some(dir)) = (&self.session_file, &self.session_dir) {
                let _ = self.try_archive_session(file, dir, &self.history);
            }
        }
        if request_tokens > self.model_context_limit {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::ContextBudgetExceeded,
            );
            return None;
        }

'''
text = text[:preflight_start] + new_preflight + text[turn_id_marker:]

text = text.replace(
    '        if submission.common_kind() == Some(SubmissionKind::PreviewRequest) {\n',
    '        if submission_kind == Some(SubmissionKind::PreviewRequest) {\n',
    1,
)
old_preview_setup = '''            let history = self.history.clone();
            let session_id = self.session_id.clone();
            let message = submission.items[0].text.clone();
            let token = CancellationToken::new();
'''
new_preview_setup = '''            let session_id = self.session_id.clone();
            let token = CancellationToken::new();
'''
if text.count(old_preview_setup) != 1:
    raise SystemExit('expected preview setup block')
text = text.replace(old_preview_setup, new_preview_setup, 1)
text = text.replace(
    '                    result = agent.preview_request(message, history) => result,\n',
    '                    result = async { Ok(agent.preview_prepared_session_turn(&prepared)) } => result,\n',
    1,
)

old_clear = '''        if let Some(agent_mut) = Arc::get_mut(&mut self.agent) {
            agent_mut.set_append_prompt_opt(None);
        }

'''
if old_clear in text:
    text = text.replace(old_clear, '', 1)

old_normal_setup = '''        let history = self.history.clone();
        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
        let items = submission.items;
'''
new_normal_setup = '''        let persistence = self.persistence.clone();
        let durable_persistence = self.durable_persistence.clone();
        let session_id = self.session_id.clone();
'''
if text.count(old_normal_setup) != 1:
    raise SystemExit('expected normal turn setup block')
text = text.replace(old_normal_setup, new_normal_setup, 1)
text = text.replace(
    '''                agent,
                items,
                history,
                event_tx,
''',
    '''                agent,
                prepared,
                event_tx,
''',
    1,
)
path.write_text(text)

# Forwarding seam consumes the frozen snapshot directly.
path = Path('crates/talos-agent/src/session/turn.rs')
text = path.read_text()
text = text.replace('use crate::Agent;\n', 'use crate::{Agent, PreparedSessionTurn};\n', 1)
text = text.replace(
    '''    pub(super) agent: Arc<Agent>,
    pub(super) items: Vec<SubmissionItem>,
    pub(super) history: Vec<Message>,
''',
    '''    pub(super) agent: Arc<Agent>,
    pub(super) prepared: PreparedSessionTurn,
''',
    1,
)
text = text.replace(
    '''        agent,
        items,
        history,
        event_tx,
''',
    '''        agent,
        prepared,
        event_tx,
''',
    1,
)
text = text.replace(
    '''        agent
            .run_for_session_turn_items(items, history, event_tx, request_context_limit)
            .await
''',
    '''        agent
            .run_prepared_session_turn(prepared, event_tx, request_context_limit)
            .await
''',
    1,
)
text = text.replace(
    'use talos_core::session::{SessionEvent, SubmissionItem, TurnCompletionStatus, TurnEventPayload};\n',
    'use talos_core::session::{SessionEvent, TurnCompletionStatus, TurnEventPayload};\n',
    1,
)
path.write_text(text)

# End-to-end regression: dynamic callbacks execute once and their exact first
# snapshot is the one observed by the provider.
path = Path('crates/talos-agent/tests/pr68_lifecycle.rs')
text = path.read_text()
if 'session_preflight_reuses_dynamic_prompt_snapshot' not in text:
    addition = r'''

struct CapturingModel {
    requests: Arc<Mutex<Vec<Vec<Message>>>>,
}

#[async_trait]
impl LanguageModel for CapturingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.requests
            .lock()
            .expect("capturing model lock")
            .push(messages.to_vec());
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let _ = tx.send(AgentEvent::TurnStart).await;
            let _ = tx
                .send(AgentEvent::TextDelta {
                    delta: "snapshot reused".into(),
                })
                .await;
            let _ = tx
                .send(AgentEvent::TurnEnd {
                    stop_reason: StopReason::EndTurn,
                    usage: Usage::default(),
                })
                .await;
        });
        Ok(rx)
    }
}

#[tokio::test]
async fn session_preflight_reuses_dynamic_prompt_snapshot() {
    let memory_calls = Arc::new(AtomicUsize::new(0));
    let todo_calls = Arc::new(AtomicUsize::new(0));
    let requests = Arc::new(Mutex::new(Vec::<Vec<Message>>::new()));

    #[allow(deprecated)]
    let mut agent = Agent::new(
        Arc::new(CapturingModel {
            requests: requests.clone(),
        }),
        ToolRegistry::new(),
    );
    let memory_counter = memory_calls.clone();
    agent.set_memory_provider(Arc::new(move |_| {
        let call = memory_counter.fetch_add(1, Ordering::SeqCst) + 1;
        Some(format!("memory-snapshot-{call}"))
    }));
    let todo_counter = todo_calls.clone();
    agent.set_todo_section_provider(Arc::new(move || {
        let call = todo_counter.fetch_add(1, Ordering::SeqCst) + 1;
        Some(format!("todo-snapshot-{call}"))
    }));

    let (handle, mut actor) = AppServerSession::new(agent, config(128_000));
    let sq_tx = handle.sq_tx;
    let mut eq_rx = handle.eq_rx;
    let actor_task = tokio::spawn(async move { actor.run().await });

    sq_tx
        .send(SessionOp::SubmitStructured {
            submission: submission(
                "prepared_snapshot_batch",
                "prepared_snapshot_item",
                SubmissionSource::User,
            ),
        })
        .await
        .unwrap();

    loop {
        let event = tokio::time::timeout(Duration::from_secs(2), eq_rx.recv())
            .await
            .expect("prepared snapshot completion timeout")
            .expect("session event channel");
        if matches!(
            event,
            SessionEvent::TurnEvent {
                payload: talos_core::session::TurnEventPayload::Completed { .. },
                ..
            }
        ) {
            break;
        }
    }

    assert_eq!(memory_calls.load(Ordering::SeqCst), 1);
    assert_eq!(todo_calls.load(Ordering::SeqCst), 1);
    let captured = requests.lock().expect("captured requests lock");
    assert_eq!(captured.len(), 1, "one provider request expected");
    let rendered = format!("{:?}", captured[0]);
    assert!(rendered.contains("memory-snapshot-1"));
    assert!(rendered.contains("todo-snapshot-1"));
    assert!(!rendered.contains("snapshot-2"));

    sq_tx.send(SessionOp::Shutdown).await.unwrap();
    actor_task.await.unwrap();
}
'''
    text += addition
path.write_text(text)
