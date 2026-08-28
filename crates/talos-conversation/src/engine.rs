use std::path::PathBuf;
use std::time::Instant;

use uuid::Uuid;

use talos_core::message::{AgentEvent, ContentPart, MessageToolResult, StopReason, Usage};
use talos_core::session::{
    MAX_STEERING_QUEUE_BYTES, MAX_STEERING_QUEUE_ITEMS, MAX_SUBMISSION_BATCH_BYTES,
    MAX_SUBMISSION_BATCH_ITEMS, MAX_SUBMISSION_ITEM_BYTES, StructuredSubmission, SubmissionItem,
    SubmissionKind, SubmissionSource, TurnCompletionStatus,
};
use talos_core::tool::ToolProvenance;

use crate::command_registry::{MOCK_REQUEST_COMMAND, command_registry};
use crate::types::{
    ChatMessage, ContentOutput, CopyScope, ExtensionSnapshot, HookDeclarationDiagnostic,
    HookSnapshot, LoadedPluginDiagnostic, McpServerDiagnostic, MessageRole, MessageSource,
    MessageStatus, ModelSwitchRequest, PluginObservation, ScrollbackState, SessionDeleteRequest,
    SessionForkRequest, SessionNewRequest, SessionResumeRequest, SkillCommandRequest,
    SkillDiagnostic, StatusSnapshot, SteeringQueueEntry, SteeringQueueSnapshot, TipKind,
    TodoCommandAction, TodoCommandRequest, TodoExportFormat, ToolCallDisplay, ToolCallInfo,
    ToolResultDisplay, TurnPhase, UiOutput,
};

mod commands;
mod projection;

pub use projection::{build_extension_snapshot, build_extension_snapshot_with_plugins};

fn next_steering_identity_namespace() -> String {
    Uuid::new_v4().to_string()
}

fn is_timeout_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("timeout") || lower.contains("timed out")
}

fn content_block(source: MessageSource, text: String) -> UiOutput {
    UiOutput::Content(ContentOutput::Block { source, text })
}

fn plugin_observation_key(provenance: &ToolProvenance) -> String {
    match provenance {
        ToolProvenance::Native => "native".to_string(),
        ToolProvenance::McpRemote { server } => {
            let server = if server.chars().count() > 24 {
                let truncated: String = server.chars().take(23).collect();
                format!("{truncated}…")
            } else {
                server.clone()
            };
            format!("mcp:{server}")
        }
        ToolProvenance::Plugin {
            name,
            version,
            carrier,
        } => {
            let name_display = if name.chars().count() > 24 {
                let truncated: String = name.chars().take(23).collect();
                format!("{truncated}…")
            } else {
                name.clone()
            };
            format!("plugin:{name_display}@{version}/{carrier}")
        }
    }
}

pub struct ConversationEngine {
    pub(crate) messages: Vec<ChatMessage>,
    /// Tool calls awaiting a result, keyed by provider tool-use identity.
    /// Keeping this side table private preserves the public transcript shape
    /// while allowing interleaved provider results to pair deterministically.
    pending_tool_calls: Vec<(String, usize)>,
    pub(crate) current_turn_text: String,
    pub(crate) steering_queue: Vec<SubmissionItem>,
    prepared_steering: Option<(String, Vec<String>)>,
    steering_identity_namespace: String,
    next_steering_sequence: u64,
    pub(crate) followup_queue: Vec<String>,
    pub(crate) usage: Usage,
    pub(crate) current_thinking_text: String,
    pub(crate) model_name: String,
    pub(crate) provider_name: String,
    variant: Option<String>,
    pub(crate) branch_id: Option<String>,
    pub(crate) plugin_observations: Vec<PluginObservation>,
    pub(crate) loaded_plugins: Vec<LoadedPluginDiagnostic>,
    pub(crate) hook_declarations: Vec<(String, String, bool)>,
    pub(crate) mcp_servers: Vec<McpServerDiagnostic>,
    pub(crate) skills: Vec<SkillDiagnostic>,
    pub(crate) scrollback: ScrollbackState,
    pub(crate) is_processing: bool,
    auto_config_enabled: bool,
    auto_override: Option<bool>,
    pub(crate) current_phase: Option<TurnPhase>,
    pub(crate) context_limit: Option<u32>,
    pub(crate) input_price_per_million: Option<f64>,
    pub(crate) output_price_per_million: Option<f64>,
    pub(crate) workspace_root: Option<PathBuf>,
    /// Resolved image-input capability for the active model (ADR-050).
    /// `/attach` consults this to fail-closed before any file read.
    pub image_input_capability: talos_core::model::ImageInputCapability,
    last_flushed_message: usize,
    content_open: bool,
    pub pending_image_attachments: Vec<talos_core::message::ContentPart>,
}

impl ConversationEngine {
    pub fn new(model_name: String, provider_name: String) -> Self {
        Self {
            messages: Vec::new(),
            pending_tool_calls: Vec::new(),
            current_turn_text: String::new(),
            steering_queue: Vec::new(),
            prepared_steering: None,
            steering_identity_namespace: next_steering_identity_namespace(),
            next_steering_sequence: 0,
            followup_queue: Vec::new(),
            usage: Usage::default(),
            current_thinking_text: String::new(),
            model_name,
            provider_name,
            variant: None,
            branch_id: None,
            plugin_observations: Vec::new(),
            loaded_plugins: Vec::new(),
            hook_declarations: Vec::new(),
            mcp_servers: Vec::new(),
            skills: Vec::new(),
            scrollback: ScrollbackState::default(),
            is_processing: false,
            auto_config_enabled: true,
            auto_override: None,
            current_phase: None,
            context_limit: None,
            input_price_per_million: None,
            output_price_per_million: None,
            workspace_root: None,
            image_input_capability: talos_core::model::ImageInputCapability::default(),
            last_flushed_message: 0,
            content_open: false,
            pending_image_attachments: Vec::new(),
        }
    }

    pub fn with_workspace_root(mut self, workspace_root: PathBuf) -> Self {
        self.workspace_root = Some(workspace_root);
        self
    }

    /// Sets the persisted configuration default for bounded auto assistance.
    #[must_use]
    pub fn with_auto_enabled(mut self, enabled: bool) -> Self {
        self.auto_config_enabled = enabled;
        self
    }

    fn auto_enabled(&self) -> bool {
        self.auto_override.unwrap_or(self.auto_config_enabled)
    }

    /// Supplies the typed set of explicitly loaded plugin packages.
    #[must_use]
    pub fn with_loaded_plugins(mut self, plugins: Vec<LoadedPluginDiagnostic>) -> Self {
        self.loaded_plugins = plugins;
        self
    }

    pub fn with_skills(mut self, skills: Vec<SkillDiagnostic>) -> Self {
        self.skills = skills;
        self
    }

    pub fn set_skills(&mut self, skills: Vec<SkillDiagnostic>) {
        self.skills = skills;
    }

    pub fn with_mcp_servers(mut self, servers: Vec<McpServerDiagnostic>) -> Self {
        self.mcp_servers = servers;
        self
    }

    pub fn with_hook_declarations(mut self, hooks: Vec<(String, String, bool)>) -> Self {
        self.hook_declarations = hooks;
        self
    }

    pub fn set_hook_declarations(&mut self, hooks: Vec<(String, String, bool)>) {
        self.hook_declarations = hooks;
    }

    pub fn status_snapshot(&self) -> StatusSnapshot {
        StatusSnapshot {
            model_name: self.model_name.clone(),
            provider: self.provider_name.clone(),
            workspace_path: String::new(),
            usage: self.usage.clone(),
            branch_id: self.branch_id.clone(),
            steering_count: self.steering_queue.len(),
            followup_count: self.followup_queue.len(),
            is_processing: self.is_processing,
            phase: self.current_phase.clone(),
            context_limit: self.context_limit,
            input_price_per_million: self.input_price_per_million,
            output_price_per_million: self.output_price_per_million,
            variant: self.variant.clone(),
            attachment_count: self.pending_image_attachments.len(),
        }
    }

    pub fn set_model_info(&mut self, info: &crate::types::ModelInfo) {
        self.model_name = info.model_name.clone();
        self.provider_name = info.provider.clone();
        self.context_limit = info.context_limit;
        self.input_price_per_million = info.input_price_per_million;
        self.output_price_per_million = info.output_price_per_million;
        self.variant = info.variant.clone();
        self.image_input_capability = info.image_input_capability;
    }

    pub fn is_processing(&self) -> bool {
        self.is_processing
    }

    /// Applies the authoritative session-level start of a user turn.
    pub fn handle_turn_started(&mut self) -> Vec<UiOutput> {
        self.pending_tool_calls.clear();
        self.is_processing = true;
        self.current_phase = Some(TurnPhase::Connecting);
        vec![UiOutput::Status(self.status_snapshot())]
    }

    /// Applies the authoritative terminal status of the whole user turn.
    pub fn handle_turn_completed(&mut self, status: &TurnCompletionStatus) -> Vec<UiOutput> {
        self.pending_tool_calls.clear();
        match status {
            TurnCompletionStatus::Success { .. } => {
                let mut outputs = Vec::new();
                self.close_content(&mut outputs);
                if let Some(thinking_outputs) = self.finalize_thinking() {
                    outputs.extend(thinking_outputs);
                }
                self.finalize_turn();
                self.last_flushed_message = self.messages.len();
                self.is_processing = false;
                self.current_phase = None;
                outputs.push(UiOutput::SteeringQueueSnapshot(
                    self.steering_queue_snapshot(),
                ));
                outputs.push(UiOutput::Status(self.status_snapshot()));
                outputs
            }
            TurnCompletionStatus::Cancelled => {
                let mut outputs = Vec::new();
                self.close_content(&mut outputs);
                self.current_turn_text.clear();
                self.current_thinking_text.clear();
                self.is_processing = false;
                self.current_phase = Some(TurnPhase::Cancelled);
                outputs.push(UiOutput::ThinkingPreview { text: None });
                outputs.push(UiOutput::SteeringQueueSnapshot(
                    self.steering_queue_snapshot(),
                ));
                outputs.push(UiOutput::Status(self.status_snapshot()));
                outputs
            }
            TurnCompletionStatus::Error { message } => {
                self.handle_agent_event(&AgentEvent::Error {
                    message: message.clone(),
                })
            }
        }
    }

    pub fn start_user_message(&mut self, msg: &str) -> Vec<UiOutput> {
        self.is_processing = true;
        self.handle_user_message(msg)
    }

    pub fn enqueue_steering(&mut self, msg: String) -> Vec<UiOutput> {
        self.enqueue_structured_steering(msg, SubmissionKind::UserTurn, Vec::new())
            .1
    }

    /// Adds one fully classified steering item to the authoritative queue.
    ///
    /// Returns whether the item was accepted plus the ordered UI projection.
    /// Item, queue, and byte limits follow ADR-056 and never truncate input.
    pub fn enqueue_structured_steering(
        &mut self,
        text: String,
        kind: SubmissionKind,
        attachments: Vec<ContentPart>,
    ) -> (bool, Vec<UiOutput>) {
        let queued_bytes: usize = self
            .steering_queue
            .iter()
            .map(SubmissionItem::text_bytes)
            .sum();
        let rejected = if text.len() > MAX_SUBMISSION_ITEM_BYTES {
            Some(format!(
                "Queued input rejected: {} bytes exceeds the {} byte per-item limit.",
                text.len(),
                MAX_SUBMISSION_ITEM_BYTES
            ))
        } else if self.steering_queue.len() >= MAX_STEERING_QUEUE_ITEMS {
            Some(format!(
                "Queued input rejected: the steering queue is limited to {MAX_STEERING_QUEUE_ITEMS} items."
            ))
        } else if queued_bytes.saturating_add(text.len()) > MAX_STEERING_QUEUE_BYTES {
            Some(format!(
                "Queued input rejected: the steering queue is limited to {MAX_STEERING_QUEUE_BYTES} text bytes."
            ))
        } else if kind == SubmissionKind::PreviewRequest && !attachments.is_empty() {
            Some("Request preview cannot be queued with image attachments.".to_string())
        } else {
            None
        };

        if let Some(message) = rejected {
            return (
                false,
                vec![
                    content_block(MessageSource::Error, format!("[Error] {message}\n")),
                    UiOutput::SteeringQueueSnapshot(self.steering_queue_snapshot()),
                    UiOutput::Status(self.status_snapshot()),
                ],
            );
        }

        let enqueue_sequence = self.next_steering_sequence;
        self.next_steering_sequence = self.next_steering_sequence.saturating_add(1);
        self.steering_queue.push(SubmissionItem {
            id: format!(
                "steering:{}:item:{enqueue_sequence}",
                self.steering_identity_namespace
            ),
            enqueue_sequence,
            kind,
            text,
            attachments,
        });
        (
            true,
            vec![
                UiOutput::Tip {
                    text: "Message queued and will send after current turn.".into(),
                    kind: TipKind::QueueHint,
                },
                UiOutput::SteeringQueueSnapshot(self.steering_queue_snapshot()),
                UiOutput::Status(self.status_snapshot()),
            ],
        )
    }

    pub fn cancel_turn(&mut self) -> Vec<UiOutput> {
        let mut outputs = Vec::new();
        self.pending_tool_calls.clear();
        self.close_content(&mut outputs);
        self.is_processing = false;
        self.current_phase = Some(TurnPhase::Cancelled);
        self.current_turn_text.clear();
        let had_thinking = !self.current_thinking_text.is_empty();
        self.current_thinking_text.clear();
        if had_thinking {
            outputs.push(UiOutput::ThinkingPreview { text: None });
        }
        outputs.extend([
            UiOutput::Tip {
                text: "Turn cancellation requested.".into(),
                kind: TipKind::ExitHint,
            },
            UiOutput::SteeringQueueSnapshot(self.steering_queue_snapshot()),
            UiOutput::Status(self.status_snapshot()),
        ]);
        outputs
    }

    pub fn handle_agent_event(&mut self, event: &AgentEvent) -> Vec<UiOutput> {
        let mut outputs = Vec::new();

        match event {
            AgentEvent::TurnStart => {
                self.pending_tool_calls.clear();
                self.is_processing = true;
                if !matches!(self.current_phase, Some(TurnPhase::Reconnecting { .. })) {
                    self.current_phase = Some(TurnPhase::Connecting);
                }
                self.current_turn_text.clear();
                self.current_thinking_text.clear();
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ProviderProgress { progress } => {
                use talos_core::provider::ProviderProgress;

                let phase = match progress {
                    ProviderProgress::InitialDispatch { .. } => TurnPhase::Connecting,
                    ProviderProgress::RetryDispatch {
                        attempt,
                        max_attempts,
                    }
                    | ProviderProgress::ScheduledBackoff {
                        attempt,
                        max_attempts,
                        ..
                    }
                    | ProviderProgress::FirstPacketWait {
                        attempt,
                        max_attempts,
                    } if *attempt > 0 => TurnPhase::Reconnecting {
                        attempt: *attempt,
                        max_attempts: *max_attempts,
                    },
                    ProviderProgress::FirstPacketWait { .. } => TurnPhase::Connecting,
                    _ => return outputs,
                };
                self.current_phase = Some(phase);
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::TextDelta { delta } => {
                if !self.current_thinking_text.is_empty()
                    && let Some(thinking_outputs) = self.finalize_thinking()
                {
                    outputs.extend(thinking_outputs);
                }
                self.current_phase = Some(TurnPhase::Generating);
                self.current_turn_text.push_str(delta);
                if !delta.is_empty() {
                    if !self.content_open {
                        self.content_open = true;
                        outputs.push(UiOutput::Content(ContentOutput::Start {
                            source: MessageSource::Assistant,
                        }));
                    }
                    outputs.push(UiOutput::Content(ContentOutput::Delta {
                        text: delta.clone(),
                    }));
                }
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ThinkingDelta { delta } => {
                self.current_phase = Some(TurnPhase::Thinking);
                self.current_thinking_text.push_str(delta);
                outputs.push(UiOutput::ThinkingPreview {
                    text: Some(self.current_thinking_text.clone()),
                });
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolCallStarted { name } => {
                if !self.current_thinking_text.is_empty()
                    && let Some(thinking_outputs) = self.finalize_thinking()
                {
                    outputs.extend(thinking_outputs);
                }
                self.current_phase = Some(TurnPhase::RunningTool { name: name.clone() });
                self.close_content(&mut outputs);
                outputs.push(UiOutput::ToolCallStarted {
                    name: name.to_string(),
                });
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolCall {
                call,
                provenance,
                summary_fields,
            } => {
                self.current_phase = Some(TurnPhase::RunningTool {
                    name: call.name.clone(),
                });
                self.close_content(&mut outputs);
                self.record_provenance(provenance);
                let message_index = self.messages.len();
                self.messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    status: MessageStatus::Completed,
                    content: String::new(),
                    tool_call: Some(ToolCallInfo {
                        tool_name: call.name.clone(),
                        arguments: serde_json::to_string_pretty(&call.input)
                            .unwrap_or_else(|_| call.input.to_string()),
                        provenance: provenance.clone(),
                        result: None,
                    }),
                    created_at: Instant::now(),
                });
                self.pending_tool_calls
                    .push((call.id.clone(), message_index));
                outputs.push(UiOutput::ToolCall(ToolCallDisplay {
                    tool_name: call.name.clone(),
                    arguments: call.input.clone(),
                    provenance: provenance.clone(),
                    summary_fields: summary_fields.clone(),
                }));
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ToolResult { result } => {
                self.close_content(&mut outputs);
                let tool_name = self.set_tool_result(result);
                outputs.push(UiOutput::ToolResult(ToolResultDisplay {
                    tool_name,
                    is_error: result.is_error,
                    content: result.content.clone(),
                }));
                self.current_phase = Some(TurnPhase::Generating);
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::TurnEnd { stop_reason, usage } => {
                self.close_content(&mut outputs);
                self.current_phase = None;
                if let Some(thinking_outputs) = self.finalize_thinking() {
                    outputs.extend(thinking_outputs);
                }
                self.finalize_turn();
                self.usage = usage.clone();
                self.last_flushed_message = self.messages.len();
                if !matches!(stop_reason, StopReason::ToolUse) {
                    self.pending_tool_calls.clear();
                }
                if matches!(stop_reason, StopReason::MaxTokens) {
                    outputs.push(UiOutput::Tip {
                        text: "Response truncated: provider reached the output token limit. Partial response preserved.".into(),
                        kind: TipKind::Error,
                    });
                }
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::Error { message } => {
                self.close_content(&mut outputs);
                self.pending_tool_calls.clear();
                self.is_processing = false;
                self.current_phase = Some(if is_timeout_error(message) {
                    TurnPhase::TimedOut
                } else {
                    TurnPhase::Failed
                });
                self.current_turn_text.clear();
                let had_thinking = !self.current_thinking_text.is_empty();
                self.current_thinking_text.clear();
                if had_thinking {
                    outputs.push(UiOutput::ThinkingPreview { text: None });
                }
                outputs.push(UiOutput::Tip {
                    text: message.clone(),
                    kind: TipKind::Error,
                });

                let text = format!("[Error] {message}");
                outputs.push(UiOutput::Content(ContentOutput::Block {
                    source: MessageSource::Error,
                    text,
                }));

                self.messages.push(ChatMessage {
                    role: MessageRole::Error,
                    status: MessageStatus::Completed,
                    content: format!("[Error] {message}"),
                    tool_call: None,
                    created_at: Instant::now(),
                });
                self.last_flushed_message = self.messages.len();

                outputs.push(UiOutput::SteeringQueueSnapshot(
                    self.steering_queue_snapshot(),
                ));
                outputs.push(UiOutput::Status(self.status_snapshot()));
            }
            AgentEvent::ReasoningComplete { .. } => {}
            _ => {}
        }

        outputs
    }

    pub fn handle_user_message(&mut self, msg: &str) -> Vec<UiOutput> {
        let msg_owned = msg.to_string();

        if !self.pending_image_attachments.is_empty() {
            let mut display_parts = vec![msg_owned.clone()];
            for part in &self.pending_image_attachments {
                if let talos_core::message::ContentPart::Image {
                    path,
                    mime,
                    byte_count,
                    content_digest: _,
                } = part
                {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("(unknown)");
                    display_parts
                        .push(format!(" [Image: {filename} ({byte_count} bytes, {mime})]"));
                }
            }
            let combined = display_parts.join("\n");
            self.messages.push(ChatMessage {
                role: MessageRole::User,
                status: MessageStatus::Completed,
                content: format!("{combined}\n"),
                tool_call: None,
                created_at: Instant::now(),
            });
            self.last_flushed_message = self.messages.len();
            vec![UiOutput::Content(ContentOutput::Block {
                source: MessageSource::User,
                text: combined,
            })]
        } else {
            self.messages.push(ChatMessage {
                role: MessageRole::User,
                status: MessageStatus::Completed,
                content: format!("{msg_owned}\n"),
                tool_call: None,
                created_at: Instant::now(),
            });
            self.last_flushed_message = self.messages.len();

            vec![UiOutput::Content(ContentOutput::Block {
                source: MessageSource::User,
                text: msg_owned,
            })]
        }
    }

    /// Drains the oldest steering message while preserving FIFO order.
    ///
    /// This method remains available only for legacy single-item callers.
    /// Transactional TUI steering uses structured prepare/commit/rollback.
    pub fn drain_steering_queue(&mut self) -> Option<String> {
        if self.steering_queue.is_empty() {
            None
        } else {
            self.prepared_steering = None;
            Some(self.steering_queue.remove(0).text)
        }
    }

    /// Freezes the next compatible bounded steering prefix without deleting it.
    ///
    /// A preview request is always a single-item submission. Normal user-turn
    /// items batch until the first incompatible kind or ADR-056 item/byte bound.
    /// Repeated calls return `None` while another transfer is prepared.
    pub fn prepare_steering_submission(&mut self) -> Option<StructuredSubmission> {
        if self.prepared_steering.is_some() {
            return None;
        }
        let first = self.steering_queue.first()?;
        let kind = first.kind;
        let max_items = if kind == SubmissionKind::PreviewRequest {
            1
        } else {
            MAX_SUBMISSION_BATCH_ITEMS
        };
        let mut total_bytes = 0usize;
        let mut items = Vec::new();
        for item in &self.steering_queue {
            if item.kind != kind || items.len() >= max_items {
                break;
            }
            let next_bytes = total_bytes.saturating_add(item.text_bytes());
            if !items.is_empty() && next_bytes > MAX_SUBMISSION_BATCH_BYTES {
                break;
            }
            if next_bytes > MAX_SUBMISSION_BATCH_BYTES {
                return None;
            }
            total_bytes = next_bytes;
            items.push(item.clone());
        }
        if items.is_empty() {
            return None;
        }
        let batch_sequence = self.next_steering_sequence;
        self.next_steering_sequence = self.next_steering_sequence.saturating_add(1);
        let id = format!(
            "steering:{}:batch:{batch_sequence}",
            self.steering_identity_namespace
        );
        self.prepared_steering = Some((
            id.clone(),
            items.iter().map(|item| item.id.clone()).collect(),
        ));
        Some(StructuredSubmission {
            id,
            source: SubmissionSource::User,
            sender_generation: 0,
            items,
        })
    }

    /// Commits a matching actor-acknowledged preparation and removes its prefix.
    ///
    /// Returns `false` without mutation for stale or mismatched acknowledgements.
    pub fn commit_prepared_steering(&mut self, submission_id: &str) -> bool {
        let Some((prepared_id, item_ids)) = self.prepared_steering.as_ref() else {
            return false;
        };
        if prepared_id != submission_id
            || self.steering_queue.len() < item_ids.len()
            || !self
                .steering_queue
                .iter()
                .zip(item_ids)
                .all(|(item, id)| &item.id == id)
        {
            return false;
        }
        self.steering_queue.drain(..item_ids.len());
        self.prepared_steering = None;
        true
    }

    /// Releases a matching failed preparation while preserving every item.
    pub fn rollback_prepared_steering(&mut self, submission_id: &str) -> bool {
        if self
            .prepared_steering
            .as_ref()
            .is_some_and(|(prepared_id, _)| prepared_id == submission_id)
        {
            self.prepared_steering = None;
            true
        } else {
            false
        }
    }

    /// Returns whether a prepare/send/ack transfer is currently pending.
    #[must_use]
    pub fn has_prepared_steering(&self) -> bool {
        self.prepared_steering.is_some()
    }

    /// Returns whether any pre-actor steering items remain authoritative here.
    #[must_use]
    pub fn has_steering(&self) -> bool {
        !self.steering_queue.is_empty()
    }

    /// Bounded FIFO snapshot of the steering queue (ADR-049).
    /// First 8 entries, 4 KiB UTF-8 per entry, exact total/omitted counts.
    pub fn steering_queue_snapshot(&self) -> SteeringQueueSnapshot {
        const MAX_ENTRIES: usize = 8;
        const MAX_BYTES: usize = 4096;
        const ELLIPSIS: &str = "…";
        let total_count = self.steering_queue.len();
        let omitted_count = total_count.saturating_sub(MAX_ENTRIES);
        let entries = self
            .steering_queue
            .iter()
            .take(MAX_ENTRIES)
            .map(|item| {
                let msg = &item.text;
                if msg.len() > MAX_BYTES {
                    let budget = MAX_BYTES - ELLIPSIS.len();
                    let mut end = budget.min(msg.len());
                    while end > 0 && !msg.is_char_boundary(end) {
                        end -= 1;
                    }
                    let text = format!("{}{ELLIPSIS}", &msg[..end]);
                    debug_assert!(
                        text.len() <= MAX_BYTES,
                        "truncated entry must be <= {MAX_BYTES} bytes"
                    );
                    SteeringQueueEntry {
                        text,
                        truncated: true,
                    }
                } else {
                    SteeringQueueEntry {
                        text: msg.clone(),
                        truncated: false,
                    }
                }
            })
            .collect();
        SteeringQueueSnapshot {
            entries,
            total_count,
            omitted_count,
        }
    }

    pub fn last_assistant_text(&self) -> Option<String> {
        self.messages.iter().rev().find_map(|msg| {
            if msg.role == MessageRole::Assistant
                && msg.tool_call.is_none()
                && !msg.content.is_empty()
            {
                Some(msg.content.clone())
            } else {
                None
            }
        })
    }

    fn close_content(&mut self, outputs: &mut Vec<UiOutput>) {
        if self.content_open {
            self.content_open = false;
            outputs.push(UiOutput::Content(ContentOutput::End));
        }
    }

    fn finalize_turn(&mut self) {
        self.current_thinking_text.clear();
        self.scrollback.scrolled_line_count = 0;
        if self.current_turn_text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.current_turn_text);
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            status: MessageStatus::Completed,
            content: text,
            tool_call: None,
            created_at: Instant::now(),
        });
    }

    fn finalize_thinking(&mut self) -> Option<Vec<UiOutput>> {
        if self.current_thinking_text.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut self.current_thinking_text);
        let display_text = format!("Thinking: {text}\n");

        self.messages.push(ChatMessage {
            role: MessageRole::Reasoning,
            status: MessageStatus::Completed,
            content: text,
            tool_call: None,
            created_at: Instant::now(),
        });

        Some(vec![
            UiOutput::ThinkingPreview { text: None },
            UiOutput::Content(ContentOutput::Block {
                source: MessageSource::Reasoning,
                text: display_text,
            }),
        ])
    }

    fn set_tool_result(&mut self, result: &MessageToolResult) -> Option<String> {
        let pending_index = self
            .pending_tool_calls
            .iter()
            .position(|(tool_use_id, _)| tool_use_id == &result.tool_use_id)?;
        let (_, message_index) = self.pending_tool_calls.remove(pending_index);
        let msg = self.messages.get_mut(message_index)?;
        let tool_call = msg.tool_call.as_mut()?;
        if tool_call.result.is_some() {
            return None;
        }
        let tool_name = tool_call.tool_name.clone();
        let is_background = matches!(tool_name.as_str(), "process")
            || matches!(tool_name.as_str(), "bash" | "exec")
                && serde_json::from_str::<serde_json::Value>(&tool_call.arguments)
                    .ok()
                    .and_then(|input| input.get("background").cloned())
                    .and_then(|value| value.as_bool())
                    == Some(true);
        tool_call.result = Some(result.clone());
        Some(if is_background {
            format!("background:{tool_name}")
        } else {
            tool_name
        })
    }

    fn record_provenance(&mut self, provenance: &ToolProvenance) {
        let key = plugin_observation_key(provenance);
        if let Some(entry) = self.plugin_observations.iter_mut().find(|e| e.key == key) {
            entry.count += 1;
        } else {
            self.plugin_observations
                .push(PluginObservation { key, count: 1 });
        }
    }
}
